use crate::engine::{Note, Params, State, Track, INITIAL_STEP_COUNT, SEQ_TRACK_COUNT};
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
                SEQ_TRACK_COUNT * 3
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
    pub fn to_state(grid: Grid) -> [Track; 8] {
        grid.chunks(3)
            .map(|g| Track {
                notes: g[0]
                    .iter()
                    .enumerate()
                    .map(|(step, _)| {
                        let cells =
                            vec![g[0][step].clone(), g[1][step].clone(), g[2][step].clone()];
                        History::parse_input(&cells, step)
                    })
                    .collect::<Vec<Option<Note>>>()
                    .try_into()
                    .unwrap(),
            })
            .collect::<Vec<Track>>()
            .try_into()
            .unwrap()
    }

    fn parse_input(input: &Vec<String>, note_index: usize) -> Option<Note> {
        let re = Regex::new(r"\d").unwrap();
        if let Some(idx) = Self::parse_pitch(input[0].as_str()) {
            return Some(Note {
                timestamp: note_index as f32,
                pitch: idx as i8,
                velocity: 100,
                parameters: {
                    let mut params = Params::new();
                    if let Ok(harmonics) = input[1].parse::<i8>() {
                        params.harmonics = Some(harmonics as f32 / 100.0);
                    }
                    if let Ok(timbre) = input[2].parse::<i8>() {
                        params.timbre = Some(timbre as f32 / 100.0);
                    }
                    params
                },
            });
        } else if re.is_match(&input[0]) {
            if let Ok(pitch) = input[0].parse::<i8>() {
                Some(Note {
                    timestamp: note_index as f32,
                    pitch,
                    velocity: 100,
                    parameters: {
                        let mut params = Params::new();
                        if let Ok(harmonics) = input[1].parse::<i8>() {
                            params.harmonics = Some(harmonics as f32 / 100.0);
                        }
                        if let Ok(timbre) = input[2].parse::<i8>() {
                            params.timbre = Some(timbre as f32 / 100.0);
                        }
                        params
                    },
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
        for (idx, pitch) in PITCHES.iter().enumerate() {
            pitch_map.insert(pitch.to_string(), (idx + 12) as i32);
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
            History::parse_input(
                &vec!["C0".to_string(), "50".to_string(), "50".to_string()],
                0
            ),
            Some(Note {
                timestamp: 0.0,
                pitch: 12,
                velocity: 100,
                parameters: Params {
                    engine: None,
                    harmonics: Some(0.5),
                    morph: Some(0.5),
                    timbre: None,
                }
            })
        );
        assert_eq!(
            History::parse_input(
                &vec!["C#0".to_string(), "50".to_string(), "50".to_string()],
                1
            ),
            Some(Note {
                timestamp: 1.0,
                pitch: 13,
                velocity: 100,
                parameters: Params {
                    engine: None,
                    harmonics: Some(0.5),
                    morph: Some(0.5),
                    timbre: None,
                }
            })
        );
        assert_eq!(
            History::parse_input(
                &vec!["C1".to_string(), "50".to_string(), "50".to_string()],
                1
            ),
            Some(Note {
                timestamp: 1.0,
                pitch: 24,
                velocity: 100,
                parameters: Params {
                    engine: None,
                    harmonics: Some(0.5),
                    morph: Some(0.5),
                    timbre: None,
                }
            })
        );
        assert_eq!(
            History::parse_input(
                &vec!["C".to_string(), "50".to_string(), "50".to_string()],
                0
            ),
            Some(Note {
                timestamp: 0.0,
                pitch: 36,
                velocity: 100,
                parameters: Params {
                    engine: None,
                    harmonics: Some(0.5),
                    morph: Some(0.5),
                    timbre: None,
                }
            })
        );
        assert_eq!(
            History::parse_input(
                &vec!["D".to_string(), "50".to_string(), "50".to_string()],
                0
            ),
            Some(Note {
                timestamp: 0.0,
                pitch: 38,
                velocity: 100,
                parameters: Params {
                    engine: None,
                    harmonics: Some(0.5),
                    morph: Some(0.5),
                    timbre: None,
                }
            })
        );
    }
}
