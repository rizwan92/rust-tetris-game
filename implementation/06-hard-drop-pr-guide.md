# PR 6 Guide: Hard Drop

This PR adds a feature the player can feel immediately:

- press `Z`
- hard drop turns on or off
- down-arrow behavior changes

So this PR is small, but satisfying.

At the same time, it is important to understand one thing clearly:

the real hard-drop movement logic was already prepared in baseline.

This PR mostly adds:

- the toggle
- the state update
- the on-screen text update

## What this PR is trying to achieve

At the end of this PR:

- `Z` should toggle hard drop on/off
- the game state should switch between:
  - soft drop gravity
  - hard drop gravity
- the UI should show:
  - `Hard Drop: Off`
  - `Hard Drop: On`

That is the whole feature.

## Starter files to compare

- `original-repo/src/hard_drop.rs`
- `original-repo/docs/hard_drop.md`

## File you will change

- `src/hard_drop.rs`

## Feature flag state

Use:

```toml
enabled_features = ["config", "collision", "score", "rng", "hard_drop"]
```

## Very important note before you start

If you followed the baseline guide exactly, then `src/board.rs` already knows
how to do the actual downward movement correctly.

That baseline code already does these things:

- loops `state.manual_drop_gravity` times when down-arrow is pressed
- stops early if the next downward step would collide
- remembers whether the piece was manually dropped
- uses the correct lock-timer path afterward

So this PR is **not** where we teach the board how to move fast.

This PR is where we teach the game:

- when hard drop is enabled
- what gravity value to use for manual drop
- what text to show on screen

That is why this PR is mainly in `src/hard_drop.rs`.

## Mental model before touching code

This feature is easiest to understand as 3 tiny systems:

1. toggle system
   - reads `Z`
   - flips a boolean

2. gameplay-state system
   - reads that boolean
   - updates `GameState.manual_drop_gravity`

3. UI system
   - reads that boolean
   - updates visible text

That is the whole design.

## Step 1: understand the two gravity values

Inside `src/data.rs` you already have:

- `SOFT_DROP_GRAVITY`
- `HARD_DROP_GRAVITY`

In plain English:

- soft drop means "move downward a little"
- hard drop means "try to move downward many times quickly"

The board input logic does not care why the gravity value changed.

It only cares about:

- what number is currently stored in `state.manual_drop_gravity`

That is why this PR only needs to update that state value.

## Step 2: replace the imports at the top of `src/hard_drop.rs`

### Why this step exists

The starter file does not yet import the game state or the gravity constants.

But this feature needs:

- `Game` for scheduling
- `GameState` for changing the manual drop gravity
- `HARD_DROP_GRAVITY` and `SOFT_DROP_GRAVITY` for the actual values

### What to replace

At the top of `src/hard_drop.rs`, replace the imports with:

```rust
use bevy::prelude::*;

use crate::{
    Game,
    data::{GameState, HARD_DROP_GRAVITY, SOFT_DROP_GRAVITY},
    ui::*,
};
```

## Step 3: add the three hard-drop systems

### Why this step exists

The starter file tells you in comments what needs to exist, but does not provide
the systems.

We will now add the 3 small systems directly under `setup_status_text`.

### What to paste

Paste this block below `setup_status_text`:

```rust
fn toggle_hard_drop(
    // Read keyboard transitions for this frame.
    keyboard: Res<ButtonInput<KeyCode>>,
    // Mutably access the single hard-drop toggle component.
    mut hard_drop: Single<&mut HardDrop>,
) {
    // Flip the boolean only on the exact frame when Z is pressed.
    if keyboard.just_pressed(KeyCode::KeyZ) {
        hard_drop.0 = !hard_drop.0;
    }
}

fn update_manual_drop_gravity(
    // Read the hard-drop toggle only when it changes.
    hard_drop: Single<&HardDrop, Changed<HardDrop>>,
    // Update the manual-drop gravity value stored in game state.
    mut state: ResMut<GameState>,
) {
    // When hard drop is enabled, use the large manual drop distance.
    // Otherwise keep the normal one-row manual drop behavior.
    state.manual_drop_gravity = if hard_drop.0 {
        HARD_DROP_GRAVITY
    } else {
        SOFT_DROP_GRAVITY
    };
}

fn update_status_text(
    // Read the hard-drop state and the UI text together when the state changes.
    mut status: Single<(&HardDrop, &mut Text), Changed<HardDrop>>,
) {
    // Convert the boolean into a user-friendly label.
    let label = if status.0.0 { "On" } else { "Off" };
    // Refresh the visible text so the UI matches the stored state.
    status.1.0 = format!("Hard Drop: {label}");
}
```

