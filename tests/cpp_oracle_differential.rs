#![allow(clippy::manual_is_multiple_of)]

#[cfg(all(feature = "rust-backend", feature = "cpp-oracle"))]
#[path = "common/extension_nesting_fixtures.rs"]
mod extension_nesting_fixtures;

#[cfg(all(feature = "rust-backend", feature = "cpp-oracle"))]
mod cpp_oracle_differential {
    use super::extension_nesting_fixtures::{EXTENSION_NESTING_ABI, EXTENSION_NESTING_CASES};
    use rs_abieos::{cpp_oracle, Abieos};
    use std::ffi::{CStr, CString};

    const TEST_ABI_CONTRACT: &str = "test.abi";
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

    #[derive(Clone, Copy)]
    struct CodecRow {
        label: &'static str,
        contract: u64,
        ty: &'static str,
        json: &'static str,
        expected_json: Option<&'static str>,
    }

    impl CodecRow {
        const fn success(
            label: &'static str,
            contract: u64,
            ty: &'static str,
            json: &'static str,
            expected_json: &'static str,
        ) -> Self {
            Self {
                label,
                contract,
                ty,
                json,
                expected_json: Some(expected_json),
            }
        }

        const fn failure(
            label: &'static str,
            contract: u64,
            ty: &'static str,
            json: &'static str,
        ) -> Self {
            Self {
                label,
                contract,
                ty,
                json,
                expected_json: None,
            }
        }
    }

    struct Oracle {
        ctx: *mut cpp_oracle::abieos_context,
    }

    impl Oracle {
        fn new() -> Self {
            let ctx = Self::create_context();
            assert!(!ctx.is_null(), "C++ oracle context should be created");
            Self { ctx }
        }

        fn create_context() -> *mut cpp_oracle::abieos_context {
            unsafe { cpp_oracle::abieos_create() }
        }

        unsafe fn destroy_context(ctx: *mut cpp_oracle::abieos_context) {
            unsafe {
                cpp_oracle::abieos_destroy(ctx);
            }
        }

        fn last_error(&self) -> String {
            unsafe {
                let ptr = cpp_oracle::abieos_get_error(self.ctx);
                if ptr.is_null() {
                    String::new()
                } else {
                    CStr::from_ptr(ptr).to_string_lossy().into_owned()
                }
            }
        }

        fn capture_status(&self, ok: cpp_oracle::abieos_bool) -> Result<(), String> {
            match ok {
                1 => Ok(()),
                _ => Err(self.last_error()),
            }
        }

        fn capture_string(&self, ptr: *const std::os::raw::c_char) -> Result<String, String> {
            if ptr.is_null() {
                Err(self.last_error())
            } else {
                unsafe { Ok(CStr::from_ptr(ptr).to_string_lossy().into_owned()) }
            }
        }

        fn capture_bin(&self) -> Result<Vec<u8>, String> {
            unsafe {
                let ptr = cpp_oracle::abieos_get_bin_data(self.ctx);
                let len = cpp_oracle::abieos_get_bin_size(self.ctx);
                if ptr.is_null() || len < 0 {
                    Err(self.last_error())
                } else {
                    Ok(std::slice::from_raw_parts(ptr.cast::<u8>(), len as usize).to_vec())
                }
            }
        }

        fn capture_bin_hex(&self) -> Result<String, String> {
            unsafe { self.capture_string(cpp_oracle::abieos_get_bin_hex(self.ctx)) }
        }

        fn string_to_name(&self, name: &str) -> u64 {
            let name = CString::new(name).unwrap();
            unsafe { cpp_oracle::abieos_string_to_name(self.ctx, name.as_ptr()) }
        }

        fn name_to_string(&self, name: u64) -> Result<String, String> {
            unsafe { self.capture_string(cpp_oracle::abieos_name_to_string(self.ctx, name)) }
        }

        fn set_abi_json(&self, contract: u64, abi: &str) -> Result<(), String> {
            let abi = CString::new(abi).unwrap();
            unsafe {
                self.capture_status(cpp_oracle::abieos_set_abi(self.ctx, contract, abi.as_ptr()))
            }
        }

        fn set_abi_bin(&self, contract: u64, abi: &[u8]) -> Result<(), String> {
            unsafe {
                self.capture_status(cpp_oracle::abieos_set_abi_bin(
                    self.ctx,
                    contract,
                    abi.as_ptr().cast(),
                    abi.len(),
                ))
            }
        }

        fn set_abi_hex(&self, contract: u64, abi_hex: &str) -> Result<(), String> {
            let abi_hex = CString::new(abi_hex).unwrap();
            unsafe {
                self.capture_status(cpp_oracle::abieos_set_abi_hex(
                    self.ctx,
                    contract,
                    abi_hex.as_ptr(),
                ))
            }
        }

        fn json_to_hex(&self, ty: &str, json: &str) -> Result<String, String> {
            self.json_to_hex_contract(0, ty, json)
        }

