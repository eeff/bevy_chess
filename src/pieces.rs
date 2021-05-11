use bevy::prelude::*;

/// Helper macro for spawning a chess piece.
#[macro_export]
macro_rules! spawn_piece {
    ($commands:expr, ($x:expr, $y:expr), $translation:expr, $type:expr, $color:expr, $material:expr, $($mesh:expr),+$(,)?) => {{
        use bevy::prelude::{PbrBundle, Transform, Vec3};
        $commands
            .spawn_bundle(PbrBundle {
                transform: Transform::from_translation(Vec3::new($x as f32, 0.0, $y as f32)),
                ..Default::default()
            })
            .insert(Piece {
                color: $color,
                piece_type: $type,
                x: $x,
                y: $y,
            })
            .with_children(|parent| {
                $(
                    parent.spawn_bundle(PbrBundle {
                        mesh: $mesh,
                        material: $material,
                        transform: {
                            let mut transform = Transform::from_translation($translation);
                            transform.apply_non_uniform_scale(Vec3::new(0.2, 0.2, 0.2));
                            transform
                        },
                        ..Default::default()
                    });
                )+
            });
    }};
}

/// Piece Plugin
pub struct PiecePlugin;

impl Plugin for PiecePlugin {
    fn build(&self, app: &mut AppBuilder) {
        app.add_startup_system(create_pieces.system())
            .add_system(move_pieces.system());
    }
}

/// Color of a chess piece
#[derive(Clone, Copy, PartialEq)]
pub enum PieceColor {
    White,
    Black,
}

/// Type of a chess piece
#[derive(Clone, Copy, PartialEq)]
pub enum PieceType {
    King,
    Queen,
    Bishop,
    Knight,
    Rook,
    Pawn,
}

/// A chess piece
#[derive(Clone, Copy)]
pub struct Piece {
    /// piece color
    pub color: PieceColor,
    /// piece type
    pub piece_type: PieceType,
    /// position x
    pub x: u8,
    /// position y
    pub y: u8,
}

impl Piece {
    /// Check if moving to the new position is valid.
    pub fn is_move_valid(&self, new_position: (u8, u8), pieces: &Vec<Piece>) -> bool {
        // Check not the same color
        // As a side effect, also guards we are not moving to the current position.
        if Some(self.color) == color_of_square(new_position, &pieces) {
            return false;
        }

        let dist = Manhatan::from((self.x, self.y), new_position);

        match self.piece_type {
            PieceType::King => {
                // Move exactly one square horizontally, vertically, or diagonally.
                dist.abs().max() == 1
            }
            PieceType::Queen => {
                // Move any number of vacant squares diagonally, horizontally, or vertically.
                is_path_empty((self.x, self.y), new_position, &pieces)
                    && (dist.dignonal() || dist.straight())
            }
            PieceType::Bishop => {
                // Move any number of vacant squares in any diagonal direction.
                is_path_empty((self.x, self.y), new_position, &pieces) && dist.dignonal()
            }
            PieceType::Knight => {
                // Move as an “L” or “7″ laid out at any horizontal or vertical angle.
                let abs = dist.abs();
                abs == Manhatan(2, 1) || abs == Manhatan(1, 2)
            }
            PieceType::Rook => {
                // Move any number of vacant squares vertically or horizontally.
                is_path_empty((self.x, self.y), new_position, pieces) && dist.straight()
            }
            PieceType::Pawn => {
                let dist = if self.color == PieceColor::White {
                    Manhatan::from(new_position, (self.x, self.y))
                } else {
                    dist
                };
                let square_color = color_of_square(new_position, &pieces);

                // A pawn cannot move backward.

                // Can move forward one square, if that square is unoccupied
                if dist == Manhatan(1, 0) {
                    if square_color.is_none() {
                        return true;
                    }
                }

                // If it has not yet moved, the pawn has the option of moving two squares
                // forward provided both squares in front of the pawn are unoccupied.
                if (self.color == PieceColor::White && self.x == 1
                    || self.color == PieceColor::Black && self.x == 6)
                    && dist == Manhatan(2, 0)
                    && is_path_empty((self.x, self.y), new_position, &pieces)
                {
                    if square_color.is_none() {
                        return true;
                    }
                }

                // Take an enemy piece on either of the two squares diagonally in front of
                // them, but cannot move to these spaces if they are vacant.
                if dist.0 == 1 && dist.1.abs() == 1 {
                    if square_color.is_some() {
                        // Must be of the opposite color
                        return true;
                    }
                }

                false
            }
        }
    }
}

