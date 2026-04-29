use crate::elements::ClipConfig;
use crate::types::*;

/// Current state of the pointer (mouse / touch) button.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
#[repr(u8)]
pub enum PointerInteractionState {
    /// The button was pressed during this frame.
    PressedThisFrame,
    /// The button is held down (pressed in a previous frame).
    Pressed,
    /// The button was released during this frame.
    ReleasedThisFrame,
    /// The button is not pressed (default).
    #[default]
    Released,
}

/// Pointer (mouse / touch) position and button state.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct PointerData {
    /// Current pointer position in layout coordinates.
    pub position: Vector2,
    /// Current button interaction state.
    pub state: PointerInteractionState,
}

/// Data returned when querying a scroll container's state.
#[derive(Debug, Clone, Copy, Default, PartialEq)]
pub struct ScrollContainerData {
    /// Current scroll offset.
    pub scroll_position: Vector2,
    /// Visible area dimensions.
    pub scroll_container_dimensions: Dimensions,
    /// Total content dimensions (may exceed container dimensions).
    pub content_dimensions: Dimensions,
    /// The clip configuration of this scroll container.
    pub config: ClipConfig,
    /// `true` if the scroll container was found.
    pub found: bool,
    /// Internal index of the scroll container.
    pub scroll_container_index: Option<usize>,
}
