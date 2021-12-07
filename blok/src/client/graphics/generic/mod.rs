//! Pipeline for rendering triangle meshes.

pub use self::fragment_shader::*;

use crate::try_gl;
use anyhow::Result;
use defer_lite::defer;
use glam::{Mat4, Vec2, Vec3};
use opengl::gl::{self, types::*};
use std::borrow::Borrow;

mod fragment_shader;

static VERTEX_SHADER_BINARY: &'static [u8] =
    include_bytes!(
        concat!(
            env!("OUT_DIR"),
            "/client/graphics/generic/shader.vert.spv",
        )
    );

/// Maximum number of bones supported.
const BONES: GLuint = 6;

/// Vertex in a model.
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
}

/// Parameters for a single rendering of a model.
pub struct Instance
{
    pub m_matrix: Mat4,
    pub bone_matrices: [Mat4; BONES as usize],
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
                /* pConstantValue */ &BONES,
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

    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn render<I, J, M, N>(models: I) -> Result<()>
        where I: IntoIterator<Item=(M, J)>
            , J: IntoIterator<Item=N>
            , M: Borrow<Model>
            , N: Borrow<Instance>
    {
        todo!()
    }
}
