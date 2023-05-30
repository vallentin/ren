// Unsafe code used for OpenGL calls
#![allow(unsafe_code)]

pub mod prelude {
    pub use super::RenderingContext;
}

use std::marker::PhantomData;

pub struct RenderingContext<'gl> {
    phantom: PhantomData<&'gl ()>,
}

impl<'gl> RenderingContext<'gl> {
    /// # Safety
    ///
    /// Must only be called on a thread where there is a current
    /// OpenGL context. The returned `RenderingContext` must only
    /// exist, while the OpenGL context is valid.
    pub unsafe fn new() -> Self {
        Self {
            phantom: PhantomData,
        }
    }

    pub fn set_clear_color(&mut self, (r, g, b, a): (f32, f32, f32, f32)) {
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
    }

    pub fn clear_color_buffer(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }
}
