#![deny(unsafe_code)]
#![forbid(elided_lifetimes_in_paths)]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unreachable_code))]

pub mod prelude {
    pub use crate::app::prelude::*;
}

mod app;
mod debug_output;

pub use crate::app::{run, run_with};
pub use crate::app::{run_headless_once, run_headless_once_with};
