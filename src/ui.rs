//! Bevy UI: persistent HUD, three-choice level-up panel, and win/loss screens.
//!
//! UI reads gameplay resources/components and never mutates grid positions. The
//! two mouse paths write intent resources (`UpgradeSelection`, `RestartRequested`)
//! that gameplay systems consume.

use bevy::picking::hover::Hovered;
use bevy::prelude::*;
use bevy::text::{FontSize, Justify};
use bevy::ui::Pressed;
use bevy::ui_widgets::{Activate, Button as UiButton};

use crate::gameplay::{
    Attack, Boss, BossPhase, Enemy, Feedback, GameSet, GameState, Health, Player, PlayerProgress,
    Recovery, RestartRequested, TurnCount, UpgradeKind, UpgradeSelection, upgrade_effect_text,
    xp_threshold_for_next_level,
};

const PANEL_BG: Color = Color::srgba(0.035, 0.045, 0.065, 0.90);
const PANEL_BORDER: Color = Color::srgb(0.30, 0.34, 0.41);
const CARD_BG: Color = Color::srgb(0.11, 0.13, 0.17);
const CARD_BORDER: Color = Color::srgb(0.36, 0.40, 0.48);
const CARD_HOVER: Color = Color::srgb(0.18, 0.22, 0.28);
const CARD_PRESSED: Color = Color::srgb(0.24, 0.33, 0.26);
const TEXT: Color = Color::srgb(0.92, 0.93, 0.95);
const MUTED: Color = Color::srgb(0.64, 0.68, 0.75);
const ACCENT: Color = Color::srgb(0.45, 0.90, 0.58);
const WARNING: Color = Color::srgb(1.0, 0.42, 0.38);
const CAUTION: Color = Color::srgb(1.0, 0.80, 0.40);
const HP_GREEN: Color = Color::srgb(0.30, 0.82, 0.42);
const HP_YELLOW: Color = Color::srgb(0.95, 0.72, 0.25);
const HP_RED: Color = Color::srgb(0.92, 0.29, 0.27);
const BOSS_RED: Color = Color::srgb(0.86, 0.25, 0.28);
const RESTART_BG: Color = Color::srgb(0.20, 0.34, 0.24);

#[derive(Component)]
pub(crate) struct HudRoot;

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HudText {
    Hp,
    LevelXp,
    Stats,
    BossHp,
    BossPhase,
    Feedback,
    Turn,
}

#[derive(Component, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HudBar {
    PlayerHp,
    BossHp,
}

#[derive(Component)]
struct BossHudPanel;

#[derive(Component)]
pub(crate) struct UpgradePanelRoot;

#[derive(Component, Clone, Copy)]
enum UpgradeText {
    Title,
    Subtitle,
    Effect(UpgradeKind),
}

#[derive(Component)]
pub(crate) struct UpgradeChoice(pub(crate) UpgradeKind);

#[derive(Component)]
pub(crate) struct EndScreenRoot;

#[derive(Component)]
pub(crate) struct RestartButton;

type PlayerStatsQuery<'w, 's> = Query<
    'w,
    's,
    (
        &'static Health,
        &'static Attack,
        &'static Recovery,
        &'static PlayerProgress,
    ),
    (With<Player>, Without<Enemy>),
>;

type BossStatusQuery<'w, 's> = Query<'w, 's, (&'static Health, &'static BossPhase), With<Boss>>;

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, spawn_hud)
            .add_systems(Update, update_hud.in_set(GameSet::Ui))
            .add_systems(Update, update_restart_button_style)
            .add_systems(OnEnter(GameState::ChoosingUpgrade), spawn_upgrade_panel)
            .add_systems(
                OnExit(GameState::ChoosingUpgrade),
                despawn_marked::<UpgradePanelRoot>,
            )
            .add_systems(
                Update,
                (update_upgrade_text, update_upgrade_card_style)
                    .run_if(in_state(GameState::ChoosingUpgrade)),
            )
            .add_systems(OnEnter(GameState::Won), spawn_victory_screen)
            .add_systems(OnEnter(GameState::Lost), spawn_defeat_screen)
            .add_systems(OnExit(GameState::Won), despawn_marked::<EndScreenRoot>)
            .add_systems(OnExit(GameState::Lost), despawn_marked::<EndScreenRoot>);
    }
}

