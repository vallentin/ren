pub mod prelude {
    pub use super::{Shader, ShaderError, ShaderStage, ShaderStageError, ShaderStageKind};
}

use std::borrow::Cow;
use std::fmt;
use std::marker::PhantomData;
use std::ptr;

use thiserror::Error;

use super::{GLHandle, RawGLHandle, RenderingContext};

macro_rules! c_str {
    ($s:literal) => {
        concat!($s, "\0").as_ptr() as *const ::std::os::raw::c_char
    };
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
#[repr(u32)]
pub enum ShaderStageKind {
    Vertex = gl::VERTEX_SHADER,
    Fragment = gl::FRAGMENT_SHADER,
    Geometry = gl::GEOMETRY_SHADER,
    Compute = gl::COMPUTE_SHADER,
}

impl ShaderStageKind {
    const fn name(&self) -> &'static str {
        match self {
            Self::Vertex => "vertex",
            Self::Fragment => "fragment",
            Self::Geometry => "geometry",
            Self::Compute => "compute",
        }
    }
}

pub struct ShaderStage<'gl> {
    handle: u32,
    kind: ShaderStageKind,
    phantom: PhantomData<&'gl ()>,
}

impl ShaderStage<'static> {
    /// # Safety
    ///
    /// Must only be called on a thread where there is a current
    /// OpenGL context. The returned `ShaderStage` must only
    /// exist, while the OpenGL context is valid.
    #[inline]
    pub unsafe fn new_unsafe(
        kind: ShaderStageKind,
        source: impl AsRef<str>,
    ) -> Result<Self, ShaderStageError> {
        Self::create(kind, source)
    }
}

impl<'gl> ShaderStage<'gl> {
    #[inline]
    pub fn new(
        _ctx: &mut RenderingContext<'gl>,
        kind: ShaderStageKind,
        source: impl AsRef<str>,
    ) -> Result<Self, ShaderStageError> {
        Self::create(kind, source)
    }

    #[inline]
    pub fn new_vertex(
        _ctx: &mut RenderingContext<'gl>,
        source: impl AsRef<str>,
    ) -> Result<Self, ShaderStageError> {
        Self::create(ShaderStageKind::Vertex, source)
    }

    #[inline]
    pub fn new_fragment(
        _ctx: &mut RenderingContext<'gl>,
        source: impl AsRef<str>,
    ) -> Result<Self, ShaderStageError> {
        Self::create(ShaderStageKind::Fragment, source)
    }

    #[inline]
    pub fn new_geometry(
        _ctx: &mut RenderingContext<'gl>,
        source: impl AsRef<str>,
    ) -> Result<Self, ShaderStageError> {
        Self::create(ShaderStageKind::Geometry, source)
    }

    #[inline]
    pub fn new_compute(
        _ctx: &mut RenderingContext<'gl>,
        source: impl AsRef<str>,
    ) -> Result<Self, ShaderStageError> {
        Self::create(ShaderStageKind::Compute, source)
    }

    fn create(kind: ShaderStageKind, source: impl AsRef<str>) -> Result<Self, ShaderStageError> {
        let mut shader = {
            let handle = unsafe { gl::CreateShader(kind as u32) };
            debug_assert_ne!(handle, 0, "failed creating {} shader stage", kind.name());
            // Constructed early to ensure `gl::DeleteShader()` is called on error
            Self {
                handle,
                kind,
                phantom: PhantomData,
            }
        };
        shader.compile(source)?;
        Ok(shader)
    }

    fn compile(&mut self, source: impl AsRef<str>) -> Result<(), ShaderStageError> {
        let source = source.as_ref();
        unsafe {
            gl::ShaderSource(
                self.handle,
                1,
                [source.as_ptr() as *const i8].as_ptr(),
                [source.len() as i32].as_ptr(),
            );
        }

        unsafe {
            gl::CompileShader(self.handle);
        }
        let is_compiled = unsafe {
            let mut status = 0;
            gl::GetShaderiv(self.handle, gl::COMPILE_STATUS, &mut status);
            status == 1
        };

        let log = get_shader_info_log(self.handle);

        if is_compiled {
            if let Some(log) = &log {
                eprintln!(
                    "Warning: Compiling {} shader stage:\n{}",
                    self.kind.name(),
                    log.trim(),
                );
            }

            Ok(())
        } else {
            let log = log
                .map(Cow::Owned)
                .unwrap_or_else(|| Cow::Borrowed("[no log]"));
            Err(ShaderStageError::Compile(
                RawGLHandle(self.handle),
                self.kind,
                log,
            ))
        }
    }
}

