# RNG Copy-Paste Cheatsheet

Use this file after config is complete.

## Commenting Rule For This File

- RNG code is easy to misunderstand, so the comments explain the order very carefully
- parameter comments explain what seed values and bag state mean
- comments inside the methods explain why `pop()` and `last()` must match each other

## Enable the feature

Set [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml) to:

```toml
enabled_features = ["config", "collision", "score", "rng"]
```

This order keeps your docs cumulative, even though `rng` technically depends only on `config`.

## What this feature changes

- enables `FixedSeed` and `RandomSeed` bag config variants
- implements seeded and unseeded random bags
- makes seeded test sequences deterministic
- extends config tests to cover the new JSON bag shapes
- explains why the order of `shuffle`, `last`, and `pop` must match

## `src/bag.rs`

### Update the `rand` import inside `mod random`

Replace:

```rust
use rand::{SeedableRng, rngs::SmallRng};
```

with:

```rust
use rand::{SeedableRng, rngs::SmallRng, seq::SliceRandom};
```

### Replace `RandomBag::from_seed`

File:
- [bag.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/bag.rs)

```rust
pub fn from_seed(seed: u64) -> Self {
    // `seed: u64` is the fixed number that makes the random order reproducible.
    // Example: if the seed is `727`, the shuffled piece order should be the same every time.
    // This is what makes deterministic RNG tests possible.

    // Start with an empty bag so the first query forces a refill.
    let remaining_pieces = vec![];
    // Create a deterministic small RNG from the provided seed.
    let rng = SmallRng::seed_from_u64(seed);

    // Return the fully initialized random bag.
    Self {
        remaining_pieces,
        rng,
    }
}
```

### Replace `RandomBag::refill`

```rust
fn refill(&mut self) {
    // Rebuild the bag with exactly one canonical copy of each tetromino type.
    self.remaining_pieces = ALL_TETROMINO_TYPES
        .into_iter()
        .map(get_tetromino)
        .collect::<Vec<_>>();

    // Shuffle the bag in place using the stored RNG.
    self.remaining_pieces.shuffle(&mut self.rng);
}
```

### Replace `RandomBag::next_tetromino`

```rust
fn next_tetromino(&mut self) -> Tetromino {
    // `&mut self` means this method is allowed to change the bag state.
    // That matters because taking the next piece removes one piece from the bag.
    // So this method cannot be `&self`.

    // Refill the bag first when it is empty.
    if self.remaining_pieces.is_empty() {
        // Rebuild and shuffle a fresh seven-piece bag.
        self.refill();
    }

    // Pop from the back so the seeded tests match the expected order.
    self.remaining_pieces
        .pop()
        .expect("bag should contain a tetromino after refill")
}
```

### Replace `RandomBag::peek`

```rust
fn peek(&mut self) -> Tetromino {
    // `peek` also uses `&mut self` because it may need to refill an empty bag.
    // Even though it does not remove a piece, it still may change internal storage first.
    // That is why the signature is mutable here too.

    // Refill the bag first when it is empty.
    if self.remaining_pieces.is_empty() {
        // Rebuild and shuffle a fresh seven-piece bag.
        self.refill();
    }

    // Peek at the same back-of-vector element that next_tetromino will pop.
    *self
        .remaining_pieces
        .last()
        .expect("bag should contain a tetromino after refill")
}
```

## `src/config.rs`

### Add these extra config tests inside the existing test module

Paste these below the config tests you added in the `config` feature doc.

```rust
#[test]
#[cfg(all(feature = "config", feature = "rng"))]
fn load_fixed_seed_config() {
    // This matches the fixed-seed JSON shape used in the provided test data.
    let json = r#"{"bag":{"FixedSeed":727},"animate_title":true}"#;

    // Load the config from JSON text.
    let cfg = GameConfig::load(json).expect("config should parse");

    // The bag should deserialize into the fixed-seed variant.
    assert_eq!(cfg.bag, BagType::FixedSeed(727));
    // The title animation flag should stay true.
    assert!(cfg.animate_title);
}

#[test]
#[cfg(all(feature = "config", feature = "rng"))]
fn load_random_seed_config() {
    // Unit enum variants serialize as a plain string in serde JSON.
    let json = r#"{"bag":"RandomSeed","animate_title":false}"#;

    // Load the config from JSON text.
    let cfg = GameConfig::load(json).expect("config should parse");

    // The bag should deserialize into the random-seed variant.
    assert_eq!(cfg.bag, BagType::RandomSeed);
    // The title animation flag should be false.
    assert!(!cfg.animate_title);
}
```

## Important seeded-order note

The provided RNG tests expect this exact behavior:

- refill the bag with all 7 tetrominoes
- `shuffle(&mut rng)`
- take pieces using `pop()` from the **back** of the vector

If you use `remove(0)` or iterate from the front, the expected seed sequences will fail.

## Test commands

Start with the direct seeded bag checks:

```bash
cargo test random_bag_impl1
cargo test random_bag_impl2
cargo test random_bag_impl3
```

Then run the new config tests:

```bash
cargo test load_fixed_seed_config
cargo test load_random_seed_config
```

Then run the whole RNG end-to-end file:

```bash
cargo test random_bag_impl
```

Then run the cumulative regression sweep:

```bash
cargo test --features test --test end_to_end -- --test-threads=1
```

## Acceptance checkpoint

Do not move to `hard_drop` until:

- all three `random_bag_impl*` tests pass
- both new config-loading tests pass
- the RNG test file passes
- the cumulative suite still passes with `enabled_features = ["config", "collision", "score", "rng"]`
