use crate::arena::*;
use crate::elements::*;
use crate::hash;
use crate::input::*;
use crate::layout::*;
use crate::render::*;
use crate::types::*;

/// Function signature for custom error handlers.
pub type ErrorHandlerFunction = fn(ErrorData);

/// Error information passed to the [`ErrorHandlerFunction`].
#[derive(Debug, Clone)]
pub struct ErrorData {
    /// The category of error.
    pub error_type: ErrorType,
    /// A human-readable description of the error.
    pub error_text: String,
    /// User data that was registered with [`Context::set_error_handler`].
    pub user_data: usize,
}

/// Categories of errors the layout engine can report.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ErrorType {
    TextMeasurementFunctionNotProvided,
    ArenaCapacityExceeded,
    ElementsCapacityExceeded,
    TextMeasurementCapacityExceeded,
    DuplicateId,
    FloatingContainerParentNotFound,
    PercentageOver1,
    InternalError,
    UnbalancedOpenClose,
}

fn default_error_handler(_error: ErrorData) {}

/// The main layout context.
///
/// `Context` holds all state needed for a single layout pass. Create one with
/// [`Context::new`], build your UI tree between [`begin_layout`](Context::begin_layout)
/// and [`end_layout`](Context::end_layout), then consume the resulting
/// [`RenderCommand`] list.
///
/// The context is designed to be reused across frames — call
/// [`begin_layout`](Context::begin_layout) at the start of each frame to
/// reset ephemeral state while preserving caches.
#[derive(Clone)]
pub struct Context {
    pub max_element_count: i32,
    pub max_measure_text_cache_word_count: i32,
    pub warnings_enabled: bool,
    pub error_handler: ErrorHandlerFunction,
    pub error_handler_user_data: usize,
    pub boolean_warnings: BooleanWarnings,
    pub warnings: Vec<Warning>,

    pub pointer_info: PointerData,
    pub layout_dimensions: Dimensions,
    pub dynamic_element_index: u32,
    pub debug_mode_enabled: bool,
    pub disable_culling: bool,
    pub external_scroll_handling_enabled: bool,
    pub shadow_layout_mode: bool,
    pub debug_selected_element_id: u32,
    pub generation: u32,

    pub measure_text_function: Option<MeasureTextFunction>,
    pub measure_text_user_data: usize,
    pub query_scroll_offset_function: Option<QueryScrollOffsetFunction>,
    pub query_scroll_offset_user_data: usize,

    // Layout Elements / Render Commands
    pub layout_elements: Vec<LayoutElement>,
    pub render_commands: Vec<RenderCommand>,
    pub open_layout_element_stack: Vec<i32>,
    pub layout_element_children: Vec<i32>,
    pub layout_element_children_buffer: Vec<i32>,
    pub text_element_data: Vec<TextElementData>,
    pub aspect_ratio_element_indexes: Vec<i32>,
    pub reusable_element_index_buffer: Vec<i32>,
    pub layout_element_clip_element_ids: Vec<i32>,

    // Configs (stored in Vecs, referenced by index in LayoutElement.element_configs)
    pub layout_element_id_strings: Vec<String>,
    pub wrapped_text_lines: Vec<WrappedTextLine>,
    pub layout_element_tree_nodes: Vec<LayoutElementTreeNode>,
    pub layout_element_tree_roots: Vec<LayoutElementTreeRoot>,

    // Hash map
    pub layout_elements_hash_map_internal: Vec<LayoutElementHashMapItem>,
    pub layout_elements_hash_map: Vec<i32>,

    // Text measure cache
    pub measure_text_hash_map_internal: Vec<MeasureTextCacheItem>,
    pub measure_text_hash_map_internal_free_list: Vec<i32>,
    pub measure_text_hash_map: Vec<i32>,
    pub measured_words: Vec<MeasuredWord>,
    pub measured_words_free_list: Vec<i32>,

    pub open_clip_element_stack: Vec<i32>,
    pub pointer_over_ids: Vec<ElementId>,
    pub scroll_container_datas: Vec<ScrollContainerDataInternal>,
    pub tree_node_visited: Vec<bool>,
    pub dynamic_string_data: Vec<char>,
    pub debug_element_data: Vec<DebugElementData>,

    // Debug view width
    pub debug_view_width: i32,
    pub debug_view_highlight_color: Color,
}

impl Context {
    /// Creates a new context with default capacity (8 192 elements, 16 384
    /// cached text words).
    pub fn new(layout_dimensions: Dimensions) -> Self {
        Self::with_capacity(layout_dimensions, 8192, 16384)
    }

