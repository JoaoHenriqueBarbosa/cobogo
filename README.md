# Cobogo

A UI layout library for Rust, ported from [Clay](https://github.com/nicbarker/clay).

Cobogo is a lightweight, renderer-agnostic layout engine that calculates element positions and sizes using a declarative, immediate-mode API. It produces render commands that can be consumed by any rendering backend.

## Features

- **Flexible sizing** &mdash; `Fit`, `Grow`, `Percent`, and `Fixed` sizing modes with min/max constraints
- **Layout directions** &mdash; Horizontal (left-to-right) and vertical (top-to-bottom) child arrangement
- **Child alignment** &mdash; Align children on both axes (left/center/right, top/center/bottom)
- **Padding and gaps** &mdash; Per-side padding and uniform child gap spacing
- **Text layout** &mdash; Word wrapping, newline wrapping, text alignment (left/center/right), and pluggable text measurement
- **Borders** &mdash; Per-side border widths with color, including borders between children
- **Corner radius** &mdash; Per-corner border radius
- **Floating elements** &mdash; Position elements relative to parent, root, or any named element with 9 anchor points
- **Clipping and scrolling** &mdash; Clip children with scroll offset support
- **Aspect ratio** &mdash; Constrain elements to a fixed aspect ratio
- **Element queries** &mdash; Look up bounding boxes by element ID after layout
- **Culling** &mdash; Automatic offscreen element culling (can be disabled)
- **Debug mode** &mdash; Visual debug overlay for layout inspection
- **Zero dependencies** &mdash; The core library has no external dependencies

## Getting Started

Add cobogo to your `Cargo.toml`:

```toml
[dependencies]
cobogo = "0.1"
```

### Basic Usage

```rust
use cobogo::context::Context;
use cobogo::elements::*;
use cobogo::layout::*;
use cobogo::types::*;

// Create a layout context with viewport dimensions
let mut ctx = Context::new(Dimensions::new(800.0, 600.0));
ctx.set_measure_text_function(my_measure_text_fn, 0);

ctx.begin_layout();

// Build a vertical container with two children
let root = ctx.id("Root");
ctx.with_element(root, ElementDeclaration {
    layout: LayoutConfig {
        sizing: Sizing::grow(),
        layout_direction: LayoutDirection::TopToBottom,
        padding: Padding::all(16),
        child_gap: 8,
        ..Default::default()
    },
    ..Default::default()
}, |ctx| {
    // Header
    let header = ctx.id("Header");
    ctx.with_element(header, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing {
                width: SizingAxis::grow(0.0, f32::MAX),
                height: SizingAxis::fixed(48.0),
            },
            ..Default::default()
        },
        background_color: Color::new(40.0, 42.0, 54.0, 255.0),
        ..Default::default()
    }, |ctx| {
        ctx.text("Hello, Cobogo!", &TextElementConfig {
            text_color: Color::new(255.0, 255.0, 255.0, 255.0),
            font_size: 16,
            ..Default::default()
        });
    });

    // Content area that grows to fill remaining space
    let content = ctx.id("Content");
    ctx.with_element(content, ElementDeclaration {
        layout: LayoutConfig {
            sizing: Sizing::grow(),
            ..Default::default()
        },
        background_color: Color::new(30.0, 30.0, 40.0, 255.0),
        ..Default::default()
    }, |_ctx| {});
});

// Finalize layout and get render commands
let render_commands = ctx.end_layout();

// Pass render_commands to your renderer
for cmd in &render_commands {
    // Each command has a bounding_box, render_data, and z_index
}
```

### Element IDs

Cobogo uses hash-based IDs to identify elements, which enables querying element positions after layout:

```rust
let button_id = ctx.id("SubmitButton");     // global ID
let item_id = ctx.idi("ListItem", index);    // indexed ID (for lists)
let local_id = ctx.id_local("Icon");         // scoped to parent element

// After layout, query element position
let data = ctx.get_element_data(&button_id);
if data.found {
    println!("Button at ({}, {})", data.bounding_box.x, data.bounding_box.y);
}
```

### Sizing Modes

```rust
SizingAxis::fit(min, max)       // Shrink to content, clamped to [min, max]
SizingAxis::grow(min, max)      // Expand to fill parent, clamped to [min, max]
SizingAxis::fixed(value)        // Exact size
SizingAxis::percent(fraction)   // Percentage of parent (0.0 to 1.0)
Sizing::grow()                  // Shorthand: grow on both axes
Sizing::fit()                   // Shorthand: fit on both axes
```

## Renderers

Cobogo is renderer-agnostic. The core library produces `RenderCommand` values that describe what to draw and where, without depending on any specific graphics library.

### cobogo-renderer-ratatui

A terminal renderer using [ratatui](https://github.com/ratatui/ratatui):

```toml
[dependencies]
cobogo = "0.1"
cobogo-renderer-ratatui = "0.1"
ratatui = "0.29"
```

```rust
use cobogo_renderer_ratatui::CobogoRatatuiRenderer;

let mut renderer = CobogoRatatuiRenderer::new();
renderer.render(&render_commands, terminal_frame.buffer_mut());
```

### Writing a Custom Renderer

Iterate over the render commands and handle each `RenderData` variant:

```rust
for cmd in &render_commands {
    let bbox = &cmd.bounding_box; // position and size
    match &cmd.render_data {
        RenderData::Rectangle(rect) => { /* fill with rect.background_color */ }
        RenderData::Text(text) => { /* draw text.text at bbox position */ }
        RenderData::Border(border) => { /* draw border lines */ }
        RenderData::Image(image) => { /* draw image */ }
        RenderData::Clip(clip) => { /* push clip region */ }
        RenderData::Custom(custom) => { /* handle custom rendering */ }
        RenderData::None => {}
    }
}
```

## Examples

The [`examples/tui-app`](examples/tui-app) directory contains an interactive terminal application demonstrating:

- Header with tab navigation
- Collapsible sidebar with selectable items
- Dashboard with stat cards
- Dark/light theme switching
- Mouse hover detection and click handling
- Keyboard navigation

Run it with:

```sh
cargo run -p tui-app
```

## Credits

Cobogo is an idiomatic Rust port of [Clay](https://github.com/nicbarker/clay) by Nic Barker.

## License

[zlib/libpng](LICENSE)
