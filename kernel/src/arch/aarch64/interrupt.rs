//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////

core::arch::global_asm!(r"
    .global vector_table
    .balign 4096
    vector_table:
        b exception_handler
    .balign 128
        b exception_handler
    .balign 128
        b exception_handler
    .balign 128
        b exception_handler
    .balign 128
        b exception_handler
    .balign 128
        b exception_handler
    .balign 128
        b exception_handler
    .balign 128
        b exception_handler
");

extern "C" {
    pub fn vector_table();
}

#[no_mangle]
pub unsafe extern "C" fn exception_handler() {
    panic!();
}
