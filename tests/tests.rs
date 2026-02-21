#[cfg(test)]
mod tests {
    use rs_abieos::Abieos;
    use crate::samples::{BIN_ACTION_TRANSFER, EOSIO_TOKEN_HEX_ABI, EOSIO_TOKEN_U64, HEX_ACTION_TRANSFER};

    #[test]
    fn new() {
        let abieos: Abieos = Abieos::new();
        assert_eq!(abieos.context.unwrap().is_null(), false, "new test failed");
        abieos.destroy();
    }

    #[test]
    fn new_from_context() {
        let old_abieos: Abieos = Abieos::new();
        let abieos: Abieos = Abieos::from_context(old_abieos.context.unwrap());
        assert_eq!(abieos.context.unwrap().is_null(), false, "new_from_context test failed");
        abieos.destroy();
    }

    #[test]
    fn string_to_name() {
        let abieos = Abieos::new();
        let true_name = EOSIO_TOKEN_U64;
        let name = abieos.string_to_name("eosio.token");
        match name {
            Ok(n) => assert_eq!(n, true_name),
            Err(e) => panic!("string_to_name test failed: {}", e)
        }
        abieos.destroy();
    }

    #[test]
    fn name_to_string() {
        let abieos = Abieos::new();
        let name = abieos.name_to_string(EOSIO_TOKEN_U64).unwrap();
        let true_name = "eosio.token".to_string();
        assert_eq!(true_name, name, "reverse name test for eosio.token - expecting: {} got {}", true_name, name);
        abieos.destroy();
    }

    #[test]
    fn set_abi_json() {
        let abi_data = match std::fs::read_to_string("abis/eosio.abi") {
            Ok(data) => data,
            Err(e) => panic!("load abi file failed: {}", e)
        };

        let abieos: Abieos = Abieos::new();
        match abieos.set_abi_json("eosio", &abi_data) {
            Ok(x) => assert_eq!(x, true, "load abi test"),
            Err(e) => panic!("set_abi_json failed: {}", e)
        }
        abieos.destroy();
    }

    #[test]
    fn set_abi_hex() {
        let abieos: Abieos = Abieos::new();
        match abieos.set_abi_hex("eosio", EOSIO_TOKEN_HEX_ABI) {
            Ok(x) => assert_eq!(x, true, "load abi test"),
            Err(e) => panic!("set_abi_hex failed: {}", e)
        }
        abieos.destroy();
    }

    #[test]
    fn set_abi_bin() {
        let abieos: Abieos = Abieos::new();

        // load abi binary from file at "abis/eosio.abi.bin"
        let abi_data = match std::fs::read("abis/eosio.abi.bin") {
            Ok(data) => data,
            Err(e) => panic!("load abi binary file failed: {}", e)
        };

        match abieos.set_abi_bin("eosio", &abi_data) {
            Ok(x) => assert_eq!(x, true, "load abi test"),
            Err(e) => panic!("set_abi_bin failed: {}", e)
        }

        abieos.destroy();
    }


