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
        match abieos.set_abi_json("eosio", abi_data) {
            Ok(x) => assert_eq!(x, true, "load abi test"),
            Err(e) => panic!("set_abi_json failed: {}", e)
        }
        abieos.destroy();
    }

    #[test]
    fn set_abi_hex() {
        let abieos: Abieos = Abieos::new();
        match abieos.set_abi_hex("eosio", EOSIO_TOKEN_HEX_ABI.to_string()) {
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

        match abieos.set_abi_bin("eosio", abi_data) {
            Ok(x) => assert_eq!(x, true, "load abi test"),
            Err(e) => panic!("set_abi_bin failed: {}", e)
        }

        abieos.destroy();
    }


    #[test]
    fn json_to_hex() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
        let json = r#"
        {
            "from":"alice",
            "to":"bob",
            "quantity":"1.0000 EOS",
            "memo":"Hello!"
        }"#;
        let bin = abieos.json_to_hex("eosio.token", "transfer", json.to_string()).unwrap();
        assert_eq!(bin, HEX_ACTION_TRANSFER);
        abieos.destroy();
    }

    #[test]
    fn json_to_bin() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
        let json = r#"
        {
            "from":"alice",
            "to":"bob",
            "quantity":"1.0000 EOS",
            "memo":"Hello!"
        }"#;
        let bin = abieos.json_to_bin("eosio.token", "transfer", json.to_string()).unwrap();
        assert_eq!(bin, BIN_ACTION_TRANSFER);
        abieos.destroy();
    }

    #[test]
    fn hex_to_json() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
        let bin = HEX_ACTION_TRANSFER;
        let json = abieos.hex_to_json("eosio.token", "transfer", bin.to_string()).unwrap();
        let reverse_bin = abieos.json_to_hex("eosio.token", "transfer", json.clone()).unwrap();
        assert_eq!(bin, reverse_bin);
        abieos.destroy();
    }

    #[test]
    fn get_type_for_action() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
        let action = "transfer";
        let type_name = abieos.get_type_for_action("eosio.token", action).unwrap();
        assert_eq!(type_name, "transfer");
        abieos.destroy();
    }

    #[test]
    fn get_type_for_action_invalid() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
        let action = "invalid";
        let type_name = abieos.get_type_for_action("eosio.token", action);
        assert_eq!(type_name.is_err(), true);
        abieos.destroy();
    }

    #[test]
    fn get_type_for_action_invalid_contract() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
        let action = "transfer";
        let type_name = abieos.get_type_for_action("invalid", action);
        assert_eq!(type_name.is_err(), true);
        abieos.destroy();
    }

    #[test]
    fn get_type_for_table() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
        let table = "accounts";
        let type_name = abieos.get_type_for_table("eosio.token", table).unwrap();
        assert_eq!(type_name, "account");
        abieos.destroy();
    }

    #[test]
    fn get_type_for_table_invalid() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
        let table = "invalid";
        let type_name = abieos.get_type_for_table("eosio.token", table);
        assert_eq!(type_name.is_err(), true);
        abieos.destroy();
    }


    #[test]
    fn get_type_for_table_invalid_contract() {
        let abieos: Abieos = Abieos::new();
        abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
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
        let abi = match abieos.abi_json_to_bin(json_data) {
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
        let json_abi = match abieos.abi_bin_to_json(abi_data) {
            Ok(x) => x,
            Err(_) => panic!("abi_bin_to_json test failed"),
        };

        // use serde to parse json and check if it is valid
        let abi: serde_json::Value = serde_json::from_str(json_abi.as_str()).unwrap();
        assert_eq!(abi["version"], "eosio::abi/1.2");

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