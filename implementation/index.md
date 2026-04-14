# Implementation Roadmap

Use this file as the entry point for the assignment workflow.

Helpful companion note:

- [minimum-required-non-todo-changes.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/minimum-required-non-todo-changes.md)
- [one-page-submission-note.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/one-page-submission-note.md)
- [viva-short-note.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/viva-short-note.md)

## Documentation Style

Every implementation file in this folder should now be read with this rule:

1. keep the inline comments when you copy the snippet
2. read the parameter comments too, not only the body comments
3. if a function signature was changed, the comments near that signature explain why each argument exists

The goal is simple:

- you should be able to read one line
- then read the comment directly near that line
- then understand what that line is doing in plain English

One more sync rule matters:

1. the code snippet in each cheat sheet should match the real Rust source for that feature
2. the inline comments in the cheat sheet should also exist in the actual source files
3. if you later revert `src/` and paste from the cheat sheet, you should get the same working code and the same study comments

## Order

Open and apply the cheat sheets in this exact order:

1. [baseline-copy-paste-cheatsheet.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/baseline-copy-paste-cheatsheet.md)
2. [config-copy-paste-cheatsheet.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/config-copy-paste-cheatsheet.md)
3. [collision-copy-paste-cheatsheet.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/collision-copy-paste-cheatsheet.md)
4. [score-copy-paste-cheatsheet.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/score-copy-paste-cheatsheet.md)
5. [rng-copy-paste-cheatsheet.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/rng-copy-paste-cheatsheet.md)
6. [hard-drop-copy-paste-cheatsheet.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/hard-drop-copy-paste-cheatsheet.md)
7. [hold-copy-paste-cheatsheet.md](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/implementation/hold-copy-paste-cheatsheet.md)

## How to use each file

For every feature:

1. Set `enabled_features` exactly as shown in that cheat sheet.
2. Paste only the snippets from that feature doc.
3. Run the smallest tests listed in that doc first.
4. Run the feature-level test command next.
5. Run the cumulative regression command last.
6. Move to the next cheat sheet only when the acceptance checkpoint is satisfied.

## Feature dependency map

- `baseline` is the foundation for everything.
- `config` should be done before every later required feature.
- `collision` should be done before `score`, `hard_drop`, and `hold`.
- `score` depends on `collision`.
- `rng` depends on `config`.
- `hard_drop` depends on `collision`.
- `hold` depends on `collision`.

## Practical rule

If a later feature test fails in a strange way, first rerun:

1. the smallest test from the current feature
2. the previous feature’s regression command

That usually tells you whether the bug is new or whether an earlier feature was broken by a later paste.

## TODO-First Rule

The assignment should still be approached as "fill the TODOs first".

After re-checking the starter repo and the tests, the honest summary is:

- `data.rs`, `config.rs`, `collision.rs`, `score.rs`, `bag.rs`, and most of `hard_drop.rs` are straightforward TODO-style fills
- `board.rs` is mostly TODO-fill work too, but later features add a few small timing markers there
- `hold.rs` needs one small Bevy input-queue helper even though the starter file only exposes `swap_hold`
- `lib.rs` needs the validated schedule ordering from the working source, because the starter-style `Update` ordering reintroduced flaky timing on macOS

So the right mental model is:

1. fill the intended TODO logic first
2. keep extra wiring only where the tests prove it is required
3. avoid inventing new systems or new architecture unless a validated test forces it

## Recommended stopping points

- Stop after `baseline` and make sure the game loop feels correct.
- Stop after `collision` because many core gameplay behaviors become real there.
- Stop after `score` because that stabilizes level and gravity behavior.
- Stop after `hold` because that completes the required feature set.