fn text_bundle(content: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        Text::new(content),
        TextFont {
            font_size: FontSize::Px(size),
            ..default()
        },
        TextColor(color),
    )
}

fn centered_text(content: impl Into<String>, size: f32, color: Color) -> impl Bundle {
    (
        text_bundle(content, size, color),
        TextLayout::justify(Justify::Center),
    )
}

fn panel_node(left: f32, top: f32, width: f32, height: f32) -> Node {
    Node {
        position_type: PositionType::Absolute,
        left: Val::Px(left),
        top: Val::Px(top),
        width: Val::Px(width),
        height: Val::Px(height),
        padding: UiRect::all(Val::Px(8.0)),
        flex_direction: FlexDirection::Column,
        row_gap: Val::Px(4.0),
        border: UiRect::all(Val::Px(1.0)),
        border_radius: BorderRadius::all(Val::Px(7.0)),
        ..default()
    }
}

fn spawn_hud(mut commands: Commands) {
    // Top-left: player health, level, XP, and combat stats.
    commands
        .spawn((
            HudRoot,
            panel_node(12.0, 8.0, 420.0, 100.0),
            BackgroundColor(PANEL_BG),
            BorderColor::all(PANEL_BORDER),
        ))
        .with_children(|panel| {
            panel.spawn((text_bundle("HP 24 / 24", 16.0, TEXT), HudText::Hp));
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(12.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.09, 0.10, 0.13)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        HudBar::PlayerHp,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(HP_GREEN),
                    ));
                });
            panel.spawn((
                text_bundle("LV 1  -  XP 0 / 8", 15.0, TEXT),
                HudText::LevelXp,
            ));
            panel.spawn((
                text_bundle("ATK 4  -  HEAL/KILL +2", 13.0, MUTED),
                HudText::Stats,
            ));
        });

    // Top-right: boss health and the telegraph warning.
    let mut boss_panel = panel_node(848.0, 8.0, 420.0, 100.0);
    boss_panel.display = Display::None;
    commands
        .spawn((
            HudRoot,
            BossHudPanel,
            boss_panel,
            BackgroundColor(PANEL_BG),
            BorderColor::all(PANEL_BORDER),
        ))
        .with_children(|panel| {
            panel.spawn((text_bundle("THE WARDEN", 18.0, WARNING), HudText::BossHp));
            panel
                .spawn((
                    Node {
                        width: Val::Percent(100.0),
                        height: Val::Px(12.0),
                        border_radius: BorderRadius::all(Val::Px(3.0)),
                        ..default()
                    },
                    BackgroundColor(Color::srgb(0.09, 0.10, 0.13)),
                ))
                .with_children(|bar| {
                    bar.spawn((
                        HudBar::BossHp,
                        Node {
                            width: Val::Percent(100.0),
                            height: Val::Percent(100.0),
                            border_radius: BorderRadius::all(Val::Px(3.0)),
                            ..default()
                        },
                        BackgroundColor(BOSS_RED),
                    ));
                });
            panel.spawn((
                text_bundle("Guarding the sealed exit.", 13.0, MUTED),
                HudText::BossPhase,
            ));
        });

    // Bottom-left: recent action feedback.
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(12.0),
                bottom: Val::Px(38.0),
                width: Val::Px(820.0),
                height: Val::Px(70.0),
                padding: UiRect::all(Val::Px(8.0)),
                flex_direction: FlexDirection::Column,
                border: UiRect::all(Val::Px(1.0)),
                border_radius: BorderRadius::all(Val::Px(7.0)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor::all(PANEL_BORDER),
        ))
        .with_children(|panel| {
            panel.spawn((
                text_bundle("Explore the dungeon.", 13.0, TEXT),
                HudText::Feedback,
            ));
        });

    // Bottom-right: turn counter.
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                right: Val::Px(12.0),
                bottom: Val::Px(38.0),
                width: Val::Px(160.0),
                height: Val::Px(32.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                border_radius: BorderRadius::all(Val::Px(6.0)),
                ..default()
            },
            BackgroundColor(PANEL_BG),
            BorderColor::all(PANEL_BORDER),
        ))
        .with_children(|panel| {
            panel.spawn((text_bundle("TURN 0", 13.0, MUTED), HudText::Turn));
        });

    // Very bottom: concise controls.
    commands
        .spawn((
            HudRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                bottom: Val::Px(8.0),
                width: Val::Percent(100.0),
                height: Val::Px(24.0),
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                ..default()
            },
        ))
        .with_children(|panel| {
            panel.spawn(centered_text(
                "WASD / Arrows: move   Space: wait   Bump: attack   1-3: choose   R: restart",
                13.0,
                MUTED,
            ));
        });
}

