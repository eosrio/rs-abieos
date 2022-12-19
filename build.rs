use std::path::PathBuf;

fn main() {
    // This is the directory where the `c` library is located.
    let libdir_path = PathBuf::from("abieos").canonicalize().expect("cannot canonicalize path");

    // Tell cargo to look for shared libraries in the specified directory
    println!("cargo:rustc-link-search={}", libdir_path.to_str().unwrap());

    // Tell cargo to tell rustc to link our `abieos` library. Cargo will
    // automatically know it must look for a `libabieos.a` file.
    println!("cargo:rustc-link-lib=rustabieos");
}
