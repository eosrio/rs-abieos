use rs_abieos::Abieos;

#[test]
fn safe_api_is_available_with_selected_backend() {
    let abieos = Abieos::new();
    let name = abieos.string_to_name("eosio").unwrap();

    assert_eq!(abieos.name_to_string(name).unwrap(), "eosio");
}

#[cfg(all(feature = "cpp-backend", not(feature = "rust-backend")))]
#[test]
fn default_cpp_backend_selects_cpp_for_safe_api() {
    let active_context = std::any::type_name::<rs_abieos::abieos_context_s>();

    assert!(
        active_context.contains("backend::cpp"),
        "safe API context should come from the C++ backend, got {active_context}"
    );
}

#[cfg(feature = "rust-backend")]
#[test]
fn rust_backend_selects_rust_for_safe_api() {
    let active_context = std::any::type_name::<rs_abieos::abieos_context_s>();

    assert!(
        active_context.contains("backend::rust"),
        "safe API context should come from the Rust backend, got {active_context}"
    );
}

#[cfg(all(feature = "rust-backend", feature = "cpp-oracle"))]
#[test]
fn rust_backend_with_cpp_oracle_keeps_oracle_separate() {
    let safe_context = std::any::type_name::<rs_abieos::abieos_context_s>();
    let oracle_context = std::any::type_name::<rs_abieos::cpp_oracle::abieos_context_s>();

    assert!(
        safe_context.contains("backend::rust"),
        "safe API context should stay on the Rust backend, got {safe_context}"
    );
    assert!(
        oracle_context.contains("backend::cpp"),
        "oracle context should come from the C++ backend, got {oracle_context}"
    );
}
