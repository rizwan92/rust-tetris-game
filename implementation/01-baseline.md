# Baseline

## Goal

Finish the baseline feature by filling the missing starter code in:

- `src/data.rs`
- `src/board.rs`

This guide is written for the current working solution. Keep the original file
layout. Paste the new code close to the starter code it belongs to.

## File 1: `src/data.rs`

### 1. Replace `Cell::rotate_90_deg_cw`

```rust
fn rotate_90_deg_cw(&self, x: f32, y: f32) -> Cell {
    // Move the cell into "rotation-center-relative" coordinates.
    // Example:
    // if the cell is at (3, 4) and the center is (0, 0),
    // then dx = 3 and dy = 4.
    let dx = self.0 as f32 - x;
    // Use the same idea for the y direction.
    // Continuing the same example, dy = 4.
    let dy = self.1 as f32 - y;
    // A clockwise 90-degree rotation changes (dx, dy) into (dy, -dx).
    // Example:
    // (3, 4) becomes (4, -3).
    // Then we shift back by the original center and round to integer cells.
    Cell((x + dy).round() as i32, (y - dx).round() as i32)
}
```

### 2. Replace `Tetromino::in_bounds`

```rust
pub fn in_bounds(&self) -> bool {
    // The tetromino is legal only when every one of its four cells is legal.
    // Example:
    // if three cells are inside the board but one is at x = -1,
    // then the whole tetromino is out of bounds.
    self.cells.iter().all(Cell::in_bounds)
}
```

### 3. Replace `Tetromino::rotate`

```rust
pub fn rotate(&mut self) {
    // The O piece keeps the same visible shape after rotation,
    // so the baseline tests expect it to remain unchanged.
    if self.is_o() {
        return;
    }

    // Read the rotation center once so every cell uses the same pivot.
    // Example:
    // the T piece on the board might rotate around (4.0, 18.0).
    let (x, y) = self.center;
    // Rotate each of the four cells around that center.
    // We build a new array instead of mutating one cell at a time.
    self.cells = self.cells.map(|cell| cell.rotate_90_deg_cw(x, y));
}
```

### 4. Replace `Tetromino::shift`

```rust
pub fn shift(&mut self, dx: i32, dy: i32) {
    // Move every cell by the same offset.
    // Example:
    // shifting by (2, -1) turns Cell(3, 5) into Cell(5, 4).
    self.cells = self.cells.map(|Cell(x, y)| Cell(x + dx, y + dy));
    // The rotation center must move by the same amount or later rotations
    // would happen around the wrong point.
    self.center = (self.center.0 + dx as f32, self.center.1 + dy as f32);
}
```

### 5. Replace `GameState::drop_interval`

```rust
pub fn drop_interval(&self) -> Duration {
    // Clamp the level so we never index past the gravity table.
    // Example:
    // if level somehow becomes 99, we still use the last valid interval.
    let level = usize::min(self.level as usize, Self::MAX_LEVEL - 1);

    // Convert "frames per drop" into seconds using the game framerate.
    // Example:
    // level 0 uses 48 frames, so the duration is 48 / 60 seconds.
    Duration::from_secs_f32(Self::INTERVALS[level] / Self::FRAMERATE)
}
```

## File 2: `src/board.rs`

### 1. Replace `LockdownTimer::start_or_advance`

```rust
fn start_or_advance(&mut self, time: Res<Time<Fixed>>) {
    // Create the one-shot timer on the first grounded tick.
    if self.0.is_none() {
        self.0 = Some(Timer::new(LOCKDOWN_DURATION, TimerMode::Once));
    }

    // Advance the existing timer by exactly one fixed-step delta.
    if let Some(timer) = &mut self.0 {
        timer.tick(time.delta());
    }
}
```

### 2. Add the new helper resource and helper functions

Paste this block right below the `LockdownTimer` code:

