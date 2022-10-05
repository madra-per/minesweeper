use cell::{Cellule, State};
use rand::Rng;
use yew::html::Scope;
use yew::{classes, html, Component, Context, Html, MouseEvent};

mod cell;

pub enum Msg {
    Start,
    Reset,
    ToggleCellule(usize),
    ToggleMark(usize),
    Stop,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum GameState {
    New,
    InProgress,
    Over,
}
pub struct App {
    state: GameState,
    cellules: Vec<Cellule>,
    cellules_width: usize,
    cellules_height: usize,
}

impl App {
    pub fn random_mutate(&mut self, idx: usize) {
        for row in 1..self.cellules_height - 1 {
            for col in 1..self.cellules_width - 1 {
                let current_idx = self.row_col_as_idx(row as isize, col as isize);
                if rand::thread_rng().gen::<u8>() < 25 {
                    self.cellules[current_idx].set_mine();
                }
            }
        }
        self.cellules[idx].reset();
        for cl in self.ref_neighbors((idx / self.cellules_width).try_into().unwrap(), (idx % self.cellules_width).try_into().unwrap()) {
            self.cellules[cl].reset();
        }
        for row in 1..self.cellules_height - 1 {
            for col in 1..self.cellules_width - 1 {
                let neighbors = self.neighbors(row as isize, col as isize);

                let current_idx = self.row_col_as_idx(row as isize, col as isize);
                self.cellules[current_idx].set_value(&neighbors);
            }
        }
    }

    fn reset(&mut self) {
        for row in 1..self.cellules_height - 1 {
            for col in 1..self.cellules_width - 1 {
                let current_idx = self.row_col_as_idx(row as isize, col as isize);
                self.cellules[current_idx].reset();
            }
        }
    }

    fn expand_zero(&mut self, idx: usize){
        for cl in self.ref_neighbors((idx / self.cellules_width).try_into().unwrap(), (idx % self.cellules_width).try_into().unwrap()) {
            if self.cellules[cl].is_hidden() {
                self.cellules[cl].toggle();
                if self.cellules[cl].is_zero() {
                    self.expand_zero(cl);
                }
            }
        }
    }

    fn neighbors(&self, row: isize, col: isize) -> [Cellule; 8] {
        [
            self.cellules[self.row_col_as_idx(row + 1, col)],
            self.cellules[self.row_col_as_idx(row + 1, col + 1)],
            self.cellules[self.row_col_as_idx(row + 1, col - 1)],
            self.cellules[self.row_col_as_idx(row - 1, col)],
            self.cellules[self.row_col_as_idx(row - 1, col + 1)],
            self.cellules[self.row_col_as_idx(row - 1, col - 1)],
            self.cellules[self.row_col_as_idx(row, col - 1)],
            self.cellules[self.row_col_as_idx(row, col + 1)],
        ]
    }

    fn ref_neighbors(&self, row: isize, col: isize) -> [usize; 8] {
        [
            self.row_col_as_idx(row + 1, col),
            self.row_col_as_idx(row + 1, col + 1),
            self.row_col_as_idx(row + 1, col - 1),
            self.row_col_as_idx(row - 1, col),
            self.row_col_as_idx(row - 1, col + 1),
            self.row_col_as_idx(row - 1, col - 1),
            self.row_col_as_idx(row, col - 1),
            self.row_col_as_idx(row, col + 1),
        ]
    }

    fn row_col_as_idx(&self, row: isize, col: isize) -> usize {
        let row = wrap(row, self.cellules_height as isize);
        let col = wrap(col, self.cellules_width as isize);

        row * self.cellules_width + col
    }

    fn view_cellule(&self, idx: usize, cellule: &Cellule, link: &Scope<Self>) -> Html {
        let cellule_status = {
            match cellule.state {
                State::Hidden => "cellule-hidden",
                State::Marked => "cellule-marked",
                State::Revealed => "cellule-revealed",
                State::Outside => "cellule-border",
            }
        };
        html! {
            <div key={idx} class={classes!("game-cellule", cellule_status)}
                oncontextmenu={link.callback(move |e: MouseEvent| {
                    e.prevent_default();
                    Msg::ToggleMark(idx)
                })}
                onclick={link.callback(move |_| {
                    Msg::ToggleCellule(idx)
                })}>
                { if cellule.is_revealed() && !cellule.is_zero() { cellule.val.to_string() } else { String::from("") } }
            </div>
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let (cellules_width, cellules_height) = (53, 40);
        let mut app = Self {
            state: GameState::New,
            cellules: vec![Cellule::new_empty(); cellules_width * cellules_height],
            cellules_width,
            cellules_height,
        };
        app.reset();
        app
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Start => {
                self.reset();
                self.state = GameState::New;
                log::info!("Start");
                true
            }
            Msg::Reset => {
                self.reset();
                self.state = GameState::New;
                log::info!("Reset");
                true
            }
            Msg::ToggleCellule(idx) => {
                if self.state == GameState::InProgress {
                    let cellule = self.cellules.get_mut(idx).unwrap();
                    cellule.toggle();
                    if cellule.is_mine() {
                        self.state = GameState::Over;
                    }   
                    else if cellule.val == 0 {
                        self.expand_zero(idx);
                    }
                } else if self.state == GameState::New {
                    self.random_mutate(idx);
                    let cellule = self.cellules.get_mut(idx).unwrap();
                    self.state = GameState::InProgress;
                    cellule.toggle();
                    self.expand_zero(idx);
                }
                true
            }
            Msg::ToggleMark(idx) => {
                let cellule = self.cellules.get_mut(idx).unwrap();
                if self.state == GameState::InProgress {
                    cellule.toggle_marked();
                }
                true
            }
            Msg::Stop => {
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let cell_rows =
            self.cellules
                .chunks(self.cellules_width)
                .enumerate()
                .map(|(y, cellules)| {
                    let idx_offset = y * self.cellules_width;

                    let cells = cellules
                        .iter()
                        .enumerate()
                        .map(|(x, cell)| self.view_cellule(idx_offset + x, cell, ctx.link()));
                    html! {
                        <div key={y} class="game-row">
                            { for cells }
                        </div>
                    }
                });

        html! {
            <div>
                <section class="game-container">
                    <header class="app-header">
                        <h1 class="app-title">{ "Minesweeper" }</h1>
                    </header>
                    <section class="game-area">
                        <div class="game">
                            { for cell_rows }
                        </div>
                        <div class="game-buttons">
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Start)}>{ "Start" }</button>
                            <button class="game-button" onclick={ctx.link().callback(|_| Msg::Reset)}>{ "Reset" }</button>
                        </div>
                    </section>
                </section>
                <footer class="app-footer">
                    <strong class="footer-text">
                      { "Minesweeper - a yew experiment " }
                    </strong>
                    <a href="https://github.com/yewstack/yew" target="_blank">{ "source" }</a>
                </footer>
            </div>
        }
    }
}

fn wrap(coord: isize, range: isize) -> usize {
    let result = if coord < 0 {
        coord + range
    } else if coord >= range {
        coord - range
    } else {
        coord
    };
    result as usize
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::trace!("Initializing yew...");
    yew::start_app::<App>();
}