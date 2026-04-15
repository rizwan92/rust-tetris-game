//! The tetris board setup

use std::time::Duration;

use crate::ui::{BG_COLOR, PADDING};

use super::data::*;
use bevy::{platform::collections::HashMap, prelude::*};
use serde::{Deserialize, Serialize};

/// The main board containing visible tiles.
#[derive(Resource)]
pub struct Board {
    // Visible tiles as entities
    cells: Vec<Vec<Entity>>,
}

/// Side-length of an *unscaled* tile in pixels.
pub const TILE_SIDE_LEN: f32 = 40.0;

/// Amount of time before a tile is locked.
pub const LOCKDOWN_DURATION: Duration = GameState::initial_drop_interval();

/// An event signalling that the game is over.
#[derive(Event)]
pub struct GameOver;

/// An block.  This is one of:
/// - an obstacle (leftovers of an inactive tetromino)
/// - a block used in the preview and held views.
#[derive(Component, Copy, Clone, PartialEq, Debug, Serialize, Deserialize)]
pub struct Block {
    /// Coordinates of this block
    pub cell: Cell,
    /// Color of this block
    pub color: Color,
}

/// A timer to count down when a piece must be inactivated after it can't be pushed down
#[derive(Resource)]
#[allow(dead_code)] // remove after your implementation
pub struct LockdownTimer(Option<Timer>);

/// Whether lifecycle tracing is enabled for test and CI runs.
pub(crate) fn trace_enabled() -> bool {
    cfg!(any(feature = "ci", feature = "test")) && std::env::var_os("BLOX_TRACE").is_some()
}

/// Print one structured lifecycle trace line when tracing is enabled.
pub(crate) fn trace_event(event: String) {
    if trace_enabled() {
        eprintln!("[blox-trace] {event}");
    }
}

/// Return a short string describing the lock timer.
pub(crate) fn lockdown_snapshot(lockdown: &LockdownTimer) -> String {
    match &lockdown.0 {
        Some(timer) => format!(
            "lockdown=some(elapsed={:.3}s duration={:.3}s finished={} just_finished={})",
            timer.elapsed_secs(),
            timer.duration().as_secs_f32(),
            timer.is_finished(),
            timer.just_finished()
        ),
        None => "lockdown=none".to_string(),
    }
}

/// Return a short string describing the gravity timer.
pub(crate) fn gravity_snapshot(state: &GameState) -> String {
    format!(
        "gravity(elapsed={:.3}s duration={:.3}s finished={} just_finished={} manual_drop={} level={})",
        state.gravity_timer.elapsed_secs(),
        state.gravity_timer.duration().as_secs_f32(),
        state.gravity_timer.is_finished(),
        state.gravity_timer.just_finished(),
        state.manual_drop_gravity,
        state.level
    )
}

#[allow(dead_code)] // remove after your implementation
impl LockdownTimer {
    // Advance the timer. Start it if it hasn't been started.
    fn start_or_advance(&mut self, time: Res<Time<Fixed>>) {
        // If this is the first frame where the piece is stuck,
        // create the one-shot timer now.
        // Example:
        // a J piece touches the floor for the first time, so lockdown begins.
        if self.0.is_none() {
            self.0 = Some(Timer::new(LOCKDOWN_DURATION, TimerMode::Once));
        }

        // Once the timer exists, move it forward by one fixed-step delta.
        // This is what eventually makes the piece lock into obstacles.
        if let Some(timer) = &mut self.0 {
            timer.tick(time.delta());
        }
    }

    // Has this timer just gone off?
    fn just_finished(&self) -> bool {
        self.0.as_ref().is_some_and(Timer::just_finished)
    }

    pub(crate) fn reset(&mut self) {
        self.0 = None;
    }
}

