pub mod prelude {
    pub use super::{Attrib, AttribBindPoint, AttribBinding, AttribFormat, AttribKind};
}

use std::mem;

pub type Attrib = AttribFormat;

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum AttribKind {
    Float1,
    Float2,
    Float3,
    Float4,
}

impl AttribKind {
    fn gl_size_type(self) -> (u8, u32) {
        match self {
            Self::Float1 => (1, gl::FLOAT),
            Self::Float2 => (2, gl::FLOAT),
            Self::Float3 => (3, gl::FLOAT),
            Self::Float4 => (4, gl::FLOAT),
        }
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct AttribFormat {
    /// Attribute index.
    pub index: u32,
    pub kind: AttribKind,
    /// Relative offset in bytes between vertex elements.
    pub offset: u32,
}

impl AttribFormat {
    #[inline]
    pub const fn new(index: u32, kind: AttribKind) -> Self {
        Self::with_offset(index, kind, 0)
    }

    #[inline]
    pub const fn with_offset(index: u32, kind: AttribKind, offset: u32) -> Self {
        Self {
            index,
            kind,
            offset,
        }
    }

    #[inline]
    pub const fn typed_offset<T>(index: u32, kind: AttribKind) -> Self {
        Self::with_offset(index, kind, mem::size_of::<T>() as u32)
    }

    #[inline]
    pub unsafe fn apply(&self, vao: u32) {
        let (size, type_) = self.kind.gl_size_type();
        gl::VertexArrayAttribFormat(vao, self.index, size as i32, type_, gl::FALSE, self.offset);
    }

    #[inline]
    pub unsafe fn enable(&self, vao: u32) {
        gl::EnableVertexArrayAttrib(vao, self.index);
    }

    #[inline]
    pub unsafe fn disable(&self, vao: u32) {
        gl::DisableVertexArrayAttrib(vao, self.index);
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct AttribBinding {
    pub attrib_index: u32,
    pub buffer_binding_index: u32,
}

impl AttribBinding {
    pub fn new(attrib_index: u32, buffer_binding_index: u32) -> Self {
        Self {
            attrib_index,
            buffer_binding_index,
        }
    }

    pub unsafe fn apply(&self, vao: u32) {
        gl::VertexArrayAttribBinding(vao, self.attrib_index, self.buffer_binding_index);
    }
}

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub struct AttribBindPoint {
    /// Index of the buffer binding point.
    pub binding_index: u32,
    /// Offset in bytes of the first element.
    pub offset: u32,
    /// Distance in bytes between elements.
    pub stride: u32,
}

impl AttribBindPoint {
    #[inline]
    pub const fn new(binding_index: u32, offset: u32, stride: u32) -> Self {
        Self {
            binding_index,
            offset,
            stride,
        }
    }

    #[inline]
    pub const fn typed_stride<T>(binding_index: u32, offset: u32) -> Self {
        Self::new(binding_index, offset, mem::size_of::<T>() as u32)
    }

    #[inline]
    pub unsafe fn apply(&self, vao: u32, buffer: u32) {
        gl::VertexArrayVertexBuffer(
            vao,
            self.binding_index,
            buffer,
            self.offset as isize,
            self.stride as i32,
        );
    }
}
