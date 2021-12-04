use opengl::gl::{self, Gl, types::*};
use std::{error::Error, fmt};

////////////////////////////////////////////////////////////////////////////////
// Invidivial errors

/// Invidivual OpenGL error.
#[derive(Clone, Copy)]
pub struct GlError
{
    error: GLenum,
}

////////////////////////////////////////////////////////////////////////////////
// Collections of errors

/// Collection of OpenGL errors.
pub struct GlErrors
{
    errors: Vec<GlError>,
}

impl GlErrors
{
    /// Collect all OpenGL errors.
    ///
    /// If there are no errors, this method returns [`Ok`].
    /// Otherwise it returns the errors in the order in which they occurred.
    /// The error queue will be left empty when this method returns.
    #[inline(never)]
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn get_gl_errors(gl: &Gl) -> Result<(), Self>
    {
        let mut errors = Vec::new();
        loop {
            let error = gl.GetError();
            if error == gl::NO_ERROR {
                break;
            }
            if error == 0 {
                // According to the OpenGL API reference:
                // > If glGetError itself generates an error, it returns 0.
                // I’m not sure what we can do in that case.
                panic!("glGetError failed");
            }
            errors.push(GlError{error});
        }
        if errors.is_empty() {
            Ok(())
        } else {
            Err(Self{errors})
        }
    }
}

////////////////////////////////////////////////////////////////////////////////
// Formatting and error trait impls

impl fmt::Display for GlError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for GlError
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        match self.error {
            gl::INVALID_ENUM      => write!(f, "GL_INVALID_ENUM"),
            gl::INVALID_VALUE     => write!(f, "GL_INVALID_VALUE"),
            gl::INVALID_OPERATION => write!(f, "GL_INVALID_OPERATION"),
            gl::INVALID_FRAMEBUFFER_OPERATION
                => write!(f, "GL_INVALID_FRAMEBUFFER_OPERATION"),
            gl::OUT_OF_MEMORY     => write!(f, "GL_OUT_OF_MEMORY"),
            gl::STACK_UNDERFLOW   => write!(f, "GL_STACK_UNDERFLOW"),
            gl::STACK_OVERFLOW    => write!(f, "GL_STACK_OVERFLOW"),
            other                 => write!(f, "{}", other),
        }
    }
}

impl Error for GlError
{
}

impl fmt::Display for GlErrors
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(self, f)
    }
}

impl fmt::Debug for GlErrors
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result
    {
        fmt::Debug::fmt(&self.errors, f)
    }
}

impl Error for GlErrors
{
}
