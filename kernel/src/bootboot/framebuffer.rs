//  Copyright 2021 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Provides a means of writing and drawing to the screen.
use core::mem::size_of;
use core::slice;
use embedded_graphics::{
    prelude::*,
    pixelcolor::Rgb888,
};
use lazy_static::lazy_static;
use spin::{Mutex, MutexGuard};
use super::{
    BOOTBOOT,
    FRAMEBUFFER,
    PixelFormat,
};

lazy_static! {
    /// The main framebuffer, which was setup by the BOOTBOOT loader.
    pub static ref CONSOLE: Console = Console(Mutex::new(
        Framebuffer {
            // SAFETY:
            // - kernel must be loaded by a BOOTBOOT-compliant loader
            // - all accesses to `FRAMEBUFFER` are synchronized through `CONSOLE`
            // - `FRAMEBUFFER` must be valid for `BOOTBOOT.fb_size` bytes
            // - all values are valid for `RawPixel`
            buffer: unsafe { slice::from_raw_parts_mut(
                FRAMEBUFFER.as_mut_ptr().cast::<RawPixel>(),
                BOOTBOOT.fb_size as usize / size_of::<RawPixel>())},

            size: Size{ width: BOOTBOOT.fb_width, height: BOOTBOOT.fb_height },
            pitch: BOOTBOOT.fb_scanline / size_of::<RawPixel>() as u32,
            pixel_format: BOOTBOOT.pixel_format(),
        }
    ));
}

/// A synchronized framebuffer.
pub struct Console(Mutex<Framebuffer>);

impl Console {
    /// Returns exclusive access to the main [`Framebuffer`].
    pub fn get() -> MutexGuard<'static, Framebuffer> {
        CONSOLE.0.lock()
    }
}

/// The raw pixel data as it appears in the framebuffer.
#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct RawPixel(u32);

impl RawPixel {
    /// Returns a `RawPixel` from an [`Rgb888`] color based on the given [`PixelFormat`].
    fn from_color(color: Rgb888, format: PixelFormat) -> Self {
        let raw_color = color.into_storage();
        let raw_pixel = match format {
            PixelFormat::Argb => raw_color,
            PixelFormat::Rgba => raw_color << 8,
            PixelFormat::Abgr => raw_color.swap_bytes() >> 8,
            PixelFormat::Bgra => raw_color.swap_bytes(),
        };

        RawPixel(raw_pixel)
    }
}

/// The video memory and metadata used for writing and drawing to a screen.
pub struct Framebuffer {
    /// The memory buffer where pixel data is written.
    buffer: &'static mut [RawPixel],
    /// The dimensions of the display in pixels.
    size: Size,
    /// The in-memory width (in pixels) of a row of pixels. Some bytes may be unused.
    pitch: u32,
    /// The format of the pixels.
    pixel_format: PixelFormat,
}

impl Framebuffer {
    /*
    /// Sets the position of the cursor, where `cursor.x` and `cursor.y` indicate the number of
    /// characters horizontally and vertically, respectively, from the top-left corner of the
    /// screen.
    pub fn set_cursor(&mut self, cursor: Point) {
        assert!(cursor.x >= 0 && cursor.y >= 0);
        let cursor = ((cursor.y << 16) + cursor.x) as u32;
        self.cursor.store(cursor, Ordering::Relaxed);
    }
    */
}

impl OriginDimensions for Framebuffer {
    fn size(&self) -> Size {
        self.size
    }
}

impl DrawTarget for Framebuffer {
    type Color = Rgb888;
    type Error = core::convert::Infallible;

    fn draw_iter<I>(&mut self, pixels: I) -> Result<(), Self::Error>
    where
        I: IntoIterator<Item = Pixel<Self::Color>>
    {
        for Pixel(point, color) in pixels {
            if self.bounding_box().contains(point) {
                let index = point.y as usize * self.pitch as usize + point.x as usize;
                // SAFETY: casting a mutable reference to a pointer and writing to it is just
                // as safe as writing directly to the mutable reference.
                unsafe { ((&mut self.buffer[index] as *mut RawPixel)
                          .write_volatile(RawPixel::from_color(color, self.pixel_format))); }
            }
        }

        Ok(())
    }
}
