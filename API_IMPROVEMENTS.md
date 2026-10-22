# rs_abieos — API Improvement Proposals (Archive)

Historical API design notes and improvements originally proposed from real-world usage in [fleet-router](https://github.com/eosrio/fleet-router) and audits of `src/lib.rs`. All items in this tracker have been implemented.

---

## Completed in v0.2.0

- [x] **#1 `Vec<u8>` → `&[u8]` for binary inputs** (`set_abi_bin`, `set_abi_bin_native`, `bin_to_json`, `abi_bin_to_json`)
- [x] **#2 `String` → `&str` for JSON/HEX inputs**
- [x] **#3 `.unwrap()` → `?` — eliminate hidden panics on invalid names**
- [x] **#4 Fix wrong error variants** (`JsonToHex` → `JsonToBin`, `GetTypeForAction` → `GetTypeForTable`)
- [x] **#5 `impl std::error::Error for AbieosError`**

## Completed in v0.3.0

- [x] **#6 Remove `Option` wrapper on `context` field and make `context` private** (exposed via `Abieos::as_ptr()`)
- [x] **#7 Address `from_context` aliased ownership hazard** (`owns_context: bool` + automatic `Drop` implementation)
- [x] **#8 Document thread-safety story (`Send` but not `Sync`)**

## Completed in v0.4.0 – v0.5.0

- [x] **Return owned `String` / `CString` from `name_to_string`, `name_to_cstr`, `json_to_hex_c`, `hex_to_json_c`** (eliminating shared-buffer aliasing UB)
- [x] **Guard all FFI pointers against null**
- [x] **Handle interior NUL bytes gracefully across all `CString::new` sites**

## Completed in v0.6.0 – v0.7.0

- [x] **Pure-Rust backend (`rust-backend`) enabled by default** (zero C++ / `bindgen` dependencies on default builds)
- [x] **Hot-loop `u64` native & buffer-reusing methods on `Abieos`** (`bin_to_json_native`, `bin_to_json_native_into`, `json_to_bin_native`, `get_type_for_action_result_native`, `decode_table_row_native`, `decode_table_row_native_into`)
- [x] **Standalone `AbiHandle` (`rust-backend`)** with full encode/decode surface (`from_json`, `from_bin`, `from_hex`, `type_for_table`, `type_for_action`, `type_for_action_result`, `bin_to_json`/`_into`, `hex_to_json`/`_into`, `json_to_bin`/`_into`, `json_to_hex`, `decode_table_row`/`_into`, `decode_action`/`_into`, `decode_action_result`/`_into`)
- [x] **Complete `AbieosContract` convenience methods** (`get_type_for_action_result`, `json_to_bin`, `bin_to_json`)
