use ren::prelude::*;

fn main() {
    ren::run(|_glfw, _wnd, events| {
        for (_timestamp, evt) in glfw::flush_messages(&events) {
            match evt {
                WindowEvent::FramebufferSize(w, h) => unsafe {
                    gl::Viewport(0, 0, w, h);
                },
                #[cfg(debug_assertions)]
                WindowEvent::Key(Key::Escape, _, glfw::Action::Press, _) => {
                    return;
                }
                WindowEvent::Close => {
                    return;
                }
                _ => {}
            }
        }

        unsafe {
            gl::ClearColor(1.0, 0.0, 1.0, 1.0);
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    });
}
