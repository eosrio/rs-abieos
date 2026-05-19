//! SWAR (SIMD-Within-A-Register) byte scanning helpers.
//!
//! The JSON tokenizer is throughput-bound on large pretty-printed ABIs
//! (`eosio.abi` is ~77 KB, heavily indented, thousands of short strings).
//! Scanning one byte at a time loses to rapidjson's vectorized scan. These
//! classic bit-twiddling primitives test all 8 bytes of a `u64` at once with
//! no SIMD intrinsics, no `unsafe`, and no dependencies — so whitespace runs
//! and string bodies advance 8 bytes per iteration.
//!
//! Formulas are the well-known "bit twiddling hacks" identities, valid for
//! the byte values used here (`hasless` requires `1 ..= 128`). They only
//! locate the same stop byte the scalar loop would; parser semantics and
//! error behavior are unchanged.

pub(crate) const LO: u64 = 0x0101_0101_0101_0101;
pub(crate) const HI: u64 = 0x8080_8080_8080_8080;

/// Read 8 bytes at `off` as a `u64`. Byte order is irrelevant: every helper
/// is lane-symmetric, so native-endian load is correct and fastest.
#[inline]
pub(crate) fn word_at(src: &[u8], off: usize) -> u64 {
    let mut b = [0u8; 8];
    b.copy_from_slice(&src[off..off + 8]);
    u64::from_ne_bytes(b)
}

/// Nonzero iff any byte of `v` is zero (per-lane `0x80` flag).
#[inline]
pub(crate) fn haszero(v: u64) -> u64 {
    v.wrapping_sub(LO) & !v & HI
}

/// Nonzero iff any byte of `x` equals `n`.
#[inline]
pub(crate) fn hasvalue(x: u64, n: u8) -> u64 {
    haszero(x ^ (LO.wrapping_mul(n as u64)))
}

/// Nonzero iff any byte of `x` is `< n` (requires `1 <= n <= 128`).
#[inline]
pub(crate) fn hasless(x: u64, n: u8) -> u64 {
    x.wrapping_sub(LO.wrapping_mul(n as u64)) & !x & HI
}
