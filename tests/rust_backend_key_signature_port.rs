#[cfg(feature = "rust-backend")]
mod rust_backend_key_signature_port {
    use rs_abieos::Abieos;

    #[derive(Clone, Copy)]
    struct SuccessCase {
        ty: &'static str,
        json: &'static str,
        expected_json: Option<&'static str>,
    }

    impl SuccessCase {
        const fn new(ty: &'static str, json: &'static str) -> Self {
            Self {
                ty,
                json,
                expected_json: None,
            }
        }

        const fn with_expected(
            ty: &'static str,
            json: &'static str,
            expected_json: &'static str,
        ) -> Self {
            Self {
                ty,
                json,
                expected_json: Some(expected_json),
            }
        }
    }

    #[derive(Clone, Copy)]
    struct ErrorCase {
        ty: &'static str,
        json: &'static str,
        expected_error: &'static str,
    }

    fn assert_check_type_cases(cases: &[SuccessCase]) {
        let abieos = Abieos::new();

        for case in cases {
            let hex = abieos
                .json_to_hex_native(0, case.ty, case.json)
                .unwrap_or_else(|err| {
                    panic!(
                        "json_to_hex_native failed for type {} and json {}: {}",
                        case.ty, case.json, err
                    )
                });
            let actual = abieos
                .hex_to_json_native(0, case.ty, &hex)
                .unwrap_or_else(|err| {
                    panic!(
                        "hex_to_json_native failed for type {} and json {} (hex {}): {}",
                        case.ty, case.json, hex, err
                    )
                });
            let expected = case.expected_json.unwrap_or(case.json);
            assert_eq!(
                actual, expected,
                "round-trip JSON mismatch for type {} and input {} (hex {})",
                case.ty, case.json, hex
            );
        }
    }

    fn assert_error_cases(cases: &[ErrorCase]) {
        let abieos = Abieos::new();

        for case in cases {
            let err = abieos
                .json_to_hex_native(0, case.ty, case.json)
                .expect_err("case should fail");
            let actual = err.to_string();
            assert!(
                actual.contains(case.expected_error),
                "expected error for type {} and json {} to contain {:?}, got {:?}",
                case.ty,
                case.json,
                case.expected_error,
                actual
            );
        }
    }

    #[test]
    fn ports_legacy_eos_public_keys_to_pub_k1_canonical_json() {
        assert_check_type_cases(&[
            SuccessCase::with_expected(
                "public_key",
                r#""EOS1111111111111111111111111111111114T1Anm""#,
                r#""PUB_K1_11111111111111111111111111111111149Mr2R""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS11111111111111111111111115qCHTcgbQwptSz99m""#,
                r#""PUB_K1_11111111111111111111111115qCHTcgbQwpvP72Uq""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS111111111111111114ZrjxJnU1LA5xSyrWMNuXTrYSJ57""#,
                r#""PUB_K1_111111111111111114ZrjxJnU1LA5xSyrWMNuXTrVub2r""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS1111111113diW7pnisfdBvHTXP7wvW5k5Ky1e5DVuF23dosU""#,
                r#""PUB_K1_1111111113diW7pnisfdBvHTXP7wvW5k5Ky1e5DVuF4PizpM""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS11DsZ6Lyr1aXpm9aBqqgV4iFJpNbSw5eE9LLTwNAxqjJgmjgbT""#,
                r#""PUB_K1_11DsZ6Lyr1aXpm9aBqqgV4iFJpNbSw5eE9LLTwNAxqjJgXSdB8""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS12wkBET2rRgE8pahuaczxKbmv7ciehqsne57F9gtzf1PVYNMRa2""#,
                r#""PUB_K1_12wkBET2rRgE8pahuaczxKbmv7ciehqsne57F9gtzf1PVb7Rf7o""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS1yp8ebBuKZ13orqUrZsGsP49e6K3ThVK1nLutxSyU5j9SaXz9a""#,
                r#""PUB_K1_1yp8ebBuKZ13orqUrZsGsP49e6K3ThVK1nLutxSyU5j9Tx1r96""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS9adaAMuB9v8yX1mZ5PtoB6VFSCeqRGjASd8ZTM6VUkiHL7mue4K""#,
                r#""PUB_K1_9adaAMuB9v8yX1mZ5PtoB6VFSCeqRGjASd8ZTM6VUkiHLB5XEdw""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS69X3383RzBZj41k73CSjUNXM5MYGpnDxyPnWUKPEtYQmTBWz4D""#,
                r#""PUB_K1_69X3383RzBZj41k73CSjUNXM5MYGpnDxyPnWUKPEtYQmVzqTY7""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS7yBtksm8Kkg85r4in4uCbfN77uRwe82apM8jjbhFVDgEgz3w8S""#,
                r#""PUB_K1_7yBtksm8Kkg85r4in4uCbfN77uRwe82apM8jjbhFVDgEcarGb8""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS7WnhaKwHpbSidYuh2DF1qAExTRUtPEdZCaZqt75cKcixuQUtdA""#,
                r#""PUB_K1_7WnhaKwHpbSidYuh2DF1qAExTRUtPEdZCaZqt75cKcixtU7gEn""#,
            ),
            SuccessCase::with_expected(
                "public_key",
                r#""EOS7Bn1YDeZ18w2N9DU4KAJxZDt6hk3L7eUwFRAc1hb5bp6xJwxNV""#,
                r#""PUB_K1_7Bn1YDeZ18w2N9DU4KAJxZDt6hk3L7eUwFRAc1hb5bp6uEBZA8""#,
            ),
        ]);
    }

