use std::env;

fn main()
{
    // Use the library search path for the target operating system.
    // The LIBRARIES_* environment variables are set by shell.nix.
    let target_family = env::var("CARGO_CFG_TARGET_OS").unwrap();
    let libraries = env::var(format!("LIBRARIES_{}", target_family)).unwrap();
    let libraries = libraries.split(';').filter(|s| !s.is_empty());
    for library in libraries {
        println!("cargo:rustc-link-search={}", library);
    }
}
