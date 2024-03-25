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
sudo apt install clang-18
```

## Setup Instructions

To use `rs_abieos` in your Rust project, you need to add it as a dependency in your `Cargo.toml` file:

```bash
cargo add rs_abieos
```

Then, run the following command to download and compile the `rs_abieos` library:

```bash
cargo build
```

### Building with Clang 18 (recommended)

Then, build the library using the following command:

```bash
CXX=clang++-18 CC=clang-18 cargo build
```

### Testing

To run the test cases, use the following command:

```bash
CXX=clang++-18 CC=clang-18 cargo test
# or
cargo test
```

## General Usage Guidelines

Step 1 - Create a new instance of `Abieos`:

```
use rs_abieos::Abieos;
let abieos = Abieos::new();
```

Step 2 - Load the ABI from a file into a contract name:

This will store the ABI in the `Abieos` instance and allow you to use the ABI-related functions.
You only need to do this once for each ABI file.

```
let abi_content = std::fs::read_to_string("path/to/your/abi/file").expect("Failed to read ABI file");
let loading_status = abieos.set_abi("CONTRACT_NAME", abi_content.as_str()).unwrap();
println!("ABI loaded: {}", loading_status);
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