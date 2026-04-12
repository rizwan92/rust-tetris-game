//! Collision detection implementation

use super::board::Block;
use super::data::*;
use bevy::prelude::*;

/// Return whether the given tetromino collides with any of the obstacles or it
/// is out of bounds.
pub fn there_is_collision(
    _tetromino: &Tetromino,
    _obstacles: Query<&Block, With<Obstacle>>,
) -> bool {
    todo!()
}

/// A system to detect the full lines (lines containing only obstacles and no
/// empty space) and to delete them.
///
/// After the lines are deleted, any obstacles above the deleted lines should be
/// moved down using naive gravity (the obstacles move down only by the number
/// of lines below them that are deleted).
pub fn delete_full_lines() {}
