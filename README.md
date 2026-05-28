# vey-mimalloc-sys

Rust FFI binding to [mimalloc](https://github.com/microsoft/mimalloc).

It will find a pre-installed jemalloc via pkg-config or vcpkg if the "vendored" feature is not selected.
And the "secure" feature will enable a secure vendored build of mimalloc.
