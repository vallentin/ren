pub mod prelude {
    pub use super::{Buffer, BufferUsage};
}

use std::ffi::c_void;
use std::fmt;
use std::marker::PhantomData;
use std::mem;

use super::{GLHandle, RenderingContext};

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum BufferUsage {
    /// Modified once and used a few times.
    Stream,
    /// Modified once and used many times.
    Static,
    /// Modified repeatedly and used many times.
    Dynamic,
}

impl BufferUsage {
    const fn gl_draw_usage(self) -> u32 {
        match self {
            Self::Stream => gl::STREAM_DRAW,
            Self::Static => gl::STATIC_DRAW,
            Self::Dynamic => gl::DYNAMIC_DRAW,
        }
    }
}

pub struct Buffer<'gl> {
    handle: u32,
    size: usize,
    phantom: PhantomData<&'gl ()>,
}

impl Buffer<'static> {
    /// # Safety
    ///
    /// Must only be called on a thread where there is a current
    /// OpenGL context. The returned `Buffer` must only
    /// exist, while the OpenGL context is valid.
    #[inline]
    pub unsafe fn new_unsafe() -> Self {
        let [buf] = Self::create_multi();
        buf
    }

    /// # Safety
    ///
    /// Must only be called on a thread where there is a current
    /// OpenGL context. The returned `Buffer` must only
    /// exist, while the OpenGL context is valid.
    #[inline]
    pub unsafe fn new_multi_unsafe<const N: usize>() -> [Self; N] {
        Self::create_multi()
    }
}

impl<'gl> Buffer<'gl> {
    #[inline]
    pub fn new(_ctx: &mut RenderingContext<'gl>) -> Self {
        let [buf] = Self::create_multi();
        buf
    }

    #[inline]
    pub fn new_multi<const N: usize>(_ctx: &mut RenderingContext<'gl>) -> [Self; N] {
        Self::create_multi()
    }

    #[inline]
    pub fn with_data<T: Copy>(
        _ctx: &mut RenderingContext<'gl>,
        usage: BufferUsage,
        data: &[T],
    ) -> Self {
        let [mut buf] = Self::create_multi();
        buf.write(usage, data);
        buf
    }

    fn create_multi<const N: usize>() -> [Self; N] {
        let mut handles = [0; N];
        unsafe {
            gl::CreateBuffers(handles.len() as i32, handles.as_mut_ptr());
        }

        handles.map(|handle| {
            debug_assert_ne!(handle, 0, "failed creating buffer");
            Self {
                handle,
                size: 0,
                phantom: PhantomData,
            }
        })
    }

    pub fn write<T: Copy>(&mut self, usage: BufferUsage, data: &[T]) {
        self.size = data.len() * mem::size_of::<T>();

        unsafe {
            gl::NamedBufferData(
                self.handle,
                self.size as isize,
                data.as_ptr() as *const c_void,
                usage.gl_draw_usage(),
            );
        }
    }

    /// Read subset of buffer data into `data` at `offset` bytes.
    ///
    /// # Panics
    ///
    /// Panics if `data` at `offset` is out of bounds.
    pub fn read<T: Copy>(&self, offset: usize, data: &mut [T]) {
        let read_size = data.len() * mem::size_of::<T>();
        let read_end = offset + read_size;

        if read_end > self.size {
            panic!(
                "index out of bounds: the size is {} but the end index is {}",
                self.size, read_end
            );
        }

        unsafe {
            gl::GetNamedBufferSubData(
                self.handle,
                offset as isize,
                read_size as isize,
                data.as_mut_ptr() as *mut c_void,
            );
        }
    }

    /// Returns the byte size of the buffer's data.
    #[inline]
    pub fn size(&self) -> usize {
        self.size
    }

    /// Returns the byte size of the buffer's data.
    pub fn gl_size(&self) -> u32 {
        let mut size = 0;
        unsafe {
            gl::GetNamedBufferParameteriv(self.handle, gl::BUFFER_SIZE, &mut size);
        }
        size as u32
    }
}

impl GLHandle for Buffer<'_> {
    #[inline]
    unsafe fn gl_handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for Buffer<'_> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteBuffers(1, &self.handle);
        }
    }
}

impl fmt::Debug for Buffer<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Buffer({})", self.handle)
    }
}
