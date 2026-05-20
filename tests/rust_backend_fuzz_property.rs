//! Milestone 8: dependency-free fuzz and property tests for the Rust backend.
//!
//! These run as ordinary `cargo test` integration tests under
//! `--no-default-features --features rust-backend` (the existing
//! `test-rust-backend` and `test-cpp-oracle` CI jobs already exercise them on
//! Linux/macOS/Windows). They use a small deterministic PRNG instead of an
//! external crate so the suite stays aligned with the project's
//! dependency-free philosophy and so every failure is reproducible.
//!
//! Reproduction / deeper local runs:
//!
//! ```text
//! ABIEOS_FUZZ_ITERS=1000000 ABIEOS_FUZZ_SEED=42 \
//!   cargo test --no-default-features --features rust-backend \
//!   --test rust_backend_fuzz_property -- --nocapture
//! ```
//!
//! On any panic the harness reports the test name, seed, iteration index and
//! the offending input (hex-encoded) so it can be replayed exactly. Deep
//! coverage-guided fuzzing lives in `fuzz/` (cargo-fuzz, nightly); see
//! `FUZZING.md`.

#![cfg(feature = "rust-backend")]

use std::panic::{catch_unwind, AssertUnwindSafe};
use std::sync::OnceLock;

use rs_abieos::Abieos;

// --- Deterministic PRNG (SplitMix64) -------------------------------------

struct Rng {
    state: u64,
}

impl Rng {
    fn new(seed: u64) -> Self {
        Self { state: seed }
    }

