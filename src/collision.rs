//! Collision detection implementation

use super::board::Block;
use super::data::*;
use bevy::prelude::*;
#[cfg(feature = "score")]
use crate::score::LinesCleared;

/// Return whether the given tetromino collides with any of the obstacles or it
/// is out of bounds.
pub fn there_is_collision(
    tetromino: &Tetromino,
    obstacles: Query<&Block, With<Obstacle>>,
) -> bool {
    if !tetromino.in_bounds() {
        return true;
    }

    tetromino
        .cells()
        .iter()
        .any(|cell| obstacles.iter().any(|block| block.cell == *cell))
}

/// A system to detect the full lines (lines containing only obstacles and no
/// empty space) and to delete them.
///
/// After the lines are deleted, any obstacles above the deleted lines should be
/// moved down using naive gravity (the obstacles move down only by the number
/// of lines below them that are deleted).
pub fn delete_full_lines(
    mut commands: Commands,
    mut obstacles: Query<(Entity, &mut Block), With<Obstacle>>,
) {
    let mut row_counts = [0usize; BOARD_HEIGHT as usize];

    for (_, block) in &obstacles {
        if block.cell.is_visible() {
            row_counts[block.cell.1 as usize] += 1;
        }
    }

    let full_rows = row_counts
        .iter()
        .enumerate()
        .filter_map(|(row, count)| (*count == BOARD_WIDTH as usize).then_some(row))
        .collect::<Vec<_>>();

    if full_rows.is_empty() {
        return;
    }

    for (entity, block) in &mut obstacles {
        if full_rows.contains(&(block.cell.1 as usize)) {
            commands.entity(entity).despawn();
        }
    }

    for (_, mut block) in &mut obstacles {
        let rows_below = full_rows
            .iter()
            .filter(|row| **row < block.cell.1 as usize)
            .count() as i32;

        if rows_below > 0 {
            block.cell.1 -= rows_below;
        }
    }

    #[cfg(feature = "score")]
    commands.trigger(LinesCleared(full_rows.len() as u32));
}
