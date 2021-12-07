use crate::{client::graphics::GlShader, try_gl};
use anyhow::Result;
use opengl::gl::{self, types::*};

/// Owned handle to an OpenGL program.
pub struct GlProgram
{
    raw: GLuint,
}

impl GlProgram
{
    /// Create and link a program.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new(shaders: &[&GlShader]) -> Result<Self>
    {
        let mut this = Self{raw: 0};

        // Create program object.
        this.raw = try_gl! { gl::CreateProgram() };

        // Attach shaders.
        for shader in shaders {
            try_gl! { gl::AttachShader(this.raw, shader.as_raw()); }
        }

        // Link program.
        try_gl! { gl::LinkProgram(this.raw); }

        // Detach shaders.
        for shader in shaders {
            try_gl! { gl::DetachShader(this.raw, shader.as_raw()); }
        }

        Ok(this)
    }

    /// The OpenGL name of the program.
    pub fn as_raw(&self) -> GLuint
    {
        self.raw
    }
}

impl Drop for GlProgram
{
    fn drop(&mut self)
    {
        // SAFETY: Provided by caller of `new`.
        unsafe {
            gl::DeleteProgram(self.raw);
        }
    }
}