```rust
/// NEW IMPLEMENTATION: Whether the current fixed step already spent its
/// automatic gravity probe on a blocked downward move.
///
/// Example:
/// if gravity tries to move the active piece from row 5 to row 4 but a locked
/// block is directly below it, we remember that blocked attempt here so the
/// lock-delay system can react correctly later in the same fixed tick.
#[derive(Resource, Default)]
pub struct BlockedAutoDrop(pub bool);

/// NEW IMPLEMENTATION: Reset the lock timer only when the moved piece is no
/// longer grounded.
///
/// Parameters:
/// - `tetromino`: the active piece after applying a legal move.
/// - `manual_drop_gravity`: how many rows the down key tries to move in one
///   input. `1` means soft drop, a larger value means hard-drop-style input.
/// - `lockdown`: the lock-delay resource we may clear.
/// - `obstacles`: the placed blocks used for collision checks.
///
/// Example:
/// if a T piece slides sideways off a ledge, the cell below becomes empty and
/// we should cancel the old lock timer.
fn reset_lockdown_after_move(
    tetromino: Tetromino,
    manual_drop_gravity: u32,
    lockdown: &mut LockdownTimer,
    obstacles: &mut Query<&Block, With<Obstacle>>,
) {
    // Check whether the moved piece is still resting on something by trying the
    // same piece one row lower.
    let mut below = tetromino;
    below.shift(0, -1);
    // Reset the lock timer when the moved piece can fall again.
    //
    // The soft-drop case is also reset here because the replay behavior expects
    // player-driven downward motion to refresh locking more aggressively.
    if !crate::there_is_collision(&below, obstacles.reborrow())
        || manual_drop_gravity == SOFT_DROP_GRAVITY
    {
        lockdown.reset();
    }
}

/// NEW IMPLEMENTATION: Move the active piece to the candidate position if the
/// move is legal.
///
/// Parameters:
/// - `tetromino`: the real active piece component we may overwrite.
/// - `candidate`: a hypothetical moved or rotated version of that piece.
/// - `manual_drop_gravity`: copied through so the lock helper can apply the
///   same movement rule.
/// - `lockdown`: the lock-delay timer resource.
/// - `obstacles`: the fixed blocks already on the board.
///
/// Returns:
/// - `true` when the move is legal and was applied.
/// - `false` when the move collides and must be ignored.
///
/// Example:
/// when the player presses left, `candidate` is the active piece shifted by
/// `(-1, 0)`. If that would hit the wall, we return `false`.
fn try_move_active(
    tetromino: &mut Tetromino,
    candidate: Tetromino,
    manual_drop_gravity: u32,
    lockdown: &mut LockdownTimer,
    obstacles: &mut Query<&Block, With<Obstacle>>,
) -> bool {
    // Reject illegal moves first so the live active piece stays unchanged.
    if crate::there_is_collision(&candidate, obstacles.reborrow()) {
        return false;
    }

    // Commit the candidate and then refresh the lock-delay state.
    *tetromino = candidate;
    reset_lockdown_after_move(*tetromino, manual_drop_gravity, lockdown, obstacles);
    true
}

/// NEW IMPLEMENTATION: Place a piece at its normal board spawn position.
///
/// Parameter:
/// - `tetromino`: a fresh canonical piece whose cells are still centered around
///   `(0, 0)` or `(0.5, -0.5)`.
///
/// Example:
/// an `O` piece is shifted to the visible spawn around row 19, while a `T`
/// piece is shifted to the normal spawn around row 18.
fn move_to_spawn_position(tetromino: &mut Tetromino) {
    // The `I` piece uses the half-cell center `(0.5, -0.5)`, so it needs a
    // slightly different spawn row from the other pieces.
    if tetromino.center == (0.5, -0.5) {
        tetromino.shift(4, 19);
    } else {
        tetromino.shift(4, 18);
    }
}

/// NEW IMPLEMENTATION: Clear one-frame gameplay input edges before a fresh
/// piece becomes active.
///
/// Parameter:
/// - `keyboard`: Bevy's button-state resource for keyboard keys.
///
/// Example:
/// if the previous piece locks on the same frame that the player pressed left,
/// we do not want the freshly spawned piece to inherit that old edge.
fn clear_gameplay_inputs(keyboard: &mut ButtonInput<KeyCode>) {
    keyboard.clear_just_pressed(KeyCode::ArrowDown);
    keyboard.clear_just_pressed(KeyCode::ArrowLeft);
    keyboard.clear_just_pressed(KeyCode::ArrowRight);
    keyboard.clear_just_pressed(KeyCode::ArrowUp);
    keyboard.clear_just_pressed(KeyCode::Space);
}
```

