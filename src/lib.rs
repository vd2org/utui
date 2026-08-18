#![doc = include_str!("../README.md")]
#![warn(missing_docs, rustdoc::broken_intra_doc_links)]

mod area;
mod context;
mod tools;
mod internal;
mod tui;

pub mod widgets;

pub use area::Area;
pub use context::Context;
pub use tools::{init, restore, install_panic_hook};
pub use tui::Tui;
pub use widgets::*;

