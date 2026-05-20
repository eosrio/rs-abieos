#[path = "common/extension_nesting_fixtures.rs"]
mod extension_nesting_fixtures;

#[cfg(feature = "rust-backend")]
mod rust_backend_extension_nesting {
    use super::extension_nesting_fixtures::{EXTENSION_NESTING_ABI, EXTENSION_NESTING_CASES};
    use rs_abieos::Abieos;

    #[test]
    fn test_binary_extension_skipping_nesting() {
        let abieos = Abieos::new();
        abieos
            .set_abi_json_native(0, EXTENSION_NESTING_ABI)
            .expect("Failed to set ABI");

        for case in EXTENSION_NESTING_CASES {
            let hex = abieos
                .json_to_hex_native(0, case.ty, case.json)
                .unwrap_or_else(|err| {
                    panic!(
                        "json_to_hex_native failed for {} ({}) and json {}: {}",
                        case.label, case.ty, case.json, err
                    )
                });
            let actual = abieos
                .hex_to_json_native(0, case.ty, &hex)
                .unwrap_or_else(|err| {
                    panic!(
                        "hex_to_json_native failed for {} ({}) and json {} (hex {}): {}",
                        case.label, case.ty, case.json, hex, err
                    )
                });
            assert_eq!(
                actual, case.expected_json,
                "round-trip JSON mismatch for {} ({}) and input {} (hex {})",
                case.label, case.ty, case.json, hex
            );
        }
    }
}
