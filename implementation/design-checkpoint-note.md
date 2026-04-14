# Design Checkpoint Note

Use this note as the current discussion checkpoint before making any larger
logic changes.

## Current conclusion

The main instability is not spread across the whole project.

It is concentrated in the **active-piece lifecycle**:

1. when a new piece becomes active
2. when gravity is allowed to affect it
3. when a grounded piece starts locking
4. when the piece finally locks and is replaced

## Main files involved

Primary focus:

1. [src/board.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/board.rs)

Secondary focus:

1. [src/hold.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hold.rs)

## Why this is the current pointer

Most flaky or conflicting failures are caused by one of these:

- a newly active piece falling one row too early
- a grounded piece locking one step too early or too late
- a hold-swapped piece behaving differently from a normally spawned piece
- replay timing and ordinary timing needing different behavior

So this is the best current pointer because it targets the smallest area with
the biggest effect.

## Current recommended direction

Do **not** rewrite the whole project.

Do **not** keep patching random failing tests one by one.

Instead, if we continue redesign work, we should redesign the lifecycle rules:

1. spawn rule
2. gravity-eligibility rule
3. grounded-to-lock rule
4. lock-to-replace rule
5. hold activation rule
6. replay vs automatic timing rule

## Decision checkpoint

If the next redesign attempt works, we keep moving.

If it does not solve the problem cleanly, we return to this checkpoint and
re-evaluate from here instead of drifting into random fixes again.

## Short summary

Current best architectural pointer:

- redesign the timing/state-transition core
- keep the rest of the project structure
- treat `board.rs` as the main lifecycle owner
- treat `hold.rs` as a secondary entry point that must obey the same lifecycle rules
