# Hold Copy-Paste Cheatsheet

Use this file after collision is complete and your preview/active spawning already works.

## Commenting Rule For This File

- hold is the most timing-sensitive feature, so the comments are intentionally extra literal
- changed parameters are explained directly in the function signature
- if one line looks strange, read the comment above it before changing it

## Enable the feature

Set [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml) to:

```toml
enabled_features = ["config", "collision", "score", "rng", "hard_drop", "hold"]
```

## What this feature changes

- lets the player press `X` to hold the active piece
- moves the current active piece into the hold window
- swaps an already held piece back into active play
- resolves spawn collisions by kicking upward up to 4 times
- updates the logical `Next` tetromino when the bag is consumed during first hold

## Coordinate rules you must keep

Use these validated rules:

- board piece -> hold window:
  - center `(2.5, 2.5)` for `I` and `O`
  - center `(2.0, 2.0)` for every other piece
- hold piece -> board:
  - `I` uses a different `y` shift depending on whether it is horizontal or vertical
  - `O` uses rounded center alignment
  - `T`, `L`, `J`, `S`, `Z` use floored center alignment
- if there is no held piece yet:
  - use the currently displayed logical `Next` tetromino as the source
  - do not rebuild that source from scratch before the swap

## `src/hold.rs`

### Update `HoldPlugin`

Use `PostUpdate` in the validated version:

```rust
impl Plugin for HoldPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_hold_window.in_set(Game))
            .add_systems(PostUpdate, swap_hold.in_set(Game))
            .add_systems(PostUpdate, redraw_side_board::<Hold>.in_set(Game));
    }
}
```

### Replace `swap_hold`

File:
- [hold.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hold.rs)

