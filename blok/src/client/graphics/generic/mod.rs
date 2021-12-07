pub use self::fragment_shader::*;

use anyhow::Result;
use glam::{Mat4, Vec2, Vec3};
use opengl::gl::types::*;
use std::borrow::Borrow;

mod fragment_shader;

#[repr(C)]
pub struct Vertex
{
    pub position: Vec3,
    pub texcoord: Vec2,

    /// Index of the bones to apply.
    ///
    /// At the moment exactly one bone is applied to each vertex.
    /// To apply no bones, set the bone rotation and position to zero.
    pub bone: u32,
}

pub struct Model<const BONES: usize>
{
    vertex_buffer: GLuint,
    index_buffer: GLuint,
}

pub struct Instance<const BONES: usize>
{
    pub m_matrix: Mat4,
    pub bone_matrices: [Mat4; BONES],
}

pub struct Pipeline<const BONES: usize>
{
}

impl<const BONES: usize> Pipeline<BONES>
{
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new() -> Result<Self>
    {
        todo!()
    }

    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn render<I, J, M, N>(models: I) -> Result<()>
        where I: IntoIterator<Item=(M, J)>
            , J: IntoIterator<Item=N>
            , M: Borrow<Model<BONES>>
            , N: Borrow<Instance<BONES>>
    {
        todo!()
    }
}
