use crate::arena::*;
use crate::context::{Context, ErrorType};
use crate::elements::*;
use crate::hash;
use crate::types::*;

impl Context {
    pub(crate) fn measure_text_cached(
        &mut self,
        text: &str,
        config: &TextElementConfig,
    ) -> MeasureTextCacheItem {
        let measure_fn = match self.measure_text_function {
            Some(f) => f,
            None => {
                if !self.boolean_warnings.text_measurement_function_not_set {
                    self.boolean_warnings.text_measurement_function_not_set = true;
                    self.report_error(
                        ErrorType::TextMeasurementFunctionNotProvided,
                        "Clay's internal MeasureText function is null. You may have forgotten to call set_measure_text_function().",
                    );
                }
                return MeasureTextCacheItem::default();
            }
        };

        let id = hash::hash_string_contents_with_config(
            text,
            false,
            config.font_id,
            config.font_size,
            config.letter_spacing,
        );

        let hash_map_cap = (self.max_measure_text_cache_word_count / 32).max(1) as usize;
        let hash_bucket = (id as usize) % hash_map_cap;

        // Search existing cache
        let mut element_index_previous: i32 = 0;
        let mut element_index = if hash_bucket < self.measure_text_hash_map.len() {
            self.measure_text_hash_map[hash_bucket]
        } else {
            0
        };

        while element_index != 0 {
            let idx = element_index as usize;
            if idx >= self.measure_text_hash_map_internal.len() {
                break;
            }
            let hash_entry = &self.measure_text_hash_map_internal[idx];
            if hash_entry.id == id {
                self.measure_text_hash_map_internal[idx].generation = self.generation;
                return self.measure_text_hash_map_internal[idx].clone();
            }
            // Evict stale entries
            if self.generation - hash_entry.generation > 2 {
                let mut next_word_index = hash_entry.measured_words_start_index;
                while next_word_index != -1 {
                    if let Some(word) = self.measured_words.get(next_word_index as usize) {
                        let next = word.next;
                        self.measured_words_free_list.push(next_word_index);
                        next_word_index = next;
                    } else {
                        break;
                    }
                }
                let next_index = hash_entry.next_index;
                self.measure_text_hash_map_internal[idx] = MeasureTextCacheItem {
                    measured_words_start_index: -1,
                    ..Default::default()
                };
                self.measure_text_hash_map_internal_free_list.push(element_index);
                if element_index_previous == 0 {
                    if hash_bucket < self.measure_text_hash_map.len() {
                        self.measure_text_hash_map[hash_bucket] = next_index;
                    }
                } else {
                    self.measure_text_hash_map_internal[element_index_previous as usize]
                        .next_index = next_index;
                }
                element_index = next_index;
            } else {
                element_index_previous = element_index;
                element_index = hash_entry.next_index;
            }
        }

        // Create new cache item
        let new_cache_item = MeasureTextCacheItem {
            measured_words_start_index: -1,
            id,
            generation: self.generation,
            ..Default::default()
        };

        let new_item_index: i32;
        if let Some(&free_idx) = self.measure_text_hash_map_internal_free_list.last() {
            self.measure_text_hash_map_internal_free_list.pop();
            new_item_index = free_idx;
            if (new_item_index as usize) < self.measure_text_hash_map_internal.len() {
                self.measure_text_hash_map_internal[new_item_index as usize] =
                    new_cache_item;
            }
        } else {
            if self.measure_text_hash_map_internal.len()
                >= self.measure_text_hash_map_internal.capacity() - 1
            {
                if !self.boolean_warnings.max_text_measure_cache_exceeded {
                    self.report_error(
                        ErrorType::ElementsCapacityExceeded,
                        "Clay ran out of capacity while attempting to measure text elements.",
                    );
                    self.boolean_warnings.max_text_measure_cache_exceeded = true;
                }
                return MeasureTextCacheItem::default();
            }
            self.measure_text_hash_map_internal.push(new_cache_item);
            new_item_index = (self.measure_text_hash_map_internal.len() - 1) as i32;
        }

        // Measure words
        let user_data = self.measure_text_user_data;
        let space_width = measure_fn(" ", config, user_data).width;

        let mut start: usize = 0;
        let mut end: usize = 0;
        let mut line_width: f32 = 0.0;
        let mut measured_width: f32 = 0.0;
        let mut measured_height: f32 = 0.0;
        let mut temp_word = MeasuredWord {
            next: -1,
            ..Default::default()
        };
        let mut previous_word_index: i32 = -1;
        let bytes = text.as_bytes();

        while end < bytes.len() {
            if self.measured_words.len() >= self.measured_words.capacity() - 1 {
                if !self.boolean_warnings.max_text_measure_cache_exceeded {
                    self.report_error(
                        ErrorType::TextMeasurementCapacityExceeded,
                        "Clay has run out of space in its internal text measurement cache.",
                    );
                    self.boolean_warnings.max_text_measure_cache_exceeded = true;
                }
                return MeasureTextCacheItem::default();
            }

            let current = bytes[end];
            if current == b' ' || current == b'\n' {
                let length = end - start;
                let mut dimensions = Dimensions::default();
                if length > 0 {
                    let slice = &text[start..end];
                    dimensions = measure_fn(slice, config, user_data);
                }

                let min_w =
                    &mut self.measure_text_hash_map_internal[new_item_index as usize].min_width;
                *min_w = min_w.max(dimensions.width);
                measured_height = measured_height.max(dimensions.height);

                if current == b' ' {
                    dimensions.width += space_width;
                    let word = MeasuredWord {
                        start_offset: start as i32,
                        length: (length + 1) as i32,
                        width: dimensions.width,
                        next: -1,
                    };
                    previous_word_index =
                        self.add_measured_word(word, previous_word_index, &mut temp_word);
                    line_width += dimensions.width;
                }
                if current == b'\n' {
                    if length > 0 {
                        let word = MeasuredWord {
                            start_offset: start as i32,
                            length: length as i32,
                            width: dimensions.width,
                            next: -1,
                        };
                        previous_word_index =
                            self.add_measured_word(word, previous_word_index, &mut temp_word);
                    }
                    let newline_word = MeasuredWord {
                        start_offset: (end + 1) as i32,
                        length: 0,
                        width: 0.0,
                        next: -1,
                    };
                    previous_word_index =
                        self.add_measured_word(newline_word, previous_word_index, &mut temp_word);
                    line_width += dimensions.width;
                    measured_width = measured_width.max(line_width);
                    self.measure_text_hash_map_internal[new_item_index as usize]
                        .contains_newlines = true;
                    line_width = 0.0;
                }
                start = end + 1;
            }
            end += 1;
        }

        if end > start {
            let slice = &text[start..end];
            let dimensions = measure_fn(slice, config, user_data);
            let word = MeasuredWord {
                start_offset: start as i32,
                length: (end - start) as i32,
                width: dimensions.width,
                next: -1,
            };
            self.add_measured_word(word, previous_word_index, &mut temp_word);
            line_width += dimensions.width;
            measured_height = measured_height.max(dimensions.height);
            let min_w =
                &mut self.measure_text_hash_map_internal[new_item_index as usize].min_width;
            *min_w = min_w.max(dimensions.width);
        }

        measured_width = measured_width.max(line_width) - config.letter_spacing as f32;

        self.measure_text_hash_map_internal[new_item_index as usize]
            .measured_words_start_index = temp_word.next;
        self.measure_text_hash_map_internal[new_item_index as usize]
            .unwrapped_dimensions
            .width = measured_width;
        self.measure_text_hash_map_internal[new_item_index as usize]
            .unwrapped_dimensions
            .height = measured_height;

        if element_index_previous != 0 {
            self.measure_text_hash_map_internal[element_index_previous as usize].next_index =
                new_item_index;
        } else if hash_bucket < self.measure_text_hash_map.len() {
            self.measure_text_hash_map[hash_bucket] = new_item_index;
        }

        self.measure_text_hash_map_internal[new_item_index as usize].clone()
    }

    fn add_measured_word(
        &mut self,
        word: MeasuredWord,
        previous_word_index: i32,
        temp_word: &mut MeasuredWord,
    ) -> i32 {
        let new_index: i32;
        if let Some(&free_idx) = self.measured_words_free_list.last() {
            self.measured_words_free_list.pop();
            new_index = free_idx;
            if (new_index as usize) < self.measured_words.len() {
                self.measured_words[new_index as usize] = word;
            }
        } else {
            self.measured_words.push(word);
            new_index = (self.measured_words.len() - 1) as i32;
        }

        if previous_word_index == -1 {
            temp_word.next = new_index;
        } else if (previous_word_index as usize) < self.measured_words.len() {
            self.measured_words[previous_word_index as usize].next = new_index;
        }
        new_index
    }

    pub(crate) fn int_to_string(&mut self, integer: i32) -> String {
        integer.to_string()
    }
}
