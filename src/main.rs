use egui::Pos2;
use macroquad::prelude::*;

//const BOARD_SIZE: IVec2 = const_ivec2!([26, 20]);

struct Board_Config {
    pub board_size: IVec2,
    pub hovered: bool,  
    pub auto_run: bool,
    pub drawing: bool,
    pub erasing: bool, 
    pub error: bool,
}

impl Board_Config {
    pub fn new(board_size: IVec2) -> Self {
        Self {
            board_size,
            hovered: false,
            auto_run: false,
            drawing: false,
            erasing: false,
            error: false
        }
    }
}

struct Board {
    pub cx: i32,
    pub cy: i32,
    pub states: Vec<i8>,
}

struct Rules {
    pub birth: Vec<bool>,
    pub survive: Vec<bool>,
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
}

#[macroquad::main("Cellular Automata")]
async fn main() {
    let mut game: Board = Board::new(const_ivec2!([26, 24]));

    let rules = Rules {
        birth: vec![false, false, false, true, false, false, false, false, false],
        survive: vec![false, false, true, true, false, false, false, false, false],
    };

    game.set_cell_at_position(1, 1, 1);
    game.set_cell_at_position(2, 1, 1);
    game.set_cell_at_position(3, 1, 1);

    let mut cell_below_mouse = IVec2::new(0, 0);

    let mut window_cfg = Board_Config::new(
        const_ivec2!([26, 20])
    );

    loop {
        clear_background(WHITE);

        if is_mouse_button_down(MouseButton::Left) {
            let fx = mouse_position().0 as f32;
            let fy = mouse_position().1 as f32;

            let wx = screen_width() / game.cx as f32;
            let wy = screen_height() / game.cy as f32;

            let x = (fx / wx) as i32;
            let y = (fy / wy) as i32;

            cell_below_mouse = IVec2::new(x, y);
        }

        egui_macroquad::ui(|egui_ctx| {
            let mut window = egui::Window::new("Configurations");
            let mut response = window.show(egui_ctx, |ui| {
                    window_cfg.error = false;
                    ui.label("Simulation Settings");
                    if ui.button("Next step").clicked() {
                        next_step(&mut game, &rules);
                    }
                    ui.checkbox(&mut window_cfg.auto_run, "Running");
                    ui.label("Cell controls");
                    ui.checkbox(&mut window_cfg.drawing, "Draw Cells");
                    ui.checkbox(&mut window_cfg.erasing, "Erase Cells");
                    ui.label("Board Controls");
                    ui.add(egui::Slider::new(&mut window_cfg.board_size.x, 1..=100).text("Board Width"));
                    ui.add(egui::Slider::new(&mut window_cfg.board_size.y, 1..=100).text("Board Height"));
                                        
                    //Error handling
                    if window_cfg.drawing && window_cfg.erasing {
                        ui.label("You can't draw and erase at the same time");
                        window_cfg.error = true;
                    }
                }).unwrap().response;

            window_cfg.hovered = response.rect.contains(Pos2::new(mouse_position().0 as f32, mouse_position().1 as f32));
        });

        if game.cx != window_cfg.board_size.x || game.cy != window_cfg.board_size.y {
            game = Board::new(const_ivec2!([window_cfg.board_size.x, window_cfg.board_size.y]));
        }

        if window_cfg.error {
            egui_macroquad::draw();
            next_frame().await;
            continue;
        }

        if is_mouse_button_down(MouseButton::Left) && !window_cfg.hovered {
            if window_cfg.drawing {
                game.set_cell_at_position(cell_below_mouse.x, cell_below_mouse.y, 1);
            } else if window_cfg.erasing {
                game.set_cell_at_position(cell_below_mouse.x, cell_below_mouse.y, 0);
            }
        }

        if(window_cfg.auto_run) {
            next_step(&mut game, &rules);
        }

        draw_board(&game);

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

            if game.get_cell_at_position(x, y) == 1 {
                if rules.survive[neighbors as usize] {
                    new_game_state.set_cell_at_position(x, y, 1);
                } else {
                    new_game_state.set_cell_at_position(x, y, 0);
                }
            } else {
                if rules.birth[neighbors as usize] {
                    new_game_state.set_cell_at_position(x, y, 1);
                } else {
                    new_game_state.set_cell_at_position(x, y, 0);
                }
            }
        }
    }

    game.states = new_game_state.states.clone();
}

fn draw_board(game: &Board) {
    let wx = screen_width() / game.cx as f32;
    let wy = screen_height() / game.cy as f32;

    for x in 0..game.cx {
        for y in 0..game.cy {
            let px = x as f32 * wx;
            let py = y as f32 * wy;

            if game.states[(x + y * game.cx) as usize] == 0 {
                draw_rectangle_lines(px as f32, py as f32, wx, wy, 3.0, BLACK);
            } else {
                draw_rectangle(px as f32, py as f32, wx, wy, BLACK);
            }

            //Draws the rectangle on the Macroquad window
        }
    }
}
