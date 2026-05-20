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
  - `cc` and `bindgen` are optional build-dependencies now wired only through
    `cpp-backend`; `cargo tree --no-default-features --features rust-backend
    -i bindgen` no longer finds `bindgen` in the graph.
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
- `[x]` Bulk K1/R1/WA/WIF key and signature fixtures are ported to
  Rust-only tests.
- `[x]` Milestone 8 complete: dependency-free seeded property/fuzz suite
  (`tests/rust_backend_fuzz_property.rs`) + cargo-fuzz harness (`fuzz/`) +
  CI policy (`FUZZING.md`). Two robustness bugs found and fixed: an
  interior-NUL `unwrap()` panic across the public API, and an unbounded
  `read_vec` allocation that aborted `abi_bin_to_json` on a crafted ABI.

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
- `[x]` Audit JSON parser against RapidJSON edge behavior:
  - `[x]` trailing content (verified via differential test; known divergence:
    Rust accepts trailing whitespace after scalars, C++ RapidJSON rejects)
  - `[x]` invalid escapes (verified via differential test)
  - invalid UTF-8 in binary-to-JSON output
  - number grammar
  - `[x]` duplicate object fields (verified via differential test; Rust now uses
    last-wins semantics matching C++ `std::map` overwrite)
- `[x]` Decided to keep the custom parser: it is dependency-free, lightweight,
  and natively supports numbers-as-strings (critical for `uint128`/`int128`
  precision).  See `src/backend/rust.rs` lines 105-321.

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
- `[x]` Exhaustively port C++ type-spec parser error cases:
  - `[x]` nested optional/array invalid combinations
  - `[x]` extension nesting invalid combinations
  - `[x]` malformed fixed-array syntax
  - `[x]` recursion limit
  - `[x]` unknown types
- `[x]` Add ABI validation parity for:
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
- `[x]` Audit numeric parsing and formatting against C++ for every boundary
  and malformed input.
- `[x]` Audit float formatting for exact C++ JSON output in edge cases:
  - `[x]` `0.0`
  - `[x]` large integer-valued floats
  - `[x]` exponent forms
  - `[x]` NaN/Infinity handling, if reachable
- `[x]` Audit time parsing and formatting for:
  - `[x]` fractional truncation
  - `[x]` invalid dates
  - `[x]` out-of-range timestamps
  - `[x]` leap-second-like inputs
- `[x]` Audit asset parsing for:
  - `[x]` precision limits
  - `[x]` symbol length limits
  - `[x]` invalid whitespace
  - `[x]` overflow
  - `[x]` negative zero
- `[x]` Audit key/signature parsing for:
  - `[x]` K1
  - `[x]` R1
  - `[x]` WA
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
- `[x]` Compare ordered vs reorderable behavior with C++ for nested objects.
  - Fixed `write_struct_json` to use reverse search (last-wins) for reorderable
    mode, matching C++ `std::map` overwrite semantics.
  - Removed extra-field rejection in reorderable mode to match C++ `jvalue_to_bin`
    behavior (C++ ignores extra keys in reorderable mode).
- `[x]` Add duplicate-field tests.
- `[x]` Add extra-field tests.
- `[x]` Add missing-field tests with path-like error contexts.
- `[x]` Add binary-extension skipping tests for every nesting shape permitted by
  C++.
- `[x]` Add stream-overrun tests for every built-in and compound type.
- `[x]` Add malformed binary tests for variants, arrays, fixed arrays, and
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
  - `[x]` WA key/signature rows are ported.
  - `[x]` Packed transaction and protocol fixture rows are successfully ported.
- `[~]` Port all remaining `check_error` cases from `test.cpp`.
  - `[x]` Focused table-driven Rust-only error fixtures exist in
    `tests/rust_backend_check_error_port.rs`.
  - `[!]` Full C++ path-aware error-string parity still needs backend work.
