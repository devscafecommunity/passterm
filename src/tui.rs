use super::vault::storage;
use super::vault::Vault;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::prelude::Stylize;
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::Color,
    widgets::{Block, Borders, List, ListItem, Paragraph},
    Frame, Terminal,
};
use std::io;

pub struct App {
    pub vault: Option<Vault>,
    pub password: String,
    pub entries: Vec<String>,
    pub selected: usize,
    pub show_password_input: bool,
    pub password_input: String,
    pub adding_entry: bool,
    pub new_entry_id: String,
    pub new_vars: Vec<(String, String)>,
    pub current_var: usize,
    pub show_secret: bool,
    pub secret_entry: String,
    pub secret_index: usize,
}

impl App {
    pub fn new() -> Self {
        Self {
            vault: None,
            password: String::new(),
            entries: Vec::new(),
            selected: 0,
            show_password_input: false,
            password_input: String::new(),
            adding_entry: false,
            new_entry_id: String::new(),
            new_vars: Vec::new(),
            current_var: 0,
            show_secret: false,
            secret_entry: String::new(),
            secret_index: 0,
        }
    }

    pub fn unlock(&mut self) -> bool {
        match storage::load_vault(&self.password) {
            Ok(v) => {
                self.entries = v.entries.keys().cloned().collect();
                self.vault = Some(v);
                true
            }
            Err(_) => false,
        }
    }

    pub fn save_entry(&mut self) {
        if let Some(ref mut v) = self.vault {
            if !self.new_entry_id.is_empty() {
                let vars: std::collections::HashMap<String, String> = self
                    .new_vars
                    .iter()
                    .filter(|(k, _)| !k.is_empty())
                    .map(|(k, v)| (k.clone(), v.clone()))
                    .collect();
                v.add_entry(self.new_entry_id.clone(), vars);
                let _ = storage::save_vault(v, &self.password);
                self.entries = v.entries.keys().cloned().collect();
            }
        }
    }
}

pub fn run() -> io::Result<()> {
    enable_raw_mode()?;
    let mut stdout = io::stdout();
    execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
    let backend = CrosstermBackend::new(stdout);
    let mut terminal = Terminal::new(backend)?;

    let mut app = App::new();
    let mut should_quit = false;

    loop {
        terminal.draw(|f| ui(f, &mut app))?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    handle_key(key.code, &mut app, &mut should_quit);
                }
            }
        }

        if should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    execute!(
        terminal.backend_mut(),
        LeaveAlternateScreen,
        DisableMouseCapture
    )?;
    terminal.show_cursor()?;

    Ok(())
}

fn handle_key(code: KeyCode, app: &mut App, should_quit: &mut bool) {
    match code {
        KeyCode::Char('q') if !app.adding_entry && !app.show_password_input && !app.show_secret => {
            *should_quit = true;
        }
        KeyCode::Esc => {
            app.show_password_input = false;
            app.password_input.clear();
            app.adding_entry = false;
            app.new_entry_id.clear();
            app.new_vars.clear();
            app.show_secret = false;
        }
        KeyCode::Down => {
            if app.show_secret {
                if let Some(v) = app
                    .vault
                    .as_ref()
                    .and_then(|v| v.entries.get(&app.secret_entry))
                {
                    let total = v.variables.len();
                    if app.secret_index < total.saturating_sub(1) {
                        app.secret_index += 1;
                    }
                }
            } else if app.adding_entry {
                if app.current_var < app.new_vars.len().saturating_sub(1) {
                    app.current_var += 1;
                }
            } else if !app.entries.is_empty() && app.selected < app.entries.len() - 1 {
                app.selected += 1;
            }
        }
        KeyCode::Up => {
            if app.show_secret {
                app.secret_index = app.secret_index.saturating_sub(1);
            } else if app.adding_entry {
                app.current_var = app.current_var.saturating_sub(1);
            } else if app.selected > 0 {
                app.selected -= 1;
            }
        }
        KeyCode::Enter => {
            if app.vault.is_none() {
                if !app.show_password_input {
                    app.show_password_input = true;
                } else if !app.password_input.is_empty() {
                    app.password = app.password_input.clone();
                    if app.unlock() {
                        app.show_password_input = false;
                        app.password_input.clear();
                    } else {
                        app.password_input.clear();
                    }
                }
            } else if app.adding_entry {
                app.save_entry();
                app.adding_entry = false;
                app.new_entry_id.clear();
                app.new_vars.clear();
            } else if !app.entries.is_empty() && !app.show_secret {
                app.secret_entry = app.entries[app.selected].clone();
                app.secret_index = 0;
                app.show_secret = true;
            }
        }
        KeyCode::Char('a') => {
            if app.vault.is_some() && !app.adding_entry && !app.show_secret {
                app.adding_entry = true;
                app.new_entry_id.clear();
                app.new_vars = vec![(String::new(), String::new())];
                app.current_var = 0;
            }
        }
        KeyCode::Char('d') => {
            if app.vault.is_some()
                && !app.adding_entry
                && !app.show_secret
                && !app.entries.is_empty()
            {
                let entry_id = app.entries.remove(app.selected);
                if let Some(ref mut v) = app.vault {
                    v.remove_entry(&entry_id);
                    let _ = storage::save_vault(v, &app.password);
                }
                if app.selected > 0 && app.selected >= app.entries.len() {
                    app.selected = app.entries.len().saturating_sub(1);
                }
            }
        }
        KeyCode::Char('c') => {
            use std::process::Command as ProcCommand;
            if app.show_secret {
                if let Some(v) = app
                    .vault
                    .as_ref()
                    .and_then(|v| v.entries.get(&app.secret_entry))
                {
                    if let Some((_, val)) = v.variables.iter().nth(app.secret_index) {
                        let _ = ProcCommand::new("sh")
                            .args([
                                "-c",
                                &format!("echo '{}' | xclip -selection clipboard", val),
                            ])
                            .output();
                    }
                }
            }
        }
        KeyCode::Char('n') if app.adding_entry => {
            app.new_vars.push((String::new(), String::new()));
        }
        KeyCode::Backspace if app.show_password_input && !app.password_input.is_empty() => {
            app.password_input.pop();
        }
        KeyCode::Char(c) => {
            if app.show_password_input {
                app.password_input.push(c);
            } else if app.adding_entry {
                let idx = app.current_var;
                let field = if idx % 2 == 0 { 0 } else { 1 };
                if let Some((ref mut k, ref mut v)) = app.new_vars.get_mut(idx) {
                    if field == 0 {
                        k.push(c);
                    } else {
                        v.push(c);
                    }
                }
            }
        }
        _ => {}
    }
}

