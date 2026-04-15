# Collision

## Goal

Implement the `collision` feature by filling only the starter skeleton in:

- `Cargo.toml`
- `src/collision.rs`
- the small spawn-time collision check in `src/board.rs`

This feature is still a small feature, but it unlocks a lot of real gameplay:

- pieces stop when they touch obstacles
- full rows disappear
- rows above fall down with naive gravity
- spawning into a filled top area causes game over

## Step 1: Enable the feature in `Cargo.toml`

Find this line in [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml):

```toml
enabled_features = ["config"]
```

Replace it with:

```toml
enabled_features = ["config", "collision"]
```

Why:

- this branch is now implementing the real collision module
- `collision` depends on `config`, so we keep both

## Step 2: Replace `there_is_collision`

Find this starter code in [src/collision.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/collision.rs):

```rust
pub fn there_is_collision(
    _tetromino: &Tetromino,
    _obstacles: Query<&Block, With<Obstacle>>,
) -> bool {
    todo!()
}
```

Replace it with:

```rust
pub fn there_is_collision(
    tetromino: &Tetromino,
    obstacles: Query<&Block, With<Obstacle>>,
) -> bool {
    // First handle the simple illegal case:
    // if any cell leaves the board, the placement is illegal.
    // Example:
    // if one cell is at x = -1, the piece is colliding with the wall.
    if !tetromino.in_bounds() {
        return true;
    }

    // Then check whether any tetromino cell overlaps an obstacle block.
    // Example:
    // if the active piece wants to use Cell(4, 0) and an obstacle already
    // lives at Cell(4, 0), that is also a collision.
    for &cell in tetromino.cells() {
        if obstacles.iter().any(|block| block.cell == cell) {
            return true;
        }
    }

    // If neither of the illegal cases happened, the placement is legal.
    false
}
```

Why this works:

- out-of-bounds is collision
- overlapping an obstacle is collision
- otherwise there is no collision

## Step 3: Replace `delete_full_lines`

Find this starter code in [src/collision.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/collision.rs):

```rust
pub fn delete_full_lines() {}
```

Replace it with:

```rust
pub fn delete_full_lines(
    mut commands: Commands,
    mut obstacles: Query<(Entity, &mut Block), With<Obstacle>>,
) {
    // Count how many obstacle blocks appear in each visible row.
    // A row is full only if it contains exactly BOARD_WIDTH blocks.
    let mut counts = [0u32; BOARD_HEIGHT as usize];
    for (_, block) in &obstacles {
        if block.cell.is_visible() {
            counts[block.cell.1 as usize] += 1;
        }
    }

    // Mark every row that is full.
    let full_rows = counts.map(|count| count == BOARD_WIDTH);
    let lines_cleared = full_rows.iter().filter(|is_full| **is_full).count();

    // If nothing is full, there is nothing to delete.
    if lines_cleared == 0 {
        return;
    }

    // Visit every obstacle block once.
    for (entity, mut block) in &mut obstacles {
        let y = block.cell.1;

        // Blocks on cleared rows disappear.
        // Example:
        // if row 0 is full, every block with y == 0 is despawned.
        if (0..BOARD_HEIGHT as i32).contains(&y) && full_rows[y as usize] {
            commands.entity(entity).despawn();
            continue;
        }

        // Naive gravity moves a remaining block down by the number of cleared
        // rows strictly below it.
        // Example:
        // if rows 1 and 3 are cleared, a block at y = 5 moves down by 2.
        let drop_by = full_rows
            .iter()
            .enumerate()
            .filter(|(row, is_full)| **is_full && (*row as i32) < y)
            .count() as i32;

        if drop_by > 0 {
            block.cell.1 -= drop_by;
        }
    }

    // Later, the score feature listens for this event.
    #[cfg(feature = "score")]
    commands.trigger(crate::score::LinesCleared(lines_cleared as u32));
}
```

Why this works:

- first we identify which rows are full
- then we delete blocks on those rows
- then we move higher blocks down by the number of cleared rows below them
- this is exactly what the assignment calls naive gravity

## Step 4: Add the spawn-time collision check in `src/board.rs`

The collision spec says some systems may also need to be fixed after the real
collision module is enabled.

That is because game over happens when a newly spawned piece collides
immediately with the existing stack.

Find the signature of `spawn_next_tetromino` in [src/board.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/board.rs):

```rust
pub fn spawn_next_tetromino(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    active_tetrominoes: Query<Entity, With<Active>>,
    next_tetrominoes: Query<Entity, With<Next>>,
) {
```

Replace it with:

```rust
pub fn spawn_next_tetromino(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    active_tetrominoes: Query<Entity, With<Active>>,
    next_tetrominoes: Query<Entity, With<Next>>,
    obstacles: Query<&Block, With<Obstacle>>,
) {
```

Then, after the starter code that shifts the new active piece into its spawn
position, add this block before `commands.spawn((active, Active));`:

```rust
    // If the new piece already overlaps the stack, the game is over.
    // Example:
    // when the stack reaches the top, the next piece cannot legally enter.
    if crate::there_is_collision(&active, obstacles) {
        commands.trigger(GameOver);
        return;
    }
```

Why this is needed:

- `basic_game_over` depends on spawn-time collision ending the game
- `collision.rs` can detect the overlap, but `spawn_next_tetromino` must
  actually ask that question

## Local checks

Run:

```bash
cargo fmt --all
```

Run:

```bash
cargo test --features test collision:: -- --nocapture
```

Run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end basic_stacking basic_game_over --no-fail-fast
```

Local macOS note:

- `basic_game_over` and other sleep-based tests may still be noisy locally
- use Linux CI as the final truth for timing-sensitive behavior

## Summary

This feature should end with:

- real out-of-bounds checking
- real obstacle overlap checking
- full-line deletion
- naive gravity after line clears
- spawn-time collision causing game over
