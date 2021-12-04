use gl_generator::{Api, Fallbacks, GlobalGenerator, Profile, Registry};
use std::{env, fs::File, path::Path};

fn main()
{
    let dest = env::var_os("OUT_DIR").unwrap();

    let gl_vsn = (4, 5);
    let gl_exts = ["GL_ARB_gl_spirv"];

    let mut file = File::create(Path::new(&dest).join("gl.rs")).unwrap();
    Registry::new(Api::Gl, gl_vsn, Profile::Core, Fallbacks::All, gl_exts)
        .write_bindings(GlobalGenerator, &mut file)
        .unwrap();
}
