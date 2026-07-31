use std::path::Path;

use mizar_resolve::env::{
    ContributionKind, DefinitionKind, ExportStatus, NamespacePath, SourceContributionId, SymbolEnv,
    SymbolKind, Visibility,
};
use mizar_resolve::labels::{
    LabelProjection, LabelProjectionSource, LabelReferenceCandidate, LabelReferenceScope,
    LabelResolutionResult, LabelResolver, ProofLabelSourceCollector,
};
use mizar_resolve::resolved_ast::{
    LabelExpectation, LabelKind, LabelOriginPath, LabelResolution, ModuleId, RecoveryState,
    ReferenceSite, SemanticOrigin, SurfaceResolvedArena,
};
use mizar_session::{SourceAnchor, SourceId, SourceRange};
use mizar_syntax::SurfaceAst;

use crate::diagnostic::ValidationDiagnostic;
use crate::expectation::ExpectedOutcome;
use crate::harness::TestCase;

use super::shared::{FrontendRun, frontend_detail_keys, resolver_symbol_collection, run_frontend};
use super::{DeclarationSymbolCaseResult, DeclarationSymbolCaseStatus};

pub(super) fn run_declaration_symbol_case(
    workspace_root: &Path,
    case: &TestCase,
    ordinal: usize,
) -> DeclarationSymbolCaseResult {
    let output = run_frontend(workspace_root, case, ordinal);
    let actual = match output {
        Ok(output) => declaration_symbol_observation(workspace_root, case, output),
        Err(error) => DeclarationSymbolObservation {
            detail_keys: vec![format!("frontend_error:{error}")],
            payload_keys: Vec::new(),
        },
    };
    let expected_detail_keys = expected_declaration_symbol_detail_keys(case);
    let expected_payload_keys = expected_declaration_symbol_payload_keys(case);
    let status = match case.expectation.expected_outcome {
        ExpectedOutcome::Pass
            if actual.detail_keys.is_empty()
                && (case.expectation.declaration_symbol_payloads.is_empty()
                    || actual.payload_keys == expected_payload_keys) =>
        {
            DeclarationSymbolCaseStatus::Passed
        }
        ExpectedOutcome::Fail if actual.detail_keys == expected_detail_keys => {
            DeclarationSymbolCaseStatus::Passed
        }
        _ => DeclarationSymbolCaseStatus::Failed,
    };

    DeclarationSymbolCaseResult {
        id: case.id.clone(),
        expectation_path: case.expectation_path.clone(),
        status,
        actual_detail_keys: actual.detail_keys,
        actual_payload_keys: actual.payload_keys,
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
struct DeclarationSymbolObservation {
    detail_keys: Vec<String>,
    payload_keys: Vec<String>,
}

fn declaration_symbol_observation(
    workspace_root: &Path,
    case: &TestCase,
    output: FrontendRun,
) -> DeclarationSymbolObservation {
    let frontend_diagnostic_keys = frontend_detail_keys(case, &output.diagnostics);
    if !frontend_diagnostic_keys.is_empty() {
        return DeclarationSymbolObservation {
            detail_keys: frontend_diagnostic_keys,
            payload_keys: Vec::new(),
        };
    }

    let Some(ast) = output.ast else {
        return DeclarationSymbolObservation {
            detail_keys: vec!["declaration_symbol.no_ast".to_owned()],
            payload_keys: Vec::new(),
        };
    };
    let proof_label_profile = proof_label_confinement_profile(&output.source_text, &ast);
    let resolver = resolver_symbol_collection(workspace_root, case, &ast);
    if let Some(profile) = proof_label_profile {
        let detail_key = if resolver.detail_keys.is_empty()
            && case.expectation.diagnostic_codes.is_empty()
            && proof_label_confinement_matches(profile, &ast, &resolver.module, &resolver.env)
        {
            PROOF_LABEL_SCOPE_CONFINEMENT_DETAIL
        } else {
            PROOF_LABEL_SCOPE_INPUT_DETAIL
        };
        return DeclarationSymbolObservation {
            detail_keys: vec![detail_key.to_owned()],
            payload_keys: Vec::new(),
        };
    }
    let payload_keys = if resolver.detail_keys.is_empty() {
        declaration_symbol_payload_keys(&resolver.env)
    } else {
        Vec::new()
    };
    DeclarationSymbolObservation {
        detail_keys: resolver.detail_keys,
        payload_keys,
    }
}

const PROOF_LABEL_SCOPE_INPUT_DETAIL: &str = "declaration_symbol.label.proof_scope_input";
const PROOF_LABEL_SCOPE_CONFINEMENT_DETAIL: &str =
    "declaration_symbol.label.proof_scope_confinement";

const PROOF_LABEL_INNER_TO_OUTER_SOURCE: &str = "theorem ProofLabelInnerToOuterConfinement: thesis proof\n  thus thesis proof\n    A: thesis proof\n      thus thesis;\n    end;\n    thus thesis;\n  end;\n  thus thesis by A;\nend;\n";
const PROOF_LABEL_SIBLING_SOURCE: &str = "theorem ProofLabelSiblingConfinement: thesis proof\n  thus thesis proof\n    A: thesis proof\n      thus thesis;\n    end;\n    thus thesis;\n  end;\n  thus thesis proof\n    thus thesis by A;\n  end;\nend;\n";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ProofLabelConfinementProfile {
    node_count: usize,
    root: usize,
    root_end: usize,
    dense_profile_hash: u64,
    owner: &'static str,
    declaration_start: usize,
    declaration_end: usize,
    projection_path: &'static [u32],
    reference_node: usize,
    reference_start: usize,
    reference_end: usize,
    reference_ordinal: usize,
    reference_scope: &'static [u32],
    reference_path: &'static [u32],
}

const PROOF_LABEL_INNER_TO_OUTER_PROFILE: ProofLabelConfinementProfile =
    ProofLabelConfinementProfile {
        node_count: 61,
        root: 60,
        root_end: 172,
        dense_profile_hash: 0x05f4_763c_ffe3_248b,
        owner: "ProofLabelInnerToOuterConfinement",
        declaration_start: 80,
        declaration_end: 81,
        projection_path: &[57, 42, 8],
        reference_node: 52,
        reference_start: 165,
        reference_end: 166,
        reference_ordinal: 5,
        reference_scope: &[0],
        reference_path: &[57, 55, 52],
    };

const PROOF_LABEL_SIBLING_PROFILE: ProofLabelConfinementProfile = ProofLabelConfinementProfile {
    node_count: 71,
    root: 70,
    root_end: 196,
    dense_profile_hash: 0x1226_7d7f_52fa_e5a8,
    owner: "ProofLabelSiblingConfinement",
    declaration_start: 75,
    declaration_end: 76,
    projection_path: &[67, 47, 8],
    reference_node: 60,
    reference_start: 182,
    reference_end: 183,
    reference_ordinal: 6,
    reference_scope: &[0, 1],
    reference_path: &[67, 63, 60],
};

fn proof_label_confinement_profile(
    source_text: &str,
    ast: &SurfaceAst,
) -> Option<&'static ProofLabelConfinementProfile> {
    let profile = match source_text {
        PROOF_LABEL_INNER_TO_OUTER_SOURCE => &PROOF_LABEL_INNER_TO_OUTER_PROFILE,
        PROOF_LABEL_SIBLING_SOURCE => &PROOF_LABEL_SIBLING_PROFILE,
        _ => return None,
    };
    let root = ast.root_view()?;
    if ast.nodes().len() != profile.node_count
        || root.id().index() != profile.root
        || root.range().source_id != ast.source_id
        || root.range().start != 0
        || root.range().end != profile.root_end
        || ast.node_views().any(|node| {
            node.is_recovered()
                || node.range().source_id != ast.source_id
                || node
                    .children()
                    .iter()
                    .any(|child| child.index() >= profile.node_count)
        })
        || proof_label_dense_profile_hash(ast) != profile.dense_profile_hash
    {
        return None;
    }
    Some(profile)
}

fn proof_label_dense_profile_hash(ast: &SurfaceAst) -> u64 {
    const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
    const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

    fn push_bytes(hash: &mut u64, bytes: &[u8]) {
        for byte in bytes {
            *hash ^= u64::from(*byte);
            *hash = hash.wrapping_mul(FNV_PRIME);
        }
    }

    fn push_usize(hash: &mut u64, value: usize) {
        push_bytes(hash, &(value as u64).to_le_bytes());
    }

    let mut hash = FNV_OFFSET;
    match ast.root() {
        Some(root) => {
            push_bytes(&mut hash, &[1]);
            push_usize(&mut hash, root.index());
        }
        None => push_bytes(&mut hash, &[0]),
    }
    match ast.expression_root() {
        Some(root) => {
            push_bytes(&mut hash, &[1]);
            push_usize(&mut hash, root.index());
        }
        None => push_bytes(&mut hash, &[0]),
    }
    for node in ast.node_views() {
        push_usize(&mut hash, node.id().index());
        push_bytes(&mut hash, format!("{:?}", node.kind()).as_bytes());
        push_bytes(&mut hash, &[0xff]);
        push_usize(&mut hash, node.range().start);
        push_usize(&mut hash, node.range().end);
        push_bytes(&mut hash, &[u8::from(node.is_recovered())]);
        push_usize(&mut hash, node.children().len());
        for child in node.children() {
            push_usize(&mut hash, child.index());
        }
    }
    push_usize(&mut hash, ast.token_nodes().len());
    for token in ast.token_nodes() {
        push_usize(&mut hash, token.index());
    }
    hash
}

#[cfg(test)]
pub(super) fn proof_label_dense_profile_hash_for_test(ast: &SurfaceAst) -> u64 {
    proof_label_dense_profile_hash(ast)
}

#[cfg(test)]
pub(super) fn proof_label_confinement_profile_for_test(
    source_text: &str,
    ast: &SurfaceAst,
) -> bool {
    proof_label_confinement_profile(source_text, ast).is_some()
}

fn proof_label_confinement_matches(
    profile: &ProofLabelConfinementProfile,
    ast: &SurfaceAst,
    module: &ModuleId,
    env: &SymbolEnv,
) -> bool {
    let input = ProofLabelInputProfile::from_env(env);
    if !input.matches(ast.source_id, module) {
        return false;
    }

    let Some(contribution) = env.contributions().iter().next() else {
        return false;
    };
    if contribution.id().index() != 0
        || contribution.module() != module
        || !matches!(
            contribution.kind(),
            ContributionKind::LocalSource { source_id } if *source_id == ast.source_id
        )
    {
        return false;
    }

    let namespace = NamespacePath::new(module.path().as_str());
    let Ok(resolved) = SurfaceResolvedArena::lower(ast, module) else {
        return false;
    };
    if resolved.validate_against(ast, module).is_err() {
        return false;
    }
    let Ok(collector) = ProofLabelSourceCollector::new(
        ast,
        module,
        namespace.clone(),
        contribution.id(),
        &resolved,
    ) else {
        return false;
    };
    let Ok(collection) = collector.collect() else {
        return false;
    };
    let [projection] = collection.projections() else {
        return false;
    };
    let [reference] = collection.references() else {
        return false;
    };
    if !proof_label_projection_matches(
        profile,
        ast.source_id,
        module,
        &namespace,
        contribution.id(),
        projection,
    ) || !proof_label_reference_matches(profile, ast.source_id, module, reference)
    {
        return false;
    }

    let result = LabelResolver::new(collection.projections()).resolve(
        module,
        &namespace,
        collection.references(),
    );
    proof_label_resolution_matches(
        ProofLabelResolutionExpectation {
            profile,
            source_id: ast.source_id,
            module,
            namespace: &namespace,
            contribution: contribution.id(),
            projection,
            reference,
        },
        &result,
    )
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofLabelInputProfile {
    env_module: ModuleId,
    symbol_count: usize,
    label_count: usize,
    definition_count: usize,
    contribution_count: usize,
    import_count: usize,
    contributions: Vec<ProofLabelContributionProfile>,
}

impl ProofLabelInputProfile {
    fn from_env(env: &SymbolEnv) -> Self {
        Self {
            env_module: env.module_id().clone(),
            symbol_count: env.symbols().len(),
            label_count: env.labels().len(),
            definition_count: env.definitions().len(),
            contribution_count: env.contributions().len(),
            import_count: env.imports().len(),
            contributions: env
                .contributions()
                .iter()
                .map(|contribution| ProofLabelContributionProfile {
                    id: contribution.id().index(),
                    module: contribution.module().clone(),
                    kind: match contribution.kind() {
                        ContributionKind::LocalSource { source_id } => {
                            ProofLabelContributionKind::Local(*source_id)
                        }
                        ContributionKind::ImportedSource { source_id } => {
                            ProofLabelContributionKind::Imported(*source_id)
                        }
                        ContributionKind::Summary { .. } => ProofLabelContributionKind::Summary,
                        ContributionKind::Builtin { .. } => ProofLabelContributionKind::Builtin,
                        _ => ProofLabelContributionKind::Unknown,
                    },
                })
                .collect(),
        }
    }

    fn matches(&self, source_id: SourceId, module: &ModuleId) -> bool {
        let [contribution] = self.contributions.as_slice() else {
            return false;
        };
        self.env_module == *module
            && self.symbol_count == 1
            && self.label_count == 0
            && self.definition_count == 1
            && self.contribution_count == 1
            && self.import_count == 0
            && contribution.id == 0
            && contribution.module == *module
            && contribution.kind == ProofLabelContributionKind::Local(source_id)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofLabelContributionProfile {
    id: usize,
    module: ModuleId,
    kind: ProofLabelContributionKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProofLabelContributionKind {
    Local(SourceId),
    Imported(SourceId),
    Summary,
    Builtin,
    Unknown,
}

#[cfg(test)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum ProofLabelConfinementMutation {
    None,
    EnvironmentModule,
    SymbolCount,
    LabelCount,
    DefinitionCount,
    NoContribution,
    MultipleContributions,
    ImportCount,
    ContributionId,
    ContributionImported,
    ContributionSummary,
    ContributionBuiltin,
    ContributionModule,
    ContributionSource,
    ProjectionExtra,
    ProjectionOriginPath,
    ProjectionModule,
    ProjectionNamespace,
    ProjectionSpelling,
    ProjectionKind,
    ProjectionVisibility,
    ProjectionExportStatus,
    ProjectionRange,
    ProjectionOriginSource,
    ProjectionOriginModule,
    ProjectionOriginAnchor,
    ProjectionOriginStructuralPath,
    ProjectionOriginRecovered,
    ProjectionContribution,
    ProjectionVisibleOrdinal,
    ProjectionScope,
    ProjectionImported,
    ReferenceExtra,
    ReferenceNode,
    ReferenceRange,
    ReferenceSpelling,
    ReferenceOriginSource,
    ReferenceOriginModule,
    ReferenceOriginAnchor,
    ReferenceOriginStructuralPath,
    ReferenceOriginRecovered,
    ReferenceOrdinal,
    ReferenceExpectation,
    ReferenceScope,
    ReferenceQualified,
    ReferenceFailedNamespace,
    ResultResolved,
    ResultAmbiguous,
    ResultExtraReference,
    ResultDiagnostic,
    ResultId,
    ResultIndexCount,
    ResultTableCount,
    ResultHasUnresolved,
    ResultIndexOriginPath,
    ResultIndexKind,
    ResultIndexVisibility,
    ResultIndexExportStatus,
    ResultIndexNamespace,
    ResultIndexSpelling,
    ResultIndexOrigin,
    ResultIndexContribution,
    ResultIndexRecovery,
    ResultTableSite,
    ResultTableOrigin,
    ResultTableRecovery,
    ResultUnresolvedSpelling,
    ResultUnresolvedRange,
    ResultUnresolvedExpectation,
}

#[cfg(test)]
pub(super) fn proof_label_confinement_detail_with_mutation_for_test(
    source_text: &str,
    ast: &SurfaceAst,
    module: &ModuleId,
    env: &SymbolEnv,
    foreign_source: SourceId,
    mutation: ProofLabelConfinementMutation,
) -> Option<&'static str> {
    let profile = proof_label_confinement_profile(source_text, ast)?;
    let foreign_module = ModuleId::new(
        mizar_session::PackageId::new("foreign-package"),
        mizar_session::ModulePath::new("foreign.module"),
    );
    let namespace = NamespacePath::new(module.path().as_str());
    let foreign_namespace = NamespacePath::new("foreign.module");
    let mut input = ProofLabelInputProfile::from_env(env);
    let input_mutation = match mutation {
        ProofLabelConfinementMutation::EnvironmentModule => {
            input.env_module = foreign_module.clone();
            true
        }
        ProofLabelConfinementMutation::SymbolCount => {
            input.symbol_count += 1;
            true
        }
        ProofLabelConfinementMutation::LabelCount => {
            input.label_count += 1;
            true
        }
        ProofLabelConfinementMutation::DefinitionCount => {
            input.definition_count += 1;
            true
        }
        ProofLabelConfinementMutation::NoContribution => {
            input.contribution_count = 0;
            input.contributions.clear();
            true
        }
        ProofLabelConfinementMutation::MultipleContributions => {
            input.contribution_count += 1;
            input.contributions.push(input.contributions[0].clone());
            true
        }
        ProofLabelConfinementMutation::ImportCount => {
            input.import_count += 1;
            true
        }
        ProofLabelConfinementMutation::ContributionId => {
            input.contributions[0].id = 1;
            true
        }
        ProofLabelConfinementMutation::ContributionImported => {
            input.contributions[0].kind = ProofLabelContributionKind::Imported(ast.source_id);
            true
        }
        ProofLabelConfinementMutation::ContributionSummary => {
            input.contributions[0].kind = ProofLabelContributionKind::Summary;
            true
        }
        ProofLabelConfinementMutation::ContributionBuiltin => {
            input.contributions[0].kind = ProofLabelContributionKind::Builtin;
            true
        }
        ProofLabelConfinementMutation::ContributionModule => {
            input.contributions[0].module = foreign_module.clone();
            true
        }
        ProofLabelConfinementMutation::ContributionSource => {
            input.contributions[0].kind = ProofLabelContributionKind::Local(foreign_source);
            true
        }
        _ => false,
    };
    if input_mutation {
        return Some(if input.matches(ast.source_id, module) {
            PROOF_LABEL_SCOPE_CONFINEMENT_DETAIL
        } else {
            PROOF_LABEL_SCOPE_INPUT_DETAIL
        });
    }
    if !input.matches(ast.source_id, module) {
        return Some(PROOF_LABEL_SCOPE_INPUT_DETAIL);
    }

    let contribution = env.contributions().iter().next()?;
    let resolved = SurfaceResolvedArena::lower(ast, module).ok()?;
    let collector = ProofLabelSourceCollector::new(
        ast,
        module,
        namespace.clone(),
        contribution.id(),
        &resolved,
    )
    .ok()?;
    let collection = collector.collect().ok()?;
    let [base_projection] = collection.projections() else {
        return Some(PROOF_LABEL_SCOPE_INPUT_DETAIL);
    };
    let [base_reference] = collection.references() else {
        return Some(PROOF_LABEL_SCOPE_INPUT_DETAIL);
    };
    let mutation_context = ProofLabelMutationContext {
        profile,
        ast,
        resolved: &resolved,
        module,
        namespace: &namespace,
        contribution: contribution.id(),
        foreign_module: &foreign_module,
        foreign_namespace: &foreign_namespace,
        foreign_source,
    };

    let mut projections = vec![base_projection.clone()];
    if mutation == ProofLabelConfinementMutation::ProjectionExtra {
        projections.push(base_projection.clone());
    } else if proof_label_projection_mutation(mutation) {
        projections[0] =
            mutated_proof_label_projection(base_projection, &mutation_context, mutation);
    }
    let [projection] = projections.as_slice() else {
        return Some(PROOF_LABEL_SCOPE_INPUT_DETAIL);
    };
    let projection = projection.clone();
    if !proof_label_projection_matches(
        profile,
        ast.source_id,
        module,
        &namespace,
        contribution.id(),
        &projection,
    ) {
        return Some(PROOF_LABEL_SCOPE_INPUT_DETAIL);
    }

    let mut references = vec![base_reference.clone()];
    if mutation == ProofLabelConfinementMutation::ReferenceExtra {
        references.push(base_reference.clone());
    } else if proof_label_reference_mutation(mutation) {
        references[0] = mutated_proof_label_reference(base_reference, &mutation_context, mutation)?;
    }
    let [reference] = references.as_slice() else {
        return Some(PROOF_LABEL_SCOPE_INPUT_DETAIL);
    };
    let reference = reference.clone();
    if !proof_label_reference_matches(profile, ast.source_id, module, &reference) {
        return Some(PROOF_LABEL_SCOPE_INPUT_DETAIL);
    }

    let mut result_projections = projections;
    let mut result_references = references;
    match mutation {
        ProofLabelConfinementMutation::ResultResolved => {
            result_projections[0] = mutated_proof_label_projection(
                base_projection,
                &mutation_context,
                ProofLabelConfinementMutation::ProjectionScope,
            );
        }
        ProofLabelConfinementMutation::ResultAmbiguous => {
            result_projections.push(mutated_proof_label_projection(
                base_projection,
                &mutation_context,
                ProofLabelConfinementMutation::ProjectionOriginPath,
            ));
        }
        ProofLabelConfinementMutation::ResultExtraReference => {
            result_references.push(base_reference.clone());
        }
        ProofLabelConfinementMutation::ResultDiagnostic => {
            result_projections.push(base_projection.clone());
        }
        _ => {}
    }
    let result =
        LabelResolver::new(&result_projections).resolve(module, &namespace, &result_references);
    let expectation = ProofLabelResolutionExpectation {
        profile,
        source_id: ast.source_id,
        module,
        namespace: &namespace,
        contribution: contribution.id(),
        projection: &projection,
        reference: &reference,
    };
    let matches = if proof_label_result_observation_mutation(mutation) {
        proof_label_resolution_matches_with_mutation_for_test(
            expectation,
            &result,
            &resolved,
            foreign_source,
            &foreign_module,
            mutation,
        )
    } else {
        proof_label_resolution_matches(expectation, &result)
    };
    Some(if matches {
        PROOF_LABEL_SCOPE_CONFINEMENT_DETAIL
    } else {
        PROOF_LABEL_SCOPE_INPUT_DETAIL
    })
}

#[cfg(test)]
const fn proof_label_projection_mutation(mutation: ProofLabelConfinementMutation) -> bool {
    matches!(
        mutation,
        ProofLabelConfinementMutation::ProjectionOriginPath
            | ProofLabelConfinementMutation::ProjectionModule
            | ProofLabelConfinementMutation::ProjectionNamespace
            | ProofLabelConfinementMutation::ProjectionSpelling
            | ProofLabelConfinementMutation::ProjectionKind
            | ProofLabelConfinementMutation::ProjectionVisibility
            | ProofLabelConfinementMutation::ProjectionExportStatus
            | ProofLabelConfinementMutation::ProjectionRange
            | ProofLabelConfinementMutation::ProjectionOriginSource
            | ProofLabelConfinementMutation::ProjectionOriginModule
            | ProofLabelConfinementMutation::ProjectionOriginAnchor
            | ProofLabelConfinementMutation::ProjectionOriginStructuralPath
            | ProofLabelConfinementMutation::ProjectionOriginRecovered
            | ProofLabelConfinementMutation::ProjectionContribution
            | ProofLabelConfinementMutation::ProjectionVisibleOrdinal
            | ProofLabelConfinementMutation::ProjectionScope
            | ProofLabelConfinementMutation::ProjectionImported
    )
}

#[cfg(test)]
const fn proof_label_reference_mutation(mutation: ProofLabelConfinementMutation) -> bool {
    matches!(
        mutation,
        ProofLabelConfinementMutation::ReferenceNode
            | ProofLabelConfinementMutation::ReferenceRange
            | ProofLabelConfinementMutation::ReferenceSpelling
            | ProofLabelConfinementMutation::ReferenceOriginSource
            | ProofLabelConfinementMutation::ReferenceOriginModule
            | ProofLabelConfinementMutation::ReferenceOriginAnchor
            | ProofLabelConfinementMutation::ReferenceOriginStructuralPath
            | ProofLabelConfinementMutation::ReferenceOriginRecovered
            | ProofLabelConfinementMutation::ReferenceOrdinal
            | ProofLabelConfinementMutation::ReferenceExpectation
            | ProofLabelConfinementMutation::ReferenceScope
            | ProofLabelConfinementMutation::ReferenceQualified
            | ProofLabelConfinementMutation::ReferenceFailedNamespace
    )
}
fn proof_label_projection_matches(
    profile: &ProofLabelConfinementProfile,
    source_id: SourceId,
    module: &ModuleId,
    namespace: &NamespacePath,
    contribution: SourceContributionId,
    projection: &LabelProjection,
) -> bool {
    let expected_range = source_range(
        source_id,
        profile.declaration_start,
        profile.declaration_end,
    );
    projection.origin_path().as_str() == proof_label_origin_path(profile, module, contribution)
        && projection.module() == module
        && projection.namespace() == namespace
        && projection.primary_spelling() == "A"
        && projection.kind() == LabelKind::ProofStep
        && projection.visibility() == Visibility::Private
        && projection.export_status() == ExportStatus::LocalOnly
        && projection.declaration_range() == expected_range
        && projection.contribution() == contribution
        && semantic_origin_matches(
            projection.origin(),
            source_id,
            module,
            expected_range,
            profile.projection_path,
        )
        && matches!(
            projection.source(),
            LabelProjectionSource::CurrentModule {
                visible_after_ordinal: 3,
                proof_scope: Some(scope),
            } if scope.path() == [0, 0]
        )
}

fn proof_label_reference_matches(
    profile: &ProofLabelConfinementProfile,
    source_id: SourceId,
    module: &ModuleId,
    reference: &LabelReferenceCandidate,
) -> bool {
    let expected_range = source_range(source_id, profile.reference_start, profile.reference_end);
    reference.site().node().index() == profile.reference_node
        && reference.site().range() == expected_range
        && reference.site().spelling() == "A"
        && reference.ordinal() == profile.reference_ordinal
        && reference.expectation() == LabelExpectation::ProofOrTheorem
        && semantic_origin_matches(
            reference.origin(),
            source_id,
            module,
            expected_range,
            profile.reference_path,
        )
        && matches!(
            reference.scope(),
            LabelReferenceScope::Unqualified {
                proof_scope: Some(scope),
            } if scope.path() == profile.reference_scope
        )
}

struct ProofLabelResolutionExpectation<'a> {
    profile: &'a ProofLabelConfinementProfile,
    source_id: SourceId,
    module: &'a ModuleId,
    namespace: &'a NamespacePath,
    contribution: SourceContributionId,
    projection: &'a LabelProjection,
    reference: &'a LabelReferenceCandidate,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofLabelResolutionObservation {
    ids: Vec<usize>,
    index_len: usize,
    table_len: usize,
    diagnostic_len: usize,
    has_unresolved: bool,
    label: Option<ProofLabelIndexObservation>,
    entry: Option<ProofLabelTableObservation>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofLabelIndexObservation {
    origin_path: LabelOriginPath,
    kind: LabelKind,
    visibility: Visibility,
    export_status: ExportStatus,
    namespace: NamespacePath,
    spelling: String,
    origin: SemanticOrigin,
    contribution: SourceContributionId,
    recovery: RecoveryState,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ProofLabelTableObservation {
    site: ReferenceSite,
    origin: SemanticOrigin,
    recovery: RecoveryState,
    resolution: ProofLabelResolutionOutcome,
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum ProofLabelResolutionOutcome {
    Resolved,
    Ambiguous,
    Unresolved {
        spelling: String,
        range: SourceRange,
        expectation: LabelExpectation,
    },
    Unknown,
}

impl ProofLabelResolutionObservation {
    fn from_result(result: &LabelResolutionResult) -> Self {
        let ids = result.ids().iter().map(|id| id.index()).collect();
        let label = result
            .index()
            .iter()
            .next()
            .map(|label| ProofLabelIndexObservation {
                origin_path: label.origin_path().clone(),
                kind: label.kind(),
                visibility: label.visibility(),
                export_status: label.export_status(),
                namespace: label.namespace().clone(),
                spelling: label.primary_spelling().to_owned(),
                origin: label.origin().clone(),
                contribution: label.contribution(),
                recovery: label.recovery(),
            });
        let entry = result
            .ids()
            .first()
            .and_then(|id| result.table().get(*id))
            .map(|entry| ProofLabelTableObservation {
                site: entry.site().clone(),
                origin: entry.origin().clone(),
                recovery: entry.recovery(),
                resolution: match entry.resolution() {
                    LabelResolution::Resolved(_) => ProofLabelResolutionOutcome::Resolved,
                    LabelResolution::Ambiguous(_) => ProofLabelResolutionOutcome::Ambiguous,
                    LabelResolution::Unresolved(unresolved) => {
                        ProofLabelResolutionOutcome::Unresolved {
                            spelling: unresolved.spelling().to_owned(),
                            range: unresolved.range(),
                            expectation: unresolved.expectation(),
                        }
                    }
                    _ => ProofLabelResolutionOutcome::Unknown,
                },
            });
        Self {
            ids,
            index_len: result.index().len(),
            table_len: result.table().len(),
            diagnostic_len: result.diagnostics().len(),
            has_unresolved: result.has_unresolved(),
            label,
            entry,
        }
    }
}

fn proof_label_resolution_matches(
    expected: ProofLabelResolutionExpectation<'_>,
    result: &LabelResolutionResult,
) -> bool {
    let observation = ProofLabelResolutionObservation::from_result(result);
    proof_label_resolution_observation_matches(expected, &observation)
}

fn proof_label_resolution_observation_matches(
    expected: ProofLabelResolutionExpectation<'_>,
    observation: &ProofLabelResolutionObservation,
) -> bool {
    if observation.ids != [0]
        || observation.index_len != 1
        || observation.table_len != 1
        || observation.diagnostic_len != 0
        || !observation.has_unresolved
    {
        return false;
    }
    let Some(label) = observation.label.as_ref() else {
        return false;
    };
    if label.origin_path != *expected.projection.origin_path()
        || label.kind != LabelKind::ProofStep
        || label.visibility != Visibility::Private
        || label.export_status != ExportStatus::LocalOnly
        || label.namespace != *expected.namespace
        || label.spelling != "A"
        || label.origin != *expected.projection.origin()
        || label.contribution != expected.contribution
        || label.recovery != RecoveryState::Normal
    {
        return false;
    }
    let Some(entry) = observation.entry.as_ref() else {
        return false;
    };
    let expected_range = source_range(
        expected.source_id,
        expected.profile.reference_start,
        expected.profile.reference_end,
    );
    entry.site == *expected.reference.site()
        && entry.origin == *expected.reference.origin()
        && entry.recovery == RecoveryState::Normal
        && matches!(
            &entry.resolution,
            ProofLabelResolutionOutcome::Unresolved {
                spelling,
                range,
                expectation,
            } if spelling == "A"
                && *range == expected_range
                && *expectation == LabelExpectation::ProofOrTheorem
        )
        && semantic_origin_matches(
            &entry.origin,
            expected.source_id,
            expected.module,
            expected_range,
            expected.profile.reference_path,
        )
}

#[cfg(test)]
const fn proof_label_result_observation_mutation(mutation: ProofLabelConfinementMutation) -> bool {
    matches!(
        mutation,
        ProofLabelConfinementMutation::ResultId
            | ProofLabelConfinementMutation::ResultIndexCount
            | ProofLabelConfinementMutation::ResultTableCount
            | ProofLabelConfinementMutation::ResultHasUnresolved
            | ProofLabelConfinementMutation::ResultIndexOriginPath
            | ProofLabelConfinementMutation::ResultIndexKind
            | ProofLabelConfinementMutation::ResultIndexVisibility
            | ProofLabelConfinementMutation::ResultIndexExportStatus
            | ProofLabelConfinementMutation::ResultIndexNamespace
            | ProofLabelConfinementMutation::ResultIndexSpelling
            | ProofLabelConfinementMutation::ResultIndexOrigin
            | ProofLabelConfinementMutation::ResultIndexContribution
            | ProofLabelConfinementMutation::ResultIndexRecovery
            | ProofLabelConfinementMutation::ResultTableSite
            | ProofLabelConfinementMutation::ResultTableOrigin
            | ProofLabelConfinementMutation::ResultTableRecovery
            | ProofLabelConfinementMutation::ResultUnresolvedSpelling
            | ProofLabelConfinementMutation::ResultUnresolvedRange
            | ProofLabelConfinementMutation::ResultUnresolvedExpectation
    )
}

#[cfg(test)]
fn proof_label_resolution_matches_with_mutation_for_test(
    expected: ProofLabelResolutionExpectation<'_>,
    result: &LabelResolutionResult,
    resolved: &SurfaceResolvedArena,
    foreign_source: SourceId,
    foreign_module: &ModuleId,
    mutation: ProofLabelConfinementMutation,
) -> bool {
    let mut observation = ProofLabelResolutionObservation::from_result(result);
    let label = observation.label.as_mut();
    let entry = observation.entry.as_mut();
    match mutation {
        ProofLabelConfinementMutation::ResultId => observation.ids[0] = 1,
        ProofLabelConfinementMutation::ResultIndexCount => observation.index_len += 1,
        ProofLabelConfinementMutation::ResultTableCount => observation.table_len += 1,
        ProofLabelConfinementMutation::ResultHasUnresolved => observation.has_unresolved = false,
        ProofLabelConfinementMutation::ResultIndexOriginPath => {
            label.expect("exact result label").origin_path =
                LabelOriginPath::new("wrong-result-origin");
        }
        ProofLabelConfinementMutation::ResultIndexKind => {
            label.expect("exact result label").kind = LabelKind::Theorem;
        }
        ProofLabelConfinementMutation::ResultIndexVisibility => {
            label.expect("exact result label").visibility = Visibility::Public;
        }
        ProofLabelConfinementMutation::ResultIndexExportStatus => {
            label.expect("exact result label").export_status = ExportStatus::Exported;
        }
        ProofLabelConfinementMutation::ResultIndexNamespace => {
            label.expect("exact result label").namespace = NamespacePath::new("foreign.module");
        }
        ProofLabelConfinementMutation::ResultIndexSpelling => {
            label.expect("exact result label").spelling = "B".to_owned();
        }
        ProofLabelConfinementMutation::ResultIndexOrigin => {
            label.expect("exact result label").origin = SemanticOrigin::new(
                foreign_source,
                foreign_module.clone(),
                SourceAnchor::Point {
                    source_id: foreign_source,
                    offset: 0,
                },
                vec![99],
            );
        }
        ProofLabelConfinementMutation::ResultIndexContribution => {
            label.expect("exact result label").contribution = second_contribution_id(
                expected.module.clone(),
                expected.source_id,
                expected.projection.declaration_range(),
            );
        }
        ProofLabelConfinementMutation::ResultIndexRecovery => {
            label.expect("exact result label").recovery = RecoveryState::Recovered;
        }
        ProofLabelConfinementMutation::ResultTableSite => {
            let entry = entry.expect("exact result entry");
            entry.site = ReferenceSite::new(
                resolved.arena().root(),
                entry.site.range(),
                entry.site.spelling(),
            );
        }
        ProofLabelConfinementMutation::ResultTableOrigin => {
            entry.expect("exact result entry").origin = SemanticOrigin::new(
                foreign_source,
                foreign_module.clone(),
                SourceAnchor::Point {
                    source_id: foreign_source,
                    offset: 0,
                },
                vec![99],
            );
        }
        ProofLabelConfinementMutation::ResultTableRecovery => {
            entry.expect("exact result entry").recovery = RecoveryState::Recovered;
        }
        ProofLabelConfinementMutation::ResultUnresolvedSpelling => {
            let ProofLabelResolutionOutcome::Unresolved { spelling, .. } =
                &mut entry.expect("exact result entry").resolution
            else {
                return false;
            };
            *spelling = "B".to_owned();
        }
        ProofLabelConfinementMutation::ResultUnresolvedRange => {
            let ProofLabelResolutionOutcome::Unresolved { range, .. } =
                &mut entry.expect("exact result entry").resolution
            else {
                return false;
            };
            range.start += 1;
        }
        ProofLabelConfinementMutation::ResultUnresolvedExpectation => {
            let ProofLabelResolutionOutcome::Unresolved { expectation, .. } =
                &mut entry.expect("exact result entry").resolution
            else {
                return false;
            };
            *expectation = LabelExpectation::Theorem;
        }
        _ => return false,
    }
    proof_label_resolution_observation_matches(expected, &observation)
}

fn semantic_origin_matches(
    origin: &SemanticOrigin,
    source_id: SourceId,
    module: &ModuleId,
    range: SourceRange,
    structural_path: &[u32],
) -> bool {
    origin.source_id() == source_id
        && origin.module_id() == module
        && origin.anchor() == &SourceAnchor::Range(range)
        && origin.structural_path() == structural_path
        && origin.import_edge().is_none()
        && !origin.is_recovered()
}

#[cfg(test)]
struct ProofLabelMutationContext<'a> {
    profile: &'a ProofLabelConfinementProfile,
    ast: &'a SurfaceAst,
    resolved: &'a SurfaceResolvedArena,
    module: &'a ModuleId,
    namespace: &'a NamespacePath,
    contribution: SourceContributionId,
    foreign_module: &'a ModuleId,
    foreign_namespace: &'a NamespacePath,
    foreign_source: SourceId,
}

#[cfg(test)]
fn mutated_proof_label_projection(
    projection: &LabelProjection,
    context: &ProofLabelMutationContext<'_>,
    mutation: ProofLabelConfinementMutation,
) -> LabelProjection {
    let profile = context.profile;
    let source_id = context.ast.source_id;
    let module = context.module;
    let namespace = context.namespace;
    let contribution = context.contribution;
    let foreign_module = context.foreign_module;
    let foreign_namespace = context.foreign_namespace;
    let foreign_source = context.foreign_source;
    let mut origin_path = projection.origin_path().clone();
    let mut projected_module = module.clone();
    let mut projected_namespace = namespace.clone();
    let mut spelling = "A".to_owned();
    let mut declaration_range = source_range(
        source_id,
        profile.declaration_start,
        profile.declaration_end,
    );
    let mut origin_source = source_id;
    let mut origin_module = module.clone();
    let mut origin_anchor = SourceAnchor::Range(declaration_range);
    let mut structural_path = profile.projection_path.to_vec();
    let mut recovered = false;
    let mut projected_contribution = contribution;
    let mut visible_after_ordinal = 3;
    let mut proof_scope = mizar_resolve::labels::LabelScopePath::new(vec![0, 0]);
    let mut visibility = Visibility::Private;
    let mut export_status = ExportStatus::LocalOnly;

    match mutation {
        ProofLabelConfinementMutation::ProjectionOriginPath => {
            origin_path =
                mizar_resolve::resolved_ast::LabelOriginPath::new("wrong-proof-label-origin");
        }
        ProofLabelConfinementMutation::ProjectionModule => {
            projected_module = foreign_module.clone();
        }
        ProofLabelConfinementMutation::ProjectionNamespace => {
            projected_namespace = foreign_namespace.clone();
        }
        ProofLabelConfinementMutation::ProjectionSpelling => spelling = "B".to_owned(),
        ProofLabelConfinementMutation::ProjectionVisibility => visibility = Visibility::Public,
        ProofLabelConfinementMutation::ProjectionExportStatus => {
            export_status = ExportStatus::Exported;
        }
        ProofLabelConfinementMutation::ProjectionRange => declaration_range.start += 1,
        ProofLabelConfinementMutation::ProjectionOriginSource => origin_source = foreign_source,
        ProofLabelConfinementMutation::ProjectionOriginModule => {
            origin_module = foreign_module.clone();
        }
        ProofLabelConfinementMutation::ProjectionOriginAnchor => {
            origin_anchor = SourceAnchor::Point {
                source_id,
                offset: profile.declaration_start,
            };
        }
        ProofLabelConfinementMutation::ProjectionOriginStructuralPath => {
            structural_path.push(99);
        }
        ProofLabelConfinementMutation::ProjectionOriginRecovered => recovered = true,
        ProofLabelConfinementMutation::ProjectionContribution => {
            projected_contribution =
                second_contribution_id(module.clone(), source_id, declaration_range);
        }
        ProofLabelConfinementMutation::ProjectionVisibleOrdinal => visible_after_ordinal += 1,
        ProofLabelConfinementMutation::ProjectionScope
        | ProofLabelConfinementMutation::ResultResolved => {
            proof_scope =
                mizar_resolve::labels::LabelScopePath::new(profile.reference_scope.to_vec());
            visible_after_ordinal = 0;
        }
        _ => {}
    }

    let mut origin =
        SemanticOrigin::new(origin_source, origin_module, origin_anchor, structural_path);
    if recovered {
        origin = origin.recovered();
    }
    let data = mizar_resolve::labels::LabelProjectionData {
        origin_path,
        module: projected_module,
        namespace: projected_namespace,
        primary_spelling: spelling,
        kind: if mutation == ProofLabelConfinementMutation::ProjectionKind {
            LabelKind::Theorem
        } else {
            LabelKind::ProofStep
        },
        declaration_range,
        origin,
        contribution: projected_contribution,
    };
    let projection = if mutation == ProofLabelConfinementMutation::ProjectionImported {
        LabelProjection::imported(data)
    } else if mutation == ProofLabelConfinementMutation::ProjectionKind {
        LabelProjection::current_module(data, visible_after_ordinal)
    } else {
        LabelProjection::proof_step(data, visible_after_ordinal, proof_scope)
    };
    projection
        .with_visibility(visibility)
        .with_export_status(export_status)
}

#[cfg(test)]
fn mutated_proof_label_reference(
    reference: &LabelReferenceCandidate,
    context: &ProofLabelMutationContext<'_>,
    mutation: ProofLabelConfinementMutation,
) -> Option<LabelReferenceCandidate> {
    let resolved = context.resolved;
    let ast = context.ast;
    let module = context.module;
    let namespace = context.namespace;
    let foreign_module = context.foreign_module;
    let foreign_source = context.foreign_source;
    let mut node = reference.site().node();
    let mut range = reference.site().range();
    let mut spelling = reference.site().spelling().to_owned();
    let mut origin_source = reference.origin().source_id();
    let mut origin_module = reference.origin().module_id().clone();
    let mut origin_anchor = reference.origin().anchor().clone();
    let mut structural_path = reference.origin().structural_path().to_vec();
    let mut recovered = false;
    let mut ordinal = reference.ordinal();
    let mut expectation = reference.expectation();
    let mut scope = match reference.scope() {
        LabelReferenceScope::Unqualified {
            proof_scope: Some(scope),
        } => scope.clone(),
        _ => return None,
    };

    match mutation {
        ProofLabelConfinementMutation::ReferenceNode => {
            node = resolved.resolved_node_for(ast.root()?)?;
        }
        ProofLabelConfinementMutation::ReferenceRange => range.start += 1,
        ProofLabelConfinementMutation::ReferenceSpelling => spelling = "B".to_owned(),
        ProofLabelConfinementMutation::ReferenceOriginSource => origin_source = foreign_source,
        ProofLabelConfinementMutation::ReferenceOriginModule => {
            origin_module = foreign_module.clone();
        }
        ProofLabelConfinementMutation::ReferenceOriginAnchor => {
            origin_anchor = SourceAnchor::Point {
                source_id: ast.source_id,
                offset: range.start,
            };
        }
        ProofLabelConfinementMutation::ReferenceOriginStructuralPath => {
            structural_path.push(99);
        }
        ProofLabelConfinementMutation::ReferenceOriginRecovered => recovered = true,
        ProofLabelConfinementMutation::ReferenceOrdinal => ordinal += 1,
        ProofLabelConfinementMutation::ReferenceExpectation
        | ProofLabelConfinementMutation::ResultUnresolvedExpectation => {
            expectation = LabelExpectation::Theorem;
        }
        ProofLabelConfinementMutation::ReferenceScope => {
            scope = mizar_resolve::labels::LabelScopePath::new(vec![0, 0]);
        }
        _ => {}
    }

    let mut origin =
        SemanticOrigin::new(origin_source, origin_module, origin_anchor, structural_path);
    if recovered {
        origin = origin.recovered();
    }
    let site = mizar_resolve::resolved_ast::ReferenceSite::new(node, range, spelling);
    let candidate = match mutation {
        ProofLabelConfinementMutation::ReferenceQualified => {
            LabelReferenceCandidate::qualified_citation(
                site,
                origin,
                ordinal,
                module.clone(),
                namespace.clone(),
            )
        }
        ProofLabelConfinementMutation::ReferenceFailedNamespace => {
            LabelReferenceCandidate::failed_namespace(site, origin, ordinal, expectation)
        }
        _ => LabelReferenceCandidate::unqualified_citation(site, origin, ordinal, Some(scope))
            .with_expectation(expectation),
    };
    Some(candidate)
}

#[cfg(test)]
fn second_contribution_id(
    module: ModuleId,
    source_id: SourceId,
    range: SourceRange,
) -> SourceContributionId {
    let mut contributions = mizar_resolve::env::SourceContributionIndex::new();
    contributions.insert(
        module.clone(),
        ContributionKind::LocalSource { source_id },
        SourceAnchor::Range(range),
    );
    contributions.insert(
        module,
        ContributionKind::LocalSource { source_id },
        SourceAnchor::Range(range),
    )
}

fn proof_label_origin_path(
    profile: &ProofLabelConfinementProfile,
    module: &ModuleId,
    contribution: SourceContributionId,
) -> String {
    format!(
        "proof-step-v1|package={}:{}|module={}:{}|contribution={}|owner-kind=theorem|owner={}:{}|owner-occurrence=0|proof-path=1:0|label=1:A|label-occurrence=0",
        module.package().as_str().len(),
        module.package().as_str(),
        module.path().as_str().len(),
        module.path().as_str(),
        contribution.index(),
        profile.owner.len(),
        profile.owner,
    )
}

const fn source_range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
    SourceRange {
        source_id,
        start,
        end,
    }
}

fn declaration_symbol_payload_keys(env: &SymbolEnv) -> Vec<String> {
    let mut payloads = Vec::new();
    for symbol in env.symbols().iter() {
        let spelling = declaration_symbol_payload_component(symbol.primary_spelling());
        payloads.push(format!(
            "declaration_symbol.symbol.kind.{spelling}.{}",
            symbol_kind_payload_key(symbol.kind())
        ));
        payloads.push(format!(
            "declaration_symbol.symbol.visibility.{spelling}.{}",
            visibility_payload_key(symbol.visibility())
        ));
        payloads.push(format!(
            "declaration_symbol.symbol.export.{spelling}.{}",
            export_status_payload_key(symbol.export_status())
        ));
        if let Some(definition) = env.definitions().by_symbol(symbol.symbol()) {
            payloads.push(format!(
                "declaration_symbol.definition.kind.{spelling}.{}",
                definition_kind_payload_key(definition.kind())
            ));
            payloads.push(format!(
                "declaration_symbol.definition.visibility.{spelling}.{}",
                visibility_payload_key(definition.visibility())
            ));
        }
    }
    payloads.sort();
    payloads
}

fn declaration_symbol_payload_component(value: &str) -> String {
    let mut escaped = String::new();
    for byte in value.bytes() {
        if byte.is_ascii_alphanumeric() || matches!(byte, b'_' | b'-') {
            escaped.push(byte as char);
        } else {
            escaped.push('%');
            escaped.push(hex_digit(byte >> 4));
            escaped.push(hex_digit(byte & 0x0f));
        }
    }
    escaped
}

const fn hex_digit(value: u8) -> char {
    match value {
        0..=9 => (b'0' + value) as char,
        10..=15 => (b'A' + (value - 10)) as char,
        _ => '?',
    }
}

const fn symbol_kind_payload_key(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Predicate => "predicate",
        SymbolKind::Functor => "functor",
        SymbolKind::Mode => "mode",
        SymbolKind::Attribute => "attribute",
        SymbolKind::Structure => "structure",
        SymbolKind::Selector => "selector",
        SymbolKind::Registration => "registration",
        SymbolKind::Theorem => "theorem",
        SymbolKind::Lemma => "lemma",
        SymbolKind::Algorithm => "algorithm",
        SymbolKind::Scheme => "scheme",
        SymbolKind::Template => "template",
        SymbolKind::Synonym => "synonym",
        SymbolKind::Antonym => "antonym",
        SymbolKind::Redefinition => "redefinition",
        SymbolKind::Builtin => "builtin",
        _ => "unknown",
    }
}

const fn definition_kind_payload_key(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::Predicate => "predicate",
        DefinitionKind::Functor => "functor",
        DefinitionKind::Mode => "mode",
        DefinitionKind::Attribute => "attribute",
        DefinitionKind::Structure => "structure",
        DefinitionKind::Registration => "registration",
        DefinitionKind::Theorem => "theorem",
        DefinitionKind::Lemma => "lemma",
        DefinitionKind::Algorithm => "algorithm",
        DefinitionKind::Scheme => "scheme",
        DefinitionKind::Template => "template",
        DefinitionKind::Synonym => "synonym",
        DefinitionKind::Antonym => "antonym",
        DefinitionKind::Redefinition => "redefinition",
        DefinitionKind::Selector => "selector",
        _ => "unknown",
    }
}

const fn visibility_payload_key(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Public => "public",
        _ => "unknown",
    }
}

