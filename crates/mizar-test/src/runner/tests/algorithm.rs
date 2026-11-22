use super::*;
use mizar_vc::dependency_slice::{
    source_unary_child_call_canonical_inputs as canonical_unary_child,
    source_unary_caller_e4_canonical_inputs as canonical_caller_e4,
    source_unary_caller_e1_inhabitation_canonical_inputs as canonical_caller_e1,
    source_unary_caller_e1_typing_canonical_inputs as canonical_caller_typing,
};

#[test]
fn unary_call_identities_match_independent_canonical_and_raw_slice_oracles() {
    use mizar_artifact::store::{
        CanonicalHashDomain, CanonicalJson as J, HashClass, SchemaVersion,
    };
    use mizar_checker::type_checker::check_source_unary_child_call;
    use mizar_core::elaborator::lower_source_unary_child_call;
    use mizar_proof::{policy::VerifierPolicy, status::{NativeUnaryChildCallExistential, SourceUnaryCallerE4Association, SourceUnaryCallerE1InhabitationAssociation}};
    use mizar_session::{Hash, bridge_fingerprint_hash, lower_hex};
    use mizar_vc::{
        dependency_slice::{
            DependencyEntryClass as E, DependencySliceCompleteness as C,
            DependencyUnknownFamily as U,
        },
        vc_ir::{VcId, VcKind, VcStatus},
    };
    let text = "definition let a be object; algorithm child(a) -> object do return a; end; end; definition let b be object; algorithm caller(b) -> object do return child(b); end; end;";
    let (source, nodes, collected, _) =
        source_registration_formula_inputs(text, "unary-child", "identity", 0);
    let typed = super::super::source_registration_receiver(&source, nodes).unwrap();
    let symbols = collected.env();
    let (core, request) = lower_source_unary_child_call(
        &check_source_unary_child_call(&source, symbols, &typed).unwrap(),
    ).unwrap();
    let before = (source.clone(), typed.clone(), symbols.clone(), core.clone(), request.clone());
    let mut child_identity = None;
    let mut caller_e4_identity = None;
    let mut caller_e1_identity = None;
    for profile in ["source-unary-child", "source-unary-caller-e4", "source-unary-caller-e1-inhabitation", "source-unary-caller-e1-typing"] {
        let caller_profile = profile == "source-unary-caller-e4";
        let inhabitation_profile = profile == "source-unary-caller-e1-inhabitation";
        let typing_profile = profile == "source-unary-caller-e1-typing";
        // Each oracle is fixed before its corresponding identity API is called. Owner data comes
        // directly from the source environment; all semantic relationships are literal.
        let owners = ["child", "caller"].map(|spelling| {
            let entry = symbols.symbols().iter().find(|entry| entry.primary_spelling() == spelling).unwrap();
            ["unary-child", "identity", entry.namespace().as_str(), spelling,
                "algorithm", "public", "false"]
        });
        let frame = |component: &str, fields: &[&[u8]]| {
            let mut bytes = format!("mizar-vc/{profile}/{component}/v1\0").into_bytes();
            for field in fields {
                bytes.extend_from_slice(&u32::try_from(field.len()).unwrap().to_be_bytes());
                bytes.extend_from_slice(field);
            }
            bytes
        };
        let mut definition_fields = owners.iter().flatten().map(|field| field.as_bytes()).collect::<Vec<_>>();
        definition_fields.push(b"callee.parameter=p0;callee.result=r0;callee.return=p0;caller.parameter=p1;caller.result=r1;caller.return=Apply(callee,p1)");
        let definition = frame("definition", &definition_fields);
        let goal = frame("goal", &[
            &definition,
            if typing_profile { b"forall caller-result.(returns(caller,caller-parameter,caller-result)->object(caller-result))" } else if inhabitation_profile { b"exists caller-result.object(caller-result)" } else if caller_profile { b"exists caller-result.returns(caller,caller-parameter,caller-result)" } else { b"exists child-result.returns(callee,caller-parameter,child-result)" },
            b"caller-parameter",
        ]);
        let context = frame("context", &[b"object(caller-parameter)", if caller_profile || inhabitation_profile || typing_profile { b"caller-result:result" } else { b"child-result:result" }]);
        let expected_goal = bridge_fingerprint_hash(&format!("mizar-vc/{profile}/goal/v1"), &goal);
        let expected_context = bridge_fingerprint_hash(&format!("mizar-vc/{profile}/context/v1"), &context);
        let vc = frame("vc", &[if typing_profile { b"unary-caller-e1-result-typing" } else if inhabitation_profile { b"unary-caller-e1-object-inhabitation" } else if caller_profile { b"unary-caller-e4-existential" } else { b"unary-child-existential" }, expected_goal.as_bytes(), expected_context.as_bytes()]);
        let expected_vc = bridge_fingerprint_hash(&format!("mizar-vc/{profile}/vc/v1"), &vc);
        let entries = [
            (E::LocalContext, format!("{profile}:context"), lower_hex(&context)),
            (E::GeneratedFormula, format!("{profile}:goal"), lower_hex(&goal)),
            (E::Definition, format!("{profile}:definition"), lower_hex(&definition)),
        ];
        let unknowns = [
            (U::DischargeEvidence, format!("{profile}:evidence"), if typing_profile { "raw source unary caller E1 typing identity omits policy-approved evidence" } else if inhabitation_profile { "raw source unary caller E1 inhabitation identity omits policy-approved evidence" } else if caller_profile { "raw source unary caller E4 identity omits policy-approved evidence" } else { "raw source unary child identity omits native evidence" }),
            (U::UpstreamPayload, format!("{profile}:footprint"), if typing_profile { "source unary caller E1 typing dependency footprint is not collected" } else if inhabitation_profile { "source unary caller E1 inhabitation dependency footprint is not collected" } else if caller_profile { "source unary caller E4 dependency footprint is not collected" } else { "source unary child dependency footprint is not collected" }),
        ];
        let kind = if typing_profile { VcKind::AlgorithmE1Typing } else if inhabitation_profile { VcKind::AlgorithmE1ResultInhabitation } else { VcKind::AlgorithmE4NormalReturn };
        let mut slice_payload = format!("dependency-slice-fingerprint-v1\nschema-version: mizar-vc-dependency-slice-v1\nkind: {kind:?}\nstatus: Open\ncompleteness: IncompleteUncacheable\n[entries]\n");
        for (class, _, payload) in &entries {
            slice_payload.push_str(&format!("{class:?}\t{payload}\n"));
        }
        slice_payload.push_str("[unknowns]\n");
        for (family, _, reason) in &unknowns {
            slice_payload.push_str(&format!("{family:?}\t{reason}\n"));
        }
        // Existing raw-slice fingerprint v1, independently applied to the literal
        // expected entries and unknowns rather than any returned slice fields.
        let mut lanes = [0x6d_69_7a_61_72_2d_76_63_u64, 0x64_65_70_2d_73_6c_69_63,
            0x65_2d_66_69_6e_67_65_72, 0x70_72_69_6e_74_2d_76_31];
        for (index, byte) in slice_payload.bytes().enumerate() {
            let lane = index % 4;
            lanes[lane] ^= u64::from(byte).wrapping_add(0x9e37_79b9_7f4a_7c15)
                .wrapping_add((index as u64).rotate_left(lane as u32 + 1));
            lanes[lane] = lanes[lane].rotate_left(11 + lane as u32).wrapping_mul(0x1000_0000_01b3);
        }
        lanes[0] ^= slice_payload.len() as u64;
        lanes[1] ^= (slice_payload.len() as u64).rotate_left(17);
        lanes[2] ^= lanes[0].rotate_left(7);
        lanes[3] ^= lanes[1].rotate_left(13);
        let mut bytes = [0; Hash::BYTE_LEN];
        for (chunk, lane) in bytes.as_chunks_mut::<8>().0.iter_mut().zip(lanes) {
            chunk.copy_from_slice(&lane.to_be_bytes());
        }
        let expected_slice = Hash::from_bytes(bytes);
        let anchor = format!("{profile}-candidate-anchor-v1:{}", lower_hex(expected_vc.as_bytes()));
        let obligation = J::object([
            ("obligation_anchor", J::string(&anchor)),
            ("vc_fingerprint", J::string(lower_hex(expected_vc.as_bytes()))),
            ("canonical_goal_hash", J::string(lower_hex(expected_goal.as_bytes()))),
            ("local_context_fingerprint", J::string(lower_hex(expected_context.as_bytes()))),
            ("vc_local_dependency_slice_fingerprint", J::string(lower_hex(expected_slice.as_bytes()))),
        ]).unwrap();
        let expected_obligation = CanonicalHashDomain::new(
            HashClass::Interface, format!("mizar-proof/{profile}/obligation"), SchemaVersion::new(1, 0),
        ).hash(&obligation, &[]);
        let policy = VerifierPolicy::default();
        let descriptor_before = mizar_vc::generator::generate_source_unary_child_call_existential_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        let (_, retained) = mizar_vc::generator::check_source_unary_child_call_existential_result(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        let retained_before = (retained.target(), retained.witness(), retained.context().to_vec(), retained.checker_schema_version());
        let caller_e4 = mizar_vc::generator::generate_source_unary_child_call_caller_e4_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert_ne!(caller_e4.1[0], descriptor_before.1[0]);
        let native = NativeUnaryChildCallExistential::from_source(
            &source, &typed, symbols, &core, &request, &policy,
        ).unwrap();
        let caller = SourceUnaryCallerE4Association::from_source(&source, &typed, symbols, &core, &request).unwrap();
        let caller_before = (caller.request().clone(), caller.descriptor().clone(), caller.result().target(), caller.result().witness(), caller.result().context().to_vec(), caller.result().checker_schema_version());
        let inhabitation = SourceUnaryCallerE1InhabitationAssociation::from_source(&source, &typed, symbols, &core, &request).unwrap();
        let e1_before = (inhabitation.descriptor().clone(), inhabitation.result().binder(), inhabitation.result().context().to_vec(), inhabitation.result().checker_schema_version());
        let caller_typing = mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert_eq!(caller_typing.0, caller_e4.0);
        assert_eq!(caller_typing.1[0], caller_e4.1[0]);
        assert_eq!(mizar_vc::generator::check_source_unary_child_call_caller_e1_typing_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap(), caller_typing);
        assert!(mizar_kernel::checker::check_unary_caller_return_typing(
            &core, &request, retained.target().1, caller.result().target(), inhabitation.result().binder(),
            ((inhabitation.result().binder(), None), "object"), retained.context(),
        ));
        let typing = mizar_proof::status::SourceUnaryCallerE1TypingAssociation::from_source(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert_eq!(typing.descriptor(), &caller_typing);
        assert_eq!(caller_typing.1.len(), 4);
        let raw = if typing_profile { canonical_caller_typing(&source, &typed, symbols, &core, &request) } else if inhabitation_profile { canonical_caller_e1(&source, &typed, symbols, &core, &request) } else if caller_profile { canonical_caller_e4(&source, &typed, symbols, &core, &request) } else { canonical_unary_child(&source, &typed, symbols, &core, &request) }.unwrap();
        assert_eq!((raw.0, raw.1, raw.2), (expected_vc, expected_goal, expected_context));
        let (identity, slice) = if typing_profile {
            typing.obligation_identity(&source, &typed, symbols, &core, &request)
        } else if inhabitation_profile {
            inhabitation.obligation_identity(&source, &typed, symbols, &core, &request)
        } else if caller_profile {
            caller.obligation_identity(&source, &typed, symbols, &core, &request)
        } else {
            native.obligation_identity(&source, &typed, symbols, &core, &request, &policy)
        }.unwrap().unwrap();
        if typing_profile {
            assert_ne!(Some(&identity), child_identity.as_ref());
            assert_ne!(Some(&identity), caller_e4_identity.as_ref());
            assert_ne!(Some(&identity), caller_e1_identity.as_ref());
        } else if inhabitation_profile {
            caller_e1_identity = Some(identity.clone());
            let native_e1 = mizar_proof::status::NativeUnaryCallerE1Inhabitation::from_source(
                &source, &typed, symbols, &core, &request, &policy,
            ).unwrap();
            for _ in 0..2 {
                assert_eq!(native_e1.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), (identity.clone(), slice.clone()));
            }
            assert_ne!(Some(&identity), child_identity.as_ref());
            assert_ne!(Some(&identity), caller_e4_identity.as_ref());
        } else if caller_profile {
            caller_e4_identity = Some(identity.clone());
            assert_ne!(Some(&identity), child_identity.as_ref());
            let native_caller = mizar_proof::status::NativeUnaryCallerE4Existential::from_source(
                &source, &typed, symbols, &core, &request, &policy,
            ).unwrap();
            for _ in 0..2 {
                assert_eq!(native_caller.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), (identity.clone(), slice.clone()));
            }
            let (consumed, fingerprint) = mizar_proof::status::consume_source_unary_caller_e4_existential(
                &source, &typed, symbols, &core, &request, &policy,
            ).unwrap();
            assert_eq!(fingerprint, policy.policy_fingerprint());
            assert_eq!(consumed.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), (identity.clone(), slice.clone()));
        } else {
            child_identity = Some(identity.clone());
        }
        assert_eq!(mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap(), caller_typing);
        assert_eq!(typing.request(), &request);
        assert_eq!(typing.descriptor(), &caller_typing);
        assert_eq!(slice, raw.3);
        assert_eq!(identity.obligation_id(), anchor);
        assert_eq!(identity.obligation_anchor().as_str(), anchor);
        assert_eq!(identity.obligation_fingerprint(), expected_obligation);
        assert_eq!(identity.vc_fingerprint(), expected_vc);
        assert_eq!(identity.local_context_fingerprint(), expected_context);
        assert_eq!(identity.dependency_slice_fingerprint(), expected_slice);
        assert_eq!(slice.fingerprint().hash(), expected_slice);
        assert_eq!(slice.vc(), VcId::new(0));
        assert_eq!(slice.kind(), &kind);
        assert_eq!(slice.status(), &VcStatus::Open);
        assert_eq!(slice.completeness(), C::IncompleteUncacheable);
        assert!(!slice.is_complete());
        assert!(slice.requires_cache_miss());
        assert_eq!(slice.entries().iter().map(|entry| (entry.class(), entry.local_key(), entry.payload())).collect::<Vec<_>>(), entries.iter().map(|(class, key, payload)| (*class, key.as_str(), payload.as_str())).collect::<Vec<_>>());
        assert_eq!(slice.unknowns().iter().map(|unknown| (unknown.family(), unknown.local_key(), unknown.reason())).collect::<Vec<_>>(), unknowns.iter().map(|(family, key, reason)| (*family, key.as_str(), *reason)).collect::<Vec<_>>());
        assert_eq!(mizar_vc::generator::generate_source_unary_child_call_existential_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap(), descriptor_before);
        assert_eq!((retained.target(), retained.witness(), retained.context().to_vec(), retained.checker_schema_version()), retained_before);
        assert!(std::ptr::eq(retained.core(), &core));
        assert!(core.generated().is_empty() && core.proofs().is_empty() && core.obligation_seeds().is_empty());
        assert_eq!((caller.request().clone(), caller.descriptor().clone(), caller.result().target(), caller.result().witness(), caller.result().context().to_vec(), caller.result().checker_schema_version()), caller_before);
        assert!(std::ptr::eq(caller.result().core(), &core));
        assert_eq!(mizar_vc::generator::generate_source_unary_child_call_caller_e1_inhabitation_request(&source, &typed, symbols, &core, &request).unwrap(), e1_before.0);
        assert_eq!((inhabitation.descriptor().clone(), inhabitation.result().binder(), inhabitation.result().context().to_vec(), inhabitation.result().checker_schema_version()), e1_before);
        assert!(std::ptr::eq(inhabitation.result().core(), &core));
    }
    assert_eq!((source, typed, symbols.clone(), core, request), before);
}

#[test]
fn unary_call_identities_normalize_binders_and_track_semantic_owners() {
    use mizar_checker::type_checker::check_source_unary_child_call;
    use mizar_core::elaborator::lower_source_unary_child_call;
    use mizar_proof::{policy::VerifierPolicy, status::{NativeUnaryCallerE4Existential, NativeUnaryChildCallExistential, SourceUnaryCallerE4Association, SourceUnaryCallerE1InhabitationAssociation}};
    let mut baseline = None;
    for (package, module, callee, caller, formal, actual, flags, trivia, invariant) in [
        ("unary-child", "identity", "child", "caller", "a", "b", [false, false], " ", true),
        ("unary-child", "identity", "child", "caller", "formal", "actual", [false, false], "\n:: trivia\n", true),
        ("unary-child", "identity", "child", "caller", "same", "same", [false, false], " ", true),
        ("other-package", "identity", "child", "caller", "a", "b", [false, false], " ", false),
        ("unary-child", "other", "child", "caller", "a", "b", [false, false], " ", false),
        ("unary-child", "identity", "renamed", "caller", "a", "b", [false, false], " ", false),
        ("unary-child", "identity", "child", "renamed", "a", "b", [false, false], " ", false),
        ("unary-child", "identity", "child", "caller", "a", "b", [true, false], " ", false),
        ("unary-child", "identity", "child", "caller", "a", "b", [false, true], " ", false),
    ] {
        let flag = |yes| if yes { "terminating " } else { "" };
        let text = format!("definition let {formal} be object; {}algorithm {callee}({formal}) -> object do return {formal}; end; end;{trivia}definition let {actual} be object; {}algorithm {caller}({actual}) -> object do return {callee}({actual}); end; end;", flag(flags[0]), flag(flags[1]));
        let (source, nodes, collected, _) = source_registration_formula_inputs(&text, package, module, 0);
        let typed = super::super::source_registration_receiver(&source, nodes).unwrap();
        let symbols = collected.env();
        let (core, request) = lower_source_unary_child_call(
            &check_source_unary_child_call(&source, symbols, &typed).unwrap(),
        ).unwrap();
        let raw = canonical_unary_child(&source, &typed, symbols, &core, &request).unwrap();
        let policy = VerifierPolicy::default();
        let native = NativeUnaryChildCallExistential::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
        let identity = native.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap();
        assert_eq!(identity.1, raw.3);
        let caller = SourceUnaryCallerE4Association::from_source(&source, &typed, symbols, &core, &request).unwrap();
        let caller_raw = canonical_caller_e4(&source, &typed, symbols, &core, &request).unwrap();
        let caller_identity = caller.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
        assert_eq!(caller_identity.1, caller_raw.3);
        let e1_raw = canonical_caller_e1(&source, &typed, symbols, &core, &request).unwrap();
        let inhabitation = SourceUnaryCallerE1InhabitationAssociation::from_source(&source, &typed, symbols, &core, &request).unwrap();
        let e1_identity = inhabitation.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
        assert_eq!(e1_identity.1, e1_raw.3);
        let typing_raw = canonical_caller_typing(&source, &typed, symbols, &core, &request).unwrap();
        let typing = mizar_proof::status::SourceUnaryCallerE1TypingAssociation::from_source(&source, &typed, symbols, &core, &request).unwrap();
        let typing_identity = typing.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
        assert_eq!(typing_identity.1, typing_raw.3);
        let native_caller = NativeUnaryCallerE4Existential::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
        let native_e1 = mizar_proof::status::NativeUnaryCallerE1Inhabitation::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
        for _ in 0..2 {
            assert_eq!(canonical_caller_typing(&source, &typed, symbols, &core, &request).unwrap(), typing_raw);
            assert_eq!(typing.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), typing_identity);
            assert_eq!(canonical_caller_e1(&source, &typed, symbols, &core, &request).unwrap(), e1_raw);
            assert_eq!(inhabitation.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), e1_identity);
            assert_eq!(native_e1.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), e1_identity);
            assert_eq!(canonical_caller_e4(&source, &typed, symbols, &core, &request).unwrap(), caller_raw);
            assert_eq!(caller.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), caller_identity);
            assert_eq!(native_caller.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), caller_identity);
            assert_eq!(canonical_unary_child(&source, &typed, symbols, &core, &request).unwrap(), raw);
            assert_eq!(native.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), identity);
        }
        let (original_raw, original_identity, original_caller_raw, original_caller_identity, original_e1_raw, original_e1_identity, original_typing_raw, original_typing_identity) = baseline.get_or_insert_with(|| (raw.clone(), identity.clone(), caller_raw.clone(), caller_identity.clone(), e1_raw.clone(), e1_identity.clone(), typing_raw.clone(), typing_identity.clone()));
        if invariant {
            assert_eq!(&typing_raw, original_typing_raw);
            assert_eq!(&typing_identity, original_typing_identity);
            assert_eq!(&e1_raw, original_e1_raw);
            assert_eq!(&e1_identity, original_e1_identity);
            assert_eq!(&caller_raw, original_caller_raw);
            assert_eq!(&caller_identity, original_caller_identity);
            assert_eq!(&raw, original_raw);
            assert_eq!(&identity, original_identity);
        } else {
            assert_ne!(typing_raw.0, original_typing_raw.0);
            assert_ne!(typing_raw.1, original_typing_raw.1);
            assert_eq!(typing_raw.2, original_typing_raw.2);
            assert_ne!(typing_raw.3.fingerprint(), original_typing_raw.3.fingerprint());
            assert_ne!(&typing_identity.0, &original_typing_identity.0);
            assert_ne!(e1_raw.0, original_e1_raw.0);
            assert_ne!(e1_raw.1, original_e1_raw.1);
            assert_eq!(e1_raw.2, original_e1_raw.2);
            assert_ne!(e1_raw.3.fingerprint(), original_e1_raw.3.fingerprint());
            assert_ne!(&e1_identity.0, &original_e1_identity.0);
            assert_ne!(caller_raw.0, original_caller_raw.0);
            assert_ne!(caller_raw.1, original_caller_raw.1);
            assert_eq!(caller_raw.2, original_caller_raw.2);
            assert_ne!(caller_raw.3.fingerprint(), original_caller_raw.3.fingerprint());
            assert_ne!(&caller_identity.0, &original_caller_identity.0);
            assert_ne!(raw.0, original_raw.0);
            assert_ne!(raw.1, original_raw.1);
            assert_eq!(raw.2, original_raw.2);
            assert_ne!(raw.3.fingerprint(), original_raw.3.fingerprint());
            assert_ne!(&identity.0, &original_identity.0);
        }
        assert!(core.generated().is_empty() && core.proofs().is_empty() && core.obligation_seeds().is_empty());
        assert!(raw.3.requires_cache_miss());
        assert!(caller_raw.3.requires_cache_miss());
        assert!(e1_raw.3.requires_cache_miss());
        assert!(typing_raw.3.requires_cache_miss());
    }
}

