# Backend Performance: C++ vs Pure Rust

Validation of the experimental pure-Rust backend against the vendored C++
`abieos` backend (Milestone 9 of `PURE_RUST_PORT_TRACKER.md`).

## Methodology

- Harness: Criterion, `benches/backend_bench.rs`.
- The benchmark drives **only the safe `rs_abieos::Abieos` public API**, so the
  exact same Rust code path is measured for both backends; only the compiled-in
  backend changes via feature flags. This makes the comparison apples-to-apples
  (including the safe wrapper's per-call `CString`/name overhead, which is
  identical on both sides).
- `[profile.bench]`: `lto = true`, `codegen-units = 1` (applied to both).
- Run:
  ```bash
  cargo bench --no-default-features --features cpp-backend  -- --save-baseline cpp
  cargo bench --no-default-features --features rust-backend -- --baseline cpp
  ```
- Fixtures: canonical `eosio.token` `transfer` action (JSON/hex/bin), the full
  real-world `abis/eosio.abi` (JSON, ~77 KB) and `abis/eosio.abi.bin` (~42 KB),
  `eosio.token` hex ABI, and name conversion.
- Settings for the recorded run: warm-up 1 s, measurement 3 s, 40 samples.
  All 13 benchmarks completed without error on **both** backends, so the safe
  API is functionally correct on both for every fixture exercised here.

## Results (medians)

Two Rust columns: **Rust v1** = initial port; **Rust v2** = after the
zero-copy JSON parser + result-buffer-reuse + borrowed-arg pass. C++ is
unchanged between runs (same saved baseline).

| Benchmark | C++ | Rust v1 | Rust v2 | v2 / C++ | Status |
|---|--:|--:|--:|--:|---|
| `abi_convert/abi_bin_to_json_eosio` | 453.4 µs | 139.8 µs | 136.3 µs | **0.30×** | **Rust 3.33× faster** |
| `codec/json_to_hex_transfer` | 768 ns | 999 ns | 602 ns | **0.78×** | **Rust 1.28× faster** |
| `codec/json_to_bin_transfer` | 691 ns | 905 ns | 555 ns | **0.80×** | **Rust 1.24× faster** |
| `codec/hex_to_json_transfer` | 820 ns | 945 ns | 720 ns | **0.88×** | **Rust 1.14× faster** |
| `codec/bin_to_json_transfer` | 689 ns | 813 ns | 653 ns | **0.95×** | **Rust 1.05× faster** |
| `name/string_to_name` | 19.6 ns | 31.8 ns | 18.8 ns | 0.96× | parity (≤10%) |
| `name/name_to_string` | 26.7 ns | 54.2 ns | 27.1 ns | 1.02× | parity (≤10%) |
| `context/create_destroy` | 13.2 ns | 33.5 ns | 13.5 ns | 1.03× | parity (≤10%) |
| `abi_load/set_abi_hex_token` | 5.15 µs | 8.29 µs | 8.05 µs | 1.56× | Rust 1.56× slower |
| `codec/cold_load_and_json_to_hex` | 5.75 µs | 11.42 µs | 10.50 µs | 1.83× | Rust 1.83× slower |
| `abi_load/set_abi_bin_eosio` | 91.0 µs | 194.2 µs | 179.4 µs | 1.97× | Rust 1.97× slower |
| `abi_convert/abi_json_to_bin_eosio` | 112.1 µs | 292.1 µs | 224.4 µs | 2.00× | Rust 2.00× slower |
| `abi_load/set_abi_json_eosio` | 176.8 µs | 443.0 µs | 384.8 µs | 2.18× | Rust 2.18× slower |

All deltas are statistically significant (Criterion `p < 0.05`). Correctness
re-validated after the optimization pass: the full
`rust-backend + cpp-oracle` suite (Rust parity + C++ oracle differential) and
doctests pass (exit 0, 0 failures).

## Conclusion

Acceptance gate: **"Rust backend within 10% of C++ median throughput before
default flip."**

- **Gate status: 8 of 13 paths PASS** (5 Rust-faster, 3 parity), up from 1/13
  in v1. The optimization pass — zero-copy `Cow` JSON parser, reused result
  buffers, borrowed C-string args — was highly effective:
  - **Hot-path codec flipped from slower to faster than C++.** `json_to_hex`
    went 1.30× slower → **1.28× faster**; the four single-action codec paths
    are all now Rust-faster (1.05×–1.28×).
  - **Fixed-cost overhead eliminated.** `context/create_destroy` (2.55×→1.03×),
    `name_to_string` (2.03×→1.02×), `string_to_name` (1.62×→0.96×) are now at
    parity or faster — confirming v1's regressions were per-call allocation,
    now removed.
  - `abi_bin_to_json` remains a standout: **3.33× faster** than C++.
- **5 paths still fail, all in ABI ingestion/model build:** `set_abi_json`
  2.18×, `abi_json_to_bin` 2.00×, `set_abi_bin` 1.97×, `set_abi_hex` 1.56×,
  and `cold_load_and_json_to_hex` 1.83× (dominated by the `set_abi_hex` it
  includes). These paths build the ABI type-resolution model; the zero-copy
  parser barely helps `set_abi_bin`/`set_abi_hex` because they decode the
  *binary* ABI format, not JSON. The remaining cost is ABI-model construction
  (per-type `String`/`Vec`/`Arc` allocation during struct/variant resolution),
  not serialization.

The pure-Rust port is **correct and now performance-competitive on the
runtime hot path** (serialize/deserialize), but **ABI loading is still ~2×
slower**. Next optimization target is `AbiDef`/`Abi` construction: intern type
names and reduce per-type allocation during resolution. Default flip should
wait until ABI-load paths reach the 10% budget or the deviation is explicitly
accepted (ABI load is typically one-time per contract, unlike codec which is
per-message — an argument for accepting it with documentation).

## Reproducing

```bash
cargo bench --no-default-features --features cpp-backend  -- --save-baseline cpp
cargo bench --no-default-features --features rust-backend -- --baseline cpp
```

HTML reports: `target/criterion/report/index.html`.
