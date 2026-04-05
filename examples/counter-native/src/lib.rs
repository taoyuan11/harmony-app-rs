use std::ffi::c_char;
use std::sync::atomic::{AtomicU32, Ordering};

static COUNTER: AtomicU32 = AtomicU32::new(0);
static MESSAGE: &[u8] = b"Hello from Rust and OHOS!\0";

#[unsafe(no_mangle)]
pub extern "C" fn ohos_app_get_message() -> *const c_char {
    MESSAGE.as_ptr().cast()
}

#[unsafe(no_mangle)]
pub extern "C" fn ohos_app_increment_counter() -> u32 {
    COUNTER.fetch_add(1, Ordering::SeqCst) + 1
}
