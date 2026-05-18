#[cfg(all(feature = "rust-backend", feature = "cpp-oracle"))]
mod cpp_oracle_differential {
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
                "json_to_hex status mismatch for {}",
                row.label
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
                CodecRow::success("extension empty", contract, "s3", r#"{}"#, r#"{}"#),
                CodecRow::success(
                    "extension first field",
                    contract,
                    "s3",
                    r#"{"z1":7}"#,
                    r#"{"z1":7}"#,
                ),
                CodecRow::success(
                    "extension variant",
                    contract,
                    "s3",
                    r#"{"z1":7,"z2":["int8",6]}"#,
                    r#"{"z1":7,"z2":["int8",6]}"#,
                ),
                CodecRow::success(
                    "extension trailing empty skipped",
                    contract,
                    "s3",
                    r#"{"z1":7,"z2":["int8",6],"z3":{}}"#,
                    r#"{"z1":7,"z2":["int8",6]}"#,
                ),
                CodecRow::success(
                    "extension nested one field",
                    contract,
                    "s3",
                    r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9}}"#,
                    r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9}}"#,
                ),
                CodecRow::success(
                    "extension nested two fields",
                    contract,
                    "s3",
                    r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9,"y2":10}}"#,
                    r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9,"y2":10}}"#,
                ),
                CodecRow::success("optional extension empty", contract, "s4", r#"{}"#, r#"{}"#),
                CodecRow::success(
                    "optional extension null",
                    contract,
                    "s4",
                    r#"{"a1":null}"#,
                    r#"{"a1":null}"#,
                ),
                CodecRow::success(
                    "optional extension value",
                    contract,
                    "s4",
                    r#"{"a1":7}"#,
                    r#"{"a1":7}"#,
                ),
                CodecRow::success(
                    "optional extension array empty",
                    contract,
                    "s4",
                    r#"{"a1":null,"b1":[]}"#,
                    r#"{"a1":null,"b1":[]}"#,
                ),
                CodecRow::success(
                    "optional extension array values",
                    contract,
                    "s4",
                    r#"{"a1":null,"b1":[5,6,7]}"#,
                    r#"{"a1":null,"b1":[5,6,7]}"#,
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
        assert_eq!(
            fresh_rust.json_to_hex_native(0, "uint8", "1").unwrap(),
            "01"
        );
        assert_eq!(fresh_oracle.json_to_hex("uint8", "1").unwrap(), "01");
        assert_eq!(fresh_oracle.hex_to_json("uint8", "01").unwrap(), "1");
    }
}
