use std::fs::read_to_string;
use rs_abieos::Abieos;

fn main() {
    let abieos: Abieos = Abieos::new();
    println!("{:?}", abieos.context);
    let original_name = String::from("tibs");
    let native_name = abieos.string_to_name(original_name.clone());
    let name_as_string = abieos.name_to_string(native_name);
    println!("{original_name} => {native_name}");
    if name_as_string == original_name {
        println!("Names matched!");
    }

    // loading an abi
    let abi_content = read_to_string("abis/eosio.abi").expect("Failed to read ABI file");
    let loading_status = abieos.load_abi(String::from("eosio"), abi_content);
    println!("{}", loading_status);

    let json_sample = read_to_string("abis/sample.json").expect("Failed to read JSON file");
    let result = abieos.json_to_hex(
        String::from("eosio"),
        String::from("buyrex"),
        json_sample,
    );
    println!("{}", result);

    // deserialize
    let ds_result = abieos.hex_to_json(
        String::from("eosio"),
        String::from("buyrex"),
        result,
    );
    println!("{}", ds_result);
}
