use crate::{client::graphics::GenericFragmentShader, try_gl};
use anyhow::Result;
use defer_lite::defer;
use glam::{IVec2, IVec3, Mat4};
use opengl::gl::{self, types::*};
use std::{borrow::Borrow, ptr::null};

static VERTEX_SHADER_BINARY: &'static [u8] =
    include_bytes!(
        concat!(
            env!("OUT_DIR"),
            "/client/graphics/trivial_block_pipeline/shader.vert.spv",
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
///
/// A trivial block is an opaque unit cube at integer coordinates.
/// The vertex shader will generate the four vertices of each face,
/// so the buffers passed to this pipeline store only one entry for each face.
/// Faces that are adjacent to other trivial blocks do not have to be included.
pub struct TrivialBlockPipeline
{
    program: GLuint,
    vertex_array: GLuint,
}

impl Drop for TrivialBlockPipeline
{
    fn drop(&mut self)
    {
        // SAFETY: Provided by caller of `new`.
        unsafe {
            gl::DeleteProgram(self.program);
            gl::DeleteVertexArrays(1, &self.vertex_array);
        }
    }
}

impl TrivialBlockPipeline
{
    /// Compile the pipeline.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new(fragment_shader: &GenericFragmentShader) -> Result<Self>
    {
        let mut this = Self{program: 0, vertex_array: 0};
        this.make_program(fragment_shader)?;
        this.make_vertex_array()?;
        Ok(this)
    }

    unsafe fn make_program(&mut self, fragment_shader: &GenericFragmentShader)
        -> Result<()>
    {
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

        self.program = try_gl! { gl::CreateProgram() };

        try_gl! { gl::AttachShader(self.program, vertex_shader); }
        try_gl! { gl::AttachShader(self.program, fragment_shader.as_raw()); }
        try_gl! { gl::LinkProgram(self.program); }
        try_gl! { gl::DetachShader(self.program, fragment_shader.as_raw()); }
        try_gl! { gl::DetachShader(self.program, vertex_shader); }

        Ok(())
    }

    unsafe fn make_vertex_array(&mut self) -> Result<()>
    {
        // Create vertex array.
        try_gl! { gl::CreateVertexArrays(1, &mut self.vertex_array); }

        // Convenient alias.
        let vao = self.vertex_array;

        // Enable vertex attributes.
        try_gl! { gl::EnableVertexArrayAttrib(vao, 0); }
        try_gl! { gl::EnableVertexArrayAttrib(vao, 1); }
        try_gl! { gl::EnableVertexArrayAttrib(vao, 2); }
        try_gl! { gl::EnableVertexArrayAttrib(vao, 3); }

        // Associate the attributes with the sole binding.
        try_gl! { gl::VertexArrayAttribBinding(vao, 0, 0); }
        try_gl! { gl::VertexArrayAttribBinding(vao, 1, 0); }
        try_gl! { gl::VertexArrayAttribBinding(vao, 2, 0); }
        try_gl! { gl::VertexArrayAttribBinding(vao, 3, 0); }

        // Configure the formats of the attributes.
        try_gl! { gl::VertexArrayAttribIFormat(vao, 0, 1, gl::UNSIGNED_BYTE,  0); }
        try_gl! { gl::VertexArrayAttribIFormat(vao, 1, 1, gl::UNSIGNED_BYTE,  1); }
        try_gl! { gl::VertexArrayAttribIFormat(vao, 2, 1, gl::UNSIGNED_SHORT, 2); }
        try_gl! { gl::VertexArrayAttribIFormat(vao, 3, 1, gl::UNSIGNED_SHORT, 4); }

        // We draw quads, and each quad has four vertices.
        try_gl! { gl::VertexArrayBindingDivisor(vao, 0, 4); }

        Ok(())
    }

    /// Render a collection of sets of trivial block faces.
    ///
    /// # Parameters
    ///
    /// <dl>
    /// <dt><code>atlas_size</code></dt>
    /// <dd>The number of textures in the texture atlas.</dd>
    /// <dt><code>vp_matrix</code></dt>
    /// <dd>The view–projection matrix to apply to each face.</dd>
    /// </dl>
    ///
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn render<'a, I, M>(
        &self,
        atlas_size: &IVec2,
        vp_matrix: &Mat4,
        models: I,
    ) -> Result<()>
        where I: IntoIterator<Item=M>
            , M: Borrow<TrivialBlockFaceSet>
    {
        // Select program and vertex array.
        try_gl! { gl::UseProgram(self.program); }
        try_gl! { gl::BindVertexArray(self.vertex_array); }

        // Configure face culling.
        try_gl! { gl::Enable(gl::CULL_FACE); }
        try_gl! { gl::CullFace(gl::BACK); }
        try_gl! { gl::FrontFace(gl::CCW); }

        // Set uniforms common to all chunks.
        try_gl! { gl::Uniform2f(1, atlas_size.x as f32, atlas_size.y as f32); }

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