        fn json_to_hex_contract(
            &self,
            contract: u64,
            ty: &str,
            json: &str,
        ) -> Result<String, String> {
            let ty = CString::new(ty).unwrap();
            let json = CString::new(json).unwrap();
            unsafe {
                self.capture_status(cpp_oracle::abieos_json_to_bin_reorderable(
                    self.ctx,
                    contract,
                    ty.as_ptr(),
                    json.as_ptr(),
                ))?;
            }
            self.capture_bin_hex()
        }

        fn hex_to_json(&self, ty: &str, hex: &str) -> Result<String, String> {
            self.hex_to_json_contract(0, ty, hex)
        }

        fn hex_to_json_contract(
            &self,
            contract: u64,
            ty: &str,
            hex: &str,
        ) -> Result<String, String> {
            let ty = CString::new(ty).unwrap();
            let hex = CString::new(hex).unwrap();
            unsafe {
                self.capture_string(cpp_oracle::abieos_hex_to_json(
                    self.ctx,
                    contract,
                    ty.as_ptr(),
                    hex.as_ptr(),
                ))
            }
        }

        fn abi_json_to_bin(&self, abi_json: &str) -> Result<Vec<u8>, String> {
            let abi_json = CString::new(abi_json).unwrap();
            unsafe {
                self.capture_status(cpp_oracle::abieos_abi_json_to_bin(
                    self.ctx,
                    abi_json.as_ptr(),
                ))?;
            }
            self.capture_bin()
        }

        fn abi_json_to_hex(&self, abi_json: &str) -> Result<String, String> {
            let abi_json = CString::new(abi_json).unwrap();
            unsafe {
                self.capture_status(cpp_oracle::abieos_abi_json_to_bin(
                    self.ctx,
                    abi_json.as_ptr(),
                ))?;
            }
            self.capture_bin_hex()
        }

        fn abi_bin_to_json(&self, abi: &[u8]) -> Result<String, String> {
            unsafe {
                self.capture_string(cpp_oracle::abieos_abi_bin_to_json(
                    self.ctx,
                    abi.as_ptr().cast(),
                    abi.len(),
                ))
            }
        }

        fn abi_hex_to_json(&self, abi_hex: &str) -> Result<String, String> {
            let abi = hex_to_bytes(abi_hex)?;
            self.abi_bin_to_json(&abi)
        }
    }

    impl Drop for Oracle {
        fn drop(&mut self) {
            unsafe {
                Self::destroy_context(self.ctx);
            }
        }
    }

