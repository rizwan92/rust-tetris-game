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

### Replace `HardDropPlugin::build`

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

## `src/board.rs`

### Update `LockdownTimer::start_or_advance`

Use this validated helper once `hard_drop` is active:

```rust
fn start_or_advance(&mut self, duration: Duration, time: &Time<Fixed>, tick_on_create: bool) {
    // Create the timer the first time we discover the piece is stuck.
    // We pass the duration in so normal lock and hard-drop lock can differ.
    if self.0.is_none() {
        // Start a one-shot timer for this lock window.
        self.0 = Some(Timer::new(duration, TimerMode::Once));
        if !tick_on_create {
            // In replay-driven tests, keep the original exact timing and wait until next frame.
            return;
        }
    }

    // Advance the timer once per fixed step while the piece is stuck.
    // In non-replay runs we also count the creation frame, which reduces timing flakiness.
    if let Some(timer) = &mut self.0 {
        timer.tick(time.delta());
    }
}
```

### Keep the down-arrow part of `handle_user_input` aligned with hard drop

Use this validated shape inside the `ArrowDown` branch:

```rust
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        // Track whether the player-managed downward move actually changed position.
        let mut moved = false;

        // Repeat downward steps based on the current manual drop gravity setting.
        for _ in 0..state.manual_drop_gravity {
            // Try one downward step at a time.
            let mut candidate = *tetromino;
            // Move the candidate one row lower.
            candidate.shift(0, -1);
            // Stop immediately when the next step would collide.
            if crate::there_is_collision(&candidate, obstacles.reborrow()) {
                break;
            }
            // Remember that at least one manual step succeeded.
            moved = true;
            // Commit the legal downward step.
            *tetromino = candidate;
        }

        // Check whether the piece is now resting after the manual drop finished.
        let landed = if moved {
            // Build one more downward candidate.
            let mut candidate = *tetromino;
            // Look one row below the final manual-drop position.
            candidate.shift(0, -1);
            // `true` means the piece is now sitting on the floor or obstacles.
            crate::there_is_collision(&candidate, obstacles.reborrow())
        } else {
            // No successful move means no landing caused by this key press.
            false
        };

        if moved {
            // Keep the manual-drop marker for this piece once the player pushed it down.
            // Example: if soft drop moved the piece several rows and gravity lands the final row,
            // we still want the shorter manual-drop lock timing later.
            commands.entity(entity).insert(ManualDropped);
            if landed && state.manual_drop_gravity > SOFT_DROP_GRAVITY {
                // Hard-drop is the narrower case: only mark it when the fast drop
                // actually reaches the resting position on this key press.
                commands.entity(entity).insert(HardDropped);
            }
        }
    }
```

### Do not clear the drop markers while the piece is still falling

Inside `deactivate_if_stuck`, keep the free-falling branch like this:

```rust
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        // Reset only the lockdown timer here.
        // Do not clear `ManualDropped`, because a piece that was soft-dropped earlier
        // still needs the shorter lock timing when it eventually settles.
        lockdown.reset();
        return;
    }
```

### Keep `deactivate_if_stuck` replay-safe and non-replay-stable

Use this shape for the lock-timer part:

```rust
pub fn deactivate_if_stuck(
    // Use commands to despawn the active piece and spawn obstacle blocks.
    mut commands: Commands,
    // Fixed-step time drives the lock timer.
    time: Res<Time<Fixed>>,
    // Read the time strategy so replay tests can keep their exact timing.
    time_strategy: Res<bevy::time::TimeUpdateStrategy>,
    // Store the current lock timer resource.
    mut lockdown: ResMut<LockdownTimer>,
    // Carry gravity when the piece was manually dropped.
    mut carry_gravity_timer: ResMut<CarryGravityTimer>,
    // Read the active piece and the two drop markers.
    active: Query<(Entity, &Tetromino, Has<HardDropped>, Has<ManualDropped>), With<Active>>,
    // Read obstacles for collision checks.
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // The rest of the function stays the same until duration selection.

    let duration = if manual_dropped {
        HARD_DROP_LOCKDOWN_DURATION
    } else {
        LOCKDOWN_DURATION
    };

    // Replay tests use `ManualDuration`, and they need exact recorded timing.
    // The ordinary timing path on macOS is more jittery, so we also count the
    // creation frame there to make the non-replay tests less flaky.
    let tick_on_create = !matches!(
        *time_strategy,
        bevy::time::TimeUpdateStrategy::ManualDuration(_)
    );

    // Start or advance the timer with the chosen duration and timing mode.
    lockdown.start_or_advance(duration, &time, tick_on_create);
    if !lockdown.just_finished() {
        return;
    }

    // The rest of the obstacle-spawn and cleanup code stays unchanged.
}
```

## Notes for later features

- This feature depends on the baseline version of `handle_user_input` already looping `state.manual_drop_gravity` times on down-arrow.
- In the validated gameplay path, `ManualDropped` should stay on the piece after any successful manual downward move.
- `HardDropped` is narrower: only set it when the fast manual drop actually lands on the floor or on obstacles.
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
