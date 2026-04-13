//! Hard drop implementation
use bevy::prelude::*;

use crate::{
    Game,
    data::{GameState, HARD_DROP_GRAVITY, SOFT_DROP_GRAVITY},
    ui::*,
};

/// Tracker for the hard drop status
#[derive(Component, Default)]
pub struct HardDrop(pub bool);

pub fn setup_status_text(mut commands: Commands) {
    commands.spawn(mk_text(
        "Hard Drop: Off",
        (
            Node {
                width: percent(40),
                height: percent(100),
                row_gap: px(10),
                ..default()
            },
            HardDrop(false),
            UiTransform::from_translation(Val2::percent(10.0, 50.0)),
        ),
        None,
    ));
}

// TODO: create systems for:
// - handling the user input (toggle hard drop when Z is pressed)
// - updating the gravity value based on HardDrop
// - updating the status text based on HardDrop
// for efficiency, use `Changed<HardDrop>` in your queries when it makes sense.

fn toggle_hard_drop(
    keyboard: Res<ButtonInput<KeyCode>>,
    mut hard_drop: Single<&mut HardDrop>,
) {
    if keyboard.just_pressed(KeyCode::KeyZ) {
        hard_drop.0 = !hard_drop.0;
    }
}

fn update_manual_drop_gravity(
    hard_drop: Single<&HardDrop, Changed<HardDrop>>,
    mut state: ResMut<GameState>,
) {
    state.manual_drop_gravity = if hard_drop.0 {
        HARD_DROP_GRAVITY
    } else {
        SOFT_DROP_GRAVITY
    };
}

fn update_status_text(
    mut status: Single<(&HardDrop, &mut Text), Changed<HardDrop>>,
) {
    let label = if status.0.0 { "On" } else { "Off" };
    status.1.0 = format!("Hard Drop: {label}");
}

pub struct HardDropPlugin;

impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_text.in_set(Game))
            .add_systems(
                Update,
                (
                    toggle_hard_drop,
                    update_manual_drop_gravity,
                    update_status_text,
                )
                    .chain()
                    .in_set(Game),
            );
    }
}
