# Hard Drop

## Goal

Implement the `hard_drop` feature by filling only the starter skeleton in:

- `Cargo.toml`
- `src/hard_drop.rs`

This feature does not rewrite the board logic.

Instead, it adds a small toggle that changes how the existing down-arrow logic
behaves:

- hard drop off: manual gravity is `1`
- hard drop on: manual gravity is `20`

Because baseline already made down-arrow use `manual_drop_gravity`, this feature
stays nicely modular.

## Step 1: Enable the feature in `Cargo.toml`

Find this line in [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml):

```toml
enabled_features = ["config", "collision", "score", "rng"]
```

Replace it with:

```toml
enabled_features = ["config", "collision", "score", "rng", "hard_drop"]
```

Why:

- the plugin in `src/hard_drop.rs` only exists when the `hard_drop` feature is
  enabled

## Step 2: Keep the status text setup, but add a doc comment

The starter `setup_status_text` code is already basically correct.

You only need to keep it and let the later systems update the text value.

## Step 3: Add `toggle_hard_drop`

In [src/hard_drop.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hard_drop.rs), replace the big TODO comment block by adding this system:

```rust
fn toggle_hard_drop(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hard_drop: Single<&mut HardDrop>,
) {
    // The replay recordings show that hard drop flips on key press.
    // Example:
    // pressing Z once changes false -> true.
    if keyboard.just_pressed(KeyCode::KeyZ) {
        hard_drop.0 = !hard_drop.0;
    }
}
```

Why this is correct:

- the recorded tests use `KeyZ`
- the state changes on press, not on release
- there is only one hard-drop status entity, so `Single` is the right choice

## Step 4: Add `update_drop_gravity`

Add this system below it:

```rust
fn update_drop_gravity(
    hard_drop: Query<&HardDrop, Changed<HardDrop>>,
    mut state: ResMut<GameState>,
) {
    for hard_drop in &hard_drop {
        state.manual_drop_gravity = if hard_drop.0 {
            HARD_DROP_GRAVITY
        } else {
            SOFT_DROP_GRAVITY
        };
    }
}
```

Why this works:

- when the flag becomes `true`, down-arrow now behaves like 20 repeated drops
- when the flag becomes `false`, it goes back to the normal one-row behavior
- `Changed<HardDrop>` means we only update the state when the toggle actually
  changes

## Step 5: Add `update_status_text`

Add this system:

```rust
fn update_status_text(mut text: Query<(&HardDrop, &mut Text), Changed<HardDrop>>) {
    for (hard_drop, mut text) in &mut text {
        text.0 = if hard_drop.0 {
            "Hard Drop: On".to_string()
        } else {
            "Hard Drop: Off".to_string()
        };
    }
}
```

Why:

- the `HardDrop` component and the `Text` live on the same UI entity
- the text should update only when the toggle changes

## Step 6: Finish the plugin

Find this starter code in [src/hard_drop.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hard_drop.rs):

```rust
impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_text);
        todo!("add your systems here.  They should go in Update, and in the Game system set.")
    }
}
```

Replace it with:

```rust
impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_text.in_set(Game))
            .add_systems(
                Update,
                (toggle_hard_drop, update_drop_gravity, update_status_text).in_set(Game),
            );
    }
}
```

Why:

- the starter spec says these systems belong in `Update`
- they should also be in the shared `Game` set
- this keeps test injection ordering consistent with the rest of the app

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
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end hard_drop_deterministic hard_drop_deterministic2 hard_drop_67 hard_drop_727 --no-fail-fast
```

Because the hard-drop tests are replay-based, they are better local signal than
the sleep-based realtime tests.

## Summary

This feature should end with:

- Z toggling hard drop on and off
- the UI showing `Hard Drop: On` or `Hard Drop: Off`
- `manual_drop_gravity` switching between `1` and `20`