### 3. Update `setup_board`

At the end of `setup_board`, make sure these resource lines are present:

```rust
commands.add_observer(exit_on_game_over);
commands.insert_resource(LockdownTimer(None));
commands.insert_resource(BlockedAutoDrop::default());
```

Why this matters:

- `LockdownTimer(None)` keeps the lock-delay state ready from the start
- `BlockedAutoDrop::default()` is part of the newer baseline flow
- without that extra resource, the later gravity and lock systems will not work
  the way the final baseline expects

### 4. Replace `handle_user_input`

Paste this function:

```rust
/// NEW IMPLEMENTATION: Handle user input for moving and rotating the active
/// tetromino.
///
/// Parameters:
/// - `keyboard`: Bevy button-state resource holding one-frame key edges.
/// - `state`: the global game state, mainly used here for manual drop gravity.
/// - `lockdown`: the current lock-delay resource.
/// - `tetrominoes`: query for the single active piece.
/// - `obstacles`: query for the already-locked blocks.
///
/// Example:
/// if the player presses right, we build a shifted candidate and ask
/// `try_move_active` whether the move is legal.
pub fn handle_user_input(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    mut tetrominoes: Query<&mut Tetromino, With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // The system itself stays small and Bevy-shaped:
    // read resources/components, then delegate the repeated move rules to
    // small helpers.
    //
    // `single_mut()` is correct here because there should be at most one
    // falling active piece at a time.
    let Ok(mut tetromino) = tetrominoes.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        // Manual drop may move multiple rows in one input when hard drop is on.
        // Example:
        // soft drop uses 1 step, hard drop uses 20 attempts.
        for _ in 0..state.manual_drop_gravity {
            let mut candidate = *tetromino;
            candidate.shift(0, -1);
            if !try_move_active(
                &mut tetromino,
                candidate,
                state.manual_drop_gravity,
                &mut lockdown,
                &mut obstacles,
            ) {
                break;
            }
        }
        keyboard.clear_just_pressed(KeyCode::ArrowDown);
    }

    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        // Build a "one cell left" candidate and let the shared helper decide if
        // the move is legal.
        let mut candidate = *tetromino;
        candidate.shift(-1, 0);
        let _ = try_move_active(
            &mut tetromino,
            candidate,
            state.manual_drop_gravity,
            &mut lockdown,
            &mut obstacles,
        );
        keyboard.clear_just_pressed(KeyCode::ArrowLeft);
    }

    if keyboard.just_pressed(KeyCode::ArrowRight) {
        // Same idea as left movement, but one cell to the right.
        let mut candidate = *tetromino;
        candidate.shift(1, 0);
        let _ = try_move_active(
            &mut tetromino,
            candidate,
            state.manual_drop_gravity,
            &mut lockdown,
            &mut obstacles,
        );
        keyboard.clear_just_pressed(KeyCode::ArrowRight);
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::Space) {
        // In this codebase both Up and Space rotate the active piece.
        let mut candidate = *tetromino;
        candidate.rotate();
        let _ = try_move_active(
            &mut tetromino,
            candidate,
            state.manual_drop_gravity,
            &mut lockdown,
            &mut obstacles,
        );
        keyboard.clear_just_pressed(KeyCode::ArrowUp);
        keyboard.clear_just_pressed(KeyCode::Space);
    }
}
```

### 5. Replace `gravity`

