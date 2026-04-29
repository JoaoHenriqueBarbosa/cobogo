use crate::elements::ClipConfig;
use crate::types::*;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PointerInteractionState {
    PressedThisFrame,
    Pressed,
    ReleasedThisFrame,
    #[default]
    Released,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PointerData {
    pub position: Vector2,
    pub state: PointerInteractionState,
}

#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollContainerData {
    pub scroll_position: Vector2,
    pub scroll_container_dimensions: Dimensions,
    pub content_dimensions: Dimensions,
    pub config: ClipConfig,
    pub found: bool,
    pub scroll_container_index: Option<usize>,
}
