//! Pipeline for rendering opaque unit cubes at integer coordinates.

use crate::{
    client::graphics::{
        GlBuffer,
        GlProgram,
        GlShader,
        GlUniform,
        generic::FragmentShader,
    },
    try_gl,
};
use anyhow::Result;
use glam::{IVec2, IVec3, Mat4};
use opengl::gl::{self, types::*};
use std::{borrow::Borrow, mem::size_of};

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
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Face
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
pub struct FaceSet
{
    /// The faces to draw for this chunk.
    pub faces: GlBuffer<Face>,

    /// An increment of 1 in either dimension corresponds
    /// to the adjacent chunk in that dimension.
    pub chunk_position: IVec3,
}

/// Specialized pipeline for rendering trivial blocks.
///
/// A trivial block is an opaque unit cube at integer coordinates.
/// The vertex shader will generate the four vertices of each face,
/// so the buffers passed to this pipeline store only one entry for each face.
/// Faces that are adjacent to other trivial blocks do not have to be included.
pub struct Pipeline
{
    program: GlProgram,
    vertex_array: GLuint,
}

impl Drop for Pipeline
{
    fn drop(&mut self)
    {
        // SAFETY: Provided by caller of `new`.
        unsafe {
            gl::DeleteVertexArrays(1, &self.vertex_array);
        }
    }
}

impl Pipeline
{
    /// Compile the pipeline.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new(fragment_shader: &FragmentShader) -> Result<Self>
    {
        let program = Self::make_program(fragment_shader)?;
        let mut this = Self{program, vertex_array: 0};
        this.make_vertex_array()?;
        Ok(this)
    }

    unsafe fn make_program(fragment_shader: &FragmentShader)
        -> Result<GlProgram>
    {
        let vertex_shader = GlShader::new(
            /* shader_type      */ gl::VERTEX_SHADER,
            /* shader_binary    */ VERTEX_SHADER_BINARY,
            /* constant_indices */ &[],
            /* constant_values  */ &[],
        )?;
        GlProgram::new(&[&vertex_shader, fragment_shader.as_shader()])
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

        // There is only one buffer entry for each face,
        // and faces consist of four vertices (its corners).
        // So drawing must advance only once every four vertices.
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
            , M: Borrow<FaceSet>
    {
        self.pre_render(atlas_size)?;
        for model in models {
            let model = model.borrow();
            self.render_one(vp_matrix, model)?;
        }
        Ok(())
    }

    /// Implementation detail of `render`.
    unsafe fn pre_render(&self, atlas_size: &IVec2) -> Result<()>
    {
        // Select program and vertex array.
        try_gl! { gl::UseProgram(self.program.as_raw()); }
        try_gl! { gl::BindVertexArray(self.vertex_array); }

        // Configure face culling.
        try_gl! { gl::Enable(gl::CULL_FACE); }
        try_gl! { gl::CullFace(gl::BACK); }
        try_gl! { gl::FrontFace(gl::CCW); }

        // Set uniforms common to all chunks.
        atlas_size.as_vec2().gl_uniform(1)?;

        Ok(())
    }

    /// Implementation detail of `render`.
    unsafe fn render_one(&self, vp_matrix: &Mat4, model: &FaceSet)
        -> Result<()>
    {
        // Compute the MVP matrix for this chunk.
        let m_vector = (16 * model.chunk_position).as_vec3();
        let m_matrix = Mat4::from_translation(m_vector);
        let mvp_matrix = *vp_matrix * m_matrix;

        // Set uniforms specific to this chunk.
        mvp_matrix.gl_uniform(2)?;

        // Select the buffer to read faces from.
        try_gl! {
            gl::BindVertexBuffer(
                /* bindingindex */ 0,
                /* buffer       */ model.faces.as_raw(),
                /* offset       */ 0,
                /* stride       */ size_of::<Face>() as _,
            );
        }

        // Draw all the faces in a single draw call.
        try_gl! {
            gl::DrawArraysInstanced(
                /* mode  */ gl::TRIANGLE_FAN,
                /* first */ 0,

                // Every face consists of four vertices.
                /* count */ 4,

                // According to the glDrawArraysInstanced manual entry,
                // attributes with divisor N advance once every N instances.
                // We want to advance once for each face, and our divisor is 4,
                // so we must multiply the face count by 4 here.
                /* primcount */ (4 * model.faces.len()) as _,
            );
        }

        Ok(())
    }
}
