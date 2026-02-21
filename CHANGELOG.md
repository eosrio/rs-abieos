# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
