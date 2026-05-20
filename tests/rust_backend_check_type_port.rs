#[cfg(feature = "rust-backend")]
mod rust_backend_check_type_port {
    use rs_abieos::Abieos;

    #[derive(Clone, Copy)]
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

    fn assert_check_type_cases(cases: &[Case]) {
        let abieos = Abieos::new();

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
    fn rust_backend_ports_additional_integer_and_varint_success_cases() {
        assert_check_type_cases(&[
            Case::new("bool", "true"),
            Case::new("bool", "false"),
            Case::new("int8", "0"),
            Case::new("int8", "127"),
            Case::new("int8", "-128"),
            Case::new("uint8", "0"),
            Case::new("uint8", "1"),
            Case::new("uint8", "254"),
            Case::new("uint8", "255"),
            Case::new("int16", "0"),
            Case::new("int16", "32767"),
            Case::new("int16", "-32768"),
            Case::new("uint16", "0"),
            Case::new("uint16", "65535"),
            Case::new("int32", "0"),
            Case::new("int32", "2147483647"),
            Case::new("int32", "-2147483648"),
            Case::new("uint32", "0"),
            Case::new("uint32", "4294967295"),
            Case::with_expected("int64", "0", r#""0""#),
            Case::with_expected("int64", "1", r#""1""#),
            Case::with_expected("int64", "-1", r#""-1""#),
            Case::new("int64", r#""0""#),
            Case::new("int64", r#""9223372036854775807""#),
            Case::new("int64", r#""-9223372036854775808""#),
            Case::new("uint64", r#""0""#),
            Case::new("uint64", r#""18446744073709551615""#),
            Case::new("int128", r#""0""#),
            Case::new("int128", r#""1""#),
            Case::new("int128", r#""-1""#),
            Case::new("int128", r#""18446744073709551615""#),
            Case::new("int128", r#""-18446744073709551615""#),
            Case::new("int128", r#""170141183460469231731687303715884105727""#),
            Case::new("int128", r#""-170141183460469231731687303715884105727""#),
            Case::new("int128", r#""-170141183460469231731687303715884105728""#),
            Case::new("uint128", r#""0""#),
            Case::new("uint128", r#""1""#),
            Case::new("uint128", r#""18446744073709551615""#),
            Case::new("uint128", r#""340282366920938463463374607431768211454""#),
            Case::new("uint128", r#""340282366920938463463374607431768211455""#),
            Case::new("varuint32", "0"),
            Case::new("varuint32", "127"),
            Case::new("varuint32", "128"),
            Case::new("varuint32", "129"),
            Case::new("varuint32", "16383"),
            Case::new("varuint32", "16384"),
            Case::new("varuint32", "16385"),
            Case::new("varuint32", "2097151"),
            Case::new("varuint32", "2097152"),
            Case::new("varuint32", "2097153"),
            Case::new("varuint32", "268435455"),
            Case::new("varuint32", "268435456"),
            Case::new("varuint32", "268435457"),
            Case::new("varuint32", "4294967294"),
            Case::new("varuint32", "4294967295"),
            Case::new("varint32", "0"),
            Case::new("varint32", "-1"),
            Case::new("varint32", "1"),
            Case::new("varint32", "-2"),
            Case::new("varint32", "2"),
            Case::new("varint32", "-2147483647"),
            Case::new("varint32", "2147483647"),
            Case::new("varint32", "-2147483648"),
        ]);
    }

    #[test]
    fn rust_backend_ports_float_time_name_bytes_string_and_checksum_success_cases() {
        assert_check_type_cases(&[
            Case::with_expected("float32", "0.0", "0"),
            Case::new("float32", "0.125"),
            Case::new("float32", "-0.125"),
            Case::with_expected("float64", "0.0", "0"),
            Case::new("float64", "0.125"),
            Case::new("float64", "-0.125"),
            Case::with_expected(
                "float64",
                "151115727451828646838272.0",
                "151115727451828646838272",
            ),
            Case::with_expected(
                "float64",
                "-151115727451828646838272.0",
                "-151115727451828646838272",
            ),
            Case::new("float128", r#""00000000000000000000000000000000""#),
            Case::new("float128", r#""FFFFFFFFFFFFFFFFFFFFFFFFFFFFFFFF""#),
            Case::new("float128", r#""12345678ABCDEF12345678ABCDEF1234""#),
            Case::new("time_point_sec", r#""1970-01-01T00:00:00.000""#),
            Case::new("time_point_sec", r#""2018-06-15T19:17:47.000""#),
            Case::new("time_point_sec", r#""2030-06-15T19:17:47.000""#),
            Case::new("time_point", r#""1970-01-01T00:00:00.000""#),
            Case::new("time_point", r#""1970-01-01T00:00:00.001""#),
            Case::new("time_point", r#""1970-01-01T00:00:00.002""#),
            Case::new("time_point", r#""1970-01-01T00:00:00.010""#),
            Case::new("time_point", r#""1970-01-01T00:00:00.100""#),
            Case::new("time_point", r#""2018-06-15T19:17:47.000""#),
            Case::new("time_point", r#""2018-06-15T19:17:47.999""#),
            Case::new("time_point", r#""2030-06-15T19:17:47.999""#),
            Case::with_expected(
                "time_point",
                r#""2000-12-31T23:59:59.999999""#,
                r#""2000-12-31T23:59:59.999""#,
            ),
            Case::new("block_timestamp_type", r#""2000-01-01T00:00:00.000""#),
            Case::new("block_timestamp_type", r#""2000-01-01T00:00:00.500""#),
            Case::new("block_timestamp_type", r#""2000-01-01T00:00:01.000""#),
            Case::new("block_timestamp_type", r#""2018-06-15T19:17:47.500""#),
            Case::new("block_timestamp_type", r#""2018-06-15T19:17:48.000""#),
            Case::new("name", r#""""#),
            Case::new("name", r#""1""#),
            Case::new("name", r#""abcd""#),
            Case::new("name", r#""ab.cd.ef""#),
            Case::new("name", r#""ab.cd.ef.1234""#),
            Case::with_expected("name", r#""..ab.cd.ef..""#, r#""..ab.cd.ef""#),
            Case::new("name", r#""zzzzzzzzzzzz""#),
            Case::new("bytes", r#""""#),
            Case::new("bytes", r#""00""#),
            Case::new("bytes", r#""AABBCCDDEEFF00010203040506070809""#),
            Case::new("string", r#""""#),
            Case::new("string", r#""z""#),
            Case::new("string", r#""This is a string.""#),
            Case::new("string", r#""' + '*'.repeat(128) + '""#),
            Case::new(
                "string",
                r#""\u0000  这是一个测试  Это тест  هذا اختبار 👍""#,
            ),
            Case::new(
                "checksum160",
                r#""0000000000000000000000000000000000000000""#,
            ),
            Case::new(
                "checksum160",
                r#""123456789ABCDEF01234567890ABCDEF70123456""#,
            ),
            Case::new(
                "checksum256",
                r#""0000000000000000000000000000000000000000000000000000000000000000""#,
            ),
            Case::new(
                "checksum256",
                r#""0987654321ABCDEF0987654321FFFF1234567890ABCDEF001234567890ABCDEF""#,
            ),
            Case::new(
                "checksum512",
                r#""00000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000""#,
            ),
            Case::new(
                "checksum512",
                r#""0987654321ABCDEF0987654321FFFF1234567890ABCDEF001234567890ABCDEF0987654321ABCDEF0987654321FFFF1234567890ABCDEF001234567890ABCDEF""#,
            ),
        ]);
    }

    #[test]
    fn rust_backend_ports_additional_array_and_bitset_shape_success_cases() {
        assert_check_type_cases(&[
            Case::new("uint8[]", r#"[]"#),
            Case::new("uint8[]", r#"[10]"#),
            Case::new("uint8[]", r#"[10,9]"#),
            Case::new("uint8[]", r#"[10,9,8]"#),
            Case::new("uint8[1]", r#"[10]"#),
            Case::new("uint8[2]", r#"[10,9]"#),
            Case::new("uint8[3]", r#"[10,9,8]"#),
            Case::new("string[]", r#"["hello","world"]"#),
            Case::new("string[][]", r#"[["A"],["B"],["C","D"]]"#),
            Case::new("uint8[][]", r#"[[1]]"#),
            Case::new("uint8[][][]", r#"[[[1,2,3],[4,5,6]],[[7,8,9],[]]]"#),
            Case::new("bitset", r#""""#),
            Case::new("bitset", r#""0""#),
            Case::new("bitset", r#""11""#),
            Case::new("bitset", r#""011""#),
            Case::new("bitset", r#""110001011""#),
            Case::new("bitset", r#""1100010110110""#),
            Case::new("bitset", r#""11000101101100011010101110""#),
            Case::new("bitset", r#""11000101101100011010101110100110""#),
            Case::new("bitset", r#""110001011011000110101011101001100110""#),
            Case::new("bitset", r#""110001011011000110101011101001100110000110""#),
            Case::new(
                "bitset",
                r#""110001011011000110101011101001100110000110000000000000000001""#,
            ),
            Case::new(
                "bitset",
                r#""110001011011000110101011101001100110000111111111111111111110""#,
            ),
        ]);
    }

    #[test]
    fn rust_backend_ports_additional_symbol_asset_success_cases() {
        assert_check_type_cases(&[
            Case::new("symbol_code", r#""A""#),
            Case::new("symbol_code", r#""B""#),
            Case::new("symbol_code", r#""SYS""#),
            Case::new("symbol", r#""0,A""#),
            Case::new("symbol", r#""1,Z""#),
            Case::new("symbol", r#""4,SYS""#),
            Case::new("asset", r#""0 FOO""#),
            Case::new("asset", r#""0.0 FOO""#),
            Case::new("asset", r#""0.00 FOO""#),
            Case::new("asset", r#""0.000 FOO""#),
            Case::new("asset", r#""1.2345 SYS""#),
            Case::new("asset", r#""-1.2345 SYS""#),
            Case::new("asset[]", r#"[]"#),
            Case::new("asset[]", r#"["0 FOO"]"#),
            Case::new("asset[]", r#"["0 FOO","0.000 FOO"]"#),
            Case::new("asset[1]", r#"["0 FOO"]"#),
            Case::new("asset[2]", r#"["0 FOO","0.000 FOO"]"#),
            Case::new("asset?", r#"null"#),
            Case::new("asset?", r#""0.123456 SIX""#),
            Case::new("extended_asset", r#"{"quantity":"0 FOO","contract":"bar"}"#),
            Case::new(
                "extended_asset",
                r#"{"quantity":"0.123456 SIX","contract":"seven"}"#,
            ),
        ]);
    }
}