    /// Creates a new context with explicit capacity limits.
    pub fn with_capacity(
        layout_dimensions: Dimensions,
        max_element_count: i32,
        max_measure_text_cache_word_count: i32,
    ) -> Self {
        let cap = max_element_count as usize;
        let word_cap = max_measure_text_cache_word_count as usize;
        let mut ctx = Context {
            max_element_count,
            max_measure_text_cache_word_count,
            warnings_enabled: true,
            error_handler: default_error_handler,
            error_handler_user_data: 0,
            boolean_warnings: BooleanWarnings::default(),
            warnings: Vec::with_capacity(100),
            pointer_info: PointerData::default(),
            layout_dimensions,
            dynamic_element_index: 0,
            debug_mode_enabled: false,
            disable_culling: false,
            external_scroll_handling_enabled: false,
            shadow_layout_mode: false,
            debug_selected_element_id: 0,
            generation: 0,
            measure_text_function: None,
            measure_text_user_data: 0,
            query_scroll_offset_function: None,
            query_scroll_offset_user_data: 0,
            layout_elements: Vec::with_capacity(cap),
            render_commands: Vec::with_capacity(cap),
            open_layout_element_stack: Vec::with_capacity(cap),
            layout_element_children: Vec::with_capacity(cap),
            layout_element_children_buffer: Vec::with_capacity(cap),
            text_element_data: Vec::with_capacity(cap),
            aspect_ratio_element_indexes: Vec::with_capacity(cap),
            reusable_element_index_buffer: Vec::with_capacity(cap),
            layout_element_clip_element_ids: Vec::with_capacity(cap),
            layout_element_id_strings: Vec::with_capacity(cap),
            wrapped_text_lines: Vec::with_capacity(cap),
            layout_element_tree_nodes: Vec::with_capacity(cap),
            layout_element_tree_roots: Vec::with_capacity(cap),
            layout_elements_hash_map_internal: Vec::with_capacity(cap),
            layout_elements_hash_map: vec![-1; cap],
            measure_text_hash_map_internal: Vec::with_capacity(cap),
            measure_text_hash_map_internal_free_list: Vec::with_capacity(cap),
            measure_text_hash_map: vec![0; cap],
            measured_words: Vec::with_capacity(word_cap),
            measured_words_free_list: Vec::with_capacity(word_cap),
            open_clip_element_stack: Vec::with_capacity(cap),
            pointer_over_ids: Vec::with_capacity(cap),
            scroll_container_datas: Vec::with_capacity(100),
            tree_node_visited: vec![false; cap],
            dynamic_string_data: Vec::with_capacity(cap),
            debug_element_data: Vec::with_capacity(cap),
            debug_view_width: 400,
            debug_view_highlight_color: Color::new(168.0, 66.0, 28.0, 100.0),
        };
        // Reserve index 0 in measure text hash map internal (0 = "no next element")
        ctx.measure_text_hash_map_internal.push(MeasureTextCacheItem::default());
        ctx
    }

    /// Registers a custom error handler.
    pub fn set_error_handler(&mut self, handler: ErrorHandlerFunction, user_data: usize) {
        self.error_handler = handler;
        self.error_handler_user_data = user_data;
    }

    pub(crate) fn report_error(&self, error_type: ErrorType, error_text: &str) {
        (self.error_handler)(ErrorData {
            error_type,
            error_text: error_text.to_string(),
            user_data: self.error_handler_user_data,
        });
    }

    fn initialize_ephemeral_memory(&mut self) {
        self.layout_element_children_buffer.clear();
        self.layout_elements.clear();
        self.warnings.clear();
        self.layout_element_id_strings.clear();
        self.wrapped_text_lines.clear();
        self.layout_element_tree_nodes.clear();
        self.layout_element_tree_roots.clear();
        self.layout_element_children.clear();
        self.open_layout_element_stack.clear();
        self.text_element_data.clear();
        self.aspect_ratio_element_indexes.clear();
        self.render_commands.clear();
        self.tree_node_visited.fill(false);
        self.open_clip_element_stack.clear();
        self.reusable_element_index_buffer.clear();
        self.layout_element_clip_element_ids.clear();
        self.dynamic_string_data.clear();
    }

    /// Updates the viewport dimensions used for layout.
    pub fn set_layout_dimensions(&mut self, dimensions: Dimensions) {
        self.layout_dimensions = dimensions;
    }

    /// Returns the current viewport dimensions.
    pub fn get_layout_dimensions(&self) -> Dimensions {
        self.layout_dimensions
    }

    /// Sets the text measurement callback.
    ///
    /// This **must** be called before any [`text`](Context::text) elements
    /// are used, otherwise the layout engine cannot determine text
    /// dimensions.
    pub fn set_measure_text_function(
        &mut self,
        func: MeasureTextFunction,
        user_data: usize,
    ) {
        self.measure_text_function = Some(func);
        self.measure_text_user_data = user_data;
    }

    /// Sets the callback used to query external scroll offsets.
    pub fn set_query_scroll_offset_function(
        &mut self,
        func: QueryScrollOffsetFunction,
        user_data: usize,
    ) {
        self.query_scroll_offset_function = Some(func);
        self.query_scroll_offset_user_data = user_data;
    }

    /// Enables or disables the debug visualization overlay.
    pub fn set_debug_mode_enabled(&mut self, enabled: bool) {
        self.debug_mode_enabled = enabled;
    }

    /// Returns `true` if debug mode is enabled.
    pub fn is_debug_mode_enabled(&self) -> bool {
        self.debug_mode_enabled
    }

    /// Enables or disables offscreen element culling.
    pub fn set_culling_enabled(&mut self, enabled: bool) {
        self.disable_culling = !enabled;
    }

    /// Enables or disables external scroll handling.
    pub fn set_external_scroll_handling_enabled(&mut self, enabled: bool) {
        self.external_scroll_handling_enabled = enabled;
    }

    /// Returns the maximum number of elements this context can hold.
    pub fn get_max_element_count(&self) -> i32 {
        self.max_element_count
    }

