#![no_main]
//! Fuzz `bin_to_json` / `hex_to_json` (binary -> value JSON). The first input
//! byte selects a builtin type decoded from contract 0; the remaining bytes
//! are the binary payload. Also exercises a real loaded ABI type so compound
//! decoding (arrays/structs/variants) is reached. Must never panic/abort.

use libfuzzer_sys::fuzz_target;
use rs_abieos::{AbiLike, Abieos, NameLike};

const TYPES: &[&str] = &[
    "bool", "int8", "uint8", "int16", "uint16", "int32", "uint32", "int64",
    "uint64", "int128", "uint128", "varuint32", "varint32", "float32",
    "float64", "float128", "time_point", "time_point_sec",
    "block_timestamp_type", "name", "bytes", "string", "checksum160",
    "checksum256", "checksum512", "symbol", "symbol_code", "asset",
    "public_key", "private_key", "signature",
];

const TX_ABI: &str = include_str!("../../abis/transaction.abi.json");

fuzz_target!(|data: &[u8]| {
    let (ty, payload) = match data.split_first() {
        Some((sel, rest)) => (TYPES[*sel as usize % TYPES.len()], rest),
        None => return,
    };
    let abieos = Abieos::new();
    let _ = abieos.bin_to_json("eosio", ty, payload);

    // Compound path through a real ABI.
    let mut c = abieos.contract(NameLike::StringRef("eosio"));
    if c.load_abi(AbiLike::Json(TX_ABI.to_string())).is_ok() {
        let _ = c.hex_to_json("transaction", &hex(payload));
    }
});

fn hex(bytes: &[u8]) -> String {
    let mut s = String::with_capacity(bytes.len() * 2);
    for b in bytes {
        s.push(char::from_digit((b >> 4) as u32, 16).unwrap());
        s.push(char::from_digit((b & 0xf) as u32, 16).unwrap());
    }
    s
}
