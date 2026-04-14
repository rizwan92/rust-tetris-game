# Baseline Copy-Paste Cheatsheet

Use this file for the baseline feature only.

## Commenting Rule For This File

When you copy a snippet from this file:

1. keep the comments with the code
2. read the comments on the function parameters too
3. treat the comments as the explanation of why that exact line exists

The baseline file is the foundation, so the comments here are intentionally very literal.

## Important Sync Note

This baseline file is synced to the validated **baseline solution path**.

It is **not** meant to match the final all-features `src/` tree line-for-line.

That means:

1. the `src/data.rs` snippets are still very close to the current final source
2. some `src/board.rs` and `src/lib.rs` snippets are intentionally **baseline-only**
3. later feature docs will replace parts of those baseline snippets on purpose

So if you later compare this file against the fully finished source tree, some functions will look different.
That is expected.
The correct workflow is:

1. revert to the starter or baseline state
2. paste the baseline snippets from this file
3. then apply later feature docs one by one in order

Keep [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml) at:

```toml
enabled_features = []
```

Run baseline tests with:

```bash
cargo test --features test --test end_to_end -- --test-threads=1
```

Because you are on macOS, keep `--test-threads=1`.

## `src/data.rs`

### Replace `Cell::rotate_90_deg_cw`

File:
- [data.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/data.rs)

```rust
fn rotate_90_deg_cw(&self, x: f32, y: f32) -> Cell {
    // Shift the cell so the rotation center becomes the local origin.
    let dx = self.0 as f32 - x;
    // Shift the cell so the rotation center becomes the local origin.
    let dy = self.1 as f32 - y;
    // Apply clockwise rotation: (dx, dy) -> (dy, -dx), then shift back.
    Cell((x + dy).round() as i32, (y - dx).round() as i32)
}
```

### Replace `Tetromino::in_bounds`

```rust
pub fn in_bounds(&self) -> bool {
    // A tetromino is legal only when all of its cells are in bounds.
    self.cells.iter().all(Cell::in_bounds)
}
```

### Replace `Tetromino::rotate`

```rust
pub fn rotate(&mut self) {
    // The O piece does not visibly change under rotation.
    if self.is_o() {
        return;
    }

    // Read the current rotation center once.
    let (x, y) = self.center;
    // Rotate every cell around that same center.
    self.cells = self.cells.map(|cell| cell.rotate_90_deg_cw(x, y));
}
```

### Replace `Tetromino::shift`

```rust
pub fn shift(&mut self, dx: i32, dy: i32) {
    // Move each cell by the requested offset.
    self.cells = self.cells.map(|Cell(x, y)| Cell(x + dx, y + dy));
    // Move the rotation center by the same offset.
    self.center = (self.center.0 + dx as f32, self.center.1 + dy as f32);
}
```

### Replace `GameState::drop_interval`

```rust
pub fn drop_interval(&self) -> Duration {
    // Clamp the level so we never index past the gravity table.
    let level = usize::min(self.level as usize, Self::MAX_LEVEL - 1);
    // Convert the frame-based table entry into a real time duration.
    Duration::from_secs_f32(Self::INTERVALS[level] / Self::FRAMERATE)
}
```

### Add `JustSpawned` right after `Active`

This correction came from validating the snippets in the real codebase.

```rust
/// Whether this active tetromino was spawned this frame
#[derive(Component, Copy, Clone)]
pub struct JustSpawned;
```

## `src/board.rs`

## Validation corrections

When I tested the pasted baseline against the real app, two extra corrections were needed:

- newly spawned active pieces should ignore movement input for one update
- baseline gameplay input and redraw ordering should stay in `PostUpdate`

That means the validated baseline uses:

- `Query<&mut Tetromino, (With<Active>, Without<JustSpawned>)>` in `handle_user_input`
- a small `clear_just_spawned` system
- `(active_tetromino, Active, JustSpawned)` in `spawn_next_tetromino`
- `PostUpdate` wiring for input and redraw systems inside `build_app`

One more validated note:

- the working source keeps baseline gameplay input and redraw systems in `PostUpdate`
- moving them back to the starter-style plain `Update` schedule brought back flaky baseline timing on macOS

So for this assignment, that schedule change should be treated as a required baseline fix, not as extra architecture.

## Later-feature note

The current final repository has extra gameplay markers such as `ManualDropped`,
`HardDropped`, and `CarryGravityTimer`.

Those are **not** part of the baseline target.

So in this file:

