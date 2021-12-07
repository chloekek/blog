use glam::{Mat4, Quat, Vec3};
use opengl::gl::types::*;

#[repr(C)]
pub struct Bone
{
    pub position: Vec3,
    pub rotation: Quat,
}

#[repr(C)]
pub struct Instance
{
    pub m_matrix: Mat4,
    pub bones: [Bone; 6],
}

pub struct InstanceSet
{
    buffer: GLuint,
}
