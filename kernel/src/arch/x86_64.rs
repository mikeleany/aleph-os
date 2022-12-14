//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Functionality specific to the `x86_64` architecture.

use core::sync::atomic::{AtomicBool, Ordering};

use x86_64::{
    structures::{
        idt::InterruptDescriptorTable,
        paging::{PageSize, Size4KiB},
        DescriptorTablePointer,
    },
    PhysAddr, VirtAddr,
};

use interrupt::IntVec;

use crate::{
    arch::mem::{PageMapping, PHYSICAL_MEMORY_MAP_MAX_SIZE},
    bootboot::{BOOTBOOT, IDENTITY_MAP_MAX},
    mem::{Pager, PhysicalAddress},
};

pub mod mem;

/// Performs initialization required for `x86_64`.
pub fn init() {
    static INITIALIZED: AtomicBool = AtomicBool::new(false);
    static mut IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

    if INITIALIZED.swap(true, Ordering::Acquire) {
        return;
    }

    let double_fault =
        VirtAddr::from_ptr(interrupt::trampoline::<{ IntVec::DOUBLE_FAULT.0 }> as *const ());
    let segment_not_present =
        VirtAddr::from_ptr(interrupt::trampoline::<{ IntVec::SEGMENT_NOT_PRESENT.0 }> as *const ());

    // SAFETY: `trampoline` can handle interrupts with or without error codes
    //         `trampoline<8>` does not return
    //         access to `IDT` is synchronized with `INITIALIZED`
    unsafe { IDT.double_fault.set_handler_addr(double_fault) };
    // SAFETY: `trampoline` can handle interrupts with or without error codes
    //         access to `IDT` is synchronized with `INITIALIZED`
    unsafe {
        IDT.segment_not_present
            .set_handler_addr(segment_not_present)
    };

    let idt_ptr = DescriptorTablePointer {
        limit: (core::mem::size_of::<InterruptDescriptorTable>() - 1)
            .try_into()
            .unwrap(),
        base: VirtAddr::from_ptr(
            // SAFETY: access to `IDT` is synchronized with `INITIALIZED`
            unsafe { &IDT } as *const _,
        ),
    };

    // SAFETY: `idt_ptr` is a valid pointer to `IDT`
    unsafe { x86_64::instructions::tables::lidt(&idt_ptr) };

    let mmap_ent = BOOTBOOT.memory_map().last().unwrap();
    let mem_size = mmap_ent
        .address()
        .saturating_add(mmap_ent.size())
        .try_into()
        .unwrap();
    let mem_size = usize::min(mem_size, PHYSICAL_MEMORY_MAP_MAX_SIZE);
    let id_map_size = usize::min(mem_size, IDENTITY_MAP_MAX);

    let mut free_frames = BOOTBOOT
        .free_frames::<{ Size4KiB::SIZE }>()
        .map(|addr| PhysAddr::from_usize(addr.try_into().unwrap()).unwrap());

    let null_frame = free_frames.next();

    // SAFETY:
    // - `PHYSICAL_ADDRESS_MAP.base()` is reserved for the physical memory map for up to
    //   `PHYSICAL_MEMORY_MAP_MAX_SIZE` bytes.
    // - `mem_size` is no larger than `PHYSICAL_MEMORY_MAP_MAX_SIZE`.
    // - BOOTBOOT identity maps the entire physical memory space up to `IDENTITY_MAP_MAX` bytes,
    //   and `id_map_size` is the smaller of `mem_size` and `IDENTITY_MAP_MAX`.
    // - The frames in `free_frames` are available for use.
    // - Only those frames remaining after this call will be pushed to the frame allocater.
    // - The frame at address 0 has been removed from `free_frames`.
    unsafe {
        PageMapping::map_physical_mem(mem_size, id_map_size, &mut free_frames).unwrap();
    }
}

pub mod interrupt {
    //! Interrupt handlers.

    use x86_64::structures::idt::{DescriptorTable, SelectorErrorCode};

    #[cfg(doc)]
    use x86_64::structures::idt::InterruptDescriptorTable;

    /// An interrupt vector.
    ///
    /// Vectors `0..32` are reserved for system exceptions. All others are available for use as
    /// user interrupts.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
    #[repr(transparent)]
    pub struct IntVec(pub u8);

    impl IntVec {
        /// Divide-by-zero-error exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::divide_error`] for details.
        pub const DIVIDE_BY_ZERO_ERROR: Self = Self(0);

        /// Debug exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::debug`] for details.
        pub const DEBUG: Self = Self(1);

        /// Non-maskable interrupt.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::non_maskable_interrupt`] for details.
        pub const NON_MASKABLE_INTERRUPT: Self = Self(2);

        /// Breakpoint exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::breakpoint`] for details.
        pub const BREAKPOINT: Self = Self(3);

        /// Overflow exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::overflow`] for details.
        pub const OVERFLOW: Self = Self(4);

        /// Boundr-range exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::bound_range_exceeded`] for details.
        pub const BOUND_RANGE: Self = Self(5);

