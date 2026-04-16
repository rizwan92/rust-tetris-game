# Hard Drop

## Goal

Finish the hard-drop feature in:

- `src/hard_drop.rs`

This guide assumes collision is already done.

## 1. Replace `toggle_hard_drop`

Paste this function:

```rust
/// Flip the hard-drop flag when the player presses Z.
fn toggle_hard_drop(keyboard: Res<ButtonInput<KeyCode>>, mut hard_drop: Single<&mut HardDrop>) {
    // The replay recordings show that hard drop flips on key press, not key
    // release.
    // Example:
    // pressing Z once changes `false` -> `true`.
    if keyboard.just_pressed(KeyCode::KeyZ) {
        hard_drop.0 = !hard_drop.0;
    }
}
```

## 2. Replace `update_drop_gravity`

Paste this function:

```rust
/// Update the manual drop amount whenever the hard-drop flag changes.
fn update_drop_gravity(
    hard_drop: Query<&HardDrop, Changed<HardDrop>>,
    mut state: ResMut<GameState>,
) {
    // Only do work when the flag actually changed.
    for hard_drop in &hard_drop {
        // Hard drop means the down input should behave like "drop one row"
        // many times in the same frame.
        // Example:
        // Off -> manual gravity 1
        // On  -> manual gravity 20
        state.manual_drop_gravity = if hard_drop.0 {
            HARD_DROP_GRAVITY
        } else {
            SOFT_DROP_GRAVITY
        };
    }
}
```

## 3. Replace `update_status_text`

Paste this function:

```rust
/// Rewrite the status text whenever the hard-drop flag changes.
fn update_status_text(mut text: Query<(&HardDrop, &mut Text), Changed<HardDrop>>) {
    // Again, `Changed<HardDrop>` keeps this system cheap.
    for (hard_drop, mut text) in &mut text {
        // Show a short human-readable status string in the UI.
        // Example:
        // if hard drop is enabled, the text becomes "Hard Drop: On".
        text.0 = if hard_drop.0 {
            "Hard Drop: On".to_string()
        } else {
            "Hard Drop: Off".to_string()
        };
    }
}
```

## 4. Replace the plugin

Make sure the plugin looks like this:

```rust
/// Plugin that adds hard-drop toggle behavior and the status text.
pub struct HardDropPlugin;

impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_text.in_set(Game))
            .add_systems(
                Update,
                (toggle_hard_drop, update_drop_gravity, update_status_text)
                    .chain()
                    .before(crate::board::handle_user_input)
                    .in_set(Game),
            );
    }
}
```

## 5. Local checks

Run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end hard_drop_67 hard_drop_727 hard_drop_deterministic hard_drop_deterministic2 hard_drop_hold_0 --no-fail-fast
```
