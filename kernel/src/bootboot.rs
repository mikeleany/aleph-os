//  Copyright 2021 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Provides information about the environment from the [BOOTBOOT] loader.
//!
//! [BOOTBOOT]: https://gitlab.com/bztsrc/bootboot

mod framebuffer;
use core::{mem::size_of, ops::Range, slice};

pub use framebuffer::{Console, Framebuffer};

extern "C" {
    /// The BOOTBOOT information structure.
    ///
    /// Imported from the symbol `bootboot`.
    ///
    /// # Safety
    /// This static is always safe to use assuming the kernel is loaded by a BOOTBOOT-compliant
    /// loader.
    /// Use [`BOOTBOOT`] instead to avoid using the `unsafe` keyword.
    #[link_name = "bootboot"]
    pub static BOOTBOOT_EXT: Bootboot;

    /// The framebuffer set up by the loader.
    ///
    /// Imported from the symbol `fb`.
    ///
    /// # Safety
    /// For safe use of this structure, all of the following conditions must be met.
    /// - the kernel must be loaded by a BOOTBOOT-compliant loader.
    /// - as with all mutable statics, the user ensure that access is synchronized between threads
    ///
    /// Note that while `FRAMEBUFFER` is defined here as a zero-length array, it is actually valid
    /// for [`BOOTBOOT.fb_size`] bytes, but Rust has no way to indicate this at compile-time.
    ///
    /// [`BOOTBOOT.fb_size`]: Bootboot::fb_size
    #[link_name = "fb"]
    pub static mut FRAMEBUFFER: [u8; 0];
}

/// A safe reference to the BOOTBOOT information structure.
pub static BOOTBOOT: &Bootboot = {
    // SAFETY: the kernel must be loaded by a BOOTBOOT-compliant loader
    unsafe { &BOOTBOOT_EXT }
};

/// The color format for a pixel in the [`FRAMEBUFFER`].
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub enum PixelFormat {
    /// 32-bit color in ARGB order.
    Argb = 0,
    /// 32-bit color in RGBA order.
    Rgba = 1,
    /// 32-bit color in ABGR order.
    Abgr = 2,
    /// 32-bit color in BGRA order.
    Bgra = 3,
}

/// The BOOTBOOT information structure.
#[repr(C)]
#[derive(Debug)]
pub struct Bootboot {
    /// The BOOTBOOT magic value which must be the byte string `b"BOOT"`
    pub magic: [u8; 4],
    /// The size of the bootboot structure, including the memory map, in bytes.
    pub size: u32,
    /// Information regarding how the kernel was loaded.
    pub protocol: u8,
    /// The framebuffer's color format.
    pub fb_type: u8,
    /// The number of CPU cores.
    pub numcores: u16,
    /// The bootstrap processor ID.
    pub bspid: u16,
    /// The timezone, if it can be determined, in minutes before or after UTC. Zero, if the
    /// timezone cannot be determined.
    pub timezone: i16,
    /// The UTC date and time in binary-coded decimal, formatted as yyyymmddhhmmss.
    pub datetime: [u8; 8],
    /// The **physical** address of the ramdisk (mapped in the positive address range).
    pub initrd_ptr: u64,
    /// The size, in bytes, of the ramdisk.
    pub initrd_size: u64,
    /// The **physical** address of the framebuffer. Use a reference or pointer to [`FRAMEBUFFER`]
    /// to get the virtual address.
    pub fb_ptr: u64,
    /// The size, in bytes, of the framebuffer.
    pub fb_size: u32,
    /// The display width of the framebuffer in pixels. Note that the actual memory width may be
    /// larger.
    pub fb_width: u32,
    /// The height of the framebuffer in pixels.
    pub fb_height: u32,
    /// The memory width of the framebuffer in bytes.
    pub fb_scanline: u32,
    /// Information specific to the x86-64 architecture.
    #[cfg(target_arch = "x86_64")]
    pub arch: ArchX86_64,
    /// Information specific to the AArch64 architecture.
    #[cfg(target_arch = "aarch64")]
    pub arch: ArchAarch64,
    /// The beginning of the memory map.
    mmap: [MMapEnt; 0],
}

impl Bootboot {
    /// Returns the [`PixelFormat`] that should be used for the [`FRAMEBUFFER`].
    pub fn pixel_format(&self) -> PixelFormat {
        match self.fb_type {
            0 => PixelFormat::Argb,
            1 => PixelFormat::Rgba,
            2 => PixelFormat::Abgr,
            3 => PixelFormat::Bgra,
            t => panic!("BOOTBOOT.fb_type has an invalid value: {t}"),
        }
    }

