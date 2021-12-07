use crate::try_gl;
use anyhow::Result;
use opengl::gl::{self, types::*};
use std::{marker::PhantomData, mem::size_of_val};

/// Owned handle to an OpenGL buffer.
pub struct GlBuffer<T>
    where T: Copy
{
    _phantom: PhantomData<*mut [T]>,
    raw: GLuint,
    len: usize,
}

impl<T> GlBuffer<T>
    where T: Copy
{
    /// Create an empty buffer.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new() -> Result<Self>
    {
        let mut this = Self{_phantom: PhantomData, raw: 0, len: 0};
        try_gl! { gl::CreateBuffers(1, &mut this.raw); }
        Ok(this)
    }

    /// Create a buffer and upload data for it.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new_upload(data: &[T], usage: GLenum) -> Result<Self>
    {
        let mut this = Self::new()?;
        this.upload(data, usage)?;
        Ok(this)
    }

    /// Upload data to the buffer.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn upload(&mut self, data: &[T], usage: GLenum) -> Result<()>
    {
        try_gl! {
            gl::NamedBufferData(
                /* buffer */ self.raw,
                /* size   */ size_of_val(data) as _,
                /* data   */ data.as_ptr() as _,
                /* usage  */ usage,
            );
        }
        self.len = data.len();
        Ok(())
    }

    /// The OpenGL name of the buffer.
    pub fn as_raw(&self) -> GLuint
    {
        self.raw
    }

    /// The number of elements passed to `upload`.
    pub fn len(&self) -> usize
    {
        self.len
    }
}

impl<T> Drop for GlBuffer<T>
    where T: Copy
{
    fn drop(&mut self)
    {
        // SAFETY: Provided by caller of `new`.
        unsafe {
            gl::DeleteBuffers(1, &self.raw);
        }
    }
}
