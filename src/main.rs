//! First visual checkpoint for a small, turn-based dungeon game.

use bevy::prelude::*;

/// The player's width and height in world units; one unit is one pixel at default zoom.
const PLAYER_SIZE: f32 = 32.0;

#[derive(Component, Debug)]
struct Player;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::srgb(0.04, 0.05, 0.08)))
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Dungeon escape — first scene".into(),
                resolution: (960, 640).into(),
                ..default()
            }),
            ..default()
        }))
        .add_systems(Startup, setup)
        .run();
}

fn setup(mut commands: Commands) {
    commands.spawn(Camera2d);
    commands.spawn((
        Player,
        Sprite::from_color(Color::srgb(0.3, 0.9, 0.5), Vec2::splat(PLAYER_SIZE)),
        Transform::from_xyz(0.0, 0.0, 0.0),
    ));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_creates_one_camera_and_visible_player_without_duplicates() {
        let mut app = App::new();
        app.add_systems(Startup, setup);

        app.update();
        app.update();

        let world = app.world_mut();
        assert_eq!(
            world.query::<&Camera2d>().iter(world).count(),
            1,
            "startup should create exactly one 2D camera"
        );

        let mut players =
            world.query_filtered::<(&Sprite, &Transform, &Visibility), With<Player>>();
        let (sprite, transform, visibility) = players
            .single(world)
            .expect("startup should create exactly one renderable player");
        assert_eq!(transform.translation, Vec3::ZERO);
        assert_eq!(sprite.custom_size, Some(Vec2::splat(PLAYER_SIZE)));
        assert!(
            sprite.color.alpha() > 0.0,
            "the player must not be transparent"
        );
        assert_ne!(*visibility, Visibility::Hidden);
    }
}
