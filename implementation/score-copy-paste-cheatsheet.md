# Score Copy-Paste Cheatsheet

Use this file after collision is complete.

## Enable the feature

Set [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml) to:

```toml
enabled_features = ["config", "collision", "score"]
```

## What this feature changes

- updates `GameState` when lines are cleared
- computes score based on the assignment multiplier table
- levels up according to the assignment threshold
- updates the gravity timer when level changes
- shows score, level, and total lines in the UI

## `src/score.rs`

### Replace `update_score`

File:
- [score.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/score.rs)

```rust
fn update_score(event: On<LinesCleared>, mut state: ResMut<GameState>) {
    // Read how many lines were cleared by this event.
    let lines_cleared = event.0;
    // The assignment guarantees at most four cleared lines at once.
    assert!(lines_cleared <= 4);

    // Convert the cleared-line count into the scoring multiplier from the spec.
    let multiplier = match lines_cleared {
        // No lines means no score change.
        0 => 0,
        // Single line clear score.
        1 => 40,
        // Double line clear score.
        2 => 100,
        // Triple line clear score.
        3 => 300,
        // Tetris score.
        4 => 1200,
        // The assert above should make every other case impossible.
        _ => unreachable!("line clear count must be between 0 and 4"),
    };

    // Apply the score formula: multiplier * (current level + 1).
    state.score += multiplier * (state.level + 1);
    // Track total cleared lines across the whole game.
    state.lines_cleared += lines_cleared;
    // Track cleared lines toward the next level threshold.
    state.lines_cleared_since_last_level += lines_cleared;

    // Keep leveling up while the accumulated lines still satisfy the next threshold.
    while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        // Remove the threshold that was just satisfied.
        state.lines_cleared_since_last_level -= (state.level + 1) * 10;
        // Advance the level by one.
        state.level += 1;
        // Update gravity to match the new level.
        state.gravity_timer.set_duration(state.drop_interval());
        // Reset the timer so the new speed starts cleanly.
        state.gravity_timer.reset();
    }
}
```

### Replace `update_score_text`

```rust
fn update_score_text(
    // Read the score-related game state.
    state: Res<GameState>,
    // Access the score text UI node mutably.
    mut score_text: Single<&mut Text, With<ScoreMarker>>,
) {
    // Skip work when score-related state has not changed.
    if !state.is_changed() {
        // Leave the existing text alone.
        return;
    }

    // Render score, level, and total cleared lines as a compact multi-line HUD.
    score_text.0 = format!(
        "Score: {}\nLevel: {}\nLines: {}",
        state.score(),
        state.level(),
        state.lines_cleared
    );
}
```

## Notes for later features

- `collision` already emits `LinesCleared`, so no extra event wiring is needed if you used the future-proof collision snippet.
- `hard_drop` and `hold` recorded tests can also check score-related state later, so keep this behavior stable.

## Test commands

Start with the smallest recorded checks:

```bash
cargo test --features test --test end_to_end score_deterministic -- --test-threads=1
cargo test --features test --test end_to_end score_deterministic2 -- --test-threads=1
```

Then run the whole score test file:

```bash
cargo test --features test --test end_to_end score_ -- --test-threads=1
```

`level_up` becomes relevant later once both `hard_drop` and `hold` are also enabled.

Then run the cumulative regression sweep for baseline + config + collision + score:

```bash
cargo test --features test --test end_to_end -- --test-threads=1
```

## Acceptance checkpoint

Do not move to `rng` until:

- the two smallest score recordings pass
- the score test file passes
- the cumulative suite still passes with `enabled_features = ["config", "collision", "score"]`
