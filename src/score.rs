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

    if lines_cleared == 0 {
        return;
    }

    let multiplier = match lines_cleared {
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => unreachable!("more than 4 lines cannot be cleared at once"),
    };

    state.score += multiplier * (state.level + 1);

    state.lines_cleared += lines_cleared;
    state.lines_cleared_since_last_level += lines_cleared;

    while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        state.lines_cleared_since_last_level -= (state.level + 1) * 10;
        state.level += 1;

        let carried_elapsed = state.gravity_timer.elapsed();
        let new_duration = state.drop_interval();
        state.gravity_timer = Timer::new(new_duration, TimerMode::Repeating);
        if carried_elapsed >= new_duration {
            state.gravity_timer.almost_finish();
        } else {
            state.gravity_timer.set_elapsed(carried_elapsed);
        }
    }
}

/// Update the score text.
fn update_score_text(state: Res<GameState>, mut text: Single<&mut Text, With<ScoreMarker>>) {
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
