//  Copyright 2021 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
//! Provides a means of writing and drawing to the screen.
use core::fmt;
use core::mem::size_of;
use core::slice;
use embedded_graphics::{
    prelude::*,
    mono_font::{ MonoFont, MonoTextStyle },
    pixelcolor::Rgb888,
    text::Text,
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

            max_chars: Size {
                width: BOOTBOOT.fb_width / Framebuffer::FONT_SIZE.width,
                height: BOOTBOOT.fb_height / Framebuffer::FONT_SIZE.height,
            },
            cursor: Point::zero(),
            text_color: Rgb888::CSS_GRAY,
        }
    ));
}

/// A synchronized framebuffer.
#[derive(Debug)]
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
#[derive(Debug)]
pub struct Framebuffer {
    /// The memory buffer where pixel data is written.
    buffer: &'static mut [RawPixel],
    /// The dimensions of the display in pixels.
    size: Size,
    /// The in-memory width (in pixels) of a row of pixels. Some bytes may be unused.
    pitch: u32,
    /// The format of the pixels.
    pixel_format: PixelFormat,

    /// The dimensions of the display in characters.
    max_chars: Size,
    /// The cursor location in characters.
    cursor: Point,
    /// The foreground color to use when printing text.
    text_color: Rgb888,
}

impl Framebuffer {
    const FONT: MonoFont<'static> = embedded_graphics::mono_font::iso_8859_1::FONT_9X15;
    const FONT_SIZE: Size = Size {
        width: Self::FONT.character_size.width + Self::FONT.character_spacing,
        height: Self::FONT.character_size.height,
    };
    const TAB: &'static str = "        ";

    pub fn cursor_pixel(&self) -> Point {
        self.cursor.component_mul(Point::zero() + Self::FONT_SIZE)
    }

    /// Sets the position of the cursor, where `cursor.x` and `cursor.y` indicate the number of
    /// characters horizontally and vertically, respectively, from the top-left corner of the
    /// screen.
    pub fn set_cursor(&mut self, cursor: Point) {
        self.cursor = cursor;
    }
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

impl fmt::Write for Framebuffer {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        let char_style = MonoTextStyle::new(&Framebuffer::FONT, self.text_color);

        let mut start_index = None;
        let mut char_count = 0;

        for (i, c) in s.char_indices() {
            if c.is_control() {
                if let Some(si) = start_index {
                    Text::new(&s[si..i], self.cursor_pixel(), char_style)
                        .draw(self).expect("draw text");
                    start_index = None;
                    self.cursor.x += char_count as i32;
                    char_count = 0;
                }

                match c {
                    '\t' => {
                        let spaces = &Self::TAB[self.cursor.x as usize % Self::TAB.len()..];
                        Text::new(spaces, self.cursor_pixel(), char_style)
                            .draw(self).expect("draw spaces");
                        self.cursor.x += spaces.len() as i32;
                    },
                    '\n' => {
                        self.cursor.x = 0;
                        self.cursor.y += 1;
                        // TODO: scrolling
                    },
                    _ => { /*ignored */ },
                }
            } else {
                char_count += 1;
                if self.cursor.x as u32 + char_count > self.max_chars.width {
                    if let Some(si) = start_index {
                        Text::new(&s[si..i], self.cursor_pixel(), char_style)
                            .draw(self).expect("draw text");
                        start_index = Some(i);
                        char_count = 1;
                    }

                    self.cursor.x = 0;
                    self.cursor.y += 1;
                    // TODO: scrolling
                } else {
                    start_index.get_or_insert(i);
                }
            }
        }

        if let Some(si) = start_index {
            Text::new(&s[si..], self.cursor_pixel(), char_style)
                .draw(self).expect("drawing text");
            self.cursor.x += char_count as i32;
        }

        Ok(())
    }
}
