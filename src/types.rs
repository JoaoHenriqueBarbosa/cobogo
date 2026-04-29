#[derive(Debug, Clone, Default, PartialEq)]
pub struct StringSlice {
    pub text: String,
    pub base: String,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Dimensions {
    pub width: f32,
    pub height: f32,
}

impl Dimensions {
    pub fn new(width: f32, height: f32) -> Self {
        Self { width, height }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Vector2 {
    pub x: f32,
    pub y: f32,
}

impl Vector2 {
    pub fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct Color {
    pub r: f32,
    pub g: f32,
    pub b: f32,
    pub a: f32,
}

impl Color {
    pub fn new(r: f32, g: f32, b: f32, a: f32) -> Self {
        Self { r, g, b, a }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct BoundingBox {
    pub x: f32,
    pub y: f32,
    pub width: f32,
    pub height: f32,
}

impl BoundingBox {
    pub fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self { x, y, width, height }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct CornerRadius {
    pub top_left: f32,
    pub top_right: f32,
    pub bottom_left: f32,
    pub bottom_right: f32,
}

impl CornerRadius {
    pub fn all(radius: f32) -> Self {
        Self {
            top_left: radius,
            top_right: radius,
            bottom_left: radius,
            bottom_right: radius,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BorderWidth {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
    pub between_children: u16,
}

impl BorderWidth {
    pub fn outside(width: u16) -> Self {
        Self {
            left: width,
            right: width,
            top: width,
            bottom: width,
            between_children: 0,
        }
    }

    pub fn all(width: u16) -> Self {
        Self {
            left: width,
            right: width,
            top: width,
            bottom: width,
            between_children: width,
        }
    }

    pub fn is_zero(&self) -> bool {
        self.left == 0
            && self.right == 0
            && self.top == 0
            && self.bottom == 0
            && self.between_children == 0
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct BooleanWarnings {
    pub max_elements_exceeded: bool,
    pub max_render_commands_exceeded: bool,
    pub max_text_measure_cache_exceeded: bool,
    pub text_measurement_function_not_set: bool,
}
