use super::*;

#[test]
fn source_public_algorithm_export_candidates_preserve_original_rows() {
    use mizar_artifact::{
        module_summary::{
            ExportedSymbolSummary, MODULE_SUMMARY_SCHEMA_FAMILY as FAMILY, ModuleSummaryIdentity,
            SourceRangeSummary, current_schema_version as schema,
        },
        store::{CanonicalHashDomain, CanonicalJson as J, HashClass},
    };
    use mizar_resolve::symbols::source_public_algorithm_export_candidates as exports;
    use mizar_resolve::{
        declarations::DeclarationShellKind,
        env::{ExportStatus, SymbolKind, Visibility},
        symbols::source_declaration_shells,
    };
    use mizar_session::{Hash, SourceAnchor};
    let original = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    let display =
        "let a be object ; terminating algorithm swapalgo ( a ) -> object ensures result = a";
    let mut baseline = None;
    for case in 0..15 {
        let text = match case {
            1 => original.replace("x := c;", "x := a;"),
            2 => format!(":: λ\n{original}"),
            3 => original
                .replace("let a be object", "let input be object")
                .replace("swapalgo(a)", "copyalgo(input)")
                .replace("result = a", "result = input")
                .replace(":= a;", ":= input;")
                .replace("= a;", "= input;"),
            4 => original.replace("terminating ", ""),
            5 => original.replace("    ensures result = a\n", ""),
            6 => original.replace("let a be", "let a being"),
            7 => original.replace("terminating algorithm", "public terminating algorithm"),
            8 => original.replace("terminating algorithm", "private terminating algorithm"),
            9 => String::new(),
            13 => original.replace("    x := c;", "    const d := a;\n    x := c;"),
            _ => original.to_owned(),
        };
        let package = if case == 10 { "foreign" } else { "pkg" };
        let module = if case == 11 { "Nested.Main" } else { "owners" };
        let identity = ModuleSummaryIdentity {
            package_id: package.into(),
            package_version: Some(if case == 14 { "2.0" } else { "1.0" }.into()),
            lockfile_identity: Some("caller-lock".into()),
            module_path: module.into(),
            language_edition: "2026".into(),
        };
        let skip = if case == 12 { 7 } else { 0 };
        let (frontend, source, collected) =
            source_registration_owner_inputs(&text, package, module, skip);
        assert!(frontend.diagnostics.is_empty());
        assert!(collected.diagnostics().is_empty());
        let ast = frontend.ast.as_ref().unwrap();
        let env = collected.env();
        let shells = source_declaration_shells(&source, env).unwrap();
        if case == 9 {
            assert!(shells.declarations().is_empty() && env.symbols().is_empty());
            assert_eq!(exports(&source, env, &identity).unwrap(), Some(vec![]));
            continue;
        }
        let owners = env
            .symbols()
            .iter()
            .filter(|owner| owner.kind() == SymbolKind::Algorithm)
            .collect::<Vec<_>>();
        let [owner] = owners.as_slice() else {
            panic!("one actual algorithm")
        };
        let name = if case == 3 { "copyalgo" } else { "swapalgo" };
        assert_eq!(owner.primary_spelling(), name);
        assert_eq!(owner.symbol().module(), source.module());
        assert_eq!(owner.namespace().as_str(), module);
        let fqn = owner.symbol().fqn().as_str();
        assert!(fqn.starts_with(&format!("{package}::{module}::contribution=")));
        let shell = shells
            .declarations()
            .iter()
            .find(|shell| shell.kind() == DeclarationShellKind::AlgorithmDefinition)
            .unwrap();
        assert_eq!(shells.declarations().len(), 2);
        let start = text
            .find("terminating algorithm")
            .or_else(|| text.find("algorithm "))
            .unwrap();
        let end = text.find("  end;").unwrap() + "  end;".len();
        assert_eq!((shell.range().start, shell.range().end), (start, end));
        assert_eq!(owner.origin().anchor(), &SourceAnchor::Range(shell.range()));
        if case == 8 {
            assert_eq!(
                (owner.visibility(), owner.export_status()),
                (Visibility::Private, ExportStatus::LocalOnly)
            );
            assert_eq!(exports(&source, env, &identity).unwrap(), Some(vec![]));
            continue;
        }
        assert_eq!(
            (owner.visibility(), owner.export_status()),
            (Visibility::Public, ExportStatus::Exported)
        );
        let rendered = match case {
            3 => "let input be object ; terminating algorithm copyalgo ( input ) -> object ensures result = input".to_owned(),
            4 => display.replace("terminating ", ""),
            5 => display.replace(" ensures result = a", ""),
            6 => display.replace(" be ", " being "),
            _ => display.to_owned(),
        };
        let mut row = ExportedSymbolSummary {
            origin_id: fqn.into(),
            fully_qualified_name: fqn.into(),
            namespace_path: module.split('.').map(str::to_owned).collect(),
            visibility: "public".into(),
            declaration_kind: "algorithm".into(),
            source_range: SourceRangeSummary {
                start_byte: start as u64,
                end_byte: end as u64,
            },
            rendered_signature: rendered.clone(),
            interface_fingerprint: Hash::from_bytes([0; Hash::BYTE_LEN]),
            proof_status: None,
        };
        row.interface_fingerprint = row
            .compute_interface_fingerprint(schema(), &identity)
            .unwrap();
        if case == 0 {
            let symbol = J::Object(
                [
                    ("origin_id", fqn),
                    ("fully_qualified_name", fqn),
                    ("visibility", "public"),
                    ("declaration_kind", "algorithm"),
                    ("rendered_signature", rendered.as_str()),
                ]
                .into_iter()
                .map(|(k, v)| (k.into(), J::string(v)))
                .chain([
                    (
                        "namespace_path".into(),
                        J::array(module.split('.').map(J::string)),
                    ),
                    ("proof_status".into(), J::Null),
                ])
                .collect(),
            );
            let module_json = J::object([
                ("package_id", J::string(package)),
                ("package_version", J::string("1.0")),
                ("lockfile_identity", J::string("caller-lock")),
                ("module_path", J::string(module)),
                ("language_edition", J::string("2026")),
            ])
            .unwrap();
            let projection = J::object([("module", module_json), ("symbol", symbol)]).unwrap();
            let domain = CanonicalHashDomain::new(HashClass::Interface, FAMILY, schema());
            assert_eq!(row.interface_fingerprint, domain.hash(&projection, &[]));
            let signature = owner.signature().unwrap().clone();
            baseline = Some((signature, row.clone(), ast.source_id));
        }
        let (signature, base, source_id) = baseline.as_ref().unwrap();
        if matches!(case, 1 | 13) {
            assert_ne!(owner.signature().unwrap(), signature);
        }
        if matches!(case, 1 | 2 | 12 | 13) {
            assert_eq!(row.interface_fingerprint, base.interface_fingerprint);
        }
        if matches!(case, 1 | 12) {
            assert_eq!(&row, base);
        }
        if matches!(case, 2 | 13) {
            assert_ne!(row.source_range, base.source_range);
        }
        if case == 12 {
            assert_ne!(ast.source_id, *source_id);
        }
        if matches!(case, 3..=6 | 10 | 11 | 14) {
            assert_ne!(row.interface_fingerprint, base.interface_fingerprint);
        }
        let before = (source.clone(), env.clone(), identity.clone());
        let actual = exports(&source, env, &identity).unwrap();
        assert_eq!(actual, Some(vec![row]));
        assert_eq!(exports(&source, env, &identity).unwrap(), actual);
        assert_eq!((source, env.clone(), identity), before);
    }
}

