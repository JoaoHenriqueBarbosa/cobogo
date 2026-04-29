use crate::types::*;

/// Render data for a text element.
#[derive(Debug, Clone, Default)]
pub struct TextRenderData {
    /// The text string to render.
    pub string_contents: String,
    /// The base characters (before wrapping) for reference.
    pub base_chars: String,
    /// Text foreground color.
    pub text_color: Color,
    /// Application-defined font identifier.
    pub font_id: u16,
    /// Font size in layout units.
    pub font_size: u16,
    /// Extra horizontal spacing between characters.
    pub letter_spacing: u16,
    /// Vertical line height.
    pub line_height: u16,
}

/// Render data for a filled rectangle.
#[derive(Debug, Clone, Default)]
pub struct RectangleRenderData {
    /// Fill color.
    pub background_color: Color,
    /// Per-corner border radius.
    pub corner_radius: CornerRadius,
}

/// Render data for an image element.
#[derive(Debug, Clone, Default)]
pub struct ImageRenderData {
    /// Background / tint color.
    pub background_color: Color,
    /// Per-corner border radius.
    pub corner_radius: CornerRadius,
    /// Application-defined image data handle.
    pub image_data: usize,
}

/// Render data for a custom (application-defined) element.
#[derive(Debug, Clone, Default)]
pub struct CustomRenderData {
    /// Background color.
    pub background_color: Color,
    /// Per-corner border radius.
    pub corner_radius: CornerRadius,
    /// Application-defined data handle.
    pub custom_data: usize,
}

/// Render data for a clip region.
///
/// A `Clip` command opens a clip region; the corresponding [`RenderData::None`]
/// command closes it.
#[derive(Debug, Clone, Default)]
pub struct ClipRenderData {
    /// Whether horizontal clipping is active.
    pub horizontal: bool,
    /// Whether vertical clipping is active.
    pub vertical: bool,
}

/// Render data for element borders.
#[derive(Debug, Clone, Default)]
pub struct BorderRenderData {
    /// Border color.
    pub color: Color,
    /// Per-corner border radius.
    pub corner_radius: CornerRadius,
    /// Per-side border widths.
    pub width: BorderWidth,
}

/// The payload of a [`RenderCommand`], describing *what* to draw.
#[derive(Debug, Clone)]
pub enum RenderData {
    /// No rendering (also used to close a clip region).
    None,
    /// A filled rectangle.
    Rectangle(RectangleRenderData),
    /// Element borders.
    Border(BorderRenderData),
    /// A text string.
    Text(TextRenderData),
    /// An image.
    Image(ImageRenderData),
    /// Application-defined custom rendering.
    Custom(CustomRenderData),
    /// Opens a clip region. Closed by a subsequent [`RenderData::None`].
    Clip(ClipRenderData),
}

impl Default for RenderData {
    fn default() -> Self {
        RenderData::None
    }
}

/// A single instruction produced by the layout engine telling the renderer
/// *what* to draw and *where*.
///
/// After calling [`Context::end_layout`](crate::context::Context::end_layout),
/// iterate over the returned `Vec<RenderCommand>` and handle each
/// [`render_data`](RenderCommand::render_data) variant.
#[derive(Debug, Clone, Default)]
pub struct RenderCommand {
    /// Position and size of this element.
    pub bounding_box: BoundingBox,
    /// What to draw.
    pub render_data: RenderData,
    /// Arbitrary user data forwarded from the element declaration.
    pub user_data: usize,
    /// The element's hashed ID.
    pub id: u32,
    /// Stacking order for z-sorting.
    pub z_index: i16,
}
