#![feature(exit_status_error)]

use std::{env, fs::create_dir_all, path::Path, process::Command};

fn main()
{
    set_link_search();
    compile_shaders();
}

fn set_link_search()
{
    // Use the library search path for the target operating system.
    // The LIBRARIES_* environment variables are set by shell.nix.
    let target_family = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let libraries_var = format!("LIBRARIES_{}", target_family);
    println!("cargo:rerun-if-env-changed={}", libraries_var);
    let libraries = env::var(libraries_var).unwrap();
    let libraries = libraries.split(';').filter(|s| !s.is_empty());
    for library in libraries {
        println!("cargo:rustc-link-search={}", library);
    }
}

fn compile_shaders()
{
    let shaders = &[
        ("frag", "client/graphics/generic_fragment_shader/shader.frag"),
        ("vert", "client/graphics/trivial_block/shader.vert"),
    ];

    let optimize = env::var("OPT_LEVEL").unwrap() != "0";
    let out_dir = env::var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);

    for (kind, source_path) in shaders {
        let src_source_path = Path::new("src").join(source_path);
        println!("cargo:rerun-if-changed={}", src_source_path.display());

        // Compute path for SPIR-V file.
        let mut spirv_path = Path::join(out_dir, source_path);
        spirv_path.set_extension(format!("{}.spv", kind));

        // Create directory for SPIR-V file.
        let spirv_dir = spirv_path.parent().unwrap();
        create_dir_all(&spirv_dir).unwrap();

        // Compile GLSL into SPIR-V.
        Command::new("glslc")
            .arg(if optimize { "-O" } else { "-O0" })
            .arg("--target-env=opengl4.5")
            .arg("-o")
            .arg(&spirv_path)
            .arg(src_source_path)
            .status().unwrap()
            .exit_ok().unwrap();
    }
}
