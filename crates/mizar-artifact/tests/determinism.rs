use mizar_artifact::{
    ArtifactManifest, BuildProvenance, DependencyArtifactHash, ExportedItem, ExportedLabel,
    ExpressionMetadata, ImportedInterface, ManifestTransaction, MetadataField, ModuleArtifactEntry,
    ModuleSummary, ObligationMetadata, ProjectedDiagnostic, ProofWitnessEntry, ProofWitnessRef,
    RegistrationSummary, VerifiedArtifact,
};
use mizar_artifact::{ArtifactStore, content_hash_str};
use mizar_artifact::{ClusterTraceRef, ExportedRegistration};
use mizar_session::Hash;
use std::fs;

#[test]
fn construction_order_does_not_change_public_artifact_bytes_or_hashes() {
    let first = observation(false);
    let second = observation(true);

    assert_eq!(first.module_summary_json, second.module_summary_json);
    assert_eq!(
        first.registration_summary_json,
        second.registration_summary_json
    );
    assert_eq!(first.verified_artifact_json, second.verified_artifact_json);
    assert_eq!(first.manifest_json, second.manifest_json);
    assert_eq!(first.interface_hash, second.interface_hash);
    assert_eq!(first.implementation_hash, second.implementation_hash);
}

#[test]
fn re_emitted_manifest_bytes_are_identical() {
    let root = temp_root("reemission");
    let store = ArtifactStore::new(&root);
    let artifact = verified_artifact(false);
    let artifact_hash = store
        .write_named_str("alpha.mizir.json", &artifact.to_json_string())
        .unwrap();
    let witness_hash = store
        .write_named_str("proof-witnesses/alpha/vc-1.json", "{\"proof\":true}")
        .unwrap();
    let entry = ModuleArtifactEntry::new(
        "alpha",
        "src/alpha.miz",
        content_hash_str("source"),
        "alpha.mizir.json",
        artifact_hash,
        artifact.interface_hash,
        artifact.implementation_hash,
        vec![ProofWitnessEntry::new(
            "wit-1",
            "proof-witnesses/alpha/vc-1.json",
            witness_hash,
        )],
        Some(artifact.diagnostics_hash()),
    );

    let mut first_tx = ManifestTransaction::begin(
        "pkg",
        "0.1.0",
        content_hash_str("lock"),
        "toolchain",
        content_hash_str("config"),
        None,
    );
    first_tx.stage_module(entry.clone());
    let first = first_tx.commit(&store).unwrap().to_json_string();

    let mut second_tx = ManifestTransaction::begin(
        "pkg",
        "0.1.0",
        content_hash_str("lock"),
        "toolchain",
        content_hash_str("config"),
        Some(ArtifactManifest::read_from_store(&store).unwrap()),
    );
    second_tx.stage_module(entry);
    let second = second_tx.commit(&store).unwrap().to_json_string();

    assert_eq!(first, second);
    fs::remove_dir_all(root).unwrap();
}

#[test]
fn canonical_manifest_golden_bytes_are_stable() {
    let zero = Hash::from_bytes([0; Hash::BYTE_LEN]);
    let manifest = ArtifactManifest::new(
        "pkg",
        "0.1.0",
        zero,
        "toolchain",
        zero,
        vec![ModuleArtifactEntry::new(
            "alpha",
            "src/alpha.miz",
            zero,
            "alpha.mizir.json",
            zero,
            zero,
            zero,
            vec![ProofWitnessEntry::new(
                "wit-1",
                "proof-witnesses/alpha/vc-1.json",
                zero,
            )],
            Some(zero),
        )],
    );

    assert_eq!(
        manifest.to_json_string(),
        "{\"lockfile_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"modules\":[{\"artifact_file\":\"alpha.mizir.json\",\"artifact_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"diagnostics_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"implementation_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"interface_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"module_path\":\"alpha\",\"proof_witnesses\":[{\"content_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\",\"path\":\"proof-witnesses/alpha/vc-1.json\",\"witness_id\":\"wit-1\"}],\"source_file\":\"src/alpha.miz\",\"source_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\"}],\"package_id\":\"pkg\",\"package_version\":\"0.1.0\",\"schema_version\":\"mizar-artifact/manifest/v1\",\"toolchain\":\"toolchain\",\"verifier_config_hash\":\"0000000000000000000000000000000000000000000000000000000000000000\"}"
    );
}

#[derive(Debug, PartialEq, Eq)]
struct Observation {
    module_summary_json: String,
    registration_summary_json: String,
    verified_artifact_json: String,
    manifest_json: String,
    interface_hash: mizar_session::Hash,
    implementation_hash: mizar_session::Hash,
}

