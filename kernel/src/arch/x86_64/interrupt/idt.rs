//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Provides the [`InterruptDescriptorTable`], a system table which describes how the CPU should
//! handle each interrupt.
use super::super::{common::DescriptorTablePtr, segment::Selector};
use super::{Handler, Interrupt, Vector};
use core::num::NonZeroU8;
use crossbeam_utils::atomic::AtomicCell;

/// A system table used to provide [`Handler`]s for the various interrupts that can occur.
#[derive(Debug)]
#[repr(C, align(16))]
pub struct InterruptDescriptorTable {
    table: [AtomicCell<Option<GateDescriptor>>; Self::LENGTH],
}

impl InterruptDescriptorTable {
    const LENGTH: usize = 256;

    /// Creates a new interrupt descriptor table with all handlers set to `None`.
    pub const fn new() -> Self {
        // needed because AtomicCell is not `Copy`, and other ways of contructing an array of them
        // are not `const`
        #[allow(clippy::declare_interior_mutable_const)]
        const NONE: AtomicCell<Option<GateDescriptor>> = AtomicCell::new(None);

        InterruptDescriptorTable {
            table: [NONE; Self::LENGTH],
        }
    }

    /// Adds the [`Handler`] for [`Interrupt<V>`] to the table.
    pub fn handler<const V: Vector>(&self)
    where
        Interrupt<V>: Handler,
    {
        let addr = Interrupt::<V>::handler as usize;
        let addr_lower: u16 = (addr & 0xffff).try_into().expect("masking lower 16 bits");
        let addr_middle: u16 = ((addr >> 16) & 0xffff)
            .try_into()
            .expect("shifting to and masking lower 16 bits");
        let addr_upper: u32 = (addr >> 32).try_into().expect("shifting to lower 32 bits");

        self.table[usize::from(V)].store(Some(GateDescriptor {
            addr_lower,
            addr_middle,
            addr_upper,

            selector: Selector::cs(),
            ist_index: None,
            attr: Attributes::INTERRUPT_GATE,

            reserved: 0,
        }));
    }

    /// Loads the table into the CPU's interrupt descriptor table register (IDTR).
    ///
    /// # Safety
    /// The table and the interrupt handlers it points to must remain present at the same locations
    /// in virtual memory unless and until another table is loaded. If an exception or interrupt
    /// occurs and there is not a valid IDT at the loaded address, the result is undefined behavior.
    ///
    /// While not undefined behavior, care should also be taken to ensure that a double or triple
    /// fault does not occur due to missing exception gates.
    ///
    /// Care should also be taken to ensure that deadlock cannot occur between an interrupt handler
    /// and the interrupted process.
    pub unsafe fn load(&'static self) {
        // SAFETY: caller must meet the safety criteria
        unsafe {
            DescriptorTablePtr::new(self).load();
        }
    }
}

/// A descriptor for an interrupt or trap gate.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, align(16))]
pub struct GateDescriptor {
    /// Bits 0-15 of the interrupt handler address
    addr_lower: u16,
    /// Code segment selector
    selector: Option<Selector>,
    /// Index into the interrupt stack table (IST)
    ist_index: Option<NonZeroU8>,
    /// Descriptor attributes
    attr: Attributes,
    /// Bits 16-31 of the interrupt handler address
    addr_middle: u16,
    /// Bits 32-63 of the interrupt handler address
    addr_upper: u32,
    /// Reserved, must be zero
    reserved: u32,
}

/// Gate descriptor attributes.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(transparent)]
pub struct Attributes(NonZeroU8);

impl Attributes {
    /// Descriptor attributes for interrupt gate, where interrupts are automatically disabled.
    ///
    /// Present bit: 1
    /// DPL: 0
    /// Type: Interrupt gate (0xe)
    pub const INTERRUPT_GATE: Attributes = Attributes(NonZeroU8::new(0x8e).expect("0x8e != 0"));
}
