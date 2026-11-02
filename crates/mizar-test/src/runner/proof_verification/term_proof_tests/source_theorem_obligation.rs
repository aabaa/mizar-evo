use super::*;

#[test]
fn source_theorem_obligation_identity_exact_genuine_fields_and_slices() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_artifact::store::{CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion};
    use mizar_proof::status::{
        source_theorem_canonical_inputs as canonical,
        source_theorem_obligation_identity as identity,
    };
    use mizar_vc::{dependency_slice::DependencySliceCompleteness, vc_ir::*};
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let expected = [
        "5a352e84b26834eb3cc0e54edd522604c625a953b3ba47f927cd762001c116c9",
        "8ba4633c3ed51f7af5c2744adba8c89fe6fb65da7fb01b506305fe4afbc884db",
        "d87df55e0d47933114a69f77e96d84c426c5b30369eb451834358f1c6b4b08a1",
    ];
    // Complete Resolver46 bytes and candidate literals frozen before this implementation.
    let expected_syntax = concat!(
        "6d697a61722d7265736f6c76652f736f757263652d7468656f72656d2d73796e7461782f7631000060000000",
        "0600020065000000077468656f72656d000000000002006400000001540000000000020066000000013a0000",
        "0000002c000000010037000000040002006500000003666f7200000000003800000003000200640000000158",
        "0000000000020065000000056265696e67000000000015000000010019000000010002006500000003736574",
        "000000000002006500000005686f6c647300000000002d00000003001c00000001001d000000010002006400",
        "000001580000000000020066000000013d00000000001c00000001001d000000010002006400000001580000",
        "0000006200000004000200650000000570726f6f6600000000003d0000000300020065000000036c65740000",
        "0000003e00000003000200640000000158000000000002006500000002626500000000001500000001001900",
        "00000100020065000000037365740000000000020066000000013b0000000000540000000300020065000000",
        "047468757300000000004000000001002c00000001002d00000003001c00000001001d000000010002006400",
        "000001580000000000020066000000013d00000000001c00000001001d000000010002006400000001580000",
        "000000020066000000013b000000000002006500000003656e640000000000020066000000013b00000000",
    )
    .as_bytes()
    .as_chunks::<2>()
    .0
    .iter()
    .map(|pair| u8::from_str_radix(std::str::from_utf8(pair).unwrap(), 16).unwrap())
    .collect::<Vec<_>>();
    for (label, digest) in [
        (
            "T",
            "9135ff7f3f377b4585f23b8677c68aa131190b4d10864bf4ecc4c623cb098d7a",
        ),
        (
            "U",
            "9135ff7f3f377b4511b939a5737fde9431190b4d10864bf48c80a6b4e18f3ff3",
        ),
    ] {
        let text = original.replace("theorem T:", &format!("theorem {label}:"));
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let symbols = collected.env();
        let ast = frontend.ast.as_ref().unwrap();
        let before = (source.clone(), collected.env().clone());
        with_source_theorem_check(ast, &source, symbols, |check| {
            let check_before = format!("{check:?}");
            let owner = format!(concat!(
                "{{\"module_path\":\"owners\",\"namespace\":\"owners\",\"package_id\":\"owner-test\",",
                "\"role\":\"theorem\",\"spelling\":\"{}\",\"status\":\"unmodified\",\"visibility\":\"public\"}}\n"
            ), label).into_bytes();
            let mut syntax = expected_syntax.clone();
            assert_eq!(syntax.len(), 527);
            assert_eq!(syntax[72], b'T');
            syntax[72] = label.as_bytes()[0];
            let syntax_rows = mizar_resolve::symbols::source_theorem_syntax_payloads_from_arena(&source, symbols)?;
            assert_eq!(syntax_rows.len(), 1);
            assert_eq!(syntax_rows.get(&check.owners()[0].0.symbol), Some(&syntax));
            let anchor = format!("source-theorem-candidate-anchor-v1:{digest}");
            for ordinal in [777, 778] {
                let snapshot = snapshot_id(ordinal);
                // Request the unchanged old producer FIRST; its entire real slice is the operand.
                let old = canonical(check, snapshot, VcId::new(0))?;
                let hashes = [old.0.hash(), old.1, old.2.hash(), old.3.fingerprint().hash()].map(|hash| {
                    hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
                });
                assert_eq!(hashes[..3], expected);
                let obligation = CanonicalJson::object([
                    ("obligation_anchor", CanonicalJson::string(&anchor)),
                    ("vc_fingerprint", CanonicalJson::string(expected[0])),
                    ("canonical_goal_hash", CanonicalJson::string(expected[1])),
                    ("local_context_fingerprint", CanonicalJson::string(expected[2])),
                    ("vc_local_dependency_slice_fingerprint", CanonicalJson::string(&hashes[3])),
                ]).map_err(|error| error.to_string())?;
                let fingerprint = CanonicalHashDomain::new(
                    HashClass::Interface, "mizar-proof/source-theorem-original-obligation", SchemaVersion::new(1, 0),
                ).hash(&obligation, &[]);
                let actual = identity(&source, check, symbols, snapshot, VcId::new(0))?;
                assert_eq!(actual.0.obligation_id(), anchor);
                assert_eq!(actual.0.obligation_anchor().as_str(), anchor);
                assert_eq!(actual.0.obligation_fingerprint(), fingerprint);
                assert_eq!(actual.0.vc_fingerprint(), old.0.hash());
                assert_eq!(actual.0.local_context_fingerprint(), old.2.hash());
                assert_eq!(actual.0.dependency_slice_fingerprint(), old.3.fingerprint().hash());
                assert_eq!(actual.1, old.3);
                assert_eq!(actual.1.vc(), VcId::new(0));
                assert_eq!(actual.1.kind(), &VcKind::TheoremProofStep);
                assert_eq!(actual.1.status(), &VcStatus::Open);
                assert_eq!(actual.1.entries(), old.3.entries());
                assert_eq!(actual.1.unknowns(), old.3.unknowns());
                assert_eq!(actual.1.fingerprint(), old.3.fingerprint());
                assert!(!actual.1.entries().is_empty());
                assert!(!actual.1.unknowns().is_empty());
                assert_eq!(actual.1.completeness(), DependencySliceCompleteness::IncompleteUncacheable);
                assert!(!actual.1.is_complete());
                assert!(actual.1.requires_cache_miss());
                let raw = source_theorem_candidate_anchor_hash(&owner, &syntax, old.0, old.1, old.2)?;
                assert_eq!(raw.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>(), digest);
                for (owner_bytes, syntax_bytes) in [(&[][..], syntax.as_slice()), (owner.as_slice(), &[][..])] {
                    assert_eq!(source_theorem_candidate_anchor_hash(owner_bytes, syntax_bytes, old.0, old.1, old.2).unwrap_err(),
                        "unsupported source theorem candidate anchor");
                }
                assert_eq!(actual, identity(&source, check, symbols, snapshot, VcId::new(0))?);
                assert_eq!(old, canonical(check, snapshot, VcId::new(0))?);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        }).unwrap();
        assert_eq!((source, collected.env().clone()), before);
    }
}

