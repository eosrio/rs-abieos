# rs_abieos 0.2.0 — API Improvement Proposals

Suggestions based on real-world usage in [fleet-router](https://github.com/eosrio/fleet-router) and a full audit of `lib.rs` (v0.1.5, 594 lines).

---

## 1. Accept `&[u8]` instead of `Vec<u8>` for binary inputs

**Affected methods:** `bin_to_json`, `set_abi_bin`, `set_abi_bin_native`, `abi_bin_to_json`

The C FFI only needs a `*const c_char` pointer and a `usize` length. Taking `Vec<u8>` forces callers to clone data they already have as slices (e.g. from WebSocket frames, mmap'd files, or zero-copy buffers).

```diff
 // bin_to_json
-pub fn bin_to_json(&self, account: &str, datatype: &str, bin: Vec<u8>) -> Result<String, AbieosError>
+pub fn bin_to_json(&self, account: &str, datatype: &str, bin: &[u8]) -> Result<String, AbieosError>

 // set_abi_bin
-pub fn set_abi_bin(&self, contract: &str, abi_bin: Vec<u8>) -> Result<bool, AbieosError>
+pub fn set_abi_bin(&self, contract: &str, abi_bin: &[u8]) -> Result<bool, AbieosError>

 // set_abi_bin_native
-pub fn set_abi_bin_native(&self, contract: u64, abi_bin: Vec<u8>) -> Result<bool, AbieosError>
+pub fn set_abi_bin_native(&self, contract: u64, abi_bin: &[u8]) -> Result<bool, AbieosError>

 // abi_bin_to_json
-pub fn abi_bin_to_json(&self, abi: Vec<u8>) -> Result<String, AbieosError>
+pub fn abi_bin_to_json(&self, abi: &[u8]) -> Result<String, AbieosError>
```

**Impact:** Zero-cost for existing callers (just add `&` before their `Vec`). Enables zero-copy consumers.

---

## 2. Accept `&str` instead of `String` for JSON/HEX inputs

**Affected methods:** `set_abi_json`, `set_abi_json_native`, `set_abi_hex`, `set_abi_hex_native`, `json_to_bin`, `json_to_hex`, `json_to_hex_native`, `hex_to_json`, `hex_to_json_native`, `abi_json_to_bin`

Same rationale — `CString::new()` works equally well from `&str`, and callers often want to keep their `String` or only have a `&str`:

```diff
-pub fn set_abi_json(&self, contract: &str, abi_json: String) -> Result<bool, AbieosError>
+pub fn set_abi_json(&self, contract: &str, abi_json: &str) -> Result<bool, AbieosError>

-pub fn json_to_bin(&self, account: &str, datatype: &str, json: String) -> Result<Vec<u8>, AbieosError>
+pub fn json_to_bin(&self, account: &str, datatype: &str, json: &str) -> Result<Vec<u8>, AbieosError>

-pub fn hex_to_json(&self, account: &str, datatype: &str, hex: String) -> Result<String, AbieosError>
+pub fn hex_to_json(&self, account: &str, datatype: &str, hex: &str) -> Result<String, AbieosError>
```

**Impact:** Minor breakage — callers passing `String` just add `.as_str()` or `&`. But it's Rust-idiomatic and more flexible.

---

## 3. Eliminate hidden panics — propagate errors with `?`

Several methods that return `Result` silently `.unwrap()` on internal `string_to_name` calls, turning recoverable errors into panics:

| Method | Line | Unwrap |
|--------|------|--------|
| `hex_to_json` | 417 | `self.string_to_name(account).unwrap()` |
| `bin_to_json` | 445 | `self.string_to_name(account).unwrap()` |
| `get_type_for_action` | 478-479 | `.unwrap()` × 2 |
| `get_type_for_table` | 504-505 | `.unwrap()` × 2 |
| `get_type_for_action_result` | 530-531 | `.unwrap()` × 2 |

**Fix:** Replace `.unwrap()` with `?` — these methods already return `Result`, so propagation is trivial:

```diff
 pub fn bin_to_json(&self, account: &str, datatype: &str, bin: &[u8]) -> Result<String, AbieosError> {
     let ctx = self.ctx();
-    let account = self.string_to_name(account).unwrap();
+    let account = self.string_to_name(account)?;
```

> [!CAUTION]
> This is a correctness bug. Invalid account names currently crash the process instead of returning an error.

---

## 4. Fix wrong error variant in `json_to_bin`

`json_to_bin` (line 409) returns `AbieosError::JsonToHex` on failure, but it's a binary output operation, not hex. Add a dedicated variant:

```diff
 // abieos_error.rs
 pub enum AbieosError {
     // ...
     JsonToHex(String),
+    JsonToBin(String),
     // ...
 }

 // lib.rs - json_to_bin
-_ => Err(AbieosError::JsonToHex(self.get_error()))
+_ => Err(AbieosError::JsonToBin(self.get_error()))
```

---

## 5. Implement `std::error::Error` for `AbieosError`

Currently `AbieosError` only implements `Display` + `Debug`. Adding the `Error` trait enables seamless integration with `anyhow`, `thiserror`, `eyre`, and the `?` operator in generic error contexts:

```rust
impl std::error::Error for AbieosError {}
```

One line, zero cost, massive ergonomics improvement.

---

## 6. Remove `Option` wrapper on `context` field

```rust
pub struct Abieos {
    pub context: Option<*mut abieos_context>,  // always Some(...)
    pub is_destroyed: bool,
}
```

`Abieos::new()` always sets `Some(...)`, `from_context()` always sets `Some(...)`. The `Option` only forces every method to go through `ctx()` which calls `.unwrap()`:

```rust
fn ctx(&self) -> *mut abieos_context {
    self.context.unwrap()  // panics if None — but it's never None
}
```

**Proposal:** Either:
- Store `*mut abieos_context` directly (it can never be None in practice)
- Or make `new()` return `Result<Abieos, AbieosError>` if creation can fail, and keep the struct field non-optional

```diff
 pub struct Abieos {
-    pub context: Option<*mut abieos_context>,
+    context: *mut abieos_context,
     pub is_destroyed: bool,
 }
```

> [!NOTE]
> Making `context` private is also recommended — exposing raw pointers in the public API invites misuse.

---

## 7. Address `from_context` aliased ownership hazard

`from_context` wraps a raw pointer in a new `Abieos` struct:

```rust
pub fn from_context(context: *mut abieos_context) -> Abieos {
    Abieos { context: Some(context), is_destroyed: false }
}
```

This creates **two `Abieos` values owning the same C context**. If both call `destroy()`, it's a double-free. In fleet-router, this is worked around by never calling `destroy()` — but it's a footgun.

**Options:**

**A. Borrow-based wrapper (safest):**
```rust
pub struct AbieosRef<'a> {
    context: *mut abieos_context,
    _lifetime: PhantomData<&'a Abieos>,
}
```

**B. Explicit non-owning flag:**
```rust
pub fn from_context(context: *mut abieos_context) -> Abieos {
    Abieos { context, is_destroyed: false, owns_context: false }
}
// Only destroy if owns_context is true
```

**C. Implement `Drop` with `Arc`-style reference counting.**

---

## 8. Consider thread-safety story (`Sync`)

`Abieos` implements `Send` but not `Sync`:

```rust
unsafe impl Send for Abieos {}
// no Sync impl
```

This forces all multi-threaded consumers into `Arc<Mutex<Abieos>>`, which serializes every encode/decode operation — a significant bottleneck for high-throughput proxies like fleet-router.

**If the C library is safe to call from multiple threads on the same context:** Add `unsafe impl Sync for Abieos {}`.

**If not (more likely):** Document this explicitly and suggest callers create one `Abieos` per thread instead of sharing one behind a `Mutex`. Example:

```rust
// Instead of:
let shared = Arc::new(Mutex::new(Abieos::new()));

// Recommend:
// Create one Abieos per thread/task — each gets its own C context
let abieos = Abieos::new();
```

---

## Summary

| # | Change | Breaking? | Priority |
|---|--------|-----------|----------|
| 1 | `Vec<u8>` → `&[u8]` | Minor | 🔴 High |
| 2 | `String` → `&str` | Minor | 🔴 High |
| 3 | `.unwrap()` → `?` | No | 🔴 High (bug fix) |
| 4 | Wrong error variant | No | 🟡 Medium |
| 5 | `impl Error` | No | 🟡 Medium |
| 6 | Remove `Option` on context | Internal | 🟡 Medium |
| 7 | `from_context` ownership | API change | 🟠 Design |
| 8 | `Sync` / threading docs | API change | 🟠 Design |
