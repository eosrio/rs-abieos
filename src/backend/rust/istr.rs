//! Small-string-optimized immutable string.
//!
//! ABI documents are almost entirely short identifiers (`"name"`, `"uint64"`,
//! `"account_name"`, field names …). The previous `Arc<str>` representation
//! heap-allocated on *every* such string during ABI parsing/resolution, which
//! was the entire remaining performance gap versus C++ — whose `std::string`
//! short-string optimization stores ≤15 bytes inline with no allocation.
//!
//! `IStr` mirrors that: strings up to [`INLINE_CAP`] bytes live inline (clone
//! is a `memcpy`, no atomics, no heap), longer ones fall back to `Arc<str>`
//! (shared, cheap clone). It is a drop-in for the old `Arc<str>`: it derefs to
//! `str`, and hashes/orders/compares exactly as `str`/`&str` do, so
//! `FnvMap<IStr, _>` still supports `&str` lookups.
//!
//! Dependency-free by design (the Rust backend pulls in no crates).

use std::borrow::Borrow;
use std::cmp::Ordering;
use std::fmt;
use std::hash::{Hash, Hasher};
use std::ops::Deref;
use std::sync::Arc;

/// Max bytes stored inline. Covers effectively every ABI type/field name
/// (`"block_timestamp_type"` = 20, `"permission_level"` = 16, …); longer
/// values (ricardian text, some versions) spill to the heap.
pub(crate) const INLINE_CAP: usize = 22;

#[derive(Clone)]
pub(crate) enum IStr {
    Inline { len: u8, buf: [u8; INLINE_CAP] },
    Heap(Arc<str>),
}

impl IStr {
    #[inline]
    pub(crate) fn as_str(&self) -> &str {
        match self {
            // SAFETY: every constructor copies bytes from a valid `&str`, so
            // the inline region `[..len]` is always valid UTF-8.
            IStr::Inline { len, buf } => unsafe {
                std::str::from_utf8_unchecked(&buf[..*len as usize])
            },
            IStr::Heap(s) => s,
        }
    }
}

impl From<&str> for IStr {
    #[inline]
    fn from(s: &str) -> Self {
        let bytes = s.as_bytes();
        if bytes.len() <= INLINE_CAP {
            let mut buf = [0u8; INLINE_CAP];
            buf[..bytes.len()].copy_from_slice(bytes);
            IStr::Inline {
                len: bytes.len() as u8,
                buf,
            }
        } else {
            IStr::Heap(Arc::from(s))
        }
    }
}

impl Default for IStr {
    #[inline]
    fn default() -> Self {
        IStr::Inline {
            len: 0,
            buf: [0u8; INLINE_CAP],
        }
    }
}

impl Deref for IStr {
    type Target = str;
    #[inline]
    fn deref(&self) -> &str {
        self.as_str()
    }
}

impl AsRef<str> for IStr {
    #[inline]
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}

impl Borrow<str> for IStr {
    #[inline]
    fn borrow(&self) -> &str {
        self.as_str()
    }
}

impl Hash for IStr {
    #[inline]
    fn hash<H: Hasher>(&self, state: &mut H) {
        // Identical to `<str as Hash>` / `<Arc<str> as Hash>`, so `&str`
        // lookups against `FnvMap<IStr, _>` hash consistently.
        self.as_str().hash(state)
    }
}

impl PartialEq for IStr {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.as_str() == other.as_str()
    }
}
impl Eq for IStr {}

impl PartialEq<str> for IStr {
    #[inline]
    fn eq(&self, other: &str) -> bool {
        self.as_str() == other
    }
}

impl PartialOrd for IStr {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for IStr {
    #[inline]
    fn cmp(&self, other: &Self) -> Ordering {
        self.as_str().cmp(other.as_str())
    }
}

impl fmt::Display for IStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.as_str())
    }
}

impl fmt::Debug for IStr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(self.as_str(), f)
    }
}
