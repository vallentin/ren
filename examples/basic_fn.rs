use std::error;
use std::io::{self, Write};
use std::process::exit;

use ren::prelude::*;

fn main() {
    exit({
        let code = match try_main() {
            Ok(()) => 0,
            Err(err) => {
                eprintln!("error: {}", err);
                1
            }
        };
        let _ = io::stdout().flush();
        let _ = io::stderr().flush();
        code
    });
}

fn try_main() -> Result<(), Box<dyn error::Error>> {
    ren::run_glfw(|_glfw, wnd, events| {
        for (_timestamp, evt) in glfw::flush_messages(&events) {
            match evt {
                WindowEvent::FramebufferSize(w, h) => unsafe {
                    gl::Viewport(0, 0, w, h);
                },
                #[cfg(debug_assertions)]
                WindowEvent::Key(Key::Escape, _, Action::Press, _) => wnd.set_should_close(true),
                WindowEvent::Close => wnd.set_should_close(true),
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    })
}
