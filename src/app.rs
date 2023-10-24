use cpal::traits::{DeviceTrait, HostTrait, StreamTrait};
use crossbeam::channel::*;
use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    queue,
    style::{self, Stylize},
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use std::{
    io::{stdout, Result, Write},
    time::Duration,
};

use crate::engine::Engine;
use crate::history::{Grid, History};

pub const SAMPLE_RATE: f32 = 48000.0;
const CELL_WIDTH: usize = 4;
const CELL_HEIGHT: usize = 1;

#[derive(Clone, Copy)]
enum EditingMode {
    Normal,
    Insert,
    Command,
}

#[derive(Clone)]
pub enum Command {
    Insert { x: usize, y: usize, input: String },
    Delete { x: usize, y: usize },
}

pub struct App {
    x: usize,
    y: usize,
    active_step: i8,
    mode: EditingMode,
    cmd_line: String,
    curr_input: Vec<char>,
    history: History,
    exit: bool,
}

impl App {
    pub fn new() -> App {
        App {
            x: 0,
            y: 0,
            active_step: 0,
            mode: EditingMode::Normal,
            cmd_line: String::from(""),
            curr_input: vec![],
            history: History::new(),
            exit: false,
        }
    }

    fn get_grid(&self) -> &Grid {
        self.history.get_grid()
    }

    fn update_input_line(&self) -> Result<()> {
        let mut stdout = stdout();
        queue!(stdout, cursor::MoveTo(0, 16))?;
        print!("{}", self.cmd_line);
        queue!(stdout, cursor::MoveTo(self.x as u16, self.y as u16))?;
        Ok(())
    }

    fn align_cursor_to_grid(&mut self) {
        self.x = (self.x / CELL_WIDTH) * CELL_WIDTH;
    }

    fn draw(&mut self) -> Result<()> {
        let mut stdout = stdout();

        // clear the terminal
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;

        for (x, track) in self.get_grid().iter().enumerate() {
            for (y, cell) in track.iter().enumerate() {
                let x = x * CELL_WIDTH;
                queue!(stdout, cursor::MoveTo(x as u16, y as u16))?;
                print!("{}", cell);
            }

            for i in 0..CELL_WIDTH {
                let x = x * CELL_WIDTH + i;
                queue!(
                    stdout,
                    cursor::MoveTo(x as u16, self.active_step as u16),
                    style::PrintStyledContent("░".dark_magenta())
                )?;
            }
        }

        queue!(stdout, cursor::MoveTo(self.x as u16, self.y as u16))?;

        match self.mode {
            EditingMode::Normal => {
                queue!(stdout, cursor::SetCursorStyle::SteadyBlock).unwrap();
                self.cmd_line = "-- NORMAL --".to_string();
            }
            EditingMode::Insert => {
                queue!(stdout, cursor::SetCursorStyle::SteadyBar).unwrap();
                self.cmd_line = "-- INSERT --".to_string();
            }
            EditingMode::Command => {
                queue!(stdout, cursor::SetCursorStyle::SteadyBar).unwrap();
            }
        }
        self.update_input_line()?;

        stdout.flush()?;
        Ok(())
    }

