use crossterm::{
    cursor,
    event::{poll, read, Event, KeyCode},
    queue,
    terminal::{self, disable_raw_mode, enable_raw_mode},
};
use log::LevelFilter;
use simplelog::*;
use std::{
    fmt::Display,
    fs::File,
    io::{stdout, Result, Write},
    time::Duration,
};

const CELL_WIDTH: usize = 4;
const CELL_HEIGHT: usize = 1;

#[derive(Clone, Copy)]
enum Mode {
    Normal,
    Insert,
    Command,
}

#[derive(Debug)]
struct Transaction {
    cmds: Vec<Command>,
}

impl Display for Transaction {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut s = String::from("");
        for cmd in &self.cmds {
            s.push_str(&format!("{}\n", cmd));
        }
        write!(f, "{}", s)
    }
}

#[derive(Debug, Clone)]
enum Command {
    Insert { x: usize, y: usize, ch: char },
}

impl Display for Command {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Command::Insert { x, y, ch } => write!(f, "Insert({}, {}, {})", x, y, ch),
        }
    }
}

struct App {
    x: usize,
    y: usize,
    grid: Vec<Vec<String>>,
    mode: Mode,
    input_line: String,
    history: Vec<Transaction>,
    history_pos: usize,
    current_transaction: Option<Transaction>,
    exit: bool,
}

impl App {
    fn new() -> App {
        App {
            x: 0,
            y: 0,
            grid: vec![vec!["___ ".to_string(); 8]; 16],
            mode: Mode::Normal,
            input_line: String::from(""),
            history: Vec::new(),
            history_pos: 0,
            current_transaction: None,
            exit: false,
        }
    }

    fn update_input_line(&self) -> Result<()> {
        let mut stdout = stdout();
        queue!(stdout, cursor::MoveTo(0, self.grid.len() as u16))?;
        print!("{}", self.input_line);
        queue!(stdout, cursor::MoveTo(self.x as u16, self.y as u16))?;
        Ok(())
    }

    fn draw(&self) -> Result<()> {
        let mut stdout = stdout();

        // clear the terminal
        queue!(stdout, terminal::Clear(terminal::ClearType::All))?;

        for (y, row) in self.grid.iter().enumerate() {
            for (x, cell) in row.iter().enumerate() {
                let x = x * CELL_WIDTH;
                queue!(stdout, cursor::MoveTo(x as u16, y as u16))?;
                print!("{}", cell);
            }
        }

        queue!(stdout, cursor::MoveTo(self.x as u16, self.y as u16))?;

        match self.mode {
            Mode::Normal => {
                queue!(stdout, cursor::SetCursorStyle::SteadyBlock).unwrap();
            }
            Mode::Insert => {
                queue!(stdout, cursor::SetCursorStyle::SteadyBar).unwrap();
                self.update_input_line()?;
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
                (Mode::Normal, KeyCode::Char('h')) => {
                    self.x -= CELL_WIDTH;
                }
                (Mode::Normal, KeyCode::Char('j')) => {
                    self.y += CELL_HEIGHT;
                }
                (Mode::Normal, KeyCode::Char('k')) => {
                    self.y -= CELL_HEIGHT;
                }
                (Mode::Normal, KeyCode::Char('l')) => {
                    self.x += CELL_WIDTH;
                }
                (Mode::Normal, KeyCode::Char('u')) => {
                    self.undo();
                }
                (Mode::Normal, KeyCode::Char('r')) => {
                    self.redo();
                }
                (Mode::Normal, KeyCode::Char('i')) => {
                    self.mode = Mode::Insert;
                    self.input_line = "-- INSERT --".to_string();
                    self.current_transaction = Transaction { cmds: Vec::new() }.into();
                }
                (Mode::Normal, KeyCode::Char('x')) => {
                    self.grid[self.y][self.x / CELL_WIDTH] = "___ ".to_string();
                }
                (Mode::Normal, KeyCode::Char(':')) => {
                    self.input_line = ":".to_string();
                    self.mode = Mode::Command;
                }
                (Mode::Command, KeyCode::Enter) => {
                    if self.input_line.trim() == ":q" {
                        self.exit = true
                    }
                    self.input_line = String::from("");
                    self.mode = Mode::Normal;
                }
                (Mode::Command, KeyCode::Char(c)) => {
                    self.input_line.push(c);
                }
                (Mode::Command, KeyCode::Backspace) => {
                    self.input_line.pop();
                }
                (Mode::Command, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                }
                (Mode::Insert, KeyCode::Char(ch)) => {
                    let cmd = Command::Insert {
                        x: self.x / CELL_WIDTH,
                        y: self.y,
                        ch,
                    };
                    self.apply(cmd);
                    self.x += 1;
                }
                (Mode::Insert, KeyCode::Esc) => {
                    self.mode = Mode::Normal;
                    self.close_transaction();
                    // align x to grid
                    self.x = (self.x / CELL_WIDTH) * CELL_WIDTH;
                }
                (_, _) => {}
            },
            _ => (),
        }
    }

    fn apply(&mut self, cmd: Command) {
        match cmd {
            Command::Insert { x, y, ch } => {
                App::append_to_cell(&mut self.grid[y][x], ch);
            }
        }
        self.current_transaction.as_mut().unwrap().cmds.push(cmd);
    }

    fn undo(&mut self) {
        if self.history_pos > 0 {
            // reverse the command
            self.history_pos -= 1;
            let transaction = &self.history[self.history_pos];

            for cmd in &transaction.cmds {
                match cmd {
                    Command::Insert { x, y, ch: _ } => self.grid[*y][*x] = "___ ".to_string(),
                }
            }
        }
    }

    fn close_transaction(&mut self) {
        self.history.push(self.current_transaction.take().unwrap());
        self.history_pos += 1;
    }

    fn redo(&mut self) {
        if self.history_pos < self.history.len() {
            let transaction = &self.history[self.history_pos];
            self.history_pos += 1;

            // re-apply the command
            // TODO: get rid of this duplication?
            for cmd in &transaction.cmds {
                match cmd {
                    Command::Insert { x, y, ch } => {
                        App::append_to_cell(&mut self.grid[*y][*x], *ch);
                    }
                }
            }
        }
    }

    fn append_to_cell(s: &mut String, ch: char) {
        if let Some(i) = s.chars().position(|c| c == '_') {
            s.replace_range(i..i + 1, &ch.to_string());
        }
    }

    fn run(&mut self) -> Result<()> {
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

fn main() -> Result<()> {
    // init logging
    let log_file = File::create("log.txt").unwrap();
    WriteLogger::init(LevelFilter::Info, Config::default(), log_file).unwrap();

    let mut app = App::new();
    app.run()?;
    Ok(())
}