    /// Sets the maximum number of elements this context can hold.
    pub fn set_max_element_count(&mut self, count: i32) {
        self.max_element_count = count;
    }

    /// Returns the maximum number of cached text measurement words.
    pub fn get_max_measure_text_cache_word_count(&self) -> i32 {
        self.max_measure_text_cache_word_count
    }

    /// Sets the maximum number of cached text measurement words.
    pub fn set_max_measure_text_cache_word_count(&mut self, count: i32) {
        self.max_measure_text_cache_word_count = count;
    }

    /// Clears the text measurement cache.
    pub fn reset_measure_text_cache(&mut self) {
        self.measure_text_hash_map_internal.clear();
        self.measure_text_hash_map_internal_free_list.clear();
        self.measure_text_hash_map.fill(0);
        self.measured_words.clear();
        self.measured_words_free_list.clear();
        self.measure_text_hash_map_internal.push(MeasureTextCacheItem::default());
    }

    // ========================================================================
    // Hash map operations
    // ========================================================================

    /// Looks up an element's internal hash map entry by hashed ID.
    pub fn get_hash_map_item(&self, id: u32) -> Option<&LayoutElementHashMapItem> {
        let capacity = self.layout_elements_hash_map.len();
        if capacity == 0 {
            return None;
        }
        let hash_bucket = (id as usize) % capacity;
        let mut element_index = self.layout_elements_hash_map[hash_bucket];
        while element_index != -1 {
            let idx = element_index as usize;
            if idx >= self.layout_elements_hash_map_internal.len() {
                return None;
            }
            let hash_entry = &self.layout_elements_hash_map_internal[idx];
            if hash_entry.element_id.id == id {
                return Some(hash_entry);
            }
            element_index = hash_entry.next_index;
        }
        None
    }

    /// Mutable version of [`get_hash_map_item`](Context::get_hash_map_item).
    pub fn get_hash_map_item_mut(&mut self, id: u32) -> Option<&mut LayoutElementHashMapItem> {
        let capacity = self.layout_elements_hash_map.len();
        if capacity == 0 {
            return None;
        }
        let hash_bucket = (id as usize) % capacity;
        let mut element_index = self.layout_elements_hash_map[hash_bucket];
        while element_index != -1 {
            let idx = element_index as usize;
            if idx >= self.layout_elements_hash_map_internal.len() {
                return None;
            }
            if self.layout_elements_hash_map_internal[idx].element_id.id == id {
                return Some(&mut self.layout_elements_hash_map_internal[idx]);
            }
            element_index = self.layout_elements_hash_map_internal[idx].next_index;
        }
        None
    }

    pub(crate) fn add_hash_map_item(
        &mut self,
        element_id: ElementId,
        layout_element_index: i32,
    ) -> Option<usize> {
        let capacity = self.layout_elements_hash_map.len();
        if capacity == 0
            || self.layout_elements_hash_map_internal.len()
                >= self.layout_elements_hash_map_internal.capacity() - 1
        {
            return None;
        }
        let hash_bucket = (element_id.id as usize) % capacity;
        let mut hash_item_previous: i32 = -1;
        let mut hash_item_index = self.layout_elements_hash_map[hash_bucket];

        while hash_item_index != -1 {
            let idx = hash_item_index as usize;
            let hash_item = &self.layout_elements_hash_map_internal[idx];
            if hash_item.element_id.id == element_id.id {
                if hash_item.generation <= self.generation {
                    let item = &mut self.layout_elements_hash_map_internal[idx];
                    item.element_id = element_id;
                    item.generation = self.generation + 1;
                    item.layout_element_index = layout_element_index;
                    item.debug_data.collision = false;
                    item.on_hover_function = None;
                    item.hover_function_user_data = 0;
                } else {
                    self.report_error(
                        ErrorType::DuplicateId,
                        "An element with this ID was already previously declared during this layout.",
                    );
                    if self.debug_mode_enabled {
                        self.layout_elements_hash_map_internal[idx]
                            .debug_data
                            .collision = true;
                    }
                }
                return Some(idx);
            }
            hash_item_previous = hash_item_index;
            hash_item_index = self.layout_elements_hash_map_internal[idx].next_index;
        }

        let item = LayoutElementHashMapItem {
            element_id,
            layout_element_index,
            next_index: -1,
            generation: self.generation + 1,
            debug_data: DebugElementData::default(),
            ..Default::default()
        };
        self.layout_elements_hash_map_internal.push(item);
        self.debug_element_data.push(DebugElementData::default());
        let new_index = (self.layout_elements_hash_map_internal.len() - 1) as i32;

        if hash_item_previous != -1 {
            self.layout_elements_hash_map_internal[hash_item_previous as usize].next_index =
                new_index;
        } else {
            self.layout_elements_hash_map[hash_bucket] = new_index;
        }
        Some(new_index as usize)
    }

    // ========================================================================
    // Element operations
    // ========================================================================

    pub(crate) fn get_open_layout_element_index(&self) -> i32 {
        *self.open_layout_element_stack.last().unwrap_or(&0)
    }

