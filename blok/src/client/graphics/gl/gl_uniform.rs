use crate::try_gl;
use anyhow::Result;
use glam::{Mat4, Vec2};
use opengl::gl::{self, types::*};

/// Trait for objects that can be specified as uniforms.
pub trait GlUniform
{
    /// Specify the uniform value for the given uniform.
    #[doc = crate::doc_safety_opengl!()]
    unsafe fn gl_uniform(&self, location: GLint) -> Result<()>;
}

impl GlUniform for Vec2
{
    unsafe fn gl_uniform(&self, location: GLint) -> Result<()>
    {
        try_gl! { gl::Uniform2f(location, self.x, self.y); }
        Ok(())
    }
}

impl GlUniform for Mat4
{
    unsafe fn gl_uniform(&self, location: GLint) -> Result<()>
    {
        try_gl! {
            gl::UniformMatrix4fv(
                /* location  */ location,
                /* count     */ 1,
                /* transpose */ gl::FALSE,
                /* value     */ self.as_ref().as_ptr(),
            );
        }
        Ok(())
    }
}

impl GlUniform for [Mat4]
{
    unsafe fn gl_uniform(&self, location: GLint) -> Result<()>
    {
        try_gl! {
            gl::UniformMatrix4fv(
                /* location  */ location,
                /* count     */ self.len() as _,
                /* transpose */ gl::FALSE,
                /* value     */ self.as_ptr() as _,
            );
        }
        Ok(())
    }
}
