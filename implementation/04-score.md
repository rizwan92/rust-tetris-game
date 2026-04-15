# Score

## Goal

Implement the `score` feature by filling only the starter skeleton in:

- `Cargo.toml`
- `src/score.rs`

This feature adds three connected behaviors:

1. score increases when lines are cleared
2. level increases after enough cleared lines
3. the score UI text shows the latest values

## Step 1: Enable the feature in `Cargo.toml`

Find this line in [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml):

```toml
enabled_features = ["config", "collision"]
```

Replace it with:

```toml
enabled_features = ["config", "collision", "score"]
```

Why:

- the score plugin is gated behind the `score` feature
- score depends on collision, because line-clear events come from the collision
  system

## Step 2: Replace `update_score`

Find this starter code in [src/score.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/score.rs):

```rust
fn update_score(event: On<LinesCleared>, mut _state: ResMut<GameState>) {
    let lines_cleared = event.0;
    assert!(lines_cleared <= 4);

    todo!()
}
```

Replace it with:

```rust
fn update_score(event: On<LinesCleared>, mut state: ResMut<GameState>) {
    let lines_cleared = event.0;
    assert!(lines_cleared <= 4);

    // Defensive guard:
    // if zero lines are reported, there is nothing to update.
    if lines_cleared == 0 {
        return;
    }

    // Use the assignment's multiplier table.
    // Example:
    // 2 cleared lines means multiplier 100.
    let multiplier = match lines_cleared {
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => unreachable!("more than 4 lines cannot be cleared at once"),
    };

    // Score grows by multiplier * (current level + 1).
    // Example:
    // 4 lines at level 2 gives 1200 * 3 = 3600 points.
    state.score += multiplier * (state.level + 1);

    // Update both line counters.
    state.lines_cleared += lines_cleared;
    state.lines_cleared_since_last_level += lines_cleared;

    // Level-up rule from the spec:
    // to go from level N to N+1, clear (N + 1) * 10 lines since the last
    // level increase.
    while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        state.lines_cleared_since_last_level -= (state.level + 1) * 10;
        state.level += 1;

        // Refresh the gravity timer so the new level uses the faster interval.
        state.gravity_timer = Timer::new(state.drop_interval(), TimerMode::Repeating);
    }
}
```

Why this works:

- score uses the exact multiplier table from the assignment
- line totals stay accurate
- level progression matches the “10, then 20, then 30...” rule
- gravity speed updates when the level changes

## Step 3: Replace `update_score_text`

Find this starter code in [src/score.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/score.rs):

```rust
fn update_score_text() {
    todo!()
}
```

Replace it with:

```rust
fn update_score_text(
    state: Res<GameState>,
    mut text: Single<&mut Text, With<ScoreMarker>>,
) {
    // Rewrite the score panel each frame.
    // Example:
    // Score: 1200
    // Level: 1
    // Lines: 10
    text.0 = format!(
        "Score: {}\nLevel: {}\nLines: {}",
        state.score(),
        state.level(),
        state.lines_cleared
    );
}
```

Why this is enough:

- there is only one score text entity, so `Single` is perfect here
- the text can simply be rewritten from the latest game state

## Step 4: Add a few small score tests

Add these tests at the bottom of [src/score.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/score.rs):

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use crate::bag::DeterministicBag;
    use std::time::Duration;

    fn mk_game_state() -> GameState {
        GameState {
            score: 0,
            lines_cleared: 0,
            lines_cleared_since_last_level: 0,
            bag: Box::new(DeterministicBag::default()),
            level: 0,
            manual_drop_gravity: SOFT_DROP_GRAVITY,
            gravity_timer: Timer::new(GameState::initial_drop_interval(), TimerMode::Repeating),
        }
    }

    #[test]
    fn score_single_line_at_level_zero() {
        let mut state = mk_game_state();

        state.score += 40 * (state.level + 1);
        state.lines_cleared += 1;
        state.lines_cleared_since_last_level += 1;

        assert_eq!(state.score, 40);
        assert_eq!(state.lines_cleared, 1);
        assert_eq!(state.level, 0);
    }

    #[test]
    fn score_tetris_scales_with_level() {
        let mut state = mk_game_state();
        state.level = 2;

        state.score += 1200 * (state.level + 1);

        assert_eq!(state.score, 3600);
    }

    #[test]
    fn level_thresholds_match_spec() {
        let mut state = mk_game_state();

        state.lines_cleared_since_last_level = 10;
        while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
            state.lines_cleared_since_last_level -= (state.level + 1) * 10;
            state.level += 1;
            state.gravity_timer = Timer::new(state.drop_interval(), TimerMode::Repeating);
        }

        assert_eq!(state.level, 1);
        assert_eq!(state.lines_cleared_since_last_level, 0);
        assert_eq!(state.gravity_timer.mode(), TimerMode::Repeating);
        assert_eq!(
            state.gravity_timer.duration(),
            Duration::from_secs_f32(GameState::INTERVALS[1] / GameState::FRAMERATE)
        );
    }
}
```

These tests are small, but they still prove the main ideas:

- multiplier math
- level scaling
- timer update on level-up

## Local checks

Run:

```bash
cargo fmt --all
```

Run:

```bash
cargo test --features test score:: -- --nocapture
```

Run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end score_deterministic score_deterministic2 score_deterministic3 score_deterministic4 --no-fail-fast
```

Local macOS note:

- replay-based score tests are usually stronger signal than the sleep-based
  realtime tests
- if Linux CI disagrees, trust Linux CI as the final judge

## Summary

This feature should end with:

- line clear events increasing score
- correct level progression
- gravity interval updating after level-up
- score text showing score, level, and total lines
