//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! `x86_64`-specific types, methods and functions for dealing with memory.

use x86_64::structures::paging::{
    mapper::PageTableFrameMapping, FrameAllocator, MappedPageTable, Mapper, Page, PageTable,
    PageTableFlags, PhysFrame, Size2MiB, Size4KiB,
};

use crate::mem::{Pager, PhysicalAddress, PhysicalMemoryMap, VirtualAddress};

pub use x86_64::{PhysAddr, VirtAddr};

impl PhysicalAddress for PhysAddr {
    fn from_usize(addr: usize) -> Option<Self> {
        Self::try_new(addr.try_into().unwrap()).ok()
    }

    fn to_usize(self) -> usize {
        self.as_u64().try_into().unwrap()
    }
}

impl VirtualAddress for VirtAddr {
    fn from_usize(addr: usize) -> Option<Self> {
        Self::try_new(addr.try_into().unwrap()).ok()
    }

    fn to_usize(self) -> usize {
        self.as_u64().try_into().unwrap()
    }
}

/// The location where physical memory is mapped.
// TODO: this should really us `new` instead of `new_truncate`, but `new` isn't `const`.
pub static PHYSICAL_MEMORY_MAP: PhysicalMemoryMap<VirtAddr> =
    PhysicalMemoryMap::new(VirtAddr::new_truncate(0xffff_8000_0000_0000));
/// The maximum size of `PHYSICAL_MEMORY_MAP`.
pub const PHYSICAL_MEMORY_MAP_MAX_SIZE: usize = 0x0000_4000_0000_0000;

/// A page table heirarchy.
#[derive(Debug)]
pub struct PageMapping {
    pml4: PhysAddr,
}

impl PageMapping {}

impl Pager for PageMapping {
    type Error = ();
    type PhysAddr = PhysAddr;
    type VirtAddr = VirtAddr;

    fn current() -> Self {
        let cr3: usize;

        // SAFETY: reading the `cr3` register is sound
        unsafe {
            core::arch::asm!("mov {}, cr3", out(reg) cr3);
        }

        Self {
            pml4: PhysAddr::from_usize(cr3 & !0xfff)
                .expect("`cr3` must hold a valid physical address"),
        }
    }

    fn new_user_page(&mut self, _addr: Self::VirtAddr) -> Result<(), Self::Error> {
        todo!()
    }

    fn new_kernel_page(&mut self, _addr: Self::VirtAddr) -> Result<(), Self::Error> {
        todo!()
    }

    unsafe fn unmap(&mut self, _addr: Self::VirtAddr) -> Result<Self::PhysAddr, Self::Error> {
        todo!()
    }

    unsafe fn map_physical_mem<I: Iterator<Item = Self::PhysAddr>>(
        mem_size: usize,
        identity_mapped_size: usize,
        free_frames: &mut I,
    ) -> Result<usize, ()> {
        let mapping = Self::current();
        log::debug!("{mapping:#0x?}");
        let translator = MaybeIdentityMapped(identity_mapped_size);
        // SAFETY: `mapping` contains a valid page-table heirarchy pulled from the currently
        // active heirarchy.
        let mut mapper = unsafe {
            MappedPageTable::new(
                mapping
                    .pml4
                    .identity_mapped()
                    .ok_or(())?
                    .as_mut()
                    .ok_or(())?,
                translator,
            )
        };
        let mut frame_alloc = FrameIterator(free_frames);

        let mut frame = PhysFrame::<Size2MiB>::containing_address(PhysAddr::zero());
        let mut page = Page::<Size2MiB>::containing_address(PHYSICAL_MEMORY_MAP.base());
        while frame.start_address().as_u64() < mem_size.try_into().unwrap() {
            if frame.start_address().is_aligned(0x4000_0000u64) {
                log::debug!("mapping {frame:?} to {page:?}");
            }

            // SAFETY: The physical memory map is never used for any other purpose. Frames
            // within the memory map are only ever accessed using the physical memory map
            // when free, unused frames are allocated for page tables.
            unsafe {
                mapper
                    .map_to(
                        page,
                        frame,
                        PageTableFlags::PRESENT | PageTableFlags::WRITABLE | PageTableFlags::GLOBAL,
                        &mut frame_alloc,
                    )
                    .map_err(|_| ())?
                    .flush();
            }

            frame += 1;
            page += 1;

            let mapped_size: usize = frame.start_address().as_u64().try_into().unwrap();
            PHYSICAL_MEMORY_MAP.extend(mapped_size);
        }

        Ok(0)
    }
}

struct MaybeIdentityMapped(usize);

// SAFETY: `frame_to_pointer` validates that the frame is either in the main memory map or
// is identity mapped.
unsafe impl PageTableFrameMapping for MaybeIdentityMapped {
    fn frame_to_pointer(&self, frame: PhysFrame) -> *mut PageTable {
        let addr = frame.start_address();

        addr.mapped()
            .unwrap_or_else(|| {
                if addr < PhysAddr::from_usize(self.0).unwrap() {
                    addr.identity_mapped().unwrap()
                } else {
                    panic!("{addr:?}: address not mapped");
                }
            })
            .as_ptr_mut()
    }
}

struct FrameIterator<'a, I: Iterator<Item = PhysAddr>>(&'a mut I);

// SAFETY: `FrameIterator` will only ever return unused frames.
unsafe impl<'a, I: Iterator<Item = PhysAddr>> FrameAllocator<Size4KiB> for FrameIterator<'a, I> {
    fn allocate_frame(&mut self) -> Option<PhysFrame<Size4KiB>> {
        let ret = Some(PhysFrame::from_start_address(self.0.next()?).unwrap());
        log::debug!("{ret:?}");

        ret
    }
}
