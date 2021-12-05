//! Graphics rendering pipelines.

pub use self::generic_fragment_shader::*;
pub use self::gl_debug::*;
pub use self::gl_error::*;
pub use self::trivial_block::*;

pub mod parameters;

mod generic_fragment_shader;
mod gl_debug;
mod gl_error;
mod trivial_block;