fn observation(reverse: bool) -> Observation {
    let module_summary = module_summary(reverse);
    let registration_summary = registration_summary(reverse);
    let artifact = verified_artifact(reverse);
    let manifest = ArtifactManifest::new(
        "pkg",
        "0.1.0",
        content_hash_str("lock"),
        "toolchain",
        content_hash_str("config"),
        vec![ModuleArtifactEntry::new(
            "alpha",
            "src/alpha.miz",
            content_hash_str("source"),
            "alpha.mizir.json",
            content_hash_str(&artifact.to_json_string()),
            artifact.interface_hash,
            artifact.implementation_hash,
            vec![ProofWitnessEntry::new(
                "wit-1",
                "proof-witnesses/alpha/vc-1.json",
                content_hash_str("{\"proof\":true}"),
            )],
            Some(artifact.diagnostics_hash()),
        )],
    );

    Observation {
        module_summary_json: module_summary.to_json_string(),
        registration_summary_json: registration_summary.to_json_string(),
        verified_artifact_json: artifact.to_json_string(),
        manifest_json: manifest.to_json_string(),
        interface_hash: artifact.interface_hash,
        implementation_hash: artifact.implementation_hash,
    }
}

fn module_summary(reverse: bool) -> ModuleSummary {
    let mut exports = vec![
        ExportedItem::new(
            "theorem",
            "pkg::alpha::Th1",
            "Th1",
            "public",
            "for x holds x = x",
            None,
        ),
        ExportedItem::new(
            "predicate",
            "pkg::alpha::P/1",
            "P",
            "public",
            "P(x: set)",
            None,
        ),
    ];
    let mut labels = vec![
        ExportedLabel::new(
            "label:Th1",
            "Th1",
            "theorem",
            Some("pkg::alpha::Th1".to_owned()),
        ),
        ExportedLabel::new("label:Def1", "Def1", "definition", None),
    ];
    let mut imports = vec![
        ImportedInterface::new("mml.hidden", content_hash_str("hidden")),
        ImportedInterface::new("mml.tarski", content_hash_str("tarski")),
    ];
    if reverse {
        exports.reverse();
        labels.reverse();
        imports.reverse();
    }
    ModuleSummary::new(
        "pkg",
        "alpha",
        content_hash_str("source"),
        exports,
        labels,
        imports,
        Some("local".to_owned()),
    )
}

fn registration_summary(reverse: bool) -> RegistrationSummary {
    let mut registrations = vec![
        ExportedRegistration::new(
            "reg-1",
            "RegA",
            "existential",
            "T:set",
            "cluster set -> non empty",
            "inhabited set",
            "kernel_verified",
            Some("wit-1".to_owned()),
        ),
        ExportedRegistration::new(
            "reg-2",
            "RegB",
            "cluster",
            "T:integer",
            "cluster integer -> non empty",
            "non empty integer",
            "kernel_verified",
            Some("wit-2".to_owned()),
        ),
    ];
    let mut traces = vec![
        ClusterTraceRef::new(
            "traces/alpha.cluster.json",
            content_hash_str("{\"trace\":1}"),
            "cluster",
            "trace-v1",
        ),
        ClusterTraceRef::new(
            "traces/alpha.reduction.json",
            content_hash_str("{\"trace\":2}"),
            "reduction",
            "trace-v1",
        ),
    ];
    if reverse {
        registrations.reverse();
        traces.reverse();
    }
    RegistrationSummary::new(
        "pkg",
        "alpha",
        registrations,
        traces,
        Some("local".to_owned()),
    )
}

fn verified_artifact(reverse: bool) -> VerifiedArtifact {
    let mut exports = module_summary(reverse).exports;
    let mut expressions = vec![
        ExpressionMetadata::new(
            "expr-1",
            1,
            "term",
            "set",
            vec![MetadataField::new("hover", "x: set").unwrap()],
        ),
        ExpressionMetadata::new(
            "expr-2",
            2,
            "formula",
            "bool",
            vec![MetadataField::new("hover", "x = x").unwrap()],
        ),
    ];
    let mut obligations = vec![
        ObligationMetadata::new(
            "vc-1",
            1,
            "theorem",
            "kernel_verified",
            Some("wit-1".to_owned()),
        ),
        ObligationMetadata::new("vc-2", 2, "definition", "open", None),
    ];
    let mut diagnostics = vec![
        ProjectedDiagnostic::new(1, "W0001", "warning", "first", None),
        ProjectedDiagnostic::new(2, "W0002", "warning", "second", None),
    ];
    if reverse {
        exports.reverse();
        expressions.reverse();
        obligations.reverse();
        diagnostics.reverse();
    }
    VerifiedArtifact::new(
        "src/alpha.miz",
        content_hash_str("source"),
        Some("local-time".to_owned()),
        "alpha",
        exports,
        expressions,
        obligations,
        vec![ProofWitnessRef::from_content(
            "wit-1",
            "vc-1",
            "proof-witnesses/alpha/vc-1.json",
            "{\"proof\":true}",
            "atp_certificate",
            "kernel_verified",
            "kernel-profile-v1",
            "mizar-proof",
            vec!["A".to_owned()],
        )],
        diagnostics,
        BuildProvenance::new(
            "toolchain",
            "2024",
            content_hash_str("lock"),
            content_hash_str("config"),
            vec![DependencyArtifactHash::new("dep", content_hash_str("dep"))],
            content_hash_str("cache"),
            Some("run".to_owned()),
        ),
    )
}

fn temp_root(name: &str) -> std::path::PathBuf {
    let root = std::env::temp_dir().join(format!(
        "mizar_artifact_determinism_{name}_{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    root
}
