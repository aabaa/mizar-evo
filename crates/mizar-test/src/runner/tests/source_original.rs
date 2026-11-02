use super::*;

#[test]
fn source_original_obligation_identity_exact_profile_and_full_original_slices() {
    use mizar_artifact::store::{CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion};
    use mizar_proof::status::{
        ProofObligationIdentity, source_registration_candidate_anchor as anchor,
        source_registration_obligation_identity as identity,
    };
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, DependencyUnknownFamily,
            try_compute_dependency_slices,
        },
        vc_ir::VcStatus,
    };
    let snapshot = super::super::shared::snapshot_id(0);
    for (count, case) in [
        (2, step5c3_registration_case()),
        (6, step5c13_registration_case()),
    ] {
        let text = std::fs::read_to_string(case.source_path).unwrap();
        let (source, nodes, collected, vcs) =
            source_dependency_inputs(&text, "vc_fingerprints", 0, snapshot);
        let before = (source.clone(), nodes.clone(), collected.env().clone());
        let original = try_compute_dependency_slices(DependencySliceInput {
            vc_set: &vcs,
            discharge_output: None,
        })
        .unwrap();
        assert_eq!(vcs.vcs().len(), count);
        for vc in vcs.vcs() {
            let (expected_anchor, full, goal, context) =
                anchor(&source, &nodes, collected.env(), snapshot, vc.id).unwrap();
            let expected_slice = original.slice_for(vc.id).unwrap();
            let hex = |hash: mizar_session::Hash| {
                hash.as_bytes()
                    .iter()
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>()
            };
            // Literal profile/object is fixed independently before calling the new facade.
            let object = CanonicalJson::object([
                (
                    "obligation_anchor",
                    CanonicalJson::string(expected_anchor.as_str()),
                ),
                ("vc_fingerprint", CanonicalJson::string(hex(full.hash()))),
                ("canonical_goal_hash", CanonicalJson::string(hex(goal))),
                (
                    "local_context_fingerprint",
                    CanonicalJson::string(hex(context)),
                ),
                (
                    "vc_local_dependency_slice_fingerprint",
                    CanonicalJson::string(hex(expected_slice.fingerprint().hash())),
                ),
            ])
            .unwrap();
            let fingerprint = CanonicalHashDomain::new(
                HashClass::Interface,
                "mizar-proof/source-registration-original-obligation",
                SchemaVersion::new(1, 0),
            )
            .hash(&object, &[]);
            let expected = ProofObligationIdentity::new(
                expected_anchor.as_str(),
                expected_anchor.clone(),
                fingerprint,
                full.hash(),
                context,
                expected_slice.fingerprint().hash(),
            )
            .unwrap();
            let actual = identity(&source, &nodes, collected.env(), snapshot, vc.id).unwrap();
            assert_eq!(actual, (expected, expected_slice.clone()));
            assert_eq!(actual.1.status(), &VcStatus::Open);
            assert_eq!(
                actual.1.completeness(),
                DependencySliceCompleteness::IncompleteUncacheable
            );
            assert!(actual.1.requires_cache_miss());
            for family in [
                DependencyUnknownFamily::UpstreamPayload,
                DependencyUnknownFamily::Anchor,
            ] {
                assert!(
                    actual
                        .1
                        .unknowns()
                        .iter()
                        .any(|unknown| unknown.family() == family)
                );
            }
        }
        assert_eq!((source, nodes, collected.env().clone()), before);
    }
}

