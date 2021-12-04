use anyhow::{Context, Result};
use opengl::gl::{self, types::*};
use std::{error::Error, fmt};

////////////////////////////////////////////////////////////////////////////////
// Error propagation macro

/// For use by [`try_gl`].
#[doc(hidden)]
pub fn context<C>(result: Result<(), GlErrors>, context: C) -> Result<()>
    where C: 'static + fmt::Display + Send + Sync
{
    result.context(context)
}

/// Evaluate a call to OpenGL and check its errors.
///
/// If there are any errors, they are returned from the enclosing function.
/// The errors are annotated with the name of the OpenGL function that failed.
#[doc = crate::doc_safety_opengl!()]
#[macro_export]
macro_rules! try_gl
{
    { $gl:ident :: $proc:ident ( $($argument:expr),* $(,)? ) ; } => {
        { try_gl! { $gl::$proc($($argument),*) } ; }
    };

    { $gl:ident :: $proc:ident ( $($argument:expr),* $(,)? ) } => {
        {
            let result = $gl::$proc($($argument),*);
            $crate::client::graphics::context(
                $crate::client::graphics::GlErrors::get_gl_errors(),
                concat!("gl", stringify!($proc)),
            )?;
            result
        }
    };
}

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
    #[doc = crate::doc_safety_opengl!()]
    #[inline(never)]
    pub unsafe fn get_gl_errors() -> Result<(), Self>
    {
        let mut errors = Vec::new();
        loop {
            let error = gl::GetError();
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