// Create a logical tile to insert into a board.
//
// Board width and board height are the information of the board this tile is
// placed in.
fn mk_tile(
    x: u32,
    y: u32,
    board_width: u32,
    board_height: u32,
    tile_mesh: Handle<Mesh>,
    tile_material: Handle<ColorMaterial>,
) -> impl Bundle {
    (
        Mesh2d(tile_mesh),
        MeshMaterial2d(tile_material),
        (
            Text2d(format!("{x},{y}")),
            TextFont {
                font_size: 12.0,
                ..Default::default()
            },
        ),
        Transform::from_xyz(
            (x as f32 - board_width as f32 / 2.0) * TILE_SIDE_LEN + PADDING / 2.0,
            (y as f32 - board_height as f32 / 2.0) * TILE_SIDE_LEN + PADDING / 2.0,
            0.0,
        )
        .with_scale(Vec3::splat(0.8)),
    )
}

/// The calculated window height based on the board size.
pub const fn window_height() -> f32 {
    TILE_SIDE_LEN * BOARD_HEIGHT as f32 + PADDING * 2.0 + crate::ui::TEXT_SIZE * 2.0
}

/// The calculated window width based on the board size.
pub const fn window_width() -> f32 {
    window_height()
}

/// Set up a side window to show the next piece or the hold area.
pub fn spawn_side_window(
    transform: Transform,
    mesh: Handle<Mesh>,
    material: Handle<ColorMaterial>,
    commands: &mut Commands,
    title: &str,
    marker: impl Component + Copy,
) {
    commands
        .spawn((transform, Visibility::default()))
        .with_children(|parent| {
            (0..5).for_each(|y| {
                (0..5).for_each(|x| {
                    parent.spawn((
                        mk_tile(x, y, 5, 5, mesh.clone(), material.clone()),
                        Block {
                            cell: Cell(x as i32, y as i32),
                            color: BG_COLOR,
                        },
                        marker,
                    ));
                })
            });

            parent.spawn((
                Transform::from_xyz(-4. * TILE_SIDE_LEN * 0.5, 5. * TILE_SIDE_LEN * 0.5, -1.0),
                Text2d::new(title),
            ));
        });
}

/// Set up the window. Only used when not testing.
#[cfg(all(not(feature = "ci"), not(feature = "test")))]
pub fn setup_window(mut window: Single<&mut Window>) {
    window.resolution.set(window_height(), window_height());
}

