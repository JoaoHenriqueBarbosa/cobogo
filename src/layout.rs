#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LayoutDirection {
    LeftToRight = 0,
    TopToBottom = 1,
}

impl Default for LayoutDirection {
    fn default() -> Self {
        Self::LeftToRight
    }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct ChildAlignment {
    pub x: AlignmentX,
    pub y: AlignmentY,
}

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

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum SizingType {
    Fit(SizingMinMax),
    Grow(SizingMinMax),
    Percent(f32),
    Fixed(f32),
}

impl Default for SizingType {
    fn default() -> Self {
        Self::Fit(SizingMinMax::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct SizingAxis {
    pub sizing: SizingType,
}

impl SizingAxis {
    pub fn fit(min: f32, max: f32) -> Self {
        Self {
            sizing: SizingType::Fit(SizingMinMax { min, max }),
        }
    }

    pub fn grow(min: f32, max: f32) -> Self {
        Self {
            sizing: SizingType::Grow(SizingMinMax { min, max }),
        }
    }

    pub fn percent(p: f32) -> Self {
        Self {
            sizing: SizingType::Percent(p),
        }
    }

    pub fn fixed(size: f32) -> Self {
        Self {
            sizing: SizingType::Fixed(size),
        }
    }

    pub fn is_percent(&self) -> bool {
        matches!(self.sizing, SizingType::Percent(_))
    }

    pub fn is_fixed(&self) -> bool {
        matches!(self.sizing, SizingType::Fixed(_))
    }

    pub fn min_max(&self) -> SizingMinMax {
        match self.sizing {
            SizingType::Fit(mm) | SizingType::Grow(mm) => mm,
            SizingType::Fixed(size) => SizingMinMax { min: size, max: size },
            SizingType::Percent(_) => SizingMinMax::default(),
        }
    }

    pub fn percent_value(&self) -> f32 {
        match self.sizing {
            SizingType::Percent(p) => p,
            _ => 0.0,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct Sizing {
    pub width: SizingAxis,
    pub height: SizingAxis,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Padding {
    pub left: u16,
    pub right: u16,
    pub top: u16,
    pub bottom: u16,
}

impl Padding {
    pub fn all(v: u16) -> Self {
        Self {
            left: v,
            right: v,
            top: v,
            bottom: v,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default)]
pub struct LayoutConfig {
    pub sizing: Sizing,
    pub padding: Padding,
    pub child_gap: u16,
    pub child_alignment: ChildAlignment,
    pub layout_direction: LayoutDirection,
}
