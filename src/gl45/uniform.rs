pub mod prelude {
    pub use super::{SetUniform, UniformLocation};
}

use std::ffi::{c_char, CStr, CString};
use std::fmt;

#[cfg(feature = "glam")]
use glam::Mat4;

use super::{GLHandle, Shader};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
#[repr(transparent)]
pub struct UniformLocation(pub(crate) u32);

impl UniformLocation {
    /// Returns `None` if `name` does not correspond to an active uniform variable.
    ///
    /// Panics if `name` contains a nul byte.
    #[inline]
    pub(crate) fn get_uniform_location(
        program: u32,
        name: impl AsRef<str>,
    ) -> Option<UniformLocation> {
        let name = name.as_ref();

        // Technically, a nul byte is valid UTF-8 and Rust strings
        // may contain them. However, it is assumed to never be the
        // case, when calling this method. Otherwise calling this
        // method will segfault in release builds.
        let c_name = CString::new(name);
        #[cfg(debug_assertions)]
        let c_name = c_name.unwrap_or_else(|err| panic!("{name:?} contains a nul byte: {err}"));
        #[cfg(not(debug_assertions))]
        let c_name = unsafe { c_name.unwrap_unchecked() };

        Self::get_uniform_location_from_c_str(program, c_name)
    }

    /// Returns `None` if `name` does not correspond to an active uniform variable.
    ///
    /// Panics if `name` does not end with a nul byte, or
    /// if `name` contains interior nul bytes.
    pub(crate) fn get_uniform_location_from_bytes_with_nul(
        program: u32,
        name: &[u8],
    ) -> Option<Self> {
        let name = CStr::from_bytes_with_nul(name).unwrap();
        Self::get_uniform_location_from_c_str(program, name)
    }

    pub(crate) unsafe fn get_uniform_location_from_bytes_with_nul_unchecked(
        program: u32,
        name: impl AsRef<[u8]>,
    ) -> Option<Self> {
        let name = CStr::from_bytes_with_nul_unchecked(name.as_ref());
        Self::get_uniform_location_from_c_str(program, name)
    }

    #[inline]
    pub(crate) fn get_uniform_location_from_c_str(
        program: u32,
        name: impl AsRef<CStr>,
    ) -> Option<Self> {
        let name = name.as_ref().as_ptr();
        unsafe { Self::get_uniform_location_from_c_char_ptr(program, name) }
    }

    /// The `name` must be a null-terminated string.
    #[inline]
    pub(crate) unsafe fn get_uniform_location_from_c_char_ptr(
        program: u32,
        name: *const c_char,
    ) -> Option<Self> {
        let loc = gl::GetUniformLocation(program, name);
        if loc >= 0 {
            Some(Self(loc as u32))
        } else {
            None
        }
    }
}

impl fmt::Debug for UniformLocation {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "UniformLocation({})", self.0)
    }
}

pub trait SetUniform<T>
where
    T: Copy,
{
    fn set_uniform(&self, loc: UniformLocation, value: T);
}

impl SetUniform<f32> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: f32) {
        unsafe {
            gl::ProgramUniform1f(self.gl_handle(), loc.0 as i32, value);
        }
    }
}

impl SetUniform<(f32,)> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, (x,): (f32,)) {
        self.set_uniform(loc, x);
    }
}

impl SetUniform<(f32, f32)> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, (x, y): (f32, f32)) {
        unsafe {
            gl::ProgramUniform2f(self.gl_handle(), loc.0 as i32, x, y);
        }
    }
}

impl SetUniform<(f32, f32, f32)> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, (x, y, z): (f32, f32, f32)) {
        unsafe {
            gl::ProgramUniform3f(self.gl_handle(), loc.0 as i32, x, y, z);
        }
    }
}

impl SetUniform<(f32, f32, f32, f32)> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, (x, y, z, w): (f32, f32, f32, f32)) {
        unsafe {
            gl::ProgramUniform4f(self.gl_handle(), loc.0 as i32, x, y, z, w);
        }
    }
}

impl SetUniform<[f32; 1]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: [f32; 1]) {
        self.set_uniform(loc, value[0]);
    }
}

impl SetUniform<[f32; 2]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: [f32; 2]) {
        unsafe {
            gl::ProgramUniform2fv(self.gl_handle(), loc.0 as i32, 1, value.as_ptr());
        }
    }
}

impl SetUniform<[f32; 3]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: [f32; 3]) {
        unsafe {
            gl::ProgramUniform3fv(self.gl_handle(), loc.0 as i32, 1, value.as_ptr());
        }
    }
}

impl SetUniform<[f32; 4]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: [f32; 4]) {
        unsafe {
            gl::ProgramUniform4fv(self.gl_handle(), loc.0 as i32, 1, value.as_ptr());
        }
    }
}

impl SetUniform<&[f32; 16]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: &[f32; 16]) {
        unsafe {
            gl::ProgramUniformMatrix4fv(
                self.gl_handle(),
                loc.0 as i32,
                1,
                gl::FALSE,
                value.as_ptr(),
            );
        }
    }
}

#[cfg(feature = "glam")]
impl SetUniform<&Mat4> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: &Mat4) {
        self.set_uniform(loc, value.as_ref())
    }
}

#[cfg(feature = "glam")]
impl SetUniform<Mat4> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: Mat4) {
        self.set_uniform(loc, value.as_ref())
    }
}

impl SetUniform<i32> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: i32) {
        unsafe {
            gl::ProgramUniform1i(self.gl_handle(), loc.0 as i32, value);
        }
    }
}

impl SetUniform<(i32,)> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, (x,): (i32,)) {
        self.set_uniform(loc, x);
    }
}

impl SetUniform<(i32, i32)> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, (x, y): (i32, i32)) {
        unsafe {
            gl::ProgramUniform2i(self.gl_handle(), loc.0 as i32, x, y);
        }
    }
}

impl SetUniform<(i32, i32, i32)> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, (x, y, z): (i32, i32, i32)) {
        unsafe {
            gl::ProgramUniform3i(self.gl_handle(), loc.0 as i32, x, y, z);
        }
    }
}

impl SetUniform<(i32, i32, i32, i32)> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, (x, y, z, w): (i32, i32, i32, i32)) {
        unsafe {
            gl::ProgramUniform4i(self.gl_handle(), loc.0 as i32, x, y, z, w);
        }
    }
}

impl SetUniform<[i32; 1]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: [i32; 1]) {
        self.set_uniform(loc, value[0]);
    }
}

impl SetUniform<[i32; 2]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: [i32; 2]) {
        unsafe {
            gl::ProgramUniform2iv(self.gl_handle(), loc.0 as i32, 1, value.as_ptr());
        }
    }
}

impl SetUniform<[i32; 3]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: [i32; 3]) {
        unsafe {
            gl::ProgramUniform3iv(self.gl_handle(), loc.0 as i32, 1, value.as_ptr());
        }
    }
}

impl SetUniform<[i32; 4]> for Shader<'_> {
    #[inline]
    fn set_uniform(&self, loc: UniformLocation, value: [i32; 4]) {
        unsafe {
            gl::ProgramUniform4iv(self.gl_handle(), loc.0 as i32, 1, value.as_ptr());
        }
    }
}
