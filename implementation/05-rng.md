# RNG

## Goal

Implement the `rng` feature by filling only the starter skeleton in:

- `Cargo.toml`
- `src/bag.rs`

This feature adds the random 7-bag behavior used by the later replay and hard
drop tests.

## Step 1: Enable the feature in `Cargo.toml`

Find this line in [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml):

```toml
enabled_features = ["config", "collision", "score"]
```

Replace it with:

```toml
enabled_features = ["config", "collision", "score", "rng"]
```

Why:

- the random bag implementation is behind the `rng` feature flag
- this also enables the extra config variants `FixedSeed` and `RandomSeed`

## Step 2: Replace `RandomBag::from_seed`

Find this starter code in [src/bag.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/bag.rs):

```rust
pub fn from_seed(_seed: u64) -> Self {
    todo!("Create an empty bag, seed the RNG from the given value.")
}
```

Replace it with:

```rust
pub fn from_seed(seed: u64) -> Self {
    // Start with an empty bag and a deterministic RNG state.
    // Example:
    // seed 727 must always give the same sequence during tests.
    Self {
        remaining_pieces: vec![],
        rng: SmallRng::seed_from_u64(seed),
    }
}
```

Why:

- the bag should start empty
- the RNG must be deterministic for fixed-seed tests

## Step 3: Replace `refill`

Find this starter code in [src/bag.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/bag.rs):

```rust
fn refill(&mut self) {
    debug_assert!(self.remaining_pieces.is_empty());
    todo!()
}
```

Replace it with:

```rust
fn refill(&mut self) {
    debug_assert!(self.remaining_pieces.is_empty());

    // Build one of each canonical tetromino.
    // Example:
    // before shuffling, this is S, Z, L, J, T, O, I.
    self.remaining_pieces = ALL_TETROMINO_TYPES.map(get_tetromino).to_vec();

    // Shuffle the 7-piece bag in place with the bag's RNG state.
    self.remaining_pieces.shuffle(&mut self.rng);
}
```

Important note:

- use the library shuffle directly
- do not invent custom swap logic
- the tests expect the exact order produced by this style

## Step 4: Replace `next_tetromino`

Find:

```rust
fn next_tetromino(&mut self) -> Tetromino {
    todo!("Get the next tetromino from the bag.  Refill it if necessary")
}
```

Replace it with:

```rust
fn next_tetromino(&mut self) -> Tetromino {
    // Refill lazily if the bag is empty.
    if self.remaining_pieces.is_empty() {
        self.refill();
    }

    // Remove the next piece from the back of the vector.
    // This choice matters because `peek()` must use the same end.
    self.remaining_pieces
        .pop()
        .expect("bag should contain a tetromino after refill")
}
```

## Step 5: Replace `peek`

Find:

```rust
fn peek(&mut self) -> Tetromino {
    todo!()
}
```

Replace it with:

```rust
fn peek(&mut self) -> Tetromino {
    // Refill lazily if needed, just like `next_tetromino()`.
    if self.remaining_pieces.is_empty() {
        self.refill();
    }

    // Read from the same end that `next_tetromino()` removes from.
    // That is what makes the tests say "peek and next must agree."
    *self
        .remaining_pieces
        .last()
        .expect("bag should contain a tetromino after refill")
}
```

## Why the “same end of the vector” rule matters

Suppose the shuffled vector is:

```text
[S, Z, L, J, T, O, I]
```

If:

- `peek()` looks at the front
- but `next_tetromino()` removes from the back

then:

- `peek()` says the next piece is `S`
- `next_tetromino()` returns `I`

and the tests fail immediately.

So both functions must use the same end.

## Local checks

Run:

```bash
cargo fmt --all
```

Run:

```bash
cargo test --features test config::tests -- --nocapture
```

Run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end random_bag_impl1 random_bag_impl2 random_bag_impl3 --no-fail-fast
```

These RNG tests are deterministic, so they are good signal even before Linux CI.

## Summary

This feature should end with:

- deterministic fixed-seed random bags
- shuffled 7-piece refill behavior
- `peek()` and `next_tetromino()` agreeing with each other
