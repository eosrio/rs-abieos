#[cfg(feature = "rust-backend")]
mod rust_backend_type_spec_error_port {
    use rs_abieos::Abieos;

    struct Case {
        label: &'static str,
        ty: &'static str,
        json: &'static str,
        contains: &'static [&'static str],
    }

    fn err_string<T>(result: Result<T, impl std::fmt::Display>, label: &str) -> String {
        result
            .err()
            .map(|err| err.to_string())
            .unwrap_or_else(|| panic!("{label}: case unexpectedly succeeded"))
    }

    fn assert_contains_all(err: &str, contains: &[&str], label: &str) {
        for expected in contains {
            assert!(
                err.contains(expected),
                "{label}: expected error to contain {expected:?}, got {err:?}"
            );
        }
    }

    #[test]
    fn rust_backend_ports_malformed_fixed_array_type_spec_errors() {
        let abieos = Abieos::new();

        for case in [
            Case {
                label: "unmatched fixed-array close bracket",
                ty: "int80]",
                json: "0",
                contains: &["without matching"],
            },
            Case {
                label: "zero-length fixed array",
                ty: "int8[0]",
                json: "[]",
                contains: &["Zero size fixed arrays not allowed"],
            },
            Case {
                label: "negative fixed array",
                ty: "int8[-1]",
                json: "[]",
                contains: &["Negative size fixed arrays not allowed"],
            },
            Case {
                label: "hex-like fixed-array size",
                ty: "int8[0x5]",
                json: "[]",
                // C++ reports "Unexpected size specification"; Rust currently
                // rejects this earlier as a leading-zero length.
                contains: &["Leading zeros not allowed"],
            },
            Case {
                label: "leading-zero fixed-array size",
                ty: "int8[010]",
                json: "[]",
                contains: &["Leading zeros not allowed"],
            },
            Case {
                label: "plus-sign fixed-array size",
                ty: "int8[+5]",
                json: "[]",
                contains: &["Unexpected size specification"],
            },
            Case {
                label: "optional inside array nesting",
                ty: "int8?[]",
                json: "[]",
                contains: &["Invalid array nesting"],
            },
            Case {
                label: "optional inside fixed-array nesting",
                ty: "int8?[1]",
                json: "[]",
                contains: &["Invalid array nesting"],
            },
            Case {
                label: "binary extension inside optional nesting",
                ty: "int8$?",
                json: "null",
                contains: &["Invalid optional nesting"],
            },
            Case {
                label: "binary extension inside array nesting",
                ty: "int8$[]",
                json: "[]",
                contains: &["Invalid array nesting"],
            },
            Case {
                label: "binary extension inside fixed-array nesting",
                ty: "int8$[11]",
                json: "[]",
                contains: &["Invalid array nesting"],
            },
            Case {
                label: "nested binary extension",
                ty: "int8$$",
                json: "0",
                contains: &["Invalid extension nesting"],
            },
        ] {
            let err = err_string(abieos.json_to_hex_native(0, case.ty, case.json), case.label);
            assert_contains_all(&err, case.contains, case.label);
        }
    }

    #[test]
    fn rust_backend_ports_recursion_limit_and_unknown_type_errors() {
        let abieos = Abieos::new();
        let nested_array = format!("{}{}{}", "[".repeat(130), "[]", "]".repeat(130));

        let err = err_string(
            abieos.json_to_hex_native(0, "int8", &nested_array),
            "deep json recursion limit",
        );
        assert_contains_all(
            &err,
            &["recursion limit reached"],
            "deep json recursion limit",
        );

        let err = err_string(abieos.json_to_hex_native(0, "fee", "0"), "unknown type");
        assert_contains_all(&err, &["unknown type", "fee"], "unknown type");
    }

    #[test]
    fn rust_backend_ports_extension_typedef_error() {
        let abieos = Abieos::new();
        let bad_abi = r#"{
            "version": "eosio::abi/1.0",
            "types": [
                {
                    "new_type_name": "my_alias",
                    "type": "int8$"
                }
            ],
            "structs": [],
            "actions": [],
            "tables": [],
            "ricardian_clauses": [],
            "error_messages": [],
            "abi_extensions": [],
            "variants": []
        }"#;
        let err = abieos.set_abi_json("bad", bad_abi).unwrap_err().to_string();
        assert!(
            err.contains("Extension typedef not allowed"),
            "expected extension typedef error, got {err}"
        );
    }
}
