# PR 7 Guide: Hold

This is the most timing-sensitive feature after baseline.

It is also one of the most interesting features, because it mixes:

- game rules
- board positioning
- preview/hold windows
- Bevy input timing

So this guide goes slower on purpose.

## What this PR is trying to achieve

At the end of this PR, pressing `X` should let the player:

- move the current active piece into the hold window
- bring a held piece back into active play
- if there is no held piece yet:
  - use the current `Next` piece as the replacement
- keep the hold preview updated

It also must follow one important safety rule:

- if the swapped-in piece does not fit, try moving it upward up to 4 times
- if it still does not fit, cancel the swap

That is the whole feature.

## Starter files to compare

- `original-repo/src/hold.rs`
- `original-repo/docs/hold.md`

## File you will change

- `src/hold.rs`

## Feature flag state

Use:

```toml
enabled_features = ["config", "collision", "score", "rng", "hard_drop", "hold"]
```

## Very important dependency from baseline

This feature depends on the shared activation helper that baseline already added
in `src/board.rs`:

- `TimingMode`
- `ActivationSource`
- `activate_tetromino(...)`

Why this matters:

when hold creates a new active piece, we do **not** want hold to invent a
separate timing system.

We want:

- normal spawn
- hold swap spawn

to follow the same activation rules.

That is why this guide reuses the board helper instead of making hold manage its
own spawn timing.

## Mental model before touching code

Hold has 2 main gameplay paths.

### Path 1: first hold

There is no held piece yet.

So pressing `X` should:

1. move the active piece into the hold window
2. take the currently displayed `Next` piece
3. make that next piece the new active piece
4. consume the bag only after that replacement is confirmed legal
5. rebuild the preview

### Path 2: swap with an existing held piece

There is already a held piece.

So pressing `X` should:

1. move the active piece into the hold window
2. take the held piece out
3. try to place it on the board
4. if it fits, make it active
5. if it does not fit even after upward kicks, cancel the swap

That is the whole logic at a high level.

## Why this feature is timing-sensitive

The starter file makes hold sound like:

- "if X is pressed, swap"

But in Bevy, input timing and fixed-step timing can miss each other.

That means:

- the user can press `X`
- but the gameplay system may not consume it at the right moment

The smallest reliable fix is:

1. store the request in a resource
2. try to consume it in `FixedUpdate`
3. keep a `PostUpdate` fallback

That is why this guide uses `PendingHold`.

## Step 1: add `PendingHold` and `queue_hold_input`

### Why this step exists

We need to remember:

- "the player pressed X"

until the gameplay logic actually consumes that request.

### What to do

Open `src/hold.rs`.

Find the `Hold` component near the top.

Directly below it, paste this block:

```rust
/// Whether the player requested a hold swap this frame.
#[derive(Resource, Default)]
pub struct PendingHold(pub bool);

fn queue_hold_input(
    // Read keyboard transitions for the current schedule.
    keyboard: Res<ButtonInput<KeyCode>>,
    // Store a pending hold request until gameplay logic consumes it.
    mut pending_hold: ResMut<PendingHold>,
) {
    // Remember the X press so either the fixed-step or frame-step hold system can use it.
    if keyboard.just_pressed(KeyCode::KeyX) {
        pending_hold.0 = true;
    }
}
```

### What this does in simple English

This says:

- if the player pressed `X`
- remember that fact in a tiny resource

Then later:

- `swap_hold` checks that resource
- consumes it once
- performs the real swap

## Step 2: understand the 3 helper ideas inside `swap_hold`

Before pasting the full function, understand the 3 helper ideas it uses.

### Helper idea 1: `to_hold(...)`

This moves a board tetromino into hold-window coordinates.

Important rule:

- `I` and `O` use center `(2.5, 2.5)`
- all other pieces use center `(2.0, 2.0)`

### Helper idea 2: `to_board(...)`

This moves a held tetromino from hold-window coordinates back into board
coordinates.

Important rule:

- `I` is special and needs different `y` shifts depending on orientation

### Helper idea 3: `try_resolve(...)`

This is the collision-resolution helper.

It tries:

- original position
- then up by 1
- then up by 2
- then up by 3
- then up by 4

