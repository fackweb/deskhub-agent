use std::env;
use std::ptr::null_mut;
use windows_sys::Win32::System::Diagnostics::Debug::{
    FormatMessageW, FORMAT_MESSAGE_FROM_SYSTEM, FORMAT_MESSAGE_IGNORE_INSERTS,
};

//Get the absolute path of the current executable
pub fn get_executable_path() -> Option<String> {
    if let Ok(exe_path) = env::current_exe() {
        if let Some(path) = exe_path.to_str() {
            return Some(path.to_string());
        }
    }
    None
}

pub fn get_last_error_message(error_code: u32) -> String {
    let mut message_buffer: Vec<u16> = Vec::with_capacity(256);
    unsafe {
        let size = FormatMessageW(
            FORMAT_MESSAGE_FROM_SYSTEM | FORMAT_MESSAGE_IGNORE_INSERTS,
            null_mut(),
            error_code,
            0,
            message_buffer.as_mut_ptr(),
            message_buffer.capacity() as u32,
            null_mut(),
        );
        if size > 0 {
            message_buffer.set_len(size as usize);
            return String::from_utf16_lossy(&message_buffer);
        }
    }
    "Unknown error".to_string()
}
