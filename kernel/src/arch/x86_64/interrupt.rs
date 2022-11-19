//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Interrupts, interrupt handlers, and the interrupt descriptor table (IDT).
use self::exception::{
    ControlProtectionErrorCode, PageFaultErrorCode, SecurityErrorCode, SelectorErrorCode,
    VmExitCode,
};

use super::segment::Selector;
use core::cell::UnsafeCell;

pub mod exception;
pub mod idt;
use idt::InterruptDescriptorTable;

/// Performs initialization necessary for handling interrupts and exceptions.
pub fn init() -> &'static InterruptDescriptorTable {
    static IDT: InterruptDescriptorTable = InterruptDescriptorTable::new();

    IDT.handler::<8>();

    // SAFETY: the table and interrupt handlers will remain in memory until system restart
    unsafe {
        IDT.load();
    }

    &IDT
}

pub trait IntReturn {}
impl IntReturn for () {}
impl IntReturn for ! {}
pub struct InterruptHandler<E = (), R: IntReturn = ()>(
    extern "x86-interrupt" fn(StackFrame<E>) -> R,
);

pub struct ExceptionTable {
    pub divide_by_zero: Option<InterruptHandler>,
    pub debug: Option<InterruptHandler>,
    pub non_maskable_interrupt: Option<InterruptHandler>,
    pub breakpoint: Option<InterruptHandler>,
    pub overflow: Option<InterruptHandler>,
    pub bound_range: Option<InterruptHandler>,
    pub invalid_opcode: Option<InterruptHandler>,
    pub device_not_available: Option<InterruptHandler>,
    pub double_fault: Option<InterruptHandler<Option<SelectorErrorCode>, !>>,
    pub invalid_tss: Option<InterruptHandler<Option<SelectorErrorCode>>>,
    pub segment_not_present: Option<InterruptHandler<Option<SelectorErrorCode>>>,
    pub stack: Option<InterruptHandler<Option<SelectorErrorCode>>>,
    pub general_protection: Option<InterruptHandler<Option<SelectorErrorCode>>>,
    pub page_fault: Option<InterruptHandler<PageFaultErrorCode>>,
    pub x87_floating_point: Option<InterruptHandler>,
    pub alignment_check: Option<InterruptHandler<Option<SelectorErrorCode>>>,
    pub machine_check: Option<InterruptHandler<(), !>>,
    pub simd_floating_point: Option<InterruptHandler>,
    pub control_protection: Option<InterruptHandler<ControlProtectionErrorCode>>,
    pub hypervisor_injection: Option<InterruptHandler>,
    pub vmm_communication: Option<InterruptHandler<VmExitCode>>,
    pub security: Option<InterruptHandler<SecurityErrorCode>>,
}

impl ExceptionTable {
    pub const fn new() -> Self {
        ExceptionTable {
            divide_by_zero: None,
            debug: None,
            non_maskable_interrupt: None,
            breakpoint: None,
            overflow: None,
            bound_range: None,
            invalid_opcode: None,
            device_not_available: None,
            double_fault: None,
            invalid_tss: None,
            segment_not_present: None,
            stack: None,
            general_protection: None,
            page_fault: None,
            x87_floating_point: None,
            alignment_check: None,
            machine_check: None,
            simd_floating_point: None,
            control_protection: None,
            hypervisor_injection: None,
            vmm_communication: None,
            security: None,
        }
    }
}

static EXCEPTION_HANDLERS: ExceptionTable = {
    let mut exceptions = ExceptionTable::new();

    extern "x86-interrupt" fn double_fault(_: StackFrame<Option<SelectorErrorCode>>) -> ! {
        log::error!("\t\t***DOUBLE FAULT EXCEPTION***");
        panic!("double fault");
    }
    exceptions.double_fault = Some(InterruptHandler(double_fault));

    exceptions
};

pub struct UserVector(Vector);

impl UserVector {
    pub fn try_from_vector(v: Vector) -> Result<Self, ()> {
        if v >= 32 {
            Ok(UserVector(v))
        } else {
            Err(())
        }
    }
}

pub struct UserInterruptTable([Option<InterruptHandler>; 256 - 32]);

impl UserInterruptTable {
    pub const fn get(&self, v: UserVector) -> Option<InterruptHandler> {
        todo!()
    }
}

/// An interrupt vector number, which can be any number from 0 through 255.
pub type Vector = u8;

