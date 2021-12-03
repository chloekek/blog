use gl_generator::{Api, Fallbacks, Profile, Registry, StructGenerator};
use std::{env, fs::File, path::Path};

fn main()
{
    let dest = env::var("OUT_DIR").unwrap();

    let gl_vsn = (4, 5);
    let gl_exts = [];

    let wgl_vsn = (1, 0);
    let wgl_exts = [
        "WGL_ARB_create_context",
        "WGL_ARB_create_context_profile",
        "WGL_ARB_pixel_format",
    ];

    let mut file = File::create(Path::new(&dest).join("gl.rs")).unwrap();
    Registry::new(Api::Gl, gl_vsn, Profile::Core, Fallbacks::All, gl_exts)
        .write_bindings(StructGenerator, &mut file)
        .unwrap();

    let mut file = File::create(Path::new(&dest).join("wgl.rs")).unwrap();
    Registry::new(Api::Wgl, wgl_vsn, Profile::Core, Fallbacks::All, wgl_exts)
        .write_bindings(StructGenerator, &mut file)
        .unwrap();
}
