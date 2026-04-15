//! Hard drop implementation
use bevy::prelude::*;

use crate::ui::*;
use crate::{Game, data::*};

/// Tracker for the hard drop status
#[derive(Component, Default)]
pub struct HardDrop(pub bool);

/// Create the hard-drop status text in the UI.
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

/// Flip the hard-drop flag when the player presses Z.
fn toggle_hard_drop(keyboard: Res<ButtonInput<KeyCode>>, mut hard_drop: Single<&mut HardDrop>) {
    // The replay recordings show that hard drop flips on key press, not key
    // release.
    // Example:
    // pressing Z once changes `false` -> `true`.
    if keyboard.just_pressed(KeyCode::KeyZ) {
        hard_drop.0 = !hard_drop.0;
        crate::board::trace_event(format!(
            "toggle_hard_drop: changed hard_drop to {}",
            hard_drop.0
        ));
    }
}

/// Update the manual drop amount whenever the hard-drop flag changes.
fn update_drop_gravity(
    hard_drop: Query<&HardDrop, Changed<HardDrop>>,
    mut state: ResMut<GameState>,
) {
    // Only do work when the flag actually changed.
    for hard_drop in &hard_drop {
        // Hard drop means the down input should behave like "drop one row"
        // many times in the same frame.
        // Example:
        // Off -> manual gravity 1
        // On  -> manual gravity 20
        state.manual_drop_gravity = if hard_drop.0 {
            HARD_DROP_GRAVITY
        } else {
            SOFT_DROP_GRAVITY
        };
        crate::board::trace_event(format!(
            "update_drop_gravity: hard_drop={} manual_drop_gravity={}",
            hard_drop.0, state.manual_drop_gravity
        ));
    }
}

/// Rewrite the status text whenever the hard-drop flag changes.
fn update_status_text(mut text: Query<(&HardDrop, &mut Text), Changed<HardDrop>>) {
    // Again, `Changed<HardDrop>` keeps this system cheap.
    for (hard_drop, mut text) in &mut text {
        // Show a short human-readable status string in the UI.
        // Example:
        // if hard drop is enabled, the text becomes "Hard Drop: On".
        text.0 = if hard_drop.0 {
            "Hard Drop: On".to_string()
        } else {
            "Hard Drop: Off".to_string()
        };
    }
}

/// Plugin that adds hard-drop toggle behavior and the status text.
pub struct HardDropPlugin;

impl Plugin for HardDropPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, setup_status_text.in_set(Game))
            .add_systems(
                Update,
                (toggle_hard_drop, update_drop_gravity, update_status_text)
                    .chain()
                    .before(crate::board::handle_user_input)
                    .in_set(Game),
            );
    }
}
