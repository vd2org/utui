#![doc = include_str!("../README.md")]
#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod area;
mod context;
mod internal;
mod tui;

/// Widgets for text, vertical layout, and scrollable lists.
pub mod widgets;

pub use area::Area;
pub use context::Context;
pub use tui::Tui;
pub use widgets::*;
