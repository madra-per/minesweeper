use cell::{Cellule, State};
use rand::Rng;
use std::collections::HashSet;
use wasm_bindgen::JsCast;
use yew::html::Scope;
use yew::{classes, html, Component, Context, Html, MouseEvent, InputEvent};

mod cell;

pub enum Msg {
    Start,
    ToggleCellule(usize),
    ToggleMark(usize),
    Chord(usize),
    ClearChord,
    ToggleMode,
    ToggleSettings,
    SetWidth(String),
    SetHeight(String),
    SetMines(String),
    SetAngelAttempts(String),
    SetMaxFailedLogs(String),
    ToggleDebug,
    Preset(u8),
    DownloadLogs,
    Stop,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameMode {
    Normal,
    Angel,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GameState {
    New,
    InProgress,
    Won,
    Over,
}
pub struct App {
    state: GameState,
    mode: GameMode,
    cellules: Vec<Cellule>,
    cellules_width: usize,
    cellules_height: usize,
    mine_count: usize,
    angel_attempts: usize,
    max_failed_logs: usize,
    show_settings: bool,
    show_debug: bool,
    chord_active: bool,
    debug_log: Vec<String>,
    failed_logs: Vec<String>,
    // Settings input buffers (not yet applied)
    input_width: String,
    input_height: String,
    input_mines: String,
    input_angel_attempts: String,
    input_max_failed_logs: String,
}

impl App {
    pub fn random_mutate(&mut self, idx: usize) {
        let excluded: HashSet<usize> = {
            let mut set = HashSet::new();
            set.insert(idx);
            for cl in self.ref_neighbors(
                    (idx / self.cellules_width) as isize,
                    (idx % self.cellules_width) as isize) {
                set.insert(cl);
            }
            set
        };

        let mut candidates: Vec<usize> = (1..self.cellules_height - 1)
            .flat_map(|row| (1..self.cellules_width - 1).map(move |col| (row, col)))
            .map(|(row, col)| self.row_col_as_idx(row as isize, col as isize))
            .filter(|i| !excluded.contains(i))
            .collect();

        let mut rng = rand::thread_rng();
        let mines_to_place = self.mine_count.min(candidates.len());
        for i in 0..mines_to_place {
            let j = rng.gen_range(i..candidates.len());
            candidates.swap(i, j);
        }
        for &i in &candidates[..mines_to_place] {
            self.cellules[i].set_mine();
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

    fn chord(&mut self, idx: usize) -> bool {
        let cellule = self.cellules[idx];
        if !cellule.is_revealed() || cellule.val <= 0 {
            return false;
        }
        let row = (idx / self.cellules_width) as isize;
        let col = (idx % self.cellules_width) as isize;
        let neighbor_indices = self.ref_neighbors(row, col);

        let marked_count = neighbor_indices.iter()
            .filter(|&&i| self.cellules[i].is_marked())
            .count() as i8;

        if marked_count != cellule.val {
            return false;
        }

        let hidden_neighbors: Vec<usize> = neighbor_indices.iter()
            .copied()
            .filter(|&i| self.cellules[i].is_hidden())
            .collect();

        if hidden_neighbors.is_empty() {
            return false;
        }

        // In angel mode, check if any hidden neighbor is a mine and try to relocate
        let has_mine = hidden_neighbors.iter().any(|&i| self.cellules[i].is_mine());
        if has_mine && self.mode == GameMode::Angel {
            if !self.angel_relocate(&hidden_neighbors) {
                self.state = GameState::Over;
                self.reveal_all_mines();
                return true;
            }
        }

        let mut changed = false;
        for &ni in &hidden_neighbors {
            self.cellules[ni].toggle();
            changed = true;
            if self.cellules[ni].is_mine() {
                self.state = GameState::Over;
                self.reveal_all_mines();
                return true;
            } else if self.cellules[ni].is_zero() {
                self.expand_zero(ni);
            }
        }
        if changed && self.state == GameState::InProgress {
            self.check_and_set_win();
        }
        changed
    }

    fn expand_zero(&mut self, idx: usize){
        for cl in self.ref_neighbors(   
                (idx / self.cellules_width).try_into().unwrap(),
                (idx % self.cellules_width).try_into().unwrap()) {
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

    fn remaining_mines(&self) -> isize {
        let total_mines = self.cellules.iter().filter(|c| c.is_mine()).count() as isize;
        let marked = self.cellules.iter().filter(|c| c.is_marked()).count() as isize;
        total_mines - marked
    }

    fn check_win(&self) -> bool {
        self.cellules.iter().all(|c| {
            c.state == State::Outside || c.is_revealed() || c.is_mine()
        })
    }

    fn reveal_all_mines(&mut self) {
        for c in self.cellules.iter_mut() {
            if c.is_mine() {
                c.set_revealed();
            }
        }
    }

    fn check_and_set_win(&mut self) {
        if self.check_win() {
            self.state = GameState::Won;
        }
    }

    /// Try to find a valid mine layout where all `safe_cells` are mine-free
    /// and all revealed cells keep their current neighbor counts.
    /// Returns true if a valid layout was found (board is updated).
    fn angel_relocate(&mut self, safe_cells: &[usize]) -> bool {
        let mines_to_move: Vec<usize> = safe_cells.iter()
            .copied()
            .filter(|&i| self.cellules[i].is_mine())
            .collect();

        if mines_to_move.is_empty() {
            self.debug_log.push("Angel: no mines to move".to_string());
            return true;
        }

        let safe_coords: Vec<String> = mines_to_move.iter()
            .map(|&i| format!("({},{})", i / self.cellules_width, i % self.cellules_width))
            .collect();
        self.debug_log.push(format!("Angel: moving {} mine(s) at {}", mines_to_move.len(), safe_coords.join(", ")));

        let original = self.cellules.clone();
        let safe_set: HashSet<usize> = safe_cells.iter().copied().collect();

        // Fast path: try simple single-mine swaps
        let mut all_moved = true;
        for &mine_idx in &mines_to_move {
            if !self.cellules[mine_idx].is_mine() {
                continue;
            }
            if !self.move_single_mine(mine_idx, &safe_set) {
                all_moved = false;
                break;
            }
        }
        if all_moved {
            self.debug_log.push("Angel: SUCCESS via direct swap".to_string());
            return true;
        }

        // Fallback: constraint propagation + random redistribution
        self.cellules = original.clone();
        self.debug_log.push("Angel: direct swap failed, trying redistribution...".to_string());

        if self.angel_redistribute(&safe_set, &original) {
            return true;
        }

        self.cellules = original;
        self.debug_log.push("Angel: FAILED — no valid relocation found".to_string());
        self.debug_log.push(self.board_snapshot(&safe_set));

        // Store failed attempt log
        if self.max_failed_logs > 0 {
            let snapshot = self.debug_log.join("\n");
            self.failed_logs.push(snapshot);
            while self.failed_logs.len() > self.max_failed_logs {
                self.failed_logs.remove(0);
            }
            #[cfg(not(test))]
            save_failed_logs(&self.failed_logs);
        }

        false
    }

    /// Serialize board state as machine-readable text.
    /// Format: JSON-like with metadata and a per-cell grid.
    /// Each cell: R0-R8 = revealed 0-8, M = mine(hidden), H = hidden(safe),
    /// F = flagged mine, f = flagged(no mine), S = safe_cell(mine that triggered angel)
    fn board_snapshot(&self, safe_cells: &HashSet<usize>) -> String {
        let inner_w = self.cellules_width - 2;
        let inner_h = self.cellules_height - 2;
        let mut rows: Vec<String> = Vec::new();
        for row in 1..self.cellules_height - 1 {
            let mut cells: Vec<String> = Vec::new();
            for col in 1..self.cellules_width - 1 {
                let idx = row * self.cellules_width + col;
                let cell = &self.cellules[idx];
                let code = if safe_cells.contains(&idx) {
                    "S".to_string()
                } else {
                    match cell.state {
                        State::Revealed => {
                            format!("R{}", cell.val)
                        }
                        State::Marked => {
                            if cell.is_mine() { "F".to_string() } else { "f".to_string() }
                        }
                        State::Hidden => {
                            if cell.is_mine() { "M".to_string() } else { "H".to_string() }
                        }
                        State::Outside => "X".to_string(),
                    }
                };
                cells.push(code);
            }
            rows.push(cells.join(","));
        }
        format!("BOARD:{}x{}:mines={}:attempts={}\n{}",
            inner_w, inner_h, self.mine_count, self.angel_attempts,
            rows.join("\n"))
    }

    /// Fast path: move a single mine to a cell with identical revealed-neighbor signature.
    fn move_single_mine(&mut self, from: usize, safe_set: &HashSet<usize>) -> bool {
        let from_row = (from / self.cellules_width) as isize;
        let from_col = (from % self.cellules_width) as isize;
        let from_neighbor_indices = self.ref_neighbors(from_row, from_col);

        let from_revealed: HashSet<usize> = from_neighbor_indices.iter()
            .copied()
            .filter(|&i| self.cellules[i].is_revealed())
            .collect();

        let candidates: Vec<usize> = self.cellules.iter().enumerate()
            .filter(|(i, c)| {
                c.is_hidden() && !c.is_mine() && !safe_set.contains(i) && c.state != State::Outside
            })
            .map(|(i, _)| i)
            .collect();

        for &to in &candidates {
            let to_row = (to / self.cellules_width) as isize;
            let to_col = (to % self.cellules_width) as isize;
            let to_neighbor_indices = self.ref_neighbors(to_row, to_col);

            let to_revealed: HashSet<usize> = to_neighbor_indices.iter()
                .copied()
                .filter(|&i| self.cellules[i].is_revealed())
                .collect();

            if from_revealed != to_revealed {
                continue;
            }

            self.cellules[from].val = 0;
            self.cellules[to].set_mine();

            let affected: HashSet<usize> = from_neighbor_indices.iter()
                .chain(to_neighbor_indices.iter())
                .copied()
                .chain(std::iter::once(from))
                .collect();

            for &i in &affected {
                if self.cellules[i].state != State::Outside && !self.cellules[i].is_mine() {
                    let row = (i / self.cellules_width) as isize;
                    let col = (i % self.cellules_width) as isize;
                    let neighbors = self.neighbors(row, col);
                    self.cellules[i].set_value(&neighbors);
                }
            }

            self.debug_log.push(format!(
                "Angel: moved mine ({},{}) → ({},{})",
                from / self.cellules_width, from % self.cellules_width,
                to / self.cellules_width, to % self.cellules_width
            ));
            return true;
        }
        false
    }

    /// Fallback: use constraint propagation to find forced positions, then randomly
    /// place remaining mines among truly unconstrained cells.
    fn angel_redistribute(&mut self, safe_set: &HashSet<usize>, original: &[Cellule]) -> bool {
        let revealed: Vec<(usize, i8)> = original.iter().enumerate()
            .filter(|(_, c)| c.is_revealed())
            .map(|(i, c)| (i, c.val))
            .collect();

        let total_mines = original.iter().filter(|c| c.is_mine()).count();

        let mut must_safe: HashSet<usize> = safe_set.clone();
        let mut must_mine: HashSet<usize> = HashSet::new();
        let mut unknown: HashSet<usize> = original.iter().enumerate()
            .filter(|(_, c)| c.state != State::Outside && !c.is_revealed())
            .map(|(i, _)| i)
            .filter(|i| !must_safe.contains(i))
            .collect();

        // Constraint propagation
        let mut changed = true;
        while changed {
            changed = false;
            for &(ri, val) in &revealed {
                let row = (ri / self.cellules_width) as isize;
                let col = (ri % self.cellules_width) as isize;
                let neighbors = self.ref_neighbors(row, col);

                let known_mines = neighbors.iter()
                    .filter(|n| must_mine.contains(n)).count() as i8;
                let unknown_neighbors: Vec<usize> = neighbors.iter()
                    .filter(|n| unknown.contains(n)).copied().collect();
                let unknown_count = unknown_neighbors.len() as i8;

                if known_mines > val || known_mines + unknown_count < val {
                    self.debug_log.push(format!(
                        "Angel: contradiction at ({},{}) — need {} mines, have {} forced + {} unknown",
                        ri / self.cellules_width, ri % self.cellules_width,
                        val, known_mines, unknown_count));
                    return false;
                }

                if known_mines == val && !unknown_neighbors.is_empty() {
                    for &n in &unknown_neighbors {
                        must_safe.insert(n);
                        unknown.remove(&n);
                        changed = true;
                    }
                }

                if known_mines + unknown_count == val && !unknown_neighbors.is_empty() {
                    for &n in &unknown_neighbors {
                        must_mine.insert(n);
                        unknown.remove(&n);
                        changed = true;
                    }
                }
            }
        }

        let remaining_mines = total_mines as isize - must_mine.len() as isize;
        if remaining_mines < 0 || remaining_mines as usize > unknown.len() {
            self.debug_log.push(format!(
                "Angel: FAILED — need {} more mines but {} unknown cells",
                remaining_mines, unknown.len()));
            return false;
        }

        self.debug_log.push(format!(
            "Angel: {} forced mines, {} forced safe, {} unknown, {} to randomize",
            must_mine.len(), must_safe.len(), unknown.len(), remaining_mines));

        let unknown_vec: Vec<usize> = unknown.into_iter().collect();
        let remaining_mines = remaining_mines as usize;
        let mut rng = rand::thread_rng();

        for attempt in 0..self.angel_attempts {
            // Clear all inner cell values
            for row in 1..self.cellules_height - 1 {
                for col in 1..self.cellules_width - 1 {
                    let i = self.row_col_as_idx(row as isize, col as isize);
                    self.cellules[i].val = 0;
                }
            }

            // Place forced mines
            for &i in &must_mine {
                self.cellules[i].set_mine();
            }

            // Randomly place remaining mines among unknown cells
            let mut shuffled = unknown_vec.clone();
            if remaining_mines > 0 {
                for i in 0..remaining_mines {
                    let j = rng.gen_range(i..shuffled.len());
                    shuffled.swap(i, j);
                }
                for &i in &shuffled[..remaining_mines] {
                    self.cellules[i].set_mine();
                }
            }

            // Recalculate neighbor counts
            for row in 1..self.cellules_height - 1 {
                for col in 1..self.cellules_width - 1 {
                    let i = self.row_col_as_idx(row as isize, col as isize);
                    if !self.cellules[i].is_mine() {
                        let neighbors = self.neighbors(row as isize, col as isize);
                        self.cellules[i].set_value(&neighbors);
                    }
                }
            }

            // Verify all revealed cells keep their values
            if revealed.iter().all(|&(i, val)| self.cellules[i].val == val) {
                self.debug_log.push(format!("Angel: SUCCESS via redistribution after {} attempt(s)", attempt + 1));
                return true;
            }
        }

        self.debug_log.push(format!("Angel: redistribution FAILED — {} attempts exhausted", self.angel_attempts));
        false
    }

    fn apply_settings(&mut self) {
        let inner_w = self.input_width.parse::<usize>().unwrap_or(14).max(4).min(40);
        let inner_h = self.input_height.parse::<usize>().unwrap_or(8).max(4).min(25);
        let max_mines = (inner_w * inner_h).saturating_sub(9);
        let mines = self.input_mines.parse::<usize>().unwrap_or(24).max(1).min(max_mines);
        let attempts = self.input_angel_attempts.parse::<usize>().unwrap_or(1000).max(1).min(100_000);
        let max_logs = self.input_max_failed_logs.parse::<usize>().unwrap_or(10).max(0).min(1000);

        self.cellules_width = inner_w + 2;
        self.cellules_height = inner_h + 2;
        self.mine_count = mines;
        self.angel_attempts = attempts;
        self.max_failed_logs = max_logs;
        self.cellules = vec![Cellule::new_empty(); self.cellules_width * self.cellules_height];
        self.reset();
        self.state = GameState::New;
        self.debug_log.clear();
        self.failed_logs.clear();
        save_failed_logs(&self.failed_logs);

        self.input_width = inner_w.to_string();
        self.input_height = inner_h.to_string();
        self.input_mines = mines.to_string();
        self.input_angel_attempts = attempts.to_string();
        self.input_max_failed_logs = max_logs.to_string();

        save_settings(inner_w, inner_h, mines, attempts, &self.mode);

        let base_w = (inner_w as u32) * 22 + 10;
        let base_h = (inner_h as u32) * 22 + 70;
        let win_w = if self.mode == GameMode::Angel && self.show_debug { base_w + 300 } else { base_w };
        resize_window(win_w.max(300), base_h.max(200));
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
                })}
                onmousedown={link.callback(move |e: MouseEvent| {
                    if e.buttons() == 3 {
                        e.prevent_default();
                        Msg::Chord(idx)
                    } else {
                        Msg::ClearChord
                    }
                })}>
                { cellule.get_visual() }
            </div>
        }
    }
}

impl Component for App {
    type Message = Msg;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        let (inner_w, inner_h, mine_count, angel_attempts, mode) =
            if let Some((w, h, m, a, mode)) = load_settings() {
                let w = w.max(4).min(40);
                let h = h.max(4).min(25);
                let max_m = (w * h).saturating_sub(9);
                let m = m.max(1).min(max_m);
                let a = a.max(1).min(100_000);
                (w, h, m, a, mode)
            } else {
                (14, 8, 24, 1000, GameMode::Normal)
            };

        let (cellules_width, cellules_height) = (inner_w + 2, inner_h + 2);
        let mut app = Self {
            state: GameState::New,
            mode,
            cellules: vec![Cellule::new_empty(); cellules_width * cellules_height],
            cellules_width,
            cellules_height,
            mine_count,
            angel_attempts,
            max_failed_logs: 10,
            show_settings: false,
            show_debug: false,
            chord_active: false,
            debug_log: Vec::new(),
            failed_logs: load_failed_logs(),
            input_width: inner_w.to_string(),
            input_height: inner_h.to_string(),
            input_mines: mine_count.to_string(),
            input_angel_attempts: angel_attempts.to_string(),
            input_max_failed_logs: "10".to_string(),
        };
        app.reset();

        let base_w = (inner_w as u32) * 22 + 10;
        let base_h = (inner_h as u32) * 22 + 70;
        resize_window(base_w.max(300), base_h.max(200));

        app
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            Msg::Start => {
                self.show_settings = false;
                self.apply_settings();
                true
            }
            Msg::Chord(idx) => {
                if self.chord_active || self.state != GameState::InProgress {
                    return false;
                }
                self.chord_active = true;
                self.chord(idx)
            }
            Msg::ClearChord => {
                self.chord_active = false;
                false
            }
            Msg::ToggleCellule(idx) => {
                if self.chord_active {
                    return false;
                }
                if self.state == GameState::InProgress {
                    let cellule = self.cellules.get_mut(idx).unwrap();
                    if cellule.is_marked(){
                        return false
                    }
                    if cellule.is_mine() && self.mode == GameMode::Angel {
                        let row = idx / self.cellules_width;
                        let col = idx % self.cellules_width;
                        self.debug_log.push(format!("Click on MINE at ({},{}) — angel mode active", row, col));
                        if self.angel_relocate(&[idx]) {
                            // Mine was moved, cell is now safe
                            let cellule = self.cellules.get_mut(idx).unwrap();
                            cellule.toggle();
                            if cellule.is_zero() {
                                self.expand_zero(idx);
                            }
                        } else {
                            self.debug_log.push("Angel: could not save — game over".to_string());
                            let cellule = self.cellules.get_mut(idx).unwrap();
                            cellule.toggle();
                            self.state = GameState::Over;
                            self.reveal_all_mines();
                        }
                    } else if cellule.is_mine() {
                        self.debug_log.push(format!("Click on MINE at ({},{}) — normal mode, game over",
                            idx / self.cellules_width, idx % self.cellules_width));
                        cellule.toggle();
                        self.state = GameState::Over;
                        self.reveal_all_mines();
                    } else {
                        cellule.toggle();
                        if cellule.val == 0 {
                            self.expand_zero(idx);
                        }
                    }
                    if self.state == GameState::InProgress {
                        self.check_and_set_win();
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
                if self.chord_active {
                    return false;
                }
                let cellule = self.cellules.get_mut(idx).unwrap();
                if self.state == GameState::InProgress {
                    cellule.toggle_marked();
                }
                true
            }
            Msg::Stop => {
                false
            }
            Msg::ToggleMode => {
                self.mode = match self.mode {
                    GameMode::Normal => GameMode::Angel,
                    GameMode::Angel => GameMode::Normal,
                };
                self.reset();
                self.state = GameState::New;
                self.debug_log.clear();
                self.debug_log.push(format!("Mode switched to {:?}", self.mode));
                let inner_w = self.cellules_width.saturating_sub(2) as u32;
                let inner_h = self.cellules_height.saturating_sub(2) as u32;
                let base_w = inner_w * 22 + 10;
                let base_h = inner_h * 22 + 70;
                match self.mode {
                    GameMode::Angel if self.show_debug => resize_window((base_w + 300).max(300), base_h.max(200)),
                    _ => resize_window(base_w.max(300), base_h.max(200)),
                }
                save_settings(inner_w as usize, inner_h as usize, self.mine_count, self.angel_attempts, &self.mode);
                true
            }
            Msg::ToggleSettings => {
                if self.show_settings {
                    // Closing settings — apply and start new game
                    self.show_settings = false;
                    self.apply_settings();
                } else {
                    self.show_settings = true;
                }
                true
            }
            Msg::SetWidth(v) => { self.input_width = v; false }
            Msg::SetHeight(v) => { self.input_height = v; false }
            Msg::SetMines(v) => { self.input_mines = v; false }
            Msg::SetAngelAttempts(v) => { self.input_angel_attempts = v; false }
            Msg::SetMaxFailedLogs(v) => { self.input_max_failed_logs = v; false }
            Msg::ToggleDebug => {
                self.show_debug = !self.show_debug;
                let inner_w = self.cellules_width.saturating_sub(2) as u32;
                let inner_h = self.cellules_height.saturating_sub(2) as u32;
                let base_w = inner_w * 22 + 10;
                let base_h = inner_h * 22 + 70;
                if self.mode == GameMode::Angel && self.show_debug {
                    resize_window((base_w + 300).max(300), base_h.max(200));
                } else {
                    resize_window(base_w.max(300), base_h.max(200));
                }
                true
            }
            Msg::Preset(level) => {
                let (w, h, m) = match level {
                    0 => (9, 9, 10),      // Beginner
                    1 => (16, 16, 40),     // Intermediate
                    _ => (30, 16, 99),     // Expert
                };
                self.input_width = w.to_string();
                self.input_height = h.to_string();
                self.input_mines = m.to_string();
                self.show_settings = false;
                self.apply_settings();
                true
            }
            Msg::DownloadLogs => {
                if !self.failed_logs.is_empty() {
                    let mut content = String::new();
                    for (i, entry) in self.failed_logs.iter().enumerate() {
                        content.push_str(&format!("══ Failed attempt #{} ══\n{}\n\n", i + 1, entry));
                    }
                    download_text(&content, "angel_failed_logs.txt");
                }
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

        let game_over_overlay = match self.state {
            GameState::Over => html! {
                <div class="game-overlay game-overlay-loss">
                    <div class="game-overlay-text">{ "💥 Game Over" }</div>
                </div>
            },
            GameState::Won => html! {
                <div class="game-overlay game-overlay-win">
                    <div class="game-overlay-text">{ "🎉 You Win!" }</div>
                </div>
            },
            _ => html! {},
        };

        let new_game_icon = match self.mode {
            GameMode::Normal => "🧑",
            GameMode::Angel => "😇",
        };

        let mode_class = match self.mode {
            GameMode::Normal => "mode-button",
            GameMode::Angel => "mode-button mode-active",
        };

        let debug_panel = if self.mode == GameMode::Angel && self.show_debug {
            let mut log_text = self.debug_log.join("\n");
            if !self.failed_logs.is_empty() {
                log_text.push_str(&format!("\n\n═══ Failed attempts ({}) ═══", self.failed_logs.len()));
                for (i, entry) in self.failed_logs.iter().enumerate().rev() {
                    log_text.push_str(&format!("\n── #{} ──\n{}", i + 1, entry));
                }
            }
            let has_logs = !self.failed_logs.is_empty();
            html! {
                <div class="debug-panel">
                    <textarea class="debug-log" readonly=true value={log_text} rows="8" />
                    { if has_logs {
                        html! {
                            <button class="download-logs-button"
                                onclick={ctx.link().callback(|_| Msg::DownloadLogs)}>
                                { format!("📥 Download {} failed log(s)", self.failed_logs.len()) }
                            </button>
                        }
                    } else {
                        html! {}
                    }}
                </div>
            }
        } else {
            html! {}
        };

        let container_class = if self.mode == GameMode::Angel && self.show_debug {
            "game-container game-container-debug"
        } else {
            "game-container"
        };

        let settings_panel = if self.show_settings {
            let inner_w = self.cellules_width.saturating_sub(2);
            let inner_h = self.cellules_height.saturating_sub(2);
            let max_mines = (inner_w * inner_h).saturating_sub(9);
            let close_settings = ctx.link().callback(|_| Msg::ToggleSettings);
            html! {
                <>
                <div class="settings-backdrop" onclick={close_settings} />
                <div class="settings-panel">
                    <div class="settings-presets">
                        <button class="preset-button" onclick={ctx.link().callback(|_| Msg::Preset(0))}>{ "Small" }</button>
                        <button class="preset-button" onclick={ctx.link().callback(|_| Msg::Preset(1))}>{ "Medium" }</button>
                        <button class="preset-button" onclick={ctx.link().callback(|_| Msg::Preset(2))}>{ "Large" }</button>
                    </div>
                    <div class="settings-row">
                        <label>{ "Width" }</label>
                        <input type="number" min="4" max="40"
                            value={self.input_width.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| Msg::SetWidth(get_input_value(e)))} />
                    </div>
                    <div class="settings-row">
                        <label>{ "Height" }</label>
                        <input type="number" min="4" max="25"
                            value={self.input_height.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| Msg::SetHeight(get_input_value(e)))} />
                    </div>
                    <div class="settings-row">
                        <label>{ format!("Mines (max {})", max_mines) }</label>
                        <input type="number" min="1" max={max_mines.to_string()}
                            value={self.input_mines.clone()}
                            oninput={ctx.link().callback(|e: InputEvent| Msg::SetMines(get_input_value(e)))} />
                    </div>
                    <details class="settings-advanced">
                        <summary>{ "Advanced" }</summary>
                        <div class="settings-row">
                            <label>{ "Angel attempts" }</label>
                            <input type="number" min="1" max="100000"
                                value={self.input_angel_attempts.clone()}
                                oninput={ctx.link().callback(|e: InputEvent| Msg::SetAngelAttempts(get_input_value(e)))} />
                        </div>
                        <div class="settings-row">
                            <label>{ "Failed logs to keep" }</label>
                            <input type="number" min="0" max="1000"
                                value={self.input_max_failed_logs.clone()}
                                oninput={ctx.link().callback(|e: InputEvent| Msg::SetMaxFailedLogs(get_input_value(e)))} />
                        </div>
                        <div class="settings-row">
                            <label>{ "Show debug log" }</label>
                            <input type="checkbox"
                                checked={self.show_debug}
                                onchange={ctx.link().callback(|_| Msg::ToggleDebug)} />
                        </div>
                    </details>
                    <div class="settings-hint">{ "Applied on New Game" }</div>
                </div>
                </>
            }
        } else {
            html! {}
        };

        html! {
            <div>
                <section class={container_class}>
                    <div class="game-main">
                        <header class="app-header">
                            <div class="game-buttons">
                                <span class="mine-counter">{ "⚡ " }{ self.remaining_mines() }</span>
                                <button class="game-button" onclick={ctx.link().callback(|_| Msg::Start)}>{ new_game_icon }</button>
                                <button class={mode_class} onclick={ctx.link().callback(|_| Msg::ToggleMode)}>
                                    { "↔️" }
                                </button>
                                <button class="settings-button" onclick={ctx.link().callback(|_| Msg::ToggleSettings)}>{ "⚙️" }</button>
                            </div>
                        </header>
                        { settings_panel }
                        <section class="game-area">
                            <div class="game">
                                { for cell_rows }
                                { game_over_overlay }
                            </div>
                        </section>
                    </div>
                    { debug_panel }
                </section>
            </div>
        }
    }
}

fn get_input_value(e: InputEvent) -> String {
    e.target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
        .map(|input| input.value())
        .unwrap_or_default()
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

fn resize_window(width: u32, height: u32) {
    let js = format!(
        "try {{ \
            const w = window.__TAURI__.window; \
            w.appWindow.setSize(new w.LogicalSize({}, {})); \
        }} catch(e) {{ console.log('resize unavailable:', e); }}",
        width, height
    );
    let _ = js_sys::eval(&js);
}

fn save_settings(width: usize, height: usize, mines: usize, attempts: usize, mode: &GameMode) {
    let mode_str = match mode {
        GameMode::Angel => "angel",
        GameMode::Normal => "normal",
    };
    let js = format!(
        "try {{ localStorage.setItem('minesweeper_settings', \
        JSON.stringify({{width:{},height:{},mines:{},attempts:{},mode:'{}'}})); \
        }} catch(e) {{}}",
        width, height, mines, attempts, mode_str
    );
    let _ = js_sys::eval(&js);
}

/// Returns (width, height, mines, attempts, mode) or None
fn load_settings() -> Option<(usize, usize, usize, usize, GameMode)> {
    let js = "try { localStorage.getItem('minesweeper_settings') } catch(e) { null }";
    let val = js_sys::eval(js).ok()?;
    let json_str = val.as_string()?;
    // Parse JSON manually via js
    let parse_js = format!(
        "try {{ var s = JSON.parse('{}'); \
        [s.width, s.height, s.mines, s.attempts, s.mode].join(',') \
        }} catch(e) {{ null }}",
        json_str.replace('\'', "\\'")
    );
    let parsed = js_sys::eval(&parse_js).ok()?;
    let csv = parsed.as_string()?;
    let parts: Vec<&str> = csv.split(',').collect();
    if parts.len() != 5 { return None; }
    let w = parts[0].parse().ok()?;
    let h = parts[1].parse().ok()?;
    let m = parts[2].parse().ok()?;
    let a = parts[3].parse().ok()?;
    let mode = if parts[4] == "angel" { GameMode::Angel } else { GameMode::Normal };
    Some((w, h, m, a, mode))
}

fn save_failed_logs(logs: &[String]) {
    let escaped: Vec<String> = logs.iter()
        .map(|l| l.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n"))
        .collect();
    let js = format!(
        "try {{ localStorage.setItem('minesweeper_failed_logs', \
        JSON.stringify([{}])); }} catch(e) {{}}",
        escaped.iter().map(|l| format!("'{}'", l)).collect::<Vec<_>>().join(",")
    );
    let _ = js_sys::eval(&js);
}

fn load_failed_logs() -> Vec<String> {
    let js = "try { \
        var arr = JSON.parse(localStorage.getItem('minesweeper_failed_logs') || '[]'); \
        arr.join('\\n---LOGSEP---\\n'); \
    } catch(e) { '' }";
    if let Ok(val) = js_sys::eval(js) {
        if let Some(s) = val.as_string() {
            if s.is_empty() { return Vec::new(); }
            return s.split("\n---LOGSEP---\n").map(String::from).collect();
        }
    }
    Vec::new()
}

fn download_text(content: &str, filename: &str) {
    let escaped = content.replace('\\', "\\\\").replace('\'', "\\'").replace('\n', "\\n");
    let js = format!(
        "try {{ \
            var blob = new Blob(['{}'], {{type:'text/plain'}}); \
            var a = document.createElement('a'); \
            a.href = URL.createObjectURL(blob); \
            a.download = '{}'; \
            a.click(); \
            URL.revokeObjectURL(a.href); \
        }} catch(e) {{ console.log('download failed:', e); }}",
        escaped, filename
    );
    let _ = js_sys::eval(&js);
}

fn main() {
    wasm_logger::init(wasm_logger::Config::default());
    log::trace!("Initializing yew...");
    yew::start_app::<App>();
}

#[cfg(test)]
mod tests {
    use super::*;

    fn new_app(width: usize, height: usize) -> App {
        let inner_w = width.saturating_sub(2);
        let inner_h = height.saturating_sub(2);
        let mine_count = (inner_w * inner_h) / 5;
        let mut app = App {
            state: GameState::New,
            mode: GameMode::Normal,
            cellules: vec![Cellule::new_empty(); width * height],
            cellules_width: width,
            cellules_height: height,
            mine_count,
            angel_attempts: 1000,
            max_failed_logs: 10,
            show_settings: false,
            show_debug: false,
            chord_active: false,
            debug_log: Vec::new(),
            failed_logs: Vec::new(),
            input_width: inner_w.to_string(),
            input_height: inner_h.to_string(),
            input_mines: mine_count.to_string(),
            input_angel_attempts: "1000".to_string(),
            input_max_failed_logs: "10".to_string(),
        };
        app.reset();
        app
    }

    // --- wrap ---

    #[test]
    fn wrap_normal_values() {
        assert_eq!(wrap(0, 10), 0);
        assert_eq!(wrap(5, 10), 5);
        assert_eq!(wrap(9, 10), 9);
    }

    #[test]
    fn wrap_negative() {
        assert_eq!(wrap(-1, 10), 9);
        assert_eq!(wrap(-3, 10), 7);
    }

    #[test]
    fn wrap_overflow() {
        assert_eq!(wrap(10, 10), 0);
        assert_eq!(wrap(12, 10), 2);
    }

    // --- row_col_as_idx ---

    #[test]
    fn row_col_as_idx_basic() {
        let app = new_app(10, 8);
        assert_eq!(app.row_col_as_idx(0, 0), 0);
        assert_eq!(app.row_col_as_idx(0, 5), 5);
        assert_eq!(app.row_col_as_idx(2, 3), 23); // 2*10 + 3
    }

    #[test]
    fn row_col_as_idx_wraps() {
        let app = new_app(10, 8);
        // col wraps: -1 -> 9
        assert_eq!(app.row_col_as_idx(0, -1), 9);
        // row wraps: -1 -> 7
        assert_eq!(app.row_col_as_idx(-1, 0), 70); // 7*10
    }

    // --- reset ---

    #[test]
    fn reset_clears_inner_cells() {
        let mut app = new_app(6, 6);
        // Place mines in inner area
        let idx = app.row_col_as_idx(2, 2);
        app.cellules[idx].set_mine();
        app.cellules[idx].set_revealed();

        app.reset();

        // Inner cells should be hidden with val -2
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                assert!(app.cellules[i].is_hidden());
                assert_eq!(app.cellules[i].val, -2);
            }
        }
    }

    #[test]
    fn reset_preserves_border() {
        let app = new_app(6, 6);
        // Border cells (row 0) should remain Outside
        for col in 0..6 {
            assert_eq!(app.cellules[col].state, State::Outside);
        }
    }

    // --- neighbors / ref_neighbors ---

    #[test]
    fn ref_neighbors_returns_8_indices() {
        let app = new_app(10, 8);
        let refs = app.ref_neighbors(3, 3);
        assert_eq!(refs.len(), 8);
        // All should be unique
        let mut sorted = refs.to_vec();
        sorted.sort();
        sorted.dedup();
        assert_eq!(sorted.len(), 8);
    }

    #[test]
    fn neighbors_returns_correct_cells() {
        let mut app = new_app(10, 8);
        let mine_idx = app.row_col_as_idx(2, 3);
        app.cellules[mine_idx].set_mine();

        let nbrs = app.neighbors(2, 2);
        // (2,3) is a neighbor of (2,2), so exactly one mine
        let mine_count = nbrs.iter().filter(|c| c.is_mine()).count();
        assert_eq!(mine_count, 1);
    }

    // --- random_mutate ---

    #[test]
    fn random_mutate_clicked_cell_is_safe() {
        let mut app = new_app(10, 8);
        let click_idx = app.row_col_as_idx(3, 4);
        app.random_mutate(click_idx);

        assert!(!app.cellules[click_idx].is_mine());
    }

    #[test]
    fn random_mutate_neighbors_of_click_are_safe() {
        let mut app = new_app(10, 8);
        let click_idx = app.row_col_as_idx(3, 4);
        app.random_mutate(click_idx);

        for nbr_idx in app.ref_neighbors(3, 4) {
            assert!(!app.cellules[nbr_idx].is_mine());
        }
    }

    #[test]
    fn random_mutate_sets_neighbor_counts() {
        let mut app = new_app(10, 8);
        let click_idx = app.row_col_as_idx(3, 4);
        app.random_mutate(click_idx);

        // Every inner non-mine cell should have val in 0..=8
        for row in 1..7 {
            for col in 1..9 {
                let i = app.row_col_as_idx(row, col);
                let c = app.cellules[i];
                if !c.is_mine() {
                    assert!(c.val >= 0 && c.val <= 8, "val {} at ({},{})", c.val, row, col);
                }
            }
        }
    }

    // --- expand_zero ---

    #[test]
    fn expand_zero_reveals_connected_zeros() {
        // Build a small grid with no mines → all inner cells are zero
        let mut app = new_app(6, 6);
        // Reset sets all inner to val=-2, so manually set vals to 0
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].val = 0;
            }
        }

