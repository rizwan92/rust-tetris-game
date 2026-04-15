# Config

## Goal

Implement the `config` feature by filling only the starter skeleton in:

- `Cargo.toml`
- `src/config.rs`

This feature is intentionally small.

The assignment spec itself says the implementation is very short, and most of
the real work is writing the tests carefully.

## Step 1: Enable the feature in `Cargo.toml`

Find this line in [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml):

```toml
enabled_features = []
```

Replace it with:

```toml
enabled_features = ["config"]
```

Why:

- the assignment says each feature branch should enable the feature it is
  implementing
- this also makes the CI `ci` feature pull in `config` through `common`

## Step 2: Make `GameConfig` serializable and deserializable

Find this in [src/config.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/config.rs):

```rust
#[derive(Debug, PartialEq, Eq)]
pub struct GameConfig {
```

Replace it with:

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameConfig {
```

Why:

- serde can only read JSON into a type when that type implements `Deserialize`
- it can only write the type back out when it implements `Serialize`

## Step 3: Replace `GameConfig::load`

Find:

```rust
pub fn load(_json: &str) -> Result<Self, serde_json::Error> {
    todo!()
}
```

Replace it with:

```rust
pub fn load(json: &str) -> Result<Self, serde_json::Error> {
    // Deserialize the JSON text directly into `GameConfig`.
    // Example:
    // {"bag":"Deterministic","animate_title":true}
    // becomes a fully typed Rust value.
    serde_json::from_str(json)
}
```

Why this works:

- the JSON string is parsed by serde_json
- serde uses the derived `Deserialize` implementation on `GameConfig`
- `BagType` is already serde-enabled, so the bag field works too

## Step 4: Add the config tests

Add these tests inside the existing `#[cfg(test)]` module in [src/config.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/config.rs):

```rust
#[test]
#[cfg(feature = "config")]
fn load_deterministic_config() {
    // This is the smallest valid config in the baseline/config stage.
    let json = r#"{"bag":"Deterministic","animate_title":true}"#;

    // Parse the JSON into the strongly typed game config.
    let cfg = GameConfig::load(json).expect("config JSON should parse");

    // Check that each field was read correctly.
    assert_eq!(
        cfg,
        GameConfig {
            bag: BagType::Deterministic,
            animate_title: true,
        }
    );
}

#[test]
#[cfg(feature = "config")]
fn load_animate_title_false() {
    // This test focuses only on the animate_title flag.
    let json = r#"{"bag":"Deterministic","animate_title":false}"#;

    let cfg = GameConfig::load(json).expect("config JSON should parse");

    assert!(!cfg.animate_title);
    assert_eq!(cfg.bag, BagType::Deterministic);
}

#[test]
#[cfg(feature = "config")]
fn load_rejects_invalid_json() {
    // Missing the closing brace, so serde_json should reject it.
    let json = r#"{"bag":"Deterministic","animate_title":true"#;

    assert!(GameConfig::load(json).is_err());
}
```

Why these tests are enough for this feature right now:

- one test checks a valid config with `animate_title = true`
- one test checks a valid config with `animate_title = false`
- one test checks invalid JSON

That matches the stage we are currently in because:

- only `Deterministic` bag exists in the non-`rng` stage
- the other bag variants will be tested later in the `rng` feature

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
cargo clippy --features test -- -D warnings
```

This feature should not depend on timing-sensitive end-to-end behavior, so local
results here are good signal even on macOS.

## Summary

This feature should end with:

- `GameConfig` being serde-ready
- `GameConfig::load` working
- clear unit tests proving valid and invalid config loading

After this branch is stable, the next feature should be `collision`.