    fn generate_id_for_anonymous_element(&mut self, element_index: i32) -> ElementId {
        let parent_index = if self.open_layout_element_stack.len() >= 2 {
            self.open_layout_element_stack[self.open_layout_element_stack.len() - 2]
        } else {
            0
        };
        let parent = &self.layout_elements[parent_index as usize];
        let offset = parent.children.elements.len() as u32 + parent.floating_children_count as u32;
        let parent_id = parent.id;
        let element_id = hash::hash_number(offset, parent_id);
        self.layout_elements[element_index as usize].id = element_id.id;
        self.add_hash_map_item(element_id.clone(), element_index);
        self.layout_element_id_strings.push(element_id.string_id.clone());
        element_id
    }

    /// Opens a new anonymous element and pushes it onto the layout stack.
    ///
    /// Prefer [`with_anonymous_element`](Context::with_anonymous_element) for
    /// a closure-based API that automatically closes the element.
    pub fn open_element(&mut self) {
        if self.layout_elements.len() >= self.max_element_count as usize - 1
            || self.boolean_warnings.max_elements_exceeded
        {
            self.boolean_warnings.max_elements_exceeded = true;
            return;
        }
        let element = LayoutElement::default();
        self.layout_elements.push(element);
        let idx = (self.layout_elements.len() - 1) as i32;
        self.open_layout_element_stack.push(idx);
        self.generate_id_for_anonymous_element(idx);

        let clip_id = self
            .open_clip_element_stack
            .last()
            .copied()
            .unwrap_or(0);
        while self.layout_element_clip_element_ids.len() <= idx as usize {
            self.layout_element_clip_element_ids.push(0);
        }
        self.layout_element_clip_element_ids[idx as usize] = clip_id;
    }

    /// Opens a new element with an explicit ID and pushes it onto the layout
    /// stack.
    ///
    /// Prefer [`with_element`](Context::with_element) for a closure-based API
    /// that automatically closes the element.
    pub fn open_element_with_id(&mut self, element_id: ElementId) {
        if self.layout_elements.len() >= self.max_element_count as usize - 1
            || self.boolean_warnings.max_elements_exceeded
        {
            self.boolean_warnings.max_elements_exceeded = true;
            return;
        }
        let mut element = LayoutElement::default();
        element.id = element_id.id;
        self.layout_elements.push(element);
        let idx = (self.layout_elements.len() - 1) as i32;
        self.open_layout_element_stack.push(idx);
        self.add_hash_map_item(element_id.clone(), idx);
        self.layout_element_id_strings.push(element_id.string_id);

        let clip_id = self.open_clip_element_stack.last().copied().unwrap_or(0);
        while self.layout_element_clip_element_ids.len() <= idx as usize {
            self.layout_element_clip_element_ids.push(0);
        }
        self.layout_element_clip_element_ids[idx as usize] = clip_id;
    }

