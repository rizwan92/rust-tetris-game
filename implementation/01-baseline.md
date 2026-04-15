# Baseline

## Goal

Finish the baseline feature by filling only the missing starter code in:

- `src/data.rs`
- `src/board.rs`

This guide is intentionally copy-paste friendly.

The idea is:

1. find the TODO in the starter file
2. replace it with the matching snippet below
3. read the comment lines to understand why the code works
4. run the small local checks before moving on

## Important rule for this branch

We are keeping the starter structure.

That means:

- no new architecture
- no lifecycle framework
- no extra timing system
- no new struct unless a later feature truly needs it

## File 1: `src/data.rs`

### 1. Replace `Cell::rotate_90_deg_cw`

Find this starter code in [src/data.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/data.rs):

```rust
fn rotate_90_deg_cw(&self, _x: f32, _y: f32) -> Cell {
    todo!("copy from earlier assignments")
}
```

Replace it with:

```rust
fn rotate_90_deg_cw(&self, x: f32, y: f32) -> Cell {
    // Move the cell into coordinates relative to the rotation center.
    // Example:
    // if the cell is (3, 4) and the center is (0, 0),
    // then dx = 3 and dy = 4.
    let dx = self.0 as f32 - x;

    // Do the same for the y direction.
    // Continuing the same example, dy = 4.
    let dy = self.1 as f32 - y;

    // A clockwise 90-degree rotation changes (dx, dy) into (dy, -dx).
    // Example:
    // (3, 4) becomes (4, -3).
    // Then we shift back by the original center.
    Cell((x + dy).round() as i32, (y - dx).round() as i32)
}
```

What this does:

- converts the cell into local coordinates
- rotates around the origin
- shifts back into board coordinates

### 2. Replace `Tetromino::in_bounds`

Find:

```rust
pub fn in_bounds(&self) -> bool {
    todo!("copy from earlier")
}
```

Replace it with:

```rust
pub fn in_bounds(&self) -> bool {
    // The whole tetromino is legal only when all four cells are legal.
    // Example:
    // if three cells are inside the board but one is at x = -1,
    // the whole tetromino is out of bounds.
    self.cells.iter().all(Cell::in_bounds)
}
```

### 3. Replace `Tetromino::rotate`

Find:

```rust
pub fn rotate(&mut self) {
    if self.is_o() {
        return;
    }

    todo!("rotate everything 90 degrees around the center.")
}
```

Replace it with:

```rust
pub fn rotate(&mut self) {
    // The O piece looks the same after rotation, so the baseline tests
    // expect it to remain unchanged.
    if self.is_o() {
        return;
    }

    // Read the rotation center once.
    // Example:
    // a T piece on the board may rotate around (4.0, 18.0).
    let (x, y) = self.center;

    // Rotate each of the four cells around that same center.
    self.cells = self.cells.map(|cell| cell.rotate_90_deg_cw(x, y));
}
```

### 4. Replace `Tetromino::shift`

Find:

```rust
pub fn shift(&mut self, _dx: i32, _dy: i32) {
    todo!("copy from earlier")
}
```

Replace it with:

```rust
pub fn shift(&mut self, dx: i32, dy: i32) {
    // Move every cell by the same offset.
    // Example:
    // shifting by (2, -1) turns Cell(3, 5) into Cell(5, 4).
    self.cells = self.cells.map(|Cell(x, y)| Cell(x + dx, y + dy));

    // The rotation center must move by the same amount.
    // Otherwise later rotations would happen around the wrong point.
    self.center = (self.center.0 + dx as f32, self.center.1 + dy as f32);
}
```

### 5. Replace `GameState::drop_interval`

Find:

```rust
pub fn drop_interval(&self) -> Duration {
    todo!("this calculation can use floats directly unlike the one below")
}
```

Replace it with:

```rust
pub fn drop_interval(&self) -> Duration {
    // Clamp the level so we never index past the gravity table.
    // Example:
    // if level somehow becomes 99, we still use the last valid interval.
    let level = usize::min(self.level as usize, Self::MAX_LEVEL - 1);

    // Convert "frames per drop" into seconds using the fixed game framerate.
    // Example:
    // level 0 uses 48 frames, so the duration is 48 / 60 seconds.
    Duration::from_secs_f32(Self::INTERVALS[level] / Self::FRAMERATE)
}
```

## File 2: `src/board.rs`

### 1. Replace `LockdownTimer::start_or_advance`

Find:

