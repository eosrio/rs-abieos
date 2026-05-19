#![no_main]
//! Fuzz `abi_bin_to_json` (ABI binary -> ABI JSON). The whole input is the
//! ABI binary. This target found the unbounded-allocation abort fixed in
//! `read_vec` (a crafted vector length requesting ~182 GiB); it must never
//! panic/abort and must reject malformed length fields promptly.

use libfuzzer_sys::fuzz_target;
use rs_abieos::Abieos;

fuzz_target!(|data: &[u8]| {
    let abieos = Abieos::new();
    let _ = abieos.abi_bin_to_json(data);
});
