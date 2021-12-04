use anyhow::{Context, Result};
use crate::client::graphics::GlErrors;
use opengl::gl::{Gl, types::*};
use std::{cell::RefCell, ffi::c_void, slice};

/// Buffer into which to collect OpenGL debug messages.
///
/// This is not to be confused with a vertex buffer.
pub struct GlDebugMessageBuffer
{
    messages: RefCell<Vec<String>>,
}

impl GlDebugMessageBuffer
{
    /// Create an empty buffer.
    pub fn new() -> Self
    {
        Self{messages: RefCell::new(Vec::new())}
    }

    /// Call `glDebugMessageCallback` with appropriate arguments.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn install(&self, gl: &Gl) -> Result<()>
    {
        gl.DebugMessageCallback(
            Some(Self::debug_callback),
            self as *const Self as *mut c_void,
        );
        GlErrors::get_gl_errors(gl).context("glDebugMessageCallback")
    }

    /// Write all collected debug messages to stderr and clear the buffer.
    pub fn flush(&self)
    {
        let mut messages = self.messages.borrow_mut();
        for message in messages.drain(..) {
            eprintln!("{}", message);
        }
    }

    extern "system" fn debug_callback(
        _source:    GLenum,
        _type:      GLenum,
        _id:        GLuint,
        _severity:  GLenum,
        length:     GLsizei,
        message:    *const GLchar,
        user_param: *mut c_void,
    )
    {
        unsafe {
            let this = user_param as *mut Self;
            let message = slice::from_raw_parts(message as _, length as usize);
            let message = String::from_utf8_lossy(message).into_owned();
            let mut messages = (*this).messages.borrow_mut();
            messages.push(message);
        }
    }
}
