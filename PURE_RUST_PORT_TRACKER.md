# Pure Rust Port Task Tracker

This tracker is the working checklist for taking `rs_abieos` from the current
dual-backend migration scaffold to a pure-Rust default backend with parity
against the vendored C++ `abieos` implementation.

## Goal

Ship a pure-Rust backend that preserves the current safe Rust API, matches the
vendored C++ `abieos.h` reachable behavior, removes C++ toolchain requirements
from the default build, and improves cross-compilation, speed, and
maintainability.

## Status Key

- `[x]` Done
- `[ ]` Not started
- `[~]` In progress
- `[!]` Blocked or needs external/toolchain validation

## Current Snapshot

- `[x]` Existing public Rust API is routed through an internal backend module.
- `[x]` `cpp-backend` remains the default backend.
- `[x]` `rust-backend` builds without C++/bindgen when selected with
  `--no-default-features --features rust-backend`.
- `[x]` `cpp-oracle` can expose C++ bindings separately as
  `rs_abieos::cpp_oracle`.
- `[x]` Rust backend passes the existing Rust test suite under `rust-backend`.
- `[x]` Initial C++ fixture-derived parity tests exist for scalar types,
  binary extensions, fixed arrays, variants, bitsets, keys/signatures, nested
  arrays, and transaction packing through contract `0`.
- `[x]` Additional table-driven Rust-only `check_type` fixtures cover more
  integer/varint boundaries, float/time/name/string/bytes/checksum rows,
  arrays, bitsets, symbols, and assets.
- `[x]` Feature-combination tests and README guidance document backend
  selection, `rust-backend + cpp-oracle` precedence, CI commands, and MSVC
  limitations.
- `[x]` C++ oracle differential tests now use a reusable harness and compare
  status, bytes/hex, and deterministic JSON for current parity fixtures.
- `[!]` C++ oracle mode is intended to run on Linux CI; it cannot be validated
  on the local MSVC shell because the vendored C++ backend is intentionally
  unsupported for MSVC.
- `[x]` Large integer-valued `float64` JSON output now matches the C++ abieos
  fixed-format fixture rows.
- `[x]` Focused Rust-only `check_error` fixtures cover numeric/type errors,
  malformed built-ins, fixed-array/type-spec errors, variants, nested struct
  shape errors, and stream overruns.
- `[x]` Bulk K1/R1/WIF key and signature fixtures are ported to Rust-only tests.

## Milestone 1: Backend Architecture

- `[x]` Add backend feature flags:
  - `cpp-backend`
  - `rust-backend`
  - `cpp-oracle`
- `[x]` Keep `cpp-backend` as the default during migration.
- `[x]` Make `build.rs` skip C++ compilation and bindgen when `cpp-backend` is
  disabled.
- `[x]` Add active backend router module.
- `[x]` Move generated/bindgen C++ surface behind `backend::cpp`.
- `[x]` Add Rust compatibility backend behind `backend::rust`.
- `[x]` Preserve public safe API types and method signatures:
  - `Abieos`
  - `AbieosContract`
  - `NameLike`
  - `AbiLike`
  - `AbieosError`
- `[x]` Make `rust-backend + cpp-oracle` select Rust for the safe API while
  exposing C++ only through `rs_abieos::cpp_oracle`.
- `[x]` Add compile-fail or feature-combination tests for unsupported feature
  combinations.
- `[x]` Add documentation showing recommended feature usage for applications,
  CI, and library consumers.

## Milestone 2: Rust Backend Core

- `[x]` Implement context lifecycle:
  - create
  - destroy
  - last error
  - reusable string result buffer
  - reusable binary result buffer
- `[x]` Match null-context behavior for public FFI entry points used by the safe
  wrapper.
- `[x]` Treat null string arguments as empty strings where the C++ API does.
- `[x]` Implement name conversion:
  - string to name
  - name to string
- `[x]` Implement hex encode/decode helpers.
- `[x]` Implement binary reader/writer:
  - fixed integers
  - varuint32
  - varint32
  - strings
  - byte vectors
  - arrays
- `[x]` Implement JSON parser sufficient for abieos-style ABI and value JSON.
- `[x]` Preserve non-ASCII string content correctly.
- `[ ]` Audit JSON parser against RapidJSON edge behavior:
  - trailing content
  - invalid escapes
  - invalid UTF-8 in binary-to-JSON output
  - number grammar
  - duplicate object fields
