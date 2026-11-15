use mizar_session::{
    Hash, PipelinePhase, bridge_fingerprint_hash, domain_hasher, lower_hex, module_work_unit_label,
};

#[test]
fn pipeline_phases_keep_identity_and_cli_labels() {
    let phases = [
        (
            PipelinePhase::PackageResolve,
            "PackageResolve",
            "package_resolve",
        ),
        (PipelinePhase::SourceLoad, "SourceLoad", "source_load"),
        (PipelinePhase::Frontend, "Frontend", "frontend"),
        (
            PipelinePhase::ModuleResolve,
            "ModuleResolve",
            "module_resolve",
        ),
        (
            PipelinePhase::SignatureCollection,
            "SignatureCollection",
            "signature_collection",
        ),
        (PipelinePhase::TypeChecking, "TypeChecking", "type_checking"),
        (
            PipelinePhase::RegistrationResolution,
            "RegistrationResolution",
            "registration_resolution",
        ),
        (
            PipelinePhase::OverloadResolution,
            "OverloadResolution",
            "overload_resolution",
        ),
        (PipelinePhase::Elaboration, "Elaboration", "elaboration"),
        (
            PipelinePhase::AlgorithmPreparation,
            "AlgorithmPreparation",
            "algorithm_preparation",
        ),
        (PipelinePhase::VcGenerate, "VcGenerate", "vc_generate"),
        (PipelinePhase::VcDischarge, "VcDischarge", "vc_discharge"),
        (PipelinePhase::AtpSolve, "AtpSolve", "atp_solve"),
        (PipelinePhase::BackendRun, "BackendRun", "backend_run"),
        (PipelinePhase::KernelCheck, "KernelCheck", "kernel_check"),
        (
            PipelinePhase::ArtifactCommit,
            "ArtifactCommit",
            "artifact_commit",
        ),
        (
            PipelinePhase::DocumentationExtract,
            "DocumentationExtract",
            "documentation_extract",
        ),
    ];

    for (phase, identity, cli) in phases {
        assert_eq!(phase.as_str(), identity);
        assert_eq!(phase.as_cli_str(), cli);
    }
}

#[test]
fn module_work_unit_labels_keep_debug_quoted_format() {
    assert_eq!(
        module_work_unit_label("mml", "algebra"),
        "\"mml\":\"algebra\""
    );
    assert_eq!(module_work_unit_label("", ""), "\"\":\"\"");
    assert_eq!(
        module_work_unit_label("pkg\n", "dir\\module\"雪"),
        "\"pkg\\n\":\"dir\\\\module\\\"雪\"",
    );
}

#[test]
fn hash_lower_hex_round_trips_and_rejects_noncanonical_input() {
    let zero = Hash::from_bytes([0; Hash::BYTE_LEN]);
    assert_eq!(
        zero.to_lower_hex(),
        "0000000000000000000000000000000000000000000000000000000000000000"
    );

    let full = Hash::from_bytes([0xff; Hash::BYTE_LEN]);
    assert_eq!(
        full.to_lower_hex(),
        "ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff"
    );
    for hash in [zero, full] {
        assert_eq!(Hash::from_lower_hex(&hash.to_lower_hex()), Some(hash));
    }
    let all_bytes: Vec<u8> = (0..=255).collect();
    assert_eq!(
        lower_hex(&all_bytes),
        concat!(
            "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f",
            "202122232425262728292a2b2c2d2e2f303132333435363738393a3b3c3d3e3f",
            "404142434445464748494a4b4c4d4e4f505152535455565758595a5b5c5d5e5f",
            "606162636465666768696a6b6c6d6e6f707172737475767778797a7b7c7d7e7f",
            "808182838485868788898a8b8c8d8e8f909192939495969798999a9b9c9d9e9f",
            "a0a1a2a3a4a5a6a7a8a9aaabacadaeafb0b1b2b3b4b5b6b7b8b9babbbcbdbebf",
            "c0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedf",
            "e0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff",
        )
    );

    let mixed_bytes = core::array::from_fn(|index| {
        u8::try_from(index).expect("Hash byte index should fit in u8")
    });
    let mixed = Hash::from_bytes(mixed_bytes);
    let mixed_hex = "000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f";
    assert_eq!(mixed.to_lower_hex(), mixed_hex);
    assert_eq!(Hash::from_lower_hex(mixed_hex), Some(mixed));
    assert_eq!(Hash::from_lower_hex(&mixed_hex.to_uppercase()), None);

    for malformed in [
        "00",
        "g000000000000000000000000000000000000000000000000000000000000000",
        "０000000000000000000000000000000000000000000000000000000000000000",
    ] {
        assert_eq!(Hash::from_lower_hex(malformed), None, "{malformed:?}");
    }
    assert_eq!(Hash::from_lower_hex(&"é".repeat(32)), None);
}

