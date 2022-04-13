//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Exceptions and handlers.
//!
//! The first 32 [`Interrupt`] [`Vector`][super::Vector]s (0 through 31) are reserved for
//! pre-defined interrupts called exceptions. Most of these are caused by errors of some kind,
//! though some are not.
//!
//! # Kinds of Exceptions
//!
//! There are four general kinds, or categories, of exceptions based on the cause of the exception
//! and what the return pointer points to.
//!
//! - An **interrupt** exception is an exception caused by an external interrupt, rather than by
//! the current instruction. The return pointer will point to the instruction after the instruction
//! where the interrupt occurred.
//!
//! - A **trap** exception is an exception where the return pointer points to the instruction after
//! the instruction that caused the exception.
//!
//! - A **fault** exception is an exception where the return pointer points to the instruction that
//! caused the exception.
//!
//! - An **abort** exception is an exception where it is generally impossible to resume execution
//! of the process in which the excepiton occurred. The value of the return pointer is generally
//! undefined, and returning from an abort is undefined behavior, unless the return pointer is
//! first changed to a known good value.
//!
//! # Contributory and Benign Exceptions
//!
//! An exception can also be either **contributory** or **benign**. When a **contributory**
//! exception happens while attempting to call the handler for a prior **contributory** exception,
//! a [`DoubleFault`] exception occurs. For example, if a [`DivideByZeroException`] occurs, but in
//! an attempt to call its exception handler, the CPU encounters a [`SegmentNotPresentException`],
//! this will cause a `DoubleFault`. **Benign** exceptions do not contribute to a `DoubleFault`.
//!
//! If a third exception occurs -- whether **contributory** or **benign** -- while attempting to
//! call the `DoubleFault` exception handler, the processor shuts down -- generally resulting in a
//! system reboot. This is commonly referred to as a [triple fault].
//!
//! [triple fault]: https://en.wikipedia.org/wiki/Triple_fault
//!
//! # Exception Classification
//!
//! | Vector | Mnemonic | Exception                         | Kind          | Contributory |
//! |--------|----------|-----------------------------------|---------------|--------------|
//! |  0     | #DE      | [`DivideByZeroException`]         | Fault         | Yes          |
//! |  1     | #DB      | [`DebugException`]                | Fault or Trap | No           |
//! |  2     | NMI      | [`NonMaskableInterrupt`]          | Interrupt     | No           |
//! |  3     | #BP      | [`BreakpointException`]           | Trap          | No           |
//! |  4     | #OF      | [`OverflowException`]             | Trap          | No           |
//! |  5     | #BR      | [`BoundRangeException`]           | Fault         | No           |
//! |  6     | #UD      | [`InvalidOpcodeException`]        | Fault         | No           |
//! |  7     | #NM      | [`DeviceNotAvailableException`]   | Fault         | No           |
//! |  8     | #DF      | [`DoubleFault`]                   | **Abort**     | --           |
//! |  9     | --       | --- Reserved ---                  | --            | --           |
//! | 10     | #TS      | [`InvalidTssException`]           | Fault         | Yes          |
//! | 11     | #NP      | [`SegmentNotPresentException`]    | Fault         | Yes          |
//! | 12     | #SS      | [`StackException`]                | Fault         | Yes          |
//! | 13     | #GP      | [`GeneralProtectionException`]    | Fault         | Yes          |
//! | 14     | #PF      | [`PageFault`]                     | Fault         | Partially    |
//! | 15     | --       | --- Reserved ---                  | --            | --           |
//! | 16     | #MF      | [`X87FloatingPointException`]     | Fault         | No           |
//! | 17     | #AC      | [`AlignmentCheckException`]       | Fault         | No           |
//! | 18     | #MC      | [`MachineCheckException`]         | **Abort**     | No           |
//! | 19     | #XF      | [`SimdFloatingPointException`]    | Fault         | No           |
//! | 20     | --       | --- Reserved ---                  | --            | --           |
//! | 21     | #CP      | [`ControlProtectionException`]    | Fault         | Yes          |
//! | 22-27  | --       | --- Reserved ---                  | --            | --           |
//! | 28     | #HV      | [`HypervisorInjection`]           | *uncertain*   | No           |
//! | 29     | #VC      | [`VmmCommunication`]              | Fault         | Yes          |
//! | 30     | #SX      | [`SecurityException`]             | Fault         | Yes          |
//! | 31     | --       | --- Reserved ---                  | --            | --           |
use super::{Handler, Interrupt, Param, StackFrame};

/// #DE -- the divide-by-zero-error exception.
///
/// Occurs if the denominator of a division is zero, or if the result is too large to fit in the
/// destination.
///
/// This exception is a [contributory] [fault].
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#DE")]
pub type DivideByZeroException = Interrupt<0>;

