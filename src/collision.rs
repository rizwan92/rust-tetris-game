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
    // First reject any position that leaves the legal board area.
    // This includes going past the walls, the floor, or the spawn ceiling.
    if !tetromino.in_bounds() {
        return true;
    }

    // Then check whether any tetromino cell overlaps an obstacle cell.
    // If even one overlap exists, the whole candidate position is illegal.
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
    // Count how many obstacle blocks exist in each visible row.
    // This first pass tells us which rows are full.
    let mut row_counts = [0usize; BOARD_HEIGHT as usize];

    for (_, block) in &obstacles {
        // Ignore invisible spawn rows when checking for visible line clears.
        if block.cell.is_visible() {
            row_counts[block.cell.1 as usize] += 1;
        }
    }

    // A full row is any row whose block count matches the board width.
    let full_rows = row_counts
        .iter()
        .enumerate()
        .filter_map(|(row, count)| (*count == BOARD_WIDTH as usize).then_some(row))
        .collect::<Vec<_>>();

    // Stop immediately when there is nothing to clear.
    if full_rows.is_empty() {
        return;
    }

    for (entity, block) in &mut obstacles {
        // Despawn every obstacle that sits on a row we are deleting.
        if full_rows.contains(&(block.cell.1 as usize)) {
            commands.entity(entity).despawn();
        }
    }

    for (_, mut block) in &mut obstacles {
        // Count how many cleared rows are strictly below this obstacle.
        // That number is exactly how far naive gravity should move it down.
        let rows_below = full_rows
            .iter()
            .filter(|row| **row < block.cell.1 as usize)
            .count() as i32;

        // Move the block down only when at least one cleared row is below it.
        if rows_below > 0 {
            block.cell.1 -= rows_below;
        }
    }

    #[cfg(feature = "score")]
    // Emit a scoring event so the score system can react later.
    commands.trigger(LinesCleared(full_rows.len() as u32));
}