    fn next_u64(&mut self) -> u64 {
        // SplitMix64 — tiny, well-distributed, exactly reproducible.
        self.state = self.state.wrapping_add(0x9E37_79B9_7F4A_7C15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
        z ^ (z >> 31)
    }

    fn below(&mut self, n: usize) -> usize {
        if n == 0 {
            0
        } else {
            (self.next_u64() % n as u64) as usize
        }
    }

    fn byte(&mut self) -> u8 {
        (self.next_u64() & 0xFF) as u8
    }

    fn bool(&mut self) -> bool {
        self.next_u64() & 1 == 1
    }

    /// A random byte buffer, length in `[0, max]`.
    fn bytes(&mut self, max: usize) -> Vec<u8> {
        let len = self.below(max + 1);
        (0..len).map(|_| self.byte()).collect()
    }

    /// A random string biased towards JSON / ABI structural characters so the
    /// parsers are exercised on near-miss inputs, not just noise.
    fn junk_string(&mut self, max: usize) -> String {
        const ALPHABET: &[u8] = b"{}[]\":,.0123456789-+eEtruefalsn \t\n\\/abcdefABCDEF\0\x7f";
        let len = self.below(max + 1);
        (0..len)
            .map(|_| ALPHABET[self.below(ALPHABET.len())] as char)
            .collect()
    }
}

// --- Harness -------------------------------------------------------------

fn env_usize(key: &str, default: usize) -> usize {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn env_u64(key: &str, default: u64) -> u64 {
    std::env::var(key)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn iters() -> usize {
    // Kept modest so the four stable CI jobs stay fast; crank locally via env.
    static N: OnceLock<usize> = OnceLock::new();
    *N.get_or_init(|| env_usize("ABIEOS_FUZZ_ITERS", 400))
}

fn seed() -> u64 {
    static S: OnceLock<u64> = OnceLock::new();
    *S.get_or_init(|| env_u64("ABIEOS_FUZZ_SEED", 0x0DDB_1A5E_5BAD_F00D))
}

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push_str(&format!("{b:02x}"));
    }
    s
}

/// Run `body` for every iteration, turning any panic into a reproducible
/// failure that names the seed, iteration and the input that triggered it.
/// The Rust backend never crosses an FFI boundary, so a Rust panic unwinds
/// normally and is safely catchable here.
fn fuzz<F>(name: &str, mut body: F)
where
    F: FnMut(&mut Rng, usize) -> Vec<u8>,
{
    let base = seed();
    let n = iters();
    for i in 0..n {
        // Per-iteration seed derived from the base seed so a single
        // (seed, iteration) pair fully determines the input.
        let mut rng = Rng::new(base ^ (i as u64).wrapping_mul(0x100_0000_01B3));
        let mut captured = Vec::new();
        let result = catch_unwind(AssertUnwindSafe(|| {
            captured = body(&mut rng, i);
        }));
        if let Err(payload) = result {
            let msg = payload
                .downcast_ref::<&str>()
                .map(|s| s.to_string())
                .or_else(|| payload.downcast_ref::<String>().cloned())
                .unwrap_or_else(|| "<non-string panic payload>".to_string());
            panic!(
                "{name} panicked at iteration {i}\n  reproduce: \
                 ABIEOS_FUZZ_SEED={base} ABIEOS_FUZZ_ITERS={} cargo test \
                 --no-default-features --features rust-backend --test \
                 rust_backend_fuzz_property\n  input(hex)={}\n  panic: {msg}",
                i + 1,
                hex(&captured),
            );
        }
    }
}

// --- Fuzz: never panic on malformed input --------------------------------

#[test]
fn fuzz_json_to_bin_no_panic() {
    fuzz("fuzz_json_to_bin_no_panic", |rng, _| {
        let json = rng.junk_string(256);
        let abieos = Abieos::new();
        // Random builtin/garbage type name, contract 0 (builtin namespace).
        let ty = pick_type_name(rng);
        let _ = abieos.json_to_hex_native(0, ty, &json);
        let _ = abieos.json_to_hex_native(0, ty, &json); // buffer-reuse path
        json.into_bytes()
    });
}

#[test]
fn fuzz_bin_to_json_no_panic() {
    fuzz("fuzz_bin_to_json_no_panic", |rng, _| {
        let bin = rng.bytes(256);
        let abieos = Abieos::new();
        let ty = pick_type_name(rng);
        let _ = abieos.bin_to_json("eosio", ty, &bin);
        let _ = abieos.hex_to_json_native(0, ty, &hex(&bin));
        bin
    });
}

#[test]
fn fuzz_abi_json_to_bin_no_panic() {
    fuzz("fuzz_abi_json_to_bin_no_panic", |rng, _| {
        let json = rng.junk_string(512);
        let abieos = Abieos::new();
        let _ = abieos.abi_json_to_bin(&json);
        json.into_bytes()
    });
}

#[test]
fn fuzz_abi_bin_to_json_no_panic() {
    fuzz("fuzz_abi_bin_to_json_no_panic", |rng, _| {
        let bin = rng.bytes(512);
        let abieos = Abieos::new();
        let _ = abieos.abi_bin_to_json(&bin);
        bin
    });
}

#[test]
fn fuzz_malformed_hex_no_panic() {
    fuzz("fuzz_malformed_hex_no_panic", |rng, _| {
        // Mix of odd lengths, non-hex chars, very long, empty.
        const CHARS: &[u8] = b"0123456789abcdefABCDEFxyzg!  \n";
        let len = rng.below(300);
        let s: String = (0..len)
            .map(|_| CHARS[rng.below(CHARS.len())] as char)
            .collect();
        let abieos = Abieos::new();
        let _ = abieos.hex_to_json_native(0, "uint32", &s);
        let _ = abieos.hex_to_json("eosio", "transaction", &s);
        s.into_bytes()
    });
}

#[test]
fn fuzz_malformed_key_signature_no_panic() {
    fuzz("fuzz_malformed_key_signature_no_panic", |rng, _| {
        const KEY_CHARS: &[u8] = b"PUBKEYSIGabcdefghijklmnopqrstuvwxyz123456789_ 0OIl+/=";
        let len = rng.below(120);
        let raw: String = (0..len)
            .map(|_| KEY_CHARS[rng.below(KEY_CHARS.len())] as char)
            .collect();
        let json = format!("\"{}\"", raw.replace('"', ""));
        let abieos = Abieos::new();
        for ty in ["public_key", "private_key", "signature"] {
            let _ = abieos.json_to_hex_native(0, ty, &json);
        }
        json.into_bytes()
    });
}

#[test]
fn fuzz_malformed_asset_time_no_panic() {
    fuzz("fuzz_malformed_asset_time_no_panic", |rng, _| {
        const CHARS: &[u8] = b"0123456789.,- :TZ+/ABCEOSY";
        let len = rng.below(40);
        let raw: String = (0..len)
            .map(|_| CHARS[rng.below(CHARS.len())] as char)
            .collect();
        let json = format!("\"{}\"", raw);
        let abieos = Abieos::new();
        for ty in [
            "asset",
            "symbol",
            "symbol_code",
            "time_point",
            "time_point_sec",
            "block_timestamp_type",
        ] {
            let _ = abieos.json_to_hex_native(0, ty, &json);
        }
        json.into_bytes()
    });
}

fn pick_type_name<'a>(rng: &mut Rng) -> &'a str {
    const TYPES: &[&str] = &[
        "bool",
        "int8",
        "uint8",
        "int16",
        "uint16",
        "int32",
        "uint32",
        "int64",
        "uint64",
        "int128",
        "uint128",
        "varuint32",
        "varint32",
        "float32",
        "float64",
        "float128",
        "time_point",
        "time_point_sec",
        "block_timestamp_type",
        "name",
        "bytes",
        "string",
        "checksum160",
        "checksum256",
        "checksum512",
        "symbol",
        "symbol_code",
        "asset",
        "public_key",
        "signature",
        "not_a_real_type",
        "",
    ];
    TYPES[rng.below(TYPES.len())]
}

// --- Property: round-trip stability --------------------------------------

/// Generate a JSON value that is *valid* for `ty` so the round-trip can be a
/// strict equality check.
fn valid_value(rng: &mut Rng, ty: &str) -> String {
    match ty {
        "bool" => if rng.bool() { "true" } else { "false" }.to_string(),
        "int8" => (rng.byte() as i8).to_string(),
        "uint8" => rng.byte().to_string(),
        "int16" => (rng.next_u64() as i16).to_string(),
        "uint16" => (rng.next_u64() as u16).to_string(),
        "int32" => (rng.next_u64() as i32).to_string(),
        "uint32" => (rng.next_u64() as u32).to_string(),
        // 64/128-bit integers are emitted as JSON strings by abieos.
        "int64" => format!("\"{}\"", rng.next_u64() as i64),
        "uint64" => format!("\"{}\"", rng.next_u64()),
        "int128" => format!("\"{}\"", rng.next_u64() as i64),
        "uint128" => format!("\"{}\"", rng.next_u64()),
        "name" => {
            // Safe charset/length so no normalization perturbs the round-trip.
            const C: &[u8] = b"abcdefghijklmnopqrstuvwxyz12345";
            let len = 1 + rng.below(12);
            let s: String = (0..len).map(|_| C[rng.below(C.len())] as char).collect();
            format!("\"{s}\"")
        }
        "string" => {
            const C: &[u8] = b"abcdefghijklmnopqrstuvwxyz0123456789 ";
            let len = rng.below(24);
            let mut s: String = (0..len).map(|_| C[rng.below(C.len())] as char).collect();
            if rng.bool() {
                s.push('\u{2603}'); // non-ASCII preservation
            }
            format!("\"{s}\"")
        }
        "bytes" => {
            let b = rng.bytes(24);
            format!("\"{}\"", hex(&b).to_uppercase())
        }
        "checksum256" => {
            let b = rng.bytes(32);
            let mut full = b.clone();
            full.resize(32, 0);
            format!("\"{}\"", hex(&full).to_uppercase())
        }
        "symbol" => format!("\"{},ABC\"", 1 + rng.below(8)),
        "symbol_code" => "\"ABC\"".to_string(),
        "asset" => format!("\"{}.0000 ABC\"", rng.below(100000)),
        _ => "0".to_string(),
    }
}

const ROUNDTRIP_TYPES: &[&str] = &[
    "bool",
    "int8",
    "uint8",
    "int16",
    "uint16",
    "int32",
    "uint32",
    "int64",
    "uint64",
    "int128",
    "uint128",
    "name",
    "string",
    "bytes",
    "checksum256",
    "symbol",
    "symbol_code",
    "asset",
];

#[test]
fn prop_roundtrip_json_bin_json() {
    let base = seed();
    let n = iters();
    for i in 0..n {
        let mut rng = Rng::new(base ^ (i as u64).wrapping_mul(0x9E37_79B1));
        let ty = ROUNDTRIP_TYPES[rng.below(ROUNDTRIP_TYPES.len())];
        let value = valid_value(&mut rng, ty);
        let abieos = Abieos::new();

        let bin0 = match abieos.json_to_hex_native(0, ty, &value) {
            Ok(b) => b,
            Err(e) => panic!(
                "valid value rejected (seed={base}, iter={i}): \
                 type={ty} value={value} err={e}"
            ),
        };
        let json1 = abieos
            .hex_to_json_native(0, ty, &bin0)
            .unwrap_or_else(|e| panic!("bin->json failed: type={ty} hex={bin0} err={e}"));
        // JSON -> bin -> JSON -> bin must be binary-stable (the canonical
        // invariant; JSON text may legitimately re-canonicalize).
        let bin1 = abieos
            .json_to_hex_native(0, ty, &json1)
            .unwrap_or_else(|e| panic!("re-encode failed: type={ty} json={json1} err={e}"));
        assert_eq!(
            bin0, bin1,
            "binary not stable (seed={base}, iter={i}): type={ty} \
             value={value} json1={json1}"
        );
        // bin -> JSON -> bin -> JSON must be JSON-stable once canonicalized.
        let json2 = abieos
            .hex_to_json_native(0, ty, &bin1)
            .unwrap_or_else(|e| panic!("second bin->json failed: type={ty} err={e}"));
        assert_eq!(
            json1, json2,
            "json not stable after canonicalization (seed={base}, iter={i}): \
             type={ty} value={value}"
        );
    }
}

#[test]
fn prop_roundtrip_abi_json_bin_json() {
    // Real ABI fixtures: ABI JSON -> bin -> JSON -> bin must be bin-stable,
    // and the recovered JSON must itself round-trip.
    const ABIS: &[(&str, &str)] = &[
        ("transaction", include_str!("../abis/transaction.abi.json")),
        ("testkv", include_str!("../abis/testkv.abi.json")),
        (
            "packed_transaction",
            include_str!("../abis/packed_transaction.abi.json"),
        ),
        ("ship", include_str!("../abis/ship.abi.json")),
    ];
    for (label, abi_json) in ABIS {
        let abieos = Abieos::new();
        let bin0 = abieos
            .abi_json_to_bin(abi_json)
            .unwrap_or_else(|e| panic!("abi_json_to_bin failed for {label}: {e}"));
        let json1 = abieos
            .abi_bin_to_json(&bin0)
            .unwrap_or_else(|e| panic!("abi_bin_to_json failed for {label}: {e}"));
        let bin1 = abieos
            .abi_json_to_bin(&json1)
            .unwrap_or_else(|e| panic!("abi_json_to_bin (2nd) failed for {label}: {e}"));
        assert_eq!(
            bin0, bin1,
            "ABI binary not stable across JSON round-trip for {label}"
        );
        let json2 = abieos
            .abi_bin_to_json(&bin1)
            .unwrap_or_else(|e| panic!("abi_bin_to_json (2nd) failed for {label}: {e}"));
        assert_eq!(json1, json2, "ABI JSON not stable for {label}");
    }
}

#[test]
fn prop_roundtrip_abi_bin_json_bin() {
    // Real binary ABI fixture: bin -> JSON -> bin must be bin-stable.
    let abi_bin: &[u8] = include_bytes!("../abis/eosio.abi.bin");
    let abieos = Abieos::new();
    let json = abieos
        .abi_bin_to_json(abi_bin)
        .expect("abi_bin_to_json failed for eosio.abi.bin");
    let bin = abieos
        .abi_json_to_bin(&json)
        .expect("abi_json_to_bin failed for recovered eosio JSON");
    let json2 = abieos
        .abi_bin_to_json(&bin)
        .expect("abi_bin_to_json (2nd) failed for eosio");
    assert_eq!(
        json, json2,
        "eosio ABI JSON not stable across bin round-trip"
    );
}

// --- Property: recursion limits fail gracefully --------------------------

#[test]
fn prop_recursion_limit_type_spec() {
    // The Rust backend bails type resolution at depth 32 with an error.
    // Deeply nested array/optional type specs must return Err, never abort
    // the process via stack overflow.
    let abieos = Abieos::new();
    for depth in [33usize, 64, 200, 1000] {
        let ty = format!("int8{}", "[]".repeat(depth));
        let r = abieos.json_to_hex_native(0, &ty, "[]");
        assert!(
            r.is_err(),
            "expected Err for {depth}-deep array type spec, got Ok"
        );
        let ty_opt = format!("int8{}", "?".repeat(depth));
        let r = abieos.json_to_hex_native(0, &ty_opt, "null");
        assert!(
            r.is_err(),
            "expected Err for {depth}-deep optional type spec, got Ok"
        );
    }
}

#[test]
fn prop_recursion_limit_json() {
    // The JSON parser bails at depth 128. A several-hundred-deep document
    // must return Err, not overflow the stack.
    let abieos = Abieos::new();
    for depth in [200usize, 1000, 5000] {
        let nested = format!("{}{}", "[".repeat(depth), "]".repeat(depth));
        let r = abieos.json_to_hex_native(0, "bytes", &nested);
        assert!(
            r.is_err(),
            "expected Err for {depth}-deep nested JSON array, got Ok"
        );
        // Also exercise the ABI JSON parser path.
        let r = abieos.abi_json_to_bin(&nested);
        assert!(
            r.is_err(),
            "expected Err for {depth}-deep nested ABI JSON, got Ok"
        );
    }
}

// --- Property: duplicate / out-of-order fields ---------------------------

const DUP_ABI: &str = r#"{
    "version": "eosio::abi/1.1",
    "structs": [
        {
            "name": "rec",
            "base": "",
            "fields": [
                {"name": "a", "type": "int32"},
                {"name": "b", "type": "string"},
                {"name": "c", "type": "uint8"}
            ]
        }
    ],
    "actions": [],
    "tables": []
}"#;

