# Rust Abieos

`rs_abieos` is a Rust library that provides a wrapper for the `abieos` C++ library. It allows you to handle data from
Antelope blockchains by providing functionalities such as converting between binary and JSON formats for ABI files,
converting between native and string names, and more.

This wrapper is currently based on the vanilla version of the [AntelopeIO/abieos](https://github.com/AntelopeIO/abieos)

Test cases are being completely rewritten in Rust. They can be found in the `tests` directory.

`bin-src/main.rs` is a simple executable example that demonstrates how to use the library.

## Pre requisites

- Linux
- C++ toolchain. You can use alternative compilers to build the library.
  We recommend using Clang 18 to build the `abieos` C++ library.

Make sure you have Clang 18 installed on your system:

```bash
wget https://apt.llvm.org/llvm.sh
chmod +x llvm.sh
sudo ./llvm.sh 18
```

## Setup Instructions

To use `rs_abieos` in your Rust project, you need to add it as a dependency in your `Cargo.toml` file:

```bash
cargo add rs_abieos
```

Then, run the following command to download and compile the `rs_abieos` library:

```bash
cargo build
# or if you have another default compiler, use clang-18 to build the library
CXX=clang++-18 CC=clang-18 cargo build
```

### Testing

To run the test cases, use the following command:

```bash
cargo test
# or
CXX=clang++-18 CC=clang-18 cargo test
```

## Quick Usage Example

Short example of how to use the library on new Rust binary project:

```rust
// Step 1 - Bring rs_abieos into scope
use rs_abieos::Abieos;

fn main() {
  // Step 2 - Create an instance of Abieos
  let abieos = Abieos::new();

  // Let's try to convert a string to u64 name
  let name = "alice";
  let name_u64 = abieos.string_to_name(name).unwrap();
  println!("Name: {}, Name_u64: {}", name, name_u64);
}
```

## Detailed Example

For this example, download the [eosio.system abi file](https://raw.githubusercontent.com/eosrio/rs-abieos/master/abis/eosio.abi) and copy to your project root as eosio.abi.json

```rust
use rs_abieos::{AbiLike, Abieos, NameLike};

fn main() {
  let abieos = Abieos::new();
  let path = "eosio.abi.json";

  // Read the ABI file
  let abi_content = match std::fs::read_to_string(path) {
    Ok(content) => content,
    Err(e) => {
      eprintln!("Failed to read ABI file: {}", e);
      return;
    }
  };

  // define the NameLike enum to hold the account name using either a string (String) or a reference (StringRef) or an u64 (U64)
  let account_name = "eosio";
  let eosio = NameLike::StringRef(&account_name);

  // create a eosio contract instance using the NameLike enum
  let mut eosio_contract = abieos.contract(eosio);

  // load the abi using the contract instance
  // load_abi method takes an AbiLike enum which can be either a Json (String), Hex (String) or Bin (Vec<u8>)
  let load_status = eosio_contract.load_abi(AbiLike::Json(abi_content));

  // check if the abi was loaded successfully
  match load_status {
    Ok(_) => println!("ABI loaded successfully"),
    Err(e) => eprintln!("Failed to load ABI: {}", e),
  }

  // Let's serialize an action using the eosio contract instance

  // define the action data (example eosio::buyram)
  let action_data = r#"{
            "payer":"alice",
            "receiver":"bob",
            "quant":"100.0000 SYS"
    }"#;

  // retrieve the datatype for the action
  let datatype = match eosio_contract.get_type_for_action("buyram") {
    Ok(datatype) => datatype,
    Err(e) => {
      eprintln!("Failed to get datatype for action: {}", e);
      return;
    }
  };

  // serialize the action data
  let serialized_action = eosio_contract.json_to_hex(datatype.as_str(), action_data.to_string());

  let hex_action = match serialized_action {
    Ok(hex_data) => {
      println!("Serialized action data: {}", hex_data);
      hex_data
    }
    Err(e) => {
      eprintln!("Failed to serialize action data: {}", e);
      return;
    },
  };

  // Let's deserialize the serialized action data
  let json_action = eosio_contract.hex_to_json(datatype.as_str(), hex_action);

  match json_action {
    Ok(json_data) => println!("Deserialized action data: {}", json_data),
    Err(e) => eprintln!("Failed to deserialize action data: {}", e),
  }

}
```

## API Documentation

Please refer to the library's API documentation for more detailed information on each function.