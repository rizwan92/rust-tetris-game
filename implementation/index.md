# Implementation Notes

This directory is where we keep the feature-by-feature notes for the rebuilt
starter-based solution.

Our branch strategy is:

1. `starter-reset` is the clean base branch.
2. Each feature starts from that base or from the previous feature branch.
3. We keep the code close to the original assignment skeleton.
4. We avoid adding new structs or helper functions unless the feature truly
   needs them.

Current feature notes:

1. [Baseline](./01-baseline.md)
2. [Config](./02-config.md)
3. [Collision](./03-collision.md)
4. [Score](./04-score.md)
5. [RNG](./05-rng.md)

Local testing rule:

- We trust compile errors, `cargo fmt`, `cargo clippy`, and deterministic tests.
- We treat some sleep-based end-to-end failures on macOS as timing noise and let
  Linux CI be the final judge for those.