// SAFETY: divide-by-zero exceptions do not pass an error code and are not abort exceptions, so the
// default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for DivideByZeroException {}

/// #DB -- the debug exception.
///
/// Debug information is stored in the debug-status register, `dr6`.
///
/// This exception is a [benign] [fault] or [trap], depending on the cause of the exception.
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
/// [trap]: index.html#kinds-of-exceptions
#[doc(alias = "#DB")]
pub type DebugException = Interrupt<1>;

// SAFETY: debug exceptions do not pass an error code and are not abort exceptions, so the default
// `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for DebugException {}

/// NMI -- the non-maskable-interrupt exception.
///
/// `NonMaskableInterrupt` exceptions cannot be masked. However, when an NMI exception occurs
/// the processor won't recognize any further NMI exceptions until an `iret` istruction is
/// executed.
///
/// This exception is a [benign] [interrupt].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [interrupt]: index.html#kinds-of-exceptions
#[doc(alias = "NMI")]
pub type NonMaskableInterrupt = Interrupt<2>;

// SAFETY: non-maskable-interrupt exceptions do not pass an error code and are not abort exceptions,
// so the default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for NonMaskableInterrupt {}

/// #BP -- the breakpoint exception.
///
/// Occurs when the `int3` instruction is executed.
///
/// This exception is a [benign] [trap].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [trap]: index.html#kinds-of-exceptions
#[doc(alias = "#BP")]
pub type BreakpointException = Interrupt<3>;

// SAFETY: breakpoint exceptions do not pass an error code and are not abort exceptions, so the
// default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for BreakpointException {}

/// #OF -- the overflow exception.
///
/// Occurs if the `into` instruction is executed when the overflow flag in the `rflags` register is
/// set.
///
/// This exception is a [benign] [trap].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [trap]: index.html#kinds-of-exceptions
#[doc(alias = "#OF")]
pub type OverflowException = Interrupt<4>;

// SAFETY: overflow exceptions do not pass an error code and are not abort exceptions, so the
// default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for OverflowException {}

/// #BR -- the bound-range exception.
///
/// Occurs when the `bound` instruction determines that an array index is outside the bounds of the
/// array.
///
/// This exception is a [benign] [fault].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#BR")]
pub type BoundRangeException = Interrupt<5>;

// SAFETY: bound-range exceptions do not pass an error code and are not abort exceptions, so the
// default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for BoundRangeException {}

/// #UD -- the invalid-opcode exception.
///
/// Can occur due to the execution of any opcode that is reserved or undefined in the current mode
/// or due to the execution of the `ud0`, `ud1`, or `ud2` instruction.
///
/// This exception is a [benign] [fault].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#UD")]
pub type InvalidOpcodeException = Interrupt<6>;

// SAFETY: invalid-opcode exceptions do not pass an error code and are not abort exceptions, so the
// default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for InvalidOpcodeException {}

/// #NM -- the device-not-available exception.
///
/// Occurs if an attempt is made to access certain processor features when they are not available.
///
/// This exception is a [benign] [fault].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#NM")]
pub type DeviceNotAvailableException = Interrupt<7>;

// SAFETY: device-not-available exceptions do not pass an error code and are not abort exceptions,
// so the default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for DeviceNotAvailableException {}

/// #DF -- the double-fault exception.
///
/// A `DoubleFault` exception occurs when a [contributory] exception happens during the handling
/// of a prior contributory exception. For example, if a divide-by-zero exception occurs, but
/// in an attempt to call its exception handler, the CPU encounters a segment-not-present
/// exception, this will cause a double fault. If a third exception (even an otherwise [benign]
/// exception) occurs while attempting to call the double-fault-exception handler, the
/// processor shuts down -- generally resulting in a system reboot. This is commonly referred
/// to as a triple fault.
///
/// This exception receives an error code containing an `Option`-wrapped [`SelectorErrorCode`],
/// which should always be `None`.
///
/// # Safety
/// This exception is an [abort]. The value of the return pointer is undefined, and executing an
/// `iret` instruction without first setting the return pointer to a known-good value will cause
/// undefined behavior.
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [benign]: index.html#contributory-and-benign-exceptions
/// [abort]: index.html#kinds-of-exceptions
#[doc(alias = "#DF")]
pub type DoubleFault = Interrupt<8>;

// SAFETY: double-fault exceptions pass an error code, and the handler accepts a 64-bit aligned
// 32-bit error code.
// This is an abort exception, so it returns `!` (never).
unsafe impl Param for DoubleFault {
    type ErrorCode = Aligned<Option<SelectorErrorCode>>;
    type Return = !;
}

impl Handler for DoubleFault {
    unsafe extern "x86-interrupt" fn handler(_: StackFrame<Self::ErrorCode>) -> ! {
        log::error!("\t\t***DOUBLE FAULT EXCEPTION***");
        panic!("double fault");
    }
}