#[allow(clippy::too_many_arguments)]
fn update_hud(
    players: PlayerStatsQuery,
    bosses: BossStatusQuery,
    feedback: Res<Feedback>,
    turn: Res<TurnCount>,
    mut texts: Query<(&HudText, &mut Text, &mut TextColor)>,
    mut bars: Query<(&HudBar, &mut Node, &mut BackgroundColor), Without<BossHudPanel>>,
    mut boss_panels: Query<&mut Node, With<BossHudPanel>>,
) {
    let player = players.single().ok();
    let boss = bosses.single().ok();

    for (kind, mut text, mut color) in &mut texts {
        match kind {
            HudText::Hp => {
                if let Some((health, _, _, _)) = player {
                    text.0 = format!("HP {} / {}", health.current, health.max);
                    color.0 = if health.fraction() > 0.25 {
                        TEXT
                    } else {
                        WARNING
                    };
                } else {
                    text.0 = "HP -- / --".to_string();
                    color.0 = MUTED;
                }
            }
            HudText::LevelXp => {
                if let Some((_, _, _, progress)) = player {
                    text.0 = match xp_threshold_for_next_level(progress.level) {
                        Some(threshold) => format!(
                            "LEVEL {}  -  XP {} / {}",
                            progress.level, progress.xp, threshold
                        ),
                        None => format!("LEVEL {}  -  MAX", progress.level),
                    };
                } else {
                    text.0 = "LEVEL --  -  XP --".to_string();
                }
                color.0 = TEXT;
            }
            HudText::Stats => {
                if let Some((_, attack, recovery, _)) = player {
                    text.0 = format!(
                        "ATK {}  -  HEAL/KILL +{}",
                        attack.damage, recovery.heal_on_kill
                    );
                } else {
                    text.0 = "ATK --  -  HEAL/KILL --".to_string();
                }
                color.0 = MUTED;
            }
            HudText::BossHp => {
                if let Some((health, _)) = boss {
                    text.0 = format!("THE WARDEN   {} / {}", health.current, health.max);
                } else {
                    text.0 = "THE WARDEN".to_string();
                }
                color.0 = WARNING;
            }
            HudText::BossPhase => {
                if let Some((_, phase)) = boss {
                    let (message, phase_color) = match phase {
                        BossPhase::Ready => ("Guarding the sealed exit.", MUTED),
                        BossPhase::WindingUp { .. } => {
                            ("HEAVY ATTACK MARKED - MOVE OFF THE RED TILE!", WARNING)
                        }
                        BossPhase::Recovering => ("Recovering - strike now!", CAUTION),
                    };
                    text.0 = message.to_string();
                    color.0 = phase_color;
                } else {
                    text.0 = "The Warden has fallen. The exit is open.".to_string();
                    color.0 = ACCENT;
                }
            }
            HudText::Feedback => {
                text.0 = if feedback.lines.is_empty() {
                    "Explore the dungeon.".to_string()
                } else {
                    feedback.lines.join("\n")
                };
                color.0 = TEXT;
            }
            HudText::Turn => {
                text.0 = format!("TURN {}", turn.0);
                color.0 = MUTED;
            }
        }
    }

    for (kind, mut node, mut background) in &mut bars {
        match kind {
            HudBar::PlayerHp => {
                let fraction = player
                    .map(|(health, _, _, _)| health.fraction())
                    .unwrap_or(0.0);
                node.width = Val::Percent(fraction * 100.0);
                background.0 = if fraction > 0.5 {
                    HP_GREEN
                } else if fraction > 0.25 {
                    HP_YELLOW
                } else {
                    HP_RED
                };
            }
            HudBar::BossHp => {
                let fraction = boss.map(|(health, _)| health.fraction()).unwrap_or(0.0);
                node.width = Val::Percent(fraction * 100.0);
                background.0 = BOSS_RED;
            }
        }
    }

    for mut node in &mut boss_panels {
        node.display = if boss.is_some() {
            Display::Flex
        } else {
            Display::None
        };
    }
}

