# Cobogo

**A renderer-agnostic UI layout engine for Rust — an idiomatic port of [Clay](https://github.com/nicbarker/clay).**

[![Crates.io](https://img.shields.io/crates/v/cobogo.svg?logo=rust)](https://crates.io/crates/cobogo)
[![Documentation](https://docs.rs/cobogo/badge.svg)](https://docs.rs/cobogo)
[![License: Zlib](https://img.shields.io/badge/license-Zlib-blue.svg)](LICENSE)
[![Rust 2024](https://img.shields.io/badge/rust-2024%20(1.85%2B)-orange.svg)](https://www.rust-lang.org)
[![Tests](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/JoaoHenriqueBarbosa/cobogo/main/.github/badges/tests.json)](https://github.com/JoaoHenriqueBarbosa/cobogo/actions)
[![Lines of code](https://img.shields.io/endpoint?url=https://raw.githubusercontent.com/JoaoHenriqueBarbosa/cobogo/main/.github/badges/loc.json)](https://github.com/JoaoHenriqueBarbosa/cobogo)

Cobogo calculates the position and size of every element in a UI from a declarative, immediate-mode description, then emits a **flat list of `RenderCommand`s** that any backend can draw. It does not render anything itself: the core crate has **zero dependencies** and contains no drawing code at all. That separation is the whole point — you describe your layout once and pick (or write) a renderer separately.

> Named after the [cobogó](https://en.wikipedia.org/wiki/Cobogó) — the perforated modernist bricks that structure light and space in Brazilian architecture. Fitting for a library whose only job is to structure space.

## Highlights

- **Zero-dependency core** — the entire layout algorithm is self-contained; `[dependencies]` in the core crate is empty.
- **Renderer-agnostic** — the core only produces a `RenderData` enum (`Rectangle`, `Text`, `Border`, `Image`, `Clip`, `Custom`). Bring your own backend, or use the included terminal renderer.
- **Faithful port of Clay's multi-pass layout algorithm** — sizing along X, text wrapping, aspect-ratio fitting, height propagation, sizing along Y, z-index sorting, and command generation run as distinct passes over the element tree.
- **Data-oriented internals** — elements live in parallel `Vec`s (structure-of-arrays) rather than a tree of pointers, and command generation walks the tree with an explicit stack instead of recursion.
- **Idiomatic closure-based API** — `with_element(id, decl, |ctx| { … })` opens and automatically closes an element at the end of the closure, replacing Clay's manual begin/end bookkeeping. A lower-level imperative API is also available.
- **Text measurement cache** — measured strings are cached with a free-list and generation-based eviction, and caches survive across frames (only ephemeral memory is cleared between layouts).
- **Interaction support** — pointer/hover queries, hover callbacks, and scroll containers with drag momentum.
- **Included terminal renderer** — [`cobogo-renderer-ratatui`](renderers/ratatui) shows the full renderer pattern, including a clip stack with rectangle intersection.

## Requirements

- **Rust 2024 edition**, toolchain **1.85 or newer** (this is the crate's declared `rust-version`).
- No system libraries, no build scripts, no code generation. Adding the crate is enough.

## Installation

```toml
[dependencies]
cobogo = "0.1"
```

## Usage

Create a context, describe your layout between `begin_layout` and `end_layout`, and consume the render commands:

```rust
use cobogo::context::Context;
use cobogo::elements::*;
use cobogo::layout::*;
use cobogo::types::*;

// Create a layout context with viewport dimensions.
let mut ctx = Context::new(Dimensions::new(800.0, 600.0));
ctx.set_measure_text_function(my_measure_text_fn, 0);

ctx.begin_layout();

// Build a vertical container with two children.
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
    // Header: full width, fixed height.
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

    // Content area that grows to fill the remaining space.
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

// Finalize layout and get the render commands.
let render_commands = ctx.end_layout();

for cmd in &render_commands {
    // Each command has a bounding_box, render_data, id, and z_index.
}
```

> **Text requires a measure function.** Because the core has no font engine, you must call `set_measure_text_function` before laying out any text. The function receives a string and its config and returns `Dimensions`. Renderers typically supply one that matches their backend (e.g. `text.len()` columns for a terminal).

### Element IDs and queries

Cobogo uses hash-based IDs, so you can look up an element's computed position after layout:

```rust
let button_id = ctx.id("SubmitButton");   // global ID
let item_id   = ctx.idi("ListItem", index); // indexed ID (for lists)
let local_id  = ctx.id_local("Icon");       // scoped to the current parent

let data = ctx.get_element_data(&button_id);
if data.found {
    println!("Button at ({}, {})", data.bounding_box.x, data.bounding_box.y);
}
```

### Sizing modes

```rust
SizingAxis::fit(min, max)       // shrink to content, clamped to [min, max]
SizingAxis::grow(min, max)      // expand to fill parent, clamped to [min, max]
SizingAxis::fixed(value)        // exact size
SizingAxis::percent(fraction)   // percentage of parent (0.0..=1.0)
Sizing::grow()                  // shorthand: grow on both axes
Sizing::fit()                   // shorthand: fit on both axes
```

### What the layout engine supports

Each of these is backed by real code in the core (`layout_calc.rs`, `input_handling.rs`):

- **Flexible sizing** — `Fit`, `Grow`, `Percent`, and `Fixed`, each with min/max clamps.
- **Layout directions** — left-to-right and top-to-bottom child arrangement.
- **Child alignment** — on both axes (left/center/right, top/center/bottom).
- **Padding and gaps** — per-side padding and uniform child-gap spacing.
- **Text layout** — word wrapping, newline handling, and per-config text alignment.
- **Borders** — per-side widths and color, including borders drawn *between* children.
- **Corner radius** — per-corner radii (honored by renderers that can draw them).
- **Floating elements** — positioned relative to parent, root, or any named element, with 9 attach points.
- **Clipping and scrolling** — clip children and offset them via scroll containers.
- **Aspect ratio** — constrain an element to a fixed width/height ratio.
- **Culling** — offscreen elements are skipped during command generation (can be disabled).

## Renderers

The core produces `RenderCommand` values that describe *what* to draw and *where*, independent of any graphics library.

### cobogo-renderer-ratatui

A terminal renderer built on [ratatui](https://github.com/ratatui/ratatui):

```toml
[dependencies]
cobogo = "0.1"
cobogo-renderer-ratatui = "0.1"
ratatui = "0.29"
```

```rust
use cobogo_renderer_ratatui::CobogoRatatuiRenderer;

let mut renderer = CobogoRatatuiRenderer::new();
renderer.render(&render_commands, frame.buffer_mut());
```

The terminal is a constrained target: **corner radius has no effect** (cells can't be rounded) and **images render as a gray `[img]` placeholder** rather than real pixels. Rectangles, text, borders, and clipping all work.

### Writing a custom renderer

Iterate the commands and handle each `RenderData` variant — that is the entire contract:

```rust
for cmd in &render_commands {
    let bbox = &cmd.bounding_box; // position and size
    match &cmd.render_data {
        RenderData::Rectangle(rect)  => { /* fill with rect.background_color */ }
        RenderData::Text(text)       => { /* draw text.text at bbox */ }
        RenderData::Border(border)   => { /* draw border lines */ }
        RenderData::Image(image)     => { /* draw image */ }
        RenderData::Clip(clip)       => { /* push clip region */ }
        RenderData::Custom(custom)   => { /* your call */ }
        RenderData::None             => {}
    }
}
```

## Example

The [`examples/tui-app`](examples/tui-app) directory contains an interactive terminal app that exercises the library end to end: a header with tab navigation, a collapsible sidebar of selectable items, a dashboard of stat cards, dark/light theme switching, mouse hover/click, and keyboard navigation.

```sh
cargo run -p tui-app
```

It needs an interactive terminal (it enables raw mode, the alternate screen, and mouse capture). Controls: `q` quit · `s` toggle sidebar · `Tab` switch tabs · `↑`/`↓` navigate · `Enter` activate · `t` toggle theme · mouse hover/click.

## Development

Everything runs from the workspace root with the standard Cargo tooling:

```sh
cargo build --workspace     # build all three crates
cargo test  --workspace     # run the test suite (unit + integration + doctests)
cargo run   -p tui-app      # run the interactive example (interactive terminal required)
cargo doc   --open          # build and open the API docs
```

### Testing scope — honest note

The test suite is currently a **smoke-level** safety net, not a proof of layout correctness. The tests confirm that a context can be constructed, that ID hashing is deterministic, and that a non-trivial tree produces a non-empty command list. **They do not assert the actual computed coordinates or sizes.** Treat green tests as "it runs and hasn't obviously regressed," not as "the geometry is verified." Contributions that pin exact bounding boxes are especially welcome (see [CONTRIBUTING.md](CONTRIBUTING.md)).

## Architecture

A single Cargo workspace with three crates:

```
cobogo/
├── Cargo.toml                       # workspace + core crate `cobogo` (zero deps)
├── LICENSE                          # zlib/libpng
├── src/                             # core layout engine (14 modules)
│   ├── lib.rs                       # re-exports, crate docs, doctest
│   ├── context.rs                   # Context: begin/end_layout, with_element, queries
│   ├── layout_calc.rs               # the multi-pass layout algorithm + command generation
│   ├── input_handling.rs            # pointer, hover, scroll containers, drag momentum
│   ├── text.rs                      # text-measurement cache + word splitting
│   ├── elements.rs                  # ElementDeclaration, configs, ElementId
│   ├── layout.rs                    # LayoutConfig, Sizing, Padding, alignment
│   ├── debug.rs                     # (simplified) debug overlay
│   ├── types.rs                     # Dimensions, Vector2, Color, BoundingBox, …
│   ├── hash.rs                      # Clay-style ID hashing + unit tests
│   ├── arena.rs                     # internal structure-of-arrays storage
│   ├── render.rs                    # RenderCommand, RenderData
│   └── input.rs                     # PointerData, ScrollContainerData
├── renderers/
│   └── ratatui/                     # crate `cobogo-renderer-ratatui`
│       └── src/lib.rs               # CobogoRatatuiRenderer, clip stack, borders
├── examples/
│   └── tui-app/                     # crate `tui-app` (not published)
│       └── src/                     # event loop + declarative UI with themes
└── tests/
    └── basic.rs                     # integration tests (smoke-level)
```

**A note on names.** Cobogo is a port, and some of Clay's internal string constants (hash seeds, error messages, debug-panel labels) still literally read `"Clay"`. These are internal identifiers and messages — they don't affect behavior — but if you see `"Clay ran out of capacity"` in an error, that's why.

## Project status

Cobogo is an **early-stage (0.1.x) but functional** port. The layout pipeline, the closure API, the text cache, input handling, and the terminal renderer all work and are published to crates.io. What it is *not* yet: geometry-verified (see the testing note above), warning-free, or feature-frozen. The debug overlay in particular is a **simplified** version of Clay's inspector — it reports the element count and offers a close button, not the full per-element inspection panel. If you need a mature layout engine today, use the original [Clay](https://github.com/nicbarker/clay); if you want a Rust-native one and are comfortable on the edge, Cobogo is usable and contributions are welcome.

## Credits

Cobogo is an idiomatic Rust port of [Clay](https://github.com/nicbarker/clay) by **Nic Barker**. The layout algorithm, the immediate-mode design, and the text-cache strategy are all his; this project translates them to Rust.

## License

Licensed under the [zlib/libpng license](LICENSE) — the same license as the original Clay. As a port, it carries a dual copyright (Nic Barker for the original C library, João Henrique Barbosa for the Rust port) and is plainly marked as an altered source version, per the license's requirements.
