# How Tests Work

This guide explains how the test suite runs the game, which files are involved,
and how the tests decide whether your code is correct.

This file is especially useful if you are asking questions like:

- "Which file starts the tests?"
- "How does the test press keys?"
- "How does the test know which game code is running?"
- "How does replay validation work?"
- "Why do Linux and macOS feel different?"

## Start Here

If you want to understand the test flow, read the files in this order:

1. `tests/end_to_end/main.rs`
2. `tests/end_to_end/common.rs`
3. `src/lib.rs`
4. the feature test file you care about, such as:
   - `tests/end_to_end/baseline.rs`
   - `tests/end_to_end/collision.rs`
   - `tests/end_to_end/score.rs`
   - `tests/end_to_end/hard_drop.rs`
   - `tests/end_to_end/hold.rs`
5. `src/rr/test_replay.rs` for replay-based tests

## Big Picture

The end-to-end tests do not just call one small function and compare one return
value.

Instead, the tests:

1. create a Bevy app
2. build the game inside that app
3. inject fake keyboard input
4. run Bevy update loops
5. read the ECS world
6. compare the real game state with the expected game state

So the tests are closer to "simulate a real game" than "call a helper
function."

## File 1: `tests/end_to_end/main.rs`

This is the custom entry point for the end-to-end suite.

Important idea:

- this test target uses a custom harness
- it does **not** use the default Rust test harness for this suite

Why:

- Bevy apps and real-time systems do not work well with the normal
  "many tests in one process" style
- the project uses `libtest_mimic_collect` instead

Simple meaning:

- this file is the "front door" for end-to-end tests
- it loads all the feature test modules

It also prints a warning on non-Linux systems.

That warning matters because:

- Linux is the trusted grading environment
- macOS can show extra timing noise

## File 2: `tests/end_to_end/common.rs`

This is the shared test helper file.

This file is one of the most important files for understanding the test suite.

It gives the tests:

- a headless Bevy app
- keyboard simulation
- helper methods for reading pieces and obstacles
- replay-test helpers

### The `Headless` app

The `Headless` type creates a Bevy app without a normal game window.

Simple example:

- in the real game, the player sees a window
- in the tests, we usually do not need a visible window
- we only need the ECS world and the game systems

So the tests create a headless app and still run the same game logic.

### `blox::build_app(...)`

Inside `Headless::new(...)`, the helper calls:

```rust
blox::build_app(&mut app, cfg);
```

This is the moment where the real game code gets attached to the Bevy app.

That means the tests are not running a fake copy of the game.

They are running your real systems from `src/lib.rs` and the feature files.

### Keyboard simulation

The helper stores key state in `KeySequence`.

Then this system runs:

```rust
simulate_key_presses
```

What it does:

1. release keys that were down last frame but not this frame
2. press keys that are down this frame

Simple example:

- previous frame: `Left`
- current frame: `[]`
- result: the helper releases `Left`

Another example:

- previous frame: `[]`
- current frame: `[ArrowUp]`
- result: the helper presses `ArrowUp`

So this is how tests pretend that a player pressed a key.

### `release_then_press(...)`

This helper is used in many direct gameplay tests.

What it does:

1. release all keys
2. run one update
3. press the requested key
4. run one more update

Why:

- it gives a clean input edge
- it avoids reusing an old pressed key accidentally

## File 3: `src/lib.rs`

This file is the main game entry point.

It tells Bevy which systems run and in which schedule.

This file is important because it answers:

- which code runs in `Startup`
- which code runs in `Update`
- which code runs in `FixedUpdate`

### The system sets

You will see three important sets:

- `PreGame`
- `Game`
- `PostGame`

Meaning:

- `PreGame`: test systems run before the real game systems
- `Game`: your actual game systems run here
- `PostGame`: test systems can observe results after the game systems

This is how the harness avoids races.

Simple example:

1. test injects keyboard input in `PreGame`
2. your game consumes that input in `Game`
3. test checks the result in `PostGame`

That ordering is one of the reasons the suite stays deterministic.

### Where the real gameplay systems are registered

`build_app(...)` adds systems like:

- `setup_board`
- `spawn_next_tetromino`
- `gravity`
- `deactivate_if_stuck`
- `delete_full_lines`
- `handle_user_input`
- `redraw_board`
- `redraw_side_board::<Next>`

So if a test fails, these are the real systems that are being exercised.

## Which Test File Checks Which Feature?