#[test]
fn source_original_obligation_identity_edits_snapshots_and_current_handles() {
    use mizar_proof::status::{
        source_registration_candidate_anchor as anchor,
        source_registration_obligation_identity as identity,
    };
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        vc_ir::VcStatus,
    };
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let ra = text.find("  cluster OAExists:").unwrap();
    let rb = text.find("  cluster OBExists:").unwrap();
    let rab = text.find("  cluster OABExists:").unwrap();
    let end = text[rab..].find("end;\nend;").unwrap() + rab + "end;\n".len();
    let reordered = format!(
        "{}{}{}{}{}",
        &text[..ra],
        &text[rab..end],
        &text[ra..rb],
        &text[rb..rab],
        &text[end..]
    );
    assert_ne!(reordered, text);
    let mut baseline = std::collections::BTreeMap::new();
    for (index, edited, module, skip) in [
        (0, text.clone(), "vc_fingerprints", 0),
        (
            1,
            format!("\n:: neutral trivia\n{text}"),
            "vc_fingerprints",
            0,
        ),
        (2, text.replace('X', "Subject"), "vc_fingerprints", 7),
        (3, reordered, "vc_fingerprints", 0),
        (
            4,
            text.replace("OAExists", "RenamedExists"),
            "vc_fingerprints",
            0,
        ),
        (5, text.clone(), "other_module", 0),
        (
            6,
            text.replacen("means X = X", "means not X = X", 1),
            "vc_fingerprints",
            0,
        ),
    ] {
        let (_, _, _, owners) = source_registration_vc_fingerprint_inputs(&edited, module, skip);
        for snapshot in [
            super::super::shared::snapshot_id(0),
            super::super::shared::snapshot_id(41),
        ] {
            let (source, nodes, collected, vcs) =
                source_dependency_inputs(&edited, module, skip, snapshot);
            let before = (source.clone(), nodes.clone(), collected.env().clone());
            let slices = try_compute_dependency_slices(DependencySliceInput {
                vc_set: &vcs,
                discharge_output: None,
            })
            .unwrap();
            for (id, owner) in &owners {
                let components = anchor(&source, &nodes, collected.env(), snapshot, *id).unwrap();
                let (actual, slice) =
                    identity(&source, &nodes, collected.env(), snapshot, *id).unwrap();
                assert_eq!(actual.obligation_id(), components.0.as_str());
                assert_eq!(actual.obligation_anchor(), &components.0);
                assert_eq!(actual.vc_fingerprint(), components.1.hash());
                assert_eq!(actual.local_context_fingerprint(), components.3);
                assert_eq!(
                    actual.dependency_slice_fingerprint(),
                    slices.slice_for(*id).unwrap().fingerprint().hash()
                );
                assert_eq!(&slice, slices.slice_for(*id).unwrap());
                assert_eq!(slice.status(), &VcStatus::Open);
                assert_eq!(
                    slice.completeness(),
                    DependencySliceCompleteness::IncompleteUncacheable
                );
                assert!(slice.requires_cache_miss());
                assert!(!slice.unknowns().is_empty());
                let name = owner.replace("RenamedExists:", "OAExists:");
                if index == 0 && snapshot == super::super::shared::snapshot_id(0) {
                    baseline.insert(name, (actual, components, *id));
                    continue;
                }
                let (old, old_components, old_id) = &baseline[&name];
                let changed = match index {
                    4 => name.starts_with("OAExists:"),
                    5 => true,
                    6 => name == "OAExists:1" || name == "OABExists:1",
                    _ => false,
                };
                assert_eq!(
                    components.0 != old_components.0,
                    changed,
                    "anchor {index}: {name}"
                );
                assert_eq!(
                    components.1 != old_components.1,
                    changed,
                    "VC {index}: {name}"
                );
                assert_eq!(
                    components.3 != old_components.3,
                    changed,
                    "context {index}: {name}"
                );
                if changed {
                    assert_ne!(
                        actual.obligation_fingerprint(),
                        old.obligation_fingerprint()
                    );
                }
                if index < 4 {
                    assert_eq!(components, *old_components);
                    assert_eq!(
                        actual.obligation_fingerprint() == old.obligation_fingerprint(),
                        actual.dependency_slice_fingerprint() == old.dependency_slice_fingerprint()
                    );
                }
                if index == 3 && name == "OABExists:1" {
                    assert_ne!(id, old_id);
                    // The old numeric handle still selects the current graph, never its old owner.
                    let current =
                        identity(&source, &nodes, collected.env(), snapshot, *old_id).unwrap();
                    let selected =
                        anchor(&source, &nodes, collected.env(), snapshot, *old_id).unwrap();
                    assert_eq!(current.0.obligation_anchor(), &selected.0);
                    assert_ne!(current.0.obligation_anchor(), old.obligation_anchor());
                    assert_eq!(&current.1, slices.slice_for(*old_id).unwrap());
                }
            }
            assert_eq!((source, nodes, collected.env().clone()), before);
        }
    }
}

