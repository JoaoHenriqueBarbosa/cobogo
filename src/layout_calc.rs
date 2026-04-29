use crate::arena::*;
use crate::context::Context;
use crate::elements::*;
use crate::hash;
use crate::layout::*;
use crate::render::*;
use crate::types::*;

const EPSILON: f32 = 0.01;

fn safe_substr(s: &str, offset: usize, len: usize) -> String {
    let bytes = s.as_bytes();
    let start = offset.min(bytes.len());
    let end = (offset + len).min(bytes.len());
    String::from_utf8_lossy(&bytes[start..end]).to_string()
}

fn float_equal(left: f32, right: f32) -> bool {
    (left - right).abs() < EPSILON
}

impl Context {
    pub(crate) fn size_containers_along_axis(&mut self, x_axis: bool) {
        for root_index in 0..self.layout_element_tree_roots.len() {
            let root_layout_index = self.layout_element_tree_roots[root_index].layout_element_index as usize;

            // Size floating containers to their parents
            let is_floating = Self::element_has_config_type(
                &self.layout_elements[root_layout_index],
                |c| matches!(c, ElementConfig::Floating(_)),
            );
            if is_floating {
                let parent_id = self.layout_elements[root_layout_index]
                    .element_configs
                    .iter()
                    .find_map(|c| match c {
                        ElementConfig::Floating(f) => Some(f.parent_id),
                        _ => None,
                    })
                    .unwrap_or(0);

                if let Some(parent_item) = self.get_hash_map_item(parent_id) {
                    let parent_le_idx = parent_item.layout_element_index as usize;
                    let parent_dims = self.layout_elements[parent_le_idx].dimensions;
                    let root_sizing = self.layout_elements[root_layout_index].layout_config.sizing;

                    if x_axis {
                        match root_sizing.width.sizing {
                            SizingType::Grow(_) => {
                                self.layout_elements[root_layout_index].dimensions.width = parent_dims.width;
                            }
                            SizingType::Percent(p) => {
                                self.layout_elements[root_layout_index].dimensions.width = parent_dims.width * p;
                            }
                            _ => {}
                        }
                    } else {
                        match root_sizing.height.sizing {
                            SizingType::Grow(_) => {
                                self.layout_elements[root_layout_index].dimensions.height = parent_dims.height;
                            }
                            SizingType::Percent(p) => {
                                self.layout_elements[root_layout_index].dimensions.height = parent_dims.height * p;
                            }
                            _ => {}
                        }
                    }
                }
            }

            // Clamp root element
            self.clamp_element_axis(root_layout_index, true);
            self.clamp_element_axis(root_layout_index, false);

            // BFS through children
            let mut bfs_buffer: Vec<usize> = vec![root_layout_index];
            let mut bfs_read = 0;

            while bfs_read < bfs_buffer.len() {
                let parent_index = bfs_buffer[bfs_read];
                bfs_read += 1;

                self.size_children_of_parent(parent_index, x_axis, &mut bfs_buffer);
            }
        }
    }