fn spawn_upgrade_panel(mut commands: Commands, players: PlayerStatsQuery) {
    let Ok((health, attack, recovery, progress)) = players.single() else {
        return;
    };

    commands
        .spawn((
            UpgradePanelRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(16.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.92)),
        ))
        .with_children(|root| {
            root.spawn((
                centered_text("LEVEL UP - CHOOSE ONE", 32.0, ACCENT),
                UpgradeText::Title,
            ));
            root.spawn((
                centered_text(
                    format!(
                        "{} level-up choice{} waiting",
                        progress.pending_upgrades,
                        if progress.pending_upgrades == 1 {
                            ""
                        } else {
                            "s"
                        }
                    ),
                    16.0,
                    MUTED,
                ),
                UpgradeText::Subtitle,
            ));

            root.spawn(Node {
                flex_direction: FlexDirection::Row,
                column_gap: Val::Px(18.0),
                ..default()
            })
            .with_children(|row| {
                for (index, kind) in UpgradeKind::ALL.into_iter().enumerate() {
                    spawn_upgrade_card(row, kind, index + 1, health, attack, recovery);
                }
            });

            root.spawn(centered_text(
                "Press 1, 2, or 3 - or click a card",
                14.0,
                MUTED,
            ));
        });
}

fn spawn_upgrade_card(
    parent: &mut ChildSpawnerCommands,
    kind: UpgradeKind,
    number: usize,
    health: &Health,
    attack: &Attack,
    recovery: &Recovery,
) {
    parent
        .spawn((
            UiButton,
            Hovered::default(),
            UpgradeChoice(kind),
            Node {
                width: Val::Px(300.0),
                height: Val::Px(230.0),
                padding: UiRect::all(Val::Px(16.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(12.0),
                border: UiRect::all(Val::Px(2.0)),
                border_radius: BorderRadius::all(Val::Px(10.0)),
                ..default()
            },
            BackgroundColor(CARD_BG),
            BorderColor::all(CARD_BORDER),
        ))
        .observe(
            move |_: On<Activate>, mut selection: ResMut<UpgradeSelection>| {
                selection.0 = Some(kind);
            },
        )
        .with_children(|card| {
            card.spawn(centered_text(
                format!("{number}  {}", kind.title().to_uppercase()),
                22.0,
                ACCENT,
            ));
            card.spawn((
                centered_text(
                    upgrade_effect_text(kind, health, attack, recovery),
                    20.0,
                    TEXT,
                ),
                UpgradeText::Effect(kind),
            ));
            card.spawn(centered_text(kind.flavor(), 14.0, MUTED));
        });
}

fn update_upgrade_text(players: PlayerStatsQuery, mut texts: Query<(&UpgradeText, &mut Text)>) {
    let Ok((health, attack, recovery, progress)) = players.single() else {
        return;
    };

    for (kind, mut text) in &mut texts {
        text.0 = match kind {
            UpgradeText::Title => format!("LEVEL {} - CHOOSE ONE", progress.level),
            UpgradeText::Subtitle => format!(
                "{} level-up choice{} waiting",
                progress.pending_upgrades,
                if progress.pending_upgrades == 1 {
                    ""
                } else {
                    "s"
                }
            ),
            UpgradeText::Effect(kind) => upgrade_effect_text(*kind, health, attack, recovery),
        };
    }
}

fn update_upgrade_card_style(
    mut cards: Query<(
        &Hovered,
        Has<Pressed>,
        &UpgradeChoice,
        &mut BackgroundColor,
        &mut BorderColor,
    )>,
) {
    for (hovered, pressed, choice, mut background, mut border) in &mut cards {
        background.0 = hover_color(hovered.get(), pressed, CARD_BG, CARD_HOVER, CARD_PRESSED);
        border.set_all(if hovered.get() || pressed {
            upgrade_accent(choice.0)
        } else {
            CARD_BORDER
        });
    }
}

fn upgrade_accent(kind: UpgradeKind) -> Color {
    match kind {
        UpgradeKind::Vitality => ACCENT,
        UpgradeKind::Strength => CAUTION,
        UpgradeKind::Recovery => Color::srgb(0.45, 0.72, 1.0),
    }
}

fn update_restart_button_style(
    mut buttons: Query<(&Hovered, Has<Pressed>, &mut BackgroundColor), With<RestartButton>>,
) {
    for (hovered, pressed, mut background) in &mut buttons {
        background.0 = hover_color(
            hovered.get(),
            pressed,
            RESTART_BG,
            Color::srgb(0.28, 0.48, 0.33),
            Color::srgb(0.38, 0.62, 0.44),
        );
    }
}

fn hover_color(
    hovered: bool,
    pressed: bool,
    normal: Color,
    hover: Color,
    pressed_color: Color,
) -> Color {
    if pressed {
        pressed_color
    } else if hovered {
        hover
    } else {
        normal
    }
}

fn spawn_victory_screen(
    mut commands: Commands,
    players: Query<(&Health, &PlayerProgress), With<Player>>,
    turn: Res<TurnCount>,
) {
    spawn_end_screen(&mut commands, players.single().ok(), turn.0, true);
}

fn spawn_defeat_screen(
    mut commands: Commands,
    players: Query<(&Health, &PlayerProgress), With<Player>>,
    turn: Res<TurnCount>,
) {
    spawn_end_screen(&mut commands, players.single().ok(), turn.0, false);
}

fn spawn_end_screen(
    commands: &mut Commands,
    player: Option<(&Health, &PlayerProgress)>,
    turns: u32,
    won: bool,
) {
    let (title, title_color, subtitle) = if won {
        ("VICTORY", ACCENT, "You escaped the dungeon.")
    } else {
        (
            "YOU DIED",
            WARNING,
            "The dungeon claims another adventurer.",
        )
    };

    let stats = if let Some((health, progress)) = player {
        format!(
            "Level {}  -  XP {}  -  {} turns\nFinal HP {} / {}",
            progress.level, progress.xp, turns, health.current, health.max
        )
    } else {
        format!("{turns} turns")
    };

    commands
        .spawn((
            EndScreenRoot,
            Node {
                position_type: PositionType::Absolute,
                left: Val::Px(0.0),
                top: Val::Px(0.0),
                width: Val::Percent(100.0),
                height: Val::Percent(100.0),
                padding: UiRect::all(Val::Px(24.0)),
                flex_direction: FlexDirection::Column,
                align_items: AlignItems::Center,
                justify_content: JustifyContent::Center,
                row_gap: Val::Px(18.0),
                ..default()
            },
            BackgroundColor(Color::srgba(0.02, 0.03, 0.05, 0.94)),
        ))
        .with_children(|root| {
            root.spawn(centered_text(title, 54.0, title_color));
            root.spawn(centered_text(subtitle, 20.0, TEXT));
            root.spawn(centered_text(stats, 16.0, MUTED));

            root.spawn((
                UiButton,
                Hovered::default(),
                RestartButton,
                Node {
                    width: Val::Px(280.0),
                    height: Val::Px(58.0),
                    margin: UiRect {
                        top: Val::Px(12.0),
                        ..default()
                    },
                    align_items: AlignItems::Center,
                    justify_content: JustifyContent::Center,
                    border: UiRect::all(Val::Px(2.0)),
                    border_radius: BorderRadius::all(Val::Px(9.0)),
                    ..default()
                },
                BackgroundColor(RESTART_BG),
                BorderColor::all(ACCENT),
            ))
            .observe(|_: On<Activate>, mut request: ResMut<RestartRequested>| {
                request.0 = true;
            })
            .with_children(|button| {
                button.spawn(centered_text("RESTART   (R)", 22.0, TEXT));
            });

            root.spawn(centered_text(
                "Restart resets enemies, XP, upgrades, and all run state.",
                13.0,
                MUTED,
            ));
        });
}

fn despawn_marked<T: Component>(mut commands: Commands, entities: Query<Entity, With<T>>) {
    for entity in &entities {
        commands.entity(entity).despawn();
    }
}
