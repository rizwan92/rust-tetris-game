# PR 5 Guide: RNG

This PR makes the bag random, but still predictable when tests need it.

That is the key idea:

- real randomness for normal play
- deterministic randomness for fixed-seed tests

So this PR is really about controlled randomness.

## What this PR is trying to achieve

At the end of this PR, the bag should do 4 things correctly:

1. start empty
2. refill itself only when empty
3. refill with exactly one copy of each tetromino
4. shuffle in a deterministic way when a fixed seed is used

It also needs one important behavior contract:

- `peek()` must show the same piece that `next_tetromino()` would remove next

That one detail is very important for preview logic and tests.

## Starter files to compare

- `original-repo/src/bag.rs`
- `original-repo/docs/rng.md`

## File you will change

- `src/bag.rs`

## Feature flag state

Use:

```toml
enabled_features = ["config", "collision", "score", "rng"]
```

## Mental model before touching code

Think of the bag like this:

- it stores a list of remaining pieces
- when that list becomes empty, it rebuilds itself
- rebuilding means:
  - take one of each tetromino type
  - shuffle them
  - store them in the bag

Then there are 2 ways to ask for the next piece:

- `peek()`
  - look without removing
- `next_tetromino()`
  - remove and return

Those two functions must agree about which piece is "next".

## Important design decision in this implementation

In this project, the correct tested behavior is:

- use the back of the vector as the "front" of the bag

That means:

- `peek()` uses `.last()`
- `next_tetromino()` uses `.pop()`

If one function looks at the front and the other removes from the back, tests
will fail.

## Step 1: update the imports inside the random bag module

### Why this step exists

You already have:

- `SmallRng`
- `SeedableRng`

But for shuffling you also need:

- `SliceRandom`

### What to replace

Inside the `random` module in `src/bag.rs`, find:

```rust
use rand::{SeedableRng, rngs::SmallRng};
```

Replace it with:

```rust
use rand::{SeedableRng, rngs::SmallRng, seq::SliceRandom};
```

### Why `SliceRandom` matters

This trait gives you `.shuffle(...)` on vectors and slices.

Without it, you cannot randomize the bag order the way the assignment expects.

## Step 2: replace `RandomBag::from_seed`

### Why this step exists

Tests need to say:

"given this exact seed, the bag order should always come out the same"

That is what makes RNG testable.

### What to replace

Find the starter function:

```rust
pub fn from_seed(_seed: u64) -> Self {
    todo!("Create an empty bag, seed the RNG from the given value.")
}
```

Replace it with:

```rust
pub fn from_seed(seed: u64) -> Self {
    // `seed: u64` is the fixed number that makes the random order repeat.
    // Example: using seed `727` should always give the same shuffled order.
    // That is what makes seeded tests deterministic.
    Self {
        // Start empty so the first peek/pop forces a refill.
        remaining_pieces: vec![],
        // Build a reproducible RNG from the provided seed.
        rng: SmallRng::seed_from_u64(seed),
    }
}
```

### What this function is doing in simple English

It builds a bag that:

- currently has no pieces stored
- remembers a seeded random number generator

Later, when the bag first needs a piece, it will refill using that RNG.

## Step 3: replace `refill`

### Why this step exists

This is the heart of the feature.

When the bag runs out, this function rebuilds it.

### What to replace

Find the starter function:

```rust
fn refill(&mut self) {
    debug_assert!(self.remaining_pieces.is_empty());
    todo!()
}
```

Replace it with:

```rust
fn refill(&mut self) {
    // This function should only run when the bag is empty.
    debug_assert!(self.remaining_pieces.is_empty());
    // Rebuild the bag with exactly one copy of each tetromino type.
    self.remaining_pieces = ALL_TETROMINO_TYPES
        .into_iter()
        .map(get_tetromino)
        .collect::<Vec<_>>();
    // Shuffle the vector in place using the stored RNG.
    // The shuffled order is what later `peek` and `pop` will follow.
    self.remaining_pieces.shuffle(&mut self.rng);
}
```

## Step 4: understand `refill` in 3 simple parts

### Part 1: only refill when empty

This line:

```rust
debug_assert!(self.remaining_pieces.is_empty());
```

means:

"this function is supposed to be called only when there are no pieces left"

That keeps the bag logic clean.

### Part 2: build one of each piece

This part:

