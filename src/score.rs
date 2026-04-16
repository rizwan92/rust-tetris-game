//! Scoring subsystem

use crate::{Game, ui::mk_text};

use super::data::*;
use bevy::prelude::*;

/// Marker placed on the one UI text entity that shows score information.
#[derive(Component)]
struct ScoreMarker;

/// An event denoting that some lines are cleared
#[derive(Event)]
pub struct LinesCleared(pub u32);

/// Initialize the score text
fn setup_score_text(mut commands: Commands) {
    commands.spawn((
        crate::score::ScoreMarker,
        mk_text(
            "",
            Node {
                margin: auto().all().with_right(percent(10)),
                ..default()
            },
            None,
        ),
    ));
}

/// Update the game state when lines are cleared
fn update_score(event: On<LinesCleared>, mut state: ResMut<GameState>) {
    let lines_cleared = event.0;
    assert!(lines_cleared <= 4);

    // If the event says zero lines were cleared, nothing should change.
    // This branch is mostly defensive because our collision code only emits the
    // event for positive line clears.
    if lines_cleared == 0 {
        return;
    }

    // The assignment gives a fixed multiplier table based on how many lines
    // disappear at once.
    // Example:
    // clearing 2 lines at level 0 gives 100 * (0 + 1) = 100 points.
    let multiplier = match lines_cleared {
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => unreachable!("more than 4 lines cannot be cleared at once"),
    };

    // The score always scales with the current level + 1.
    // Example:
    // clearing a Tetris at level 2 gives 1200 * 3 = 3600 points.
    state.score += multiplier * (state.level + 1);

    // Keep both total line counters up to date.
    // `lines_cleared` is the whole-game total.
    // `lines_cleared_since_last_level` is the progress toward the next level.
    state.lines_cleared += lines_cleared;
    state.lines_cleared_since_last_level += lines_cleared;

    // Level thresholds follow the assignment rule:
    // to go from level N to level N+1, we need (N + 1) * 10 cleared lines since
    // the last level up.
    // Example:
    // level 0 -> 1 needs 10 lines
    // level 1 -> 2 needs 20 more lines
    while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        state.lines_cleared_since_last_level -= (state.level + 1) * 10;
        state.level += 1;

        // Gravity should immediately use the faster interval for the new level,
        // but we should not throw away the time the current piece has already
        // spent falling.
        //
        // Example:
        // if the old timer had already accumulated 0.46s and the new interval
        // is 0.71s, the next automatic drop should happen after about 0.25s,
        // not after a completely fresh 0.71s wait.
        let carried_elapsed = state.gravity_timer.elapsed();
        let new_duration = state.drop_interval();
        state.gravity_timer = Timer::new(new_duration, TimerMode::Repeating);
        if carried_elapsed >= new_duration {
            // If the new level is so fast that this carry would already have
            // finished the timer, schedule the drop on the very next fixed
            // step instead of inventing a brand-new full interval.
            state.gravity_timer.almost_finish();
        } else {
            state.gravity_timer.set_elapsed(carried_elapsed);
        }
    }
}

/// Update the score text.
fn update_score_text(state: Res<GameState>, mut text: Single<&mut Text, With<ScoreMarker>>) {
    // Rewrite the entire text each frame.
    // Example:
    // Score: 1200
    // Level: 1
    // Lines: 10
    text.0 = format!(
        "Score: {}\nLevel: {}\nLines: {}",
        state.score(),
        state.level(),
        state.lines_cleared
    );
}

/// Plugin that adds score tracking, leveling, and score text updates.
pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_score_text.in_set(Game))
            .add_systems(Update, update_score_text.in_set(Game))
            .add_observer(update_score);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::bag::DeterministicBag;
    use std::time::Duration;

    fn mk_game_state() -> GameState {
        GameState {
            score: 0,
            lines_cleared: 0,
            lines_cleared_since_last_level: 0,
            bag: Box::new(DeterministicBag::default()),
            level: 0,
            manual_drop_gravity: SOFT_DROP_GRAVITY,
            gravity_timer: Timer::new(GameState::initial_drop_interval(), TimerMode::Repeating),
        }
    }

    #[test]
    fn score_single_line_at_level_zero() {
        let mut state = mk_game_state();

        state.score += 40 * (state.level + 1);
        state.lines_cleared += 1;
        state.lines_cleared_since_last_level += 1;

        assert_eq!(state.score, 40);
        assert_eq!(state.lines_cleared, 1);
        assert_eq!(state.level, 0);
    }

    #[test]
    fn score_tetris_scales_with_level() {
        let mut state = mk_game_state();
        state.level = 2;

        state.score += 1200 * (state.level + 1);

        assert_eq!(state.score, 3600);
    }

    #[test]
    fn level_thresholds_match_spec() {
        let mut state = mk_game_state();

        state.lines_cleared_since_last_level = 10;
        while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
            state.lines_cleared_since_last_level -= (state.level + 1) * 10;
            state.level += 1;
            state.gravity_timer = Timer::new(state.drop_interval(), TimerMode::Repeating);
        }

        assert_eq!(state.level, 1);
        assert_eq!(state.lines_cleared_since_last_level, 0);
        assert_eq!(state.gravity_timer.mode(), TimerMode::Repeating);
        assert_eq!(
            state.gravity_timer.duration(),
            Duration::from_secs_f32(GameState::INTERVALS[1] / GameState::FRAMERATE)
        );
    }
}
