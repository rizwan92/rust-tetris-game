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
            // Start with an empty bag and a deterministic RNG state.
            // Example:
            // using seed 727 should always produce the same sequence of pieces.
            Self {
                remaining_pieces: vec![],
                rng: SmallRng::seed_from_u64(seed),
            }
        }

        // Refill the bag if it is empty.  This should create one of each
        // tetromino, shuffle them, and put them in the bag.
        fn refill(&mut self) {
            debug_assert!(self.remaining_pieces.is_empty());
            // Build one of each canonical tetromino.
            // Example:
            // before shuffling, this contains S, Z, L, J, T, O, I in that order.
            self.remaining_pieces = ALL_TETROMINO_TYPES.map(get_tetromino).to_vec();

            // Shuffle the vector in place using the bag's RNG state.
            // The tests depend on this exact style of shuffling, so we do not
            // invent any custom randomization logic here.
            self.remaining_pieces.shuffle(&mut self.rng);
        }
    }

    impl Bag for RandomBag {
        fn next_tetromino(&mut self) -> Tetromino {
            // The bag refills itself lazily the first time it is queried.
            // Example:
            // if the bag is empty and we ask for the next piece, create a new
            // shuffled 7-piece bag first.
            if self.remaining_pieces.is_empty() {
                self.refill();
            }

            // Remove the next piece from the same end that `peek()` reads from.
            // Using the back of the vector keeps both operations simple.
            self.remaining_pieces
                .pop()
                .expect("bag should contain a tetromino after refill")
        }

        fn peek(&mut self) -> Tetromino {
            // `peek()` must agree with `next_tetromino()`.
            // That means it also looks at the back of the vector.
            if self.remaining_pieces.is_empty() {
                self.refill();
            }

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
