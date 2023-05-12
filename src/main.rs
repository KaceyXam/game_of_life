use std::time::Duration;

use bevy::prelude::*;
use bevy::window::{PresentMode, WindowResolution};
use rand::Rng;

const GRID_WIDTH: usize = 30;
const GRID_HEIGHT: usize = 20;
const CELL_SIZE: f32 = 10.0;

fn main() {
    App::new()
        .insert_resource(ClearColor(Color::MIDNIGHT_BLUE))
        .init_resource::<GameData>()
        .insert_resource(SimulationTick {
            timer: Timer::new(Duration::from_millis(50), TimerMode::Repeating),
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: String::from("Cellular Automata"),
                resolution: WindowResolution::new(
                    (GRID_WIDTH as f32) * CELL_SIZE,
                    (GRID_HEIGHT as f32) * CELL_SIZE,
                ),
                present_mode: PresentMode::AutoVsync,
                ..Default::default()
            }),
            ..Default::default()
        }))
        .add_startup_system(setup_camera)
        .add_system(render_board)
        .add_system(execute_step)
        .add_system(pause_sim)
        .add_system(add_cells)
        .run();
}

#[derive(Clone, Copy)]
struct Cell {
    alive: bool,
}

#[derive(Resource)]
struct GameData {
    board: [[Cell; GRID_WIDTH]; GRID_HEIGHT],
}

impl Default for GameData {
    fn default() -> Self {
        let mut board = [[Cell { alive: false }; GRID_WIDTH]; GRID_HEIGHT];
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                if rand::thread_rng().gen_bool(0.999) {
                    board[y][x].alive = true;
                }
            }
        }
        GameData { board }
    }
}

#[derive(Component)]
struct CellComponent;

#[derive(Resource)]
struct SimulationTick {
    timer: Timer,
}

fn setup_camera(mut commands: Commands) {
    let board_width = GRID_WIDTH as f32 * CELL_SIZE;
    let board_height = GRID_HEIGHT as f32 * CELL_SIZE;

    commands.spawn(Camera2dBundle {
        transform: Transform {
            translation: Vec3::from([board_width / 2.0, board_height / 2.0, 500.0]),
            ..Default::default()
        },
        ..Default::default()
    });
}

fn pause_sim(keyboard: Res<Input<KeyCode>>, mut sim_tick: ResMut<SimulationTick>) {
    if keyboard.any_just_pressed([KeyCode::Space]) {
        if sim_tick.timer.paused() {
            sim_tick.timer.unpause();
        } else {
            sim_tick.timer.pause();
        }
    }
}

fn render_board(
    mut commands: Commands,
    game_data: Res<GameData>,
    cells: Query<Entity, With<CellComponent>>,
) {
    // if !sim_tick.timer.just_finished() {
    //     return;
    // }
    for cell in cells.iter() {
        commands.entity(cell).despawn();
    }
    for x in 0..GRID_WIDTH {
        for y in 0..GRID_HEIGHT {
            if game_data.board[y][x].alive {
                commands.spawn((
                    SpriteBundle {
                        sprite: Sprite {
                            custom_size: Some(Vec2::from([CELL_SIZE, CELL_SIZE])),
                            color: { Color::WHITE },
                            ..Default::default()
                        },
                        transform: Transform {
                            translation: Vec3::from([
                                x as f32 * CELL_SIZE,
                                y as f32 * CELL_SIZE,
                                0.0,
                            ]),
                            ..Default::default()
                        },
                        ..Default::default()
                    },
                    CellComponent,
                ));
            }
        }
    }
}

fn execute_step(
    mut game_data: ResMut<GameData>,
    time: Res<Time>,
    mut sim_tick: ResMut<SimulationTick>,
) {
    sim_tick.timer.tick(time.delta());
    let old_board = game_data.board.clone();
    if sim_tick.timer.just_finished() {
        for x in 0..GRID_WIDTH {
            for y in 0..GRID_HEIGHT {
                let surrounding = surrounding_count(&old_board, [x, y]);
                if !old_board[y][x].alive && surrounding == 3 {
                    game_data.board[y][x].alive = true;
                } else if old_board[y][x].alive && surrounding == 2 || surrounding == 3 {
                } else {
                    game_data.board[y][x].alive = false;
                }
            }
        }
    }
}

fn surrounding_count(board: &[[Cell; GRID_WIDTH]; GRID_HEIGHT], current: [usize; 2]) -> usize {
    let directions: Vec<[isize; 2]> = vec![
        [-1, -1],
        [0, -1],
        [1, -1],
        [-1, 0],
        [1, 0],
        [-1, 1],
        [0, 1],
        [1, 1],
    ];
    let mut count: usize = 0;
    for direction in directions.iter() {
        let new_x = if current[0] == 0 {
            GRID_WIDTH - 1
        } else if current[0] == GRID_WIDTH - 1 {
            0
        } else {
            (current[0] as isize + direction[0]) as usize
        };
        let new_y = if current[1] == 0 {
            GRID_HEIGHT - 1
        } else if current[1] == GRID_HEIGHT - 1 {
            0
        } else {
            (current[1] as isize + direction[1]) as usize
        };
        if board[new_y][new_x].alive {
            count += 1;
        }
    }
    count
}

fn add_cells(
    mut game_data: ResMut<GameData>,
    windows: Query<&Window>,
    mouse: Res<Input<MouseButton>>,
) {
    if mouse.pressed(MouseButton::Left) {
        let main_window = windows.get_single().unwrap();
        if let Some(position) = main_window.cursor_position() {
            let x = (position.x / CELL_SIZE) as usize;
            let y = (position.y / CELL_SIZE) as usize;
            game_data.board[y][x].alive = true;
        }
    }
}