- `handle_user_input` is the validated baseline version
- `spawn_next_tetromino` is the validated baseline version
- `build_app` is the validated baseline wiring

Later feature docs intentionally replace those parts.

### Replace `LockdownTimer::start_or_advance`

This is the baseline-only signature.
Later `hard_drop` work changes the real source to pass an explicit duration and
to branch slightly between replay timing and ordinary timing.

File:
- [board.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/board.rs)

```rust
fn start_or_advance(&mut self, time: &Time<Fixed>) {
    // If the lockdown timer does not exist yet, create it and wait until next tick.
    if self.0.is_none() {
        // Create a one-shot timer for the lockdown window.
        self.0 = Some(Timer::new(LOCKDOWN_DURATION, TimerMode::Once));
        // Do not tick on the same frame we started it.
        return;
    }

    // Otherwise advance the existing lockdown timer by one fixed-step delta.
    if let Some(timer) = &mut self.0 {
        // Tick the timer using fixed update time.
        timer.tick(time.delta());
    }
}
```

### Replace `handle_user_input`

This is the baseline-only input system.
Later the final source adds extra drop markers for hard-drop timing.

```rust
pub fn handle_user_input(
    // Read keyboard transitions for this frame.
    keyboard: Res<ButtonInput<KeyCode>>,
    // Read manual-drop gravity from the game state.
    state: Res<GameState>,
    // Access the currently active tetromino mutably, but skip brand-new spawns for one frame.
    mut active: Query<&mut Tetromino, (With<Active>, Without<JustSpawned>)>,
    // Access obstacles for collision checks.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // If there is no active piece, there is nothing to move.
    let Ok(mut tetromino) = active.single_mut() else {
        // Exit when no active tetromino exists.
        return;
    };

    // Process manual downward movement first, as required by the spec.
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        // Repeat the downward step up to the configured manual gravity amount.
        for _ in 0..state.manual_drop_gravity {
            // Work on a candidate copy before committing the move.
            let mut candidate = *tetromino;
            // Try moving one row down.
            candidate.shift(0, -1);
            // Stop dropping if the next step would be illegal.
            if crate::there_is_collision(&candidate, obstacles.reborrow()) {
                // Exit the manual-drop loop as soon as collision happens.
                break;
            }
            // Commit the legal downward move.
            *tetromino = candidate;
        }
    }

    // Process left movement second.
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        // Create a candidate move.
        let mut candidate = *tetromino;
        // Shift one cell to the left.
        candidate.shift(-1, 0);
        // Commit only if the new position is legal.
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            // Store the legal candidate back into the active piece.
            *tetromino = candidate;
        }
    }

    // Process right movement third.
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        // Create a candidate move.
        let mut candidate = *tetromino;
        // Shift one cell to the right.
        candidate.shift(1, 0);
        // Commit only if the new position is legal.
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            // Store the legal candidate back into the active piece.
            *tetromino = candidate;
        }
    }

    // Process rotation last; up and space should both trigger the same rotate action.
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::Space) {
        // Create a candidate rotation.
        let mut candidate = *tetromino;
        // Rotate the candidate piece.
        candidate.rotate();
        // Commit only if the rotated shape is legal.
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            // Store the legal rotated candidate.
            *tetromino = candidate;
        }
    }
}
```

### Add `clear_just_spawned`

This helper exists only in the validated baseline path.
Later feature work may remove it or fold the behavior into different systems.

```rust
pub fn clear_just_spawned(
    // Use commands so we can remove the one-frame spawn marker.
    mut commands: Commands,
    // Read every newly spawned active tetromino.
    fresh: Query<Entity, With<JustSpawned>>,
) {
    // Remove the marker after input had one frame to ignore the fresh piece.
    for entity in &fresh {
        commands.entity(entity).remove::<JustSpawned>();
    }
}
```

### Replace `gravity`

```rust
pub fn gravity(
    // Use fixed-step time so tests and gameplay stay aligned.
    time: Res<Time<Fixed>>,
    // Mutably access the game state so we can tick its gravity timer.
    mut state: ResMut<GameState>,
    // Access the active tetromino mutably.
    mut active: Query<&mut Tetromino, With<Active>>,
    // Access obstacles for collision checks.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Advance the repeating gravity timer by one fixed-step delta.
    state.gravity_timer.tick(time.delta());
    // Do nothing until the timer finishes.
    if !state.gravity_timer.just_finished() {
        // Exit early when gravity should not fire this frame.
        return;
    }

    // If no active piece exists, gravity has nothing to update.
    let Ok(mut tetromino) = active.single_mut() else {
        // Exit when there is no active tetromino.
        return;
    };

    // Create a candidate copy before moving.
    let mut candidate = *tetromino;
    // Try moving the piece down by one row.
    candidate.shift(0, -1);
    // Commit only when the new location is legal.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        // Store the legal gravity move.
        *tetromino = candidate;
    }
}
```