    fn process_key(&mut self, key: Event) {
        match key {
            Event::Key(event) => match (self.mode, event.code) {
                (EditingMode::Normal, KeyCode::Char(ch)) => {
                    self.align_cursor_to_grid();
                    self.curr_input.clear();
                    match ch {
                        'h' => {
                            if self.x > 0 {
                                self.x -= CELL_WIDTH;
                            } else {
                                self.x = self.get_grid().len() * CELL_WIDTH - CELL_WIDTH;
                            }
                        }
                        'j' => {
                            if self.y + CELL_HEIGHT < self.get_grid()[self.x / CELL_WIDTH].len() {
                                self.y += CELL_HEIGHT;
                            } else {
                                self.y = 0;
                            }
                        }
                        'k' => {
                            if self.y > 0 {
                                self.y -= CELL_HEIGHT;
                            } else {
                                self.y = self.get_grid()[self.x / CELL_WIDTH].len() - CELL_HEIGHT;
                            }
                        }
                        'l' => {
                            if self.x + CELL_WIDTH < self.get_grid().len() * CELL_WIDTH {
                                self.x += CELL_WIDTH;
                            } else {
                                self.x = 0;
                            }
                        }
                        'u' => {
                            self.history.undo();
                        }
                        'r' => {
                            self.history.redo();
                        }
                        'x' => {
                            let cmd = Command::Delete {
                                x: self.x / CELL_WIDTH,
                                y: self.y,
                            };
                            self.apply(cmd);
                        }
                        ':' => {
                            self.cmd_line = ":".to_string();
                            self.mode = EditingMode::Command;
                        }
                        'i' => {
                            self.mode = EditingMode::Insert;
                        }
                        _ => {}
                    }
                }
                (EditingMode::Insert, KeyCode::Char(ch)) => {
                    self.curr_input.push(ch);

                    self.update_selected_cell();
                    self.x += 1;
                }
                (EditingMode::Insert, KeyCode::Esc) => {
                    self.mode = EditingMode::Normal;
                }
                (EditingMode::Command, KeyCode::Enter) => {
                    if self.cmd_line.trim() == ":q" {
                        self.exit = true
                    }
                    self.cmd_line = String::from("");
                    self.mode = EditingMode::Normal;
                }
                (EditingMode::Command, KeyCode::Char(c)) => {
                    self.cmd_line.push(c);
                }
                (EditingMode::Command, KeyCode::Backspace) => {
                    self.cmd_line.pop();
                }
                (_, _) => {}
            },
            _ => (),
        }
    }

    fn update_selected_cell(&mut self) {
        let input = self.curr_input.clone().into_iter().collect::<String>();
        let cmd = Command::Insert {
            x: self.x / CELL_WIDTH,
            y: self.y,
            input: input,
        };

        self.apply(cmd);
    }

    fn apply(&mut self, cmd: Command) {
        let mut state = self.get_grid().clone();
        match cmd {
            Command::Insert { x, y, input } => state[x][y] = input,
            Command::Delete { x, y } => state[x][y] = "___ ".to_string(),
        }

        self.history.push(state);
    }

    pub fn run(&mut self) -> anyhow::Result<()> {
        let mut engine = Engine::new();
        engine.init();

        let host = cpal::default_host();
        let device = host
            .default_output_device()
            .expect("Failed to get default output device");
        let config = device.default_output_config().unwrap();

        let channels = config.channels() as usize;

        let (_, rx) = &self.history.channel;
        let rx = rx.clone();

        let (_, ui_rx) = &engine.ui_channel;
        let ui_rx = ui_rx.clone();

        let err_fn = |err| eprintln!("an error occurred on stream: {}", err);

        let stream = device.build_output_stream(
            &config.into(),
            move |data: &mut [f32], _: &cpal::OutputCallbackInfo| {
                // update state if needed
                if let Ok(grid) = rx.try_recv() {
                    engine.set_state(grid);
                }
                for frame in data.chunks_mut(channels) {
                    for sample in frame.iter_mut() {
                        *sample = engine.tick();
                    }
                }
            },
            err_fn,
            None,
        )?;
        stream.play()?;

        self.draw_ui(ui_rx)?;

        Ok(())
    }

    fn draw_ui(&mut self, rx: Receiver<i8>) -> anyhow::Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;

        loop {
            rx.try_recv().map(|s| self.active_step = s).ok();
            self.draw()?;

            if poll(Duration::from_millis(10))? {
                let evt = read()?;
                self.process_key(evt);
                if self.exit {
                    break;
                }
            }
        }

        disable_raw_mode()?;
        queue!(stdout, cursor::Show)?;

        Ok(())
    }
}