    #[test]
    fn json_to_hex() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let json = r#"
        {
            "from":"alice",
            "to":"bob",
            "quantity":"1.0000 EOS",
            "memo":"Hello!"
        }"#;
        let bin = abieos.json_to_hex("eosio.token", "transfer", json).unwrap();
        assert_eq!(bin, HEX_ACTION_TRANSFER);
        abieos.destroy();
    }

    #[test]
    fn json_to_bin() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let json = r#"
        {
            "from":"alice",
            "to":"bob",
            "quantity":"1.0000 EOS",
            "memo":"Hello!"
        }"#;
        let bin = abieos.json_to_bin("eosio.token", "transfer", json).unwrap();
        assert_eq!(bin, BIN_ACTION_TRANSFER);
        abieos.destroy();
    }

    #[test]
    fn hex_to_json() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let bin = HEX_ACTION_TRANSFER;
        let json = abieos.hex_to_json("eosio.token", "transfer", bin).unwrap();
        let reverse_bin = abieos.json_to_hex("eosio.token", "transfer", &json).unwrap();
        assert_eq!(bin, reverse_bin);
        abieos.destroy();
    }

    #[test]
    fn get_type_for_action() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let action = "transfer";
        let type_name = abieos.get_type_for_action("eosio.token", action).unwrap();
        assert_eq!(type_name, "transfer");
        abieos.destroy();
    }

    #[test]
    fn get_type_for_action_invalid() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let action = "invalid";
        let type_name = abieos.get_type_for_action("eosio.token", action);
        assert_eq!(type_name.is_err(), true);
        abieos.destroy();
    }

    #[test]
    fn get_type_for_action_invalid_contract() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let action = "transfer";
        let type_name = abieos.get_type_for_action("invalid", action);
        assert_eq!(type_name.is_err(), true);
        abieos.destroy();
    }

    #[test]
    fn get_type_for_table() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let table = "accounts";
        let type_name = abieos.get_type_for_table("eosio.token", table).unwrap();
        assert_eq!(type_name, "account");
        abieos.destroy();
    }

    #[test]
    fn get_type_for_table_invalid() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let table = "invalid";
        let type_name = abieos.get_type_for_table("eosio.token", table);
        assert_eq!(type_name.is_err(), true);
        abieos.destroy();
    }


    #[test]
    fn get_type_for_table_invalid_contract() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let table = "accounts";
        let type_name = abieos.get_type_for_table("invalid", table);
        assert_eq!(type_name.is_err(), true);
        abieos.destroy();
    }

    #[ignore]
    #[test]
    fn get_type_for_action_result() {
        // todo: Find example for action result
    }

    #[test]
    fn abi_json_to_bin() {
        let abieos: Abieos = Abieos::new();

        // load abi binary from file at "abis/eosio.abi.bin"
        let abi_data = match std::fs::read("abis/eosio.abi.bin") {
            Ok(data) => data,
            Err(e) => panic!("load abi binary file failed: {}", e)
        };

        // load abi json from file at "abis/eosio.abi.bin"
        let json_data = match std::fs::read_to_string("abis/eosio.abi") {
            Ok(data) => data,
            Err(e) => panic!("load abi json file failed: {}", e)
        };


        // convert abi binary to json
        let abi = match abieos.abi_json_to_bin(&json_data) {
            Ok(x) => x,
            Err(_) => panic!("abi_json_to_bin test failed"),
        };

        assert_eq!(abi, abi_data, "abi_json_to_bin test failed");
        abieos.destroy();
    }

    #[test]
    fn abi_bin_to_json() {
        let abieos: Abieos = Abieos::new();

        // load abi from file at "abis/eosio.abi.bin"
        let abi_data = match std::fs::read("abis/eosio.abi.bin") {
            Ok(data) => data,
            Err(e) => panic!("load abi file failed: {}", e)
        };

        // convert abi binary to json
        let json_abi = match abieos.abi_bin_to_json(&abi_data) {
            Ok(x) => x,
            Err(_) => panic!("abi_bin_to_json test failed"),
        };
        assert!(json_abi.starts_with('{') && json_abi.ends_with('}'));
        abieos.destroy();
    }

    // --- Default trait ---

    #[test]
    fn default_trait() {
        let abieos: Abieos = Abieos::default();
        assert!(!abieos.context.unwrap().is_null(), "default trait should create a valid context");
        abieos.destroy();
    }

    // --- bin_to_json ---

    #[test]
    fn bin_to_json() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();
        let json = abieos.bin_to_json("eosio.token", "transfer", BIN_ACTION_TRANSFER).unwrap();
        // round-trip: bin -> json -> bin
        let bin = abieos.json_to_bin("eosio.token", "transfer", &json).unwrap();
        assert_eq!(bin, BIN_ACTION_TRANSFER);
        abieos.destroy();
    }

    // --- delete_contract ---

    #[test]
    fn delete_contract() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();

        // contract exists, should delete successfully
        let result = abieos.delete_contract("eosio.token").unwrap();
        assert!(result, "delete_contract should return true for existing contract");

        // contract no longer exists, get_type_for_action should fail
        let type_result = abieos.get_type_for_action("eosio.token", "transfer");
        assert!(type_result.is_err(), "should fail after contract is deleted");

        abieos.destroy();
    }

    #[test]
    fn delete_contract_nonexistent() {
        let abieos: Abieos = Abieos::new();
        // deleting a contract that was never loaded
        let result = abieos.delete_contract("eosio.token").unwrap();
        assert!(!result, "delete_contract should return false for non-existent contract");
        abieos.destroy();
    }

    #[test]
    fn delete_contract_native() {
        let abieos: Abieos = Abieos::new();
        let token_u64 = abieos.string_to_name("eosio.token").unwrap();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI).unwrap();

        let result = abieos.delete_contract_native(token_u64).unwrap();
        assert!(result, "delete_contract_native should return true for existing contract");
        abieos.destroy();
    }

    // --- Native (u64) variants ---

    #[test]
    fn set_abi_json_native() {
        let abi_data = std::fs::read_to_string("abis/eosio.abi").unwrap();
        let abieos: Abieos = Abieos::new();
        let eosio_u64 = abieos.string_to_name("eosio").unwrap();
        let result = abieos.set_abi_json_native(eosio_u64, &abi_data).unwrap();
        assert!(result);
        abieos.destroy();
    }

    #[test]
    fn set_abi_hex_native() {
        let abieos: Abieos = Abieos::new();
        let token_u64 = abieos.string_to_name("eosio.token").unwrap();
        let result = abieos.set_abi_hex_native(token_u64, EOSIO_TOKEN_HEX_ABI).unwrap();
        assert!(result);
        abieos.destroy();
    }

    #[test]
    fn set_abi_bin_native() {
        let abieos: Abieos = Abieos::new();
        let eosio_u64 = abieos.string_to_name("eosio").unwrap();
        let abi_data = std::fs::read("abis/eosio.abi.bin").unwrap();
        let result = abieos.set_abi_bin_native(eosio_u64, &abi_data).unwrap();
        assert!(result);
        abieos.destroy();
    }

    #[test]
    fn json_to_hex_native() {
        let abieos: Abieos = Abieos::new();
        let token_u64 = abieos.string_to_name("eosio.token").unwrap();
        abieos.set_abi_hex_native(token_u64, EOSIO_TOKEN_HEX_ABI).unwrap();
        let json = r#"{"from":"alice","to":"bob","quantity":"1.0000 EOS","memo":"Hello!"}"#;
        let hex = abieos.json_to_hex_native(token_u64, "transfer", json).unwrap();
        assert_eq!(hex, HEX_ACTION_TRANSFER);
        abieos.destroy();
    }

    #[test]
    fn hex_to_json_native() {
        let abieos: Abieos = Abieos::new();
        let token_u64 = abieos.string_to_name("eosio.token").unwrap();
        abieos.set_abi_hex_native(token_u64, EOSIO_TOKEN_HEX_ABI).unwrap();
        let json = abieos.hex_to_json_native(token_u64, "transfer", HEX_ACTION_TRANSFER).unwrap();
        // round-trip: hex -> json -> hex
        let hex = abieos.json_to_hex_native(token_u64, "transfer", &json).unwrap();
        assert_eq!(hex, HEX_ACTION_TRANSFER);
        abieos.destroy();
    }

    // --- AbieosContract workflow ---

    #[test]
    fn contract_with_string_ref() {
        let abieos: Abieos = Abieos::new();
        let name = "eosio.token";
        let mut contract = abieos.contract(rs_abieos::NameLike::StringRef(&name));
        contract.load_abi(rs_abieos::AbiLike::Hex(EOSIO_TOKEN_HEX_ABI.to_string())).unwrap();
        assert!(contract.abiLoaded);

        let datatype = contract.get_type_for_action("transfer").unwrap();
        assert_eq!(datatype, "transfer");
        abieos.destroy();
    }

    #[test]
    fn contract_with_string() {
        let abieos: Abieos = Abieos::new();
        let mut contract = abieos.contract(rs_abieos::NameLike::String("eosio.token".to_string()));
        contract.load_abi(rs_abieos::AbiLike::Hex(EOSIO_TOKEN_HEX_ABI.to_string())).unwrap();
        assert!(contract.abiLoaded);
        abieos.destroy();
    }

    #[test]
    fn contract_with_u64() {
        let abieos: Abieos = Abieos::new();
        let mut contract = abieos.contract(rs_abieos::NameLike::U64(EOSIO_TOKEN_U64));
        contract.load_abi(rs_abieos::AbiLike::Hex(EOSIO_TOKEN_HEX_ABI.to_string())).unwrap();
        assert!(contract.abiLoaded);
        abieos.destroy();
    }

    #[test]
    fn contract_with_i32() {
        let abieos: Abieos = Abieos::new();
        let contract = abieos.contract(rs_abieos::NameLike::I32(1));
        assert!(!contract.abiLoaded);
        // i32(1) is a valid (if unusual) contract name
        let _ = contract;
        abieos.destroy();
    }

    #[test]
    fn contract_load_json_file() {
        let abieos: Abieos = Abieos::new();
        let name = "eosio";
        let mut contract = abieos.contract(rs_abieos::NameLike::StringRef(&name));
        contract.load_json_file("abis/eosio.abi").unwrap();
        assert!(contract.abiLoaded);
        abieos.destroy();
    }

    #[test]
    fn contract_load_abi_json() {
        let abieos: Abieos = Abieos::new();
        let abi_data = std::fs::read_to_string("abis/eosio.abi").unwrap();
        let name = "eosio";
        let mut contract = abieos.contract(rs_abieos::NameLike::StringRef(&name));
        contract.load_abi(rs_abieos::AbiLike::Json(abi_data)).unwrap();
        assert!(contract.abiLoaded);
        abieos.destroy();
    }

    #[test]
    fn contract_load_abi_bin() {
        let abieos: Abieos = Abieos::new();
        let abi_data = std::fs::read("abis/eosio.abi.bin").unwrap();
        let name = "eosio";
        let mut contract = abieos.contract(rs_abieos::NameLike::StringRef(&name));
        contract.load_abi(rs_abieos::AbiLike::Bin(abi_data)).unwrap();
        assert!(contract.abiLoaded);
        abieos.destroy();
    }

    #[test]
    fn contract_json_to_hex() {
        let abieos: Abieos = Abieos::new();
        let name = "eosio.token";
        let mut contract = abieos.contract(rs_abieos::NameLike::StringRef(&name));
        contract.load_abi(rs_abieos::AbiLike::Hex(EOSIO_TOKEN_HEX_ABI.to_string())).unwrap();

        let json = r#"{"from":"alice","to":"bob","quantity":"1.0000 EOS","memo":"Hello!"}"#;
        let hex = contract.json_to_hex("transfer", json).unwrap();
        assert_eq!(hex, HEX_ACTION_TRANSFER);
        abieos.destroy();
    }

    #[test]
    fn contract_hex_to_json() {
        let abieos: Abieos = Abieos::new();
        let name = "eosio.token";
        let mut contract = abieos.contract(rs_abieos::NameLike::StringRef(&name));
        contract.load_abi(rs_abieos::AbiLike::Hex(EOSIO_TOKEN_HEX_ABI.to_string())).unwrap();

        let json = contract.hex_to_json("transfer", HEX_ACTION_TRANSFER).unwrap();
        // round-trip verification
        let hex = contract.json_to_hex("transfer", &json).unwrap();
        assert_eq!(hex, HEX_ACTION_TRANSFER);
        abieos.destroy();
    }

    #[test]
    fn contract_get_type_for_table() {
        let abieos: Abieos = Abieos::new();
        let name = "eosio.token";
        let mut contract = abieos.contract(rs_abieos::NameLike::StringRef(&name));
        contract.load_abi(rs_abieos::AbiLike::Hex(EOSIO_TOKEN_HEX_ABI.to_string())).unwrap();

        let table_type = contract.get_type_for_table("accounts").unwrap();
        assert_eq!(table_type, "account");
        abieos.destroy();
    }

    // --- C-string variants ---

    #[test]
    fn c_string_to_name() {
        let abieos: Abieos = Abieos::new();
        let name = std::ffi::CStr::from_bytes_with_nul(b"eosio.token\0").unwrap();
        let name_u64 = abieos.c_string_to_name(name);
        assert_eq!(name_u64, EOSIO_TOKEN_U64);
        abieos.destroy();
    }

    #[test]
    fn name_to_cstr() {
        let abieos: Abieos = Abieos::new();
        let cstr = abieos.name_to_cstr(EOSIO_TOKEN_U64);
        assert_eq!(cstr.to_str().unwrap(), "eosio.token");
        abieos.destroy();
    }

    #[test]
    fn string_to_name_too_long() {
        let abieos: Abieos = Abieos::new();
        let result = abieos.string_to_name("thisnamewaytolong");
        assert!(result.is_err(), "names longer than 13 chars should fail");
        abieos.destroy();
    }
}

