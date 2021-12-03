#![cfg(target_os = "linux")]

use anyhow::Result;
use opengl::gl::Gl;

/// Create a window with a suitable current OpenGL context.
pub unsafe fn with_environment<F, R>(_then: F) -> Result<R>
    where F: FnOnce(&Gl) -> Result<R>
{
    todo!()
}
