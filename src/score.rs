//! Scoring subsystem

use crate::{Game, ui::mk_text};

use super::data::*;
use bevy::prelude::*;

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
fn update_score(
    // Wake up when a line-clear event is triggered.
    event: On<LinesCleared>,
    // Mutably access the global game state so score and level can change.
    mut state: ResMut<GameState>,
) {
    // `event` tells us how many lines were cleared in one action.
    // `state` is the global game state that stores score, level, and timers.
    let lines_cleared = event.0;

    // Tetris can clear at most four lines at once.
    assert!(lines_cleared <= 4);

    // Convert line-count into the assignment's score multiplier table.
    let multiplier = match lines_cleared {
        0 => 0,
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => unreachable!("line clear count must be between 0 and 4"),
    };

    // Score formula from the assignment: multiplier * (level + 1).
    state.score += multiplier * (state.level + 1);
    // Track the total number of cleared lines across the full game.
    state.lines_cleared += lines_cleared;
    // Track the lines contributing toward the next level-up threshold.
    state.lines_cleared_since_last_level += lines_cleared;

    // Use a loop because one clear could, in theory, cross multiple thresholds.
    while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        // Remove the threshold that was just satisfied.
        state.lines_cleared_since_last_level -= (state.level + 1) * 10;
        // Increase the level by one.
        state.level += 1;
        // Recompute gravity for the new level.
        let interval = state.drop_interval();
        state.gravity_timer.set_duration(interval);
        // Do not reset the timer here.
        // Keeping the current progress matched the validated replay behavior better.
    }
}

/// Update the score text.
fn update_score_text(
    // Read the latest score-related values.
    state: Res<GameState>,
    // Mutably access the HUD text that shows score information.
    mut score_text: Single<&mut Text, With<ScoreMarker>>,
) {
    // Skip extra UI work when the score state did not change.
    if !state.is_changed() {
        return;
    }

    // Rebuild the HUD text from the latest score, level, and line totals.
    score_text.0 = format!(
        "Score: {}\nLevel: {}\nLines: {}",
        state.score(),
        state.level(),
        state.lines_cleared
    );
}

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_score_text.in_set(Game))
            .add_systems(Update, update_score_text.in_set(Game))
            .add_observer(update_score);
    }
}
