# Hard Drop Copy-Paste Cheatsheet

Use this file after collision is complete and your baseline input handling already respects `manual_drop_gravity`.

## Commenting Rule For This File

- comments explain both the toggle logic and the UI update logic
- changed parameters are commented in the signature
- short examples are included where the meaning could be confusing

## Enable the feature

Set [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml) to:

```toml
enabled_features = ["config", "collision", "score", "rng", "hard_drop"]
```

## What this feature changes

- adds a `Z` key toggle for hard drop
- switches `GameState.manual_drop_gravity` between soft and hard values
- updates the on-screen hard-drop status text
- registers the new systems inside `HardDropPlugin`
- explains the changed parameters and body lines in plain English

## `src/hard_drop.rs`

### Replace the imports at the top

Replace:

```rust
use bevy::prelude::*;

use crate::ui::*;
```

with:

```rust
use bevy::prelude::*;

use crate::{
    Game,
    data::{GameState, HARD_DROP_GRAVITY, SOFT_DROP_GRAVITY},
    ui::*,
};
```

### Add these three systems below `setup_status_text`

```rust
fn toggle_hard_drop(
    // Read keyboard transitions for this frame.
    // We only want the exact press moment, not continuous holding.
    keyboard: Res<ButtonInput<KeyCode>>,
    // Access the single hard-drop toggle component mutably.
    // This component stores one boolean like `true` or `false`.
    mut hard_drop: Single<&mut HardDrop>,
) {
    // Flip the hard-drop state when Z is pressed.
    if keyboard.just_pressed(KeyCode::KeyZ) {
        // Invert the boolean toggle in place.
        hard_drop.0 = !hard_drop.0;
    }
}

fn update_manual_drop_gravity(
    // Read the hard-drop toggle only when it has changed.
    // `Changed<HardDrop>` avoids extra work every frame.
    hard_drop: Single<&HardDrop, Changed<HardDrop>>,
    // Update the game-state gravity used by manual down presses.
    // This is the value that `ArrowDown` uses, not the automatic gravity timer.
    mut state: ResMut<GameState>,
) {
    // Use hard-drop gravity when the toggle is on, otherwise use soft-drop gravity.
    state.manual_drop_gravity = if hard_drop.0 {
        HARD_DROP_GRAVITY
    } else {
        SOFT_DROP_GRAVITY
    };
}

fn update_status_text(
    // Read the hard-drop component and the text together only when the toggle changed.
    // The first item is the boolean state, and the second item is the visible text.
    mut status: Single<(&HardDrop, &mut Text), Changed<HardDrop>>,
) {
    // Choose the user-visible label from the boolean toggle.
    let label = if status.0.0 { "On" } else { "Off" };
    // Replace the displayed status text.
    status.1 .0 = format!("Hard Drop: {label}");
}
```

### Replace `HardDropPlugin::build`

```rust
impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        // Spawn the status text as part of the game startup sequence.
        app.add_systems(Startup, setup_status_text.in_set(Game))
            // Run the hard-drop systems during Update inside the Game system set.
            // The validated version keeps the systems chained so the order is predictable.
            .add_systems(
                Update,
                (
                    toggle_hard_drop,
                    update_manual_drop_gravity,
                    update_status_text,
                )
                    // First flip the toggle, then update gravity, then refresh the text.
                    // This prevents the text and the stored gravity from getting out of sync.
                    .chain()
                    .in_set(Game),
            );
    }
}
```

## Notes for later features

- This feature depends on the baseline version of `handle_user_input` already looping `state.manual_drop_gravity` times on down-arrow.
- `hold` does not change the hard-drop toggle itself, so these systems can stay isolated in `hard_drop.rs`.

## Test commands

Start with the smallest recorded tests:

```bash
cargo test --features test --test end_to_end hard_drop_deterministic -- --test-threads=1
cargo test --features test --test end_to_end hard_drop_deterministic2 -- --test-threads=1
```

Then run the whole hard-drop file:

```bash
cargo test --features test --test end_to_end hard_drop_ -- --test-threads=1
```

Then run the cumulative regression sweep:

```bash
cargo test --features test --test end_to_end -- --test-threads=1
```

## Acceptance checkpoint

Do not move to `hold` until:

- the deterministic hard-drop recordings pass
- the full hard-drop test file passes
- the cumulative suite still passes with `enabled_features = ["config", "collision", "score", "rng", "hard_drop"]`
