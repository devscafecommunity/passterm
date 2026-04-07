use super::vault::storage;
use super::vault::Vault;
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout},
    style::{Color, Style},
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
    pub secret_key: String,
    pub secret_value: String,
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
            secret_key: String::new(),
            secret_value: String::new(),
        }
    }

    pub fn unlock(&mut self) -> bool {
        match storage::load_vault(&self.password) {
            Ok(vault) => {
                self.entries = vault.entries.keys().cloned().collect();
                self.vault = Some(vault);
                true
            }
            Err(_) => false,
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

        if event::poll(std::time::Duration::from_millis(16))? {
            let evt = event::read()?;
            if let Event::Key(key) = evt {
                if key.kind == KeyEventKind::Press {
                    handle_key(key.code, key.modifiers, &mut app, &mut should_quit);
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

fn handle_key(
    code: KeyCode,
    _modifiers: event::KeyModifiers,
    app: &mut App,
    should_quit: &mut bool,
) {
    match code {
        KeyCode::Char('q') if !app.adding_entry && !app.show_password_input && !app.show_secret => {
            *should_quit = true;
        }
        KeyCode::Esc if app.show_password_input => {
            app.show_password_input = false;
            app.password_input.clear();
        }
        KeyCode::Esc if app.adding_entry => {
            app.adding_entry = false;
            app.new_entry_id.clear();
            app.new_vars.clear();
        }
        KeyCode::Esc if app.show_secret => {
            app.show_secret = false;
            app.secret_key.clear();
            app.secret_value.clear();
        }
        KeyCode::Down => {
            if app.entries.is_empty() {
                return;
            }
            if app.adding_entry {
                if app.current_var < app.new_vars.len().saturating_sub(1) {
                    app.current_var += 1;
                }
            } else if app.selected < app.entries.len() - 1 {
                app.selected += 1;
            }
        }
        KeyCode::Up => {
            if app.entries.is_empty() {
                return;
            }
            if app.adding_entry {
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
            } else if !app.entries.is_empty() && !app.adding_entry {
                let entry_id = app.entries[app.selected].clone();
                let vault_ref = app.vault.as_ref().unwrap();
                if let Some(entry) = vault_ref.entries.get(&entry_id) {
                    app.show_secret = true;
                    if let Some((key, val)) = entry.variables.iter().next() {
                        app.secret_key = key.clone();
                        app.secret_value = val.clone();
                    }
                }
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
        KeyCode::Char('n') if app.adding_entry => {
            app.new_vars.push((String::new(), String::new()));
        }
        KeyCode::Char('d') if app.vault.is_some() && !app.adding_entry => {
            if !app.entries.is_empty() {
                let entry_id = app.entries.remove(app.selected);
                if let Some(ref mut v) = app.vault {
                    v.remove_entry(&entry_id);
                    let _ = storage::save_vault(v, &app.password);
                }
                if app.selected > 0 && app.selected >= app.entries.len() {
                    app.selected = app.entries.len() - 1;
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
        " Passterm - Main "
    };

    f.render_widget(
        Paragraph::new(title)
            .style(Style::default().fg(Color::Cyan))
            .block(Block::default().borders(Borders::ALL).title(".pass")),
        chunks[0],
    );

    if app.vault.is_none() {
        if app.show_password_input {
            let pw: String = app.password_input.chars().map(|_| '*').collect();
            f.render_widget(
                Paragraph::new(format!("Password: {}", pw))
                    .style(Style::default().fg(Color::White))
                    .block(
                        Block::default()
                            .borders(Borders::ALL)
                            .title("Enter Password"),
                    ),
                chunks[1],
            );
        } else {
            f.render_widget(
                Paragraph::new("Press Enter to unlock (enter password)")
                    .style(Style::default().fg(Color::White))
                    .block(Block::default().borders(Borders::ALL)),
                chunks[1],
            );
        }
    } else if app.adding_entry {
        let items: Vec<ListItem> = app
            .new_vars
            .iter()
            .enumerate()
            .map(|(i, (k, v))| {
                let marker = if i == 0 { ">" } else { " " };
                ListItem::new(format!("{} {} = {}", marker, k, v))
            })
            .collect();

        f.render_widget(
            List::new(items)
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Add Secrets")),
            chunks[1],
        );
    } else if app.show_secret {
        f.render_widget(
            Paragraph::new(format!("{} = {}", app.secret_key, app.secret_value))
                .style(Style::default().fg(Color::Yellow))
                .block(Block::default().borders(Borders::ALL).title("Secret")),
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
            List::new(items)
                .style(Style::default().fg(Color::White))
                .block(Block::default().borders(Borders::ALL).title("Environments")),
            chunks[1],
        );
    }

    let help = if app.vault.is_none() {
        "[Enter] unlock"
    } else if app.adding_entry {
        "[Enter] save | [n] new var | [Esc] cancel"
    } else if app.show_secret {
        "[Esc] close"
    } else {
        "[Enter] view | [a] add | [d] delete | [q] quit"
    };

    f.render_widget(
        Paragraph::new(help)
            .style(Style::default().fg(Color::DarkGray))
            .block(Block::default().borders(Borders::ALL)),
        chunks[2],
    );
}