- `[ ]` Decide whether to keep the custom parser or replace it with a parser
  crate plus ordering-preserving object handling.

## Milestone 3: ABI Model and Resolution

- `[x]` Parse ABI JSON into Rust structs.
- `[x]` Parse ABI binary into Rust structs.
- `[x]` Convert ABI JSON to binary.
- `[x]` Convert ABI binary to compact JSON.
- `[x]` Support ABI versions:
  - `eosio::abi/1.x`
  - `eosio::abi/2.x`
- `[x]` Implement type aliases.
- `[x]` Implement struct resolution.
- `[x]` Implement order-independent base struct resolution.
- `[x]` Implement variants.
- `[x]` Implement arrays.
- `[x]` Implement fixed arrays.
- `[x]` Implement optionals.
- `[x]` Implement binary extensions.
- `[x]` Implement `extended_asset`.
- `[x]` Implement contract `0` behavior:
  - built-in namespace when no ABI is loaded
  - loaded ABI when contract `0` has an ABI
  - built-ins still available through loaded contract `0`
- `[ ]` Exhaustively port C++ type-spec parser error cases:
  - nested optional/array invalid combinations
  - extension nesting invalid combinations
  - malformed fixed-array syntax
  - recursion limit
  - unknown types
- `[ ]` Add ABI validation parity for:
  - missing type names
  - duplicate type names
  - duplicate structs
  - duplicate variants
  - invalid typedef extension use
  - invalid bases

## Milestone 4: Built-In Type Parity

- `[x]` `bool`
- `[x]` `int8`
- `[x]` `uint8`
- `[x]` `int16`
- `[x]` `uint16`
- `[x]` `int32`
- `[x]` `uint32`
- `[x]` `int64`
- `[x]` `uint64`
- `[x]` `int128`
- `[x]` `uint128`
- `[x]` `varuint32`
- `[x]` `varint32`
- `[x]` `float32`
- `[x]` `float64`
- `[x]` `float128`
- `[x]` `time_point`
- `[x]` `time_point_sec`
- `[x]` `block_timestamp_type`
- `[x]` `name`
- `[x]` `bytes`
- `[x]` `string`
- `[x]` `checksum160`
- `[x]` `checksum256`
- `[x]` `checksum512`
- `[x]` `symbol_code`
- `[x]` `symbol`
- `[x]` `asset`
- `[x]` `extended_asset`
- `[x]` `bitset`
- `[x]` `public_key`
- `[x]` `private_key`
- `[x]` `signature`
- `[ ]` Audit numeric parsing and formatting against C++ for every boundary
  and malformed input.
- `[ ]` Audit float formatting for exact C++ JSON output in edge cases:
  - `[x]` `0.0`
  - `[x]` large integer-valued floats
  - exponent forms
  - NaN/Infinity handling, if reachable
- `[ ]` Audit time parsing and formatting for:
  - fractional truncation
  - invalid dates
  - out-of-range timestamps
  - leap-second-like inputs
- `[ ]` Audit asset parsing for:
  - precision limits
  - symbol length limits
  - invalid whitespace
  - overflow
  - negative zero
- `[~]` Audit key/signature parsing for:
  - `[x]` K1
  - `[x]` R1
  - `[ ]` WA
  - `[x]` legacy EOS public keys
  - `[x]` legacy WIF private keys
  - `[x]` checksum mismatch
  - `[x]` bad size

## Milestone 5: Dynamic Serialization and Deserialization

- `[x]` JSON to binary conversion.
- `[x]` JSON to binary conversion with reorderable top-level object handling.
- `[x]` Binary to JSON conversion.
- `[x]` Hex to JSON conversion.
- `[x]` Context-owned binary result lifetime.
- `[x]` Context-owned string result lifetime.
- `[x]` Overwrite semantics for repeated calls.
- `[x]` Skipped binary-extension output behavior.
- `[x]` Fixed-array serialization/deserialization.
- `[x]` Variant serialization/deserialization.
- `[x]` Nested arrays.
- `[x]` Transaction ABI packing/unpacking fixture.
- `[ ]` Compare ordered vs reorderable behavior with C++ for nested objects.
- `[ ]` Add duplicate-field tests.
- `[ ]` Add extra-field tests.
- `[ ]` Add missing-field tests with path-like error contexts.
- `[ ]` Add binary-extension skipping tests for every nesting shape permitted by
  C++.
