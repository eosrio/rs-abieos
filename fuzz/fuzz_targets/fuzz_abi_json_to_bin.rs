#![no_main]
//! Fuzz `abi_json_to_bin` (ABI JSON -> ABI binary). The whole input is the
//! ABI JSON document. Must never panic/abort regardless of how malformed.

use libfuzzer_sys::fuzz_target;
use rs_abieos::Abieos;

fuzz_target!(|data: &[u8]| {
    let json = String::from_utf8_lossy(data);
    let abieos = Abieos::new();
    let _ = abieos.abi_json_to_bin(&json);
});