```rust
pub fn swap_hold(
    // Read keyboard state for this update tick.
    keyboard: Res<ButtonInput<KeyCode>>,
    // Use commands to despawn old logical tetromino entities and spawn new ones.
    mut commands: Commands,
    // Access the bag when we need to consume the next piece.
    mut state: ResMut<GameState>,
    // Read the currently active tetromino entity and value.
    active: Query<(Entity, &Tetromino), With<Active>>,
    // Read the currently held tetromino entity and value, if one exists.
    held: Query<(Entity, &Tetromino), With<Hold>>,
    // Read the logical next tetromino entity and value.
    // We need the value too because first hold uses the currently shown Next piece.
    next_tetrominoes: Query<(Entity, &Tetromino), With<Next>>,
    // Access obstacles for collision checks.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Only react to the exact frame when X was pressed.
    // This matches the validated implementation in the real code.
    if !keyboard.just_pressed(KeyCode::KeyX) {
        // Exit on all frames where X was not newly pressed.
        return;
    }

    // We cannot hold anything when there is no active piece.
    let Ok((active_entity, active_piece)) = active.single() else {
        // Exit when nothing is active.
        return;
    };

    // Copy the active piece so we can reuse it after despawning its entity.
    let active_piece = *active_piece;

    // Convert a board-position tetromino into hold-window coordinates.
    let to_hold = |mut tetromino: Tetromino| {
        // Detect the I piece by checking whether all cells lie on one row or one column.
        let is_i = tetromino.cells.iter().all(|cell| cell.0 == tetromino.cells[0].0)
            || tetromino.cells.iter().all(|cell| cell.1 == tetromino.cells[0].1);
        // I and O pieces need the centered preview position at (2.5, 2.5).
        let target_center = if tetromino.is_o() || is_i {
            (2.5, 2.5)
        } else {
            // All other tetrominos use the preview center at (2.0, 2.0).
            (2.0, 2.0)
        };
        // Translate the board piece into the hold-window target center.
        tetromino.shift(
            (target_center.0 - tetromino.center.0).round() as i32,
            (target_center.1 - tetromino.center.1).round() as i32,
        );
        // Return the translated tetromino.
        tetromino
    };

    // Convert a held tetromino back into board coordinates.
    let to_board = |mut tetromino: Tetromino| {
        // Detect whether the I piece is vertical.
        let vertical_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.0 == tetromino.cells[0].0);
        // Detect whether the I piece is horizontal.
        let horizontal_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.1 == tetromino.cells[0].1);
        // Use the validated y-shift for each I-orientation case.
        let y_shift = if vertical_i {
            13
        } else if horizontal_i {
            15
        } else {
            // All non-I pieces use this normal hold-to-board y shift.
            14
        };
        // Apply the board translation.
        tetromino.shift(2, y_shift);
        // Return the translated tetromino.
        tetromino
    };

    // Try to resolve collision by moving upward up to four times.
    let try_resolve = |mut tetromino: Tetromino, obstacles: &mut Query<&Block, With<Obstacle>>| {
        // Attempt the original position plus four upward kicks.
        for _ in 0..=4 {
            // Return the candidate immediately when it is legal.
            if !crate::there_is_collision(&tetromino, obstacles.reborrow()) {
                // Successful placement.
                return Some(tetromino);
            }

            // Otherwise try one row higher.
            tetromino.shift(0, 1);
        }

        // Give up after the fifth failed attempt.
        None
    };

    // Case 1: there is already a held piece, so swap it back into active play.
    if let Ok((held_entity, held_piece)) = held.single() {
        // Start from the held tetromino value.
        let candidate = *held_piece;
        // Detect whether this held piece is vertical I.
        let vertical_i = candidate
            .cells
            .iter()
            .all(|cell| cell.0 == candidate.cells[0].0);
        // Detect whether this held piece is horizontal I.
        let horizontal_i = candidate
            .cells
            .iter()
            .all(|cell| cell.1 == candidate.cells[0].1);
        // Choose the validated board-placement rule for this shape.
        let candidate = if vertical_i || horizontal_i {
            // I pieces should go through the special hold-to-board helper.
            to_board(candidate)
        } else if candidate.is_o() {
            // O pieces use rounded center alignment.
            let mut candidate = candidate;
            candidate.shift(
                (active_piece.center.0 - candidate.center.0).round() as i32,
                (active_piece.center.1 - candidate.center.1).round() as i32,
            );
            candidate
        } else {
            // The other pieces use floored center alignment.
            let mut candidate = candidate;
            candidate.shift(
                (active_piece.center.0 - candidate.center.0).floor() as i32,
                (active_piece.center.1 - candidate.center.1).floor() as i32,
            );
            candidate
        };

        // Abort the whole swap when the held piece cannot be placed legally.
        let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
            // Leave the current state unchanged on failure.
            return;
        };

        // Remove the old active entity.
        commands.entity(active_entity).despawn();
        // Remove the old held entity.
        commands.entity(held_entity).despawn();
        // Spawn the old active piece in hold coordinates.
        commands.spawn((to_hold(active_piece), Hold));
        // Spawn the held piece back as the new active piece.
        commands.spawn((candidate, Active));
        // End the system after a successful swap.
        return;
    }

    // Case 2: there is no held piece yet, so the next bag piece becomes active.
    let Ok((_, next_piece)) = next_tetrominoes.single() else {
        // Exit if the logical next piece cannot be found.
        return;
    };
    // Start from the currently displayed logical next piece.
    let candidate = *next_piece;
    // Detect whether this next piece is vertical I.
    let vertical_i = candidate
        .cells
        .iter()
        .all(|cell| cell.0 == candidate.cells[0].0);
    // Detect whether this next piece is horizontal I.
    let horizontal_i = candidate
        .cells
        .iter()
        .all(|cell| cell.1 == candidate.cells[0].1);
    // Apply the same validated shape-specific placement rules.
    let candidate = if vertical_i || horizontal_i {
        // The preview I piece needs a one-cell lift before hold-to-board conversion.
        let mut candidate = candidate;
        candidate.shift(0, 1);
        to_board(candidate)
    } else if candidate.is_o() {
        // O pieces use rounded center alignment.
        let mut candidate = candidate;
        candidate.shift(
            (active_piece.center.0 - candidate.center.0).round() as i32,
            (active_piece.center.1 - candidate.center.1).round() as i32,
        );
        candidate
    } else {
        // The other pieces use floored center alignment.
        let mut candidate = candidate;
        candidate.shift(
            (active_piece.center.0 - candidate.center.0).floor() as i32,
            (active_piece.center.1 - candidate.center.1).floor() as i32,
        );
        candidate
    };

    // Abort without consuming the bag if the candidate cannot be placed.
    let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
        // Leave active, hold, and next unchanged on failure.
        return;
    };

    // Consume the bag only after the placement is confirmed legal.
    state.bag.next_tetromino();

    // Remove the old active entity.
    commands.entity(active_entity).despawn();
    // Move the old active piece into the hold window.
    commands.spawn((to_hold(active_piece), Hold));
    // Spawn the validated next piece as the new active tetromino.
    commands.spawn((candidate, Active));

    // Refresh the logical next tetromino because the bag has advanced.
    for (entity, _) in &next_tetrominoes {
        // Remove the stale logical preview entity.
        commands.entity(entity).despawn();
    }

    // Build the new preview tetromino from the updated bag front.
    let mut next_piece = state.bag.peek();
    // Shift it into the side-window coordinates.
    next_piece.shift(2, 2);
    // Spawn the refreshed logical preview tetromino.
    commands.spawn((next_piece, Next));
}
```

## Notes

- Do not eagerly consume the bag before you know the replacement active piece can be placed.
- The validated version of this feature runs in `PostUpdate`.
- The hold tests expect the hold preview to preserve the current orientation, but to be re-centered into the hold window.
- The first-hold path should use the logical `Next` tetromino already on screen.
- The swap logic is shape-sensitive, especially for `I` and `O`.

## Test commands

Start with the smallest hold-specific checks:

```bash
cargo test --features test --test end_to_end first_hold -- --test-threads=1
cargo test --features test --test end_to_end next_hold -- --test-threads=1
```

Then run the whole hold file:

```bash
cargo test --features test --test end_to_end hold -- --test-threads=1
```

Then run the cumulative regression sweep:

```bash
cargo test --features test --test end_to_end -- --test-threads=1
```

## Acceptance checkpoint

You are done with the required features when:

- `first_hold` passes
- `next_hold` passes
- the hold test file passes
- the cumulative suite still passes with `enabled_features = ["config", "collision", "score", "rng", "hard_drop", "hold"]`
