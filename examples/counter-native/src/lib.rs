use std::ffi::{CString, c_char};
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);
static MESSAGE: &[u8] = b"Hello from Rust and OHOS!\0";

#[allow(dead_code)]
mod ohos_log {
    use super::CString;
    #[cfg(target_env = "ohos")]
    use std::ffi::{c_char, c_int};

    pub const DEBUG: u32 = 3;
    pub const INFO: u32 = 4;
    pub const WARN: u32 = 5;
    pub const ERROR: u32 = 6;
    pub const FATAL: u32 = 7;
    pub const DOMAIN: u32 = 0x3433;

    #[cfg(target_env = "ohos")]
    unsafe extern "C" {
        fn cargo_ohos_app_hilog(
            level: u32,
            domain: u32,
            tag: *const c_char,
            message: *const c_char,
        ) -> c_int;
    }

    pub fn debug(tag: &str, message: impl AsRef<str>) -> bool {
        write(DEBUG, DOMAIN, tag, message)
    }

    pub fn info(tag: &str, message: impl AsRef<str>) -> bool {
        write(INFO, DOMAIN, tag, message)
    }

    pub fn warn(tag: &str, message: impl AsRef<str>) -> bool {
        write(WARN, DOMAIN, tag, message)
    }

    pub fn error(tag: &str, message: impl AsRef<str>) -> bool {
        write(ERROR, DOMAIN, tag, message)
    }

    pub fn fatal(tag: &str, message: impl AsRef<str>) -> bool {
        write(FATAL, DOMAIN, tag, message)
    }

    pub fn write(level: u32, domain: u32, tag: &str, message: impl AsRef<str>) -> bool {
        let tag = sanitize(tag, "rust");
        let message = sanitize(message.as_ref(), "");
        #[cfg(target_env = "ohos")]
        unsafe {
            return cargo_ohos_app_hilog(level, domain, tag.as_ptr(), message.as_ptr()) >= 0;
        }

        #[cfg(not(target_env = "ohos"))]
        {
            let _ = (level, domain, tag, message);
            false
        }
    }

    fn sanitize(value: &str, fallback: &str) -> CString {
        let normalized = if value.is_empty() {
            fallback.to_string()
        } else {
            value.replace('\0', " ")
        };
        CString::new(normalized).unwrap_or_else(|_| CString::new(fallback).unwrap())
    }
}

#[cfg(target_env = "ohos")]
unsafe extern "C" {
    fn cargo_ohos_app_density_scale() -> f64;
    fn cargo_ohos_app_font_scale() -> f64;
}

fn current_runtime_scales() -> (f64, f64) {
    #[cfg(target_env = "ohos")]
    unsafe {
        return (cargo_ohos_app_density_scale(), cargo_ohos_app_font_scale());
    }

    #[cfg(not(target_env = "ohos"))]
    {
        (1.0, 1.0)
    }
}

#[unsafe(no_mangle)]
pub extern "C" fn ohos_app_get_message() -> *const c_char {
    let _ = ohos_log::info("counter-native", "ohos_app_get_message called from DevEco shell");
    MESSAGE.as_ptr().cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn ohos_app_increment_counter() -> u32 {
    let value = COUNTER.fetch_add(1, Ordering::SeqCst) + 1;
    let (density_scale, font_scale) = current_runtime_scales();
    let _ = ohos_log::debug(
        "counter-native",
        format!(
            "counter incremented to {value}, density_scale={density_scale:.4}, font_scale={font_scale:.4}"
        ),
    );
    value
}
