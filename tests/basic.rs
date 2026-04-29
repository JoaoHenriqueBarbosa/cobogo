use cobogo::context::Context;
use cobogo::elements::*;
use cobogo::layout::*;
use cobogo::types::*;

fn dummy_measure_text(text: &str, config: &TextElementConfig, _user_data: usize) -> Dimensions {
    Dimensions::new(text.len() as f32 * config.font_size as f32 * 0.6, config.font_size as f32 * 1.2)
}

#[test]
fn test_context_creation() {
    let ctx = Context::new(Dimensions::new(800.0, 600.0));
    assert_eq!(ctx.get_layout_dimensions(), Dimensions::new(800.0, 600.0));
    assert_eq!(ctx.get_max_element_count(), 8192);
}

#[test]
fn test_element_id_hashing() {
    let ctx = Context::new(Dimensions::new(800.0, 600.0));
    let id1 = ctx.id("test");
    let id2 = ctx.id("test");
    assert_eq!(id1.id, id2.id);
    assert_ne!(id1.id, 0);

    let id3 = ctx.id("other");
    assert_ne!(id1.id, id3.id);
}

#[test]
fn test_basic_layout_flow() {
    let mut ctx = Context::new(Dimensions::new(800.0, 600.0));
    ctx.set_measure_text_function(dummy_measure_text, 0);

    ctx.begin_layout();

    let container_id = ctx.id("Container");
    ctx.with_element(
        container_id,
        ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::grow(0.0, f32::MAX),
                    height: SizingAxis::grow(0.0, f32::MAX),
                },
                layout_direction: LayoutDirection::TopToBottom,
                padding: Padding::all(16),
                child_gap: 8,
                ..Default::default()
            },
            background_color: Color::new(255.0, 255.0, 255.0, 255.0),
            ..Default::default()
        },
        |ctx| {
            ctx.text(
                "Hello, Clay!",
                &TextElementConfig {
                    font_size: 24,
                    text_color: Color::new(0.0, 0.0, 0.0, 255.0),
                    ..Default::default()
                },
            );

            let child_id = ctx.id("Child");
            ctx.with_element(
                child_id,
                ElementDeclaration {
                    layout: LayoutConfig {
                        sizing: Sizing {
                            width: SizingAxis::fixed(200.0),
                            height: SizingAxis::fixed(100.0),
                        },
                        ..Default::default()
                    },
                    background_color: Color::new(200.0, 100.0, 50.0, 255.0),
                    corner_radius: CornerRadius::all(8.0),
                    ..Default::default()
                },
                |_ctx| {},
            );
        },
    );

    let render_commands = ctx.end_layout();
    assert!(!render_commands.is_empty(), "Should produce render commands");
}

#[test]
fn test_nested_elements() {
    let mut ctx = Context::new(Dimensions::new(1024.0, 768.0));
    ctx.set_measure_text_function(dummy_measure_text, 0);

    ctx.begin_layout();

    let root_id = ctx.id("Root");
    ctx.with_element(
        root_id,
        ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::grow(0.0, f32::MAX),
                    height: SizingAxis::grow(0.0, f32::MAX),
                },
                layout_direction: LayoutDirection::LeftToRight,
                child_gap: 10,
                ..Default::default()
            },
            ..Default::default()
        },
        |ctx| {
            for i in 0..3 {
                let child_id = ctx.idi("Child", i);
                ctx.with_element(
                    child_id,
                    ElementDeclaration {
                        layout: LayoutConfig {
                            sizing: Sizing {
                                width: SizingAxis::grow(0.0, f32::MAX),
                                height: SizingAxis::fixed(50.0),
                            },
                            ..Default::default()
                        },
                        background_color: Color::new(100.0, 150.0, 200.0, 255.0),
                        ..Default::default()
                    },
                    |_ctx| {},
                );
            }
        },
    );

    let render_commands = ctx.end_layout();
    assert!(render_commands.len() >= 3, "Should have at least 3 render commands for 3 children");
}

#[test]
fn test_text_elements() {
    let mut ctx = Context::new(Dimensions::new(800.0, 600.0));
    ctx.set_measure_text_function(dummy_measure_text, 0);

    ctx.begin_layout();

    let root_id = ctx.id("Root");
    ctx.with_element(
        root_id,
        ElementDeclaration {
            layout: LayoutConfig {
                sizing: Sizing {
                    width: SizingAxis::fixed(800.0),
                    height: SizingAxis::fixed(600.0),
                },
                layout_direction: LayoutDirection::TopToBottom,
                ..Default::default()
            },
            ..Default::default()
        },
        |ctx| {
            ctx.text(
                "First line",
                &TextElementConfig {
                    font_size: 16,
                    text_color: Color::new(0.0, 0.0, 0.0, 255.0),
                    ..Default::default()
                },
            );
            ctx.text(
                "Second line",
                &TextElementConfig {
                    font_size: 16,
                    text_color: Color::new(0.0, 0.0, 0.0, 255.0),
                    ..Default::default()
                },
            );
        },
    );

    let render_commands = ctx.end_layout();
    assert!(!render_commands.is_empty());
}
