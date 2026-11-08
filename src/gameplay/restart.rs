//! Restart flow: tear down one run and let `OnEnter(Playing)` build a new one.

use bevy::prelude::*;

use super::model::{
    Feedback, GameState, PendingAction, RestartRequested, RunEntity, TurnCount, TurnFlow,
    UpgradeSelection,
};

/// Available only on the win/loss screens. R or the on-screen button both set
/// `RestartRequested`; this system performs the shared reset exactly once.
///
/// The camera and the persistent HUD are deliberately not `RunEntity`, so they
/// survive. Upgrade/end panels are state-scoped UI and are removed by their
/// `OnExit` systems.
#[allow(clippy::too_many_arguments)]
pub fn restart_run(
    mut commands: Commands,
    keys: Res<ButtonInput<KeyCode>>,
    mut requested: ResMut<RestartRequested>,
    run_entities: Query<Entity, With<RunEntity>>,
    mut feedback: ResMut<Feedback>,
    mut turn: ResMut<TurnCount>,
    mut flow: ResMut<TurnFlow>,
    mut pending_action: ResMut<PendingAction>,
    mut upgrade_selection: ResMut<UpgradeSelection>,
    mut next_state: ResMut<NextState<GameState>>,
) {
    if !requested.0 && !keys.just_pressed(KeyCode::KeyR) {
        return;
    }

    requested.0 = false;
    for entity in &run_entities {
        commands.entity(entity).despawn();
    }

    *feedback = Feedback::default();
    *turn = TurnCount::default();
    *flow = TurnFlow::default();
    *pending_action = PendingAction::default();
    *upgrade_selection = UpgradeSelection::default();
    next_state.set(GameState::Playing);
}