/// Create the board and initialize game data
pub fn setup_board(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<ColorMaterial>>,
) {
    // Background color
    commands.insert_resource(ClearColor(Color::BLACK));

    // Set up the camera
    commands.spawn(Camera2d);

    let mesh = meshes.add(Rectangle::new(TILE_SIDE_LEN, TILE_SIDE_LEN));
    let material = materials.add(BG_COLOR);

    // Set up the board
    let cells = (0..BOARD_HEIGHT)
        .map(|y| {
            (0..BOARD_WIDTH)
                .map(|x| {
                    commands
                        .spawn(mk_tile(
                            x,
                            y,
                            BOARD_WIDTH,
                            BOARD_HEIGHT,
                            mesh.clone(),
                            material.clone(),
                        ))
                        .id()
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    commands.insert_resource(Board { cells });

    spawn_side_window(
        Transform::from_xyz(
            (BOARD_WIDTH + 5) as f32 * TILE_SIDE_LEN * 0.5 + PADDING,
            window_height() * 0.5 - 5.0 * TILE_SIDE_LEN * 0.5 - PADDING,
            0.0,
        ),
        mesh.clone(),
        material.clone(),
        &mut commands,
        "Next",
        Next,
    );

    commands.add_observer(exit_on_game_over);
    commands.insert_resource(LockdownTimer(None));
}

/// Handle user input for the purposes of moving and/or rotating the tetromino.
pub fn handle_user_input(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    mut tetrominoes: Query<&mut Tetromino, With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    let down = keyboard.just_pressed(KeyCode::ArrowDown);
    let left = keyboard.just_pressed(KeyCode::ArrowLeft);
    let right = keyboard.just_pressed(KeyCode::ArrowRight);
    let up = keyboard.just_pressed(KeyCode::ArrowUp);
    let space = keyboard.just_pressed(KeyCode::Space);
    let saw_input = down || left || right || up || space;

    // If there is no active tetromino yet, there is nothing to move.
    let Ok(mut tetromino) = tetrominoes.single_mut() else {
        if saw_input {
            trace_event(format!(
                "handle_user_input: saw input down={down} left={left} right={right} up={up} space={space} but no active piece existed"
            ));
        }
        return;
    };

    if saw_input {
        trace_event(format!(
            "handle_user_input: begin down={down} left={left} right={right} up={up} space={space} active_before={:?} {} {}",
            *tetromino,
            gravity_snapshot(&state),
            lockdown_snapshot(&lockdown)
        ));
    }

    // Down happens first by the baseline spec.
    // Example:
    // if the player presses down and right together, we must drop first.
    if keyboard.just_pressed(KeyCode::ArrowDown) {
        // The game state decides how strong a manual drop is.
        // In the baseline this is usually 1, but hard drop will reuse this later.
        for _ in 0..state.manual_drop_gravity {
            // Create a candidate piece one row lower.
            let mut candidate = *tetromino;
            candidate.shift(0, -1);
            // If moving down would be illegal, stop the manual drop loop.
            if crate::there_is_collision(&candidate, obstacles.reborrow()) {
                break;
            }
            // Otherwise accept the lower position.
            *tetromino = candidate;
            // A successful move means the piece is no longer "waiting to lock"
            // at the previous position, so reset the lockdown timer.
            lockdown.reset();
            trace_event(format!(
                "handle_user_input: soft-drop moved active to {:?} and reset lockdown",
                *tetromino
            ));
        }
        keyboard.clear_just_pressed(KeyCode::ArrowDown);
    }

    // Left happens after down.
    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        // Try the move on a copy first.
        let mut candidate = *tetromino;
        candidate.shift(-1, 0);
        // Only commit the move when it stays legal.
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            lockdown.reset();
            trace_event(format!(
                "handle_user_input: moved left to {:?} and reset lockdown",
                *tetromino
            ));
        }
        keyboard.clear_just_pressed(KeyCode::ArrowLeft);
    }

    // Right happens after left.
    if keyboard.just_pressed(KeyCode::ArrowRight) {
        let mut candidate = *tetromino;
        candidate.shift(1, 0);
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            lockdown.reset();
            trace_event(format!(
                "handle_user_input: moved right to {:?} and reset lockdown",
                *tetromino
            ));
        }
        keyboard.clear_just_pressed(KeyCode::ArrowRight);
    }

    // Up or Space means rotate.
    // Using `||` here matches the spec: pressing both should still rotate once.
    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::Space) {
        // Rotate a copy first so illegal rotations can be rejected safely.
        let mut candidate = *tetromino;
        candidate.rotate();
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            lockdown.reset();
            trace_event(format!(
                "handle_user_input: rotated to {:?} and reset lockdown",
                *tetromino
            ));
        }
        keyboard.clear_just_pressed(KeyCode::ArrowUp);
        keyboard.clear_just_pressed(KeyCode::Space);
    }

    if saw_input {
        trace_event(format!(
            "handle_user_input: end active_after={:?} {}",
            *tetromino,
            lockdown_snapshot(&lockdown)
        ));
    }
}

