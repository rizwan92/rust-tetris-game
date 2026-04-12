# Hold Copy-Paste Cheatsheet

Use this file after collision is complete and your preview/active spawning already works.

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

## Coordinate rule you must keep

Use these exact translations:

- active board piece -> hold window: `shift(-2, -16)`
- hold window piece -> active board: `shift(2, 16)`

This matches the coordinates the provided hold tests expect.

## `src/hold.rs`

### Replace `swap_hold`

File:
- [hold.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hold.rs)

```rust
pub fn swap_hold(
    // Read keyboard transitions for this fixed-update tick.
    keyboard: Res<ButtonInput<KeyCode>>,
    // Use commands to despawn old logical tetromino entities and spawn new ones.
    mut commands: Commands,
    // Access the bag when we need to consume the next piece.
    mut state: ResMut<GameState>,
    // Read the currently active tetromino entity and value.
    active: Query<(Entity, &Tetromino), With<Active>>,
    // Read the currently held tetromino entity and value, if one exists.
    held: Query<(Entity, &Tetromino), With<Hold>>,
    // Read the logical next tetromino entity so we can refresh it when the bag advances.
    next_tetrominoes: Query<Entity, (With<Next>, With<Tetromino>)>,
    // Access obstacles for collision checks.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Only react when X is pressed.
    if !keyboard.just_pressed(KeyCode::KeyX) {
        // Exit immediately on all other frames.
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
        // Shift from board coordinates into hold-window coordinates.
        tetromino.shift(-2, -16);
        // Return the translated tetromino.
        tetromino
    };

    // Convert a held tetromino back into board coordinates.
    let to_board = |mut tetromino: Tetromino| {
        // Shift from hold-window coordinates back into board coordinates.
        tetromino.shift(2, 16);
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
        // Translate the held piece back into board coordinates.
        let candidate = to_board(*held_piece);

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
    let mut candidate = state.bag.peek();
    // Move the peeked piece into the standard board spawn position.
    candidate.shift(4, BOARD_HEIGHT as i32 - 1 - candidate.bounds().top);

    // Abort without consuming the bag if the candidate cannot be placed.
    let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
        // Leave active, hold, and next unchanged on failure.
        return;
    };

    // Consume the same next piece that we just validated via peek().
    state.bag.next_tetromino();

    // Remove the old active entity.
    commands.entity(active_entity).despawn();
    // Move the old active piece into the hold window.
    commands.spawn((to_hold(active_piece), Hold));
    // Spawn the validated next piece as the new active tetromino.
    commands.spawn((candidate, Active));

    // Refresh the logical next tetromino because the bag has advanced.
    for entity in &next_tetrominoes {
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
- The hold tests expect the active piece moved into hold to preserve its board-relative shape, not reset to canonical local coordinates.
- This system is registered in `FixedUpdate`, so use `just_pressed` exactly once per swap trigger.

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