If none are legal:

- return `None`
- abort the swap

That helper is what makes hold collision recovery work.

## Step 3: replace `swap_hold`

### Why this is the big step

This one function owns almost all hold gameplay behavior:

- first hold
- swap hold
- collision resolution
- preview refresh
- shared activation helper usage

So it is normal that the function is long.

### What to replace

In `src/hold.rs`, replace the starter `swap_hold` with this full function:

```rust
pub fn swap_hold(
    // Read and clear the pending hold request.
    mut pending_hold: ResMut<PendingHold>,
    // Read the time strategy so the shared activation helper can choose replay vs runtime timing.
    time_strategy: Res<bevy::time::TimeUpdateStrategy>,
    // Use commands to despawn and respawn the logical tetromino entities.
    mut commands: Commands,
    // Access the bag because first hold consumes the current next piece.
    mut state: ResMut<GameState>,
    // Read the current active piece and its entity id.
    active: Query<(Entity, &Tetromino), With<Active>>,
    // Read the current held piece and its entity id, if it exists.
    held: Query<(Entity, &Tetromino), With<Hold>>,
    // Read the logical Next entity and value.
    // The value matters because first hold uses the currently displayed next piece.
    next_tetrominoes: Query<(Entity, &Tetromino), With<Next>>,
    // Access obstacles so we can reject illegal swap results.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Only run when a queued hold input is waiting to be processed.
    if !pending_hold.0 {
        return;
    }
    // Consume the queued request so we do not swap twice in one frame.
    pending_hold.0 = false;

    // Stop immediately when there is no active piece.
    let Ok((active_entity, active_piece)) = active.single() else {
        return;
    };
    let active_piece = *active_piece;
    // Normalize Bevy's time strategy once so both hold paths use the same activation rules.
    let timing_mode = crate::board::timing_mode(&time_strategy);

    let to_hold = |mut tetromino: Tetromino| {
        // Detect the I piece by checking whether all cells are on one row or column.
        let is_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.0 == tetromino.cells[0].0)
            || tetromino
                .cells
                .iter()
                .all(|cell| cell.1 == tetromino.cells[0].1);
        // I and O pieces use the centered `(2.5, 2.5)` preview position.
        let target_center = if tetromino.is_o() || is_i {
            (2.5, 2.5)
        } else {
            // The other pieces use the `(2.0, 2.0)` preview position.
            (2.0, 2.0)
        };
        // Translate the piece from board space into hold-window space.
        tetromino.shift(
            (target_center.0 - tetromino.center.0).round() as i32,
            (target_center.1 - tetromino.center.1).round() as i32,
        );
        tetromino
    };

    let to_board = |mut tetromino: Tetromino| {
        // Detect whether the I piece is vertical in the hold window.
        let vertical_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.0 == tetromino.cells[0].0);
        // Detect whether the I piece is horizontal in the hold window.
        let horizontal_i = tetromino
            .cells
            .iter()
            .all(|cell| cell.1 == tetromino.cells[0].1);
        // Different I orientations need different y-shifts to line up correctly on the board.
        let y_shift = if vertical_i {
            13
        } else if horizontal_i {
            15
        } else {
            // All non-I pieces use the normal hold-to-board y shift.
            14
        };
        // Translate the held piece into board coordinates.
        tetromino.shift(2, y_shift);
        tetromino
    };

    let try_resolve = |mut tetromino: Tetromino, obstacles: &mut Query<&Block, With<Obstacle>>| {
        // Try the original placement plus up to four upward kicks.
        for _ in 0..=4 {
            if !crate::there_is_collision(&tetromino, obstacles.reborrow()) {
                return Some(tetromino);
            }
            // Move the candidate up by one row and try again.
            tetromino.shift(0, 1);
        }
        // Give up if all five attempts are illegal.
        None
    };

    if let Ok((held_entity, held_piece)) = held.single() {
        // Start from the currently held tetromino value.
        let candidate = *held_piece;
        // Detect whether the held piece is a vertical I.
        let vertical_i = candidate
            .cells
            .iter()
            .all(|cell| cell.0 == candidate.cells[0].0);
        // Detect whether the held piece is a horizontal I.
        let horizontal_i = candidate
            .cells
            .iter()
            .all(|cell| cell.1 == candidate.cells[0].1);
        // Choose the correct board-placement rule for this shape.
        let candidate = if vertical_i || horizontal_i {
            if active_piece.is_o() {
                // When the current active piece is O, align the held I using rounded centers.
                // This matches the recorded swap where I takes over the O piece's board position.
                let mut candidate = candidate;
                candidate.shift(
                    (active_piece.center.0 - candidate.center.0).round() as i32,
                    (active_piece.center.1 - candidate.center.1).round() as i32,
                );
                candidate
            } else {
                to_board(candidate)
            }
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
        // Abort the swap if the held piece still cannot be placed legally.
        let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
            return;
        };

        // Replace the active/held pair with the swapped values.
        commands.entity(active_entity).despawn();
        commands.entity(held_entity).despawn();
        commands.spawn((to_hold(active_piece), Hold));
        // Activate the swapped-in piece through the shared board helper so hold
        // swaps and normal bag spawns use the same lifecycle entry rules.
        activate_tetromino(
            &mut commands,
            &mut state,
            candidate,
            ActivationSource::HoldSwap,
            timing_mode,
            None,
            &mut obstacles,
        );
        return;
    }

    let Ok((_, next_piece)) = next_tetrominoes.single() else {
        return;
    };
    // Start from the logical Next tetromino already shown by the game.
    let candidate = *next_piece;
    // Detect whether the next piece is a vertical I.
    let vertical_i = candidate
        .cells
        .iter()
        .all(|cell| cell.0 == candidate.cells[0].0);
    // Detect whether the next piece is a horizontal I.
    let horizontal_i = candidate
        .cells
        .iter()
        .all(|cell| cell.1 == candidate.cells[0].1);
    // Choose the validated placement rule for the first-hold replacement piece.
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

    // Abort before consuming the bag if the placement is illegal.
    let Some(candidate) = try_resolve(candidate, &mut obstacles) else {
        return;
    };

    // Consume the bag only after the replacement piece is confirmed legal.
    state.bag.next_tetromino();

    // Move the active piece into hold and make the candidate active.
    commands.entity(active_entity).despawn();
    commands.spawn((to_hold(active_piece), Hold));
    // Use the same activation helper here too so first-hold timing matches later swaps.
    activate_tetromino(
        &mut commands,
        &mut state,
        candidate,
        ActivationSource::HoldSwap,
        timing_mode,
        None,
        &mut obstacles,
    );

    // Refresh the logical Next piece because the bag front changed.
    for (entity, _) in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    // Rebuild the preview from the new bag front and move it into preview coordinates.
    let mut next_piece = state.bag.peek();
    next_piece.shift(2, 2);
    commands.spawn((next_piece, Next));
}
```

