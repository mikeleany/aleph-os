//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Types, methods and functions for dealing with memory.

use core::{
    ops::Add,
    sync::atomic::{AtomicUsize, Ordering},
};

use crate::arch::mem::{VirtAddr, PHYSICAL_MEMORY_MAP};

/// An interface for physical addresses.
pub trait PhysicalAddress: Copy {
    /// Try to create an address from a `usize`. Returns `None` if `addr` is not a valid
    /// physical address.
    fn from_usize(addr: usize) -> Option<Self>;

    /// Converts an address to `usize`.
    fn to_usize(self) -> usize;

    /// Returns true if `alignment` is a power of two and `self` is aligned to `alignment`.
    fn is_aligned(self, alignment: usize) -> bool {
        alignment.is_power_of_two() && self.to_usize() & (alignment - 1) == 0
    }

    /// Converts the address to a virtual address in the kernel's physical memory map. Returns
    /// `None` if the address isn't mapped.
    fn mapped(self) -> Option<VirtAddr> {
        PHYSICAL_MEMORY_MAP.mapped(self)
    }

    /// Converts the address to a virtual address assuming the address is identity mapped.
    fn identity_mapped(self) -> Option<VirtAddr> {
        VirtAddr::from_usize(self.to_usize())
    }
}

/// An interface for virtual addresses.
pub trait VirtualAddress: Copy {
    /// Try to create an address from a `usize`. Returns `None` if `addr` is not a valid
    /// virtual address.
    fn from_usize(addr: usize) -> Option<Self>;

    /// Converts an address to `usize`.
    fn to_usize(self) -> usize;

    /// Returns true if `alignment` is a power of two and `self` is aligned to `alignment`.
    fn is_aligned(self, alignment: usize) -> bool {
        alignment.is_power_of_two() && self.to_usize() & (alignment - 1) == 0
    }

    /// Converts the address to a `const` pointer.
    fn as_ptr<T>(self) -> *const T {
        self.to_usize() as *const _
    }

    /// Converts the address to a `mut` pointer.
    fn as_ptr_mut<T>(self) -> *mut T {
        self.to_usize() as *mut _
    }

    /// Converts the address to a shared reference.
    ///
    /// # Safety
    /// See [`(*const T)::as_ref`](https://doc.rust-lang.org/core/primitive.pointer.html#method.as_ref)
    unsafe fn as_ref<'a, T>(self) -> Option<&'a T> {
        // SAFETY: the caller is responsible for guaranteeing safety
        unsafe { self.as_ptr::<T>().as_ref() }
    }

    /// Converts the address to an exclusive reference.
    ///
    /// # Safety
    /// See [`(*mut T)::as_mut`](https://doc.rust-lang.org/core/primitive.pointer.html#method.as_mut)
    unsafe fn as_mut<'a, T>(self) -> Option<&'a mut T> {
        // SAFETY: the caller is responsible for guaranteeing safety
        unsafe { self.as_ptr_mut::<T>().as_mut() }
    }
}

/// A structure that provides a translation from physical memory addresses to virtual addresses.
#[derive(Debug)]
pub struct PhysicalMemoryMap<V: VirtualAddress> {
    base: V,
    size: AtomicUsize,
}

impl<V: VirtualAddress + Add<usize, Output = V>> PhysicalMemoryMap<V> {
    /// Creates a new, empty physical memory map.
    pub const fn new(base: V) -> Self {
        PhysicalMemoryMap {
            base,
            size: AtomicUsize::new(0),
        }
    }

    /// Returns the base address of the physical memory map.
    pub const fn base(&self) -> V {
        self.base
    }

    /// Returns the `VirtualAddress` where `addr` is mapped, or `None` if the address is not mapped.
    pub fn mapped<P: PhysicalAddress>(&self, addr: P) -> Option<V> {
        if addr.to_usize() < self.size.load(Ordering::Acquire) {
            Some(self.base + addr.to_usize())
        } else {
            None
        }
    }

    /// Atomically extends the size of the physical memory map to `size`. If the memory map is
    /// already larger than or equal to `size`, its size remains unchanged.
    ///
    /// Returns the previous size.
    pub fn extend(&self, size: usize) -> usize {
        self.size.fetch_max(size, Ordering::AcqRel)
    }
}

/// A type which can be used to map and unmap pages of memory.
pub trait Pager {
    /// The type of error which may be returned by paging methods.
    type Error;

    /// A type representing a physical memory address.
    type PhysAddr: PhysicalAddress;

    /// A type representing a virtual memory address.
    type VirtAddr: VirtualAddress;

    /// Returns the currently active pager.
    fn current() -> Self;

    /// Maps a new user-space page at `addr`.
    ///
    /// A new frame is automatically allocated, along with any other frames the `Pager` may need`.
    fn new_user_page(&mut self, addr: Self::VirtAddr) -> Result<(), Self::Error>;

    /// Maps a new kernel-space page at `addr`.
    ///
    /// A new frame is automatically allocated, along with any other frames the `Pager` may need`.
    fn new_kernel_page(&mut self, addr: Self::VirtAddr) -> Result<(), Self::Error>;

    /// Removes the mapping for the page containing `addr`. This method does not deallocate
    /// the frame. It will instead return physical address of the frame, which the caller
    /// may use to deallocate it.
    ///
    /// # Safety
    /// - `addr` must belong to a currently mapped memory page.
    /// - Immediate undefined behavior results if any references point to anywhere within the
    /// page being unmapped or if any stack allocated data is stored within the page.
    /// - It is also undefined behavior if any `Rust` managed smart pointers point to data
    /// within the page.
    /// - Any raw pointers which point to data within the page immediately become invalid, so
    /// dereferencing those pointers would result in undefined behavior.
    unsafe fn unmap(&mut self, addr: Self::VirtAddr) -> Result<Self::PhysAddr, Self::Error>;

    /// Maps `mem_size` bytes of physical memory at `PHYSICAL_MEMORY_MAP.base()`.
    ///
    /// The function may assume that the first `identity_mapped_size` bytes of physical memory
    /// are currently identity mapped. `free_frames` must be an iterator of physical memory
    /// frames which are available for use. Any frames removed from the iterator will no longer
    /// be available.
    ///
    /// # Safety
    /// - `PHYSICAL_ADDRESS_MAP.base()` for `mem_size` bytes must not be used for any other purpose
    /// than mapping physical memory.
    /// - Virtual addresses from 0 to `identity_mapped_size` must be mapped to the equivalent
    /// physical addresses and must not be used for any other purpose.
    /// - The frames returned from `free_frames` must be available for use. That is, they must not
    /// be used for any other purpose.
    /// - Frames removed from `free_frames` must be considered allocated and must not be used
    /// outside this function unless this function shares it in some way.
    /// - `free_frames` must **not** return the physical address 0, as the identity mapping of that
    /// address corresponds to the null pointer, and dereferencing a null pointer results in
    /// undefined behavior.
    unsafe fn map_physical_mem<I: Iterator<Item = Self::PhysAddr>>(
        mem_size: usize,
        identity_mapped_size: usize,
        free_frames: &mut I,
    ) -> Result<usize, Self::Error>;
}
