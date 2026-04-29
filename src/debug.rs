use crate::context::Context;
use crate::elements::*;
use crate::layout::*;
use crate::types::*;

#[derive(Debug, Clone)]
pub struct DebugElementConfigTypeLabelConfig {
    pub label: &'static str,
    pub color: Color,
}

pub fn get_element_config_type_label(config: &ElementConfig) -> DebugElementConfigTypeLabelConfig {
    match config {
        ElementConfig::Shared(_) => DebugElementConfigTypeLabelConfig {
            label: "Shared",
            color: Color::new(243.0, 134.0, 48.0, 255.0),
        },
        ElementConfig::Text(_) => DebugElementConfigTypeLabelConfig {
            label: "Text",
            color: Color::new(105.0, 210.0, 231.0, 255.0),
        },
        ElementConfig::Aspect(_) => DebugElementConfigTypeLabelConfig {
            label: "Aspect",
            color: Color::new(101.0, 149.0, 194.0, 255.0),
        },
        ElementConfig::Image(_) => DebugElementConfigTypeLabelConfig {
            label: "Image",
            color: Color::new(121.0, 189.0, 154.0, 255.0),
        },
        ElementConfig::Floating(_) => DebugElementConfigTypeLabelConfig {
            label: "Floating",
            color: Color::new(250.0, 105.0, 0.0, 255.0),
        },
        ElementConfig::Clip(_) => DebugElementConfigTypeLabelConfig {
            label: "Scroll",
            color: Color::new(242.0, 196.0, 90.0, 255.0),
        },
        ElementConfig::Border(_) => DebugElementConfigTypeLabelConfig {
            label: "Border",
            color: Color::new(108.0, 91.0, 123.0, 255.0),
        },
        ElementConfig::Custom(_) => DebugElementConfigTypeLabelConfig {
            label: "Custom",
            color: Color::new(11.0, 72.0, 107.0, 255.0),
        },
        ElementConfig::None => DebugElementConfigTypeLabelConfig {
            label: "Error",
            color: Color::new(0.0, 0.0, 0.0, 255.0),
        },
    }
}

impl Context {
    pub fn render_debug_view(&mut self) {
        // The debug view in C uses the CLAY macro system to build a complex UI.
        // In Rust, we implement this using the closure-based API.
        // This is a simplified version — the full debug view would use the same
        // with_element/text API to build the inspector panel.

        let close_button_id = crate::hash::hash_string("Clay__DebugViewTopHeaderCloseButtonOuter", 0);

        // Check if close button was clicked
        if self.pointer_info.state == crate::input::PointerInteractionState::PressedThisFrame {
            for poi in &self.pointer_over_ids {
                if poi.id == close_button_id.id {
                    self.debug_mode_enabled = false;
                    return;
                }
            }
        }

        let initial_roots_length = self.layout_element_tree_roots.len();
        let debug_view_width = self.debug_view_width as f32;
        let layout_height = self.layout_dimensions.height;
        let _highlight_color = self.debug_view_highlight_color;

        let debug_bg_1 = Color::new(58.0, 56.0, 52.0, 255.0);
        let debug_bg_2 = Color::new(62.0, 60.0, 58.0, 255.0);
        let debug_color_3 = Color::new(141.0, 133.0, 135.0, 255.0);
        let debug_color_4 = Color::new(238.0, 226.0, 231.0, 255.0);

        // Build debug view using the element API
        let debug_view_id = crate::hash::hash_string("Clay__DebugView", 0);
        self.open_element_with_id(debug_view_id);
        self.configure_open_element(&ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::fixed(debug_view_width),
                    height: SizingAxis::fixed(layout_height),
                },
                layout_direction: LayoutDirection::TopToBottom,
                ..Default::default()
            },
            floating: FloatingConfig {
                z_index: 32765,
                attach_points: FloatingAttachPoints {
                    element: FloatingAttachPointType::LeftCenter,
                    parent: FloatingAttachPointType::RightCenter,
                },
                attach_to: FloatingAttachTo::Root,
                clip_to: FloatingClipTo::AttachedParent,
                ..Default::default()
            },
            border: BorderConfig {
                color: debug_color_3,
                width: crate::types::BorderWidth {
                    bottom: 1,
                    ..Default::default()
                },
            },
            ..Default::default()
        });

        // Header
        let header_text_config = TextElementConfig {
            text_color: debug_color_4,
            font_size: 16,
            wrap_mode: TextWrapMode::None,
            ..Default::default()
        };

        self.open_element();
        self.configure_open_element(&ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::grow(0.0, f32::MAX),
                    height: SizingAxis::fixed(30.0),
                },
                padding: Padding {
                    left: 10,
                    right: 10,
                    ..Default::default()
                },
                child_alignment: crate::layout::ChildAlignment {
                    y: AlignmentY::Center,
                    ..Default::default()
                },
                ..Default::default()
            },
            background_color: debug_bg_2,
            ..Default::default()
        });
        self.open_text_element("Clay Debug Tools", &header_text_config);
        self.close_element();

        // Simplified: just show element count info
        self.open_element();
        self.configure_open_element(&ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::grow(0.0, f32::MAX),
                    height: SizingAxis::grow(0.0, f32::MAX),
                },
                layout_direction: LayoutDirection::TopToBottom,
                ..Default::default()
            },
            background_color: debug_bg_1,
            ..Default::default()
        });

        let info = format!("Elements: {}", initial_roots_length);
        let info_config = TextElementConfig {
            text_color: debug_color_4,
            font_size: 16,
            ..Default::default()
        };
        self.open_text_element(&info, &info_config);
        self.close_element();

        self.close_element(); // debug view root
    }
}
