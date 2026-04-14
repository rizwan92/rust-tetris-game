# Viva Short Note

I implemented the assignment feature by feature in dependency order: baseline,
config, collision, score, rng, hard drop, and hold.

Most of the work was done by filling the intended `TODO` logic in the provided
files.

The only small extra changes were in
[src/lib.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/lib.rs),
[src/board.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/board.rs),
and [src/hold.rs](/Users/rizwan/Desktop/rizwan/projects/milestone-1-Varun1421-main/src/hold.rs),
because Bevy schedule timing and input timing needed small fixes for the tests.

So the solution still follows the assignment spirit:
fill the provided logic first, keep changes minimal, and only add the smallest
runtime fixes that testing proved were necessary.
