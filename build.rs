use std::process::Command;

fn update_submodules() {
    // call "git submodule update --init --recursive"
    let git_update_result = Command::new("git")
        .args(["submodule", "update", "--init", "--recursive", "-f"])
        .output()
        .expect("Failed to execute git");

    println!("git submodule update --init --recursive: {}", String::from_utf8_lossy(&git_update_result.stderr));
}

fn build_linux() {

    update_submodules();

    // Build native bindings :: "bindgen lib/abieos/src/abieos.h -o src/bindings.rs"
    bindgen::builder()
        .header("lib/abieos/src/abieos.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file("src/bindings.rs")
        .expect("Couldn't write bindings!");

    // Build fpconv library
    cc::Build::new()
        .includes(&[
            "lib/abieos/include",
        ])
        .files(&[
            "lib/abieos/include/eosio/fpconv.c",
        ])
        .static_flag(true)
        .out_dir("target/lib/fpconv")
        .compile("fpconv");

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
            "lib/abieos/src/crypto.cpp"
        ])
        .flag("-Wall")
        .flag("-Wextra")
        .flag("-Wno-unused-parameter")
        .flag("-std=gnu++17")
        .static_flag(true)
        .cpp_link_stdlib("stdc++")
        .out_dir("target/lib/abieos")
        .compile("abieos");

    println!("cargo:rustc-link-search=target/lib/build");
    println!("cargo:rustc-link-lib=static=abieos");
    println!("cargo:rustc-link-lib=stdc++");
}

fn main() {

    if std::env::var("DOCS_RS").is_ok() {
        // Skip building on docs.rs
        return;
    }

    match sys_info::os_type() {
        Ok(os_type) => {
            match os_type.as_str() {
                "Linux" => {
                    println!("OS Type: Linux");
                    build_linux();
                }
                _ => {
                    println!("OS Type: {}", os_type);
                    panic!("Unsupported OS");
                }
            }
        }
        Err(e) => {
            println!("Error: {}", e);
        }
    }
}