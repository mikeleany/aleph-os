/*
 * mykernel/rust/link.ld
 *
 * Copyright (C) 2017 - 2021 Vinay Chandra
 * Copyright (C) 2021 Michael Leany
 *
 * Permission is hereby granted, free of charge, to any person
 * obtaining a copy of this software and associated documentation
 * files (the "Software"), to deal in the Software without
 * restriction, including without limitation the rights to use, copy,
 * modify, merge, publish, distribute, sublicense, and/or sell copies
 * of the Software, and to permit persons to whom the Software is
 * furnished to do so, subject to the following conditions:
 *
 * The above copyright notice and this permission notice shall be
 * included in all copies or substantial portions of the Software.
 *
 * THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND,
 * EXPRESS OR IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF
 * MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
 * NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
 * HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY,
 * WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
 * OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER
 * DEALINGS IN THE SOFTWARE.
 *
 */

initstack       = 0x4000;
mmio            = 0xffffffffc0000000;
fb              = 0xffffffffc4000000;
bootboot        = 0xffffffffc8000000;
environment     = 0xffffffffc8001000;
KERNEL_OFFSET   = 0xffffffffe0000000;

PHDRS
{
  boot PT_LOAD FILEHDR PHDRS;                  /* one single loadable segment */
}
SECTIONS
{
    . = KERNEL_OFFSET;
    .text . + SIZEOF_HEADERS : AT(ADDR(.text) - KERNEL_OFFSET + SIZEOF_HEADERS) {
        __text_start = .;
        KEEP(*(.text.boot)) *(.text .text.* .gnu.linkonce.t*)   /* code */
        . = ALIGN(4096);
        __text_end = .;
    } :boot

    .rodata : AT(ADDR(.rodata) - KERNEL_OFFSET) {
        __rodata_start = .;
        *(.rodata*)
        . = ALIGN(4096);
        __rodata_end = .;
    } :boot

    .data : AT(ADDR(.data) - KERNEL_OFFSET) {
        __data_start = .;
        *(.data*)
        . = ALIGN(4096);
        __data_end = .;
        __bss_start = .;
        *(.bss*)
        . = ALIGN(4096);
        __bss_end = .;
    } :boot

    /DISCARD/ : { *(.eh_frame) *(.comment) }
}