/// A unique type for each interrupt [`Vector`].
///
/// These types cannot be instantiated, but are used to implement interrupt [`Handler`]s for
/// specific interrupt `Vector`s.
#[derive(Debug)]
pub enum Interrupt<const V: Vector> {}

impl<const V: Vector> Interrupt<V> {
    /// The [`Vector`] corresponding to this interrupt.
    pub const VECTOR: Vector = V;
}

struct Condition<const B: bool>;
trait True {}
impl True for Condition<true> {}
trait False {}
impl False for Condition<false> {}

/// Contains the pointers and flags necessary to return to an interrupted process.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
#[repr(C, align(8))]
pub struct ReturnPointers {
    /// The return instruction pointer.
    pub rip: *const u8,
    /// The return code segment selector.
    pub cs: Option<Selector>,
    /// The `rflags` register of the interrupted process.
    pub rflags: u64,
    /// The return stack pointer.
    pub rsp: *const u8,
    /// The return stack segment selector.
    pub ss: Option<Selector>,
}

/// The stack frame of an interrupt handler, including the [`ReturnPointers`] and [`ErrorCode`], if
/// applicable.
///
/// The error code must be either 8 bytes (with any padding) or 0 bytes, and must be FFI safe.
/// The default is `()`.
///
/// [`ErrorCode`]: Param::ErrorCode
#[derive(Debug)]
#[repr(C, align(8))]
pub struct StackFrame<E = ()> {
    /// The error code, if applicable.
    pub error_code: E,
    stack_frame: UnsafeCell<ReturnPointers>,
}

mod sealed {
    pub trait InterruptReturn {}
    impl InterruptReturn for () {}
    impl InterruptReturn for ! {}
}
use sealed::InterruptReturn;

/// Trait to indicate the [`ErrorCode`] and [`Return`] types that a [`Handler`] must have for an
/// [`Interrupt`].
///
/// While the `Return` type can be specified, this does not allow the handler to return a value, as
/// the type must be either `()` or `!`. This is only so that [abort] exceptions can be forced to
/// diverge.
///
/// # Safety
/// Most interrupts -- including all user interrupts, [`Vector`]s 32 through 255 -- must use the
/// default, zero-byte [`ErrorCode`]. Some [`exception`]s do require a 64-bit-aligned `ErrorCode`
/// that is no more than 64 bits in size (including padding). This `ErrorCode` must be FFI-safe,
/// and meet the requirements of the CPU documentation. This includes not depending on the value of
/// padding bits, as the value of these bits is undefined.
///
/// The [`Return`] type must be `!` for [abort] exceptions.
///
/// [`ErrorCode`]: Param::ErrorCode
/// [`Return`]: Param::Return
/// [abort]: exception/index.html#kinds-of-exceptions
pub unsafe trait Param {
    /// The error code for an interrupt handler, which defaults to a zero-sized type.
    ///
    /// This type must match the CPU documentation, using the default, zero-sized type when the CPU
    /// does not pass an error code.
    ///
    /// # Safety
    /// See the trait-level documentation for [safety] requirements.
    ///
    /// [safety]: #safety
    type ErrorCode = ();

    /// The return type for an interrupt handler, which can be either `()` or `!`.
    ///
    /// # Safety
    /// The `Return` type must be `!` for [abort] exceptions.
    ///
    /// [abort]: exception/index.html#kinds-of-exceptions
    type Return: InterruptReturn = ();
}

/// A trait for implementing [`Interrupt`] handlers.
pub trait Handler
where
    Self: Param,
{
    /// An interrupt handler.
    ///
    /// For most interrupts, [`Param::ErrorCode`] and [`Param::Return`] both have the `()` type.
    /// However, some exceptions have a non-zero-sized error code and/or `!` as the return type.
    ///
    /// # Safety
    /// This function must not ever be called directly, but must only be called by the CPU in
    /// response to the intended interrupt, and the intended interrupt must never be invoked in a
    /// way that is incompatible with the interrupt handler.
    ///
    /// As a more specific example, an interrupt which takes an error code must never be invoked in
    /// a way that does not push the required error code to the stack. Such incorrect invocation
    /// may include, but is not limited to, hardware interrupts using the same interrupt vector as
    /// an exception which requires an error code, or invoking such an interrupt manually with the
    /// `int` instruction, or giving untrusted code access to do so.
    unsafe extern "x86-interrupt" fn handler(
        stack_frame: StackFrame<Self::ErrorCode>,
    ) -> Self::Return;
}

