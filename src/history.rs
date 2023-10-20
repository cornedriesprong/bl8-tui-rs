use crate::engine::SEQ_TRACK_COUNT;
use crossbeam::channel::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Parameters {
    pub engine: Option<f32>,
    pub harmonics: Option<f32>,
    pub morph: Option<f32>,
    pub timbre: Option<f32>,
}

impl Parameters {
    fn new() -> Parameters {
        Parameters {
            engine: None,
            harmonics: None,
            morph: None,
            timbre: None,
        }
    }
}

#[derive(Clone, Copy, PartialEq, Debug)]
pub struct Note {
    pub timestamp: f32,
    pub pitch: i8,
    pub velocity: i8,
    pub parameters: Parameters,
}

impl Note {
    pub fn new(timestamp: f32, pitch: i8, velocity: i8) -> Note {
        Note {
            timestamp,
            pitch,
            velocity,
            parameters: Parameters::new(),
        }
    }
}

#[derive(Clone, Debug)]
pub struct Track {
    pub notes: [Option<Note>; 16],
}

pub type State = [Track; SEQ_TRACK_COUNT];

pub struct History {
    history: Vec<State>,
    pos: usize,
    pub channel: (Sender<State>, Receiver<State>),
}

impl History {
    pub fn new() -> History {
        History {
            history: vec![[
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
                Track { notes: [None; 16] },
            ]],
            pos: 0,
            channel: crossbeam::channel::unbounded(),
        }
    }

    pub fn get_state(&self) -> &State {
        &self.history[self.pos]
    }

    pub fn push(&mut self, state: State) {
        self.channel.0.send(state.clone()).unwrap();
        self.history.truncate(self.pos + 1);
        self.history.push(state);
        self.pos += 1;
    }

    pub fn undo(&mut self) {
        if self.pos > 0 {
            self.pos -= 1;
        }

        let state = self.history[self.pos].clone();
        self.channel.0.send(state).unwrap();
    }

    pub fn redo(&mut self) {
        if self.pos < self.history.len() - 1 {
            self.pos += 1;
        }

        let state = self.history[self.pos].clone();
        self.channel.0.send(state).unwrap();
    }
}
