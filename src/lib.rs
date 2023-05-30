#![deny(unsafe_code)]
#![forbid(elided_lifetimes_in_paths)]
#![cfg_attr(debug_assertions, allow(dead_code, unused_imports, unreachable_code))]

pub mod prelude {
    pub use crate::app::prelude::*;
    pub use crate::gl45::prelude::*;
}

mod app;
mod debug_output;
mod gl45;

pub use crate::app::*;
pub use crate::gl45::*;

/// Run an [`App`] with the default [`AppOptions`], i.e. the same as:
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
/// ren::run_with!(MyApp, AppOptions::default()).unwrap();
/// ```
///
/// See [`run_with!`] for more information.
#[macro_export]
macro_rules! run {
    // See `crate::app::InitApp` for why this macro is necessary
    ($ty:ty) => {{
        $crate::run_with!($ty, $crate::AppOptions::<'_>::default())
    }};
}

/// Run an [`App`] with some [`AppOptions`].
///
/// ```no_run
/// use std::convert::Infallible;
/// use ren::prelude::*;
///
/// fn main() {
///     ren::run_with!(MyApp, AppOptions::default()).unwrap();
/// }
///
/// struct MyApp;
///
/// impl<'gl> App<'gl> for MyApp {
///     type Err = Infallible;
///
///     fn init(ctx: &mut RenderingContext<'gl>) -> Result<Self, Self::Err> {
///         ctx.set_clear_color((1.0, 1.0, 0.0, 1.0));
///         Ok(Self {})
///     }
///
///     fn draw(&mut self, ctx: &mut RenderingContext<'gl>, wnd: &Window) {
///         ctx.clear_color_buffer();
///     }
/// }
/// ```
#[macro_export]
macro_rules! run_with {
    // See `crate::app::InitApp` for why this macro is necessary
    ($ty:ty, $opts:expr) => {{
        fn init<'gl>(ctx: &mut RenderingContext<'gl>) -> Result<$ty, <$ty as App<'gl>>::Err> {
            <$ty>::init(ctx)
        }
        $crate::_run_app_with($opts, init)
    }};
}
