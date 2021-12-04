//! Graphics rendering pipelines.

pub use self::gl_debug::*;
pub use self::gl_error::*;
pub use self::trivial_block::*;

pub mod parameters;

mod gl_debug;
mod gl_error;
mod trivial_block;