#[test]
fn prop_duplicate_and_reordered_fields() {
    let abieos = Abieos::new();
    abieos.set_abi_json("test", DUP_ABI).expect("load DUP_ABI");

    // Canonical, ordered, single-key form.
    let canonical = r#"{"a":7,"b":"hello","c":255}"#;
    let want = abieos
        .json_to_hex("test", "rec", canonical)
        .expect("canonical encode");

    let base = seed();
    let n = iters().min(200);
    for i in 0..n {
        let mut rng = Rng::new(base ^ (i as u64).wrapping_mul(0xA24B_AED4));

        // Shuffle the three keys.
        let mut keys = [("a", "7"), ("b", "\"hello\""), ("c", "255")];
        for k in (1..keys.len()).rev() {
            keys.swap(k, rng.below(k + 1));
        }

        // Inject decoy duplicate keys *before* the winning value; abieos uses
        // last-wins (std::map overwrite) semantics, so the final value wins.
        let mut parts: Vec<String> = Vec::new();
        for (name, val) in keys {
            if rng.bool() {
                let decoy = match name {
                    "a" => "-1",
                    "b" => "\"WRONG\"",
                    _ => "0",
                };
                parts.push(format!("\"{name}\":{decoy}"));
            }
            parts.push(format!("\"{name}\":{val}"));
        }
        let obj = format!("{{{}}}", parts.join(","));

        let got = abieos.json_to_hex("test", "rec", &obj).unwrap_or_else(|e| {
            panic!(
                "duplicate/reordered encode failed (seed={base}, iter={i}): \
                 obj={obj} err={e}"
            )
        });
        assert_eq!(
            got, want,
            "last-wins / order-independence violated (seed={base}, iter={i}): \
             obj={obj}"
        );
    }
}

