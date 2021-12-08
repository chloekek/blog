//! Pipeline for rendering triangle meshes.

pub use self::fragment_shader::*;

use crate::{
    client::graphics::{GlBuffer, GlProgram, GlShader, GlUniform},
    try_gl,
};
use anyhow::Result;
use glam::{Mat4, Vec2, Vec3};
use opengl::gl::{self, types::*};
use std::{borrow::Borrow, mem::size_of, ptr::null};

mod fragment_shader;

static VERTEX_SHADER_BINARY: &'static [u8] =
    include_bytes!(
        concat!(
            env!("OUT_DIR"),
            "/client/graphics/generic/shader.vert.spv",
        )
    );

/// Maximum number of bones supported.
pub const BONES: usize = 6;

/// Vertex in a model’s vertex buffer.
#[derive(Clone, Copy)]
#[repr(C)]
pub struct Vertex
{
    /// Position of the vertex in model space.
    pub position: Vec3,

    /// Texture coordinates of the vertex.
    pub texcoord: Vec2,

    /// Index of the bones to apply.
    ///
    /// At the moment exactly one bone is applied to each vertex.
    /// To apply no bone, set the bone to the identity matrix.
    pub bone: u32,
}

/// Vertex and index buffer for a model.
pub struct Model
{
    /// The vertices of the model.
    pub vertices: GlBuffer<Vertex>,

    /// The vertex indices of the model.
    pub indices: GlBuffer<u32>,
}

/// Parameters for a single rendering of a model.
pub struct Instance
{
    /// Model matrix for the instance.
    pub m_matrix: Mat4,

    /// Bone matrices for the instance.
    pub bone_matrices: [Mat4; BONES],
}

/// Pipeline for rendering triangle meshes.
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
            /* constant_indices */ &[0],
            /* constant_values  */ &[BONES as _],
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

        // Associate the attributes with the sole binding.
        try_gl! { gl::VertexArrayAttribBinding(vao, 0, 0); }
        try_gl! { gl::VertexArrayAttribBinding(vao, 1, 0); }
        try_gl! { gl::VertexArrayAttribBinding(vao, 2, 0); }

        // Configure the formats of the attributes.
        try_gl! { gl::VertexArrayAttribFormat(vao, 0, 3, gl::FLOAT, gl::FALSE, 0); }
        try_gl! { gl::VertexArrayAttribFormat(vao, 1, 2, gl::FLOAT, gl::FALSE, 12); }
        try_gl! { gl::VertexArrayAttribIFormat(vao, 2, 1, gl::UNSIGNED_INT, 20); }

        Ok(())
    }

    /// Render a collection of instances of models.
    ///
    /// The method signature looks a bit complicated.
    /// You pass a sequence of models to render instances of.
    /// For each model you also pass a sequence of instances.
    /// The pipeline will set up rendering of each model only once,
    /// then render all instances of that model in sequence.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn render<I, J, M, N>(&self, vp_matrix: &Mat4, models: I)
        -> Result<()>
        where I: IntoIterator<Item=(M, J)>
            , J: IntoIterator<Item=N>
            , M: Borrow<Model>
            , N: Borrow<Instance>
    {
        self.pre_render()?;
        for (model, instances) in models {
            let model = model.borrow();
            self.pre_render_model(model)?;
            for instance in instances {
                let instance = instance.borrow();
                self.render_instance(vp_matrix, model, instance)?;
            }
        }
        Ok(())
    }

    /// Implementation detail of `render`.
    unsafe fn pre_render(&self) -> Result<()>
    {
        // Select program and vertex array.
        try_gl! { gl::UseProgram(self.program.as_raw()); }
        try_gl! { gl::BindVertexArray(self.vertex_array); }

        // Configure face culling.
        try_gl! { gl::Enable(gl::CULL_FACE); }
        try_gl! { gl::CullFace(gl::BACK); }
        try_gl! { gl::FrontFace(gl::CCW); }

        Ok(())
    }

    /// Implementation detail of `render`.
    unsafe fn pre_render_model(&self, model: &Model) -> Result<()>
    {
        // Bind vertex buffer.
        try_gl! {
            gl::BindVertexBuffer(
                /* bindingindex */ 0,
                /* buffer       */ model.vertices.as_raw(),
                /* offset       */ 0,
                /* stride       */ size_of::<Vertex>() as _,
            );
        }

        // Bind index buffer.
        try_gl! {
            gl::BindBuffer(
                /* target */ gl::ELEMENT_ARRAY_BUFFER,
                /* buffer */ model.indices.as_raw(),
            );
        }

        Ok(())
    }

    /// Implementation detail of `render`.
    unsafe fn render_instance(
        &self,
        vp_matrix: &Mat4,
        model: &Model,
        instance: &Instance,
    ) -> Result<()>
    {
        // Compute the MVP matrix for this instance.
        let mvp_matrix = *vp_matrix * instance.m_matrix;

        // Set uniforms specific to this instance.
        mvp_matrix.gl_uniform(0)?;
        instance.bone_matrices.gl_uniform(1)?;

        // Draw model for this instance.
        try_gl! {
            gl::DrawElements(
                /* mode    */ gl::TRIANGLES,
                /* count   */ model.indices.len() as _,
                /* type    */ gl::UNSIGNED_INT,
                /* indices */ null(),
            );
        }

        Ok(())
    }
}
