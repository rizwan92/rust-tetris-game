//! Game configuration used for testing and user-initiated setup.

use bevy::time::{Timer, TimerMode};
use serde::{Deserialize, Serialize};

use crate::{bag::*, data::*};

/// Game configuration to read from the user or from the tests.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub struct GameConfig {
    /// The type of bag for this game.
    pub bag: BagType,

    /// Whether to animate the title text.
    pub animate_title: bool,
}

impl GameConfig {
    /// Build an initial game state based on this configuration.
    pub fn build_game_state(&self) -> GameState {
        GameState {
            manual_drop_gravity: SOFT_DROP_GRAVITY,
            bag: self.bag.create_bag(),
            score: 0,
            lines_cleared: 0,
            lines_cleared_since_last_level: 0,
            level: 0,
            gravity_timer: Timer::new(GameState::initial_drop_interval(), TimerMode::Repeating),
        }
    }

    #[cfg(feature = "config")]
    /// Read a configuration from given JSON data.
    pub fn load(json: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(json)
    }
}

/// What type of bag to create in the initial state.
#[derive(Serialize, Deserialize, Debug, PartialEq, Eq)]
pub enum BagType {
    /// A deterministic bag that cycles through all tetrominos
    Deterministic,
    #[cfg(feature = "rng")]
    /// A randomized bag with a given starting random seed
    FixedSeed(u64),
    #[cfg(feature = "rng")]
    /// A randomized bag with a seed picked at runtime
    RandomSeed,
}

impl BagType {
    /// Create a new bag based on the parameters specified by this object.
    pub fn create_bag(&self) -> Box<dyn Bag + Sync> {
        use BagType::*;

        match self {
            Deterministic => Box::new(DeterministicBag::default()),
            #[cfg(feature = "rng")]
            FixedSeed(seed) => Box::new(RandomBag::from_seed(*seed)),
            #[cfg(feature = "rng")]
            RandomSeed => Box::new(RandomBag::default()),
        }
    }
}

impl Default for BagType {
    #[cfg(feature = "rng")]
    fn default() -> BagType {
        Self::RandomSeed
    }

    #[cfg(not(feature = "rng"))]
    fn default() -> BagType {
        Self::Deterministic
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::any::Any;

    #[test]
    fn default_game_state() {
        let cfg = GameConfig {
            bag: BagType::Deterministic,
            animate_title: true,
        };
        let state = cfg.build_game_state();

        assert_eq!(state.manual_drop_gravity, SOFT_DROP_GRAVITY);
        assert!((state.bag.as_ref() as &dyn Any).is::<DeterministicBag>());
        assert_eq!(state.score, 0);
        assert_eq!(state.lines_cleared, 0);
        assert_eq!(state.lines_cleared_since_last_level, 0);
        assert_eq!(state.level, 0);
        assert_eq!(
            state.gravity_timer.duration(),
            GameState::initial_drop_interval()
        );
        assert_eq!(state.gravity_timer.mode(), TimerMode::Repeating);
    }

    #[test]
    #[cfg(feature = "config")]
    fn load_deterministic_config() {
        let json = r#"{"bag":"Deterministic","animate_title":true}"#;
        let cfg = GameConfig::load(json).expect("config should parse");
        assert_eq!(cfg.bag, BagType::Deterministic);
        assert!(cfg.animate_title);
    }

    #[test]
    #[cfg(feature = "config")]
    fn load_animate_title_false() {
        let json = r#"{"bag":"Deterministic","animate_title":false}"#;
        let cfg = GameConfig::load(json).expect("config should parse");
        assert_eq!(cfg.bag, BagType::Deterministic);
        assert!(!cfg.animate_title);
    }

    #[test]
    #[cfg(feature = "config")]
    fn load_rejects_invalid_json() {
        let json = r#"{"bag":"Deterministic","animate_title":tru"#;
        assert!(GameConfig::load(json).is_err());
    }

    #[test]
    #[cfg(feature = "rng")]
    fn bag_creation() {
        let cfg = GameConfig {
            bag: BagType::Deterministic,
            animate_title: true,
        };
        let state = cfg.build_game_state();

        assert!((state.bag.as_ref() as &dyn Any).is::<DeterministicBag>());

        let cfg = GameConfig {
            bag: BagType::FixedSeed(727),
            animate_title: true,
        };
        let state = cfg.build_game_state();

        assert!((state.bag.as_ref() as &dyn Any).is::<RandomBag>());
        assert_eq!(
            (state.bag.as_ref() as &dyn Any).downcast_ref::<RandomBag>(),
            Some(&RandomBag::from_seed(727))
        );

        let cfg = GameConfig {
            bag: BagType::RandomSeed,
            animate_title: true,
        };
        let state1 = cfg.build_game_state();
        let state2 = cfg.build_game_state();

        assert!((state1.bag.as_ref() as &dyn Any).is::<RandomBag>());
        assert!((state2.bag.as_ref() as &dyn Any).is::<RandomBag>());
        assert_ne!(
            (state1.bag.as_ref() as &dyn Any).downcast_ref::<RandomBag>(),
            (state2.bag.as_ref() as &dyn Any).downcast_ref::<RandomBag>()
        );
    }

    #[test]
    #[cfg(all(feature = "config", feature = "rng"))]
    fn load_fixed_seed_config() {
        let json = r#"{"bag":{"FixedSeed":727},"animate_title":true}"#;
        let cfg = GameConfig::load(json).expect("config should parse");
        assert_eq!(cfg.bag, BagType::FixedSeed(727));
        assert!(cfg.animate_title);
    }

    #[test]
    #[cfg(all(feature = "config", feature = "rng"))]
    fn load_random_seed_config() {
        let json = r#"{"bag":"RandomSeed","animate_title":false}"#;
        let cfg = GameConfig::load(json).expect("config should parse");
        assert_eq!(cfg.bag, BagType::RandomSeed);
        assert!(!cfg.animate_title);
    }
}
