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
pub const LOCKDOWN_DURATION: Duration = Duration::from_millis(2400);

/// Amount of time before a fast-dropped tile is locked.
pub const HARD_DROP_LOCKDOWN_DURATION: Duration = Duration::from_millis(800);

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

/// Whether the next spawned piece should inherit the current gravity timer.
#[derive(Resource, Default)]
pub struct CarryGravityTimer(pub bool);

/// Whether gameplay timing is driven by deterministic replay time or normal runtime time.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum TimingMode {
    /// Use the exact manual-timing behavior required by recorded replays.
    Replay,
    /// Use the ordinary runtime behavior used by live play and timing-sensitive tests.
    Realtime,
}

/// Why a tetromino is becoming the active piece.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum ActivationSource {
    /// The piece came from the bag as part of the normal spawn pipeline.
    BagSpawn,
    /// The piece came from the hold window and should follow the same activation rules.
    HoldSwap,
}

/// Which lock-delay policy should be used once the active piece is grounded.
#[derive(Copy, Clone, Eq, PartialEq, Debug)]
pub(crate) enum LockKind {
    /// Use the normal lock duration for pieces that landed without manual drop input.
    Normal,
    /// Use the shorter manual-drop lock duration for soft-drop and hard-drop landings.
    ManualDrop,
}

/// Convert Bevy's time strategy into the smaller lifecycle timing modes the board uses.
pub(crate) fn timing_mode(time_strategy: &bevy::time::TimeUpdateStrategy) -> TimingMode {
    if matches!(
        *time_strategy,
        bevy::time::TimeUpdateStrategy::ManualDuration(_)
    ) {
        TimingMode::Replay
    } else {
        TimingMode::Realtime
    }
}

// Detect the I tetromino by checking whether all four cells share one row or one column.
fn is_i_piece(tetromino: &Tetromino) -> bool {
    tetromino
        .cells
        .iter()
        .all(|cell| cell.0 == tetromino.cells[0].0)
        || tetromino
            .cells
            .iter()
            .all(|cell| cell.1 == tetromino.cells[0].1)
}

// Decide whether a newly active piece should skip ordinary gravity for one activation window.
fn should_delay_activation_gravity(
    tetromino: &Tetromino,
    source: ActivationSource,
    timing_mode: TimingMode,
) -> bool {
    // Replays are already deterministic, so they should keep the original spawn timing.
    if timing_mode == TimingMode::Replay {
        return false;
    }

    match source {
        // Ordinary bag spawns only need the narrow O-piece shield that stabilized the baseline tests.
        ActivationSource::BagSpawn => tetromino.is_o(),
        // Hold swaps need the same O-piece protection plus the I-piece timing protection.
        ActivationSource::HoldSwap => tetromino.is_o() || is_i_piece(tetromino),
    }
}

// Translate the current drop markers into the lock policy used by grounded pieces.
fn lock_kind(has_hard_drop: bool, has_manual_drop: bool) -> LockKind {
    if has_hard_drop || has_manual_drop {
        LockKind::ManualDrop
    } else {
        LockKind::Normal
    }
}

// Convert the chosen lock policy into the actual duration used by the lockdown timer.
fn lock_duration(lock_kind: LockKind) -> Duration {
    match lock_kind {
        LockKind::Normal => LOCKDOWN_DURATION,
        LockKind::ManualDrop => HARD_DROP_LOCKDOWN_DURATION,
    }
}

// Decide whether the lockdown timer should count the frame where it was created.
fn lockdown_ticks_on_create(timing_mode: TimingMode) -> bool {
    timing_mode == TimingMode::Realtime
}

/// Activate a tetromino using the shared lifecycle rules for bag spawns and hold swaps.
pub(crate) fn activate_tetromino(
    commands: &mut Commands,
    state: &mut GameState,
    tetromino: Tetromino,
    source: ActivationSource,
    timing_mode: TimingMode,
    carry_gravity_timer: Option<&mut CarryGravityTimer>,
    obstacles: &mut Query<&Block, With<Obstacle>>,
) -> bool {
    // Both bag spawns and hold swaps must reject illegal activation positions.
    if crate::there_is_collision(&tetromino, obstacles.reborrow()) {
        if source == ActivationSource::BagSpawn {
            // Normal spawns end the game when the spawn position is blocked.
            commands.trigger(GameOver);
        }
        // Hold swaps treat an illegal replacement as an aborted swap instead.
        return false;
    }

    // Spawn the new active piece and mark it as freshly activated when timing protection is needed.
    if should_delay_activation_gravity(&tetromino, source, timing_mode) {
        commands.spawn((tetromino, Active, JustSpawned));
    } else {
        commands.spawn((tetromino, Active));
    }

    // Bag spawns either reset gravity or preserve it according to the carry flag.
    // Hold swaps intentionally preserve the current gravity timer so they stay in the
    // same fixed-step cadence as the pre-existing validated behavior.
    let keep_gravity_timer = match source {
        ActivationSource::BagSpawn => carry_gravity_timer
            .as_deref()
            .is_some_and(|carry_gravity_timer| carry_gravity_timer.0),
        ActivationSource::HoldSwap => true,
    };
    if !keep_gravity_timer {
        state.gravity_timer.reset();
    }
    if let Some(carry_gravity_timer) = carry_gravity_timer {
        carry_gravity_timer.0 = false;
    }

    true
}

