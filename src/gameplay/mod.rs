//! Gameplay plugin and the explicit turn/state system ordering.

mod model;
mod progress;
mod restart;
mod turn;

#[cfg(test)]
mod tests;

pub use model::*;
pub use progress::{handle_upgrade_selection, upgrade_effect_text, xp_threshold_for_next_level};
pub use restart::restart_run;

use bevy::prelude::*;

use crate::grid::DungeonMap;

/// Broad update order. Gameplay sets run only in `Playing` (except upgrade
/// selection and restart, which have their own state conditions); presentation
/// and UI always run so panels and transforms stay current in every state.
#[derive(SystemSet, Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum GameSet {
    Input,
    PlayerAction,
    EnemyPhase,
    PostTurn,
    Upgrade,
    Presentation,
    Ui,
}

pub struct GamePlugin;

impl Plugin for GamePlugin {
    fn build(&self, app: &mut App) {
        app.init_state::<GameState>()
            .insert_resource(DungeonMap::fixed())
            .init_resource::<Feedback>()
            .init_resource::<TurnCount>()
            .init_resource::<TurnFlow>()
            .init_resource::<PendingAction>()
            .init_resource::<UpgradeSelection>()
            .init_resource::<RestartRequested>()
            .configure_sets(
                Update,
                (
                    GameSet::Input,
                    GameSet::PlayerAction,
                    GameSet::EnemyPhase,
                    GameSet::PostTurn,
                    GameSet::Upgrade,
                    GameSet::Presentation,
                    GameSet::Ui,
                )
                    .chain(),
            )
            .configure_sets(Update, GameSet::Input.run_if(in_state(GameState::Playing)))
            .configure_sets(
                Update,
                GameSet::PlayerAction.run_if(in_state(GameState::Playing)),
            )
            .configure_sets(
                Update,
                GameSet::EnemyPhase.run_if(in_state(GameState::Playing)),
            )
            .configure_sets(
                Update,
                GameSet::PostTurn.run_if(in_state(GameState::Playing)),
            )
            .configure_sets(
                Update,
                GameSet::Upgrade.run_if(in_state(GameState::ChoosingUpgrade)),
            )
            .add_systems(
                Update,
                (
                    turn::collect_player_action.in_set(GameSet::Input),
                    turn::resolve_player_action.in_set(GameSet::PlayerAction),
                    turn::resolve_enemy_phase.in_set(GameSet::EnemyPhase),
                    turn::finish_turn.in_set(GameSet::PostTurn),
                )
                    .chain(),
            )
            .add_systems(Update, handle_upgrade_selection.in_set(GameSet::Upgrade))
            .add_systems(Update, restart_run.run_if(in_state(GameState::Won)))
            .add_systems(Update, restart_run.run_if(in_state(GameState::Lost)));
    }
}
