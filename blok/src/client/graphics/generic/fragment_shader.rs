use crate::client::graphics::GlShader;
use anyhow::Result;
use opengl::gl;

static FRAGMENT_SHADER_BINARY: &'static [u8] =
    include_bytes!(
        concat!(
            env!("OUT_DIR"),
            "/client/graphics/generic/shader.frag.spv",
        )
    );

/// Fragment shader used by most pipelines.
pub struct FragmentShader
{
    inner: GlShader,
}

impl FragmentShader
{
    /// Compile the shader.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new() -> Result<Self>
    {
        let inner = GlShader::new(
            /* shader_type      */ gl::FRAGMENT_SHADER,
            /* shader_binary    */ FRAGMENT_SHADER_BINARY,
            /* constant_indices */ &[],
            /* constant_values  */ &[],
        )?;
        Ok(Self{inner})
    }

    /// The underlying shader.
    pub fn as_shader(&self) -> &GlShader
    {
        &self.inner
    }
}