#[allow(dead_code)] // remove after your implementation
impl LockdownTimer {
    // Advance the timer. Start it if it hasn't been started.
    fn start_or_advance(&mut self, duration: Duration, time: &Time<Fixed>, tick_on_create: bool) {
        // Create the timer the first time we discover the piece is stuck.
        // We use the provided duration so soft lock and hard-drop lock can differ.
        if self.0.is_none() {
            self.0 = Some(Timer::new(duration, TimerMode::Once));
            if !tick_on_create {
                return;
            }
        }

        // Advance the timer once per fixed step while the piece is stuck.
        // In non-replay runs we can also count the creation frame to reduce timing flakiness.
        if let Some(timer) = &mut self.0 {
            timer.tick(time.delta());
        }
    }

    // Has this timer just gone off?
    fn just_finished(&self) -> bool {
        self.0.as_ref().is_some_and(Timer::just_finished)
    }

    fn reset(&mut self) {
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
    commands.insert_resource(CarryGravityTimer::default());
}

/// Handle user input for the purposes of moving and/or rotating the tetromino.
pub fn handle_user_input(
    mut commands: Commands,
    keyboard: Res<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut active: Query<(Entity, &mut Tetromino), With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    let Ok((entity, mut tetromino)) = active.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        let mut moved = false;

        for _ in 0..state.manual_drop_gravity {
            let mut candidate = *tetromino;
            candidate.shift(0, -1);
            if crate::there_is_collision(&candidate, obstacles.reborrow()) {
                break;
            }
            moved = true;
            *tetromino = candidate;
        }

        let landed = if moved {
            let mut candidate = *tetromino;
            candidate.shift(0, -1);
            crate::there_is_collision(&candidate, obstacles.reborrow())
        } else {
            false
        };

        if moved {
            // Once the player manually moves a piece downward, keep that information on the
            // piece so later lock timing can still treat it as a manual drop even if gravity
            // handles the final row before the piece becomes stuck.
            commands.entity(entity).insert(ManualDropped);
            if landed && state.manual_drop_gravity > SOFT_DROP_GRAVITY {
                // Hard drop is narrower: only the fast manual drop that actually reaches the
                // resting position should use the dedicated hard-drop marker.
                commands.entity(entity).insert(HardDropped);
            }
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        let mut candidate = *tetromino;
        candidate.shift(-1, 0);
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowRight) {
        let mut candidate = *tetromino;
        candidate.shift(1, 0);
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
        }
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::Space) {
        let mut candidate = *tetromino;
        candidate.rotate();
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
        }
    }
}

/// Drop the piece whenever the gravity timer goes off
pub fn gravity(
    time: Res<Time<Fixed>>,
    mut state: ResMut<GameState>,
    mut active: Query<&mut Tetromino, (With<Active>, Without<JustSpawned>)>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Advance the automatic gravity timer every fixed update.
    state.gravity_timer.tick(time.delta());
    // Exit until the timer says it is time to drop again.
    if !state.gravity_timer.just_finished() {
        return;
    }

    // Exit when there is no active piece to drop.
    let Ok(mut tetromino) = active.single_mut() else {
        return;
    };

    // Try moving the active piece down by exactly one row.
    let mut candidate = *tetromino;
    candidate.shift(0, -1);
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        // Commit the gravity move only if the destination is legal.
        *tetromino = candidate;
    }
}

/// Remove the one-frame spawn marker from active tetrominos.
pub fn clear_just_spawned(
    // Use commands so the temporary marker can be removed after the frame finishes.
    mut commands: Commands,
    // Read every active piece that still has the fresh-spawn marker.
    fresh: Query<(Entity, Ref<JustSpawned>)>,
) {
    // Keep the marker on the exact frame where it was added.
    // Remove it on the following frame so the piece skips one full extra update.
    for (entity, just_spawned) in &fresh {
        if just_spawned.is_added() {
            continue;
        }
        commands.entity(entity).remove::<JustSpawned>();
    }
}

