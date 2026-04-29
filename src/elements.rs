use crate::layout::LayoutConfig;
use crate::types::*;

/// Unique identifier for a layout element.
///
/// Generated via [`Context::id`](crate::context::Context::id),
/// [`Context::idi`](crate::context::Context::idi), or
/// [`Context::id_local`](crate::context::Context::id_local).
/// After layout, pass an `ElementId` to
/// [`Context::get_element_data`](crate::context::Context::get_element_data)
/// to query its bounding box.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct ElementId {
    /// The computed hash.
    pub id: u32,
    /// Offset used for indexed IDs.
    pub offset: u32,
    /// Base hash before offset was applied.
    pub base_id: u32,
    /// The original string label.
    pub string_id: String,
}

/// How text wraps when it exceeds the available width.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TextWrapMode {
    /// Wrap at word boundaries (default).
    #[default]
    Words,
    /// Wrap only at explicit newline characters.
    Newlines,
    /// Never wrap.
    None,
}

/// Horizontal text alignment within its element.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum TextAlignment {
    #[default]
    Left,
    Center,
    Right,
}

/// Configuration for a text element.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct TextElementConfig {
    /// Arbitrary user data passed through to the text measurement function.
    pub user_data: usize,
    /// Text foreground color.
    pub text_color: Color,
    /// Font identifier (application-defined).
    pub font_id: u16,
    /// Font size in layout units.
    pub font_size: u16,
    /// Extra horizontal spacing between characters.
    pub letter_spacing: u16,
    /// Vertical line height. When `0`, the measured height is used.
    pub line_height: u16,
    /// Text wrapping mode.
    pub wrap_mode: TextWrapMode,
    /// Horizontal text alignment.
    pub text_alignment: TextAlignment,
}

/// Constrains an element to a fixed aspect ratio.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct AspectRatioConfig {
    /// Width divided by height (e.g. `16.0 / 9.0`).
    pub aspect_ratio: f32,
}

/// Configuration for an image element.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ImageConfig {
    /// Application-defined image data handle (e.g. a texture ID).
    pub image_data: usize,
}

/// Anchor point on an element or its parent used for floating attachment.
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

/// Pair of anchor points that define how a floating element attaches to its
/// reference.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct FloatingAttachPoints {
    /// Anchor point on the floating element itself.
    pub element: FloatingAttachPointType,
    /// Anchor point on the parent or reference element.
    pub parent: FloatingAttachPointType,
}

/// Whether a floating element captures pointer events.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PointerCaptureMode {
    /// The floating element captures pointer events (default).
    #[default]
    Capture,
    /// Pointer events pass through to elements underneath.
    Passthrough,
}

/// What element a floating element is positioned relative to.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FloatingAttachTo {
    /// Not a floating element (default).
    #[default]
    None,
    /// Float relative to the direct parent.
    Parent,
    /// Float relative to a specific element identified by
    /// [`FloatingConfig::parent_id`].
    ElementWithId,
    /// Float relative to the root container.
    Root,
}

/// Whether a floating element clips to its attached parent.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum FloatingClipTo {
    /// No clipping (default).
    #[default]
    None,
    /// Clip to the attached parent's bounds.
    AttachedParent,
}

/// Configuration for a floating (absolutely positioned) element.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct FloatingConfig {
    /// Pixel offset from the computed anchor position.
    pub offset: Vector2,
    /// Extra expansion added to the parent's bounding box for layout purposes.
    pub expand: Dimensions,
    /// ID of the element to attach to (when using
    /// [`FloatingAttachTo::ElementWithId`]).
    pub parent_id: u32,
    /// Stacking order. Higher values render on top.
    pub z_index: i16,
    /// Anchor points for positioning.
    pub attach_points: FloatingAttachPoints,
    /// Pointer event capture behavior.
    pub pointer_capture_mode: PointerCaptureMode,
    /// What this element floats relative to.
    pub attach_to: FloatingAttachTo,
    /// Whether to clip to the attached parent.
    pub clip_to: FloatingClipTo,
}

/// Arbitrary application-defined data attached to an element.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CustomConfig {
    /// Application-defined data handle.
    pub custom_data: usize,
}

/// Clipping and scrolling configuration for a container element.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ClipConfig {
    /// Clip children that overflow horizontally.
    pub horizontal: bool,
    /// Clip children that overflow vertically.
    pub vertical: bool,
    /// Offset applied to children (used for scroll position).
    pub child_offset: Vector2,
}

/// Border color and per-side widths.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BorderConfig {
    /// Border color.
    pub color: Color,
    /// Per-side border widths.
    pub width: BorderWidth,
}

/// Result of querying an element's position after layout.
///
/// Returned by
/// [`Context::get_element_data`](crate::context::Context::get_element_data).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ElementData {
    /// The element's computed bounding box.
    pub bounding_box: BoundingBox,
    /// `true` if the element was found in the layout tree.
    pub found: bool,
}

/// Full declaration of an element's visual and layout properties.
///
/// Pass this to
/// [`Context::with_element`](crate::context::Context::with_element) or
/// [`Context::configure_open_element`](crate::context::Context::configure_open_element).
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ElementDeclaration {
    /// Layout sizing, padding, gap, alignment, and direction.
    pub layout: LayoutConfig,
    /// Background fill color.
    pub background_color: Color,
    /// Per-corner border radius.
    pub corner_radius: CornerRadius,
    /// Aspect ratio constraint.
    pub aspect_ratio: AspectRatioConfig,
    /// Image configuration.
    pub image: ImageConfig,
    /// Floating (absolute positioning) configuration.
    pub floating: FloatingConfig,
    /// Custom application data.
    pub custom: CustomConfig,
    /// Clipping / scrolling configuration.
    pub clip: ClipConfig,
    /// Border configuration.
    pub border: BorderConfig,
    /// Arbitrary user data forwarded to render commands.
    pub user_data: usize,
}

/// Shared visual properties extracted from an [`ElementDeclaration`].
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct SharedElementConfig {
    pub background_color: Color,
    pub corner_radius: CornerRadius,
    pub user_data: usize,
}

/// Tagged union of all possible element configuration types.
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