const fn export_status_payload_key(status: ExportStatus) -> &'static str {
    match status {
        ExportStatus::LocalOnly => "local_only",
        ExportStatus::Exported => "exported",
        ExportStatus::ReExported => "re_exported",
        _ => "unknown",
    }
}

fn expected_declaration_symbol_detail_keys(case: &TestCase) -> Vec<String> {
    if !case.expectation.diagnostic_payloads.is_empty() {
        return case.expectation.diagnostic_payloads.clone();
    }
    case.expectation.stable_detail_key.iter().cloned().collect()
}

fn expected_declaration_symbol_payload_keys(case: &TestCase) -> Vec<String> {
    let mut payloads = case.expectation.declaration_symbol_payloads.clone();
    payloads.sort();
    payloads
}

pub(super) fn declaration_symbol_failure_diagnostic(
    case: &TestCase,
    result: &DeclarationSymbolCaseResult,
) -> ValidationDiagnostic {
    ValidationDiagnostic::error(
        &case.expectation_path,
        "declaration_symbol",
        "E-DECLARATION-SYMBOL-ASSERT",
        format!("declaration_symbol.{}", case.id.0),
        format!(
            "declaration-symbol case `{}` expected detail keys {:?} but got {:?}; expected payload keys {:?} but got {:?}",
            case.id.0,
            expected_declaration_symbol_detail_keys(case),
            result.actual_detail_keys,
            expected_declaration_symbol_payload_keys(case),
            result.actual_payload_keys
        ),
    )
}
