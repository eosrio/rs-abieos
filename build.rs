#[cfg(feature = "cpp-backend")]
use std::env;
#[cfg(feature = "cpp-backend")]
use std::path::Path;
#[cfg(feature = "cpp-backend")]
use std::process::Command;

/// Ensure the vendored `abieos` sources are present.
///
/// If the submodule is already populated (e.g. the crate was packaged or the
/// repo was cloned with `--recurse-submodules`) we do nothing and never touch
/// `git` — this keeps the build working in environments where `git` is not on
/// `PATH`. Only when the sources are missing do we attempt to fetch them.
#[cfg(feature = "cpp-backend")]
fn ensure_submodule() {
    let abieos_header = Path::new("lib/abieos/src/abieos.h");
    if abieos_header.exists() {
        return;
    }

    match Command::new("git")
        .args(["submodule", "update", "--init", "--recursive", "-f"])
        .output()
    {
        Ok(output) => {
            if !output.status.success() {
                println!(
                    "cargo:warning=`git submodule update` failed: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            println!(
                "cargo:warning=could not run `git` to fetch the abieos submodule ({e}). \
                 Clone the repository with `--recurse-submodules` or populate `lib/abieos` manually."
            );
        }
    }

    if !abieos_header.exists() {
        panic!(
            "The `lib/abieos` submodule is missing and could not be fetched automatically. \
             Run `git submodule update --init --recursive` and rebuild."
        );
    }
}

/// Generate the FFI bindings for the abieos C API into `OUT_DIR`.
///
/// Bindings are written to `OUT_DIR`, never the source tree: `cargo package`
/// / `cargo publish` verification forbids build scripts from modifying tracked
/// files, and writing into `src/` also caused the committed `bindings.rs` to
/// churn per platform. `lib.rs` always `include!`s `$OUT_DIR/bindings.rs`.
#[cfg(feature = "cpp-backend")]
fn generate_bindings(out_bindings: &Path) {
    bindgen::builder()
        .header("lib/abieos/src/abieos.h")
        .parse_callbacks(Box::new(bindgen::CargoCallbacks::new()))
        .generate()
        .expect("Unable to generate bindings")
        .write_to_file(out_bindings)
        .expect("Couldn't write bindings!");
}

#[cfg(feature = "cpp-backend")]
fn main() {
    let out_dir = env::var_os("OUT_DIR").expect("OUT_DIR not set by cargo");
    let out_bindings = Path::new(&out_dir).join("bindings.rs");

    if env::var("DOCS_RS").is_ok() {
        // docs.rs has no C++ compiler or submodule: skip the native build and
        // bindgen, falling back to the committed canonical bindings so that
        // `lib.rs` (which always includes `$OUT_DIR/bindings.rs`) still builds.
        std::fs::copy("src/bindings.rs", &out_bindings)
            .expect("Couldn't stage committed bindings for docs.rs");
        return;
    }

    // Set by Cargo for build scripts; correct even when cross-compiling
    // (unlike the host-only `sys-info` crate this used to use). Resolve the
    // target *first* so an unsupported target fails with an actionable
    // message before bindgen/libclang or the submodule can raise a more
    // confusing error (e.g. "Unable to find libclang").
    let target_os = env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();
    let target_env = env::var("CARGO_CFG_TARGET_ENV").unwrap_or_default();

    // The vendored abieos C++ relies on GCC/Clang extensions (`__int128`,
    // `__attribute__`, `__builtin_unreachable`) *and* on libstdc++/libc++
    // standard-library semantics — most notably that `std::string_view`'s
    // iterator is a raw `const char*` (eosio/to_json.hpp) and on the set of
    // headers libstdc++ pulls in transitively. MSVC's STL satisfies none of
    // this, and there is no compiler flag that makes clang++ targeting the
    // MSVC ABI behave like libstdc++. Building for `*-pc-windows-msvc` is
    // therefore not supported; fail fast with an actionable message instead of
    // emitting a wall of C++ template errors.
    if target_os == "windows" && target_env == "msvc" {
        panic!(
            "\n\nrs_abieos cannot be built for the `x86_64-pc-windows-msvc` target.\n\
             The vendored abieos C++ library depends on libstdc++/libc++ semantics that\n\
             MSVC's STL does not provide. Build for the GNU (MinGW-w64) target instead:\n\n\
             \x20   rustup target add x86_64-pc-windows-gnu\n\
             \x20   cargo build --target x86_64-pc-windows-gnu\n\n\
             A MinGW-w64 `g++` must be on PATH (e.g. WinLibs or MSYS2). See the README\n\
             \"Windows\" section for details.\n"
        );
    }

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=lib/abieos/src/abieos.cpp");
    println!("cargo:rerun-if-changed=lib/abieos/src/abi.cpp");
    println!("cargo:rerun-if-changed=lib/abieos/src/crypto.cpp");

    ensure_submodule();
    generate_bindings(&out_bindings);

    let mut build = cc::Build::new();
    build
        .cpp(true)
        .includes([
            "lib/abieos/external/rapidjson/include",
            "lib/abieos/include",
        ])
        .files([
            "lib/abieos/src/abieos.cpp",
            "lib/abieos/src/abi.cpp",
            "lib/abieos/src/crypto.cpp",
        ]);

    // GCC/Clang-style flags. `-std=gnu++17` is required (abieos uses GNU
    // extensions); the warning flags are best-effort.
    build
        .flag("-std=gnu++17")
        .flag_if_supported("-Wall")
        .flag_if_supported("-Wextra")
        .flag_if_supported("-Wno-unused-parameter");

    // Link the right C++ standard library for the target. `cc` emits the
    // appropriate `rustc-link-lib` directive itself, so no manual
    // `cargo:rustc-link-*` lines are needed (the old hard-coded
    // `target/lib/build` search path was a no-op bug).
    match target_os.as_str() {
        // macOS / iOS use libc++.
        "macos" | "ios" => {
            build.cpp_link_stdlib("c++");
        }
        // Linux, *-windows-gnu (MinGW-w64) and other GNU targets use libstdc++.
        // (The unsupported windows-msvc target already panicked above.)
        _ => {
            build.cpp_link_stdlib("stdc++");
        }
    }

    build.compile("abieos");
}

#[cfg(not(feature = "cpp-backend"))]
fn main() {
    println!("cargo:rerun-if-changed=build.rs");
}