    fn result_status<T, E>(result: &Result<T, E>) -> &'static str {
        if result.is_ok() {
            "ok"
        } else {
            "err"
        }
    }

    fn compare_codec_rows(rust: &Abieos, oracle: &Oracle, rows: &[CodecRow]) {
        for row in rows {
            let rust_hex = rust
                .json_to_hex_native(row.contract, row.ty, row.json)
                .map_err(|e| e.to_string());
            let oracle_hex = oracle.json_to_hex_contract(row.contract, row.ty, row.json);
            assert_eq!(
                result_status(&rust_hex),
                result_status(&oracle_hex),
                "json_to_hex status mismatch for {}. Rust hex: {:?}, Oracle error: {:?}",
                row.label,
                rust_hex,
                oracle.last_error()
            );

            match (rust_hex, oracle_hex, row.expected_json) {
                (Ok(rust_hex), Ok(oracle_hex), Some(expected_json)) => {
                    assert_eq!(
                        rust_hex, oracle_hex,
                        "json_to_hex mismatch for {}",
                        row.label
                    );

                    let rust_json = rust
                        .hex_to_json_native(row.contract, row.ty, &rust_hex)
                        .map_err(|e| e.to_string());
                    let oracle_json =
                        oracle.hex_to_json_contract(row.contract, row.ty, &oracle_hex);
                    assert_eq!(
                        result_status(&rust_json),
                        result_status(&oracle_json),
                        "hex_to_json status mismatch for {}",
                        row.label
                    );

                    let rust_json = rust_json.unwrap();
                    let oracle_json = oracle_json.unwrap();
                    assert_eq!(
                        rust_json, oracle_json,
                        "hex_to_json mismatch for {}",
                        row.label
                    );
                    assert_eq!(
                        rust_json, expected_json,
                        "deterministic JSON mismatch for {}",
                        row.label
                    );
                }
                (Ok(_), Ok(_), None) => panic!("{} unexpectedly succeeded", row.label),
                (Err(rust_error), Err(oracle_error), None) => {
                    assert!(!rust_error.is_empty(), "Rust error should not be empty");
                    assert!(!oracle_error.is_empty(), "oracle error should not be empty");
                }
                _ => unreachable!("status assertion above guarantees matching result shapes"),
            }
        }
    }

    fn hex_to_bytes(hex: &str) -> Result<Vec<u8>, String> {
        if hex.len() % 2 != 0 {
            return Err(format!("odd-length hex string: {hex}"));
        }

        hex.as_bytes()
            .chunks_exact(2)
            .map(|pair| {
                let pair = std::str::from_utf8(pair).map_err(|e| e.to_string())?;
                u8::from_str_radix(pair, 16).map_err(|e| e.to_string())
            })
            .collect()
    }

    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|b| format!("{b:02X}")).collect()
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_names() {
        let rust = Abieos::new();
        let oracle = Oracle::new();

        for name in ["", "1", "abcd", "ab.cd.ef", "eosio.token", "..ab.cd.ef.."] {
            let rust_name = rust.string_to_name(name).unwrap();
            let oracle_name = oracle.string_to_name(name);
            assert_eq!(rust_name, oracle_name, "string_to_name mismatch for {name}");
            assert_eq!(
                rust.name_to_string(rust_name).unwrap(),
                oracle.name_to_string(oracle_name).unwrap(),
                "name_to_string mismatch for {name}"
            );
        }
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_builtin_codec_rows() {
        let rust = Abieos::new();
        let oracle = Oracle::new();
        rust.set_abi_json_native(0, r#"{"version": "eosio::abi/1.1"}"#)
            .unwrap();
        oracle
            .set_abi_json(0, r#"{"version": "eosio::abi/1.1"}"#)
            .unwrap();

        compare_codec_rows(
            &rust,
            &oracle,
            &[
                CodecRow::success("bool true", 0, "bool", "true", "true"),
                CodecRow::success("bool false", 0, "bool", "false", "false"),
                CodecRow::success("int8 min", 0, "int8", "-128", "-128"),
                CodecRow::success("int8 max", 0, "int8", "127", "127"),
                CodecRow::success("uint8 max", 0, "uint8", "255", "255"),
                CodecRow::success("int16 min", 0, "int16", "-32768", "-32768"),
                CodecRow::success("uint16 max", 0, "uint16", "65535", "65535"),
                CodecRow::success("int32 min", 0, "int32", "-2147483648", "-2147483648"),
                CodecRow::success("uint32 max", 0, "uint32", "4294967295", "4294967295"),
                CodecRow::success(
                    "int64 min",
                    0,
                    "int64",
                    "-9223372036854775808",
                    r#""-9223372036854775808""#,
                ),
                CodecRow::success(
                    "uint64 max",
                    0,
                    "uint64",
                    "18446744073709551615",
                    r#""18446744073709551615""#,
                ),
                CodecRow::success(
                    "int128 max",
                    0,
                    "int128",
                    r#""170141183460469231731687303715884105727""#,
                    r#""170141183460469231731687303715884105727""#,
                ),
                CodecRow::success(
                    "uint128 max",
                    0,
                    "uint128",
                    r#""340282366920938463463374607431768211455""#,
                    r#""340282366920938463463374607431768211455""#,
                ),
                CodecRow::success("varuint32 max", 0, "varuint32", "4294967295", "4294967295"),
                CodecRow::success("varint32 min", 0, "varint32", "-2147483648", "-2147483648"),
                CodecRow::success("float32", 0, "float32", "0.125", "0.125"),
                CodecRow::success("float64", 0, "float64", "-0.125", "-0.125"),
                CodecRow::success(
                    "float128",
                    0,
                    "float128",
                    r#""12345678ABCDEF12345678ABCDEF1234""#,
                    r#""12345678ABCDEF12345678ABCDEF1234""#,
                ),
                CodecRow::success(
                    "time_point_sec",
                    0,
                    "time_point_sec",
                    r#""2018-06-15T19:17:47.000""#,
                    r#""2018-06-15T19:17:47.000""#,
                ),
                CodecRow::success(
                    "time_point",
                    0,
                    "time_point",
                    r#""2000-12-31T23:59:59.999999""#,
                    r#""2000-12-31T23:59:59.999""#,
                ),
                CodecRow::success(
                    "block_timestamp_type",
                    0,
                    "block_timestamp_type",
                    r#""2000-01-01T00:00:00.500""#,
                    r#""2000-01-01T00:00:00.500""#,
                ),
                CodecRow::success("name", 0, "name", r#""..ab.cd.ef..""#, r#""..ab.cd.ef""#),
                CodecRow::success(
                    "bytes",
                    0,
                    "bytes",
                    r#""AABBCCDDEEFF00010203040506070809""#,
                    r#""AABBCCDDEEFF00010203040506070809""#,
                ),
                CodecRow::success(
                    "checksum160",
                    0,
                    "checksum160",
                    r#""123456789ABCDEF01234567890ABCDEF70123456""#,
                    r#""123456789ABCDEF01234567890ABCDEF70123456""#,
                ),
                CodecRow::success(
                    "checksum256",
                    0,
                    "checksum256",
                    r#""0987654321ABCDEF0987654321FFFF1234567890ABCDEF001234567890ABCDEF""#,
                    r#""0987654321ABCDEF0987654321FFFF1234567890ABCDEF001234567890ABCDEF""#,
                ),
                CodecRow::success("symbol_code", 0, "symbol_code", r#""SYS""#, r#""SYS""#),
                CodecRow::success("symbol", 0, "symbol", r#""4,SYS""#, r#""4,SYS""#),
                CodecRow::success("asset", 0, "asset", r#""-1.2345 SYS""#, r#""-1.2345 SYS""#),
                CodecRow::success(
                    "asset array",
                    0,
                    "asset[]",
                    r#"["0 FOO","0.000 FOO"]"#,
                    r#"["0 FOO","0.000 FOO"]"#,
                ),
                CodecRow::success(
                    "asset fixed array",
                    0,
                    "asset[2]",
                    r#"["0 FOO","0.000 FOO"]"#,
                    r#"["0 FOO","0.000 FOO"]"#,
                ),
                CodecRow::success("asset optional none", 0, "asset?", "null", "null"),
                CodecRow::success(
                    "asset optional some",
                    0,
                    "asset?",
                    r#""0.123456 SIX""#,
                    r#""0.123456 SIX""#,
                ),
                CodecRow::success(
                    "extended_asset",
                    0,
                    "extended_asset",
                    r#"{"quantity":"0.123456 SIX","contract":"seven"}"#,
                    r#"{"quantity":"0.123456 SIX","contract":"seven"}"#,
                ),
                CodecRow::success(
                    "string array",
                    0,
                    "string[]",
                    r#"["hello","world"]"#,
                    r#"["hello","world"]"#,
                ),
                CodecRow::success(
                    "string nested array",
                    0,
                    "string[][]",
                    r#"[["A"],["B"],["C","D"]]"#,
                    r#"[["A"],["B"],["C","D"]]"#,
                ),
                CodecRow::success("uint8 array", 0, "uint8[]", r#"[10,9,8]"#, r#"[10,9,8]"#),
                CodecRow::success(
                    "uint8 fixed array",
                    0,
                    "uint8[3]",
                    r#"[10,9,8]"#,
                    r#"[10,9,8]"#,
                ),
                CodecRow::success("uint8 nested array", 0, "uint8[][]", r#"[[1]]"#, r#"[[1]]"#),
                CodecRow::success(
                    "uint8 deeply nested array",
                    0,
                    "uint8[][][]",
                    r#"[[[1,2,3],[4,5,6]],[[7,8,9],[]]]"#,
                    r#"[[[1,2,3],[4,5,6]],[[7,8,9],[]]]"#,
                ),
                CodecRow::success("bitset empty", 0, "bitset", r#""""#, r#""""#),
                CodecRow::success("bitset zero", 0, "bitset", r#""0""#, r#""0""#),
                CodecRow::success("bitset short", 0, "bitset", r#""11""#, r#""11""#),
                CodecRow::success("bitset leading zero", 0, "bitset", r#""011""#, r#""011""#),
                CodecRow::success(
                    "bitset medium",
                    0,
                    "bitset",
                    r#""1100010110110""#,
                    r#""1100010110110""#,
                ),
                CodecRow::success(
                    "bitset long",
                    0,
                    "bitset",
                    r#""110001011011000110101011101001100110000110""#,
                    r#""110001011011000110101011101001100110000110""#,
                ),
                CodecRow::success(
                    "bitset long padded",
                    0,
                    "bitset",
                    r#""110001011011000110101011101001100110000111111111111111111110""#,
                    r#""110001011011000110101011101001100110000111111111111111111110""#,
                ),
                CodecRow::failure("uint8 overflow", 0, "uint8", "256"),
                CodecRow::failure("int8 underflow", 0, "int8", "-129"),
                CodecRow::failure("varuint32 negative", 0, "varuint32", "-1"),
                CodecRow::failure("bytes odd hex", 0, "bytes", r#""0""#),
                CodecRow::failure("checksum256 too short", 0, "checksum256", r#""a0""#),
                CodecRow::failure("symbol_code lowercase", 0, "symbol_code", r#""lower""#),
                CodecRow::failure("asset null", 0, "asset", "null"),
                // Milestone 4 — Boundary Audits (Numeric, Float, Time, Asset)
                CodecRow::failure("uint8 with plus sign", 0, "uint8", r#""+5""#),
                CodecRow::failure("uint64 with space", 0, "uint64", r#"" 123""#),
                CodecRow::failure("uint64 with trailing space", 0, "uint64", r#""123 ""#),
                CodecRow::failure("uint64 with trailing char", 0, "uint64", r#""123a""#),
                CodecRow::failure("float32 invalid string", 0, "float32", r#""abc""#),
                CodecRow::failure("bytes invalid hex characters", 0, "bytes", r#""ZZ""#),
                CodecRow::success(
                    "time_point civil date",
                    0,
                    "time_point",
                    r#""1970-01-01T00:00:00.000000""#,
                    r#""1970-01-01T00:00:00.000""#,
                ),
                CodecRow::success(
                    "time_point_sec civil date",
                    0,
                    "time_point_sec",
                    r#""1970-01-01T00:00:00""#,
                    r#""1970-01-01T00:00:00.000""#,
                ),
                CodecRow::failure(
                    "time_point invalid format",
                    0,
                    "time_point",
                    r#""1970-01-01""#,
                ),
                CodecRow::failure(
                    "time_point_sec invalid format",
                    0,
                    "time_point_sec",
                    r#""1970-01-01 00:00:00""#,
                ),
                CodecRow::success(
                    "asset without dot",
                    0,
                    "asset",
                    r#""10 SYS""#,
                    r#""10 SYS""#,
                ),
                CodecRow::success("negative asset", 0, "asset", r#""-10 SYS""#, r#""-10 SYS""#),
                CodecRow::success(
                    "asset double spaces",
                    0,
                    "asset",
                    r#""10  SYS""#,
                    r#""10 SYS""#,
                ),
                CodecRow::failure("asset lowercase symbol", 0, "asset", r#""10.0000 sys""#),
                CodecRow::success(
                    "string with emoji",
                    0,
                    "string",
                    r#""hello 🌞""#,
                    r#""hello 🌞""#,
                ),
                CodecRow::success(
                    "string with null",
                    0,
                    "string",
                    r#""a\u0000b""#,
                    r#""a\u0000b""#,
                ),
                CodecRow::success(
                    "float64 large",
                    0,
                    "float64",
                    "1.7976931348623157e+308",
                    "1.7976931348623157e+308",
                ),
                CodecRow::success("float64 small", 0, "float64", "1e-307", "1e-307"),
            ],
        );
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_loaded_abi_codec_rows() {
        let rust = Abieos::new();
        let oracle = Oracle::new();
        let contract = rust.string_to_name(TEST_ABI_CONTRACT).unwrap();
        assert_eq!(contract, oracle.string_to_name(TEST_ABI_CONTRACT));
        rust.set_abi_json(TEST_ABI_CONTRACT, TEST_ABI).unwrap();
        oracle.set_abi_json(contract, TEST_ABI).unwrap();

        compare_codec_rows(
            &rust,
            &oracle,
            &[
                CodecRow::success(
                    "fixed int8 array",
                    contract,
                    "s8",
                    r#"{"a1":[1,27]}"#,
                    r#"{"a1":[1,27]}"#,
                ),
                CodecRow::success(
                    "int8 variant",
                    contract,
                    "v1",
                    r#"["int8",7]"#,
                    r#"["int8",7]"#,
                ),
                CodecRow::success(
                    "struct variant",
                    contract,
                    "v1",
                    r#"["s1",{"x1":6}]"#,
                    r#"["s1",{"x1":6}]"#,
                ),
                CodecRow::success(
                    "fixed struct array",
                    contract,
                    "s9",
                    r#"{"a1":[{"x1":6},{"x1":16}]}"#,
                    r#"{"a1":[{"x1":6},{"x1":16}]}"#,
                ),
                CodecRow::success(
                    "struct bitset",
                    contract,
                    "s7",
                    r#"{"bs":"110001011"}"#,
                    r#"{"bs":"110001011"}"#,
                ),
                CodecRow::success(
                    "legacy public key canonicalization",
                    contract,
                    "public_key_holder",
                    r#"{"key":"EOS1111111111111111111111111111111114T1Anm"}"#,
                    r#"{"key":"PUB_K1_11111111111111111111111111111111149Mr2R"}"#,
                ),
                CodecRow::success(
                    "legacy private key canonicalization",
                    contract,
                    "private_key_holder",
                    r#"{"key":"5KQwrPbwdL6PhXujxW37FSSQZ1JiwsST4cqQzDeyXtP79zkvFD3"}"#,
                    r#"{"key":"PVT_K1_2bfGi9rYsXQSXXTvJbDAPhHLQUojjaNLomdm3cEJ1XTzMqUt3V"}"#,
                ),
                CodecRow::success(
                    "signature",
                    contract,
                    "signature_holder",
                    r#"{"sig":"SIG_K1_Kg2UKjXTX48gw2wWH4zmsZmWu3yarcfC21Bd9JPj7QoDURqiAacCHmtExPk3syPb2tFLsp1R4ttXLXgr7FYgDvKPC5RCkx"}"#,
                    r#"{"sig":"SIG_K1_Kg2UKjXTX48gw2wWH4zmsZmWu3yarcfC21Bd9JPj7QoDURqiAacCHmtExPk3syPb2tFLsp1R4ttXLXgr7FYgDvKPC5RCkx"}"#,
                ),
                CodecRow::failure("fixed array too short", contract, "s8", r#"{"a1":[1]}"#),
                CodecRow::failure("fixed array too long", contract, "s8", r#"{"a1":[1,2,3]}"#),
                CodecRow::failure(
                    "public key bad checksum",
                    contract,
                    "public_key_holder",
                    r#"{"key":"PUB_K1_11111111111111111111111111111111149Mr2S"}"#,
                ),
            ],
        );
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_extension_nesting_fixture_table() {
        let rust = Abieos::new();
        let oracle = Oracle::new();
        let contract = rust.string_to_name(TEST_ABI_CONTRACT).unwrap();
        assert_eq!(contract, oracle.string_to_name(TEST_ABI_CONTRACT));
        rust.set_abi_json(TEST_ABI_CONTRACT, EXTENSION_NESTING_ABI)
            .unwrap();
        oracle
            .set_abi_json(contract, EXTENSION_NESTING_ABI)
            .unwrap();

        let rows: Vec<_> = EXTENSION_NESTING_CASES
            .iter()
            .map(|case| {
                CodecRow::success(case.label, contract, case.ty, case.json, case.expected_json)
            })
            .collect();
        compare_codec_rows(&rust, &oracle, &rows);
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_new_abi_fixtures() {
        let rust = Abieos::new();
        let oracle = Oracle::new();

        // 1. Load testkv
        let testkv_contract = rust.string_to_name("testkv").unwrap();
        let testkv_abi = include_str!("../abis/testkv.abi.json");
        rust.set_abi_json("testkv", testkv_abi).unwrap();
        oracle.set_abi_json(testkv_contract, testkv_abi).unwrap();

        // 2. Load packed_transaction
        let packed_tx_contract = rust.string_to_name("packed.tx").unwrap();
        let packed_tx_abi = include_str!("../abis/packed_transaction.abi.json");
        rust.set_abi_json("packed.tx", packed_tx_abi).unwrap();
        oracle
            .set_abi_json(packed_tx_contract, packed_tx_abi)
            .unwrap();

        // 3. Load ship
        let ship_contract = rust.string_to_name("ship").unwrap();
        let ship_abi = include_str!("../abis/ship.abi.json");
        rust.set_abi_json("ship", ship_abi).unwrap();
        oracle.set_abi_json(ship_contract, ship_abi).unwrap();

        compare_codec_rows(
            &rust,
            &oracle,
            &[
                // testkv: my_struct
                CodecRow::success(
                    "testkv my_struct",
                    testkv_contract,
                    "my_struct",
                    r#"{"primary":"user1","foo":"hello","bar":123456,"fullname":"Igor Lins","age":30}"#,
                    r#"{"primary":"user1","foo":"hello","bar":"123456","fullname":"Igor Lins","age":30}"#,
                ),
                // testkv: tuple_string_uint32
                CodecRow::success(
                    "testkv tuple",
                    testkv_contract,
                    "tuple_string_uint32",
                    r#"{"field_0":"test","field_1":999}"#,
                    r#"{"field_0":"test","field_1":999}"#,
                ),
                // packed_transaction: action
                CodecRow::success(
                    "packed_tx action",
                    packed_tx_contract,
                    "action",
                    r#"{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6"}"#,
                    r#"{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6"}"#,
                ),
                // ship: block_position
                CodecRow::success(
                    "ship block_position",
                    ship_contract,
                    "block_position",
                    r#"{"block_num":1000,"block_id":"000003E800000000000000000000000000000000000000000000000000000000"}"#,
                    r#"{"block_num":1000,"block_id":"000003E800000000000000000000000000000000000000000000000000000000"}"#,
                ),
            ],
        );
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_abi_json_and_binary_helpers() {
        let rust = Abieos::new();
        let oracle = Oracle::new();
        let contract = rust.string_to_name(TEST_ABI_CONTRACT).unwrap();
        assert_eq!(contract, oracle.string_to_name(TEST_ABI_CONTRACT));

        let rust_bin = rust.abi_json_to_bin(TEST_ABI).unwrap();
        let oracle_bin = oracle.abi_json_to_bin(TEST_ABI).unwrap();
        assert_eq!(rust_bin, oracle_bin, "abi_json_to_bin bytes mismatch");

        let rust_hex = bytes_to_hex(&rust_bin);
        let oracle_hex = oracle.abi_json_to_hex(TEST_ABI).unwrap();
        assert_eq!(rust_hex, oracle_hex, "abi_json_to_bin hex mismatch");

        let rust_json = rust.abi_bin_to_json(&rust_bin).unwrap();
        let oracle_json = oracle.abi_bin_to_json(&oracle_bin).unwrap();
        assert_eq!(rust_json, oracle_json, "abi_bin_to_json mismatch");

        assert_eq!(
            oracle.abi_hex_to_json(&oracle_hex).unwrap(),
            oracle_json,
            "abi_hex_to_json mismatch"
        );

        let rust_from_bin = Abieos::new();
        let oracle_from_bin = Oracle::new();
        rust_from_bin
            .set_abi_bin_native(contract, &rust_bin)
            .unwrap();
        oracle_from_bin.set_abi_bin(contract, &oracle_bin).unwrap();
        compare_codec_rows(
            &rust_from_bin,
            &oracle_from_bin,
            &[CodecRow::success(
                "ABI loaded from binary",
                contract,
                "s8",
                r#"{"a1":[1,27]}"#,
                r#"{"a1":[1,27]}"#,
            )],
        );

        let rust_from_hex = Abieos::new();
        let oracle_from_hex = Oracle::new();
        rust_from_hex
            .set_abi_hex_native(contract, &rust_hex)
            .unwrap();
        oracle_from_hex.set_abi_hex(contract, &oracle_hex).unwrap();
        compare_codec_rows(
            &rust_from_hex,
            &oracle_from_hex,
            &[CodecRow::success(
                "ABI loaded from hex",
                contract,
                "s7",
                r#"{"bs":"110001011"}"#,
                r#"{"bs":"110001011"}"#,
            )],
        );
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_contract_zero_transaction_abi() {
        let rust = Abieos::new();
        let oracle = Oracle::new();
        let transaction_abi = include_str!("../abis/transaction.abi.json");
        rust.set_abi_json_native(0, transaction_abi).unwrap();
        oracle.set_abi_json(0, transaction_abi).unwrap();

        compare_codec_rows(
            &rust,
            &oracle,
            &[
                CodecRow::success(
                    "transaction ordered",
                    0,
                    "transaction",
                    r#"{"expiration":"2009-02-13T23:31:31.000","ref_block_num":1234,"ref_block_prefix":5678,"max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}"#,
                    r#"{"expiration":"2009-02-13T23:31:31.000","ref_block_num":1234,"ref_block_prefix":5678,"max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}"#,
                ),
                CodecRow::success(
                    "transaction unordered",
                    0,
                    "transaction",
                    r#"{"ref_block_num":1234,"ref_block_prefix":5678,"expiration":"2009-02-13T23:31:31.000","max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}"#,
                    r#"{"expiration":"2009-02-13T23:31:31.000","ref_block_num":1234,"ref_block_prefix":5678,"max_net_usage_words":0,"max_cpu_usage_ms":0,"delay_sec":0,"context_free_actions":[],"actions":[{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"}],"transaction_extensions":[]}"#,
                ),
                CodecRow::success("contract zero builtin", 0, "uint8", "1", "1"),
            ],
        );

        let fresh_rust = Abieos::new();
        let fresh_oracle = Oracle::new();
        fresh_rust
            .set_abi_json_native(0, r#"{"version": "eosio::abi/1.1"}"#)
            .unwrap();
        fresh_oracle
            .set_abi_json(0, r#"{"version": "eosio::abi/1.1"}"#)
            .unwrap();
        assert_eq!(
            fresh_rust.json_to_hex_native(0, "uint8", "1").unwrap(),
            "01"
        );
        assert_eq!(fresh_oracle.json_to_hex("uint8", "1").unwrap(), "01");
        assert_eq!(fresh_oracle.hex_to_json("uint8", "01").unwrap(), "1");
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_duplicate_and_extra_fields() {
        let rust = Abieos::new();
        let oracle = Oracle::new();
        let contract = rust.string_to_name(TEST_ABI_CONTRACT).unwrap();
        assert_eq!(contract, oracle.string_to_name(TEST_ABI_CONTRACT));
        rust.set_abi_json(TEST_ABI_CONTRACT, TEST_ABI).unwrap();
        oracle.set_abi_json(contract, TEST_ABI).unwrap();

        compare_codec_rows(
            &rust,
            &oracle,
            &[
                // Duplicate fields: C++ std::map overwrites → last value wins.
                // {"x1":1, "x1":2} → x1=2
                CodecRow::success(
                    "duplicate field last-wins",
                    contract,
                    "s1",
                    r#"{"x1":1, "x1":2}"#,
                    r#"{"x1":2}"#,
                ),
                // Extra fields: C++ reorderable silently ignores extra keys.
                CodecRow::success(
                    "extra field ignored in reorderable",
                    contract,
                    "s1",
                    r#"{"x1":5, "extra":99}"#,
                    r#"{"x1":5}"#,
                ),
                // Duplicate + extra combined
                CodecRow::success(
                    "duplicate and extra fields together",
                    contract,
                    "s1",
                    r#"{"x1":1, "extra":42, "x1":3}"#,
                    r#"{"x1":3}"#,
                ),
                // All-extension struct with only extra fields: both fields are
                // extensions, so they can be skipped, and the extra field is ignored.
                CodecRow::success(
                    "all-extension struct with extra field only",
                    contract,
                    "s4",
                    r#"{"foo":7}"#,
                    r#"{}"#,
                ),
            ],
        );
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_missing_field_errors() {
        let rust = Abieos::new();
        let oracle = Oracle::new();
        let contract = rust.string_to_name(TEST_ABI_CONTRACT).unwrap();
        assert_eq!(contract, oracle.string_to_name(TEST_ABI_CONTRACT));
        rust.set_abi_json(TEST_ABI_CONTRACT, TEST_ABI).unwrap();
        oracle.set_abi_json(contract, TEST_ABI).unwrap();

        compare_codec_rows(
            &rust,
            &oracle,
            &[
                // Missing required field
                CodecRow::failure("s5 missing all fields", contract, "s5", r#"{}"#),
                CodecRow::failure("s5 missing x2 and x3", contract, "s5", r#"{"x1":1}"#),
                CodecRow::failure("s5 missing x3", contract, "s5", r#"{"x1":1,"x2":2}"#),
                // Not an object
                CodecRow::failure("s1 rejects null", contract, "s1", "null"),
                CodecRow::failure("s1 rejects array", contract, "s1", "[1]"),
            ],
        );
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_trailing_and_malformed_json() {
        let rust = Abieos::new();
        let oracle = Oracle::new();

        // Trailing content after valid JSON (native types, no contract needed)
        let trailing_cases: &[(&str, &str, &str)] = &[
            ("int8 trailing text", "int8", "1 extra"),
            ("int8 trailing brace", "int8", "1}"),
            ("string trailing text", "string", r#""hello" extra"#),
            ("bool trailing comma", "bool", "true,"),
        ];

        for &(label, ty, json) in trailing_cases {
            let rust_result = rust
                .json_to_hex_native(0, ty, json)
                .map_err(|e| e.to_string());
            let oracle_result = oracle.json_to_hex(ty, json);
            assert_eq!(
                result_status(&rust_result),
                result_status(&oracle_result),
                "trailing content status mismatch for {label}: rust={rust_result:?} oracle={oracle_result:?}"
            );
            assert!(
                rust_result.is_err(),
                "trailing content should fail for {label}"
            );
        }

        // Invalid escape sequences
        let escape_cases: &[(&str, &str, &str)] = &[
            ("invalid backslash-x", "string", r#""\x""#),
            ("incomplete unicode escape", "string", r#""\u12""#),
            ("bare backslash at end", "string", r#""\"#),
        ];

        for &(label, ty, json) in escape_cases {
            let rust_result = rust
                .json_to_hex_native(0, ty, json)
                .map_err(|e| e.to_string());
            let oracle_result = oracle.json_to_hex(ty, json);
            assert_eq!(
                result_status(&rust_result),
                result_status(&oracle_result),
                "invalid escape status mismatch for {label}: rust={rust_result:?} oracle={oracle_result:?}"
            );
            assert!(
                rust_result.is_err(),
                "invalid escape should fail for {label}"
            );
        }

        // Known divergence: trailing whitespace after a scalar value.
        // Rust's custom parser skips trailing whitespace (line 122-124 in
        // rust.rs), matching the JSON spec.  C++ uses RapidJSON's SAX
        // streaming parser which, for native types, rejects trailing
        // whitespace as the SAX parser internally calls `complete()` which
        // expects full consumption.  We assert Rust succeeds independently.
        let rust_ws = rust
            .json_to_hex_native(0, "int8", "1   ")
            .map_err(|e| e.to_string());
        assert!(rust_ws.is_ok(), "Rust should accept trailing whitespace");
        assert_eq!(rust_ws.unwrap(), "01");
    }

    #[test]
    fn rust_backend_matches_cpp_oracle_for_binary_overrun_edges() {
        let rust = Abieos::new();
        let oracle = Oracle::new();
        let contract = rust.string_to_name(TEST_ABI_CONTRACT).unwrap();
        assert_eq!(contract, oracle.string_to_name(TEST_ABI_CONTRACT));
        rust.set_abi_json(TEST_ABI_CONTRACT, TEST_ABI).unwrap();
        oracle.set_abi_json(contract, TEST_ABI).unwrap();

        // Additional binary overrun cases that test specific edge boundaries
        let overrun_cases: &[(&str, u64, &str, &str)] = &[
            // Completely empty binary for various types
            ("int8 empty", 0, "int8", ""),
            ("int32 empty", 0, "int32", ""),
            ("int64 empty", 0, "int64", ""),
            ("float32 empty", 0, "float32", ""),
            ("float64 empty", 0, "float64", ""),
            ("name empty", 0, "name", ""),
            // One byte short for multi-byte types
            ("int16 one byte", 0, "int16", "01"),
            ("int32 three bytes", 0, "int32", "010203"),
            ("int64 seven bytes", 0, "int64", "01020304050607"),
            ("float32 three bytes", 0, "float32", "010203"),
            ("float64 seven bytes", 0, "float64", "01020304050607"),
            // Struct with partial data
            ("s5 one byte", contract, "s5", "01"),
            ("s5 two bytes", contract, "s5", "0102"),
            // Variant with invalid index
            ("v1 invalid index", contract, "v1", "FF"),
        ];

        for &(label, contract_id, ty, hex) in overrun_cases {
            let rust_result = if contract_id == 0 {
                rust.hex_to_json_native(0, ty, hex)
                    .map_err(|e| e.to_string())
            } else {
                rust.hex_to_json(TEST_ABI_CONTRACT, ty, hex)
                    .map_err(|e| e.to_string())
            };
            let oracle_result = oracle.hex_to_json_contract(contract_id, ty, hex);
            assert_eq!(
                result_status(&rust_result),
                result_status(&oracle_result),
                "binary overrun status mismatch for {label}: rust={rust_result:?} oracle={oracle_result:?}"
            );
        }
    }
    #[test]
    fn rust_backend_matches_cpp_oracle_for_randomized_names() {
        let rust = Abieos::new();
        let oracle = Oracle::new();

        let chars = ".12345abcdefghijklmnopqrstuvwxyz";
        let mut seed = 12345u64;

        for _ in 0..1000 {
            let mut name = String::new();
            let len = (seed % 14) as usize; // up to 13 chars
            seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);

            for _ in 0..len {
                let idx = (seed % (chars.len() as u64)) as usize;
                name.push(chars.as_bytes()[idx] as char);
                seed = seed.wrapping_mul(6364136223846793005).wrapping_add(1);
            }

            let rust_name = rust.string_to_name(&name);
            let oracle_name = oracle.string_to_name(&name);

            if let Ok(r) = rust_name {
                assert_eq!(
                    r, oracle_name,
                    "string_to_name mismatch for random name '{}'",
                    name
                );
                let r_str = rust.name_to_string(r).unwrap();
                let o_str = oracle.name_to_string(oracle_name).unwrap();
                assert_eq!(
                    r_str, o_str,
                    "name_to_string mismatch for random name '{}'",
                    name
                );
            }
        }
    }
}