/// Manhatan distance.
#[derive(Clone, Copy, PartialEq, Eq)]
struct Manhatan(i8, i8);

impl Manhatan {
    /// Manhantan distance of two position.
    #[inline]
    fn from((x1, y1): (u8, u8), (x2, y2): (u8, u8)) -> Manhatan {
        Manhatan(x1 as i8 - x2 as i8, y1 as i8 - y2 as i8)
    }

    /// Absolute value of manhatan distance.
    #[inline]
    fn abs(self) -> Manhatan {
        Manhatan(self.0.abs(), self.1.abs())
    }

    /// Max of one component distance.
    #[inline]
    fn max(self) -> i8 {
        self.0.max(self.1)
    }

    /// Min of one component distance.
    #[inline]
    fn min(self) -> i8 {
        self.0.min(self.1)
    }

    /// Check if is straight, aka horizontal or vertical.
    #[inline]
    fn straight(self) -> bool {
        self.abs().min() == 0
    }

    /// Check if is diagonal.
    #[inline]
    fn dignonal(self) -> bool {
        let abs = self.abs();
        abs.0 == abs.1
    }
}

/// Get color of piece in the square on the given position.
fn color_of_square(position: (u8, u8), pieces: &Vec<Piece>) -> Option<PieceColor> {
    for piece in pieces {
        if piece.x == position.0 && piece.y == position.1 {
            return Some(piece.color);
        }
    }
    None
}

/// Check the path from a starting position to an end position is empty.
/// The path could be a row, a column or a diagonal.
fn is_path_empty((x1, y1): (u8, u8), (x2, y2): (u8, u8), pieces: &Vec<Piece>) -> bool {
    // Same column
    if x1 == x2 {
        for piece in pieces {
            if piece.x == x1 && (piece.y > y1 && piece.y < y2 || piece.y > y2 && piece.y < y1) {
                return false;
            }
        }
    }

    // Same row
    if y1 == y2 {
        for piece in pieces {
            if piece.y == y1 && (piece.x > x1 && piece.x < x2 || piece.x > x2 && piece.x < x1) {
                return false;
            }
        }
    }

    // Diagonal
    let xdiff = x1 as i8 - x2 as i8;
    let ydiff = y1 as i8 - y2 as i8;
    if xdiff.abs() == ydiff.abs() {
        for i in 1..xdiff.abs() {
            let pos = (
                (x1 as i8 + xdiff.signum() * i) as u8,
                (y1 as i8 + ydiff.signum() * i) as u8,
            );
            if color_of_square(pos, pieces).is_some() {
                return false;
            }
        }
    }

    true
}

