use bevy::app::AppExit;
use bevy::prelude::*;
use bevy_mod_picking::{MeshButtonMaterials, PickableBundle, PickingEvent, SelectionEvent};

use crate::pieces::*;

use std::fmt;

pub struct Square {
    pub x: u8,
    pub y: u8,
}

#[derive(Default)]
pub struct SelectedSquare {
    entity: Option<Entity>,
}

#[derive(Default)]
pub struct SelectedPiece {
    entity: Option<Entity>,
}

struct Taken;

pub struct PlayerTurn(pub PieceColor);

impl PlayerTurn {
    fn toggle(&mut self) {
        self.0 = match self.0 {
            PieceColor::White => PieceColor::Black,
            PieceColor::Black => PieceColor::White,
        }
    }
}

impl Default for PlayerTurn {
    fn default() -> Self {
        Self(PieceColor::White)
    }
}

impl fmt::Display for PlayerTurn {
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        write!(
            formatter,
            "{}",
            match self.0 {
                PieceColor::White => "White",
                PieceColor::Black => "Black",
            }
        )
    }
}

pub struct BoardPlugin;

impl Plugin for BoardPlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.init_resource::<SelectedSquare>()
            .init_resource::<SelectedPiece>()
            .init_resource::<PlayerTurn>()
            .add_startup_system(create_board.system())
            .add_system(select_squares.system().label("select_square"))
            .add_system(
                select_piece
                    .system()
                    .label("select_piece")
                    .after("select_square"),
            )
            .add_system(
                move_piece
                    .system()
                    .label("move_piece")
                    .after("select_piece"),
            )
            .add_system(
                remove_taken_pieces
                    .system()
                    .label("remove_taken_piece")
                    .after("move_piece"),
            );
    }
}

fn create_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    mut button_materials: ResMut<MeshButtonMaterials>,
) {
    let mesh = meshes.add(Mesh::from(shape::Plane { size: 1. }));

    let white_material = materials.add(Color::rgb(1.0, 0.9, 0.9).into());
    let black_material = materials.add(Color::rgb(0.0, 0.1, 0.1).into());

    button_materials.hovered = materials.add(Color::rgb(0.8, 0.3, 0.3).into());
    button_materials.selected = materials.add(Color::rgb(0.9, 0.1, 0.1).into());

    // 64 squares
    for i in 0..8 {
        for j in 0..8 {
            // Alternating square pattern
            let square_material = if (i + j + 1) % 2 == 0 {
                white_material.clone()
            } else {
                black_material.clone()
            };
            commands
                .spawn_bundle(PbrBundle {
                    mesh: mesh.clone(),
                    material: square_material,
                    transform: Transform::from_translation(Vec3::new(i as f32, 0.0, j as f32)),
                    ..Default::default()
                })
                .insert_bundle(PickableBundle::default())
                .insert(Square { x: i, y: j });
        }
    }
}

fn select_squares(
    mut selected_square: ResMut<SelectedSquare>,
    mut picking_event_reader: EventReader<PickingEvent>,
) {
    // Interested only in selectino event
    let selection_events = picking_event_reader.iter().filter_map(|e| match e {
        PickingEvent::Selection(selection) => Some(selection),
        _ => None,
    });

    for event in selection_events {
        match event {
            SelectionEvent::JustSelected(entity) => {
                // Mark selected square
                selected_square.entity.replace(*entity);
                // println!("select square: {:?}", *entity);
            }
            SelectionEvent::JustDeselected(entity) => {
                if Some(*entity) == selected_square.entity {
                    // println!("deselect square: {:?}", *entity);
                    selected_square.entity = None;
                }
            }
        }
    }
}

fn select_piece(
    selected_square: Res<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    turn: Res<PlayerTurn>,
    mut squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
) {
    if selected_square.is_changed() {
        if let Some(square_entity) = selected_square.entity {
            let square = squares_query.get_mut(square_entity).unwrap();
            if selected_piece.entity.is_none() {
                for (piece_entity, piece) in pieces_query.iter_mut() {
                    if piece.x == square.x && piece.y == square.y && piece.color == turn.0 {
                        selected_piece.entity = Some(piece_entity);
                        // println!("select piece: {:?}", piece_entity);
                        break;
                    }
                }
            }
        } else {
            // println!("deselect piece: {:?}", selected_piece.entity);
            selected_piece.entity.take();
        }
    }
}