// --- Regression: unbounded allocation from a crafted ABI length ----------

/// Deterministic guard for the memory-exhaustion abort found by
/// `fuzz_abi_bin_to_json_no_panic`: an ABI binary whose first vector
/// (`types`) declares a ~4.29e9 element count made the parser
/// `Vec::with_capacity(count)` and request ~182 GiB, aborting the process
/// before any data was validated.
///
/// Layout: `version` istr (varuint length + bytes) then the `types`
/// vector (varuint count + entries). `0x00` is an empty version string;
/// `FF FF FF FF 0F` is the varuint32 encoding of `u32::MAX`. Both forms must
/// now return `Err` promptly instead of aborting.
#[test]
fn regression_abi_bin_unbounded_alloc() {
    let abieos = Abieos::new();

    // Empty version, then a u32::MAX `types` count with no element bytes.
    let crafted = [0x00, 0xFF, 0xFF, 0xFF, 0xFF, 0x0F];
    assert!(
        abieos.abi_bin_to_json(&crafted).is_err(),
        "crafted huge ABI types count must error, not abort"
    );

    // Same attack behind a realistic version string.
    let version = b"eosio::abi/1.1";
    let mut crafted2 = vec![version.len() as u8];
    crafted2.extend_from_slice(version);
    crafted2.extend_from_slice(&[0xFF, 0xFF, 0xFF, 0xFF, 0x0F]);
    assert!(
        abieos.abi_bin_to_json(&crafted2).is_err(),
        "crafted huge ABI types count (versioned) must error, not abort"
    );
}
