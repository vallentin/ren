pub mod prelude {
    pub use super::{VertexArray, VertexArrayDesc};
}

use std::fmt;
use std::marker::PhantomData;

use crate::AttribBinding;

use super::{Attrib, AttribBindPoint, Buffer, GLHandle, RenderingContext};

#[derive(Clone, Debug)]
pub struct VertexArrayDesc<'gl, 'a> {
    buffers: Vec<&'a Buffer<'gl>>,
    bind_points: Vec<AttribBindPoint>,
    bindings: Vec<AttribBinding>,
    attribs: Vec<Attrib>,
}

impl<'gl, 'a> VertexArrayDesc<'gl, 'a> {
    pub fn new() -> Self {
        Self {
            buffers: Vec::new(),
            bind_points: Vec::new(),
            bindings: Vec::new(),
            attribs: Vec::new(),
        }
    }

    pub fn with_buffer(mut self, buffer: &'a Buffer<'gl>) -> Self {
        self.buffers.push(buffer);
        self
    }

    pub fn with_bind_point(mut self, bind_point: AttribBindPoint) -> Self {
        self.bind_points.push(bind_point);
        self
    }

    pub fn with_binding(mut self, binding: AttribBinding) -> Self {
        self.bindings.push(binding);
        self
    }

    pub fn with_attrib(mut self, attrib: Attrib) -> Self {
        self.attribs.push(attrib);
        self
    }

    pub unsafe fn apply(&self, vao: u32) {
        for (buffer_index, bind_point) in self.bind_points.iter().enumerate() {
            let buffer = &self.buffers[buffer_index];
            bind_point.apply(vao, buffer.gl_handle());
        }

        for binding in &self.bindings {
            binding.apply(vao);
        }

        for attrib in &self.attribs {
            attrib.enable(vao);
            attrib.apply(vao);
        }
    }
}

impl<'gl, 'a> AsRef<VertexArrayDesc<'gl, 'a>> for VertexArrayDesc<'gl, 'a> {
    #[inline]
    fn as_ref(&self) -> &VertexArrayDesc<'gl, 'a> {
        self
    }
}

pub struct VertexArray<'gl> {
    handle: u32,
    phantom: PhantomData<&'gl ()>,
}

impl VertexArray<'static> {
    /// # Safety
    ///
    /// Must only be called on a thread where there is a current
    /// OpenGL context. The returned `VertexArray` must only
    /// exist, while the OpenGL context is valid.
    #[inline]
    pub unsafe fn new_unsafe<'gl, 'a>(desc: impl AsRef<VertexArrayDesc<'gl, 'a>>) -> Self
    where
        'gl: 'a,
    {
        let arr = Self::create();
        unsafe {
            desc.as_ref().apply(arr.handle);
        }
        arr
    }
}

impl<'gl> VertexArray<'gl> {
    #[inline]
    pub fn new<'a>(
        _ctx: &mut RenderingContext<'gl>,
        desc: impl AsRef<VertexArrayDesc<'gl, 'a>>,
    ) -> Self
    where
        'gl: 'a,
    {
        let arr = Self::create();
        unsafe {
            desc.as_ref().apply(arr.handle);
        }
        arr
    }

    fn create() -> Self {
        let mut handle = 0;
        unsafe {
            gl::CreateVertexArrays(1, &mut handle);
        }
        debug_assert_ne!(handle, 0, "failed creating vertex array");
        Self {
            handle,
            phantom: PhantomData,
        }
    }

    #[inline]
    pub unsafe fn bind(&self) {
        gl::BindVertexArray(self.handle);
    }

    #[inline]
    pub unsafe fn draw_triangles(&self, first: u32, tri_count: u32) {
        self.draw_arrays(gl::TRIANGLES, first * 3, tri_count * 3);
    }

    #[inline]
    pub unsafe fn draw_points(&self, first: u32, vertex_count: u32) {
        self.draw_arrays(gl::POINTS, first, vertex_count);
    }

    #[inline]
    unsafe fn draw_arrays(&self, mode: u32, first: u32, vertex_count: u32) {
        gl::DrawArrays(mode, first as i32, vertex_count as i32);
    }
}

impl GLHandle for VertexArray<'_> {
    #[inline]
    unsafe fn gl_handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for VertexArray<'_> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteVertexArrays(1, &self.handle);
        }
    }
}

impl fmt::Debug for VertexArray<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "VertexArray({})", self.handle)
    }
}
