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
fn update_score(event: On<LinesCleared>, mut _state: ResMut<GameState>) {
    let lines_cleared = event.0;
    assert!(lines_cleared <= 4);

    todo!()
}

/// Update the score text.
fn update_score_text() {
    todo!()
}

pub struct ScorePlugin;

impl Plugin for ScorePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_score_text.in_set(Game))
            .add_systems(Update, update_score_text.in_set(Game))
            .add_observer(update_score);
    }
}
