//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Architecture-specific functionality.

#[cfg(target_arch = "x86_64")]
mod x86_64;
#[cfg(target_arch = "x86_64")]
pub use self::x86_64::*;

#[cfg(target_arch = "aarch64")]
mod aarch64 {
    /// Performs initialization required for `aarch64`.
    pub fn init() {}
}
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;
