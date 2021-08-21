# The Aleph Operating System

The Aleph Operating System, or Aleph OS, is a hobby operating system written in Rust. It is the successor to [Myros]. It is dual-platform, running on x86-64 PCs and Raspberry Pi 3 and 4. Note that PC builds do not currently have legacy BIOS boot support.

**WARNING: Aleph OS is provided without warranty of any kind (see [LICENSE](LICENSE)). I take no responsibility if running Aleph OS on real hardware bricks or otherwise damages your device.**

[Myros]: https://mikeleany.github.io/myros/

## Supported Rust Version

Building Aleph OS currently requires a nightly version of Rust and has only been tested with version "1.56.0-nightly (ad981d58e 2021-08-08)". It uses the 2021 edition of Rust, so it cannot use versions from before the [2021 edition public testing period] began. It's also possible that later nightly versions of Rust introduce breaking changes, so only the tested version is guaranteed to work.

I would like to eventually be able to build Aleph OS on stable rust, however this will likely require stabilization of many unstable Rust features and so will probably be years in the future, if at all.

Unstable features that are currently used, or that I expect to use include:
- `edition2021`. Targeted for stabilization "for Rust 1.56, which will be released on October 21st, 2021."
- [`build-std`]. I expect this feature will eventually be stabilized, but it could take a while.
  - ["Adding Rust-Stable libstd Support for Xous"] documents a possible stable workaround.
  - For Aarch64, I could probably build without `build-std` if I used another method to pull in the linker script.
- [`asm`]. This feature appears to be on a path to stabilization, but I don't know when that might happen.
  - Not currently used, but I expect to make heavy use of this feature.
  - Alternatively, I could put all assembly code in separate assembly files and import them using `extern "C"`.
- [`naked_fns`]. I would think this feature would be stabilized eventually, but I don't know that for sure.
  - Not currently used, but I expect to use this feature for interrupt handlers.
  - Alternatively, I could put trampoline functions in separate assembly files and import them using `extern "C"`.

[2021 edition public testing period]: https://blog.rust-lang.org/2021/07/21/Rust-2021-public-testing.html
[`build-std`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std
["Adding Rust-Stable libstd Support for Xous"]: https://www.crowdsupply.com/sutajio-kosagi/precursor/updates/adding-rust-stable-libstd-support-for-xous
[`asm`]: https://doc.rust-lang.org/beta/unstable-book/library-features/asm.html
[`naked_fns`]: https://github.com/nox/rust-rfcs/blob/master/text/1201-naked-fns.md

## Building Aleph OS
As mentioned above, building Aleph OS requires a nightly verison of Rust. If you don't already have the nightly channel installed, you can do so with the following command:
```bash
rustup toolchain install nightly
```

Aleph OS also uses the [`build-std`] feature. Using thins feature requires the `rust-src` component, which can be installed with the following command:
```bash
rustup component add rust-src
```

To create a bootable disk image, [BOOTBOOT]'s [mkbootimg] is used. You will need to [download][BOOTBOOT downloads] and install it somewhere in your `$PATH`.

After the above software is installed, go to the project's root directory and run one of the following commands:
```bash
make # defaults to x86-64
make arch=x86_64
make arch=aarch64
```

If you have [QEMU] installed, any of the following commands will run it in a QEMU virtual machine:
```bash
make run #defaults to x86-64
make run arch=x86_64
make run arch=aarch64
```

[`build-std`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std
[BOOTBOOT]: https://gitlab.com/bztsrc/bootboot
[mkbootimg]: https://gitlab.com/bztsrc/bootboot/-/tree/master/mkbootimg
[BOOTBOOT downloads]: https://gitlab.com/bztsrc/bootboot/tree/binaries/
[QEMU]: https://www.qemu.org/
