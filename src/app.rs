// Unsafe code used for OpenGL calls
#![allow(unsafe_code)]

pub mod prelude {
    pub use glfw::{
        Action, Context, Glfw, Key, Modifiers, MouseButton, Scancode, Window, WindowEvent,
    };
    pub use glfw_ext::WindowExt;

    pub use super::{AppOptions, EventReceiver};
}

pub use glfw::{Action, Context, Glfw, Key, Modifiers, MouseButton, Scancode, Window, WindowEvent};

use std::sync::mpsc::Receiver;

#[cfg(debug_assertions)]
use std::iter;

use glfw::{OpenGlProfileHint, WindowHint, WindowMode};
use glfw_ext::WindowExt;

use crate::debug_output::{init_debug_output, is_debug_output_supported};

pub type EventReceiver = Receiver<(f64, WindowEvent)>;

#[derive(Clone, Debug)]
pub struct AppOptions<'a> {
    pub title: &'a str,
    pub window_size: (u32, u32),
    pub gl_version: (u32, u32),
    pub gl_debug_output: bool,
}

impl Default for AppOptions<'static> {
    fn default() -> Self {
        Self {
            title: env!("CARGO_PKG_NAME"),
            window_size: Self::DEFAULT_WINDOW_SIZE,
            gl_version: Self::DEFAULT_GL_VERSION,
            gl_debug_output: Self::DEFAULT_GL_DEBUG_OUTPUT,
        }
    }
}

impl AppOptions<'static> {
    pub const DEFAULT_TITLE: &str = env!("CARGO_PKG_NAME");
    pub const DEFAULT_WINDOW_SIZE: (u32, u32) = (856, 482);
    pub const DEFAULT_GL_VERSION: (u32, u32) = (4, 5);
    pub const DEFAULT_GL_DEBUG_OUTPUT: bool = cfg!(debug_assertions);
}

pub fn run<F>(f: F)
where
    F: FnMut(&mut Glfw, &mut Window, &mut EventReceiver),
{
    run_with(AppOptions::default(), f);
}

pub fn run_with<F>(opts: AppOptions<'_>, mut f: F)
where
    F: FnMut(&mut Glfw, &mut Window, &mut EventReceiver),
{
    let (mut glfw, mut wnd, mut events) = init(opts, true);

    while !wnd.should_close() {
        glfw.poll_events();

        f(&mut glfw, &mut wnd, &mut events);

        wnd.swap_buffers();

        #[cfg(debug_assertions)]
        {
            iter::from_fn(|| match unsafe { gl::GetError() } {
                gl::NO_ERROR => None,
                err => Some(err),
            })
            .for_each(|err| eprintln!("gl error: 0x{:04X}", err));
        }
    }
}

pub fn run_headless_once<F>(f: F)
where
    F: FnOnce(),
{
    run_headless_once_with(AppOptions::default(), f);
}

pub fn run_headless_once_with<F>(opts: AppOptions<'_>, f: F)
where
    F: FnOnce(),
{
    let (_glfw, _wnd, _events) = init(opts, false);
    f();
}

fn init(opts: AppOptions<'_>, visible: bool) -> (Glfw, Window, EventReceiver) {
    let mut glfw = glfw::init(Some(glfw::Callback {
        f: |err, desc, _| panic!("glfw error [{}]: {}", err, desc),
        data: (),
    }))
    .expect("unable to initialize glfw");

    glfw.window_hint(WindowHint::ContextVersion(
        opts.gl_version.0,
        opts.gl_version.1,
    ));
    glfw.window_hint(WindowHint::OpenGlProfile(OpenGlProfileHint::Core));
    glfw.window_hint(WindowHint::OpenGlForwardCompat(true));
    glfw.window_hint(WindowHint::OpenGlDebugContext(
        opts.gl_debug_output && is_debug_output_supported(opts.gl_version),
    ));
    glfw.window_hint(WindowHint::Visible(false));

    let (mut wnd, events) = glfw
        .create_window(1280, 720, env!("CARGO_PKG_NAME"), WindowMode::Windowed)
        .unwrap();

    wnd.set_key_polling(true);
    wnd.set_mouse_button_polling(true);
    wnd.set_cursor_pos_polling(true);
    wnd.set_scroll_polling(true);
    wnd.set_framebuffer_size_polling(true);
    wnd.set_close_polling(true);

    wnd.try_center();

    wnd.make_current();

    gl::load_with(|symbol| wnd.get_proc_address(symbol) as *const _);

    if opts.gl_debug_output {
        if is_debug_output_supported(opts.gl_version) && init_debug_output() {
            println!("Enabled OpenGL debug output");
        } else {
            eprintln!("Warning: OpenGL debug output not supported");
        }
    }

    if visible {
        wnd.show();
    }

    (glfw, wnd, events)
}
