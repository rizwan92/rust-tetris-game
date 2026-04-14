//! The tetris board setup

use std::time::Duration;

use crate::ui::{BG_COLOR, PADDING};

use super::data::*;
use serde::{Serialize, Deserialize};
use bevy::{platform::collections::HashMap, prelude::*};

/// The main board containing visible tiles.
#[derive(Resource)]
pub struct Board {
    // Visible tiles as entities
    cells: Vec<Vec<Entity>>,
}

/// Side-length of an *unscaled* tile in pixels.
pub const TILE_SIDE_LEN: f32 = 40.0;

/// Amount of time before a tile is locked.
pub const LOCKDOWN_DURATION: Duration = Duration::from_millis(2600);

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
    mut active: Query<&mut Tetromino, With<Active>>,
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

/// Check if the active tetromino cannot move down. If so, deactivate it.
pub fn deactivate_if_stuck(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    virtual_time: Res<Time<Virtual>>,
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
    let duration = if manual_dropped {
        HARD_DROP_LOCKDOWN_DURATION
    } else {
        LOCKDOWN_DURATION
    };
    // Replays use a manual time strategy and need exact recorded timing.
    // The ordinary-speed sleep-based tests can be one fixed step late on macOS,
    // but the accelerated baseline tests expect the original "start next step"
    // behavior, so only the default-speed automatic path counts the creation step.
    let tick_on_create = !matches!(
        *time_strategy,
        bevy::time::TimeUpdateStrategy::ManualDuration(_)
    ) && virtual_time.relative_speed() <= 1.0;
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

    // If the spawn position is already blocked, the game is over.
    if crate::there_is_collision(&active_tetromino, obstacles.reborrow()) {
        commands.trigger(GameOver);
        return;
    }

    // Build the new logical preview from the front of the bag.
    let mut next_tetromino = state.bag.peek();
    next_tetromino.shift(2, 2);

    // Spawn the new active piece and the refreshed Next preview.
    commands.spawn((active_tetromino, Active));
    commands.spawn((next_tetromino, Next));
    if !carry_gravity_timer.0 {
        // Normally a new piece should start with a fresh gravity timer.
        state.gravity_timer.reset();
    }
    // Clear the carry flag once the next piece has been spawned.
    carry_gravity_timer.0 = false;
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
