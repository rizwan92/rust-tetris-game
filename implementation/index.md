# Implementation Roadmap

Use this file as the entry point for the assignment workflow.

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

## Recommended stopping points

- Stop after `baseline` and make sure the game loop feels correct.
- Stop after `collision` because many core gameplay behaviors become real there.
- Stop after `score` because that stabilizes level and gravity behavior.
- Stop after `hold` because that completes the required feature set.
