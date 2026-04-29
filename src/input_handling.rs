use crate::context::Context;
use crate::elements::*;
use crate::input::*;
use crate::types::*;

fn point_is_inside_rect(point: Vector2, rect: BoundingBox) -> bool {
    point.x >= rect.x
        && point.x <= rect.x + rect.width
        && point.y >= rect.y
        && point.y <= rect.y + rect.height
}

impl Context {
    pub fn get_pointer_over_ids(&self) -> &[ElementId] {
        &self.pointer_over_ids
    }

    pub fn set_pointer_state(&mut self, position: Vector2, is_pointer_down: bool) {
        if self.boolean_warnings.max_elements_exceeded {
            return;
        }
        self.pointer_info.position = position;
        self.pointer_over_ids.clear();

        for root_index in (0..self.layout_element_tree_roots.len()).rev() {
            let root_layout_element_index =
                self.layout_element_tree_roots[root_index].layout_element_index;
            let root_pointer_offset = self.layout_element_tree_roots[root_index].pointer_offset;

            let mut dfs_buffer: Vec<i32> = vec![root_layout_element_index];
            let mut visited: Vec<bool> = vec![false];
            let mut found = false;

            while !dfs_buffer.is_empty() {
                let stack_top = dfs_buffer.len() - 1;
                if visited[stack_top] {
                    dfs_buffer.pop();
                    visited.pop();
                    continue;
                }
                visited[stack_top] = true;

                let current_element_index = dfs_buffer[stack_top];
                if current_element_index < 0
                    || current_element_index as usize >= self.layout_elements.len()
                {
                    dfs_buffer.pop();
                    visited.pop();
                    continue;
                }

                let current_id = self.layout_elements[current_element_index as usize].id;

                let (element_box, has_hover, hover_fn, hover_data, map_eid) =
                    match self.get_hash_map_item(current_id) {
                        Some(item) => (
                            item.bounding_box,
                            item.on_hover_function.is_some(),
                            item.on_hover_function,
                            item.hover_function_user_data,
                            item.element_id.clone(),
                        ),
                        None => {
                            dfs_buffer.pop();
                            visited.pop();
                            continue;
                        }
                    };

                let mut adjusted_box = element_box;
                adjusted_box.x -= root_pointer_offset.x;
                adjusted_box.y -= root_pointer_offset.y;

                let clip_element_id =
                    if (current_element_index as usize) < self.layout_element_clip_element_ids.len()
                    {
                        self.layout_element_clip_element_ids[current_element_index as usize]
                    } else {
                        0
                    };

                let clip_box = if clip_element_id != 0 {
                    self.get_hash_map_item(clip_element_id as u32)
                        .map(|item| item.bounding_box)
                } else {
                    None
                };

                let inside_element = point_is_inside_rect(position, adjusted_box);
                let inside_clip = clip_element_id == 0
                    || clip_box
                        .map(|cb| point_is_inside_rect(position, cb))
                        .unwrap_or(false)
                    || self.external_scroll_handling_enabled;

                if inside_element && inside_clip {
                    if has_hover {
                        if let Some(callback) = hover_fn {
                            callback(map_eid.clone(), self.pointer_info, hover_data);
                        }
                    }
                    self.pointer_over_ids.push(map_eid);
                    found = true;
                }

                let is_text = Self::element_has_config_type(
                    &self.layout_elements[current_element_index as usize],
                    |c| matches!(c, ElementConfig::Text(_)),
                );
                if is_text {
                    dfs_buffer.pop();
                    visited.pop();
                    continue;
                }

                let children: Vec<i32> = self.layout_elements[current_element_index as usize]
                    .children
                    .elements
                    .clone();
                for &child_index in children.iter().rev() {
                    dfs_buffer.push(child_index);
                    visited.push(false);
                }
            }

            if found {
                let root_element =
                    &self.layout_elements[root_layout_element_index as usize];
                let is_floating_capture = root_element.element_configs.iter().any(|c| {
                    matches!(
                        c,
                        ElementConfig::Floating(f) if f.pointer_capture_mode == PointerCaptureMode::Capture
                    )
                });
                if is_floating_capture {
                    break;
                }
            }
        }

        if is_pointer_down {
            if self.pointer_info.state == PointerInteractionState::PressedThisFrame {
                self.pointer_info.state = PointerInteractionState::Pressed;
            } else if self.pointer_info.state != PointerInteractionState::Pressed {
                self.pointer_info.state = PointerInteractionState::PressedThisFrame;
            }
        } else {
            if self.pointer_info.state == PointerInteractionState::ReleasedThisFrame {
                self.pointer_info.state = PointerInteractionState::Released;
            } else if self.pointer_info.state != PointerInteractionState::Released {
                self.pointer_info.state = PointerInteractionState::ReleasedThisFrame;
            }
        }
    }

