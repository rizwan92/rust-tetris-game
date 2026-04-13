# Collision Copy-Paste Cheatsheet

Use this file after baseline and config are complete.

## Commenting Rule For This File

- collision logic can feel abstract at first, so the comments are written very literally
- parameter comments explain what enters the function
- body comments explain what each pass over the data is doing

## Enable the feature

Set [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml) to:

```toml
enabled_features = ["config", "collision"]
```

## What this feature changes

- switches from `mock_collision.rs` to real `collision.rs`
- makes movement and spawn fail on obstacle overlap
- turns full obstacle rows into cleared lines
- applies naive gravity after line clears
- sets up a future hook for score updates with `LinesCleared`
- explains each changed line in simple English near the snippet

## `src/collision.rs`

### Add this import near the top

Keep the existing imports and add:

```rust
#[cfg(feature = "score")]
use crate::score::LinesCleared;
```

### Replace `there_is_collision`

File:
- [collision.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/collision.rs)

```rust
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
```

### Replace `delete_full_lines`

This version already includes the future `score` hook behind `#[cfg(feature = "score")]`.

```rust
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
```

## Notes for later features

- `score` will consume the `LinesCleared` event emitted here.
- Because your baseline systems already call `there_is_collision`, they will automatically pick up obstacle-aware collision after this feature is enabled.
- `spawn_next_tetromino` should now trigger game over when a new piece collides with stacked obstacles at spawn.

## Test commands

Start with the smallest collision-specific file:

```bash
cargo test --features test --test end_to_end basic_stacking -- --test-threads=1
cargo test --features test --test end_to_end basic_game_over -- --test-threads=1
```

Then run the whole collision test file:

```bash
cargo test --features test --test end_to_end basic_ -- --test-threads=1
```

Then run the cumulative regression sweep for baseline + config + collision:

```bash
cargo test --features test --test end_to_end -- --test-threads=1
```

## Acceptance checkpoint

Do not move to `score` until:

- `basic_stacking` passes
- `basic_game_over` passes
- the collision test file passes
- the cumulative end-to-end suite still passes with `enabled_features = ["config", "collision"]`