mod samples {
    pub const EOSIO_TOKEN_U64: u64 = 6138663591592764928;
    pub const EOSIO_TOKEN_HEX_ABI: &str = "0e656f73696f3a3a6162692f312e30010c6163636f756e745f6e616d65046e616d6505087472616e7366657200040466726f6d0c6163636f756e745f6e616d6502746f0c6163636f756e745f6e616d65087175616e74697479056173736574046d656d6f06737472696e67066372656174650002066973737565720c6163636f756e745f6e616d650e6d6178696d756d5f737570706c79056173736574056973737565000302746f0c6163636f756e745f6e616d65087175616e74697479056173736574046d656d6f06737472696e67076163636f756e7400010762616c616e63650561737365740e63757272656e63795f7374617473000306737570706c790561737365740a6d61785f737570706c79056173736574066973737565720c6163636f756e745f6e616d6503000000572d3ccdcd087472616e73666572000000000000a531760569737375650000000000a86cd445066372656174650002000000384f4d113203693634010863757272656e6379010675696e743634076163636f756e740000000000904dc603693634010863757272656e6379010675696e7436340e63757272656e63795f7374617473000000";

    pub const HEX_ACTION_TRANSFER: &str = "0000000000855C340000000000000E3D102700000000000004454F53000000000648656C6C6F21";