#[test]
fn unary_child_request_preserves_real_joins_renames_scopes_and_intent() {
    use mizar_checker::{
        type_checker::{TermReference, TypeHeadRef, check_source_unary_child_call},
        typed_ast::{TypeEntryActual, TypedSiteRef},
    };
    use mizar_core::{
        core_ir::{
            CoreAlgorithmStmtKind as S, CoreFormulaKind as F, CoreSourceAnchor, CoreTermKind as T,
        },
        elaborator::lower_source_unary_child_call,
    };
    use mizar_session::{SourceAnchor, SourceRange};
    use mizar_proof::{
        policy::{KernelEvidenceFormat as Format, VerifierPolicy as Policy},
        status::{NativeUnaryCallerE1Inhabitation as NativeInhabitation, SourceUnaryCallerE1InhabitationAssociation as InhabitationAssociation, NativeUnaryCallerE4Existential as NativeCaller, NativeUnaryChildCallExistential as Native, consume_source_unary_child_call_existential as consume, consume_source_unary_caller_e4_existential as consume_caller},
    };
    for (callee, caller, formal, actual, flags, trivia) in [
        ("child", "caller", "a", "b", [false, false], " "),
        (
            "renamed",
            "outer",
            "input",
            "arg",
            [true, false],
            "\n:: boundary trivia\n",
        ),
        ("inner", "outer", "same", "same", [false, true], " "),
        ("child", "caller", "a", "b", [true, true], " "),
    ] {
        let flag = |yes| if yes { "terminating " } else { "" };
        let text = format!(
            "definition let {formal} be object; {}algorithm {callee}({formal}) -> object do return {formal}; end; end;{trivia}definition let {actual} be object; {}algorithm {caller}({actual}) -> object do return {callee}({actual}); end; end;",
            flag(flags[0]),
            flag(flags[1])
        );
        let (source, nodes, collected, _) =
            source_registration_formula_inputs(&text, "unary-child", "joins", 0);
        let typed = super::super::source_registration_receiver(&source, nodes).unwrap();
        let symbols = collected.env();
        let before = (source.clone(), typed.clone(), symbols.clone());
        let checked = check_source_unary_child_call(&source, symbols, &typed).unwrap();
        let checks = checked.algorithms();
        let range = |start, len| SourceRange {
            source_id: source.source_id(),
            start,
            end: start + len,
        };
        let algorithms = [
            format!(
                "{}algorithm {callee}({formal}) -> object do return {formal}; end;",
                flag(flags[0])
            ),
            format!(
                "{}algorithm {caller}({actual}) -> object do return {callee}({actual}); end;",
                flag(flags[1])
            ),
        ]
        .map(|part| range(text.find(&part).unwrap(), part.len()));
        let declarations = [
            range(text.find("let ").unwrap() + 4, formal.len()),
            range(text.rfind("let ").unwrap() + 4, actual.len()),
        ];
        let parameter_types = [
            range(text.find("be object").unwrap() + 3, 6),
            range(text.rfind("be object").unwrap() + 3, 6),
        ];
        let result_types = [
            range(text.find("-> object").unwrap() + 3, 6),
            range(text.rfind("-> object").unwrap() + 3, 6),
        ];
        let headers = [
            range(
                text.find(&format!("{callee}({formal})")).unwrap() + callee.len() + 1,
                formal.len(),
            ),
            range(
                text.find(&format!("{caller}({actual})")).unwrap() + caller.len() + 1,
                actual.len(),
            ),
        ];
        let application_range = range(
            text.rfind(&format!("{callee}({actual})")).unwrap(),
            callee.len() + actual.len() + 2,
        );
        let arguments = [
            range(text.find("return ").unwrap() + 7, formal.len()),
            range(application_range.start + callee.len() + 1, actual.len()),
        ];
        let returns = [
            range(text.find("return ").unwrap(), formal.len() + 8),
            range(
                text.rfind("return ").unwrap(),
                application_range.end - text.rfind("return ").unwrap() + 1,
            ),
        ];
        let checked_range = |id| {
            let SourceAnchor::Range(range) = typed.nodes().node(id).unwrap().anchor else {
                panic!("actual source range")
            };
            range
        };
        assert_eq!(checked_range(checked.application()), application_range);
        assert_eq!(checked.arguments().map(checked_range), arguments);
        assert_eq!(checked.returns().map(checked_range), returns);
        let mut binding_ids = Vec::new();
        for (ordinal, check) in checks.iter().enumerate() {
            assert_eq!(check.declared_terminating(), flags[ordinal]);
            assert_eq!(checked_range(check.algorithm().0), algorithms[ordinal]);
            let declared = symbols
                .symbols()
                .iter()
                .find(|entry| entry.symbol() == &check.algorithm().1)
                .unwrap();
            assert_eq!(declared.primary_spelling(), [callee, caller][ordinal]);
            assert_eq!(
                declared.origin().anchor(),
                &typed.nodes().node(check.algorithm().0).unwrap().anchor
            );
            let inference = check.inference();
            assert!(
                inference.diagnostics().is_empty()
                    && inference.facts().is_empty()
                    && inference.candidate_sets().is_empty()
            );
            let mut references = Vec::new();
            for (_, term) in inference.terms().iter() {
                let Some(TermReference::Binding(binding)) = term.reference else {
                    panic!("real parameter binding")
                };
                references.push(binding);
                let entry = inference.type_entries().get(term.type_entry).unwrap();
                let TypeEntryActual::Known(ty) = entry.actual else {
                    panic!("authenticated actual type")
                };
                assert_eq!(
                    inference.normalized_types().get(ty).unwrap().head,
                    TypeHeadRef::BuiltinObject
                );
                assert_eq!(
                    inference.normalized_types().get(ty).unwrap().source.range,
                    parameter_types[ordinal]
                );
                assert_eq!(
                    inference
                        .normalized_types()
                        .get(entry.expected.unwrap())
                        .unwrap()
                        .head,
                    TypeHeadRef::BuiltinObject
                );
                assert_eq!(entry.owner, term.site);
                assert_eq!(entry.expected, term.expected_type);
                assert!(
                    [headers[ordinal], arguments[ordinal]]
                        .contains(&checked_range(term.site.node()))
                );
                for role in ["algorithm.actual", "algorithm.return"] {
                    let expected_owner = TypedSiteRef::Role {
                        node: term.site.node(),
                        role: role.into(),
                    };
                    let type_entry = inference
                        .type_entries()
                        .iter()
                        .find(|(_, entry)| entry.owner == expected_owner)
                        .unwrap()
                        .1;
                    assert_eq!(
                        type_entry.actual,
                        TypeEntryActual::Known(if role == "algorithm.actual" {
                            ty
                        } else {
                            entry.expected.unwrap()
                        })
                    );
                }
            }
            assert_eq!(references.len(), 2);
            assert_eq!(references[0], references[1]);
            binding_ids.push(references[0]);
        }
        assert_ne!(binding_ids[0], binding_ids[1]);
        let first = checks[0].bindings().bindings().get(binding_ids[0]).unwrap();
        let second = checks[1].bindings().bindings().get(binding_ids[1]).unwrap();
        assert_ne!(first.declaration_range, second.declaration_range);
        for (ordinal, binding) in [first, second].into_iter().enumerate() {
            assert_eq!(binding.declaration_range, declarations[ordinal]);
            assert_eq!(
                binding.type_site,
                mizar_checker::binding_env::BindingTypeSite::Source(parameter_types[ordinal])
            );
        }
        let (core, request) = lower_source_unary_child_call(&checked).unwrap();
        let callee_row = core.algorithms().get(request.callee).unwrap();
        let caller_row = core.algorithms().get(request.caller).unwrap();
        assert_eq!(core.algorithms().len(), 2);
        assert_eq!(
            (&callee_row.symbol, &caller_row.symbol),
            (&checks[0].algorithm().1, &checks[1].algorithm().1)
        );
        assert_eq!(
            callee_row.source.anchor,
            CoreSourceAnchor::SourceRange(algorithms[0])
        );
        assert_eq!(
            caller_row.source.anchor,
            CoreSourceAnchor::SourceRange(algorithms[1])
        );
        assert_eq!(
            (
                callee_row.declared_terminating,
                caller_row.declared_terminating
            ),
            (flags[0], flags[1])
        );
        assert_eq!(callee_row.params[0].var, request.formal);
        assert_eq!(
            callee_row.result.as_ref().unwrap().var,
            request.callee_result
        );
        assert_eq!(
            caller_row.result.as_ref().unwrap().var,
            request.caller_result
        );
        let vars = [
            request.formal,
            caller_row.params[0].var,
            request.callee_result,
            request.caller_result,
        ];
        assert_eq!(
            vars.into_iter()
                .collect::<std::collections::BTreeSet<_>>()
                .len(),
            4
        );
        for (binder, expected_range) in [
            (&callee_row.params[0], declarations[0]),
            (&caller_row.params[0], declarations[1]),
            (callee_row.result.as_ref().unwrap(), result_types[0]),
            (caller_row.result.as_ref().unwrap(), result_types[1]),
        ] {
            assert_eq!(
                binder.source.anchor,
                CoreSourceAnchor::SourceRange(expected_range)
            );
            let F::TypePred { subject, ty } =
                &core.formulas().get(binder.ty_guard.unwrap()).unwrap().kind
            else {
                panic!("authenticated object guard")
            };
            assert_eq!(ty.as_str(), "object");
            assert_eq!(core.terms().get(*subject).unwrap().kind, T::Var(binder.var));
        }
        let application = core.terms().get(request.application).unwrap();
        assert_eq!(
            application.kind,
            T::Apply {
                functor: callee_row.symbol.clone(),
                args: vec![request.actual]
            }
        );
        assert_eq!(
            application.source.anchor,
            CoreSourceAnchor::SourceRange(application_range)
        );
        assert_eq!(
            core.source_map().term_sources.get(&request.application),
            Some(&application.source)
        );
        let actual_row = core.terms().get(request.actual).unwrap();
        assert_eq!(actual_row.kind, T::Var(caller_row.params[0].var));
        assert_eq!(
            actual_row.source.anchor,
            CoreSourceAnchor::SourceRange(arguments[1])
        );
        assert_eq!(
            core.source_map().term_sources.get(&request.actual),
            Some(&actual_row.source)
        );
        assert_eq!(caller_row.statements, vec![request.statement]);
        for (ordinal, owner, row) in [
            (0, request.callee, callee_row),
            (1, request.caller, caller_row),
        ] {
            let statement = core.algorithm_statements().get(row.statements[0]).unwrap();
            assert_eq!(statement.owner, owner);
            assert_eq!(
                statement.source.anchor,
                CoreSourceAnchor::SourceRange(returns[ordinal])
            );
            assert_eq!(
                core.source_map().algorithm_sources.get(&row.statements[0]),
                Some(&statement.source)
            );
            let S::Return(Some(value)) = statement.kind else {
                panic!("sole actual Return")
            };
            if ordinal == 0 {
                assert_eq!(
                    core.terms().get(value).unwrap().kind,
                    T::Var(request.formal)
                );
                assert_eq!(
                    core.terms().get(value).unwrap().source.anchor,
                    CoreSourceAnchor::SourceRange(arguments[0])
                );
            } else {
                assert_eq!(value, request.application);
            }
        }
        assert!(
            core.obligation_seeds().is_empty()
                && core.generated().is_empty()
                && core.proofs().is_empty()
                && core.diagnostics().is_empty()
        );
        let core_before = (core.clone(), request.clone());
        let expected = (
            (request.formal, request.actual),
            (
                callee_row.params[0].ty_guard.unwrap(),
                caller_row.params[0].ty_guard.unwrap(),
            ),
            (request.callee_result, request.caller_result),
        );
        for _ in 0..2 {
            assert_eq!(
                mizar_vc::generator::generate_source_unary_child_call_descriptor(
                    &source, &typed, symbols, &core, &request,
                )
                .unwrap(),
                expected
            );
        }
        let existential = mizar_vc::generator::generate_source_unary_child_call_existential_request(
            &source, &typed, symbols, &core, &request,
        )
        .unwrap();
        let (context, rows, witness) = &existential;
        use mizar_vc::vc_ir::{
            ContextEntryKind, QuantifierKind, VcFormulaRef, VcGeneratedFormulaId,
            VcGeneratedFormulaShape as Shape,
        };
        let [guard, binder] = context.entries() else {
            panic!("sole caller guard and fresh result binder")
        };
        assert_eq!(guard.kind, ContextEntryKind::CheckerFact);
        assert_eq!(guard.formula, Some(VcFormulaRef::Core(expected.1.1)));
        assert!(context.policy_inputs().is_empty());
        let ContextEntryKind::BinderDeclaration { var, role } = &binder.kind else {
            panic!("result binder")
        };
        assert_eq!(role.as_str(), "result");
        assert!(binder.formula.is_none());
        assert!(!vars.contains(var));
        assert_eq!(*witness, request.actual);
        let Shape::AlgorithmReturns {
            algorithm, flow, statement, input, result,
        } = rows[0].shape else {
            panic!("original child return relation")
        };
        assert_eq!(rows[0].id, VcGeneratedFormulaId::new(0));
        assert_eq!(algorithm, request.callee);
        assert_eq!(flow, mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&request.callee]);
        assert_eq!(statement, callee_row.statements[0]);
        assert_eq!((input.var, input.definition), (caller_row.params[0].var, None));
        assert_eq!((result.var, result.definition), (*var, None));
        assert_eq!(rows[1].id, VcGeneratedFormulaId::new(1));
        assert_eq!(rows[1].shape, Shape::Quantified {
            kind: QuantifierKind::Exists,
            binders: vec![binder.id],
            body: VcFormulaRef::Generated(rows[0].id),
        });
        let caller_e1 = mizar_vc::generator::generate_source_unary_child_call_caller_e1_inhabitation_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert_eq!(&caller_e1.0, context);
        assert_eq!(caller_e1.2, request.actual);
        assert_ne!(caller_e1.2, request.application);
        assert_eq!(caller_e1.1[0].id, VcGeneratedFormulaId::new(0));
        assert_eq!(caller_e1.1[0].kind, rows[0].kind);
        assert_eq!(caller_e1.1[0].provenance, rows[0].provenance);
        assert_eq!(caller_e1.1[0].shape, Shape::ProgramTypePredicate {
            subject: mizar_vc::vc_ir::VcProgramValue { var: *var, definition: None },
            ty: "object".into(),
        });
        assert_eq!(caller_e1.1[1], rows[1]);
        let (checked_caller_e1, inhabited) =
            mizar_vc::generator::check_source_unary_child_call_caller_e1_inhabitation_result(
                &source, &typed, symbols, &core, &request,
            ).unwrap();
        assert_eq!(checked_caller_e1, caller_e1);
        assert!(std::ptr::eq(inhabited.core(), &core));
        assert_eq!(inhabited.binder(), *var);
        assert_eq!(inhabited.context(), [expected.1.1]);
        assert_eq!(inhabited.checker_schema_version(), 1);
        let inhabitation_association = InhabitationAssociation::from_source(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert!(inhabitation_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
        assert_eq!(inhabitation_association.request(), &request);
        assert_eq!(inhabitation_association.descriptor(), &caller_e1);
        let e1_raw = canonical_caller_e1(&source, &typed, symbols, &core, &request).unwrap();
        let e1_identity = inhabitation_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
        assert_eq!(e1_identity.1, e1_raw.3);
        let native_inhabitation = NativeInhabitation::from_source(
            &source, &typed, symbols, &core, &request, &Policy::default(),
        ).unwrap();
        assert!(native_inhabitation.adopt_original_request(&source, &typed, symbols, &core, &request, &Policy::default()).unwrap());
        assert_eq!(native_inhabitation.obligation_identity(&source, &typed, symbols, &core, &request, &Policy::default()).unwrap().unwrap(), e1_identity);
        let kernel_inhabited = mizar_kernel::checker::check_object_inhabitation_result(
            &core, (*var, *var, "object"), request.actual, expected.1.1, &[expected.1.1],
        ).unwrap();
        assert!(std::ptr::eq(inhabitation_association.result().core(), kernel_inhabited.core()));
        assert_eq!(inhabitation_association.result().binder(), kernel_inhabited.binder());
        assert_eq!(inhabitation_association.result().context(), kernel_inhabited.context());
        assert_eq!(inhabitation_association.result().checker_schema_version(), kernel_inhabited.checker_schema_version());
        let caller_e4 = mizar_vc::generator::generate_source_unary_child_call_caller_e4_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert_eq!(&caller_e4.0, context);
        assert_eq!(caller_e4.1[0].id, VcGeneratedFormulaId::new(0));
        assert_eq!(caller_e4.1[0].kind, rows[0].kind);
        assert_eq!(caller_e4.1[0].provenance, rows[0].provenance);
        assert_eq!(caller_e4.1[0].shape, Shape::AlgorithmReturns {
            algorithm: request.caller,
            flow: mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&request.caller],
            statement: request.statement,
            input,
            result,
        });
        assert_ne!(algorithm, request.caller);
        assert_ne!(flow, mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&request.caller]);
        assert_ne!(statement, request.statement);
        assert_eq!(caller_e4.1[1], rows[1]);
        let caller_typing = mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert_eq!(&caller_typing.0, context);
        assert_eq!(caller_typing.1[0], caller_e4.1[0]);
        for (index, row) in caller_typing.1.iter().enumerate() {
            assert_eq!(row.id, VcGeneratedFormulaId::new(index));
            assert_eq!(row.kind, mizar_vc::vc_ir::VcGeneratedFormulaKind::SplitGoal);
            assert!(row.provenance.is_empty());
        }
        assert_eq!(caller_typing.1[0].shape, Shape::AlgorithmReturns {
            algorithm: request.caller,
            flow: mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&request.caller],
            statement: caller_row.statements[0],
            input: mizar_vc::vc_ir::VcProgramValue { var: caller_row.params[0].var, definition: None },
            result: mizar_vc::vc_ir::VcProgramValue { var: *var, definition: None },
        });
        assert_eq!(caller_typing.1[1].kind, caller_e4.1[1].kind);
        assert_eq!(caller_typing.1[1].provenance, caller_e4.1[1].provenance);
        assert_eq!(caller_typing.1[1].shape, Shape::ProgramTypePredicate {
            subject: mizar_vc::vc_ir::VcProgramValue { var: *var, definition: None },
            ty: "object".into(),
        });
        assert_eq!(caller_typing.1[2].shape, Shape::Implies {
            premise: VcFormulaRef::Generated(VcGeneratedFormulaId::new(0)),
            conclusion: VcFormulaRef::Generated(VcGeneratedFormulaId::new(1)),
        });
        assert_eq!(caller_typing.1[3].shape, Shape::Quantified {
            kind: QuantifierKind::Forall,
            binders: vec![binder.id],
            body: VcFormulaRef::Generated(VcGeneratedFormulaId::new(2)),
        });
        assert_eq!(mizar_vc::generator::check_source_unary_child_call_caller_e1_typing_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap(), caller_typing);
        assert!(mizar_kernel::checker::check_unary_caller_return_typing(
            &core, &request, flow,
            (request.caller, mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&request.caller], caller_row.statements[0], (caller_row.params[0].var, None), (*var, None)),
            *var, ((*var, None), "object"), &[expected.1.1],
        ));
        let typing_association = mizar_proof::status::SourceUnaryCallerE1TypingAssociation::from_source(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert_eq!(typing_association.request(), &request);
        assert_eq!(typing_association.descriptor(), &caller_typing);
        let typing_raw = canonical_caller_typing(&source, &typed, symbols, &core, &request).unwrap();
        let typing_identity = typing_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
        assert_eq!(typing_identity.1, typing_raw.3);
        assert!(typing_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
        let (checked_existential, retained) =
            mizar_vc::generator::check_source_unary_child_call_existential_result(
                &source, &typed, symbols, &core, &request,
            ).unwrap();
        let (composed_descriptor, composed) =
            mizar_vc::generator::check_source_unary_child_call_caller_e4_result(
                &source, &typed, symbols, &core, &request, &retained,
            ).unwrap();
        assert_eq!(composed_descriptor, caller_e4);
        assert_eq!(composed.witness(), request.actual);
        assert!(std::ptr::eq(composed.core(), &core));
        assert_eq!(composed.target(), (
            request.caller,
            mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&request.caller],
            request.statement, (input.var, None), (*var, None),
        ));
        assert_eq!(composed.context(), [expected.1.1]);
        assert_eq!(composed.checker_schema_version(), retained.checker_schema_version());
        assert_eq!(retained.target(), (algorithm, flow, statement, (input.var, None), (*var, None)));
        let caller_association = mizar_proof::status::SourceUnaryCallerE4Association::from_source(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        assert!(caller_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
        assert_eq!(caller_association.request(), &request);
        assert_eq!(caller_association.descriptor(), &caller_e4);
        assert!(std::ptr::eq(caller_association.result().core(), &core));
        assert_eq!(caller_association.result().target(), composed.target());
        assert_eq!(caller_association.result().witness(), composed.witness());
        assert_eq!(caller_association.result().context(), composed.context());
        assert_eq!(caller_association.result().checker_schema_version(), composed.checker_schema_version());
        let association = mizar_proof::status::SourceUnaryChildCallAssociation::from_source(
            &source, &typed, symbols, &core, &request,
        ).unwrap();
        let policy = Policy::default();
        let (consumed_caller, caller_fingerprint) = consume_caller(
            &source, &typed, symbols, &core, &request, &policy,
        ).unwrap();
        assert_eq!(caller_fingerprint, policy.policy_fingerprint());
        let native_caller = NativeCaller::from_source(
            &source, &typed, symbols, &core, &request, &policy,
        ).unwrap();
        assert!(native_caller.adopt_original_request(&source, &typed, symbols, &core, &request, &policy).unwrap());
        let caller_identity_before = caller_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
        assert_eq!(native_caller.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), caller_identity_before);
        let native = Native::from_source(
            &source, &typed, symbols, &core, &request, &policy,
        ).unwrap();
        assert!(native.adopt_original_request(&source, &typed, symbols, &core, &request, &policy).unwrap());
        let identity_before = native.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap();
        let raw = canonical_unary_child(&source, &typed, symbols, &core, &request).unwrap();
        assert_eq!(identity_before.0.vc_fingerprint(), raw.0);
        assert_eq!(identity_before.1, raw.3);
        let (consumed, fingerprint) = consume(
            &source, &typed, symbols, &core, &request, &policy,
        ).unwrap();
        assert_eq!(fingerprint, policy.policy_fingerprint());
        assert_eq!(checked_existential, existential);
        assert!(std::ptr::eq(retained.core(), &core));
        assert_eq!(retained.target(), (algorithm, flow, statement, (input.var, None), (*var, None)));
        assert_eq!(retained.witness(), request.actual);
        assert_eq!(retained.context(), [expected.1.1]);
        let schema = retained.checker_schema_version();
        for denied in [
            policy.clone().with_kernel_evidence_formats([]),
            policy.clone().with_kernel_evidence_formats([Format::FormulaSubstitution]),
            policy.clone().with_checker_schema_version(schema - 1),
            policy.clone().with_kernel_evidence_formats([Format::BuiltinKernelEvidence])
                .with_checker_schema_version(schema + 1),
        ] {
            let denied_before = denied.clone();
            assert_eq!(NativeInhabitation::from_source(&source, &typed, symbols, &core, &request, &denied).err().unwrap(),
                "algorithm.proof.native_e1_policy_rejected");
            assert_eq!(native_inhabitation.adopt_original_request(&source, &typed, symbols, &core, &request, &denied).unwrap_err(),
                "algorithm.proof.native_e1_policy_rejected");
            assert_eq!(native_inhabitation.obligation_identity(&source, &typed, symbols, &core, &request, &denied).unwrap_err(),
                "algorithm.proof.native_e1_policy_rejected");
            assert_eq!(consume(&source, &typed, symbols, &core, &request, &denied).err().unwrap(),
                "algorithm.proof.unary_child_policy_rejected");
            assert_eq!(consume_caller(&source, &typed, symbols, &core, &request, &denied).err().unwrap(),
                "algorithm.proof.unary_caller_e4_policy_rejected");
            assert_eq!(NativeCaller::from_source(&source, &typed, symbols, &core, &request, &denied).err().unwrap(),
                "algorithm.proof.unary_caller_e4_policy_rejected");
            assert_eq!(native_caller.adopt_original_request(&source, &typed, symbols, &core, &request, &denied).unwrap_err(),
                "algorithm.proof.unary_caller_e4_policy_rejected");
            assert_eq!(native_caller.obligation_identity(&source, &typed, symbols, &core, &request, &denied).unwrap_err(),
                "algorithm.proof.unary_caller_e4_policy_rejected");
            assert_eq!(inhabitation_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), e1_identity);
            assert_eq!(typing_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), typing_identity);
            assert_eq!(caller_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), caller_identity_before);
            assert_eq!(caller_fingerprint, policy.policy_fingerprint());
            assert_eq!(Native::from_source(&source, &typed, symbols, &core, &request, &denied).err().unwrap(),
                "algorithm.proof.unary_child_policy_rejected");
            assert_eq!(native.adopt_original_request(&source, &typed, symbols, &core, &request, &denied).unwrap_err(),
                "algorithm.proof.unary_child_policy_rejected");
            assert_eq!(native.obligation_identity(&source, &typed, symbols, &core, &request, &denied).unwrap_err(),
                "algorithm.proof.unary_child_policy_rejected");
            assert_eq!(denied, denied_before);
            assert_eq!(fingerprint, policy.policy_fingerprint());
        }
        for current_policy in [
            policy.clone(),
            policy.clone().with_kernel_evidence_formats([Format::BuiltinKernelEvidence]),
            Policy::development(),
            Policy::interactive(),
            policy.clone().with_profile_id("unary-child-current"),
            policy.clone().with_schema_version(policy.schema_version() + 1),
        ] {
            let policy_before = current_policy.clone();
            for _ in 0..2 {
                let (current, current_fingerprint) = consume(
                    &source, &typed, symbols, &core, &request, &current_policy,
                ).unwrap();
                let (current_caller, current_caller_fingerprint) = consume_caller(
                    &source, &typed, symbols, &core, &request, &current_policy,
                ).unwrap();
                assert_eq!(current_caller_fingerprint, current_policy.policy_fingerprint());
                for product in [&caller_association, &consumed_caller, &current_caller] {
                    assert!(product.matches_source(&source, &typed, symbols, &core, &request).unwrap());
                    assert_eq!(product.request(), &request);
                    assert_eq!(product.descriptor(), &caller_e4);
                    assert!(std::ptr::eq(product.result().core(), &core));
                    assert_eq!(product.result().target(), composed.target());
                    assert_eq!(product.result().witness(), request.actual);
                    assert_eq!(product.result().context(), composed.context());
                    assert_eq!(product.result().checker_schema_version(), composed.checker_schema_version());
                }
                let current_native_inhabitation = NativeInhabitation::from_source(&source, &typed, symbols, &core, &request, &current_policy).unwrap();
                for product in [&native_inhabitation, &current_native_inhabitation] {
                    assert!(product.adopt_original_request(&source, &typed, symbols, &core, &request, &current_policy).unwrap());
                    assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request, &current_policy).unwrap().unwrap(), e1_identity);
                }
                let current_native_caller = NativeCaller::from_source(&source, &typed, symbols, &core, &request, &current_policy).unwrap();
                for product in [&native_caller, &current_native_caller] {
                    assert!(product.adopt_original_request(&source, &typed, symbols, &core, &request, &current_policy).unwrap());
                    assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request, &current_policy).unwrap().unwrap(), caller_identity_before);
                }
                let current_native = Native::from_source(&source, &typed, symbols, &core, &request, &current_policy).unwrap();
                for product in [&native, &current_native] {
                    assert!(product.adopt_original_request(&source, &typed, symbols, &core, &request, &current_policy).unwrap());
                    assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request, &current_policy).unwrap().unwrap(), identity_before);
                }
                assert_eq!(current_fingerprint, current_policy.policy_fingerprint());
                for product in [&association, &consumed, &current] {
                    assert!(product.matches_source(&source, &typed, symbols, &core, &request).unwrap());
                    assert_eq!(product.request(), &request);
                    assert_eq!(product.descriptor(), &existential);
                    let result = product.result();
                    assert!(std::ptr::eq(result.core(), &core));
                    assert_eq!(result.target(), retained.target());
                    assert_eq!(result.witness(), retained.witness());
                    assert_eq!(result.context(), retained.context());
                    assert_eq!(result.checker_schema_version(), schema);
                }
            }
            assert_eq!(current_policy, policy_before);
        }
        assert_eq!(mizar_vc::generator::generate_source_unary_child_call_existential_request(
            &source, &typed, symbols, &core, &request,
        ).unwrap(), existential);
        let cloned_core = core.clone();
        for current_core in [&core, &cloned_core] {
            assert_eq!(mizar_vc::generator::generate_source_unary_child_call_caller_e4_request(
                &source, &typed, symbols, current_core, &request,
            ).unwrap(), caller_e4);
            let (_, current_child) = mizar_vc::generator::check_source_unary_child_call_existential_result(
                &source, &typed, symbols, current_core, &request,
            ).unwrap();
            let (current_association, current_fingerprint) = consume_caller(
                &source, &typed, symbols, current_core, &request, &policy,
            ).unwrap();
            assert_eq!(current_fingerprint, policy.policy_fingerprint());
            for _ in 0..2 {
                assert_eq!(mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
                    &source, &typed, symbols, current_core, &request,
                ).unwrap(), caller_typing);
                assert_eq!(mizar_vc::generator::check_source_unary_child_call_caller_e1_typing_request(
                    &source, &typed, symbols, current_core, &request,
                ).unwrap(), caller_typing);
                let current_typing = mizar_proof::status::SourceUnaryCallerE1TypingAssociation::from_source(
                    &source, &typed, symbols, current_core, &request,
                ).unwrap();
                assert_eq!(current_typing.request(), &request);
                assert_eq!(current_typing.descriptor(), &caller_typing);
                assert_eq!(canonical_caller_typing(&source, &typed, symbols, current_core, &request).unwrap(), typing_raw);
                assert_eq!(current_typing.obligation_identity(&source, &typed, symbols, current_core, &request).unwrap().unwrap(), typing_identity);
                assert_eq!(typing_association.obligation_identity(&source, &typed, symbols, current_core, &request).unwrap(), std::ptr::eq(current_core, &core).then(|| typing_identity.clone()));
                assert_eq!(current_typing.obligation_identity(&source, &typed, symbols, &core, &request).unwrap(), std::ptr::eq(current_core, &core).then(|| typing_identity.clone()));
                assert!(current_typing.matches_source(&source, &typed, symbols, current_core, &request).unwrap());
                assert_eq!(typing_association.matches_source(&source, &typed, symbols, current_core, &request).unwrap(), std::ptr::eq(current_core, &core));
                assert_eq!(current_typing.matches_source(&source, &typed, symbols, &core, &request).unwrap(), std::ptr::eq(current_core, &core));
                assert!(mizar_kernel::checker::check_unary_caller_return_typing(
                    current_core, &request, flow, composed.target(), *var,
                    ((*var, None), "object"), &[expected.1.1],
                ));
                assert_eq!(mizar_vc::generator::generate_source_unary_child_call_caller_e1_inhabitation_request(
                    &source, &typed, symbols, current_core, &request,
                ).unwrap(), caller_e1);
                let (descriptor, result) = mizar_vc::generator::check_source_unary_child_call_caller_e1_inhabitation_result(
                    &source, &typed, symbols, current_core, &request,
                ).unwrap();
                assert_eq!(descriptor, caller_e1);
                let current_inhabitation = InhabitationAssociation::from_source(
                    &source, &typed, symbols, current_core, &request,
                ).unwrap();
                assert_eq!(canonical_caller_e1(&source, &typed, symbols, current_core, &request).unwrap(), e1_raw);
                assert_eq!(current_inhabitation.obligation_identity(&source, &typed, symbols, current_core, &request).unwrap().unwrap(), e1_identity);
                assert_eq!(inhabitation_association.obligation_identity(&source, &typed, symbols, current_core, &request).unwrap(), std::ptr::eq(current_core, &core).then(|| e1_identity.clone()));
                assert_eq!(current_inhabitation.obligation_identity(&source, &typed, symbols, &core, &request).unwrap(), std::ptr::eq(current_core, &core).then(|| e1_identity.clone()));
                assert!(current_inhabitation.matches_source(&source, &typed, symbols, current_core, &request).unwrap());
                assert_eq!(inhabitation_association.matches_source(&source, &typed, symbols, current_core, &request).unwrap(), std::ptr::eq(current_core, &core));
                assert_eq!(current_inhabitation.matches_source(&source, &typed, symbols, &core, &request).unwrap(), std::ptr::eq(current_core, &core));
                assert_eq!(current_inhabitation.request(), &request);
                assert_eq!(current_inhabitation.descriptor(), &caller_e1);
                assert!(std::ptr::eq(current_inhabitation.result().core(), current_core));
                assert_eq!(current_inhabitation.result().binder(), inhabited.binder());
                assert_eq!(current_inhabitation.result().context(), inhabited.context());
                assert_eq!(current_inhabitation.result().checker_schema_version(), inhabited.checker_schema_version());
                assert!(std::ptr::eq(result.core(), current_core));
                assert_eq!(result.binder(), inhabited.binder());
                assert_eq!(result.context(), inhabited.context());
                assert_eq!(result.checker_schema_version(), inhabited.checker_schema_version());
                let (descriptor, result) = mizar_vc::generator::check_source_unary_child_call_caller_e4_result(
                    &source, &typed, symbols, current_core, &request, &current_child,
                ).unwrap();
                assert_eq!(descriptor, caller_e4);
                assert!(std::ptr::eq(result.core(), current_core));
                assert_eq!(result.target(), composed.target());
                assert_eq!(result.witness(), request.actual);
                assert_eq!(result.context(), composed.context());
                assert_eq!(result.checker_schema_version(), composed.checker_schema_version());
                assert!(current_association.matches_source(&source, &typed, symbols, current_core, &request).unwrap());
                assert_eq!(current_association.matches_source(&source, &typed, symbols, &core, &request).unwrap(), std::ptr::eq(current_core, &core));
                assert_eq!(caller_association.matches_source(&source, &typed, symbols, current_core, &request).unwrap(), std::ptr::eq(current_core, &core));
                assert_eq!(current_association.request(), &request);
                assert_eq!(current_association.descriptor(), &descriptor);
                assert!(std::ptr::eq(current_association.result().core(), current_core));
                assert_eq!(current_association.result().target(), result.target());
                assert_eq!(current_association.result().witness(), result.witness());
                assert_eq!(current_association.result().context(), result.context());
                assert_eq!(current_association.result().checker_schema_version(), result.checker_schema_version());
            }
        }
        assert_eq!(typing_association.request(), &request);
        assert_eq!(typing_association.descriptor(), &caller_typing);
        assert_eq!(caller_association.request(), &request);
        assert_eq!(caller_association.descriptor(), &caller_e4);
        assert!(inhabitation_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
        assert_eq!(inhabitation_association.request(), &request);
        assert_eq!(inhabitation_association.descriptor(), &caller_e1);
        assert!(std::ptr::eq(inhabitation_association.result().core(), &core));
        assert_eq!(inhabitation_association.result().binder(), inhabited.binder());
        assert_eq!(inhabitation_association.result().context(), inhabited.context());
        assert_eq!(inhabitation_association.result().checker_schema_version(), inhabited.checker_schema_version());
        assert_eq!(cloned_core, core_before.0);
        assert_eq!((&core, &request), (&core_before.0, &core_before.1));
        assert_eq!(
            lower_source_unary_child_call(
                &check_source_unary_child_call(&source, symbols, &typed).unwrap()
            )
            .unwrap(),
            (core, request)
        );
        assert_eq!((source, typed, symbols.clone()), before);
    }
}

