//! Input collection, the player's action, the enemy phase, and post-turn checks.
//!
//! The order is explicit in `GamePlugin`:
//! input -> player action -> enemy phase -> finish turn.

use std::collections::HashSet;

use bevy::prelude::*;

use crate::balance;
use crate::grid::{Direction, DungeonMap, EXIT_POSITION, GridPosition};

use super::model::{
    Attack, BossPhase, Enemy, EnemyKind, Exit, Feedback, GameState, Health, PendingAction, Player,
    PlayerAction, PlayerProgress, Recovery, TurnCount, TurnFlow,
};
use super::progress::grant_kill_rewards;

/// Movement keys and their directions. Order breaks ties when several keys are
/// freshly pressed in the same frame.
const MOVEMENT_KEYS: [(KeyCode, Direction); 8] = [
    (KeyCode::KeyW, Direction::North),
    (KeyCode::ArrowUp, Direction::North),
    (KeyCode::KeyS, Direction::South),
    (KeyCode::ArrowDown, Direction::South),
    (KeyCode::KeyA, Direction::West),
    (KeyCode::ArrowLeft, Direction::West),
    (KeyCode::KeyD, Direction::East),
    (KeyCode::ArrowRight, Direction::East),
];

type PlayerActionQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut GridPosition,
        &'static mut Health,
        &'static mut PlayerProgress,
        &'static Attack,
        &'static Recovery,
    ),
    (With<Player>, Without<Enemy>),
>;

type EnemyBumpQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static GridPosition,
        &'static mut Health,
        &'static Enemy,
    ),
    (With<Enemy>, Without<Player>),
>;

type PlayerPhaseQuery<'w, 's> =
    Query<'w, 's, (&'static mut GridPosition, &'static mut Health), (With<Player>, Without<Enemy>)>;

type EnemyPhaseQuery<'w, 's> = Query<
    'w,
    's,
    (
        Entity,
        &'static Enemy,
        &'static mut GridPosition,
        &'static Health,
        &'static Attack,
        Option<&'static mut BossPhase>,
    ),
    (With<Enemy>, Without<Player>),
>;

type PlayerStatusQuery<'w, 's> =
    Query<'w, 's, (&'static Health, &'static PlayerProgress), (With<Player>, Without<Enemy>)>;

/// One fresh key press becomes at most one action. Space waits; movement keys
/// request a cardinal step. Diagonal input is impossible by construction.
pub(super) fn collect_player_action(
    keys: Res<ButtonInput<KeyCode>>,
    mut pending: ResMut<PendingAction>,
) {
    if pending.0.is_some() {
        return;
    }

    if keys.just_pressed(KeyCode::Space) {
        pending.0 = Some(PlayerAction::Wait);
        return;
    }

    for (key, direction) in MOVEMENT_KEYS {
        if keys.just_pressed(key) {
            pending.0 = Some(PlayerAction::Move(direction));
            return;
        }
    }
}

fn consume_turn(turn: &mut TurnCount, flow: &mut TurnFlow) {
    turn.0 += 1;
    flow.enemy_phase_due = true;
    flow.just_finished_turn = false;
}

