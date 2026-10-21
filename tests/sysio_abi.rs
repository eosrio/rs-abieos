#![cfg(feature = "rust-backend")]

use rs_abieos::{AbiHandle, AbiLike, Abieos, NameLike};

#[test]
fn sysio_table_names_do_not_alias_eosio_names() {
    let abi_json = r#"{
        "version":"sysio::abi/1.2",
        "structs":[
            {"name":"A","base":"","fields":[{"name":"x","type":"uint8"}]},
            {"name":"B","base":"","fields":[{"name":"x","type":"uint16"}]}
        ],
        "tables":[
            {"name":"foo.bar","type":"A","table_id":1},
            {"name":"foo_bar","type":"B","table_id":2},
            {"name":"LongTableNameWithSharedPrefix-A","type":"A","table_id":3},
            {"name":"LongTableNameWithSharedPrefix-B","type":"B","table_id":4},
            {"name":"foo.bar.","type":"B","table_id":5}
        ]
    }"#;
    let abieos = Abieos::new();
    let binary = abieos.abi_json_to_bin(abi_json).unwrap();
    for mut handle in [
        AbiHandle::from_json(abi_json).unwrap(),
        AbiHandle::from_bin(&binary).unwrap(),
    ] {
        assert_eq!(handle.type_for_table_name("foo.bar"), Some("A"));
        assert_eq!(handle.type_for_table_name("foo_bar"), Some("B"));
        assert_eq!(handle.type_for_table_name("foo.bar."), Some("B"));
        assert!(handle.type_for_table_name("foo-bar").is_none());
        let numeric = abieos.string_to_name("foo.bar").unwrap();
        assert_eq!(handle.type_for_table(numeric), Some("A"));
        assert_eq!(
            handle.decode_table_row(numeric, &[7]).unwrap(),
            r#"{"x":7}"#
        );
        let row_type = handle.type_for_table_name("foo_bar").unwrap().to_owned();
        assert_eq!(
            handle.bin_to_json(&row_type, &[0, 1]).unwrap(),
            r#"{"x":256}"#
        );
    }
    for abi in [AbiLike::Json(abi_json.into()), AbiLike::Bin(binary)] {
        let mut contract = abieos.contract(NameLike::StringRef("test"));
        contract.load_abi(abi).unwrap();
        for (name, expected) in [
            ("foo.bar", "A"),
            ("foo_bar", "B"),
            ("LongTableNameWithSharedPrefix-A", "A"),
            ("LongTableNameWithSharedPrefix-B", "B"),
            ("foo.bar.", "B"),
        ] {
            assert_eq!(abieos.get_type_for_table("test", name).unwrap(), expected);
            assert_eq!(contract.get_type_for_table(name).unwrap(), expected);
        }
        // The legacy numeric lookup must still find the canonical EOSIO name,
        // regardless of a later arbitrary name with the same lossy projection.
        let account = abieos.string_to_name("test").unwrap();
        let table = abieos.string_to_name("foo.bar").unwrap();
        assert_eq!(
            abieos.get_type_for_table_native(account, table).unwrap(),
            "A"
        );
        assert!(abieos.get_type_for_table("test", "foo-bar").is_err());
    }
}

