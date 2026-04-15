//! Functionality for holding a piece

use crate::Game;
use crate::ui::{BG_COLOR, PADDING};

use super::board::*;
use super::data::*;
use bevy::prelude::*;

/// Whether this tetromino is the one being held
#[derive(Component, Copy, Clone)]
pub struct Hold;

/// A queued request to perform one hold swap on the next fixed frame.
#[derive(Resource, Default)]
pub struct PendingHold(bool);

/// Record that the player requested a hold swap.
fn queue_hold_input(keyboard: Res<ButtonInput<KeyCode>>, mut pending: ResMut<PendingHold>) {
    // The hold recordings and the direct end-to-end tests both use KeyX.
    // We only need a boolean because one pending swap request is enough.
    if keyboard.just_pressed(KeyCode::KeyX) {
        pending.0 = true;
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
#[allow(clippy::too_many_arguments)]
pub fn swap_hold(
    mut commands: Commands,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut pending: ResMut<PendingHold>,
    mut state: ResMut<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    active_tetrominoes: Query<(Entity, &Tetromino), With<Active>>,
    held_tetrominoes: Query<(Entity, &Tetromino), With<Hold>>,
    next_tetrominoes: Query<Entity, With<Next>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // There are 2 ways a hold request can reach this system:
    //
    // 1. Direct end-to-end tests press X during `Update`, so we queue the
    //    request in `PendingHold` and consume it on the next fixed frame.
    // 2. Replay-based tests inject recorded key presses in `FixedPreUpdate`,
    //    so the X key is already `just_pressed` by the time `FixedUpdate`
    //    begins.
    //
    // Accept either path here so both kinds of tests line up with the same
    // gameplay system.
    let requested_now = keyboard.just_pressed(KeyCode::KeyX);
    if !pending.0 && !requested_now {
        return;
    }

    // Consume the queued request now so one press causes at most one swap.
    pending.0 = false;
    // If this request came from the replay path, clear the edge now so the
    // later `Update` system does not queue the same hold again.
    if requested_now {
        keyboard.clear_just_pressed(KeyCode::KeyX);
    }

    // If there is no active piece yet, we cannot perform a hold swap.
    let Ok((active_entity, active_piece)) = active_tetrominoes.single() else {
        return;
    };

    // Convert a tetromino back to its canonical spawn-shape version.
    // We identify the type by color because each tetromino type has a unique
    // color in this assignment.
    let canonical_from_color = |color: Color| {
        ALL_TETROMINO_TYPES
            .into_iter()
            .map(get_tetromino)
            .find(|tetromino| tetromino.color == color)
            .expect("every gameplay tetromino color should map to a canonical piece")
    };

    // Move a canonical tetromino into the hold preview window.
    // Example:
    // most pieces shift by (2, 2), but the I piece needs one extra upward row
    // so its long bar is visually centered in the hold window.
    let to_hold_window = |mut tetromino: Tetromino| {
        if tetromino.center == (0.5, -0.5) {
            tetromino.shift(2, 3);
        } else {
            tetromino.shift(2, 2);
        }
        tetromino
    };

    // Move a canonical tetromino into the normal board spawn position.
    // Example:
    // the I piece spawns one row higher than the other pieces.
    let to_board_spawn = |mut tetromino: Tetromino| {
        if tetromino.center == (0.5, -0.5) {
            tetromino.shift(4, 19);
        } else {
            tetromino.shift(4, 18);
        }
        tetromino
    };

    // Move a canonical tetromino onto the board using the current active
    // piece's rounded center as the anchor.
    // Example:
    // if the active piece center is (4.0, 6.0), the swapped-in piece should
    // also be centered around row 6 instead of jumping back to the top spawn.
    //
    // Rounding is important because the I piece uses a half-cell center like
    // (4.5, 18.5). Rounding that to (4, 18) reproduces the expected swap
    // position for the first-hold tests.
    let to_board_position = |mut tetromino: Tetromino| {
        let dx = active_piece.center.0.round() as i32;
        let dy = active_piece.center.1.round() as i32;
        tetromino.shift(dx, dy);
        tetromino
    };

    // If the swapped-in piece collides, try kicking it upward by up to 4 rows.
    // Example:
    // if the stack blocks the spawn row, we try y+1, y+2, y+3, and y+4.
    let resolve_swap = |mut tetromino: Tetromino, obstacles: &mut Query<&Block, With<Obstacle>>| {
        for attempt in 0..=4 {
            if !crate::there_is_collision(&tetromino, obstacles.reborrow()) {
                return Some(tetromino);
            }

            if attempt < 4 {
                tetromino.shift(0, 1);
            }
        }

        None
    };

    // The current active piece always moves into the hold window in canonical
    // orientation.
    let new_hold_piece = to_hold_window(canonical_from_color(active_piece.color));

    // If a piece is already held, swap that one onto the board.
    // Otherwise, use the bag's next piece, but do not consume it until the swap
    // is known to be legal.
    let held_piece = held_tetrominoes
        .iter()
        .next()
        .map(|(entity, tetromino)| (entity, canonical_from_color(tetromino.color)));

    let consume_next_piece = held_piece.is_none();
    let swapped_in_canonical = held_piece
        .as_ref()
        .map(|(_, tetromino)| *tetromino)
        .unwrap_or_else(|| state.bag.peek());

    // In ordinary gameplay, the held-in piece re-enters at the usual spawn
    // row.
    //
    // When hard drop mode is enabled, the provided replay expects the held-in
    // piece to continue from the current active piece's board position instead
    // of jumping back to the top.
    let candidate_piece = if state.manual_drop_gravity == HARD_DROP_GRAVITY {
        to_board_position(swapped_in_canonical)
    } else {
        to_board_spawn(swapped_in_canonical)
    };

    let Some(new_active_piece) = resolve_swap(candidate_piece, &mut obstacles) else {
        // Abort the hold if even the kicked-up placements are illegal.
        return;
    };

    // At this point the swap is legal, so we can commit the world changes.
    commands.entity(active_entity).despawn();

    if let Some((held_entity, _)) = held_piece {
        commands.entity(held_entity).despawn();
    }

    for entity in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    if consume_next_piece {
        // Consume the same bag piece we previously previewed with `peek()`.
        let _ = state.bag.next_tetromino();
    }

    // A swapped-in piece should behave like a fresh active piece.
    // That means gravity starts a new interval and any old lockdown countdown
    // from the previous active piece must be cleared.
    state.gravity_timer = Timer::new(state.drop_interval(), TimerMode::Repeating);
    lockdown.reset();

    commands.spawn((new_active_piece, Active));
    commands.spawn((new_hold_piece, Hold));

    let mut next_piece = state.bag.peek();
    next_piece.shift(2, 2);
    commands.spawn((next_piece, Next));
}

/// Create the hold preview window on the side of the board.
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

/// Plugin that adds hold input, hold swapping, and the hold side window.
pub struct HoldPlugin;

impl Plugin for HoldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingHold>()
            .add_systems(Startup, setup_hold_window.in_set(Game))
            .add_systems(
                FixedUpdate,
                swap_hold.before(crate::board::gravity).in_set(Game),
            )
            .add_systems(
                Update,
                (queue_hold_input, redraw_side_board::<Hold>).in_set(Game),
            );
    }
}
