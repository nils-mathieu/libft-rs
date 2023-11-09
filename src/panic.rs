//! Provides a panic handler implementation that prints to the standard error stream.

use core::panic::PanicInfo;

/// This function is called when something in the code panics.
///
/// Because we can never support unwinding, we just abort the process. This is fine because
/// any panic should generally be considered a bug in the program, and aborting is the only
/// sensible thing to do.
#[panic_handler]
fn handle_panic(_info: &PanicInfo) -> ! {
    unsafe { libc::abort() }
}

// On some platforms, there seem to be some generated code forcing the a exception handler
// personality to be present in the binary even if it never actually used.
#[lang = "eh_personality"]
fn he_personality() {}
