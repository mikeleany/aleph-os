//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////
#![doc = include_str!("../README.md")]
#![doc(html_logo_url = "https://mikeleany.github.io/images/aleph-os.png")]
#![no_std]
#![feature(abi_x86_interrupt)]
#![feature(associated_type_defaults)]
#![feature(const_option)]
#![feature(never_type)]
#![deny(unaligned_references)]
#![deny(unsafe_op_in_unsafe_fn)]
#![warn(missing_debug_implementations)]
#![warn(missing_docs)]
#![warn(unused_extern_crates)]
#![warn(clippy::todo)]
#![warn(clippy::undocumented_unsafe_blocks)]
#![warn(clippy::unwrap_used)]

pub mod arch;