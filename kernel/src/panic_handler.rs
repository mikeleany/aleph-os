//! Provides a [panic handler] for the kernel.
//!
//! Rust requires each binary to have exactly one panic handler. Usually the panic handler is
//! provided by `std`, but `std`'s panic handler is not available for [`no_std`] binaries.
//!
//! [panic handler]: https://doc.rust-lang.org/stable/reference/runtime.html#the-panic_handler-attribute
//! [`no_std`]: https://doc.rust-lang.org/stable/reference/names/preludes.html#the-no_std-attribute
use core::panic::PanicInfo;

/// The kernel's panic handler.
///
/// It logs an [error][log::error] and halts execution.
#[panic_handler]
fn panic(info: &PanicInfo) -> ! {
    log::error!("{}", info);

    loop {}
}