#[test]
fn source_public_algorithm_export_candidates_preserve_refusals() {
    use mizar_artifact::module_summary::ModuleSummaryIdentity;
    use mizar_resolve::symbols::source_public_algorithm_export_candidates as exports;
    use mizar_resolve::symbols::{source_declaration_shells, validate_source_symbol_env};
    let original = include_str!(
        "../../../../../tests/miz/pass/algorithms/pass_proof_verification_algorithm_var_const_assert_001.miz"
    );
    let identity = ModuleSummaryIdentity {
        package_id: "export-test".into(),
        package_version: None,
        lockfile_identity: None,
        module_path: "owners".into(),
        language_edition: "2026".into(),
    };
    let invalid = ModuleSummaryIdentity {
        language_edition: String::new(),
        ..identity.clone()
    };
    let wrong = ModuleSummaryIdentity {
        module_path: "foreign".into(),
        ..identity.clone()
    };
    let wrong_package = ModuleSummaryIdentity {
        package_id: "foreign".into(),
        ..identity.clone()
    };
    let (_, source, _) = source_registration_owner_inputs(original, "export-test", "owners", 0);
    let (_, _, foreign) = source_registration_owner_inputs(
        &original.replace("swapalgo", "foreign"),
        "export-test",
        "owners",
        0,
    );
    let (_, _, augmented) = source_registration_owner_inputs(
        &format!("{original}\n{}", original.replace("swapalgo", "second")),
        "export-test",
        "owners",
        0,
    );
    for env in [foreign.env(), augmented.env()] {
        let error = validate_source_symbol_env(&source, env).unwrap_err();
        assert_eq!(
            error,
            "source symbol environment does not match its declaration collection"
        );
        for supplied in [&identity, &invalid, &wrong, &wrong_package] {
            assert_eq!(exports(&source, env, supplied).unwrap_err(), error);
        }
    }
    let recovered = include_str!(
        "../../../../../tests/miz/fail/parser/fail_parser_algorithm_verification_recovery_001.miz"
    );
    let (frontend, source, recovered) =
        source_registration_owner_inputs(recovered, "export-test", "owners", 0);
    assert!(!frontend.diagnostics.is_empty());
    let error = validate_source_symbol_env(&source, recovered.env()).unwrap_err();
    assert_eq!(
        error,
        "source symbol environment does not match its declaration collection"
    );
    for supplied in [&identity, &invalid, &wrong, &wrong_package] {
        assert_eq!(
            exports(&source, recovered.env(), supplied).unwrap_err(),
            error
        );
    }
    let multiple = format!("{original}\n{}", original.replace("swapalgo", "second"));
    let two_algorithms = original.replace(
        "  end;\nend;",
        "  end;\n  algorithm second(a) -> object\n  do return a; end;\nend;",
    );
    for (text, private_or_empty) in [
        (String::new(), true),
        (
            original.replace("terminating algorithm", "private terminating algorithm"),
            true,
        ),
        (original.replace("  let a be object;\n", ""), false),
        (
            original.replace("let a be object;", "let a be object;\n  let b be object;"),
            false,
        ),
        (
            original.replace("let a be object", "let a, b be object"),
            false,
        ),
        (original.replace("be object", "be set"), false),
        (original.replace("swapalgo(a)", "swapalgo(b)"), false),
        (original.replace("swapalgo(a)", "swapalgo(a, a)"), false),
        (original.replace("swapalgo(a)", "swapalgo[t](a)"), false),
        (original.replace(" -> object", ""), false),
        (original.replace("-> object", "-> set"), false),
        (
            original.replace("    ensures", "    requires a = a\n    ensures"),
            false,
        ),
        (
            original.replace("ensures result = a", "ensures a = a"),
            false,
        ),
        (
            original
                .replace("terminating algorithm", "private terminating algorithm")
                .replace("-> object", "-> set"),
            false,
        ),
        (format!("{original}\nreserve Extra for object;\n"), false),
        (multiple, false),
        (two_algorithms, false),
    ] {
        let (frontend, source, collected) =
            source_registration_owner_inputs(&text, "export-test", "owners", 0);
        assert!(frontend.diagnostics.is_empty(), "{text}");
        assert!(collected.diagnostics().is_empty(), "{text}");
        validate_source_symbol_env(&source, collected.env()).unwrap();
        source_declaration_shells(&source, collected.env()).unwrap();
        let error = invalid.canonical_json().unwrap_err().to_string();
        assert!(error.contains("language_edition"), "{private_or_empty}");
        wrong.canonical_json().unwrap();
        assert_ne!(wrong.module_path, source.module().path().as_str());
        let env = collected.env();
        let expected = if private_or_empty { Some(vec![]) } else { None };
        assert_eq!(exports(&source, env, &identity).unwrap(), expected);
        assert_eq!(exports(&source, env, &invalid).unwrap_err(), error);
        for supplied in [&wrong, &wrong_package] {
            assert_eq!(
                exports(&source, env, supplied).unwrap_err(),
                "export identity does not match source module"
            );
        }
    }
}
