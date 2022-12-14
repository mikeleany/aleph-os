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

    pub mod mem {
        //! `aarch64`-specific types, methods and functions for dealing with memory.
        use core::ops::Add;

        use crate::mem::{PhysicalMemoryMap, VirtualAddress};

        /// Virtual address.
        #[derive(Debug, Clone, Copy)]
        pub struct VirtAddr(usize);

        impl VirtualAddress for VirtAddr {
            fn from_usize(_addr: usize) -> Option<Self> {
                unimplemented!()
            }

            fn to_usize(self) -> usize {
                unimplemented!()
            }
        }

        impl Add<usize> for VirtAddr {
            type Output = Self;

            fn add(self, _rhs: usize) -> Self::Output {
                unimplemented!()
            }
        }

        /// The location where physical memory is mapped.
        pub static PHYSICAL_MEMORY_MAP: PhysicalMemoryMap<VirtAddr> =
            PhysicalMemoryMap::new(VirtAddr(0xffff_8000_0000_0000));
    }
}
#[cfg(target_arch = "aarch64")]
pub use aarch64::*;