    fn size_children_of_parent(&mut self, parent_index: usize, x_axis: bool, bfs_buffer: &mut Vec<usize>) {
        let parent_config = self.layout_elements[parent_index].layout_config;
        let parent_size = if x_axis { self.layout_elements[parent_index].dimensions.width } else { self.layout_elements[parent_index].dimensions.height };
        let parent_padding = if x_axis {
            (parent_config.padding.left + parent_config.padding.right) as f32
        } else {
            (parent_config.padding.top + parent_config.padding.bottom) as f32
        };
        let sizing_along_axis = (x_axis && parent_config.layout_direction == LayoutDirection::LeftToRight)
            || (!x_axis && parent_config.layout_direction == LayoutDirection::TopToBottom);
        let parent_child_gap = parent_config.child_gap as f32;

        let children: Vec<i32> = self.layout_elements[parent_index].children.elements.clone();

        let mut grow_container_count = 0i32;
        let mut inner_content_size: f32 = 0.0;
        let mut total_padding_and_child_gaps = parent_padding;
        let mut resizable_indices: Vec<usize> = Vec::new();

        for (child_offset, &child_element_index) in children.iter().enumerate() {
            let ci = child_element_index as usize;
            let child_sizing = if x_axis {
                self.layout_elements[ci].layout_config.sizing.width.sizing
            } else {
                self.layout_elements[ci].layout_config.sizing.height.sizing
            };
            let child_size = if x_axis { self.layout_elements[ci].dimensions.width } else { self.layout_elements[ci].dimensions.height };

            let is_text = Self::element_has_config_type(&self.layout_elements[ci], |c| matches!(c, ElementConfig::Text(_)));
            if !is_text && !self.layout_elements[ci].children.elements.is_empty() {
                bfs_buffer.push(ci);
            }

            let is_resizable = !matches!(child_sizing, SizingType::Percent(_) | SizingType::Fixed(_))
                && (!is_text || {
                    self.layout_elements[ci].element_configs.iter().any(|c| match c {
                        ElementConfig::Text(t) => t.wrap_mode == TextWrapMode::Words,
                        _ => false,
                    })
                });
            if is_resizable {
                resizable_indices.push(ci);
            }

            if sizing_along_axis {
                if !matches!(child_sizing, SizingType::Percent(_)) {
                    inner_content_size += child_size;
                }
                if matches!(child_sizing, SizingType::Grow(_)) {
                    grow_container_count += 1;
                }
                if child_offset > 0 {
                    inner_content_size += parent_child_gap;
                    total_padding_and_child_gaps += parent_child_gap;
                }
            } else {
                inner_content_size = inner_content_size.max(child_size);
            }
        }

        // Expand percentage containers
        for &child_element_index in &children {
            let ci = child_element_index as usize;
            let child_sizing = if x_axis {
                self.layout_elements[ci].layout_config.sizing.width.sizing
            } else {
                self.layout_elements[ci].layout_config.sizing.height.sizing
            };
            if let SizingType::Percent(p) = child_sizing {
                let new_size = (parent_size - total_padding_and_child_gaps) * p;
                if x_axis { self.layout_elements[ci].dimensions.width = new_size; }
                else { self.layout_elements[ci].dimensions.height = new_size; }
                if sizing_along_axis { inner_content_size += new_size; }
                self.update_aspect_ratio_box(ci);
            }
        }

        if sizing_along_axis {
            let size_to_distribute = parent_size - parent_padding - inner_content_size;
            if size_to_distribute < 0.0 {
                // Check if parent clips
                let clips_axis = self.layout_elements[parent_index].element_configs.iter().any(|c| match c {
                    ElementConfig::Clip(clip) => if x_axis { clip.horizontal } else { clip.vertical },
                    _ => false,
                });
                if !clips_axis {
                    self.compress_children(&mut resizable_indices, x_axis, size_to_distribute);
                }
            } else if size_to_distribute > 0.0 && grow_container_count > 0 {
                resizable_indices.retain(|&ci| {
                    let s = if x_axis { self.layout_elements[ci].layout_config.sizing.width.sizing } else { self.layout_elements[ci].layout_config.sizing.height.sizing };
                    matches!(s, SizingType::Grow(_))
                });
                self.expand_children(&mut resizable_indices, x_axis, size_to_distribute);
            }
        } else {
            // Off-axis sizing
            for &ci in &resizable_indices {
                let child_sizing = if x_axis { self.layout_elements[ci].layout_config.sizing.width.sizing } else { self.layout_elements[ci].layout_config.sizing.height.sizing };
                let min_size = if x_axis { self.layout_elements[ci].min_dimensions.width } else { self.layout_elements[ci].min_dimensions.height };
                let mut max_size = parent_size - parent_padding;

                let clips_axis = self.layout_elements[parent_index].element_configs.iter().any(|c| match c {
                    ElementConfig::Clip(clip) => if x_axis { clip.horizontal } else { clip.vertical },
                    _ => false,
                });
                if clips_axis { max_size = max_size.max(inner_content_size); }

                let child_size = if x_axis { &mut self.layout_elements[ci].dimensions.width } else { &mut self.layout_elements[ci].dimensions.height };
                if matches!(child_sizing, SizingType::Grow(mm) if true) {
                    if let SizingType::Grow(mm) = child_sizing {
                        *child_size = max_size.min(if mm.max <= 0.0 { f32::MAX } else { mm.max });
                    }
                }
                *child_size = min_size.max(child_size.min(max_size));
            }
        }
    }

    fn compress_children(&mut self, resizable: &mut Vec<usize>, x_axis: bool, mut size_to_distribute: f32) {
        while size_to_distribute < -EPSILON && !resizable.is_empty() {
            let mut largest: f32 = 0.0;
            let mut second_largest: f32 = 0.0;
            let mut width_to_add = size_to_distribute;

            for &ci in resizable.iter() {
                let cs = if x_axis { self.layout_elements[ci].dimensions.width } else { self.layout_elements[ci].dimensions.height };
                if float_equal(cs, largest) { continue; }
                if cs > largest { second_largest = largest; largest = cs; }
                if cs < largest { second_largest = second_largest.max(cs); width_to_add = second_largest - largest; }
            }
            width_to_add = width_to_add.max(size_to_distribute / resizable.len() as f32);

            let mut i = 0;
            while i < resizable.len() {
                let ci = resizable[i];
                let min_size = if x_axis { self.layout_elements[ci].min_dimensions.width } else { self.layout_elements[ci].min_dimensions.height };
                let child_size = if x_axis { &mut self.layout_elements[ci].dimensions.width } else { &mut self.layout_elements[ci].dimensions.height };
                let previous = *child_size;
                if float_equal(*child_size, largest) {
                    *child_size += width_to_add;
                    if *child_size <= min_size {
                        *child_size = min_size;
                        resizable.remove(i);
                        continue;
                    }
                    size_to_distribute -= *child_size - previous;
                }
                i += 1;
            }
        }
    }

