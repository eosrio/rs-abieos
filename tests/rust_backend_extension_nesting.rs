#[cfg(feature = "rust-backend")]
mod rust_backend_extension_nesting {
    use rs_abieos::Abieos;

    struct Case {
        ty: &'static str,
        json: &'static str,
        expected_json: Option<&'static str>,
    }

    impl Case {
        const fn new(ty: &'static str, json: &'static str) -> Self {
            Self {
                ty,
                json,
                expected_json: None,
            }
        }

        const fn with_expected(
            ty: &'static str,
            json: &'static str,
            expected_json: &'static str,
        ) -> Self {
            Self {
                ty,
                json,
                expected_json: Some(expected_json),
            }
        }
    }

    const TEST_ABI: &str = r#"{
        "version": "eosio::abi/1.1",
        "structs": [
            {
                "name": "s1",
                "fields": [
                    { "name": "x1", "type": "int8" }
                ]
            },
            {
                "name": "s2",
                "fields": [
                    { "name": "y1", "type": "int8$" },
                    { "name": "y2", "type": "int8$" }
                ]
            },
            {
                "name": "s3",
                "fields": [
                    { "name": "z1", "type": "int8$" },
                    { "name": "z2", "type": "v1$" },
                    { "name": "z3", "type": "s2$" }
                ]
            },
            {
                "name": "s4",
                "fields": [
                    { "name": "a1", "type": "int8?$" },
                    { "name": "b1", "type": "int8[]$" }
                ]
            },
            {
                "name": "ext_struct",
                "fields": [
                    {"name": "f1", "type": "int32"},
                    {"name": "f2", "type": "int32$"},
                    {"name": "f3", "type": "int32$"}
                ]
            },
            {
                "name": "nested_ext_struct",
                "fields": [
                    {"name": "n1", "type": "int32"},
                    {"name": "n2", "type": "ext_struct$"}
                ]
            },
            {
                "name": "array_of_ext_struct",
                "fields": [
                    {"name": "a1", "type": "ext_struct[]"}
                ]
            },
            {
                "name": "ext_array_of_ext_struct",
                "fields": [
                    {"name": "a1", "type": "ext_struct[]$"}
                ]
            },
            {
                "name": "variant_of_ext_struct",
                "fields": [
                    {"name": "v1", "type": "v_ext"}
                ]
            },
            {
                "name": "complex_nesting",
                "fields": [
                    {"name": "c1", "type": "int32"},
                    {"name": "c2", "type": "nested_ext_struct$"},
                    {"name": "c3", "type": "array_of_ext_struct$"},
                    {"name": "c4", "type": "variant_of_ext_struct$"}
                ]
            },
            {
                "name": "s7",
                "fields": [
                    { "name": "bs", "type": "bitset" }
                ]
            },
            {
                "name": "s8",
                "fields": [
                    { "name": "a1", "type": "int8[2]" }
                ]
            },
            {
                "name": "s9",
                "fields": [
                    { "name": "a1", "type": "s1[2]" }
                ]
            }
        ],
        "variants": [
            {
                "name": "v1",
                "types": ["int8","s1","s2","s7"]
            },
            {
                "name": "v_ext",
                "types": ["int32", "ext_struct"]
            }
        ]
    }"#;

    fn assert_check_type_cases(cases: &[Case]) {
        let abieos = Abieos::new();
        abieos
            .set_abi_json_native(0, TEST_ABI)
            .expect("Failed to set ABI");

        for case in cases {
            let hex = abieos
                .json_to_hex_native(0, case.ty, case.json)
                .unwrap_or_else(|err| {
                    panic!(
                        "json_to_hex_native failed for type {} and json {}: {}",
                        case.ty, case.json, err
                    )
                });
            let actual = abieos
                .hex_to_json_native(0, case.ty, &hex)
                .unwrap_or_else(|err| {
                    panic!(
                        "hex_to_json_native failed for type {} and json {} (hex {}): {}",
                        case.ty, case.json, hex, err
                    )
                });
            let expected = case.expected_json.unwrap_or(case.json);
            assert_eq!(
                actual, expected,
                "round-trip JSON mismatch for type {} and input {} (hex {})",
                case.ty, case.json, hex
            );
        }
    }

    #[test]
    fn test_binary_extension_skipping_nesting() {
        assert_check_type_cases(&[
            // Ported from test.cpp
            Case::new("v1", r#"["int8",7]"#),
            Case::new("v1", r#"["s1",{"x1":6}]"#),
            Case::new("v1", r#"["s2",{"y1":5,"y2":4}]"#),
            Case::new("s3", r#"{}"#),
            Case::new("s3", r#"{"z1":7}"#),
            Case::new("s3", r#"{"z1":7,"z2":["int8",6]}"#),
            Case::with_expected(
                "s3",
                r#"{"z1":7,"z2":["int8",6],"z3":{}}"#,
                r#"{"z1":7,"z2":["int8",6]}"#,
            ),
            Case::new("s3", r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9}}"#),
            Case::new("s3", r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9,"y2":10}}"#),
            Case::new("s4", r#"{}"#),
            Case::new("s4", r#"{"a1":null}"#),
            Case::new("s4", r#"{"a1":7}"#),
            Case::new("s4", r#"{"a1":null,"b1":[]}"#),
            Case::new("s4", r#"{"a1":null,"b1":[5,6,7]}"#),
            // Additional nesting shapes

            // ext_struct: int32, int32$, int32$
            Case::new("ext_struct", r#"{"f1":1}"#),
            Case::new("ext_struct", r#"{"f1":1,"f2":2}"#),
            Case::new("ext_struct", r#"{"f1":1,"f2":2,"f3":3}"#),
            // Nested structs with extensions
            Case::new("nested_ext_struct", r#"{"n1":10}"#),
            Case::new("nested_ext_struct", r#"{"n1":10,"n2":{"f1":1}}"#),
            Case::new("nested_ext_struct", r#"{"n1":10,"n2":{"f1":1,"f2":2}}"#),
            Case::new(
                "nested_ext_struct",
                r#"{"n1":10,"n2":{"f1":1,"f2":2,"f3":3}}"#,
            ),
            // Array of structs with extensions - internal extensions MUST NOT be skipped
            Case::new("array_of_ext_struct", r#"{"a1":[]}"#),
            Case::new("array_of_ext_struct", r#"{"a1":[{"f1":1,"f2":2,"f3":3}]}"#),
            Case::new(
                "array_of_ext_struct",
                r#"{"a1":[{"f1":1,"f2":2,"f3":3},{"f1":4,"f2":5,"f3":6}]}"#,
            ),
            // Extension array of ext_struct
            Case::new("ext_array_of_ext_struct", r#"{}"#),
            Case::new("ext_array_of_ext_struct", r#"{"a1":[]}"#),
            Case::new(
                "ext_array_of_ext_struct",
                r#"{"a1":[{"f1":1,"f2":2,"f3":3}]}"#,
            ),
            // Variants containing structs with extensions
            Case::new("variant_of_ext_struct", r#"{"v1":["int32",100]}"#),
            Case::new(
                "variant_of_ext_struct",
                r#"{"v1":["ext_struct",{"f1":10}]}"#,
            ),
            Case::new(
                "variant_of_ext_struct",
                r#"{"v1":["ext_struct",{"f1":10,"f2":20}]}"#,
            ),
            Case::new(
                "variant_of_ext_struct",
                r#"{"v1":["ext_struct",{"f1":10,"f2":20,"f3":30}]}"#,
            ),
            // Complex Nesting
            Case::new("complex_nesting", r#"{"c1":1000}"#),
            // Must provide extensions in order. Cannot skip c2 but provide c3.
            Case::new(
                "complex_nesting",
                r#"{"c1":1000,"c2":{"n1":10,"n2":{"f1":1,"f2":2,"f3":3}}}"#,
            ),
            Case::new(
                "complex_nesting",
                r#"{"c1":1000,"c2":{"n1":10,"n2":{"f1":1,"f2":2,"f3":3}},"c3":{"a1":[{"f1":1,"f2":2,"f3":3}]}}"#,
            ),
            // c4 IS the last field, so its internal extensions CAN be skipped
            Case::new(
                "complex_nesting",
                r#"{"c1":1000,"c2":{"n1":10,"n2":{"f1":1,"f2":2,"f3":3}},"c3":{"a1":[{"f1":1,"f2":2,"f3":3}]},"c4":{"v1":["ext_struct",{"f1":10}]}}"#,
            ),
            // Additional cases from test.cpp
            Case::new("s8", r#"{"a1":[1,27]}"#),
            Case::new("s9", r#"{"a1":[{"x1":6},{"x1":16}]}"#),
            Case::new("s7", r#"{"bs":""}"#),
            Case::new("s7", r#"{"bs":"00000000"}"#),
            Case::new("s7", r#"{"bs":"1011001"}"#),
        ]);
    }
}