    /// Applies an [`ElementDeclaration`] to the currently open element.
    pub fn configure_open_element(&mut self, declaration: &ElementDeclaration) {
        if self.boolean_warnings.max_elements_exceeded {
            return;
        }
        let open_idx = self.get_open_layout_element_index() as usize;
        self.layout_elements[open_idx].layout_config = declaration.layout;

        // Validate percent sizing
        if let SizingType::Percent(p) = declaration.layout.sizing.width.sizing {
            if p > 1.0 {
                self.report_error(
                    ErrorType::PercentageOver1,
                    "An element was configured with CLAY_SIZING_PERCENT, but the percentage value was over 1.0.",
                );
            }
        }
        if let SizingType::Percent(p) = declaration.layout.sizing.height.sizing {
            if p > 1.0 {
                self.report_error(
                    ErrorType::PercentageOver1,
                    "An element was configured with CLAY_SIZING_PERCENT, but the percentage value was over 1.0.",
                );
            }
        }

        // Shared config
        let mut shared_config: Option<SharedElementConfig> = None;
        if declaration.background_color.a > 0.0 {
            let mut sc = SharedElementConfig::default();
            sc.background_color = declaration.background_color;
            shared_config = Some(sc);
        }
        if declaration.corner_radius != CornerRadius::default() {
            if let Some(ref mut sc) = shared_config {
                sc.corner_radius = declaration.corner_radius;
            } else {
                shared_config = Some(SharedElementConfig {
                    corner_radius: declaration.corner_radius,
                    ..Default::default()
                });
            }
        }
        if declaration.user_data != 0 {
            if let Some(ref mut sc) = shared_config {
                sc.user_data = declaration.user_data;
            } else {
                shared_config = Some(SharedElementConfig {
                    user_data: declaration.user_data,
                    ..Default::default()
                });
            }
        }
        if let Some(sc) = shared_config {
            self.layout_elements[open_idx]
                .element_configs
                .push(ElementConfig::Shared(sc));
        }

        // Image
        if declaration.image.image_data != 0 {
            self.layout_elements[open_idx]
                .element_configs
                .push(ElementConfig::Image(declaration.image));
        }

        // Aspect ratio
        if declaration.aspect_ratio.aspect_ratio > 0.0 {
            self.layout_elements[open_idx]
                .element_configs
                .push(ElementConfig::Aspect(declaration.aspect_ratio));
            self.aspect_ratio_element_indexes.push(open_idx as i32);
        }

        // Floating
        if declaration.floating.attach_to != FloatingAttachTo::None {
            let mut floating_config = declaration.floating;
            let parent_stack_idx = if self.open_layout_element_stack.len() >= 2 {
                self.open_layout_element_stack[self.open_layout_element_stack.len() - 2] as usize
            } else {
                0
            };
            let hierarchical_parent_id = self.layout_elements[parent_stack_idx].id;

            let mut clip_element_id: u32 = 0;
            match declaration.floating.attach_to {
                FloatingAttachTo::Parent => {
                    floating_config.parent_id = hierarchical_parent_id;
                    if let Some(&clip_id) = self.open_clip_element_stack.last() {
                        clip_element_id = clip_id as u32;
                    }
                }
                FloatingAttachTo::ElementWithId => {
                    if let Some(parent_item) = self.get_hash_map_item(floating_config.parent_id) {
                        let parent_le_idx = parent_item.layout_element_index;
                        if (parent_le_idx as usize) < self.layout_element_clip_element_ids.len() {
                            clip_element_id =
                                self.layout_element_clip_element_ids[parent_le_idx as usize] as u32;
                        }
                    } else {
                        self.report_error(
                            ErrorType::FloatingContainerParentNotFound,
                            "A floating element was declared with a parentId, but no element with that ID was found.",
                        );
                    }
                }
                FloatingAttachTo::Root => {
                    floating_config.parent_id =
                        hash::hash_string("Clay__RootContainer", 0).id;
                }
                FloatingAttachTo::None => {}
            }

            if declaration.floating.clip_to == FloatingClipTo::None {
                clip_element_id = 0;
            }

            let current_element_index = *self.open_layout_element_stack.last().unwrap();
            while self.layout_element_clip_element_ids.len() <= current_element_index as usize {
                self.layout_element_clip_element_ids.push(0);
            }
            self.layout_element_clip_element_ids[current_element_index as usize] =
                clip_element_id as i32;
            self.open_clip_element_stack.push(clip_element_id as i32);

            self.layout_element_tree_roots
                .push(LayoutElementTreeRoot {
                    layout_element_index: current_element_index,
                    parent_id: floating_config.parent_id,
                    clip_element_id,
                    z_index: floating_config.z_index,
                    pointer_offset: Vector2::default(),
                });

            self.layout_elements[open_idx]
                .element_configs
                .push(ElementConfig::Floating(floating_config));
        }

        // Custom
        if declaration.custom.custom_data != 0 {
            self.layout_elements[open_idx]
                .element_configs
                .push(ElementConfig::Custom(declaration.custom));
        }

        // Clip
        if declaration.clip.horizontal || declaration.clip.vertical {
            self.layout_elements[open_idx]
                .element_configs
                .push(ElementConfig::Clip(declaration.clip));

            let element_id = self.layout_elements[open_idx].id;
            self.open_clip_element_stack.push(element_id as i32);

            // Find or create scroll container data
            let mut found = false;
            for scroll_data in &mut self.scroll_container_datas {
                if element_id == scroll_data.element_id {
                    scroll_data.layout_element_index = open_idx as i32;
                    scroll_data.open_this_frame = true;
                    found = true;

                    if self.external_scroll_handling_enabled {
                        if let Some(query_fn) = self.query_scroll_offset_function {
                            scroll_data.scroll_position =
                                query_fn(scroll_data.element_id, self.query_scroll_offset_user_data);
                        }
                    }
                    break;
                }
            }
            if !found {
                let mut new_scroll = ScrollContainerDataInternal::default();
                new_scroll.layout_element_index = open_idx as i32;
                new_scroll.scroll_origin = Vector2::new(-1.0, -1.0);
                new_scroll.element_id = element_id;
                new_scroll.open_this_frame = true;
                self.scroll_container_datas.push(new_scroll);
            }
        }

        // Border
        if !declaration.border.width.is_zero() {
            self.layout_elements[open_idx]
                .element_configs
                .push(ElementConfig::Border(declaration.border));
        }
    }