fn create_pieces(
    mut commands: Commands,
    asset_server: ResMut<AssetServer>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let king_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh0/Primitive0");
    let king_cross_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh1/Primitive0");
    let pawn_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh2/Primitive0");
    let knight_1_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh3/Primitive0");
    let knight_2_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh4/Primitive0");
    let rook_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh5/Primitive0");
    let bishop_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh6/Primitive0");
    let queen_handle: Handle<Mesh> =
        asset_server.load("models/chess_kit/pieces.glb#Mesh7/Primitive0");

    let white_material = materials.add(Color::rgb(1.0, 0.8, 0.8).into());
    let black_material = materials.add(Color::rgb(0.0, 0.2, 0.2).into());

    // Rook
    spawn_piece!(
        commands,
        (0, 0),
        Vec3::new(-0.1, 0.0, 1.8),
        PieceType::Rook,
        PieceColor::White,
        white_material.clone(),
        rook_handle.clone(),
    );

    // Knight
    spawn_piece!(
        commands,
        (0, 1),
        Vec3::new(-0.2, 0.0, 0.9),
        PieceType::Knight,
        PieceColor::White,
        white_material.clone(),
        knight_1_handle.clone(),
        knight_2_handle.clone(),
    );

    // Bishop
    spawn_piece!(
        commands,
        (0, 2),
        Vec3::new(-0.1, 0.0, 0.0),
        PieceType::Bishop,
        PieceColor::White,
        white_material.clone(),
        bishop_handle.clone(),
    );

    // Queen
    spawn_piece!(
        commands,
        (0, 3),
        Vec3::new(-0.2, 0.0, -0.95),
        PieceType::Queen,
        PieceColor::White,
        white_material.clone(),
        queen_handle.clone(),
    );

    // Spawn King
    spawn_piece!(
        commands,
        (0, 4),
        Vec3::new(-0.2, 0.0, -1.9),
        PieceType::King,
        PieceColor::White,
        white_material.clone(),
        king_handle.clone(),
        king_cross_handle.clone(),
    );

    // Bishop
    spawn_piece!(
        commands,
        (0, 5),
        Vec3::new(-0.1, 0.0, 0.0),
        PieceType::Bishop,
        PieceColor::White,
        white_material.clone(),
        bishop_handle.clone(),
    );

    // Knight
    spawn_piece!(
        commands,
        (0, 6),
        Vec3::new(-0.2, 0.0, 0.9),
        PieceType::Knight,
        PieceColor::White,
        white_material.clone(),
        knight_1_handle.clone(),
        knight_2_handle.clone(),
    );

    // Rook
    spawn_piece!(
        commands,
        (0, 7),
        Vec3::new(-0.1, 0.0, 1.8),
        PieceType::Rook,
        PieceColor::White,
        white_material.clone(),
        rook_handle.clone(),
    );

    // Pawn
    for i in 0..8 {
        spawn_piece!(
            commands,
            (1, i),
            Vec3::new(-0.2, 0.0, 2.6),
            PieceType::Pawn,
            PieceColor::White,
            white_material.clone(),
            pawn_handle.clone(),
        );
    }

    // Rook
    spawn_piece!(
        commands,
        (7, 0),
        Vec3::new(-0.1, 0.0, 1.8),
        PieceType::Rook,
        PieceColor::Black,
        black_material.clone(),
        rook_handle.clone(),
    );

    // Knight
    spawn_piece!(
        commands,
        (7, 1),
        Vec3::new(-0.2, 0.0, 0.9),
        PieceType::Knight,
        PieceColor::Black,
        black_material.clone(),
        knight_1_handle.clone(),
        knight_2_handle.clone(),
    );

    // Bishop
    spawn_piece!(
        commands,
        (7, 2),
        Vec3::new(-0.1, 0.0, 0.0),
        PieceType::Bishop,
        PieceColor::Black,
        black_material.clone(),
        bishop_handle.clone(),
    );

    // Queen
    spawn_piece!(
        commands,
        (7, 3),
        Vec3::new(-0.2, 0.0, -0.95),
        PieceType::Queen,
        PieceColor::Black,
        black_material.clone(),
        queen_handle.clone(),
    );

    // Spawn King
    spawn_piece!(
        commands,
        (7, 4),
        Vec3::new(-0.2, 0.0, -1.9),
        PieceType::King,
        PieceColor::Black,
        black_material.clone(),
        king_handle.clone(),
        king_cross_handle.clone(),
    );

    // Bishop
    spawn_piece!(
        commands,
        (7, 5),
        Vec3::new(-0.1, 0.0, 0.0),
        PieceType::Bishop,
        PieceColor::Black,
        black_material.clone(),
        bishop_handle.clone(),
    );

    // Knight
    spawn_piece!(
        commands,
        (7, 6),
        Vec3::new(-0.2, 0.0, 0.9),
        PieceType::Knight,
        PieceColor::Black,
        black_material.clone(),
        knight_1_handle.clone(),
        knight_2_handle.clone(),
    );

    // Rook
    spawn_piece!(
        commands,
        (7, 7),
        Vec3::new(-0.1, 0.0, 1.8),
        PieceType::Rook,
        PieceColor::Black,
        black_material.clone(),
        rook_handle.clone(),
    );

    // Pawn
    for i in 0..8 {
        spawn_piece!(
            commands,
            (6, i),
            Vec3::new(-0.2, 0.0, 2.6),
            PieceType::Pawn,
            PieceColor::Black,
            black_material.clone(),
            pawn_handle.clone(),
        );
    }
}

fn move_pieces(time: Res<Time>, mut query: Query<(&mut Transform, &Piece)>) {
    for (mut transform, piece) in query.iter_mut() {
        let direction = Vec3::new(piece.x as f32, 0.0, piece.y as f32) - transform.translation;
        if direction.length() > 0.1 {
            transform.translation += direction.normalize() * time.delta_seconds();
        }
    }
}
