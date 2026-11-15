use super::*;

#[test]
fn task258_native_statement_fixture_and_guard_events_are_source_owned() {
    use crate::typed_ast::tests::{task257b5_native_case, task257b5_native_replay};
    let fixture = include_str!("../../../tests/testdata/native_universal_statement.miz");
    for (text, events, guards) in [
        (fixture, 1, 1),
        (
            "theorem T: for H being set holds H = H proof let L, U, R be set; thus L = R; end;",
            1,
            3,
        ),
        (
            "theorem T: for H being set for J being set holds H = J proof let L be set; let U be set; let R be set; thus L = R; end;",
            3,
            3,
        ),
    ] {
        task257b5_native_case(
            text,
            |lower, _, _, expressions, (symbols, source, scope)| {
                let lower = lower.unwrap();
                let typed = lower
                    .clone()
                    .with_source_statement_from_original(source, symbols, scope)
                    .unwrap();
                let statement = typed.source_statement().unwrap();
                let resolved = task257b5_native_replay(&typed, expressions.clone()).unwrap();
                assert_eq!(
                    (
                        statement.owners.len(),
                        statement.statements.len(),
                        statement.contexts.len(),
                        statement.input_facts.len(),
                        statement.candidate_facts.len()
                    ),
                    (1, 2, 2, guards, 2)
                );
                assert_eq!(
                    statement
                        .statements
                        .rows
                        .iter()
                        .map(|row| row.source_ordinal)
                        .collect::<Vec<_>>(),
                    [0, events + 1]
                );
                assert!(matches!(
                    statement.statements.rows[0].formula,
                    SourceStatementFormulaTarget::Composite(_)
                ));
                assert!(matches!(
                    statement.statements.rows[1].formula,
                    SourceStatementFormulaTarget::Atomic(_)
                ));
                assert!(statement.contexts.rows[0].visible_bindings.is_empty());
                assert_eq!(
                    statement.contexts.rows[1].visible_bindings,
                    statement
                        .input_facts
                        .rows
                        .iter()
                        .map(|row| row.binding)
                        .collect::<Vec<_>>()
                );
                assert!(
                    statement
                        .input_facts
                        .rows
                        .iter()
                        .all(|row| row.kind == SourceStatementInputFactKind::LetTypeGuard)
                );
                assert_eq!(
                    statement
                        .input_facts
                        .rows
                        .iter()
                        .map(|row| row.uses.len())
                        .sum::<usize>(),
                    2
                );
                if guards == 3 {
                    assert!(statement.input_facts.rows[1].uses.is_empty());
                }
                for guard in &statement.input_facts.rows {
                    assert!(
                        typed
                            .source_type()
                            .unwrap()
                            .applications()
                            .iter()
                            .any(|(_, row)| row.binding() == guard.binding)
                    );
                    assert!(
                        typed
                            .source_composite_formula()
                            .unwrap()
                            .binders()
                            .iter()
                            .all(|(_, row)| row.binding() != guard.binding)
                    );
                    for reference in &guard.uses {
                        assert_eq!(
                            typed
                                .source_term()
                                .unwrap()
                                .references()
                                .get(*reference)
                                .unwrap()
                                .binding(),
                            guard.binding
                        );
                    }
                }
                assert_eq!(
                    statement.binding_env(),
                    lower.source_context().unwrap().binding_env()
                );
                assert_eq!(
                    statement.binding_fingerprint(),
                    statement.binding_env().debug_text()
                );
                assert_eq!(
                    statement.primary_term_fingerprint(),
                    lower.source_term().unwrap().debug_text()
                );
                assert_eq!(
                    statement.atomic_formula_fingerprint(),
                    lower.source_atomic_formula().unwrap().debug_text()
                );
                assert_eq!(
                    statement.composite_formula_fingerprint(),
                    Some(
                        lower
                            .source_composite_formula()
                            .unwrap()
                            .debug_text()
                            .as_str()
                    )
                );
                assert_eq!(
                    statement.formula_composition_fingerprint(),
                    Some(
                        lower
                            .source_formula_composition()
                            .unwrap()
                            .debug_text()
                            .as_str()
                    )
                );
                assert_eq!(statement.clone(), *statement);
                assert!(statement.debug_text().contains("kind=let-type-guard"));
                assert_eq!(resolved.source_statement(), Some(statement));
                if guards == 1 {
                    for reverse in [false, true] {
                        let mut changed = statement.clone();
                        if reverse {
                            changed.input_facts.rows[0].uses.reverse();
                        } else {
                            let duplicate = changed.input_facts.rows[0].uses[0];
                            changed.input_facts.rows[0].uses.push(duplicate);
                        }
                        assert_ne!(&changed, statement);
                        assert_eq!(
                            changed.validate_native_installation(
                                &typed,
                                typed.source_composite_formula().unwrap(),
                                typed.source_formula_composition().unwrap()
                            ),
                            Err(SourceStatementError::DependencyMismatch)
                        );
                        let mut forged = typed.clone();
                        forged.inject_source_statement_for_test(changed);
                        let before = forged.clone();
                        assert_eq!(
                            task257b5_native_replay(&forged, expressions.clone()),
                            Err(ResolvedTypedAstError::InvalidSourceStatement)
                        );
                        assert_eq!(forged, before);
                        assert_eq!(
                            task257b5_native_replay(&typed, expressions.clone()).unwrap(),
                            resolved
                        );
                    }
                }
                assert_eq!(
                    task257b5_native_replay(&typed, expressions).unwrap(),
                    resolved
                );
            },
        );
    }
}