/// Drop the piece whenever the gravity timer goes off
pub fn gravity(
    time: Res<Time<Fixed>>,
    mut state: ResMut<GameState>,
    mut tetrominoes: Query<&mut Tetromino, With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    let obstacle_count = obstacles.iter().count();
    let active_before = tetrominoes
        .single()
        .ok()
        .map(|tetromino| format!("{:?}", *tetromino))
        .unwrap_or_else(|| "none".to_string());
    trace_event(format!(
        "gravity: before tick delta={:.3}s active={} obstacles={} {}",
        time.delta_secs(),
        active_before,
        obstacle_count,
        gravity_snapshot(&state)
    ));

    // Advance the repeating gravity timer every fixed frame.
    state.gravity_timer.tick(time.delta());
    // If the timer has not fired yet, do nothing this frame.
    if !state.gravity_timer.just_finished() {
        trace_event(format!(
            "gravity: timer not ready after tick remaining={:.3}s {}",
            state.gravity_timer.remaining().as_secs_f32(),
            gravity_snapshot(&state)
        ));
        return;
    }

    // No active piece means there is nothing to drop.
    let Ok(mut tetromino) = tetrominoes.single_mut() else {
        trace_event(format!(
            "gravity: timer fired but there was no active piece {}",
            gravity_snapshot(&state)
        ));
        return;
    };

    // Try moving the active piece down by one row.
    let mut candidate = *tetromino;
    candidate.shift(0, -1);
    // Only accept the move when there is no collision.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        trace_event(format!(
            "gravity: timer fired and moved active from {:?} to {:?}; remaining={:.3}s {}",
            *tetromino,
            candidate,
            state.gravity_timer.remaining().as_secs_f32(),
            gravity_snapshot(&state)
        ));
        *tetromino = candidate;
    } else {
        // Ordinary mode wants a fresh interval after a blocked automatic drop.
        // Hard-drop recordings, however, expect that carry to survive so the
        // next spawned piece can still inherit it.
        if state.manual_drop_gravity == SOFT_DROP_GRAVITY {
            state.gravity_timer.reset();
            trace_event(format!(
                "gravity: blocked from moving {:?} to {:?}; reset gravity timer after blocked ordinary auto-drop, obstacles={} {}",
                *tetromino,
                candidate,
                obstacle_count,
                gravity_snapshot(&state)
            ));
        } else {
            trace_event(format!(
                "gravity: blocked from moving {:?} to {:?}; preserved gravity timer in hard-drop mode, obstacles={} {}",
                *tetromino,
                candidate,
                obstacle_count,
                gravity_snapshot(&state)
            ));
        }
    }
}

/// Check if the active tetromino cannot move down. If so, deactivate it.
pub fn deactivate_if_stuck(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    mut lockdown: ResMut<LockdownTimer>,
    tetrominoes: Query<(Entity, &Tetromino), With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    let obstacle_count = obstacles.iter().count();
    // If there is no active tetromino, make sure the lockdown timer is clear.
    let Ok((entity, tetromino)) = tetrominoes.single() else {
        trace_event(format!(
            "deactivate_if_stuck: no active piece, resetting obstacles={} {}",
            obstacle_count,
            lockdown_snapshot(&lockdown),
        ));
        lockdown.reset();
        return;
    };

    // Check whether the piece could move down one more row.
    let mut candidate = *tetromino;
    candidate.shift(0, -1);
    // If downward movement is still possible, the piece is not stuck yet.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        trace_event(format!(
            "deactivate_if_stuck: active {:?} can still move to {:?}, resetting obstacles={} {}",
            tetromino,
            candidate,
            obstacle_count,
            lockdown_snapshot(&lockdown)
        ));
        lockdown.reset();
        return;
    }

    // The piece is resting on the floor or on something else,
    // so advance the lock countdown.
    trace_event(format!(
        "deactivate_if_stuck: active {:?} is stuck above {:?}, advancing with delta={:.3}s obstacles={} {}",
        tetromino,
        candidate,
        time.delta_secs(),
        obstacle_count,
        lockdown_snapshot(&lockdown)
    ));
    lockdown.start_or_advance(time);
    trace_event(format!(
        "deactivate_if_stuck: after advance obstacles={} {}",
        obstacle_count,
        lockdown_snapshot(&lockdown),
    ));
    // If the timer has not finished yet, keep waiting.
    if !lockdown.just_finished() {
        return;
    }

    trace_event(format!(
        "deactivate_if_stuck: locking entity {:?} as obstacles {:?}; obstacle_count_before={}",
        entity, tetromino, obstacle_count
    ));
    // The piece is officially locked now, so remove the active tetromino entity.
    commands.entity(entity).despawn();
    // Replace that tetromino with four obstacle blocks.
    // Example:
    // if a J locks at the bottom-left, we spawn four `Block + Obstacle` entities
    // at exactly those four cells.
    for &cell in tetromino.cells() {
        commands.spawn((
            Block {
                cell,
                color: tetromino.color,
            },
            Obstacle,
        ));
    }
    // Clear the timer so the next spawned piece starts fresh.
    lockdown.reset();
}

