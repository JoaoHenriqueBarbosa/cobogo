use crate::types::*;

#[derive(Debug, Clone, Default)]
pub struct TextRenderData {
    pub string_contents: String,
    pub base_chars: String,
    pub text_color: Color,
    pub font_id: u16,
    pub font_size: u16,
    pub letter_spacing: u16,
    pub line_height: u16,
}

#[derive(Debug, Clone, Default)]
pub struct RectangleRenderData {
    pub background_color: Color,
    pub corner_radius: CornerRadius,
}

#[derive(Debug, Clone, Default)]
pub struct ImageRenderData {
    pub background_color: Color,
    pub corner_radius: CornerRadius,
    pub image_data: usize,
}

#[derive(Debug, Clone, Default)]
pub struct CustomRenderData {
    pub background_color: Color,
    pub corner_radius: CornerRadius,
    pub custom_data: usize,
}

#[derive(Debug, Clone, Default)]
pub struct ClipRenderData {
    pub horizontal: bool,
    pub vertical: bool,
}

#[derive(Debug, Clone, Default)]
pub struct BorderRenderData {
    pub color: Color,
    pub corner_radius: CornerRadius,
    pub width: BorderWidth,
}

#[derive(Debug, Clone)]
pub enum RenderData {
    None,
    Rectangle(RectangleRenderData),
    Border(BorderRenderData),
    Text(TextRenderData),
    Image(ImageRenderData),
    Custom(CustomRenderData),
    Clip(ClipRenderData),
}

impl Default for RenderData {
    fn default() -> Self {
        RenderData::None
    }
}

#[derive(Debug, Clone, Default)]
pub struct RenderCommand {
    pub bounding_box: BoundingBox,
    pub render_data: RenderData,
    pub user_data: usize,
    pub id: u32,
    pub z_index: i16,
}
