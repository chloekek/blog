use crate::try_gl;
use anyhow::Result;
use opengl::gl::{self, types::*};

/// Owned handle to an OpenGL shader.
pub struct GlShader
{
    raw: GLuint,
}

impl GlShader
{
    /// Create and specialize a shader.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new(
        shader_type: GLenum,
        shader_binary: &[u8],
        constant_indices: &[GLuint],
        constant_values: &[GLuint],
    ) -> Result<Self>
    {
        let mut this = Self{raw: 0};

        // Create shader object.
        this.raw = try_gl! { gl::CreateShader(shader_type) };

        // Supply shader SPIR-V code.
        try_gl! {
            gl::ShaderBinary(
                /* count        */ 1,
                /* shaders      */ &this.raw,
                /* binaryFormat */ gl::SHADER_BINARY_FORMAT_SPIR_V_ARB,
                /* binary       */ shader_binary.as_ptr() as _,
                /* length       */ shader_binary.len() as _,
            );
        }

        // Supply specialization constants.
        assert_eq!(constant_indices.len(), constant_values.len());
        try_gl! {
            gl::SpecializeShaderARB(
                /* shader         */ this.raw,
                /* pEntryPoint    */ "main\0".as_ptr() as _,
                /* numSpecializationConstants */ constant_indices.len() as _,
                /* pConstantIndex */ constant_indices.as_ptr(),
                /* pConstantValue */ constant_values.as_ptr(),
            );
        }

        Ok(this)
    }

    /// The OpenGL name of the shader.
    pub fn as_raw(&self) -> GLuint
    {
        self.raw
    }
}

impl Drop for GlShader
{
    fn drop(&mut self)
    {
        // SAFETY: Provided by caller of `new`.
        unsafe {
            gl::DeleteShader(self.raw);
        }
    }
}
