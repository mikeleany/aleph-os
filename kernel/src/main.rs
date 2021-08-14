//  Copyright 2021 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
#![doc = include_str!("../README.md")]
#![no_std]
#![no_main]
use rlibc as _; // needed for `memcpy`, etc when using `--build-std`
#[cfg(not(test))]
mod panic_handler;

/// The kernel's entry point.
///
/// Exported as `_start`, because that is the symbol the linker uses as the entry point, and since
/// we used the [`no_main`] attribute, `_start` is not provided for us.
///
/// [`no_main`]: https://doc.rust-lang.org/stable/reference/crates-and-source-files.html#the-no_main-attribute
#[export_name = "_start"]
fn main() -> ! {
    log::info!("Hello world!");
    panic!("testing the panic handler");
}