#[test]
fn sysio_enum_numbers_are_not_symbolic_names_or_suffixes() {
    let abi = r#"{
        "version":"sysio::abi/1.2",
        "enums":[
            {"name":"tier","type":"uint8","values":[
                {"name":"TIER_0","value":1}, {"name":"2","value":7}
            ]},
            {"name":"ambiguous","type":"uint8","values":[
                {"name":"TIER_0","value":1}, {"name":"LEVEL_0","value":2}
            ]}
        ]
    }"#;
    let abieos = Abieos::new();
    let binary = abieos.abi_json_to_bin(abi).unwrap();
    for mut handle in [
        AbiHandle::from_json(abi).unwrap(),
        AbiHandle::from_bin(&binary).unwrap(),
    ] {
        assert_eq!(handle.json_to_bin("tier", "0").unwrap(), [0]);
        assert_eq!(handle.json_to_bin("tier", "2").unwrap(), [2]);
        assert_eq!(handle.json_to_bin("tier", r#""0""#).unwrap(), [1]);
        assert_eq!(handle.json_to_bin("tier", r#""2""#).unwrap(), [7]);
        assert_eq!(handle.json_to_bin("ambiguous", "0").unwrap(), [0]);
        assert!(handle.json_to_bin("ambiguous", r#""0""#).is_err());
    }
}

#[test]
fn sysio_protobuf_descriptor_objects_and_strings_preserve_binary_payload() {
    let object_abi = r#"{"version":"sysio::abi/1.3","protobuf_types":{"file":[{"name":"test.proto","syntax":"proto3","messageType":[{"name":"M","field":[{"name":"value","number":1,"type":"TYPE_UINT32"}]}]}]}}"#;
    let string_abi = r#"{"version":"sysio::abi/1.3","protobuf_types":"{\"file\":[{\"name\":\"test.proto\",\"syntax\":\"proto3\",\"messageType\":[{\"name\":\"M\",\"field\":[{\"name\":\"value\",\"number\":1,\"type\":\"TYPE_UINT32\"}]}]}]}"}"#;
    let abieos = Abieos::new();
    let object_bin = abieos.abi_json_to_bin(object_abi).unwrap();
    let string_bin = abieos.abi_json_to_bin(string_abi).unwrap();
    assert_eq!(object_bin, string_bin);
    let json = abieos.abi_bin_to_json(&object_bin).unwrap();
    assert!(
        json.contains(r#""protobuf_types":{"file":[{"name":"test.proto""#),
        "{json}"
    );
    assert!(json.contains(r#""number":1"#), "{json}");
    assert_eq!(abieos.abi_json_to_bin(&json).unwrap(), object_bin);

    // Object whitespace must survive a binary -> JSON -> binary round trip.
    let spaced = r#"{"version":"sysio::abi/1.3","protobuf_types": { "file": [] }}"#;
    let bin = abieos.abi_json_to_bin(spaced).unwrap();
    let json = abieos.abi_bin_to_json(&bin).unwrap();
    assert!(
        json.contains(r#""protobuf_types":{ "file": [] }"#),
        "{json}"
    );
    assert_eq!(abieos.abi_json_to_bin(&json).unwrap(), bin);

    // A string can also contain outer whitespace, which JSON object syntax
    // cannot distinguish from whitespace around the ABI field itself.
    let padded = r#"{"version":"sysio::abi/1.3","protobuf_types":" \n{\"file\":[]} \n"}"#;
    let bin = abieos.abi_json_to_bin(padded).unwrap();
    let json = abieos.abi_bin_to_json(&bin).unwrap();
    assert_eq!(abieos.abi_json_to_bin(&json).unwrap(), bin);
}

#[test]
fn sysio_primary_and_secondary_table_ids_require_u16_values() {
    let abieos = Abieos::new();
    for invalid in [
        "65536",
        "-1",
        "1.5",
        "1e2",
        r#""invalid""#,
        "null",
        "true",
        "[]",
        "{}",
    ] {
        for table in [
            format!(r#"{{"name":"rows","type":"uint8","table_id":{invalid}}}"#),
            format!(
                r#"{{"name":"rows","type":"uint8","secondary_indexes":[{{"name":"byfoo","key_type":"uint64","table_id":{invalid}}}]}}"#
            ),
        ] {
            let abi = format!(r#"{{"version":"sysio::abi/1.2","tables":[{table}]}}"#);
            assert!(abieos.abi_json_to_bin(&abi).is_err(), "accepted {abi}");
            assert!(abieos.set_abi_json("test", &abi).is_err(), "accepted {abi}");
            assert!(AbiHandle::from_json(&abi).is_err(), "accepted {abi}");
        }
    }
    for (valid, expected) in [
        ("0", 0),
        ("65535", 65535),
        (r#""0""#, 0),
        (r#""65535""#, 65535),
    ] {
        let abi = format!(
            r#"{{"version":"sysio::abi/1.2","tables":[{{"name":"rows","type":"uint8","table_id":{valid},"secondary_indexes":[{{"name":"byfoo","key_type":"uint64","table_id":{valid}}}]}}]}}"#
        );
        let bin = abieos.abi_json_to_bin(&abi).unwrap();
        let json = abieos.abi_bin_to_json(&bin).unwrap();
        assert_eq!(
            json.matches(&format!(r#""table_id":{expected}"#)).count(),
            2,
            "{json}"
        );
        assert_eq!(abieos.abi_json_to_bin(&json).unwrap(), bin);
    }
}

/// Real on-chain `sysio.token` raw binary ABI from Wire Testnet (`sysio::abi/1.2`).
const SYSIO_TOKEN_RAW_ABI_HEX: &str = concat!(
    "0E737973696F3A3A6162692F312E32000A076163636F756E7400010762616C616E636505617373657408",
    "616363745F6B657900010873796D5F636F64650675696E74363405636C6F73650002056F776E6572046E",
    "616D650673796D626F6C0673796D626F6C06637265617465000206697373756572046E616D650E6D6178",
    "696D756D5F737570706C790561737365740E63757272656E63795F7374617473000306737570706C7905",
    "61737365740A6D61785F737570706C7905617373657406697373756572046E616D650569737375650003",
    "02746F046E616D65087175616E74697479056173736574046D656D6F06737472696E67046F70656E0003",
    "056F776E6572046E616D650673796D626F6C0673796D626F6C0972616D5F7061796572046E616D650672",
    "65746972650002087175616E74697479056173736574046D656D6F06737472696E6708737461745F6B65",
    "7900010873796D5F636F64650675696E743634087472616E7366657200040466726F6D046E616D650274",
    "6F046E616D65087175616E74697479056173736574046D656D6F06737472696E67060000000000856944",
    "05636C6F73650000000000A86CD44506637265617465000000000000A531760569737375650000000000",
    "003055A5046F70656E0000000000A8EBB2BA0672657469726500000000572D3CCDCD087472616E736665",
    "720002086163636F756E747303693634020573636F70650873796D5F636F646502046E616D650675696E",
    "743634076163636F756E743C6400047374617403693634020573636F70650873796D5F636F646502046E",
    "616D650675696E7436340E63757272656E63795F7374617473A8100000000000000000"
);

#[test]
fn sysio_token_raw_abi_roundtrips_byte_for_byte_and_encodes_transfer() {
    let abieos = Abieos::new();
    abieos
        .set_abi_hex("sysio.token", SYSIO_TOKEN_RAW_ABI_HEX)
        .expect("set_abi_hex should accept sysio::abi/1.2 binary");

    assert_eq!(
        abieos
            .get_type_for_action("sysio.token", "transfer")
            .unwrap(),
        "transfer"
    );
    assert_eq!(
        abieos
            .get_type_for_table("sysio.token", "accounts")
            .unwrap(),
        "account"
    );
    assert_eq!(
        abieos.get_type_for_table("sysio.token", "stat").unwrap(),
        "currency_stats"
    );

    // Encode and decode a 9-decimal WIRE transfer
    let transfer_json =
        r#"{"from":"alice","to":"bob","quantity":"12.345678900 WIRE","memo":"wire testnet"}"#;
    let hex = abieos
        .json_to_hex("sysio.token", "transfer", transfer_json)
        .expect("json_to_hex transfer");
    let roundtrip = abieos
        .hex_to_json("sysio.token", "transfer", &hex)
        .expect("hex_to_json transfer");
    assert_eq!(roundtrip, transfer_json);

    // Verify binary -> JSON -> binary ABI roundtrip matches on-chain raw ABI bytes exactly
    let raw_bytes: Vec<u8> = (0..SYSIO_TOKEN_RAW_ABI_HEX.len())
        .step_by(2)
        .map(|i| u8::from_str_radix(&SYSIO_TOKEN_RAW_ABI_HEX[i..i + 2], 16).unwrap())
        .collect();
    let abi_json = abieos
        .abi_bin_to_json(&raw_bytes)
        .expect("abi_bin_to_json sysio.token");
    assert!(abi_json.contains(r#""version":"sysio::abi/1.2""#));
    assert!(abi_json.contains(r#""table_id":25660"#)); // 0x643C
    assert!(abi_json.contains(r#""table_id":4264"#)); // 0x10A8
    let reencoded = abieos
        .abi_json_to_bin(&abi_json)
        .expect("abi_json_to_bin sysio.token");
    assert_eq!(reencoded, raw_bytes);
}

#[test]
fn sysio_abi_enums_and_secondary_indexes_roundtrip() {
    let abi_json = r#"{
        "version": "sysio::abi/1.2",
        "types": [],
        "structs": [
            {
                "name": "addtrxp",
                "base": "",
                "fields": [
                    {"name": "match_type", "type": "trx_match_type"},
                    {"name": "status", "type": "bond_status"},
                    {"name": "opt_mode", "type": "trx_match_type?"},
                    {"name": "modes", "type": "trx_match_type[]"}
                ]
            },
            {
                "name": "producer_info",
                "base": "",
                "fields": [
                    {"name": "owner", "type": "name"},
                    {"name": "total_votes", "type": "float64"}
                ]
            }
        ],
        "actions": [
            {"name": "addtrxp", "type": "addtrxp", "ricardian_contract": ""}
        ],
        "tables": [
            {
                "name": "producers",
                "index_type": "i64",
                "key_names": ["owner"],
                "key_types": ["name"],
                "type": "producer_info",
                "table_id": 12,
                "secondary_indexes": [
                    {"name": "prototalvote", "key_type": "float64", "table_id": 13}
                ]
            }
        ],
        "ricardian_clauses": [],
        "error_messages": [],
        "variants": [],
        "action_results": [],
        "enums": [
            {
                "name": "trx_match_type",
                "type": "uint8",
                "values": [
                    {"name": "only", "value": 0},
                    {"name": "first", "value": 1},
                    {"name": "any", "value": 2}
                ]
            },
            {
                "name": "bond_status",
                "type": "int64",
                "values": [
                    {"name": "status_active", "value": 10},
                    {"name": "status_released", "value": 20},
                    {"name": "prefix_a_dup", "value": 30},
                    {"name": "prefix_b_dup", "value": 40}
                ]
            }
        ]
    }"#;

    let mut handle = AbiHandle::from_json(abi_json).expect("AbiHandle::from_json sysio::abi/1.2");

    // 1. Exact member names + unique '_'-suffix match ("released" matches "status_released")
    let input_json =
        r#"{"match_type":"first","status":"released","opt_mode":"any","modes":["only","first"]}"#;
    let bin = handle
        .json_to_bin("addtrxp", input_json)
        .expect("json_to_bin with enum names and suffix");
    let decoded = handle.bin_to_json("addtrxp", &bin).expect("bin_to_json");
    assert_eq!(
        decoded,
        r#"{"match_type":"first","status":"status_released","opt_mode":"any","modes":["only","first"]}"#
    );

    // 2. Integer values in JSON pack to the same binary and decode to enum member names
    let numeric_json = r#"{"match_type":1,"status":"20","opt_mode":null,"modes":[0,2]}"#;
    let bin_num = handle
        .json_to_bin("addtrxp", numeric_json)
        .expect("json_to_bin with numeric enum values");
    let decoded_num = handle
        .bin_to_json("addtrxp", &bin_num)
        .expect("bin_to_json");
    assert_eq!(
        decoded_num,
        r#"{"match_type":"first","status":"status_released","opt_mode":null,"modes":["only","any"]}"#
    );

    // 3. Unknown integer value in binary falls back to integer formatting
    let unknown_num_json = r#"{"match_type":99,"status":"999","opt_mode":null,"modes":[]}"#;
    let bin_unknown = handle
        .json_to_bin("addtrxp", unknown_num_json)
        .expect("json_to_bin unknown integer");
    let decoded_unknown = handle
        .bin_to_json("addtrxp", &bin_unknown)
        .expect("bin_to_json unknown integer");
    assert_eq!(decoded_unknown, unknown_num_json);

    // 4. Ambiguous '_'-suffix ("dup" matches both "prefix_a_dup" and "prefix_b_dup") errors
    let ambiguous_json = r#"{"match_type":"only","status":"dup","opt_mode":null,"modes":[]}"#;
    let err = handle.json_to_bin("addtrxp", ambiguous_json).unwrap_err();
    assert!(
        err.to_string().contains("Unknown or ambiguous enum value"),
        "unexpected error: {err}"
    );

    // 5. Unknown enum string name errors
    let bad_name_json = r#"{"match_type":"nonexistent","status":10,"opt_mode":null,"modes":[]}"#;
    let err = handle.json_to_bin("addtrxp", bad_name_json).unwrap_err();
    assert!(
        err.to_string().contains("Unknown or ambiguous enum value"),
        "unexpected error: {err}"
    );

    // 6. ABI JSON -> bin -> JSON roundtrip preserves enums, table_id, and secondary_indexes
    let abieos = Abieos::new();
    let abi_bin = abieos.abi_json_to_bin(abi_json).expect("abi_json_to_bin");
    let abi_json_out = abieos.abi_bin_to_json(&abi_bin).expect("abi_bin_to_json");
    assert!(abi_json_out.contains(r#""prototalvote""#));
    assert!(abi_json_out.contains(r#""trx_match_type""#));
    let abi_bin_2 = abieos
        .abi_json_to_bin(&abi_json_out)
        .expect("abi_json_to_bin second pass");
    assert_eq!(abi_bin, abi_bin_2);
}
