#[cfg(feature = "rust-backend")]
mod rust_backend_abi_conversion_port {
    use rs_abieos::Abieos;

    fn err_string<T>(result: Result<T, impl std::fmt::Display>) -> String {
        result
            .err()
            .map(|err| err.to_string())
            .expect("case unexpectedly succeeded")
    }

    fn assert_contains_all(err: &str, contains: &[&str], label: &str) {
        for expected in contains {
            assert!(
                err.contains(expected),
                "{label}: expected error to contain {expected:?}, got {err:?}"
            );
        }
    }

    fn bytes_to_hex(bytes: &[u8]) -> String {
        bytes.iter().map(|byte| format!("{byte:02x}")).collect()
    }

    fn abi_bin_with_version(version: &str) -> Vec<u8> {
        let mut bin = Vec::with_capacity(1 + version.len() + 7);
        bin.push(version.len() as u8);
        bin.extend_from_slice(version.as_bytes());
        bin.extend_from_slice(&[0; 7]);
        bin
    }

    fn truncated_abi_bin_with_version(version: &str) -> Vec<u8> {
        let mut bin = Vec::with_capacity(1 + version.len());
        bin.push(version.len() as u8);
        bin.extend_from_slice(version.as_bytes());
        bin
    }

    #[test]
    fn rust_backend_ports_set_abi_hex_edge_errors() {
        let abieos = Abieos::new();

        let empty_version_bin = abi_bin_with_version("");
        let unsupported_version_bin = abi_bin_with_version("eosio::abi/9.0");
        let truncated_abi_1_0 = truncated_abi_bin_with_version("eosio::abi/1.0");
        let truncated_abi_1_1 = truncated_abi_bin_with_version("eosio::abi/1.1");
        let empty_version_hex = bytes_to_hex(&empty_version_bin);
        let unsupported_version_hex = bytes_to_hex(&unsupported_version_bin);
        let truncated_abi_1_0_hex = bytes_to_hex(&truncated_abi_1_0);
        let truncated_abi_1_1_hex = bytes_to_hex(&truncated_abi_1_1);

        let cases: &[(&str, &str, &[&str])] = &[
            ("empty set_abi_hex", "", &["no data"]),
            (
                "bare empty version byte is truncated",
                "00",
                &["read datastream"],
            ),
            (
                "empty version",
                &empty_version_hex,
                &["unsupported abi version"],
            ),
            (
                "unsupported eosio abi 9.0",
                &unsupported_version_hex,
                &["unsupported abi version"],
            ),
            (
                "truncated eosio abi 1.0",
                &truncated_abi_1_0_hex,
                &["read datastream"],
            ),
            (
                "truncated eosio abi 1.1",
                &truncated_abi_1_1_hex,
                &["read datastream"],
            ),
            (
                "odd hex digit count",
                "0",
                &["Expected string containing hex"],
            ),
            (
                "non-hex ABI bytes",
                "zz",
                &["Expected string containing hex"],
            ),
        ];

        for (label, hex, contains) in cases.iter().copied() {
            let err = err_string(abieos.set_abi_hex_native(8, hex));
            assert_contains_all(&err, contains, label);
        }
    }

    #[test]
    fn rust_backend_ports_set_abi_bin_edge_errors() {
        let abieos = Abieos::new();

        let empty_version_bin = abi_bin_with_version("");
        let unsupported_version_bin = abi_bin_with_version("eosio::abi/9.0");
        let truncated_abi_1_0 = truncated_abi_bin_with_version("eosio::abi/1.0");
        let truncated_abi_1_1 = truncated_abi_bin_with_version("eosio::abi/1.1");

        let cases: &[(&str, &[u8], &[&str])] = &[
            ("empty set_abi_bin", &[], &["no data"]),
            (
                "bare empty version byte is truncated",
                &[0],
                &["read datastream"],
            ),
            (
                "empty version",
                &empty_version_bin,
                &["unsupported abi version"],
            ),
            (
                "unsupported eosio abi 9.0",
                &unsupported_version_bin,
                &["unsupported abi version"],
            ),
            (
                "truncated eosio abi 1.0",
                &truncated_abi_1_0,
                &["read datastream"],
            ),
            (
                "truncated eosio abi 1.1",
                &truncated_abi_1_1,
                &["read datastream"],
            ),
            (
                "unterminated version length varuint",
                &[0x80],
                &["read datastream"],
            ),
            (
                "invalid UTF-8 in version string",
                &[1, 0xff],
                &["Invalid encoding in string"],
            ),
        ];

        for (label, bin, contains) in cases.iter().copied() {
            let err = err_string(abieos.set_abi_bin_native(8, bin));
            assert_contains_all(&err, contains, label);
        }
    }

