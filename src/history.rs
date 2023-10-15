use crate::app::Note;

pub type State = Vec<Vec<Option<Note>>>;

pub struct History {
    history: Vec<State>,
    pos: usize,
}

impl History {
    pub fn new() -> History {
        History {
            history: vec![vec![vec![None; 8]; 16]],
            pos: 0,
        }
    }

    pub fn get_state(&self) -> &State {
        &self.history[self.pos]
    }

    pub fn push(&mut self, state: State) {
        self.history.truncate(self.pos + 1);
        self.history.push(state);
        self.pos += 1;
    }

    pub fn undo(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }
    }

    pub fn redo(&mut self) {
        if self.pos < self.history.len() - 1 {
            self.pos += 1;
        }
    }
}
