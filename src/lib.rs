//! # Cobogo
//!
//! A renderer-agnostic UI layout library for Rust, ported from
//! [Clay](https://github.com/nicbarker/clay).
//!
//! Cobogo calculates element positions and sizes from a declarative,
//! immediate-mode description of your UI and produces a flat list of
//! [`RenderCommand`]s that any rendering backend can consume.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use cobogo::context::Context;
//! use cobogo::elements::*;
//! use cobogo::layout::*;
//! use cobogo::types::*;
//!
//! fn measure(text: &str, _cfg: &TextElementConfig, _ud: usize) -> Dimensions {
//!     Dimensions::new(text.len() as f32, 1.0)
//! }
//!
//! let mut ctx = Context::new(Dimensions::new(80.0, 24.0));
//! ctx.set_measure_text_function(measure, 0);
//!
//! ctx.begin_layout();
//!
//! let root = ctx.id("Root");
//! ctx.with_element(root, ElementDeclaration {
//!     layout: LayoutConfig {
//!         sizing: Sizing::grow(),
//!         layout_direction: LayoutDirection::TopToBottom,
//!         ..Default::default()
//!     },
//!     ..Default::default()
//! }, |ctx| {
//!     ctx.text("Hello, Cobogo!", &TextElementConfig::default());
//! });
//!
//! let commands = ctx.end_layout();
//! // Pass `commands` to your renderer.
//! ```
//!
//! # Module overview
//!
//! | Module | Contents |
//! |--------|----------|
//! | [`types`] | Primitive types: [`Dimensions`], [`Vector2`], [`Color`], [`BoundingBox`], [`CornerRadius`], [`BorderWidth`] |
//! | [`layout`] | Layout configuration: [`LayoutConfig`], [`Sizing`], [`SizingAxis`], [`Padding`], [`LayoutDirection`] |
//! | [`elements`] | Element configuration: [`ElementDeclaration`], [`ElementId`], [`TextElementConfig`], [`FloatingConfig`], [`BorderConfig`], [`ClipConfig`] |
//! | [`render`] | Render output: [`RenderCommand`], [`RenderData`] |
//! | [`input`] | Input state: [`PointerData`], [`ScrollContainerData`] |
//! | [`context`] | The main [`Context`] that drives layout calculation |

pub mod types;
pub mod layout;
pub mod elements;
pub mod render;
pub mod input;
pub mod context;
pub mod hash;
pub mod arena;
pub mod text;
pub mod layout_calc;
pub mod render_commands;
pub mod input_handling;
pub mod debug;

pub use types::*;
pub use layout::*;
pub use elements::*;
pub use render::*;
pub use input::*;
pub use context::Context;