#[test]
fn task258_native_statement_replays_every_table_field_and_association() {
    use crate::typed_ast::tests::{task257b5_native_case, task257b5_native_replay};
    task257b5_native_case(
        "theorem T: for H being set holds H = H proof let L, U, R be set; thus L = R; end;",
        |lower, _, _, expressions, (symbols, source, scope)| {
            let typed = lower
                .unwrap()
                .with_source_statement_from_original(source, symbols, scope)
                .unwrap();
            let original = typed.clone();
            let pristine = task257b5_native_replay(&typed, expressions.clone()).unwrap();
            let statement = typed.source_statement().unwrap();
            let composite = typed.source_composite_formula().unwrap();
            let composition = typed.source_formula_composition().unwrap();
            let header_binding = composite.binders().iter().next().unwrap().1.binding();
            let header_uses = typed
                .source_term()
                .unwrap()
                .references()
                .iter()
                .filter(|(_, row)| row.binding() == header_binding)
                .map(|(id, _)| id)
                .collect::<Vec<_>>();
            let mut contributions = SourceContributionIndex::new();
            contributions.insert(
                typed.module_id().clone(),
                ContributionKind::LocalSource {
                    source_id: typed.source_id(),
                },
                SourceAnchor::Range(statement.owners.rows[0].source_range),
            );
            let foreign_contribution = contributions.insert(
                typed.module_id().clone(),
                ContributionKind::LocalSource {
                    source_id: typed.source_id(),
                },
                SourceAnchor::Range(statement.owners.rows[0].source_range),
            );
            assert_ne!(foreign_contribution, statement.owner_contribution);
            let reject = |changed: SourceStatementHandoff, mutation: &str| {
                assert_ne!(&changed, statement, "{}", mutation);
                assert_eq!(
                    changed.validate_native_installation(&typed, composite, composition),
                    Err(SourceStatementError::DependencyMismatch),
                    "{}",
                    mutation
                );
                let mut forged = typed.clone();
                forged.inject_source_statement_for_test(changed);
                let before = forged.clone();
                assert_eq!(
                    task257b5_native_replay(&forged, expressions.clone()),
                    Err(ResolvedTypedAstError::InvalidSourceStatement),
                    "{}",
                    mutation
                );
                assert_eq!(forged, before);
                assert_eq!(typed, original);
                assert_eq!(
                    task257b5_native_replay(&typed, expressions.clone()).unwrap(),
                    pristine
                );
            };
            macro_rules! reject {
                ($changed:ident, $mutation:block) => {{
                    let mut $changed = statement.clone();
                    $mutation
                    reject($changed, stringify!($mutation));
                }};
            }
            reject!(h, {
                h.source_id = source_id(2);
            });
            reject!(h, {
                h.module_id = ModuleId::new(PackageId::new("foreign"), ModulePath::new("owners"));
            });
            reject!(h, {
                h.binding_fingerprint.push('!');
            });
            reject!(h, {
                h.primary_term_fingerprint.push('!');
            });
            reject!(h, {
                h.atomic_formula_fingerprint.push('!');
            });
            reject!(h, {
                h.composite_formula_fingerprint = None;
            });
            reject!(h, {
                h.formula_composition_fingerprint = None;
            });
            reject!(h, {
                h.composite_formula_fingerprint.as_mut().unwrap().push('!');
            });
            reject!(h, {
                h.formula_composition_fingerprint
                    .as_mut()
                    .unwrap()
                    .push('!');
            });
            reject!(h, {
                let mut contexts = BindingContextTable::new();
                contexts.insert(BindingContextDraft {
                    owner: BindingContextOwner::Module,
                    parent: None,
                    layer: BindingContextLayer::Module,
                    lexical_scope: None,
                    bindings: Vec::new(),
                    visible_bindings: Vec::new(),
                    recovery: BindingContextRecovery::Normal,
                });
                h.binding_env = BindingEnv::try_new(BindingEnvParts {
                    source_id: source_id(2),
                    module_id: h.module_id.clone(),
                    contexts,
                    bindings: BindingTable::new(),
                    diagnostics: BindingDiagnosticTable::new(),
                })
                .unwrap();
                h.binding_fingerprint = h.binding_env.debug_text();
            });
            reject!(h, {
                h.owner_contribution = foreign_contribution;
            });
            reject!(h, {
                h.owners.rows[0].contribution = foreign_contribution;
            });
            reject!(h, {
                h.owner_contribution = foreign_contribution;
                h.owners.rows[0].contribution = foreign_contribution;
            });
            reject!(h, {
                h.owners.rows[0].symbol = SymbolId::new(
                    ModuleId::new(PackageId::new("foreign"), ModulePath::new("owners")),
                    h.owners.rows[0].symbol.local().clone(),
                    h.owners.rows[0].symbol.fqn().clone(),
                );
            });
            reject!(h, {
                h.owners.rows[0].site = h.statements.rows[1].site.clone();
            });
            reject!(h, {
                h.owners.rows[0].source_range.start += 1;
            });
            reject!(h, {
                h.owners.rows[0].spelling.push('!');
            });
            reject!(h, {
                h.owners.rows[0].role = SourceTheoremRole::Lemma;
            });
            reject!(h, {
                h.owners.rows[0].status = SourceTheoremStatus::Open;
            });
            reject!(h, {
                h.owners.rows[0].recovery = SourceStatementRecovery::Degraded;
            });
            reject!(h, {
                h.statements.rows[0].owner = SourceTheoremOwnerId::new(1);
            });
            reject!(h, {
                h.statements.rows[0].context = SourceStatementContextId::new(1);
            });
            reject!(h, {
                h.statements.rows[0].formula = h.statements.rows[1].formula;
            });
            reject!(h, {
                h.statements.rows[1].formula = h.statements.rows[0].formula;
                h.candidate_facts.rows[1].formula = h.statements.rows[0].formula;
            });
            reject!(h, {
                h.statements.rows[0].site = h.statements.rows[1].site.clone();
            });
            reject!(h, {
                h.statements.rows[0].source_range.start += 1;
            });
            reject!(h, {
                h.statements.rows[1].source_ordinal = 4;
            });
            reject!(h, {
                h.statements.rows[0].spelling.push('!');
            });
            reject!(h, {
                h.statements.rows[0].kind = SourceStatementKind::Assumption;
            });
            reject!(h, {
                h.statements.rows[0].recovery = SourceStatementRecovery::Degraded;
            });
            reject!(h, {
                h.contexts.rows[0].statement = SourceStatementId::new(1);
            });
            reject!(h, {
                h.contexts.rows[1].binding_context = h.contexts.rows[0].binding_context;
            });
            reject!(h, {
                h.contexts.rows[0].source_range.start += 1;
            });
            reject!(h, {
                h.contexts.rows[0].visible_bindings = h.contexts.rows[1].visible_bindings.clone();
            });
            reject!(h, {
                h.input_facts.rows[0].statement = SourceStatementId::new(0);
            });
            reject!(h, {
                h.input_facts.rows[0].context = SourceStatementContextId::new(0);
            });
            reject!(h, {
                h.input_facts.rows[0].ordinal = 1;
            });
            reject!(h, {
                h.input_facts.rows[0].kind = SourceStatementInputFactKind::ReservedTypeGuard;
            });
            reject!(h, {
                h.input_facts.rows[0].binding = header_binding;
            });
            reject!(h, {
                h.input_facts.rows[0].uses = header_uses.clone();
            });
            reject!(h, {
                h.input_facts.rows[0].uses.clear();
            });
            reject!(h, {
                h.input_facts.rows[1].uses = header_uses.clone();
            });
            reject!(h, {
                h.input_facts.rows[0].binding = h.input_facts.rows[2].binding;
                h.input_facts.rows[0].uses = h.input_facts.rows[2].uses.clone();
            });
            reject!(h, {
                h.input_facts.rows[0].binding = header_binding;
                h.input_facts.rows[0].uses = header_uses.clone();
                h.contexts.rows[1].visible_bindings[0] = header_binding;
            });
            reject!(h, {
                h.input_facts.rows[0].statement = SourceStatementId::new(0);
                h.input_facts.rows[0].context = SourceStatementContextId::new(0);
                h.input_facts.rows[0].binding = header_binding;
                h.input_facts.rows[0].uses = header_uses.clone();
                h.contexts.rows[0].visible_bindings = vec![header_binding];
            });
            reject!(h, {
                h.candidate_facts.rows[0].statement = SourceStatementId::new(1);
            });
            reject!(h, {
                h.candidate_facts.rows[0].context = SourceStatementContextId::new(1);
            });
            reject!(h, {
                h.candidate_facts.rows[0].ordinal = 1;
            });
            reject!(h, {
                h.candidate_facts.rows[0].formula = h.candidate_facts.rows[1].formula;
            });
            reject!(h, {
                h.owners.rows.clear();
            });
            reject!(h, {
                h.owners.rows.push(h.owners.rows[0].clone());
            });
            reject!(h, {
                h.statements.rows.remove(1);
            });
            reject!(h, {
                h.statements.rows.push(h.statements.rows[0].clone());
            });
            reject!(h, {
                h.statements.rows.swap(0, 1);
            });
            reject!(h, {
                h.contexts.rows.remove(1);
            });
            reject!(h, {
                h.contexts.rows.push(h.contexts.rows[0].clone());
            });
            reject!(h, {
                h.contexts.rows.swap(0, 1);
            });
            reject!(h, {
                h.input_facts.rows.remove(1);
            });
            reject!(h, {
                h.input_facts.rows.push(h.input_facts.rows[0].clone());
            });
            reject!(h, {
                h.input_facts.rows.swap(0, 2);
            });
            reject!(h, {
                h.input_facts.rows.reverse();
                h.contexts.rows[1].visible_bindings.reverse();
                for (i, row) in h.input_facts.rows.iter_mut().enumerate() {
                    row.ordinal = i;
                }
            });
            reject!(h, {
                h.candidate_facts.rows.remove(1);
            });
            reject!(h, {
                h.candidate_facts
                    .rows
                    .push(h.candidate_facts.rows[0].clone());
            });
            reject!(h, {
                h.candidate_facts.rows.swap(0, 1);
            });
            let seal = statement.checked_owner();
            for (visibility, export) in [
                (Visibility::Private, ExportStatus::Exported),
                (Visibility::Public, ExportStatus::LocalOnly),
            ] {
                reject!(h, {
                    h.checked_owner =
                        CheckedStatementOwner::from_validated_parts_and_visibility_for_test(
                            seal.symbol().clone(),
                            seal.source_range(),
                            seal.origin().clone(),
                            visibility,
                            export,
                        );
                });
            }
            let range = seal.source_range();
            let mut path = seal.origin().structural_path().to_vec();
            path.push(99);
            let mut imports = ResolvedImports::new();
            let import = imports.push_import(ResolvedImport::new(
                source.arena().root(),
                range,
                "import owners;",
                None,
                ImportResolution::Resolved(typed.module_id().clone()),
                seal.origin().clone(),
            ));
            for origin in [
                seal.origin().clone().with_import_edge(import),
                SemanticOrigin::new(
                    typed.source_id(),
                    typed.module_id().clone(),
                    SourceAnchor::Range(range),
                    path,
                ),
                SemanticOrigin::new(
                    source_id(2),
                    typed.module_id().clone(),
                    SourceAnchor::Range(range),
                    seal.origin().structural_path().to_vec(),
                ),
                SemanticOrigin::new(
                    typed.source_id(),
                    ModuleId::new(PackageId::new("foreign"), ModulePath::new("owners")),
                    SourceAnchor::Range(range),
                    seal.origin().structural_path().to_vec(),
                ),
                SemanticOrigin::new(
                    typed.source_id(),
                    typed.module_id().clone(),
                    SourceAnchor::Range(SourceRange {
                        start: range.start + 1,
                        ..range
                    }),
                    seal.origin().structural_path().to_vec(),
                ),
                seal.origin().clone().recovered(),
            ] {
                reject!(h, {
                    h.checked_owner = CheckedStatementOwner::from_validated_parts_for_test(
                        seal.symbol().clone(),
                        range,
                        origin,
                    );
                });
            }
            let shifted_range = SourceRange {
                start: range.start + 1,
                ..range
            };
            reject!(h, {
                h.checked_owner = CheckedStatementOwner::from_validated_parts_for_test(
                    seal.symbol().clone(),
                    shifted_range,
                    seal.origin().clone(),
                );
            });
        },
    );
}