    pub fn get_scroll_offset(&mut self) -> Vector2 {
        if self.boolean_warnings.max_elements_exceeded {
            return Vector2::default();
        }
        let open_idx = self.get_open_layout_element_index();
        if open_idx < 0 || open_idx as usize >= self.layout_elements.len() {
            return Vector2::default();
        }
        let element_id = self.layout_elements[open_idx as usize].id;
        if element_id == 0 {
            return Vector2::default();
        }
        for scroll_data in &self.scroll_container_datas {
            if scroll_data.layout_element_index == open_idx {
                return scroll_data.scroll_position;
            }
        }
        Vector2::default()
    }

    pub fn update_scroll_containers(
        &mut self,
        enable_drag_scrolling: bool,
        scroll_delta: Vector2,
        delta_time: f32,
    ) {
        let is_pointer_active = enable_drag_scrolling
            && (self.pointer_info.state == PointerInteractionState::Pressed
                || self.pointer_info.state == PointerInteractionState::PressedThisFrame);

        let mut highest_priority_element_index: i32 = -1;
        let mut highest_priority_scroll_index: Option<usize> = None;

        let scroll_occurred = scroll_delta.x != 0.0 || scroll_delta.y != 0.0;

        let mut i = 0;
        while i < self.scroll_container_datas.len() {
            if !self.scroll_container_datas[i].open_this_frame {
                self.scroll_container_datas.swap_remove(i);
                continue;
            }

            let element_id = self.scroll_container_datas[i].element_id;
            self.scroll_container_datas[i].open_this_frame = false;

            if self.get_hash_map_item(element_id).is_none() {
                self.scroll_container_datas.swap_remove(i);
                continue;
            }

            // Touch / click released
            if !is_pointer_active && self.scroll_container_datas[i].pointer_scroll_active {
                let sp = self.scroll_container_datas[i].scroll_position;
                let so = self.scroll_container_datas[i].scroll_origin;
                let mt = self.scroll_container_datas[i].momentum_time;

                let x_diff = sp.x - so.x;
                if x_diff < -10.0 || x_diff > 10.0 {
                    self.scroll_container_datas[i].scroll_momentum.x =
                        (sp.x - so.x) / (mt * 25.0);
                }
                let y_diff = sp.y - so.y;
                if y_diff < -10.0 || y_diff > 10.0 {
                    self.scroll_container_datas[i].scroll_momentum.y =
                        (sp.y - so.y) / (mt * 25.0);
                }
                self.scroll_container_datas[i].pointer_scroll_active = false;
                self.scroll_container_datas[i].pointer_origin = Vector2::default();
                self.scroll_container_datas[i].scroll_origin = Vector2::default();
                self.scroll_container_datas[i].momentum_time = 0.0;
            }

            // Apply existing momentum X
            let mom_x = self.scroll_container_datas[i].scroll_momentum.x;
            self.scroll_container_datas[i].scroll_position.x += mom_x;
            self.scroll_container_datas[i].scroll_momentum.x *= 0.95;
            if (self.scroll_container_datas[i].scroll_momentum.x > -0.1
                && self.scroll_container_datas[i].scroll_momentum.x < 0.1)
                || scroll_occurred
            {
                self.scroll_container_datas[i].scroll_momentum.x = 0.0;
            }

            let el_idx = self.scroll_container_datas[i].layout_element_index;
            let (el_w, el_h) = if el_idx >= 0 && (el_idx as usize) < self.layout_elements.len() {
                let dims = self.layout_elements[el_idx as usize].dimensions;
                (dims.width, dims.height)
            } else {
                (0.0, 0.0)
            };

            let content_w = self.scroll_container_datas[i].content_size.width;
            self.scroll_container_datas[i].scroll_position.x = self.scroll_container_datas[i]
                .scroll_position
                .x
                .max(-((content_w - el_w).max(0.0)))
                .min(0.0);

            // Apply existing momentum Y
            let mom_y = self.scroll_container_datas[i].scroll_momentum.y;
            self.scroll_container_datas[i].scroll_position.y += mom_y;
            self.scroll_container_datas[i].scroll_momentum.y *= 0.95;
            if (self.scroll_container_datas[i].scroll_momentum.y > -0.1
                && self.scroll_container_datas[i].scroll_momentum.y < 0.1)
                || scroll_occurred
            {
                self.scroll_container_datas[i].scroll_momentum.y = 0.0;
            }

            let content_h = self.scroll_container_datas[i].content_size.height;
            self.scroll_container_datas[i].scroll_position.y = self.scroll_container_datas[i]
                .scroll_position
                .y
                .max(-((content_h - el_h).max(0.0)))
                .min(0.0);

            // Check if this scroll container is under the pointer
            let scroll_el_id = if el_idx >= 0 && (el_idx as usize) < self.layout_elements.len() {
                self.layout_elements[el_idx as usize].id
            } else {
                0
            };
            for j in 0..self.pointer_over_ids.len() {
                if self.pointer_over_ids[j].id == scroll_el_id && scroll_el_id != 0 {
                    highest_priority_element_index = j as i32;
                    highest_priority_scroll_index = Some(i);
                }
            }

            i += 1;
        }

        if highest_priority_element_index > -1 {
            if let Some(si) = highest_priority_scroll_index {
                let el_idx = self.scroll_container_datas[si].layout_element_index;
                if el_idx >= 0 && (el_idx as usize) < self.layout_elements.len() {
                    let clip_config = self.layout_elements[el_idx as usize]
                        .element_configs
                        .iter()
                        .find_map(|c| match c {
                            ElementConfig::Clip(clip) => Some(*clip),
                            _ => None,
                        });

                    if let Some(clip) = clip_config {
                        let content_h = self.scroll_container_datas[si].content_size.height;
                        let content_w = self.scroll_container_datas[si].content_size.width;
                        let el_dims = self.layout_elements[el_idx as usize].dimensions;

                        let can_scroll_v = clip.vertical && content_h > el_dims.height;
                        let can_scroll_h = clip.horizontal && content_w > el_dims.width;

                        // Wheel scroll
                        if can_scroll_v {
                            self.scroll_container_datas[si].scroll_position.y +=
                                scroll_delta.y * 10.0;
                        }
                        if can_scroll_h {
                            self.scroll_container_datas[si].scroll_position.x +=
                                scroll_delta.x * 10.0;
                        }

                        // Drag scroll
                        if is_pointer_active {
                            self.scroll_container_datas[si].scroll_momentum = Vector2::default();
                            if !self.scroll_container_datas[si].pointer_scroll_active {
                                self.scroll_container_datas[si].pointer_origin =
                                    self.pointer_info.position;
                                self.scroll_container_datas[si].scroll_origin =
                                    self.scroll_container_datas[si].scroll_position;
                                self.scroll_container_datas[si].pointer_scroll_active = true;
                            } else {
                                let mut sdx: f32 = 0.0;
                                let mut sdy: f32 = 0.0;
                                if can_scroll_h {
                                    let old_x =
                                        self.scroll_container_datas[si].scroll_position.x;
                                    let origin_x =
                                        self.scroll_container_datas[si].scroll_origin.x;
                                    let ptr_origin_x =
                                        self.scroll_container_datas[si].pointer_origin.x;
                                    let bb_w =
                                        self.scroll_container_datas[si].bounding_box.width;
                                    self.scroll_container_datas[si].scroll_position.x =
                                        (origin_x
                                            + (self.pointer_info.position.x - ptr_origin_x))
                                            .max(-(content_w - bb_w))
                                            .min(0.0);
                                    sdx = self.scroll_container_datas[si].scroll_position.x
                                        - old_x;
                                }
                                if can_scroll_v {
                                    let old_y =
                                        self.scroll_container_datas[si].scroll_position.y;
                                    let origin_y =
                                        self.scroll_container_datas[si].scroll_origin.y;
                                    let ptr_origin_y =
                                        self.scroll_container_datas[si].pointer_origin.y;
                                    let bb_h =
                                        self.scroll_container_datas[si].bounding_box.height;
                                    self.scroll_container_datas[si].scroll_position.y =
                                        (origin_y
                                            + (self.pointer_info.position.y - ptr_origin_y))
                                            .max(-(content_h - bb_h))
                                            .min(0.0);
                                    sdy = self.scroll_container_datas[si].scroll_position.y
                                        - old_y;
                                }
                                let mt = self.scroll_container_datas[si].momentum_time;
                                if sdx > -0.1
                                    && sdx < 0.1
                                    && sdy > -0.1
                                    && sdy < 0.1
                                    && mt > 0.15
                                {
                                    self.scroll_container_datas[si].momentum_time = 0.0;
                                    self.scroll_container_datas[si].pointer_origin =
                                        self.pointer_info.position;
                                    self.scroll_container_datas[si].scroll_origin =
                                        self.scroll_container_datas[si].scroll_position;
                                } else {
                                    self.scroll_container_datas[si].momentum_time += delta_time;
                                }
                            }
                        }

                        // Final clamp
                        if can_scroll_v {
                            self.scroll_container_datas[si].scroll_position.y =
                                self.scroll_container_datas[si]
                                    .scroll_position
                                    .y
                                    .max(-(content_h - el_dims.height))
                                    .min(0.0);
                        }
                        if can_scroll_h {
                            self.scroll_container_datas[si].scroll_position.x =
                                self.scroll_container_datas[si]
                                    .scroll_position
                                    .x
                                    .max(-(content_w - el_dims.width))
                                    .min(0.0);
                        }
                    }
                }
            }
        }
    }

