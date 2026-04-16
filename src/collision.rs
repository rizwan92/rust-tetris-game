//! Collision detection implementation

use super::board::Block;
use super::data::*;
use bevy::prelude::*;

/// Return whether the given tetromino collides with any of the obstacles or it
/// is out of bounds.
pub fn there_is_collision(tetromino: &Tetromino, obstacles: Query<&Block, With<Obstacle>>) -> bool {
    // First handle the simplest illegal case:
    // if any cell leaves the board, the whole tetromino placement is illegal.
    // Example:
    // a piece with one cell at x = -1 is already colliding with the wall.
    if !tetromino.in_bounds() {
        return true;
    }

    // Then check whether any active-piece cell overlaps an existing obstacle.
    // Example:
    // if the active L piece wants to move into Cell(4, 0) and an obstacle block
    // already lives at Cell(4, 0), that move must be rejected.
    for &cell in tetromino.cells() {
        if obstacles.iter().any(|block| block.cell == cell) {
            return true;
        }
    }

    // If we reach this point, the tetromino is fully inside the board and it
    // does not overlap any obstacle block.
    false
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
    // Count how many obstacle blocks appear in each visible row.
    // A row is full only if it contains exactly BOARD_WIDTH obstacle blocks.
    let mut counts = [0u32; BOARD_HEIGHT as usize];
    for (_, block) in &obstacles {
        if block.cell.is_visible() {
            counts[block.cell.1 as usize] += 1;
        }
    }

    // Record which rows are full so we can:
    // 1. despawn blocks on those rows
    // 2. move higher rows down by the right amount
    let full_rows = counts.map(|count| count == BOARD_WIDTH);
    let lines_cleared = full_rows.iter().filter(|is_full| **is_full).count();

    // If there are no full rows, this system has nothing to do this frame.
    if lines_cleared == 0 {
        return;
    }

    // Walk through every obstacle block once.
    for (entity, mut block) in &mut obstacles {
        let y = block.cell.1;

        // Any obstacle that sits on a cleared row must be removed.
        // Example:
        // if row 0 is full, every obstacle with cell.y == 0 disappears.
        if (0..BOARD_HEIGHT as i32).contains(&y) && full_rows[y as usize] {
            commands.entity(entity).despawn();
            continue;
        }

        // Naive gravity means we move a remaining block down by the number of
        // cleared rows strictly below it.
        // Example:
        // if rows 2 and 5 are cleared, then a block at y = 7 moves down by 2.
        let drop_by = full_rows
            .iter()
            .enumerate()
            .filter(|(row, is_full)| **is_full && (*row as i32) < y)
            .count() as i32;

        // Only adjust the y coordinate when something was cleared below.
        if drop_by > 0 {
            block.cell.1 -= drop_by;
        }
    }

    // When the score feature is enabled, line clear scoring listens to this
    // event. We trigger it only after we know how many rows disappeared.
    #[cfg(feature = "score")]
    commands.trigger(crate::score::LinesCleared(lines_cleared as u32));
}
