# Copyright 2021 Michael Leany
# 
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
####################################################################################################
arch := x86_64
profile := debug

ifeq ($(arch),x86_64)
kernel-target := x86_64-unknown-none
cargoflags-kernel := --target $(kernel-target)
cargo-toolchain := stable
qemu-drivespec := format=raw
qemuflags := -bios OVMF.fd -smp 4
else ifeq ($(arch),aarch64)
kernel-target := aarch64-unknown-none-softfloat
cargoflags-kernel := --target $(kernel-target)
cargo-toolchain := stable
qemu-deps := bootboot/bootboot.img
qemu-drivespec := format=raw,if=sd
qemuflags := -M raspi3b -kernel bootboot/bootboot.img
endif
rustflags-kernel := -C link-args=--script=aleph-naught.ld -C relocation-model=static

ifeq ($(profile),release)
cargoflags := $(cargoflags) --release
endif

builddir := target/$(arch)/$(profile)/
kernel-builddir := kernel/target/$(kernel-target)/$(profile)/

-include $(kernel-builddir)aleph-naught.d

$(builddir)aleph-os-$(arch).img: aleph-os-image-$(arch).json aleph-os.conf $(kernel-builddir)aleph-naught bootboot/mkbootimg
	mkdir -pv $(builddir)disk-image/boot/
	cp -v $(kernel-builddir)aleph-naught $(builddir)disk-image/boot/
	bootboot/mkbootimg aleph-os-image-$(arch).json $(builddir)aleph-os-$(arch).img

$(kernel-builddir)aleph-naught: kernel/Cargo.toml kernel/aleph-naught.ld
	cargo +$(cargo-toolchain) fmt $(cargoflags) --manifest-path $<
	cargo +$(cargo-toolchain) clippy $(cargoflags) $(cargoflags-kernel) --manifest-path $<
	RUSTFLAGS="$(rustflags-kernel)" cargo +$(cargo-toolchain) build $(cargoflags) $(cargoflags-kernel) --manifest-path $<

.PHONY: doc run clean

doc:
	cargo +$(cargo-toolchain) doc $(cargoflags) $(cargoflags-kernel) --no-deps --manifest-path kernel/Cargo.toml

qemu: $(builddir)aleph-os-$(arch).img $(qemu-deps)
	qemu-system-$(arch) $(qemuflags) -drive $(qemu-drivespec),file=$<

bootboot/mkbootimg:
	mkdir -pv bootboot
	curl -o bootboot/mkbootimg.zip https://gitlab.com/bztsrc/bootboot/-/raw/binaries/mkbootimg-Linux.zip
	unzip -d bootboot/ bootboot/mkbootimg.zip mkbootimg

bootboot/bootboot.img:
	mkdir -pv bootboot
	curl -o bootboot/bootboot.img https://gitlab.com/bztsrc/bootboot/raw/master/dist/bootboot.img

clean:
	cargo clean --manifest-path kernel/Cargo.toml
	rm -rfv target
