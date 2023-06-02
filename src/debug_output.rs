// Unsafe code used for OpenGL calls
#![allow(unsafe_code)]

use std::ffi::{c_void, CStr};
use std::ptr;
use std::slice;

pub(crate) fn is_debug_output_supported((major, minor): (u32, u32)) -> bool {
    ((major == 4) && (minor >= 3)) || (major > 4)
}

#[must_use]
pub(crate) fn init_debug_output() -> bool {
    let debug_context = unsafe {
        let mut flags = 0;
        gl::GetIntegerv(gl::CONTEXT_FLAGS, &mut flags);
        (flags & (gl::CONTEXT_FLAG_DEBUG_BIT as i32)) != 0
    };

    let (major, minor) = unsafe {
        let (mut major, mut minor) = (0, 0);
        gl::GetIntegerv(gl::MAJOR_VERSION, &mut major);
        gl::GetIntegerv(gl::MINOR_VERSION, &mut minor);
        (major, minor)
    };

    let debug_output_supported = is_debug_output_supported((major as u32, minor as u32));
    if debug_context && debug_output_supported {
        unsafe {
            gl::Enable(gl::DEBUG_OUTPUT);
            gl::Enable(gl::DEBUG_OUTPUT_SYNCHRONOUS);

            gl::DebugMessageCallback(Some(debug_output), ptr::null());
            gl::DebugMessageControl(
                gl::DONT_CARE,
                gl::DONT_CARE,
                gl::DONT_CARE,
                0,
                ptr::null(),
                gl::TRUE,
            );
        }

        true
    } else {
        false
    }
}

pub(crate) extern "system" fn debug_output(
    source: u32,
    message_type: u32,
    id: u32,
    severity: u32,
    length: i32,
    message: *const i8,
    _user_param: *mut c_void,
) {
    // All OpenGL Errors, shader compilation/linking errors, or highly-dangerous undefined behavior
    // use gl::DEBUG_SEVERITY_HIGH as HIGH;
    // Major performance warnings, shader compilation/linking warnings, or the use of deprecated functionality
    // use gl::DEBUG_SEVERITY_MEDIUM as MEDIUM;
    // Redundant state change performance warning, or unimportant undefined behavior
    // use gl::DEBUG_SEVERITY_LOW as LOW;
    // Anything that isn't an error or performance issue.
    use gl::DEBUG_SEVERITY_NOTIFICATION as NOTIFICATION;
    // Reference: https://www.khronos.org/opengl/wiki/Debug_Output

    match (id, severity) {
        // Ignore "buffer will use video memory as source" notifications
        (131_185, NOTIFICATION) => return,
        _ => {}
    }

    let message = unsafe {
        CStr::from_bytes_with_nul_unchecked(slice::from_raw_parts(
            message as *const u8,
            length as usize,
        ))
    };
    let message = message.to_str().unwrap();

    eprintln!("Message: {}", message);
    eprintln!(
        "Source: {}",
        match source {
            gl::DEBUG_SOURCE_API => "API",
            gl::DEBUG_SOURCE_SHADER_COMPILER => "Shader Compiler",
            gl::DEBUG_SOURCE_WINDOW_SYSTEM => "Window System",
            gl::DEBUG_SOURCE_THIRD_PARTY => "Third Party",
            gl::DEBUG_SOURCE_APPLICATION => "Application",
            gl::DEBUG_SOURCE_OTHER => "Other",
            _ => "Unknown",
        }
    );
    eprintln!(
        "Type: {}",
        match message_type {
            gl::DEBUG_TYPE_ERROR => "Error",
            gl::DEBUG_TYPE_DEPRECATED_BEHAVIOR => "Deprecated Behavior",
            gl::DEBUG_TYPE_UNDEFINED_BEHAVIOR => "Undefined Behavior",
            gl::DEBUG_TYPE_PERFORMANCE => "Performance",
            gl::DEBUG_TYPE_PORTABILITY => "Portability",
            gl::DEBUG_TYPE_MARKER => "Marker",
            gl::DEBUG_TYPE_PUSH_GROUP => "Push Group",
            gl::DEBUG_TYPE_POP_GROUP => "Pop Group",
            gl::DEBUG_TYPE_OTHER => "Other",
            _ => "Unknown",
        }
    );
    eprintln!("ID: {}", id);
    eprintln!(
        "Severity: {}",
        match severity {
            gl::DEBUG_SEVERITY_HIGH => "High",
            gl::DEBUG_SEVERITY_MEDIUM => "Medium",
            gl::DEBUG_SEVERITY_LOW => "Low",
            gl::DEBUG_SEVERITY_NOTIFICATION => "Notification",
            _ => "Unknown",
        }
    );
}
