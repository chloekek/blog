/// OpenGL version number.
///
/// I refuse to use OpenGL without direct state access,
/// so we need at least this ancient version.
#[allow(missing_docs)]
pub mod opengl {
    pub const MAJOR: i32 = 4;
    pub const MINOR: i32 = 5;
}

/// Pixel format parameters.
///
/// These are sensible values that were picked arbitrarily.
#[allow(missing_docs)]
pub mod pixel_format {
    pub const COLOR_BITS: i32 = 24;
    pub const ALPHA_BITS: i32 = 8;
    pub const DEPTH_BITS: i32 = 24;
    pub const STENCIL_BITS: i32 = 8;
}