    pub fn hovered(&self) -> bool {
        if self.boolean_warnings.max_elements_exceeded {
            return false;
        }
        let open_idx = self.get_open_layout_element_index();
        if open_idx < 0 || open_idx as usize >= self.layout_elements.len() {
            return false;
        }
        let element_id = self.layout_elements[open_idx as usize].id;
        if element_id == 0 {
            return false;
        }
        self.pointer_over_ids.iter().any(|eid| eid.id == element_id)
    }

    pub fn on_hover(
        &mut self,
        callback: fn(ElementId, PointerData, usize),
        user_data: usize,
    ) {
        if self.boolean_warnings.max_elements_exceeded {
            return;
        }
        let open_idx = self.get_open_layout_element_index();
        if open_idx < 0 || open_idx as usize >= self.layout_elements.len() {
            return;
        }
        let element_id = self.layout_elements[open_idx as usize].id;
        if element_id == 0 {
            return;
        }
        if let Some(item) = self.get_hash_map_item_mut(element_id) {
            item.on_hover_function = Some(callback);
            item.hover_function_user_data = user_data;
        }
    }

    pub fn pointer_over(&self, element_id: &ElementId) -> bool {
        self.pointer_over_ids
            .iter()
            .any(|eid| eid.id == element_id.id)
    }