impl GLHandle for ShaderStage<'_> {
    #[inline]
    unsafe fn gl_handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for ShaderStage<'_> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteShader(self.handle);
        }
    }
}

impl fmt::Debug for ShaderStage<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shader({}, {:?})", self.handle, self.kind)
    }
}

impl<'gl> AsRef<ShaderStage<'gl>> for ShaderStage<'gl> {
    #[inline]
    fn as_ref(&self) -> &ShaderStage<'gl> {
        self
    }
}

pub struct Shader<'gl> {
    handle: u32,
    phantom: PhantomData<&'gl ()>,
}

impl Shader<'static> {
    /// # Safety
    ///
    /// Must only be called on a thread where there is a current
    /// OpenGL context. The returned `Shader` must only
    /// exist, while the OpenGL context is valid.
    #[inline]
    pub unsafe fn new_unsafe<'a>(
        stages: &[impl AsRef<ShaderStage<'a>>],
    ) -> Result<Self, ShaderError> {
        Self::create(stages)
    }
}

impl<'gl> Shader<'gl> {
    #[inline]
    pub fn new<'a>(
        _ctx: &mut RenderingContext<'gl>,
        stages: &[impl AsRef<ShaderStage<'a>>],
    ) -> Result<Self, ShaderError> {
        Self::create(stages)
    }

    fn create<'a>(stages: &[impl AsRef<ShaderStage<'a>>]) -> Result<Self, ShaderError> {
        let mut shader = {
            let handle = unsafe { gl::CreateProgram() };
            debug_assert_ne!(handle, 0, "failed creating shader program");
            // Constructed early to ensure `gl::DeleteProgram()` is called on error
            Self {
                handle,
                phantom: PhantomData,
            }
        };
        unsafe {
            attach_shaders(
                shader.handle,
                stages.iter().map(|stage| stage.as_ref().handle),
            );
        }
        let res = shader.init();
        unsafe {
            detach_shaders(
                shader.handle,
                stages.iter().map(|stage| stage.as_ref().handle),
            );
        }
        match res {
            Ok(()) => Ok(shader),
            Err(err) => Err(err),
        }
    }

    #[inline]
    fn bind_data_locations(&mut self) {
        unsafe {
            gl::BindFragDataLocation(self.handle, 0, c_str!("fragColor"));
        }
    }

    fn link(&mut self) -> Result<(), ShaderError> {
        unsafe {
            gl::LinkProgram(self.handle);
        }

        let is_linked = unsafe {
            let mut status = 0;
            gl::GetProgramiv(self.handle, gl::LINK_STATUS, &mut status);
            status == 1
        };
        self.check_log("Linking", is_linked)
            .map_err(|log| ShaderError::Link(RawGLHandle(self.handle), log))
    }

    fn validate(&mut self) -> Result<(), ShaderError> {
        unsafe {
            gl::ValidateProgram(self.handle);
        }

        let is_validated = unsafe {
            let mut status = 0;
            gl::GetProgramiv(self.handle, gl::VALIDATE_STATUS, &mut status);
            status == 1
        };
        self.check_log("Validating", is_validated)
            .map_err(|log| ShaderError::Validation(RawGLHandle(self.handle), log))
    }

    fn check_log(&self, op: &str, was_success: bool) -> Result<(), Cow<'static, str>> {
        let log = get_program_info_log(self.handle);
        if was_success {
            if let Some(log) = &log {
                eprintln!("Warning: {op} shader program:\n{}", log.trim());
            }
            Ok(())
        } else {
            let log = log
                .map(Cow::Owned)
                .unwrap_or_else(|| Cow::Borrowed("[no log]"));
            Err(log)
        }
    }

    fn init(&mut self) -> Result<(), ShaderError> {
        self.bind_data_locations();
        self.link()?;
        self.validate()?;
        Ok(())
    }

    #[inline]
    pub unsafe fn bind(&self) {
        gl::UseProgram(self.handle);
    }
}

