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

### Replace `RandomBag::refill`

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

### Replace `RandomBag::next_tetromino`

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

### Replace `RandomBag::peek`

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

## `src/config.rs`

### Add these extra config tests inside the existing test module

Paste these below the config tests you added in the `config` feature doc.

```rust
#[test]
#[cfg(all(feature = "config", feature = "rng"))]
fn load_fixed_seed_config() {
    // This JSON uses serde's tagged enum shape for `FixedSeed`.
    let json = r#"{"bag":{"FixedSeed":727},"animate_title":true}"#;
    // Parse the JSON into a config value.
    let cfg = GameConfig::load(json).expect("config should parse");
    // Check that the bag became the right fixed-seed variant.
    assert_eq!(cfg.bag, BagType::FixedSeed(727));
    // Check that the title flag also parsed correctly.
    assert!(cfg.animate_title);
}

#[test]
#[cfg(all(feature = "config", feature = "rng"))]
fn load_random_seed_config() {
    // Unit enum variants are represented as plain strings in serde JSON.
    let json = r#"{"bag":"RandomSeed","animate_title":false}"#;
    // Parse the JSON into a config value.
    let cfg = GameConfig::load(json).expect("config should parse");
    // Check that the random-seed variant was selected.
    assert_eq!(cfg.bag, BagType::RandomSeed);
    // Check that the title flag stayed false.
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
