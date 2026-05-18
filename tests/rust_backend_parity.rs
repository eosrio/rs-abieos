#[cfg(feature = "rust-backend")]
mod rust_backend_parity {
    use rs_abieos::Abieos;

    const TEST_ABI: &str = r#"{
        "version": "eosio::abi/1.1",
        "structs": [
            {
                "name": "s1",
                "fields": [
                    {"name": "x1", "type": "int8"}
                ]
            },
            {
                "name": "s2",
                "fields": [
                    {"name": "y1", "type": "int8$"},
                    {"name": "y2", "type": "int8$"}
                ]
            },
            {
                "name": "s3",
                "fields": [
                    {"name": "z1", "type": "int8$"},
                    {"name": "z2", "type": "v1$"},
                    {"name": "z3", "type": "s2$"}
                ]
            },
            {
                "name": "s4",
                "fields": [
                    {"name": "a1", "type": "int8?$"},
                    {"name": "b1", "type": "int8[]$"}
                ]
            },
            {
                "name": "s5",
                "fields": [
                    {"name": "x1", "type": "int8"},
                    {"name": "x2", "type": "int8"},
                    {"name": "x3", "type": "s6"}
                ]
            },
            {
                "name": "s6",
                "fields": [
                    {"name": "c1", "type": "int8"},
                    {"name": "c2", "type": "s5[]"},
                    {"name": "c3", "type": "int8"}
                ]
            },
            {
                "name": "s7",
                "fields": [
                    {"name": "bs", "type": "bitset"}
                ]
            },
            {
                "name": "s8",
                "fields": [
                    {"name": "a1", "type": "int8[2]"}
                ]
            },
            {
                "name": "s9",
                "fields": [
                    {"name": "a1", "type": "s1[2]"}
                ]
            },
            {
                "name": "public_key_holder",
                "fields": [
                    {"name": "key", "type": "public_key"}
                ]
            },
            {
                "name": "private_key_holder",
                "fields": [
                    {"name": "key", "type": "private_key"}
                ]
            },
            {
                "name": "signature_holder",
                "fields": [
                    {"name": "sig", "type": "signature"}
                ]
            }
        ],
        "variants": [
            {
                "name": "v1",
                "types": ["int8", "s1", "s2", "s7"]
            }
        ]
    }"#;

    #[test]
    fn rust_backend_handles_fixed_array_variant_and_bitset() {
        let abieos = Abieos::new();
        abieos.set_abi_json("test.abi", TEST_ABI).unwrap();

        let fixed = abieos
            .json_to_hex("test.abi", "s8", r#"{"a1":[1,27]}"#)
            .unwrap();
        assert_eq!(fixed, "011B");
        let fixed_json = abieos.hex_to_json("test.abi", "s8", &fixed).unwrap();
        assert_eq!(fixed_json, r#"{"a1":[1,27]}"#);

        let variant = abieos
            .json_to_hex("test.abi", "v1", r#"["int8",7]"#)
            .unwrap();
        assert_eq!(variant, "0007");

        let struct_variant = abieos
            .json_to_hex("test.abi", "v1", r#"["s1",{"x1":6}]"#)
            .unwrap();
        assert_eq!(struct_variant, "0106");

        let struct_fixed = abieos
            .json_to_hex("test.abi", "s9", r#"{"a1":[{"x1":6},{"x1":16}]}"#)
            .unwrap();
        assert_eq!(struct_fixed, "0610");

        let bitset = abieos
            .json_to_hex("test.abi", "s7", r#"{"bs":"110001011"}"#)
            .unwrap();
        assert_eq!(bitset, "098B01");
        let bitset_json = abieos.hex_to_json("test.abi", "s7", &bitset).unwrap();
        assert_eq!(bitset_json, r#"{"bs":"110001011"}"#);
    }

    #[test]
    fn rust_backend_handles_cpp_binary_extension_fixtures() {
        let abieos = Abieos::new();
        abieos.set_abi_json("test.abi", TEST_ABI).unwrap();

        for (ty, json, expected) in [
            ("s3", r#"{}"#, r#"{}"#),
            ("s3", r#"{"z1":7}"#, r#"{"z1":7}"#),
            (
                "s3",
                r#"{"z1":7,"z2":["int8",6]}"#,
                r#"{"z1":7,"z2":["int8",6]}"#,
            ),
            (
                "s3",
                r#"{"z1":7,"z2":["int8",6],"z3":{}}"#,
                r#"{"z1":7,"z2":["int8",6]}"#,
            ),
            (
                "s3",
                r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9}}"#,
                r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9}}"#,
            ),
            (
                "s3",
                r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9,"y2":10}}"#,
                r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9,"y2":10}}"#,
            ),
            ("s4", r#"{}"#, r#"{}"#),
            ("s4", r#"{"a1":null}"#, r#"{"a1":null}"#),
            ("s4", r#"{"a1":7}"#, r#"{"a1":7}"#),
            ("s4", r#"{"a1":null,"b1":[]}"#, r#"{"a1":null,"b1":[]}"#),
            (
                "s4",
                r#"{"a1":null,"b1":[5,6,7]}"#,
                r#"{"a1":null,"b1":[5,6,7]}"#,
            ),
        ] {
            let hex = abieos
                .json_to_hex("test.abi", ty, json)
                .unwrap_or_else(|e| panic!("json_to_hex failed for {ty} {json}: {e}"));
            let actual = abieos
                .hex_to_json("test.abi", ty, &hex)
                .unwrap_or_else(|e| panic!("hex_to_json failed for {ty} {json} ({hex}): {e}"));
            assert_eq!(actual, expected, "extension round-trip mismatch for {ty}");
        }

        assert!(
            abieos
                .json_to_hex("test.abi", "s8", r#"{"a1":[1]}"#)
                .is_err(),
            "fixed arrays should reject short input"
        );
        assert!(
            abieos
                .json_to_hex("test.abi", "s8", r#"{"a1":[1,2,3]}"#)
                .is_err(),
            "fixed arrays should reject long input"
        );
    }

    #[test]
    fn rust_backend_matches_cpp_key_string_canonicalization() {
        let abieos = Abieos::new();
        abieos.set_abi_json("test.abi", TEST_ABI).unwrap();

        let public_hex = abieos
            .json_to_hex(
                "test.abi",
                "public_key_holder",
                r#"{"key":"EOS1111111111111111111111111111111114T1Anm"}"#,
            )
            .unwrap();
        let public_json = abieos
            .hex_to_json("test.abi", "public_key_holder", &public_hex)
            .unwrap();
        assert_eq!(
            public_json,
            r#"{"key":"PUB_K1_11111111111111111111111111111111149Mr2R"}"#
        );

        let private_hex = abieos
            .json_to_hex(
                "test.abi",
                "private_key_holder",
                r#"{"key":"5KQwrPbwdL6PhXujxW37FSSQZ1JiwsST4cqQzDeyXtP79zkvFD3"}"#,
            )
            .unwrap();
        let private_json = abieos
            .hex_to_json("test.abi", "private_key_holder", &private_hex)
            .unwrap();
        assert_eq!(
            private_json,
            r#"{"key":"PVT_K1_2bfGi9rYsXQSXXTvJbDAPhHLQUojjaNLomdm3cEJ1XTzMqUt3V"}"#
        );

        let signature = "SIG_K1_Kg2UKjXTX48gw2wWH4zmsZmWu3yarcfC21Bd9JPj7QoDURqiAacCHmtExPk3syPb2tFLsp1R4ttXLXgr7FYgDvKPC5RCkx";
        let signature_json = format!(r#"{{"sig":"{}"}}"#, signature);
        let signature_hex = abieos
            .json_to_hex("test.abi", "signature_holder", &signature_json)
            .unwrap();
        let signature_back = abieos
            .hex_to_json("test.abi", "signature_holder", &signature_hex)
            .unwrap();
        assert_eq!(signature_back, signature_json);

        let invalid = abieos.json_to_hex(
            "test.abi",
            "public_key_holder",
            r#"{"key":"PUB_K1_11111111111111111111111111111111149Mr2S"}"#,
        );
        assert!(invalid.is_err(), "bad key checksum should be rejected");
    }

    #[test]
    fn rust_backend_handles_cpp_builtin_contract_scalar_fixtures() {
        let abieos = Abieos::new();

        for (ty, json, expected) in [
            ("bool", "true", "true"),
            ("bool", "false", "false"),
            ("int8", "-128", "-128"),
            ("int8", "127", "127"),
            ("uint8", "255", "255"),
            ("int16", "-32768", "-32768"),
            ("uint16", "65535", "65535"),
            ("int32", "-2147483648", "-2147483648"),
            ("uint32", "4294967295", "4294967295"),
            ("int64", "-9223372036854775808", r#""-9223372036854775808""#),
            (
                "uint64",
                "18446744073709551615",
                r#""18446744073709551615""#,
            ),
            (
                "int128",
                r#""170141183460469231731687303715884105727""#,
                r#""170141183460469231731687303715884105727""#,
            ),
            (
                "uint128",
                r#""340282366920938463463374607431768211455""#,
                r#""340282366920938463463374607431768211455""#,
            ),
            ("varuint32", "4294967295", "4294967295"),
            ("varint32", "-2147483648", "-2147483648"),
            ("float32", "0.125", "0.125"),
            ("float64", "-0.125", "-0.125"),
            (
                "float128",
                r#""12345678ABCDEF12345678ABCDEF1234""#,
                r#""12345678ABCDEF12345678ABCDEF1234""#,
            ),
            (
                "time_point_sec",
                r#""2018-06-15T19:17:47.000""#,
                r#""2018-06-15T19:17:47.000""#,
            ),
            (
                "time_point",
                r#""2000-12-31T23:59:59.999999""#,
                r#""2000-12-31T23:59:59.999""#,
            ),
            (
                "block_timestamp_type",
                r#""2000-01-01T00:00:00.500""#,
                r#""2000-01-01T00:00:00.500""#,
            ),
            ("name", r#""..ab.cd.ef..""#, r#""..ab.cd.ef""#),
            (
                "bytes",
                r#""AABBCCDDEEFF00010203040506070809""#,
                r#""AABBCCDDEEFF00010203040506070809""#,
            ),
            (
                "checksum160",
                r#""123456789ABCDEF01234567890ABCDEF70123456""#,
                r#""123456789ABCDEF01234567890ABCDEF70123456""#,
            ),
            (
                "checksum256",
                r#""0987654321ABCDEF0987654321FFFF1234567890ABCDEF001234567890ABCDEF""#,
                r#""0987654321ABCDEF0987654321FFFF1234567890ABCDEF001234567890ABCDEF""#,
            ),
            ("symbol_code", r#""SYS""#, r#""SYS""#),
            ("symbol", r#""4,SYS""#, r#""4,SYS""#),
            ("asset", r#""-1.2345 SYS""#, r#""-1.2345 SYS""#),
            (
                "asset[]",
                r#"["0 FOO","0.000 FOO"]"#,
                r#"["0 FOO","0.000 FOO"]"#,
            ),
            (
                "asset[2]",
                r#"["0 FOO","0.000 FOO"]"#,
                r#"["0 FOO","0.000 FOO"]"#,
            ),
            ("asset?", "null", "null"),
            ("asset?", r#""0.123456 SIX""#, r#""0.123456 SIX""#),
            (
                "extended_asset",
                r#"{"quantity":"0.123456 SIX","contract":"seven"}"#,
                r#"{"quantity":"0.123456 SIX","contract":"seven"}"#,
            ),
        ] {
            let hex = abieos
                .json_to_hex_native(0, ty, json)
                .unwrap_or_else(|e| panic!("json_to_hex_native failed for {ty} {json}: {e}"));
            let actual = abieos.hex_to_json_native(0, ty, &hex).unwrap_or_else(|e| {
                panic!("hex_to_json_native failed for {ty} {json} ({hex}): {e}")
            });
            assert_eq!(actual, expected, "round-trip mismatch for {ty} {json}");
        }

        for (ty, json) in [
            ("uint8", "256"),
            ("int8", "-129"),
            ("varuint32", "-1"),
            ("bytes", r#""0""#),
            ("checksum256", r#""a0""#),
            ("symbol_code", r#""lower""#),
            ("asset", "null"),
        ] {
            assert!(
                abieos.json_to_hex_native(0, ty, json).is_err(),
                "{ty} {json} should be rejected"
            );
        }
    }

    #[test]
    fn rust_backend_handles_cpp_nested_array_and_bitset_fixtures() {
        let abieos = Abieos::new();

        for (ty, json) in [
            ("string[]", r#"["hello","world"]"#),
            ("string[][]", r#"[["A"],["B"],["C","D"]]"#),
            ("uint8[]", r#"[10,9,8]"#),
            ("uint8[3]", r#"[10,9,8]"#),
            ("uint8[][]", r#"[[1]]"#),
            ("uint8[][][]", r#"[[[1,2,3],[4,5,6]],[[7,8,9],[]]]"#),
            ("bitset", r#""""#),
            ("bitset", r#""0""#),
            ("bitset", r#""11""#),
            ("bitset", r#""011""#),
            ("bitset", r#""1100010110110""#),
            ("bitset", r#""110001011011000110101011101001100110000110""#),
            (
                "bitset",
                r#""110001011011000110101011101001100110000111111111111111111110""#,
            ),
        ] {
            let hex = abieos
                .json_to_hex_native(0, ty, json)
                .unwrap_or_else(|e| panic!("json_to_hex_native failed for {ty} {json}: {e}"));
            let actual = abieos.hex_to_json_native(0, ty, &hex).unwrap_or_else(|e| {
                panic!("hex_to_json_native failed for {ty} {json} ({hex}): {e}")
            });
            assert_eq!(actual, json, "round-trip mismatch for {ty} {json}");
        }
    }

    #[test]
    fn rust_backend_prefers_loaded_contract_zero_abi_over_builtin_namespace() {
        let abieos = Abieos::new();
        let transaction_abi = include_str!("../abis/transaction.abi.json");
        abieos.set_abi_json_native(0, transaction_abi).unwrap();

        let ordered = r#"{"expiration":"2009-02-13T23:31:31.000","ref_block_num":1234,"ref_block_prefix":5678,"max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}"#;
        let unordered = r#"{"ref_block_num":1234,"ref_block_prefix":5678,"expiration":"2009-02-13T23:31:31.000","max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}"#;

        let ordered_hex = abieos
            .json_to_hex_native(0, "transaction", ordered)
            .unwrap();
        let unordered_hex = abieos
            .json_to_hex_native(0, "transaction", unordered)
            .unwrap();
        assert_eq!(
            ordered_hex, unordered_hex,
            "reorderable JSON should pack to the same transaction bytes"
        );

        let json = abieos
            .hex_to_json_native(0, "transaction", &ordered_hex)
            .unwrap();
        assert_eq!(json, ordered);

        assert_eq!(
            abieos.json_to_hex_native(0, "uint8", "1").unwrap(),
            "01",
            "a loaded ABI at contract 0 should still expose builtins"
        );

        let fresh = Abieos::new();
        assert_eq!(
            fresh.json_to_hex_native(0, "uint8", "1").unwrap(),
            "01",
            "contract 0 should still provide builtins when no ABI is loaded"
        );
    }
}