    fn expand_children(&mut self, resizable: &mut Vec<usize>, x_axis: bool, mut size_to_distribute: f32) {
        while size_to_distribute > EPSILON && !resizable.is_empty() {
            let mut smallest: f32 = f32::MAX;
            let mut second_smallest: f32 = f32::MAX;
            let mut width_to_add = size_to_distribute;

            for &ci in resizable.iter() {
                let cs = if x_axis { self.layout_elements[ci].dimensions.width } else { self.layout_elements[ci].dimensions.height };
                if float_equal(cs, smallest) { continue; }
                if cs < smallest { second_smallest = smallest; smallest = cs; }
                if cs > smallest { second_smallest = second_smallest.min(cs); width_to_add = second_smallest - smallest; }
            }
            width_to_add = width_to_add.min(size_to_distribute / resizable.len() as f32);

            let mut i = 0;
            while i < resizable.len() {
                let ci = resizable[i];
                let max_size = if x_axis {
                    let mm = self.layout_elements[ci].layout_config.sizing.width.min_max();
                    if mm.max <= 0.0 { f32::MAX } else { mm.max }
                } else {
                    let mm = self.layout_elements[ci].layout_config.sizing.height.min_max();
                    if mm.max <= 0.0 { f32::MAX } else { mm.max }
                };
                let child_size = if x_axis { &mut self.layout_elements[ci].dimensions.width } else { &mut self.layout_elements[ci].dimensions.height };
                let previous = *child_size;
                if float_equal(*child_size, smallest) {
                    *child_size += width_to_add;
                    if *child_size >= max_size {
                        *child_size = max_size;
                        resizable.remove(i);
                        continue;
                    }
                    size_to_distribute -= *child_size - previous;
                }
                i += 1;
            }
        }
    }

    fn clamp_element_axis(&mut self, idx: usize, x_axis: bool) {
        let sizing = if x_axis {
            self.layout_elements[idx].layout_config.sizing.width
        } else {
            self.layout_elements[idx].layout_config.sizing.height
        };
        if !sizing.is_percent() {
            let mm = sizing.min_max();
            let dim = if x_axis {
                &mut self.layout_elements[idx].dimensions.width
            } else {
                &mut self.layout_elements[idx].dimensions.height
            };
            *dim = dim.max(mm.min).min(if mm.max <= 0.0 { f32::MAX } else { mm.max });
        }
    }

    pub(crate) fn calculate_final_layout(&mut self) {
        // 1. Size along X axis
        self.size_containers_along_axis(true);

        // 2. Wrap text
        self.wrap_text();

        // 3. Scale vertical heights for aspect ratio
        self.scale_aspect_ratio_vertical();

        // 4. Propagate height changes
        self.propagate_height_changes();

        // 5. Size along Y axis
        self.size_containers_along_axis(false);

        // 6. Scale horizontal widths for aspect ratio
        self.scale_aspect_ratio_horizontal();

        // 7. Sort tree roots by z-index
        self.sort_tree_roots_by_z_index();

        // 8. Calculate final positions and generate render commands
        self.generate_render_commands();
    }

    fn wrap_text(&mut self) {
        let measure_fn = match self.measure_text_function {
            Some(f) => f,
            None => return,
        };
        let user_data = self.measure_text_user_data;

        for text_idx in 0..self.text_element_data.len() {
            let wrapped_start = self.wrapped_text_lines.len();
            self.text_element_data[text_idx].wrapped_lines_start = wrapped_start;
            self.text_element_data[text_idx].wrapped_lines_count = 0;

            let element_index = self.text_element_data[text_idx].element_index as usize;
            let container_width = self.layout_elements[element_index].dimensions.width;
            let text = self.text_element_data[text_idx].text.clone();
            let pref_dims = self.text_element_data[text_idx].preferred_dimensions;

            let text_config = self.layout_elements[element_index]
                .element_configs.iter().find_map(|c| match c {
                    ElementConfig::Text(t) => Some(*t),
                    _ => None,
                }).unwrap_or_default();

            let cache_item = self.measure_text_cached(&text, &text_config);
            let line_height = if text_config.line_height > 0 {
                text_config.line_height as f32
            } else {
                pref_dims.height
            };

            if !cache_item.contains_newlines && pref_dims.width <= container_width {
                self.wrapped_text_lines.push(WrappedTextLine {
                    dimensions: Dimensions::new(container_width, line_height),
                    text: text.clone(),
                });
                self.text_element_data[text_idx].wrapped_lines_count = 1;
                continue;
            }

            let space_width = measure_fn(" ", &text_config, user_data).width;
            let mut word_index = cache_item.measured_words_start_index;
            let mut line_width: f32 = 0.0;
            let mut line_length_chars: i32 = 0;
            let mut line_start_offset: usize = 0;

            while word_index != -1 {
                if word_index < 0 || word_index as usize >= self.measured_words.len() { break; }
                let word = self.measured_words[word_index as usize].clone();

                if line_length_chars == 0 && line_width + word.width > container_width {
                    self.wrapped_text_lines.push(WrappedTextLine {
                        dimensions: Dimensions::new(word.width, line_height),
                        text: safe_substr(&text, word.start_offset as usize, word.length as usize),
                    });
                    self.text_element_data[text_idx].wrapped_lines_count += 1;
                    word_index = word.next;
                    line_start_offset = (word.start_offset + word.length) as usize;
                } else if word.length == 0 || line_width + word.width > container_width {
                    let final_char_is_space = line_start_offset + line_length_chars as usize > 0
                        && text.as_bytes().get(line_start_offset + line_length_chars as usize - 1) == Some(&b' ');
                    let adj_width = if final_char_is_space { line_width - space_width } else { line_width };
                    let adj_len = if final_char_is_space { line_length_chars - 1 } else { line_length_chars };

                    self.wrapped_text_lines.push(WrappedTextLine {
                        dimensions: Dimensions::new(adj_width, line_height),
                        text: safe_substr(&text, line_start_offset, adj_len.max(0) as usize),
                    });
                    self.text_element_data[text_idx].wrapped_lines_count += 1;

                    if line_length_chars == 0 || word.length == 0 {
                        word_index = word.next;
                    }
                    line_width = 0.0;
                    line_length_chars = 0;
                    line_start_offset = word.start_offset as usize;
                } else {
                    line_width += word.width + text_config.letter_spacing as f32;
                    line_length_chars += word.length;
                    word_index = word.next;
                }
            }

            if line_length_chars > 0 {
                self.wrapped_text_lines.push(WrappedTextLine {
                    dimensions: Dimensions::new(line_width - text_config.letter_spacing as f32, line_height),
                    text: safe_substr(&text, line_start_offset, line_length_chars as usize),
                });
                self.text_element_data[text_idx].wrapped_lines_count += 1;
            }

            let line_count = self.text_element_data[text_idx].wrapped_lines_count;
            self.layout_elements[element_index].dimensions.height = line_height * line_count as f32;
        }
    }

