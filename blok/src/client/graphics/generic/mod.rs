//! Pipeline for rendering triangle meshes.

pub use self::fragment_shader::*;

use crate::try_gl;
use anyhow::Result;
use defer_lite::defer;
use glam::{Mat4, Vec2, Vec3};
use opengl::gl::{self, types::*};
use std::{borrow::Borrow, mem::{size_of, size_of_val}, ptr::null};

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
#[repr(C)]
pub struct Vertex
{
    pub position: Vec3,
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
    vertex_buffer: GLuint,
    index_buffer: GLuint,
    index_count: usize,
}

impl Drop for Model
{
    fn drop(&mut self)
    {
        // SAFETY: Provided by caller of `new`.
        unsafe {
            gl::DeleteBuffers(1, &self.vertex_buffer);
            gl::DeleteBuffers(1, &self.index_buffer);
        }
    }
}

impl Model
{
    /// Create an empty model.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new() -> Result<Self>
    {
        let mut this = Self{vertex_buffer: 0, index_buffer: 0, index_count: 0};
        try_gl! { gl::CreateBuffers(1, &mut this.vertex_buffer); }
        try_gl! { gl::CreateBuffers(1, &mut this.index_buffer); }
        Ok(this)
    }

    /// Upload the vertices of the model.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn upload_vertices(&mut self, data: &[Vertex]) -> Result<()>
    {
        try_gl! {
            gl::NamedBufferData(
                /* buffer */ self.vertex_buffer,
                /* size   */ size_of_val(data) as _,
                /* data   */ data.as_ptr() as _,
                /* usage  */ gl::STATIC_DRAW,
            );
        }
        Ok(())
    }

    /// Upload the indices of the model.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn upload_indices(&mut self, data: &[u32]) -> Result<()>
    {
        try_gl! {
            gl::NamedBufferData(
                /* buffer */ self.index_buffer,
                /* size   */ size_of_val(data) as _,
                /* data   */ data.as_ptr() as _,
                /* usage  */ gl::STATIC_DRAW,
            );
        }
        self.index_count = data.len();
        Ok(())
    }
}

/// Parameters for a single rendering of a model.
pub struct Instance
{
    pub m_matrix: Mat4,
    pub bone_matrices: [Mat4; BONES],
}

/// Pipeline for rendering triangle meshes.
pub struct Pipeline
{
    program: GLuint,
    vertex_array: GLuint,
}

impl Drop for Pipeline
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

impl Pipeline
{
    /// Compile the pipeline.
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new(fragment_shader: &FragmentShader) -> Result<Self>
    {
        let mut this = Self{program: 0, vertex_array: 0};
        this.make_program(fragment_shader)?;
        this.make_vertex_array()?;
        Ok(this)
    }

    unsafe fn make_program(&mut self, fragment_shader: &FragmentShader)
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
                /* numSpecializationConstants */ 1,
                /* pConstantIndex */ &0,
                /* pConstantValue */ &(BONES as _),
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
        try_gl! { gl::UseProgram(self.program); }
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
                /* buffer       */ model.vertex_buffer,
                /* offset       */ 0,
                /* stride       */ size_of::<Vertex>() as _,
            );
        }

        // Bind index buffer.
        try_gl! {
            gl::BindBuffer(
                /* target */ gl::ELEMENT_ARRAY_BUFFER,
                /* buffer */ model.index_buffer,
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
        let mvp_matrix = mvp_matrix.as_ref().as_ptr();

        // Set uniforms specific to this instance.
        try_gl! {
            gl::UniformMatrix4fv(
                /* location  */ 0,
                /* count     */ 1,
                /* transpose */ gl::FALSE,
                /* value     */ mvp_matrix,
            );
        }
        try_gl! {
            gl::UniformMatrix4fv(
                /* location  */ 1,
                /* count     */ BONES as _,
                /* transpose */ gl::FALSE,
                /* value     */ instance.bone_matrices.as_ptr() as _,
            );
        }

        // Draw model for this instance.
        try_gl! {
            gl::DrawElements(
                /* mode    */ gl::TRIANGLES,
                /* count   */ model.index_count as _,
                /* type    */ gl::UNSIGNED_INT,
                /* indices */ null(),
            );
        }

        Ok(())
    }
}
