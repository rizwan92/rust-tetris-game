# RNG

## Goal

Finish the random bag feature in:

- `src/bag.rs`

This guide assumes config is already done.

## 1. Replace `RandomBag::from_seed`

Paste this function:

```rust
/// Create a bag from given starting RNG seed.
pub fn from_seed(seed: u64) -> Self {
    // Start with an empty bag and a deterministic RNG state.
    // Example:
    // using seed 727 should always produce the same sequence of pieces.
    Self {
        remaining_pieces: vec![],
        rng: SmallRng::seed_from_u64(seed),
    }
}
```

## 2. Replace `refill`

Paste this function:

```rust
// Refill the bag if it is empty.  This should create one of each
// tetromino, shuffle them, and put them in the bag.
fn refill(&mut self) {
    debug_assert!(self.remaining_pieces.is_empty());
    // Build one of each canonical tetromino.
    // Example:
    // before shuffling, this contains S, Z, L, J, T, O, I in that order.
    self.remaining_pieces = ALL_TETROMINO_TYPES.map(get_tetromino).to_vec();

    // Shuffle the vector in place using the bag's RNG state.
    // The tests depend on this exact style of shuffling, so we do not
    // invent any custom randomization logic here.
    self.remaining_pieces.shuffle(&mut self.rng);
}
```

## 3. Replace the `Bag for RandomBag` impl

Paste this block:

```rust
impl Bag for RandomBag {
    fn next_tetromino(&mut self) -> Tetromino {
        // The bag refills itself lazily the first time it is queried.
        // Example:
        // if the bag is empty and we ask for the next piece, create a new
        // shuffled 7-piece bag first.
        if self.remaining_pieces.is_empty() {
            self.refill();
        }

        // Remove the next piece from the same end that `peek()` reads from.
        // Using the back of the vector keeps both operations simple.
        self.remaining_pieces
            .pop()
            .expect("bag should contain a tetromino after refill")
    }

    fn peek(&mut self) -> Tetromino {
        // `peek()` must agree with `next_tetromino()`.
        // That means it also looks at the back of the vector.
        if self.remaining_pieces.is_empty() {
            self.refill();
        }

        *self
            .remaining_pieces
            .last()
            .expect("bag should contain a tetromino after refill")
    }
}
```

## 4. Local checks

Run:

```bash
cargo nextest run --features test --retries 0 --test-threads=1 --test end_to_end random_bag_impl1 random_bag_impl2 random_bag_impl3 --no-fail-fast
```
