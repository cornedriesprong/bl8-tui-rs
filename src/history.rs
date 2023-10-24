use crate::engine::{Note, Parameters, State, Track, INITIAL_STEP_COUNT, SEQ_TRACK_COUNT};
use crossbeam::channel::*;
use regex::Regex;

pub type Grid = Vec<Vec<String>>;

pub struct History {
    history: Vec<Grid>,
    pos: usize,
    pub channel: (Sender<State>, Receiver<State>),
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
        let state = Self::to_state(grid.clone());
        self.channel.0.send(state).unwrap();
        self.history.truncate(self.pos + 1);
        self.history.push(grid);
        self.pos += 1;
    }

    pub fn undo(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }

        let grid = self.history[self.pos].clone();
        let state = Self::to_state(grid.clone());
        self.channel.0.send(state).unwrap();
    }

    pub fn redo(&mut self) {
        if self.pos < self.history.len() - 1 {
            self.pos += 1;
        }

        let grid = self.history[self.pos].clone();
        let state = Self::to_state(grid.clone());
        self.channel.0.send(state).unwrap();
    }

    pub fn to_state(grid: Grid) -> State {
        grid.iter()
            .map(|track| Track {
                notes: track
                    .iter()
                    .enumerate()
                    .map(|(index, cell)| History::parse_input(cell, index))
                    .collect::<Vec<Option<Note>>>()
                    .try_into()
                    .unwrap(),
            })
            .collect::<Vec<Track>>()
            .try_into()
            .unwrap()
    }

    fn parse_input(input: &String, note_index: usize) -> Option<Note> {
        let re = Regex::new(r"\d").unwrap();
        if re.is_match(&input) {
            if let Ok(pitch) = input.parse::<i8>() {
                Some(Note {
                    timestamp: note_index as f32,
                    pitch,
                    velocity: 100,
                    parameters: Parameters::new(),
                })
            } else {
                None
            }
        } else {
            None
        }
    }
}