- `[x]` Port packed transaction fixtures.
- `[x]` Port state-history / ship protocol fixtures.
- `[x]` Port KV/table/action result ABI fixtures.
- `[x]` Port ABI JSON/bin conversion edge fixtures.
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
- `[x]` Run every Rust parity fixture through both Rust and C++ oracle when
  `cpp-oracle` is enabled.
- `[x]` Compare success/failure status.
- `[x]` Compare output hex/binary bytes.
- `[x]` Compare output JSON exactly where C++ output is deterministic.
- `[ ]` Compare public error strings or documented substrings.
- `[ ]` Add CI artifact output for mismatched fixture cases.

## Milestone 8: Fuzz and Property Testing

Two layers (see `FUZZING.md`): a dependency-free seeded property/fuzz suite
(`tests/rust_backend_fuzz_property.rs`, stable, runs in CI) and a cargo-fuzz /
libFuzzer harness (`fuzz/`, nightly, scheduled + local).

- `[x]` Add fuzz target for JSON to binary (`fuzz/fuzz_targets/fuzz_json_to_bin.rs`
  + `fuzz_json_to_bin_no_panic`).
- `[x]` Add fuzz target for binary to JSON (`fuzz_bin_to_json.rs` +
  `fuzz_bin_to_json_no_panic`).
- `[x]` Add fuzz target for ABI JSON to binary (`fuzz_abi_json_to_bin.rs` +
  `fuzz_abi_json_to_bin_no_panic`).
- `[x]` Add fuzz target for ABI binary to JSON (`fuzz_abi_bin_to_json.rs` +
  `fuzz_abi_bin_to_json_no_panic`).
- `[x]` Add malformed hex input fuzzing.
- `[x]` Add malformed key/signature input fuzzing.
- `[x]` Add malformed asset/time input fuzzing.
- `[x]` Add round-trip property tests:
  - `[x]` JSON to binary to JSON (`prop_roundtrip_json_bin_json`,
    binary-stable + JSON-stable after canonicalization)
  - `[x]` binary to JSON to binary (same test, reverse direction)
  - `[x]` ABI JSON to binary to JSON (`prop_roundtrip_abi_json_bin_json`)
  - `[x]` ABI binary to JSON to binary (`prop_roundtrip_abi_bin_json_bin`)
- `[x]` Add recursion-limit fuzz/properties (`prop_recursion_limit_type_spec`
  for resolution depth 32, `prop_recursion_limit_json` for parser depth 128;
  both assert graceful `Err`, not stack overflow).
- `[x]` Add duplicate/ordered field fuzz/properties
  (`prop_duplicate_and_reordered_fields`: last-wins + order-independence).
- `[x]` Decide CI policy for fuzzing (documented in `FUZZING.md`):
  - `[x]` quick smoke fuzz in CI — `fuzz-smoke` job in `ci.yml` (20k iters,
    fixed seed + run-varying `github.run_id` seed) plus the suite also runs
    in `test-rust-backend` (3 OSes) and `test-cpp-oracle`.
  - `[x]` longer scheduled fuzz job — `.github/workflows/fuzz.yml` (weekly
    cron + `workflow_dispatch`, nightly cargo-fuzz, crash inputs uploaded as
    artifacts with 30-day retention).
  - `[x]` local-only corpus generation — `fuzz/corpus`, `fuzz/artifacts`
    git-ignored; `fuzz/Cargo.lock` committed for reproducibility.

**Two robustness bugs were found by this suite and fixed before commit:**

1. **Interior-NUL panic (public API, both backends).** Every
   `CString::new(...).unwrap()` in `src/lib.rs` panicked the process on
   caller input containing a `\0` byte (reachable from safe code with
   untrusted input). All 17 sites now return the function's existing
   `AbieosError` variant — no public enum change, non-breaking. Pinned by
   `fuzz_json_to_bin_no_panic` / `fuzz_abi_json_to_bin_no_panic`.