fn move_piece(
    mut commands: Commands,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut turn: ResMut<PlayerTurn>,
    mut squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
) {
    if !selected_square.is_changed() {
        return;
    }

    let square_entity = match selected_square.entity {
        Some(entity) => entity,
        None => return,
    };

    if selected_piece.is_changed() {
        return;
    }

    let piece_entity = match selected_piece.entity {
        Some(entity) => entity,
        None => return,
    };

    let square = squares_query.get_mut(square_entity).unwrap();
    let pieces: Vec<Piece> = pieces_query.iter_mut().map(|(_, p)| *p).collect();
    // Find piece at the selected square
    let other_entity = pieces_query
        .iter_mut()
        .filter_map(|(e, piece)| {
            if piece.x == square.x && piece.y == square.y {
                Some(e)
            } else {
                None
            }
        })
        .next();
    let (_, mut piece) = pieces_query.get_mut(piece_entity).unwrap();

    if piece.is_move_valid((square.x, square.y), &pieces) {
        if other_entity.is_some() {
            commands.entity(other_entity.unwrap()).insert(Taken);
        }

        // Move selected piece
        piece.x = square.x;
        piece.y = square.y;

        // Update turn
        turn.toggle();
    }

    // Clear selected square and selected piece
    selected_square.entity = None;
    selected_piece.entity = None;
    // println!("deselect from move");
}

fn remove_taken_pieces(
    mut commands: Commands,
    mut app_exit_events: EventWriter<AppExit>,
    query: Query<(Entity, &Piece, &Taken)>,
) {
    for (entity, piece, _) in query.iter() {
        // If king is taken, game is over
        if piece.piece_type == PieceType::King {
            app_exit_events.send(AppExit);
        }
        // Remove the piece
        commands.entity(entity).despawn_recursive();
    }
}

fn _select_squares(
    mut commands: Commands,
    mut selected_square: ResMut<SelectedSquare>,
    mut selected_piece: ResMut<SelectedPiece>,
    mut turn: ResMut<PlayerTurn>,
    mut picking_event_reader: EventReader<PickingEvent>,
    mut app_exit_events: EventWriter<AppExit>,
    mut squares_query: Query<&Square>,
    mut pieces_query: Query<(Entity, &mut Piece)>,
) {
    // Interested only in selectino event
    let selection_events = picking_event_reader.iter().filter_map(|e| match e {
        PickingEvent::Selection(selection) => Some(selection),
        _ => None,
    });

    let mut deselected = false;
    let pieces: Vec<_> = pieces_query.iter_mut().map(|(_, p)| *p).collect();

    for event in selection_events {
        match event {
            SelectionEvent::JustSelected(entity) => {
                let square_entity = *entity;
                let square = squares_query.get_mut(square_entity).unwrap();

                // Mark selected square
                deselected = false;
                selected_square.entity = Some(square_entity);
                if let Some(peice_entity) = selected_piece.entity {
                    // Find piece at the selected square
                    let other = pieces_query
                        .iter_mut()
                        .filter_map(|(e, piece)| {
                            if piece.x == square.x && piece.y == square.y {
                                Some((e, *piece))
                            } else {
                                None
                            }
                        })
                        .next();
                    // Move the selected piece to the selected square
                    let (_, mut piece) = pieces_query.get_mut(peice_entity).unwrap();
                    if piece.is_move_valid((square.x, square.y), &pieces) {
                        // Remove the piece at the selected square
                        if other.is_some() {
                            let (other_entity, other_piece) = other.unwrap();
                            // If king is taken, game is over
                            if other_piece.piece_type == PieceType::King {
                                app_exit_events.send(AppExit);
                            }
                            commands.entity(other_entity).despawn_recursive();
                        }
                        // Move selected piece
                        piece.x = square.x;
                        piece.y = square.y;
                        // Update turn
                        turn.toggle();
                    }
                    // Clear selected square and selected piece
                    selected_square.entity = None;
                    selected_piece.entity = None;
                } else {
                    // Select the piece in the currently selected square
                    for (piece_entity, piece) in pieces_query.iter_mut() {
                        if piece.x == square.x && piece.y == square.y && piece.color == turn.0 {
                            selected_piece.entity = Some(piece_entity);
                            break;
                        }
                    }
                }
            }
            SelectionEvent::JustDeselected(entity) => {
                deselected = Some(*entity) == selected_square.entity;
            }
        };
    }

    if deselected {
        // Deselection
        selected_square.entity = None;
        selected_piece.entity = None;
    }
}
