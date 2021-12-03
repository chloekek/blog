use anyhow::Result;

#[cfg(target_os = "linux")]
use blok::client::linux::with_environment;

#[cfg(target_os = "windows")]
use blok::client::windows::with_environment;

fn main() -> Result<()>
{
    unsafe {
        with_environment(|gl| {
            println!("大家好！");
            gl.ClearColor(0.1, 0.2, 0.9, 1.0);
            gl.Clear(opengl::gl::COLOR_BUFFER_BIT);
            std::thread::sleep(std::time::Duration::from_secs(1));
            Ok(())
        })
    }
}
