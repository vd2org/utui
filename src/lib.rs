#![doc = include_str!("../README.md")]
#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod area;
mod context;
mod internal;
mod tools;
mod tui;

pub mod widgets;

pub use area::Area;
pub use context::Context;
pub use tools::{init, install_panic_hook, restore};
pub use tui::Tui;
pub use widgets::*;
