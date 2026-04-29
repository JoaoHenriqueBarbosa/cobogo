use cobogo::render::{RenderCommand, RenderData};
use cobogo::types::{BoundingBox, Color as ClayColor};
use ratatui::buffer::Buffer;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};

pub struct CobogoRatatuiRenderer {
    clip_stack: Vec<Rect>,
}

impl CobogoRatatuiRenderer {
    pub fn new() -> Self {
        Self {
            clip_stack: Vec::new(),
        }
    }

    pub fn render(&mut self, commands: &[RenderCommand], buf: &mut Buffer) {
        self.clip_stack.clear();

        for cmd in commands {
            let rect = bbox_to_rect(&cmd.bounding_box, buf.area);

            match &cmd.render_data {
                RenderData::Rectangle(data) => {
                    if let Some(clipped) = self.clip_rect(rect) {
                        fill_rect(buf, clipped, cobogo_to_color(&data.background_color));
                    }
                }
                RenderData::Text(data) => {
                    if let Some(clipped) = self.clip_rect(rect) {
                        render_text(buf, clipped, &data.string_contents, cobogo_to_color(&data.text_color));
                    }
                }
                RenderData::Border(data) => {
                    if let Some(clipped) = self.clip_rect(rect) {
                        render_border(buf, clipped, cobogo_to_color(&data.color), &data.width);
                    }
                }
                RenderData::Image(_) => {
                    if let Some(clipped) = self.clip_rect(rect) {
                        fill_rect(buf, clipped, Color::DarkGray);
                        render_text(buf, clipped, "[img]", Color::Gray);
                    }
                }
                RenderData::Custom(_) => {
                    if let Some(clipped) = self.clip_rect(rect) {
                        fill_rect(buf, clipped, Color::DarkGray);
                    }
                }
                RenderData::Clip(_) => {
                    self.clip_stack.push(rect);
                }
                RenderData::None => {
                    self.clip_stack.pop();
                }
            }
        }
    }

    fn clip_rect(&self, rect: Rect) -> Option<Rect> {
        if rect.width == 0 || rect.height == 0 {
            return None;
        }
        let mut result = rect;
        for clip in &self.clip_stack {
            result = intersect(result, *clip);
            if result.width == 0 || result.height == 0 {
                return None;
            }
        }
        Some(result)
    }
}

impl Default for CobogoRatatuiRenderer {
    fn default() -> Self {
        Self::new()
    }
}

fn bbox_to_rect(bbox: &BoundingBox, area: Rect) -> Rect {
    let x = (bbox.x as i32).max(0).min(area.width as i32) as u16;
    let y = (bbox.y as i32).max(0).min(area.height as i32) as u16;
    let w = (bbox.width as i32).max(0).min((area.width as i32) - x as i32).max(0) as u16;
    let h = (bbox.height as i32).max(0).min((area.height as i32) - y as i32).max(0) as u16;
    Rect::new(x, y, w, h)
}

fn intersect(a: Rect, b: Rect) -> Rect {
    let x1 = a.x.max(b.x);
    let y1 = a.y.max(b.y);
    let x2 = (a.x + a.width).min(b.x + b.width);
    let y2 = (a.y + a.height).min(b.y + b.height);
    if x2 <= x1 || y2 <= y1 {
        Rect::new(0, 0, 0, 0)
    } else {
        Rect::new(x1, y1, x2 - x1, y2 - y1)
    }
}

fn cobogo_to_color(c: &ClayColor) -> Color {
    Color::Rgb(c.r as u8, c.g as u8, c.b as u8)
}

fn fill_rect(buf: &mut Buffer, rect: Rect, color: Color) {
    let style = Style::default().bg(color);
    for y in rect.y..rect.y + rect.height {
        for x in rect.x..rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_style(style);
                if cell.symbol() == "" {
                    cell.set_char(' ');
                }
            }
        }
    }
}

fn render_text(buf: &mut Buffer, rect: Rect, text: &str, fg: Color) {
    if rect.height == 0 || rect.width == 0 {
        return;
    }
    let style = Style::default().fg(fg);
    let mut x = rect.x;
    let y = rect.y;
    for ch in text.chars() {
        if x >= rect.x + rect.width {
            break;
        }
        if let Some(cell) = buf.cell_mut((x, y)) {
            cell.set_char(ch);
            cell.set_style(style);
        }
        x += 1;
    }
}

fn render_border(
    buf: &mut Buffer,
    rect: Rect,
    color: Color,
    width: &cobogo::types::BorderWidth,
) {
    let style = Style::default().fg(color);

    if width.top > 0 && rect.height > 0 {
        for x in rect.x..rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((x, rect.y)) {
                cell.set_char('─');
                cell.set_style(style);
            }
        }
    }
    if width.bottom > 0 && rect.height > 1 {
        let y = rect.y + rect.height - 1;
        for x in rect.x..rect.x + rect.width {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char('─');
                cell.set_style(style);
            }
        }
    }
    if width.left > 0 && rect.width > 0 {
        for y in rect.y..rect.y + rect.height {
            if let Some(cell) = buf.cell_mut((rect.x, y)) {
                cell.set_char('│');
                cell.set_style(style);
            }
        }
    }
    if width.right > 0 && rect.width > 1 {
        let x = rect.x + rect.width - 1;
        for y in rect.y..rect.y + rect.height {
            if let Some(cell) = buf.cell_mut((x, y)) {
                cell.set_char('│');
                cell.set_style(style);
            }
        }
    }

    // Corners
    if width.top > 0 && width.left > 0 {
        if let Some(cell) = buf.cell_mut((rect.x, rect.y)) {
            cell.set_char('┌');
            cell.set_style(style);
        }
    }
    if width.top > 0 && width.right > 0 && rect.width > 1 {
        if let Some(cell) = buf.cell_mut((rect.x + rect.width - 1, rect.y)) {
            cell.set_char('┐');
            cell.set_style(style);
        }
    }
    if width.bottom > 0 && width.left > 0 && rect.height > 1 {
        if let Some(cell) = buf.cell_mut((rect.x, rect.y + rect.height - 1)) {
            cell.set_char('└');
            cell.set_style(style);
        }
    }
    if width.bottom > 0 && width.right > 0 && rect.width > 1 && rect.height > 1 {
        if let Some(cell) = buf.cell_mut((rect.x + rect.width - 1, rect.y + rect.height - 1)) {
            cell.set_char('┘');
            cell.set_style(style);
        }
    }
}
