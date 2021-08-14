# Copyright 2021 Michael Leany
# 
# This Source Code Form is subject to the terms of the Mozilla Public
# License, v. 2.0. If a copy of the MPL was not distributed with this
# file, You can obtain one at http://mozilla.org/MPL/2.0/.
#
####################################################################################################
arch := x86_64
profile := debug
cargo-debug-flags := 
cargo-release-flags := --release
cargo-flags := $(cargo-$(profile)-flags)
outdir := target/$(arch)/$(profile)/
kernel-target-dir := kernel/target/$(arch)-aleph_os-kernel/$(profile)/

$(outdir)aleph-os-$(arch).img: aleph-os-image.json aleph-os.conf $(kernel-target-dir)aleph-os.kernel
	mkdir -pv $(outdir)disk-image/boot/
	cp -v aleph-os-image.json aleph-os.conf $(outdir)
	cp -v $(kernel-target-dir)aleph-os.kernel $(outdir)disk-image/boot/
	cd $(outdir) && mkbootimg aleph-os-image.json aleph-os-$(arch).img

$(kernel-target-dir)aleph-os.kernel $(kernel-target-dir)aleph-os.d: kernel/Cargo.toml
	cargo build -Z build-std=core,alloc --manifest-path kernel/Cargo.toml --target kernel/custom-targets/$(arch)-aleph_os-kernel.json $(cargo-flags)

include $(kernel-target-dir)aleph-os.d

.PHONY: doc run clean

doc:
	cargo doc -Z build-std=core,alloc --no-deps --manifest-path kernel/Cargo.toml --target kernel/custom-targets/$(arch)-aleph_os-kernel.json $(cargo-flags)

run: $(outdir)aleph-os-$(arch).img
	qemu-system-x86_64 -drive format=raw,file=$(outdir)aleph-os-$(arch).img -bios OVMF.fd

clean:
	rm -rfv $(outdir)*
	rmdir -pv $(outdir)
	cargo clean -Z build-std=core,alloc --manifest-path kernel/Cargo.toml --target kernel/custom-targets/$(arch)-aleph_os-kernel.json $(cargo-flags)
