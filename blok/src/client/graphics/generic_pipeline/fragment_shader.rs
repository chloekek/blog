use crate::try_gl;
use anyhow::Result;
use opengl::gl::{self, types::*};
use std::ptr::null;

static FRAGMENT_SHADER_BINARY: &'static [u8] =
    include_bytes!(
        concat!(
            env!("OUT_DIR"),
            "/client/graphics/generic_pipeline/shader.frag.spv",
        )
    );

/// Fragment shader used by most pipelines.
pub struct GenericFragmentShader
{
    shader: GLuint,
}

impl Drop for GenericFragmentShader
{
    fn drop(&mut self)
    {
        // SAFETY: Provided by caller of `new`.
        unsafe {
            gl::DeleteShader(self.shader);
        }
    }
}

impl GenericFragmentShader
{
    /// Compile the shader.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new() -> Result<Self>
    {
        // Mutating self so that if any step fails,
        // then the previous steps get cleaned up.
        let mut this = Self{shader: 0};

        this.shader = try_gl! { gl::CreateShader(gl::FRAGMENT_SHADER) };

        try_gl! {
            gl::ShaderBinary(
                /* count        */ 1,
                /* shaders      */ &this.shader,
                /* binaryFormat */ gl::SHADER_BINARY_FORMAT_SPIR_V_ARB,
                /* binary       */ FRAGMENT_SHADER_BINARY.as_ptr() as _,
                /* length       */ FRAGMENT_SHADER_BINARY.len() as _,
            );
        }

        try_gl! {
            gl::SpecializeShaderARB(
                /* shader         */ this.shader,
                /* pEntryPoint    */ "main\0".as_ptr() as _,
                /* numSpecializationConstants */ 0,
                /* pConstantIndex */ null(),
                /* pConstantValue */ null(),
            );
        }

        Ok(this)
    }

    /// Obtain the OpenGL name of the shader.
    pub fn as_raw(&self) -> GLuint
    {
        self.shader
    }
}
