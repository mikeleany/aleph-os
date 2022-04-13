//  Copyright 2022 Michael Leany
//
//  This Source Code Form is subject to the terms of the Mozilla Public
//  License, v. 2.0. If a copy of the MPL was not distributed with this
//  file, You can obtain one at http://mozilla.org/MPL/2.0/.
//
////////////////////////////////////////////////////////////////////////////////////////////////////

pub mod interrupt;

pub fn init() {
    unsafe {
        core::arch::asm!(
            "adr x0, {vector_table}",
            "msr VBAR_EL1, x0",
            vector_table = sym interrupt::vector_table,
            options(nostack)
        );
        let ptr = 0x8000_0000_0000_0000 as *const u8;
        ptr.read_volatile();
    }
}