## Step 4: understand the 3 systems one by one

### System 1: `toggle_hard_drop`

This system does exactly one thing:

- if `Z` was just pressed
- flip the boolean

So:

- `false` becomes `true`
- `true` becomes `false`

That is the toggle behavior.

### System 2: `update_manual_drop_gravity`

This system turns the boolean into real gameplay behavior.

If hard drop is:

- on -> use `HARD_DROP_GRAVITY`
- off -> use `SOFT_DROP_GRAVITY`

This is the bridge between:

- UI/input state
- actual game movement

### System 3: `update_status_text`

This system is just visual feedback.

It turns:

- `true` into `"On"`
- `false` into `"Off"`

and updates the on-screen text.

That way the player can see the mode clearly.

## Step 5: understand why `Changed<HardDrop>` is good here

Two of the systems use:

```rust
Changed<HardDrop>
```

Why?

Because we only need to update:

- the manual drop gravity
- the text

when the toggle actually changes.

That means:

- fewer unnecessary updates
- cleaner logic

The toggle system itself runs every frame and listens for `Z`.
The other two only react when the value changed.

## Step 6: replace `HardDropPlugin::build`

### Why this step exists

The plugin is what actually installs your systems into the game.

If you forget this step, your new functions exist in the file, but they never
run.

### What to replace

Replace the starter plugin build function with:

```rust
impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_text.in_set(Game))
            .add_systems(
                Update,
                (
                    toggle_hard_drop,
                    update_manual_drop_gravity,
                    update_status_text,
                )
                    // Keep the systems in a predictable order:
                    // toggle first, then update the gameplay value, then update the UI text.
                    .chain()
                    .in_set(Game),
            );
    }
}
```

## Step 7: understand the order inside the plugin

The systems are chained in this order:

1. `toggle_hard_drop`
2. `update_manual_drop_gravity`
3. `update_status_text`

That order matters.

Why?

Because:

- first we flip the boolean
- then we update the gameplay state from the new boolean
- then we update the UI text from the new boolean

So one key press causes one clean sequence.

## What this feature fixes conceptually

Before this PR:

- down-arrow always behaves the same
- there is no hard-drop mode
- the hard-drop status text never changes

After this PR:

- `Z` flips the mode
- the game state changes from soft-drop gravity to hard-drop gravity
- the UI shows whether hard drop is on or off

That is exactly what the assignment asked for.

## Common beginner confusion here

### "Why are we not editing `board.rs` here?"

Because if you followed the baseline guide, `board.rs` already uses:

```rust
state.manual_drop_gravity
```

So `board.rs` is already waiting for this PR.

This PR just changes that number.

### "Why is this called hard drop if down-arrow is still used?"

Because in this project, hard drop is implemented as:

- changing how strongly down-arrow behaves

So:

- same key
- different gravity value

### "Why do we need a separate UI system?"

Because gameplay state and text rendering are different responsibilities.

One changes the logic.
The other changes what the player sees.

## Tests for this PR

### Hard-drop-specific recorded tests

Run:

```bash
cargo nextest run --features test,config,collision,score,rng,hard_drop --test end_to_end \
  hard_drop_deterministic hard_drop_deterministic2 hard_drop_67 hard_drop_727 \
  hard_drop_hold_0 \
  --no-fail-fast
```

These tests make sure:

- the toggle affects replay behavior correctly
- manual drop gravity changes correctly
- the feature still behaves correctly in more complex recordings

### Small realtime regression check

Then rerun:

```bash
cargo nextest run --features test,config,collision,score,rng,hard_drop --test end_to_end \
  gravity1 gravity_and_input \
  --no-fail-fast
```

Why this second check?

Because hard drop changes input-related gameplay state, so it is worth making
sure the earlier baseline timing path still behaves correctly too.

## When this PR is done

Stop this PR when:

- `Z` toggles the mode
- `GameState.manual_drop_gravity` changes between soft and hard values
- status text changes on screen
- hard-drop recorded tests are green

Do not start hold in the same PR.