/// #TS -- the invalid-TSS exception.
///
/// Occurs if a control transfer through a gate descriptor where the descriptor references an
/// invalid stack segment selector.
///
/// This exception receives an error code containing an `Option`-wrapped [`SelectorErrorCode`],
/// which should give the index of the stack segment descriptor that caused the exception.
///
/// This exception is a [contributory] [fault].
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#TS")]
pub type InvalidTssException = Interrupt<10>;

// SAFETY: invalid-TSS exceptions pass an error code, and the handler accepts a 64-bit aligned
// 32-bit error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for InvalidTssException {
    type ErrorCode = Aligned<Option<SelectorErrorCode>>;
}

/// #NP -- the segment-not-present exception.
///
/// Occurs if an attempt is made to load a segment or gate descriptor when the present bit is not
/// set, except when loading the stack segment selector (`ss` register), in which case a
/// [`StackException`] occurs instead.
///
/// This exception receives an error code containing an `Option`-wrapped [`SelectorErrorCode`],
/// which should give the index of the segment descriptor that caused the exception.
///
/// This exception is a [contributory] [fault].
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#NP")]
pub type SegmentNotPresentException = Interrupt<11>;

// SAFETY: segment-not-present exceptions pass an error code, and the handler accepts a 64-bit
// aligned 32-bit error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for SegmentNotPresentException {
    type ErrorCode = Aligned<Option<SelectorErrorCode>>;
}

/// #SS -- the stack exception.
///
/// Occurs if an attempt is made to access memory using the `rsp` or `rbp` register (including with
/// the `push` and `pop` instructions) using an address that is not in canonical form or if an
/// attempt is made to load the `ss` register with a reference to a segment descriptor when the
/// present bit is not set.
///
/// This exception receives an error code containing an `Option`-wrapped [`SelectorErrorCode`],
/// which should give the index of the segment descriptor that caused the exception, if applicable.
///
/// This exception is a [contributory] [fault].
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#SS")]
pub type StackException = Interrupt<12>;

// SAFETY: stack exceptions pass an error code, and the handler accepts a 64-bit aligned 32-bit
// error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for StackException {
    type ErrorCode = Aligned<Option<SelectorErrorCode>>;
}

/// #GP -- the general-protection exception.
///
/// Can be caused by a large number of things. Consult the *AMD64 Architecture Programmer’s Manual,
/// Volume 2: System Programming* for the details.
///
/// This exception receives an error code containing an `Option`-wrapped [`SelectorErrorCode`],
/// which should give the index of the segment descriptor that caused the exception, if applicable.
///
/// This exception is a [contributory] [fault].
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#GP")]
pub type GeneralProtectionException = Interrupt<13>;

// SAFETY: general-protection exceptions pass an error code, and the handler accepts a 64-bit
// aligned 32-bit error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for GeneralProtectionException {
    type ErrorCode = Aligned<Option<SelectorErrorCode>>;
}

/// #PF -- the page-fault exception.
///
/// Occurs if an attempt is made to access a page that cannot be accessed.
///
/// This exception receives an error code of type [`PageFaultErrorCode`]. In addition, the virtual
/// address that caused the page fault is stored in the `cr2` register.
///
/// This exception is a partially-[contributory] [fault]. It is always contributory if it is the
/// first exception. However, it is only contributory as a second exception if the first exception
/// was also a `PageFault`.
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#PF")]
pub type PageFault = Interrupt<14>;

// SAFETY: page-fault exceptions pass an error code, and the handler accepts a 64-bit aligned 32-bit
// error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for PageFault {
    type ErrorCode = Aligned<PageFaultErrorCode>;
}

/// #MF -- the x87-floating-point exception.
///
/// Exception information is stored in the x87 status-word register.
///
/// This exception is a [benign], imprecise [fault].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#MF")]
pub type X87FloatingPointException = Interrupt<15>;

// SAFETY: x87-floating-point exceptions do not pass an error code and are not abort exceptions, so
// the default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for X87FloatingPointException {}

/// #AC -- the alignment-check exception.
///
/// Occurs when an unaligned memory access is performed and alignment checking is enabled.
///
/// This exception receives an error code containing an `Option`-wrapped [`SelectorErrorCode`],
/// which should always be `None`.
///
/// This exception is a [benign] [fault].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#AC")]
pub type AlignmentCheckException = Interrupt<17>;

// SAFETY: machine-check exceptions pass an error code, and the handler accepts a 64-bit aligned
// 32-bit error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for AlignmentCheckException {
    type ErrorCode = Aligned<Option<SelectorErrorCode>>;
}