    fn scale_aspect_ratio_vertical(&mut self) {
        let indices: Vec<i32> = self.aspect_ratio_element_indexes.clone();
        for &idx in &indices {
            let i = idx as usize;
            let aspect = self.layout_elements[i].element_configs.iter().find_map(|c| match c {
                ElementConfig::Aspect(a) => Some(a.aspect_ratio),
                _ => None,
            });
            if let Some(ratio) = aspect {
                if ratio > 0.0 {
                    self.layout_elements[i].dimensions.height = (1.0 / ratio) * self.layout_elements[i].dimensions.width;
                    let h = self.layout_elements[i].dimensions.height;
                    match &mut self.layout_elements[i].layout_config.sizing.height.sizing {
                        SizingType::Fit(mm) | SizingType::Grow(mm) => mm.max = h,
                        _ => {}
                    }
                }
            }
        }
    }

    fn propagate_height_changes(&mut self) {
        let mut dfs: Vec<(usize, bool)> = Vec::new();
        for i in 0..self.layout_element_tree_roots.len() {
            let idx = self.layout_element_tree_roots[i].layout_element_index as usize;
            dfs.push((idx, false));
        }

        while let Some(&(current_idx, visited)) = dfs.last() {
            if !visited {
                dfs.last_mut().unwrap().1 = true;
                let element = &self.layout_elements[current_idx];
                let is_text = Self::element_has_config_type(element, |c| matches!(c, ElementConfig::Text(_)));
                if is_text || element.children.elements.is_empty() {
                    dfs.pop();
                    continue;
                }
                let children: Vec<i32> = element.children.elements.clone();
                for &child_idx in &children {
                    dfs.push((child_idx as usize, false));
                }
                continue;
            }
            dfs.pop();

            let config = self.layout_elements[current_idx].layout_config;
            let children: Vec<i32> = self.layout_elements[current_idx].children.elements.clone();

            match config.layout_direction {
                LayoutDirection::LeftToRight => {
                    for &child_idx in &children {
                        let child_h = self.layout_elements[child_idx as usize].dimensions.height;
                        let h_with_pad = (child_h + config.padding.top as f32 + config.padding.bottom as f32)
                            .max(self.layout_elements[current_idx].dimensions.height);
                        let mm = config.sizing.height.min_max();
                        let max = if mm.max <= 0.0 { f32::MAX } else { mm.max };
                        self.layout_elements[current_idx].dimensions.height = h_with_pad.max(mm.min).min(max);
                    }
                }
                LayoutDirection::TopToBottom => {
                    let mut content_h = (config.padding.top + config.padding.bottom) as f32;
                    for &child_idx in &children {
                        content_h += self.layout_elements[child_idx as usize].dimensions.height;
                    }
                    content_h += (children.len() as i32 - 1).max(0) as f32 * config.child_gap as f32;
                    let mm = config.sizing.height.min_max();
                    let max = if mm.max <= 0.0 { f32::MAX } else { mm.max };
                    self.layout_elements[current_idx].dimensions.height = content_h.max(mm.min).min(max);
                }
            }
        }
    }

    fn scale_aspect_ratio_horizontal(&mut self) {
        let indices: Vec<i32> = self.aspect_ratio_element_indexes.clone();
        for &idx in &indices {
            let i = idx as usize;
            let aspect = self.layout_elements[i].element_configs.iter().find_map(|c| match c {
                ElementConfig::Aspect(a) => Some(a.aspect_ratio),
                _ => None,
            });
            if let Some(ratio) = aspect {
                if ratio > 0.0 {
                    self.layout_elements[i].dimensions.width = ratio * self.layout_elements[i].dimensions.height;
                }
            }
        }
    }

    fn sort_tree_roots_by_z_index(&mut self) {
        let len = self.layout_element_tree_roots.len();
        if len <= 1 { return; }
        for sort_max in (1..len).rev() {
            for i in 0..sort_max {
                if self.layout_element_tree_roots[i + 1].z_index < self.layout_element_tree_roots[i].z_index {
                    self.layout_element_tree_roots.swap(i, i + 1);
                }
            }
        }
    }