    #[test]
    fn ports_pub_k1_pub_r1_and_pub_wa_public_key_success_cases() {
        assert_check_type_cases(&[
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_11111111111111111111111111111111149Mr2R""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_11111111111111111111111115qCHTcgbQwpvP72Uq""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_111111111111111114ZrjxJnU1LA5xSyrWMNuXTrVub2r""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_1111111113diW7pnisfdBvHTXP7wvW5k5Ky1e5DVuF4PizpM""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_11DsZ6Lyr1aXpm9aBqqgV4iFJpNbSw5eE9LLTwNAxqjJgXSdB8""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_12wkBET2rRgE8pahuaczxKbmv7ciehqsne57F9gtzf1PVb7Rf7o""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_1yp8ebBuKZ13orqUrZsGsP49e6K3ThVK1nLutxSyU5j9Tx1r96""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_9adaAMuB9v8yX1mZ5PtoB6VFSCeqRGjASd8ZTM6VUkiHLB5XEdw""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_69X3383RzBZj41k73CSjUNXM5MYGpnDxyPnWUKPEtYQmVzqTY7""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_7yBtksm8Kkg85r4in4uCbfN77uRwe82apM8jjbhFVDgEcarGb8""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_7WnhaKwHpbSidYuh2DF1qAExTRUtPEdZCaZqt75cKcixtU7gEn""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_K1_7Bn1YDeZ18w2N9DU4KAJxZDt6hk3L7eUwFRAc1hb5bp6uEBZA8""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_R1_1111111111111111111111111111111116amPNj""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_R1_67vQGPDMCR4gbqYV3hkfNz3BfzRmmSj27kFDKrwDbaZKtaX36u""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_R1_6FPFZqw5ahYrR9jD96yDbbDNTdKtNqRbze6oTDLntrsANgQKZu""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_R1_7zetsBPJwGQqgmhVjviZUfoBMktHinmTqtLczbQqrBjhaBgi6x""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_WA_8PPYTWYNkRqrveNAoX7PJWDtSqDUp3c29QGBfr6MD9EaLocaPBmsk5QAHWq4vEQt2""#,
            ),
            SuccessCase::new(
                "public_key",
                r#""PUB_WA_6VFnP5vnq1GjNyMR7S17e2yp6SRoChiborF2LumbnXvMTsPASXykJaBBGLhprXTpk""#,
            ),
        ]);
    }

    #[test]
    fn ports_private_key_success_and_legacy_wif_canonicalization_cases() {
        assert_check_type_cases(&[
            SuccessCase::new(
                "private_key",
                r#""PVT_R1_PtoxLPzJZURZmPS4e26pjBiAn41mkkLPrET5qHnwDvbvqFEL6""#,
            ),
            SuccessCase::new(
                "private_key",
                r#""PVT_R1_vbRKUuE34hjMVQiePj2FEjM8FvuG7yemzQsmzx89kPS9J8Coz""#,
            ),
            SuccessCase::with_expected(
                "private_key",
                r#""5KQwrPbwdL6PhXujxW37FSSQZ1JiwsST4cqQzDeyXtP79zkvFD3""#,
                r#""PVT_K1_2bfGi9rYsXQSXXTvJbDAPhHLQUojjaNLomdm3cEJ1XTzMqUt3V""#,
            ),
        ]);
    }

    #[test]
    fn ports_signature_k1_r1_and_wa_success_cases() {
        assert_check_type_cases(&[
            SuccessCase::new(
                "signature",
                r#""SIG_K1_Kg2UKjXTX48gw2wWH4zmsZmWu3yarcfC21Bd9JPj7QoDURqiAacCHmtExPk3syPb2tFLsp1R4ttXLXgr7FYgDvKPC5RCkx""#,
            ),
            SuccessCase::new(
                "signature",
                r#""SIG_R1_Kfh19CfEcQ6pxkMBz6xe9mtqKuPooaoyatPYWtwXbtwHUHU8YLzxPGvZhkqgnp82J41e9R6r5mcpnxy1wAf1w9Vyo9wybZ""#,
            ),
            SuccessCase::new(
                "signature",
                r#""SIG_WA_FjWGWXz7AC54NrVWXS8y8DGu1aesCr7oFiFmVg4a1QfNS74JwaVkqkN8xbMD64uvcsmPvtNnA9du6G6nSsWuyT9tM8CQw9mV1BSbWEs8hjF1uFBP1QHAEadvhkZQPU1FTyPMz4jevaHYMQgfMiAf3QoPhPn9RGxzvNph8Zrd6F3pKpZkUe92tGQU8PQvEMa22ELPvdXzxXC6qUKnKVSH4gK7BXw168jb5d3nnWrpQ1yrLTWB4xizEMpN8sTfsgScKKx1QajX2uNUahQEb1cxipQZbVMApifHEUsK45PqsNxfXvb""#,
            ),
            SuccessCase::new(
                "signature",
                r#""SIG_WA_FejsRu4VrdwoZ27v2D3wmp4Kge46JJSqWsiMgbJapVuuYnPDyZZjJSTggdHUNPMp3zt2fGfAdpWY7ScsohZzWTJ1iTerbab2pNE6Tso7MJRjdMAG56K4fjrASEK6QsUs7rxG9Syp7kstBcq8eZidayrtK9YSH1MCNTAqrDPMbN366vR8q5XeN5BSDmyDsqmjsMMSKWMeEbUi7jNHKLziZY6dKHNqDYqjmDmuXoevxyDRWrNVHjAzvBtfTuVtj2r5tCScdCZ3a7yQ1D2zZvstphB4t5HN9YXw1HGS3yKCY6uRZ2V""#,
            ),
        ]);
    }

    #[test]
    fn ports_key_and_signature_failure_cases() {
        assert_error_cases(&[
            ErrorCase {
                ty: "public_key",
                json: "true",
                expected_error: "expected string",
            },
            ErrorCase {
                ty: "private_key",
                json: "true",
                expected_error: "expected string",
            },
            ErrorCase {
                ty: "signature",
                json: "true",
                expected_error: "expected string",
            },
            ErrorCase {
                ty: "public_key",
                json: r#""foo""#,
                expected_error: "unrecognized key format",
            },
            ErrorCase {
                ty: "private_key",
                json: r#""PVT_X1_11111""#,
                expected_error: "expected private_key",
            },
            ErrorCase {
                ty: "signature",
                json: r#""foo""#,
                expected_error: "unrecognized key format",
            },
            ErrorCase {
                ty: "public_key",
                json: r#""PUB_K1_11111111111111111111111111111111149Mr2S""#,
                expected_error: "expected key",
            },
            ErrorCase {
                ty: "private_key",
                json: r#""PVT_R1_PtoxLPzJZURZmPS4e26pjBiAn41mkkLPrET5qHnwDvbvqFEL7""#,
                expected_error: "expected key",
            },
            ErrorCase {
                ty: "signature",
                json: r#""SIG_K1_Kg2UKjXTX48gw2wWH4zmsZmWu3yarcfC21Bd9JPj7QoDURqiAacCHmtExPk3syPb2tFLsp1R4ttXLXgr7FYgDvKPC5RCky""#,
                expected_error: "expected key",
            },
            ErrorCase {
                ty: "public_key",
                json: r#""PUB_K1_11111""#,
                expected_error: "key has invalid size",
            },
            ErrorCase {
                ty: "private_key",
                json: r#""11111""#,
                expected_error: "key has invalid size",
            },
            ErrorCase {
                ty: "signature",
                json: r#""SIG_K1_11111""#,
                expected_error: "key has invalid size",
            },
        ]);
    }
}