#[test]
fn source_theorem_obligation_identity_neutral_allocation_and_participating_edits() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_proof::status::{
        source_theorem_canonical_inputs as canonical,
        source_theorem_obligation_identity as identity,
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let expected = "source-theorem-candidate-anchor-v1:9135ff7f3f377b4585f23b8677c68aa131190b4d10864bf4ecc4c623cb098d7a";
    let mut prior = Vec::new();
    let mut original_source = None;
    for (text, package, module, skip, neutral) in [
        (original.to_owned(), "owner-test", "owners", 0, true),
        (
            format!(
                ":: 日本語 λ\n{}\n",
                original.replace(" proof ", "\nproof\n")
            ),
            "owner-test",
            "owners",
            0,
            true,
        ),
        (original.to_owned(), "owner-test", "owners", 7, true),
        (original.to_owned(), "other-package", "owners", 0, false),
        (original.to_owned(), "owner-test", "other_module", 0, false),
        (original.to_owned(), "owner-test", "nested.owners", 0, false),
        (
            original.replace("theorem T:", "theorem U:"),
            "owner-test",
            "owners",
            0,
            false,
        ),
        (
            original.replace(
                "for X being set holds X = X",
                "for Bound being set holds Bound = Bound",
            ),
            "owner-test",
            "owners",
            0,
            false,
        ),
        (
            original.replace(
                "let X be set; thus X = X",
                "let Local be set; thus Local = Local",
            ),
            "owner-test",
            "owners",
            0,
            false,
        ),
    ] {
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let symbols = collected.env();
        let ast = frontend.ast.as_ref().unwrap();
        let before = (source.clone(), collected.env().clone(), prior.clone());
        if skip == 7 {
            assert_ne!(Some(source.source_id()), original_source);
        } else if original_source.is_none() {
            original_source = Some(source.source_id());
        }
        with_source_theorem_check(ast, &source, symbols, |check| {
            let check_before = format!("{check:?}");
            for ordinal in [777, 778] {
                let snapshot = snapshot_id(ordinal);
                let old = canonical(check, snapshot, VcId::new(0))?;
                let actual = identity(&source, check, symbols, snapshot, VcId::new(0))?;
                if neutral {
                    assert_eq!(actual.0.obligation_anchor().as_str(), expected);
                } else {
                    assert_ne!(actual.0.obligation_anchor().as_str(), expected);
                }
                assert_eq!(actual.1, old.3);
                assert_eq!(actual.0.vc_fingerprint(), old.0.hash());
                assert_eq!(actual.0.local_context_fingerprint(), old.2.hash());
                assert_eq!(
                    actual.0.dependency_slice_fingerprint(),
                    old.3.fingerprint().hash()
                );
                assert_eq!(old, canonical(check, snapshot, VcId::new(0))?);
                prior.push(actual);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, collected.env()), (&before.0, &before.1));
        assert_eq!(&prior[..before.2.len()], before.2);
    }
}

#[test]
fn source_theorem_obligation_identity_preserve_upstream_errors_and_inputs() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_proof::status::{
        source_theorem_canonical_inputs as canonical,
        source_theorem_obligation_identity as identity,
    };
    use mizar_vc::vc_ir::VcId;
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let snapshot = snapshot_id(777);
    let absent = VcId::new(usize::MAX);
    for text in [
        original.to_owned(),
        original.replacen("theorem", "lemma", 1),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!(
            "{original}\n{}",
            original.replace("theorem T:", "theorem U:")
        ),
    ] {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let symbols = collected.env();
        let ast = frontend.ast.as_ref().unwrap();
        let before = (source.clone(), collected.env().clone());
        with_source_theorem_check(ast, &source, symbols, |check| {
            let check_before = format!("{check:?}");
            if text == original {
                let old = canonical(check, snapshot, VcId::new(0))?;
                let prior = identity(&source, check, symbols, snapshot, VcId::new(0))?;
                assert_eq!(
                    identity(&source, check, symbols, snapshot, absent).unwrap_err(),
                    canonical(check, snapshot, absent).unwrap_err()
                );
                for text in [
                    original.replace("theorem T:", "theorem Foreign:"),
                    format!(
                        "{original}\n{}",
                        original.replace("theorem T:", "theorem U:")
                    ),
                    "theorem T: for X being set holds X = ;".to_owned(),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 0);
                    assert_eq!(
                        !other_frontend.diagnostics.is_empty(),
                        text.ends_with("= ;")
                    );
                    let other_before = (other_source.clone(), other.env().clone());
                    for (current, symbols) in [
                        (&other_source, other.env()),
                        (&source, other.env()),
                        (&other_source, collected.env()),
                    ] {
                        let expected = SourceVariableSemanticsChecker::check_theorem_skeletons(
                            current,
                            check.typed_ast(),
                            check.scope(),
                            symbols,
                            check.labels(),
                            check.resolved_labels(),
                            None,
                        )
                        .unwrap_err();
                        for selected in [VcId::new(0), absent] {
                            assert_eq!(
                                identity(current, check, symbols, snapshot, selected).unwrap_err(),
                                expected
                            );
                        }
                    }
                    assert_eq!((other_source, other.env().clone()), other_before);
                }
                assert_eq!(old, canonical(check, snapshot, VcId::new(0))?);
                assert_eq!(
                    prior,
                    identity(&source, check, symbols, snapshot, VcId::new(0))?
                );
            } else {
                for selected in [VcId::new(0), absent] {
                    let expected = canonical(check, snapshot, selected).unwrap_err();
                    assert_eq!(
                        identity(&source, check, symbols, snapshot, selected).unwrap_err(),
                        expected
                    );
                }
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, collected.env().clone()), before);
    }
}