    /// Returns a reference to the memory map.
    pub fn memory_map(&self) -> &[MMapEnt] {
        let n = (self.size as usize - size_of::<Self>()) / size_of::<MMapEnt>();

        // SAFETY: BOOTBOOT guarantees that this memory is used for the memory map
        // TODO: determine if pointer provenance still makes this unsound
        unsafe { slice::from_raw_parts(self.mmap.as_ptr(), n) }
    }

    /// Returns an iterator over free frames of memory.
    pub fn free_frames<const FRAME_SIZE: u64>(&'static self) -> FreeFrames<FRAME_SIZE> {
        const { assert!(FRAME_SIZE.is_power_of_two()) };

        let mem_map = self.memory_map().iter();
        FreeFrames {
            mem_map,
            frames: 0..0,
        }
    }
}

/// x86-64-specific fields of the BOOTBOOT information structure.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ArchX86_64 {
    /// The **physical** address of the ACPI memory.
    pub acpi_ptr: u64,
    /// The **physical** address of the SMBI memory.
    pub smbi_ptr: u64,
    /// The **physical** address of the EFI memory.
    pub efi_ptr: u64,
    /// The **physical** address of the MP memory.
    pub mp_ptr: u64,
    _unused: [u64; 4],
}

/// AArch64-specific fields of the BOOTBOOT information structure.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct ArchAarch64 {
    /// The **physical** address of the ACPI memory.
    pub acpi_ptr: u64,
    /// The **physical** address of the BCM2837 memory mapped I/O.
    pub mmio_ptr: u64,
    /// The **physical** address of the EFI memory.
    pub efi_ptr: u64,
    _unused: [u64; 5],
}

/// An entry in the memory map.
#[repr(C)]
#[derive(Debug, Copy, Clone)]
pub struct MMapEnt {
    /// The physical memory address.
    ptr: u64,
    /// The size in bytes.
    size: u64,
}

impl MMapEnt {
    /// Returns the 64-bit physical address of the memory region.
    pub fn address(&self) -> u64 {
        self.ptr
    }

    /// Returns the 64-bit length of the memory region.
    pub fn size(&self) -> u64 {
        self.size & !0xf
    }

    /// Returns `true` if the memory region contains the given address.
    pub fn contains(&self, value: u64) -> bool {
        value >= self.address() && value - self.address() < self.size()
    }

    /// Returns the state of the memory region.
    pub fn mem_type(&self) -> MemType {
        match self.size & 0xf {
            1 => MemType::Free,
            2 => MemType::Acpi,
            3 => MemType::Mmio,
            _ => MemType::Used,
        }
    }
}

/// A type of memory.
#[non_exhaustive]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MemType {
    /// The memory is currently used.
    Used = 0,
    /// The memory is available for use.
    Free = 1,
    /// The memory is used for ACPI.
    Acpi = 2,
    /// The memory is used for memory-mapped I/O.
    Mmio = 3,
}

/// An iterator over free frames of memory.
#[derive(Debug, Clone)]
pub struct FreeFrames<const FRAME_SIZE: u64> {
    mem_map: slice::Iter<'static, MMapEnt>,
    frames: Range<u64>,
}

impl<const FRAME_SIZE: u64> Iterator for FreeFrames<FRAME_SIZE> {
    type Item = u64;

    fn next(&mut self) -> Option<Self::Item> {
        const { assert!(FRAME_SIZE.is_power_of_two()) };
        let frame_mask: u64 = FRAME_SIZE - 1;

        let mut frame = self.frames.next();

        while frame.is_none() {
            if let Some(mmap_ent) = self.mem_map.next() {
                if mmap_ent.mem_type() != MemType::Free {
                    continue;
                }
                let offset = mmap_ent.address() & frame_mask;
                let start = mmap_ent.address() / FRAME_SIZE;
                let (start, len) = if offset == 0 {
                    (start, mmap_ent.size() / FRAME_SIZE)
                } else {
                    (start + 1, (mmap_ent.size() - offset) / FRAME_SIZE)
                };

                self.frames = start..(start + len);
                frame = self.frames.next();
            }
        }

        frame.map(|frame| frame * FRAME_SIZE)
    }
}