    /// Closes the currently open element, computes its intrinsic size from
    /// its children, and pops it from the layout stack.
    pub fn close_element(&mut self) {
        if self.boolean_warnings.max_elements_exceeded {
            return;
        }

        let open_idx = self.get_open_layout_element_index() as usize;
        let layout_config = self.layout_elements[open_idx].layout_config;

        // Check for clip/floating to pop clip stack
        let mut element_has_clip_horizontal = false;
        let mut element_has_clip_vertical = false;
        for config in &self.layout_elements[open_idx].element_configs {
            match config {
                ElementConfig::Clip(clip) => {
                    element_has_clip_horizontal = clip.horizontal;
                    element_has_clip_vertical = clip.vertical;
                    self.open_clip_element_stack.pop();
                    break;
                }
                ElementConfig::Floating(_) => {
                    self.open_clip_element_stack.pop();
                }
                _ => {}
            }
        }

        let left_right_padding =
            layout_config.padding.left as f32 + layout_config.padding.right as f32;
        let top_bottom_padding =
            layout_config.padding.top as f32 + layout_config.padding.bottom as f32;

        // Collect children from buffer
        let children_count = self.layout_elements[open_idx].children.elements.len() as i32;
        let buffer_len = self.layout_element_children_buffer.len();

        let children_indices: Vec<i32> = if children_count > 0 {
            let start = buffer_len - children_count as usize;
            self.layout_element_children_buffer[start..buffer_len].to_vec()
        } else {
            vec![]
        };

        // Store children properly
        self.layout_elements[open_idx].children.elements = children_indices.clone();

        // Copy children to layout_element_children
        match layout_config.layout_direction {
            LayoutDirection::LeftToRight => {
                let mut width = left_right_padding;
                let mut min_width = left_right_padding;
                let mut height: f32 = 0.0;
                let mut min_height: f32 = 0.0;
                for &child_index in &children_indices {
                    let child = &self.layout_elements[child_index as usize];
                    width += child.dimensions.width;
                    height = height.max(child.dimensions.height + top_bottom_padding);
                    if !element_has_clip_horizontal {
                        min_width += child.min_dimensions.width;
                    }
                    if !element_has_clip_vertical {
                        min_height =
                            min_height.max(child.min_dimensions.height + top_bottom_padding);
                    }
                    self.layout_element_children.push(child_index);
                }
                let child_gap =
                    (children_indices.len() as i32 - 1).max(0) as f32 * layout_config.child_gap as f32;
                width += child_gap;
                if !element_has_clip_horizontal {
                    min_width += child_gap;
                }
                self.layout_elements[open_idx].dimensions.width = width;
                self.layout_elements[open_idx].dimensions.height = height;
                self.layout_elements[open_idx].min_dimensions.width = min_width;
                self.layout_elements[open_idx].min_dimensions.height = min_height;
            }
            LayoutDirection::TopToBottom => {
                let mut width: f32 = 0.0;
                let mut min_width: f32 = 0.0;
                let mut height = top_bottom_padding;
                let mut min_height = top_bottom_padding;
                for &child_index in &children_indices {
                    let child = &self.layout_elements[child_index as usize];
                    height += child.dimensions.height;
                    width = width.max(child.dimensions.width + left_right_padding);
                    if !element_has_clip_vertical {
                        min_height += child.min_dimensions.height;
                    }
                    if !element_has_clip_horizontal {
                        min_width =
                            min_width.max(child.min_dimensions.width + left_right_padding);
                    }
                    self.layout_element_children.push(child_index);
                }
                let child_gap =
                    (children_indices.len() as i32 - 1).max(0) as f32 * layout_config.child_gap as f32;
                height += child_gap;
                if !element_has_clip_vertical {
                    min_height += child_gap;
                }
                self.layout_elements[open_idx].dimensions.width = width;
                self.layout_elements[open_idx].dimensions.height = height;
                self.layout_elements[open_idx].min_dimensions.width = min_width;
                self.layout_elements[open_idx].min_dimensions.height = min_height;
            }
        }

        // Remove children from buffer
        if children_count > 0 {
            self.layout_element_children_buffer
                .truncate(buffer_len - children_count as usize);
        }

        // Clamp width
        let sizing_w = &mut self.layout_elements[open_idx].layout_config.sizing.width;
        if !sizing_w.is_percent() {
            let mut mm = sizing_w.min_max();
            if mm.max <= 0.0 {
                mm.max = f32::MAX;
            }
            // Update the sizing with corrected max
            let new_sizing = match sizing_w.sizing {
                SizingType::Fit(_) => SizingType::Fit(mm),
                SizingType::Grow(_) => SizingType::Grow(mm),
                SizingType::Fixed(_) => SizingType::Fixed(mm.min),
                other => other,
            };
            sizing_w.sizing = new_sizing;

            let w = &mut self.layout_elements[open_idx].dimensions.width;
            *w = w.max(mm.min).min(mm.max);
            let mw = &mut self.layout_elements[open_idx].min_dimensions.width;
            *mw = mw.max(mm.min).min(mm.max);
        } else {
            self.layout_elements[open_idx].dimensions.width = 0.0;
        }

        // Clamp height
        let sizing_h = &mut self.layout_elements[open_idx].layout_config.sizing.height;
        if !sizing_h.is_percent() {
            let mut mm = sizing_h.min_max();
            if mm.max <= 0.0 {
                mm.max = f32::MAX;
            }
            let new_sizing = match sizing_h.sizing {
                SizingType::Fit(_) => SizingType::Fit(mm),
                SizingType::Grow(_) => SizingType::Grow(mm),
                SizingType::Fixed(_) => SizingType::Fixed(mm.min),
                other => other,
            };
            sizing_h.sizing = new_sizing;

            let h = &mut self.layout_elements[open_idx].dimensions.height;
            *h = h.max(mm.min).min(mm.max);
            let mh = &mut self.layout_elements[open_idx].min_dimensions.height;
            *mh = mh.max(mm.min).min(mm.max);
        } else {
            self.layout_elements[open_idx].dimensions.height = 0.0;
        }

        // Aspect ratio update
        self.update_aspect_ratio_box(open_idx);

        // Check if floating
        let element_is_floating = self.layout_elements[open_idx]
            .element_configs
            .iter()
            .any(|c| matches!(c, ElementConfig::Floating(_)));

        // Pop from open stack
        self.open_layout_element_stack.pop();
        let closing_element_index = open_idx as i32;

        if self.open_layout_element_stack.len() > 1 {
            if element_is_floating {
                let parent_idx = self.get_open_layout_element_index() as usize;
                self.layout_elements[parent_idx].floating_children_count += 1;
                return;
            }
            let parent_idx = self.get_open_layout_element_index() as usize;
            self.layout_elements[parent_idx]
                .children
                .elements
                .push(closing_element_index);
            self.layout_element_children_buffer
                .push(closing_element_index);
        }
    }