#[test]
fn source_original_obligation_identity_real_errors_precede_absent_handle() {
    use mizar_checker::registration_resolution::check_source_existential_registration_proof as check;
    use mizar_proof::status::{
        source_registration_candidate_anchor as anchor,
        source_registration_dependency_slice as slice,
        source_registration_obligation_identity as identity,
    };
    use mizar_vc::vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion};
    let text = std::fs::read_to_string(step5c13_registration_case().source_path).unwrap();
    let snapshot = super::super::shared::snapshot_id(0);
    let (source, nodes, collected, vcs) =
        source_dependency_inputs(&text, "vc_fingerprints", 0, snapshot);
    let before = (source.clone(), nodes.clone(), collected.env().clone());
    let selected = vcs.vcs()[0].id;
    let absent = VcId::new(vcs.vcs().len());
    let invalid = anchor(&source, &nodes, collected.env(), snapshot, absent).unwrap_err();
    assert_eq!(invalid, "registration.proof.invalid_source_anchor");
    assert_ne!(
        invalid,
        slice(&source, &nodes, collected.env(), snapshot, absent).unwrap_err()
    );
    assert_eq!(
        identity(&source, &nodes, collected.env(), snapshot, absent).unwrap_err(),
        invalid
    );
    let (foreign_source, foreign_nodes, foreign_env, _) = source_registration_formula_inputs(
        &text.replace("oamarked", "foreignmarked"),
        "vc-fingerprint-test",
        "vc_fingerprints",
        0,
    );
    let foreign_before = (
        foreign_source.clone(),
        foreign_nodes.clone(),
        foreign_env.env().clone(),
    );
    for (s, n, e) in [
        (&foreign_source, &nodes, collected.env()),
        (&source, &foreign_nodes, collected.env()),
        (&source, &nodes, foreign_env.env()),
    ] {
        let upstream = check(s, n, e).unwrap_err();
        assert_ne!(upstream, invalid);
        for id in [selected, absent] {
            assert_eq!(identity(s, n, e, snapshot, id).unwrap_err(), upstream);
        }
    }
    assert_eq!(
        (foreign_source, foreign_nodes, foreign_env.env().clone()),
        foreign_before
    );
    let ra = text.find("  cluster OAExists:").unwrap();
    let rb = text.find("  cluster OBExists:").unwrap();
    let rab = text.find("  cluster OABExists:").unwrap();
    let swapped = format!(
        "{}{}{}{}",
        &text[..ra],
        &text[rb..rab],
        &text[ra..rb],
        &text[rab..]
    );
    let malformed = text.replacen("    take the set;\n", "", 1);
    for (index, edited) in [malformed, swapped].into_iter().enumerate() {
        assert_ne!(edited, text);
        let (s, n, e, _) = source_registration_formula_inputs(
            &edited,
            "vc-fingerprint-test",
            "vc_fingerprints",
            0,
        );
        let inputs_before = (s.clone(), n.clone(), e.env().clone());
        let upstream = if index == 0 {
            check(&s, &n, e.env()).unwrap_err()
        } else {
            let checked = check(&s, &n, e.env()).unwrap();
            let core =
                mizar_core::elaborator::lower_source_existential_registration(&checked).unwrap();
            mizar_vc::generator::generate_source_existential_registration(
                &core,
                snapshot,
                &GenerationSchemaVersion::new("source-existential-registration-v1"),
                &VcSchemaVersion::new("vc-v1"),
            )
            .unwrap_err()
        };
        assert_ne!(upstream, invalid);
        for id in [selected, absent] {
            assert_eq!(
                identity(&s, &n, e.env(), snapshot, id).unwrap_err(),
                upstream
            );
        }
        assert_eq!((s, n, e.env().clone()), inputs_before);
    }
    assert_eq!((source, nodes, collected.env().clone()), before);
}
