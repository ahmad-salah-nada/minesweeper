use std::vec;
use eframe::egui::{self, RichText};
use egui::Grid;
use rand::Rng;

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct Cell {
    is_mine: bool,
    is_flagged: bool,
    is_revealed: bool,
    adjacent_mines: u8,
}

impl Default for Cell {
    fn default() -> Self {
        Self {
            is_mine: false,
            is_flagged: false,
            is_revealed: false,
            adjacent_mines: 0,
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct GameState {
    grid: Vec<Vec<Cell>>,
    game_over: bool,
    game_won: bool,
    width: usize,
    height: usize,
    mines_count: usize,
    mines_left: i32,
}

impl Default for GameState {
    fn default() -> Self {
        Self {
            grid: Vec::new(),
            game_won: false,
            game_over: false,
            width: 0,
            height: 0,
            mines_count: 0,
            mines_left: 0,
        }
    }
}

impl GameState {
    fn new(width: usize, height: usize, mines_count: usize) -> Self {
        let mut grid = Vec::with_capacity(height);

        for _ in 0..height {
            let mut row = Vec::with_capacity(width);
            for _ in 0..width {
                row.push(Cell{
                    is_mine: false,
                    is_flagged: false,
                    is_revealed: false,
                    adjacent_mines: 0,
                });
            }
            grid.push(row);
        }

        let mut rng = rand::thread_rng();
        let mut mines_placed = 0;

        while mines_placed < mines_count {
            let x = rng.gen_range(0..width);
            let y = rng.gen_range(0..height);

            if !grid[y][x].is_mine {
                grid[y][x].is_mine = true;
                mines_placed += 1;
            }
        }

        for i in 0..height {
            for j in 0..width {
                grid[i][j].adjacent_mines = Self::count_adjacent_mines(&grid, j, i, width, height);
            }
        }

        Self {
            grid: grid,
            game_won: false,
            game_over: false,
            width: width,
            height: height,
            mines_count: mines_count,
            mines_left: mines_count as i32,
        }
    }

    fn count_adjacent_mines(grid: &Vec<Vec<Cell>>, x: usize, y: usize, width: usize, height: usize) -> u8 {
        let mut count = 0;
        for i in x.saturating_sub(1)..=usize::min(x + 1, width - 1) {
            for j in y.saturating_sub(1)..=usize::min(y + 1, height - 1) {
                count += (!(i == x && j == y) && grid[j][i].is_mine) as u8;
            }
        }
        count
    }

    fn reveal_cell(&mut self, x: usize, y: usize, clicked: bool) {
        if self.grid[y][x].is_revealed { return; }
        self.grid[y][x].is_revealed = true;

        if clicked && self.grid[y][x].is_mine { self.game_over = true; }
        if self.grid[y][x].is_flagged { self.mines_left += 1; }
        if self.grid[y][x].is_mine || self.grid[y][x].adjacent_mines > 0 { return; }

        for i in x.saturating_sub(1)..=usize::min(x + 1, self.width - 1) {
            for j in y.saturating_sub(1)..=usize::min(y + 1, self.height - 1) {
                self.reveal_cell(i, j, false);
            }
        }
    }
}

#[derive(serde::Deserialize, serde::Serialize)]
#[serde(default)]
struct MinesweeperApp {
    score: u32,
    difficulty: u8,
    game_state: GameState,
}

impl Default for MinesweeperApp {
    fn default() -> Self {
        Self {
            score: 0,
            difficulty: 2,
            game_state: GameState::new(40, 16, 99),
        }
    }
}

impl MinesweeperApp {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        if let Some(storage) = cc.storage {
            return eframe::get_value(storage, eframe::APP_KEY).unwrap_or_default();
        }
        Default::default()
    }
    pub fn easy(&mut self) {
        self.difficulty = 0;
        self.game_state = GameState::new(9, 9, 10);
    }
    pub fn medium(&mut self) {
        self.difficulty = 1;
        self.game_state = GameState::new(16, 16, 40);
    }
    pub fn hard(&mut self) {
        self.difficulty = 2;
        self.game_state = GameState::new(40, 16, 99);
    }
}

impl eframe::App for MinesweeperApp {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        eframe::set_value(storage, eframe::APP_KEY, self);
    }

    fn update(&mut self, ctx: &egui::Context, frame: &mut eframe::Frame) {
        egui::CentralPanel::default().show(ctx, |ui| {

            egui::TopBottomPanel::top("Menu").show(ctx, |ui| {
    
                egui::menu::bar(ui, |ui| {

                    ui.menu_button("Menu", |ui| {
                        if ui.button("Easy").clicked() {
                            self.easy();
                        }
                        if ui.button("Medium").clicked() {
                            self.medium();
                        }
                        if ui.button("Hard").clicked() {
                            self.hard();
                        }
                        if ui.button("Quit").clicked() {
                            ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                        }
                    });
                        
                    ui.add_space(16.0);
                    
                    egui::widgets::global_dark_light_mode_buttons(ui);

                    ui.add_space(16.0);

                    ui.label(String::from("Score: ") + &self.score.to_string()[..]);

                    ui.add_space(6.0);

                    if ui.button("Reset Score").clicked() {
                        self.score = 0;
                    }
                });
                ui.add_space(2.0);

                egui::menu::bar(ui, |ui| {
                    ui.label(String::from("Mines left: ") + &self.game_state.mines_left.to_string()[..]);
                    
                    ui.add_space(16.0);
    
                    if ui.button("Restart").clicked() {
                        self.game_state = GameState::new(self.game_state.width, self.game_state.height, self.game_state.mines_count);
                    }
                });
                
            });

            egui::CentralPanel::default().show(ctx, |ui| {
                let mut total_revealed = 0;

                egui::Grid::new("grid_id").spacing(egui::Vec2::new(-18.0, 3.0)).show(ui, |ui| {
                    for y in 0..self.game_state.height {
                        for x in 0..self.game_state.width {
                            let cell = &mut self.game_state.grid[y][x];
                            if self.game_state.game_over {
                                if cell.is_mine {
                                    ui.colored_label(egui::Color32::RED, "X");
                                    continue;
                                }
                            }
                            if cell.is_revealed {
                                if cell.is_mine {
                                    ui.colored_label(egui::Color32::RED, "X");
                                } else {
                                    ui.label(String::from(" ") + &cell.adjacent_mines.to_string()[..]);
                                }
                                total_revealed += 1;
                            } else {
                                let mut s: RichText = egui::RichText::new("?");
                                if cell.is_flagged{
                                    s = egui::RichText::new("F").color(egui::Color32::GREEN);
                                };
                                let b = ui.button(s);
                                if self.game_state.game_over || self.game_state.game_won {
                                    continue;
                                }
                                if b.clicked_by(egui::PointerButton::Secondary){
                                    if cell.is_flagged {
                                        cell.is_flagged = false;
                                        self.game_state.mines_left += 1;
                                    }
                                    else{
                                        cell.is_flagged = true;
                                        self.game_state.mines_left -= 1;
                                    }
                                }
                                else if b.clicked() {
                                    self.game_state.reveal_cell(x, y, true);
                                }
                                
                            }
                        }
                        ui.end_row();
                    }
                });

                if !self.game_state.game_over && total_revealed == self.game_state.width * self.game_state.height - self.game_state.mines_count {
                    self.score += !self.game_state.game_won as u32;
                    self.game_state.game_won = true;
                }
            });
        });
    }
}

fn main() -> eframe::Result<()> {
    let native_options = eframe::NativeOptions::default();
    eframe::run_native(
        "Minesweeper",
        native_options,
        Box::new(|cc| Box::new(MinesweeperApp::new(cc))),
    )
}