# PR 4 Guide: Score

This PR makes line clears matter to the player.

Before this PR:

- rows may disappear
- but score may stay unchanged
- level may stay unchanged
- HUD text may stay stale

After this PR:

- line clears increase score
- enough cleared lines increase level
- the score text updates on screen

So this PR is really about turning "board events" into "game progress".

## What this PR is trying to achieve

At the end of this PR, one line-clear event should affect 3 things:

1. score
2. level
3. score HUD text

That is the simplest way to think about this feature.

## Starter files to compare

- `original-repo/src/score.rs`
- `original-repo/docs/score.md`

## Files you will change

- `src/score.rs`
- `src/collision.rs` should already be triggering `LinesCleared(...)` if you followed PR 3

## Feature flag state

Use:

```toml
enabled_features = ["config", "collision", "score"]
```

## Mental model before touching code

This feature works like a chain:

1. collision system clears rows
2. collision system triggers `LinesCleared(n)`
3. score system receives that event
4. score system updates:
   - `state.score`
   - `state.lines_cleared`
   - `state.lines_cleared_since_last_level`
   - `state.level`
5. HUD text reads the latest `GameState` and redraws the text

So this PR is mostly about reacting to one event in the right way.

## Important dependency from PR 3

This feature only works if `src/collision.rs` already contains:

```rust
#[cfg(feature = "score")]
commands.trigger(LinesCleared(full_rows.len() as u32));
```

Why?

Because that line is what tells the score system:

"some lines were just cleared"

If that line is missing, the score observer will never run.

## Step 1: understand the score rules before writing code

The assignment uses this multiplier table:

| Lines cleared | Multiplier |
|---|---:|
| 1 | 40 |
| 2 | 100 |
| 3 | 300 |
| 4 | 1200 |

And the actual score formula is:

```text
score += multiplier * (current level + 1)
```

### Example

If the current level is `0` and the player clears 2 lines:

- multiplier = `100`
- level factor = `0 + 1 = 1`
- score increase = `100 * 1 = 100`

If the current level is `2` and the player clears 4 lines:

- multiplier = `1200`
- level factor = `2 + 1 = 3`
- score increase = `1200 * 3 = 3600`

That is the math we are implementing.

## Step 2: understand the level-up rule before writing code

The rule is:

- to reach level 1, clear 10 lines
- to reach level 2, clear 20 more lines
- to reach level 3, clear 30 more lines
- and so on

So the threshold is not always 10.
It grows with the level.

That is why the code uses:

```rust
(state.level + 1) * 10
```

### Example

If you are at:

- level 0

then next threshold is:

- `(0 + 1) * 10 = 10`

If you are at:

- level 1

then next threshold is:

- `(1 + 1) * 10 = 20`

That is why we keep both:

- total cleared lines
- lines cleared since last level-up

## Step 3: replace `update_score`

### Why this function matters

This is the core logic of the whole score feature.

It is the place where one `LinesCleared(...)` event becomes:

- score increase
- line counters update
- possible level-up
- gravity interval refresh

### What to replace

Open `src/score.rs`.

Find the starter function:

```rust
fn update_score(event: On<LinesCleared>, mut _state: ResMut<GameState>) {
    let lines_cleared = event.0;
    assert!(lines_cleared <= 4);

    todo!()
}
```

Replace it with:

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

## Step 4: understand `update_score` in 4 simple parts

### Part 1: read the event

This line:

```rust
let lines_cleared = event.0;
```

extracts how many lines were just cleared in one action.

Example:

- single clear -> `1`
- double clear -> `2`
- tetris clear -> `4`

### Part 2: compute the multiplier

This match block:

```rust
let multiplier = match lines_cleared {
    0 => 0,
    1 => 40,
    2 => 100,
    3 => 300,
    4 => 1200,
    _ => unreachable!("line clear count must be between 0 and 4"),
};
```

turns "number of lines" into "base score value".

### Part 3: update score and counters

These lines:

```rust
state.score += multiplier * (state.level + 1);
state.lines_cleared += lines_cleared;
state.lines_cleared_since_last_level += lines_cleared;
```

update:

- the visible score
- the lifetime total lines
- the progress toward the next level

### Part 4: handle level-up

This loop:

```rust
while state.lines_cleared_since_last_level >= (state.level + 1) * 10
```

checks whether the player crossed a level threshold.

If yes:

- consume that threshold
- increase level
- refresh the gravity interval

## Step 5: replace `update_score_text`

### Why this function matters

Even if the score logic is correct internally, the player still needs to see it.

So this function is the display layer:

- read latest state
- convert it into text
- update the HUD

### What to replace

Find the starter function:

```rust
fn update_score_text() {
    todo!()
}
```

Replace it with:

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

## Step 6: understand why `is_changed()` is useful

This line:

```rust
if !state.is_changed() {
    return;
}
```

means:

"if the score-related state did not change, do not waste time rebuilding the text"

That is a small efficiency improvement, and it also makes the function easier to
reason about:

- state changed -> redraw text
- state did not change -> leave text alone

## Step 7: understand why the gravity timer duration changes here

This line:

```rust
state.gravity_timer.set_duration(interval);
```

matters because higher levels should make pieces fall faster.

So when the level increases, the score system also refreshes the gravity speed.

### Why we do not reset the timer

The code intentionally does this:

```rust
state.gravity_timer.set_duration(interval);
```

and not a full reset.

Why?

Because keeping the current timer progress matched the validated replay behavior
better in this project.

So this is not random.
It is part of the tested final behavior.

## Common beginner confusion here

### "Why is scoring in `score.rs` and not in `collision.rs`?"

Because collision should only detect:

- rows were cleared

Score should react to that result.

That separation is cleaner:

- collision detects
- score reacts

### "Why use an observer/event instead of calling the score function directly?"

Because the event system cleanly connects the two systems without hard-coding
the score logic into collision logic.

### "Why is there a `0 => 0` case if 0 cleared lines should not happen?"

It makes the match total and harmless.

Even if a zero-line event is unusual, mapping it to zero score is safe.

## Tests for this PR

### Score-focused replay tests

Run:

```bash
cargo nextest run --features test,config,collision,score --test end_to_end \
  score_deterministic score_deterministic2 score_deterministic3 score_deterministic4 level_up \
  --no-fail-fast
```

This checks:

- single clear scoring
- multi-line clear scoring
- deterministic replay correctness
- level-up behavior

### Wider regression check

Then run:

```bash
cargo nextest run --features test,config,collision,score --test end_to_end --no-fail-fast
```

This makes sure adding score logic did not disturb the earlier gameplay flow.

## When this PR is done

Stop this PR when:

- the correct score multiplier is used for 1/2/3/4 line clears
- level-up happens at the right thresholds
- gravity interval updates after level-up
- HUD text reflects score, level, and line totals
- replay-based score tests are green

Do not start RNG in the same PR.
