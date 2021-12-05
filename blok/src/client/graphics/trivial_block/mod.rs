use crate::{client::graphics::GenericFragmentShader, try_gl};
use anyhow::Result;
use defer_lite::defer;
use glam::{IVec3, Mat4};
use opengl::gl::{self, types::*};
use std::{borrow::Borrow, ptr::null};

static VERTEX_SHADER_BINARY: &'static [u8] =
    include_bytes!(
        concat!(
            env!("OUT_DIR"),
            "/client/graphics/trivial_block/shader.vert.spv",
        )
    );

/// Vertex buffer entry for the trivial block drawing pipeline.
///
/// Each entry represents a single face of a cube to be drawn.
/// Only visible faces should have entries constructed for them.
/// X, Y, and Z represent the position of the block within the chunk.
/// An increment of 1 in either dimension corresponds
/// to the adjacent block in that dimension.
/// U and V represent the position of the texture within the texture atlas.
/// An increment of 1 in either dimension corresponds
/// to the adjacent texture in that dimension.
#[repr(C)]
pub struct TrivialBlockFace
{
    /// X coordinate in the 4 MSbs, Y coordinate in the 4 LSbs.
    pub xy: u8,

    /// Z coordinate in the 4 MSbs, face selector in the 4 LSbs.
    ///
    /// The face selector must range only from 0 through 5,
    /// as there are only six faces in a cube.
    pub zf: u8,

    /// U coordinate.
    pub u: u16,

    /// V coordinate.
    pub v: u16,
}

/// Set of trivial block faces that appear in a chunk.
pub struct TrivialBlockFaceSet
{
    /// The position of the chunk that contains the faces.
    ///
    /// An increment of 1 in either dimension corresponds
    /// to the adjacent chunk in that dimension.
    pub chunk_position: IVec3,

    // pub buffer: BufferRef<TrivialBlockFace>,
}

/// Specialized pipeline for rendering trivial blocks.
pub struct TrivialBlockPipeline
{
    program: GLuint,
}

impl Drop for TrivialBlockPipeline
{
    fn drop(&mut self)
    {
        // SAFETY: Provided by caller of `new`.
        unsafe {
            gl::DeleteProgram(self.program);
        }
    }
}

impl TrivialBlockPipeline
{
    /// Compile the pipeline.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new(fragment_shader: &GenericFragmentShader) -> Result<Self>
    {
        // Mutating self so that if any step fails,
        // then the previous steps get cleaned up.
        let mut this = Self{program: 0};

        let vertex_shader = try_gl! { gl::CreateShader(gl::VERTEX_SHADER) };
        defer! { gl::DeleteShader(vertex_shader); }

        try_gl! {
            gl::ShaderBinary(
                /* count        */ 1,
                /* shaders      */ &vertex_shader,
                /* binaryFormat */ gl::SHADER_BINARY_FORMAT_SPIR_V_ARB,
                /* binary       */ VERTEX_SHADER_BINARY.as_ptr() as _,
                /* length       */ VERTEX_SHADER_BINARY.len() as _,
            );
        }

        try_gl! {
            gl::SpecializeShaderARB(
                /* shader         */ vertex_shader,
                /* pEntryPoint    */ "main\0".as_ptr() as _,
                /* numSpecializationConstants */ 0,
                /* pConstantIndex */ null(),
                /* pConstantValue */ null(),
            );
        }

        this.program = try_gl! { gl::CreateProgram() };

        try_gl! { gl::AttachShader(this.program, vertex_shader); }
        try_gl! { gl::AttachShader(this.program, fragment_shader.as_raw()); }
        try_gl! { gl::LinkProgram(this.program); }
        try_gl! { gl::DetachShader(this.program, fragment_shader.as_raw()); }
        try_gl! { gl::DetachShader(this.program, vertex_shader); }

        Ok(this)
    }

    /// Render a collection of sets of trivial block faces.
    ///
    /// # Parameters
    ///
    /// <dl>
    /// <dt><code>atlas_len</code></dt>
    /// <dd>The number of textures in one dimension of the texture atlas.</dd>
    /// <dt><code>vp_matrix</code></dt>
    /// <dd>The view–projection matrix to apply to each face.</dd>
    /// </dl>
    ///
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn render<'a, I, M>(
        &self,
        atlas_len: usize,
        vp_matrix: &Mat4,
        models: I,
    ) -> Result<()>
        where I: IntoIterator<Item=M>
            , M: Borrow<TrivialBlockFaceSet>
    {
        try_gl! { gl::UseProgram(self.program); }

        try_gl! { gl::Uniform1f(1, atlas_len as f32); }

        for model in models {
            let model = model.borrow();

            // Compute the MVP matrix for the chunk.
            let m_vector = (16 * model.chunk_position).as_vec3();
            let m_matrix = Mat4::from_translation(m_vector);
            let mvp_matrix = *vp_matrix * m_matrix;

            // Render the faces of the chunk.
            self.render_one(&mvp_matrix)?;
        }

        Ok(())
    }

    /// Implementation detail of `render`.
    unsafe fn render_one(&self, mvp_matrix: &Mat4) -> Result<()>
    {
        let mvp_matrix = mvp_matrix.as_ref().as_ptr();

        try_gl! { gl::UniformMatrix4fv(2, 1, gl::FALSE, mvp_matrix); }

        Ok(())
    }
}