```rust
/// NEW IMPLEMENTATION: Drop the piece whenever the gravity timer goes off.
///
/// Parameters:
/// - `time`: Bevy fixed-step clock resource.
/// - `state`: mutable game state that owns the repeating gravity timer.
/// - `blocked_auto_drop`: remembers whether this tick's automatic drop was
///   blocked by a collision.
/// - `tetrominoes`: query for the active piece.
/// - `obstacles`: query for placed blocks.
///
/// Example:
/// if level-0 gravity reaches its interval, this system tries to move the
/// active piece down by one row.
pub fn gravity(
    time: Res<Time<Fixed>>,
    mut state: ResMut<GameState>,
    mut blocked_auto_drop: ResMut<BlockedAutoDrop>,
    mut tetrominoes: Query<&mut Tetromino, With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Start each fixed tick assuming gravity is not blocked.
    blocked_auto_drop.0 = false;
    // Advance the repeating gravity timer using fixed-step time.
    state.gravity_timer.tick(time.delta());
    // Only act on the tick where gravity actually finishes.
    if !state.gravity_timer.just_finished() {
        return;
    }

    let Ok(mut tetromino) = tetrominoes.single_mut() else {
        return;
    };

    let mut candidate = *tetromino;
    candidate.shift(0, -1);
    // If the path below is free, gravity moves the piece one row down.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        *tetromino = candidate;
    } else {
        // Otherwise remember that automatic gravity hit something this tick.
        blocked_auto_drop.0 = true;
    }
}
```

### 6. Replace `deactivate_if_stuck`

```rust
/// NEW IMPLEMENTATION: Check if the active tetromino cannot move down. If so,
/// deactivate it.
///
/// Parameters:
/// - `commands`: command buffer used to despawn and spawn entities.
/// - `time`: fixed-step time resource used to advance the lock timer.
/// - `state`: global state, used here for soft-drop vs hard-drop behavior.
/// - `lockdown`: the lock-delay timer resource.
/// - `blocked_auto_drop`: whether gravity already tried and failed to move
///   downward in this same fixed tick.
/// - `tetrominoes`: query for the active entity and component.
/// - `obstacles`: query for locked blocks.
///
/// Example:
/// if an `L` piece lands on the stack and stays grounded for the full lock
/// delay, this system converts its four cells into `Obstacle` entities.
pub fn deactivate_if_stuck(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    blocked_auto_drop: Res<BlockedAutoDrop>,
    tetrominoes: Query<(Entity, &Tetromino), With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    let waiting_before_lock = lockdown.0.is_some();
    let Ok((entity, tetromino)) = tetrominoes.single() else {
        lockdown.reset();
        return;
    };

    let mut candidate = *tetromino;
    candidate.shift(0, -1);
    // If the active piece can still fall, it is not stuck, so any old lock
    // countdown must be cancelled.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        lockdown.reset();
        return;
    }

    if waiting_before_lock && blocked_auto_drop.0 && state.manual_drop_gravity == SOFT_DROP_GRAVITY
    {
        // Replay-correct behavior: after a blocked automatic drop in soft mode,
        // do not also advance lockdown again in the exact same fixed tick.
        return;
    }

    // Advance or start the lock timer now that the piece is confirmed stuck.
    lockdown.start_or_advance(time);
    if !lockdown.just_finished() {
        return;
    }

    // The piece has fully locked in place. Remove the active entity and replace
    // it with four obstacle blocks that stay on the board permanently.
    commands.entity(entity).despawn();
    for &cell in tetromino.cells() {
        commands.spawn((
            Block {
                cell,
                color: tetromino.color,
            },
            Obstacle,
        ));
    }
    lockdown.reset();
}
```

### 7. Replace `spawn_next_tetromino`

