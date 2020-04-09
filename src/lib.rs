#![feature(associated_type_defaults)]
#![feature(box_patterns)]
#![feature(box_syntax)]
#![feature(clamp)]
#![feature(const_fn)]
#![feature(div_duration)]
#![feature(drain_filter)]
#![feature(fn_traits)]
#![feature(trait_alias)]
#![feature(try_blocks)]
#![feature(unboxed_closures)]

#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate rutie;

pub use crate::engine::{Engine, Error as EngineError};
pub use crate::modules::slack::SlackModule;
pub use crate::modules::tone::ToneModule;
pub use crate::modules::{EngineModule, GameState, WindowOptions};

mod engine;
pub mod ext;
mod modules;
