use crate::client::graphics::GlErrors;
use anyhow::{Context, Result};
use opengl::gl::{Gl, types::*};

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
    // pub chunk_position: ivec3,
    // pub buffer: BufferRef<TrivialBlockFace>,
}

/// Specialized pipeline for rendering trivial blocks.
pub struct TrivialBlockPipeline
{
    program: GLuint,
}

impl TrivialBlockPipeline
{
    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn new(gl: &Gl) -> Result<Self>
    {
        let program = gl.CreateProgram();
        GlErrors::get_gl_errors(gl).context("glCreateProgram")?;

        Ok(Self{program})
    }

    #[doc = crate::doc_safety_opengl!()]
    pub unsafe fn render<I, M>(models: I)
        where I: IntoIterator<Item=M>
            , M: AsRef<TrivialBlockFaceSet>
    {
    }
}
