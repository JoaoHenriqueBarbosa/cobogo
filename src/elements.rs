use crate::layout::LayoutConfig;
use crate::types::*;

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ElementId {
    pub id: u32,
    pub offset: u32,
    pub base_id: u32,
    pub string_id: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TextWrapMode {
    #[default]
    Words,
    Newlines,
    None,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TextElementConfig {
    pub user_data: usize,
    pub text_color: Color,
    pub font_id: u16,
    pub font_size: u16,
    pub letter_spacing: u16,
    pub line_height: u16,
    pub wrap_mode: TextWrapMode,
    pub text_alignment: TextAlignment,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AspectRatioConfig {
    pub aspect_ratio: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ImageConfig {
    pub image_data: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FloatingAttachPointType {
    #[default]
    LeftTop,
    LeftCenter,
    LeftBottom,
    CenterTop,
    CenterCenter,
    CenterBottom,
    RightTop,
    RightCenter,
    RightBottom,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FloatingAttachPoints {
    pub element: FloatingAttachPointType,
    pub parent: FloatingAttachPointType,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PointerCaptureMode {
    #[default]
    Capture,
    Passthrough,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FloatingAttachTo {
    #[default]
    None,
    Parent,
    ElementWithId,
    Root,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FloatingClipTo {
    #[default]
    None,
    AttachedParent,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FloatingConfig {
    pub offset: Vector2,
    pub expand: Dimensions,
    pub parent_id: u32,
    pub z_index: i16,
    pub attach_points: FloatingAttachPoints,
    pub pointer_capture_mode: PointerCaptureMode,
    pub attach_to: FloatingAttachTo,
    pub clip_to: FloatingClipTo,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CustomConfig {
    pub custom_data: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ClipConfig {
    pub horizontal: bool,
    pub vertical: bool,
    pub child_offset: Vector2,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BorderConfig {
    pub color: Color,
    pub width: BorderWidth,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ElementData {
    pub bounding_box: BoundingBox,
    pub found: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ElementDeclaration {
    pub layout: LayoutConfig,
    pub background_color: Color,
    pub corner_radius: CornerRadius,
    pub aspect_ratio: AspectRatioConfig,
    pub image: ImageConfig,
    pub floating: FloatingConfig,
    pub custom: CustomConfig,
    pub clip: ClipConfig,
    pub border: BorderConfig,
    pub user_data: usize,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SharedElementConfig {
    pub background_color: Color,
    pub corner_radius: CornerRadius,
    pub user_data: usize,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ElementConfig {
    None,
    Border(BorderConfig),
    Floating(FloatingConfig),
    Clip(ClipConfig),
    Aspect(AspectRatioConfig),
    Image(ImageConfig),
    Text(TextElementConfig),
    Custom(CustomConfig),
    Shared(SharedElementConfig),
}

impl Default for ElementConfig {
    fn default() -> Self {
        ElementConfig::None
    }
}
