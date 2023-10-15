use crate::app::{Command, Note};

pub struct History {
    hist: Vec<Command>,
    pos: usize,
}

impl History {
    pub fn new() -> History {
        History {
            hist: Vec::new(),
            pos: 0,
        }
    }

    pub fn push(&mut self, cmd: Command) {
        self.hist.push(cmd);
        self.pos += 1;
    }

    pub fn undo(&mut self, grid: &mut Vec<Vec<Option<Note>>>) {
        if self.pos > 0 {
            self.pos -= 1;
            let cmd = &self.hist[self.pos];

            match cmd {
                Command::Insert { x, y, note: _ } => grid[*y][*x] = None,
            }
        }
    }

    pub fn redo(&mut self, grid: &mut Vec<Vec<Option<Note>>>) {
        if self.pos < self.hist.len() {
            let cmd = &self.hist[self.pos];
            self.pos += 1;

            match cmd {
                Command::Insert { x, y, note } => grid[*y][*x] = Some(*note),
            }
        }
    }
}
