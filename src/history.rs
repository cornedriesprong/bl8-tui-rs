use crate::engine::{INITIAL_STEP_COUNT, SEQ_TRACK_COUNT};
use crossbeam::channel::*;

pub type Grid = Vec<Vec<String>>;

pub struct History {
    history: Vec<Grid>,
    pos: usize,
    pub channel: (Sender<Grid>, Receiver<Grid>),
}

impl History {
    pub fn new() -> History {
        History {
            history: vec![vec![
                vec!["___ ".to_string(); INITIAL_STEP_COUNT];
                SEQ_TRACK_COUNT
            ]],
            pos: 0,
            channel: crossbeam::channel::unbounded(),
        }
    }

    pub fn get_grid(&self) -> &Grid {
        &self.history[self.pos]
    }

    pub fn push(&mut self, grid: Grid) {
        self.channel.0.send(grid.clone()).unwrap();
        self.history.truncate(self.pos + 1);
        self.history.push(grid);
        self.pos += 1;
    }

    pub fn undo(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }

        let grid = self.history[self.pos].clone();
        self.channel.0.send(grid).unwrap();
    }

    pub fn redo(&mut self) {
        if self.pos < self.history.len() - 1 {
            self.pos += 1;
        }

        let state = self.history[self.pos].clone();
        self.channel.0.send(state).unwrap();
    }
}