#[test]
fn task258_native_statement_coherent_label_swap_requires_fresh_resolver_seal() {
    use crate::typed_ast::tests::{task257b5_native_case, task257b5_native_replay};
    let text = include_str!("../../../tests/testdata/native_universal_statement.miz");
    let renamed = text.replace("NativeUniversalStatement", "AlphaXUniversalStatement");
    assert_eq!(text.len(), renamed.len());
    task257b5_native_case(
        text,
        |old, _, _, old_expressions, (old_symbols, old_source, old_scope)| {
            let old = old
                .unwrap()
                .with_source_statement_from_original(old_source, old_symbols, old_scope)
                .unwrap();
            let old_final = task257b5_native_replay(&old, old_expressions.clone()).unwrap();
            task257b5_native_case(
                &renamed,
                |new, _, _, expressions, (symbols, source, scope)| {
                    let new = new
                        .unwrap()
                        .with_source_statement_from_original(source, symbols, scope)
                        .unwrap();
                    let new_final = task257b5_native_replay(&new, expressions.clone()).unwrap();
                    let old_statement = old.source_statement().unwrap();
                    let statement = new.source_statement().unwrap();
                    assert_eq!(old.source_id(), new.source_id());
                    assert_eq!(old.module_id(), new.module_id());
                    assert_eq!(
                        old_statement.checked_owner.origin(),
                        statement.checked_owner.origin()
                    );
                    assert_eq!(
                        old_statement.checked_owner.source_range(),
                        statement.checked_owner.source_range()
                    );
                    assert_eq!(
                        old_statement.owner_contribution,
                        statement.owner_contribution
                    );
                    assert_ne!(
                        old_statement.checked_owner.symbol(),
                        statement.checked_owner.symbol()
                    );
                    assert_ne!(
                        old_statement.owners.rows[0].spelling,
                        statement.owners.rows[0].spelling
                    );
                    assert_eq!(
                        (
                            old_statement.binding_fingerprint(),
                            old_statement.primary_term_fingerprint(),
                            old_statement.atomic_formula_fingerprint(),
                            old_statement.composite_formula_fingerprint(),
                            old_statement.formula_composition_fingerprint()
                        ),
                        (
                            statement.binding_fingerprint(),
                            statement.primary_term_fingerprint(),
                            statement.atomic_formula_fingerprint(),
                            statement.composite_formula_fingerprint(),
                            statement.formula_composition_fingerprint()
                        )
                    );
                    let mut renamed_owner = old_statement.owners.rows[0].clone();
                    renamed_owner.symbol = statement.owners.rows[0].symbol.clone();
                    renamed_owner.spelling = statement.owners.rows[0].spelling.clone();
                    assert_eq!(renamed_owner, statement.owners.rows[0]);
                    assert_eq!(old.source_context(), new.source_context());
                    assert_eq!(old.source_type(), new.source_type());
                    assert_eq!(old.source_term(), new.source_term());
                    assert_eq!(old.source_atomic_formula(), new.source_atomic_formula());
                    assert_eq!(
                        old.source_composite_formula(),
                        new.source_composite_formula()
                    );
                    assert_eq!(
                        old.source_formula_composition(),
                        new.source_formula_composition()
                    );
                    let mut forged_statement = statement.clone();
                    forged_statement.checked_owner = old_statement.checked_owner.clone();
                    forged_statement.owners.rows[0].symbol =
                        old_statement.owners.rows[0].symbol.clone();
                    assert_eq!(
                        forged_statement.owners.rows[0].symbol,
                        *forged_statement.checked_owner.symbol()
                    );
                    assert_eq!(
                        forged_statement.owners.rows[0].spelling,
                        statement.owners.rows[0].spelling
                    );
                    assert_ne!(&forged_statement, statement);
                    assert_eq!(
                        forged_statement.validate_native_installation(
                            &new,
                            new.source_composite_formula().unwrap(),
                            new.source_formula_composition().unwrap()
                        ),
                        Err(SourceStatementError::DependencyMismatch)
                    );
                    let mut forged = new.clone();
                    forged.inject_source_statement_for_test(forged_statement);
                    assert_eq!(
                        forged.source_node_entries().unwrap().collect::<Vec<_>>(),
                        new.source_node_entries().unwrap().collect::<Vec<_>>()
                    );
                    let before = forged.clone();
                    assert_eq!(
                        task257b5_native_replay(&forged, expressions.clone()),
                        Err(ResolvedTypedAstError::InvalidSourceStatement)
                    );
                    assert_eq!(forged, before);
                    assert_eq!(
                        task257b5_native_replay(&new, expressions).unwrap(),
                        new_final
                    );
                    assert_eq!(
                        task257b5_native_replay(&old, old_expressions).unwrap(),
                        old_final
                    );
                },
            );
        },
    );
}
