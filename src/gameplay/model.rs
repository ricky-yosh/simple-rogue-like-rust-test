//! Components, resources, and small value types shared by gameplay systems.

use bevy::prelude::*;

use crate::balance;
use crate::grid::{Direction, GridPosition};

/// Marker for the one player entity.
#[derive(Component, Debug)]
pub struct Player;

/// Marker for everything spawned as part of one run. Restart despawns these.
#[derive(Component, Debug)]
pub struct RunEntity;

/// Marker for the boss. There is exactly one per run.
#[derive(Component, Debug)]
pub struct Boss;

/// The two regular enemy types plus the boss.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum EnemyKind {
    Pursuer,
    Brute,
    Boss,
}

impl EnemyKind {
    pub const fn name(self) -> &'static str {
        match self {
            Self::Pursuer => "Skeleton",
            Self::Brute => "Brute",
            Self::Boss => "The Warden",
        }
    }

    pub const fn max_hp(self) -> i32 {
        match self {
            Self::Pursuer => balance::SKELETON_MAX_HP,
            Self::Brute => balance::BRUTE_MAX_HP,
            Self::Boss => balance::BOSS_MAX_HP,
        }
    }

    pub const fn attack(self) -> i32 {
        match self {
            Self::Pursuer => balance::SKELETON_ATTACK,
            Self::Brute => balance::BRUTE_ATTACK,
            Self::Boss => balance::BOSS_HEAVY_DAMAGE,
        }
    }

    pub const fn xp(self) -> i32 {
        match self {
            Self::Pursuer => balance::SKELETON_XP,
            Self::Brute => balance::BRUTE_XP,
            Self::Boss => balance::BOSS_XP,
        }
    }
}

/// An enemy's stable identity and turn order.
#[derive(Component, Debug)]
pub struct Enemy {
    pub kind: EnemyKind,
    /// Lower numbers act first. Assigned when the run is spawned.
    pub order: u32,
}

/// Current and maximum hit points.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Health {
    pub current: i32,
    pub max: i32,
}

impl Health {
    pub const fn new(max: i32) -> Self {
        Self { current: max, max }
    }

    pub fn is_dead(self) -> bool {
        self.current <= 0
    }

    /// Applies damage and returns `true` only on the blow that kills.
    pub fn take_damage(&mut self, damage: i32) -> bool {
        let was_alive = self.current > 0;
        self.current = (self.current - damage.max(0)).max(0);
        was_alive && self.current == 0
    }

    /// Heals without exceeding the maximum, returning the amount restored.
    pub fn heal(&mut self, amount: i32) -> i32 {
        let before = self.current;
        self.current = (self.current + amount.max(0)).min(self.max);
        self.current - before
    }

    pub fn fraction(self) -> f32 {
        if self.max <= 0 {
            return 0.0;
        }
        (self.current as f32 / self.max as f32).clamp(0.0, 1.0)
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Attack {
    pub damage: i32,
}

impl Attack {
    pub const fn new(damage: i32) -> Self {
        Self { damage }
    }
}

#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Recovery {
    pub heal_on_kill: i32,
}

impl Recovery {
    pub const fn new(heal_on_kill: i32) -> Self {
        Self { heal_on_kill }
    }
}

/// The player's level, accumulated XP, and queued level-up choices.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct PlayerProgress {
    pub level: u32,
    pub xp: i32,
    pub pending_upgrades: u32,
}

impl Default for PlayerProgress {
    fn default() -> Self {
        Self {
            level: 1,
            xp: 0,
            pending_upgrades: 0,
        }
    }
}

/// The exit door. It is blocked until the boss dies.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub struct Exit {
    pub locked: bool,
}

/// The boss's small, explicit turn cycle.
#[derive(Component, Clone, Copy, Debug, PartialEq, Eq)]
pub enum BossPhase {
    /// Waiting for the player to enter threat range.
    Ready,
    /// A heavy attack is marked on `target`; it resolves on the boss's next turn.
    WindingUp { target: GridPosition },
    /// The boss spends one turn recovering after a heavy attack.
    Recovering,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PlayerAction {
    Move(Direction),
    Wait,
}

/// A fresh action for this frame, produced by input and consumed by resolution.
#[derive(Resource, Default)]
pub struct PendingAction(pub Option<PlayerAction>);

/// Short-lived bookkeeping that makes the turn boundary explicit.
#[derive(Resource, Default)]
pub struct TurnFlow {
    /// The player action consumed a turn and enemies still need to act.
    pub enemy_phase_due: bool,
    /// Enemy actions have finished; post-turn state checks should run.
    pub just_finished_turn: bool,
}

/// Monotonic count of consumed player turns. Blocked actions and upgrade
/// choices do not increment it.
#[derive(Resource, Default, Debug)]
pub struct TurnCount(pub u32);

/// Recent messages for the HUD, oldest first.
#[derive(Resource, Default, Debug)]
pub struct Feedback {
    pub lines: Vec<String>,
}

impl Feedback {
    pub const MAX_LINES: usize = 3;

    pub fn clear(&mut self) {
        self.lines.clear();
    }

    pub fn say(&mut self, message: impl Into<String>) {
        self.lines.push(message.into());
        if self.lines.len() > Self::MAX_LINES {
            self.lines.drain(..self.lines.len() - Self::MAX_LINES);
        }
    }
}

/// Repeatable level-up choices. The same three are always eligible.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub enum UpgradeKind {
    Vitality,
    Strength,
    Recovery,
}

impl UpgradeKind {
    pub const ALL: [Self; 3] = [Self::Vitality, Self::Strength, Self::Recovery];

    pub const fn title(self) -> &'static str {
        match self {
            Self::Vitality => "Vitality",
            Self::Strength => "Strength",
            Self::Recovery => "Recovery",
        }
    }

    pub const fn flavor(self) -> &'static str {
        match self {
            Self::Vitality => "A bigger reserve and an immediate heal.",
            Self::Strength => "Every melee hit lands harder.",
            Self::Recovery => "Recover more health after each kill.",
        }
    }
}

/// Set by an upgrade card's mouse observer and read by the selection system.
#[derive(Resource, Default)]
pub struct UpgradeSelection(pub Option<UpgradeKind>);

/// Set by the end-screen restart button's mouse observer.
#[derive(Resource, Default)]
pub struct RestartRequested(pub bool);

/// The four gameplay modes. Rendering and UI react to these states.
#[derive(States, Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum GameState {
    #[default]
    Playing,
    ChoosingUpgrade,
    Won,
    Lost,
}
