# The Aleph Operating System

The Aleph Operating System, or Aleph OS, is a hobby operating system written in Rust. It is the successor to [Myros]. It is dual-platform, running on x86-64 PCs and Raspberry Pi 3 and 4. Note that PC builds do not currently have legacy BIOS boot support.

**WARNING: Aleph OS is provided without warranty of any kind (see [LICENSE](LICENSE)). I take no responsibility if running Aleph OS on real hardware bricks or otherwise damages your device.**

[Myros]: https://mikeleany.github.io/myros/

## Supported Rust Version

Building Aleph OS for x86-64 currently requires a nightly version of Rust, though the stable toolchain can currently be used for AArch64. Aleph OS requires the 2021 edition of Rust which was stabilized in Rust 1.56.0.

While the x86-64 version currently requires the nightly toolchain, I would like to eventually be able to build the entire OS without the use of unstable features. Currently I'm only using one unstable feature (`build-std`), but there are a few unstable features that I would rather not do without. They include the following:
- [`build-std`]. This is currently used for x86-64, because Rust doesn't currently support the `x86_64-unknown-none` target, let alone provide pre-built libraries. I and others are working on [changing that][PR 89062].
  - Not required for AArch64 since it has [tier 2 support].
  - ["Adding Rust-Stable libstd Support for Xous"] documents a possible stable workaround for x86-64 until `x86_64-unknown-none` gets tier 2 support.
- [`asm`]. This feature appears to be on a path to stabilization, but I don't know when that might happen.
  - Not currently used, but I expect to make heavy use of this feature.
  - Alternatively, I could put all assembly code in separate assembly files and import them using `extern "C"`.
- [`naked_fns`]. I would think this feature would be stabilized eventually, but I don't know that for sure.
  - Not currently used, but I expect to use this feature for interrupt handlers.
  - Alternatively, I could put trampoline functions in separate assembly files and import them using `extern "C"`.

[`build-std`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std
[PR 89062]: https://github.com/rust-lang/rust/pull/89062
[tier 2 support]: https://doc.rust-lang.org/nightly/rustc/target-tier-policy.html
["Adding Rust-Stable libstd Support for Xous"]: https://www.crowdsupply.com/sutajio-kosagi/precursor/updates/adding-rust-stable-libstd-support-for-xous
[`asm`]: https://doc.rust-lang.org/beta/unstable-book/library-features/asm.html
[`naked_fns`]: https://github.com/nox/rust-rfcs/blob/master/text/1201-naked-fns.md

## Building Aleph OS
As mentioned above, building Aleph OS for x86-64 requires a nightly verison of Rust. If you don't already have the nightly channel installed, you can install it with the following command:
```bash
rustup toolchain install nightly
```

Aleph OS also uses the [`build-std`] feature for x86-64. Using this feature requires the `rust-src` component, which can be installed with the following command:
```bash
rustup component add rust-src
```

After the above components are installed, go to the project's root directory and run one of the following commands:
```bash
make # defaults to x86-64
make arch=x86_64
make arch=aarch64
```

If you have [QEMU] installed, any of the following commands will run it in a QEMU virtual machine:
```bash
make qemu #defaults to x86-64
make qemu arch=x86_64
make qemu arch=aarch64
```

[`build-std`]: https://doc.rust-lang.org/nightly/cargo/reference/unstable.html#build-std
[QEMU]: https://www.qemu.org/