#[test]
fn source_theorem_obligation_identity_schemas_match_original_producers() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_artifact::store::{
        CanonicalHashDomain, CanonicalJson, HashClass, SchemaVersion, canonical_json_bytes,
    };
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_core::{
        core_ir::{CoreProofStatus, ObligationSeedStatus},
        elaborator::{
            lower_source_theorem_skeletons as lower,
            source_theorem_formula_payloads as formulas_for,
        },
    };
    use mizar_proof::status::{
        source_theorem_obligation_identity as legacy,
        source_theorem_obligation_identity_with_schemas as identity,
    };
    use mizar_resolve::symbols::source_theorem_syntax_payloads_from_arena as syntax_for;
    use mizar_vc::{
        dependency_slice::{
            DependencySliceCompleteness, DependencySliceInput, try_compute_dependency_slices,
        },
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{
            GenerationSchemaVersion, VcKind, VcSchemaVersion, VcStatus,
            source_theorem_candidate_anchor_hash,
        },
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let mut first_source = None;
    let mut prior = Vec::new();
    for (case, text) in [
        original.to_owned(),
        original.replace(" X", " Local"),
        format!(":: 日本語 λ\n{original}\n"),
        original.replace("T:", "Renamed:"),
        original.to_owned(),
        original.to_owned(),
        original.to_owned(),
    ]
    .into_iter()
    .enumerate()
    {
        let (package, module, skip) = match case {
            4 => ("other-package", "nested.owners", 0),
            6 => ("owner-test", "owners", 7),
            _ => ("owner-test", "owners", 0),
        };
        let (frontend, source, collected) = inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let env = collected.env();
        assert_eq!(ast.source_id, source.source_id());
        if case == 0 {
            first_source = Some(source.source_id());
        }
        if skip == 7 {
            assert_ne!(Some(source.source_id()), first_source);
        }
        let source_before = (source.clone(), env.clone(), prior.clone());
        with_source_theorem_check(ast, &source, env, |check| {
            let check_before = format!("{check:?}");
            let stamps = if case == 0 {
                vec![
                    ("source-theorem-local-v1", "vc-v1"),
                    ("λ\n\0", ""),
                    ("", "vc\t\0"),
                ]
            } else {
                vec![("source-theorem-local-v1", "vc-v1")]
            };
            for (generation, schema) in stamps {
                let generation = GenerationSchemaVersion::new(generation);
                let schema = VcSchemaVersion::new(schema);
                let snapshot = snapshot_id(if case == 5 { 778 } else { 777 });
                let fresh = SourceVariableSemanticsChecker::check_theorem_skeletons(
                    &source,
                    check.typed_ast(),
                    check.scope(),
                    env,
                    check.labels(),
                    check.resolved_labels(),
                    None,
                )?;
                assert_eq!(fresh.owners(), check.owners());
                let core = lower(&fresh, None)?;
                let roots = core
                    .obligation_seeds()
                    .iter()
                    .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
                    .collect::<Vec<_>>();
                let formulas = formulas_for(&fresh, &core, &roots)?;
                let vcs = generate(&core, snapshot, &generation, &schema)?;
                let [vc] = vcs.vcs() else { panic!() };
                let id = vc.id;
                let canonical = vcs.source_theorem_canonical_components(id, &core, &formulas)?;
                let slices = try_compute_dependency_slices(DependencySliceInput {
                    vc_set: &vcs,
                    discharge_output: None,
                })
                .map_err(|error| error.to_string())?;
                let slice = slices.slice_for(id).unwrap().clone();
                let syntax = syntax_for(&source, env)?;
                let [(owner, visibility)] = fresh.owners() else {
                    panic!()
                };
                assert_eq!(*visibility, "public");
                let spelling = if case == 3 { "Renamed" } else { "T" };
                assert_eq!(owner.spelling, spelling);
                let owner_json = CanonicalJson::object([
                    ("package_id", CanonicalJson::string(package)),
                    ("module_path", CanonicalJson::string(module)),
                    ("namespace", CanonicalJson::string(module)),
                    ("spelling", CanonicalJson::string(spelling)),
                    ("role", CanonicalJson::string("theorem")),
                    ("status", CanonicalJson::string("unmodified")),
                    ("visibility", CanonicalJson::string("public")),
                ])
                .map_err(|error| error.to_string())?;
                let anchor_hash = source_theorem_candidate_anchor_hash(
                    &canonical_json_bytes(&owner_json),
                    syntax.get(&owner.symbol).unwrap(),
                    canonical.0,
                    canonical.1,
                    canonical.2,
                )?;
                let hashes = [
                    anchor_hash,
                    canonical.0.hash(),
                    canonical.1,
                    canonical.2.hash(),
                    slice.fingerprint().hash(),
                ]
                .map(|hash| {
                    hash.as_bytes()
                        .iter()
                        .map(|byte| format!("{byte:02x}"))
                        .collect::<String>()
                });
                let anchor = format!("source-theorem-candidate-anchor-v1:{}", hashes[0]);
                let obligation = CanonicalJson::object([
                    ("obligation_anchor", CanonicalJson::string(&anchor)),
                    ("vc_fingerprint", CanonicalJson::string(&hashes[1])),
                    ("canonical_goal_hash", CanonicalJson::string(&hashes[2])),
                    (
                        "local_context_fingerprint",
                        CanonicalJson::string(&hashes[3]),
                    ),
                    (
                        "vc_local_dependency_slice_fingerprint",
                        CanonicalJson::string(&hashes[4]),
                    ),
                ])
                .map_err(|error| error.to_string())?;
                let fingerprint = CanonicalHashDomain::new(
                    HashClass::Interface,
                    "mizar-proof/source-theorem-original-obligation",
                    SchemaVersion::new(1, 0),
                )
                .hash(&obligation, &[]);
                let legacy_result = if generation.as_str() == "source-theorem-local-v1"
                    && schema.as_str() == "vc-v1"
                {
                    Some(legacy(&source, check, env, snapshot, id)?)
                } else {
                    None
                };
                assert_eq!(core.source_id(), ast.source_id);
                assert_eq!(core.module_id(), source.module());
                assert_eq!(vcs.source(), ast.source_id);
                assert_eq!(vcs.snapshot(), snapshot);
                assert_eq!(vcs.schema_version(), &schema);
                assert_eq!(vc.anchor.generation_schema_version, generation);
                assert_eq!(vc.status, VcStatus::Open);
                assert!(
                    core.proofs()
                        .iter()
                        .all(|(_, proof)| proof.status == CoreProofStatus::PendingAutomaticProof)
                );
                assert!(
                    core.obligation_seeds()
                        .iter()
                        .all(|(_, seed)| seed.status == ObligationSeedStatus::Active)
                );
                let before = (
                    core.clone(),
                    vcs.clone(),
                    roots.clone(),
                    formulas.clone(),
                    slice.clone(),
                    generation.clone(),
                    schema.clone(),
                    legacy_result.clone(),
                );
                let actual = identity(&source, check, env, snapshot, id, (&generation, &schema))?;
                assert_eq!(actual.0.obligation_id(), anchor);
                assert_eq!(actual.0.obligation_anchor().as_str(), anchor);
                assert_eq!(actual.0.obligation_fingerprint(), fingerprint);
                assert_eq!(actual.0.vc_fingerprint(), canonical.0.hash());
                assert_eq!(actual.0.local_context_fingerprint(), canonical.2.hash());
                assert_eq!(
                    actual.0.dependency_slice_fingerprint(),
                    slice.fingerprint().hash()
                );
                assert_eq!(actual.1, slice);
                assert_eq!(actual.1.vc(), id);
                assert_eq!(actual.1.kind(), &VcKind::TheoremProofStep);
                assert_eq!(actual.1.status(), &VcStatus::Open);
                assert_eq!(actual.1.entries(), slice.entries());
                assert_eq!(actual.1.unknowns(), slice.unknowns());
                assert!(!actual.1.entries().is_empty() && !actual.1.unknowns().is_empty());
                assert_eq!(actual.1.fingerprint(), slice.fingerprint());
                assert_eq!(
                    actual.1.completeness(),
                    DependencySliceCompleteness::IncompleteUncacheable
                );
                assert!(!actual.1.is_complete() && actual.1.requires_cache_miss());
                if let Some(expected) = &legacy_result {
                    assert_eq!(&actual, expected);
                }
                assert_eq!(
                    (
                        core,
                        vcs,
                        roots,
                        formulas,
                        slice,
                        generation,
                        schema,
                        legacy_result
                    ),
                    before
                );
                prior.push(actual);
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((&source, env), (&source_before.0, &source_before.1));
        assert_eq!(&prior[..source_before.2.len()], source_before.2);
    }
}

#[test]
fn source_theorem_obligation_identity_schemas_preserve_errors() {
    use super::super::super::tests::source_registration_owner_inputs as inputs;
    use mizar_checker::type_checker::SourceVariableSemanticsChecker;
    use mizar_core::elaborator::{
        lower_source_theorem_skeletons as lower, source_theorem_formula_payloads as formulas_for,
    };
    use mizar_proof::status::source_theorem_obligation_identity_with_schemas as identity;
    use mizar_resolve::symbols::source_theorem_syntax_payloads_from_arena as syntax_for;
    use mizar_vc::{
        generator::generate_source_theorem_vcs as generate,
        vc_ir::{GenerationSchemaVersion, VcId, VcSchemaVersion},
    };
    let original = "theorem T: for X being set holds X = X proof let X be set; thus X = X; end;";
    let generation = GenerationSchemaVersion::new("λ\n\0");
    let schema = VcSchemaVersion::new("");
    let snapshot = snapshot_id(777);
    let config_before = (generation.clone(), schema.clone());
    for (case, text) in [
        original.to_owned(),
        "open theorem O: for X being set holds X = X;".to_owned(),
        format!("{original}\n{}", original.replace("T:", "U:")),
    ]
    .into_iter()
    .enumerate()
    {
        let (frontend, source, collected) = inputs(&text, "owner-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty() && collected.diagnostics().is_empty());
        let env = collected.env();
        let before = (source.clone(), env.clone());
        with_source_theorem_check(frontend.ast.as_ref().unwrap(), &source, env, |check| {
            let check_before = format!("{check:?}");
            let fresh = SourceVariableSemanticsChecker::check_theorem_skeletons(
                &source,
                check.typed_ast(),
                check.scope(),
                env,
                check.labels(),
                check.resolved_labels(),
                None,
            )?;
            if case == 0 {
                let core = lower(&fresh, None)?;
                let roots = core
                    .obligation_seeds()
                    .iter()
                    .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
                    .collect::<Vec<_>>();
                let formulas = formulas_for(&fresh, &core, &roots)?;
                let vcs = generate(&core, snapshot, &generation, &schema)?;
                let expected = vcs
                    .source_theorem_canonical_components(VcId::new(usize::MAX), &core, &formulas)
                    .unwrap_err();
                assert_eq!(
                    identity(
                        &source,
                        check,
                        env,
                        snapshot,
                        VcId::new(usize::MAX),
                        (&generation, &schema)
                    )
                    .unwrap_err(),
                    expected
                );
                for (text, recovered) in [
                    (original.replace("T:", "Foreign:"), false),
                    ("theorem T: for X being set holds X = ;".to_owned(), true),
                ] {
                    let (other_frontend, other_source, other) =
                        inputs(&text, "owner-test", "owners", 7);
                    assert_eq!(!other_frontend.diagnostics.is_empty(), recovered);
                    assert_ne!(source.source_id(), other_source.source_id());
                    let foreign_before = (other_source.clone(), other.env().clone());
                    let (current, symbols) = if recovered {
                        (&other_source, other.env())
                    } else {
                        (&source, other.env())
                    };
                    let expected = SourceVariableSemanticsChecker::check_theorem_skeletons(
                        current,
                        check.typed_ast(),
                        check.scope(),
                        symbols,
                        check.labels(),
                        check.resolved_labels(),
                        None,
                    )
                    .unwrap_err();
                    assert_eq!(
                        identity(
                            current,
                            check,
                            symbols,
                            snapshot,
                            VcId::new(0),
                            (&generation, &schema)
                        )
                        .unwrap_err(),
                        expected
                    );
                    assert_eq!((other_source, other.env().clone()), foreign_before);
                }
            } else if case == 1 {
                let expected = lower(&fresh, None).unwrap_err();
                assert_eq!(
                    identity(
                        &source,
                        check,
                        env,
                        snapshot,
                        VcId::new(0),
                        (&generation, &schema)
                    )
                    .unwrap_err(),
                    expected
                );
            } else {
                let core = lower(&fresh, None)?;
                let roots = core
                    .obligation_seeds()
                    .iter()
                    .flat_map(|(_, seed)| seed.context.iter().copied().chain(seed.goal))
                    .collect::<Vec<_>>();
                let expected = formulas_for(&fresh, &core, &roots).unwrap_err();
                assert_eq!(syntax_for(&source, env)?.len(), 2);
                assert_ne!(expected, "theorem.proof.invalid_source_identity");
                assert_eq!(
                    identity(
                        &source,
                        check,
                        env,
                        snapshot,
                        VcId::new(0),
                        (&generation, &schema)
                    )
                    .unwrap_err(),
                    expected
                );
            }
            assert_eq!(format!("{check:?}"), check_before);
            Ok(())
        })
        .unwrap();
        assert_eq!((source, env.clone()), before);
    }
    assert_eq!((generation, schema), config_before);
}