## Step 4: understand the two big branches inside `swap_hold`

### Branch 1: swap with an existing held piece

This branch starts here:

```rust
if let Ok((held_entity, held_piece)) = held.single() {
```

Meaning:

- there is already something in the hold window

So the code:

1. reads the held piece
2. converts it into board coordinates
3. tries to resolve collisions by kicking upward
4. if successful:
   - despawns old active
   - despawns old held
   - spawns old active into hold
   - activates held piece on board

### Branch 2: first hold

If there is no held piece yet, we go to the second half.

That branch:

1. uses the currently displayed `Next` piece
2. converts it into board coordinates
3. checks whether that placement is legal
4. **only then** consumes the bag
5. moves the old active piece into hold
6. rebuilds the `Next` preview

That "only then" part is very important.

## Step 5: understand why the bag is consumed late in first hold

This line happens only after legality is confirmed:

```rust
state.bag.next_tetromino();
```

Why is that important?

Because the assignment specifically warns:

- do not eagerly pop the next piece

If you consume the bag too early and the swap later fails:

- your preview and bag state become wrong

So the correct order is:

1. build candidate
2. test candidate legality
3. only then consume the bag

## Step 6: understand the coordinate rules

This feature has several shape-specific coordinate rules.

You do **not** need to invent them yourself if you follow the final code.

### Rule A: board piece -> hold window