    pub(crate) fn update_aspect_ratio_box(&mut self, element_index: usize) {
        let element = &self.layout_elements[element_index];
        let aspect_config = element.element_configs.iter().find_map(|c| match c {
            ElementConfig::Aspect(a) => Some(*a),
            _ => None,
        });
        if let Some(aspect) = aspect_config {
            if aspect.aspect_ratio == 0.0 {
                return;
            }
            let dims = self.layout_elements[element_index].dimensions;
            if dims.width == 0.0 && dims.height != 0.0 {
                self.layout_elements[element_index].dimensions.width =
                    dims.height * aspect.aspect_ratio;
            } else if dims.width != 0.0 && dims.height == 0.0 {
                self.layout_elements[element_index].dimensions.height =
                    dims.width * (1.0 / aspect.aspect_ratio);
            }
        }
    }

    /// Opens a leaf text element, measures it, and immediately attaches it
    /// to the current parent.
    pub fn open_text_element(&mut self, text: &str, text_config: &TextElementConfig) {
        if self.layout_elements.len() >= self.max_element_count as usize - 1
            || self.boolean_warnings.max_elements_exceeded
        {
            self.boolean_warnings.max_elements_exceeded = true;
            return;
        }

        let parent_idx = self.get_open_layout_element_index() as usize;

        let text_element = LayoutElement::default();
        self.layout_elements.push(text_element.clone());
        let text_element_index = (self.layout_elements.len() - 1) as i32;

        // Set clip
        let clip_id = self.open_clip_element_stack.last().copied().unwrap_or(0);
        while self.layout_element_clip_element_ids.len() <= text_element_index as usize {
            self.layout_element_clip_element_ids.push(0);
        }
        self.layout_element_clip_element_ids[text_element_index as usize] = clip_id;

        self.layout_element_children_buffer.push(text_element_index);

        // Measure text
        let text_measured = self.measure_text_cached(text, text_config);

        // Generate ID
        let parent = &self.layout_elements[parent_idx];
        let offset =
            parent.children.elements.len() as u32 + parent.floating_children_count as u32;
        let parent_id = parent.id;
        let element_id = hash::hash_number(offset, parent_id);
        self.layout_elements[text_element_index as usize].id = element_id.id;
        self.add_hash_map_item(element_id.clone(), text_element_index);
        self.layout_element_id_strings.push(element_id.string_id);

        // Set dimensions
        let text_height = if text_config.line_height > 0 {
            text_config.line_height as f32
        } else {
            text_measured.unwrapped_dimensions.height
        };
        let text_dims = Dimensions::new(text_measured.unwrapped_dimensions.width, text_height);
        let min_dims = Dimensions::new(text_measured.min_width, text_height);

        let text_data_index = self.text_element_data.len();
        self.text_element_data.push(TextElementData {
            text: text.to_string(),
            preferred_dimensions: text_measured.unwrapped_dimensions,
            element_index: text_element_index,
            wrapped_lines_start: 0,
            wrapped_lines_count: 0,
        });

        let el = &mut self.layout_elements[text_element_index as usize];
        el.dimensions = text_dims;
        el.min_dimensions = min_dims;
        el.text_element_data_index = Some(text_data_index);
        el.element_configs
            .push(ElementConfig::Text(*text_config));
        // No need to set layout_config — text uses LAYOUT_DEFAULT

        self.layout_elements[parent_idx]
            .children
            .elements
            .push(text_element_index);
    }

    // ========================================================================
    // Begin / End Layout
    // ========================================================================