- `[ ]` Add stream-overrun tests for every built-in and compound type.
- `[ ]` Add malformed binary tests for variants, arrays, fixed arrays, and
  extensions.

## Milestone 6: C++ Fixture Porting

- `[x]` Port current Rust tests unchanged to `rust-backend`.
- `[x]` Port representative scalar fixtures from `lib/abieos/src/test.cpp`.
- `[x]` Port key/signature fixture coverage.
- `[x]` Port bitset fixture coverage.
- `[x]` Port binary-extension fixture coverage.
- `[x]` Port fixed-array and fixed-struct-array fixture coverage.
- `[x]` Port nested-array fixture coverage.
- `[x]` Port transaction fixture coverage.
- `[~]` Port all remaining `check_type` cases from `test.cpp`.
  - `[x]` Additional built-in success rows are table-driven in
    `tests/rust_backend_check_type_port.rs`.
  - `[x]` Large integer-valued `float64` rows are ported.
  - `[x]` Bulk K1/R1/WIF public/private key and signature rows are ported.
  - `[ ]` WA key/signature rows, packed transaction, and protocol fixture rows
    remain.
- `[~]` Port all remaining `check_error` cases from `test.cpp`.
  - `[x]` Focused table-driven Rust-only error fixtures exist in
    `tests/rust_backend_check_error_port.rs`.
  - `[!]` Full C++ path-aware error-string parity still needs backend work.
- `[ ]` Port packed transaction fixtures.
- `[ ]` Port state-history / ship protocol fixtures.
- `[ ]` Port KV/table/action result ABI fixtures.
- `[ ]` Port ABI JSON/bin conversion edge fixtures.
- `[ ]` Convert fixtures into data-driven tables shared by Rust-only and
  oracle-differential tests.

## Milestone 7: C++ Oracle Differential Tests

- `[x]` Add `cpp-oracle` feature.
- `[x]` Expose C++ bindings as `rs_abieos::cpp_oracle`.
- `[x]` Add oracle smoke test for:
  - name conversion
  - built-in scalar JSON to binary
  - built-in scalar binary to JSON
- `[x]` Add oracle smoke test for transaction ABI loaded at contract `0`.
- `[x]` Create reusable oracle harness helpers:
  - context creation/destruction
  - ABI loading
  - JSON to hex
  - hex to JSON
  - ABI JSON to binary
  - ABI binary to JSON
  - error capture
- `[~]` Run every Rust parity fixture through both Rust and C++ oracle when
  `cpp-oracle` is enabled.
- `[x]` Compare success/failure status.
- `[x]` Compare output hex/binary bytes.
- `[x]` Compare output JSON exactly where C++ output is deterministic.
- `[ ]` Compare public error strings or documented substrings.
- `[ ]` Add CI artifact output for mismatched fixture cases.

## Milestone 8: Fuzz and Property Testing

- `[ ]` Add fuzz target for JSON to binary.
- `[ ]` Add fuzz target for binary to JSON.
- `[ ]` Add fuzz target for ABI JSON to binary.
- `[ ]` Add fuzz target for ABI binary to JSON.
- `[ ]` Add malformed hex input fuzzing.
- `[ ]` Add malformed key/signature input fuzzing.
- `[ ]` Add malformed asset/time input fuzzing.
- `[ ]` Add round-trip property tests:
  - JSON to binary to JSON
  - binary to JSON to binary
  - ABI JSON to binary to JSON
  - ABI binary to JSON to binary
- `[ ]` Add recursion-limit fuzz/properties.
- `[ ]` Add duplicate/ordered field fuzz/properties.
- `[ ]` Decide CI policy for fuzzing:
  - quick smoke fuzz in CI
  - longer scheduled fuzz job
  - local-only corpus generation

## Milestone 9: Benchmarks

- `[ ]` Add Criterion benchmark harness.
- `[ ]` Benchmark C++ backend baseline.
- `[ ]` Benchmark Rust backend.
- `[ ]` Add benchmark cases:
  - context creation/destruction
  - ABI JSON load
  - ABI binary load
  - ABI hex load
  - name conversion
  - JSON to binary ordered
  - JSON to binary reorderable
  - binary to JSON
  - hex to JSON
  - ABI JSON to binary
  - ABI binary to JSON
  - transaction packing
  - transaction unpacking
- `[ ]` Define acceptance threshold:
  - Rust backend within 10% of C++ median throughput before default flip.