/// Check if the active tetromino cannot move down. If so, deactivate it.
pub fn deactivate_if_stuck(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    time_strategy: Res<bevy::time::TimeUpdateStrategy>,
    mut lockdown: ResMut<LockdownTimer>,
    mut carry_gravity_timer: ResMut<CarryGravityTimer>,
    active: Query<(Entity, &Tetromino, Has<HardDropped>, Has<ManualDropped>), With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // When there is no active piece, clear the lockdown state and stop.
    let Ok((entity, tetromino, _hard_dropped, manual_dropped)) = active.single() else {
        lockdown.reset();
        return;
    };

    // Check whether the current active piece is blocked one row below.
    let tetromino = *tetromino;
    let mut candidate = tetromino;
    candidate.shift(0, -1);

    // If the piece can still move down, it is not stuck yet.
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        lockdown.reset();
        return;
    }

    // Manually dropped pieces lock faster than normally landed pieces.
    let timing_mode = timing_mode(&time_strategy);
    let duration = lock_duration(lock_kind(_hard_dropped, manual_dropped));
    // Replays keep exact recorded timing, while ordinary runs count the creation
    // step too so the macOS lock timing stays stable for the baseline tests.
    let tick_on_create = lockdown_ticks_on_create(timing_mode);
    // Start or advance the lockdown timer using that duration.
    lockdown.start_or_advance(duration, &time, tick_on_create);
    if !lockdown.just_finished() {
        return;
    }

    // Once the timer finishes, turn the tetromino into obstacle blocks.
    commands.entity(entity).despawn();
    for cell in tetromino.cells {
        commands.spawn((
            Block {
                cell,
                color: tetromino.color,
            },
            Obstacle,
        ));
    }
    // Carry the gravity timer only when the piece was manually dropped.
    carry_gravity_timer.0 = manual_dropped;
    lockdown.reset();
}

/// Spawn the next tetromino if there is no active tetromino.  This should also
/// update the next tetromino window.
pub fn spawn_next_tetromino(
    mut commands: Commands,
    mut state: ResMut<GameState>,
    time_strategy: Res<bevy::time::TimeUpdateStrategy>,
    mut lockdown: ResMut<LockdownTimer>,
    mut carry_gravity_timer: ResMut<CarryGravityTimer>,
    active: Query<(), (With<Active>, With<Tetromino>)>,
    next_tetrominoes: Query<Entity, (With<Next>, With<Tetromino>)>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    // Stop when an active piece already exists.
    if active.iter().next().is_some() {
        return;
    }

    // Remove the previous logical Next preview before rebuilding it.
    for entity in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    // Pull the next playable tetromino from the bag.
    let mut active_tetromino = state.bag.next_tetromino();
    // Shift it into the board spawn position.
    active_tetromino.shift(4, BOARD_HEIGHT as i32 - 1 - active_tetromino.bounds().top);

    // Build the new logical preview from the front of the bag.
    let mut next_tetromino = state.bag.peek();
    next_tetromino.shift(2, 2);

    // Activate the new bag piece through the shared lifecycle helper so bag spawns
    // and hold swaps follow the same activation contract.
    if !activate_tetromino(
        &mut commands,
        &mut state,
        active_tetromino,
        ActivationSource::BagSpawn,
        timing_mode(&time_strategy),
        Some(&mut carry_gravity_timer),
        &mut obstacles,
    ) {
        return;
    }

    // Spawn the refreshed Next preview after the active piece has been accepted.
    commands.spawn((next_tetromino, Next));
    lockdown.reset();
}

/// Redraw the board.
pub fn redraw_board(
    mut commands: Commands,
    mut materials: ResMut<Assets<ColorMaterial>>,
    tetrominoes: Query<&Tetromino, With<Active>>,
    obstacles: Query<&Block, With<Obstacle>>,
    mut board: ResMut<Board>,
) {
    // you just need to populate this map (see the loop at the end)
    // you'll also need to make it mutable
    let mut colors = HashMap::<Entity, Color>::new();

    for tetromino in &tetrominoes {
        for cell in tetromino.cells.iter().copied().filter(Cell::is_visible) {
            let entity = board.cells[cell.1 as usize][cell.0 as usize];
            colors.insert(entity, tetromino.color);
        }
    }

    for block in &obstacles {
        if block.cell.is_visible() {
            let entity = board.cells[block.cell.1 as usize][block.cell.0 as usize];
            colors.insert(entity, block.color);
        }
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
    mut side_board: Query<(&mut Block, Entity), With<Marker>>,
    tetromino: Option<Single<&Tetromino, With<Marker>>>,
) {
    for (mut block, entity) in &mut side_board {
        let color = if let Some(t) = tetromino.as_ref() {
            if t.cells.contains(&block.cell) {
                t.color
            } else {
                BG_COLOR
            }
        } else {
            BG_COLOR
        };

        block.color = color;
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