        let start = app.row_col_as_idx(2, 2);
        app.cellules[start].set_revealed();
        app.expand_zero(start);

        // All inner cells should be revealed
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                assert!(app.cellules[i].is_revealed(), "({},{}) not revealed", row, col);
            }
        }
    }

    #[test]
    fn expand_zero_stops_at_numbered_cells() {
        let mut app = new_app(6, 6);
        // Set all inner to 0
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].val = 0;
            }
        }
        // Place a wall of numbered cells at col=3
        for row in 1..5 {
            let i = app.row_col_as_idx(row, 3);
            app.cellules[i].val = 1;
        }

        let start = app.row_col_as_idx(2, 1);
        app.cellules[start].set_revealed();
        app.expand_zero(start);

        // Cells at col=1,2 should be revealed; col=3 revealed (numbered stops further expansion)
        // but col=4 should remain hidden
        for row in 1..5 {
            let i = app.row_col_as_idx(row, 4);
            assert!(app.cellules[i].is_hidden(), "({},4) should still be hidden", row);
        }
    }

    #[test]
    fn expand_zero_does_not_reveal_marked() {
        let mut app = new_app(6, 6);
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].val = 0;
            }
        }
        // Mark a neighbor
        let marked_idx = app.row_col_as_idx(2, 3);
        app.cellules[marked_idx].set_marked();

        let start = app.row_col_as_idx(2, 2);
        app.cellules[start].set_revealed();
        app.expand_zero(start);

        assert!(app.cellules[marked_idx].is_marked(), "marked cell should stay marked");
    }

    // --- GameState transitions ---

    #[test]
    fn initial_state_is_new() {
        let app = new_app(10, 8);
        assert_eq!(app.state, GameState::New);
    }

    // --- chord ---

    /// Helper: set up a 6x6 grid in InProgress state with a specific layout.
    /// Places a mine at (2,3), sets neighbor counts, reveals cell (2,2).
    fn setup_chord_grid() -> App {
        let mut app = new_app(6, 6);
        app.state = GameState::InProgress;
        // Clear all inner cells to 0
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].val = 0;
            }
        }
        // Place one mine at (2,3)
        let mine_idx = app.row_col_as_idx(2, 3);
        app.cellules[mine_idx].set_mine();
        // Recalculate neighbor counts for inner cells
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                if !app.cellules[i].is_mine() {
                    let neighbors = app.neighbors(row as isize, col as isize);
                    app.cellules[i].set_value(&neighbors);
                }
            }
        }
        app
    }

    #[test]
    fn chord_reveals_hidden_neighbors_when_marks_match() {
        let mut app = setup_chord_grid();
        // (2,2) has val=1 (one mine neighbor at (2,3))
        let center = app.row_col_as_idx(2, 2);
        app.cellules[center].set_revealed();
        // Mark the mine
        let mine = app.row_col_as_idx(2, 3);
        app.cellules[mine].set_marked();

        let changed = app.chord(center);
        assert!(changed);
        // All hidden neighbors of (2,2) should now be revealed
        for &ni in &app.ref_neighbors(2, 2) {
            let c = app.cellules[ni];
            assert!(c.is_revealed() || c.is_marked() || c.state == State::Outside,
                "neighbor at idx {} should be revealed or marked", ni);
        }
    }

    #[test]
    fn chord_noop_when_marks_dont_match() {
        let mut app = setup_chord_grid();
        let center = app.row_col_as_idx(2, 2);
        app.cellules[center].set_revealed();
        // Don't mark anything — marked_count=0, cell val=1
        let changed = app.chord(center);
        assert!(!changed);
        // Neighbors should remain hidden
        for &ni in &app.ref_neighbors(2, 2) {
            let c = app.cellules[ni];
            assert!(c.is_hidden() || c.state == State::Outside || c.is_mine(),
                "neighbor should still be hidden");
        }
    }

    #[test]
    fn chord_noop_on_hidden_cell() {
        let mut app = setup_chord_grid();
        let center = app.row_col_as_idx(2, 2);
        // Cell is hidden (not revealed)
        assert!(!app.chord(center));
    }

    #[test]
    fn chord_noop_on_zero_cell() {
        let mut app = setup_chord_grid();
        // (1,1) has val=0 (no mine neighbors)
        let center = app.row_col_as_idx(1, 1);
        app.cellules[center].set_revealed();
        assert!(!app.chord(center));
    }

    #[test]
    fn chord_triggers_game_over_on_wrong_mark() {
        let mut app = setup_chord_grid();
        // (2,2) has val=1. Mark a non-mine neighbor instead of the actual mine.
        let center = app.row_col_as_idx(2, 2);
        app.cellules[center].set_revealed();
        let wrong = app.row_col_as_idx(2, 1);
        app.cellules[wrong].set_marked();

        app.chord(center);
        // The real mine at (2,3) gets revealed → game over
        assert_eq!(app.state, GameState::Over);
    }

    #[test]
    fn chord_expands_zero_neighbors() {
        let mut app = new_app(6, 6);
        app.state = GameState::InProgress;
        // All inner cells = 0 except place one mine at (1,4)
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].val = 0;
            }
        }
        let mine = app.row_col_as_idx(1, 4);
        app.cellules[mine].set_mine();
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                if !app.cellules[i].is_mine() {
                    let neighbors = app.neighbors(row as isize, col as isize);
                    app.cellules[i].set_value(&neighbors);
                }
            }
        }
        // (1,3) should have val=1. Reveal it and mark the mine.
        let center = app.row_col_as_idx(1, 3);
        app.cellules[center].set_revealed();
        app.cellules[mine].set_marked();

        app.chord(center);
        // (1,2) is a zero neighbor of (1,3), expand_zero should reveal connected zeros
        let zero_cell = app.row_col_as_idx(1, 2);
        assert!(app.cellules[zero_cell].is_revealed());
    }

    // --- angel mode ---

    /// Helper: set up an angel-mode grid with known mines and some revealed cells.
    fn setup_angel_grid() -> App {
        let mut app = new_app(6, 6);
        app.state = GameState::InProgress;
        app.mode = GameMode::Angel;
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].val = 0;
            }
        }
        // Place mine at (3,3)
        let mine = app.row_col_as_idx(3, 3);
        app.cellules[mine].set_mine();
        // Recalculate
        for row in 1..5 {
            for col in 1..5 {
                let i = app.row_col_as_idx(row, col);
                if !app.cellules[i].is_mine() {
                    let neighbors = app.neighbors(row as isize, col as isize);
                    app.cellules[i].set_value(&neighbors);
                }
            }
        }
        // Reveal (1,1) — far from mine, val=0
        let revealed = app.row_col_as_idx(1, 1);
        app.cellules[revealed].set_revealed();
        app
    }

    #[test]
    fn angel_relocate_moves_mine_away() {
        let mut app = setup_angel_grid();
        let mine_idx = app.row_col_as_idx(3, 3);
        assert!(app.cellules[mine_idx].is_mine());

        let result = app.angel_relocate(&[mine_idx]);
        assert!(result);
        assert!(!app.cellules[mine_idx].is_mine());

        // Total mine count preserved
        let mine_count = app.cellules.iter().filter(|c| c.is_mine()).count();
        assert_eq!(mine_count, 1);
    }

    #[test]
    fn angel_relocate_preserves_revealed_values() {
        let mut app = setup_angel_grid();
        let revealed_idx = app.row_col_as_idx(1, 1);
        let original_val = app.cellules[revealed_idx].val;

        let mine_idx = app.row_col_as_idx(3, 3);
        app.angel_relocate(&[mine_idx]);

        assert_eq!(app.cellules[revealed_idx].val, original_val);
        assert!(app.cellules[revealed_idx].is_revealed());
    }

    #[test]
    fn angel_relocate_fails_when_impossible() {
        // Grid with only 1 non-revealed inner cell that isn't the clicked cell
        let mut app = new_app(4, 4);
        app.state = GameState::InProgress;
        app.mode = GameMode::Angel;
        // 4x4 grid: inner cells are (1,1), (1,2), (2,1), (2,2)
        for row in 1..3 {
            for col in 1..3 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].val = 0;
            }
        }
        // Make all inner cells mines
        for row in 1..3 {
            for col in 1..3 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].set_mine();
            }
        }
        // Click on (1,1) — all 4 cells are mines, can't place 4 mines in 3 candidates
        let click = app.row_col_as_idx(1, 1);
        let result = app.angel_relocate(&[click]);
        assert!(!result);
    }

    #[test]
    fn angel_relocate_restores_on_failure() {
        let mut app = new_app(4, 4);
        app.state = GameState::InProgress;
        app.mode = GameMode::Angel;
        for row in 1..3 {
            for col in 1..3 {
                let i = app.row_col_as_idx(row, col);
                app.cellules[i].set_mine();
            }
        }
        let original: Vec<i8> = app.cellules.iter().map(|c| c.val).collect();
        let click = app.row_col_as_idx(1, 1);
        app.angel_relocate(&[click]);
        let after: Vec<i8> = app.cellules.iter().map(|c| c.val).collect();
        assert_eq!(original, after);
    }
}
