use glam::{Mat4, Quat, Vec3};
use opengl::gl::types::*;

#[repr(C)]
pub struct GenericBone
{
    pub position: Vec3,
    pub rotation: Quat,
}

#[repr(C)]
pub struct GenericInstance
{
    pub m_matrix: Mat4,
    pub bones: [GenericBone; 6],
}

pub struct GenericInstanceSet
{
    buffer: GLuint,
}
