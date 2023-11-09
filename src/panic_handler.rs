//! Provides a panic handler implementation that prints to the standard error stream.

use core::panic::PanicInfo;
use core::sync::atomic::AtomicPtr;
use core::sync::atomic::Ordering::Relaxed;

/// The default panic handler, loaded into the [`PANIC_HANDLER`] function pointer by
/// default.
fn default_panic_handler(_info: &PanicInfo) -> ! {
    unsafe { libc::abort() }
}

/// The global panic handler to be called when something the code panics.
static PANIC_HANDLER: AtomicPtr<()> = AtomicPtr::new(default_panic_handler as _);

/// Sets the global panic handler function.
pub fn set_panic_handler(handler: fn(info: &PanicInfo) -> !) {
    PANIC_HANDLER.store(handler as _, Relaxed);
}

/// This function is called when something in the code panics.
///
/// Because we can never support unwinding, we just abort the process. This is fine because
/// any panic should generally be considered a bug in the program, and aborting is the only
/// sensible thing to do.
#[panic_handler]
fn handle_panic(info: &PanicInfo) -> ! {
    // Load the panic handler function pointer and call it.
    let fn_ptr = PANIC_HANDLER.load(Relaxed);
    let fn_ptr: fn(info: &PanicInfo) -> ! = unsafe { core::mem::transmute(fn_ptr) };
    fn_ptr(info)
}

// On some platforms, there seem to be some generated code forcing the a exception handler
// personality to be present in the binary even if it never actually used.
#[lang = "eh_personality"]
fn he_personality() {}
