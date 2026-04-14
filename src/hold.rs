//! Functionality for holding a piece

use crate::Game;
use crate::ui::{BG_COLOR, PADDING};

use super::board::*;
use super::data::*;
use bevy::prelude::*;

/// Whether this tetromino is the one being held
#[derive(Component, Copy, Clone)]
pub struct Hold;

/// Whether the player requested a hold swap this frame.
#[derive(Resource, Default)]
pub struct PendingHold(pub bool);

fn queue_hold_input(
    // Read keyboard transitions for the current schedule.
    keyboard: Res<ButtonInput<KeyCode>>,
    // Store a pending hold request until gameplay logic consumes it.
    mut pending_hold: ResMut<PendingHold>,
) {
    // Remember the X press so either the fixed-step or frame-step hold system can use it.
    if keyboard.just_pressed(KeyCode::KeyX) {
        pending_hold.0 = true;
    }
}

/// Swap the current piece and the piece in the hold window on user input.
///
/// If no piece is held, then take the next piece as the active piece and move
/// the current piece to the hold window.
///
/// This system also has to make sure that the swap is legal and kick the piece
/// up by up to 4 times until the swap is legal.  If that is not possible, then
/// abort the swap.
pub fn swap_hold(
    // Read and clear the pending hold request.
    mut pending_hold: ResMut<PendingHold>,
    // Use commands to despawn and respawn the logical tetromino entities.
    mut commands: Commands,
    // Access the bag because first hold consumes the current next piece.
    mut state: ResMut<GameState>,
    // Read the current active piece and its entity id.
    active: Query<(Entity, &Tetromino), With<Active>>,
    // Read the current held piece and its entity id, if it exists.
    held: Query<(Entity, &Tetromino), With<Hold>>,
    // Read the logical Next entity and value.
    // The value matters because first hold uses the currently displayed next piece.
    next_tetrominoes: Query<(Entity, &Tetromino), With<Next>>,
    // Access obstacles so we can reject illegal swap results.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Only run when a queued hold input is waiting to be processed.
    if !pending_hold.0 {
        return;
    }
    // Consume the queued request so we do not swap twice in one frame.
    pending_hold.0 = false;

    // Stop immediately when there is no active piece.
    let Ok((active_entity, active_piece)) = active.single() else {
        return;
    };
    let active_piece = *active_piece;

    let to_hold = |mut tetromino: Tetromino| {
        // Detect the I piece by checking whether all cells are on one row or column.
        let is_i = tetromino.cells.iter().all(|cell| cell.0 == tetromino.cells[0].0)
            || tetromino.cells.iter().all(|cell| cell.1 == tetromino.cells[0].1);
        // I and O pieces use the centered `(2.5, 2.5)` preview position.
        let target_center = if tetromino.is_o() || is_i {
            (2.5, 2.5)
        } else {
            // The other pieces use the `(2.0, 2.0)` preview position.
            (2.0, 2.0)
        };
        // Translate the piece from board space into hold-window space.
        tetromino.shift(
            (target_center.0 - tetromino.center.0).round() as i32,
            (target_center.1 - tetromino.center.1).round() as i32,
        );
        tetromino
    };

    let to_board = |mut tetromino: Tetromino| {
        // Detect whether the I piece is vertical in the hold window.
        let vertical_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.0 == tetromino.cells[0].0);
        // Detect whether the I piece is horizontal in the hold window.
        let horizontal_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.1 == tetromino.cells[0].1);
        // Different I orientations need different y-shifts to line up correctly on the board.
        let y_shift = if vertical_i {
            13
        } else if horizontal_i {
            15
        } else {
            // All non-I pieces use the normal hold-to-board y shift.
            14
        };
        // Translate the held piece into board coordinates.
        tetromino.shift(2, y_shift);
        tetromino
    };

    let try_resolve = |mut tetromino: Tetromino, obstacles: &mut Query<&Block, With<Obstacle>>| {
        // Try the original placement plus up to four upward kicks.
        for _ in 0..=4 {
            if !crate::there_is_collision(&tetromino, obstacles.reborrow()) {
                return Some(tetromino);
            }
            // Move the candidate up by one row and try again.
            tetromino.shift(0, 1);
        }
        // Give up if all five attempts are illegal.
        None
    };

    if let Ok((held_entity, held_piece)) = held.single() {
        // Start from the currently held tetromino value.
        let candidate = *held_piece;
        // Detect whether the held piece is a vertical I.
        let vertical_i = candidate
            .cells
            .iter()
            .all(|cell| cell.0 == candidate.cells[0].0);
        // Detect whether the held piece is a horizontal I.
        let horizontal_i = candidate
            .cells
            .iter()
            .all(|cell| cell.1 == candidate.cells[0].1);
        // Choose the correct board-placement rule for this shape.
        let candidate = if vertical_i || horizontal_i {
            if active_piece.is_o() {
                // When the current active piece is O, align the held I using rounded centers.
                // This matches the recorded swap where I takes over the O piece's board position.
                let mut candidate = candidate;
                candidate.shift(
                    (active_piece.center.0 - candidate.center.0).round() as i32,
                    (active_piece.center.1 - candidate.center.1).round() as i32,
                );
                candidate
            } else {
                to_board(candidate)
            }
        } else if candidate.is_o() {
            // O pieces use rounded center alignment.
            let mut candidate = candidate;
            candidate.shift(
                (active_piece.center.0 - candidate.center.0).round() as i32,
                (active_piece.center.1 - candidate.center.1).round() as i32,
            );
            candidate
        } else {
            // The other pieces use floored center alignment.
            let mut candidate = candidate;
            candidate.shift(
                (active_piece.center.0 - candidate.center.0).floor() as i32,
                (active_piece.center.1 - candidate.center.1).floor() as i32,
            );
            candidate
        };
        // Abort the swap if the held piece still cannot be placed legally.
        let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
            return;
        };

        // Replace the active/held pair with the swapped values.
        commands.entity(active_entity).despawn();
        commands.entity(held_entity).despawn();
        commands.spawn((to_hold(active_piece), Hold));
        commands.spawn((candidate, Active));
        return;
    }

    let Ok((_, next_piece)) = next_tetrominoes.single() else {
        return;
    };
    // Start from the logical Next tetromino already shown by the game.
    let candidate = *next_piece;
    // Detect whether the next piece is a vertical I.
    let vertical_i = candidate
        .cells
        .iter()
        .all(|cell| cell.0 == candidate.cells[0].0);
    // Detect whether the next piece is a horizontal I.
    let horizontal_i = candidate
        .cells
        .iter()
        .all(|cell| cell.1 == candidate.cells[0].1);
    // Choose the validated placement rule for the first-hold replacement piece.
    let candidate = if vertical_i || horizontal_i {
        // The preview I piece needs a one-cell lift before hold-to-board conversion.
        let mut candidate = candidate;
        candidate.shift(0, 1);
        to_board(candidate)
    } else if candidate.is_o() {
        // O pieces use rounded center alignment.
        let mut candidate = candidate;
        candidate.shift(
            (active_piece.center.0 - candidate.center.0).round() as i32,
            (active_piece.center.1 - candidate.center.1).round() as i32,
        );
        candidate
    } else {
        // The other pieces use floored center alignment.
        let mut candidate = candidate;
        candidate.shift(
            (active_piece.center.0 - candidate.center.0).floor() as i32,
            (active_piece.center.1 - candidate.center.1).floor() as i32,
        );
        candidate
    };

    // Abort before consuming the bag if the placement is illegal.
    let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
        return;
    };

    // Consume the bag only after the replacement piece is confirmed legal.
    state.bag.next_tetromino();

    // Move the active piece into hold and make the candidate active.
    commands.entity(active_entity).despawn();
    commands.spawn((to_hold(active_piece), Hold));
    commands.spawn((candidate, Active));

    // Refresh the logical Next piece because the bag front changed.
    for (entity, _) in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    // Rebuild the preview from the new bag front and move it into preview coordinates.
    let mut next_piece = state.bag.peek();
    next_piece.shift(2, 2);
    commands.spawn((next_piece, Next));
}

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

pub struct HoldPlugin;

impl Plugin for HoldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingHold>()
            .add_systems(Startup, setup_hold_window.in_set(Game))
            .add_systems(Update, queue_hold_input.in_set(Game))
            .add_systems(
                FixedUpdate,
                // Read hold input during the fixed-step gameplay loop so it can affect
                // the currently active piece before lock/spawn logic runs.
                swap_hold.in_set(Game).before(crate::board::gravity),
            )
            // Fall back to frame-step processing when no fixed-step run happened this frame.
            .add_systems(PostUpdate, swap_hold.in_set(Game))
            .add_systems(PostUpdate, redraw_side_board::<Hold>.in_set(Game));
    }
}
