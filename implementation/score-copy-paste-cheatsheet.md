# Score Copy-Paste Cheatsheet

Use this file after collision is complete.

## Commenting Rule For This File

- this file explains both the math and the timer updates in simple words
- comments on parameters tell you what comes into the function
- comments inside the loop explain why level-up logic is written as a `while`

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
- keeps the comments very literal so you can study the math line by line

## `src/score.rs`

### Replace `update_score`

File:
- [score.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/score.rs)

```rust
fn update_score(
    // Wake up when a line-clear event is triggered.
    event: On<LinesCleared>,
    // Mutably access the global game state so score and level can change.
    mut state: ResMut<GameState>,
) {
    // `event` tells us how many lines were cleared in one action.
    // `state` is the global game state that stores score, level, and timers.
    let lines_cleared = event.0;

    // Tetris can clear at most four lines at once.
    assert!(lines_cleared <= 4);

    // Convert line-count into the assignment's score multiplier table.
    let multiplier = match lines_cleared {
        0 => 0,
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => unreachable!("line clear count must be between 0 and 4"),
    };

    // Score formula from the assignment: multiplier * (level + 1).
    state.score += multiplier * (state.level + 1);
    // Track the total number of cleared lines across the full game.
    state.lines_cleared += lines_cleared;
    // Track the lines contributing toward the next level-up threshold.
    state.lines_cleared_since_last_level += lines_cleared;

    // Use a loop because one clear could, in theory, cross multiple thresholds.
    while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        // Remove the threshold that was just satisfied.
        state.lines_cleared_since_last_level -= (state.level + 1) * 10;
        // Increase the level by one.
        state.level += 1;
        // Recompute gravity for the new level.
        let interval = state.drop_interval();
        state.gravity_timer.set_duration(interval);
        // Do not reset the timer here.
        // Keeping the current progress matched the validated replay behavior better.
    }
}
```

### Replace `update_score_text`

```rust
fn update_score_text(
    // Read the latest score-related values.
    state: Res<GameState>,
    // Mutably access the HUD text that shows score information.
    mut score_text: Single<&mut Text, With<ScoreMarker>>,
) {
    // Skip extra UI work when the score state did not change.
    if !state.is_changed() {
        return;
    }

    // Rebuild the HUD text from the latest score, level, and line totals.
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