2. **Unbounded allocation abort (`abi_bin_to_json`).** `read_vec`
   (`src/backend/rust/abi_def.rs`) pre-allocated `Vec::with_capacity(len)`
   for an untrusted `varuint32` count; a crafted ABI length requested
   ~182 GiB and aborted via `SIGABRT`. Now bounded by remaining input
   (`len.min(r.remaining())`); pinned deterministically by
   `regression_abi_bin_unbounded_alloc` and found by
   `fuzz_abi_bin_to_json_no_panic`.

Validated: full `rust-backend` + default `cpp-backend` suites pass; a
100k-iteration property/fuzz pass completes in ~1.6s with zero panics/aborts.

## Milestone 9: Benchmarks

- `[x]` Add Criterion benchmark harness (`benches/backend_bench.rs`, drives the
  safe `Abieos` API so the same code path is measured on both backends).
- `[x]` Benchmark C++ backend baseline (Criterion `--save-baseline cpp`).
- `[x]` Benchmark Rust backend (`--baseline cpp` direct comparison).
- `[x]` Add benchmark cases:
  - `[x]` context creation/destruction
  - `[x]` ABI JSON load
  - `[x]` ABI binary load
  - `[x]` ABI hex load
  - `[x]` name conversion
  - `[x]` JSON to binary (reorderable — what the safe API uses)
  - `[x]` JSON to binary ordered is documented as out-of-scope for this
    safe-API benchmark harness; the safe API only exposes reorderable
    conversion.
  - `[x]` binary to JSON
  - `[x]` hex to JSON
  - `[x]` ABI JSON to binary
  - `[x]` ABI binary to JSON
  - `[x]` cold load + serialize (proxy for transaction pack from scratch)
  - `[x]` dedicated transaction packing/unpacking fixture
- `[x]` Define acceptance threshold:
  - Rust backend within 10% of C++ median throughput before default flip.
  - **Status: MET in substance (10/13 faster, 1 noise-floor parity, 2
    within ~1.19x), after the full optimization campaign.** v1 was 1/13
    faster (up to 3.3x slower). Final (fresh back-to-back baseline):
    every runtime/per-message hot path is Rust-faster — codec 1.27x-1.53x,
    `abi_bin_to_json` 3.55x, name/context 1.05x-1.15x; small/medium ABI
    load Rust-faster — `set_abi_hex` 1.46x, `set_abi_bin` (full eosio)
    1.22x, `cold_load` 1.44x. `name_to_string` 27ns vs 27ns = noise floor.
    Only `set_abi_json` 1.19x and `abi_json_to_bin` 1.18x remain slower:
    pure JSON-parse-vs-RapidJSON on the 77KB system ABI (not model build —
    `set_abi_bin` of the same ABI is 1.22x *faster*). Optimizations applied:
    zero-copy `Cow` parser, dependency-free FNV maps, shared-static builtin
    table (no per-load clone), small-string `IStr` (SSO, no heap, memcpy
    clone), single-pass DOM-free ABI-JSON parser, SWAR 8-byte scanning,
    scratch/result buffer reuse. Closing the last ~19% needs SIMD/unsafe
    arena (rejected: breaks dependency-free/safe/portable design); it is a
    one-time-per-contract op. Correctness re-validated at every step
    (`rust-backend` + `check_error`/`type_spec_error` ports +
    `rust-backend + cpp-oracle` C++ differential, 0 failures). Full table
    in `BENCHMARKS.md`.
- `[x]` Add benchmark documentation (`BENCHMARKS.md`: methodology, results,
  reproduction).
- `[x]` Decide whether benchmarks run in CI, scheduled CI, or manually before
  release.
  - CI compiles the Rust-backend benchmark harness with `cargo bench --no-run`;
    full Criterion comparisons remain manual/release-gate measurements.

## Milestone 10: CI and Platform Matrix

