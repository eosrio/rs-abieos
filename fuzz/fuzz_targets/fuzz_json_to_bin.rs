#![no_main]
//! Fuzz `json_to_hex` (value JSON -> binary). The first input byte selects a
//! builtin type; the rest is treated as the JSON value. The invariant is
//! "never panic / abort" — the call must always return a `Result`.

use libfuzzer_sys::fuzz_target;
use rs_abieos::Abieos;

const TYPES: &[&str] = &[
    "bool", "int8", "uint8", "int16", "uint16", "int32", "uint32", "int64",
    "uint64", "int128", "uint128", "varuint32", "varint32", "float32",
    "float64", "float128", "time_point", "time_point_sec",
    "block_timestamp_type", "name", "bytes", "string", "checksum160",
    "checksum256", "checksum512", "symbol", "symbol_code", "asset",
    "public_key", "private_key", "signature",
];

fuzz_target!(|data: &[u8]| {
    let (ty, json) = match data.split_first() {
        Some((sel, rest)) => (TYPES[*sel as usize % TYPES.len()], rest),
        None => return,
    };
    let json = String::from_utf8_lossy(json);
    let abieos = Abieos::new();
    let _ = abieos.json_to_hex_native(0, ty, &json);
});
