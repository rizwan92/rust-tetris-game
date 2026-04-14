# PR 2 Guide: Config

This PR is much smaller than baseline, but it is still important.

Why?

Because this is the first place where the game learns how to read outside data.

If config loading is wrong:

- tests may use the wrong bag
- title animation may not match the input config
- later deterministic tests become harder to trust

So even though the code change is small, the idea is important.

## What this PR is trying to achieve

At the end of this PR, the game should be able to read JSON like this:

```json
{"bag":"Deterministic","animate_title":true}
```

and turn it into a real Rust `GameConfig`.

In simple English:

- JSON text comes in
- serde reads it
- Rust structs/enums are created from it

That is the whole feature.

## Why the university asked for tests here

This feature does not have a big visible gameplay effect by itself.

So instead of waiting for later bugs, the right way to prove it works is:

- write good unit tests now
- make sure `GameConfig::load` behaves correctly now

That is why this PR is:

- a little implementation code
- plus several useful tests

## Starter files to compare

- `original-repo/src/config.rs`
- `original-repo/docs/config.md`

## File you will change

- `src/config.rs`

## Feature flag state

Use:

```toml
enabled_features = ["config"]
```

## Mental model before touching code

There are only 2 main config pieces in this PR:

- `bag`
  - which bag implementation the game should use
- `animate_title`
  - whether the title animation should run

And there are only 2 main Rust types involved:

- `GameConfig`
  - the full config object
- `BagType`
  - the enum describing which bag style to use

For serde to read JSON into Rust, those types must derive:

- `Serialize`
- `Deserialize`

That is the first step.

## Step 1: derive serde support on the config types

### Why this step exists

Without `Deserialize`, serde cannot turn JSON text into your Rust type.

Without `Serialize`, the type is incomplete for normal serde usage and tests.

So the first thing we do is teach Rust:

"these config types are allowed to be read from and written to JSON."

### What to do

Open `src/config.rs`.

Find the starter derive on `GameConfig`.

It will look like this:

```rust
#[derive(Debug, PartialEq, Eq)]
pub struct GameConfig {
```

Replace it with:

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameConfig {
```

Then find the derive on `BagType`.

Replace it with:

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum BagType {
```

### What this fixes

This prepares both types so serde can understand them.

If you skip this step, `GameConfig::load` cannot work.

## Step 2: replace `GameConfig::load`

### Why this step is so short

Once the types derive `Deserialize`, the actual load function becomes very small.

Serde already knows how to do the heavy work.

So this feature is one of those nice cases where:

- good type design
- makes implementation tiny

### What to replace

In `src/config.rs`, find the starter function:

```rust
#[cfg(feature = "config")]
/// Read a configuration from given JSON data.
pub fn load(_json: &str) -> Result<Self, serde_json::Error> {
    todo!()
}
```

Replace it with:

```rust
#[cfg(feature = "config")]
/// Read a configuration from given JSON data.
pub fn load(json: &str) -> Result<Self, serde_json::Error> {
    // `json: &str` means this function receives raw JSON text.
    // Example input: `{"bag":"Deterministic","animate_title":true}`.
    // We return either a parsed config or a serde parsing error.

    // Ask serde_json to build the whole `GameConfig` from the JSON text.
    // This works because the struct already derives `Deserialize`.
    // So serde knows how to map JSON fields into Rust fields.
    serde_json::from_str(json)
}
```

### What this line really means

This line:

```rust
serde_json::from_str(json)
```

means:

"take the JSON string and try to build a `GameConfig` from it"

If the JSON is valid:

- you get `Ok(GameConfig { ... })`

If the JSON is broken:

- you get `Err(...)`

That is exactly what we want.

## Step 3: add unit tests in the existing test module

### Why these tests matter

This PR is not mainly about visible gameplay.

So your tests are the proof that:

- the right bag variant was loaded
- the boolean flag was loaded
- broken JSON fails cleanly

### Where to put them

Stay inside the existing `#[cfg(test)] mod tests` at the bottom of
`src/config.rs`.

Do not create a new test file.

### Paste these tests

```rust
#[test]
#[cfg(feature = "config")]
fn load_deterministic_config() {
    // This JSON uses the simple deterministic bag form.
    // It matches the config style used by the provided test data.
    let json = r#"{"bag":"Deterministic","animate_title":true}"#;
    // Parse the JSON into a real `GameConfig`.
    let cfg = GameConfig::load(json).expect("config should parse");
    // Check that the bag variant is correct after parsing.
    assert_eq!(cfg.bag, BagType::Deterministic);
    // Check that the title-animation flag stayed true.
    assert!(cfg.animate_title);
}

#[test]
#[cfg(feature = "config")]
fn load_animate_title_false() {
    // This JSON keeps the same bag but changes the title flag.
    let json = r#"{"bag":"Deterministic","animate_title":false}"#;
    // Parse the JSON into a real `GameConfig`.
    let cfg = GameConfig::load(json).expect("config should parse");
    // The bag should still be deterministic.
    assert_eq!(cfg.bag, BagType::Deterministic);
    // The animation flag should now be false.
    assert!(!cfg.animate_title);
}

#[test]
#[cfg(feature = "config")]
fn load_rejects_invalid_json() {
    // This JSON is intentionally broken at the end.
    // The goal of this test is to make sure invalid config text fails cleanly.
    let json = r#"{"bag":"Deterministic","animate_title":tru"#;
    // Invalid JSON should return an error instead of a valid config.
    assert!(GameConfig::load(json).is_err());
}

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

## Step 4: understand why two tests are behind `rng`

You may notice these two tests:

- `load_fixed_seed_config`
- `load_random_seed_config`

have:

```rust
#[cfg(all(feature = "config", feature = "rng"))]
```

### Why that is correct

Those bag variants do not exist unless `rng` is enabled.

So:

- it is safe to add the tests now
- but they should only compile later when `rng` exists too

That gives you a nice future-ready test set without breaking this PR.

## Common beginner confusion here

### "Why is the code so small?"

Because serde is doing the real parsing work for you.

That is normal.

A feature is still valid even if the implementation is short, as long as the
behavior is correct and well-tested.

### "Why are we testing strings instead of gameplay?"

Because this PR is about reading config data, not about visible game movement.

So the best direct tests are config parsing tests.

### "Why compare the enum exactly?"

Because we want to verify not just "something parsed", but:

- the exact correct bag variant was selected

That is much stronger.

## Tests for this PR

### Main config test run

Run:

```bash
cargo test --features test,config --lib config::
```

This checks:

- deterministic config loading
- animate-title flag loading
- invalid JSON failure

### Small gameplay safety check

Then run one baseline-style gameplay check:

```bash
cargo nextest run --features test,config --test end_to_end gravity1 --no-fail-fast
```

Why this extra test?

Because it confirms that enabling `config` did not accidentally disturb the
existing baseline behavior.

## When this PR is done

Stop this PR when:

- `GameConfig` and `BagType` derive serde correctly
- `GameConfig::load` returns parsed configs from JSON
- invalid JSON returns an error
- config unit tests are green
- a small gameplay regression check is still green

Do not start collision in the same PR.
