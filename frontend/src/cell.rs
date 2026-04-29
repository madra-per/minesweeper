#[derive(Clone, Copy, Debug, PartialEq, Eq)]
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

    pub fn get_visual(self) -> String {
        if self.is_marked(){
            return String::from("⚡")
        }
        if !self.is_revealed(){
            return String::from("")
        }
        if self.is_zero(){
            return String::from("")
        }
        if self.is_mine(){
            return String::from("❌")
        }
        return self.val.to_string()
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

#[cfg(test)]
mod tests {
    use super::*;

    fn hidden_cell(val: i8) -> Cellule {
        Cellule { state: State::Hidden, val }
    }

    fn mine_cell() -> Cellule {
        Cellule { state: State::Hidden, val: -1 }
    }

    // --- Construction & initial state ---

    #[test]
    fn new_empty_is_outside_with_reset_val() {
        let c = Cellule::new_empty();
        assert_eq!(c.state, State::Outside);
        assert_eq!(c.val, -2);
    }

    // --- State transitions ---

    #[test]
    fn set_revealed() {
        let mut c = hidden_cell(0);
        c.set_revealed();
        assert!(c.is_revealed());
    }

    #[test]
    fn set_hidden() {
        let mut c = Cellule { state: State::Revealed, val: 0 };
        c.set_hidden();
        assert!(c.is_hidden());
    }

    #[test]
    fn set_marked() {
        let mut c = hidden_cell(0);
        c.set_marked();
        assert!(c.is_marked());
    }

    #[test]
    fn set_mine() {
        let mut c = hidden_cell(0);
        c.set_mine();
        assert!(c.is_mine());
        assert_eq!(c.val, -1);
    }

    #[test]
    fn reset_clears_val_and_sets_hidden() {
        let mut c = Cellule { state: State::Revealed, val: 3 };
        c.reset();
        assert_eq!(c.val, -2);
        assert!(c.is_hidden());
    }

    // --- Predicates ---

    #[test]
    fn is_zero() {
        assert!(hidden_cell(0).is_zero());
        assert!(!hidden_cell(1).is_zero());
        assert!(!mine_cell().is_zero());
    }

    #[test]
    fn is_mine() {
        assert!(mine_cell().is_mine());
        assert!(!hidden_cell(0).is_mine());
    }

    // --- toggle & toggle_marked ---

    #[test]
    fn toggle_reveals_hidden_cell() {
        let mut c = hidden_cell(3);
        c.toggle();
        assert!(c.is_revealed());
    }

    #[test]
    fn toggle_does_nothing_if_already_revealed() {
        let mut c = Cellule { state: State::Revealed, val: 3 };
        c.toggle();
        assert!(c.is_revealed());
    }

    #[test]
    fn toggle_marked_from_hidden_to_marked() {
        let mut c = hidden_cell(0);
        c.toggle_marked();
        assert!(c.is_marked());
    }

    #[test]
    fn toggle_marked_from_marked_to_hidden() {
        let mut c = Cellule { state: State::Marked, val: 0 };
        c.toggle_marked();
        assert!(c.is_hidden());
    }

    #[test]
    fn toggle_marked_noop_on_revealed() {
        let mut c = Cellule { state: State::Revealed, val: 0 };
        c.toggle_marked();
        assert!(c.is_revealed());
    }

    // --- set_value & count_neighbor_mines ---

    #[test]
    fn count_neighbor_mines_all_safe() {
        let neighbors = [hidden_cell(0); 8];
        assert_eq!(Cellule::count_neighbor_mines(&neighbors), 0);
    }

    #[test]
    fn count_neighbor_mines_some_mines() {
        let mut neighbors = [hidden_cell(0); 8];
        neighbors[0] = mine_cell();
        neighbors[3] = mine_cell();
        neighbors[7] = mine_cell();
        assert_eq!(Cellule::count_neighbor_mines(&neighbors), 3);
    }

    #[test]
    fn count_neighbor_mines_all_mines() {
        let neighbors = [mine_cell(); 8];
        assert_eq!(Cellule::count_neighbor_mines(&neighbors), 8);
    }

    #[test]
    fn set_value_counts_neighbors() {
        let mut c = hidden_cell(0);
        let mut neighbors = [hidden_cell(0); 8];
        neighbors[1] = mine_cell();
        neighbors[5] = mine_cell();
        c.set_value(&neighbors);
        assert_eq!(c.val, 2);
    }

    #[test]
    fn set_value_skipped_for_mine() {
        let mut c = mine_cell();
        let neighbors = [hidden_cell(0); 8];
        c.set_value(&neighbors);
        assert!(c.is_mine()); // val unchanged
    }

    // --- get_visual ---

    #[test]
    fn visual_hidden_is_empty() {
        assert_eq!(hidden_cell(3).get_visual(), "");
    }

    #[test]
    fn visual_marked_is_flag() {
        let c = Cellule { state: State::Marked, val: 3 };
        assert_eq!(c.get_visual(), "⚡");
    }

    #[test]
    fn visual_revealed_zero_is_empty() {
        let c = Cellule { state: State::Revealed, val: 0 };
        assert_eq!(c.get_visual(), "");
    }

    #[test]
    fn visual_revealed_mine_is_x() {
        let c = Cellule { state: State::Revealed, val: -1 };
        assert_eq!(c.get_visual(), "❌");
    }

    #[test]
    fn visual_revealed_number() {
        for n in 1..=8i8 {
            let c = Cellule { state: State::Revealed, val: n };
            assert_eq!(c.get_visual(), n.to_string());
        }
    }
}
