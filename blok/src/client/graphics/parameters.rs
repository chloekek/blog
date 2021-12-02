//! Parameters for the graphics system.

/// OpenGL version number.
///
/// I refuse to use OpenGL without direct state access,
/// so we need at least this ancient version.
#[allow(missing_docs)]
pub mod opengl {
    pub const MAJOR: u8 = 4;
    pub const MINOR: u8 = 5;
}

/// Pixel format parameters.
///
/// These are sensible values that were picked arbitrarily.
#[allow(missing_docs)]
pub mod pixel_format {
    pub const COLOR_BITS: u8 = 8;
    pub const ALPHA_BITS: u8 = 8;
    pub const DEPTH_BITS: u8 = 24;
}
