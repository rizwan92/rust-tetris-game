//! Functionality for holding a piece

use crate::Game;
use crate::ui::{BG_COLOR, PADDING};

use super::board::*;
use super::data::*;
use bevy::prelude::*;

/// Whether this tetromino is the one being held
#[derive(Component, Copy, Clone)]
pub struct Hold;

/// NEW IMPLEMENTATION: Recover the canonical spawn tetromino that matches a
/// gameplay color.
///
/// Simple example:
/// if the current piece is blue, this helper returns the normal blue `T`
/// tetromino shape.
fn canonical_from_color(color: Color) -> Tetromino {
    ALL_TETROMINO_TYPES
        .into_iter()
        .map(get_tetromino)
        .find(|tetromino| tetromino.color == color)
        .expect("every gameplay tetromino color should map to a canonical piece")
}

/// NEW IMPLEMENTATION: Place a tetromino into the 5x5 hold preview window.
///
/// Simple example:
/// if the player holds a long `I` piece, this helper moves it into the middle
/// of the hold box so it looks centered.
fn move_to_hold_window(mut tetromino: Tetromino) -> Tetromino {
    let canonical = canonical_from_color(tetromino.color);
    // NEW IMPLEMENTATION: the `I` piece has a different center, so it needs a
    // slightly different preview center to look correct.
    let preview_center = if canonical.center == (0.5, -0.5) {
        (2.5, 2.5)
    } else {
        (canonical.center.0 + 2.0, canonical.center.1 + 2.0)
    };

    // NEW IMPLEMENTATION: turn the preview center difference into a simple grid
    // shift.
    let dx = (preview_center.0 - tetromino.center().0).round() as i32;
    let dy = (preview_center.1 - tetromino.center().1).round() as i32;
    tetromino.shift(dx, dy);
    tetromino
}

/// NEW IMPLEMENTATION: Place a tetromino at its normal spawn row on the main
/// board.
///
/// Simple example:
/// when a held piece comes back to the board, it should start where a normal
/// newly spawned piece would start.
fn move_to_board_spawn(mut tetromino: Tetromino) -> Tetromino {
    if tetromino.center == (0.5, -0.5) {
        tetromino.shift(4, 19);
    } else {
        tetromino.shift(4, 18);
    }
    tetromino
}

/// NEW IMPLEMENTATION: Reuse the outgoing active piece offset when a hold swap
/// spawns a new piece.
///
/// Simple example:
/// if the current piece already moved one column left, the piece coming out of
/// hold should appear one column left too.
fn move_to_active_anchor(active_piece: Tetromino, mut tetromino: Tetromino) -> Tetromino {
    let active_spawn = move_to_board_spawn(canonical_from_color(active_piece.color));
    // NEW IMPLEMENTATION: measure how far the current active piece moved away
    // from its normal spawn point.
    let dx = (active_piece.center().0 - active_spawn.center().0).round() as i32;
    let dy = (active_piece.center().1 - active_spawn.center().1).round() as i32;

    let incoming_spawn = move_to_board_spawn(canonical_from_color(tetromino.color));
    // NEW IMPLEMENTATION: first move the incoming piece to its own normal spawn
    // point.
    let center_dx = (incoming_spawn.center().0 - tetromino.center().0).round() as i32;
    let center_dy = (incoming_spawn.center().1 - tetromino.center().1).round() as i32;
    tetromino.shift(center_dx, center_dy);
    // NEW IMPLEMENTATION: then copy the old active piece offset.
    tetromino.shift(dx, dy);
    tetromino
}

/// NEW IMPLEMENTATION: Try the swapped-in hold piece at its spawn row, kicking
/// it up if needed.
///
/// Simple example:
/// if the held piece overlaps the stack by one row, we try the same piece one
/// row higher, then two rows higher, and so on.
fn resolve_hold_swap(
    mut tetromino: Tetromino,
    obstacles: &mut Query<&Block, With<Obstacle>>,
) -> Option<Tetromino> {
    for attempt in 0..=4 {
        // NEW IMPLEMENTATION: stop at the first legal position.
        if !crate::there_is_collision(&tetromino, obstacles.reborrow()) {
            return Some(tetromino);
        }

        if attempt < 4 {
            // NEW IMPLEMENTATION: kick the piece up by one row and try again.
            tetromino.shift(0, 1);
        }
    }

    None
}