    fn generate_render_commands(&mut self) {
        self.render_commands.clear();

        for root_index in 0..self.layout_element_tree_roots.len() {
            let root = self.layout_element_tree_roots[root_index].clone();
            let root_element_index = root.layout_element_index as usize;
            let root_element_id = self.layout_elements[root_element_index].id;
            let root_children_len = self.layout_elements[root_element_index].children.elements.len() as u32;
            let mut root_position = Vector2::default();

            // Position floating containers
            self.position_floating_root(root_element_index, &root, &mut root_position);

            // Clip for floating elements with clip
            if root.clip_element_id != 0 {
                if let Some(clip_item) = self.get_hash_map_item(root.clip_element_id) {
                    let clip_box = clip_item.bounding_box;
                    let clip_le_idx = clip_item.layout_element_index as usize;

                    if self.external_scroll_handling_enabled {
                        if let Some(clip_config) = self.layout_elements[clip_le_idx].element_configs.iter().find_map(|c| match c {
                            ElementConfig::Clip(cc) => Some(*cc),
                            _ => None,
                        }) {
                            if clip_config.horizontal { root_position.x += clip_config.child_offset.x; }
                            if clip_config.vertical { root_position.y += clip_config.child_offset.y; }
                        }
                    }

                    self.add_render_command(RenderCommand {
                        bounding_box: clip_box,
                        id: hash::hash_number(root_element_id, root_children_len + 10).id,
                        z_index: root.z_index,
                        render_data: RenderData::None,
                        ..Default::default()
                    });
                }
            }

            // DFS
            let root_padding = self.layout_elements[root_element_index].layout_config.padding;
            let mut dfs: Vec<DfsNode> = vec![DfsNode {
                element_index: root_element_index,
                position: root_position,
                next_child_offset: Vector2::new(root_padding.left as f32, root_padding.top as f32),
                visited: false,
            }];

            while let Some(node) = dfs.last_mut() {
                let node_idx = node.element_index;
                let node_pos = node.position;
                let z_index = root.z_index;

                if !node.visited {
                    node.visited = true;
                    self.process_dfs_downward(&mut dfs, node_idx, node_pos, z_index, root_element_index, &root);
                } else {
                    self.process_dfs_upward(&mut dfs, node_idx, z_index, root_element_index, &root);
                }
            }

            // End scissor for root clip
            if root.clip_element_id != 0 {
                self.add_render_command(RenderCommand {
                    id: hash::hash_number(root_element_id, root_children_len + 11).id,
                    render_data: RenderData::None,
                    ..Default::default()
                });
            }
        }
    }

    fn position_floating_root(&self, root_idx: usize, root: &LayoutElementTreeRoot, root_position: &mut Vector2) {
        let is_floating = Self::element_has_config_type(&self.layout_elements[root_idx], |c| matches!(c, ElementConfig::Floating(_)));
        if !is_floating { return; }

        let parent_item = match self.get_hash_map_item(root.parent_id) {
            Some(item) => item,
            None => return,
        };
        let config = match self.layout_elements[root_idx].element_configs.iter().find_map(|c| match c {
            ElementConfig::Floating(f) => Some(*f),
            _ => None,
        }) {
            Some(c) => c,
            None => return,
        };

        let root_dims = self.layout_elements[root_idx].dimensions;
        let parent_box = parent_item.bounding_box;
        let mut target = Vector2::default();

        // X from parent attach point
        match config.attach_points.parent {
            FloatingAttachPointType::LeftTop | FloatingAttachPointType::LeftCenter | FloatingAttachPointType::LeftBottom => target.x = parent_box.x,
            FloatingAttachPointType::CenterTop | FloatingAttachPointType::CenterCenter | FloatingAttachPointType::CenterBottom => target.x = parent_box.x + parent_box.width / 2.0,
            FloatingAttachPointType::RightTop | FloatingAttachPointType::RightCenter | FloatingAttachPointType::RightBottom => target.x = parent_box.x + parent_box.width,
        }
        // X from element attach point
        match config.attach_points.element {
            FloatingAttachPointType::CenterTop | FloatingAttachPointType::CenterCenter | FloatingAttachPointType::CenterBottom => target.x -= root_dims.width / 2.0,
            FloatingAttachPointType::RightTop | FloatingAttachPointType::RightCenter | FloatingAttachPointType::RightBottom => target.x -= root_dims.width,
            _ => {}
        }
        // Y from parent attach point
        match config.attach_points.parent {
            FloatingAttachPointType::LeftTop | FloatingAttachPointType::CenterTop | FloatingAttachPointType::RightTop => target.y = parent_box.y,
            FloatingAttachPointType::LeftCenter | FloatingAttachPointType::CenterCenter | FloatingAttachPointType::RightCenter => target.y = parent_box.y + parent_box.height / 2.0,
            FloatingAttachPointType::LeftBottom | FloatingAttachPointType::CenterBottom | FloatingAttachPointType::RightBottom => target.y = parent_box.y + parent_box.height,
        }
        // Y from element attach point
        match config.attach_points.element {
            FloatingAttachPointType::LeftCenter | FloatingAttachPointType::CenterCenter | FloatingAttachPointType::RightCenter => target.y -= root_dims.height / 2.0,
            FloatingAttachPointType::LeftBottom | FloatingAttachPointType::CenterBottom | FloatingAttachPointType::RightBottom => target.y -= root_dims.height,
            _ => {}
        }

        target.x += config.offset.x;
        target.y += config.offset.y;
        *root_position = target;
    }