- `[ ]` Add benchmark documentation.
- `[ ]` Decide whether benchmarks run in CI, scheduled CI, or manually before
  release.

## Milestone 10: CI and Platform Matrix

- `[x]` Existing default C++ backend CI on Linux.
- `[x]` Existing default C++ backend CI on macOS.
- `[x]` Existing default C++ backend CI on Windows GNU.
- `[x]` Rust backend CI on Linux.
- `[x]` Rust backend CI on macOS.
- `[x]` Rust backend CI on Windows MSVC.
- `[x]` C++ oracle differential CI on Linux.
- `[ ]` Add Rust backend CI on Windows GNU.
- `[ ]` Add Rust backend CI on additional stable targets if cheap:
  - musl
  - aarch64 Linux
  - macOS arm64
- `[ ]` Add `cargo check --all-features` strategy or explicit feature-matrix
  checks.
- `[ ]` Add CI job for docs build with `rust-backend`.
- `[ ]` Add CI job for examples/binary under `rust-backend`.
- `[ ]` Add scheduled full oracle parity job.

## Milestone 11: Documentation

- `[x]` Document backend feature flags in README.
- `[x]` Document Rust backend test command.
- `[x]` Document C++ oracle test command.
- `[x]` Document MSVC recommendation to use `rust-backend`.
- `[ ]` Add architecture notes:
  - active backend router
  - C++ oracle role
  - result buffer ownership model
  - parity scope
- `[ ]` Add migration guide for users:
  - current default
  - opt into Rust backend
  - report parity mismatch
  - expected build requirements by feature
- `[ ]` Add contributor guide for adding parity fixtures.
- `[ ]` Add release checklist to README or this tracker.

## Milestone 12: Default Flip Readiness

- `[ ]` Full Rust parity fixture suite passes.
- `[ ]` Full C++ oracle differential suite passes on Linux.
- `[ ]` Rust backend CI passes on Linux, macOS, Windows GNU, and Windows MSVC.
- `[ ]` Benchmarks meet threshold or deviations are accepted/documented.
- `[ ]` Fuzz/property test suite has no known blockers.
- `[~]` Known error-string differences are eliminated or explicitly accepted.
- `[ ]` No public safe API breakage.
- `[ ]` README updated to announce Rust backend as default.
- `[ ]` `Cargo.toml` default feature flips from `cpp-backend` to
  `rust-backend`.
- `[ ]` `cpp-backend` remains available as an opt-in compatibility backend for
  one transition period.
- `[ ]` `cpp-oracle` remains available for regression checks.
- `[ ]` CI updated so Rust default path runs without C++ toolchain.
- `[ ]` Release notes explain the default flip and opt-in C++ fallback.

## Milestone 13: Cleanup After Default Flip

- `[ ]` Remove bindgen requirement from default build path.
- `[ ]` Remove C++ compiler requirement from default build path.
- `[ ]` Keep vendored C++ submodule only for oracle/compatibility feature while
  needed.
- `[ ]` Decide deprecation timeline for `cpp-backend`.
- `[ ]` Decide deprecation timeline for `cpp-oracle`.
- `[ ]` Remove obsolete Windows GNU-only guidance for default builds.
- `[ ]` Revisit `no_std`, WASM, musl, iOS, and Android as follow-up targets.

## Open Risks

- `[ ]` Error-string parity may require more path-aware error context than the
  current Rust backend has.
- `[ ]` JSON parser behavior may diverge from RapidJSON on malformed inputs.
- `[ ]` Float formatting and parsing may diverge in edge cases.
- `[ ]` Reorderable object handling may differ for nested structs.
- `[ ]` WA key/signature cases need broader fixture coverage.
- `[ ]` State-history/ship protocol fixtures may expose ABI surface not yet
  covered by the current Rust backend.
- `[ ]` Performance is unknown until Criterion benchmarks exist.

## Suggested Next Work Order

1. Convert all remaining `check_type` cases from `lib/abieos/src/test.cpp` into
   table-driven Rust fixtures.
2. Convert all remaining `check_error` cases from `test.cpp`.
3. Expand `cpp_oracle_differential.rs` so the same fixture tables compare Rust
   and C++ outputs automatically.
4. Add packed transaction and state-history fixtures.
5. Add property/fuzz tests for malformed and round-trip cases.
6. Add Criterion benchmarks and compare Rust against C++.
7. Flip the default only after parity, CI, fuzz, and benchmark gates pass.