/// Spawn the next tetromino if there is no active tetromino.  This should also
/// update the next tetromino window.
pub fn spawn_next_tetromino(
    mut commands: Commands,
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    mut state: ResMut<GameState>,
    active_tetrominoes: Query<Entity, With<Active>>,
    next_tetrominoes: Query<Entity, With<Next>>,
    obstacles: Query<&Block, With<Obstacle>>,
) {
    // Only spawn when there is no active tetromino on the board.
    if !active_tetrominoes.is_empty() {
        return;
    }

    trace_event(format!(
        "spawn_next_tetromino: spawning new active piece from bag with next_preview={:?} {}",
        state.bag.peek(),
        gravity_snapshot(&state)
    ));

    // Remove the old preview piece before creating the new one.
    for entity in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    // Take the next real piece out of the bag.
    let mut active = state.bag.next_tetromino();
    // Most pieces spawn with their logical center shifted by (4, 18).
    // Example:
    // J, L, S, Z, and T all use this branch.
    if active.center == (0.5, -0.5) {
        // The I piece is one row higher because its center is different.
        active.shift(4, 19);
    } else {
        active.shift(4, 18);
    }
    trace_event(format!(
        "spawn_next_tetromino: positioned fresh active {:?} with remaining={:.3}s elapsed={:.3}s just_finished={}",
        active,
        state.gravity_timer.remaining().as_secs_f32(),
        state.gravity_timer.elapsed_secs(),
        state.gravity_timer.just_finished()
    ));

    // Collision-enabled gameplay must also check whether the new spawn position
    // is already occupied.
    // Example:
    // if the stack has reached the top and the new piece overlaps it
    // immediately, the game is over and we should not spawn an illegal piece.
    if crate::there_is_collision(&active, obstacles) {
        trace_event(format!(
            "spawn_next_tetromino: spawn collision for {:?}, triggering game over",
            active
        ));
        commands.trigger(GameOver);
        return;
    }

    // Preserve most of the repeating gravity carry across piece transitions.
    //
    // There is one special case to normalize first:
    // if the previous piece locked right after gravity wrapped this frame,
    // Bevy's repeating timer now looks like it has almost no carry
    // (`elapsed ~= 0`). The replay data instead expects that fresh spawn to
    // behave like gravity is almost ready to fire again, so convert that
    // wrapped carry back into a near-finished timer.
    if state.gravity_timer.just_finished() {
        let duration = state.gravity_timer.duration();
        state.gravity_timer.reset();
        state
            .gravity_timer
            .set_elapsed(duration - crate::rr::FIXED_FRAME_DURATION);
        trace_event(format!(
            "spawn_next_tetromino: restored wrapped gravity carry for spawn {}",
            gravity_snapshot(&state)
        ));
    // In ordinary mode we reset a nearly-finished timer so a freshly spawned
    // piece does not drop almost immediately.
    //
    // Hard-drop replays depend on preserving that carry instead, so only apply
    // this smoothing when the game is still in its baseline soft-drop mode.
    } else if state.manual_drop_gravity == SOFT_DROP_GRAVITY
        && state.gravity_timer.remaining()
            <= crate::rr::FIXED_FRAME_DURATION + crate::rr::FIXED_FRAME_DURATION
    {
        state.gravity_timer.reset();
        trace_event(format!(
            "spawn_next_tetromino: reset gravity timer on near-finished spawn {}",
            gravity_snapshot(&state)
        ));
    }
    trace_event(format!(
        "spawn_next_tetromino: final carry before spawn remaining={:.3}s elapsed={:.3}s {}",
        state.gravity_timer.remaining().as_secs_f32(),
        state.gravity_timer.elapsed_secs(),
        gravity_snapshot(&state)
    ));

    // Spawn the active gameplay piece.
    //
    keyboard.clear_just_pressed(KeyCode::ArrowDown);
    keyboard.clear_just_pressed(KeyCode::ArrowLeft);
    keyboard.clear_just_pressed(KeyCode::ArrowRight);
    keyboard.clear_just_pressed(KeyCode::ArrowUp);
    keyboard.clear_just_pressed(KeyCode::Space);
    trace_event(format!(
        "spawn_next_tetromino: committed active={:?} next_preview_before_shift={:?} {}",
        active,
        state.bag.peek(),
        gravity_snapshot(&state)
    ));
    commands.spawn((active, Active));

    // Peek at the upcoming piece without consuming it.
    let mut next = state.bag.peek();
    // Shift the preview tetromino into the center of the 5x5 next window.
    next.shift(2, 2);
    // Spawn the logical preview piece.
    trace_event(format!(
        "spawn_next_tetromino: spawned next preview {:?}",
        next
    ));
    commands.spawn((next, Next));
}