    /// Starts a new layout pass.
    ///
    /// Call this at the beginning of each frame, then build your element tree,
    /// and finish with [`end_layout`](Context::end_layout).
    pub fn begin_layout(&mut self) {
        self.initialize_ephemeral_memory();
        self.generation += 1;
        self.dynamic_element_index = 0;

        let mut root_width = self.layout_dimensions.width;
        let root_height = self.layout_dimensions.height;
        if self.debug_mode_enabled {
            root_width -= self.debug_view_width as f32;
        }
        self.boolean_warnings = BooleanWarnings::default();

        let root_id = hash::hash_string("Clay__RootContainer", 0);
        self.open_element_with_id(root_id);
        self.configure_open_element(&ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::fixed(root_width),
                    height: SizingAxis::fixed(root_height),
                },
                ..Default::default()
            },
            ..Default::default()
        });
        self.open_layout_element_stack.push(0);
        self.layout_element_tree_roots
            .push(LayoutElementTreeRoot {
                layout_element_index: 0,
                ..Default::default()
            });
    }

    /// Finishes the layout pass, runs the layout algorithm, and returns the
    /// list of render commands.
    pub fn end_layout(&mut self) -> Vec<RenderCommand> {
        self.close_element();

        if self.open_layout_element_stack.len() > 1 {
            self.report_error(
                ErrorType::UnbalancedOpenClose,
                "There were still open layout elements when EndLayout was called.",
            );
        }

        self.calculate_final_layout();
        self.render_commands.clone()
    }

    /// Begins a *shadow* layout pass that runs layout calculations without
    /// producing render commands. Useful for pre-measuring content.
    pub fn begin_shadow_layout(&mut self) {
        self.shadow_layout_mode = true;
        self.begin_layout();
    }

    /// Finishes a shadow layout pass.
    pub fn end_shadow_layout(&mut self) {
        self.close_element();
        if self.open_layout_element_stack.len() > 1 {
            self.report_error(
                ErrorType::UnbalancedOpenClose,
                "There were still open layout elements when EndShadowLayout was called.",
            );
        }
        self.calculate_final_layout();
        self.shadow_layout_mode = false;
    }

    // ========================================================================
    // Public element-building closure API
    // ========================================================================

    /// Declares a named element, executes the `children` closure to build
    /// its subtree, and closes the element automatically.
    ///
    /// This is the primary API for building UI trees.
    pub fn with_element(
        &mut self,
        id: ElementId,
        declaration: ElementDeclaration,
        children: impl FnOnce(&mut Self),
    ) {
        self.open_element_with_id(id);
        self.configure_open_element(&declaration);
        children(self);
        self.close_element();
    }

    /// Like [`with_element`](Context::with_element), but generates an
    /// automatic ID instead of requiring one.
    pub fn with_anonymous_element(
        &mut self,
        declaration: ElementDeclaration,
        children: impl FnOnce(&mut Self),
    ) {
        self.open_element();
        self.configure_open_element(&declaration);
        children(self);
        self.close_element();
    }

    /// Adds a text leaf element to the currently open parent.
    pub fn text(&mut self, text: &str, config: &TextElementConfig) {
        self.open_text_element(text, config);
    }

    // ========================================================================
    // Element ID helpers
    // ========================================================================

    /// Creates a globally unique [`ElementId`] from a string label.
    pub fn id(&self, label: &str) -> ElementId {
        hash::hash_string(label, 0)
    }

    /// Creates an indexed [`ElementId`], useful for elements generated in a
    /// loop (e.g. list items).
    pub fn idi(&self, label: &str, index: u32) -> ElementId {
        hash::hash_string_with_offset(label, index, 0)
    }

    /// Creates an [`ElementId`] scoped to the current parent element,
    /// avoiding collisions with identically-named elements elsewhere in
    /// the tree.
    pub fn id_local(&self, label: &str) -> ElementId {
        let parent_id = self.get_parent_element_id();
        hash::hash_string(label, parent_id)
    }

    fn get_parent_element_id(&self) -> u32 {
        if self.open_layout_element_stack.len() >= 2 {
            let parent_index =
                self.open_layout_element_stack[self.open_layout_element_stack.len() - 2] as usize;
            self.layout_elements[parent_index].id
        } else {
            0
        }
    }

    // ========================================================================
    // Query functions
    // ========================================================================

    /// Computes an [`ElementId`] from a string, equivalent to [`id`](Context::id).
    pub fn get_element_id(&self, id_string: &str) -> ElementId {
        hash::hash_string(id_string, 0)
    }

    /// Computes an indexed [`ElementId`], equivalent to [`idi`](Context::idi).
    pub fn get_element_id_with_index(&self, id_string: &str, index: u32) -> ElementId {
        hash::hash_string_with_offset(id_string, index, 0)
    }

    /// Queries an element's bounding box after layout has been computed.
    ///
    /// Returns [`ElementData`] with `found: true` if the element exists.
    pub fn get_element_data(&self, id: &ElementId) -> ElementData {
        if let Some(item) = self.get_hash_map_item(id.id) {
            ElementData {
                bounding_box: item.bounding_box,
                found: true,
            }
        } else {
            ElementData::default()
        }
    }

    /// Returns a reference to a render command by index.
    pub fn get_render_command(&self, index: usize) -> Option<&RenderCommand> {
        self.render_commands.get(index)
    }

    // ========================================================================
    // Render command helper
    // ========================================================================

    pub(crate) fn add_render_command(&mut self, command: RenderCommand) {
        if self.shadow_layout_mode {
            return;
        }
        if self.render_commands.len() < self.max_element_count as usize - 1 {
            self.render_commands.push(command);
        } else if !self.boolean_warnings.max_render_commands_exceeded {
            self.boolean_warnings.max_render_commands_exceeded = true;
            self.report_error(
                ErrorType::ElementsCapacityExceeded,
                "Clay ran out of capacity while attempting to create render commands.",
            );
        }
    }

    pub(crate) fn element_is_offscreen(&self, bounding_box: &BoundingBox) -> bool {
        if self.disable_culling {
            return false;
        }
        bounding_box.x > self.layout_dimensions.width
            || bounding_box.y > self.layout_dimensions.height
            || bounding_box.x + bounding_box.width < 0.0
            || bounding_box.y + bounding_box.height < 0.0
    }

    pub(crate) fn element_has_config(
        element: &LayoutElement,
        config_type: &ElementConfig,
    ) -> bool {
        element
            .element_configs
            .iter()
            .any(|c| std::mem::discriminant(c) == std::mem::discriminant(config_type))
    }

    pub(crate) fn element_has_config_type(
        element: &LayoutElement,
        check: fn(&ElementConfig) -> bool,
    ) -> bool {
        element.element_configs.iter().any(|c| check(c))
    }

    pub(crate) fn find_element_config<'a>(
        element: &'a LayoutElement,
        check: fn(&ElementConfig) -> bool,
    ) -> Option<&'a ElementConfig> {
        element.element_configs.iter().find(|c| check(c))
    }
}