```rust
self.remaining_pieces = ALL_TETROMINO_TYPES
    .into_iter()
    .map(get_tetromino)
    .collect::<Vec<_>>();
```

means:

- take every tetromino type once
- turn each type into a real tetromino value
- collect them into a vector

So before shuffling, the bag contains exactly one copy of each shape.

### Part 3: shuffle with the stored RNG

This line:

```rust
self.remaining_pieces.shuffle(&mut self.rng);
```

randomizes the order.

Because it uses the stored RNG:

- a fixed seed gives a repeatable order
- a runtime RNG gives a different order for real play

## Step 5: replace `next_tetromino`

### Why this step exists

This is the function that actually consumes the next piece.

It must:

1. refill first if needed
2. remove the next piece from the correct end of the vector

### What to replace

Find the starter function:

```rust
fn next_tetromino(&mut self) -> Tetromino {
    todo!("Get the next tetromino from the bag.  Refill it if necessary")
}
```

Replace it with:

```rust
fn next_tetromino(&mut self) -> Tetromino {
    // Refill first if there are no pieces left to take.
    if self.remaining_pieces.is_empty() {
        self.refill();
    }

    // Take from the back so seeded tests match the expected order.
    // This must stay consistent with `peek`, which uses `last()`.
    self.remaining_pieces
        .pop()
        .expect("bag should contain a tetromino after refill")
}
```

### Why `.pop()` is correct here

`.pop()` removes from the back of the vector.

Since this implementation treats the back as "the next piece", `.pop()` is the
correct consuming operation.

## Step 6: replace `peek`

### Why this step exists

The preview system needs to know what the next piece will be, without removing
it from the bag.

So `peek()` must look at the same place that `next_tetromino()` would remove
from.

### What to replace

Find the starter function:

```rust
fn peek(&mut self) -> Tetromino {
    todo!()
}
```

Replace it with:

```rust
fn peek(&mut self) -> Tetromino {
    // Refill first if there are no pieces left to inspect.
    if self.remaining_pieces.is_empty() {
        self.refill();
    }

    // Look at the same back element that `next_tetromino` will remove.
    // Using `last()` here keeps `peek` and `pop()` in sync.
    *self
        .remaining_pieces
        .last()
        .expect("bag should contain a tetromino after refill")
}
```

## Step 7: understand the most important agreement in this feature

These two lines are a pair:

```rust
self.remaining_pieces.pop()
```

and

```rust
self.remaining_pieces.last()
```

That pairing means:

- `peek()` sees the back
- `next_tetromino()` removes from the back

So both functions are talking about the same "next" piece.

### Why this matters so much

If you accidentally write:

- `peek()` using `.first()`
- but `next_tetromino()` using `.pop()`

then:

- the preview says one thing
- the actual next piece is another thing

and RNG tests fail.

## Example

Imagine the bag vector is:

```text
[J, T, O]
```

and we treat the back as the next piece.

Then:

- `peek()` should return `O`
- `next_tetromino()` should remove and return `O`

After that, the bag becomes:

```text
[J, T]
```

That is the behavior the tests expect.

## Common beginner confusion here

### "Why start with an empty bag?"

Because the refill logic is supposed to own bag creation.

That gives one consistent rule:

- if empty -> refill

instead of having special setup logic and special refill logic.

### "Why not refill immediately in `from_seed`?"

You could design a system that way, but the tested implementation here expects:

- bag starts empty
- first `peek()` or `next_tetromino()` triggers refill

### "Why do we need deterministic randomness?"

Because tests need repeatable behavior.

If the same seed gave different sequences, the tests would be unreliable.

## Tests for this PR

### Main RNG tests

Run:

```bash
cargo nextest run --features test,config,collision,score,rng --test end_to_end \
  random_bag_impl1 random_bag_impl2 random_bag_impl3 \
  --no-fail-fast
```

This checks:

- refill behavior
- seeded order
- `peek()` / `next_tetromino()` consistency

### Re-run config tests too

Then run:

```bash
cargo test --features test,config,rng --lib config::
```

Why?

Because config also has RNG-related bag config tests once `rng` is enabled.

## When this PR is done

Stop this PR when:

- seeded runs are deterministic
- the bag refills only when empty
- refill creates one copy of each tetromino
- `peek()` matches the next piece that `next_tetromino()` will remove

Do not start hard drop in the same PR.