    fn process_dfs_downward(
        &mut self, dfs: &mut Vec<DfsNode>, node_idx: usize, node_pos: Vector2,
        z_index: i16, _root_element_index: usize, root: &LayoutElementTreeRoot,
    ) {
        // Extract all needed data from element upfront to avoid borrow conflicts
        let layout_config = self.layout_elements[node_idx].layout_config;
        let dims = self.layout_elements[node_idx].dimensions;
        let element_id = self.layout_elements[node_idx].id;
        let configs: Vec<ElementConfig> = self.layout_elements[node_idx].element_configs.clone();

        let mut bbox = BoundingBox::new(node_pos.x, node_pos.y, dims.width, dims.height);

        // Apply floating expand
        if let Some(floating) = configs.iter().find_map(|c| match c {
            ElementConfig::Floating(f) => Some(*f), _ => None,
        }) {
            bbox.x -= floating.expand.width;
            bbox.width += floating.expand.width * 2.0;
            bbox.y -= floating.expand.height;
            bbox.height += floating.expand.height * 2.0;
        }

        // Update scroll container
        let mut scroll_offset = Vector2::default();
        if let Some(clip_config) = configs.iter().find_map(|c| match c {
            ElementConfig::Clip(cc) => Some(*cc), _ => None,
        }) {
            for sd in &mut self.scroll_container_datas {
                if sd.layout_element_index == node_idx as i32 {
                    sd.bounding_box = bbox;
                    scroll_offset = clip_config.child_offset;
                    if self.external_scroll_handling_enabled {
                        scroll_offset = Vector2::default();
                    }
                    break;
                }
            }
        }

        // Update hash map bounding box
        if let Some(item) = self.get_hash_map_item_mut(element_id) {
            item.bounding_box = bbox;
        }

        // Determine shared config
        let shared = configs.iter().find_map(|c| match c {
            ElementConfig::Shared(s) => Some(*s), _ => None,
        }).unwrap_or_default();

        let offscreen = self.element_is_offscreen(&bbox);
        let mut emit_rectangle = shared.background_color.a > 0.0;
        for config in &configs {
            let should_render = !offscreen;
            match config {
                ElementConfig::Clip(cc) => {
                    if should_render {
                        self.add_render_command(RenderCommand {
                            bounding_box: bbox,
                            render_data: RenderData::Clip(ClipRenderData { horizontal: cc.horizontal, vertical: cc.vertical }),
                            user_data: shared.user_data, id: element_id, z_index,
                            ..Default::default()
                        });
                    }
                }
                ElementConfig::Image(img) => {
                    if should_render {
                        self.add_render_command(RenderCommand {
                            bounding_box: bbox,
                            render_data: RenderData::Image(ImageRenderData {
                                background_color: shared.background_color,
                                corner_radius: shared.corner_radius,
                                image_data: img.image_data,
                            }),
                            user_data: shared.user_data, id: element_id, z_index,
                            ..Default::default()
                        });
                    }
                    emit_rectangle = false;
                }
                ElementConfig::Text(text_config) => {
                    if should_render {
                        self.emit_text_render_commands(node_idx, bbox, text_config, z_index, root);
                    }
                }
                ElementConfig::Custom(custom) => {
                    if should_render {
                        self.add_render_command(RenderCommand {
                            bounding_box: bbox,
                            render_data: RenderData::Custom(CustomRenderData {
                                background_color: shared.background_color,
                                corner_radius: shared.corner_radius,
                                custom_data: custom.custom_data,
                            }),
                            user_data: shared.user_data, id: element_id, z_index,
                            ..Default::default()
                        });
                    }
                    emit_rectangle = false;
                }
                _ => {}
            }
        }

        if emit_rectangle && !offscreen {
            self.add_render_command(RenderCommand {
                bounding_box: bbox,
                render_data: RenderData::Rectangle(RectangleRenderData {
                    background_color: shared.background_color,
                    corner_radius: shared.corner_radius,
                }),
                user_data: shared.user_data, id: element_id, z_index,
                ..Default::default()
            });
        }

        // Calculate content alignment and add children to DFS
        let is_text = Self::element_has_config_type(&self.layout_elements[node_idx], |c| matches!(c, ElementConfig::Text(_)));
        if is_text { return; }

        let children: Vec<i32> = self.layout_elements[node_idx].children.elements.clone();
        let dfs_node = dfs.last_mut().unwrap();

        // On-axis alignment
        let mut content_size = Dimensions::default();
        if layout_config.layout_direction == LayoutDirection::LeftToRight {
            for &ci in &children {
                let child = &self.layout_elements[ci as usize];
                content_size.width += child.dimensions.width;
                content_size.height = content_size.height.max(child.dimensions.height);
            }
            content_size.width += (children.len() as i32 - 1).max(0) as f32 * layout_config.child_gap as f32;
            let extra = (dims.width - (layout_config.padding.left + layout_config.padding.right) as f32 - content_size.width).max(0.0);
            let offset = match layout_config.child_alignment.x {
                AlignmentX::Left => 0.0,
                AlignmentX::Center => extra / 2.0,
                AlignmentX::Right => extra,
            };
            dfs_node.next_child_offset.x += offset;
        } else {
            for &ci in &children {
                let child = &self.layout_elements[ci as usize];
                content_size.width = content_size.width.max(child.dimensions.width);
                content_size.height += child.dimensions.height;
            }
            content_size.height += (children.len() as i32 - 1).max(0) as f32 * layout_config.child_gap as f32;
            let extra = (dims.height - (layout_config.padding.top + layout_config.padding.bottom) as f32 - content_size.height).max(0.0);
            let offset = match layout_config.child_alignment.y {
                AlignmentY::Top => 0.0,
                AlignmentY::Center => extra / 2.0,
                AlignmentY::Bottom => extra,
            };
            dfs_node.next_child_offset.y += offset;
        }

        // Update scroll container content size
        for sd in &mut self.scroll_container_datas {
            if sd.layout_element_index == node_idx as i32 {
                sd.content_size = Dimensions::new(
                    content_size.width + (layout_config.padding.left + layout_config.padding.right) as f32,
                    content_size.height + (layout_config.padding.top + layout_config.padding.bottom) as f32,
                );
                break;
            }
        }

        // Add children in reverse (stack-based DFS)
        let current_next_offset = dfs.last().unwrap().next_child_offset;
        let current_pos = dfs.last().unwrap().position;

        let mut running_offset = current_next_offset;
        for (i, &ci) in children.iter().enumerate() {
            let child = &self.layout_elements[ci as usize];
            let child_dims = child.dimensions;
            let child_padding = child.layout_config.padding;

            // Off-axis alignment
            let mut child_offset = running_offset;
            if layout_config.layout_direction == LayoutDirection::LeftToRight {
                child_offset.y = layout_config.padding.top as f32;
                let space = dims.height - (layout_config.padding.top + layout_config.padding.bottom) as f32 - child_dims.height;
                match layout_config.child_alignment.y {
                    AlignmentY::Center => child_offset.y += space / 2.0,
                    AlignmentY::Bottom => child_offset.y += space,
                    AlignmentY::Top => {}
                }
            } else {
                child_offset.x = layout_config.padding.left as f32;
                let space = dims.width - (layout_config.padding.left + layout_config.padding.right) as f32 - child_dims.width;
                match layout_config.child_alignment.x {
                    AlignmentX::Center => child_offset.x += space / 2.0,
                    AlignmentX::Right => child_offset.x += space,
                    AlignmentX::Left => {}
                }
            }

            let child_position = Vector2::new(
                current_pos.x + child_offset.x + scroll_offset.x,
                current_pos.y + child_offset.y + scroll_offset.y,
            );

            // We insert at end but process will happen in stack order
            // For correct layout order we need to push in reverse
            let insert_idx = dfs.len();
            dfs.push(DfsNode {
                element_index: ci as usize,
                position: child_position,
                next_child_offset: Vector2::new(child_padding.left as f32, child_padding.top as f32),
                visited: false,
            });

            // Update running offset
            if layout_config.layout_direction == LayoutDirection::LeftToRight {
                running_offset.x += child_dims.width + layout_config.child_gap as f32;
            } else {
                running_offset.y += child_dims.height + layout_config.child_gap as f32;
            }
        }

        // Reverse the children we just added so DFS processes them in correct order
        let children_start = dfs.len() - children.len();
        dfs[children_start..].reverse();
    }

