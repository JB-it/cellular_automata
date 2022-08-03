use egui::Pos2;
use macroquad::prelude::*;
use std::mem;
use std::sync::{Arc, Mutex};
use std::{thread, time};

#[derive(Clone)]
struct SimulationConfig {
    pub board_size: IVec2,
    pub hovered: bool,
    pub auto_run: bool,
    pub should_simulate_next_frame: bool,
    pub wait_time: f32,
    pub drawing: bool,
    pub erasing: bool,
    pub draw_borders: bool,
    pub error: bool,
    pub alive_color: Color,
    pub dead_color: Color,
}

impl SimulationConfig {
    pub fn new(board_size: IVec2) -> Self {
        Self {
            board_size,
            hovered: false,
            auto_run: false,
            should_simulate_next_frame: false,
            wait_time: 0.0,
            drawing: false,
            erasing: false,
            draw_borders: true,
            error: false,
            alive_color: BLACK,
            dead_color: WHITE,
        }
    }
}

#[derive(Clone)]
struct Board {
    pub cx: i32,
    pub cy: i32,
    pub states: Vec<i8>,
}

#[derive(Clone)]
struct Rules {
    pub birth: Vec<bool>,
    pub survive: Vec<bool>,
    pub adding_lifetime: i8,
}

impl Board {
    pub fn new(dimensions: IVec2) -> Board {
        Board {
            cx: dimensions.x,
            cy: dimensions.y,
            states: vec![0; (dimensions.x * dimensions.y) as usize],
        }
    }

    pub fn get_cell_at_position(&self, x: i32, y: i32) -> i8 {
        let nx = (x + self.cx) % self.cx;
        let ny = (y + self.cy) % self.cy;

        let index = (ny * self.cx + nx) as usize;
        self.states[index]
    }

    pub fn set_cell_at_position(&mut self, x: i32, y: i32, state: i8) {
        let index = (y * self.cx + x) as usize;
        self.states[index] = state;
    }

    pub fn lower_cell_lifetime(&mut self, x: i32, y: i32, amount: i8) {
        let index = (y * self.cx + x) as usize;
        self.states[index] -= amount;
        self.states[index] = self.states[index].max(0);
    }
}

