#[derive(Clone, Copy, PartialEq, Eq)]
pub enum State {
    Hidden,
    Revealed,
    Marked,
    Outside,
}

#[derive(Clone, Copy)]
pub struct Cellule {
    pub state: State,
    pub val: i8,
}

impl Cellule {
    pub fn new_empty() -> Self {
        Self { state: State::Outside, val: -2 }
    }

    pub fn set_revealed(&mut self) {
        self.state = State::Revealed;
    }

    pub fn set_hidden(&mut self) {
        self.state = State::Hidden;
    }

    pub fn set_marked(&mut self) {
        self.state = State::Marked;
    }

    pub fn set_mine(&mut self) {
        self.val = -1;
    }

    pub fn reset(&mut self) {
        self.val = -2;
        self.state = State::Hidden;
    }

    pub fn set_value(&mut self, neighbors: &[Self]) {
        if !self.is_mine() {
            self.val = Self::count_neighbor_mines(neighbors);
        }
    }

    pub fn is_marked(self) -> bool {
        self.state == State::Marked
    }

    pub fn is_hidden(self) -> bool {
        self.state == State::Hidden
    }

    pub fn is_revealed(self) -> bool {
        self.state == State::Revealed
    }

    pub fn is_mine(self) -> bool {
        self.val == -1
    }

    pub fn is_zero(self) -> bool {
        self.val == 0
    }

    pub fn toggle_marked(&mut self) {
        if self.is_marked(){
            self.set_hidden();
        }
        else if self.is_hidden() {
            self.set_marked();
        }
    }

    pub fn toggle(&mut self) {
        if self.is_hidden(){
            self.set_revealed();
        }
    }

    pub fn count_neighbor_mines(neighbors: &[Self]) -> i8 {
        neighbors.iter().filter(|n| n.is_mine()).count().try_into().unwrap()
    }
}