    fn emit_text_render_commands(
        &mut self, node_idx: usize, bbox: BoundingBox,
        text_config: &TextElementConfig, z_index: i16, root: &LayoutElementTreeRoot,
    ) {
        let text_data_idx = match self.layout_elements[node_idx].text_element_data_index {
            Some(idx) => idx,
            None => return,
        };
        let pref_height = self.text_element_data[text_data_idx].preferred_dimensions.height;
        let natural_line_height = pref_height;
        let final_line_height = if text_config.line_height > 0 { text_config.line_height as f32 } else { natural_line_height };
        let line_height_offset = (final_line_height - natural_line_height) / 2.0;
        let mut y_position = line_height_offset;

        let lines_start = self.text_element_data[text_data_idx].wrapped_lines_start;
        let lines_count = self.text_element_data[text_data_idx].wrapped_lines_count;
        let base_text = self.text_element_data[text_data_idx].text.clone();
        let element_id = self.layout_elements[node_idx].id;

        for line_index in 0..lines_count {
            let wl_idx = lines_start + line_index;
            if wl_idx >= self.wrapped_text_lines.len() { break; }
            let line = self.wrapped_text_lines[wl_idx].clone();

            if line.text.is_empty() {
                y_position += final_line_height;
                continue;
            }

            let offset = match text_config.text_alignment {
                TextAlignment::Left => 0.0,
                TextAlignment::Center => (bbox.width - line.dimensions.width) / 2.0,
                TextAlignment::Right => bbox.width - line.dimensions.width,
            };

            self.add_render_command(RenderCommand {
                bounding_box: BoundingBox::new(bbox.x + offset, bbox.y + y_position, line.dimensions.width, line.dimensions.height),
                render_data: RenderData::Text(TextRenderData {
                    string_contents: line.text,
                    base_chars: base_text.clone(),
                    text_color: text_config.text_color,
                    font_id: text_config.font_id,
                    font_size: text_config.font_size,
                    letter_spacing: text_config.letter_spacing,
                    line_height: text_config.line_height,
                }),
                user_data: text_config.user_data,
                id: hash::hash_number(line_index as u32, element_id).id,
                z_index,
            });

            y_position += final_line_height;
            if !self.disable_culling && (bbox.y + y_position > self.layout_dimensions.height) {
                break;
            }
        }
    }