/// #MC -- the machine-check exception.
///
/// The `MachineCheckException` is model specific.
///
/// Error information is stored in model-specific registers (MSRs).
///
/// # Safety
/// This exception is a [benign] [abort]. The value of the return pointer is generally undefined,
/// and executing an `iret` instruction without first setting the return pointer to a known-good
/// value will cause undefined behavior.
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [abort]: index.html#kinds-of-exceptions
#[doc(alias = "#MC")]
pub type MachineCheckException = Interrupt<18>;

// SAFETY: machine-check exceptions do not pass an error code, so the default `ErrorCode` can and
// should be used.
// This is an abort exception, so it returns `!` (never).
unsafe impl Param for MachineCheckException {
    type Return = !;
}

/// #XF -- the SIMD-floating-point exception.
///
/// Exception information is stored in the SSE floating-point `mxcsr` register.
///
/// This exception is a [benign] [fault].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#XF")]
pub type SimdFloatingPointException = Interrupt<19>;

// SAFETY: SIMD-floating-point exceptions do not pass an error code and are not abort exceptions, so
// the default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for SimdFloatingPointException {}

/// #CP -- the control-protection exception.
///
/// Can only occur while shadow stacks are enabled.
///
/// This exception receives an error code of type [`ControlProtectionErrorCode`].
///
/// This exception is a [benign] [fault].
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#CP")]
pub type ControlProtectionException = Interrupt<21>;

// SAFETY: control-protection exceptions pass an error code, and the handler accepts a 64-bit
// aligned 32-bit error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for ControlProtectionException {
    type ErrorCode = Aligned<ControlProtectionErrorCode>;
}

/// #HV -- the hypervisor-injection exception.
///
/// May be injected by a hypervisor into a secure guest VM.
///
/// This exception is a [benign]. Whether this is a [fault], [trap] or [interrupt]-type exception is
/// uncertain.
///
/// [benign]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
/// [trap]: index.html#kinds-of-exceptions
/// [interrupt]: index.html#kinds-of-exceptions
#[doc(alias = "#HV")]
pub type HypervisorInjection = Interrupt<28>;

// SAFETY: hypervisor-injection exceptions do not pass an error code and are not abort exceptions,
// so the default `ErrorCode` and `Return` types can and should be used.
unsafe impl Param for HypervisorInjection {}

/// #VC -- the VMM-communication exception.
///
/// Can occur when certain events occur inside a secure guest VM.
///
/// This exception receives an error code of type [`VmExitCode`].
///
/// This exception is a [contributory] [fault].
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#VC")]
pub type VmmCommunication = Interrupt<29>;

// SAFETY: vmm-communication exceptions pass an error code, and the handler accepts a 64-bit aligned
// 32-bit error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for VmmCommunication {
    type ErrorCode = Aligned<VmExitCode>;
}

/// #SX -- the security exception.
///
/// Can be generated by security-sensitive events under SVM.
///
/// This exception receives an error code of type [`SecurityErrorCode`].
///
/// This exception is a [contributory] [fault].
///
/// [contributory]: index.html#contributory-and-benign-exceptions
/// [fault]: index.html#kinds-of-exceptions
#[doc(alias = "#SX")]
pub type SecurityException = Interrupt<30>;

// SAFETY: security exceptions pass an error code, and the handler accepts a 64-bit aligned 32-bit
// error code.
// This is not an abort exception, so the default `Return` type can be used.
unsafe impl Param for SecurityException {
    type ErrorCode = Aligned<SecurityErrorCode>;
}

mod error_codes {
    use core::num::NonZeroU32;
    use core::ops::Deref;

    #[cfg(doc)]
    use super::{PageFault, ControlProtectionException, VmmCommunication, SecurityException};

    /// Aligns a type to 64 bits, to make the type safe to use for an interrupt error code.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(C, align(8))]
    pub struct Aligned<T>(T);

    impl<T> Deref for Aligned<T> {
        type Target = T;

        fn deref(&self) -> &Self::Target {
            &self.0
        }
    }

    /// An error code that gives an index into a descriptor table (GDT, LDT or IDT), to indicate the
    /// descriptor that caused an exception.
    ///
    /// To be FFI-safe, the `SelectorErrorCode` should always be wrapped in an `Option`. Some
    /// exceptions will pass `None` as the error code.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct SelectorErrorCode(NonZeroU32);

    /// An error code that gives information about the kind of memory access that caused a
    /// [`PageFault`].
    ///
    /// The virtual address that caused the `PageFault` is stored in the `cr2` register.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct PageFaultErrorCode(u32);

    /// An error code that indicates the cause of a [`ControlProtectionException`].
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct ControlProtectionErrorCode(u32);

    /// The error code for a [`VmmCommunication`] exception.
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct VmExitCode(u32);

    /// An error code that indicates the cause of a [`SecurityException`].
    #[derive(Debug, Copy, Clone, Eq, PartialEq)]
    #[repr(transparent)]
    pub struct SecurityErrorCode(u32);
}
pub use error_codes::*;