### Replace `deactivate_if_stuck`

```rust
pub fn deactivate_if_stuck(
    // Use commands to despawn active pieces and spawn obstacles.
    mut commands: Commands,
    // Use fixed-step time for the lockdown timer.
    time: Res<Time<Fixed>>,
    // Access the lockdown timer resource mutably.
    mut lockdown: ResMut<LockdownTimer>,
    // Read the active tetromino entity and value.
    active: Query<(Entity, &Tetromino), With<Active>>,
    // Access obstacles for collision checks.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // If there is no active piece, also clear any leftover lockdown state.
    let Ok((entity, tetromino)) = active.single() else {
        // Reset the timer when nothing is active.
        lockdown.reset();
        // Exit because there is nothing to deactivate.
        return;
    };

    // Copy the active tetromino so we can test a downward move.
    let tetromino = *tetromino;
    // Build a downward candidate.
    let mut candidate = tetromino;
    // Try shifting it down by one row.
    candidate.shift(0, -1);

    // If the piece can still move down, it is not stuck yet.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        // Clear any existing lockdown timer because the piece is free again.
        lockdown.reset();
        // Exit because the piece should stay active.
        return;
    }

    // The piece is resting, so start or advance the lockdown timer.
    lockdown.start_or_advance(&time);
    // Wait until the timer actually finishes.
    if !lockdown.just_finished() {
        // Exit early until lockdown completes.
        return;
    }

    // Remove the active tetromino entity.
    commands.entity(entity).despawn();
    // Convert each of the four tetromino cells into an obstacle block.
    for cell in tetromino.cells {
        // Spawn one obstacle per cell with the same color.
        commands.spawn((
            Block {
                // Keep the exact board cell.
                cell,
                // Keep the original tetromino color.
                color: tetromino.color,
            },
            // Mark the block as an obstacle.
            Obstacle,
        ));
    }
    // Clear the lockdown timer so the next piece starts fresh.
    lockdown.reset();
}
```

### Replace `spawn_next_tetromino`

This is the validated baseline spawn flow.
Later features may replace the final source version with gravity-carry logic instead.

```rust
pub fn spawn_next_tetromino(
    // Use commands to spawn/despawn tetromino entities.
    mut commands: Commands,
    // Access the bag and gravity timer through game state.
    mut state: ResMut<GameState>,
    // Reset lockdown state when a new piece appears.
    mut lockdown: ResMut<LockdownTimer>,
    // Check whether an active tetromino already exists.
    active: Query<(), (With<Active>, With<Tetromino>)>,
    // Access existing logical preview tetromino entities.
    next_tetrominoes: Query<Entity, (With<Next>, With<Tetromino>)>,
    // Access obstacles for collision checks.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Never spawn a second active piece while one is already present.
    if active.iter().next().is_some() {
        // Exit because an active tetromino already exists.
        return;
    }

    // Remove the old logical preview tetromino, if any.
    for entity in &next_tetrominoes {
        // Despawn the old preview entity.
        commands.entity(entity).despawn();
    }

    // Pull the next gameplay piece from the bag.
    let mut active_tetromino = state.bag.next_tetromino();
    // Shift it into the main board spawn position.
    active_tetromino.shift(4, BOARD_HEIGHT as i32 - 1 - active_tetromino.bounds().top);

    // End the game if the spawn location is already illegal.
    if crate::there_is_collision(&active_tetromino, obstacles.reborrow()) {
        // Trigger the game-over event instead of spawning the piece.
        commands.trigger(GameOver);
        // Exit because the game cannot continue.
        return;
    }

    // Peek at the upcoming piece for the preview window.
    let mut next_tetromino = state.bag.peek();
    // Shift the preview piece into the 5x5 side window coordinates.
    next_tetromino.shift(2, 2);

    // Spawn the active tetromino entity with one-frame input protection.
    commands.spawn((active_tetromino, Active, JustSpawned));
    // Spawn the logical preview tetromino entity.
    commands.spawn((next_tetromino, Next));
    // Reset gravity so the new piece starts with a fresh interval.
    state.gravity_timer.reset();
    // Reset lockdown state for the new active piece.
    lockdown.reset();
}
```

