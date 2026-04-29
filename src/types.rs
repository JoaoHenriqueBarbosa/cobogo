/// A slice of text with an associated base string.
///
/// Used internally to represent text fragments during layout.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct StringSlice {
    /// The text content of this slice.
    pub text: String,
    /// The original base string this slice was derived from.
    pub base: String,
}

/// 2D size with width and height.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Dimensions {
    pub width: f32,
    pub height: f32,
}

impl Dimensions {
    /// Creates a new `Dimensions`.
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

/// 2D position or offset.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    /// Creates a new `Vector2`.
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// RGBA color with components in the 0–255 range stored as `f32`.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    /// Alpha channel. `0.0` is fully transparent, `255.0` is fully opaque.
    pub a: f32,
}

impl Color {
    /// Creates a new `Color`.
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

/// Axis-aligned bounding box defined by position and size.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    /// Creates a new `BoundingBox`.
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

/// Per-corner border radius.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl CornerRadius {
    /// Creates a `CornerRadius` with the same value on all four corners.
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }
}

/// Per-side border width, including an optional width between children.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BorderWidth {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
    /// Width of the border drawn between sibling children.
    pub between_children: u16,
}

impl BorderWidth {
    /// Creates a `BorderWidth` with the same value on all outer sides and no
    /// border between children.
    pub fn outside(width: u16) -> Self {
        Self {
            left: width,
            right: width,
            top: width,
            bottom: width,
            between_children: 0,
        }
    }

    /// Creates a `BorderWidth` with the same value on all sides *and* between
    /// children.
    pub fn all(width: u16) -> Self {
        Self {
            left: width,
            right: width,
            top: width,
            bottom: width,
            between_children: width,
        }
    }

    /// Returns `true` if every width component is zero.
    pub fn is_zero(&self) -> bool {
        self.left == 0
            && self.right == 0
            && self.top == 0
            && self.bottom == 0
            && self.between_children == 0
    }
}

/// Flags set by the layout engine when capacity limits are exceeded.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BooleanWarnings {
    pub max_elements_exceeded: bool,
    pub max_render_commands_exceeded: bool,
    pub max_text_measure_cache_exceeded: bool,
    pub text_measurement_function_not_set: bool,
}
