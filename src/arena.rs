use crate::elements::*;
use crate::input::*;
use crate::layout::*;
use crate::types::*;

#[derive(Debug, Clone, Default)]
pub struct Warning {
    pub base_message: String,
    pub dynamic_message: String,
}

#[derive(Debug, Clone, Default)]
pub struct WrappedTextLine {
    pub dimensions: Dimensions,
    pub text: String,
}

#[derive(Debug, Clone, Default)]
pub struct TextElementData {
    pub text: String,
    pub preferred_dimensions: Dimensions,
    pub element_index: i32,
    pub wrapped_lines_start: usize,
    pub wrapped_lines_count: usize,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutElementChildren {
    pub elements: Vec<i32>,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutElement {
    pub children: LayoutElementChildren,
    pub text_element_data_index: Option<usize>,
    pub dimensions: Dimensions,
    pub min_dimensions: Dimensions,
    pub layout_config: LayoutConfig,
    pub element_configs: Vec<ElementConfig>,
    pub id: u32,
    pub floating_children_count: u16,
}

#[derive(Debug, Clone, Default)]
pub struct ScrollContainerDataInternal {
    pub layout_element_index: i32,
    pub bounding_box: BoundingBox,
    pub content_size: Dimensions,
    pub scroll_origin: Vector2,
    pub pointer_origin: Vector2,
    pub scroll_momentum: Vector2,
    pub scroll_position: Vector2,
    pub previous_delta: Vector2,
    pub momentum_time: f32,
    pub element_id: u32,
    pub open_this_frame: bool,
    pub pointer_scroll_active: bool,
}

#[derive(Debug, Clone, Default)]
pub struct DebugElementData {
    pub collision: bool,
    pub collapsed: bool,
}

#[derive(Debug, Clone)]
pub struct LayoutElementHashMapItem {
    pub bounding_box: BoundingBox,
    pub element_id: ElementId,
    pub layout_element_index: i32,
    pub on_hover_function: Option<fn(ElementId, PointerData, usize)>,
    pub hover_function_user_data: usize,
    pub next_index: i32,
    pub generation: u32,
    pub debug_data: DebugElementData,
}

impl Default for LayoutElementHashMapItem {
    fn default() -> Self {
        Self {
            bounding_box: BoundingBox::default(),
            element_id: ElementId::default(),
            layout_element_index: 0,
            on_hover_function: None,
            hover_function_user_data: 0,
            next_index: -1,
            generation: 0,
            debug_data: DebugElementData::default(),
        }
    }
}

#[derive(Debug, Clone, Default)]
pub struct MeasuredWord {
    pub start_offset: i32,
    pub length: i32,
    pub width: f32,
    pub next: i32,
}

#[derive(Debug, Clone, Default)]
pub struct MeasureTextCacheItem {
    pub unwrapped_dimensions: Dimensions,
    pub measured_words_start_index: i32,
    pub min_width: f32,
    pub contains_newlines: bool,
    pub id: u32,
    pub next_index: i32,
    pub generation: u32,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutElementTreeNode {
    pub layout_element_index: i32,
    pub position: Vector2,
    pub next_child_offset: Vector2,
}

#[derive(Debug, Clone, Default)]
pub struct LayoutElementTreeRoot {
    pub layout_element_index: i32,
    pub parent_id: u32,
    pub clip_element_id: u32,
    pub z_index: i16,
    pub pointer_offset: Vector2,
}

pub type MeasureTextFunction = fn(&str, &TextElementConfig, usize) -> Dimensions;
pub type QueryScrollOffsetFunction = fn(u32, usize) -> Vector2;