impl GLHandle for Shader<'_> {
    #[inline]
    unsafe fn gl_handle(&self) -> u32 {
        self.handle
    }
}

impl Drop for Shader<'_> {
    fn drop(&mut self) {
        unsafe {
            gl::DeleteProgram(self.handle);
        }
    }
}

impl fmt::Debug for Shader<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Shader({})", self.handle)
    }
}

#[inline]
unsafe fn attach_shader(program: u32, shader: u32) {
    debug_assert_ne!(program, 0, "attaching to invalid shader program handle");
    debug_assert_ne!(shader, 0, "attaching invalid shader stage handle");

    gl::AttachShader(program, shader);
}

#[inline]
unsafe fn detach_shader(program: u32, shader: u32) {
    debug_assert_ne!(program, 0, "detaching to invalid shader program handle");
    debug_assert_ne!(shader, 0, "detaching invalid shader stage handle");

    unsafe {
        gl::DetachShader(program, shader);
    }
}

#[inline]
unsafe fn attach_shaders(program: u32, shaders: impl Iterator<Item = u32>) {
    for shader in shaders {
        attach_shader(program, shader);
    }
}

#[inline]
unsafe fn detach_shaders(program: u32, shaders: impl Iterator<Item = u32>) {
    for shader in shaders {
        detach_shader(program, shader);
    }
}

fn get_shader_info_log(handle: u32) -> Option<String> {
    // Length of the null-terminated info log string or 0 if the shader has no info log
    let mut len = 0;
    unsafe {
        gl::GetShaderiv(handle, gl::INFO_LOG_LENGTH, &mut len);
    }
    if len == 0 {
        return None;
    }

    let mut log = Vec::with_capacity(len as usize);

    unsafe {
        // Set the length of `log` to `len - 1` (the log excluding the null terminator)
        //
        // Safety: `u8` is `Copy` and does not `impl Drop`, while
        // `len - 1` is less than `capacity()`
        log.set_len((len - 1) as usize);

        gl::GetShaderInfoLog(handle, len, ptr::null_mut(), log.as_mut_ptr() as *mut _);
    }

    match String::from_utf8(log) {
        Ok(log) => Some(log),
        Err(err) => Some(String::from_utf8_lossy(&err.into_bytes()).into_owned()),
    }
}

fn get_program_info_log(handle: u32) -> Option<String> {
    // Length of the null-terminated info log string or 0 if the program has no info log
    let mut len = 0;
    unsafe {
        gl::GetProgramiv(handle, gl::INFO_LOG_LENGTH, &mut len);
    }

    if len == 0 {
        return None;
    }
    let mut log = Vec::with_capacity(len as usize);

    unsafe {
        // Set the length of `log` to `len - 1` (the log excluding the null terminator)
        //
        // Safety: `u8` is `Copy` and does not `impl Drop`, while
        // `len - 1` is less than `capacity()`
        log.set_len((len - 1) as usize);

        gl::GetProgramInfoLog(handle, len, ptr::null_mut(), log.as_mut_ptr() as *mut _);
    }

    match String::from_utf8(log) {
        Ok(log) => Some(log),
        Err(err) => Some(String::from_utf8_lossy(&err.into_bytes()).into_owned()),
    }
}

#[derive(Error, Debug)]
pub enum ShaderStageError {
    #[error("compiling {} shader stage [{0}] failed: {2}", .1.name())]
    Compile(RawGLHandle, ShaderStageKind, Cow<'static, str>),
}

#[derive(Error, Debug)]
pub enum ShaderError {
    #[error("linking shader program [{0}] failed: {1}")]
    Link(RawGLHandle, Cow<'static, str>),
    #[error("validating shader program [{0}] failed: {1}")]
    Validation(RawGLHandle, Cow<'static, str>),
}
