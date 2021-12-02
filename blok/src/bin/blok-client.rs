use anyhow::{Result, anyhow};
use blok::client::graphics::parameters;
use opengl::gl::{self, Gl};
use std::ffi::c_void;

fn main() -> Result<()>
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
    let gl = Gl::load_with(|proc_name| {
        sdl_video.gl_get_proc_address(proc_name) as *const c_void
    });

    'outer: loop {

        // Handle SDL events.
        for sdl_event in sdl_event_pump.poll_iter() {
            match sdl_event {
                sdl2::event::Event::Quit{..} => break 'outer,
                _ => (),
            }
        }

        // Draw to the buffer.
        unsafe {
            gl.ClearColor(0.1, 0.2, 0.9, 1.0);
            gl.Clear(gl::COLOR_BUFFER_BIT);
        }

        // Present buffer we drew to.
        sdl_window.gl_swap_window();

    }

    Ok(())
}
