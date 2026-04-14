# One-Page Submission Note

## Project approach

This assignment was completed feature by feature in the intended dependency order:

1. baseline
2. config
3. collision
4. score
5. rng
6. hard_drop
7. hold

For most of the work, the implementation was done by filling the intended
`TODO` logic in the provided source files.

## Important implementation note

The project uses Bevy, so a few small runtime-order fixes were also required to
make the provided tests pass reliably on this machine.

These validated non-TODO adjustments were limited to:

1. [src/lib.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/lib.rs)
2. [src/board.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/board.rs)
3. [src/hold.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hold.rs)

## What stayed simple

These files are still mainly straightforward assignment logic:

1. [src/data.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/data.rs)
2. [src/config.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/config.rs)
3. [src/collision.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/collision.rs)
4. [src/score.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/score.rs)
5. [src/bag.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/bag.rs)
6. [src/hard_drop.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hard_drop.rs)

## Why the small extra changes were needed

- Bevy schedule timing affected some gameplay/input tests
- replay-based tests and normal timing tests needed slightly different lock timing
- hold input could be missed without a small queued-input step

## Final summary

The overall implementation still follows the assignment spirit:

1. fill the provided TODO logic first
2. keep changes minimal
3. add only the smallest extra Bevy-specific fixes that testing proved were necessary
