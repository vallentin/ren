// Unsafe code used for OpenGL calls
#![allow(unsafe_code)]

pub mod prelude {
    pub use glfw::{
        Action, Context, Glfw, Key, Modifiers, MouseButton, Scancode, Window, WindowEvent,
    };
    pub use glfw_ext::WindowExt;

    pub use super::{App, AppOptions, EventReceiver};
}

pub use glfw::{Action, Context, Glfw, Key, Modifiers, MouseButton, Scancode, Window, WindowEvent};

use std::error;
use std::sync::mpsc::Receiver;

#[cfg(debug_assertions)]
use std::iter;

use glfw::{OpenGlProfileHint, WindowHint, WindowMode};
use glfw_ext::WindowExt;

use crate::debug_output::{init_debug_output, is_debug_output_supported};
use crate::gl45::RenderingContext;

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

#[allow(unused_variables)]
pub trait App<'gl>: Sized {
    type Err: Into<Box<dyn error::Error>>;

    fn init(ctx: &mut RenderingContext<'gl>) -> Result<Self, Self::Err>;
    fn update(&mut self, ctx: &mut RenderingContext<'gl>, wnd: &mut Window) {}
    fn draw(&mut self, ctx: &mut RenderingContext<'gl>, wnd: &Window);
    fn on_event(&mut self, evt: WindowEvent, ctx: &mut RenderingContext<'gl>, wnd: &mut Window) {}
}

/// This is a helper trait, as it is currently not
/// possible to specify the following bound:
///
/// ```ignore
/// where
///     for<'gl> impl FnOnce(&mut RenderingContext<'gl>) -> App<'gl>
/// ```
///
/// See [`_run_app`] for more information.
///
/// Issue: [#70263](https://github.com/rust-lang/rust/issues/70263)
#[doc(hidden)]
pub trait InitApp<'gl> {
    type App: App<'gl>;

    fn init(
        self,
        ctx: &mut RenderingContext<'gl>,
    ) -> Result<Self::App, <Self::App as App<'gl>>::Err>;
}

impl<'gl, T: 'gl, F> InitApp<'gl> for F
where
    F: FnOnce(&mut RenderingContext<'gl>) -> Result<T, T::Err>,
    T: App<'gl>,
{
    type App = T;

    fn init(
        self,
        ctx: &mut RenderingContext<'gl>,
    ) -> Result<Self::App, <Self::App as App<'gl>>::Err> {
        (self)(ctx)
    }
}

/// This function is currently hidden, to avoid confusion with closures,
/// as it would not be possible to call [`_run_app`]:
///
/// ```compile_fail
/// # use std::convert::Infallible;
/// use ren::prelude::*;
/// # struct MyApp;
/// # impl<'gl> App<'gl> for MyApp {
/// #     type Err = Infallible;
/// #     fn init(ctx: &mut RenderingContext<'gl>) -> Result<Self, Self::Err> { Ok(Self {}) }
/// #     fn draw(&mut self, ctx: &mut RenderingContext<'gl>, wnd: &Window) {}
/// # }
/// ren::_run_app(|ctx| MyApp::init(ctx)).unwrap();
/// ```
///
/// Instead it requires defining a function:
///
/// ```no_run
/// # use std::convert::Infallible;
/// use ren::prelude::*;
/// # struct MyApp;
/// # impl<'gl> App<'gl> for MyApp {
/// #     type Err = Infallible;
/// #     fn init(ctx: &mut RenderingContext<'gl>) -> Result<Self, Self::Err> { Ok(Self {}) }
/// #     fn draw(&mut self, ctx: &mut RenderingContext<'gl>, wnd: &Window) {}
/// # }
/// fn init<'gl>(ctx: &mut RenderingContext<'gl>) -> Result<MyApp, <MyApp as App<'gl>>::Err> {
///     MyApp::init(ctx)
/// }
/// ren::_run_app(init).unwrap();
/// ```
///
/// This is due to a HRTB issue with closures. Thus to avoid confusion
/// this requirement hidden away in [`run!`](crate::run!) and
/// [`run_with!`](crate::run_with!).
///
/// Issue: [#70263](https://github.com/rust-lang/rust/issues/70263)
#[doc(hidden)]
pub fn _run_app<F>(f: F) -> Result<(), Box<dyn error::Error>>
where
    F: for<'gl> InitApp<'gl>,
{
    _run_app_with(AppOptions::default(), f)
}

#[doc(hidden)]
pub fn _run_app_with<F>(opts: AppOptions<'_>, f: F) -> Result<(), Box<dyn error::Error>>
where
    F: for<'gl> InitApp<'gl>,
{
    let (mut glfw, mut wnd, events) = init(opts, true);
    // Safety: OpenGL context is current and `RenderingContext` cannot escape the closure
    let mut ctx = unsafe { RenderingContext::new() };
    let mut app = f.init(&mut ctx).map_err(Into::into)?;

    'main: while !wnd.should_close() {
        glfw.poll_events();

        for (_timestamp, evt) in glfw::flush_messages(&events) {
            match evt {
                WindowEvent::FramebufferSize(w, h) => unsafe {
                    gl::Viewport(0, 0, w, h);
                },
                #[cfg(debug_assertions)]
                WindowEvent::Key(Key::Escape, _, glfw::Action::Press, _) => {
                    break 'main;
                }
                WindowEvent::Close => {
                    break 'main;
                }
                _ => {}
            }

            app.on_event(evt, &mut ctx, &mut wnd);
        }

        app.update(&mut ctx, &mut wnd);
        app.draw(&mut ctx, &wnd);

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

    Ok(())
}

pub fn run_headless_once<F>(f: F)
where
    F: for<'a> FnOnce(&mut RenderingContext<'a>),
{
    run_headless_once_with(AppOptions::default(), f);
}

pub fn run_headless_once_with<F>(opts: AppOptions<'_>, f: F)
where
    F: for<'a> FnOnce(&mut RenderingContext<'a>),
{
    let (_glfw, _wnd, _events) = init(opts, false);
    // Safety: OpenGL context is current and `RenderingContext` cannot escape the closure
    let mut ctx = unsafe { RenderingContext::new() };
    f(&mut ctx);
}

pub fn run_glfw<F>(f: F) -> Result<(), Box<dyn error::Error>>
where
    F: FnMut(&mut Glfw, &mut Window, &mut EventReceiver),
{
    run_glfw_with(AppOptions::default(), f)
}

pub fn run_glfw_with<F>(opts: AppOptions<'_>, mut f: F) -> Result<(), Box<dyn error::Error>>
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

    Ok(())
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
