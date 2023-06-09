// Unsafe code used for OpenGL calls
#![allow(unsafe_code)]

pub mod prelude {
    pub use super::array::prelude::*;
    pub use super::attrib::prelude::*;
    pub use super::buffer::prelude::*;
    pub use super::shader::prelude::*;
    pub use super::texture::prelude::*;
    pub use super::uniform::prelude::*;

    pub use super::RenderingContext;
}

mod array;
mod attrib;
mod buffer;
mod shader;
mod texture;
mod uniform;

pub use self::array::*;
pub use self::attrib::*;
pub use self::buffer::*;
pub use self::shader::*;
pub use self::texture::*;
pub use self::uniform::*;

use std::fmt;
use std::marker::PhantomData;

pub trait GLHandle {
    unsafe fn gl_handle(&self) -> u32;
}

/// OpenGL handle wrapped in a struct, to ensure
/// the handle cannot "accidentally" be used.
///
/// Any handle wrapped in a `RawGLHandle` is not
/// guaranteed to be valid. For instance when
/// `RawGLHandle` is included in an error, produced
/// by a constructor method, then it is deleted before
/// returning.
///
/// Other cases such as method operating on existing
/// handles, e.g. writing and reading data. In these
/// cases the handles are likely still valid.
///
/// If needed, the inner OpenGL handle can be extracted
/// by calling <code>handle.[gl_handle()](RawGLHandle::gl_handle)</code>.
#[derive(PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Debug)]
#[repr(transparent)]
pub struct RawGLHandle(pub(crate) u32);

impl GLHandle for RawGLHandle {
    #[inline]
    unsafe fn gl_handle(&self) -> u32 {
        self.0
    }
}

impl fmt::Display for RawGLHandle {
    #[inline]
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(f)
    }
}

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
        self::texture::init();

        Self {
            phantom: PhantomData,
        }
    }

    #[inline]
    pub fn set_clear_color(&mut self, (r, g, b, a): (f32, f32, f32, f32)) {
        unsafe {
            gl::ClearColor(r, g, b, a);
        }
    }

    #[inline]
    pub fn clear_color_buffer(&mut self) {
        unsafe {
            gl::Clear(gl::COLOR_BUFFER_BIT);
        }
    }

    #[inline]
    pub fn create_buffer(&mut self) -> Buffer<'gl> {
        Buffer::new(self)
    }

    #[inline]
    pub fn create_buffers<const N: usize>(&mut self) -> [Buffer<'gl>; N] {
        Buffer::new_multi(self)
    }

    #[inline]
    pub fn create_buffer_with_data<T: Copy>(
        &mut self,
        usage: BufferUsage,
        data: &[T],
    ) -> Buffer<'gl> {
        Buffer::with_data(self, usage, data)
    }

    #[inline]
    pub fn create_vertex_array<'a>(
        &mut self,
        desc: impl AsRef<VertexArrayDesc<'gl, 'a>>,
    ) -> VertexArray<'gl>
    where
        'gl: 'a,
    {
        VertexArray::new(self, desc)
    }

    #[inline]
    pub fn create_texture(
        &mut self,
        size: (u32, u32),
        internal_format: InternalFormat,
    ) -> Texture<'gl> {
        Texture::new(self, size, internal_format)
    }

    #[inline]
    pub fn create_shader_stage(
        &mut self,
        kind: ShaderStageKind,
        source: impl AsRef<str>,
    ) -> Result<ShaderStage<'gl>, ShaderStageError> {
        ShaderStage::new(self, kind, source)
    }

    #[inline]
    pub fn create_shader_stage_vertex(
        &mut self,
        source: impl AsRef<str>,
    ) -> Result<ShaderStage<'gl>, ShaderStageError> {
        ShaderStage::new_vertex(self, source)
    }

    #[inline]
    pub fn create_shader_stage_fragment(
        &mut self,
        source: impl AsRef<str>,
    ) -> Result<ShaderStage<'gl>, ShaderStageError> {
        ShaderStage::new_fragment(self, source)
    }

    #[inline]
    pub fn create_shader_stage_geometry(
        &mut self,
        source: impl AsRef<str>,
    ) -> Result<ShaderStage<'gl>, ShaderStageError> {
        ShaderStage::new_geometry(self, source)
    }

    #[inline]
    pub fn create_shader_stage_compute(
        &mut self,
        source: impl AsRef<str>,
    ) -> Result<ShaderStage<'gl>, ShaderStageError> {
        ShaderStage::new_compute(self, source)
    }

    #[inline]
    pub fn create_shader<'a>(
        &mut self,
        stages: &[impl AsRef<ShaderStage<'a>>],
    ) -> Result<Shader<'gl>, ShaderError> {
        Shader::new(self, stages)
    }
}
