//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Functionality specific to the `x86_64` architecture.

pub mod interrupt;
pub use interrupt::init;

/// Provides functions and structures for segmentation.
///
/// In 64-bit long mode, segmentation must be set up even though it's not really used.
pub mod segment {
    use core::num::NonZeroU16;

    /// An index into a descriptor table.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct Selector(NonZeroU16);

    impl Selector {
        /// Returns the value of the `cs` register.
        pub fn cs() -> Option<Self> {
            let selector;

            // SAFETY: no undefined behavior can occur from copying the `cs` register
            unsafe {
                core::arch::asm!(
                    "mov {selector:x}, cs",
                    selector = out(reg) selector,
                );

                Some(Selector(NonZeroU16::new(selector)?))
            }
        }
    }
}

mod common {
    use super::interrupt::idt::InterruptDescriptorTable;

    /// A pointer to a descriptor table as used by the `lgdt` and `lidt` instructions.
    #[repr(C, packed)]
    pub struct DescriptorTablePtr<T> {
        limit: u16,
        ptr: *const T,
    }

    impl<T> DescriptorTablePtr<T> {
        pub fn new(table: &'static T) -> Self {
            DescriptorTablePtr {
                ptr: table as *const T,
                limit: core::mem::size_of::<T>()
                    .try_into()
                    .expect("size of table must fit in 16 bits"),
            }
        }
    }

    impl DescriptorTablePtr<InterruptDescriptorTable> {
        /// Loads the table into the CPU's interrupt descriptor table register (IDTR).
        ///
        /// # Safety
        /// The table and the interrupt handlers it points to must remain present at the same
        /// locations in virtual memory unless and until another table is loaded. If an exception or
        /// interrupt occurs and there is not a valid IDT at the loaded address, the result is
        /// undefined behavior.
        ///
        /// While not undefined behavior, care should also be taken to ensure that a double or
        /// triple fault does not occur due to missing exception gates.
        ///
        /// Care should also be taken to ensure that deadlock cannot occur between an interrupt
        /// handler and the interrupted process.
        pub unsafe fn load(&self) {
            // SAFETY: the caller gaurantees that the table and handlers will remain in memory at
            // least unil they're no longer needed.
            // The implementor of the `Handler`s guarantees that all other safety requirements are
            // met.
            unsafe {
                core::arch::asm!("lidt [{}]", in(reg) self, options(nostack));
            }
        }
    }
}
