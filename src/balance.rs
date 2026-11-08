//! Combat, progression, and upgrade tuning in one named place.
//!
//! These are deliberately modest values for a short run. Change numbers here
//! rather than scattering magic values through systems.

// Player.
pub const PLAYER_START_MAX_HP: i32 = 24;
pub const PLAYER_START_ATTACK: i32 = 4;
pub const PLAYER_START_HEAL_ON_KILL: i32 = 2;

// Pursuer: fragile, quick, and always acts after a player turn.
pub const SKELETON_MAX_HP: i32 = 8;
pub const SKELETON_ATTACK: i32 = 2;
pub const SKELETON_XP: i32 = 4;

// Brute: tougher and slower; acts on every second player turn.
pub const BRUTE_MAX_HP: i32 = 14;
pub const BRUTE_ATTACK: i32 = 3;
pub const BRUTE_XP: i32 = 7;

// Boss: stationary, telegraphed heavy attack, then one recovery turn.
pub const BOSS_MAX_HP: i32 = 32;
pub const BOSS_HEAVY_DAMAGE: i32 = 7;
pub const BOSS_THREAT_RANGE: i32 = 3;
pub const BOSS_XP: i32 = 0;

/// Cumulative XP needed for levels 2, 3, and 4.
pub const XP_THRESHOLDS: [i32; 3] = [8, 18, 30];
pub const MAX_LEVEL: u32 = 4;

// Upgrades. Each is repeatable.
pub const VITALITY_MAX_HP: i32 = 4;
pub const VITALITY_HEAL: i32 = 4;
pub const STRENGTH_ATTACK: i32 = 1;
pub const RECOVERY_HEAL: i32 = 1;
