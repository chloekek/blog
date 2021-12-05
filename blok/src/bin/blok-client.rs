use anyhow::{Result, anyhow};
use blok::{
    client::graphics::{
        GenericFragmentShader,
        TrivialBlockFaceSet,
        TrivialBlockPipeline,
        parameters,
    },
    try_gl,
};
use glam::{Mat4, ivec2, ivec3};
use opengl::gl;
use std::ffi::c_void;

fn main() -> Result<()>
{
    unsafe {
        unsafe_main()
    }
}

unsafe fn unsafe_main() -> Result<()>
{
    // Obtain SDL features.
    let sdl_context = sdl2::init().map_err(|e| anyhow!(e))?;
    let sdl_video = sdl_context.video().map_err(|e| anyhow!(e))?;
    let mut sdl_event_pump = sdl_context.event_pump().map_err(|e| anyhow!(e))?;

    // Set required OpenGL features.
    let gl_attr = sdl_video.gl_attr();
    gl_attr.set_context_major_version(parameters::opengl::MAJOR);
    gl_attr.set_context_minor_version(parameters::opengl::MINOR);
    gl_attr.set_red_size(parameters::pixel_format::COLOR_BITS);
    gl_attr.set_green_size(parameters::pixel_format::COLOR_BITS);
    gl_attr.set_blue_size(parameters::pixel_format::COLOR_BITS);
    gl_attr.set_alpha_size(parameters::pixel_format::ALPHA_BITS);
    gl_attr.set_depth_size(parameters::pixel_format::DEPTH_BITS);

    // Create SDL window.
    let sdl_window =
        sdl_video
        .window("Blok", 640, 480)
        .opengl()
        .build().map_err(|e| anyhow!(e))?;

    // Assign the OpenGL context to a variable to inhibit dropping.
    let _gl_context = sdl_window.gl_create_context().map_err(|e| anyhow!(e))?;

    // Load OpenGL procedures into global function pointers.
    gl::load_with(|proc_name| {
        sdl_video.gl_get_proc_address(proc_name) as *const c_void
    });

    // Create rendering pipelines.
    let generic_fragment_shader = GenericFragmentShader::new()?;
    let trivial_block_pipeline = TrivialBlockPipeline::new(&generic_fragment_shader)?;

    'outer: loop {


        // Handle SDL events.
        for sdl_event in sdl_event_pump.poll_iter() {
            match sdl_event {
                sdl2::event::Event::Quit{..} => break 'outer,
                _ => (),
            }
        }

        draw(&trivial_block_pipeline)?;

        // Present buffer we drew to.
        sdl_window.gl_swap_window();

    }

    Ok(())
}

unsafe fn draw(trivial_block_pipeline: &TrivialBlockPipeline) -> Result<()>
{
    // Draw to the buffer.
    try_gl! { gl::ClearColor(0.1, 0.2, 0.9, 1.0); }
    try_gl! { gl::Clear(gl::COLOR_BUFFER_BIT); }
    let tbfs = TrivialBlockFaceSet::new(ivec3(0, 0, 0))?;
    trivial_block_pipeline.render(&ivec2(16, 8), &Mat4::IDENTITY, [&tbfs])?;
    Ok(())
}