```rust
/// NEW IMPLEMENTATION: Spawn the next tetromino if there is no active
/// tetromino. This should also update the next tetromino window.
///
/// Parameters:
/// - `commands`: command buffer used to spawn and despawn piece entities.
/// - `keyboard`: keyboard resource, so old one-frame edges can be cleared
///   before a fresh active piece appears.
/// - `state`: global game state containing the bag and gravity timer.
/// - `active_tetrominoes`: query used to check whether an active piece already
///   exists.
/// - `next_tetrominoes`: query for the logical preview tetromino entity only.
///   We intentionally require both `Next` and `Tetromino` so we do not despawn
///   the preview-board background tiles.
/// - `obstacles`: query for collision against placed blocks.
///
/// Example:
/// after the current piece locks, this system promotes the preview piece to the
/// active piece and spawns a fresh preview piece from the bag.
pub fn spawn_next_tetromino(
    mut commands: Commands,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<GameState>,
    active_tetrominoes: Query<Entity, With<Active>>,
    next_tetrominoes: Query<Entity, (With<Next>, With<Tetromino>)>,
    obstacles: Query<&Block, With<Obstacle>>,
) {
    if !active_tetrominoes.is_empty() {
        return;
    }

    // Remove only the old logical preview piece, not the 5x5 preview-board
    // tiles.
    for entity in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    // Pull the next canonical piece from the bag and move it to the board spawn
    // coordinates.
    let mut active = state.bag.next_tetromino();
    move_to_spawn_position(&mut active);

    // If even the spawn position collides, the game is over.
    if crate::there_is_collision(&active, obstacles) {
        commands.trigger(GameOver);
        return;
    }

    // Replay timing expects tiny spawn-boundary differences to be smoothed
    // slightly differently in hard-drop and soft-drop modes.
    let soft_spawn_smoothing = crate::rr::FIXED_FRAME_DURATION.mul_f32(0.5);
    let reset_for_hard_drop = state.manual_drop_gravity > SOFT_DROP_GRAVITY
        && state.gravity_timer.remaining() <= crate::rr::FIXED_FRAME_DURATION;
    let reset_for_soft_drop = state.manual_drop_gravity == SOFT_DROP_GRAVITY
        && state.gravity_timer.remaining() < soft_spawn_smoothing;
    if reset_for_hard_drop || reset_for_soft_drop {
        state.gravity_timer.reset();
    }

    // Clear inherited one-frame input edges before making the new piece active.
    clear_gameplay_inputs(&mut keyboard);
    commands.spawn((active, Active));

    // Peek the next bag piece, place it inside the 5x5 preview grid, and spawn
    // it with the `Next` marker.
    let mut next = state.bag.peek();
    next.shift(2, 2);
    commands.spawn((next, Next));
}
```

### 8. Replace `redraw_board`

```rust
pub fn redraw_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tetrominoes: Query<&Tetromino, With<Active>>,
    obstacles: Query<&Block, With<Obstacle>>,
    mut board: ResMut<Board>,
) {
    let mut colors = HashMap::<Entity, Color>::new();

    for tetromino in &tetrominoes {
        for &Cell(x, y) in tetromino.cells().iter().filter(|cell| cell.is_visible()) {
            colors.insert(board.cells[y as usize][x as usize], tetromino.color);
        }
    }

    for block in obstacles.iter().filter(|block| block.cell.is_visible()) {
        let Cell(x, y) = block.cell;
        colors.insert(board.cells[y as usize][x as usize], block.color);
    }

    // re-draw the whole board
    for entity in board.cells.iter_mut().flat_map(|row| row.iter_mut()) {
        commands.entity(*entity).insert(MeshMaterial2d(
            materials.add(colors.get(entity).copied().unwrap_or(BG_COLOR)),
        ));
    }
}
```

### 9. Replace `redraw_side_board`

```rust
/// NEW IMPLEMENTATION: Redraw the side board with the given marker component.
///
/// Parameters:
/// - `commands`: used to replace tile materials.
/// - `materials`: Bevy asset storage for color materials.
/// - `side_board`: the 5x5 preview board tiles for either `Next` or `Hold`.
/// - `tetromino`: the logical preview piece for the same marker, if one exists.
///
/// Example:
/// in the `Next` window, the four preview cells that belong to the upcoming
/// piece are painted with its color, and the other 21 cells stay black.
pub fn redraw_side_board<Marker: Component>(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    side_board: Query<(&mut Block, Entity), With<Marker>>,
    tetromino: Option<Single<&Tetromino, With<Marker>>>,
) {
    for (block, entity) in &side_board {
        // Compare each preview-board tile's logical `Cell` against the preview
        // tetromino's four cells. Matching tiles get the tetromino color.
        let color = tetromino
            .as_ref()
            .filter(|tetromino| tetromino.cells().contains(&block.cell))
            .map(|tetromino| tetromino.color)
            .unwrap_or(BG_COLOR);
        commands
            .entity(entity)
            .insert(MeshMaterial2d(materials.add(color)));
    }
}
```

## 3. Local checks

Run:

```bash
cargo test --features test data::tests -- --nocapture
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end i_spawn j_spawn shift1 l_rotate o_rotate --no-fail-fast
```
