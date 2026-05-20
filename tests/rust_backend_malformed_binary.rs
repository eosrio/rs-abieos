#[cfg(feature = "rust-backend")]
mod rust_backend_malformed_binary {
    use rs_abieos::Abieos;

    const TEST_ABI_CONTRACT: &str = "test.abi";

    const TEST_ABI: &str = r#"{
        "version": "eosio::abi/1.1",
        "structs": [
            {
                "name": "s_array",
                "fields": [
                    {"name": "v", "type": "int8[]"}
                ]
            },
            {
                "name": "s_fixed_array",
                "fields": [
                    {"name": "v", "type": "int8[2]"}
                ]
            },
            {
                "name": "s_extension",
                "fields": [
                    {"name": "v1", "type": "int8"},
                    {"name": "v2", "type": "int16$"}
                ]
            },
            {
                "name": "s_extension_string",
                "fields": [
                    {"name": "v1", "type": "int8"},
                    {"name": "v2", "type": "string$"}
                ]
            },
            {
                "name": "s_variant",
                "fields": [
                    {"name": "v", "type": "v1"}
                ]
            },
            {
                "name": "s_array_variant",
                "fields": [
                    {"name": "v", "type": "v1[]"}
                ]
            },
            {
                "name": "s_string",
                "fields": [
                    {"name": "v", "type": "string"}
                ]
            }
        ],
        "variants": [
            {
                "name": "v1",
                "types": ["int8", "int16"]
            }
        ]
    }"#;

    fn abieos_with_test_abi() -> Abieos {
        let abieos = Abieos::new();
        abieos.set_abi_json(TEST_ABI_CONTRACT, TEST_ABI).unwrap();
        abieos
    }

    fn check_error(abieos: &Abieos, ty: &str, hex: &str, expected_err: &str) {
        let result = abieos.hex_to_json(TEST_ABI_CONTRACT, ty, hex);
        match result {
            Ok(json) => panic!(
                "Expected error for type {} with hex {}, but got success: {}",
                ty, hex, json
            ),
            Err(e) => {
                let err_msg = e.to_string();
                assert!(
                    err_msg.contains(expected_err),
                    "Error message {:?} did not contain {:?}",
                    err_msg,
                    expected_err
                );
            }
        }
    }

    #[test]
    fn test_malformed_array() {
        let abieos = abieos_with_test_abi();
        // Array of int8.
        // Valid: [] -> 00
        // Valid: [1, 2] -> 020102

        // Truncated length (varuint32)
        // 0x80 is first byte of a 2-byte varuint32, but it's missing the second byte.
        check_error(&abieos, "s_array", "80", "read datastream");

        // Truncated data
        // Length 2, but only 1 byte provided
        check_error(&abieos, "s_array", "0201", "read datastream");
    }

    #[test]
    fn test_malformed_fixed_array() {
        let abieos = abieos_with_test_abi();
        // Fixed array [2] of int8.
        // Valid: [1, 2] -> 0102

        // Truncated data
        check_error(&abieos, "s_fixed_array", "01", "read datastream");
        check_error(&abieos, "s_fixed_array", "", "read datastream");
    }

    #[test]
    fn test_malformed_variant() {
        let abieos = abieos_with_test_abi();
        // Variant v1: ["int8", "int16"]
        // Valid index 0 (int8): 00 01 -> [ "int8", 1 ]
        // Valid index 1 (int16): 01 0100 -> [ "int16", 1 ]

        // Truncated index (varuint32)
        check_error(&abieos, "s_variant", "80", "read datastream");

        // Bad index
        check_error(&abieos, "s_variant", "02", "bad variant index");

        // Truncated value (index 1 is int16, needs 2 bytes)
        check_error(&abieos, "s_variant", "0101", "read datastream");
    }

    #[test]
    fn test_malformed_extension() {
        let abieos = abieos_with_test_abi();
        // s_extension: {v1: int8, v2: int16$}
        // Valid: v1=1, v2 missing -> 01
        // Valid: v1=1, v2=2 -> 010200

        // Truncated v1
        check_error(&abieos, "s_extension", "", "read datastream");

        // Truncated v2 (extension data exists but is incomplete)
        check_error(&abieos, "s_extension", "0102", "read datastream");

        // s_extension_string: {v1: int8, v2: string$}
        // Truncated string length
        check_error(&abieos, "s_extension_string", "0180", "read datastream");
        // Truncated string data (length 5, only 2 bytes)
        check_error(&abieos, "s_extension_string", "01056162", "read datastream");
    }

    #[test]
    fn test_deeply_nested_malformed() {
        let abieos = abieos_with_test_abi();
        // Array of variants
        // [ {v: ["int8", 1]}, {v: ["int16", 256]} ]
        // hex: 02 (length 2)
        //   00 (variant index 0) 01 (int8 value 1)
        //   01 (variant index 1) 0001 (int16 value 256)
        // Result: 020001010001

        // Truncate at second element's variant index
        check_error(&abieos, "s_array_variant", "020001", "read datastream");

        // Truncate at second element's value
        check_error(&abieos, "s_array_variant", "0200010100", "read datastream");
    }

    #[test]
    fn test_malformed_varuint32() {
        let abieos = abieos_with_test_abi();
        // varuint32 is used for array lengths and variant indices.
        // A varuint32 with too many bytes (more than 5 bytes / 35 bits) is invalid.
        // 808080808001 -> 5 bytes of 0x80 (128) and one 0x01. Total 6 bytes.
        check_error(
            &abieos,
            "s_array",
            "808080808001",
            "invalid variable-length unsigned integer",
        );
    }

    #[test]
    fn test_malformed_string() {
        let abieos = abieos_with_test_abi();
        // Valid string "abc": 03 616263
        // Invalid UTF-8: 01 FF
        check_error(&abieos, "s_string", "01FF", "Invalid encoding in string");
    }
}
