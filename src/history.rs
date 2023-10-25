use crate::engine::{Note, Parameters, State, Track, INITIAL_STEP_COUNT, SEQ_TRACK_COUNT};
use crossbeam::channel::*;
use regex::Regex;
use std::collections::HashMap;

pub const PITCHES: [&str; 11] = ["C", "C#", "D", "D#", "E", "F", "F#", "G", "G#", "A", "B"];
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
        if let Some(index) = Self::parse_pitch(input) {
            return Some(Note {
                timestamp: note_index as f32,
                pitch: index as i8,
                velocity: 100,
                parameters: Parameters::new(),
            });
        } else if re.is_match(&input) {
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

    fn get_pitch(input: &str, len: usize, pitch_map: &HashMap<String, i32>) -> Option<i32> {
        if input.len() >= len && input[0..len].chars().all(|c| c.is_alphabetic() || c == '#') {
            let note = input[0..len].to_uppercase();
            let octave = input[len..].parse::<i32>().unwrap_or(2);
            return pitch_map.get(&note).map(|n| n + octave * 12);
        }
        None
    }

    fn parse_pitch(input: &str) -> Option<i32> {
        let mut pitch_map = HashMap::new();
        for (index, pitch) in PITCHES.iter().enumerate() {
            pitch_map.insert(pitch.to_string(), (index + 12) as i32);
        }

        Self::get_pitch(input, 2, &pitch_map)
            .or_else(|| Self::get_pitch(input, 1, &pitch_map))
            .or_else(|| Self::get_pitch(input, 0, &pitch_map))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_input() {
        assert_eq!(
            History::parse_input(&"C0".to_string(), 0),
            Some(Note {
                timestamp: 0.0,
                pitch: 12,
                velocity: 100,
                parameters: Parameters::new(),
            })
        );
        assert_eq!(
            History::parse_input(&"C#0".to_string(), 1),
            Some(Note {
                timestamp: 1.0,
                pitch: 13,
                velocity: 100,
                parameters: Parameters::new(),
            })
        );
        assert_eq!(
            History::parse_input(&"C1".to_string(), 1),
            Some(Note {
                timestamp: 1.0,
                pitch: 24,
                velocity: 100,
                parameters: Parameters::new(),
            })
        );
        assert_eq!(
            History::parse_input(&"C".to_string(), 0),
            Some(Note {
                timestamp: 0.0,
                pitch: 36,
                velocity: 100,
                parameters: Parameters::new(),
            })
        );
        assert_eq!(
            History::parse_input(&"D".to_string(), 0),
            Some(Note {
                timestamp: 0.0,
                pitch: 38,
                velocity: 100,
                parameters: Parameters::new(),
            })
        );
    }
}
