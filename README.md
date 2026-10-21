# Rust Abieos

[crates-badge]: https://img.shields.io/crates/v/rs_abieos.svg
[crates-url]: https://crates.io/crates/rs_abieos
[ci-badge]: https://github.com/eosrio/rs-abieos/actions/workflows/ci.yml/badge.svg
[ci-url]: https://github.com/eosrio/rs-abieos/actions/workflows/ci.yml
[coveralls-badge]: https://coveralls.io/repos/github/eosrio/rs-abieos/badge.svg?branch=master
[coveralls-url]: https://coveralls.io/github/eosrio/rs-abieos?branch=master

[![Crates.io][crates-badge]][crates-url]
[![CI][ci-badge]][ci-url]
[![Coverage Status][coveralls-badge]][coveralls-url]

#### [API Documentation](https://docs.rs/rs_abieos/)

`rs_abieos` is a high-performance Rust library for Antelope/EOSIO ABI serialization, deserialization, and name conversion.

Starting in **v0.7.0**, `rs_abieos` uses its **pure-Rust backend (`rust-backend`) by default** — zero C/C++ toolchain dependencies, no `libclang` or `bindgen` required, and full native support across Linux, macOS, and Windows (both `msvc` and `gnu`). The original vendored [AntelopeIO/abieos](https://github.com/AntelopeIO/abieos) C++ library remains available as an opt-in feature (`cpp-backend` / `cpp-oracle`).

## Requirements

### Default Build (`rust-backend`)

Only a standard Rust toolchain (Edition 2024 / Rust 1.85+) is required. No C++ compiler, `cmake`, or `libclang` is needed:

```bash
cargo add rs_abieos
cargo build
```

### Optional C++ Backend Requirements (`cpp-backend` / `cpp-oracle`)

If you opt into `cpp-backend` or `cpp-oracle`, `rs_abieos` builds the vendored [`abieos`](https://github.com/AntelopeIO/abieos) C++ library from source, requiring a C++17 toolchain and `libclang` (used by `bindgen`). **Linux**, **Windows** (`x86_64-pc-windows-gnu`), and **macOS** (Apple Clang + `libc++`) are verified in CI.

#### Linux (`cpp-backend`)

- A C++ toolchain (Clang 18+ or GCC) and `libclang`:

  ```bash
  sudo apt-get update && sudo apt-get install -y clang cmake
  ```

#### macOS (`cpp-backend`)

- Xcode command-line tools (`xcode-select --install`) provide Clang and `libclang`. The build links `libc++` automatically.

#### Windows (`cpp-backend`)

The vendored C++ `abieos` depends on libstdc++/libc++ standard-library semantics that MSVC's STL does not provide. While the default `rust-backend` works natively on `*-pc-windows-msvc`, building `cpp-backend` on Windows requires the `x86_64-pc-windows-gnu` (MinGW-w64) target:

1. Install **MinGW-w64 g++** (e.g. via [WinLibs](https://winlibs.com/) or MSYS2):
   ```powershell
   winget install --id BrechtSanders.WinLibs.POSIX.MSVCRT -e --scope user
   ```
2. Install **LLVM** for `libclang` (required by `bindgen`):
   ```powershell
   winget install --id LLVM.LLVM -e
   $env:LIBCLANG_PATH = "C:\Program Files\LLVM\bin"
   ```
3. Add the Rust GNU target:
   ```powershell
   rustup target add x86_64-pc-windows-gnu
   ```

## Setup Instructions

Add `rs_abieos` to your `Cargo.toml`:

```bash
cargo add rs_abieos
```

### Backend Features

`rs_abieos` provides a dual-backend architecture:

- `rust-backend` (**default**): uses the pure-Rust engine. Requires no C++ compiler, `libclang`, or `bindgen` at build time, works on all Rust targets (including `*-pc-windows-msvc`), and exposes [`AbiHandle`] for standalone, context-free ABI serialization and deserialization.
- `cpp-backend`: builds and links the vendored C++ `abieos` library (`default-features = false, features = ["cpp-backend"]`).
- `cpp-oracle`: builds the vendored C++ backend as an oracle for differential parity tests. When `rust-backend` is also enabled, the safe Rust API uses the Rust backend and exposes the C++ oracle bindings as `rs_abieos::cpp_oracle`.

Recommended CI checks:

```bash
cargo check --all-targets
cargo check --all-targets --no-default-features --features cpp-backend
cargo check --all-targets --no-default-features --features "rust-backend cpp-oracle"
cargo build --bin rs_abieos
cargo build --examples
```

Run tests with the default pure-Rust backend:

```bash
cargo test
```

On a host with the C++ toolchain installed, run the C++ backend or differential oracle tests with:

```bash
cargo test --no-default-features --features cpp-backend
cargo test --no-default-features --features "rust-backend cpp-oracle"
```

## Quick Usage Example

### Standalone `AbiHandle` (Recommended for `rust-backend`)

`AbiHandle` holds a pre-parsed ABI directly without a C context or single-slot-per-account map, making it ideal for high-throughput and historical block/delta decoders:

```rust
use rs_abieos::{AbiHandle, Abieos};

fn main() {
    let abieos = Abieos::new();
    let transfer_u64 = abieos.string_to_name("transfer").unwrap();

    let abi_json = std::fs::read_to_string("abis/eosio.abi").unwrap();
    let mut handle = AbiHandle::from_json(&abi_json).unwrap();

    // Look up action / table / action_result types by u64 name
    println!("Action type: {:?}", handle.type_for_action(transfer_u64));

    // Serialize JSON -> binary / hex and deserialize back into a reusable String buffer
    let bin = handle.json_to_bin("buyrambytes", r#"{"payer":"alice","receiver":"bob","bytes":8192}"#).unwrap();
    let mut out = String::new();
    handle.bin_to_json_into("buyrambytes", &bin, &mut out).unwrap();
    println!("Decoded JSON: {out}");
}
```

### Context-Based `Abieos` & `AbieosContract`

```rust
use rs_abieos::{AbiLike, Abieos, NameLike};

fn main() {
    let abieos = Abieos::new();
    let abi_content = std::fs::read_to_string("abis/eosio.abi").expect("Failed to read ABI file");

    let mut eosio_contract = abieos.contract(NameLike::StringRef("eosio"));
    eosio_contract
        .load_abi(AbiLike::Json(abi_content))
        .expect("Failed to load ABI");

    let action_data = r#"{
        "payer":"alice",
        "receiver":"bob",
        "bytes":8192
    }"#;

    let datatype = eosio_contract
        .get_type_for_action("buyrambytes")
        .expect("Failed to get datatype for action");

    let hex_action = eosio_contract
        .json_to_hex(&datatype, action_data)
        .expect("Failed to serialize action data");
    println!("Serialized action data: {hex_action}");

    let json_action = eosio_contract
        .hex_to_json(&datatype, &hex_action)
        .expect("Failed to deserialize action data");
    println!("Deserialized action data: {json_action}");
}
```

`bin-src/main.rs` contains a more detailed executable example demonstrating multiple use cases for the library.

Please refer to the [API documentation](https://docs.rs/rs_abieos/) for detailed information on every type and method.

## Architecture & Design Notes

### Active Backend Router
The safe Rust public API (`Abieos`, `AbieosContract`, `NameLike`, `AbiLike`, `AbieosError`) is decoupled from the underlying engine. It routes calls to either `backend::rust` (the default pure-Rust engine) or `backend::cpp` (FFI bindings to C++ `abieos`) based on the enabled Cargo features.

### C++ Oracle Role
The `cpp-oracle` feature exposes the original C++ backend side-by-side with the Rust backend. When both `rust-backend` and `cpp-oracle` are active, the safe public API uses the pure Rust engine while the C++ engine remains accessible via `rs_abieos::cpp_oracle`. This enables differential testing where randomized property fuzzing and table-driven parity suites compare outputs, exit codes, and error messages in real time.

### Result Buffer Ownership and Thread Safety
To eliminate the UB risk of use-after-overwrite (where successive calls to FFI functions overwrite the same context result buffer), the safe Rust wrapper copies results into owned Rust types (`String`, `Vec<u8>`) or caller-supplied buffers (`*_into` methods). Both `Abieos` and `AbiHandle` implement `Send` (move between threads freely) and use one instance per worker thread for lock-free parallel decoding.

### Parity Scope
The Rust backend also reads Wire Network `sysio::abi/1.*` ABIs, including
table IDs, secondary indexes, enums, and protobuf descriptor metadata. Wire
table names are matched exactly by `Abieos::get_type_for_table`,
`AbieosContract::get_type_for_table`, and `AbiHandle::type_for_table_name`.
The existing numeric table APIs use legacy EOSIO names and only expose Wire
tables whose names can be represented exactly in that format. Resolve an
arbitrary Wire name with `type_for_table_name`, then pass its row type to
`bin_to_json` when using a standalone handle.

The pure Rust backend implements the C++ `abieos` parsing and serialization rules, including:
- Order-independent base struct resolution
- Pseudo-types (`optional` `?`, `array` `[]`, fixed-size arrays `[N]`, `binary_extension` `$`)
- Variants and `extended_asset`
- `action_results` (ABI 1.2)
- Cryptographic keys (`K1`, `R1`, `WA`, legacy `EOS` and `WIF`) and signature parsing/formatting.

---

## Migration Guide

### Upgrading to v0.7.0 (`rust-backend` Default)
In `0.7.0`, `rust-backend` is enabled by default:

```toml
[dependencies]
rs_abieos = "0.7"
```

If you previously specified `default-features = false, features = ["rust-backend"]`, that configuration continues to work unchanged.

If you specifically need the legacy C++ FFI backend:

```toml
[dependencies]
rs_abieos = { version = "0.7", default-features = false, features = ["cpp-backend"] }
```

### Build Requirements Matrix

| Feature | Backend Engine | C++ Compiler Required | Clang/Bindgen Required | Windows MSVC Support |
| :--- | :--- | :--- | :--- | :--- |
| `rust-backend` (**default**) | Pure Rust | No | No | Yes |
| `cpp-backend` | C++ `abieos.h` | Yes | Yes | No (`windows-gnu` only) |
| `rust-backend` + `cpp-oracle` | Pure Rust + C++ Oracle | Yes | Yes | No (`windows-gnu` only) |

If you discover any behavioral divergence between the C++ and Rust backends on valid or invalid inputs, please file an issue with the hex or JSON payload to help us expand the parity fixture coverage.

---

## Contributor Guide

### Adding Parity Fixtures
If you want to add test coverage for a specific type or format:
1. **Table-driven types**: Add a row to the appropriate table in `tests/rust_backend_check_type_port.rs` or `tests/rust_backend_key_signature_port.rs`.
2. **Error cases**: Add a row in `tests/rust_backend_check_error_port.rs` (checking custom structures, invalid payloads, overflows, or overruns).
3. **Differential tests**: Add a row to `tests/cpp_oracle_differential.rs`. These automatically run against both Rust and C++ backends on Linux CI to ensure exact byte-for-byte and JSON-for-JSON identity.

---

## Release Checklist
Before tagging and publishing a new release:
- [ ] Run format check: `cargo fmt --all -- --check`
- [ ] Run lints across feature sets:
  - `cargo clippy --all-targets -- -D warnings`
  - `cargo clippy --all-targets --no-default-features --features cpp-backend -- -D warnings`
  - `cargo clippy --all-targets --all-features -- -D warnings`
- [ ] Verify test suites pass:
  - `cargo test`
  - `cargo test --no-default-features --features cpp-backend`
  - `cargo test --no-default-features --features "rust-backend cpp-oracle"`
- [ ] Run property and smoke fuzzing: `ABIEOS_FUZZ_ITERS=50000 cargo test --test rust_backend_fuzz_property`
- [ ] Confirm Criterion benchmarks compile: `cargo bench --no-run`
- [ ] Check documentation builds cleanly with `-D warnings`: `RUSTDOCFLAGS="-D warnings" cargo doc --no-deps --all-features`
