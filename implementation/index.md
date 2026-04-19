# Implementation Guides

This directory was rebuilt from scratch for the current working code on
`cleanup-refactor`.

These guides are for students who want:

1. the original starter file layout to stay in place
2. copy-paste snippets for each feature
3. comments in simple English inside the snippets
4. one feature at a time, in the same order as the assignment

How to use these guides:

1. start with `01-baseline.md`
2. finish one feature completely before moving to the next
3. paste each snippet into the same file and near the same place described in
   the guide
4. run the test commands at the end of each guide

Important rule:

- keep the older starter code where it already is
- only paste the newer implementation snippets into the TODO areas or directly
  near the related starter code
- do not move whole systems around just to “clean up” the file

Feature order:

1. [Baseline](./01-baseline.md)
2. [Config](./02-config.md)
3. [Collision](./03-collision.md)
4. [Score](./04-score.md)
5. [RNG](./05-rng.md)
6. [Hard Drop](./06-hard-drop.md)
7. [Hold](./07-hold.md)

Extra reading:

1. [How Tests Work](./08-how-tests-work.md)

Suggested check after every feature:

```bash
cargo fmt --all
cargo clippy --no-default-features --features ci -- -D warnings
```

Final Linux check:

```bash
cargo nextest run --no-default-features --features ci --verbose --no-fail-fast --retries 2
```