```rust
fn start_or_advance(&mut self, _time: Res<Time<Fixed>>) {
    todo!()
}
```

Replace it with:

```rust
fn start_or_advance(&mut self, time: Res<Time<Fixed>>) {
    // If this is the first frame where the piece is stuck,
    // create the one-shot lockdown timer.
    // Example:
    // a J piece reaches the floor, so lockdown starts now.
    if self.0.is_none() {
        self.0 = Some(Timer::new(LOCKDOWN_DURATION, TimerMode::Once));
    }

    // Advance the timer by one fixed-update delta.
    if let Some(timer) = &mut self.0 {
        timer.tick(time.delta());
    }
}
```

### 2. Replace `handle_user_input`

Find:

```rust
pub fn handle_user_input() {}
```

Replace it with:

```rust
pub fn handle_user_input(
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    mut tetrominoes: Query<&mut Tetromino, With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // If there is no active piece, there is nothing to move.
    let Ok(mut tetromino) = tetrominoes.single_mut() else {
        return;
    };

    // Down happens first by the assignment spec.
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        // The number of rows dropped depends on the game state.
        // In the baseline this is normally 1.
        for _ in 0..state.manual_drop_gravity {
            // Build a candidate one row lower.
            let mut candidate = *tetromino;
            candidate.shift(0, -1);

            // Stop if that move would be illegal.
            if crate::there_is_collision(&candidate, obstacles.reborrow()) {
                break;
            }

            // Accept the move.
            *tetromino = candidate;

            // A successful move means the previous "stuck" state no longer matters.
            lockdown.reset();
        }
    }

    // Left happens after down.
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        let mut candidate = *tetromino;
        candidate.shift(-1, 0);
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            lockdown.reset();
        }
    }

    // Right happens after left.
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        let mut candidate = *tetromino;
        candidate.shift(1, 0);
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            lockdown.reset();
        }
    }

    // Up or Space means rotate.
    // Using `||` means pressing both still rotates only once.
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::Space) {
        let mut candidate = *tetromino;
        candidate.rotate();
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            lockdown.reset();
        }
    }
}
```

Important example:

- if the player presses `Down + Right` in the same frame,
  the code drops first and moves right second

### 3. Replace `gravity`

Find:

```rust
pub fn gravity() {}
```

Replace it with:

```rust
pub fn gravity(
    time: Res<Time<Fixed>>,
    mut state: ResMut<GameState>,
    mut tetrominoes: Query<&mut Tetromino, With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Advance the repeating gravity timer each fixed frame.
    state.gravity_timer.tick(time.delta());

    // If the timer has not fired yet, do nothing.
    if !state.gravity_timer.just_finished() {
        return;
    }

    // No active piece means nothing can fall.
    let Ok(mut tetromino) = tetrominoes.single_mut() else {
        return;
    };

    // Try moving the active piece down by one row.
    let mut candidate = *tetromino;
    candidate.shift(0, -1);

    // Commit the fall only when it stays legal.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        *tetromino = candidate;
    }
}
```

### 4. Replace `deactivate_if_stuck`

Find:

```rust
pub fn deactivate_if_stuck() {}
```

Replace it with:

```rust
pub fn deactivate_if_stuck(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut lockdown: ResMut<LockdownTimer>,
    tetrominoes: Query<(Entity, &Tetromino), With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // If there is no active tetromino, clear the timer and stop.
    let Ok((entity, tetromino)) = tetrominoes.single() else {
        lockdown.reset();
        return;
    };

    // Check whether the active piece could still move down.
    let mut candidate = *tetromino;
    candidate.shift(0, -1);

    // If it can still fall, it should not lock yet.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        lockdown.reset();
        return;
    }

    // The piece is resting on something, so advance the lockdown timer.
    lockdown.start_or_advance(time);

    // If the timer has not finished yet, keep waiting.
    if !lockdown.just_finished() {
        return;
    }

    // The active tetromino is now locked, so remove its active entity.
    commands.entity(entity).despawn();

    // Replace it with four obstacle blocks at the same cells.
    // Example:
    // a locked J at the bottom becomes four separate obstacle blocks.
    for &cell in tetromino.cells() {
        commands.spawn((
            Block {
                cell,
                color: tetromino.color,
            },
            Obstacle,
        ));
    }

    // Reset the lockdown timer for the next piece.
    lockdown.reset();
}
```

### 5. Replace `spawn_next_tetromino`

Find:

```rust
pub fn spawn_next_tetromino() {}
```

Replace it with:

```rust
pub fn spawn_next_tetromino(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    active_tetrominoes: Query<Entity, With<Active>>,
    next_tetrominoes: Query<Entity, With<Next>>,
) {
    // Only spawn when there is no active tetromino.
    if !active_tetrominoes.is_empty() {
        return;
    }

    // Remove the old preview piece first.
    for entity in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    // Take the next real piece from the bag.
    let mut active = state.bag.next_tetromino();

    // Most pieces spawn at board shift (4, 18).
    // The I piece uses (4, 19) because its center is different.
    if active.center == (0.5, -0.5) {
        active.shift(4, 19);
    } else {
        active.shift(4, 18);
    }

    // Spawn the active gameplay piece.
    commands.spawn((active, Active));

    // Peek at the next upcoming piece for the side window.
    let mut next = state.bag.peek();

    // Shift it into the center of the 5x5 preview board.
    next.shift(2, 2);

    // Spawn the logical preview piece.
    commands.spawn((next, Next));
}
```

Important example:

- `J` spawns with board cells around y = 18 and 19
- `I` spawns one row higher because its center is `(0.5, -0.5)`

### 6. Replace the body of `redraw_board`

Keep the function signature the same, and replace the body content with:

```rust
{
    // This map stores the color each visible board tile should receive.
    // If a tile is missing from the map, it stays black.
    let mut colors = HashMap::<Entity, Color>::new();

    // First, color the active tetromino cells.
    // Example:
    // if the active O sits at (4,18), (4,19), (5,18), (5,19),
    // those four board entities get the O color.
    for tetromino in &tetrominoes {
        for &Cell(x, y) in tetromino.cells().iter().filter(|cell| cell.is_visible()) {
            colors.insert(board.cells[y as usize][x as usize], tetromino.color);
        }
    }

    // Then color the obstacle cells.
    for block in obstacles.iter().filter(|block| block.cell.is_visible()) {
        let Cell(x, y) = block.cell;
        colors.insert(board.cells[y as usize][x as usize], block.color);
    }

    // Finally, redraw every visible tile entity.
    for entity in board.cells.iter_mut().flat_map(|row| row.iter_mut()) {
        commands.entity(*entity).insert(MeshMaterial2d(
            materials.add(colors.get(entity).copied().unwrap_or(BG_COLOR)),
        ));
    }
}
```

### 7. Replace `redraw_side_board`

Find:

```rust
pub fn redraw_side_board<Marker: Component>(
    _commands: Commands,
    _materials: ResMut<Assets<ColorMaterial>>,
    _side_board: Query<(&mut Block, Entity), With<Marker>>,
    tetromino: Option<Single<&Tetromino, With<Marker>>>,
) {
    if let Some(_t) = tetromino {
        todo!("add the drawing code here to update side_board")
    }
}
```

Replace it with:

```rust
pub fn redraw_side_board<Marker: Component>(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    side_board: Query<(&mut Block, Entity), With<Marker>>,
    tetromino: Option<Single<&Tetromino, With<Marker>>>,
) {
    // Visit every tile in the 5x5 side board.
    for (block, entity) in &side_board {
        // If the preview tetromino uses this logical cell, paint it.
        // Otherwise leave the cell black.
        let color = tetromino
            .as_ref()
            .filter(|tetromino| tetromino.cells().contains(&block.cell))
            .map(|tetromino| tetromino.color)
            .unwrap_or(BG_COLOR);

        // Write the new color back to the side-board tile entity.
        commands
            .entity(entity)
            .insert(MeshMaterial2d(materials.add(color)));
    }
}
```

## Local checks to run

### Strong local signal

Run:

```bash
cargo test --features test data::tests -- --nocapture
```

Run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end i_spawn j_spawn shift1 l_rotate --no-fail-fast
```

These are good baseline checks because they mostly test:

- geometry
- spawn placement
- shifting
- rotation

### Timing-sensitive local checks on macOS

Run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end gravity1 gravity_and_input --no-fail-fast
```

If these fail on macOS, do not panic immediately.

These are the exact kind of sleep-based tests that can be sensitive on macOS.
For this rebuild, Linux CI is the final judge for that timing path.

## Summary

The baseline implementation should now give you:

- working tetromino rotation
- working movement
- working gravity
- working locking into obstacles
- working next-piece preview
- working board redraw

After baseline is stable enough in CI, the next feature should be `config`.