/// NEW IMPLEMENTATION: Swap the current piece and the piece in the hold window
/// on user input.
///
/// If no piece is held, then take the next piece as the active piece and move
/// the current piece to the hold window.
///
/// This system also has to make sure that the swap is legal and kick the piece
/// up by up to 4 times until the swap is legal.  If that is not possible, then
/// abort the swap.
///
/// Simple example:
/// press `X` once:
/// - current active piece goes into the hold box
/// - held piece, or the next preview piece, becomes the new active piece
#[allow(clippy::too_many_arguments)]
pub fn swap_hold(
    mut commands: Commands,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    active_tetrominoes: Query<(Entity, &Tetromino), With<Active>>,
    held_tetrominoes: Query<(Entity, &Tetromino), With<Hold>>,
    next_tetrominoes: Query<Entity, (With<Next>, With<Tetromino>)>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // NEW IMPLEMENTATION: keep the Bevy system small by letting helper
    // functions do the placement math.
    if !keyboard.just_pressed(KeyCode::KeyX) {
        return;
    }
    keyboard.clear_just_pressed(KeyCode::KeyX);

    let Ok((active_entity, active_piece)) = active_tetrominoes.single() else {
        return;
    };

    // NEW IMPLEMENTATION: the old active piece always becomes the new hold
    // preview.
    let new_hold_piece = move_to_hold_window(*active_piece);

    let held_piece = held_tetrominoes.iter().next();

    // NEW IMPLEMENTATION: if hold is empty, we have to take the preview piece
    // from the bag.
    let consume_next_piece = held_piece.is_none();
    let swapped_in_canonical = held_piece
        .as_ref()
        .map(|(_, tetromino)| **tetromino)
        .unwrap_or_else(|| state.bag.peek());

    // NEW IMPLEMENTATION: place the incoming piece near the old active piece's
    // current board position.
    let candidate_piece = move_to_active_anchor(*active_piece, swapped_in_canonical);

    let Some(new_active_piece) = resolve_hold_swap(candidate_piece, &mut obstacles) else {
        return;
    };

    commands.entity(active_entity).despawn();

    if let Some((held_entity, _)) = held_piece {
        commands.entity(held_entity).despawn();
    }

    // NEW IMPLEMENTATION: delete only the logical next preview piece, not the
    // preview board tiles.
    for entity in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    if consume_next_piece {
        let _ = state.bag.next_tetromino();
    }

    keyboard.clear_just_pressed(KeyCode::ArrowDown);
    keyboard.clear_just_pressed(KeyCode::ArrowLeft);
    keyboard.clear_just_pressed(KeyCode::ArrowRight);
    keyboard.clear_just_pressed(KeyCode::ArrowUp);
    keyboard.clear_just_pressed(KeyCode::Space);
    // NEW IMPLEMENTATION: a successful swap should reset the old lock delay.
    lockdown.reset();

    commands.spawn((new_active_piece, Active));
    commands.spawn((new_hold_piece, Hold));

    let mut next_piece = state.bag.peek();
    next_piece.shift(2, 2);
    commands.spawn((next_piece, Next));
}

/// NEW IMPLEMENTATION: Create the hold preview window on the side of the board.
pub fn setup_hold_window(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    let mesh = meshes.add(Rectangle::new(TILE_SIDE_LEN, TILE_SIDE_LEN));
    let material = materials.add(BG_COLOR);

    crate::board::spawn_side_window(
        Transform::from_xyz(
            (BOARD_WIDTH + 5) as f32 * TILE_SIDE_LEN * 0.5 + PADDING,
            -window_height() * 0.5 + 5.0 * TILE_SIDE_LEN * 0.5 + PADDING,
            0.0,
        ),
        mesh.clone(),
        material.clone(),
        &mut commands,
        "Hold",
        Hold,
    );
}

/// NEW IMPLEMENTATION: Plugin that adds hold input, hold swapping, and the
/// hold side window.
pub struct HoldPlugin;

impl Plugin for HoldPlugin {
    fn build(&self, app: &mut App) {
        // NEW IMPLEMENTATION: keep the hold systems in the same general starter
        // locations so students can still follow the file easily.
        app.add_systems(Startup, setup_hold_window.in_set(Game))
            .add_systems(
                Update,
                (
                    swap_hold.before(crate::board::handle_user_input),
                    redraw_side_board::<Hold>,
                )
                    .in_set(Game),
            );
    }
}
