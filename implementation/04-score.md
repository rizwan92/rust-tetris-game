# Score

## Goal

Finish the score feature in:

- `src/score.rs`

This guide assumes collision is already done.

## 1. Add the event

Paste this near the top of `src/score.rs`:

```rust
/// An event denoting that some lines are cleared
#[derive(Event)]
pub struct LinesCleared(pub u32);
```

## 2. Replace `update_score`

Paste this function:

```rust
/// NEW IMPLEMENTATION: Update the game state when lines are cleared.
///
/// Simple example:
/// if the player clears 4 lines at level 2, the score gain is
/// `1200 * (2 + 1) = 3600`.
fn update_score(event: On<LinesCleared>, mut state: ResMut<GameState>) {
    let lines_cleared = event.0;
    assert!(lines_cleared <= 4);

    if lines_cleared == 0 {
        return;
    }

    let multiplier = match lines_cleared {
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => unreachable!("more than 4 lines cannot be cleared at once"),
    };

    state.score += multiplier * (state.level + 1);

    state.lines_cleared += lines_cleared;
    state.lines_cleared_since_last_level += lines_cleared;

    // NEW IMPLEMENTATION: allow more than one level-up if the saved line count
    // is already large enough.
    while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        state.lines_cleared_since_last_level -= (state.level + 1) * 10;
        state.level += 1;

        // NEW IMPLEMENTATION: keep the gravity progress when the level changes.
        //
        // Simple example:
        // if the old timer already waited 0.6 seconds, we should not throw that
        // away and restart from 0.0 seconds on the new level.
        let carried_elapsed = state.gravity_timer.elapsed();
        let new_duration = state.drop_interval();
        state.gravity_timer = Timer::new(new_duration, TimerMode::Repeating);
        if carried_elapsed >= new_duration {
            state.gravity_timer.almost_finish();
        } else {
            state.gravity_timer.set_elapsed(carried_elapsed);
        }
    }
}
```

## 3. Replace the plugin

Make sure the plugin looks like this:

```rust
/// NEW IMPLEMENTATION: Plugin that adds score tracking, leveling, and score
/// text updates.
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_score_text.in_set(Game))
            .add_systems(Update, update_score_text.in_set(Game))
            .add_observer(update_score);
    }
}
```

## 4. Local checks

Run:

```bash
cargo test --features test score::tests -- --nocapture
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end score_deterministic score_deterministic2 score_deterministic3 score_deterministic4 level_up --no-fail-fast
```