    pub fn get_child_insert_index(&self, container_id: &ElementId) -> i32 {
        if self.get_hash_map_item(container_id.id).is_none() {
            return -1;
        }

        let container_element = match self
            .layout_elements
            .iter()
            .find(|e| e.id == container_id.id)
        {
            Some(e) => e,
            None => return -1,
        };

        let pointer = self.pointer_info.position;
        let is_horizontal = container_element.layout_config.layout_direction
            == crate::layout::LayoutDirection::LeftToRight;

        let child_count = container_element.children.elements.len();
        if child_count == 0 {
            return 0;
        }

        for i in 0..child_count {
            let child_index = container_element.children.elements[i];
            if child_index < 0 || child_index as usize >= self.layout_elements.len() {
                continue;
            }
            let child_id = self.layout_elements[child_index as usize].id;
            let child_box = match self.get_hash_map_item(child_id) {
                Some(item) => item.bounding_box,
                None => continue,
            };

            if is_horizontal {
                let mid_x = child_box.x + child_box.width / 2.0;
                if pointer.x < mid_x {
                    return i as i32;
                }
            } else {
                let mid_y = child_box.y + child_box.height / 2.0;
                if pointer.y < mid_y {
                    return i as i32;
                }
            }
        }

        child_count as i32
    }

    pub fn get_scroll_container_data(&self, id: &ElementId) -> ScrollContainerData {
        for (i, scroll_data) in self.scroll_container_datas.iter().enumerate() {
            if scroll_data.element_id == id.id {
                let el_idx = scroll_data.layout_element_index;
                if el_idx < 0 || el_idx as usize >= self.layout_elements.len() {
                    return ScrollContainerData::default();
                }
                let scroll_element = &self.layout_elements[el_idx as usize];
                let clip_config = scroll_element.element_configs.iter().find_map(|c| match c {
                    ElementConfig::Clip(clip) => Some(*clip),
                    _ => None,
                });
                let clip = match clip_config {
                    Some(c) => c,
                    None => return ScrollContainerData::default(),
                };
                return ScrollContainerData {
                    scroll_position: scroll_data.scroll_position,
                    scroll_container_dimensions: Dimensions::new(
                        scroll_data.bounding_box.width,
                        scroll_data.bounding_box.height,
                    ),
                    content_dimensions: scroll_data.content_size,
                    config: clip,
                    found: true,
                    scroll_container_index: Some(i),
                };
            }
        }
        ScrollContainerData::default()
    }
}
