use std::ffi::{CStr, CString};
use std::fs::read_to_string;
use std::time::Instant;
use rs_abieos::{Abieos, NameLike};

pub const EOSIO_TOKEN_HEX_ABI: &str = "0e656f73696f3a3a6162692f312e30010c6163636f756e745f6e616d65046e616d6505087472616e7366657200040466726f6d0c6163636f756e745f6e616d6502746f0c6163636f756e745f6e616d65087175616e74697479056173736574046d656d6f06737472696e67066372656174650002066973737565720c6163636f756e745f6e616d650e6d6178696d756d5f737570706c79056173736574056973737565000302746f0c6163636f756e745f6e616d65087175616e74697479056173736574046d656d6f06737472696e67076163636f756e7400010762616c616e63650561737365740e63757272656e63795f7374617473000306737570706c790561737365740a6d61785f737570706c79056173736574066973737565720c6163636f756e745f6e616d6503000000572d3ccdcd087472616e73666572000000000000a531760569737375650000000000a86cd445066372656174650002000000384f4d113203693634010863757272656e6379010675696e743634076163636f756e740000000000904dc603693634010863757272656e6379010675696e7436340e63757272656e63795f7374617473000000";

fn measure_call(f: &mut dyn FnMut(), name: &str) {
    let start = Instant::now();
    f();
    let duration = start.elapsed();
    println!("⏱️ {name} took: {:?}", duration);
}

