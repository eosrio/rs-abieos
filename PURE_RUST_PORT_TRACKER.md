# Pure Rust Port Task Tracker

This tracker records the progression of `rs_abieos` from a C++-only wrapper to a
pure-Rust default backend (`rust-backend`) with differential parity against the
vendored C++ `abieos` implementation (`f7d5b45`).

## Goal

Ship a pure-Rust backend that preserves the safe Rust API, matches the vendored
C++ `abieos.h` reachable behavior, removes C++ toolchain requirements from the
default build, and improves cross-compilation, speed, and maintainability.

## Status Key

- `[x]` Done
- `[ ]` Not started
- `[~]` In progress
- `[!]` Blocked or needs external/toolchain validation

## Current Snapshot (v0.7.0)

- `[x]` Existing public Rust API is routed through an internal backend module.
- `[x]` **`rust-backend` is now the default backend (`default = ["rust-backend"]` in `v0.7.0`).**
- `[x]` Default builds require no C++ compiler, `cmake`, `libclang`, or `bindgen`.
  - `cc` and `bindgen` are optional build-dependencies wired only through
    `cpp-backend` / `cpp-oracle`; `cargo tree -i bindgen` finds nothing in the
    default graph.
- `[x]` `cpp-backend` remains available as an opt-in backend via
  `--no-default-features --features cpp-backend`.
- `[x]` `cpp-oracle` exposes C++ bindings separately as `rs_abieos::cpp_oracle`
  for differential testing alongside `rust-backend`.
- `[x]` Upstream `AntelopeIO/abieos` audited (2026-10-20): submodule `lib/abieos`
  at `f7d5b45` (2025-07-29) is confirmed to be the latest commit on `origin/main`.
- `[x]` Rust backend passes the full test suite (118 unit/integration tests +
  doctest) with zero ignored tests.
- `[x]` `AbiHandle` provides a complete standalone, context-free ABI codec
  (`from_json`, `from_bin`, `from_hex`, `type_for_table`, `type_for_action`,
  `type_for_action_result`, `bin_to_json`/`_into`, `hex_to_json`/`_into`,
  `json_to_bin`/`_into`, `json_to_hex`, `decode_table_row`/`_into`,
  `decode_action`/`_into`, `decode_action_result`/`_into`).
- `[x]` Milestone 8 complete: dependency-free seeded property/fuzz suite
  (`tests/rust_backend_fuzz_property.rs`) + cargo-fuzz harness (`fuzz/`) +
  CI policy (`FUZZING.md`).
- `[x]` Milestone 9 complete: Rust backend outperforms C++ across runtime
  serialization/deserialization paths (see `BENCHMARKS.md`).
- `[x]` Milestones 12 & 13 complete: `rust-backend` is the default in `0.7.0`,
  Edition 2024 is enabled, and documentation/CI are updated.

## Milestone 1: Backend Architecture

- `[x]` Add backend feature flags (`cpp-backend`, `rust-backend`, `cpp-oracle`).
- `[x]` Keep `cpp-backend` as the default during initial migration (`0.5.0`–`0.6.0`), then flip to `rust-backend` in `0.7.0`.
- `[x]` Make `build.rs` skip C++ compilation and bindgen when `cpp-backend` is disabled.
- `[x]` Add active backend router module.
- `[x]` Move generated/bindgen C++ surface behind `backend::cpp`.
- `[x]` Add Rust compatibility backend behind `backend::rust`.
- `[x]` Preserve public safe API types and method signatures (`Abieos`, `AbieosContract`, `NameLike`, `AbiLike`, `AbieosError`).
- `[x]` Make `rust-backend + cpp-oracle` select Rust for the safe API while exposing C++ only through `rs_abieos::cpp_oracle`.
- `[x]` Add feature-combination tests (`tests/feature_combinations.rs`).
- `[x]` Add documentation showing recommended feature usage for applications, CI, and library consumers.

## Milestone 2: Rust Backend Core

- `[x]` Implement context lifecycle (create, destroy, last error, reusable string & binary result buffers).
- `[x]` Match null-context behavior for public FFI entry points used by the safe wrapper.
- `[x]` Treat null string arguments as empty strings where the C++ API does.
- `[x]` Implement name conversion (`string_to_name`, `name_to_string`).
- `[x]` Implement hex encode/decode helpers.
- `[x]` Implement binary reader/writer (fixed integers, `varuint32`, `varint32`, strings, byte vectors, arrays).
- `[x]` Implement custom zero-copy JSON parser supporting numbers-as-strings (`uint128`/`int128` precision) and last-wins duplicate keys.

## Milestone 3: ABI Model and Resolution

