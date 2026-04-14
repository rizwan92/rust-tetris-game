# PR 3 Guide: Collision

This PR is where the board rules become real.

Before this PR, the game can already move pieces around.
After this PR, the game properly understands:

- when a move is illegal
- when a piece touches existing blocks
- when a line is full
- how blocks above a cleared line should fall

So this PR is a big step in making the board behave like real Tetris.

## What this PR is trying to achieve

At the end of this PR, the game should correctly answer 2 important questions:

1. "Is this tetromino position legal?"
2. "Did we just complete one or more full lines?"

Those 2 questions drive almost everything else:

- movement safety
- rotation safety
- stacking
- game over by blocked spawn
- later score updates

## Starter files to compare

- `original-repo/src/collision.rs`
- `original-repo/docs/collision.md`

## File you will change

- `src/collision.rs`

## Feature flag state

Use:

```toml
enabled_features = ["config", "collision"]
```

## Mental model before touching code

This file has only 2 real jobs.

### Job 1: `there_is_collision(...)`

This function answers:

"If I put this tetromino here, is that illegal?"

It should return `true` when:

- the tetromino goes out of bounds
- the tetromino overlaps an obstacle block

It should return `false` when:

- all cells stay in bounds
- none of the cells overlap an obstacle

### Job 2: `delete_full_lines(...)`

This function answers:

"After a piece locked, are any visible rows completely full?"

If yes, it should:

1. remove every obstacle block in those full rows
2. move blocks above those rows downward
3. later, when score is enabled, trigger the score event

That is the whole feature.

## Important background idea: visible rows vs invisible rows

The board has:

- 20 visible rows
- 3 invisible rows above them

This matters because:

- collision checking should respect the full legal board, including invisible rows
- full-line deletion should only count visible rows

So:

- `in_bounds()` is allowed to include invisible rows
- line clearing should use `block.cell.is_visible()`

That detail is easy to miss, and it causes confusing bugs if you forget it.

## Step 1: replace `there_is_collision`

### Why this step exists

The board code already calls `crate::there_is_collision(...)` from:

- player input
- gravity
- spawn checks
- hold/activation logic later

So if this function is correct, lots of other behavior suddenly becomes correct.

If this function is wrong, many other systems look broken even though they are
actually calling the right helper.

### What to replace

Open `src/collision.rs`.

Find the starter version:

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

### What this function is doing in simple English

It checks legality in 2 layers:

1. board bounds
2. obstacle overlap

That order is useful because:

- out-of-bounds is the cheapest thing to reject first
- only legal-in-bounds positions need obstacle comparison

### Why `.any(...)` is the right pattern here

This line:

```rust
.any(|cell| obstacles.iter().any(|block| block.cell == *cell))
```

means:

"Does there exist at least one tetromino cell that is equal to at least one
obstacle cell?"

If yes:

- collision exists

If no:

- no overlap exists

That is exactly the rule we want.

### Example

Suppose the active tetromino has one cell at `(4, 10)`.

And an obstacle block also exists at `(4, 10)`.

Then this function must return `true`, because the move is illegal.

## Step 2: replace `delete_full_lines`

### Why this step exists

Once pieces lock into obstacles, the board starts filling up.

Now we need the game to recognize:

- a row is full
- that row should disappear
- everything above should fall down

Without this function:

- rows never clear
- the game can stack forever without scoring behavior later
- obstacle positions become incorrect over time

### What to replace

Find the starter placeholder:

```rust
pub fn delete_full_lines() {}
```

Replace it with:

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

## Step 3: understand the 3 phases inside `delete_full_lines`

This function looks longer than it really is.

It has only 3 logical phases.

### Phase 1: count blocks per visible row

This part:

```rust
let mut row_counts = [0usize; BOARD_HEIGHT as usize];
```

creates one counter per visible row.

Then:

```rust
row_counts[block.cell.1 as usize] += 1;
```

increments the counter for each obstacle block in that row.

So after the loop:

- row with 10 blocks on a 10-wide board = full row

### Phase 2: despawn blocks in full rows

This part:

```rust
if full_rows.contains(&(block.cell.1 as usize)) {
    commands.entity(entity).despawn();
}
```

removes every block that belongs to a full row.

That is the "line disappears" part.

### Phase 3: move remaining blocks downward

This part:

```rust
let rows_below = full_rows
    .iter()
    .filter(|row| **row < block.cell.1 as usize)
    .count() as i32;
```

counts how many cleared rows exist below the current block.

Then:

```rust
block.cell.1 -= rows_below;
```

moves the block down by exactly that amount.

That is why this is called naive gravity:

- blocks do not fall independently by physics
- they simply shift downward by the number of cleared rows below them

## Example for naive gravity

Suppose rows `3` and `7` are cleared.

Now imagine one obstacle block is at row `10`.

There are 2 cleared rows below it:

- row `3`
- row `7`

So that block should move from:

- `10` to `8`

because it drops by 2 rows.

That is exactly what `rows_below` is calculating.

## Step 4: understand the future scoring hook

At the bottom of the function you will see:

```rust
#[cfg(feature = "score")]
commands.trigger(LinesCleared(full_rows.len() as u32));
```

### Why this is already here

Because later the score system needs to know:

- how many lines were just cleared

So this collision PR prepares that event now.

When `score` is not enabled:

- this line does nothing

When `score` is enabled later:

- the score system will react to it

This is a nice example of building features in layers.

## Common beginner confusion here

### "Why does collision.rs affect movement tests?"

Because `board.rs` already uses `there_is_collision(...)`.

So once this helper becomes real, lots of movement behavior becomes correct.

### "Why are we only counting visible rows?"

Because line clears should happen in the playable board area, not in the hidden
spawn rows.

### "Why not move blocks down first and then despawn?"

Because the full-row blocks should disappear completely.

So the correct order is:

1. identify full rows
2. despawn those row blocks
3. move the remaining blocks down

## Tests for this PR

### Main collision-focused check

Run:

```bash
cargo nextest run --features test,config,collision --test end_to_end \
  basic_stacking basic_game_over gravity1 gravity_and_input \
  --no-fail-fast
```

This checks:

- stacking
- obstacle interaction
- spawn/game-over interaction
- gravity still behaving correctly

### Wider regression check

Then run:

```bash
cargo nextest run --features test,config,collision --test end_to_end --no-fail-fast
```

This makes sure collision logic did not break the earlier baseline behaviors.

## When this PR is done

Stop this PR when:

- illegal positions are rejected correctly
- stacking works
- blocked spawns correctly end the game
- full visible rows disappear
- blocks above cleared rows move down by naive gravity

Do not start score in the same PR.
