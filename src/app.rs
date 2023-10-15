use std::{
    io::{stdout, Result, Write},
    time::Duration,
};

use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    queue,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};

use crate::history::{History, State};

const CELL_WIDTH: usize = 4;
const CELL_HEIGHT: usize = 1;

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    Command,
}

#[derive(Clone)]
pub enum Command {
    Insert { x: usize, y: usize, note: Note },
    Delete { x: usize, y: usize },
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct Note {
    pitch: i8,
}

pub struct App {
    x: usize,
    y: usize,
    mode: Mode,
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
            mode: Mode::Normal,
            cmd_line: String::from(""),
            curr_input: vec![],
            history: History::new(),
            exit: false,
        }
    }

    fn get_state(&self) -> &State {
        self.history.get_state()
    }

    fn update_input_line(&self) -> Result<()> {
        let mut stdout = stdout();
        queue!(stdout, cursor::MoveTo(0, self.get_state().len() as u16))?;
        print!("{}", self.cmd_line);
        queue!(stdout, cursor::MoveTo(self.x as u16, self.y as u16))?;
        Ok(())
    }

    fn align_cursor_to_grid(&mut self) {
        self.x = (self.x / CELL_WIDTH) * CELL_WIDTH;
    }

    fn draw(&self) -> Result<()> {
        let mut stdout = stdout();

        // clear the terminal
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;

        for (y, row) in self.get_state().iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let x = x * CELL_WIDTH;
                queue!(stdout, cursor::MoveTo(x as u16, y as u16))?;
                if let Some(note) = cell {
                    print!("{}", note.pitch);
                } else {
                    print!("___ ");
                }
            }
        }

        queue!(stdout, cursor::MoveTo(self.x as u16, self.y as u16))?;

        match self.mode {
            Mode::Normal => {
                queue!(stdout, cursor::SetCursorStyle::SteadyBlock).unwrap();
            }
            Mode::Command => {
                queue!(stdout, cursor::SetCursorStyle::SteadyBar).unwrap();
                self.update_input_line()?;
            }
        }

        stdout.flush()?;
        Ok(())
    }

    fn process_key(&mut self, key: Event) {
        match key {
            Event::Key(event) => match (self.mode, event.code) {
                (Mode::Normal, KeyCode::Char(ch)) => {
                    if ch.is_numeric() {
                        self.curr_input.push(ch);

                        // log::info!("x {:?}", self.x);
                        // log::info!("y {:?}", self.y);

                        if let Ok(pitch) = self
                            .curr_input
                            .clone()
                            .into_iter()
                            .collect::<String>()
                            .parse::<i8>()
                        {
                            let cmd = Command::Insert {
                                x: self.x / CELL_WIDTH,
                                y: self.y,
                                note: Note { pitch },
                            };

                            self.apply(cmd);
                            self.x += 1;
                        }
                    } else {
                        self.align_cursor_to_grid();
                        self.curr_input.clear();
                        match ch {
                            'h' => {
                                if self.x > 0 {
                                    self.x -= CELL_WIDTH;
                                } else {
                                    self.x =
                                        self.get_state()[self.y].len() * CELL_WIDTH - CELL_WIDTH;
                                }
                            }
                            'j' => {
                                if self.y + CELL_HEIGHT < self.get_state().len() {
                                    self.y += CELL_HEIGHT;
                                } else {
                                    self.y = 0;
                                }
                            }
                            'k' => {
                                if self.y > 0 {
                                    self.y -= CELL_HEIGHT;
                                } else {
                                    self.y = self.get_state().len() - CELL_HEIGHT;
                                }
                            }
                            'l' => {
                                if self.x + CELL_WIDTH < self.get_state()[self.y].len() * CELL_WIDTH
                                {
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
                                self.mode = Mode::Command;
                            }
                            _ => {}
                        }
                    }
                }
                (Mode::Command, KeyCode::Enter) => {
                    if self.cmd_line.trim() == ":q" {
                        self.exit = true
                    }
                    self.cmd_line = String::from("");
                    self.mode = Mode::Normal;
                }
                (Mode::Command, KeyCode::Char(c)) => {
                    self.cmd_line.push(c);
                }
                (Mode::Command, KeyCode::Backspace) => {
                    self.cmd_line.pop();
                }
                (Mode::Command, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                }
                (_, _) => {}
            },
            _ => (),
        }
    }

    fn apply(&mut self, cmd: Command) {
        let mut state = self.get_state().clone();
        match cmd {
            Command::Insert { x, y, note } => {
                state[y][x] = Some(note);
            }
            Command::Delete { x, y } => {
                state[y][x] = None;
            }
        }
        self.history.push(state);
    }

    pub fn run(&mut self) -> Result<()> {
        enable_raw_mode()?;
        let mut stdout = stdout();
        terminal::enable_raw_mode()?;

        self.draw()?;

        loop {
            if poll(Duration::from_millis(100))? {
                let evt = read()?;
                self.process_key(evt);
                self.draw()?;
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
