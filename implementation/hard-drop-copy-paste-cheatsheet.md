# Hard Drop Copy-Paste Cheatsheet

Use this file after collision is complete and your baseline input handling already respects `manual_drop_gravity`.

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
    keyboard: Res<ButtonInput<KeyCode>>,
    // Access the single hard-drop toggle component mutably.
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
    hard_drop: Single<&HardDrop, Changed<HardDrop>>,
    // Update the game-state gravity used by manual down presses.
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
            .add_systems(
                Update,
                (
                    toggle_hard_drop,
                    update_manual_drop_gravity,
                    update_status_text,
                )
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