Here is the rough feature map.

### `tests/end_to_end/baseline.rs`

Checks the baseline game behavior, such as:

- piece spawn
- shift left/right
- rotate
- gravity basics

Examples:

- `shift1`
- `i_rotate`
- `gravity1`

### `tests/end_to_end/collision.rs`

Checks:

- stacking
- obstacle collision
- game over
- full-line deletion behavior

Examples:

- `basic_stacking`
- `basic_game_over`

### `tests/end_to_end/score.rs`

Checks:

- scoring
- level progression
- replay tests that also verify score and line count

Examples:

- `score_deterministic`
- `level_up`

### `tests/end_to_end/hard_drop.rs`

Checks:

- hard drop toggle
- hard drop replay behavior

Examples:

- `hard_drop_67`
- `hard_drop_deterministic`

### `tests/end_to_end/hold.rs`

Checks:

- hold behavior
- hold + replay behavior
- hold + hard drop interactions

Examples:

- `first_hold`
- `next_hold`
- `hard_drop_hold_0`

### `tests/end_to_end/rng.rs` or bag-related tests

Checks:

- bag behavior
- deterministic randomness behavior

Examples:

- `random_bag_impl1`
- `random_bag_impl2`
- `random_bag_impl3`

## Replay-Based Tests

Replay-based tests are the most confusing at first.

The key file is:

- `src/rr/test_replay.rs`

These tests do not just press a key and look once.

Instead, they:

1. load a recorded gameplay file
2. feed recorded key events into the game
3. run the game frame by frame
4. compare the real state against recorded expected states

### What gets compared

The replay system records and compares things like:

- active tetromino cells
- active tetromino center
- next tetromino
- hold tetromino
- obstacle blocks
- score
- level
- cleared lines

So if one timer is even a little off, replay tests can fail.

### Why replay tests feel strict

Example:

- the expected state says the piece should still be at row `6`
- your game already moved it to row `5`

Visually, that may not look like a big error.

But replay validation treats that as a mismatch immediately.

That is why replay tests often reveal:

- timer bugs
- gravity/lockdown/spawn order bugs
- replay input boundary bugs

## How Validation Works

There are two common validation styles in this repo.

### Style 1: Direct state checks

A normal end-to-end test might:

1. create a `Headless` app
2. press a key
3. run one or more updates
4. read tetrominoes or obstacles from the ECS world
5. compare them with the expected result

Simple example:

- press `ArrowLeft`
- run update
- check that the active piece moved left by one cell

### Style 2: Replay validation

A replay test might:

1. load a recording from `test_data`
2. attach `TestReplayPlugin`
3. run enough update ticks to finish the recording
4. fail if too many mismatches happen

This is more strict because it checks the state over time, not just once.

## Why Linux Is the Trusted Environment

The test harness itself warns that non-Linux systems are not officially
supported for the end-to-end suite.

Why this matters:

- some tests are timing-sensitive
- Linux is the grading environment
- macOS can show extra timing noise

So if:

- Linux is green
- macOS still looks noisy

then Linux is the stronger signal for whether the logic is correct.

## The Most Important Mental Model

If you want one simple way to think about the suite, use this:

1. the test builds a Bevy app
2. your real game systems are inserted into that app
3. the test injects input before the game systems
4. the game systems run
5. the test reads ECS state after the systems run
6. the test compares that state to the expected result

That is the full idea.

## Commands You Should Know

### Local student check

```bash
cargo nextest run --features test --retries 3
```

### Linux/CI-style check

```bash
cargo nextest run --no-default-features --features ci --verbose --no-fail-fast --retries 2
```

### Why these are different

- `test` is for local student work
- `ci` is for the Linux grading-style environment

## Final Advice For Reading Failures

When a test fails, use this order:

1. find the test name
2. open the file where that test is written
3. check whether it is a direct state test or a replay test
4. open `tests/end_to_end/common.rs`
5. open `src/lib.rs`
6. then open the gameplay file for that feature

Good examples:

- baseline failure:
  - read `tests/end_to_end/baseline.rs`
  - then `src/board.rs`
- hard drop replay failure:
  - read `tests/end_to_end/hard_drop.rs`
  - then `src/hard_drop.rs`
  - then `src/rr/test_replay.rs`
- hold failure:
  - read `tests/end_to_end/hold.rs`
  - then `src/hold.rs`

That reading order usually makes the problem much easier to understand.
