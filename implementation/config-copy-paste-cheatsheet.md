# Config Copy-Paste Cheatsheet

Use this file after baseline is complete.

## Commenting Rule For This File

- the comments inside each Rust snippet explain the changed lines in simple English
- when the function signature changes, the parameter lines are also explained
- keep the comments while studying, even if you remove them later

## Enable the feature

Set [Cargo.toml](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/Cargo.toml) to:

```toml
enabled_features = ["config"]
```

## What this feature changes

- enables reading game configuration from JSON
- makes `GameConfig` serializable/deserializable
- implements `GameConfig::load`
- adds the unit tests the assignment explicitly expects
- explains the changed lines directly in the snippets so they are easier to paste and study

## `src/config.rs`

### Replace the `GameConfig` derive

Replace:

```rust
#[derive(Debug, PartialEq, Eq)]
```

with:

```rust
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
```

### Replace `GameConfig::load`

File:
- [config.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/config.rs)

```rust
pub fn load(json: &str) -> Result<Self, serde_json::Error> {
    // `json: &str` means this function receives raw JSON text.
    // Example: `{"bag":"Deterministic","animate_title":true}`.
    // The return type says we either get a `GameConfig` or a JSON parsing error.

    // Ask serde_json to deserialize the whole config from the input string.
    // This one line is enough because the struct already derives Serialize and Deserialize.
    // So serde knows how to map the JSON fields into the Rust fields.
    serde_json::from_str(json)
}
```

### Add these unit tests inside the existing `#[cfg(test)] mod tests`

Paste these below `default_game_state` and above the `#[cfg(feature = "rng")]` test.

```rust
#[test]
#[cfg(feature = "config")]
fn load_deterministic_config() {
    // This JSON matches the deterministic config shape used by the provided test data.
    let json = r#"{"bag":"Deterministic","animate_title":true}"#;

    // Load the config from JSON text.
    let cfg = GameConfig::load(json).expect("config should parse");

    // The bag should be deterministic.
    assert_eq!(cfg.bag, BagType::Deterministic);
    // The title animation flag should be true.
    assert!(cfg.animate_title);
}

#[test]
#[cfg(feature = "config")]
fn load_animate_title_false() {
    // This JSON flips only the title animation flag.
    let json = r#"{"bag":"Deterministic","animate_title":false}"#;

    // Load the config from JSON text.
    let cfg = GameConfig::load(json).expect("config should parse");

    // The bag should still be deterministic.
    assert_eq!(cfg.bag, BagType::Deterministic);
    // The title animation flag should now be false.
    assert!(!cfg.animate_title);
}

#[test]
#[cfg(feature = "config")]
fn load_rejects_invalid_json() {
    // This is intentionally malformed JSON.
    let json = r#"{"bag":"Deterministic","animate_title":tru"#;

    // Loading invalid JSON should fail.
    assert!(GameConfig::load(json).is_err());
}
```

## Notes for later features

- When you reach `rng`, come back and add config-loading tests for:
  - `{"bag":{"FixedSeed":727},"animate_title":true}`
  - `{"bag":"RandomSeed","animate_title":false}`
- The provided config files under `test_data/` already use these serde shapes.

## Test commands

Run the smallest checks first:

```bash
cargo test load_deterministic_config
cargo test load_animate_title_false
cargo test load_rejects_invalid_json
```

Then run the config-related source tests:

```bash
cargo test load_
cargo test default_game_state
```

Then run a regression sweep for baseline + config:

```bash
cargo test --features test --test end_to_end -- --test-threads=1
```

## Acceptance checkpoint

Do not move to `collision` until:

- the three new `load_*` tests pass
- `default_game_state` still passes
- the end-to-end suite still builds and runs with `enabled_features = ["config"]`
