//! Logging to logcat on Android, stderr elsewhere

use std::sync::atomic::{AtomicBool, Ordering};

static DEBUG: AtomicBool = AtomicBool::new(false);

/// Turn on or off per-change logging (`persist.vendor.go7fl.debug`)
pub fn set_debug(on: bool) {
    DEBUG.store(on, Ordering::Relaxed);
}

/// Whether per-change logging is on
#[must_use]
pub fn debug() -> bool {
    DEBUG.load(Ordering::Relaxed)
}

#[derive(Clone, Copy)]
pub enum Priority {
    Info,
    Warn,
}

#[cfg(target_os = "android")]
unsafe extern "C" {
    // liblog android/log.h
    fn __android_log_write(prio: libc::c_int, tag: *const libc::c_char, text: *const libc::c_char) -> libc::c_int;
}

/// Log line to logcat
#[cfg(target_os = "android")]
pub fn write(prio: Priority, msg: &str) {
    // android_LogPriority: INFO 4, WARN 5
    let prio = match prio {
        Priority::Info => 4,
        Priority::Warn => 5,
    };
    let text = std::ffi::CString::new(msg.replace('\0', "")).unwrap_or_default();

    // SAFETY: tag and text are NUL-terminated and outlive this call
    let _ = unsafe { __android_log_write(prio, c"frontlightd".as_ptr(), text.as_ptr()) };
}

/// Log line to stderr
#[cfg(not(target_os = "android"))]
pub fn write(prio: Priority, msg: &str) {
    let prio = match prio {
        Priority::Info => 'I',
        Priority::Warn => 'W',
    };
    eprintln!("{prio} frontlightd: {msg}");
}

macro_rules! alogi {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Priority::Info, &format!($($arg)*)) };
}
macro_rules! alogw {
    ($($arg:tt)*) => { $crate::log::write($crate::log::Priority::Warn, &format!($($arg)*)) };
}
pub(crate) use {alogi, alogw};
