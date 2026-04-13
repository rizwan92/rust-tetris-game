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
pub fn swap_hold(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut commands: Commands,
    mut state: ResMut<GameState>,
    active: Query<(Entity, &Tetromino), With<Active>>,
    held: Query<(Entity, &Tetromino), With<Hold>>,
    next_tetrominoes: Query<(Entity, &Tetromino), With<Next>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    if !keyboard.just_pressed(KeyCode::KeyX) {
        return;
    }

    let Ok((active_entity, active_piece)) = active.single() else {
        return;
    };
    let active_piece = *active_piece;

    let to_hold = |mut tetromino: Tetromino| {
        let is_i = tetromino.cells.iter().all(|cell| cell.0 == tetromino.cells[0].0)
            || tetromino.cells.iter().all(|cell| cell.1 == tetromino.cells[0].1);
        let target_center = if tetromino.is_o() || is_i {
            (2.5, 2.5)
        } else {
            (2.0, 2.0)
        };
        tetromino.shift(
            (target_center.0 - tetromino.center.0).round() as i32,
            (target_center.1 - tetromino.center.1).round() as i32,
        );
        tetromino
    };

    let to_board = |mut tetromino: Tetromino| {
        let vertical_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.0 == tetromino.cells[0].0);
        let horizontal_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.1 == tetromino.cells[0].1);
        let y_shift = if vertical_i {
            13
        } else if horizontal_i {
            15
        } else {
            14
        };
        tetromino.shift(2, y_shift);
        tetromino
    };

    let try_resolve = |mut tetromino: Tetromino, obstacles: &mut Query<&Block, With<Obstacle>>| {
        for _ in 0..=4 {
            if !crate::there_is_collision(&tetromino, obstacles.reborrow()) {
                return Some(tetromino);
            }
            tetromino.shift(0, 1);
        }
        None
    };

    if let Ok((held_entity, held_piece)) = held.single() {
        let candidate = *held_piece;
        let vertical_i = candidate
            .cells
            .iter()
            .all(|cell| cell.0 == candidate.cells[0].0);
        let horizontal_i = candidate
            .cells
            .iter()
            .all(|cell| cell.1 == candidate.cells[0].1);
        let candidate = if vertical_i || horizontal_i {
            to_board(candidate)
        } else if candidate.is_o() {
            let mut candidate = candidate;
            candidate.shift(
                (active_piece.center.0 - candidate.center.0).round() as i32,
                (active_piece.center.1 - candidate.center.1).round() as i32,
            );
            candidate
        } else {
            let mut candidate = candidate;
            candidate.shift(
                (active_piece.center.0 - candidate.center.0).floor() as i32,
                (active_piece.center.1 - candidate.center.1).floor() as i32,
            );
            candidate
        };
        let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
            return;
        };

        commands.entity(active_entity).despawn();
        commands.entity(held_entity).despawn();
        commands.spawn((to_hold(active_piece), Hold));
        commands.spawn((candidate, Active));
        return;
    }

    let Ok((_, next_piece)) = next_tetrominoes.single() else {
        return;
    };
    let candidate = *next_piece;
    let vertical_i = candidate
        .cells
        .iter()
        .all(|cell| cell.0 == candidate.cells[0].0);
    let horizontal_i = candidate
        .cells
        .iter()
        .all(|cell| cell.1 == candidate.cells[0].1);
    let candidate = if vertical_i || horizontal_i {
        let mut candidate = candidate;
        candidate.shift(0, 1);
        to_board(candidate)
    } else if candidate.is_o() {
        let mut candidate = candidate;
        candidate.shift(
            (active_piece.center.0 - candidate.center.0).round() as i32,
            (active_piece.center.1 - candidate.center.1).round() as i32,
        );
        candidate
    } else {
        let mut candidate = candidate;
        candidate.shift(
            (active_piece.center.0 - candidate.center.0).floor() as i32,
            (active_piece.center.1 - candidate.center.1).floor() as i32,
        );
        candidate
    };

    let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
        return;
    };

    state.bag.next_tetromino();

    commands.entity(active_entity).despawn();
    commands.spawn((to_hold(active_piece), Hold));
    commands.spawn((candidate, Active));

    for (entity, _) in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

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
        app.add_systems(Startup, setup_hold_window.in_set(Game))
            .add_systems(PostUpdate, swap_hold.in_set(Game))
            .add_systems(PostUpdate, redraw_side_board::<Hold>.in_set(Game));
    }
}
