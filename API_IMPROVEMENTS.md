# rs_abieos — API Improvement Proposals

Suggestions based on real-world usage in [fleet-router](https://github.com/eosrio/fleet-router) and a full audit of `lib.rs`.

---

## Completed in v0.2.0

- ~~#1 `Vec<u8>` → `&[u8]` for binary inputs~~
- ~~#2 `String` → `&str` for JSON/HEX inputs~~
- ~~#3 `.unwrap()` → `?` — eliminate hidden panics~~
- ~~#4 Fix wrong error variant (`JsonToHex` → `JsonToBin`)~~
- ~~#5 `impl std::error::Error` for `AbieosError`~~

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
| 6 | Remove `Option` on context | Internal | 🟡 Medium |
| 7 | `from_context` ownership | API change | 🟠 Design |
| 8 | `Sync` / threading docs | API change | 🟠 Design |