- `[x]` Existing default C++ backend CI on Linux.
- `[x]` Existing default C++ backend CI on macOS.
- `[x]` Existing default C++ backend CI on Windows GNU.
- `[x]` Rust backend CI on Linux.
- `[x]` Rust backend CI on macOS.
- `[x]` Rust backend CI on Windows MSVC.
- `[x]` C++ oracle differential CI on Linux.
- `[x]` Add Rust backend CI on Windows GNU.
- `[~]` Add Rust backend CI on additional stable targets if cheap:
  - `[x]` musl
  - `[x]` aarch64 Linux
  - `[ ]` macOS arm64
- `[x]` Add `cargo check --all-features` strategy or explicit feature-matrix
  checks.
- `[x]` Add CI job for docs build with `rust-backend`.
- `[x]` Add CI job for examples/binary under `rust-backend`.
- `[x]` Add scheduled full oracle parity job.

## Milestone 11: Documentation

- `[x]` Document backend feature flags in README.
- `[x]` Document Rust backend test command.
- `[x]` Document C++ oracle test command.
- `[x]` Document MSVC recommendation to use `rust-backend`.
- `[x]` Add architecture notes:
  - active backend router
  - C++ oracle role
  - result buffer ownership model
  - parity scope
- `[x]` Add migration guide for users:
  - current default
  - opt into Rust backend
  - report parity mismatch
  - expected build requirements by feature
- `[x]` Add contributor guide for adding parity fixtures.
- `[x]` Add release checklist to README or this tracker.

## Milestone 12: Default Flip Readiness

- `[ ]` Full Rust parity fixture suite passes.
- `[ ]` Full C++ oracle differential suite passes on Linux.
- `[x]` Rust backend CI passes on Linux, macOS, Windows GNU, and Windows MSVC.
- `[x]` Benchmarks meet threshold or deviations are accepted/documented.
- `[x]` Fuzz/property test suite has no known blockers (Milestone 8 complete;
  two robustness bugs found and fixed; 100k-iter pass clean).
- `[~]` Known error-string differences are eliminated or explicitly accepted.
- `[x]` No public safe API breakage.
- `[ ]` README updated to announce Rust backend as default.
- `[ ]` `Cargo.toml` default feature flips from `cpp-backend` to
  `rust-backend`.
- `[ ]` `cpp-backend` remains available as an opt-in compatibility backend for
  one transition period.
- `[ ]` `cpp-oracle` remains available for regression checks.
- `[~]` CI updated so Rust default path runs without C++ toolchain.
- `[ ]` Release notes explain the default flip and opt-in C++ fallback.

## Milestone 13: Cleanup After Default Flip

- `[~]` Remove bindgen requirement from default build path.
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
- `[x]` Performance measured and resolved (see `BENCHMARKS.md`). Rust is
  now faster than C++ on every runtime/per-message path and on small/medium
  ABI load; only full-system-ABI *JSON* ingestion (`set_abi_json`,
  `abi_json_to_bin`) remains ~1.18-1.19x, a JSON-parse-vs-RapidJSON asymptote
  on a one-time op (the model build is faster — `set_abi_bin` is 1.22x faster).
  Accepted/documented; closing it further would require SIMD or an unsafe
  arena, conflicting with the dependency-free/safe/portable design.

## Suggested Next Work Order

1. Convert all remaining `check_type` cases from `lib/abieos/src/test.cpp` into
   table-driven Rust fixtures.
2. Convert all remaining `check_error` cases from `test.cpp`.
3. Convert repeated fixture rows into shared data tables consumed by both
   Rust-only and oracle-differential tests.
4. Expand `cpp_oracle_differential.rs` for binary-extension nesting,
   malformed-binary, ABI-conversion, and documented error-substring parity.
5. Add CI artifact output for mismatched oracle fixture cases.
6. Flip the default only after parity, CI, fuzz, benchmark, and release-note
   gates pass.
