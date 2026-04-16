//! Replay a recorded gameplay, comparing against expected states

#[cfg(feature = "hard_drop")]
use crate::hard_drop::HardDrop;
#[cfg(feature = "hold")]
use crate::hold::Hold;
use crate::{
    data::{Active, GameState, Next, Obstacle},
    ui::TitleText,
};
use std::time::Duration;

use super::*;
use bevy::{color::palettes::tailwind, prelude::*, time::TimeUpdateStrategy};

/// Indicate a test failure
#[derive(Event)]
pub struct TestFail;

/// Indicate a test passing
#[derive(Event)]
pub struct TestPass;

// Counter for test statistics
#[derive(Resource, Default)]
struct TestStatistics {
    // Number of mismatched states
    mismatches: usize,
}

// Whether to ignore the score component of a snapshot.
#[derive(Resource)]
struct IgnoreScore;

// Number of mismatched states allowed for a test to pass
const MAX_STATE_MISMATCHES: usize = 1;

// Maximum number of states to look ahead for a desync
const MAX_STATE_LOOKAHEAD: usize = 10;

/// Tracks whether replay injected an input edge on the current fixed tick.
#[derive(Resource, Default)]
struct ReplayEventThisTick(Option<Duration>);

fn replay_input(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut recording: ResMut<GameRecording>,
    stats: Res<TestStatistics>,
    time: Res<Time<Fixed>>,
    mut replay_event_this_tick: ResMut<ReplayEventThisTick>,
    mut commands: Commands,
) {
    replay_event_this_tick.0 = None;

    if let Some(event) = recording
        .events
        .pop_front_if(|event| event.time <= time.elapsed())
    {
        replay_event_this_tick.0 = Some(time.elapsed());
        for key in &event.just_released {
            keyboard.release(*key);
        }
        for key in &event.just_pressed {
            keyboard.press(*key);
        }
    }

    if recording.events.is_empty() {
        if stats.mismatches <= MAX_STATE_MISMATCHES {
            commands.trigger(TestPass)
        } else {
            commands.trigger(TestFail)
        }
    }
}

#[allow(clippy::too_many_arguments)]
fn compare_states(
    active: Option<Single<&Tetromino, With<Active>>>,
    next: Option<Single<&Tetromino, With<Next>>>,
    #[cfg(feature = "hold")] hold: Option<Single<&Tetromino, With<Hold>>>,
    obstacles: Query<&Block, With<Obstacle>>,
    #[cfg(feature = "hard_drop")] hard_drop: Single<&HardDrop>,
    state: Res<GameState>,
    time: Res<Time<Fixed>>,
    mut recording: ResMut<GameRecording>,
    replay_event_this_tick: Res<ReplayEventThisTick>,
    mut stats: ResMut<TestStatistics>,
    ignore_score: Option<Res<IgnoreScore>>,
    mut commands: Commands,
) {
    let t_actual = time.elapsed();

    if recording.snapshots.is_empty() {
        return;
    }

    let actual = record_game_state(
        active,
        next,
        #[cfg(feature = "hold")]
        hold,
        obstacles,
        #[cfg(feature = "hard_drop")]
        hard_drop,
        state,
    );
    let actual = if ignore_score.is_some() {
        Snapshot {
            score: 0,
            lines_cleared: 0,
            level: 0,
            ..actual
        }
    } else {
        actual
    };

    // do a linear scan until we match a state
    let Some((skipped, _)) = recording
        .snapshots
        .iter()
        .enumerate()
        .take(MAX_STATE_LOOKAHEAD)
        .find(|(_, (_, expected))| actual == *expected)
    else {
        if replay_event_this_tick.0 == Some(t_actual) {
            return;
        }
        stats.mismatches += 1;
        if stats.mismatches > MAX_STATE_MISMATCHES {
            commands.trigger(TestFail);
        }
        return;
    };

    // drop all the states we skipped.
    //
    // using split_off because truncate_front is not stable yet.
    recording.snapshots = recording.snapshots.split_off(skipped);
}

fn observe_test_fail(
    _: On<TestFail>,
    mut title: Single<(&mut Text, &mut TextColor), With<TitleText>>,
    mut time: ResMut<Time<Virtual>>,
) {
    title.0.0 = "Test Fails!".to_string();
    title.1.0 = Color::from(tailwind::RED_400);
    time.pause();
}

fn observe_test_pass(
    _: On<TestPass>,
    mut title: Single<(&mut Text, &mut TextColor), With<TitleText>>,
    mut time: ResMut<Time<Virtual>>,
) {
    title.0.0 = "Test Passes!".to_string();
    title.1.0 = Color::from(tailwind::GREEN_400);
    time.pause();
}

// Set all score-related info to zero if we are ignoring scores.
fn adjust_scores(ignore_score: Option<Res<IgnoreScore>>, mut recording: ResMut<GameRecording>) {
    if ignore_score.is_some() {
        for (_, snapshot) in &mut recording.snapshots {
            snapshot.score = 0;
            snapshot.level = 0;
            snapshot.lines_cleared = 0;
        }
    }
}

/// A plugin for replaying a game recording.  The game recording must be given
/// as a `GameRecording` resource.
#[derive(Default)]
pub struct TestReplayPlugin {
    /// Whether this test should ignore or check scores
    pub check_scores: bool,
}

impl Plugin for TestReplayPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(FixedPreUpdate, replay_input)
            .add_systems(FixedPostUpdate, compare_states)
            .add_systems(Startup, adjust_scores)
            .add_observer(observe_test_fail)
            .add_observer(observe_test_pass)
            .insert_resource(ReplayEventThisTick::default())
            .insert_resource(TestStatistics::default())
            .insert_resource(TimeUpdateStrategy::ManualDuration(FIXED_FRAME_DURATION));

        if !self.check_scores {
            app.insert_resource(IgnoreScore);
        }
    }
}
