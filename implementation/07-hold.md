# Hold

## Goal

Implement the `hold` feature by filling only the starter skeleton in:

- `Cargo.toml`
- `src/hold.rs`

This feature is a little more complex than RNG or hard drop because the actual
swap happens in `FixedUpdate`, but the `X` key press arrives in the ordinary
`Update` input flow.

So the clean minimal solution is:

1. queue the hold request in `Update`
2. consume that queued request in `FixedUpdate`
3. redraw the hold preview in `Update`

## Step 1: Enable the feature in `Cargo.toml`

Find this line in [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml):

```toml
enabled_features = ["config", "collision", "score", "rng", "hard_drop"]
```

Replace it with:

```toml
enabled_features = ["config", "collision", "score", "rng", "hard_drop", "hold"]
```

## Step 2: Add a tiny pending-hold resource

Near the top of [src/hold.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hold.rs), add:

```rust
/// A queued request to perform one hold swap on the next fixed frame.
#[derive(Resource, Default)]
pub struct PendingHold(bool);
```

Why this is needed:

- `swap_hold` runs in `FixedUpdate`
- key presses are observed naturally in `Update`
- the resource bridges those two schedules without adding big architecture

## Step 3: Add `queue_hold_input`

Add this system above `swap_hold`:

```rust
fn queue_hold_input(keyboard: Res<ButtonInput<KeyCode>>, mut pending: ResMut<PendingHold>) {
    // The hold feature uses X.
    if keyboard.just_pressed(KeyCode::KeyX) {
        pending.0 = true;
    }
}
```

Why:

- the request is remembered until `swap_hold` consumes it
- one boolean is enough because one press should cause one swap

## Step 4: Replace `swap_hold`

Find this starter code in [src/hold.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hold.rs):

```rust
pub fn swap_hold() {}
```

Replace it with:

```rust
pub fn swap_hold(
    mut commands: Commands,
    mut pending: ResMut<PendingHold>,
    mut state: ResMut<GameState>,
    active_tetrominoes: Query<(Entity, &Tetromino), With<Active>>,
    held_tetrominoes: Query<(Entity, &Tetromino), With<Hold>>,
    next_tetrominoes: Query<Entity, With<Next>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    if !pending.0 {
        return;
    }

    pending.0 = false;

    let Ok((active_entity, active_piece)) = active_tetrominoes.single() else {
        return;
    };

    // Recover the canonical tetromino by color.
    // This works because each tetromino type has a unique color.
    let canonical_from_color = |color: Color| {
        ALL_TETROMINO_TYPES
            .into_iter()
            .map(get_tetromino)
            .find(|tetromino| tetromino.color == color)
            .expect("every gameplay tetromino color should map to a canonical piece")
    };

    // Move a canonical tetromino into the hold preview window.
    let to_hold_window = |mut tetromino: Tetromino| {
        if tetromino.center == (0.5, -0.5) {
            tetromino.shift(2, 3);
        } else {
            tetromino.shift(2, 2);
        }
        tetromino
    };

    // Move a canonical tetromino into the normal board spawn position.
    let to_board_spawn = |mut tetromino: Tetromino| {
        if tetromino.center == (0.5, -0.5) {
            tetromino.shift(4, 19);
        } else {
            tetromino.shift(4, 18);
        }
        tetromino
    };

    // Move a canonical tetromino onto the board using the current active
    // piece's rounded center as the anchor.
    let to_board_position = |mut tetromino: Tetromino| {
        let dx = active_piece.center.0.round() as i32;
        let dy = active_piece.center.1.round() as i32;
        tetromino.shift(dx, dy);
        tetromino
    };

    // Resolve collision by moving the swapped-in piece up by at most 4 rows.
    let resolve_swap = |mut tetromino: Tetromino, obstacles: &mut Query<&Block, With<Obstacle>>| {
        for attempt in 0..=4 {
            if !crate::there_is_collision(&tetromino, obstacles.reborrow()) {
                return Some(tetromino);
            }

            if attempt < 4 {
                tetromino.shift(0, 1);
            }
        }

        None
    };

    let new_hold_piece = to_hold_window(canonical_from_color(active_piece.color));

    let held_piece = held_tetrominoes.iter().next().map(|(entity, tetromino)| {
        (entity, canonical_from_color(tetromino.color))
    });

    // If no piece is held yet, we preview the bag piece first and only consume
    // it after legality is confirmed.
    let consume_next_piece = held_piece.is_none();
    let swapped_in_canonical = held_piece
        .as_ref()
        .map(|(_, tetromino)| *tetromino)
        .unwrap_or_else(|| state.bag.peek());

    let candidate_piece = if state.manual_drop_gravity == HARD_DROP_GRAVITY {
        to_board_position(swapped_in_canonical)
    } else {
        to_board_spawn(swapped_in_canonical)
    };

    let Some(new_active_piece) =
        resolve_swap(candidate_piece, &mut obstacles)
    else {
        // Abort the hold when the swapped-in piece still collides after 4 kicks.
        return;
    };

    commands.entity(active_entity).despawn();

    if let Some((held_entity, _)) = held_piece {
        commands.entity(held_entity).despawn();
    }

    for entity in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    if consume_next_piece {
        let _ = state.bag.next_tetromino();
    }

    commands.spawn((new_active_piece, Active));
    commands.spawn((new_hold_piece, Hold));

    let mut next_piece = state.bag.peek();
    next_piece.shift(2, 2);
    commands.spawn((next_piece, Next));
}
```