fn ui(f: &mut Frame, app: &mut App) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),
            Constraint::Min(0),
            Constraint::Length(3),
        ])
        .split(f.area());

    let title = if app.vault.is_none() {
        " Passterm - Locked "
    } else if app.adding_entry {
        " Passterm - Add Entry "
    } else if app.show_secret {
        " Passterm - View Secret "
    } else {
        " Passterm "
    };

    let style = ratatui::style::Style::default().fg(Color::Cyan).bold();

    f.render_widget(
        Paragraph::new(title)
            .style(style)
            .block(Block::default().borders(Borders::ALL).title(" passterm ")),
        chunks[0],
    );

    if app.vault.is_none() {
        if app.show_password_input {
            let masked: String = app.password_input.chars().map(|_| '*').collect();
            f.render_widget(
                Paragraph::new(format!("Password: {}", masked)).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(" Enter Master Password "),
                ),
                chunks[1],
            );
        } else {
            f.render_widget(
                Paragraph::new("Press ENTER to unlock vault")
                    .block(Block::default().borders(Borders::ALL)),
                chunks[1],
            );
        }
    } else if app.adding_entry {
        let input_text = if app.new_entry_id.is_empty() {
            "Entry ID: _"
        } else {
            app.new_entry_id.as_str()
        };
        f.render_widget(
            Paragraph::new(input_text)
                .fg(Color::Yellow)
                .block(Block::default().borders(Borders::ALL).title(" Entry ID ")),
            chunks[1],
        );
    } else if app.show_secret {
        let entry_id = &app.secret_entry;
        if let Some(v) = app.vault.as_ref().and_then(|v| v.entries.get(entry_id)) {
            let items: Vec<ListItem> = v
                .variables
                .iter()
                .enumerate()
                .map(|(i, (k, val))| {
                    let marker = if i == app.secret_index { ">" } else { " " };
                    let shown = if i == app.secret_index {
                        val.as_str()
                    } else {
                        "*****"
                    };
                    ListItem::new(format!("{} {} = {}", marker, k, shown))
                })
                .collect();

            f.render_widget(
                List::new(items).block(
                    Block::default()
                        .borders(Borders::ALL)
                        .title(format!(" {} ", entry_id)),
                ),
                chunks[1],
            );
        }
    } else if app.entries.is_empty() {
        f.render_widget(
            Paragraph::new("No entries. Press 'a' to add one.")
                .fg(Color::DarkGray)
                .block(Block::default().borders(Borders::ALL)),
            chunks[1],
        );
    } else {
        let items: Vec<ListItem> = app
            .entries
            .iter()
            .enumerate()
            .map(|(i, e)| {
                let marker = if i == app.selected { ">" } else { " " };
                ListItem::new(format!("{} {}", marker, e))
            })
            .collect();

        f.render_widget(
            List::new(items).block(
                Block::default()
                    .borders(Borders::ALL)
                    .title(" Environments "),
            ),
            chunks[1],
        );
    }

    let help = if app.vault.is_none() {
        "[Enter] unlock"
    } else if app.adding_entry {
        "[n] new var | [Enter] save | [Esc] cancel"
    } else if app.show_secret {
        "[c] copy value | [Enter] next | [Esc] back"
    } else {
        "[Enter] view | [a] add | [d] delete | [q] quit"
    };

    f.render_widget(
        Paragraph::new(help)
            .fg(Color::DarkGray)
            .block(Block::default().borders(Borders::ALL)),
        chunks[2],
    );
}
