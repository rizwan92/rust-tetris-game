//! Functionality for holding a piece

use crate::Game;
use crate::ui::{BG_COLOR, PADDING};

use super::board::*;
use super::data::*;
use bevy::prelude::*;

/// Whether this tetromino is the one being held
#[derive(Component, Copy, Clone)]
pub struct Hold;

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
    mut state: ResMut<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    active_tetrominoes: Query<(Entity, &Tetromino), With<Active>>,
    held_tetrominoes: Query<(Entity, &Tetromino), With<Hold>>,
    next_tetrominoes: Query<Entity, With<Next>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // We intentionally react to the X edge directly here.
    //
    // Why run this system in both `FixedUpdate` and `Update`?
    // - Replay tests inject inputs in `FixedPreUpdate`, so `FixedUpdate`
    //   should be able to consume the hold request in the same fixed frame.
    // - Direct headless tests press X during `Update`, and some of them do not
    //   guarantee that another fixed step will happen immediately afterward.
    //
    // Clearing the `just_pressed` edge after we consume it prevents a double
    // swap when both schedules run in the same app update.
    if !keyboard.just_pressed(KeyCode::KeyX) {
        return;
    }
    keyboard.clear_just_pressed(KeyCode::KeyX);

    // If there is no active piece yet, we cannot perform a hold swap.
    let Ok((active_entity, active_piece)) = active_tetrominoes.single() else {
        return;
    };

    // Convert a tetromino color back to its canonical type.
    //
    // We still need this for bag-spawn calculations because the bag stores
    // canonical tetrominoes, and because each gameplay color uniquely
    // identifies one tetromino type in this assignment.
    let canonical_from_color = |color: Color| {
        ALL_TETROMINO_TYPES
            .into_iter()
            .map(get_tetromino)
            .find(|tetromino| tetromino.color == color)
            .expect("every gameplay tetromino color should map to a canonical piece")
    };

    // Move a tetromino into the hold preview window while preserving its
    // current rotation.
    //
    // Example:
    // if a vertical I piece is held, the replay recordings expect the hold
    // window to keep it vertical instead of snapping it back to the flat
    // horizontal spawn pose.
    let to_hold_window = |mut tetromino: Tetromino| {
        let preview_center = {
            let canonical = canonical_from_color(tetromino.color);
            if canonical.center == (0.5, -0.5) {
                (2.5, 2.5)
            } else {
                (canonical.center.0 + 2.0, canonical.center.1 + 2.0)
            }
        };

        let dx = (preview_center.0 - tetromino.center().0).round() as i32;
        let dy = (preview_center.1 - tetromino.center().1).round() as i32;
        tetromino.shift(dx, dy);
        tetromino
    };

    // Move a canonical tetromino onto the board while preserving how far the
    // outgoing piece had already drifted from its own spawn location.
    //
    // Example:
    // if a falling I piece has moved down by 2 rows from its normal spawn,
    // the swapped-in S piece should also appear 2 rows below the S spawn row.
    let to_active_anchor = |mut tetromino: Tetromino| {
        let spawn_on_board = |mut tetromino: Tetromino| {
            if tetromino.center == (0.5, -0.5) {
                tetromino.shift(4, 19);
            } else {
                tetromino.shift(4, 18);
            }
            tetromino
        };

        let active_spawn = spawn_on_board(canonical_from_color(active_piece.color));
        let dx = (active_piece.center().0 - active_spawn.center().0).round() as i32;
        let dy = (active_piece.center().1 - active_spawn.center().1).round() as i32;

        let incoming_spawn = spawn_on_board(canonical_from_color(tetromino.color));
        let center_dx = (incoming_spawn.center().0 - tetromino.center().0).round() as i32;
        let center_dy = (incoming_spawn.center().1 - tetromino.center().1).round() as i32;
        tetromino.shift(center_dx, center_dy);
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

    // The replay recordings keep whatever rotation the player was actually
    // holding at the moment they pressed X, so preserve the outgoing active
    // piece shape here instead of snapping it back to the canonical pose.
    let new_hold_piece = to_hold_window(*active_piece);

    // If a piece is already held, swap that one onto the board.
    // Otherwise, use the bag's next piece, but do not consume it until the swap
    // is known to be legal.
    let held_piece = held_tetrominoes.iter().next();

    let consume_next_piece = held_piece.is_none();
    let swapped_in_canonical = held_piece
        .as_ref()
        .map(|(_, tetromino)| **tetromino)
        .unwrap_or_else(|| state.bag.peek());

    // The incoming hold piece should reuse the outgoing active piece's board
    // anchor instead of jumping back to the normal spawn row.
    let candidate_piece = to_active_anchor(swapped_in_canonical);

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

    // Clear any leftover input edge from the outgoing piece before the swapped
    // piece becomes active.
    //
    // Example:
    // if X was pressed during the same app update, the incoming held piece
    // must not also inherit an ArrowUp or ArrowLeft edge from the previous one.
    keyboard.clear_just_pressed(KeyCode::ArrowDown);
    keyboard.clear_just_pressed(KeyCode::ArrowLeft);
    keyboard.clear_just_pressed(KeyCode::ArrowRight);
    keyboard.clear_just_pressed(KeyCode::ArrowUp);
    keyboard.clear_just_pressed(KeyCode::Space);
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