### Why the color-to-canonical step is important

The hold window is showing the piece in its canonical preview form, not in
whatever rotated or shifted board state it had at the moment of the swap.

So if the active piece is an `L` somewhere on the board, we do **not** store
that exact world-space shape in the hold window.

Instead, we:

1. identify that it is an `L`
2. create a fresh canonical `L`
3. shift it into the hold preview window

That matches the expected hold snapshots.

### Why there are two board-placement rules

For ordinary hold behavior, the swapped-in piece should come back through the
normal spawn row.

Example:

- direct hold tests like `first_hold` and `next_hold` expect the incoming piece
  to appear at the usual top area

But when hard drop mode is enabled, the provided replay expects the held-in
piece to stay in the current gameplay region instead of jumping to the top.

Example:

- if hard drop is on and the current active piece is around center `(4.0, 6.0)`,
  the swapped-in piece should also appear around row `6`

So the clean minimal rule is:

1. if hard drop is off, use the normal spawn placement
2. if hard drop is on, use the current active piece's rounded center
3. in both cases, if it collides, kick it upward by up to 4 rows

## Step 5: Add doc comments to the public hold items

Add a short doc comment above:

- `setup_hold_window`
- `HoldPlugin`

This keeps the `missing_docs` warnings under control.

## Step 6: Finish the plugin

Replace the starter plugin body with:

```rust
impl Plugin for HoldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingHold>()
            .add_systems(Startup, setup_hold_window.in_set(Game))
            .add_systems(
                FixedUpdate,
                swap_hold.before(crate::board::gravity).in_set(Game),
            )
            .add_systems(
                Update,
                (queue_hold_input, redraw_side_board::<Hold>).in_set(Game),
            );
    }
}
```

Why this ordering matters:

- `queue_hold_input` runs in `Update` and remembers the key press
- `swap_hold` runs in `FixedUpdate`, which is where the real gameplay update
  happens
- `swap_hold.before(crate::board::gravity)` keeps the hold action ahead of the
  fixed gameplay fall step
- `redraw_side_board::<Hold>` stays in `Update` because it is just UI refresh

## Local checks

Run:

```bash
cargo fmt --all
```

Run:

```bash
cargo clippy --features test -- -D warnings
```

Run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end first_hold next_hold --no-fail-fast
```

If you also want a stronger replay-style signal later, run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end hard_drop_hold_0 --no-fail-fast
```

## Summary

This feature should end with:

- `X` queuing a hold request
- first hold taking the next bag piece as active
- later hold swapping active and held pieces
- legal-swap resolution by kicking up to 4 rows
- hold preview updating with canonical piece shapes