    pub const BIN_ACTION_TRANSFER: &[u8] = &[0, 0, 0, 0, 0, 133, 92, 52, 0, 0, 0, 0, 0, 0, 14, 61, 16, 39, 0, 0, 0, 0, 0, 0, 4, 69, 79, 83, 0, 0, 0, 0, 6, 72, 101, 108, 108, 111, 33];

    pub const _TEST_ABI_JSON: &str = r#"
        {
            "version": "eosio::abi/1.0",
            "structs": [
                {
                    "name": "s1",
                    "base": "",
                    "fields": [
                        {"name": "x1","types": "int8"}
                    ]
                },
                {
                    "name": "s2",
                    "base": "",
                    "fields": [
                        {"name": "y1","type": "int8$"},
                        {"name": "y2","type": "int8$"}
                    ]
                },
                {
                    "name": "s3",
                    "base": "",
                    "fields": [
                        {"name": "z1","type": "int8$"},
                        {"name": "z2","type": "v1$"},
                        {"name": "z3","type": "s2$"}
                    ]
                },
                {
                    "name": "s4",
                    "base": "",
                    "fields": [
                        {"name": "a1","type": "int8?$"},
                        {"name": "b1","type": "int8[]$"}
                    ]
                }
            ],
            "variants": [
                {
                    "name": "v1",
                    "types": ["int8","s1","s2"]
                }
            ]
        }"#;
}