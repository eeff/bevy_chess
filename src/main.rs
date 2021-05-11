use bevy::prelude::*;
use bevy_mod_picking::*;

mod board;
use board::BoardPlugin;

#[macro_use]
mod pieces;
use pieces::PiecePlugin;

mod ui;
use ui::UiPlugin;

fn main() {
    App::build()
        // Set antialiasing to use 4 samples
        .insert_resource(Msaa { samples: 4 })
        .insert_resource(WindowDescriptor {
            title: "Chess!".to_string(),
            width: 900.,
            height: 900.,
            ..Default::default()
        })
        .add_plugins(DefaultPlugins)
        .add_plugin(PickingPlugin)
        .add_plugin(InteractablePickingPlugin)
        .add_plugin(HighlightablePickingPlugin)
        .add_plugin(BoardPlugin)
        .add_plugin(PiecePlugin)
        .add_plugin(UiPlugin)
        .add_startup_system(setup.system())
        .run();
}

fn setup(mut commands: Commands) {
    // Camera
    commands
        .spawn_bundle(PerspectiveCameraBundle {
            transform: Transform::from_matrix(Mat4::from_rotation_translation(
                Quat::from_xyzw(-0.3, -0.5, -0.3, 0.5).normalize(),
                Vec3::new(-7.0, 20.0, 4.0),
            )),
            ..Default::default()
        })
        .insert_bundle(PickingCameraBundle::default());
    // Light
    commands.spawn_bundle(LightBundle {
        transform: Transform::from_translation(Vec3::new(4.0, 8.0, 4.0)),
        ..Default::default()
    });
}
