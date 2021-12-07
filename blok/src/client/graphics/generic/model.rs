use glam::{Vec2, Vec3};
use opengl::gl::types::*;

#[repr(C)]
pub struct Vertex
{
    pub position: Vec3,
    pub texcoord: Vec2,

    /// Indexes of the bones to apply.
    ///
    /// Bones are applied in the order they are given.
    /// For example, a value of `[0, 3, 2, 4]` for this field
    /// applies bone 0, then bone 3, then bone 2, and finally bone 4.
    /// To apply fewer bones than the length of the array,
    /// simply set the remaining bones’ parameters to zero.
    pub bone_path: [u8; 4],
}

pub struct Model
{
    buffer: GLuint,
}