fn main() {

    // create a new instance of abieos
    let abieos: Abieos = Abieos::new();

    // loading an abi from a file
    let abi_content = read_to_string("abis/eosio.abi").expect("Failed to read ABI file");

    // converting an abi from json to binary
    println!("\n⚡ Testing conversion from abi json to binary...");
    let abi_bin: Vec<u8> = abieos.abi_json_to_bin(abi_content.clone()).unwrap();
    if !abi_bin.is_empty() {
        println!("☑️ ABI Converted: (size: {} bytes)", abi_bin.len());

        // save the binary to a file
        std::fs::write("abis/eosio.abi.bin", abi_bin.clone()).expect("Failed to write ABI binary to file");
    } else {
        println!("❌ Failed to convert ABI to binary");
    }

    // loading an abi as binary
    println!("\n⚡ Testing loading abi as binary...");
    let loading_status = abieos.set_abi_bin("eosio", abi_bin.clone()).unwrap();
    println!("☑️ Binary Load: {}", loading_status);

    // converting an abi from binary to json
    println!("\n⚡ Testing conversion from abi binary to json...");
    let abi_json = abieos.abi_bin_to_json(abi_bin).unwrap();
    if !abi_json.is_empty() {
        // check if the json is valid json without any external libraries
        let is_valid_json = abi_json.starts_with('{') && abi_json.ends_with('}');
        if is_valid_json {
            println!("☑️ ABI_JSON is valid JSON!");
        } else {
            println!("❌ ABI_JSON is not valid JSON!");
        }
    }

    // loading an abi hex
    println!("\n⚡ Testing loading abi as hex...");
    let loading_status = abieos.set_abi_hex("eosio.token", EOSIO_TOKEN_HEX_ABI.to_string()).unwrap();
    if loading_status {
        println!("☑️ HEX Abi Loaded successfully");
    } else {
        println!("❌ Failed to load HEX Abi");
    }

    // name conversion test
    println!("\n⚡ Testing name conversion...");
    let original_name = "alice";
    let mut native_name: u64 = 0;
    let mut name_as_string: &str = "";

    // measure the time taken to convert the name
    measure_call(&mut || {
        native_name = abieos.string_to_name(original_name).unwrap();
        name_as_string = abieos.name_to_string(native_name).unwrap();
    }, "name conversion (rust string)");
    if name_as_string == original_name {
        println!("☑️ {original_name} => {native_name} => {name_as_string}");
    } else {
        println!("❌ Name conversion failed");
    }

    // name conversion test
    println!("\n⚡ Testing name conversion (C-String)...");
    let original_name = c"eosio";
    let mut native_name: u64 = 0;
    let mut name_as_string: &CStr = c"";

    // measure the time taken to convert the name
    measure_call(&mut || {
        native_name = abieos.c_string_to_name(original_name);
        name_as_string = abieos.name_to_cstr(native_name);
    }, "name conversion (C-string)");

    if name_as_string.eq(original_name) {
        println!("☑️ {:?} => {native_name} => {:?}", original_name, name_as_string);
    } else {
        println!("❌ Name conversion failed");
    }


    println!("\n⚡ Testing action type conversion...");
    match abieos.get_type_for_action("eosio.token", "transfer") {
        Ok(x) => println!("transfer_action_datatype: {x}"),
        Err(_) => {
            println!("❌ Failed to get transfer action datatype");
        }
    };

    println!("\n⚡ Testing table type conversion...");
    match abieos.get_type_for_table("eosio.token", "accounts") {
        Ok(x) => println!("account_table_datatype: {x}"),
        Err(_) => {
            println!("❌ Failed to get account table datatype");
        }
    };

    println!("\n⚡ Testing json to hex conversion...");
    let json = r#"
        {
            "from":"alice",
            "to":"bob",
            "quantity":"1.0000 EOS",
            "memo":"Hello!"
        }"#;
    let bin = match abieos.json_to_hex("eosio.token", "transfer", json.to_string()) {
        Ok(x) => {
            println!("json_to_hex: {}", x.clone());
            x
        }
        Err(_) => {
            println!("❌ Failed to convert json to hex");
            String::new()
        }
    };


    {
        let runs = 1000;
        println!("\n⚡ Testing hex to json back and forth conversion {runs} times...");
        let start = Instant::now();
        let account = "eosio.token";
        let action = "transfer";
        let mut last_json = String::new();
        for _ in 0..runs {
            let json_out = abieos.hex_to_json(account, action, bin.clone()).unwrap();
            let bin_out = abieos.json_to_hex(account, action, json_out.to_string()).unwrap();
            if !last_json.is_empty() {
                assert_eq!(json_out, last_json);
                assert_eq!(bin.clone(), bin_out.to_string());
            }
            last_json = json_out.to_string();
        }
        let duration = start.elapsed();
        // Average time
        println!("Average time elapsed in hex_to_json() is: {:?}", duration / runs);
        println!("hex_to_json: {}", last_json);
    }

    {
        let runs = 1000;
        println!("\n⚡ Testing hex to json using C-String back and forth conversion {runs} times...");
        let start = Instant::now();
        let account = c"eosio.token";
        let action = c"transfer";
        let mut last_json = c"";
        let bin_c = CString::new(bin).unwrap();
        let mut bin: &CStr = bin_c.as_c_str();
        for _ in 0..runs {
            let json_out = abieos.hex_to_json_c(account, action, bin);
            let bin_out = abieos.json_to_hex_c(account, action, json_out);
            if !last_json.is_empty() {
                assert_eq!(json_out, last_json);
                assert_eq!(bin, bin_out);
            }
            last_json = json_out;
            bin = bin_out;
        }
        let duration = start.elapsed();
        // Average time
        println!("Average time elapsed in hex_to_json_c() + json_to_hex_c() is: {:?}", duration / runs);
    }

    {
        println!("\n⚡ Testing loading abi as json...");
        let abi_content_c = CString::new(abi_content).unwrap();
        match abieos.set_abi_json_c("eosio", abi_content_c.as_ref()) {
            Ok(_) => {
                println!("☑️ JSON Abi Loaded successfully")
            }
            Err(e) => {
                println!("❌ Failed to load JSON Abi: {}", e)
            }
        };
    }


    println!("\n⚡ Testing json to hex conversion with unordered json...");
    let json_sample = read_to_string("abis/sample.json")
        .expect("Failed to read JSON file");
    println!("J1: {}", json_sample);
    let json_sample_unordered = read_to_string("abis/sample_unordered.json")
        .expect("Failed to read JSON file");
    println!("J2: {}", json_sample_unordered);

    let result = abieos.json_to_hex(
        "eosio",
        "delegatebw",
        json_sample,
    ).unwrap_or_else(|e| {
        println!("❌ Failed to convert json to hex: {}", e);
        String::new()
    });
    println!("{}", result);

    let result2 = abieos.json_to_hex(
        "eosio",
        "delegatebw",
        json_sample_unordered,
    ).unwrap_or_else(|e| {
        println!("❌ Failed to convert json to hex: {}", e);
        String::new()
    });
    println!("{}", result2);

    if result == result2 {
        println!("☑️ The result is the same for ordered and unordered JSONs");
    } else {
        println!("❌ The result is different for ordered and unordered JSONs");
    }

    assert_eq!(result, result2, "The result should be the same for ordered and unordered jsons");

    // deserialize
    println!("\n⚡ Deserializing...");
    let ds_result = abieos.hex_to_json(
        "eosio",
        "delegatebw",
        result,
    ).unwrap();
    println!("{}", ds_result);

    // transaction example
    println!("\n⚡ Serializing Transaction... (oneshot)");

    let time_start = std::time::Instant::now();
    let trx_json = r#"{
            "expiration": "2018-06-27T20:33:54.000",
            "ref_block_num": 45323,
            "ref_block_prefix": 2628749070,
            "max_net_usage_words": 0,
            "max_cpu_usage_ms": 0,
            "delay_sec": 0,
            "context_free_actions": [],
            "actions": [{
                "account": "eosio.token",
                "name": "transfer",
                "authorization":[{
                    "actor":"useraaaaaaaa",
                    "permission":"active"
                }],
                "data":"608C31C6187315D6708C31C6187315D60100000000000000045359530000000000"
            }],
            "transaction_extensions":[]
        }"#;

    match Abieos::new()
        .contract(NameLike::StringRef("eosio"))
        .load_json_file("abis/transaction.abi.json")
        .unwrap()
        .json_to_hex(
            "transaction",
            trx_json.to_string(),
        ) {
        Ok(x) => {
            println!("json_to_hex: {}", x.clone());
            let duration = time_start.elapsed();
            println!("Time elapsed (instancing, loading, serialization) was: {:?}", duration);
        }
        Err(_) => println!("❌ Failed to convert json to hex"),
    };

    measure_call(&mut || {
        abieos.contract(NameLike::StringRef("eosio")).load_json_file("abis/eosio.abi").unwrap();
    }, "loading eosio abi from file (oneshot)");

    measure_call(&mut || {
        let abi_content = read_to_string("abis/eosio.abi").unwrap();
        abieos.set_abi_json("eosio", abi_content).unwrap();
    }, "loading eosio abi from file (procedural)");

    measure_call(&mut || {
        abieos.contract(NameLike::StringRef("eosio")).load_json_file("abis/transaction.abi.json").unwrap();
    }, "loading transaction abi from file");

    let json_data = read_to_string("abis/sample.json").unwrap();

    measure_call(&mut || {
        abieos.contract(NameLike::StringRef("eosio")).json_to_hex("delegatebw", json_data.clone()).unwrap();
    }, "serializing sample action");
}
