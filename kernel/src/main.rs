//  Copyright 2021 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
#![doc = include_str!("../README.md")]
#![no_std]
#![no_main]
use core::ops::DerefMut as _;
use embedded_graphics::{
    prelude::*,
    image::Image,
    mono_font::{
        MonoTextStyle,
        iso_8859_1::FONT_10X20,
    },
    pixelcolor::Rgb888,
    text::Text,
};
use rlibc as _; // needed for `memcpy`, etc when using `--build-std`
use tinytga::DynamicTga;

#[cfg(not(test))]
mod panic_handler;
mod bootboot;
use bootboot::Console;

/// The kernel's entry point.
///
/// Exported as `_start`, because that is the symbol the linker uses as the entry point, and since
/// we used the [`no_main`] attribute, `_start` is not provided for us.
///
/// [`no_main`]: https://doc.rust-lang.org/stable/reference/crates-and-source-files.html#the-no_main-attribute
#[export_name = "_start"]
fn main() -> ! {
    // set the cursor after the image and custom text which are displayed below
    Console::get().set_cursor(Point::new(0, 11));
    // initialize the logger
    Console::init().expect("init logger");

    // display an image
    let tga = DynamicTga::<Rgb888>::from_slice(include_bytes!("../assets/aleph-os.tga"))
        .expect("load TGA image");
    let image = Image::new(&tga, Point::new(12, 0));
    image.draw(Console::get().deref_mut()).expect("display TGA image");

    // print some text in a specific font and location
    let char_style = MonoTextStyle::new(&FONT_10X20, Rgb888::WHITE);
    let line = Text::new(
        "  The Aleph Operating System\n",
        Point::zero() + image.bounding_box().size.y_axis(),
        char_style);
    line.draw(Console::get().deref_mut()).expect("printing text");

    log::info!("Hello world!");
    panic!("testing the panic handler");
}
