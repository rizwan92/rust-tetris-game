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
fn update_score(event: On<LinesCleared>, mut state: ResMut<GameState>) {
    let lines_cleared = event.0;
    assert!(lines_cleared <= 4);

    let multiplier = match lines_cleared {
        0 => 0,
        1 => 40,
        2 => 100,
        3 => 300,
        4 => 1200,
        _ => unreachable!("line clear count must be between 0 and 4"),
    };

    state.score += multiplier * (state.level + 1);
    state.lines_cleared += lines_cleared;
    state.lines_cleared_since_last_level += lines_cleared;

    while state.lines_cleared_since_last_level >= (state.level + 1) * 10 {
        state.lines_cleared_since_last_level -= (state.level + 1) * 10;
        state.level += 1;
        let interval = state.drop_interval();
        state.gravity_timer.set_duration(interval);
    }
}

/// Update the score text.
fn update_score_text(state: Res<GameState>, mut score_text: Single<&mut Text, With<ScoreMarker>>) {
    if !state.is_changed() {
        return;
    }

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
