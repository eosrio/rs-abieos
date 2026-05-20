#![allow(dead_code)]
#[cfg(all(feature = "rust-backend", feature = "cpp-oracle"))]
mod parity_measurer {
    use rs_abieos::{cpp_oracle, Abieos};
    use std::ffi::{CStr, CString};

    pub const TEST_ABI_CONTRACT: &str = "test.abi";
    pub const TEST_ABI: &str = r#"{
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
    pub struct CodecRow {
        pub label: &'static str,
        pub contract: u64,
        pub ty: &'static str,
        pub json: &'static str,
        pub expected_json: Option<&'static str>,
    }

    impl CodecRow {
        pub const fn success(
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

        pub const fn failure(
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

    pub struct Oracle {
        ctx: *mut cpp_oracle::abieos_context,
    }

    impl Oracle {
        pub fn new() -> Self {
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

        fn capture_bin_hex(&self) -> Result<String, String> {
            unsafe { self.capture_string(cpp_oracle::abieos_get_bin_hex(self.ctx)) }
        }

        pub fn string_to_name(&self, name: &str) -> u64 {
            let name = CString::new(name).unwrap();
            unsafe { cpp_oracle::abieos_string_to_name(self.ctx, name.as_ptr()) }
        }

        pub fn name_to_string(&self, name: u64) -> Result<String, String> {
            unsafe { self.capture_string(cpp_oracle::abieos_name_to_string(self.ctx, name)) }
        }

        pub fn set_abi_json(&self, contract: u64, abi: &str) -> Result<(), String> {
            let abi = CString::new(abi).unwrap();
            unsafe {
                self.capture_status(cpp_oracle::abieos_set_abi(self.ctx, contract, abi.as_ptr()))
            }
        }

        pub fn json_to_hex_contract(
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

        pub fn hex_to_json_contract(
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

    pub struct CategoryStats {
        pub name: &'static str,
        pub total: usize,
        pub matching: usize,
    }

    pub fn measure_all() {
        println!("============================================================");
        println!("         rs-abieos RUST VS C++ PARITY REPORT                ");
        println!("============================================================");

        let rust = Abieos::new();
        let oracle = Oracle::new();

        // Establish contract zero ABIs to prevent 'not loaded' errors on C++ side
        rust.set_abi_json_native(0, r#"{"version": "eosio::abi/1.1"}"#)
            .unwrap();
        oracle
            .set_abi_json(0, r#"{"version": "eosio::abi/1.1"}"#)
            .unwrap();

        // 1. Builtins Category
        let builtin_rows = [
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
                "int128 min",
                0,
                "int128",
                "-170141183460469231731687303715884105728",
                r#""-170141183460469231731687303715884105728""#,
            ),
            CodecRow::success(
                "uint128 max",
                0,
                "uint128",
                "340282366920938463463374607431768211455",
                r#""340282366920938463463374607431768211455""#,
            ),
            CodecRow::success("varuint32 max", 0, "varuint32", "4294967295", "4294967295"),
            CodecRow::success("varint32 min", 0, "varint32", "-2147483648", "-2147483648"),
            CodecRow::success("float32 simple", 0, "float32", "0.125", "0.125"),
            CodecRow::success("float64 simple", 0, "float64", "-0.125", "-0.125"),
            CodecRow::success(
                "float128 hex",
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
            CodecRow::success(
                "name dot suffix",
                0,
                "name",
                r#""..ab.cd.ef..""#,
                r#""..ab.cd.ef""#,
            ),
            CodecRow::success(
                "bytes hex representation",
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
        ];

        let mut stats = vec![];
        stats.push(run_category(
            "Built-in Types (Scalars & Builtins)",
            &rust,
            &oracle,
            &builtin_rows,
        ));

        // 2. Structs and ABIs Category
        let contract = rust.string_to_name(TEST_ABI_CONTRACT).unwrap();
        rust.set_abi_json(TEST_ABI_CONTRACT, TEST_ABI).unwrap();
        oracle.set_abi_json(contract, TEST_ABI).unwrap();

        let struct_rows = [
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
                "nested fixed array",
                contract,
                "s9",
                r#"{"a1":[{"x1":6},{"x1":16}]}"#,
                r#"{"a1":[{"x1":6},{"x1":16}]}"#,
            ),
            CodecRow::success(
                "bitset input",
                contract,
                "s7",
                r#"{"bs":"110001011"}"#,
                r#"{"bs":"110001011"}"#,
            ),
            CodecRow::success("optional extension none", contract, "s3", r#"{}"#, r#"{}"#),
            CodecRow::success(
                "optional extension partial",
                contract,
                "s3",
                r#"{"z1":7,"z2":["int8",6]}"#,
                r#"{"z1":7,"z2":["int8",6]}"#,
            ),
            CodecRow::success(
                "optional extension full",
                contract,
                "s3",
                r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9,"y2":10}}"#,
                r#"{"z1":7,"z2":["int8",6],"z3":{"y1":9,"y2":10}}"#,
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
        ];
        stats.push(run_category(
            "Complex Structs, Variants & Bitsets",
            &rust,
            &oracle,
            &struct_rows,
        ));

        // 3. New Fixtures (testkv, packed_trx, ship)
        let testkv_contract = rust.string_to_name("testkv").unwrap();
        let testkv_abi = include_str!("../abis/testkv.abi.json");
        rust.set_abi_json("testkv", testkv_abi).unwrap();
        oracle.set_abi_json(testkv_contract, testkv_abi).unwrap();

        let packed_tx_contract = rust.string_to_name("packed.tx").unwrap();
        let packed_tx_abi = include_str!("../abis/packed_transaction.abi.json");
        rust.set_abi_json("packed.tx", packed_tx_abi).unwrap();
        oracle
            .set_abi_json(packed_tx_contract, packed_tx_abi)
            .unwrap();

        let ship_contract = rust.string_to_name("ship").unwrap();
        let ship_abi = include_str!("../abis/ship.abi.json");
        rust.set_abi_json("ship", ship_abi).unwrap();
        oracle.set_abi_json(ship_contract, ship_abi).unwrap();

        let new_fixture_rows = [
            CodecRow::success(
                "testkv my_struct",
                testkv_contract,
                "my_struct",
                r#"{"primary":"user1","foo":"hello","bar":123456,"fullname":"Igor Lins","age":30}"#,
                r#"{"primary":"user1","foo":"hello","bar":"123456","fullname":"Igor Lins","age":30}"#,
            ),
            CodecRow::success(
                "testkv tuple",
                testkv_contract,
                "tuple_string_uint32",
                r#"{"field_0":"test","field_1":999}"#,
                r#"{"field_0":"test","field_1":999}"#,
            ),
            CodecRow::success(
                "packed_tx action",
                packed_tx_contract,
                "action",
                r#"{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6"}"#,
                r#"{"account":"eosio.token","name":"transfer","authorization":[{"actor":"useraaaaaaaa","permission":"active"}],"data":"608C31C6187315D6"}"#,
            ),
            CodecRow::success(
                "ship block_position",
                ship_contract,
                "block_position",
                r#"{"block_num":1000,"block_id":"000003E800000000000000000000000000000000000000000000000000000000"}"#,
                r#"{"block_num":1000,"block_id":"000003E800000000000000000000000000000000000000000000000000000000"}"#,
            ),
        ];
        stats.push(run_category(
            "New Parity Fixtures (Ship, testkv, packed_trx)",
            &rust,
            &oracle,
            &new_fixture_rows,
        ));

        // 4. Error/Failure Boundaries
        let failure_rows = [
            CodecRow::failure("uint8 overflow", 0, "uint8", "256"),
            CodecRow::failure("int8 underflow", 0, "int8", "-129"),
            CodecRow::failure("varuint32 negative", 0, "varuint32", "-1"),
            CodecRow::failure("invalid bytes formatting", 0, "bytes", r#""0""#),
            CodecRow::failure("checksum256 bad length", 0, "checksum256", r#""a0""#),
            CodecRow::failure("symbol_code lowercase", 0, "symbol_code", r#""lower""#),
            CodecRow::failure("asset null is invalid", 0, "asset", "null"),
            CodecRow::failure("fixed array too short", contract, "s8", r#"{"a1":[1]}"#),
            CodecRow::failure("fixed array too long", contract, "s8", r#"{"a1":[1,2,3]}"#),
            CodecRow::failure(
                "public key bad checksum",
                contract,
                "public_key_holder",
                r#"{"key":"PUB_K1_11111111111111111111111111111111149Mr2S"}"#,
            ),
        ];
        stats.push(run_category(
            "Boundary Errors & Input Validations",
            &rust,
            &oracle,
            &failure_rows,
        ));

        // Print final reports
        println!("============================================================");
        println!("                    CATEGORY BREAKDOWN                      ");
        println!("============================================================");
        let mut total_tests = 0;
        let mut total_matching = 0;
        for cat in &stats {
            let pct = (cat.matching as f64 / cat.total as f64) * 100.0;
            println!(
                "* {:<44} | {:>2}/{:<2} ({:.2}%)",
                cat.name, cat.matching, cat.total, pct
            );
            total_tests += cat.total;
            total_matching += cat.matching;
        }

        let overall_pct = (total_matching as f64 / total_tests as f64) * 100.0;
        println!("------------------------------------------------------------");
        println!(
            "  OVERALL PARITY SCORE                      | {:>2}/{:<2} ({:.2}%)",
            total_matching, total_tests, overall_pct
        );
        println!("============================================================");
        if total_tests == total_matching {
            println!("  STATUS: 100% PERFECT PARITY ACHIEVED! 🎉                  ");
        } else {
            println!("  STATUS: PARITY MISMATCHES DETECTED. ⚠️                     ");
        }
        println!("============================================================");
    }

    fn run_category(
        name: &'static str,
        rust: &Abieos,
        oracle: &Oracle,
        rows: &[CodecRow],
    ) -> CategoryStats {
        let mut matching = 0;
        for row in rows {
            let mut matched = true;

            let rust_hex = rust
                .json_to_hex_native(row.contract, row.ty, row.json)
                .map_err(|e| e.to_string());
            let oracle_hex = oracle.json_to_hex_contract(row.contract, row.ty, row.json);

            if result_status(&rust_hex) != result_status(&oracle_hex) {
                matched = false;
            }

            match (rust_hex, oracle_hex, row.expected_json) {
                (Ok(r_hex), Ok(o_hex), Some(expected_json)) => {
                    if r_hex != o_hex {
                        matched = false;
                    }

                    let rust_json = rust
                        .hex_to_json_native(row.contract, row.ty, &r_hex)
                        .map_err(|e| e.to_string());
                    let oracle_json = oracle.hex_to_json_contract(row.contract, row.ty, &o_hex);

                    if result_status(&rust_json) != result_status(&oracle_json) {
                        matched = false;
                    }

                    if let (Ok(r_js), Ok(o_js)) = (rust_json, oracle_json) {
                        if r_js != o_js || r_js != expected_json {
                            matched = false;
                        }
                    } else {
                        matched = false;
                    }
                }
                (Err(r_err), Err(o_err), None) => {
                    if r_err.is_empty() || o_err.is_empty() {
                        matched = false;
                    }
                }
                _ => {
                    matched = false;
                }
            }

            if matched {
                matching += 1;
            } else {
                println!("  [MISMATCH] Row '{}': Rust and C++ disagreed.", row.label);
            }
        }

        CategoryStats {
            name,
            total: rows.len(),
            matching,
        }
    }
}

fn main() {
    #[cfg(all(feature = "rust-backend", feature = "cpp-oracle"))]
    parity_measurer::measure_all();

    #[cfg(not(all(feature = "rust-backend", feature = "cpp-oracle")))]
    {
        println!(
            "Error: To run the parity measurer script, you must enable all required features."
        );
        println!("Please run the following command:");
        println!(
            "  cargo run --example measure_parity --target x86_64-pc-windows-gnu --all-features"
        );
    }
}