/// Resolves the collected action. Walls, map bounds, and a locked exit reject
/// the action without advancing a turn. A living enemy on the target tile is
/// attacked instead of being overlapped.
#[allow(clippy::too_many_arguments)]
pub(super) fn resolve_player_action(
    mut commands: Commands,
    mut pending: ResMut<PendingAction>,
    map: Res<DungeonMap>,
    mut feedback: ResMut<Feedback>,
    mut turn: ResMut<TurnCount>,
    mut flow: ResMut<TurnFlow>,
    mut next_state: ResMut<NextState<GameState>>,
    mut players: PlayerActionQuery,
    mut enemies: EnemyBumpQuery,
    mut exits: Query<&mut Exit>,
) {
    let Some(action) = pending.0.take() else {
        return;
    };
    feedback.clear();

    let Ok((mut position, mut player_health, mut progress, attack, recovery)) =
        players.single_mut()
    else {
        return;
    };

    let target = match action {
        PlayerAction::Wait => {
            feedback.say("You wait and listen.");
            consume_turn(&mut turn, &mut flow);
            return;
        }
        PlayerAction::Move(direction) => position.offset(direction.delta()),
    };

    if !map.is_walkable(target) {
        feedback.say("Blocked by a wall.");
        return;
    }

    if target == EXIT_POSITION {
        let locked = exits.iter().next().map(|exit| exit.locked).unwrap_or(true);
        if locked {
            feedback.say("The exit is sealed. Defeat the Warden first.");
            return;
        }

        feedback.say("You step through the open door and escape the dungeon!");
        flow.enemy_phase_due = false;
        flow.just_finished_turn = false;
        next_state.set(GameState::Won);
        return;
    }

    let target_enemy = enemies
        .iter()
        .find(|(_, enemy_position, enemy_health, _)| {
            **enemy_position == target && enemy_health.current > 0
        })
        .map(|(entity, _, _, enemy)| (entity, enemy.kind));

    if let Some((entity, kind)) = target_enemy {
        let Ok((_, _, mut enemy_health, enemy)) = enemies.get_mut(entity) else {
            return;
        };
        let damage = attack.damage;
        let killed = enemy_health.take_damage(damage);

        if killed {
            let reward = grant_kill_rewards(kind, &mut progress, &mut player_health, recovery);
            if reward.xp > 0 {
                feedback.say(format!("{} defeated! +{} XP.", kind.name(), reward.xp));
            } else {
                feedback.say(format!("{} defeated!", kind.name()));
            }
            if reward.healed > 0 {
                feedback.say(format!("Recovered {} HP.", reward.healed));
            }
            if reward.levels_gained > 0 {
                feedback.say("LEVEL UP! Choose an upgrade after this turn.");
            }
            if enemy.kind == EnemyKind::Boss {
                for mut exit in &mut exits {
                    exit.locked = false;
                }
                feedback.say("The Warden falls. The exit seal breaks!");
            }
            commands.entity(entity).despawn();
        } else {
            feedback.say(format!(
                "You hit {} for {} ({} HP left).",
                kind.name(),
                damage,
                enemy_health.current
            ));
        }

        consume_turn(&mut turn, &mut flow);
        return;
    }

    *position = target;
    consume_turn(&mut turn, &mut flow);
}

/// Resolves every living enemy once, in ascending `Enemy::order`.
///
/// Brutes act only on even turn numbers. Boss behavior is handled separately so
/// its telegraph/recovery cycle is easy to read. The loop stops immediately
/// when the player dies, so no later enemy acts in that turn.
pub(super) fn resolve_enemy_phase(
    map: Res<DungeonMap>,
    turn: Res<TurnCount>,
    mut feedback: ResMut<Feedback>,
    mut flow: ResMut<TurnFlow>,
    mut players: PlayerPhaseQuery,
    mut enemies: EnemyPhaseQuery,
) {
    if !flow.enemy_phase_due {
        return;
    }
    flow.enemy_phase_due = false;
    flow.just_finished_turn = true;

    let Ok((player_position, mut player_health)) = players.single_mut() else {
        return;
    };
    if player_health.is_dead() {
        return;
    }

    let mut occupied: HashSet<GridPosition> = HashSet::new();
    occupied.insert(*player_position);
    // Enemies may never stand on the exit, even after it unlocks.
    occupied.insert(EXIT_POSITION);
    for (_, _, position, health, _, _) in enemies.iter() {
        if health.current > 0 {
            occupied.insert(*position);
        }
    }

    let mut order: Vec<(u32, Entity)> = enemies
        .iter()
        .filter(|(_, _, _, health, _, _)| health.current > 0)
        .map(|(entity, enemy, _, _, _, _)| (enemy.order, entity))
        .collect();
    order.sort_by_key(|(order, _)| *order);

    for (_, entity) in order {
        if player_health.is_dead() {
            break;
        }

        let Ok((_, enemy, mut position, health, attack, boss_phase)) = enemies.get_mut(entity)
        else {
            continue;
        };
        if health.is_dead() {
            // A player attack may have killed this enemy earlier in the frame.
            continue;
        }

        match enemy.kind {
            EnemyKind::Boss => {
                let Some(mut phase) = boss_phase else {
                    continue;
                };
                boss_turn(
                    &mut phase,
                    *position,
                    *player_position,
                    &mut player_health,
                    &mut feedback,
                );
            }
            EnemyKind::Pursuer | EnemyKind::Brute => {
                if enemy.kind == EnemyKind::Brute && !turn.0.is_multiple_of(2) {
                    // The brute acts on every second consumed player turn.
                    continue;
                }

                if is_adjacent(*position, *player_position) {
                    player_health.take_damage(attack.damage);
                    feedback.say(format!(
                        "{} hits you for {}.",
                        enemy.kind.name(),
                        attack.damage
                    ));
                } else if let Some(step) =
                    next_step_toward(&map, &occupied, *position, *player_position)
                {
                    occupied.remove(&*position);
                    *position = step;
                    occupied.insert(step);
                }
            }
        }
    }

    if player_health.is_dead() {
        feedback.say("You collapse. The dungeon claims another adventurer.");
    }
}

