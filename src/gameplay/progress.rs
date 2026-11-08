//! Damage rewards, XP thresholds, and the three upgrade effects.

use bevy::prelude::*;

use crate::balance;

use super::model::{
    Attack, Enemy, EnemyKind, Feedback, GameState, Health, Player, PlayerProgress, Recovery,
    UpgradeKind, UpgradeSelection,
};

/// Cumulative XP required to reach the next level, or `None` at max level.
pub fn xp_threshold_for_next_level(level: u32) -> Option<i32> {
    if level >= balance::MAX_LEVEL {
        return None;
    }
    let index = level.saturating_sub(1) as usize;
    balance::XP_THRESHOLDS.get(index).copied()
}

/// Adds XP and queues exactly one pending choice for every threshold crossed.
pub fn award_xp(progress: &mut PlayerProgress, amount: i32) -> u32 {
    if amount <= 0 {
        return 0;
    }

    progress.xp += amount;
    let mut levels_gained = 0;
    while let Some(threshold) = xp_threshold_for_next_level(progress.level) {
        if progress.xp < threshold {
            break;
        }
        progress.level += 1;
        progress.pending_upgrades += 1;
        levels_gained += 1;
    }
    levels_gained
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct KillReward {
    pub xp: i32,
    pub levels_gained: u32,
    pub healed: i32,
}

/// Applies the rewards for a killing blow. Call only once per defeated enemy.
pub fn grant_kill_rewards(
    kind: EnemyKind,
    progress: &mut PlayerProgress,
    health: &mut Health,
    recovery: &Recovery,
) -> KillReward {
    let xp = kind.xp();
    KillReward {
        xp,
        levels_gained: award_xp(progress, xp),
        healed: health.heal(recovery.heal_on_kill),
    }
}

pub fn apply_upgrade(
    kind: UpgradeKind,
    health: &mut Health,
    attack: &mut Attack,
    recovery: &mut Recovery,
) {
    match kind {
        UpgradeKind::Vitality => {
            health.max += balance::VITALITY_MAX_HP;
            health.current = (health.current + balance::VITALITY_HEAL).clamp(0, health.max);
        }
        UpgradeKind::Strength => {
            attack.damage += balance::STRENGTH_ATTACK;
        }
        UpgradeKind::Recovery => {
            recovery.heal_on_kill += balance::RECOVERY_HEAL;
        }
    }
}

/// Exact numerical text for an upgrade card.
pub fn upgrade_effect_text(
    kind: UpgradeKind,
    health: &Health,
    attack: &Attack,
    recovery: &Recovery,
) -> String {
    match kind {
        UpgradeKind::Vitality => format!(
            "Max HP {} -> {}\nHeal {} now",
            health.max,
            health.max + balance::VITALITY_MAX_HP,
            balance::VITALITY_HEAL
        ),
        UpgradeKind::Strength => format!(
            "Attack {} -> {}",
            attack.damage,
            attack.damage + balance::STRENGTH_ATTACK
        ),
        UpgradeKind::Recovery => format!(
            "Heal on kill {} -> {}",
            recovery.heal_on_kill,
            recovery.heal_on_kill + balance::RECOVERY_HEAL
        ),
    }
}

type PlayerUpgradeQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static mut Health,
        &'static mut Attack,
        &'static mut Recovery,
        &'static mut PlayerProgress,
    ),
    (With<Player>, Without<Enemy>),
>;

