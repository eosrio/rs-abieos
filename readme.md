# Rust Abieos

`rs_abieos` is a Rust library that provides a wrapper for the `abieos` C library. It allows you to handle data from Antelope blockchains by providing functionalities such as converting between binary and JSON formats for ABI files, converting between native and string names, and more.

## Setup Instructions

To use `rs_abieos` in your Rust project, you need to add it as a dependency in your `Cargo.toml` file:

```toml
[dependencies]
rs_abieos = "0.1.1"
```

Then, run the following command to download and compile the `rs_abieos` library:

```bash
cargo build
```

## Usage Examples

Here are some examples of how to use the `rs_abieos` library:

### Creating a new instance of Abieos

```
use rs_abieos::Abieos;
let abieos = Abieos::new();
```

### Converting a string name to a native name (u64)

```
let native_name = abieos.string_to_name("eosio.token").unwrap();
println!("Native name: {}", native_name);
```

### Converting a native name (u64) to a string name

```
let string_name = abieos.name_to_string(native_name).unwrap();
println!("String name: {}", string_name);
```

### Loading an ABI from a file (require before any other ABI-related operations)

```
let abi_content = std::fs::read_to_string("path/to/your/abi/file").expect("Failed to read ABI file");
let loading_status = abieos.set_abi("CONTRACT_NAME", abi_content.as_str()).unwrap();
println!("ABI loaded: {}", loading_status);
```

### Converting JSON to HEX

```
let json = r#"
{
    "from":"alice",
    "to":"bob",
    "quantity":"1.0000 EOS",
    "memo":"Hello!"
}"#;

let hex = abieos.json_to_hex("eosio.token", "transfer", json.to_string()).unwrap();
println!("HEX: {}", hex);
```

### Converting HEX to JSON

```
let hex: &str = "0000000000855C340000000000000E3D102700000000000004454F53000000000648656C6C6F21";
let json = abieos.hex_to_json("eosio.token", "transfer", hex).unwrap();
println!("JSON: {}", json);
```

Please refer to the library's API documentation for more detailed information on each function.