    fn process_dfs_upward(
        &mut self, dfs: &mut Vec<DfsNode>, node_idx: usize,
        z_index: i16, root_element_index: usize, root: &LayoutElementTreeRoot,
    ) {
        let element = &self.layout_elements[node_idx];
        let layout_config = element.layout_config;
        let element_id = element.id;
        let children_len = element.children.elements.len() as u32;

        // Scroll offset for border rendering
        let mut scroll_offset = Vector2::default();
        let mut close_clip = false;
        if let Some(clip_config) = element.element_configs.iter().find_map(|c| match c {
            ElementConfig::Clip(cc) => Some(*cc), _ => None,
        }) {
            close_clip = true;
            for sd in &self.scroll_container_datas {
                if sd.layout_element_index == node_idx as i32 {
                    scroll_offset = clip_config.child_offset;
                    if self.external_scroll_handling_enabled {
                        scroll_offset = Vector2::default();
                    }
                    break;
                }
            }
        }

        // Borders
        let has_border = element.element_configs.iter().any(|c| matches!(c, ElementConfig::Border(_)));
        if has_border {
            if let Some(item) = self.get_hash_map_item(element_id) {
                let el_bbox = item.bounding_box;
                if !self.element_is_offscreen(&el_bbox) {
                    let shared = element.element_configs.iter().find_map(|c| match c {
                        ElementConfig::Shared(s) => Some(*s), _ => None,
                    }).unwrap_or_default();

                    let border_config = element.element_configs.iter().find_map(|c| match c {
                        ElementConfig::Border(b) => Some(*b), _ => None,
                    }).unwrap();

                    self.add_render_command(RenderCommand {
                        bounding_box: el_bbox,
                        render_data: RenderData::Border(BorderRenderData {
                            color: border_config.color,
                            corner_radius: shared.corner_radius,
                            width: border_config.width,
                        }),
                        user_data: shared.user_data,
                        id: hash::hash_number(element_id, children_len).id,
                        z_index,
                    });

                    // Between-children borders
                    if border_config.width.between_children > 0 && border_config.color.a > 0.0 {
                        let half_gap = layout_config.child_gap as f32 / 2.0;
                        let children: Vec<i32> = self.layout_elements[node_idx].children.elements.clone();
                        let el_dims = self.layout_elements[node_idx].dimensions;

                        if layout_config.layout_direction == LayoutDirection::LeftToRight {
                            let mut border_x = layout_config.padding.left as f32 - half_gap;
                            for (i, &ci) in children.iter().enumerate() {
                                if i > 0 {
                                    self.add_render_command(RenderCommand {
                                        bounding_box: BoundingBox::new(
                                            el_bbox.x + border_x + scroll_offset.x,
                                            el_bbox.y + scroll_offset.y,
                                            border_config.width.between_children as f32,
                                            el_dims.height,
                                        ),
                                        render_data: RenderData::Rectangle(RectangleRenderData {
                                            background_color: border_config.color, ..Default::default()
                                        }),
                                        user_data: shared.user_data,
                                        id: hash::hash_number(element_id, children_len + 1 + i as u32).id,
                                        z_index,
                                    });
                                }
                                border_x += self.layout_elements[ci as usize].dimensions.width + layout_config.child_gap as f32;
                            }
                        } else {
                            let mut border_y = layout_config.padding.top as f32 - half_gap;
                            for (i, &ci) in children.iter().enumerate() {
                                if i > 0 {
                                    self.add_render_command(RenderCommand {
                                        bounding_box: BoundingBox::new(
                                            el_bbox.x + scroll_offset.x,
                                            el_bbox.y + border_y + scroll_offset.y,
                                            el_dims.width,
                                            border_config.width.between_children as f32,
                                        ),
                                        render_data: RenderData::Rectangle(RectangleRenderData {
                                            background_color: border_config.color, ..Default::default()
                                        }),
                                        user_data: shared.user_data,
                                        id: hash::hash_number(element_id, children_len + 1 + i as u32).id,
                                        z_index,
                                    });
                                }
                                border_y += self.layout_elements[ci as usize].dimensions.height + layout_config.child_gap as f32;
                            }
                        }
                    }
                }
            }
        }

        if close_clip {
            let root_children_len = self.layout_elements[root_element_index].children.elements.len() as u32;
            self.add_render_command(RenderCommand {
                id: hash::hash_number(element_id, root_children_len + 11).id,
                render_data: RenderData::None,
                ..Default::default()
            });
        }

        dfs.pop();
    }
}

struct DfsNode {
    element_index: usize,
    position: Vector2,
    next_child_offset: Vector2,
    visited: bool,
}