#[macroquad::main("Cellular Automata")]
async fn main() {
    let game_mtx = Arc::new(Mutex::new(Board::new(const_ivec2!([26, 24]))));

    //Initial board setup
    let mut c_game = game_mtx.lock().unwrap();

    c_game.set_cell_at_position(1, 1, 1);
    c_game.set_cell_at_position(2, 1, 1);
    c_game.set_cell_at_position(3, 1, 1);

    drop(c_game);

    let rules = Rules {
        birth: vec![false, false, false, true, false, false, false, false, false],
        survive: vec![false, false, true, true, false, false, false, false, false],
        adding_lifetime: 1,
    };

    let mut cell_below_mouse = IVec2::new(0, 0);

    let mut simulation_cfg = SimulationConfig::new(const_ivec2!([26, 20]));

    let mut handler = thread::spawn(|| {});

    loop {
        egui_macroquad::ui(|egui_ctx| {
            let window = egui::Window::new("Configurations");
            let response = window
                .show(egui_ctx, |ui| {
                    simulation_cfg.error = false;
                    ui.label("Simulation Settings");
                    if ui.button("Next step").clicked() {
                        simulation_cfg.should_simulate_next_frame = true;
                    }
                    ui.checkbox(&mut simulation_cfg.auto_run, "Running");
                    ui.add(
                        egui::Slider::new(&mut simulation_cfg.wait_time, 0f32..=100f32)
                            .text("Tick time"),
                    );
                    ui.label("Cell controls");
                    ui.checkbox(&mut simulation_cfg.drawing, "Draw Cells");
                    ui.checkbox(&mut simulation_cfg.erasing, "Erase Cells");
                    if ui.button("Clear board").clicked() {
                        let mut c_game = game_mtx.lock().unwrap();
                        c_game.states = vec![0; (c_game.cx * c_game.cy) as usize];
                        drop(c_game);
                    }
                    if ui.button("Randomize Field").clicked() {
                        let mut c_game = game_mtx.lock().unwrap();
                        for i in 0..(c_game.cx * c_game.cy) as usize {
                            c_game.states[i] = if rand::rand() % 2 == 0 { 1 } else { 0 };
                        }
                        drop(c_game);
                    }
                    ui.label("Board Controls");
                    ui.add(
                        egui::Slider::new(&mut simulation_cfg.board_size.x, 1..=500)
                            .text("Board Width"),
                    );
                    ui.add(
                        egui::Slider::new(&mut simulation_cfg.board_size.y, 1..=500)
                            .text("Board Height"),
                    );
                    ui.checkbox(&mut simulation_cfg.draw_borders, "Draw borders");
                    //Error handling
                    if simulation_cfg.drawing && simulation_cfg.erasing {
                        ui.label("You can't draw and erase at the same time");
                        simulation_cfg.error = true;
                    }
                })
                .unwrap()
                .response;

            simulation_cfg.hovered = response.rect.contains(Pos2::new(
                mouse_position().0 as f32,
                mouse_position().1 as f32,
            ));
        });

        //Goes to next step if an error happened
        if simulation_cfg.error {
            egui_macroquad::draw();
            next_frame().await;
            continue;
        }

        let mut game = game_mtx.lock().unwrap();

        if is_mouse_button_down(MouseButton::Left) {
            let fx = mouse_position().0 as f32;
            let fy = mouse_position().1 as f32;

            let wx = screen_width() / game.cx as f32;
            let wy = screen_height() / game.cy as f32;

            let x = (fx / wx) as i32;
            let y = (fy / wy) as i32;

            cell_below_mouse = IVec2::new(x, y);
        }

        //Resizes the board if the user changes the size of the board
        if game.cx != simulation_cfg.board_size.x || game.cy != simulation_cfg.board_size.y {
            let _ = mem::replace(
                &mut *game,
                Board::new(const_ivec2!([
                    simulation_cfg.board_size.x,
                    simulation_cfg.board_size.y
                ])),
            );
        }

        //Drawing and erasing
        if is_mouse_button_down(MouseButton::Left) && !simulation_cfg.hovered {
            if simulation_cfg.drawing {
                game.set_cell_at_position(cell_below_mouse.x, cell_below_mouse.y, 1);
            } else if simulation_cfg.erasing {
                game.set_cell_at_position(cell_below_mouse.x, cell_below_mouse.y, 0);
            }
        }

        //Runs the simulation automatically
        if simulation_cfg.auto_run || simulation_cfg.should_simulate_next_frame {
            simulation_cfg.should_simulate_next_frame = false;

            if handler.is_finished() {
                let sm_cfg = simulation_cfg.clone();
                let game_for_thread = game_mtx.clone();
                let c_rules = rules.clone();
                handler = thread::spawn(move || {
                    let time = sm_cfg.wait_time;
                    thread::sleep(time::Duration::from_millis((time * 10.0) as u64));
                    let mut c_game = game_for_thread.lock().unwrap();
                    next_step(&mut c_game, &c_rules);
                });
            }
        }

        clear_background(simulation_cfg.dead_color);

        draw_board(&game, &simulation_cfg);

        egui_macroquad::draw();

        next_frame().await
    }
}

fn next_step(game: &mut Board, rules: &Rules) {
    let mut new_game_state = Board::new(const_ivec2!([game.cx, game.cy]));

    for y in 0..game.cy {
        for x in 0..game.cx {
            let mut neighbors = 0;

            for i in 0..3 {
                for j in 0..3 {
                    if i == 1 && j == 1 {
                        continue;
                    }
                    let px = x + i - 1;
                    let py = y + j - 1;
                    neighbors += game.get_cell_at_position(px, py);
                }
            }

            if game.get_cell_at_position(x, y) > 0 {
                if rules.survive[neighbors as usize] {
                    new_game_state.set_cell_at_position(x, y, 1);
                } else {
                    new_game_state.lower_cell_lifetime(x, y, 1);
                }
            } else if rules.birth[neighbors as usize] {
                new_game_state.set_cell_at_position(x, y, rules.adding_lifetime.clone());
            } else {
                new_game_state.set_cell_at_position(x, y, 0);
            }
        }
    }

    game.states = new_game_state.states.clone();
}

fn draw_board(game: &Board, simulation_cfg: &SimulationConfig) {
    let wx = screen_width() / game.cx as f32;
    let wy = screen_height() / game.cy as f32;

    for x in 0..game.cx {
        for y in 0..game.cy {
            let px = x as f32 * wx;
            let py = y as f32 * wy;

            if simulation_cfg.draw_borders {
                if game.states[(x + y * game.cx) as usize] == 0 {
                    draw_rectangle_lines(
                        px as f32,
                        py as f32,
                        wx,
                        wy,
                        3.0,
                        simulation_cfg.alive_color,
                    );
                } else {
                    draw_rectangle(px as f32, py as f32, wx, wy, simulation_cfg.alive_color);
                }
            } else if game.states[(x + y * game.cx) as usize] == 0 {
                //draw_rectangle(px as f32, py as f32, wx, wy,  WHITE);
            } else {
                draw_rectangle(px as f32, py as f32, wx, wy, simulation_cfg.alive_color);
            }

            //Draws the rectangle on the Macroquad window
        }
    }
}