## `src/lib.rs`

### Update the `PostUpdate` systems inside `build_app`

This is the baseline-only wiring.
Later features can move or replace some systems in the final source tree.

```rust
.add_systems(
    PostUpdate,
    (
        handle_user_input,
        clear_just_spawned,
        redraw_board,
        redraw_side_board::<Next>,
    )
        .chain()
        .in_set(Game),
);
```

### Replace `redraw_board`

```rust
pub fn redraw_board(
    // Use commands to replace cell materials.
    mut commands: Commands,
    // Access material assets to create new colors.
    mut materials: ResMut<Assets<ColorMaterial>>,
    // Read the active tetromino, if one exists.
    tetrominoes: Query<&Tetromino, With<Active>>,
    // Read all obstacle blocks.
    obstacles: Query<&Block, With<Obstacle>>,
    // Access the visible board cell entities.
    mut board: ResMut<Board>,
) {
    // Map visible board entities to the color they should display this frame.
    let mut colors = HashMap::<Entity, Color>::new();

    // Color visible cells belonging to the active tetromino.
    for tetromino in &tetrominoes {
        // Iterate only over visible cells so indexing stays valid.
        for cell in tetromino.cells.iter().copied().filter(Cell::is_visible) {
            // Look up the board entity at this visible cell.
            let entity = board.cells[cell.1 as usize][cell.0 as usize];
            // Store the active tetromino color for that entity.
            colors.insert(entity, tetromino.color);
        }
    }

    // Color visible cells belonging to locked obstacle blocks.
    for block in &obstacles {
        // Skip invisible obstacle cells.
        if block.cell.is_visible() {
            // Look up the board entity at this obstacle position.
            let entity = board.cells[block.cell.1 as usize][block.cell.0 as usize];
            // Store the obstacle color for that entity.
            colors.insert(entity, block.color);
        }
    }

    // Repaint every visible board tile each frame.
    for entity in board.cells.iter_mut().flat_map(|row| row.iter_mut()) {
        // Use the mapped color if present, otherwise paint the background color.
        commands.entity(*entity).insert(MeshMaterial2d(
            materials.add(colors.get(entity).copied().unwrap_or(BG_COLOR)),
        ));
    }
}
```

### Replace `redraw_side_board`

```rust
pub fn redraw_side_board<Marker: Component>(
    // Use commands to replace side-board cell materials.
    mut commands: Commands,
    // Access material assets to create new colors.
    mut materials: ResMut<Assets<ColorMaterial>>,
    // Access all 5x5 side-board cells for this marker.
    mut side_board: Query<(&mut Block, Entity), With<Marker>>,
    // Read the logical preview or hold tetromino, if it exists.
    tetromino: Option<Single<&Tetromino, With<Marker>>>,
) {
    // Repaint each side-board tile every frame.
    for (mut block, entity) in &mut side_board {
        // Decide whether this tile should show tetromino color or background.
        let color = if let Some(t) = tetromino.as_ref() {
            // Paint the tile with tetromino color when its 5x5 coordinate is occupied.
            if t.cells.contains(&block.cell) {
                // Use the tetromino color for occupied preview cells.
                t.color
            } else {
                // Use background color for empty preview cells.
                BG_COLOR
            }
        } else {
            // If there is no tetromino, the entire side board is background.
            BG_COLOR
        };

        // Keep the block model color in sync with the visual color.
        block.color = color;
        // Update the rendered material for this tile.
        commands
            .entity(entity)
            .insert(MeshMaterial2d(materials.add(color)));
    }
}
```

## Suggested test order

Run these after pasting:

```bash
cargo test rotate_cell
cargo test shift
cargo test --features test --test end_to_end i_spawn -- --test-threads=1
cargo test --features test --test end_to_end j_spawn -- --test-threads=1
cargo test --features test --test end_to_end rotate -- --test-threads=1
cargo test --features test --test end_to_end shift -- --test-threads=1
cargo test --features test --test end_to_end -- --test-threads=1
```

## Important note

On this machine, the spawn, shift, and rotate baseline tests matched these
snippets locally. The remaining gravity tests looked timing-sensitive under
plain `cargo test`, and the repo README recommends `cargo-nextest` for Bevy
tests. If your last two gravity-style tests wobble, try the assignment’s
recommended runner next.
