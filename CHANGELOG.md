# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial dual-backend architecture with the existing vendored C++ backend as
  the default and an opt-in pure-Rust backend via
  `--no-default-features --features rust-backend`.
- Pure-Rust compatibility implementation for the `abieos.h` surface used by
  the safe Rust API: contexts, name conversion, ABI JSON/bin/hex loading, ABI
  JSON/bin conversion, action/table lookups, JSON/bin/hex data conversion, and
  contract deletion.
- Rust-backend parity coverage for fixed arrays, variants, and bitset encoding.
- Rust-backend parity coverage for C++ built-in contract scalar fixtures,
  including integer boundaries, varints, time types, names, bytes, checksums,
  symbols, assets, optionals, fixed arrays, and `extended_asset`.
- Rust-backend parity coverage for C++ binary-extension fixtures, nested arrays,
  long bitsets, fixed struct arrays, variants, and transaction serialization
  through an ABI loaded at contract `0`.
- Test-only C++ oracle bindings exposed as `rs_abieos::cpp_oracle` when
  `cpp-oracle` is enabled, plus a differential smoke test for names and
  built-in scalar conversions.
- CI gates for the pure-Rust backend on Linux, macOS, and Windows MSVC, plus a
  Linux differential job that runs the Rust backend against the C++ oracle.

### Changed
- `build.rs` skips bindgen and C++ compilation when `cpp-backend` is not
  enabled, allowing the Rust backend to compile on MSVC without `libclang` or a
  C++ toolchain.
- When `rust-backend` and `cpp-oracle` are both enabled, the safe Rust API uses
  the Rust backend while the C++ implementation remains available only through
  the oracle module.
- Struct base resolution in the Rust backend is order-independent, matching the
  C++ behavior for ABIs where derived structs sort before their base structs.
- Rust backend contract `0` now uses a loaded ABI when present and falls back to
  the built-in type namespace only when no ABI is loaded there.

## [0.4.0] - 2026-05-17

### Added
- **Windows support** via the `x86_64-pc-windows-gnu` target (MinGW-w64 g++).
  Build and full test suite verified on Windows.
- **macOS/iOS support** — the build no longer panics and links `libc++`
  automatically.
- Windows setup instructions in the README (MinGW-w64 + LLVM/`libclang`).
- Windows (GNU) and macOS jobs added to CI.
- Regression test (`c_string_results_survive_subsequent_calls`) covering the
  buffer-aliasing soundness fix below.

### Fixed
- **Soundness (breaking):** `name_to_string`, `name_to_cstr`, `json_to_hex_c`
  and `hex_to_json_c` returned a borrow into the context's single reused result
  buffer. A value held across any later call into the same context was silently
  overwritten — a use-after-overwrite (and, since the buffer can reallocate, a
  use-after-free) reachable from safe code. It panicked the example binary on
  the C-string round-trip. These methods now return **owned** values.
- **Soundness:** `hex_to_json_c`, `json_to_hex_c`, `name_to_string` and
  `name_to_cstr` called `CStr::from_ptr` on FFI return values without a null
  check. The abieos C entry points are wrapped in `handle_exceptions`, which
  returns null on a null context or a C++ exception (e.g. `bad_alloc`), so
  this was undefined behavior reachable from safe code. All are now
  null-checked and return `Err`.
- `build.rs` now writes the generated bindings to `OUT_DIR` instead of
  `src/bindings.rs`. The previous behavior modified a tracked source file on
  every build, which fails `cargo package` / `cargo publish` verification and
  caused the committed bindings to churn per platform. The committed
  `src/bindings.rs` is retained solely as the docs.rs fallback.

### Changed
- **Breaking:** `name_to_string` now returns `Result<String, AbieosError>`
  (was `Result<&str, AbieosError>`).
- **Breaking:** `name_to_cstr` now returns `Result<CString, AbieosError>`
  (was `&CStr`; owned + null-checked).
- **Breaking:** `json_to_hex_c` and `hex_to_json_c` now return
  `Result<CString, AbieosError>` (was `&CStr`; they previously signalled
  errors by returning an empty/invalid `CStr`).
