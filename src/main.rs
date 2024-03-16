use std::fs::read_to_string;
use rs_abieos::Abieos;

pub const EOSIO_TOKEN_HEX_ABI: &str = "0e656f73696f3a3a6162692f312e30010c6163636f756e745f6e616d65046e616d6505087472616e7366657200040466726f6d0c6163636f756e745f6e616d6502746f0c6163636f756e745f6e616d65087175616e74697479056173736574046d656d6f06737472696e67066372656174650002066973737565720c6163636f756e745f6e616d650e6d6178696d756d5f737570706c79056173736574056973737565000302746f0c6163636f756e745f6e616d65087175616e74697479056173736574046d656d6f06737472696e67076163636f756e7400010762616c616e63650561737365740e63757272656e63795f7374617473000306737570706c790561737365740a6d61785f737570706c79056173736574066973737565720c6163636f756e745f6e616d6503000000572d3ccdcd087472616e73666572000000000000a531760569737375650000000000a86cd445066372656174650002000000384f4d113203693634010863757272656e6379010675696e743634076163636f756e740000000000904dc603693634010863757272656e6379010675696e7436340e63757272656e63795f7374617473000000";

fn main() {

    // create a new instance of abieos
    let abieos: Abieos = Abieos::new();

    // loading an abi from a file
    let abi_content = read_to_string("abis/eosio.abi").expect("Failed to read ABI file");

    // converting an abi from json to binary
    println!("\n⚡ Testing conversion from abi json to binary...");
    let abi_bin: Vec<u8> = abieos.abi_json_to_bin(abi_content.clone()).unwrap();
    if abi_bin.len() > 0 {
        println!("☑️ ABI_BIN: {:?}", abi_bin[0..10].to_vec());
        println!("☑️ ABI SIZE: {}", abi_bin.len());
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
        // check if the json is valid json using serde_json
        match serde_json::from_str::<serde_json::Value>(abi_json.as_str()) {
            Ok(value) => {
                println!("☑️ ABI_JSON is valid JSON: {}", value["version"]);
            }
            Err(e) => {
                println!("❌ ABI_JSON is not valid JSON: {}", e);
            }
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
    let original_name = "eosio";
    let native_name = abieos.string_to_name(original_name).unwrap();
    let name_as_string = abieos.name_to_string(native_name).unwrap();
    if name_as_string == original_name {
        println!("☑️ {original_name} => {native_name} => {name_as_string}");
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


    let runs = 2;
    println!("\n⚡ Testing hex to json back and forth conversion {runs} times...");
    let start = std::time::Instant::now();
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

    println!("\n⚡ Testing loading abi as json...");
    match abieos.set_abi("eosio", abi_content.as_str()) {
        Ok(_) => {
            println!("☑️ JSON Abi Loaded successfully")
        }
        Err(e) => {
            println!("❌ Failed to load JSON Abi: {}", e)
        }
    };


    let json_sample = read_to_string("abis/sample.json")
        .expect("Failed to read JSON file");
    let json_sample_unordered = read_to_string("abis/sample_unordered.json")
        .expect("Failed to read JSON file");

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

    assert_eq!(result,result2, "The result should be the same for ordered and unordered jsons");

    // deserialize
    let ds_result = abieos.hex_to_json(
        "eosio",
        "delegatebw",
        result,
    ).unwrap();
    println!("{}", ds_result);

    let parsed_json = serde_json::from_str::<serde_json::Value>(ds_result.as_str()).unwrap();
    println!("From: {} | To: {}", parsed_json["from"], parsed_json["receiver"]);
}
