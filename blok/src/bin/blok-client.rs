use anyhow::{Result, anyhow};
use blok::{
    client::graphics::{
        GlBuffer,
        generic,
        parameters,
        trivial_block,
    },
    try_gl,
};
use glam::{Mat4, Vec3, ivec2, ivec3, vec2, vec3};
use opengl::gl;
use std::{f32::consts::PI, ffi::c_void};

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
    let generic_fragment_shader = generic::FragmentShader::new()?;
    let generic_pipeline = generic::Pipeline::new(&generic_fragment_shader)?;
    let trivial_block_pipeline = trivial_block::Pipeline::new(&generic_fragment_shader)?;

    // Create rendering state.

    let model = generic::Model{
        vertices: GlBuffer::new_upload(&[
            generic::Vertex{
                position: vec3(-1.0, -1.0, 0.0),
                texcoord: vec2(0.0, 0.0),
                bone: 0,
            },
            generic::Vertex{
                position: vec3(1.0, -1.0, 0.0),
                texcoord: vec2(1.0, 0.0),
                bone: 0,
            },
            generic::Vertex{
                position: vec3(0.0, 1.0, 0.0),
                texcoord: vec2(0.0, 1.0),
                bone: 0,
            },
        ], gl::STATIC_DRAW)?,
        indices: GlBuffer::new_upload(&[
            0, 1, 2,
        ], gl::STATIC_DRAW)?,
    };

    let instance = generic::Instance{
        m_matrix: Mat4::IDENTITY,
        bone_matrices: [Mat4::IDENTITY; generic::BONES],
    };

    let generic_models: &[(_, &[_])] = &[
        (model, &[instance]),
    ];

    let trivial_block_face_sets = &[
        trivial_block::FaceSet{
            faces: GlBuffer::new_upload(&[
                // TODO: Call TrivialBlockFace::new.
                trivial_block::Face{
                    xy: 0,
                    zf: 3,
                    u: 0,
                    v: 0,
                },
                trivial_block::Face{
                    xy: 0,
                    zf: 4,
                    u: 15,
                    v: 7,
                },
                trivial_block::Face{
                    xy: 0,
                    zf: 0,
                    u: 7,
                    v: 3,
                },
            ], gl::STATIC_DRAW)?,
            chunk_position: ivec3(0, 0, 0),
        },
    ];

    'outer: loop {

        // Handle SDL events.
        for sdl_event in sdl_event_pump.poll_iter() {
            match sdl_event {
                sdl2::event::Event::Quit{..} => break 'outer,
                _ => (),
            }
        }

        draw(
            &generic_pipeline,
            &trivial_block_pipeline,
            generic_models,
            trivial_block_face_sets,
        )?;

        // Present buffer we drew to.
        sdl_window.gl_swap_window();

    }

    Ok(())
}

unsafe fn draw(
    generic_pipeline: &generic::Pipeline,
    trivial_block_pipeline: &trivial_block::Pipeline,
    generic_models: &[(generic::Model, &[generic::Instance])],
    trivial_block_face_sets: &[trivial_block::FaceSet],
) -> Result<()>
{
    try_gl! { gl::ClearColor(0.1, 0.9, 0.2, 1.0); }
    try_gl! { gl::Clear(gl::COLOR_BUFFER_BIT); }

    let v_matrix = Mat4::look_at_rh(
        /* eye    */ Vec3::new(2.0, -2.0, 2.0),
        /* center */ Vec3::new(0.0, 0.0, 0.0),
        /* up     */ Vec3::new(0.0, 0.0, 1.0),
    );

    let p_matrix = Mat4::perspective_rh(
        /* fov_y_radians */ PI / 4.0,
        /* aspect_ratio  */ 640.0 / 480.0,
        /* z_near        */ 1.0,
        /* z_far         */ 1000.0,
    );

    let vp_matrix = p_matrix * v_matrix;

    generic_pipeline.render(
        /* vp_matrix */ &vp_matrix,
        /* models    */ generic_models.iter().map(|(m, i)| (m, *i)),
    )?;

    trivial_block_pipeline.render(
        /* atlas_size */ &ivec2(16, 8),
        /* vp_matrix  */ &vp_matrix,
        /* models     */ trivial_block_face_sets,
    )?;

    Ok(())
}
