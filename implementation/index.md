# Final Implementation Guide

This folder is the final study set for the project.

The goal of these files is simple:

1. help you understand what game you are building
2. help you understand how the codebase is organized
3. help you rebuild the final working solution from the starter code
4. help you prepare one clean pull request per feature

If you follow the guides in order, you do not need to guess where the code
should go. Each guide tells you what file to open, what function or block to
find, what snippet to paste, and what tests to run before moving forward.

## How to use this folder

Use this folder like a lab manual.

1. start from the starter code inside `original-repo/`
2. open the matching final file in `src/`
3. read the guide for the current feature
4. copy the guided snippet into the right place
5. run the tests written in that guide
6. only then move to the next feature

The most important rule is: do not jump ahead.

This project becomes much easier when you build it in the same order that the
features depend on each other.

## What game you are building

This is a Tetris-like game built with Bevy.

At a high level, the game contains:

- one active tetromino that the player controls
- obstacle blocks that stay on the board after a piece locks
- a next-piece preview window
- later, a hold window
- gravity that moves the active piece down over time
- score and level progression
- a random bag that decides which piece appears next

So even though the project has several Rust files, the game itself is built
from a few repeated ideas:

- create a piece
- move it
- rotate it
- stop it when it collides
- lock it into the board
- spawn the next piece

## The codebase in simple English

The project may look large at first, but the core model is actually small.

- `Cell` means one board coordinate
- `Tetromino` means one tetris piece made of 4 cells
- `GameState` stores global game information like gravity, score, level, and bag
- `Active` marks the piece the player is controlling right now
- `Obstacle` marks cells that are already locked into the board
- `Next` marks the preview piece
- `Hold` marks the held piece

So when you read the code, try not to think of it as "many random systems".
Think of it as one game loop with a few simple steps.

## The most important concept in this project

The hardest part of the project is not the rotation formula.
The hardest part is the lifecycle of the active piece.

Every feature depends on this lifecycle:

1. a piece spawns
2. the player can move or rotate it
3. gravity pulls it downward
4. if it cannot move down anymore, lock timing starts
5. once it locks, it becomes obstacle blocks
6. then the next piece becomes active

Later features like collision, score, hard drop, and hold all build on top of
this same flow.

That is why the baseline guide matters so much.
If the baseline piece lifecycle is not stable, later features will feel much
more confusing than they really are.

## Why these guides use both `original-repo/` and the final `src/`

These guides were written by comparing:

- the starter university code inside `original-repo/`
- the final passing implementation in the main `src/` folder

This is helpful for two reasons.

First, you can see what the university originally gave you.
That tells you which TODOs or missing logic were expected from students.

Second, you can also see the final target that actually passes the tests.
That saves you from guessing where a fix should go or how the finished shape
should look.

In short:

- `original-repo/` shows the starting point
- `src/` shows the final target
- `implementation/` explains the journey from one to the other

## Why some guides add small helpers

Most of the work in this assignment is normal TODO-filling.

But in a few places, especially around Bevy timing and active-piece lifecycle,
the fully working solution needs a small helper or a small runtime fix in
addition to the obvious TODO logic.

That is not extra overengineering.
It is simply the practical code shape that made the test suite stable.

So if a guide asks you to add:

- a helper component
- a small helper function
- a scheduling line
- a shared state value

it is because that piece supports the final working behavior, not because the
feature became conceptually huge.

## Recommended PR order

Please build the project in this order.

1. [01-baseline-pr-guide.md](./01-baseline-pr-guide.md)
2. [02-config-pr-guide.md](./02-config-pr-guide.md)
3. [03-collision-pr-guide.md](./03-collision-pr-guide.md)
4. [04-score-pr-guide.md](./04-score-pr-guide.md)
5. [05-rng-pr-guide.md](./05-rng-pr-guide.md)
6. [06-hard-drop-pr-guide.md](./06-hard-drop-pr-guide.md)
7. [07-hold-pr-guide.md](./07-hold-pr-guide.md)

This order matters.

For example:

- `score` depends on line clear information from `collision`
- `hard_drop` depends on baseline movement and collision behavior
- `hold` depends on stable spawning and active-piece handling

So if you try to do later guides first, the code will seem harder than it is.

## What to do before each PR

Before you start a feature, open both versions of the file you are about to edit.

- open `original-repo/src/...`
- open `src/...`

Then ask these three questions:

1. what did the starter code already provide?
2. what logic is still missing?
3. what does the final working version look like?

This makes the guide much easier to follow because you are not copying blindly.
You are comparing:

- starter shape
- final shape
- guided explanation

## How each guide is written

Each feature guide is designed to be practical.

Inside each guide you will usually see:

- the goal of the feature in simple English
- the file or function you need to find
- the exact snippet you need to paste
- an explanation of what that code is doing
- the tests you should run before the next PR

So the guides are not just notes.
They are meant to be used as step-by-step implementation instructions.

## Suggested PR naming

If you want a simple clean sequence, use this naming:

- PR 1: `baseline`
- PR 2: `config`
- PR 3: `collision`
- PR 4: `score`
- PR 5: `rng`
- PR 6: `hard_drop`
- PR 7: `hold`

## Final expectation

Take the project one layer at a time.

Do not worry if a later feature looks difficult when viewed alone.
Most of the confusion disappears when the earlier pieces are already working.

So the best path is:

1. finish baseline
2. verify tests
3. raise the PR
4. move to the next guide

That is the simplest and safest way to reach the final working solution.
