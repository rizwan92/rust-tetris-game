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

/// Whether the current fixed step already spent its automatic gravity probe on
/// a blocked downward move.
#[derive(Resource, Default)]
pub struct BlockedAutoDrop(pub bool);

#[allow(dead_code)] // remove after your implementation
impl LockdownTimer {
    // Advance the timer. Start it if it hasn't been started.
    fn start_or_advance(&mut self, time: Res<Time<Fixed>>) {
        if self.0.is_none() {
            self.0 = Some(Timer::new(LOCKDOWN_DURATION, TimerMode::Once));
        }

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
    commands.insert_resource(BlockedAutoDrop::default());
}

/// Handle user input for the purposes of moving and/or rotating the tetromino.
pub fn handle_user_input(
    mut keyboard: ResMut<ButtonInput<KeyCode>>,
    state: Res<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    mut tetrominoes: Query<&mut Tetromino, With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    let Ok(mut tetromino) = tetrominoes.single_mut() else {
        return;
    };

    if keyboard.just_pressed(KeyCode::ArrowDown) {
        for _ in 0..state.manual_drop_gravity {
            let mut candidate = *tetromino;
            candidate.shift(0, -1);
            if crate::there_is_collision(&candidate, obstacles.reborrow()) {
                break;
            }
            *tetromino = candidate;
            let mut below = *tetromino;
            below.shift(0, -1);
            if !crate::there_is_collision(&below, obstacles.reborrow())
                || state.manual_drop_gravity == SOFT_DROP_GRAVITY
            {
                lockdown.reset();
            }
        }
        keyboard.clear_just_pressed(KeyCode::ArrowDown);
    }

    if keyboard.just_pressed(KeyCode::ArrowLeft) {
        let mut candidate = *tetromino;
        candidate.shift(-1, 0);
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            let mut below = *tetromino;
            below.shift(0, -1);
            if !crate::there_is_collision(&below, obstacles.reborrow())
                || state.manual_drop_gravity == SOFT_DROP_GRAVITY
            {
                lockdown.reset();
            }
        }
        keyboard.clear_just_pressed(KeyCode::ArrowLeft);
    }

    if keyboard.just_pressed(KeyCode::ArrowRight) {
        let mut candidate = *tetromino;
        candidate.shift(1, 0);
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            let mut below = *tetromino;
            below.shift(0, -1);
            if !crate::there_is_collision(&below, obstacles.reborrow())
                || state.manual_drop_gravity == SOFT_DROP_GRAVITY
            {
                lockdown.reset();
            }
        }
        keyboard.clear_just_pressed(KeyCode::ArrowRight);
    }

    if keyboard.just_pressed(KeyCode::ArrowUp) || keyboard.just_pressed(KeyCode::Space) {
        let mut candidate = *tetromino;
        candidate.rotate();
        if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
            *tetromino = candidate;
            let mut below = *tetromino;
            below.shift(0, -1);
            if !crate::there_is_collision(&below, obstacles.reborrow())
                || state.manual_drop_gravity == SOFT_DROP_GRAVITY
            {
                lockdown.reset();
            }
        }
        keyboard.clear_just_pressed(KeyCode::ArrowUp);
        keyboard.clear_just_pressed(KeyCode::Space);
    }
}

/// Drop the piece whenever the gravity timer goes off
pub fn gravity(
    time: Res<Time<Fixed>>,
    mut state: ResMut<GameState>,
    mut blocked_auto_drop: ResMut<BlockedAutoDrop>,
    mut tetrominoes: Query<&mut Tetromino, With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    blocked_auto_drop.0 = false;
    state.gravity_timer.tick(time.delta());
    if !state.gravity_timer.just_finished() {
        return;
    }

    let Ok(mut tetromino) = tetrominoes.single_mut() else {
        return;
    };

    let mut candidate = *tetromino;
    candidate.shift(0, -1);
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        *tetromino = candidate;
    } else {
        blocked_auto_drop.0 = true;
    }
}

/// Check if the active tetromino cannot move down. If so, deactivate it.
pub fn deactivate_if_stuck(
    mut commands: Commands,
    time: Res<Time<Fixed>>,
    state: Res<GameState>,
    mut lockdown: ResMut<LockdownTimer>,
    blocked_auto_drop: Res<BlockedAutoDrop>,
    tetrominoes: Query<(Entity, &Tetromino), With<Active>>,
    mut obstacles: Query<&Block, With<Obstacle>>,
) {
    let waiting_before_lock = lockdown.0.is_some();
    let Ok((entity, tetromino)) = tetrominoes.single() else {
        lockdown.reset();
        return;
    };

    let mut candidate = *tetromino;
    candidate.shift(0, -1);
    if !crate::there_is_collision(&candidate, obstacles.reborrow()) {
        lockdown.reset();
        return;
    }

    if waiting_before_lock && blocked_auto_drop.0 && state.manual_drop_gravity == SOFT_DROP_GRAVITY
    {
        return;
    }

    lockdown.start_or_advance(time);
    if !lockdown.just_finished() {
        return;
    }

    commands.entity(entity).despawn();
    for &cell in tetromino.cells() {
        commands.spawn((
            Block {
                cell,
                color: tetromino.color,
            },
            Obstacle,
        ));
    }
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
    if !active_tetrominoes.is_empty() {
        return;
    }

    for entity in &next_tetrominoes {
        commands.entity(entity).despawn();
    }

    let mut active = state.bag.next_tetromino();
    if active.center == (0.5, -0.5) {
        active.shift(4, 19);
    } else {
        active.shift(4, 18);
    }

    if crate::there_is_collision(&active, obstacles) {
        commands.trigger(GameOver);
        return;
    }

    let soft_spawn_smoothing = crate::rr::FIXED_FRAME_DURATION.mul_f32(0.5);
    let reset_for_hard_drop = state.manual_drop_gravity > SOFT_DROP_GRAVITY
        && state.gravity_timer.remaining() <= crate::rr::FIXED_FRAME_DURATION;
    let reset_for_soft_drop = state.manual_drop_gravity == SOFT_DROP_GRAVITY
        && state.gravity_timer.remaining() < soft_spawn_smoothing;
    if reset_for_hard_drop || reset_for_soft_drop {
        state.gravity_timer.reset();
    }

    keyboard.clear_just_pressed(KeyCode::ArrowDown);
    keyboard.clear_just_pressed(KeyCode::ArrowLeft);
    keyboard.clear_just_pressed(KeyCode::ArrowRight);
    keyboard.clear_just_pressed(KeyCode::ArrowUp);
    keyboard.clear_just_pressed(KeyCode::Space);
    commands.spawn((active, Active));

    let mut next = state.bag.peek();
    next.shift(2, 2);
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
    let mut colors = HashMap::<Entity, Color>::new();

    for tetromino in &tetrominoes {
        for &Cell(x, y) in tetromino.cells().iter().filter(|cell| cell.is_visible()) {
            colors.insert(board.cells[y as usize][x as usize], tetromino.color);
        }
    }

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
    for (block, entity) in &side_board {
        let color = tetromino
            .as_ref()
            .filter(|tetromino| tetromino.cells().contains(&block.cell))
            .map(|tetromino| tetromino.color)
            .unwrap_or(BG_COLOR);
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