Use `to_hold(...)`:

- `I` and `O` center to `(2.5, 2.5)`
- others center to `(2.0, 2.0)`

### Rule B: hold piece -> board

Use `to_board(...)`:

- vertical `I` uses one `y` shift
- horizontal `I` uses another `y` shift
- others use the normal board shift

### Rule C: alignment by piece type

When a held or preview piece returns to the board:

- `O` uses rounded center alignment
- `I` has special handling
- the other shapes use floored center alignment

That is why the code looks a little shape-specific.

It is not random.
It is matching the validated working placements.

## Step 7: understand `try_resolve(...)`

This helper is the collision-recovery part of the feature.

It does this:

1. try original placement
2. if illegal, move up 1 row
3. try again
4. repeat until 4 upward kicks have been tried

If all attempts fail:

- return `None`
- cancel the swap

### Example

Suppose the swapped-in piece overlaps obstacles at the original position.

Then `try_resolve(...)` will test:

- original position
- original + 1 row up
- original + 2 rows up
- original + 3 rows up
- original + 4 rows up

If one of those is legal, the swap is allowed.

## Step 8: update `HoldPlugin`

### Why this step exists

Like every feature plugin, this is what actually wires your systems into the
app.

If you forget it:

- the helper resource exists
- the functions exist
- but the feature still does not work

### What to replace

Replace the plugin build function with:

```rust
impl Plugin for HoldPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<PendingHold>()
            .add_systems(Startup, setup_hold_window.in_set(Game))
            .add_systems(Update, queue_hold_input.in_set(Game))
            .add_systems(
                FixedUpdate,
                // Read hold input during the fixed-step gameplay loop so it can affect
                // the currently active piece before lock/spawn logic runs.
                swap_hold.in_set(Game).before(crate::board::gravity),
            )
            // Fall back to frame-step processing when no fixed-step run happened this frame.
            .add_systems(PostUpdate, swap_hold.in_set(Game))
            .add_systems(PostUpdate, redraw_side_board::<Hold>.in_set(Game));
    }
}
```

## Step 9: understand why the plugin uses both `FixedUpdate` and `PostUpdate`

This is one of the most important timing details in the feature.

### `Update`

`queue_hold_input` runs here to remember that `X` was pressed.

### `FixedUpdate`

`swap_hold` runs here first, before gravity.

That gives the hold action a chance to affect the current piece during the main
gameplay step.

### `PostUpdate`

`swap_hold` runs again as a fallback.

Why?

Because sometimes no fixed-step run happens in that frame, and we still do not
want to lose the `X` press.

So the timing logic is:

1. remember input early
2. try gameplay swap in fixed-step
3. if no fixed-step handled it, allow fallback later

That is why `PendingHold` exists.

## Common beginner confusion here

### "Why is this feature so much longer than hard drop?"

Because hold changes several things at once:

- board active piece
- hold preview
- next preview
- sometimes bag state
- collision resolution
- timing of the activation path

So naturally it needs more code.

### "Why not just move the held piece to the old active center directly?"

Because shape-specific alignment matters.

Different pieces use:

- different centers
- different rounding/flooring rules
- special `I` placement behavior

### "Why do we use the board activation helper here?"

Because a hold-created active piece is still just:

- a new active piece

So it should follow the same activation lifecycle rules as normal spawns.

## Tests for this PR

### Hold-focused checks

Run:

```bash
cargo nextest run --features test,config,collision,score,rng,hard_drop,hold --test end_to_end \
  first_hold next_hold hard_drop_hold_0 z_rotate_hold_arrow \
  --no-fail-fast
```

These tests check:

- first hold behavior
- later hold swap behavior
- hold interaction with hard drop
- hold interaction with movement/rotation timing

### Full final confirmation

Then rerun the full suite:

```bash
cargo nextest run --features test --retries 1 --test-threads=1 --test end_to_end --no-fail-fast
```

## When this PR is done

Stop this PR when:

- first hold works
- swapping with an existing held piece works
- failed swaps are correctly aborted
- upward collision resolution works up to 4 kicks
- the next preview updates correctly after first hold
- hold-specific tests are green

This is the last required feature PR in the main sequence.