#[test]
fn unary_child_descriptor_refuses_request_core_and_authentication_drift_atomically() {
    use mizar_proof::status::SourceUnaryCallerE1TypingAssociation as TypingAssociation;
    use mizar_checker::{
        type_checker::check_source_unary_child_call as check,
        typed_ast::{TypedArena, TypedAst, TypedAstParts, TypedNodeKind},
    };
    use mizar_core::{
        core_ir::{
            CoreAlgorithmId, CoreAlgorithmStmtId, CoreAlgorithmStmtKind as S,
            CoreFormulaId, CoreFormulaKind as F, CoreIr, CoreIrParts, CoreTermId,
            CoreTermKind as T, CoreVarId,
        },
        elaborator::lower_source_unary_child_call as lower,
    };
    use mizar_vc::generator::{
        check_source_unary_child_call_caller_e1_inhabitation_result as inhabit,
        check_source_unary_child_call_caller_e1_typing_request as check_typing,
        check_source_unary_child_call_caller_e4_result as compose,
        check_source_unary_child_call_existential_result as retain,
        generate_source_unary_child_call_caller_e1_inhabitation_request as caller_e1,
        generate_source_unary_child_call_caller_e4_request as caller_e4,
        generate_source_unary_child_call_caller_e1_typing_request as caller_typing,
        generate_source_unary_child_call_descriptor as describe,
        generate_source_unary_child_call_existential_request as generate,
    };
    use mizar_kernel::checker::{
        check_unary_caller_return_typing as typing_kernel,
        check_unary_caller_returns_existential_result as compose_kernel,
        check_unary_child_returns_existential_result as kernel,
    };
    use mizar_proof::{
        policy::VerifierPolicy as Policy,
        status::{NativeUnaryCallerE1Inhabitation as NativeInhabitation, SourceUnaryCallerE1InhabitationAssociation as InhabitationAssociation, NativeUnaryCallerE4Existential as NativeCaller, NativeUnaryChildCallExistential as Native, SourceUnaryCallerE4Association as CallerAssociation, SourceUnaryChildCallAssociation as Association, consume_source_unary_child_call_existential as consume, consume_source_unary_caller_e4_existential as consume_caller},
    };
    let text = "definition let a be object; algorithm child(a) -> object do return a; end; end; definition let b be object; algorithm caller(b) -> object do return child(b); end; end;";
    let (source, nodes, collected, _) =
        source_registration_formula_inputs(text, "unary-child", "atomic", 0);
    let typed = super::super::source_registration_receiver(&source, nodes).unwrap();
    let symbols = collected.env();
    let checked = check(&source, symbols, &typed).unwrap();
    let (core, request) = lower(&checked).unwrap();
    let before = (source.clone(), typed.clone(), symbols.clone(), core.clone(), request.clone());
    let genuine = describe(&source, &typed, symbols, &core, &request).unwrap();
    let genuine_existential = generate(&source, &typed, symbols, &core, &request).unwrap();
    let genuine_caller_e1 = caller_e1(&source, &typed, symbols, &core, &request).unwrap();
    let genuine_caller_e4 = caller_e4(&source, &typed, symbols, &core, &request).unwrap();
    let genuine_typing = caller_typing(&source, &typed, symbols, &core, &request).unwrap();
    assert_eq!(check_typing(&source, &typed, symbols, &core, &request).unwrap(), genuine_typing);
    let typing_association = TypingAssociation::from_source(&source, &typed, symbols, &core, &request).unwrap();
    let genuine_typing_raw = canonical_caller_typing(&source, &typed, symbols, &core, &request).unwrap();
    let genuine_typing_identity = typing_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
    assert_eq!(genuine_typing_identity.1, genuine_typing_raw.3);
    let (_, retained) = retain(&source, &typed, symbols, &core, &request).unwrap();
    let (composed_descriptor, composed) = compose(&source, &typed, symbols, &core, &request, &retained).unwrap();
    assert_eq!(composed_descriptor, genuine_caller_e4);
    let inhabitation_association = InhabitationAssociation::from_source(&source, &typed, symbols, &core, &request).unwrap();
    let genuine_e1_raw = canonical_caller_e1(&source, &typed, symbols, &core, &request).unwrap();
    let genuine_e1_identity = inhabitation_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
    assert_eq!(genuine_e1_identity.1, genuine_e1_raw.3);
    let caller_association = CallerAssociation::from_source(&source, &typed, symbols, &core, &request).unwrap();
    let association = Association::from_source(&source, &typed, symbols, &core, &request).unwrap();
    let policy = Policy::default();
    let denied = policy.clone().with_kernel_evidence_formats([]);
    let policies_before = (policy.clone(), denied.clone());
    let native = Native::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
    let native_caller = NativeCaller::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
    let native_inhabitation = NativeInhabitation::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
    assert_eq!(native_inhabitation.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), genuine_e1_identity);
    let (consumed, fingerprint) = consume(&source, &typed, symbols, &core, &request, &policy).unwrap();
    let (consumed_caller, caller_fingerprint) = consume_caller(&source, &typed, symbols, &core, &request, &policy).unwrap();
    let genuine_caller_identity = caller_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap();
    assert_eq!(native_caller.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), genuine_caller_identity);
    let genuine_caller_raw = canonical_caller_e4(&source, &typed, symbols, &core, &request).unwrap();
    assert_eq!(genuine_caller_identity.1, genuine_caller_raw.3);
    let cloned_core = core.clone();
    let (_, cloned_child) = retain(&source, &typed, symbols, &cloned_core, &request).unwrap();
    for (current_core, foreign_child) in [(&core, &cloned_child), (&cloned_core, &retained)] {
        assert_eq!(compose(&source, &typed, symbols, current_core, &request, foreign_child).err().unwrap(),
            "algorithms.unary_child.invalid_caller_e4_request");
        assert!(compose_kernel(current_core, &request, composed.target(), foreign_child, retained.context()).is_none());
    }
    let genuine_identity = native.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap();
    assert_eq!(canonical_unary_child(&source, &typed, symbols, &cloned_core, &request).unwrap(), canonical_unary_child(&source, &typed, symbols, &core, &request).unwrap());
    assert_eq!(canonical_caller_e4(&source, &typed, symbols, &cloned_core, &request).unwrap(), genuine_caller_raw);
    assert_eq!(canonical_caller_e1(&source, &typed, symbols, &cloned_core, &request).unwrap(), genuine_e1_raw);
    assert_eq!(caller_typing(&source, &typed, symbols, &cloned_core, &request).unwrap(), genuine_typing);
    assert_eq!(check_typing(&source, &typed, symbols, &cloned_core, &request).unwrap(), genuine_typing);
    assert!(!typing_association.matches_source(&source, &typed, symbols, &cloned_core, &request).unwrap());
    assert_eq!(canonical_caller_typing(&source, &typed, symbols, &cloned_core, &request).unwrap(), genuine_typing_raw);
    assert_eq!(typing_association.obligation_identity(&source, &typed, symbols, &cloned_core, &request).unwrap(), None);
    let cloned_typing = TypingAssociation::from_source(&source, &typed, symbols, &cloned_core, &request).unwrap();
    assert_eq!(cloned_typing.request(), &request);
    assert_eq!(cloned_typing.descriptor(), &genuine_typing);
    assert!(cloned_typing.matches_source(&source, &typed, symbols, &cloned_core, &request).unwrap());
    assert!(!cloned_typing.matches_source(&source, &typed, symbols, &core, &request).unwrap());
    assert_eq!(cloned_typing.obligation_identity(&source, &typed, symbols, &cloned_core, &request).unwrap().unwrap(), genuine_typing_identity);
    assert_eq!(cloned_typing.obligation_identity(&source, &typed, symbols, &core, &request).unwrap(), None);
    assert_eq!(inhabitation_association.obligation_identity(&source, &typed, symbols, &cloned_core, &request).unwrap(), None);
    let cloned_inhabitation = InhabitationAssociation::from_source(&source, &typed, symbols, &cloned_core, &request).unwrap();
    assert_eq!(cloned_inhabitation.obligation_identity(&source, &typed, symbols, &cloned_core, &request).unwrap().unwrap(), genuine_e1_identity);
    assert_eq!(cloned_inhabitation.obligation_identity(&source, &typed, symbols, &core, &request).unwrap(), None);
    assert_eq!(caller_association.obligation_identity(&source, &typed, symbols, &cloned_core, &request).unwrap(), None);
    let (cloned_caller, cloned_caller_fingerprint) = consume_caller(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap();
    assert_eq!(cloned_caller_fingerprint, policy.policy_fingerprint());
    assert!(std::ptr::eq(cloned_caller.result().core(), &cloned_core));
    assert_eq!(cloned_caller.obligation_identity(&source, &typed, symbols, &cloned_core, &request).unwrap().unwrap(), genuine_caller_identity);
    assert_eq!(cloned_caller.obligation_identity(&source, &typed, symbols, &core, &request).unwrap(), None);
    for current_policy in [&policy, &denied] {
        assert_eq!(native.obligation_identity(&source, &typed, symbols, &cloned_core, &request, current_policy).unwrap(), None);
    }
    assert_eq!(retain(&source, &typed, symbols, &cloned_core, &request).unwrap().0, genuine_existential);
    assert!(!association.matches_source(&source, &typed, symbols, &cloned_core, &request).unwrap());
    assert!(!caller_association.matches_source(&source, &typed, symbols, &cloned_core, &request).unwrap());
    let cloned_association = Association::from_source(&source, &typed, symbols, &cloned_core, &request).unwrap();
    assert!(cloned_association.matches_source(&source, &typed, symbols, &cloned_core, &request).unwrap());
    let (cloned_consumed, cloned_fingerprint) = consume(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap();
    assert_eq!(cloned_fingerprint, policy.policy_fingerprint());
    assert_eq!(cloned_consumed.request(), &request);
    assert_eq!(cloned_consumed.descriptor(), &genuine_existential);
    assert!(std::ptr::eq(cloned_consumed.result().core(), &cloned_core));
    assert!(cloned_consumed.matches_source(&source, &typed, symbols, &cloned_core, &request).unwrap());
    assert!(!cloned_consumed.matches_source(&source, &typed, symbols, &core, &request).unwrap());
    assert!(!consumed.matches_source(&source, &typed, symbols, &cloned_core, &request).unwrap());
    assert!(!native.adopt_original_request(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap());
    assert!(!native.adopt_original_request(&source, &typed, symbols, &cloned_core, &request, &denied).unwrap());
    let cloned_native_inhabitation = NativeInhabitation::from_source(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap();
    assert!(cloned_native_inhabitation.adopt_original_request(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap());
    assert_eq!(cloned_native_inhabitation.obligation_identity(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap().unwrap(), genuine_e1_identity);
    let cloned_native_caller = NativeCaller::from_source(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap();
    assert!(cloned_native_caller.adopt_original_request(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap());
    assert_eq!(cloned_native_caller.obligation_identity(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap().unwrap(), genuine_caller_identity);
    for current_policy in [&policy, &denied] {
        assert!(!native_inhabitation.adopt_original_request(&source, &typed, symbols, &cloned_core, &request, current_policy).unwrap());
        assert!(!cloned_native_inhabitation.adopt_original_request(&source, &typed, symbols, &core, &request, current_policy).unwrap());
        assert_eq!(native_inhabitation.obligation_identity(&source, &typed, symbols, &cloned_core, &request, current_policy).unwrap(), None);
        assert_eq!(cloned_native_inhabitation.obligation_identity(&source, &typed, symbols, &core, &request, current_policy).unwrap(), None);
        assert!(!native_caller.adopt_original_request(&source, &typed, symbols, &cloned_core, &request, current_policy).unwrap());
        assert!(!cloned_native_caller.adopt_original_request(&source, &typed, symbols, &core, &request, current_policy).unwrap());
        assert_eq!(native_caller.obligation_identity(&source, &typed, symbols, &cloned_core, &request, current_policy).unwrap(), None);
        assert_eq!(cloned_native_caller.obligation_identity(&source, &typed, symbols, &core, &request, current_policy).unwrap(), None);
    }
    let cloned_native = Native::from_source(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap();
    assert!(cloned_native.adopt_original_request(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap());
    assert!(!cloned_native.adopt_original_request(&source, &typed, symbols, &core, &request, &policy).unwrap());
    assert!(!cloned_native.adopt_original_request(&source, &typed, symbols, &core, &request, &denied).unwrap());
    assert_eq!(cloned_native.obligation_identity(&source, &typed, symbols, &cloned_core, &request, &policy).unwrap().unwrap(), genuine_identity);
    assert_eq!(cloned_native.obligation_identity(&source, &typed, symbols, &core, &request, &denied).unwrap(), None);
    let target = retained.target();
    let binder = target.4.0;
    let callee = core.algorithms().get(request.callee).unwrap();
    let caller = core.algorithms().get(request.caller).unwrap();
    let formal_guard = callee.params[0].ty_guard.unwrap();
    let actual_guard = caller.params[0].ty_guard.unwrap();
    let S::Return(Some(formal_return)) = core.algorithm_statements().get(callee.statements[0]).unwrap().kind else {
        panic!("callee Return")
    };
    assert!(kernel(&core, &request, target, binder, request.actual, &[actual_guard]).is_some());
    let mizar_vc::vc_ir::VcGeneratedFormulaShape::AlgorithmReturns {
        algorithm, flow, statement, input, result,
    } = genuine_caller_e4.1[0].shape else {
        panic!("caller normal-return descriptor")
    };
    let caller_target = (algorithm, flow, statement, (input.var, input.definition), (result.var, result.definition));
    let conclusion = ((binder, None), "object");
    assert!(typing_kernel(&core, &request, target.1, caller_target, binder, conclusion, &[actual_guard]));
    assert!(typing_kernel(&cloned_core, &request, target.1, caller_target, binder, conclusion, &[actual_guard]));
    assert!(!typing_kernel(&core, &request, target.1, target, binder, conclusion, &[actual_guard]));
    assert!(!mizar_kernel::checker::check_algorithm_return_typing(&core, caller_target, binder, conclusion, &[actual_guard]));
    for wrong_flow in [caller_target.1, mizar_core::control_flow::ControlFlowId::new(usize::MAX)] {
        assert!(!typing_kernel(&core, &request, wrong_flow, caller_target, binder, conclusion, &[actual_guard]));
    }
    assert_eq!(composed.target(), caller_target);
    assert!(compose_kernel(&core, &request, caller_target, &retained, &[actual_guard]).is_some());
    assert!(compose_kernel(&core, &request, target, &retained, &[actual_guard]).is_none());
    assert!(compose_kernel(&core, &request, caller_target, &composed, &[actual_guard]).is_none());
    assert_eq!(compose(&source, &typed, symbols, &core, &request, &composed).err().unwrap(),
        "algorithms.unary_child.invalid_caller_e4_request");
    let mut formal_target = target;
    formal_target.3.0 = request.formal;
    let formal_child = mizar_kernel::checker::check_algorithm_returns_existential_result(
        &core, formal_target, binder, formal_return, &[formal_guard],
    ).expect("formal-only child evidence is independently valid");
    assert!(mizar_kernel::checker::check_algorithm_return_typing(&core, formal_target, binder, conclusion, &[formal_guard]));
    assert!(!typing_kernel(&core, &request, target.1, formal_target, binder, conclusion, &[formal_guard]));
    assert!(compose_kernel(&core, &request, caller_target, &formal_child, &[actual_guard]).is_none());
    assert_eq!(compose(&source, &typed, symbols, &core, &request, &formal_child).err().unwrap(),
        "algorithms.unary_child.invalid_caller_e4_request");
    let mut different_binder = caller_target;
    different_binder.4.0 = CoreVarId::new(binder.index() + 1);
    assert!(compose_kernel(&core, &request, different_binder, &retained, &[actual_guard]).is_none());
    assert!(kernel(&core, &request, caller_target, binder, request.actual, &[actual_guard]).is_none());
    assert!(mizar_kernel::checker::check_algorithm_returns_existential_result(
        &core, caller_target, binder, request.actual, &[actual_guard],
    ).is_none());
    assert!(mizar_kernel::checker::check_algorithm_returns_existential_result(
        &core, target, binder, request.actual, &[actual_guard],
    ).is_none(), "the unchanged formal rule does not instantiate a caller input");
    for mutation in 0..18 {
        let mut changed = target;
        match mutation {
            0 => changed.0 = request.caller,
            1 => changed.0 = CoreAlgorithmId::new(usize::MAX),
            2 => changed.1 = mizar_core::control_flow::build_control_flow_ir(&core).flow_map[&request.caller],
            3 => changed.1 = mizar_core::control_flow::ControlFlowId::new(usize::MAX),
            4 => changed.2 = request.statement,
            5 => changed.2 = CoreAlgorithmStmtId::new(usize::MAX),
            6 => changed.3.0 = request.formal,
            7 => changed.3.0 = request.caller_result,
            8 => changed.3.0 = CoreVarId::new(usize::MAX),
            9 => changed.4.0 = request.callee_result,
            10 => changed.4.0 = request.caller_result,
            11 => changed.4.0 = CoreVarId::new(usize::MAX),
            12 => changed.3.1 = Some(request.statement),
            13 => changed.4.1 = Some(callee.statements[0]),
            14 => changed.3.1 = Some(CoreAlgorithmStmtId::new(usize::MAX)),
            15 => changed.4.1 = Some(CoreAlgorithmStmtId::new(usize::MAX)),
            16 => changed.4.0 = target.3.0,
            17 => changed.3.0 = binder,
            _ => unreachable!(),
        }
        assert!(kernel(&core, &request, changed, binder, request.actual, &[actual_guard]).is_none(), "target {mutation}");
        let mut caller_changed = caller_target;
        match mutation {
            0 => caller_changed.0 = request.callee,
            1 => caller_changed.0 = changed.0,
            2 => caller_changed.1 = target.1,
            3 => caller_changed.1 = changed.1,
            4 => caller_changed.2 = target.2,
            5 => caller_changed.2 = changed.2,
            6..=8 | 17 => caller_changed.3.0 = changed.3.0,
            9..=11 | 16 => caller_changed.4.0 = changed.4.0,
            12 | 14 => caller_changed.3.1 = changed.3.1,
            13 | 15 => caller_changed.4.1 = changed.4.1,
            _ => unreachable!(),
        }
        assert!(compose_kernel(&core, &request, caller_changed, &retained, &[actual_guard]).is_none(), "caller target {mutation}");
        assert!(!typing_kernel(&core, &request, target.1, caller_changed, binder, conclusion, &[actual_guard]), "typing target {mutation}");
    }
    for wrong in [formal_return, request.application, CoreTermId::new(usize::MAX)] {
        assert!(kernel(&core, &request, target, binder, wrong, &[actual_guard]).is_none(), "witness {wrong:?}");
    }
    for wrong in [request.formal, target.3.0, request.callee_result, request.caller_result, CoreVarId::new(usize::MAX)] {
        assert!(kernel(&core, &request, target, wrong, request.actual, &[actual_guard]).is_none(), "binder {wrong:?}");
        assert!(!typing_kernel(&core, &request, target.1, caller_target, wrong, conclusion, &[actual_guard]), "typing binder {wrong:?}");
        assert!(!typing_kernel(&core, &request, target.1, caller_target, binder, ((wrong, None), "object"), &[actual_guard]), "typing subject {wrong:?}");
    }
    for captured in [request.formal, target.3.0, request.callee_result, request.caller_result] {
        let mut changed = target;
        changed.4.0 = captured;
        assert!(kernel(&core, &request, changed, captured, request.actual, &[actual_guard]).is_none(), "captured binder {captured:?}");
        let mut caller_changed = caller_target;
        caller_changed.4.0 = captured;
        assert!(!typing_kernel(&core, &request, target.1, caller_changed, captured, ((captured, None), "object"), &[actual_guard]), "typing captured binder {captured:?}");
    }
    for wrong in [((binder, None), "set"), ((binder, Some(request.statement)), "object"), ((binder, Some(CoreAlgorithmStmtId::new(usize::MAX))), "object")] {
        assert!(!typing_kernel(&core, &request, target.1, caller_target, binder, wrong, &[actual_guard]), "typing conclusion {wrong:?}");
    }
    for context in [vec![], vec![formal_guard], vec![actual_guard, formal_guard], vec![actual_guard, actual_guard], vec![CoreFormulaId::new(usize::MAX)]] {
        assert!(kernel(&core, &request, target, binder, request.actual, &context).is_none(), "context {context:?}");
        assert!(compose_kernel(&core, &request, caller_target, &retained, &context).is_none(), "caller context {context:?}");
        assert!(!typing_kernel(&core, &request, target.1, caller_target, binder, conclusion, &context), "typing context {context:?}");
    }
    for mutation in 0..17 {
        let mut changed = request.clone();
        match mutation {
            0 => changed.callee = request.caller,
            1 => changed.caller = request.callee,
            2 => changed.application = request.actual,
            3 => changed.statement = callee.statements[0],
            4 => changed.actual = request.application,
            5 => changed.formal = request.callee_result,
            6 => changed.callee_result = request.formal,
            7 => changed.caller_result = request.formal,
            8 => changed.callee = CoreAlgorithmId::new(usize::MAX),
            9 => changed.caller = CoreAlgorithmId::new(usize::MAX),
            10 => changed.application = CoreTermId::new(usize::MAX),
            11 => changed.statement = CoreAlgorithmStmtId::new(usize::MAX),
            12 => changed.actual = CoreTermId::new(usize::MAX),
            13 => changed.formal = CoreVarId::new(usize::MAX),
            14 => changed.callee_result = CoreVarId::new(usize::MAX),
            15 => changed.caller_result = CoreVarId::new(usize::MAX),
            16 => {
                std::mem::swap(&mut changed.callee, &mut changed.caller);
                std::mem::swap(&mut changed.callee_result, &mut changed.caller_result);
            }
            _ => unreachable!(),
        }
        let changed_before = changed.clone();
        assert_eq!(describe(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "request {mutation}");
        assert_eq!(canonical_unary_child(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "request {mutation}");
        assert_eq!(canonical_caller_e4(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "request {mutation}");
        assert_eq!(canonical_caller_e1(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "request {mutation}");
        assert_eq!(canonical_caller_typing(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "request {mutation}");
        assert_eq!(generate(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "existential request {mutation}");
        assert_eq!(caller_e4(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "caller E4 request {mutation}");
        assert_eq!(caller_typing(&source, &typed, symbols, &core, &changed).unwrap_err(), caller_e4(&source, &typed, symbols, &core, &changed).unwrap_err(), "caller typing request {mutation}");
        assert_eq!(check_typing(&source, &typed, symbols, &core, &changed).unwrap_err(), caller_typing(&source, &typed, symbols, &core, &changed).unwrap_err(), "caller typing check request {mutation}");
        assert_eq!(TypingAssociation::from_source(&source, &typed, symbols, &core, &changed).err().unwrap(), check_typing(&source, &typed, symbols, &core, &changed).unwrap_err(), "typing association request {mutation}");
        assert_eq!(typing_association.matches_source(&source, &typed, symbols, &core, &changed).unwrap_err(), caller_typing(&source, &typed, symbols, &core, &changed).unwrap_err(), "typing recheck request {mutation}");
        assert_eq!(typing_association.obligation_identity(&source, &typed, symbols, &core, &changed).unwrap_err(), caller_typing(&source, &typed, symbols, &core, &changed).unwrap_err(), "typing recheck request {mutation}");
        assert_eq!(caller_e1(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "caller E1 request {mutation}");
        assert_eq!(inhabit(&source, &typed, symbols, &core, &changed).err().unwrap(), "algorithms.unary_child.invalid_request", "caller E1 check request {mutation}");
        assert_eq!(compose(&source, &typed, symbols, &core, &changed, &retained).err().unwrap(), "algorithms.unary_child.invalid_request", "composition request {mutation}");
        assert_eq!(retain(&source, &typed, symbols, &core, &changed).err().unwrap(), "algorithms.unary_child.invalid_request", "retained request {mutation}");
        assert_eq!(Association::from_source(&source, &typed, symbols, &core, &changed).err().unwrap(), "algorithms.unary_child.invalid_request", "association request {mutation}");
        assert_eq!(CallerAssociation::from_source(&source, &typed, symbols, &core, &changed).err().unwrap(), "algorithms.unary_child.invalid_request", "caller association request {mutation}");
        assert_eq!(InhabitationAssociation::from_source(&source, &typed, symbols, &core, &changed).err().unwrap(), "algorithms.unary_child.invalid_request", "inhabitation association request {mutation}");
        assert_eq!(caller_association.matches_source(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "caller recheck request {mutation}");
        assert_eq!(inhabitation_association.matches_source(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "inhabitation recheck request {mutation}");
        assert_eq!(inhabitation_association.obligation_identity(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "inhabitation recheck request {mutation}");
        assert_eq!(caller_association.obligation_identity(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "caller recheck request {mutation}");
        assert_eq!(consume(&source, &typed, symbols, &core, &changed, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "consumer request {mutation}");
        assert_eq!(consume_caller(&source, &typed, symbols, &core, &changed, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "caller consumer request {mutation}");
        assert_eq!(Native::from_source(&source, &typed, symbols, &core, &changed, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "native request {mutation}");
        assert_eq!(NativeCaller::from_source(&source, &typed, symbols, &core, &changed, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "native request {mutation}");
        assert_eq!(NativeInhabitation::from_source(&source, &typed, symbols, &core, &changed, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "native request {mutation}");
        assert_eq!(native.adopt_original_request(&source, &typed, symbols, &core, &changed, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption request {mutation}");
        assert_eq!(native_caller.adopt_original_request(&source, &typed, symbols, &core, &changed, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption request {mutation}");
        assert_eq!(native_inhabitation.adopt_original_request(&source, &typed, symbols, &core, &changed, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption request {mutation}");
        assert_eq!(native_inhabitation.obligation_identity(&source, &typed, symbols, &core, &changed, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption request {mutation}");
        assert_eq!(native_caller.obligation_identity(&source, &typed, symbols, &core, &changed, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption request {mutation}");
        assert_eq!(native.obligation_identity(&source, &typed, symbols, &core, &changed, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption request {mutation}");
        assert_eq!(association.matches_source(&source, &typed, symbols, &core, &changed).unwrap_err(), "algorithms.unary_child.invalid_request", "recheck request {mutation}");
        assert!(kernel(&core, &changed, target, binder, request.actual, &[actual_guard]).is_none(), "Kernel request {mutation}");
        assert!(compose_kernel(&core, &changed, caller_target, &retained, &[actual_guard]).is_none(), "caller Kernel request {mutation}");
        assert!(!typing_kernel(&core, &changed, target.1, caller_target, binder, conclusion, &[actual_guard]), "typing request {mutation}");
        assert_eq!(changed, changed_before);
    }
    assert_eq!(describe(&source, &typed, symbols, &core, &request).unwrap(), genuine);
    assert!(association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
    assert!(caller_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
    assert!(inhabitation_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
    assert_eq!(inhabitation_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), genuine_e1_identity);
    for mutation in 0..33 {
        let mut p = CoreIrParts {
            source_id: core.source_id(), module_id: core.module_id().clone(),
            items: core.items().clone(), terms: core.terms().clone(), formulas: core.formulas().clone(),
            definitions: core.definitions().clone(), proofs: core.proofs().clone(), proof_nodes: core.proof_nodes().clone(),
            algorithms: core.algorithms().clone(), algorithm_statements: core.algorithm_statements().clone(),
            generated: core.generated().clone(), obligation_seeds: core.obligation_seeds().clone(),
            source_map: core.source_map().clone(), diagnostics: core.diagnostics().clone(),
        };
        match mutation {
            0 => p.formulas.get_mut(formal_guard).unwrap().kind = F::True,
            1 => {
                let F::TypePred { subject, .. } = &mut p.formulas.get_mut(actual_guard).unwrap().kind else { unreachable!() };
                *subject = formal_return;
            }
            2 => {
                let F::TypePred { ty, .. } = &mut p.formulas.get_mut(formal_guard).unwrap().kind else { unreachable!() };
                *ty = "set".into();
            }
            3 => {
                let T::Apply { args, .. } = &mut p.terms.get_mut(request.application).unwrap().kind else { unreachable!() };
                args[0] = formal_return;
            }
            4 => {
                let T::Apply { functor, .. } = &mut p.terms.get_mut(request.application).unwrap().kind else { unreachable!() };
                *functor = caller.symbol.clone();
            }
            5 => p.algorithm_statements.get_mut(request.statement).unwrap().kind = S::Return(Some(request.actual)),
            6 => {
                p.algorithm_statements.get_mut(request.statement).unwrap().owner = request.callee;
                p.algorithm_statements.get_mut(callee.statements[0]).unwrap().owner = request.caller;
                p.algorithms.get_mut(request.callee).unwrap().statements = caller.statements.clone();
                p.algorithms.get_mut(request.caller).unwrap().statements = callee.statements.clone();
            }
            7 => p.algorithms.get_mut(request.callee).unwrap().params[0].var = caller.params[0].var,
            8 => p.algorithms.get_mut(request.callee).unwrap().params[0].ty_guard = Some(actual_guard),
            9 => p.algorithms.get_mut(request.callee).unwrap().item = caller.item,
            10 => p.algorithms.get_mut(request.callee).unwrap().symbol = caller.symbol.clone(),
            11 => {
                p.terms.get_mut(request.actual).unwrap().source = caller.params[0].source.clone();
                p.source_map.term_sources.insert(request.actual, caller.params[0].source.clone());
            }
            12 => {
                let term = p.terms.get_mut(request.application).unwrap();
                term.source.provenance.clear();
                p.source_map.term_sources.insert(request.application, term.source.clone());
            }
            13 => {
                let item = p.items.get_mut(callee.item).unwrap();
                item.source = caller.source.clone();
                p.source_map.item_sources.insert(callee.item, item.source.clone());
            }
            14 => {
                let term = core.terms().get(request.actual).unwrap().clone();
                let id = p.terms.insert(term.clone());
                p.source_map.term_sources.insert(id, term.source);
            }
            15 => p.algorithms.get_mut(request.callee).unwrap().result.as_mut().unwrap().var = request.formal,
            16 => p.terms.get_mut(formal_return).unwrap().kind = T::Var(request.caller_result),
            17 => p.algorithms.get_mut(request.caller).unwrap().declared_terminating = true,
            18 => {
                let formula = core.formulas().get(formal_guard).unwrap().clone();
                let id = p.formulas.insert(formula.clone());
                p.source_map.formula_sources.insert(id, formula.source);
            }
            19 => p.formulas.get_mut(actual_guard).unwrap().kind = F::True,
            20 => {
                let F::TypePred { ty, .. } = &mut p.formulas.get_mut(actual_guard).unwrap().kind else { unreachable!() };
                *ty = "set".into();
            }
            21 => {
                let F::TypePred { subject, .. } = &mut p.formulas.get_mut(formal_guard).unwrap().kind else { unreachable!() };
                *subject = request.actual;
            }
            22 => p.algorithms.get_mut(request.caller).unwrap().params[0].ty_guard = None,
            23 => p.terms.get_mut(request.actual).unwrap().kind = T::Var(request.formal),
            24 => p.formulas.get_mut(actual_guard).unwrap().kind = F::Equals { left: request.actual, right: formal_return },
            25 => p.formulas.get_mut(callee.result.as_ref().unwrap().ty_guard.unwrap()).unwrap().kind = F::True,
            26 => p.formulas.get_mut(caller.result.as_ref().unwrap().ty_guard.unwrap()).unwrap().kind = F::Equals { left: request.actual, right: request.actual },
            27 => p.algorithms.get_mut(request.caller).unwrap().result.as_mut().unwrap().var = caller.params[0].var,
            28 => p.algorithm_statements.get_mut(callee.statements[0]).unwrap().kind = S::Return(Some(request.actual)),
            29 => p.algorithms.get_mut(request.caller).unwrap().statements.push(request.statement),
            30 | 31 => {
                let mut term = core.terms().get(request.actual).unwrap().clone();
                term.kind = T::Var(binder);
                let id = p.terms.insert(term.clone());
                p.source_map.term_sources.insert(id, term.source);
                if mutation == 31 {
                    p.terms.get_mut(id).unwrap().kind = T::Tuple(vec![id]);
                }
            }
            32 => p.algorithm_statements.get_mut(callee.statements[0]).unwrap().kind = S::If {
                condition: formal_guard, then_body: callee.statements.clone(), else_body: vec![],
            },
            _ => unreachable!(),
        }
        let changed = CoreIr::try_new(p).unwrap_or_else(|error| panic!("Core {mutation}: {error}"));
        assert_ne!(changed, core, "Core {mutation} must change");
        let changed_before = changed.clone();
        assert_eq!(describe(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "Core {mutation}");
        assert_eq!(canonical_unary_child(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "Core {mutation}");
        assert_eq!(canonical_caller_e4(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "Core {mutation}");
        assert_eq!(canonical_caller_e1(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "Core {mutation}");
        assert_eq!(canonical_caller_typing(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "Core {mutation}");
        assert_eq!(generate(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "existential Core {mutation}");
        assert_eq!(caller_e4(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "caller E4 Core {mutation}");
        assert_eq!(caller_typing(&source, &typed, symbols, &changed, &request).unwrap_err(), caller_e4(&source, &typed, symbols, &changed, &request).unwrap_err(), "caller typing Core {mutation}");
        assert_eq!(check_typing(&source, &typed, symbols, &changed, &request).unwrap_err(), caller_typing(&source, &typed, symbols, &changed, &request).unwrap_err(), "caller typing check Core {mutation}");
        assert_eq!(TypingAssociation::from_source(&source, &typed, symbols, &changed, &request).err().unwrap(), check_typing(&source, &typed, symbols, &changed, &request).unwrap_err(), "typing association Core {mutation}");
        assert_eq!(typing_association.matches_source(&source, &typed, symbols, &changed, &request).unwrap_err(), caller_typing(&source, &typed, symbols, &changed, &request).unwrap_err(), "typing recheck Core {mutation}");
        assert_eq!(typing_association.obligation_identity(&source, &typed, symbols, &changed, &request).unwrap_err(), caller_typing(&source, &typed, symbols, &changed, &request).unwrap_err(), "typing recheck Core {mutation}");
        assert_eq!(caller_e1(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "caller E1 Core {mutation}");
        assert_eq!(inhabit(&source, &typed, symbols, &changed, &request).err().unwrap(), "algorithms.unary_child.invalid_request", "caller E1 check Core {mutation}");
        assert_eq!(compose(&source, &typed, symbols, &changed, &request, &retained).err().unwrap(), "algorithms.unary_child.invalid_request", "composition Core {mutation}");
        assert_eq!(retain(&source, &typed, symbols, &changed, &request).err().unwrap(), "algorithms.unary_child.invalid_request", "retained Core {mutation}");
        assert_eq!(Association::from_source(&source, &typed, symbols, &changed, &request).err().unwrap(), "algorithms.unary_child.invalid_request", "association Core {mutation}");
        assert_eq!(CallerAssociation::from_source(&source, &typed, symbols, &changed, &request).err().unwrap(), "algorithms.unary_child.invalid_request", "caller association Core {mutation}");
        assert_eq!(InhabitationAssociation::from_source(&source, &typed, symbols, &changed, &request).err().unwrap(), "algorithms.unary_child.invalid_request", "inhabitation association Core {mutation}");
        assert_eq!(caller_association.matches_source(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "caller recheck Core {mutation}");
        assert_eq!(inhabitation_association.matches_source(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "inhabitation recheck Core {mutation}");
        assert_eq!(inhabitation_association.obligation_identity(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "inhabitation recheck Core {mutation}");
        assert_eq!(caller_association.obligation_identity(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "caller recheck Core {mutation}");
        assert_eq!(consume(&source, &typed, symbols, &changed, &request, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "consumer Core {mutation}");
        assert_eq!(consume_caller(&source, &typed, symbols, &changed, &request, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "caller consumer Core {mutation}");
        assert_eq!(Native::from_source(&source, &typed, symbols, &changed, &request, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "native Core {mutation}");
        assert_eq!(NativeCaller::from_source(&source, &typed, symbols, &changed, &request, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "native Core {mutation}");
        assert_eq!(NativeInhabitation::from_source(&source, &typed, symbols, &changed, &request, &denied).err().unwrap(), "algorithms.unary_child.invalid_request", "native Core {mutation}");
        assert_eq!(native.adopt_original_request(&source, &typed, symbols, &changed, &request, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption Core {mutation}");
        assert_eq!(native_caller.adopt_original_request(&source, &typed, symbols, &changed, &request, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption Core {mutation}");
        assert_eq!(native_inhabitation.adopt_original_request(&source, &typed, symbols, &changed, &request, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption Core {mutation}");
        assert_eq!(native_inhabitation.obligation_identity(&source, &typed, symbols, &changed, &request, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption Core {mutation}");
        assert_eq!(native_caller.obligation_identity(&source, &typed, symbols, &changed, &request, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption Core {mutation}");
        assert_eq!(native.obligation_identity(&source, &typed, symbols, &changed, &request, &denied).unwrap_err(), "algorithms.unary_child.invalid_request", "adoption Core {mutation}");
        assert_eq!(association.matches_source(&source, &typed, symbols, &changed, &request).unwrap_err(), "algorithms.unary_child.invalid_request", "recheck Core {mutation}");
        if matches!(mutation, 0..=8 | 10 | 15..=16 | 19..=26) {
            assert!(kernel(&changed, &request, target, binder, request.actual, &[actual_guard]).is_none(), "Kernel Core {mutation}");
        }
        assert_eq!(typing_kernel(&changed, &request, target.1, caller_target, binder, conclusion, &[actual_guard]), matches!(mutation, 9 | 11..=14 | 17..=18), "typing Core {mutation}");
        assert_eq!(changed, changed_before);
    }
    assert!(typing_kernel(&core, &request, target.1, caller_target, binder, conclusion, &[actual_guard]));
    assert_eq!(describe(&source, &typed, symbols, &core, &request).unwrap(), genuine);
    assert!(association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
    assert!(caller_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
    assert!(inhabitation_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
    assert_eq!(inhabitation_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), genuine_e1_identity);
    for (module, edited) in [
        ("foreign", text.to_owned()),
        ("atomic", text.replace("child", "edited")),
        ("atomic", text.replace("let b be object", "let b be set")),
        ("atomic", text.replace("return child(b)", "return child(b,b)")),
        ("atomic", text.replace("return child(b)", "return caller(b)")),
        ("atomic", text.replace("return child(b);", "var c := b; return child(c);")),
    ] {
        let (other_source, nodes, other_collected, _) = source_registration_formula_inputs(&edited, "unary-child", module, 0);
        let other_typed = super::super::source_registration_receiver(&other_source, nodes).unwrap();
        if module == "atomic" && edited.contains("algorithm edited(") {
            let (other_core, other_request) = lower(&check(&other_source, other_collected.env(), &other_typed).unwrap()).unwrap();
            let (other_descriptor, other_child) = retain(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            let other_caller_e4 = caller_e4(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            let other_typing = caller_typing(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            assert_eq!(check_typing(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap(), other_typing);
            assert_eq!(other_typing.0, other_caller_e4.0);
            assert_eq!(other_typing.1[0], other_caller_e4.1[0]);
            let other_typing_association = TypingAssociation::from_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            assert_eq!(other_typing_association.request(), &other_request);
            assert_eq!(other_typing_association.descriptor(), &other_typing);
            assert!(other_typing_association.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            assert!(!typing_association.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            assert!(!other_typing_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
            let other_typing_raw = canonical_caller_typing(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            let other_typing_identity = other_typing_association.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap().unwrap();
            assert_eq!(other_typing_identity.1, other_typing_raw.3);
            assert_ne!(other_typing_identity.0, genuine_typing_identity.0);
            assert_eq!(typing_association.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap(), None);
            assert_eq!(other_typing_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap(), None);
            assert_eq!(caller_typing(&source, &typed, symbols, &other_core, &other_request).unwrap_err(), caller_e4(&source, &typed, symbols, &other_core, &other_request).unwrap_err());
            assert_eq!(check_typing(&source, &typed, symbols, &other_core, &other_request).unwrap_err(), caller_typing(&source, &typed, symbols, &other_core, &other_request).unwrap_err());
            assert_eq!(caller_typing(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err(), caller_e4(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err());
            assert_eq!(check_typing(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err(), caller_typing(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err());
            assert_eq!(TypingAssociation::from_source(&source, &typed, symbols, &other_core, &other_request).err().unwrap(), check_typing(&source, &typed, symbols, &other_core, &other_request).unwrap_err());
            assert_eq!(typing_association.matches_source(&source, &typed, symbols, &other_core, &other_request).unwrap_err(), caller_typing(&source, &typed, symbols, &other_core, &other_request).unwrap_err());
            assert_eq!(typing_association.obligation_identity(&source, &typed, symbols, &other_core, &other_request).unwrap_err(), caller_typing(&source, &typed, symbols, &other_core, &other_request).unwrap_err());
            assert_eq!(canonical_caller_typing(&source, &typed, symbols, &other_core, &other_request).unwrap_err(), caller_typing(&source, &typed, symbols, &other_core, &other_request).unwrap_err());
            assert_eq!(TypingAssociation::from_source(&other_source, &other_typed, other_collected.env(), &core, &request).err().unwrap(), check_typing(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err());
            assert_eq!(other_typing_association.matches_source(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err(), caller_typing(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err());
            assert_eq!(other_typing_association.obligation_identity(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err(), caller_typing(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err());
            assert_eq!(canonical_caller_typing(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err(), caller_typing(&other_source, &other_typed, other_collected.env(), &core, &request).unwrap_err());
            let other_caller_e1 = caller_e1(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            let mut expected_caller_e1 = other_descriptor.clone();
            let mizar_vc::vc_ir::VcGeneratedFormulaShape::AlgorithmReturns { result, .. } = other_descriptor.1[0].shape else {
                panic!("edited child return relation")
            };
            expected_caller_e1.1[0].shape = mizar_vc::vc_ir::VcGeneratedFormulaShape::ProgramTypePredicate { subject: result, ty: "object".into() };
            assert_eq!(other_caller_e1, expected_caller_e1);
            let (other_checked_e1, other_inhabited) = inhabit(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            assert_eq!(other_checked_e1, other_caller_e1);
            assert!(std::ptr::eq(other_inhabited.core(), &other_core));
            assert_eq!(other_inhabited.binder(), result.var);
            assert_eq!(other_inhabited.context(), [other_core.algorithms().get(other_request.caller).unwrap().params[0].ty_guard.unwrap()]);
            assert_eq!(other_inhabited.checker_schema_version(), 1);
            let other_native_inhabitation = NativeInhabitation::from_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap();
            assert!(other_native_inhabitation.adopt_original_request(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap());
            for current_policy in [&policy, &denied] {
                assert!(!native_inhabitation.adopt_original_request(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, current_policy).unwrap());
                assert!(!other_native_inhabitation.adopt_original_request(&source, &typed, symbols, &core, &request, current_policy).unwrap());
                assert_eq!(native_inhabitation.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, current_policy).unwrap(), None);
                assert_eq!(other_native_inhabitation.obligation_identity(&source, &typed, symbols, &core, &request, current_policy).unwrap(), None);
            }
            let other_inhabitation = InhabitationAssociation::from_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            let other_e1_raw = canonical_caller_e1(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            let other_e1_identity = other_inhabitation.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap().unwrap();
            assert_eq!(other_native_inhabitation.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap().unwrap(), other_e1_identity);
            assert_eq!(other_e1_identity.1, other_e1_raw.3);
            assert_ne!(other_e1_identity.0, genuine_e1_identity.0);
            assert_eq!(inhabitation_association.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap(), None);
            assert_eq!(other_inhabitation.obligation_identity(&source, &typed, symbols, &core, &request).unwrap(), None);
            assert!(other_inhabitation.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            assert!(!inhabitation_association.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            assert!(!other_inhabitation.matches_source(&source, &typed, symbols, &core, &request).unwrap());
            assert_eq!(other_inhabitation.request(), &other_request);
            assert_eq!(other_inhabitation.descriptor(), &other_caller_e1);
            assert!(std::ptr::eq(other_inhabitation.result().core(), &other_core));
            assert_eq!(other_inhabitation.result().binder(), other_inhabited.binder());
            assert_eq!(other_inhabitation.result().context(), other_inhabited.context());
            assert_eq!(other_inhabitation.result().checker_schema_version(), other_inhabited.checker_schema_version());
            assert_eq!(other_caller_e4.0, other_descriptor.0);
            let mizar_vc::vc_ir::VcGeneratedFormulaShape::AlgorithmReturns { algorithm, flow, statement, .. } = other_caller_e4.1[0].shape else {
                panic!("edited caller return relation")
            };
            assert_eq!((algorithm, flow, statement), (other_request.caller,
                mizar_core::control_flow::build_control_flow_ir(&other_core).flow_map[&other_request.caller], other_request.statement));
            assert_eq!(other_caller_e4.1[1], other_descriptor.1[1]);
            assert_eq!(compose(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &other_child).unwrap().0, other_caller_e4);
            assert_eq!(compose(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &retained).err().unwrap(), "algorithms.unary_child.invalid_caller_e4_request");
            assert_eq!(compose(&source, &typed, symbols, &core, &request, &other_child).err().unwrap(), "algorithms.unary_child.invalid_caller_e4_request");
            let (other_caller_association, other_caller_fingerprint) = consume_caller(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap();
            assert_eq!(other_caller_fingerprint, policy.policy_fingerprint());
            assert!(!consumed_caller.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            let other_caller_identity = other_caller_association.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap().unwrap();
            assert_ne!(other_caller_identity.0, genuine_caller_identity.0);
            assert_eq!(caller_association.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap(), None);
            assert_eq!(other_caller_association.obligation_identity(&source, &typed, symbols, &core, &request).unwrap(), None);
            assert_eq!(other_caller_association.request(), &other_request);
            assert_eq!(other_caller_association.descriptor(), &other_caller_e4);
            assert!(std::ptr::eq(other_caller_association.result().core(), &other_core));
            assert!(other_caller_association.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            assert!(!caller_association.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            assert!(!other_caller_association.matches_source(&source, &typed, symbols, &core, &request).unwrap());
            let other_association = Association::from_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap();
            assert_eq!(other_association.descriptor(), &other_descriptor);
            assert!(other_association.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            assert!(!association.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            let (other_consumed, other_fingerprint) = consume(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap();
            assert_eq!(other_fingerprint, policy.policy_fingerprint());
            assert_eq!(other_consumed.request(), &other_request);
            assert_eq!(other_consumed.descriptor(), &other_descriptor);
            assert!(std::ptr::eq(other_consumed.result().core(), &other_core));
            assert!(other_consumed.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            assert!(!other_consumed.matches_source(&source, &typed, symbols, &core, &request).unwrap());
            assert!(!consumed.matches_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request).unwrap());
            let other_native_caller = NativeCaller::from_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap();
            assert!(other_native_caller.adopt_original_request(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap());
            assert_eq!(other_native_caller.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap().unwrap(), other_caller_identity);
            for current_policy in [&policy, &denied] {
                assert_eq!(native_caller.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, current_policy).unwrap(), None);
                assert_eq!(other_native_caller.obligation_identity(&source, &typed, symbols, &core, &request, current_policy).unwrap(), None);
            }
            assert!(!native_caller.adopt_original_request(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &denied).unwrap());
            assert!(!other_native_caller.adopt_original_request(&source, &typed, symbols, &core, &request, &denied).unwrap());
            let other_native = Native::from_source(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap();
            assert!(other_native.adopt_original_request(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap());
            assert!(!native.adopt_original_request(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &denied).unwrap());
            assert!(!other_native.adopt_original_request(&source, &typed, symbols, &core, &request, &denied).unwrap());
            let other_identity = other_native.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &policy).unwrap().unwrap();
            assert_ne!(other_identity.0, genuine_identity.0);
            assert_eq!(native.obligation_identity(&other_source, &other_typed, other_collected.env(), &other_core, &other_request, &denied).unwrap(), None);
            assert_eq!(other_native.obligation_identity(&source, &typed, symbols, &core, &request, &denied).unwrap(), None);
        }
        for (s, t, e) in [
            (&other_source, &typed, symbols),
            (&source, &typed, other_collected.env()),
            (&source, &other_typed, symbols),
            (&other_source, &other_typed, other_collected.env()),
        ] {
            let expected = check(s, e, t).and_then(|checked| lower(&checked));
            let error = describe(s, t, e, &core, &request).unwrap_err();
            assert_eq!(error, expected.err().unwrap_or_else(|| "algorithms.unary_child.invalid_request".into()));
            assert_eq!(canonical_unary_child(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(canonical_caller_e4(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(canonical_caller_e1(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(canonical_caller_typing(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(generate(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(caller_e4(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(caller_typing(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(check_typing(s, t, e, &core, &request).unwrap_err(), caller_typing(s, t, e, &core, &request).unwrap_err());
            assert_eq!(TypingAssociation::from_source(s, t, e, &core, &request).err().unwrap(), check_typing(s, t, e, &core, &request).unwrap_err());
            assert_eq!(typing_association.matches_source(s, t, e, &core, &request).unwrap_err(), caller_typing(s, t, e, &core, &request).unwrap_err());
            assert_eq!(typing_association.obligation_identity(s, t, e, &core, &request).unwrap_err(), caller_typing(s, t, e, &core, &request).unwrap_err());
            assert_eq!(caller_e1(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(inhabit(s, t, e, &core, &request).err().unwrap(), error);
            assert_eq!(compose(s, t, e, &core, &request, &cloned_child).err().unwrap(), error);
            assert_eq!(retain(s, t, e, &core, &request).err().unwrap(), error);
            assert_eq!(Association::from_source(s, t, e, &core, &request).err().unwrap(), error);
            assert_eq!(CallerAssociation::from_source(s, t, e, &core, &request).err().unwrap(), error);
            assert_eq!(InhabitationAssociation::from_source(s, t, e, &core, &request).err().unwrap(), error);
            assert_eq!(caller_association.matches_source(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(inhabitation_association.matches_source(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(inhabitation_association.obligation_identity(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(caller_association.obligation_identity(s, t, e, &core, &request).unwrap_err(), error);
            assert_eq!(consume(s, t, e, &core, &request, &denied).err().unwrap(), error);
            assert_eq!(consume_caller(s, t, e, &core, &request, &denied).err().unwrap(), error);
            assert_eq!(Native::from_source(s, t, e, &core, &request, &denied).err().unwrap(), error);
            assert_eq!(NativeCaller::from_source(s, t, e, &core, &request, &denied).err().unwrap(), error);
            assert_eq!(NativeInhabitation::from_source(s, t, e, &core, &request, &denied).err().unwrap(), error);
            assert_eq!(native.adopt_original_request(s, t, e, &core, &request, &denied).unwrap_err(), error);
            assert_eq!(native_caller.adopt_original_request(s, t, e, &core, &request, &denied).unwrap_err(), error);
            assert_eq!(native_inhabitation.adopt_original_request(s, t, e, &core, &request, &denied).unwrap_err(), error);
            assert_eq!(native_inhabitation.obligation_identity(s, t, e, &core, &request, &denied).unwrap_err(), error);
            assert_eq!(native_caller.obligation_identity(s, t, e, &core, &request, &denied).unwrap_err(), error);
            assert_eq!(native.obligation_identity(s, t, e, &core, &request, &denied).unwrap_err(), error);
            assert_eq!(association.matches_source(s, t, e, &core, &request).unwrap_err(), error);
        }
    }
    let mut nodes: Vec<_> = typed.nodes().iter().map(|(_, node)| node.clone()).collect();
    nodes[checked.application().index()].kind = TypedNodeKind::new("ForgedApplication");
    let mut forged = TypedAst::try_new(TypedAstParts {
        source_id: typed.source_id(), module_id: typed.module_id().clone(), resolved_root: typed.resolved_root(),
        source_context: None, source_type: None, source_attribute: None,
        nodes: TypedArena::try_new(typed.nodes().root(), nodes).unwrap(),
        contexts: Default::default(), types: Default::default(), facts: Default::default(),
        coercions: Default::default(), initial_obligations: Default::default(), diagnostics: Default::default(),
    }).unwrap();
    let _ = forged.install_source_nodes(&source);
    assert_eq!(describe(&source, &forged, symbols, &core, &request).unwrap_err(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(canonical_unary_child(&source, &forged, symbols, &core, &request).unwrap_err(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(canonical_caller_e4(&source, &forged, symbols, &core, &request).unwrap_err(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(canonical_caller_e1(&source, &forged, symbols, &core, &request).unwrap_err(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(canonical_caller_typing(&source, &forged, symbols, &core, &request).unwrap_err(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(generate(&source, &forged, symbols, &core, &request).unwrap_err(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(caller_e4(&source, &forged, symbols, &core, &request).unwrap_err(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(caller_typing(&source, &forged, symbols, &core, &request).unwrap_err(), caller_e4(&source, &forged, symbols, &core, &request).unwrap_err());
    assert_eq!(check_typing(&source, &forged, symbols, &core, &request).unwrap_err(), caller_typing(&source, &forged, symbols, &core, &request).unwrap_err());
    assert_eq!(TypingAssociation::from_source(&source, &forged, symbols, &core, &request).err().unwrap(), check_typing(&source, &forged, symbols, &core, &request).unwrap_err());
    assert_eq!(typing_association.matches_source(&source, &forged, symbols, &core, &request).unwrap_err(), caller_typing(&source, &forged, symbols, &core, &request).unwrap_err());
    assert_eq!(typing_association.obligation_identity(&source, &forged, symbols, &core, &request).unwrap_err(), caller_typing(&source, &forged, symbols, &core, &request).unwrap_err());
    assert_eq!(caller_e1(&source, &forged, symbols, &core, &request).unwrap_err(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(inhabit(&source, &forged, symbols, &core, &request).err().unwrap(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(compose(&source, &forged, symbols, &core, &request, &cloned_child).err().unwrap(), check(&source, symbols, &forged).unwrap_err());
    assert_eq!(retain(&source, &forged, symbols, &core, &request).err().unwrap(), check(&source, symbols, &forged).unwrap_err());
    let error = check(&source, symbols, &forged).unwrap_err();
    assert_eq!(Association::from_source(&source, &forged, symbols, &core, &request).err().unwrap(), error);
    assert_eq!(CallerAssociation::from_source(&source, &forged, symbols, &core, &request).err().unwrap(), error);
    assert_eq!(InhabitationAssociation::from_source(&source, &forged, symbols, &core, &request).err().unwrap(), error);
    assert_eq!(caller_association.matches_source(&source, &forged, symbols, &core, &request).unwrap_err(), error);
    assert_eq!(inhabitation_association.matches_source(&source, &forged, symbols, &core, &request).unwrap_err(), error);
    assert_eq!(inhabitation_association.obligation_identity(&source, &forged, symbols, &core, &request).unwrap_err(), error);
    assert_eq!(caller_association.obligation_identity(&source, &forged, symbols, &core, &request).unwrap_err(), error);
    assert_eq!(consume(&source, &forged, symbols, &core, &request, &denied).err().unwrap(), error);
    assert_eq!(consume_caller(&source, &forged, symbols, &core, &request, &denied).err().unwrap(), error);
    assert_eq!(Native::from_source(&source, &forged, symbols, &core, &request, &denied).err().unwrap(), error);
    assert_eq!(NativeCaller::from_source(&source, &forged, symbols, &core, &request, &denied).err().unwrap(), error);
    assert_eq!(NativeInhabitation::from_source(&source, &forged, symbols, &core, &request, &denied).err().unwrap(), error);
    assert_eq!(native.adopt_original_request(&source, &forged, symbols, &core, &request, &denied).unwrap_err(), error);
    assert_eq!(native_caller.adopt_original_request(&source, &forged, symbols, &core, &request, &denied).unwrap_err(), error);
    assert_eq!(native_inhabitation.adopt_original_request(&source, &forged, symbols, &core, &request, &denied).unwrap_err(), error);
    assert_eq!(native_inhabitation.obligation_identity(&source, &forged, symbols, &core, &request, &denied).unwrap_err(), error);
    assert_eq!(native_caller.obligation_identity(&source, &forged, symbols, &core, &request, &denied).unwrap_err(), error);
    assert_eq!(native.obligation_identity(&source, &forged, symbols, &core, &request, &denied).unwrap_err(), error);
    assert_eq!(association.matches_source(&source, &forged, symbols, &core, &request).unwrap_err(), error);
    let mut mismatched = request.clone();
    mismatched.caller = request.callee;
    assert_eq!(caller_e4(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(caller_typing(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(check_typing(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), caller_typing(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err());
    assert_eq!(TypingAssociation::from_source(&source, &forged, symbols, &cloned_core, &mismatched).err().unwrap(), check_typing(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err());
    assert_eq!(typing_association.matches_source(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), caller_typing(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err());
    assert_eq!(typing_association.obligation_identity(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), caller_typing(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err());
    assert_eq!(caller_e1(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(inhabit(&source, &forged, symbols, &cloned_core, &mismatched).err().unwrap(), error);
    assert_eq!(compose(&source, &forged, symbols, &cloned_core, &mismatched, &retained).err().unwrap(), error);
    assert_eq!(retain(&source, &forged, symbols, &cloned_core, &mismatched).err().unwrap(), error);
    assert_eq!(Association::from_source(&source, &forged, symbols, &cloned_core, &mismatched).err().unwrap(), error);
    assert_eq!(CallerAssociation::from_source(&source, &forged, symbols, &cloned_core, &mismatched).err().unwrap(), error);
    assert_eq!(InhabitationAssociation::from_source(&source, &forged, symbols, &cloned_core, &mismatched).err().unwrap(), error);
    assert_eq!(caller_association.matches_source(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(inhabitation_association.matches_source(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(inhabitation_association.obligation_identity(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(caller_association.obligation_identity(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(consume(&source, &forged, symbols, &cloned_core, &mismatched, &denied).err().unwrap(), error);
    assert_eq!(consume_caller(&source, &forged, symbols, &cloned_core, &mismatched, &denied).err().unwrap(), error);
    assert_eq!(Native::from_source(&source, &forged, symbols, &cloned_core, &mismatched, &denied).err().unwrap(), error);
    assert_eq!(NativeCaller::from_source(&source, &forged, symbols, &cloned_core, &mismatched, &denied).err().unwrap(), error);
    assert_eq!(NativeInhabitation::from_source(&source, &forged, symbols, &cloned_core, &mismatched, &denied).err().unwrap(), error);
    assert_eq!(native.adopt_original_request(&source, &forged, symbols, &cloned_core, &mismatched, &denied).unwrap_err(), error);
    assert_eq!(native_caller.adopt_original_request(&source, &forged, symbols, &cloned_core, &mismatched, &denied).unwrap_err(), error);
    assert_eq!(native_inhabitation.adopt_original_request(&source, &forged, symbols, &cloned_core, &mismatched, &denied).unwrap_err(), error);
    assert_eq!(native_inhabitation.obligation_identity(&source, &forged, symbols, &cloned_core, &mismatched, &denied).unwrap_err(), error);
    assert_eq!(native_caller.obligation_identity(&source, &forged, symbols, &cloned_core, &mismatched, &denied).unwrap_err(), error);
    assert_eq!(native.obligation_identity(&source, &forged, symbols, &cloned_core, &mismatched, &denied).unwrap_err(), error);
    assert_eq!(canonical_unary_child(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(canonical_caller_e4(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(canonical_caller_e1(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(canonical_caller_typing(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    assert_eq!(association.matches_source(&source, &forged, symbols, &cloned_core, &mismatched).unwrap_err(), error);
    let (fresh, fresh_fingerprint) = consume(&source, &typed, symbols, &core, &request, &policy).unwrap();
    let fresh_native_inhabitation = NativeInhabitation::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
    for product in [&native_inhabitation, &fresh_native_inhabitation] {
        assert!(product.adopt_original_request(&source, &typed, symbols, &core, &request, &policy).unwrap());
        assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), genuine_e1_identity);
    }
    let fresh_native_caller = NativeCaller::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
    for product in [&native_caller, &fresh_native_caller] {
        assert!(product.adopt_original_request(&source, &typed, symbols, &core, &request, &policy).unwrap());
        assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), genuine_caller_identity);
    }
    let fresh_native = Native::from_source(&source, &typed, symbols, &core, &request, &policy).unwrap();
    for product in [&native, &fresh_native] {
        assert!(product.adopt_original_request(&source, &typed, symbols, &core, &request, &policy).unwrap());
        assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request, &policy).unwrap().unwrap(), genuine_identity);
    }
    assert_eq!(fresh_fingerprint, fingerprint);
    assert_eq!(fingerprint, policy.policy_fingerprint());
    for product in [&association, &consumed, &fresh] {
        assert!(product.matches_source(&source, &typed, symbols, &core, &request).unwrap());
        assert_eq!(product.request(), &request);
        assert_eq!(product.descriptor(), &genuine_existential);
        assert!(std::ptr::eq(product.result().core(), &core));
        assert_eq!(product.result().target(), retained.target());
        assert_eq!(product.result().witness(), retained.witness());
        assert_eq!(product.result().context(), retained.context());
        assert_eq!(product.result().checker_schema_version(), retained.checker_schema_version());
    }
    assert_eq!(describe(&source, &typed, symbols, &core, &request).unwrap(), genuine);
    assert_eq!(generate(&source, &typed, symbols, &core, &request).unwrap(), genuine_existential);
    assert_eq!(caller_e4(&source, &typed, symbols, &core, &request).unwrap(), genuine_caller_e4);
    assert_eq!(caller_typing(&source, &typed, symbols, &core, &request).unwrap(), genuine_typing);
    assert_eq!(check_typing(&source, &typed, symbols, &core, &request).unwrap(), genuine_typing);
    let fresh_typing = TypingAssociation::from_source(&source, &typed, symbols, &core, &request).unwrap();
    assert_eq!(canonical_caller_typing(&source, &typed, symbols, &core, &request).unwrap(), genuine_typing_raw);
    for product in [&typing_association, &fresh_typing] {
        assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), genuine_typing_identity);
        assert!(product.matches_source(&source, &typed, symbols, &core, &request).unwrap());
        assert_eq!(product.request(), &request);
        assert_eq!(product.descriptor(), &genuine_typing);
    }
    assert_eq!(caller_e1(&source, &typed, symbols, &core, &request).unwrap(), genuine_caller_e1);
    let (recovered_e1, recovered_inhabited) = inhabit(&source, &typed, symbols, &core, &request).unwrap();
    assert_eq!(recovered_e1, genuine_caller_e1);
    assert!(std::ptr::eq(recovered_inhabited.core(), &core));
    assert_eq!(recovered_inhabited.binder(), binder);
    assert_eq!(recovered_inhabited.context(), [actual_guard]);
    assert_eq!(recovered_inhabited.checker_schema_version(), 1);
    let fresh_inhabitation = InhabitationAssociation::from_source(&source, &typed, symbols, &core, &request).unwrap();
    assert_eq!(canonical_caller_e1(&source, &typed, symbols, &core, &request).unwrap(), genuine_e1_raw);
    for product in [&inhabitation_association, &fresh_inhabitation] {
        assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), genuine_e1_identity);
        assert!(product.matches_source(&source, &typed, symbols, &core, &request).unwrap());
        assert_eq!(product.request(), &request);
        assert_eq!(product.descriptor(), &genuine_caller_e1);
        assert!(std::ptr::eq(product.result().core(), &core));
        assert_eq!(product.result().binder(), recovered_inhabited.binder());
        assert_eq!(product.result().context(), recovered_inhabited.context());
        assert_eq!(product.result().checker_schema_version(), recovered_inhabited.checker_schema_version());
    }
    assert_eq!(retain(&source, &typed, symbols, &core, &request).unwrap().0, genuine_existential);
    let (recovered_descriptor, recovered) = compose(&source, &typed, symbols, &core, &request, &retained).unwrap();
    assert_eq!(recovered_descriptor, genuine_caller_e4);
    assert_eq!(recovered.target(), composed.target());
    assert_eq!(recovered.witness(), retained.witness());
    assert_eq!(recovered.context(), retained.context());
    assert_eq!(recovered.checker_schema_version(), retained.checker_schema_version());
    let (fresh_caller_association, fresh_caller_fingerprint) = consume_caller(&source, &typed, symbols, &core, &request, &policy).unwrap();
    assert_eq!(fresh_caller_fingerprint, caller_fingerprint);
    assert_eq!(caller_fingerprint, policy.policy_fingerprint());
    assert_eq!(canonical_caller_e4(&source, &typed, symbols, &core, &request).unwrap(), genuine_caller_raw);
    for product in [&caller_association, &consumed_caller, &fresh_caller_association] {
        assert_eq!(product.obligation_identity(&source, &typed, symbols, &core, &request).unwrap().unwrap(), genuine_caller_identity);
        assert!(product.matches_source(&source, &typed, symbols, &core, &request).unwrap());
        assert_eq!(product.request(), &request);
        assert_eq!(product.descriptor(), &genuine_caller_e4);
        assert!(std::ptr::eq(product.result().core(), &core));
        assert_eq!(product.result().target(), composed.target());
        assert_eq!(product.result().witness(), composed.witness());
        assert_eq!(product.result().context(), composed.context());
        assert_eq!(product.result().checker_schema_version(), composed.checker_schema_version());
    }
    assert_eq!((policy, denied), policies_before);
    assert_eq!((source, typed, symbols.clone(), core, request), before);
}

#[test]
fn unary_child_request_keeps_old_lowering_and_direct_return_vc_boundaries() {
    use mizar_proof::status::SourceUnaryCallerE1TypingAssociation as TypingAssociation;
    use mizar_checker::type_checker::{
        check_source_algorithm_types, check_source_unary_child_call,
    };
    use mizar_core::elaborator::{lower_source_algorithms, lower_source_unary_child_call};
    let text = "definition let a be object; terminating algorithm child(a) -> object do return a; end; end; definition let b be object; terminating algorithm caller(b) -> object do return child(b); end; end;";
    let (source, nodes, collected, _) =
        source_registration_formula_inputs(text, "unary-child", "boundaries", 0);
    let typed = super::super::source_registration_receiver(&source, nodes).unwrap();
    let checked = check_source_unary_child_call(&source, collected.env(), &typed).unwrap();
    let (core, request) = lower_source_unary_child_call(&checked).unwrap();
    assert!(mizar_vc::generator::generate_source_unary_child_call_descriptor(
        &source, &typed, collected.env(), &core, &request,
    ).is_ok());
    let (existential, retained) = mizar_vc::generator::check_source_unary_child_call_existential_result(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap();
    let typing_association = TypingAssociation::from_source(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap();
    assert_eq!(typing_association.request(), &request);
    assert_eq!(typing_association.descriptor(), &mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap());
    assert!(typing_association.matches_source(&source, &typed, collected.env(), &core, &request).unwrap());
    let typing_identity = typing_association.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap().unwrap();
    assert_eq!(typing_identity.1, canonical_caller_typing(&source, &typed, collected.env(), &core, &request).unwrap().3);
    assert_eq!(typing_identity.1.kind(), &mizar_vc::vc_ir::VcKind::AlgorithmE1Typing);
    assert!(typing_identity.1.requires_cache_miss());
    assert!(mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
        &source, &typed, collected.env(), &core, &request,
    ).is_ok());
    assert_eq!(mizar_vc::generator::check_source_unary_child_call_caller_e1_typing_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap(), mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap());
    let inhabitation_association = mizar_proof::status::SourceUnaryCallerE1InhabitationAssociation::from_source(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap();
    assert!(inhabitation_association.matches_source(&source, &typed, collected.env(), &core, &request).unwrap());
    let e1_identity = inhabitation_association.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap().unwrap();
    assert_eq!(e1_identity.1, canonical_caller_e1(&source, &typed, collected.env(), &core, &request).unwrap().3);
    assert!(e1_identity.1.requires_cache_miss());
    let caller_association = mizar_proof::status::SourceUnaryCallerE4Association::from_source(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap();
    assert!(caller_association.matches_source(&source, &typed, collected.env(), &core, &request).unwrap());
    assert_ne!(caller_association.result().target(), retained.target());
    let policy = mizar_proof::policy::VerifierPolicy::default();
    let native = mizar_proof::status::NativeUnaryChildCallExistential::from_source(
        &source, &typed, collected.env(), &core, &request, &policy,
    ).unwrap();
    let native_inhabitation = mizar_proof::status::NativeUnaryCallerE1Inhabitation::from_source(
        &source, &typed, collected.env(), &core, &request, &policy,
    ).unwrap();
    assert!(native_inhabitation.adopt_original_request(&source, &typed, collected.env(), &core, &request, &policy).unwrap());
    assert_eq!(native_inhabitation.obligation_identity(&source, &typed, collected.env(), &core, &request, &policy).unwrap().unwrap(), e1_identity);
    let native_caller = mizar_proof::status::NativeUnaryCallerE4Existential::from_source(
        &source, &typed, collected.env(), &core, &request, &policy,
    ).unwrap();
    assert!(native_caller.adopt_original_request(&source, &typed, collected.env(), &core, &request, &policy).unwrap());
    let caller_identity = caller_association.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap().unwrap();
    assert_eq!(native_caller.obligation_identity(&source, &typed, collected.env(), &core, &request, &policy).unwrap().unwrap(), caller_identity);
    assert!(caller_identity.1.requires_cache_miss());
    let (consumed_caller, caller_fingerprint) = mizar_proof::status::consume_source_unary_caller_e4_existential(
        &source, &typed, collected.env(), &core, &request, &policy,
    ).unwrap();
    assert_eq!(caller_fingerprint, policy.policy_fingerprint());
    assert_eq!(consumed_caller.descriptor(), caller_association.descriptor());
    assert_eq!(consumed_caller.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap().unwrap(), caller_identity);
    assert_eq!(mizar_proof::status::consume_source_unary_caller_e4_existential(
        &source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([]),
    ).err().unwrap(), "algorithm.proof.unary_caller_e4_policy_rejected");
    assert_eq!(native.obligation_identity(&source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([])).unwrap_err(), "algorithm.proof.unary_child_policy_rejected");
    assert_eq!(native_caller.obligation_identity(&source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([])).unwrap_err(), "algorithm.proof.unary_caller_e4_policy_rejected");
    assert_eq!(native_inhabitation.obligation_identity(&source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([])).unwrap_err(), "algorithm.proof.native_e1_policy_rejected");
    assert_eq!(caller_association.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap().unwrap(), caller_identity);
    assert_eq!(inhabitation_association.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap().unwrap(), e1_identity);
    assert_eq!(typing_association.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap().unwrap(), typing_identity);
    assert_ne!(typing_identity.0, e1_identity.0);
    assert_ne!(typing_identity.0, caller_identity.0);
    let (consumed, fingerprint) = mizar_proof::status::consume_source_unary_child_call_existential(
        &source, &typed, collected.env(), &core, &request, &policy,
    ).unwrap();
    assert_eq!(consumed.descriptor(), &existential);
    assert_eq!(fingerprint, policy.policy_fingerprint());
    assert_eq!(existential.2, request.actual);
    assert!(core.generated().is_empty() && core.proofs().is_empty() && core.obligation_seeds().is_empty());
    assert!(check_source_algorithm_types(&source, typed.nodes(), collected.env()).is_err());
    assert!(lower_source_algorithms(&checked.algorithms()[1]).is_err());
    for algorithm in checked.algorithms() {
        assert!(
            mizar_vc::generator::generate_source_algorithm_direct_return_requests(
                &source,
                algorithm,
                collected.env(),
                &core,
                super::super::shared::snapshot_id(0),
                &mizar_vc::vc_ir::GenerationSchemaVersion::new("unary-child-boundary"),
                &mizar_vc::vc_ir::VcSchemaVersion::new("unary-child-boundary")
            )
            .is_err()
        );
    }
    let case = step5c14_return_case();
    let control = std::fs::read_to_string(&case.source_path).unwrap();
    let (source, nodes, collected, _) =
        source_registration_formula_inputs(&control, "unary-child", "direct_return_control", 0);
    let typed = super::super::source_registration_receiver(&source, nodes).unwrap();
    let checked = check_source_algorithm_types(&source, typed.nodes(), collected.env()).unwrap();
    let core = lower_source_algorithms(&checked).unwrap();
    let upstream = check_source_unary_child_call(&source, collected.env(), &typed).unwrap_err();
    assert_eq!(TypingAssociation::from_source(
        &source, &typed, collected.env(), &core, &request,
    ).err().unwrap(), upstream);
    assert_eq!(typing_association.matches_source(&source, &typed, collected.env(), &core, &request).unwrap_err(), upstream);
    assert_eq!(mizar_proof::status::SourceUnaryCallerE1InhabitationAssociation::from_source(
        &source, &typed, collected.env(), &core, &request,
    ).err().unwrap(), upstream);
    assert_eq!(inhabitation_association.matches_source(&source, &typed, collected.env(), &core, &request).unwrap_err(), upstream);
    assert_eq!(mizar_proof::status::SourceUnaryCallerE4Association::from_source(
        &source, &typed, collected.env(), &core, &request,
    ).err().unwrap(), upstream);
    assert_eq!(caller_association.matches_source(&source, &typed, collected.env(), &core, &request).unwrap_err(), upstream);
    assert_eq!(mizar_proof::status::NativeUnaryCallerE1Inhabitation::from_source(
        &source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([]),
    ).err().unwrap(), upstream);
    assert_eq!(native_inhabitation.adopt_original_request(&source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([])).unwrap_err(), upstream);
    assert_eq!(native_inhabitation.obligation_identity(&source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([])).unwrap_err(), upstream);
    assert_eq!(mizar_proof::status::NativeUnaryCallerE4Existential::from_source(
        &source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([]),
    ).err().unwrap(), upstream);
    assert_eq!(native_caller.adopt_original_request(&source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([])).unwrap_err(), upstream);
    assert_eq!(native_caller.obligation_identity(&source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([])).unwrap_err(), upstream);
    assert_eq!(canonical_unary_child(&source, &typed, collected.env(), &core, &request).unwrap_err(), upstream);
    assert_eq!(canonical_caller_e4(&source, &typed, collected.env(), &core, &request).unwrap_err(), upstream);
    assert_eq!(canonical_caller_e1(&source, &typed, collected.env(), &core, &request).unwrap_err(), upstream);
    assert_eq!(inhabitation_association.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap_err(), upstream);
    assert_eq!(caller_association.obligation_identity(&source, &typed, collected.env(), &core, &request).unwrap_err(), upstream);
    assert_eq!(native.obligation_identity(&source, &typed, collected.env(), &core, &request, &policy.clone().with_kernel_evidence_formats([])).unwrap_err(), upstream);
    assert_eq!(mizar_vc::generator::generate_source_unary_child_call_existential_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap_err(), upstream);
    assert_eq!(mizar_vc::generator::generate_source_unary_child_call_caller_e4_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap_err(), upstream);
    assert_eq!(mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap_err(), upstream);
    assert_eq!(mizar_vc::generator::check_source_unary_child_call_caller_e1_typing_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap_err(), mizar_vc::generator::generate_source_unary_child_call_caller_e1_typing_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap_err());
    assert_eq!(mizar_vc::generator::generate_source_unary_child_call_caller_e1_inhabitation_request(
        &source, &typed, collected.env(), &core, &request,
    ).unwrap_err(), upstream);
    assert_eq!(mizar_vc::generator::check_source_unary_child_call_caller_e1_inhabitation_result(
        &source, &typed, collected.env(), &core, &request,
    ).err().unwrap(), upstream);
    assert_eq!(mizar_vc::generator::check_source_unary_child_call_caller_e4_result(
        &source, &typed, collected.env(), &core, &request, &retained,
    ).err().unwrap(), upstream);
    assert_eq!(mizar_vc::generator::check_source_unary_child_call_existential_result(
        &source, &typed, collected.env(), &core, &request,
    ).err().unwrap(), upstream);
    assert_eq!(mizar_proof::status::consume_source_unary_child_call_existential(
        &source, &typed, collected.env(), &core, &request,
        &policy.clone().with_kernel_evidence_formats([]),
    ).err().unwrap(), upstream);
    assert_eq!(mizar_proof::status::consume_source_unary_caller_e4_existential(
        &source, &typed, collected.env(), &core, &request,
        &policy.with_kernel_evidence_formats([]),
    ).err().unwrap(), upstream);
    assert!(
        mizar_vc::generator::generate_source_algorithm_direct_return_requests(
            &source,
            &checked,
            collected.env(),
            &core,
            super::super::shared::snapshot_id(0),
            &mizar_vc::vc_ir::GenerationSchemaVersion::new("unary-child-boundary"),
            &mizar_vc::vc_ir::VcSchemaVersion::new("unary-child-boundary")
        )
        .is_ok()
    );
}

#[test]
fn algorithm_ordinary_occurrences_preserve_source_binding_targets() {
    use mizar_checker::binding_env::BindingKind;
    use mizar_checker::type_checker::{TermReference, TermStatus, TypeHeadRef};
    use mizar_checker::typed_ast::TypeEntryActual;
    use mizar_session::SourceAnchor;
    use mizar_syntax::ast::SurfaceNodeKind as K;

    let config = step5c11_config();
    let case = step5c14_return_case();
    for (label, text, declarations, expected_uses, captures) in [
        (
            "shadow_snapshot",
            "definition let a be object; terminating algorithm sample(a) -> object ensures result = a do snapshot before; var a := a; ghost var g := a; const c := a; a := c; assert not a = c; snapshot after; return a; end; end;",
            vec![15, 113, 131, 145],
            vec![
                (57, "header", Some(15)),
                (78, "term", None),
                (87, "term", Some(15)),
                (118, "term", Some(15)),
                (136, "term", Some(113)),
                (150, "term", Some(113)),
                (153, "lvalue", Some(113)),
                (158, "term", Some(145)),
                (172, "term", Some(113)),
                (176, "term", Some(145)),
                (202, "term", Some(113)),
            ],
            vec![("before", vec![15]), ("after", vec![113, 131, 145])],
        ),
        (
            "parameter_result",
            "definition let result be object; algorithm named(result) -> object do var result := result; return result; end; end;",
            vec![15, 74],
            vec![
                (49, "header", Some(15)),
                (84, "term", Some(15)),
                (99, "term", Some(74)),
            ],
            vec![],
        ),
        (
            "local_result",
            "definition let a be object; algorithm ordinary(a) -> object ensures result = a do var result := a; result := a; assert result = a; return result; end; end;",
            vec![15, 86],
            vec![
                (47, "header", Some(15)),
                (68, "term", None),
                (77, "term", Some(15)),
                (96, "term", Some(15)),
                (99, "lvalue", Some(86)),
                (109, "term", Some(15)),
                (119, "term", Some(86)),
                (128, "term", Some(15)),
                (138, "term", Some(86)),
            ],
            vec![],
        ),
    ] {
        let (source, typed, symbols) = super::super::source_registration_inputs(
            &config.workspace_root,
            &case,
            super::super::formula_statement::step5c8_test_frontend(text),
        )
        .unwrap();
        let checked =
            mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols)
                .unwrap();
        let bindings = checked.bindings().bindings();
        assert_eq!(
            bindings
                .iter()
                .filter(|(_, binding)| binding.kind != BindingKind::Generated)
                .map(|(_, binding)| binding.declaration_range.start)
                .collect::<Vec<_>>(),
            declarations,
            "{label} source declarations"
        );
        let generated = bindings
            .iter()
            .filter(|(_, binding)| binding.kind == BindingKind::Generated)
            .collect::<Vec<_>>();
        assert_eq!(generated.len(), usize::from(label != "parameter_result"));
        if let Some((id, _)) = generated.first() {
            assert_eq!(bindings.iter().nth(1).unwrap().0, *id);
        }
        let inference = checked.inference();
        assert!(inference.diagnostics().is_empty());
        assert!(inference.facts().is_empty());
        assert!(inference.candidate_sets().is_empty());
        let mut actual_uses = Vec::new();
        for (_, term) in inference.terms().iter() {
            assert_eq!(term.status, TermStatus::Inferred);
            let Some(TermReference::Binding(binding)) = term.reference else {
                panic!("{label} missing binding")
            };
            let node = typed.node(term.site.node()).unwrap();
            let SourceAnchor::Range(range) = node.anchor else {
                panic!("{label} missing source range")
            };
            let role = match source
                .arena()
                .node(node.resolved_node.unwrap())
                .unwrap()
                .kind()
            {
                K::Token(_) => "header",
                K::Lvalue => "lvalue",
                K::TermReference => "term",
                _ => panic!("{label} unexpected ordinary use owner"),
            };
            let target = bindings.get(binding).unwrap();
            let declaration =
                (target.kind != BindingKind::Generated).then_some(target.declaration_range.start);
            actual_uses.push((range.start, role, declaration));
            let TypeEntryActual::Known(ty) = inference
                .type_entries()
                .get(term.type_entry)
                .unwrap()
                .actual
            else {
                panic!("{label} unknown type")
            };
            assert_eq!(
                inference.normalized_types().get(ty).unwrap().head,
                TypeHeadRef::BuiltinObject
            );
        }
        actual_uses.sort_by_key(|(start, _, _)| *start);
        assert_eq!(actual_uses, expected_uses, "{label} source use targets");
        assert_eq!(
            checked
                .snapshots()
                .values()
                .map(|(name, captures)| (
                    name.as_str(),
                    captures
                        .iter()
                        .map(|id| bindings.get(*id).unwrap().declaration_range.start)
                        .collect::<Vec<_>>()
                ))
                .collect::<Vec<_>>(),
            captures,
            "{label} source snapshot targets"
        );
        assert_eq!(checked.typed(), &typed);
        let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
        assert_eq!(core, step5c14_static_core(&case, text).unwrap());
        eprintln!(
            "ALGORITHM_OCCURRENCE_BASELINE:{label}:{:?}",
            (&checked, &core)
        );
    }
}

#[test]
fn algorithm_formula_rhs_preserves_direct_payloads_terms_and_statement_boundaries() {
    use mizar_syntax::{SurfaceNodeKind as K, SyntaxKind as S, SurfaceFormulaPrefixOperator as P, SurfaceQuantifierKind as Q};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_algorithm_formula_rhs_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 137);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 137));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    for (kind, syntax, count) in [(K::VariableBinding, S::VariableBinding, 17), (K::AssignmentStatement, S::AssignmentStatement, 3), (K::ReturnStatement, S::ReturnStatement, 2)] {
        assert_eq!(ast.node_views().filter(|v| v.kind() == &kind).count(), count);
        assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == syntax).count(), count);
        assert_eq!(ast.snapshot_text().matches(&format!("{kind:?}")).count(), count);
    }
    for (marker, owner_kind, rhs, root_kind) in [
        ("eq := x = y,", K::VariableBinding, "x = y", K::BuiltinPredicateApplication),
        ("membership :=", K::VariableBinding, "x is object", K::IsAssertion),
        ("negative := not", K::VariableBinding, "not (x = y)", K::PrefixFormula(P::Not)),
        ("impossible :=", K::VariableBinding, "contradiction", K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Contradiction)),
        ("eq := P(x)", K::AssignmentStatement, "P(x) & Q(x)", K::BinaryFormula(mizar_syntax::SurfaceFormulaBinaryOperator { connective: mizar_syntax::SurfaceFormulaConnective::And, repeated: false })),
        ("ghost negative :=", K::AssignmentStatement, "((x = y))", K::ParenthesizedFormula),
        ("quantified :=", K::VariableBinding, "for a,b being object holds a = b", K::QuantifiedFormula(Q::Universal)),
        ("exists :=", K::VariableBinding, "ex a being object st a = x", K::QuantifiedFormula(Q::Existential)),
        ("after_binder :=", K::VariableBinding, "x = y", K::BuiltinPredicateApplication),
        ("eq := (x,y)", K::AssignmentStatement, "(x,y) |-> x = y", K::BuiltinPredicateApplication),
        ("formula_after :=", K::VariableBinding, "x = y", K::BuiltinPredicateApplication),
        ("compared :=", K::VariableBinding, "{x where x is set : x = x} = z", K::BuiltinPredicateApplication),
        ("return x = y by", K::ReturnStatement, "x = y", K::BuiltinPredicateApplication),
        ("return x = x proof", K::ReturnStatement, "x = x", K::BuiltinPredicateApplication),
    ] {
        let start = text.find(marker).unwrap();
        let rhs_start = start + text[start..].find(rhs).unwrap();
        let owner = ast.node_views().find(|v| v.kind() == &owner_kind && v.range().start == start).unwrap();
        let payloads = owner.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| matches!(v.kind(), K::FormulaExpression | K::TermExpression)).collect::<Vec<_>>();
        assert_eq!(payloads.len(), 1, "{marker}");
        let rhs_view = payloads[0];
        assert_eq!(rhs_view.kind(), &K::FormulaExpression, "{marker}");
        assert_eq!(rhs_view.range(), range(ast.source_id, rhs_start, rhs_start + rhs.len()));
        assert_eq!(rhs_view.children().len(), 1);
        let root = ast.node_view(rhs_view.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind, "{marker}");
        assert_eq!(root.range(), rhs_view.range());
        if root_kind == K::BuiltinPredicateApplication {
            assert_eq!(root.children().len(), 3);
            let left = ast.node_view(root.children()[0]).unwrap();
            assert_eq!(left.kind(), &K::TermExpression);
            assert_eq!(ast.node_view(root.children()[1]).unwrap().as_token().unwrap().text.as_ref(), "=");
            assert_eq!(ast.node_view(root.children()[2]).unwrap().kind(), &K::TermExpression);
            if rhs.starts_with("(x,y)") || rhs.starts_with('{') {
                let term = ast.node_view(left.children()[0]).unwrap();
                assert_eq!(term.kind(), if rhs.starts_with('(') { &K::BinderExpression } else { &K::SetComprehension });
                assert_eq!(&text[left.range().start..left.range().end], if rhs.starts_with('(') { "(x,y) |-> x" } else { "{x where x is set : x = x}" });
            }
        }
        if matches!(root_kind, K::BinaryFormula(_)) {
            assert_eq!(root.children().len(), 3);
            assert_eq!(ast.node_view(root.children()[0]).unwrap().kind(), &K::InlinePredicateApplication);
            assert_eq!(ast.node_view(root.children()[1]).unwrap().as_token().unwrap().text.as_ref(), "&");
            assert_eq!(ast.node_view(root.children()[2]).unwrap().kind(), &K::InlinePredicateApplication);
        }
        if root_kind == K::PrefixFormula(P::Not) {
            assert_eq!(root.children().len(), 2);
            assert_eq!(ast.node_view(root.children()[1]).unwrap().kind(), &K::ParenthesizedFormula);
        }
        if root_kind == K::ParenthesizedFormula {
            assert_eq!(ast.node_views().filter(|v| v.kind() == &K::ParenthesizedFormula && v.range().start >= rhs_start && v.range().end <= rhs_start + rhs.len()).count(), 2);
        }
        if matches!(root_kind, K::QuantifiedFormula(_)) {
            let segment = ast.node_view(root.children()[1]).unwrap();
            assert_eq!(segment.kind(), &K::QuantifierVariableSegment);
            assert_eq!(&text[segment.range().start..segment.range().end], if rhs.starts_with("for") { "a,b being object" } else { "a being object" });
        }
        let mut pending = vec![owner.id()];
        let mut tokens = Vec::new();
        let mut seen = std::collections::HashSet::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate RHS owner child: {marker}");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().filter(|v| v.range().start >= owner.range().start && v.range().end <= owner.range().end).map(|v| v.id()).collect::<Vec<_>>());
    }
    for (name, rhs, root_kind) in [
        ("term_tail", "z", K::TermReference), ("ambiguous", "P", K::TermReference), ("call", "P(x)", K::ApplicationTerm),
        ("binder", "x |-> x", K::BinderExpression), ("opaque", "{x where x is set : x = x}", K::SetComprehension),
        ("nested", "f({x where x is set : x = x})", K::ApplicationTerm), ("bracket", "[{x where x is set : x = x}, y]", K::ApplicationTerm), ("after_returns", "z", K::TermReference),
    ] {
        let marker = format!("{name} := ");
        let start = text.find(&marker).unwrap();
        let owner = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(owner.children().len(), 3);
        let payload = ast.node_view(owner.children()[2]).unwrap();
        assert_eq!(payload.kind(), &K::TermExpression, "{name}");
        assert_eq!(payload.range(), range(ast.source_id, start + marker.len(), start + marker.len() + rhs.len()));
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &root_kind);
        if rhs.contains('{') {
            assert!(ast.node_views().any(|v| v.kind() == &K::FormulaExpression && v.range().start > payload.range().start && v.range().end < payload.range().end));
        }
    }
    for (marker, expected) in [
        ("var eq :=", ["var", "eq := x = y", ",", "term_tail := z", "as", "object", "by TypeOk", ";"].as_slice()),
        ("var binder :=", &["var", "binder := x |-> x", ",", "after_binder := x = y", ";"]),
        ("var opaque :=", &["var", "opaque := {x where x is set : x = x}", ",", "formula_after := x = y", ";"]),
        ("return x = y by", &["return", "x = y", "by ReturnOk", ";"]),
        ("return x = x proof", &["return", "x = x", "proof thus thesis; end", ";"]),
    ] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| matches!(v.kind(), K::VariableDeclaration | K::ReturnStatement) && v.range().start == start).unwrap();
        assert_eq!(owner.children().iter().map(|id| { let child = ast.node_view(*id).unwrap(); &text[child.range().start..child.range().end] }).collect::<Vec<_>>(), expected);
    }
    let body = ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList).unwrap();
    assert_eq!(body.children().iter().map(|id| { let child = ast.node_view(*id).unwrap(); &text[child.range().start..child.range().end] }).collect::<Vec<_>>(), text.lines().skip(2).take(18).map(str::trim).collect::<Vec<_>>());
    let proof_start = text.find("proof thus thesis; end").unwrap();
    assert!(ast.node_views().any(|v| v.kind() == &K::ProofBlock && v.range() == range(ast.source_id, proof_start, proof_start + "proof thus thesis; end".len())));
    let final_start = text.find("set after_formula_rhs = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len())));
    let frontend = task257c4c1_frontend_output(&text, 137);
    assert!(frontend.diagnostics.is_empty());
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    let repeated = task257c4c1_frontend_output(&text, 137);
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v24");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn algorithm_formula_rhs_recovery_and_thesis_restrictions_preserve_owned_payloads() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceFormulaPrefixOperator as P, SurfaceQuantifierKind as Q};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_algorithm_formula_rhs_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 138);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 138));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 7, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    for (diagnostic, (code, offset, word, message)) in output.diagnostics.iter().zip([
        ("malformed_term_expression", 60, ";", "expected term after built-in predicate"),
        ("malformed_formula_expression", 104, ";", "expected formula after `not`"),
        ("malformed_formula_expression", 150, ";", "expected `)` to close formula delimiter"),
        ("malformed_formula_expression", 198, ",", "expected formula after `not`"),
        ("malformed_formula_expression", 254, "thesis", "`thesis` is restricted to explicit computation inputs"),
        ("malformed_formula_expression", 334, "thesis", "`thesis` is restricted to explicit computation inputs"),
        ("malformed_term_expression", 391, "thesis", "unexpected token in return statement"),
    ]) {
        assert_eq!(&text[offset..offset + word.len()], word);
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax(code.into()));
        assert_eq!(diagnostic.message.as_ref(), message);
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, offset, offset + word.len())));
        if offset == 150 {
            let opener = text.find("return (x = y").unwrap() + "return ".len();
            assert_eq!(diagnostic.secondary, [mizar_session::SourceAnchor::Range(range(ast.source_id, opener, opener + 1))]);
            assert_eq!(diagnostic.recovery_note.as_deref(), Some("insert `)` before continuing"));
        } else {
            assert!(diagnostic.secondary.is_empty());
            assert_eq!(diagnostic.recovery_note.as_deref(), Some(if code == "malformed_term_expression" { "repair the term expression before continuing" } else { "repair the formula expression before continuing" }));
        }
    }
    for (kind, count) in [(SyntaxRecoveryKind::MissingTerm, 1), (SyntaxRecoveryKind::MissingFormula, 2), (SyntaxRecoveryKind::UnmatchedOpeningDelimiter, 1), (SyntaxRecoveryKind::SkippedToken, 1)] {
        assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(kind)).count(), count);
    }
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 5);
    for (marker, owner_kind, rhs, root_kind, anchor, recovery_kind) in [
        ("eq := x =", K::VariableBinding, "x =", K::BuiltinPredicateApplication, 60, Some(SyntaxRecoveryKind::MissingTerm)),
        ("x := not ;", K::AssignmentStatement, "not", K::PrefixFormula(P::Not), 104, Some(SyntaxRecoveryKind::MissingFormula)),
        ("return (x = y", K::ReturnStatement, "(x = y", K::ParenthesizedFormula, 150, Some(SyntaxRecoveryKind::UnmatchedOpeningDelimiter)),
        ("gaps := not", K::VariableBinding, "not", K::PrefixFormula(P::Not), 198, Some(SyntaxRecoveryKind::MissingFormula)),
        ("x := not thesis", K::AssignmentStatement, "not thesis", K::PrefixFormula(P::Not), 254, None),
        ("return for a,b", K::ReturnStatement, "for a,b being object holds thesis", K::QuantifiedFormula(Q::Universal), 334, None),
    ] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| v.kind() == &owner_kind && v.range().start == start).unwrap();
        let payloads = owner.children().iter().map(|id| ast.node_view(*id).unwrap()).filter(|v| matches!(v.kind(), K::TermExpression | K::FormulaExpression)).collect::<Vec<_>>();
        assert_eq!(payloads.len(), 1);
        let payload = payloads[0];
        let rhs_start = start + text[start..].find(rhs).unwrap();
        assert_eq!(payload.kind(), &K::FormulaExpression);
        assert_eq!(payload.range(), range(ast.source_id, rhs_start, rhs_start + rhs.len()));
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind);
        if let Some(kind) = recovery_kind {
            let recovery = ast.node_view(*root.children().last().unwrap()).unwrap();
            assert_eq!(recovery.as_recovery(), Some(kind));
            assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
            assert!(recovery.children().is_empty());
        } else {
            assert!(!owner.is_recovered());
            assert!(ast.node_views().filter(|v| v.range().start >= payload.range().start && v.range().end <= payload.range().end).all(|v| v.as_recovery().is_none()));
            let thesis = ast.node_views().find(|v| v.kind() == &K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Thesis) && v.range().start == anchor).unwrap();
            assert_eq!(thesis.range(), range(ast.source_id, anchor, anchor + "thesis".len()));
        }
    }
    let direct_start = text.find("return thesis;").unwrap();
    let direct = ast.node_views().filter_map(|v| v.as_return_statement()).find(|v| v.range().start == direct_start).unwrap();
    assert_eq!(direct.children().len(), 3);
    let skipped = ast.node_view(direct.children()[1]).unwrap();
    assert_eq!(skipped.as_recovery(), Some(SyntaxRecoveryKind::SkippedToken));
    assert_eq!(skipped.range(), range(ast.source_id, 391, 397));
    assert_eq!(skipped.children().len(), 1);
    assert_eq!(ast.node_view(skipped.children()[0]).unwrap().as_token().unwrap().text.as_ref(), "thesis");
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 1);
    assert_eq!(ast.trivia().skipped_token_ranges()[0].range, skipped.range());
    for name in ["retained", "after_equality", "after_prefix", "after_group", "after_comma", "after_nested_thesis", "after_quantified_thesis", "after_direct_thesis"] {
        let spelling = format!("{name} := x");
        let start = text.find(&spelling).unwrap();
        let binding = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(binding.range(), range(ast.source_id, start, start + spelling.len()));
        assert!(!binding.is_recovered());
        assert_eq!(binding.children().len(), 3);
        let payload = ast.node_view(binding.children()[2]).unwrap();
        assert_eq!(payload.kind(), &K::TermExpression);
        assert_eq!(payload.range(), range(ast.source_id, start + spelling.len() - 1, start + spelling.len()));
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &K::TermReference);
    }
    let gaps_start = text.find("var gaps :=").unwrap();
    let gaps = ast.node_views().find(|v| v.kind() == &K::VariableDeclaration && v.range().start == gaps_start).unwrap();
    assert_eq!(gaps.children().iter().map(|id| { let v = ast.node_view(*id).unwrap(); &text[v.range().start..v.range().end] }).collect::<Vec<_>>(), ["var", "gaps := not", ",", "retained := x", ";"]);
    for owner in ast.node_views().filter(|v| matches!(v.kind(), K::VariableDeclaration | K::AssignmentStatement | K::ReturnStatement)) {
        let last = ast.node_view(*owner.children().last().unwrap()).unwrap();
        assert_eq!(last.as_token().unwrap().text.as_ref(), ";");
        assert_eq!(last.range().end, owner.range().end);
    }
    let final_start = text.find("set after_formula_rhs_failure = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len()) && !v.is_recovered()));
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn algorithm_formula_rhs_has_clean_syntax_but_no_bounded_semantic_admission() {
    use mizar_resolve::names::{SourceVariableScopeError, SourceVariableScopeResolver};
    use mizar_syntax::SurfaceNodeKind as K;

    let config = step5c11_config();
    let case = step5c14_state_case();
    let base = concat!(
        "definition\n  let x be object;\n",
        "  algorithm rhs(x) -> object\n  do\n",
        "    var local := x;\n    local := x;\n    return x;\n",
        "  end;\nend;\n",
    );
    for (selected, text) in [
        (None, base.to_owned()),
        (Some(0), base.replace("var local := x;", "var local := x = x;")),
        (Some(1), base.replace("\n    local := x;", "\n    local := x = x;")),
        (Some(2), base.replace("return x;", "return x = x;")),
    ] {
        let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(frontend.diagnostics.is_empty(), "{selected:?}: {:?}", frontend.diagnostics);
        let ast = frontend.ast.as_ref().expect("clean source must retain AST");
        assert!(ast.node_views().all(|node| !node.is_recovered() && node.as_recovery().is_none()));
        for (index, kind) in [K::VariableBinding, K::AssignmentStatement, K::ReturnStatement]
            .into_iter().enumerate()
        {
            let owner = ast.node_views().find(|node| node.kind() == &kind).unwrap();
            let rhs = owner.child_views().filter(|child| {
                matches!(child.kind(), K::TermExpression | K::FormulaExpression)
            }).collect::<Vec<_>>();
            assert_eq!(rhs.len(), 1, "{selected:?}: {kind:?}");
            assert_eq!(
                rhs[0].kind(),
                if selected == Some(index) { &K::FormulaExpression } else { &K::TermExpression },
                "{selected:?}: {kind:?}",
            );
            assert_eq!(
                &text[rhs[0].range().start..rhs[0].range().end],
                if selected == Some(index) { "x = x" } else { "x" },
            );
        }
        let (source, typed, symbols) = super::super::source_registration_inputs(
            &config.workspace_root, &case, frontend,
        ).unwrap();
        let scope = SourceVariableScopeResolver::resolve_algorithm_occurrences(&source, &symbols);
        let checked = mizar_checker::type_checker::check_source_algorithm_types(
            &source, &typed, &symbols,
        );
        if selected.is_some() {
            assert_eq!(scope.unwrap_err(), SourceVariableScopeError::InvalidShape);
            assert_eq!(checked.unwrap_err(), "algorithms.unsupported_source_types");
        } else {
            scope.unwrap();
            let checked = checked.unwrap();
            let core = mizar_core::elaborator::lower_source_algorithms(&checked).unwrap();
            assert_eq!(core.algorithms().len(), 1);
        }
    }
}

#[test]
fn algorithm_boolean_operands_preserve_references_predicates_and_declaration_tails() {
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K, SyntaxKind as S};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "pass_parser_algorithm_boolean_operands_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 161);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 161));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered() && v.as_recovery().is_none()));
    assert!(ast.trivia().skipped_token_ranges().is_empty());
    for (kind, syntax, count) in [(K::VariableDeclaration, S::VariableDeclaration, 31), (K::VariableBinding, S::VariableBinding, 38), (K::AssignmentStatement, S::AssignmentStatement, 2), (K::ReturnStatement, S::ReturnStatement, 2)] {
        assert_eq!(ast.node_views().filter(|v| v.kind() == &kind).count(), count);
        assert_eq!(ast.rowan_root().descendants().filter(|v| v.kind() == syntax).count(), count);
        assert_eq!(ast.snapshot_text().matches(&format!("{kind:?}")).count(), count);
    }
    let and = K::BinaryFormula(B { connective: C::And, repeated: false });
    for (marker, owner_kind, rhs, root_kind, references) in [
        ("bare := p & q;", K::VariableBinding, "p & q", and.clone(), ["p", "q"].as_slice()),
        ("negative :=", K::VariableBinding, "not p", K::PrefixFormula(P::Not), &["p"]),
        ("repeated := p", K::VariableBinding, "p & p", and.clone(), &["p", "p"]),
        ("grouped :=", K::VariableBinding, "(p) & ((q))", and.clone(), &["p", "q"]),
        ("bare := p & (", K::AssignmentStatement, "p & (q or not r)", and.clone(), &["p", "q", "r"]),
        ("ghost repeated :=", K::AssignmentStatement, "not (p & q)", K::PrefixFormula(P::Not), &["p", "q"]),
        ("mixed :=", K::VariableBinding, "p & x = y", and.clone(), &["p"]),
        ("inline :=", K::VariableBinding, "P(x) & q", and.clone(), &["q"]),
        ("ready :=", K::VariableBinding, "p & local_ready", and.clone(), &["p"]),
        ("typed :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("typed_group :=", K::VariableBinding, "not (q)", K::PrefixFormula(P::Not), &["q"]),
        ("init_tail :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("as_tail :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("empty_tail :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("many_tail :=", K::VariableBinding, "p & q", and.clone(), &["p", "q"]),
        ("last_many :=", K::VariableBinding, "not r", K::PrefixFormula(P::Not), &["r"]),
        ("group_tail :=", K::VariableBinding, "not (q)", K::PrefixFormula(P::Not), &["q"]),
        ("nested_tail :=", K::VariableBinding, "not ((q))", K::PrefixFormula(P::Not), &["q"]),
        ("return p & q by", K::ReturnStatement, "p & q", and.clone(), &["p", "q"]),
        ("return not (q)", K::ReturnStatement, "not (q)", K::PrefixFormula(P::Not), &["q"]),
        ("after_returns :=", K::VariableBinding, "not p", K::PrefixFormula(P::Not), &["p"]),
    ] {
        let start = text.find(marker).unwrap();
        let rhs_start = start + text[start..].find(rhs).unwrap();
        let owner = ast.node_views().find(|v| v.kind() == &owner_kind && v.range().start == start).unwrap();
        let payloads = owner.child_views().filter(|v| matches!(v.kind(), K::FormulaExpression | K::TermExpression)).collect::<Vec<_>>();
        assert_eq!(payloads.len(), 1, "{marker}");
        let payload = payloads[0];
        assert_eq!(payload.kind(), &K::FormulaExpression, "{marker}");
        assert_eq!(payload.range(), range(ast.source_id, rhs_start, rhs_start + rhs.len()));
        assert_eq!(payload.children().len(), 1);
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind, "{marker}");
        assert_eq!(root.range(), payload.range());
        let mut found = Vec::new();
        let mut pending = vec![payload.id()];
        while let Some(id) = pending.pop() {
            let parent = ast.node_view(id).unwrap();
            for child in parent.child_views() {
                if child.kind() == &K::TermExpression && matches!(parent.kind(), K::FormulaExpression | K::BinaryFormula(_) | K::PrefixFormula(_)) {
                    assert_eq!(child.children().len(), 1);
                    let reference = ast.node_view(child.children()[0]).unwrap();
                    assert_eq!(reference.kind(), &K::TermReference, "{marker}");
                    assert_eq!(reference.range(), child.range());
                    assert_eq!(reference.children().len(), 1);
                    let token = ast.node_view(reference.children()[0]).unwrap();
                    assert_eq!(token.range(), child.range());
                    assert_eq!(token.as_token().unwrap().text.as_ref(), &text[child.range().start..child.range().end]);
                    found.push((child.range().start, token.as_token().unwrap().text.as_ref()));
                }
            }
            pending.extend(parent.children().iter().rev().copied());
        }
        found.sort_by_key(|(start, _)| *start);
        assert_eq!(found.iter().map(|(_, word)| *word).collect::<Vec<_>>(), references, "{marker}");
        if root_kind == and {
            let (left, right) = rhs.split_once(" & ").unwrap();
            assert_eq!(root.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), [left, "&", right]);
        } else {
            assert_eq!(root.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["not", &rhs[4..]]);
        }
        if rhs == "p & (q or not r)" {
            let group = ast.node_view(root.children()[2]).unwrap();
            assert_eq!(group.kind(), &K::ParenthesizedFormula);
            let expression = ast.node_view(group.children()[1]).unwrap();
            let nested = ast.node_view(expression.children()[0]).unwrap();
            assert_eq!(nested.kind(), &K::BinaryFormula(B { connective: C::Or, repeated: false }));
            assert_eq!(nested.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["q", "or", "not r"]);
        }
    }
    for (name, rhs, root_kind) in [("term", "p", K::TermReference), ("term_group", "(p)", K::ParenthesizedTerm), ("call", "P(x)", K::ApplicationTerm), ("binder", "x |-> x", K::BinderExpression), ("next_init", "x = y", K::BuiltinPredicateApplication), ("next_group", "x = y", K::BuiltinPredicateApplication)] {
        let marker = format!("{name} := ");
        let start = text.find(&marker).unwrap();
        let owner = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(owner.children().len(), 3);
        let payload = ast.node_view(owner.children()[2]).unwrap();
        assert_eq!(payload.kind(), if name.starts_with("next_") { &K::FormulaExpression } else { &K::TermExpression });
        assert_eq!(payload.range(), range(ast.source_id, start + marker.len(), start + marker.len() + rhs.len()));
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &root_kind);
    }
    for (name, segment_text, argument_kind) in [
        ("pair", "q,r local_pair", K::TermReference), ("trip", "q,r,s local_trip", K::TermReference),
        ("grouped_pair", "(q),r local_pair", K::ParenthesizedTerm), ("grouped_trip", "((q)),r,s local_trip", K::ParenthesizedTerm),
        ("negated", "q,r do not local_pair", K::TermReference), ("complex", "q,f(r) local_pair", K::ApplicationTerm),
        ("container", "q,[r,s] local_pair", K::ApplicationTerm), ("binder_arg", "q,(r,s) |-> r local_pair", K::BinderExpression),
    ] {
        let start = text.find(&format!("{name} := not ")).unwrap() + name.len() + " := not ".len();
        let segment = ast.node_views().find(|v| v.kind() == &K::PredicateSegment && v.range() == range(ast.source_id, start, start + segment_text.len())).unwrap();
        let head = segment.child_views().find(|v| v.kind() == &K::PredicateHead).unwrap();
        assert_eq!(&text[head.range().start..head.range().end], if name.ends_with("trip") { "local_trip" } else { "local_pair" });
        let terms = segment.child_views().filter(|v| v.kind() == &K::TermExpression).collect::<Vec<_>>();
        assert_eq!(terms.len(), if name.ends_with("trip") { 3 } else { 2 });
        let argument = if name.starts_with("grouped_") { terms[0] } else { *terms.last().unwrap() };
        assert_eq!(ast.node_view(argument.children()[0]).unwrap().kind(), &argument_kind);
        assert_eq!(segment.child_views().filter_map(|v| v.as_token().map(|t| t.text.as_ref())).collect::<Vec<_>>(), if name == "negated" { vec![",", "do", "not"] } else if name.ends_with("trip") { vec![",", ","] } else { vec![","] });
        let token = ast.token_views().find(|v| v.range() == head.range()).unwrap().as_token().unwrap();
        assert_eq!(token.kind, mizar_syntax::SurfaceTokenKind::UserSymbol);
    }
    for name in ["atomic", "nested_atomic", "binder_atom"] {
        let start = text.find(&format!("{name} := not ")).unwrap() + name.len() + " := not ".len();
        let atom = ast.node_views().find(|v| v.kind() == &K::BuiltinPredicateApplication && v.range().start == start).unwrap();
        let left = ast.node_view(atom.children()[0]).unwrap();
        assert_eq!(left.kind(), &K::TermExpression);
        assert_eq!(ast.node_view(left.children()[0]).unwrap().kind(), &K::ParenthesizedTerm);
    }
    for (marker, expected) in [
        ("var typed :=", ["var", "typed := p & q", "as", "expr of boolean", "by TypeOk", ";"].as_slice()),
        ("var typed_group :=", &["var", "typed_group := not (q)", "as", "expr of boolean", "proof thus thesis; end", ";"]),
        ("var init_tail :=", &["var", "init_tail := p & q", ",", "next_init := x = y", ";"]),
        ("var as_tail :=", &["var", "as_tail := p & q", ",", "next_as", "as", "expr of boolean", "proof thus thesis; end", ";"]),
        ("var empty_tail :=", &["var", "empty_tail := p & q", ",", "next_empty", ";"]),
        ("var many_tail :=", &["var", "many_tail := p & q", ",", "next_many", ",", "last_many := not r", ";"]),
        ("var group_tail :=", &["var", "group_tail := not (q)", ",", "next_group := x = y", ";"]),
        ("var nested_tail :=", &["var", "nested_tail := not ((q))", ",", "next_nested", "as", "expr of boolean", ";"]),
        ("return p & q by", &["return", "p & q", "by ReturnOk", ";"]),
        ("return not (q)", &["return", "not (q)", "proof thus thesis; end", ";"]),
    ] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| matches!(v.kind(), K::VariableDeclaration | K::ReturnStatement) && v.range().start == start).unwrap();
        assert_eq!(owner.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), expected, "{marker}");
        if expected.contains(&"as") {
            let parts = owner.child_views().collect::<Vec<_>>();
            let as_index = parts.iter().position(|v| v.as_token().is_some_and(|t| t.text.as_ref() == "as")).unwrap();
            assert_eq!(parts[as_index + 1].kind(), &K::TypeExpression);
            assert!(parts[..as_index].iter().filter(|v| v.kind() == &K::VariableBinding).all(|v| v.child_views().all(|c| c.kind() != &K::TypeExpression)));
        }
    }
    let list = ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList).unwrap();
    assert_eq!(list.children().len(), 35);
    assert!(list.child_views().all(|v| matches!(v.kind(), K::VariableDeclaration | K::AssignmentStatement | K::ReturnStatement)));
    assert_eq!(&text[ast.node_view(*list.children().last().unwrap()).unwrap().range().start..list.range().end], "var after_returns := not p;");
    for proof in ast.node_views().filter(|v| v.kind() == &K::ProofBlock) {
        assert_eq!(&text[proof.range().start..proof.range().end], "proof thus thesis; end");
        assert!(!ast.node_views().any(|v| v.kind() == &K::FormulaExpression && v.range().start < proof.range().start && v.range().end > proof.range().start));
    }
    let final_start = text.find("set after_boolean_operands = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len())));
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate Boolean RHS source child {id:?}");
        let node = ast.node_view(id).unwrap();
        if node.as_token().is_some() { tokens.push(id); }
        pending.extend(node.children().iter().rev().copied());
    }
    assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
    let frontend = task257c4c1_frontend_output(&text, 161);
    assert!(frontend.diagnostics.is_empty());
    assert_eq!(frontend.ast.as_ref().unwrap().snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
    let repeated = task257c4c1_frontend_output(&text, 161);
    assert_eq!(frontend.cache_keys, repeated.cache_keys);
    assert_eq!(frontend.ast, repeated.ast);
    let token_key = &frontend.cache_keys.tokens;
    assert_eq!(token_key.version.as_ref(), "mizar-frontend/token-stream-cache-key/v6");
    let mut retained_token = token_key.clone();
    retained_token.version = "mizar-frontend/token-stream-cache-key/v4".into();
    assert_eq!(token_key.lexical_hash, retained_token.lexical_hash);
    assert_eq!(token_key.active_lexical_environment, retained_token.active_lexical_environment);
    assert_eq!(token_key.parser_context, retained_token.parser_context);
    assert_eq!(token_key.parser_lexing_plan, retained_token.parser_lexing_plan);
    assert_ne!(token_key.stable_hash(), retained_token.stable_hash());
    let current = frontend.cache_keys.ast.as_ref().unwrap();
    assert_eq!(current.parser_version.version.as_ref(), "mizar-parser/surface-ast-v30");
    let mut retained = current.clone();
    retained.parser_version = mizar_frontend::parsing::ParserCacheKeyVersion::new("mizar-parser/surface-ast-v28");
    assert_eq!(current.token_stream_hash, retained.token_stream_hash);
    assert_eq!(current.parser_inputs_hash, retained.parser_inputs_hash);
    assert_ne!(current.stable_hash(), retained.stable_hash());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn algorithm_boolean_operand_recovery_preserves_existing_owners_and_successors() {
    use mizar_frontend::orchestration::{DiagnosticCode, DiagnosticLocation};
    use mizar_syntax::{SurfaceFormulaBinaryOperator as B, SurfaceFormulaConnective as C, SurfaceFormulaPrefixOperator as P, SurfaceNodeKind as K};
    let config = step5c11_config();
    let plan = build_test_plan(&config).unwrap();
    let case = plan.cases.iter().find(|case| case.id.0 == "fail_parser_algorithm_boolean_operands_001").unwrap();
    let text = std::fs::read_to_string(&case.source_path).unwrap();
    let first = super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 162);
    assert_eq!(first.status, super::super::ParseOnlyCaseStatus::Passed, "{first:?}");
    assert_eq!(first, super::super::run_parse_only_case(&config.workspace_root, &config.workspace_root.join("tests"), case, 162));
    let output = super::super::formula_statement::step5c8_test_frontend(&text);
    assert_eq!(output.diagnostics.len(), 11, "{:?}", output.diagnostics);
    let ast = output.ast.unwrap();
    for (diagnostic, (marker, word, code, message)) in output.diagnostics.iter().zip([
        ("p & ;", ";", "malformed_formula_expression", "expected formula after connective"),
        ("not ;", ";", "malformed_formula_expression", "expected formula after `not`"),
        ("return (p & q;", ";", "malformed_formula_expression", "expected `)` to close formula delimiter"),
        ("missing_binding := p, ;", ";", "malformed_term_expression", "expected declaration binding after `,`"),
        ("const missing_const;", ";", "malformed_term_expression", "expected `:=` in const declaration binding"),
        ("missing_initializer := ;", ";", "malformed_term_expression", "expected initializer term after `:=`"),
        ("return not thesis;", "thesis", "malformed_formula_expression", "`thesis` is restricted to explicit computation inputs"),
        ("p := p & q as", "as", "malformed_term_expression", "unexpected token in assignment"),
        ("p := p & q,", ",", "malformed_term_expression", "unexpected token in assignment"),
        ("return p & q as", "as", "malformed_term_expression", "unexpected token in return statement"),
        ("return p & q,", ",", "malformed_term_expression", "unexpected token in return statement"),
    ]) {
        let start = text.find(marker).unwrap();
        let anchor = start + marker.rfind(word).unwrap();
        assert_eq!(diagnostic.code, DiagnosticCode::Syntax(code.into()));
        assert_eq!(diagnostic.message.as_ref(), message);
        assert_eq!(diagnostic.location, DiagnosticLocation::SourceRange(range(ast.source_id, anchor, anchor + word.len())));
        if marker.starts_with("return (") {
            assert_eq!(diagnostic.secondary, [mizar_session::SourceAnchor::Range(range(ast.source_id, start + 7, start + 8))]);
            assert_eq!(diagnostic.recovery_note.as_deref(), Some("insert `)` before continuing"));
        } else {
            assert!(diagnostic.secondary.is_empty());
            assert_eq!(diagnostic.recovery_note.as_deref(), Some(if code == "malformed_term_expression" { "repair the term expression before continuing" } else { "repair the formula expression before continuing" }));
        }
    }
    for (marker, rhs, root_kind, recovery_kind) in [
        ("missing_conjunction :=", "p &", K::BinaryFormula(B { connective: C::And, repeated: false }), SyntaxRecoveryKind::MissingFormula),
        ("missing_not :=", "not", K::PrefixFormula(P::Not), SyntaxRecoveryKind::MissingFormula),
        ("return (p & q", "(p & q", K::ParenthesizedFormula, SyntaxRecoveryKind::UnmatchedOpeningDelimiter),
    ] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| matches!(v.kind(), K::VariableBinding | K::ReturnStatement) && v.range().start == start).unwrap();
        let payload = owner.child_views().find(|v| v.kind() == &K::FormulaExpression).unwrap();
        let rhs_start = start + if marker.contains(":=") { marker.len() + 1 } else { "return ".len() };
        assert_eq!(payload.range(), range(ast.source_id, rhs_start, rhs_start + rhs.len()));
        let root = ast.node_view(payload.children()[0]).unwrap();
        assert_eq!(root.kind(), &root_kind);
        let recovery = ast.node_view(*root.children().last().unwrap()).unwrap();
        assert_eq!(recovery.as_recovery(), Some(recovery_kind));
        let anchor = start + text[start..].find(';').unwrap();
        assert_eq!(recovery.range(), range(ast.source_id, anchor, anchor));
        assert!(recovery.children().is_empty());
    }
    for (marker, skip) in [("p := p & q as", "as expr of boolean"), ("p := p & q,", ", forbidden_assignment"), ("return p & q as", "as expr of boolean"), ("return p & q,", ", forbidden_return")] {
        let start = text.find(marker).unwrap();
        let owner = ast.node_views().find(|v| matches!(v.kind(), K::AssignmentStatement | K::ReturnStatement) && v.range().start == start).unwrap();
        let parts = owner.child_views().collect::<Vec<_>>();
        let payload = parts.iter().find(|v| v.kind() == &K::FormulaExpression).unwrap();
        assert_eq!(&text[payload.range().start..payload.range().end], "p & q");
        let skipped = parts.iter().find(|v| v.as_recovery() == Some(SyntaxRecoveryKind::SkippedToken)).unwrap();
        assert_eq!(&text[skipped.range().start..skipped.range().end], skip);
        assert!(skipped.child_views().all(|v| v.as_token().is_some()));
        assert_eq!(parts.last().unwrap().as_token().unwrap().text.as_ref(), ";");
        assert_eq!(parts.last().unwrap().range().end, owner.range().end);
        assert!(ast.trivia().skipped_token_ranges().iter().any(|v| v.range == skipped.range()));
    }
    let start = text.find("var missing_binding").unwrap();
    let declaration = ast.node_views().find(|v| v.kind() == &K::VariableDeclaration && v.range().start == start).unwrap();
    assert_eq!(declaration.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["var", "missing_binding := p", ",", "", ";"]);
    let binding = ast.node_view(declaration.children()[3]).unwrap();
    assert_eq!(binding.kind(), &K::VariableBinding);
    assert_eq!(ast.node_view(binding.children()[0]).unwrap().as_recovery(), Some(SyntaxRecoveryKind::MissingTerm));
    for name in ["missing_const", "missing_initializer"] {
        let start = text.find(&format!("const {name}")).unwrap() + "const ".len();
        let binding = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(ast.node_view(*binding.children().last().unwrap()).unwrap().as_recovery(), Some(SyntaxRecoveryKind::MissingTerm));
    }
    let start = text.find("return not thesis").unwrap();
    let returned = ast.node_views().filter_map(|v| v.as_return_statement()).find(|v| v.range().start == start).unwrap();
    assert!(!returned.is_recovered());
    let payload = ast.node_view(returned.children()[1]).unwrap();
    assert_eq!(&text[payload.range().start..payload.range().end], "not thesis");
    assert!(ast.node_views().any(|v| v.kind() == &K::FormulaConstant(mizar_syntax::SurfaceFormulaConstant::Thesis) && v.range().start == payload.range().start + 4));
    for name in ["after_conjunction", "after_not", "after_group", "after_binding", "after_const", "after_initializer", "after_thesis", "after_assignment_as", "after_assignment_comma", "after_return_as", "after_return_comma"] {
        let spelling = format!("{name} := p");
        let start = text.find(&spelling).unwrap();
        let binding = ast.node_views().filter_map(|v| v.as_variable_binding()).find(|v| v.range().start == start).unwrap();
        assert_eq!(binding.range(), range(ast.source_id, start, start + spelling.len()));
        assert!(!binding.is_recovered());
        let payload = ast.node_view(binding.children()[2]).unwrap();
        assert_eq!(payload.kind(), &K::TermExpression);
        assert_eq!(ast.node_view(payload.children()[0]).unwrap().kind(), &K::TermReference);
    }
    for (kind, count) in [(SyntaxRecoveryKind::MissingFormula, 2), (SyntaxRecoveryKind::UnmatchedOpeningDelimiter, 1), (SyntaxRecoveryKind::MissingTerm, 3), (SyntaxRecoveryKind::SkippedToken, 4)] {
        assert_eq!(ast.node_views().filter(|v| v.as_recovery() == Some(kind)).count(), count);
    }
    assert_eq!(ast.node_views().filter_map(|v| v.as_recovery()).count(), 10);
    assert_eq!(ast.trivia().skipped_token_ranges().len(), 4);
    assert_eq!(ast.node_views().find(|v| v.kind() == &K::AlgorithmStatementList).unwrap().children().len(), 22);
    let final_start = text.find("set after_boolean_failure = sentinel;").unwrap();
    assert!(ast.node_views().filter_map(|v| v.as_set_statement()).any(|v| v.range() == range(ast.source_id, final_start, text.trim_end().len()) && !v.is_recovered()));
    let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
    let mut seen = std::collections::HashSet::new();
    let mut tokens = Vec::new();
    while let Some(id) = pending.pop() {
        assert!(seen.insert(id), "duplicate recovered Boolean RHS source child {id:?}");
        let node = ast.node_view(id).unwrap();
        if node.as_token().is_some() { tokens.push(id); }
        pending.extend(node.children().iter().rev().copied());
    }
    assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
    let bytes = ast.canonical_bytes().unwrap();
    let restored = mizar_syntax::SurfaceAst::from_canonical_bytes(&bytes, ast.source_id).unwrap();
    assert_eq!(restored.snapshot_text_with_trivia(), ast.snapshot_text_with_trivia());
}

#[test]
fn algorithm_boolean_rhs_keeps_expr_source_but_consumers_refuse_its_profile() {
    use mizar_resolve::names::{SourceVariableScopeError, SourceVariableScopeResolver};
    use mizar_syntax::SurfaceNodeKind as K;
    let config = step5c11_config();
    let case = step5c14_state_case();
    let base = concat!(
        "definition\n  let p,q be expr of boolean;\n",
        "  algorithm rhs(p,q) -> expr of boolean\n  do\n",
        "    var local := p;\n    local := p;\n    return p;\n",
        "    return p;\n  end;\nend;\n",
    );
    let object = "definition let x be object; algorithm rhs(x) -> object do var local := x; local := x; return x; end; end;";
    for (selected, text) in [
        (None, object.to_owned()),
        (None, base.to_owned()), // The unchanged term RHS already has an unsupported expr profile.
        (Some(0), base.replace("var local := p;", "var local := not p;")),
        (Some(1), base.replace("\n    local := p;", "\n    local := p & q;")),
        (Some(2), base.replacen("return p;", "return (p) & q;", 1)),
    ] {
        let expr_source = text.contains("expr of boolean");
        let frontend = super::super::formula_statement::step5c8_test_frontend(&text);
        assert!(frontend.diagnostics.is_empty(), "{selected:?}: {:?}", frontend.diagnostics);
        let ast = frontend.ast.as_ref().unwrap();
        assert!(ast.node_views().all(|v| !v.is_recovered() && v.as_recovery().is_none()));
        assert!(ast.trivia().skipped_token_ranges().is_empty());
        let body = ast.node_views().find(|v| v.kind() == &K::AlgorithmBody).unwrap();
        assert_eq!(body.children().len(), 3);
        assert_eq!(body.child_views().next().unwrap().as_token().unwrap().text.as_ref(), "do");
        assert_eq!(body.child_views().last().unwrap().as_token().unwrap().text.as_ref(), "end");
        assert_eq!(ast.node_view(body.children()[1]).unwrap().kind(), &K::AlgorithmStatementList);
        let statements = ast.node_view(body.children()[1]).unwrap().child_views().collect::<Vec<_>>();
        let kinds = [&K::VariableDeclaration, &K::AssignmentStatement, &K::ReturnStatement, &K::ReturnStatement];
        assert_eq!(statements.iter().map(|v| v.kind()).collect::<Vec<_>>(), kinds[..if expr_source { 4 } else { 3 }]);
        for statement in &statements {
            assert_eq!(statement.child_views().last().unwrap().as_token().unwrap().text.as_ref(), ";");
        }
        assert_eq!(statements.last().unwrap().range().start, text.rfind(if expr_source { "return p;" } else { "return x;" }).unwrap());
        if expr_source {
            let parameter = ast.node_views().find(|v| v.kind() == &K::DefinitionParameter).unwrap();
            assert_eq!(&text[parameter.range().start..parameter.range().end], "let p,q be expr of boolean;");
            let params = ast.node_views().find(|v| v.kind() == &K::AlgorithmParameters).unwrap();
            assert_eq!(&text[params.range().start..params.range().end], "(p,q)");
            assert_eq!(ast.node_views().filter(|v| v.kind() == &K::TypeExpression && &text[v.range().start..v.range().end] == "expr of boolean").count(), 2);
        }
        for (index, kind) in [K::VariableBinding, K::AssignmentStatement, K::ReturnStatement].into_iter().enumerate() {
            let owner = ast.node_views().find(|v| v.kind() == &kind).unwrap();
            let rhs = owner.child_views().filter(|v| matches!(v.kind(), K::TermExpression | K::FormulaExpression)).collect::<Vec<_>>();
            assert_eq!(rhs.len(), 1);
            let rhs = rhs[0];
            let expected = if selected == Some(index) { ["not p", "p & q", "(p) & q"][index] } else if expr_source { "p" } else { "x" };
            assert_eq!(&text[rhs.range().start..rhs.range().end], expected);
            assert_eq!(rhs.kind(), if selected == Some(index) { &K::FormulaExpression } else { &K::TermExpression });
            assert_eq!(rhs.children().len(), 1);
            if selected != Some(index) { continue; }
            let root = ast.node_view(rhs.children()[0]).unwrap();
            let operands = ast.node_views().filter(|v| v.kind() == &K::TermExpression && v.range().start >= rhs.range().start && v.range().end <= rhs.range().end).collect::<Vec<_>>();
            let names = expected.char_indices().filter(|(_, ch)| matches!(*ch, 'p' | 'q')).collect::<Vec<_>>();
            assert_eq!(operands.len(), names.len());
            for (operand, (offset, name)) in operands.iter().zip(names) {
                assert_eq!((operand.range().start, operand.range().end), (rhs.range().start + offset, rhs.range().start + offset + 1));
                assert_eq!(operand.children().len(), 1);
                let reference = ast.node_view(operand.children()[0]).unwrap();
                assert_eq!(reference.kind(), &K::TermReference);
                assert_eq!(reference.range(), operand.range());
                assert_eq!(reference.children().len(), 1);
                let token = ast.node_view(reference.children()[0]).unwrap();
                assert_eq!(token.range(), reference.range());
                assert_eq!(token.as_token().unwrap().text.as_ref(), name.to_string());
            }
            assert_eq!(root.children().last(), Some(&operands.last().unwrap().id()));
            if index == 0 {
                assert_eq!(root.kind(), &K::PrefixFormula(mizar_syntax::SurfaceFormulaPrefixOperator::Not));
                assert_eq!(root.children().len(), 2);
            } else {
                assert_eq!(root.kind(), &K::BinaryFormula(mizar_syntax::SurfaceFormulaBinaryOperator { connective: mizar_syntax::SurfaceFormulaConnective::And, repeated: false }));
                assert_eq!(root.children().len(), 3);
                if index == 1 { assert_eq!(root.children()[0], operands[0].id()); }
                else {
                    let group = ast.node_view(root.children()[0]).unwrap();
                    assert_eq!(group.kind(), &K::ParenthesizedFormula);
                    assert_eq!(group.child_views().map(|v| &text[v.range().start..v.range().end]).collect::<Vec<_>>(), ["(", "p", ")"]);
                    let inner = ast.node_view(group.children()[1]).unwrap();
                    assert_eq!(inner.kind(), &K::FormulaExpression);
                    assert_eq!(inner.children(), &[operands[0].id()]);
                }
            }
        }
        assert!(!ast.node_views().any(|v| matches!(v.kind(), K::PredicateApplication | K::InlinePredicateApplication | K::MatchCase)));
        let mut pending = vec![ast.node_views().find(|v| v.kind() == &K::CompilationUnit).unwrap().id()];
        let mut seen = std::collections::HashSet::new();
        let mut tokens = Vec::new();
        while let Some(id) = pending.pop() {
            assert!(seen.insert(id), "duplicate RHS source child {id:?}");
            let node = ast.node_view(id).unwrap();
            if node.as_token().is_some() { tokens.push(id); }
            pending.extend(node.children().iter().rev().copied());
        }
        assert_eq!(tokens, ast.token_views().map(|v| v.id()).collect::<Vec<_>>());
        let (source, typed, symbols) = super::super::source_registration_inputs(&config.workspace_root, &case, frontend).unwrap();
        let scope = SourceVariableScopeResolver::resolve_algorithm_occurrences(&source, &symbols);
        let checked = mizar_checker::type_checker::check_source_algorithm_types(&source, &typed, &symbols);
        if expr_source {
            // Both consumers require object formals/results and refuse before inspecting these RHSs.
            assert_eq!(scope.unwrap_err(), SourceVariableScopeError::InvalidShape);
            assert_eq!(checked.unwrap_err(), "algorithms.unsupported_source_types");
        } else {
            // This supported object/Core control validates the harness only.
            scope.unwrap();
            assert_eq!(mizar_core::elaborator::lower_source_algorithms(&checked.unwrap()).unwrap().algorithms().len(), 1);
        }
    }
}

#[test]
fn algorithm_boolean_expr_targets_preserve_identifier_and_symbol_roles() {
    use mizar_syntax::{SurfaceNodeKind as K, SurfaceTokenKind as T};
    for (role, text) in [
        ("term", "definition let boolean be object; algorithm f(boolean) -> object do var value := g[boolean](boolean); return boolean; end; end;"),
        ("symbol", "definition mode RegisteredBoolean: boolean is object; let p be expr of boolean; algorithm f(p) -> expr of boolean do return p; end; end;"),
        ("qualified", "definition mode RegisteredType: T is object; let p be expr of boolean.T; algorithm f(p) -> object do return p; end; end;"),
        ("raw", "definition let p be expr of boolean; algorithm f(p) -> expr of boolean do return p; end; end;"),
    ] {
        let output = super::super::formula_statement::step5c8_test_frontend(text);
        assert!(output.diagnostics.is_empty(), "{role}: {:?}", output.diagnostics);
        let ast = output.ast.unwrap();
        assert!(ast.node_views().all(|v| !v.is_recovered()));
        if role == "term" {
            let arguments = ast.node_views().find(|v| v.kind() == &K::TemplateArguments).unwrap();
            let wrapper = arguments.child_views().find(|v| v.kind() == &K::TemplateArgument).unwrap();
            assert_eq!(wrapper.children().len(), 1);
            let argument = ast.node_view(wrapper.children()[0]).unwrap();
            assert_eq!(argument.kind(), &K::TermExpression);
            assert_eq!(&text[argument.range().start..argument.range().end], "boolean");
            assert!(ast.token_views().filter(|v| v.as_token().unwrap().text.as_ref() == "boolean").all(|v| v.as_token().unwrap().kind == T::Identifier));
            continue;
        }
        let outer = ast.node_views().find(|v| v.kind() == &K::TypeHead && text[v.range().start..v.range().end].starts_with("expr of ")).unwrap();
        let parts = outer.child_views().collect::<Vec<_>>();
        assert_eq!(parts.len(), 3);
        assert_eq!(parts[0].as_token().unwrap().text.as_ref(), "expr");
        assert_eq!(parts[1].as_token().unwrap().text.as_ref(), "of");
        assert_eq!(parts[2].kind(), &K::TypeExpression);
        let head = ast.node_view(parts[2].children()[0]).unwrap();
        assert_eq!(head.kind(), &K::TypeHead);
        assert_eq!(head.range(), parts[2].range());
        assert_eq!(head.children().len(), 1);
        let leaf = ast.node_view(head.children()[0]).unwrap();
        if role == "raw" {
            assert_eq!(leaf.as_token().unwrap().kind, T::Identifier);
            assert_eq!(leaf.as_token().unwrap().text.as_ref(), "boolean");
            assert_eq!(leaf.range(), head.range());
        } else {
            assert_eq!(leaf.kind(), &K::QualifiedSymbol);
            assert_eq!(&text[leaf.range().start..leaf.range().end], if role == "symbol" { "boolean" } else { "boolean.T" });
            assert!(ast.token_views().any(|v| v.as_token().unwrap().kind == T::UserSymbol && v.range().start >= head.range().start && v.range().end <= head.range().end));
        }
    }
}

#[test]
fn algorithm_boolean_predicate_commas_do_not_introduce_lexical_bindings() {
    use mizar_frontend::lexing::BindingShapeKind as B;
    let text = concat!(
        "definition let x,y,z be object; pred Pair: x,y local_pair means x = y; ",
        "pred Trip: x,y,z local_trip means x = y; algorithm f(x,y,z) -> object do ",
        "var a,b as object; var c := x, d, e := y; const k := x, l := y; ",
        "var pair_result := not x,y local_pair; var trip_result := not x,y,z local_trip; ",
        "var grouped := not (x),y local_pair; return x; end; end;",
    );
    let output = task257c4c1_frontend_output(text, 163);
    assert!(output.diagnostics.is_empty(), "{:?}", output.diagnostics);
    let ast = output.ast.as_ref().unwrap();
    assert!(ast.node_views().all(|v| !v.is_recovered()));
    let bindings = output.tokens.scope_view.frames.iter().flat_map(|frame| &frame.bindings)
        .filter(|binding| matches!(binding.kind, B::Var | B::Const)).collect::<Vec<_>>();
    assert_eq!(bindings.iter().map(|binding| binding.spelling.as_ref()).collect::<Vec<_>>(), ["a", "b", "c", "d", "e", "k", "l", "pair_result", "trip_result", "grouped"]);
    for binding in bindings {
        assert_eq!(&text[binding.introduced_at.start..binding.introduced_at.end], binding.spelling.as_ref());
        assert_eq!(binding.introduced_at.source_id, ast.source_id);
    }
}
