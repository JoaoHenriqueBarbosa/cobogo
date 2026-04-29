mod ui;

use crossterm::{
    event::{self, Event, KeyCode, KeyEventKind},
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::stdout;

use clay_renderer_ratatui::ClayRatatuiRenderer;

pub struct App {
    pub sidebar_visible: bool,
    pub active_tab: usize,
    pub selected_item: usize,
    pub status_message: String,
    pub counter: u32,
    pub theme_dark: bool,
    pub should_quit: bool,
}

impl App {
    fn new() -> Self {
        Self {
            sidebar_visible: true,
            active_tab: 0,
            selected_item: 0,
            status_message: "Press ? for help".into(),
            counter: 0,
            theme_dark: true,
            should_quit: false,
        }
    }

    fn handle_key(&mut self, code: KeyCode) {
        match code {
            KeyCode::Char('q') => self.should_quit = true,
            KeyCode::Char('s') => {
                self.sidebar_visible = !self.sidebar_visible;
                self.status_message = if self.sidebar_visible {
                    "Sidebar shown".into()
                } else {
                    "Sidebar hidden".into()
                };
            }
            KeyCode::Tab => {
                self.active_tab = (self.active_tab + 1) % 3;
                self.status_message = format!("Switched to tab {}", self.active_tab + 1);
            }
            KeyCode::Up => {
                if self.selected_item > 0 {
                    self.selected_item -= 1;
                }
                self.status_message = format!("Selected item {}", self.selected_item);
            }
            KeyCode::Down => {
                if self.selected_item < 5 {
                    self.selected_item += 1;
                }
                self.status_message = format!("Selected item {}", self.selected_item);
            }
            KeyCode::Enter => {
                self.counter += 1;
                self.status_message = format!("Action! Counter: {}", self.counter);
            }
            KeyCode::Char('t') => {
                self.theme_dark = !self.theme_dark;
                self.status_message = if self.theme_dark {
                    "Dark theme".into()
                } else {
                    "Light theme".into()
                };
            }
            KeyCode::Char('?') => {
                self.status_message =
                    "q:quit s:sidebar Tab:tabs ↑↓:nav Enter:action t:theme".into();
            }
            _ => {}
        }
    }
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let mut renderer = ClayRatatuiRenderer::new();

    loop {
        terminal.draw(|frame| {
            let size = frame.area();
            let commands = ui::build_layout(&app, size.width as f32, size.height as f32);
            renderer.render(&commands, frame.buffer_mut());
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            if let Event::Key(key) = event::read()? {
                if key.kind == KeyEventKind::Press {
                    app.handle_key(key.code);
                }
            }
        }

        if app.should_quit {
            break;
        }
    }

    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
