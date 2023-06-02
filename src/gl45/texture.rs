pub mod prelude {
    pub use super::{InternalFormat, PixelFormat, Texture, TextureFilter, TextureWrap};
}

use std::ffi::c_void;
use std::fmt;
use std::marker::PhantomData;

use super::{GLHandle, RenderingContext};

pub(super) unsafe fn init() {
    gl::PixelStorei(gl::UNPACK_ALIGNMENT, 1);
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u32)]
pub enum PixelFormat {
    R = gl::RED,
    Rg = gl::RG,
    Rgb = gl::RGB,
    Rgba = gl::RGBA,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u32)]
pub enum InternalFormat {
    R8 = gl::R8,
    Rg8 = gl::RG8,
    Rgb8 = gl::RGB8,
    Rgba8 = gl::RGBA8,
}

#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
#[repr(u32)]
pub enum TextureWrap {
    Repeat = gl::REPEAT,
    #[default]
    ClampToEdge = gl::CLAMP_TO_EDGE,
    MirroredRepeat = gl::MIRRORED_REPEAT,
}

#[derive(PartialEq, Eq, Clone, Copy, Default, Debug)]
#[repr(u32)]
pub enum TextureFilter {
    #[default]
    Nearest = gl::NEAREST,
    Linear = gl::LINEAR,
}

pub struct Texture<'gl> {
    handle: u32,
    size: (u32, u32),
    phantom: PhantomData<&'gl ()>,
}

impl Texture<'static> {
    /// # Safety
    ///
    /// Must only be called on a thread where there is a current
    /// OpenGL context. The returned `Texture` must only
    /// exist, while the OpenGL context is valid.
    #[inline]
    pub unsafe fn new_unsafe(size: (u32, u32), internal_format: InternalFormat) -> Self {
        Self::create(size, internal_format)
    }
}

impl<'gl> Texture<'gl> {
    #[inline]
    pub fn new(
        _ctx: &mut RenderingContext<'gl>,
        size: (u32, u32),
        internal_format: InternalFormat,
    ) -> Self {
        Self::create(size, internal_format)
    }

    fn create(size: (u32, u32), internal_format: InternalFormat) -> Self {
        let mut tex = {
            let mut handle = 0;
            unsafe {
                gl::CreateTextures(gl::TEXTURE_2D, 1, &mut handle);
            }
            debug_assert_ne!(handle, 0, "failed creating texture");
            // Constructed early to ensure `gl::DeleteTextures()` is called on error
            Self {
                handle,
                size,
                phantom: PhantomData,
            }
        };

        unsafe {
            gl::TextureStorage2D(
                tex.handle,
                1,
                internal_format as u32,
                tex.size.0 as i32,
                tex.size.1 as i32,
            );
        }

        tex.set_wrap(TextureWrap::default());
        tex.set_filter(TextureFilter::default());

        tex.set_parameter(gl::TEXTURE_BASE_LEVEL, 0);
        tex.set_parameter(gl::TEXTURE_MAX_LEVEL, 0);

        tex
    }

    #[inline]
    pub fn upload_image_data(
        &mut self,
        (width, height): (u32, u32),
        format: PixelFormat,
        pixels: impl AsRef<[u8]>,
    ) {
        self.upload_sub_image_data((0, 0), (width, height), format, pixels);
    }

    #[inline]
    pub unsafe fn upload_image_data_from_ptr(
        &mut self,
        (width, height): (u32, u32),
        format: PixelFormat,
        pixels: *const u8,
    ) {
        self.upload_sub_image_data_from_ptr((0, 0), (width, height), format, pixels);
    }

    pub fn upload_sub_image_data(
        &mut self,
        (x, y): (u32, u32),
        (width, height): (u32, u32),
        format: PixelFormat,
        pixels: impl AsRef<[u8]>,
    ) {
        let pixels = pixels.as_ref();

        debug_assert!(((width as usize) * (height as usize)) <= pixels.len());

        unsafe {
            self.upload_sub_image_data_from_ptr((x, y), (width, height), format, pixels.as_ptr());
        }
    }

    pub unsafe fn upload_sub_image_data_from_ptr(
        &mut self,
        (x, y): (u32, u32),
        (width, height): (u32, u32),
        format: PixelFormat,
        pixels: *const u8,
    ) {
        debug_assert!(x < (i32::MAX as u32));
        debug_assert!(y < (i32::MAX as u32));
        debug_assert!(width < (i32::MAX as u32));
        debug_assert!(height < (i32::MAX as u32));

        debug_assert!(self.size.0 >= (x + width));
        debug_assert!(self.size.1 >= (y + height));

        debug_assert!((self.size.0 * self.size.1) >= (width * height));

        unsafe {
            gl::TextureSubImage2D(
                self.handle,
                0,
                x as i32,
                y as i32,
                width as i32,
                height as i32,
                format as u32,
                gl::UNSIGNED_BYTE,
                pixels as *const c_void,
            );
        }
    }

    #[inline]
    pub fn set_wrap(&mut self, wrap: TextureWrap) {
        self.set_wrap_u(wrap);
        self.set_wrap_v(wrap);
    }

    #[inline]
    pub fn set_wrap_u(&mut self, wrap: TextureWrap) {
        self.set_parameter(gl::TEXTURE_WRAP_S, wrap as i32);
    }

    #[inline]
    pub fn set_wrap_v(&mut self, wrap: TextureWrap) {
        self.set_parameter(gl::TEXTURE_WRAP_T, wrap as i32);
    }

    #[inline]
    pub fn set_filter(&mut self, filter: TextureFilter) {
        self.set_parameter(gl::TEXTURE_MIN_FILTER, filter as i32);
        self.set_parameter(gl::TEXTURE_MAG_FILTER, filter as i32);
    }

    #[inline]
    fn set_parameter(&mut self, name: u32, value: i32) {
        unsafe {
            gl::TextureParameteri(self.handle, name, value);
        }
    }

    #[inline]
    pub unsafe fn bind(&self, unit: u32) {
        gl::BindTextureUnit(unit, self.handle);
    }

    #[inline]
    pub fn size(&self) -> (u32, u32) {
        self.size
    }
}

impl GLHandle for Texture<'_> {
    #[inline]
    unsafe fn gl_handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for Texture<'_> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteTextures(1, &self.handle);
        }
    }
}

impl fmt::Debug for Texture<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Texture({}, {:?})", self.handle, self.size)
    }
}
