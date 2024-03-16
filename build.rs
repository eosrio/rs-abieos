use std::process::Command;
use bindgen;
use cc;

fn main() {

    // call "git submodule update --init --recursive"
    let git_update_result = Command::new("git")
        .args(&["submodule", "update", "--init", "--recursive", "-f"])
        .output()
        .expect("Failed to execute git");

    println!("git submodule update --init --recursive: {}", String::from_utf8_lossy(&git_update_result.stderr));


    // // Build native bindings :: "bindgen lib/abieos/src/abieos.h -o src/bindings.rs"
    bindgen::builder()
        .header("lib/abieos/src/abieos.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");

    // Build the native library with rust
    cc::Build::new()
        .cpp(true)
        .includes(&[
            "lib/abieos/external/rapidjson/include",
            "lib/abieos/include"
        ])
        .files(&[
            "lib/abieos/src/abieos.cpp",
            "lib/abieos/src/abi.cpp",
            "lib/abieos/src/crypto.cpp",
            "lib/abieos/include/eosio/fpconv.c",
        ])
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wno-unused-parameter")
        .flag("-std=c++17")
        .static_flag(true)
        .cpp_link_stdlib("stdc++")
        .out_dir("target/lib/abieos")
        .compile("abieos");

    // Add search path for the shared library
    println!("cargo:rustc-link-search={}/build", "target/lib");

    // Link the shared abieos library
    println!("cargo:rustc-link-lib=static=abieos");

    // Link the system C++ shared library.
    if sys_info::os_type().unwrap() == "Linux" {
        println!("cargo:rustc-link-lib=stdc++");
    } else {
        println!("cargo:rustc-link-lib=c++");
    }
}