- `build.rs` is now cross-platform and target-aware. It selects the C++
  standard library per target (`libstdc++` on Linux/MinGW, `libc++` on macOS)
  using Cargo's `CARGO_CFG_TARGET_*` env vars instead of the host-only
  `sys-info` crate, and no longer panics on non-Linux hosts.
- `git submodule update` in the build script is now non-fatal when the
  vendored `lib/abieos` sources are already present (works without `git` on
  `PATH`, e.g. packaged crates).
- Raised the declared `cc` build-dependency floor `1.0.90` → `1.2.62` to match
  the tested version (`cc` and `bindgen` are both already at latest stable).

### Removed
- `sys-info` build dependency (replaced by Cargo-provided target env vars).
- Dead `cargo:rustc-link-search=target/lib/build` directive (a no-op bug; `cc`
  emits the correct search path itself).

### Notes
- The `x86_64-pc-windows-msvc` target is **not supported**: the vendored
  abieos C++ depends on libstdc++/libc++ semantics MSVC's STL lacks. The build
  fails fast with guidance to use the GNU target.

## [0.3.0] - 2025-02-21

### Added
- `Drop` trait implementation — contexts are automatically freed when `Abieos` goes out of scope.
- `as_ptr()` method to access the raw context pointer for FFI interop.
- Thread-safety documentation on the `Abieos` struct explaining `Send` but not `Sync`.

### Changed
- **Breaking:** `context` field is now private. Use `as_ptr()` for raw pointer access.
- **Breaking:** `from_context()` now creates a non-owning wrapper (won't free the context on drop).
- **Breaking:** `destroy()` method removed — replaced by automatic `Drop`.

### Removed
- Removed `is_destroyed` field (was set but never checked).
- Removed `Option` wrapper on context (was always `Some`).

## [0.2.0] - 2025-02-21

### Added
- `delete_contract` and `delete_contract_native` methods on `Abieos` to remove a loaded contract from the context.
- `JsonToBin` error variant for accurate error reporting from `json_to_bin`.
- `impl std::error::Error for AbieosError` — enables `anyhow`, `thiserror`, `eyre` compatibility.
- Upstream abieos support for `bitset` ABI type.
- Upstream abieos support for fixed-size array types (e.g. `uint8[32]`).
- Upstream abieos support for nested array types (array of arrays, optional arrays).
- Upstream abieos `abieos_delete_contract` C API function.

### Changed
- **Breaking:** All methods now accept `&str` instead of `String` for JSON/HEX inputs.
- **Breaking:** All methods now accept `&[u8]` instead of `Vec<u8>` for binary inputs.
- Updated `abieos` C++ submodule from `ae6854e` to `f7d5b45` (latest upstream main).
- Bumped `bindgen` build dependency from `0.69.4` to `0.72.1`.

### Fixed
- **Bug:** `hex_to_json`, `bin_to_json`, `get_type_for_action`, `get_type_for_table`, and `get_type_for_action_result` no longer panic on invalid names — they now propagate errors via `Result`.
- **Bug:** `json_to_bin` now returns `AbieosError::JsonToBin` instead of `AbieosError::JsonToHex`.
- **Bug:** `get_type_for_table_native` now returns `AbieosError::GetTypeForTable` instead of `AbieosError::GetTypeForAction`.
- Upstream fix for `to_json` conversion of `double` values.
- Upstream fix for integer overflow in serialization.
- Upstream fix for `memcpy()` with NULL source pointer.

### Removed
- Removed `fpconv` C library compilation (upstream removed it).
- Removed deprecated `.static_flag(true)` calls in `build.rs`.

## [0.1.5] - 2024-08-01

- Initial tracked release on crates.io.
- Rust wrapper for abieos C library with support for:
  - Name conversion (`string_to_name`, `name_to_string`)
  - ABI loading (JSON, HEX, binary formats)
  - JSON ↔ HEX ↔ binary serialization/deserialization
  - Action and table type lookups
  - ABI format conversion (`abi_bin_to_json`, `abi_json_to_bin`)