/// Reads a card click or keys 1/2/3, applies exactly one upgrade, and leaves
/// the upgrade state only when the queue is empty. This system runs only in
/// `GameState::ChoosingUpgrade`; choosing never consumes a turn.
pub fn handle_upgrade_selection(
    keys: Res<ButtonInput<KeyCode>>,
    mut selection: ResMut<UpgradeSelection>,
    mut players: PlayerUpgradeQuery,
    mut feedback: ResMut<Feedback>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    let chosen = selection.0.take().or_else(|| {
        if keys.just_pressed(KeyCode::Digit1) || keys.just_pressed(KeyCode::Numpad1) {
            Some(UpgradeKind::Vitality)
        } else if keys.just_pressed(KeyCode::Digit2) || keys.just_pressed(KeyCode::Numpad2) {
            Some(UpgradeKind::Strength)
        } else if keys.just_pressed(KeyCode::Digit3) || keys.just_pressed(KeyCode::Numpad3) {
            Some(UpgradeKind::Recovery)
        } else {
            None
        }
    });

    let Some(kind) = chosen else {
        return;
    };
    let Ok((mut health, mut attack, mut recovery, mut progress)) = players.single_mut() else {
        return;
    };

    if progress.pending_upgrades == 0 {
        next_state.set(GameState::Playing);
        return;
    }

    apply_upgrade(kind, &mut health, &mut attack, &mut recovery);
    progress.pending_upgrades -= 1;
    feedback.say(format!("Upgrade chosen: {}.", kind.title()));

    if progress.pending_upgrades == 0 {
        next_state.set(GameState::Playing);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn xp_thresholds_queue_one_choice_per_level() {
        let mut progress = PlayerProgress::default();

        assert_eq!(award_xp(&mut progress, 7), 0);
        assert_eq!(progress.level, 1);
        assert_eq!(progress.pending_upgrades, 0);

        // 7 + 15 = 22 crosses 8 and 18, but not 30.
        assert_eq!(award_xp(&mut progress, 15), 2);
        assert_eq!(progress.level, 3);
        assert_eq!(progress.xp, 22);
        assert_eq!(progress.pending_upgrades, 2);
    }

    #[test]
    fn xp_stops_queuing_choices_at_max_level() {
        let mut progress = PlayerProgress {
            level: 1,
            xp: 0,
            pending_upgrades: 0,
        };

        assert_eq!(award_xp(&mut progress, 1_000), balance::MAX_LEVEL - 1);
        assert_eq!(progress.level, balance::MAX_LEVEL);
        assert_eq!(progress.pending_upgrades, balance::MAX_LEVEL - 1);
        assert_eq!(award_xp(&mut progress, 1_000), 0);
    }

    #[test]
    fn damage_reports_only_the_killing_blow() {
        let mut health = Health::new(8);

        assert!(!health.take_damage(3));
        assert_eq!(health.current, 5);
        assert!(health.take_damage(5));
        assert_eq!(health.current, 0);
        assert!(
            !health.take_damage(1),
            "an already dead enemy cannot die again"
        );
    }

    #[test]
    fn healing_is_clamped_to_maximum() {
        let mut health = Health::new(10);
        health.current = 8;

        assert_eq!(health.heal(5), 2);
        assert_eq!(health.current, 10);
        assert_eq!(health.heal(2), 0);
    }

    #[test]
    fn vitality_raises_max_and_heals() {
        let mut health = Health::new(24);
        health.current = 20;
        let mut attack = Attack::new(4);
        let mut recovery = Recovery::new(2);

        apply_upgrade(
            UpgradeKind::Vitality,
            &mut health,
            &mut attack,
            &mut recovery,
        );

        assert_eq!(health.max, 28);
        assert_eq!(health.current, 24);
        assert_eq!(attack.damage, 4);
        assert_eq!(recovery.heal_on_kill, 2);
    }

    #[test]
    fn strength_raises_melee_damage() {
        let mut health = Health::new(24);
        let mut attack = Attack::new(4);
        let mut recovery = Recovery::new(2);

        apply_upgrade(
            UpgradeKind::Strength,
            &mut health,
            &mut attack,
            &mut recovery,
        );

        assert_eq!(attack.damage, 5);
        assert_eq!(health.max, 24);
        assert_eq!(recovery.heal_on_kill, 2);
    }

    #[test]
    fn recovery_raises_heal_on_kill() {
        let mut health = Health::new(24);
        let mut attack = Attack::new(4);
        let mut recovery = Recovery::new(2);

        apply_upgrade(
            UpgradeKind::Recovery,
            &mut health,
            &mut attack,
            &mut recovery,
        );

        assert_eq!(recovery.heal_on_kill, 3);
        assert_eq!(health.max, 24);
        assert_eq!(attack.damage, 4);
    }

    #[test]
    fn kill_reward_heals_and_can_cross_a_level_threshold() {
        let mut progress = PlayerProgress {
            level: 1,
            xp: 5,
            pending_upgrades: 0,
        };
        let mut health = Health::new(24);
        health.current = 20;

        let reward = grant_kill_rewards(
            EnemyKind::Pursuer,
            &mut progress,
            &mut health,
            &Recovery::new(2),
        );

        assert_eq!(reward.xp, 4);
        assert_eq!(reward.levels_gained, 1);
        assert_eq!(reward.healed, 2);
        assert_eq!(progress.xp, 9);
        assert_eq!(progress.level, 2);
        assert_eq!(progress.pending_upgrades, 1);
        assert_eq!(health.current, 22);
    }

    #[test]
    fn card_text_shows_current_and_next_numbers() {
        let health = Health::new(24);
        let attack = Attack::new(4);
        let recovery = Recovery::new(2);

        assert_eq!(
            upgrade_effect_text(UpgradeKind::Vitality, &health, &attack, &recovery),
            "Max HP 24 -> 28\nHeal 4 now"
        );
        assert_eq!(
            upgrade_effect_text(UpgradeKind::Strength, &health, &attack, &recovery),
            "Attack 4 -> 5"
        );
        assert_eq!(
            upgrade_effect_text(UpgradeKind::Recovery, &health, &attack, &recovery),
            "Heal on kill 2 -> 3"
        );
    }
}