- `[x]` Parse ABI JSON and binary into Rust structs (`AbiDef`).
- `[x]` Convert ABI JSON <-> binary (`abi_json_to_bin`, `abi_bin_to_json`).
- `[x]` Support ABI versions `eosio::abi/1.x` and `eosio::abi/2.x`.
- `[x]` Implement type aliases, order-independent struct inheritance, variants, arrays, fixed arrays, optionals, binary extensions, `extended_asset`, and `action_results`.
- `[x]` Implement contract `0` behavior (built-in namespace fallback + loaded ABI override).
- `[x]` Exhaustively port C++ type-spec parser and ABI validation error cases.

## Milestone 4: Built-In Type Parity

- `[x]` All 33 built-in types (`bool`, `int8`..`uint128`, `varuint32`, `varint32`, `float32`, `float64`, `float128`, `time_point`, `time_point_sec`, `block_timestamp_type`, `name`, `bytes`, `string`, `checksum160`/`256`/`512`, `symbol_code`, `symbol`, `asset`, `extended_asset`, `bitset`, `public_key`, `private_key`, `signature`).
- `[x]` Boundary and malformed-input audits for numbers, floats, dates/timestamps, assets, and `K1`/`R1`/`WA`/legacy keys and signatures.

## Milestone 5: Dynamic Serialization and Deserialization

- `[x]` JSON <-> binary / hex conversion (ordered and reorderable).
- `[x]` Context-owned result buffers and zero-allocation `*_into` methods.
- `[x]` Binary-extension skipping, fixed arrays, variants, nested arrays, and stream-overrun handling.

## Milestone 6: C++ Fixture Porting

- `[x]` Ported scalar, key/signature, bitset, binary-extension, fixed-array, nested-array, transaction, `packed_transaction`, `ship`, and `testkv` ABI fixtures from `lib/abieos/src/test.cpp`.
- `[x]` Shared binary-extension nesting fixtures between `tests/rust_backend_extension_nesting.rs` and `tests/cpp_oracle_differential.rs`.

## Milestone 7: C++ Oracle Differential Tests

- `[x]` Reusable differential harness in `tests/cpp_oracle_differential.rs` comparing status, binary/hex output, and deterministic JSON output between `rust-backend` and `cpp-oracle`.

## Milestone 8: Fuzz and Property Testing

- `[x]` Seeded property/fuzz test suite (`tests/rust_backend_fuzz_property.rs`) and `cargo-fuzz` targets (`fuzz/`).
- `[x]` Fixed interior-NUL panic and unbounded `read_vec` allocation discovered during fuzzing.

## Milestone 9: Benchmarks

- `[x]` Criterion benchmark suite (`benches/backend_bench.rs`) and results documented in `BENCHMARKS.md`.

## Milestone 10: CI and Platform Matrix

- `[x]` Default Rust backend tested on Linux, macOS, Windows MSVC, and Windows GNU, plus `musl` and `aarch64` target checks.
- `[x]` C++ backend tested on Linux, macOS, and Windows GNU via `--no-default-features --features cpp-backend`.
- `[x]` C++ oracle differential suite and property/fuzz smoke suite run in CI.

## Milestone 11: Documentation

- `[x]` Updated `README.md`, `CHANGELOG.md`, `BENCHMARKS.md`, and `FUZZING.md`.

## Milestone 12: Default Flip Readiness (Completed in v0.7.0)

- `[x]` Full Rust parity fixture suite passes.
- `[x]` Full C++ oracle differential suite passes on Linux.
- `[x]` Rust backend CI passes on Linux, macOS, Windows GNU, and Windows MSVC.
- `[x]` Benchmarks meet threshold and are documented in `BENCHMARKS.md`.
- `[x]` Fuzz/property test suite has no known blockers.
- `[x]` Known error-string differences are documented/handled in differential tests.
- `[x]` No public safe API breakage.
- `[x]` `README.md` updated to announce Rust backend as default.
- `[x]` `Cargo.toml` default feature flipped from `cpp-backend` to `rust-backend`.
- `[x]` `cpp-backend` remains available as an opt-in compatibility backend.
- `[x]` `cpp-oracle` remains available for regression checks.
- `[x]` CI updated so default path runs `rust-backend` and C++ jobs explicitly pass `--no-default-features --features cpp-backend`.
- `[x]` `CHANGELOG.md` documents the `0.7.0` default flip and opt-in C++ fallback.

## Milestone 13: Cleanup After Default Flip

- `[x]` Remove `bindgen` requirement from default build path.
- `[x]` Remove C++ compiler requirement from default build path.
- `[x]` Keep vendored C++ submodule only for `cpp-backend` / `cpp-oracle` features.
- `[x]` Remove obsolete Windows GNU-only guidance for default builds (moved under optional `cpp-backend` section).
- `[ ]` Decide future deprecation timeline for `cpp-backend` (retained throughout `0.7.x`).
- `[ ]` Revisit `no_std`, `wasm32-unknown-unknown`, iOS, and Android as follow-up targets.
