mod ui;

use crossterm::{
    event::{
        self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode, KeyEventKind, MouseButton,
        MouseEventKind,
    },
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
    ExecutableCommand,
};
use ratatui::prelude::*;
use std::io::stdout;

use clay::types::*;
use clay_renderer_ratatui::ClayRatatuiRenderer;

pub struct App {
    pub sidebar_visible: bool,
    pub active_tab: usize,
    pub selected_item: usize,
    pub status_message: String,
    pub counter: u32,
    pub theme_dark: bool,
    pub should_quit: bool,
    pub hover_element: String,
    pub mouse_pos: (u16, u16),
}

impl App {
    fn new() -> Self {
        Self {
            sidebar_visible: true,
            active_tab: 0,
            selected_item: 0,
            status_message: "Press ? for help │ Mouse enabled".into(),
            counter: 0,
            theme_dark: true,
            should_quit: false,
            hover_element: String::new(),
            mouse_pos: (0, 0),
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
                    "q:quit s:sidebar Tab:tabs ↑↓:nav Enter:action t:theme │ Mouse enabled".into();
            }
            _ => {}
        }
    }

    fn handle_mouse_click(&mut self, ctx: &clay::context::Context, x: u16, y: u16) {
        let pos = Vector2::new(x as f32, y as f32);

        // Check tabs
        let tab_names = ["Dashboard", "Settings", "About"];
        for (i, _) in tab_names.iter().enumerate() {
            let tab_id = ctx.get_element_id_with_index("Tab", i as u32);
            let data = ctx.get_element_data(&tab_id);
            if data.found && point_in_bbox(pos, &data.bounding_box) {
                self.active_tab = i;
                self.status_message = format!("Clicked tab: {}", tab_names[i]);
                return;
            }
        }

        // Check sidebar items
        let sidebar_items = ["Overview", "Analytics", "Reports", "Users", "Config", "Logs"];
        for (i, name) in sidebar_items.iter().enumerate() {
            let item_id = ctx.get_element_id_with_index("SidebarItem", i as u32);
            let data = ctx.get_element_data(&item_id);
            if data.found && point_in_bbox(pos, &data.bounding_box) {
                self.selected_item = i;
                self.status_message = format!("Clicked: {}", name);
                return;
            }
        }

        // Check stat cards
        for i in 0..4u32 {
            let card_id = ctx.get_element_id_with_index("StatCard", i);
            let data = ctx.get_element_data(&card_id);
            if data.found && point_in_bbox(pos, &data.bounding_box) {
                self.counter += 1;
                self.status_message = format!("Clicked card {} │ Counter: {}", i + 1, self.counter);
                return;
            }
        }

        self.status_message = format!("Click at ({}, {})", x, y);
    }

    fn handle_mouse_move(&mut self, ctx: &clay::context::Context, x: u16, y: u16) {
        self.mouse_pos = (x, y);
        let pos = Vector2::new(x as f32, y as f32);

        // Check hover on tabs
        let tab_names = ["Dashboard", "Settings", "About"];
        for (i, name) in tab_names.iter().enumerate() {
            let tab_id = ctx.get_element_id_with_index("Tab", i as u32);
            let data = ctx.get_element_data(&tab_id);
            if data.found && point_in_bbox(pos, &data.bounding_box) {
                self.hover_element = format!("Tab:{}", name);
                return;
            }
        }

        // Check hover on sidebar items
        let sidebar_items = ["Overview", "Analytics", "Reports", "Users", "Config", "Logs"];
        for (i, name) in sidebar_items.iter().enumerate() {
            let item_id = ctx.get_element_id_with_index("SidebarItem", i as u32);
            let data = ctx.get_element_data(&item_id);
            if data.found && point_in_bbox(pos, &data.bounding_box) {
                self.hover_element = format!("Sidebar:{}", name);
                return;
            }
        }

        // Check named regions
        for region in &["Header", "Sidebar", "Content", "Footer"] {
            let id = ctx.get_element_id(region);
            let data = ctx.get_element_data(&id);
            if data.found && point_in_bbox(pos, &data.bounding_box) {
                self.hover_element = region.to_string();
                return;
            }
        }

        self.hover_element.clear();
    }

    fn handle_scroll(&mut self, _ctx: &clay::context::Context, _x: u16, _y: u16, delta: i16) {
        if delta > 0 && self.selected_item > 0 {
            self.selected_item -= 1;
        } else if delta < 0 && self.selected_item < 5 {
            self.selected_item += 1;
        }
        self.status_message = format!("Scroll → item {}", self.selected_item);
    }
}

fn point_in_bbox(pos: Vector2, bbox: &BoundingBox) -> bool {
    pos.x >= bbox.x
        && pos.x < bbox.x + bbox.width
        && pos.y >= bbox.y
        && pos.y < bbox.y + bbox.height
}

fn main() -> std::io::Result<()> {
    enable_raw_mode()?;
    stdout().execute(EnterAlternateScreen)?;
    stdout().execute(EnableMouseCapture)?;
    let mut terminal = Terminal::new(CrosstermBackend::new(stdout()))?;

    let mut app = App::new();
    let mut renderer = ClayRatatuiRenderer::new();
    let mut last_layout: Option<clay::context::Context> = None;

    loop {
        terminal.draw(|frame| {
            let size = frame.area();
            let result = ui::build_layout(&app, size.width as f32, size.height as f32);
            renderer.render(&result.commands, frame.buffer_mut());
            last_layout = Some(result.ctx);
        })?;

        if event::poll(std::time::Duration::from_millis(50))? {
            match event::read()? {
                Event::Key(key) => {
                    if key.kind == KeyEventKind::Press {
                        app.handle_key(key.code);
                    }
                }
                Event::Mouse(mouse) => {
                    if let Some(ref ctx) = last_layout {
                        match mouse.kind {
                            MouseEventKind::Down(MouseButton::Left) => {
                                app.handle_mouse_click(ctx, mouse.column, mouse.row);
                            }
                            MouseEventKind::Moved => {
                                app.handle_mouse_move(ctx, mouse.column, mouse.row);
                            }
                            MouseEventKind::ScrollUp => {
                                app.handle_scroll(ctx, mouse.column, mouse.row, 1);
                            }
                            MouseEventKind::ScrollDown => {
                                app.handle_scroll(ctx, mouse.column, mouse.row, -1);
                            }
                            _ => {}
                        }
                    }
                }
                _ => {}
            }
        }

        if app.should_quit {
            break;
        }
    }

    stdout().execute(DisableMouseCapture)?;
    disable_raw_mode()?;
    stdout().execute(LeaveAlternateScreen)?;
    Ok(())
}