#[test]
fn domain_hashers_keep_length_framed_blake3_digests() {
    let empty = domain_hasher("");
    assert_eq!(
        empty.finalize().to_hex().to_string(),
        "71e0a99173564931c0b8acc52d2685a8e39c64dc52e3d02390fdac2a12b155cb"
    );

    let mut ordinary = domain_hasher("cache-key");
    ordinary.update(b"artifact");
    assert_eq!(
        ordinary.finalize().to_hex().to_string(),
        "75384b2bf5868a33e70983875fb7c280ba4dfd8a4fffe74b098bcd9961ee50c0"
    );

    let mut unicode = domain_hasher("λ/長");
    unicode.update("payload-é".as_bytes());
    assert_eq!(
        unicode.finalize().to_hex().to_string(),
        "232d40954c92f89304347a0a578c9c129ffd8df3ee6a2fb74ae3be75c0a43946"
    );

    let mut two_domains = domain_hasher("outer");
    two_domains.update(&("inner".len() as u64).to_le_bytes());
    two_domains.update(b"inner");
    two_domains.update(b"payload");
    let framed = two_domains.finalize().to_hex().to_string();
    assert_eq!(
        framed,
        "0f4623e04c490616454c3de1df508e7e16e1e8d66f493d352572a1701e239a0f"
    );

    let mut concatenated = domain_hasher("outerinner");
    concatenated.update(b"payload");
    assert_eq!(
        concatenated.finalize().to_hex().to_string(),
        "b41ffb14bfef439d15fc8432710c35619f3bbd398bd636afde947dc8797a535e",
    );
    assert_ne!(
        framed,
        "b41ffb14bfef439d15fc8432710c35619f3bbd398bd636afde947dc8797a535e"
    );
}

#[test]
fn bridge_hashes_keep_predecessor_golden_values() {
    assert_eq!(
        bridge_fingerprint_hash("", &[]).to_lower_hex(),
        "3fba9630b640faae70726f6f662d6964a92a6b30124d7a692b8482a2c85ea33f",
    );
    assert_eq!(
        bridge_fingerprint_hash("vc", b"payload").to_lower_hex(),
        "ea757d7c032f2e5bc88b1009df289164c9a022fcdbc4908684fe4ae94b4ad218",
    );
    assert_eq!(
        bridge_fingerprint_hash("mizar-vc-λ/長", "é\0雪".as_bytes()).to_lower_hex(),
        "964f7f84a705bae1b2f8315870b4e0b6e15b4e69bc78242d518945ef094f0191",
    );

    let long_payload: Vec<u8> = (0..=255).cycle().take(1024).collect();
    assert_eq!(
        bridge_fingerprint_hash("kernel", &long_payload).to_lower_hex(),
        "9487b400faaf306a1caa453a1a7e7312cd0e9b834945ac8cd3fe1c4b580895ff",
    );
    assert_eq!(
        bridge_fingerprint_hash(&"long-domain/".repeat(128), &long_payload).to_lower_hex(),
        "04407b23ae75ad1be1fd1bfa75a40ce4a9d4be8009bdebb48b8c346f24cbc438",
    );
}
