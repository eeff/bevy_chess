use crate::board::*;
use bevy::prelude::*;

/// Text entity marker
struct NextMoveText;

/// Initialize UiCamera and text
fn init_next_move_text(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut color_material: ResMut<Assets<ColorMaterial>>,
) {
    let font = asset_server.load("fonts/FiraSans-Bold.ttf");
    let material = color_material.add(Color::NONE.into());

    commands.spawn_bundle(UiCameraBundle::default());
    commands
        .spawn_bundle(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                position: Rect {
                    left: Val::Px(10.0),
                    top: Val::Px(10.0),
                    ..Default::default()
                },
                ..Default::default()
            },
            material,
            ..Default::default()
        })
        .with_children(|parent| {
            parent
                .spawn_bundle(TextBundle {
                    text: Text::with_section(
                        "Next move: White".to_string(),
                        TextStyle {
                            font,
                            font_size: 40.0,
                            color: Color::rgb(0.8, 0.8, 0.8),
                        },
                        TextAlignment::default(),
                    ),
                    ..Default::default()
                })
                .insert(NextMoveText);
        });
}

/// Update text with turn
fn next_move_text_update(turn: ResMut<PlayerTurn>, mut query: Query<(&mut Text, &NextMoveText)>) {
    for (mut text, _) in query.iter_mut() {
        if turn.is_changed() {
            text.sections[0].value = format!("Next move: {}", *turn);
        }
    }
}

pub struct UiPlugin;

impl Plugin for UiPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(init_next_move_text.system())
            .add_system(next_move_text_update.system());
    }
}