/// User interrupts.
// SAFETY: user interrupts do not pass an error code and are not abort exceptions, so the default
// `ErrorCode` and `Return` types can and should be used. This safety comment applies to all
// `unsafe impl`s in this module.
#[allow(clippy::undocumented_unsafe_blocks)]
mod user {
    use super::{Interrupt, Param};

    unsafe impl Param for Interrupt<0x20> {}
    unsafe impl Param for Interrupt<0x21> {}
    unsafe impl Param for Interrupt<0x22> {}
    unsafe impl Param for Interrupt<0x23> {}
    unsafe impl Param for Interrupt<0x24> {}
    unsafe impl Param for Interrupt<0x25> {}
    unsafe impl Param for Interrupt<0x26> {}
    unsafe impl Param for Interrupt<0x27> {}
    unsafe impl Param for Interrupt<0x28> {}
    unsafe impl Param for Interrupt<0x29> {}
    unsafe impl Param for Interrupt<0x2a> {}
    unsafe impl Param for Interrupt<0x2b> {}
    unsafe impl Param for Interrupt<0x2c> {}
    unsafe impl Param for Interrupt<0x2d> {}
    unsafe impl Param for Interrupt<0x2e> {}
    unsafe impl Param for Interrupt<0x2f> {}
    unsafe impl Param for Interrupt<0x30> {}
    unsafe impl Param for Interrupt<0x31> {}
    unsafe impl Param for Interrupt<0x32> {}
    unsafe impl Param for Interrupt<0x33> {}
    unsafe impl Param for Interrupt<0x34> {}
    unsafe impl Param for Interrupt<0x35> {}
    unsafe impl Param for Interrupt<0x36> {}
    unsafe impl Param for Interrupt<0x37> {}
    unsafe impl Param for Interrupt<0x38> {}
    unsafe impl Param for Interrupt<0x39> {}
    unsafe impl Param for Interrupt<0x3a> {}
    unsafe impl Param for Interrupt<0x3b> {}
    unsafe impl Param for Interrupt<0x3c> {}
    unsafe impl Param for Interrupt<0x3d> {}
    unsafe impl Param for Interrupt<0x3e> {}
    unsafe impl Param for Interrupt<0x3f> {}
    unsafe impl Param for Interrupt<0x40> {}
    unsafe impl Param for Interrupt<0x41> {}
    unsafe impl Param for Interrupt<0x42> {}
    unsafe impl Param for Interrupt<0x43> {}
    unsafe impl Param for Interrupt<0x44> {}
    unsafe impl Param for Interrupt<0x45> {}
    unsafe impl Param for Interrupt<0x46> {}
    unsafe impl Param for Interrupt<0x47> {}
    unsafe impl Param for Interrupt<0x48> {}
    unsafe impl Param for Interrupt<0x49> {}
    unsafe impl Param for Interrupt<0x4a> {}
    unsafe impl Param for Interrupt<0x4b> {}
    unsafe impl Param for Interrupt<0x4c> {}
    unsafe impl Param for Interrupt<0x4d> {}
    unsafe impl Param for Interrupt<0x4e> {}
    unsafe impl Param for Interrupt<0x4f> {}
    unsafe impl Param for Interrupt<0x50> {}
    unsafe impl Param for Interrupt<0x51> {}
    unsafe impl Param for Interrupt<0x52> {}
    unsafe impl Param for Interrupt<0x53> {}
    unsafe impl Param for Interrupt<0x54> {}
    unsafe impl Param for Interrupt<0x55> {}
    unsafe impl Param for Interrupt<0x56> {}
    unsafe impl Param for Interrupt<0x57> {}
    unsafe impl Param for Interrupt<0x58> {}
    unsafe impl Param for Interrupt<0x59> {}
    unsafe impl Param for Interrupt<0x5a> {}
    unsafe impl Param for Interrupt<0x5b> {}
    unsafe impl Param for Interrupt<0x5c> {}
    unsafe impl Param for Interrupt<0x5d> {}
    unsafe impl Param for Interrupt<0x5e> {}
    unsafe impl Param for Interrupt<0x5f> {}
    unsafe impl Param for Interrupt<0x60> {}
    unsafe impl Param for Interrupt<0x61> {}
    unsafe impl Param for Interrupt<0x62> {}
    unsafe impl Param for Interrupt<0x63> {}
    unsafe impl Param for Interrupt<0x64> {}
    unsafe impl Param for Interrupt<0x65> {}
    unsafe impl Param for Interrupt<0x66> {}
    unsafe impl Param for Interrupt<0x67> {}
    unsafe impl Param for Interrupt<0x68> {}
    unsafe impl Param for Interrupt<0x69> {}
    unsafe impl Param for Interrupt<0x6a> {}
    unsafe impl Param for Interrupt<0x6b> {}
    unsafe impl Param for Interrupt<0x6c> {}
    unsafe impl Param for Interrupt<0x6d> {}
    unsafe impl Param for Interrupt<0x6e> {}
    unsafe impl Param for Interrupt<0x6f> {}
    unsafe impl Param for Interrupt<0x70> {}
    unsafe impl Param for Interrupt<0x71> {}
    unsafe impl Param for Interrupt<0x72> {}
    unsafe impl Param for Interrupt<0x73> {}
    unsafe impl Param for Interrupt<0x74> {}
    unsafe impl Param for Interrupt<0x75> {}
    unsafe impl Param for Interrupt<0x76> {}
    unsafe impl Param for Interrupt<0x77> {}
    unsafe impl Param for Interrupt<0x78> {}
    unsafe impl Param for Interrupt<0x79> {}
    unsafe impl Param for Interrupt<0x7a> {}
    unsafe impl Param for Interrupt<0x7b> {}
    unsafe impl Param for Interrupt<0x7c> {}
    unsafe impl Param for Interrupt<0x7d> {}
    unsafe impl Param for Interrupt<0x7e> {}
    unsafe impl Param for Interrupt<0x7f> {}
    unsafe impl Param for Interrupt<0x80> {}
    unsafe impl Param for Interrupt<0x81> {}
    unsafe impl Param for Interrupt<0x82> {}
    unsafe impl Param for Interrupt<0x83> {}
    unsafe impl Param for Interrupt<0x84> {}
    unsafe impl Param for Interrupt<0x85> {}
    unsafe impl Param for Interrupt<0x86> {}
    unsafe impl Param for Interrupt<0x87> {}
    unsafe impl Param for Interrupt<0x88> {}
    unsafe impl Param for Interrupt<0x89> {}
    unsafe impl Param for Interrupt<0x8a> {}
    unsafe impl Param for Interrupt<0x8b> {}
    unsafe impl Param for Interrupt<0x8c> {}
    unsafe impl Param for Interrupt<0x8d> {}
    unsafe impl Param for Interrupt<0x8e> {}
    unsafe impl Param for Interrupt<0x8f> {}
    unsafe impl Param for Interrupt<0x90> {}
    unsafe impl Param for Interrupt<0x91> {}
    unsafe impl Param for Interrupt<0x92> {}
    unsafe impl Param for Interrupt<0x93> {}
    unsafe impl Param for Interrupt<0x94> {}
    unsafe impl Param for Interrupt<0x95> {}
    unsafe impl Param for Interrupt<0x96> {}
    unsafe impl Param for Interrupt<0x97> {}
    unsafe impl Param for Interrupt<0x98> {}
    unsafe impl Param for Interrupt<0x99> {}
    unsafe impl Param for Interrupt<0x9a> {}
    unsafe impl Param for Interrupt<0x9b> {}
    unsafe impl Param for Interrupt<0x9c> {}
    unsafe impl Param for Interrupt<0x9d> {}
    unsafe impl Param for Interrupt<0x9e> {}
    unsafe impl Param for Interrupt<0x9f> {}
    unsafe impl Param for Interrupt<0xa0> {}
    unsafe impl Param for Interrupt<0xa1> {}
    unsafe impl Param for Interrupt<0xa2> {}
    unsafe impl Param for Interrupt<0xa3> {}
    unsafe impl Param for Interrupt<0xa4> {}
    unsafe impl Param for Interrupt<0xa5> {}
    unsafe impl Param for Interrupt<0xa6> {}
    unsafe impl Param for Interrupt<0xa7> {}
    unsafe impl Param for Interrupt<0xa8> {}
    unsafe impl Param for Interrupt<0xa9> {}
    unsafe impl Param for Interrupt<0xaa> {}
    unsafe impl Param for Interrupt<0xab> {}
    unsafe impl Param for Interrupt<0xac> {}
    unsafe impl Param for Interrupt<0xad> {}
    unsafe impl Param for Interrupt<0xae> {}
    unsafe impl Param for Interrupt<0xaf> {}
    unsafe impl Param for Interrupt<0xb0> {}
    unsafe impl Param for Interrupt<0xb1> {}
    unsafe impl Param for Interrupt<0xb2> {}
    unsafe impl Param for Interrupt<0xb3> {}
    unsafe impl Param for Interrupt<0xb4> {}
    unsafe impl Param for Interrupt<0xb5> {}
    unsafe impl Param for Interrupt<0xb6> {}
    unsafe impl Param for Interrupt<0xb7> {}
    unsafe impl Param for Interrupt<0xb8> {}
    unsafe impl Param for Interrupt<0xb9> {}
    unsafe impl Param for Interrupt<0xba> {}
    unsafe impl Param for Interrupt<0xbb> {}
    unsafe impl Param for Interrupt<0xbc> {}
    unsafe impl Param for Interrupt<0xbd> {}
    unsafe impl Param for Interrupt<0xbe> {}
    unsafe impl Param for Interrupt<0xbf> {}
    unsafe impl Param for Interrupt<0xc0> {}
    unsafe impl Param for Interrupt<0xc1> {}
    unsafe impl Param for Interrupt<0xc2> {}
    unsafe impl Param for Interrupt<0xc3> {}
    unsafe impl Param for Interrupt<0xc4> {}
    unsafe impl Param for Interrupt<0xc5> {}
    unsafe impl Param for Interrupt<0xc6> {}
    unsafe impl Param for Interrupt<0xc7> {}
    unsafe impl Param for Interrupt<0xc8> {}
    unsafe impl Param for Interrupt<0xc9> {}
    unsafe impl Param for Interrupt<0xca> {}
    unsafe impl Param for Interrupt<0xcb> {}
    unsafe impl Param for Interrupt<0xcc> {}
    unsafe impl Param for Interrupt<0xcd> {}
    unsafe impl Param for Interrupt<0xce> {}
    unsafe impl Param for Interrupt<0xcf> {}
    unsafe impl Param for Interrupt<0xd0> {}
    unsafe impl Param for Interrupt<0xd1> {}
    unsafe impl Param for Interrupt<0xd2> {}
    unsafe impl Param for Interrupt<0xd3> {}
    unsafe impl Param for Interrupt<0xd4> {}
    unsafe impl Param for Interrupt<0xd5> {}
    unsafe impl Param for Interrupt<0xd6> {}
    unsafe impl Param for Interrupt<0xd7> {}
    unsafe impl Param for Interrupt<0xd8> {}
    unsafe impl Param for Interrupt<0xd9> {}
    unsafe impl Param for Interrupt<0xda> {}
    unsafe impl Param for Interrupt<0xdb> {}
    unsafe impl Param for Interrupt<0xdc> {}
    unsafe impl Param for Interrupt<0xdd> {}
    unsafe impl Param for Interrupt<0xde> {}
    unsafe impl Param for Interrupt<0xdf> {}
    unsafe impl Param for Interrupt<0xe0> {}
    unsafe impl Param for Interrupt<0xe1> {}
    unsafe impl Param for Interrupt<0xe2> {}
    unsafe impl Param for Interrupt<0xe3> {}
    unsafe impl Param for Interrupt<0xe4> {}
    unsafe impl Param for Interrupt<0xe5> {}
    unsafe impl Param for Interrupt<0xe6> {}
    unsafe impl Param for Interrupt<0xe7> {}
    unsafe impl Param for Interrupt<0xe8> {}
    unsafe impl Param for Interrupt<0xe9> {}
    unsafe impl Param for Interrupt<0xea> {}
    unsafe impl Param for Interrupt<0xeb> {}
    unsafe impl Param for Interrupt<0xec> {}
    unsafe impl Param for Interrupt<0xed> {}
    unsafe impl Param for Interrupt<0xee> {}
    unsafe impl Param for Interrupt<0xef> {}
    unsafe impl Param for Interrupt<0xf0> {}
    unsafe impl Param for Interrupt<0xf1> {}
    unsafe impl Param for Interrupt<0xf2> {}
    unsafe impl Param for Interrupt<0xf3> {}
    unsafe impl Param for Interrupt<0xf4> {}
    unsafe impl Param for Interrupt<0xf5> {}
    unsafe impl Param for Interrupt<0xf6> {}
    unsafe impl Param for Interrupt<0xf7> {}
    unsafe impl Param for Interrupt<0xf8> {}
    unsafe impl Param for Interrupt<0xf9> {}
    unsafe impl Param for Interrupt<0xfa> {}
    unsafe impl Param for Interrupt<0xfb> {}
    unsafe impl Param for Interrupt<0xfc> {}
    unsafe impl Param for Interrupt<0xfd> {}
    unsafe impl Param for Interrupt<0xfe> {}
    unsafe impl Param for Interrupt<0xff> {}
}