/// Redraw the board.
pub fn redraw_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tetrominoes: Query<&Tetromino, With<Active>>,
    obstacles: Query<&Block, With<Obstacle>>,
    mut board: ResMut<Board>,
) {
    // This map stores the color each visible board tile should receive.
    // If a tile is missing from the map, it falls back to the black background.
    let mut colors = HashMap::<Entity, Color>::new();

    // First, paint the currently active tetromino.
    // Example:
    // if the active O sits at cells (4,18), (4,19), (5,18), (5,19),
    // those four board tile entities get the O color.
    for tetromino in &tetrominoes {
        for &Cell(x, y) in tetromino.cells().iter().filter(|cell| cell.is_visible()) {
            colors.insert(board.cells[y as usize][x as usize], tetromino.color);
        }
    }

    // Then, paint the obstacle blocks.
    // Obstacles overwrite the same map because they are part of the final board state.
    for block in obstacles.iter().filter(|block| block.cell.is_visible()) {
        let Cell(x, y) = block.cell;
        colors.insert(board.cells[y as usize][x as usize], block.color);
    }

    // re-draw the whole board
    for entity in board.cells.iter_mut().flat_map(|row| row.iter_mut()) {
        commands.entity(*entity).insert(MeshMaterial2d(
            materials.add(colors.get(entity).copied().unwrap_or(BG_COLOR)),
        ));
    }
}

/// Redraw the side board with the given marker component.
pub fn redraw_side_board<Marker: Component>(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    side_board: Query<(&mut Block, Entity), With<Marker>>,
    tetromino: Option<Single<&Tetromino, With<Marker>>>,
) {
    // Walk through every tile in the side board.
    for (block, entity) in &side_board {
        // If the preview/held tetromino contains this logical side-board cell,
        // use the tetromino color; otherwise use the background color.
        // Example:
        // for the next preview, only four cells out of the 5x5 window will be colored.
        let color = tetromino
            .as_ref()
            .filter(|tetromino| tetromino.cells().contains(&block.cell))
            .map(|tetromino| tetromino.color)
            .unwrap_or(BG_COLOR);
        // Replace the material on that tile so the side window redraws correctly.
        commands
            .entity(entity)
            .insert(MeshMaterial2d(materials.add(color)));
    }
}

/// Trigger game over when Escape is pressed
pub fn game_over_on_esc(keyboard: Res<ButtonInput<KeyCode>>, mut commands: Commands) {
    if keyboard.just_released(KeyCode::Escape) {
        commands.trigger(GameOver);
    }
}

/// Exit the program when the game over event is triggered
pub fn exit_on_game_over(_: On<GameOver>, mut exit: MessageWriter<AppExit>) {
    exit.write(AppExit::Success);
}
