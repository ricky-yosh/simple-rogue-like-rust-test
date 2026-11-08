//! A short, handcrafted, turn-based dungeon roguelike built with Bevy 0.20.

mod balance;
mod gameplay;
mod grid;
mod render;
mod ui;

use bevy::prelude::*;

use gameplay::GamePlugin;
use render::RenderPlugin;
use ui::UiPlugin;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.025, 0.03, 0.045)))
        .add_plugins(
            DefaultPlugins
                .set(ImagePlugin::default_nearest())
                .set(WindowPlugin {
                    primary_window: Some(Window {
                        title: "Dungeon escape".into(),
                        resolution: (1280, 800).into(),
                        ..default()
                    }),
                    ..default()
                }),
        )
        .add_plugins((GamePlugin, RenderPlugin, UiPlugin))
        .run();
}
