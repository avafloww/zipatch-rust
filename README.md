# zipatch

A Rust library for parsing and applying ZiPatch files, as used in Final Fantasy XIV.

This library is mostly a port from [ZiPatchLib in C#](https://github.com/avafloww/zitool/tree/main/ZiPatchLib).
It has yet to be extensively tested, but is used successfully for boot patching
in [Thaliak v2](https://github.com/CrystallineTools/Thaliak/blob/main/v2). Contributions are welcome!

## Minimum Supported Rust Version (MSRV)

This crate requires **Rust 1.78.0 or later**.

The MSRV may be increased in minor version updates. We will ensure that at least the latest stable Rust version at the
time of release is supported.