    #[test]
    fn rust_backend_ports_abi_json_to_bin_version_errors() {
        let abieos = Abieos::new();

        let cases = [
            (
                "set_abi_json rejects unsupported version",
                r#"{"version":"eosio::abi/9.0"}"#,
                &["unsupported abi version"][..],
            ),
            (
                "abi_json_to_bin rejects unsupported version",
                r#"{"version":"eosio::abi/9.0"}"#,
                &["unsupported abi version"][..],
            ),
            (
                "set_abi_json rejects non-string version",
                r#"{"version":true}"#,
                &["expected string"][..],
            ),
            (
                "abi_json_to_bin rejects non-string version",
                r#"{"version":true}"#,
                &["expected string"][..],
            ),
        ];

        for (label, abi_json, contains) in cases {
            let err = if label.starts_with("set_abi_json") {
                err_string(abieos.set_abi_json_native(8, abi_json))
            } else {
                err_string(abieos.abi_json_to_bin(abi_json))
            };
            assert_contains_all(&err, contains, label);
        }
    }

    #[test]
    fn rust_backend_ports_abi_bin_to_json_edge_errors() {
        let abieos = Abieos::new();

        let empty_version_bin = abi_bin_with_version("");
        let unsupported_version_bin = abi_bin_with_version("eosio::abi/9.0");
        let truncated_abi_1_0 = truncated_abi_bin_with_version("eosio::abi/1.0");
        let truncated_abi_1_1 = truncated_abi_bin_with_version("eosio::abi/1.1");

        let cases: &[(&str, &[u8], &[&str])] = &[
            ("empty abi_bin_to_json", &[], &["no data"]),
            (
                "bare empty version byte is truncated",
                &[0],
                &["read datastream"],
            ),
            (
                "empty version",
                &empty_version_bin,
                &["unsupported abi version"],
            ),
            (
                "unsupported eosio abi 9.0",
                &unsupported_version_bin,
                &["unsupported abi version"],
            ),
            (
                "truncated eosio abi 1.0",
                &truncated_abi_1_0,
                &["read datastream"],
            ),
            (
                "truncated eosio abi 1.1",
                &truncated_abi_1_1,
                &["read datastream"],
            ),
            (
                "unterminated version length varuint",
                &[0x80],
                &["read datastream"],
            ),
            (
                "invalid UTF-8 in version string",
                &[1, 0xff],
                &["Invalid encoding in string"],
            ),
        ];

        for (label, bin, contains) in cases.iter().copied() {
            let err = err_string(abieos.abi_bin_to_json(bin));
            assert_contains_all(&err, contains, label);
        }
    }

    #[test]
    fn rust_backend_ports_minimal_json_abi_setup_and_conversion() {
        let abieos = Abieos::new();

        assert!(abieos
            .set_abi_json_native(8, r#"{"version":"eosio::abi/1.0"}"#)
            .expect("minimal abi/1.0 setup should succeed"));
        assert!(abieos
            .set_abi_json_native(8, r#"{"version":"eosio::abi/1.1"}"#)
            .expect("minimal abi/1.1 setup should succeed"));

        let bin = abieos
            .abi_json_to_bin(r#"{"version":"eosio::abi/1.1"}"#)
            .expect("minimal abi_json_to_bin should succeed");
        let json = abieos
            .abi_bin_to_json(&bin)
            .expect("minimal abi_bin_to_json should succeed");

        assert!(
            json.contains(r#""version":"eosio::abi/1.1""#),
            "expected recovered JSON version, got {json:?}"
        );
    }
}
