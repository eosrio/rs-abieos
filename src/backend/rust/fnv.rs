//! Dependency-free FNV-1a hasher.
//!
//! The Rust backend deliberately pulls in no external crates, but the standard
//! library's default `HashMap` hasher (SipHash) is slow for the very short
//! string keys (`"uint64"`, `"name"`, field names, …) and `u64` keys this
//! backend hashes constantly during ABI resolution and the codec hot path.
//! FNV-1a is tiny, allocation-free, and dramatically faster for these keys.
//!
//! `str`/`Arc<str>` and `&str` hash to the same bytes (both forward to
//! `str::hash`), so `FnvMap<Arc<str>, _>` supports `&str` lookups.

use std::collections::HashMap;
use std::hash::{BuildHasher, Hasher};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

pub(crate) struct FnvHasher(u64);

impl Default for FnvHasher {
    #[inline]
    fn default() -> Self {
        FnvHasher(FNV_OFFSET)
    }
}

impl Hasher for FnvHasher {
    #[inline]
    fn finish(&self) -> u64 {
        self.0
    }

    #[inline]
    fn write(&mut self, bytes: &[u8]) {
        let mut hash = self.0;
        for &b in bytes {
            hash ^= b as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        self.0 = hash;
    }
}

#[derive(Clone, Default)]
pub(crate) struct FnvBuildHasher;

impl BuildHasher for FnvBuildHasher {
    type Hasher = FnvHasher;
    #[inline]
    fn build_hasher(&self) -> FnvHasher {
        FnvHasher::default()
    }
}

pub(crate) type FnvMap<K, V> = HashMap<K, V, FnvBuildHasher>;

#[inline]
pub(crate) fn fnv_map_with_capacity<K, V>(cap: usize) -> FnvMap<K, V> {
    HashMap::with_capacity_and_hasher(cap, FnvBuildHasher)
}
