//  Copyright 2021 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
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
    log::error!("{info}");

    loop {}
}
