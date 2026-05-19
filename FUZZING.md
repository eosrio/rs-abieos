# Fuzzing & Property Testing

Milestone 8 of the pure-Rust port. Two complementary layers:

| Layer | Tool | Toolchain | Where it runs |
|-------|------|-----------|---------------|
| Seeded property/fuzz suite | `tests/rust_backend_fuzz_property.rs` (dependency-free) | stable | every push/PR (ci.yml `fuzz-smoke`, `test-rust-backend`, `test-cpp-oracle`) |
| Coverage-guided fuzzing | `fuzz/` (cargo-fuzz / libFuzzer) | nightly | weekly schedule + manual (fuzz.yml); local |

The property suite is deliberately dependency-free (a small SplitMix64 PRNG
instead of `proptest`/`arbitrary`) to match the project's no-dependency
philosophy and to keep every failure exactly reproducible from a
`(seed, iteration)` pair. cargo-fuzz provides the deep, coverage-guided
exploration that random property testing cannot.

## 1. Seeded property/fuzz suite (stable, in CI)

`tests/rust_backend_fuzz_property.rs` exercises the public safe `Abieos`
API only, so it is valid on any backend; it is gated to `rust-backend`
(the port's focus), consistent with the other `tests/rust_backend_*` files.

Coverage:

- **No-panic fuzz** — `json_to_bin`, `bin_to_json`, `abi_json_to_bin`,
  `abi_bin_to_json`, malformed hex, malformed key/signature, malformed
  asset/time. Each call must return a `Result`, never panic or abort.
- **Round-trip properties** — value `JSON→bin→JSON→bin` is binary-stable
  and JSON-stable after canonicalization; real ABI fixtures round-trip
  `ABI JSON→bin→JSON→bin` and `ABI bin→JSON→bin`.
- **Recursion limits** — deeply nested type specs (backend bails at
  resolution depth 32) and deeply nested JSON (parser bails at depth 128)
  must return `Err`, never overflow the stack.
- **Duplicate / reordered fields** — last-wins + order-independence,
  matching the documented C++ `std::map` overwrite semantics.
- **Regression guard** — `regression_abi_bin_unbounded_alloc` pins the
  crafted-ABI-length memory-exhaustion abort (see Findings).

Run it:

```bash
cargo test --no-default-features --features rust-backend \
  --test rust_backend_fuzz_property
```

Tunable via environment variables:

| Var | Default | Meaning |
|-----|---------|---------|
| `ABIEOS_FUZZ_ITERS` | `400` | iterations per fuzz test (CI smoke uses `20000`) |
| `ABIEOS_FUZZ_SEED`  | `0x0DDB1A5E5BADF00D` | base PRNG seed |

Deep local run / reproduce a CI failure:

```bash
ABIEOS_FUZZ_ITERS=1000000 ABIEOS_FUZZ_SEED=42 \
  cargo test --no-default-features --features rust-backend \
  --test rust_backend_fuzz_property -- --nocapture
```

On a panic the harness prints the test name, seed, iteration index and the
offending input (hex), with the exact command to replay it. Iteration `i`
is fully determined by `(seed, i)` and is independent of the total count,
so a reported failure replays deterministically.

## 2. Coverage-guided fuzzing (nightly, scheduled / local)

`fuzz/` is a separate cargo package with its own `[workspace]` table, so
stable `cargo build`/`cargo test` at the repo root never touch it. Targets:
`fuzz_json_to_bin`, `fuzz_bin_to_json`, `fuzz_abi_json_to_bin`,
`fuzz_abi_bin_to_json`.

```bash
rustup toolchain install nightly
cargo install cargo-fuzz --locked

# Fuzz a target (Ctrl-C to stop):
cargo +nightly fuzz run fuzz_abi_bin_to_json

# Time-boxed, as CI runs it:
cargo +nightly fuzz run fuzz_abi_bin_to_json -- \
  -max_total_time=300 -rss_limit_mb=4096

# Replay a crashing input:
cargo +nightly fuzz run fuzz_abi_bin_to_json fuzz/artifacts/fuzz_abi_bin_to_json/crash-<hash>
```

Corpus generation is local-only: `fuzz/corpus/` and `fuzz/artifacts/` are
git-ignored; `fuzz/Cargo.lock` is committed for reproducibility. A useful
seed corpus is the `abis/` fixtures (binary ABIs for the `*_bin_*` targets,
JSON for the `*_json_*` targets).

## CI policy (Milestone 8 decision)

- **Per push/PR (stable, fast):** the seeded property/fuzz suite runs in
  `test-rust-backend` (3 OSes) and `test-cpp-oracle`, plus a dedicated
  `fuzz-smoke` job at 20k iterations with both a fixed seed and a
  run-varying seed (`github.run_id`) so new random inputs are tried on
  every run while regressions still reproduce from the fixed seed.
- **Weekly + on-demand (nightly, deep):** `.github/workflows/fuzz.yml`
  runs each cargo-fuzz target for a bounded time and uploads any crash
  input as a build artifact (30-day retention) for local replay.
- **Local:** unbounded `cargo fuzz run` and corpus growth.

## Findings (fixed by this milestone)

Both were found by the suite above and fixed before it was committed:

1. **Interior-NUL panic (public API, all backends).** Every
   `CString::new(...).unwrap()` in `src/lib.rs` panicked the process when
   caller input contained a `\0` byte. Reachable from safe code with
   untrusted input. Fixed: all 17 sites now return the function's existing
   `AbieosError` variant (no public enum change).
2. **Unbounded allocation abort (`abi_bin_to_json`).** `read_vec` in
   `src/backend/rust/abi_def.rs` did `Vec::with_capacity(len)` where `len`
   is an untrusted `varuint32` (up to ~4.29e9); a crafted ABI length field
   requested ~182 GiB and aborted the process via `SIGABRT`. Fixed: the
   pre-allocation is bounded by the bytes still available
   (`len.min(r.remaining())`) — every element consumes ≥1 byte, so any
   larger length is malformed and now fails fast with an `Err`.