fn is_adjacent(a: GridPosition, b: GridPosition) -> bool {
    a.manhattan_distance(b) == 1
}

/// Picks one deterministic cardinal step toward `target`, preferring the axis
/// with the larger distance. Walls and occupied tiles are skipped. A blocked
/// enemy simply waits; no pathfinding search is needed for this small map.
pub fn next_step_toward(
    map: &DungeonMap,
    occupied: &HashSet<GridPosition>,
    from: GridPosition,
    target: GridPosition,
) -> Option<GridPosition> {
    let dx = target.x - from.x;
    let dy = target.y - from.y;
    let candidates = if dx.abs() >= dy.abs() {
        [(dx.signum(), 0), (0, dy.signum())]
    } else {
        [(0, dy.signum()), (dx.signum(), 0)]
    };

    candidates
        .into_iter()
        .filter(|(x, y)| *x != 0 || *y != 0)
        .map(|(x, y)| from.offset((x, y)))
        .find(|position| map.is_walkable(*position) && !occupied.contains(position))
}

fn boss_turn(
    phase: &mut BossPhase,
    boss_position: GridPosition,
    player_position: GridPosition,
    player_health: &mut Health,
    feedback: &mut Feedback,
) {
    match *phase {
        BossPhase::Ready => {
            if player_position.manhattan_distance(boss_position) <= balance::BOSS_THREAT_RANGE {
                *phase = BossPhase::WindingUp {
                    target: player_position,
                };
                feedback.say("THE WARDEN winds up! A heavy attack is marked on your tile.");
            }
        }
        BossPhase::WindingUp { target } => {
            if player_position == target {
                player_health.take_damage(balance::BOSS_HEAVY_DAMAGE);
                feedback.say(format!(
                    "The Warden's heavy blow hits you for {}!",
                    balance::BOSS_HEAVY_DAMAGE
                ));
            } else {
                feedback.say("The Warden's heavy blow crashes down on the empty marked tile.");
            }
            *phase = BossPhase::Recovering;
        }
        BossPhase::Recovering => {
            *phase = BossPhase::Ready;
            feedback.say("The Warden recovers and lowers its guard.");
        }
    }
}

/// Runs after the enemy phase. Death beats a pending level-up, so a run can
/// never stop at an upgrade panel after the player has died.
pub(super) fn finish_turn(
    mut flow: ResMut<TurnFlow>,
    players: PlayerStatusQuery,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !flow.just_finished_turn {
        return;
    }
    flow.just_finished_turn = false;

    let Ok((health, progress)) = players.single() else {
        return;
    };

    if health.is_dead() {
        next_state.set(GameState::Lost);
    } else if progress.pending_upgrades > 0 {
        next_state.set(GameState::ChoosingUpgrade);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_step_prefers_larger_axis() {
        let map = DungeonMap::from_rows(&[".....", ".....", ".....", "....."]);
        let occupied = HashSet::new();
        let from = GridPosition::new(0, 0);

        assert_eq!(
            next_step_toward(&map, &occupied, from, GridPosition::new(3, 1)),
            Some(GridPosition::new(1, 0))
        );
        assert_eq!(
            next_step_toward(&map, &occupied, from, GridPosition::new(1, 3)),
            Some(GridPosition::new(0, 1))
        );
    }

    #[test]
    fn blocked_primary_axis_falls_back_to_the_other() {
        let map = DungeonMap::from_rows(&["....", "....", ".#..", "...."]);
        let occupied = HashSet::new();
        let from = GridPosition::new(0, 1);

        // North/east primary x step is the wall at (1, 1); fall back north.
        assert_eq!(
            next_step_toward(&map, &occupied, from, GridPosition::new(3, 2)),
            Some(GridPosition::new(0, 2))
        );
    }

    #[test]
    fn step_never_targets_an_occupied_tile() {
        let map = DungeonMap::from_rows(&["...", "...", "..."]);
        let mut occupied = HashSet::new();
        occupied.insert(GridPosition::new(1, 1));

        assert_eq!(
            next_step_toward(
                &map,
                &occupied,
                GridPosition::new(0, 1),
                GridPosition::new(2, 1)
            ),
            None
        );
    }
}
