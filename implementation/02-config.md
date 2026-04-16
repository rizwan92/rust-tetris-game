# Config

## Goal

Finish the config feature with the smallest possible changes in:

- `src/config.rs`

This guide assumes baseline is already done.

## 1. Add the right derives to `GameConfig`

Make sure `GameConfig` looks like this:

```rust
/// Game configuration to read from the user or from the tests.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameConfig {
    /// The type of bag for this game.
    pub bag: BagType,

    /// Whether to animate the title text.
    pub animate_title: bool,
}
```

Why:

- `Serialize` and `Deserialize` let `serde_json` read and write the config
- `Debug`, `PartialEq`, and `Eq` make the tests simple

## 2. Replace `GameConfig::load`

Find the config TODO in `src/config.rs` and replace it with:

```rust
#[cfg(feature = "config")]
/// Read a configuration from given JSON data.
pub fn load(json: &str) -> Result<Self, serde_json::Error> {
    // Deserialize the JSON text directly into `GameConfig`.
    // Example:
    // {"bag":"Deterministic","animate_title":true}
    // becomes a fully typed Rust value.
    serde_json::from_str(json)
}
```

Why:

- the assignment only needs standard JSON parsing here
- `serde_json::from_str` already does the exact job

## 3. Local checks

Run:

```bash
cargo test --features test config::tests -- --nocapture
cargo clippy --features test -- -D warnings
```