        /// Invalid-opcode exception
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::invalid_opcode`] for details.
        pub const INVALID_OPCODE: Self = Self(6);

        /// Device-not-available exeption.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::device_not_available`] for details.
        pub const DEVICE_NOT_AVAILABLE: Self = Self(7);

        /// Double-fault exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::double_fault`] for details.
        pub const DOUBLE_FAULT: Self = Self(8);

        /// Invalid-TSS exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::invalid_tss`] for details.
        pub const INVALID_TSS: Self = Self(10);

        /// Segment-not-present exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::segment_not_present`] for details.
        pub const SEGMENT_NOT_PRESENT: Self = Self(11);

        /// Stack exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::stack_segment_fault`] for details.
        pub const STACK: Self = Self(12);

        /// General-protection exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::general_protection_fault`] for details.
        pub const GENERAL_PROTECTION: Self = Self(13);

        /// Page-fault exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::page_fault`] for details.
        pub const PAGE_FAULT: Self = Self(14);

        /// x87 floating-point exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::x87_floating_point`] for details.
        pub const X87_FLOATING_POINT: Self = Self(16);

        /// Alignment-check exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::alignment_check`] for details.
        pub const ALIGNMENT_CHECK: Self = Self(17);

        /// Machine-check exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::machine_check`] for details.
        pub const MACHINE_CHECK: Self = Self(18);

        /// SIMD floating-point exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::simd_floating_point`] for details.
        pub const SIMD_FLOATING_POINT: Self = Self(19);

        /// Control-protection exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::divide_error`] for details.
        pub const CONTROL_PROTECTION: Self = Self(21);

        /// Hypervisor-injection exception.
        pub const HYPERVISOR_INJECTION: Self = Self(28);

        /// VMM-communication exception.
        pub const VMM_COMMUNICATION: Self = Self(29);

        /// Security exception.
        ///
        /// See [`x86_64`]'s [`InterruptDescriptorTable::security_exception`] for details.
        pub const SECURITY: Self = Self(30);

        /// Returns true if the interrupt vector is in the range (`0..32`) reserved for exceptions
        /// (even if the vector isn't currently used).
        pub fn is_exception(self) -> bool {
            self.0 < 32
        }

        /// Returns true if the interrupt vector is in the range (`32..=255`) available for user
        /// interrupts.
        pub fn is_user_interrupt(self) -> bool {
            self.0 >= 32
        }
    }

    /// Interrupt handler trampoline.
    ///
    /// # Safety
    /// This function is not safe to call directly, but it can be used as an x86_64 interrupt
    /// handler, whether or not the interrupt has an error code. If no error code is passed by the
    /// CPU, then `0` is pushed as the error code.
    #[naked]
    pub unsafe extern "C" fn trampoline<const VEC: u8>() {
        // SAFETY: see comments below
        unsafe {
            core::arch::asm!(
                // push error code if not present, which ensures a consistent stack layout
                "bt rsp, 3",
                "jnc 1f",
                "push 0",

                // preserves necessary registers for C calling convention
                "1:",
                "push rdi",
                "push rsi",
                "push rdx",
                "push rcx",
                "push rax",
                "push r8",
                "push r9",
                "push r10",
                "push r11",
                "cld",

                // SAFETY: this points to the interrupt stack frame
                // CAUTION: modifying the stack layout may invalidate this pointer
                "lea rdi, [rsp+0x50]",
                "mov rsi, {vec}",
                // SAFETY: this points to the error code
                // CAUTION: modifying the stack layout may invalidate this pointer
                "mov rdx, [rsp+0x48]",

                // SAFETY: `handler` uses the C calling convention so any of the callee-saved
                //         registers are preserved by `handler`. Caller-saved registers have been
                //         saved and are restored below
                "call {handler}",

                // restore registers previously preserved
                "pop r11",
                "pop r10",
                "pop r9",
                "pop r8",
                "pop rax",
                "pop rcx",
                "pop rdx",
                "pop rsi",
                "pop rdi",
                // remove error code
                "add rsp, 8",

                // SAFETY: rsp now points to the interrupt stack frame, without the error code
                // CAUTION: when making changes to the stack, care must be taken to ensure
                //          the safety statement above remains true
                "iretq",

                vec = const VEC,
                handler = sym handler,
                options(noreturn),
            );
        }
    }

    unsafe extern "C" fn handler(stack_frame: &[usize; 5], vec: IntVec, error_code: u64) {
        let stack_frame_ptr = stack_frame as *const _;
        log::info!("stack_frame_ptr = {stack_frame_ptr:?}");
        log::info!("stack_frame = {stack_frame:x?}");
        log::info!("vec = {vec:?}");
        log::info!("error_code = {error_code:x}");

        match vec {
            IntVec::SEGMENT_NOT_PRESENT => {
                let err = SelectorErrorCode::new_truncate(error_code);
                match err.descriptor_table() {
                    DescriptorTable::Idt => {
                        panic!("handler not present: interrupt vector {}", err.index() / 2)
                    }
                    _ => panic!("segment not present: {err:?}"),
                }
            }
            vec => unimplemented!("handler for interrupt vector {vec:?}"),
        }
    }
}
