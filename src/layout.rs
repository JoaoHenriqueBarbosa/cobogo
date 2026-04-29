/// Direction in which children are arranged inside a container.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LayoutDirection {
    /// Children flow from left to right (default).
    LeftToRight = 0,
    /// Children flow from top to bottom.
    TopToBottom = 1,
}

impl Default for LayoutDirection {
    fn default() -> Self {
        Self::LeftToRight
    }
}

/// Horizontal alignment of children within their parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlignmentX {
    Left = 0,
    Right = 1,
    Center = 2,
}

impl Default for AlignmentX {
    fn default() -> Self {
        Self::Left
    }
}

/// Vertical alignment of children within their parent.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum AlignmentY {
    Top = 0,
    Bottom = 1,
    Center = 2,
}

impl Default for AlignmentY {
    fn default() -> Self {
        Self::Top
    }
}

/// Combined horizontal and vertical child alignment.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ChildAlignment {
    pub x: AlignmentX,
    pub y: AlignmentY,
}

/// Minimum and maximum constraints for a sizing axis.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SizingMinMax {
    pub min: f32,
    pub max: f32,
}

impl Default for SizingMinMax {
    fn default() -> Self {
        Self { min: 0.0, max: 0.0 }
    }
}

/// How a single axis is sized.
///
/// See [`SizingAxis`] for convenient constructors.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizingType {
    /// Shrink to fit content, clamped to \[min, max\].
    Fit(SizingMinMax),
    /// Expand to fill available space, clamped to \[min, max\].
    Grow(SizingMinMax),
    /// Size as a fraction of the parent (0.0–1.0).
    Percent(f32),
    /// Exact fixed size in layout units.
    Fixed(f32),
}

impl Default for SizingType {
    fn default() -> Self {
        Self::Fit(SizingMinMax::default())
    }
}

/// Sizing configuration for one axis (width **or** height).
///
/// Use the static constructors [`fit`](SizingAxis::fit),
/// [`grow`](SizingAxis::grow), [`percent`](SizingAxis::percent), and
/// [`fixed`](SizingAxis::fixed) to build values.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SizingAxis {
    pub sizing: SizingType,
}

impl SizingAxis {
    /// Shrink to fit content, clamped to \[`min`, `max`\].
    pub fn fit(min: f32, max: f32) -> Self {
        Self {
            sizing: SizingType::Fit(SizingMinMax { min, max }),
        }
    }

    /// Expand to fill available space, clamped to \[`min`, `max`\].
    pub fn grow(min: f32, max: f32) -> Self {
        Self {
            sizing: SizingType::Grow(SizingMinMax { min, max }),
        }
    }

    /// Size as a fraction of the parent. `p` should be in the range 0.0–1.0.
    pub fn percent(p: f32) -> Self {
        Self {
            sizing: SizingType::Percent(p),
        }
    }

    /// Exact fixed size in layout units.
    pub fn fixed(size: f32) -> Self {
        Self {
            sizing: SizingType::Fixed(size),
        }
    }

    /// Returns `true` if this axis uses percentage sizing.
    pub fn is_percent(&self) -> bool {
        matches!(self.sizing, SizingType::Percent(_))
    }

    /// Returns `true` if this axis uses fixed sizing.
    pub fn is_fixed(&self) -> bool {
        matches!(self.sizing, SizingType::Fixed(_))
    }

    /// Returns the min/max constraints for this axis.
    pub fn min_max(&self) -> SizingMinMax {
        match self.sizing {
            SizingType::Fit(mm) | SizingType::Grow(mm) => mm,
            SizingType::Fixed(size) => SizingMinMax { min: size, max: size },
            SizingType::Percent(_) => SizingMinMax::default(),
        }
    }

    /// Returns the percentage value, or `0.0` if this axis is not
    /// percentage-based.
    pub fn percent_value(&self) -> f32 {
        match self.sizing {
            SizingType::Percent(p) => p,
            _ => 0.0,
        }
    }
}

/// Width and height sizing for an element.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sizing {
    pub width: SizingAxis,
    pub height: SizingAxis,
}

impl Sizing {
    /// Shorthand: grow on both axes (fill available space).
    pub fn grow() -> Self {
        Self {
            width: SizingAxis::grow(0.0, f32::MAX),
            height: SizingAxis::grow(0.0, f32::MAX),
        }
    }

    /// Shorthand: fit on both axes (shrink to content).
    pub fn fit() -> Self {
        Self {
            width: SizingAxis::fit(0.0, f32::MAX),
            height: SizingAxis::fit(0.0, f32::MAX),
        }
    }
}

/// Per-side padding inside an element.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Padding {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
}

impl Padding {
    /// Creates a `Padding` with the same value on all four sides.
    pub fn all(v: u16) -> Self {
        Self {
            left: v,
            right: v,
            top: v,
            bottom: v,
        }
    }
}

/// Complete layout configuration for an element.
#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutConfig {
    /// Width and height sizing rules.
    pub sizing: Sizing,
    /// Inner padding.
    pub padding: Padding,
    /// Spacing between children in the layout direction.
    pub child_gap: u16,
    /// How children are aligned within available space.
    pub child_alignment: ChildAlignment,
    /// Direction in which children are arranged.
    pub layout_direction: LayoutDirection,
}
