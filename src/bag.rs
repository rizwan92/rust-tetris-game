//! Next-tile bag implementations

use std::any::Any;

use crate::data::*;

#[cfg(feature = "rng")]
pub use random::*;

/// A generic interface for the next tile bag
pub trait Bag: Send + Any {
    /// Take the next tetromino out of the bag
    fn next_tetromino(&mut self) -> Tetromino;
    /// Peek at the next tetromino, do not take it out.
    ///
    /// This function takes `self` as mutable in case we need to refill the bag.
    fn peek(&mut self) -> Tetromino;
}

/// A deterministic tile bag that cycles between tetrominos.
#[derive(Default, Debug)]
pub struct DeterministicBag {
    next_tile_index: usize,
}

impl Bag for DeterministicBag {
    /// Take the next tetromino out of the bag.
    fn next_tetromino(&mut self) -> Tetromino {
        let t = self.peek();
        self.next_tile_index = (self.next_tile_index + 1) % ALL_TETROMINO_TYPES.len();
        t
    }

    /// Peek at the next tetromino.
    ///
    /// This function is `&mut self` in case the bag needs to be refilled.
    fn peek(&mut self) -> Tetromino {
        get_tetromino(ALL_TETROMINO_TYPES[self.next_tile_index])
    }
}

#[cfg(feature = "rng")]
mod random {
    use super::*;
    use rand::{SeedableRng, rngs::SmallRng, seq::SliceRandom};

    /// The random tile bag
    #[derive(PartialEq, Debug)]
    pub struct RandomBag {
        remaining_pieces: Vec<Tetromino>,
        rng: SmallRng,
    }

    impl RandomBag {
        /// Create a bag from given starting RNG seed.
        pub fn from_seed(seed: u64) -> Self {
            // `seed: u64` is the fixed number that makes the random order repeat.
            // Example: using seed `727` should always give the same shuffled order.
            // That is what makes seeded tests deterministic.
            Self {
                // Start empty so the first peek/pop forces a refill.
                remaining_pieces: vec![],
                // Build a reproducible RNG from the provided seed.
                rng: SmallRng::seed_from_u64(seed),
            }
        }

        // Refill the bag if it is empty.  This should create one of each
        // tetromino, shuffle them, and put them in the bag.
        fn refill(&mut self) {
            // This function should only run when the bag is empty.
            debug_assert!(self.remaining_pieces.is_empty());
            // Rebuild the bag with exactly one copy of each tetromino type.
            self.remaining_pieces = ALL_TETROMINO_TYPES
                .into_iter()
                .map(get_tetromino)
                .collect::<Vec<_>>();
            // Shuffle the vector in place using the stored RNG.
            // The shuffled order is what later `peek` and `pop` will follow.
            self.remaining_pieces.shuffle(&mut self.rng);
        }
    }

    impl Bag for RandomBag {
        fn next_tetromino(&mut self) -> Tetromino {
            // Refill first if there are no pieces left to take.
            if self.remaining_pieces.is_empty() {
                self.refill();
            }

            // Take from the back so seeded tests match the expected order.
            // This must stay consistent with `peek`, which uses `last()`.
            self.remaining_pieces
                .pop()
                .expect("bag should contain a tetromino after refill")
        }

        fn peek(&mut self) -> Tetromino {
            // Refill first if there are no pieces left to inspect.
            if self.remaining_pieces.is_empty() {
                self.refill();
            }

            // Look at the same back element that `next_tetromino` will remove.
            // Using `last()` here keeps `peek` and `pop()` in sync.
            *self
                .remaining_pieces
                .last()
                .expect("bag should contain a tetromino after refill")
        }
    }

    impl Default for RandomBag {
        fn default() -> Self {
            Self {
                remaining_pieces: vec![],
                rng: SmallRng::from_os_rng(),
            }
        }
    }
}
