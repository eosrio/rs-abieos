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

    #[test]
    fn rust_backend_handles_new_abi_fixtures() {
        let abieos = Abieos::new();

        // 1. Load testkv.abi.json
        let testkv_abi = include_str!("../abis/testkv.abi.json");
        abieos.set_abi_json("testkv", testkv_abi).unwrap();

        // Test my_struct serialization/deserialization
        let my_struct_json =
            r#"{"primary":"user1","foo":"hello","bar":123456,"fullname":"Igor Lins","age":30}"#;
        let my_struct_expected =
            r#"{"primary":"user1","foo":"hello","bar":"123456","fullname":"Igor Lins","age":30}"#;
        let my_struct_hex = abieos
            .json_to_hex("testkv", "my_struct", my_struct_json)
            .unwrap();
        let my_struct_back = abieos
            .hex_to_json("testkv", "my_struct", &my_struct_hex)
            .unwrap();
        assert_eq!(my_struct_back, my_struct_expected);

        // Test tuple_string_uint32 serialization/deserialization
        let tuple_json = r#"{"field_0":"test","field_1":999}"#;
        let tuple_hex = abieos
            .json_to_hex("testkv", "tuple_string_uint32", tuple_json)
            .unwrap();
        let tuple_back = abieos
            .hex_to_json("testkv", "tuple_string_uint32", &tuple_hex)
            .unwrap();
        assert_eq!(tuple_back, tuple_json);

        // 2. Load packed_transaction.abi.json
        let packed_tx_abi = include_str!("../abis/packed_transaction.abi.json");
        abieos.set_abi_json("packed_tx", packed_tx_abi).unwrap();

        // Test action serialization/deserialization
        let action_json = r#"{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6"}"#;
        let action_hex = abieos
            .json_to_hex("packed_tx", "action", action_json)
            .unwrap();
        let action_back = abieos
            .hex_to_json("packed_tx", "action", &action_hex)
            .unwrap();
        assert_eq!(action_back, action_json);

        // 3. Load ship.abi.json
        let ship_abi = include_str!("../abis/ship.abi.json");
        abieos.set_abi_json("ship", ship_abi).unwrap();

        // Test block_position serialization/deserialization
        let block_pos_json = r#"{"block_num":1000,"block_id":"000003E800000000000000000000000000000000000000000000000000000000"}"#;
        let block_pos_hex = abieos
            .json_to_hex("ship", "block_position", block_pos_json)
            .unwrap();
        let block_pos_back = abieos
            .hex_to_json("ship", "block_position", &block_pos_hex)
            .unwrap();
        assert_eq!(block_pos_back, block_pos_json);
    }

    #[test]
    fn rust_backend_handles_cpp_complex_fixtures() {
        let abieos = Abieos::new();

        // Setup ABIs as in C++ test
        let token_hex_abi = "0e656f73696f3a3a6162692f312e30010c6163636f756e745f6e616d65046e61\
                             6d6505087472616e7366657200040466726f6d0c6163636f756e745f6e616d65\
                             02746f0c6163636f756e745f6e616d65087175616e7469747905617373657404\
                             6d656d6f06737472696e67066372656174650002066973737565720c6163636f\
                             756e745f6e616d650e6d6178696d756d5f737570706c79056173736574056973\
                             737565000302746f0c6163636f756e745f6e616d65087175616e746974790561\
                             73736574046d656d6f06737472696e67076163636f756e7400010762616c616e\
                             63650561737365740e63757272656e63795f7374617473000306737570706c79\
                             0561737365740a6d61785f737570706c79056173736574066973737565720c61\
                             63636f756e745f6e616d6503000000572d3ccdcd087472616e73666572000000\
                             000000a531760569737375650000000000a86cd4450663726561746500020000\
                             00384f4d113203693634010863757272656e6379010675696e74363407616363\
                             6f756e740000000000904dc603693634010863757272656e6379010675696e74\
                             36340e63757272656e63795f7374617473000000";

        let transaction_abi = include_str!("../abis/transaction.abi.json");
        let packed_tx_abi = include_str!("../abis/packed_transaction.abi.json");
        let ship_abi = include_str!("../abis/ship.abi.json");

        let token_name = abieos.string_to_name("eosio.token").unwrap();

        abieos
            .set_abi_hex_native(token_name, token_hex_abi)
            .unwrap();
        abieos.set_abi_json_native(0, transaction_abi).unwrap();
        abieos.set_abi_json_native(1, packed_tx_abi).unwrap();
        abieos.set_abi_json_native(2, ship_abi).unwrap();

        // 1003: transfer
        let transfer_json = r#"{"from":"useraaaaaaaa","to":"useraaaaaaab","quantity":"0.0001 SYS","memo":"test memo"}"#;
        let hex = abieos
            .json_to_hex_native(token_name, "transfer", transfer_json)
            .unwrap();
        let back = abieos
            .hex_to_json_native(token_name, "transfer", &hex)
            .unwrap();
        assert_eq!(back, transfer_json);

        // 1005: transaction
        let transaction_json = r#"{"expiration":"2009-02-13T23:31:31.000","ref_block_num":1234,"ref_block_prefix":5678,"max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}"#;
        let hex = abieos
            .json_to_hex_native(0, "transaction", transaction_json)
            .unwrap();
        let back = abieos.hex_to_json_native(0, "transaction", &hex).unwrap();
        assert_eq!(back, transaction_json);

        // 1009: transfer reorder
        let transfer_unorder = r#"{"to":"useraaaaaaab","memo":"test memo","from":"useraaaaaaaa","quantity":"0.0001 SYS"}"#;
        let hex = abieos
            .json_to_hex_native(token_name, "transfer", transfer_unorder)
            .unwrap();
        let back = abieos
            .hex_to_json_native(token_name, "transfer", &hex)
            .unwrap();
        assert_eq!(back, transfer_json);

        // 1013: transaction reorder
        let transaction_unorder = r#"{"ref_block_num":1234,"ref_block_prefix":5678,"expiration":"2009-02-13T23:31:31.000","max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}"#;
        let hex = abieos
            .json_to_hex_native(0, "transaction", transaction_unorder)
            .unwrap();
        let back = abieos.hex_to_json_native(0, "transaction", &hex).unwrap();
        assert_eq!(back, transaction_json);

        // 1019: packed_transaction_v0
        let packed_tx_json = r#"{"signatures":["SIG_K1_K5PGhrkUBkThs8zdTD9mGUJZvxL4eU46UjfYJSEdZ9PXS2Cgv5jAk57yTx4xnrdSocQm6DDvTaEJZi5WLBsoZC4XYNS8b3"],"compression":0,"packed_context_free_data":"","packed_trx":{"expiration":"2009-02-13T23:31:31.000","ref_block_num":1234,"ref_block_prefix":5678,"max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}}"#;
        let hex = abieos
            .json_to_hex_native(1, "packed_transaction_v0", packed_tx_json)
            .unwrap();
        let back = abieos
            .hex_to_json_native(1, "packed_transaction_v0", &hex)
            .unwrap();
        assert_eq!(back, packed_tx_json);

        // 1021: transaction_trace
        let trace_json = r#"["transaction_trace_v0",{"id":"3098EA9476266BFA957C13FA73C26806D78753099CE8DEF2A650971F07595A69","status":0,"cpu_usage_us":2000,"net_usage_words":25,"elapsed":"194","net_usage":"200","scheduled":false,"action_traces":[["action_trace_v1",{"action_ordinal":1,"creator_action_ordinal":0,"receipt":["action_receipt_v0",{"receiver":"eosio","act_digest":"F2FDEEFF77EFC899EED23EE05F9469357A096DC3083D493571CF68A422C69EFE","global_sequence":"11","recv_sequence":"11","auth_sequence":[{"account":"eosio","sequence":"11"}],"code_sequence":2,"abi_sequence":0}],"receiver":"eosio","act":{"account":"eosio","name":"newaccount","authorization":[{"actor":"eosio","permission":"active"}],"data":"0000000000EA305500409406A888CCA501000000010002C0DED2BC1F1305FB0FAAC5E6C03EE3A1924234985427B6167CA569D13DF435CF0100000001000000010002C0DED2BC1F1305FB0FAAC5E6C03EE3A1924234985427B6167CA569D13DF435CF01000000"},"context_free":false,"elapsed":"83","console":"","account_ram_deltas":[{"account":"oracle.aml","delta":"2724"}],"except":null,"error_code":null,"return_value":""}]],"account_ram_delta":null,"except":null,"error_code":null,"failed_dtrx_trace":null,"partial":null}]"#;
        let hex = abieos
            .json_to_hex_native(2, "transaction_trace", trace_json)
            .unwrap();
        let back = abieos
            .hex_to_json_native(2, "transaction_trace", &hex)
            .unwrap();
        assert_eq!(back, trace_json);
    }
}
