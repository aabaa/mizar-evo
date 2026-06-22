//! Cluster closure and trace data layer for checker phase 7.

use crate::registration_resolution::{
    ActivatedRegistration, CheckerRegistrationId, RegistrationDatabase, RegistrationTriggerKey,
};
use mizar_resolve::{
    env::{RegistrationId as ResolverRegistrationId, RegistrationKind as ResolverRegistrationKind},
    resolved_ast::ModuleId,
};
use mizar_session::{SourceId, SourceRange};
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

macro_rules! dense_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(usize);

        impl $name {
            pub const fn new(index: usize) -> Self {
                Self(index)
            }

            pub const fn index(self) -> usize {
                self.0
            }
        }
    };
}

macro_rules! string_key {
    ($name:ident) => {
        #[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Self {
                Self(value.into())
            }

            pub fn as_str(&self) -> &str {
                &self.0
            }
        }

        impl From<&str> for $name {
            fn from(value: &str) -> Self {
                Self::new(value)
            }
        }

        impl From<String> for $name {
            fn from(value: String) -> Self {
                Self::new(value)
            }
        }
    };
}

dense_id!(ClusterFactId);
dense_id!(ClusterStepId);
dense_id!(ClusterDiagnosticId);

string_key!(ClusterFactFingerprint);
string_key!(ClusterTypeFingerprint);
string_key!(ClusterAttributeFingerprint);
string_key!(ClusterRuleFingerprint);
string_key!(ClusterAuditKey);
string_key!(ClusterOrderingVersion);
string_key!(ClusterTraversalCacheKey);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterClosureOutput {
    status: ClusterClosureStatus,
    trace: ResolutionTrace,
    closure_facts: ClusterFactTable,
    diagnostics: ClusterDiagnosticTable,
}

impl ClusterClosureOutput {
    pub const fn status(&self) -> ClusterClosureStatus {
        self.status
    }

    pub const fn trace(&self) -> &ResolutionTrace {
        &self.trace
    }

    pub const fn closure_facts(&self) -> &ClusterFactTable {
        &self.closure_facts
    }

    pub const fn diagnostics(&self) -> &ClusterDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("cluster-closure-debug-v1\n");
        let _ = writeln!(
            output,
            "status={}",
            cluster_closure_status_name(self.status)
        );
        write_trace(&mut output, &self.trace);
        write_facts(&mut output, "closure-facts", &self.closure_facts);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterClosureStatus {
    Complete,
    Incomplete,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterTraceBuilder {
    config: ClusterTraversalConfig,
}

impl ClusterTraceBuilder {
    pub const fn new(config: ClusterTraversalConfig) -> Self {
        Self { config }
    }

    pub fn close(
        &self,
        database: &RegistrationDatabase,
        source_id: SourceId,
        module_id: ModuleId,
        initial_facts: impl IntoIterator<Item = ClusterFactInput>,
        rules: impl IntoIterator<Item = ClusterRuleInput>,
    ) -> ClusterClosureOutput {
        let mut diagnostics = ClusterDiagnosticTable::new();
        let mut facts = ClusterFactTable::new();
        for fact in initial_facts {
            facts.insert(ClusterFactDraft {
                fingerprint: fact.fingerprint,
                source_type: fact.source_type,
                attribute: fact.attribute,
                generated_type: fact.generated_type,
                provenance: ClusterFactProvenance::Input,
                source_range: fact.source_range,
            });
        }

        let mut rules = rules
            .into_iter()
            .filter_map(|rule| validate_rule(database, rule, &mut diagnostics))
            .collect::<Vec<_>>();
        rules.sort_by_key(valid_rule_order_key);

        let input_fact_count = facts.len();
        let mut steps = Vec::new();
        let mut derived_facts = ClusterFactTable::new();
        let mut present = facts.fingerprint_set();
        let mut closure_state = present
            .iter()
            .cloned()
            .map(|fingerprint| {
                (
                    fingerprint.clone(),
                    ClusterFactClosureState::input(fingerprint),
                )
            })
            .collect::<BTreeMap<_, _>>();
        let mut failed_candidates = BTreeSet::new();
        let mut bounded_saturation_reached = false;
        let mut fatal_failure_seen = false;
        let mut changed = true;

        'closure: while changed {
            changed = false;
            for rule in &rules {
                if !rule
                    .input
                    .antecedents
                    .iter()
                    .all(|antecedent| present.contains(antecedent))
                {
                    continue;
                }

                let candidate_key = cluster_candidate_key(rule);
                if failed_candidates.contains(&candidate_key) {
                    continue;
                }
                if let Some(loop_fact) = active_loop_fact(&closure_state, rule) {
                    diagnostics.insert(ClusterDiagnosticDraft {
                        registration: Some(rule.input.registration),
                        class: ClusterDiagnosticClass::ClusterLoop,
                        severity: ClusterDiagnosticSeverity::Error,
                        message_key: "checker.cluster_trace.cluster_loop".to_owned(),
                        detail: Some(loop_fact.as_str().to_owned()),
                        source_range: rule.input.source_range,
                        recovery: ClusterDiagnosticRecovery::Fatal,
                    });
                    failed_candidates.insert(candidate_key);
                    fatal_failure_seen = true;
                    break 'closure;
                }
                if present.contains(&rule.input.generated_fact) {
                    continue;
                }
                if let Some(conflict) = rule
                    .input
                    .conflicts
                    .iter()
                    .find(|conflict| present.contains(*conflict))
                {
                    diagnostics.insert(ClusterDiagnosticDraft {
                        registration: Some(rule.input.registration),
                        class: ClusterDiagnosticClass::ClusterContradiction,
                        severity: ClusterDiagnosticSeverity::Error,
                        message_key: "checker.cluster_trace.cluster_contradiction".to_owned(),
                        detail: Some(conflict.as_str().to_owned()),
                        source_range: rule.input.source_range,
                        recovery: ClusterDiagnosticRecovery::Fatal,
                    });
                    failed_candidates.insert(candidate_key);
                    fatal_failure_seen = true;
                    break 'closure;
                }
                let candidate_depth = candidate_depth(&closure_state, &rule.input.antecedents);
                if candidate_depth > self.config.max_cluster_depth {
                    diagnostics.insert(ClusterDiagnosticDraft {
                        registration: Some(rule.input.registration),
                        class: ClusterDiagnosticClass::ClusterBoundExceeded,
                        severity: ClusterDiagnosticSeverity::Error,
                        message_key: "checker.cluster_trace.cluster_bound_exceeded".to_owned(),
                        detail: Some(format!(
                            "depth:{candidate_depth}>max:{}",
                            self.config.max_cluster_depth
                        )),
                        source_range: rule.input.source_range,
                        recovery: ClusterDiagnosticRecovery::Fatal,
                    });
                    failed_candidates.insert(candidate_key);
                    bounded_saturation_reached = true;
                    fatal_failure_seen = true;
                    break 'closure;
                }
                if derived_facts.len() >= self.config.max_generated_facts {
                    diagnostics.insert(ClusterDiagnosticDraft {
                        registration: Some(rule.input.registration),
                        class: ClusterDiagnosticClass::ClusterBoundExceeded,
                        severity: ClusterDiagnosticSeverity::Error,
                        message_key: "checker.cluster_trace.cluster_bound_exceeded".to_owned(),
                        detail: Some(format!(
                            "generated:{}>=max:{}",
                            derived_facts.len(),
                            self.config.max_generated_facts
                        )),
                        source_range: rule.input.source_range,
                        recovery: ClusterDiagnosticRecovery::Fatal,
                    });
                    failed_candidates.insert(candidate_key);
                    bounded_saturation_reached = true;
                    fatal_failure_seen = true;
                    break 'closure;
                }

                let step_id = ClusterStepId::new(steps.len());
                let antecedent_sources = rule
                    .input
                    .antecedents
                    .iter()
                    .filter_map(|fingerprint| {
                        facts
                            .get_by_fingerprint(fingerprint)
                            .map(|fact| ClusterAntecedentRef {
                                fingerprint: fingerprint.clone(),
                                source_range: fact.source_range(),
                            })
                    })
                    .collect();
                let fact = ClusterFactDraft {
                    fingerprint: rule.input.generated_fact.clone(),
                    source_type: rule.input.source_type.clone(),
                    attribute: rule.input.generated_attribute.clone(),
                    generated_type: rule.input.generated_type.clone(),
                    provenance: ClusterFactProvenance::TraceStep(step_id),
                    source_range: rule.input.source_range,
                };
                facts.insert(fact.clone());
                derived_facts.insert(fact);
                present.insert(rule.input.generated_fact.clone());
                closure_state.insert(
                    rule.input.generated_fact.clone(),
                    candidate_closure_state(
                        &closure_state,
                        &rule.input.antecedents,
                        rule.input.generated_fact.clone(),
                        candidate_depth,
                    ),
                );
                steps.push(ResolutionTraceStep::Cluster(ClusterStep {
                    id: step_id,
                    source_type: rule.input.source_type.clone(),
                    applied_cluster: rule.input.registration,
                    resolver_registration: rule.active.resolver_registration(),
                    rule_kind: rule.input.kind,
                    rule_fingerprint: rule.rule_fingerprint.clone(),
                    generated_attribute: rule.input.generated_attribute.clone(),
                    generated_type: rule.input.generated_type.clone(),
                    generated_fact: rule.input.generated_fact.clone(),
                    antecedents: rule.input.antecedents.clone(),
                    antecedent_sources,
                    audit_key: rule.audit_key.clone(),
                    rule_source_range: rule.input.source_range,
                }));
                changed = true;
            }
        }

        let profile = ClusterTraversalProfile {
            ordering_version: ClusterOrderingVersion::new("cluster-trace-order-v1"),
            cache_key_material: traversal_cache_key_material(&self.config),
            max_cluster_depth: self.config.max_cluster_depth,
            max_generated_facts: self.config.max_generated_facts,
            bounded_saturation_reached,
            input_fact_count,
            derived_fact_count: derived_facts.len(),
            cluster_step_count: steps.len(),
            reduction_step_count: 0,
            diagnostic_count: diagnostics.len(),
        };
        ClusterClosureOutput {
            status: if fatal_failure_seen {
                ClusterClosureStatus::Incomplete
            } else {
                ClusterClosureStatus::Complete
            },
            trace: ResolutionTrace {
                source_id,
                module_id,
                steps,
                derived_facts,
                traversal_profile: profile,
            },
            closure_facts: facts,
            diagnostics,
        }
    }
}

impl Default for ClusterTraceBuilder {
    fn default() -> Self {
        Self::new(ClusterTraversalConfig::default())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClusterTraversalConfig {
    pub max_cluster_depth: usize,
    pub max_generated_facts: usize,
}

impl Default for ClusterTraversalConfig {
    fn default() -> Self {
        Self {
            max_cluster_depth: 128,
            max_generated_facts: 4096,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolutionTrace {
    source_id: SourceId,
    module_id: ModuleId,
    steps: Vec<ResolutionTraceStep>,
    derived_facts: ClusterFactTable,
    traversal_profile: ClusterTraversalProfile,
}

impl ResolutionTrace {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub fn steps(&self) -> &[ResolutionTraceStep] {
        &self.steps
    }

    pub const fn derived_facts(&self) -> &ClusterFactTable {
        &self.derived_facts
    }

    pub const fn traversal_profile(&self) -> &ClusterTraversalProfile {
        &self.traversal_profile
    }

    pub fn replay(
        &self,
        database: &RegistrationDatabase,
        initial_facts: impl IntoIterator<Item = ClusterFactInput>,
    ) -> ClusterReplayReport {
        let mut facts = ClusterFactTable::new();
        for fact in initial_facts {
            facts.insert(ClusterFactDraft {
                fingerprint: fact.fingerprint,
                source_type: fact.source_type,
                attribute: fact.attribute,
                generated_type: fact.generated_type,
                provenance: ClusterFactProvenance::Input,
                source_range: fact.source_range,
            });
        }

        let mut diagnostics = ClusterDiagnosticTable::new();
        let mut present = facts.fingerprint_set();
        for step in &self.steps {
            let ResolutionTraceStep::Cluster(step) = step;
            let Some(active) = database.activated().get(step.applied_cluster) else {
                diagnostics.insert(ClusterDiagnosticDraft {
                    registration: Some(step.applied_cluster),
                    class: ClusterDiagnosticClass::ReplayFailure,
                    severity: ClusterDiagnosticSeverity::Error,
                    message_key: "checker.cluster_trace.replay_invisible_registration".to_owned(),
                    detail: None,
                    source_range: step.rule_source_range,
                    recovery: ClusterDiagnosticRecovery::Degraded,
                });
                continue;
            };
            if active.kind() != ResolverRegistrationKind::Cluster {
                diagnostics.insert(ClusterDiagnosticDraft {
                    registration: Some(step.applied_cluster),
                    class: ClusterDiagnosticClass::ReplayFailure,
                    severity: ClusterDiagnosticSeverity::Error,
                    message_key: "checker.cluster_trace.replay_non_cluster_registration".to_owned(),
                    detail: Some(registration_kind_name(active.kind()).to_owned()),
                    source_range: step.rule_source_range,
                    recovery: ClusterDiagnosticRecovery::Degraded,
                });
                continue;
            }
            if active.resolver_registration() != step.resolver_registration {
                diagnostics.insert(ClusterDiagnosticDraft {
                    registration: Some(step.applied_cluster),
                    class: ClusterDiagnosticClass::ReplayFailure,
                    severity: ClusterDiagnosticSeverity::Error,
                    message_key: "checker.cluster_trace.replay_resolver_mismatch".to_owned(),
                    detail: Some(format!(
                        "{} != {}",
                        active.resolver_registration().index(),
                        step.resolver_registration.index()
                    )),
                    source_range: step.rule_source_range,
                    recovery: ClusterDiagnosticRecovery::Degraded,
                });
                continue;
            }
            if active_rule_fingerprint(active) != step.rule_fingerprint.as_str() {
                diagnostics.insert(ClusterDiagnosticDraft {
                    registration: Some(step.applied_cluster),
                    class: ClusterDiagnosticClass::ReplayFailure,
                    severity: ClusterDiagnosticSeverity::Error,
                    message_key: "checker.cluster_trace.replay_fingerprint_mismatch".to_owned(),
                    detail: Some(format!(
                        "{} != {}",
                        active_rule_fingerprint(active),
                        step.rule_fingerprint.as_str()
                    )),
                    source_range: step.rule_source_range,
                    recovery: ClusterDiagnosticRecovery::Degraded,
                });
                continue;
            }
            let expected_audit = cluster_audit_key_for(
                &step.source_type,
                &step.generated_attribute,
                &step.rule_fingerprint,
                active,
            );
            if expected_audit != step.audit_key {
                diagnostics.insert(ClusterDiagnosticDraft {
                    registration: Some(step.applied_cluster),
                    class: ClusterDiagnosticClass::ReplayFailure,
                    severity: ClusterDiagnosticSeverity::Error,
                    message_key: "checker.cluster_trace.replay_audit_key_mismatch".to_owned(),
                    detail: Some(format!(
                        "{} != {}",
                        expected_audit.as_str(),
                        step.audit_key.as_str()
                    )),
                    source_range: step.rule_source_range,
                    recovery: ClusterDiagnosticRecovery::Degraded,
                });
                continue;
            }
            if let Some(missing) = step
                .antecedents
                .iter()
                .find(|antecedent| !present.contains(*antecedent))
            {
                diagnostics.insert(ClusterDiagnosticDraft {
                    registration: Some(step.applied_cluster),
                    class: ClusterDiagnosticClass::ReplayFailure,
                    severity: ClusterDiagnosticSeverity::Error,
                    message_key: "checker.cluster_trace.replay_missing_antecedent".to_owned(),
                    detail: Some(missing.as_str().to_owned()),
                    source_range: step.rule_source_range,
                    recovery: ClusterDiagnosticRecovery::Degraded,
                });
                continue;
            }
            if !self.derived_facts.contains(&step.generated_fact) {
                diagnostics.insert(ClusterDiagnosticDraft {
                    registration: Some(step.applied_cluster),
                    class: ClusterDiagnosticClass::ReplayFailure,
                    severity: ClusterDiagnosticSeverity::Error,
                    message_key: "checker.cluster_trace.replay_missing_derived_fact".to_owned(),
                    detail: Some(step.generated_fact.as_str().to_owned()),
                    source_range: step.rule_source_range,
                    recovery: ClusterDiagnosticRecovery::Degraded,
                });
                continue;
            }
            let fact = ClusterFactDraft {
                fingerprint: step.generated_fact.clone(),
                source_type: step.source_type.clone(),
                attribute: step.generated_attribute.clone(),
                generated_type: step.generated_type.clone(),
                provenance: ClusterFactProvenance::TraceStep(step.id),
                source_range: step.rule_source_range,
            };
            facts.insert(fact);
            present.insert(step.generated_fact.clone());
        }

        ClusterReplayReport {
            status: if diagnostics.is_empty() {
                ClusterReplayStatus::Valid
            } else {
                ClusterReplayStatus::Invalid
            },
            replayed_facts: facts,
            diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ResolutionTraceStep {
    Cluster(ClusterStep),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterStep {
    id: ClusterStepId,
    source_type: ClusterTypeFingerprint,
    applied_cluster: CheckerRegistrationId,
    resolver_registration: ResolverRegistrationId,
    rule_kind: ClusterRuleKind,
    rule_fingerprint: ClusterRuleFingerprint,
    generated_attribute: ClusterAttributeFingerprint,
    generated_type: ClusterTypeFingerprint,
    generated_fact: ClusterFactFingerprint,
    antecedents: Vec<ClusterFactFingerprint>,
    antecedent_sources: Vec<ClusterAntecedentRef>,
    audit_key: ClusterAuditKey,
    rule_source_range: SourceRange,
}

impl ClusterStep {
    pub const fn id(&self) -> ClusterStepId {
        self.id
    }

    pub const fn source_type(&self) -> &ClusterTypeFingerprint {
        &self.source_type
    }

    pub const fn applied_cluster(&self) -> CheckerRegistrationId {
        self.applied_cluster
    }

    pub const fn resolver_registration(&self) -> ResolverRegistrationId {
        self.resolver_registration
    }

    pub const fn rule_kind(&self) -> ClusterRuleKind {
        self.rule_kind
    }

    pub const fn rule_fingerprint(&self) -> &ClusterRuleFingerprint {
        &self.rule_fingerprint
    }

    pub const fn generated_attribute(&self) -> &ClusterAttributeFingerprint {
        &self.generated_attribute
    }

    pub const fn generated_type(&self) -> &ClusterTypeFingerprint {
        &self.generated_type
    }

    pub const fn generated_fact(&self) -> &ClusterFactFingerprint {
        &self.generated_fact
    }

    pub fn antecedents(&self) -> &[ClusterFactFingerprint] {
        &self.antecedents
    }

    pub fn antecedent_sources(&self) -> &[ClusterAntecedentRef] {
        &self.antecedent_sources
    }

    pub const fn audit_key(&self) -> &ClusterAuditKey {
        &self.audit_key
    }

    pub const fn rule_source_range(&self) -> SourceRange {
        self.rule_source_range
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterAntecedentRef {
    fingerprint: ClusterFactFingerprint,
    source_range: SourceRange,
}

impl ClusterAntecedentRef {
    pub const fn fingerprint(&self) -> &ClusterFactFingerprint {
        &self.fingerprint
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterTraversalProfile {
    ordering_version: ClusterOrderingVersion,
    cache_key_material: ClusterTraversalCacheKey,
    max_cluster_depth: usize,
    max_generated_facts: usize,
    bounded_saturation_reached: bool,
    input_fact_count: usize,
    derived_fact_count: usize,
    cluster_step_count: usize,
    reduction_step_count: usize,
    diagnostic_count: usize,
}

impl ClusterTraversalProfile {
    pub const fn ordering_version(&self) -> &ClusterOrderingVersion {
        &self.ordering_version
    }

    pub const fn cache_key_material(&self) -> &ClusterTraversalCacheKey {
        &self.cache_key_material
    }

    pub const fn max_cluster_depth(&self) -> usize {
        self.max_cluster_depth
    }

    pub const fn max_generated_facts(&self) -> usize {
        self.max_generated_facts
    }

    pub const fn bounded_saturation_reached(&self) -> bool {
        self.bounded_saturation_reached
    }

    pub const fn input_fact_count(&self) -> usize {
        self.input_fact_count
    }

    pub const fn derived_fact_count(&self) -> usize {
        self.derived_fact_count
    }

    pub const fn cluster_step_count(&self) -> usize {
        self.cluster_step_count
    }

    pub const fn reduction_step_count(&self) -> usize {
        self.reduction_step_count
    }

    pub const fn diagnostic_count(&self) -> usize {
        self.diagnostic_count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterReplayReport {
    status: ClusterReplayStatus,
    replayed_facts: ClusterFactTable,
    diagnostics: ClusterDiagnosticTable,
}

impl ClusterReplayReport {
    pub const fn status(&self) -> ClusterReplayStatus {
        self.status
    }

    pub const fn replayed_facts(&self) -> &ClusterFactTable {
        &self.replayed_facts
    }

    pub const fn diagnostics(&self) -> &ClusterDiagnosticTable {
        &self.diagnostics
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterReplayStatus {
    Valid,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterFactInput {
    fingerprint: ClusterFactFingerprint,
    source_type: ClusterTypeFingerprint,
    attribute: ClusterAttributeFingerprint,
    generated_type: ClusterTypeFingerprint,
    source_range: SourceRange,
}

impl ClusterFactInput {
    pub fn new(
        fingerprint: impl Into<ClusterFactFingerprint>,
        source_type: impl Into<ClusterTypeFingerprint>,
        attribute: impl Into<ClusterAttributeFingerprint>,
        generated_type: impl Into<ClusterTypeFingerprint>,
        source_range: SourceRange,
    ) -> Self {
        Self {
            fingerprint: fingerprint.into(),
            source_type: source_type.into(),
            attribute: attribute.into(),
            generated_type: generated_type.into(),
            source_range,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterRuleInput {
    registration: CheckerRegistrationId,
    kind: ClusterRuleKind,
    trigger: RegistrationTriggerKey,
    source_type: ClusterTypeFingerprint,
    antecedents: Vec<ClusterFactFingerprint>,
    conflicts: Vec<ClusterFactFingerprint>,
    generated_attribute: ClusterAttributeFingerprint,
    generated_type: ClusterTypeFingerprint,
    generated_fact: ClusterFactFingerprint,
    rule_fingerprint: ClusterRuleFingerprint,
    source_range: SourceRange,
}

impl ClusterRuleInput {
    pub fn new(draft: ClusterRuleDraft) -> Self {
        Self {
            registration: draft.registration,
            kind: draft.kind,
            trigger: draft.trigger,
            source_type: draft.source_type,
            antecedents: Vec::new(),
            conflicts: Vec::new(),
            generated_attribute: draft.generated_attribute,
            generated_type: draft.generated_type,
            generated_fact: draft.generated_fact,
            rule_fingerprint: draft.rule_fingerprint,
            source_range: draft.source_range,
        }
    }

    pub fn with_antecedents(
        mut self,
        antecedents: impl IntoIterator<Item = ClusterFactFingerprint>,
    ) -> Self {
        self.antecedents = antecedents.into_iter().collect();
        self.antecedents.sort();
        self.antecedents.dedup();
        self
    }

    pub fn with_conflicts(
        mut self,
        conflicts: impl IntoIterator<Item = ClusterFactFingerprint>,
    ) -> Self {
        self.conflicts = conflicts.into_iter().collect();
        self.conflicts.sort();
        self.conflicts.dedup();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterRuleDraft {
    pub registration: CheckerRegistrationId,
    pub kind: ClusterRuleKind,
    pub trigger: RegistrationTriggerKey,
    pub source_type: ClusterTypeFingerprint,
    pub generated_attribute: ClusterAttributeFingerprint,
    pub generated_type: ClusterTypeFingerprint,
    pub generated_fact: ClusterFactFingerprint,
    pub rule_fingerprint: ClusterRuleFingerprint,
    pub source_range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ValidClusterRule {
    input: ClusterRuleInput,
    active: ActivatedRegistration,
    rule_fingerprint: ClusterRuleFingerprint,
    audit_key: ClusterAuditKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterRuleKind {
    Conditional,
    Functorial,
    Existential,
}

impl ClusterRuleKind {
    const fn can_fire_in_task_16(self) -> bool {
        matches!(self, Self::Conditional | Self::Functorial)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterFact {
    id: ClusterFactId,
    fingerprint: ClusterFactFingerprint,
    source_type: ClusterTypeFingerprint,
    attribute: ClusterAttributeFingerprint,
    generated_type: ClusterTypeFingerprint,
    provenance: ClusterFactProvenance,
    source_range: SourceRange,
}

impl ClusterFact {
    pub const fn id(&self) -> ClusterFactId {
        self.id
    }

    pub const fn fingerprint(&self) -> &ClusterFactFingerprint {
        &self.fingerprint
    }

    pub const fn source_type(&self) -> &ClusterTypeFingerprint {
        &self.source_type
    }

    pub const fn attribute(&self) -> &ClusterAttributeFingerprint {
        &self.attribute
    }

    pub const fn generated_type(&self) -> &ClusterTypeFingerprint {
        &self.generated_type
    }

    pub const fn provenance(&self) -> &ClusterFactProvenance {
        &self.provenance
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterFactDraft {
    pub fingerprint: ClusterFactFingerprint,
    pub source_type: ClusterTypeFingerprint,
    pub attribute: ClusterAttributeFingerprint,
    pub generated_type: ClusterTypeFingerprint,
    pub provenance: ClusterFactProvenance,
    pub source_range: SourceRange,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClusterFactTable {
    entries: Vec<ClusterFact>,
    by_fingerprint: BTreeMap<ClusterFactFingerprint, ClusterFactId>,
}

impl ClusterFactTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            by_fingerprint: BTreeMap::new(),
        }
    }

    pub fn insert(&mut self, draft: ClusterFactDraft) -> ClusterFactId {
        if let Some(id) = self.by_fingerprint.get(&draft.fingerprint) {
            return *id;
        }
        let id = ClusterFactId::new(self.entries.len());
        self.by_fingerprint.insert(draft.fingerprint.clone(), id);
        self.entries.push(ClusterFact {
            id,
            fingerprint: draft.fingerprint,
            source_type: draft.source_type,
            attribute: draft.attribute,
            generated_type: draft.generated_type,
            provenance: draft.provenance,
            source_range: draft.source_range,
        });
        id
    }

    pub fn get(&self, id: ClusterFactId) -> Option<&ClusterFact> {
        self.entries.get(id.index())
    }

    pub fn contains(&self, fingerprint: &ClusterFactFingerprint) -> bool {
        self.by_fingerprint.contains_key(fingerprint)
    }

    pub fn get_by_fingerprint(&self, fingerprint: &ClusterFactFingerprint) -> Option<&ClusterFact> {
        self.by_fingerprint
            .get(fingerprint)
            .and_then(|id| self.get(*id))
    }

    pub fn iter(&self) -> impl Iterator<Item = (ClusterFactId, &ClusterFact)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (ClusterFactId, &ClusterFact)> {
        self.by_fingerprint
            .values()
            .copied()
            .map(|id| (id, &self.entries[id.index()]))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn fingerprint_set(&self) -> BTreeSet<ClusterFactFingerprint> {
        self.by_fingerprint.keys().cloned().collect()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterFactProvenance {
    Input,
    TraceStep(ClusterStepId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterDiagnostic {
    id: ClusterDiagnosticId,
    registration: Option<CheckerRegistrationId>,
    class: ClusterDiagnosticClass,
    severity: ClusterDiagnosticSeverity,
    message_key: String,
    detail: Option<String>,
    source_range: SourceRange,
    recovery: ClusterDiagnosticRecovery,
}

impl ClusterDiagnostic {
    pub const fn id(&self) -> ClusterDiagnosticId {
        self.id
    }

    pub const fn registration(&self) -> Option<CheckerRegistrationId> {
        self.registration
    }

    pub const fn class(&self) -> ClusterDiagnosticClass {
        self.class
    }

    pub const fn severity(&self) -> ClusterDiagnosticSeverity {
        self.severity
    }

    pub fn message_key(&self) -> &str {
        &self.message_key
    }

    pub fn detail(&self) -> Option<&str> {
        self.detail.as_deref()
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn recovery(&self) -> ClusterDiagnosticRecovery {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ClusterDiagnosticDraft {
    pub registration: Option<CheckerRegistrationId>,
    pub class: ClusterDiagnosticClass,
    pub severity: ClusterDiagnosticSeverity,
    pub message_key: String,
    pub detail: Option<String>,
    pub source_range: SourceRange,
    pub recovery: ClusterDiagnosticRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ClusterDiagnosticTable {
    entries: Vec<ClusterDiagnostic>,
}

impl ClusterDiagnosticTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: ClusterDiagnosticDraft) -> ClusterDiagnosticId {
        let id = ClusterDiagnosticId::new(self.entries.len());
        self.entries.push(ClusterDiagnostic {
            id,
            registration: draft.registration,
            class: draft.class,
            severity: draft.severity,
            message_key: draft.message_key,
            detail: draft.detail,
            source_range: draft.source_range,
            recovery: draft.recovery,
        });
        id
    }

    pub fn iter(&self) -> impl Iterator<Item = (ClusterDiagnosticId, &ClusterDiagnostic)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (ClusterDiagnosticId, &ClusterDiagnostic)> {
        let mut entries = self.entries.iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| diagnostic_order_key(entry));
        entries.into_iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterDiagnosticClass {
    InvisibleRegistration,
    InvalidRulePayload,
    ClusterLoop,
    ClusterBoundExceeded,
    ClusterContradiction,
    ReplayFailure,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterDiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ClusterDiagnosticRecovery {
    Normal,
    Degraded,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ClusterFactClosureState {
    depth: usize,
    ancestry: BTreeSet<ClusterFactFingerprint>,
}

impl ClusterFactClosureState {
    fn input(fingerprint: ClusterFactFingerprint) -> Self {
        Self {
            depth: 0,
            ancestry: BTreeSet::from([fingerprint]),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct ClusterCandidateKey {
    registration: CheckerRegistrationId,
    rule_fingerprint: ClusterRuleFingerprint,
    generated_fact: ClusterFactFingerprint,
}

fn validate_rule(
    database: &RegistrationDatabase,
    rule: ClusterRuleInput,
    diagnostics: &mut ClusterDiagnosticTable,
) -> Option<ValidClusterRule> {
    let Some(active) = database.activated().get(rule.registration).cloned() else {
        diagnostics.insert(ClusterDiagnosticDraft {
            registration: Some(rule.registration),
            class: ClusterDiagnosticClass::InvisibleRegistration,
            severity: ClusterDiagnosticSeverity::Error,
            message_key: "checker.cluster_trace.invisible_registration".to_owned(),
            detail: None,
            source_range: rule.source_range,
            recovery: ClusterDiagnosticRecovery::Degraded,
        });
        return None;
    };

    if active.kind() != ResolverRegistrationKind::Cluster {
        diagnostics.insert(ClusterDiagnosticDraft {
            registration: Some(rule.registration),
            class: ClusterDiagnosticClass::InvalidRulePayload,
            severity: ClusterDiagnosticSeverity::Error,
            message_key: "checker.cluster_trace.non_cluster_registration".to_owned(),
            detail: Some(registration_kind_name(active.kind()).to_owned()),
            source_range: rule.source_range,
            recovery: ClusterDiagnosticRecovery::Degraded,
        });
        return None;
    }

    if !rule.kind.can_fire_in_task_16() {
        diagnostics.insert(ClusterDiagnosticDraft {
            registration: Some(rule.registration),
            class: ClusterDiagnosticClass::InvalidRulePayload,
            severity: ClusterDiagnosticSeverity::Error,
            message_key: "checker.cluster_trace.unsupported_rule_kind".to_owned(),
            detail: Some(cluster_rule_kind_name(rule.kind).to_owned()),
            source_range: rule.source_range,
            recovery: ClusterDiagnosticRecovery::Degraded,
        });
        return None;
    }

    if active.trigger().as_str() != rule.trigger.as_str() {
        diagnostics.insert(ClusterDiagnosticDraft {
            registration: Some(rule.registration),
            class: ClusterDiagnosticClass::InvalidRulePayload,
            severity: ClusterDiagnosticSeverity::Error,
            message_key: "checker.cluster_trace.trigger_mismatch".to_owned(),
            detail: Some(format!(
                "{} != {}",
                active.trigger().as_str(),
                rule.trigger.as_str()
            )),
            source_range: rule.source_range,
            recovery: ClusterDiagnosticRecovery::Degraded,
        });
        return None;
    }

    if active_rule_fingerprint(&active) != rule.rule_fingerprint.as_str() {
        diagnostics.insert(ClusterDiagnosticDraft {
            registration: Some(rule.registration),
            class: ClusterDiagnosticClass::InvalidRulePayload,
            severity: ClusterDiagnosticSeverity::Error,
            message_key: "checker.cluster_trace.fingerprint_mismatch".to_owned(),
            detail: Some(format!(
                "{} != {}",
                active_rule_fingerprint(&active),
                rule.rule_fingerprint.as_str()
            )),
            source_range: rule.source_range,
            recovery: ClusterDiagnosticRecovery::Degraded,
        });
        return None;
    }

    let audit_key = cluster_audit_key(&rule, &active);
    let rule_fingerprint = rule.rule_fingerprint.clone();
    Some(ValidClusterRule {
        input: rule,
        active,
        rule_fingerprint,
        audit_key,
    })
}

fn valid_rule_order_key(
    rule: &ValidClusterRule,
) -> (String, String, String, Vec<u32>, usize, String, String) {
    let origin = rule.active.source().origin();
    (
        rule.input.source_type.as_str().to_owned(),
        origin.module_id().package().as_str().to_owned(),
        origin.module_id().path().as_str().to_owned(),
        origin.structural_path().to_vec(),
        rule.active.resolver_registration().index(),
        rule.input.generated_attribute.as_str().to_owned(),
        rule.rule_fingerprint.as_str().to_owned(),
    )
}

fn active_loop_fact(
    closure_state: &BTreeMap<ClusterFactFingerprint, ClusterFactClosureState>,
    rule: &ValidClusterRule,
) -> Option<ClusterFactFingerprint> {
    rule.input.antecedents.iter().find_map(|antecedent| {
        closure_state
            .get(antecedent)
            .filter(|state| state.ancestry.contains(&rule.input.generated_fact))
            .map(|_| antecedent.clone())
    })
}

fn candidate_depth(
    closure_state: &BTreeMap<ClusterFactFingerprint, ClusterFactClosureState>,
    antecedents: &[ClusterFactFingerprint],
) -> usize {
    antecedents
        .iter()
        .filter_map(|antecedent| closure_state.get(antecedent).map(|state| state.depth))
        .max()
        .unwrap_or(0)
        + 1
}

fn candidate_closure_state(
    closure_state: &BTreeMap<ClusterFactFingerprint, ClusterFactClosureState>,
    antecedents: &[ClusterFactFingerprint],
    generated_fact: ClusterFactFingerprint,
    depth: usize,
) -> ClusterFactClosureState {
    let mut ancestry = BTreeSet::new();
    for antecedent in antecedents {
        if let Some(state) = closure_state.get(antecedent) {
            ancestry.extend(state.ancestry.iter().cloned());
        }
    }
    ancestry.insert(generated_fact);
    ClusterFactClosureState { depth, ancestry }
}

fn cluster_candidate_key(rule: &ValidClusterRule) -> ClusterCandidateKey {
    ClusterCandidateKey {
        registration: rule.input.registration,
        rule_fingerprint: rule.rule_fingerprint.clone(),
        generated_fact: rule.input.generated_fact.clone(),
    }
}

fn traversal_cache_key_material(config: &ClusterTraversalConfig) -> ClusterTraversalCacheKey {
    ClusterTraversalCacheKey::new(format!(
        "order=cluster-trace-order-v1;max_depth={};max_generated={}",
        config.max_cluster_depth, config.max_generated_facts
    ))
}

fn diagnostic_order_key(diagnostic: &ClusterDiagnostic) -> (Option<usize>, u8, u8, String, String) {
    (
        diagnostic.registration.map(CheckerRegistrationId::index),
        diagnostic_class_rank(diagnostic.class),
        diagnostic_severity_rank(diagnostic.severity),
        diagnostic.message_key.clone(),
        diagnostic.detail.clone().unwrap_or_default(),
    )
}

fn cluster_audit_key(rule: &ClusterRuleInput, active: &ActivatedRegistration) -> ClusterAuditKey {
    cluster_audit_key_for(
        &rule.source_type,
        &rule.generated_attribute,
        &rule.rule_fingerprint,
        active,
    )
}

fn cluster_audit_key_for(
    source_type: &ClusterTypeFingerprint,
    generated_attribute: &ClusterAttributeFingerprint,
    rule_fingerprint: &ClusterRuleFingerprint,
    active: &ActivatedRegistration,
) -> ClusterAuditKey {
    let origin = active.source().origin();
    ClusterAuditKey::new(format!(
        "source={};module={}::{};path={};resolver={};attribute={};fingerprint={}",
        source_type.as_str(),
        origin.module_id().package().as_str(),
        origin.module_id().path().as_str(),
        format_u32_path(origin.structural_path()),
        active.resolver_registration().index(),
        generated_attribute.as_str(),
        rule_fingerprint.as_str()
    ))
}

fn active_rule_fingerprint(active: &ActivatedRegistration) -> &str {
    active.fingerprint().map_or_else(
        || active.pattern().as_str(),
        |fingerprint| fingerprint.as_str(),
    )
}

fn format_u32_path(path: &[u32]) -> String {
    let mut output = String::new();
    for (index, value) in path.iter().enumerate() {
        if index > 0 {
            output.push('.');
        }
        let _ = write!(output, "{value}");
    }
    output
}

fn diagnostic_class_rank(class: ClusterDiagnosticClass) -> u8 {
    match class {
        ClusterDiagnosticClass::InvisibleRegistration => 0,
        ClusterDiagnosticClass::InvalidRulePayload => 1,
        ClusterDiagnosticClass::ClusterLoop => 2,
        ClusterDiagnosticClass::ClusterBoundExceeded => 3,
        ClusterDiagnosticClass::ClusterContradiction => 4,
        ClusterDiagnosticClass::ReplayFailure => 5,
    }
}

fn diagnostic_severity_rank(severity: ClusterDiagnosticSeverity) -> u8 {
    match severity {
        ClusterDiagnosticSeverity::Error => 0,
        ClusterDiagnosticSeverity::Warning => 1,
        ClusterDiagnosticSeverity::Note => 2,
    }
}

fn cluster_closure_status_name(status: ClusterClosureStatus) -> &'static str {
    match status {
        ClusterClosureStatus::Complete => "complete",
        ClusterClosureStatus::Incomplete => "incomplete",
    }
}

fn write_trace(output: &mut String, trace: &ResolutionTrace) {
    output.push_str("trace:\n");
    let _ = writeln!(
        output,
        "  source={:?} module={}::{}",
        trace.source_id,
        trace.module_id.package().as_str(),
        trace.module_id.path().as_str()
    );
    write_profile(output, &trace.traversal_profile);
    output.push_str("  steps:\n");
    if trace.steps.is_empty() {
        output.push_str("    <none>\n");
    }
    for step in &trace.steps {
        let ResolutionTraceStep::Cluster(step) = step;
        let _ = write!(
            output,
            "    cluster#{} registration#{} resolver#{} kind={} source=\"",
            step.id.index(),
            step.applied_cluster.index(),
            step.resolver_registration.index(),
            cluster_rule_kind_name(step.rule_kind)
        );
        write_escaped(output, step.source_type.as_str());
        output.push_str("\" antecedents=");
        write_fact_fingerprints(output, &step.antecedents);
        output.push_str(" generated=\"");
        write_escaped(output, step.generated_fact.as_str());
        output.push_str("\" attribute=\"");
        write_escaped(output, step.generated_attribute.as_str());
        output.push_str("\" type=\"");
        write_escaped(output, step.generated_type.as_str());
        output.push_str("\" fingerprint=\"");
        write_escaped(output, step.rule_fingerprint.as_str());
        output.push_str("\" audit=\"");
        write_escaped(output, step.audit_key.as_str());
        output.push_str("\"\n");
    }
    write_facts(output, "  derived-facts", &trace.derived_facts);
}

fn write_profile(output: &mut String, profile: &ClusterTraversalProfile) {
    let _ = writeln!(
        output,
        "  profile order=\"{}\" cache=\"{}\" max_depth={} max_generated={} bounded={} input={} derived={} cluster_steps={} reduction_steps={} diagnostics={}",
        escaped_display(profile.ordering_version.as_str()),
        escaped_display(profile.cache_key_material.as_str()),
        profile.max_cluster_depth,
        profile.max_generated_facts,
        profile.bounded_saturation_reached,
        profile.input_fact_count,
        profile.derived_fact_count,
        profile.cluster_step_count,
        profile.reduction_step_count,
        profile.diagnostic_count
    );
}

fn write_facts(output: &mut String, label: &str, facts: &ClusterFactTable) {
    let _ = writeln!(output, "{label}:");
    if facts.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, fact) in facts.canonical_iter() {
        let _ = write!(output, "  fact#{} fingerprint=\"", id.index());
        write_escaped(output, fact.fingerprint.as_str());
        output.push_str("\" source=\"");
        write_escaped(output, fact.source_type.as_str());
        output.push_str("\" attribute=\"");
        write_escaped(output, fact.attribute.as_str());
        output.push_str("\" type=\"");
        write_escaped(output, fact.generated_type.as_str());
        output.push_str("\" provenance=");
        write_fact_provenance(output, &fact.provenance);
        output.push('\n');
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &ClusterDiagnosticTable) {
    output.push_str("diagnostics:\n");
    if diagnostics.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, diagnostic) in diagnostics.canonical_iter() {
        let _ = write!(output, "  diagnostic#{} registration=", id.index());
        if let Some(registration) = diagnostic.registration {
            let _ = write!(output, "registration#{}", registration.index());
        } else {
            output.push_str("<none>");
        }
        output.push_str(" class=");
        output.push_str(diagnostic_class_name(diagnostic.class));
        output.push_str(" severity=");
        output.push_str(diagnostic_severity_name(diagnostic.severity));
        output.push_str(" message=\"");
        write_escaped(output, &diagnostic.message_key);
        output.push_str("\" detail=");
        if let Some(detail) = &diagnostic.detail {
            output.push('"');
            write_escaped(output, detail);
            output.push('"');
        } else {
            output.push_str("<none>");
        }
        output.push_str(" recovery=");
        output.push_str(diagnostic_recovery_name(diagnostic.recovery));
        output.push('\n');
    }
}

fn write_fact_fingerprints(output: &mut String, facts: &[ClusterFactFingerprint]) {
    output.push('[');
    for (index, fact) in facts.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('"');
        write_escaped(output, fact.as_str());
        output.push('"');
    }
    output.push(']');
}

fn write_fact_provenance(output: &mut String, provenance: &ClusterFactProvenance) {
    match provenance {
        ClusterFactProvenance::Input => output.push_str("input"),
        ClusterFactProvenance::TraceStep(step) => {
            let _ = write!(output, "cluster_step#{}", step.index());
        }
    }
}

fn registration_kind_name(kind: ResolverRegistrationKind) -> &'static str {
    match kind {
        ResolverRegistrationKind::Cluster => "cluster",
        ResolverRegistrationKind::Identify => "identify",
        ResolverRegistrationKind::Reduction => "reduction",
        ResolverRegistrationKind::Property => "property",
        _ => "unknown",
    }
}

fn cluster_rule_kind_name(kind: ClusterRuleKind) -> &'static str {
    match kind {
        ClusterRuleKind::Conditional => "conditional",
        ClusterRuleKind::Functorial => "functorial",
        ClusterRuleKind::Existential => "existential",
    }
}

fn diagnostic_class_name(class: ClusterDiagnosticClass) -> &'static str {
    match class {
        ClusterDiagnosticClass::InvisibleRegistration => "invisible_registration",
        ClusterDiagnosticClass::InvalidRulePayload => "invalid_rule_payload",
        ClusterDiagnosticClass::ClusterLoop => "cluster_loop",
        ClusterDiagnosticClass::ClusterBoundExceeded => "cluster_bound_exceeded",
        ClusterDiagnosticClass::ClusterContradiction => "cluster_contradiction",
        ClusterDiagnosticClass::ReplayFailure => "replay_failure",
    }
}

fn diagnostic_severity_name(severity: ClusterDiagnosticSeverity) -> &'static str {
    match severity {
        ClusterDiagnosticSeverity::Error => "error",
        ClusterDiagnosticSeverity::Warning => "warning",
        ClusterDiagnosticSeverity::Note => "note",
    }
}

fn diagnostic_recovery_name(recovery: ClusterDiagnosticRecovery) -> &'static str {
    match recovery {
        ClusterDiagnosticRecovery::Normal => "normal",
        ClusterDiagnosticRecovery::Degraded => "degraded",
        ClusterDiagnosticRecovery::Fatal => "fatal",
    }
}

fn escaped_display(value: &str) -> String {
    let mut output = String::new();
    write_escaped(&mut output, value);
    output
}

fn write_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            _ => output.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::registration_resolution::{
        ActivationInput, RegistrationFingerprint as ActiveRegistrationFingerprint,
    };
    use mizar_resolve::{
        env::{
            RegistrationIndex, SignatureShell, SourceContributionId, SourceContributionIndex,
            SymbolEnv, SymbolEnvIndexes,
        },
        resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin, SymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceAnchor,
    };

    #[test]
    fn closure_records_replayable_cluster_steps_and_derived_facts() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B", 20)],
        );

        assert!(output.diagnostics().is_empty());
        assert_eq!(output.status(), ClusterClosureStatus::Complete);
        assert_eq!(output.trace().steps().len(), 1);
        assert!(
            output
                .trace()
                .derived_facts()
                .contains(&ClusterFactFingerprint::new("fact:B"))
        );
        let replay = output
            .trace()
            .replay(&fixture.database, [fact("fact:A", "type:T", "attr:A", 10)]);
        assert_eq!(replay.status(), ClusterReplayStatus::Valid);
        assert_eq!(
            fact_fingerprints(output.trace().derived_facts()),
            vec!["fact:B"]
        );
        assert_eq!(
            fact_fingerprints(replay.replayed_facts()),
            vec!["fact:A", "fact:B"]
        );
        let derived = output
            .trace()
            .derived_facts()
            .get_by_fingerprint(&ClusterFactFingerprint::new("fact:B"))
            .unwrap();
        assert!(matches!(
            derived.provenance(),
            ClusterFactProvenance::TraceStep(step) if *step == ClusterStepId::new(0)
        ));
    }

    #[test]
    fn inactive_and_non_cluster_registrations_do_not_fire() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(
                    fixture.pending_cluster,
                    "trigger:pending",
                    "fact:A",
                    "fact:P",
                    30,
                ),
                rule(
                    fixture.malformed_cluster,
                    "trigger:bad",
                    "fact:A",
                    "fact:M",
                    31,
                ),
                rule(fixture.reduction, "trigger:R", "fact:A", "fact:R", 32),
                rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:X", 33)
                    .with_kind(ClusterRuleKind::Existential),
            ],
        );

        assert_eq!(output.trace().steps().len(), 0);
        assert_eq!(output.trace().derived_facts().len(), 0);
        assert_messages(
            output.diagnostics(),
            &[
                "checker.cluster_trace.unsupported_rule_kind",
                "checker.cluster_trace.invisible_registration",
                "checker.cluster_trace.invisible_registration",
                "checker.cluster_trace.non_cluster_registration",
            ],
        );
    }

    #[test]
    fn closure_order_is_deterministic_across_rule_and_activation_permutations() {
        let fixture = env_fixture_with_activation_order(false);
        let reversed = env_fixture_with_activation_order(true);
        let rules = [
            rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:C", 22),
            rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B", 20),
        ];
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            rules,
        );
        let reversed_output = ClusterTraceBuilder::default().close(
            &reversed.database,
            reversed.source_id,
            reversed.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(reversed.cluster_a, "trigger:A", "fact:A", "fact:B", 20),
                rule(reversed.cluster_b, "trigger:B", "fact:A", "fact:C", 22),
            ],
        );

        let generated = generated_facts(output.trace());
        let reversed_generated = generated_facts(reversed_output.trace());
        assert_eq!(generated, vec!["fact:B", "fact:C"]);
        assert_eq!(generated, reversed_generated);
        assert_eq!(output.debug_text(), reversed_output.debug_text());
    }

    #[test]
    fn conditional_clusters_record_all_antecedent_fact_refs() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [
                fact("fact:A", "type:T", "attr:A", 10),
                fact("fact:C", "type:T", "attr:C", 11),
            ],
            [
                rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:D", 40).with_antecedents(
                    vec![
                        ClusterFactFingerprint::new("fact:C"),
                        ClusterFactFingerprint::new("fact:A"),
                    ],
                ),
            ],
        );

        let [ResolutionTraceStep::Cluster(step)] = output.trace().steps() else {
            panic!("expected one cluster step");
        };
        assert_eq!(
            step.antecedents(),
            &[
                ClusterFactFingerprint::new("fact:A"),
                ClusterFactFingerprint::new("fact:C"),
            ]
        );
    }

    #[test]
    fn subtype_compatible_payloads_are_preserved_in_conditional_steps() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:Sub:A", "type:SubT", "attr:A", 10)],
            [rule_with_types(
                fixture.cluster_a,
                "trigger:A",
                "fact:Sub:A",
                "fact:Sub:B",
                "type:SubT",
                "type:SubT",
                41,
            )],
        );

        let [ResolutionTraceStep::Cluster(step)] = output.trace().steps() else {
            panic!("expected one cluster step");
        };
        assert_eq!(step.source_type().as_str(), "type:SubT");
        assert_eq!(step.generated_type().as_str(), "type:SubT");
        assert_eq!(step.rule_source_range(), range(source_id(), 41, 42));
        assert_eq!(
            step.antecedent_sources(),
            &[ClusterAntecedentRef {
                fingerprint: ClusterFactFingerprint::new("fact:Sub:A"),
                source_range: range(source_id(), 10, 11),
            }]
        );
    }

    #[test]
    fn transitive_chains_preserve_every_intermediate_step() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B", 20),
                rule(fixture.cluster_b, "trigger:B", "fact:B", "fact:C", 21),
            ],
        );

        assert_eq!(generated_facts(output.trace()), vec!["fact:B", "fact:C"]);
        assert_ordered_fragments(
            &output.debug_text(),
            &[
                "cluster#0 registration#0",
                "generated=\"fact:B\"",
                "cluster#1 registration#1",
                "antecedents=[\"fact:B\"]",
                "generated=\"fact:C\"",
            ],
        );
    }

    #[test]
    fn mismatched_active_payloads_are_rejected_before_firing() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(fixture.cluster_a, "wrong-trigger", "fact:A", "fact:B", 20),
                rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:C", 21)
                    .with_rule_fingerprint("wrong-fingerprint"),
            ],
        );

        assert_eq!(output.trace().steps().len(), 0);
        assert_messages(
            output.diagnostics(),
            &[
                "checker.cluster_trace.trigger_mismatch",
                "checker.cluster_trace.fingerprint_mismatch",
            ],
        );
    }

    #[test]
    fn rejected_recovered_and_unaccepted_registrations_do_not_fire() {
        let fixture = rejected_env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(
                    fixture.missing_pattern,
                    "trigger:missing",
                    "fact:A",
                    "fact:missing",
                    50,
                ),
                rule(
                    fixture.recovered,
                    "trigger:recovered",
                    "fact:A",
                    "fact:recovered",
                    51,
                ),
            ],
        );

        assert_eq!(output.trace().steps().len(), 0);
        assert_messages(
            output.diagnostics(),
            &[
                "checker.cluster_trace.invisible_registration",
                "checker.cluster_trace.invisible_registration",
            ],
        );
    }

    #[test]
    fn duplicate_generated_fingerprints_are_deduplicated_before_step_emission() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:B", 21),
                rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B", 20),
            ],
        );

        assert_eq!(output.trace().steps().len(), 1);
        assert!(output.diagnostics().is_empty());
        assert_eq!(output.status(), ClusterClosureStatus::Complete);
        assert_eq!(
            fact_fingerprints(output.trace().derived_facts()),
            vec!["fact:B"]
        );
        let [ResolutionTraceStep::Cluster(step)] = output.trace().steps() else {
            panic!("expected one cluster step");
        };
        assert_eq!(step.applied_cluster(), fixture.cluster_a);
    }

    #[test]
    fn distinct_generated_fingerprints_are_not_deduplicated_by_payload_collision() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B-left", 23)
                    .with_generated_attribute("attr:shared"),
                rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:B-right", 24)
                    .with_generated_attribute("attr:shared"),
            ],
        );

        assert!(output.diagnostics().is_empty());
        assert_eq!(output.status(), ClusterClosureStatus::Complete);
        assert_eq!(
            generated_facts(output.trace()),
            vec!["fact:B-left", "fact:B-right"]
        );
        assert_eq!(
            fact_fingerprints(output.trace().derived_facts()),
            vec!["fact:B-left", "fact:B-right"]
        );
    }

    #[test]
    fn direct_and_indirect_cluster_loops_are_diagnosed_without_insertion() {
        let fixture = env_fixture();
        let direct = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:A", 70)],
        );

        assert_eq!(direct.status(), ClusterClosureStatus::Incomplete);
        assert_eq!(direct.trace().steps().len(), 0);
        assert_messages(
            direct.diagnostics(),
            &["checker.cluster_trace.cluster_loop"],
        );
        assert_eq!(fact_fingerprints(direct.closure_facts()), vec!["fact:A"]);

        let indirect = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B", 71),
                rule(fixture.cluster_b, "trigger:B", "fact:B", "fact:A", 72),
            ],
        );

        assert_eq!(indirect.status(), ClusterClosureStatus::Incomplete);
        assert_eq!(generated_facts(indirect.trace()), vec!["fact:B"]);
        assert_messages(
            indirect.diagnostics(),
            &["checker.cluster_trace.cluster_loop"],
        );
        assert_eq!(
            fact_fingerprints(indirect.closure_facts()),
            vec!["fact:A", "fact:B"]
        );
    }

    #[test]
    fn cluster_depth_and_generated_fact_bounds_are_visible_and_do_not_truncate() {
        let fixture = env_fixture();
        let depth_limited = ClusterTraceBuilder::new(ClusterTraversalConfig {
            max_cluster_depth: 1,
            max_generated_facts: 8,
        })
        .close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B", 80),
                rule(fixture.cluster_b, "trigger:B", "fact:B", "fact:C", 81),
                rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:D", 82),
            ],
        );

        assert_eq!(depth_limited.status(), ClusterClosureStatus::Incomplete);
        assert_eq!(generated_facts(depth_limited.trace()), vec!["fact:B"]);
        assert_eq!(
            depth_limited
                .trace()
                .traversal_profile()
                .cache_key_material()
                .as_str(),
            "order=cluster-trace-order-v1;max_depth=1;max_generated=8"
        );
        assert_eq!(
            depth_limited
                .trace()
                .traversal_profile()
                .max_cluster_depth(),
            1
        );
        assert!(
            depth_limited
                .trace()
                .traversal_profile()
                .bounded_saturation_reached()
        );
        assert_messages(
            depth_limited.diagnostics(),
            &["checker.cluster_trace.cluster_bound_exceeded"],
        );
        assert_eq!(
            fact_fingerprints(depth_limited.closure_facts()),
            vec!["fact:A", "fact:B"]
        );
        assert!(
            !depth_limited
                .closure_facts()
                .contains(&ClusterFactFingerprint::new("fact:D"))
        );

        let generated_limited = ClusterTraceBuilder::new(ClusterTraversalConfig {
            max_cluster_depth: 8,
            max_generated_facts: 1,
        })
        .close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B", 82),
                rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:C", 83),
            ],
        );

        assert_eq!(generated_limited.status(), ClusterClosureStatus::Incomplete);
        assert_eq!(generated_facts(generated_limited.trace()), vec!["fact:B"]);
        assert_eq!(
            generated_limited
                .trace()
                .traversal_profile()
                .cache_key_material()
                .as_str(),
            "order=cluster-trace-order-v1;max_depth=8;max_generated=1"
        );
        assert_eq!(
            generated_limited
                .trace()
                .traversal_profile()
                .max_generated_facts(),
            1
        );
        assert!(
            generated_limited
                .trace()
                .traversal_profile()
                .bounded_saturation_reached()
        );
        assert_messages(
            generated_limited.diagnostics(),
            &["checker.cluster_trace.cluster_bound_exceeded"],
        );
        assert_eq!(
            fact_fingerprints(generated_limited.closure_facts()),
            vec!["fact:A", "fact:B"]
        );
    }

    #[test]
    fn zero_antecedent_clusters_have_depth_one_for_bounds() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::new(ClusterTraversalConfig {
            max_cluster_depth: 0,
            max_generated_facts: 8,
        })
        .close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [],
            [rule(fixture.cluster_a, "trigger:A", "unused", "fact:B", 84).with_antecedents([])],
        );

        assert_eq!(output.status(), ClusterClosureStatus::Incomplete);
        assert_eq!(output.trace().steps().len(), 0);
        assert!(
            output
                .trace()
                .traversal_profile()
                .bounded_saturation_reached()
        );
        assert_messages(
            output.diagnostics(),
            &["checker.cluster_trace.cluster_bound_exceeded"],
        );
    }

    #[test]
    fn explicit_cluster_contradictions_are_fatal_without_degraded_facts() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [
                rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:not-B", 89),
                rule(fixture.cluster_b, "trigger:B", "fact:A", "fact:B", 90)
                    .with_conflicts([ClusterFactFingerprint::new("fact:not-B")]),
            ],
        );

        assert_eq!(output.status(), ClusterClosureStatus::Incomplete);
        assert_eq!(generated_facts(output.trace()), vec!["fact:not-B"]);
        assert!(
            output
                .closure_facts()
                .contains(&ClusterFactFingerprint::new("fact:not-B"))
        );
        assert!(
            !output
                .closure_facts()
                .contains(&ClusterFactFingerprint::new("fact:B"))
        );
        assert!(
            !output
                .trace()
                .derived_facts()
                .contains(&ClusterFactFingerprint::new("fact:B"))
        );
        assert_messages(
            output.diagnostics(),
            &["checker.cluster_trace.cluster_contradiction"],
        );
    }

    #[test]
    fn replay_revalidates_active_registration_fingerprint() {
        let fixture = env_fixture();
        let output = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [rule(fixture.cluster_a, "trigger:A", "fact:A", "fact:B", 20)],
        );
        let replay_fixture = pattern_fallback_env_fixture();

        let replay = output.trace().replay(
            &replay_fixture.database,
            [fact("fact:A", "type:T", "attr:A", 10)],
        );

        assert_eq!(replay.status(), ClusterReplayStatus::Invalid);
        assert_messages(
            replay.diagnostics(),
            &["checker.cluster_trace.replay_fingerprint_mismatch"],
        );
    }

    #[test]
    fn active_pattern_fallback_must_match_rule_fingerprint() {
        let fixture = pattern_fallback_env_fixture();
        let accepted = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [rule(
                fixture.cluster,
                "trigger:no-fingerprint",
                "fact:A",
                "fact:B",
                60,
            )
            .with_rule_fingerprint("pattern:trigger:no-fingerprint")],
        );
        assert_eq!(accepted.trace().steps().len(), 1);

        let rejected = ClusterTraceBuilder::default().close(
            &fixture.database,
            fixture.source_id,
            fixture.module.clone(),
            [fact("fact:A", "type:T", "attr:A", 10)],
            [rule(
                fixture.cluster,
                "trigger:no-fingerprint",
                "fact:A",
                "fact:C",
                61,
            )
            .with_rule_fingerprint("invented:fingerprint")],
        );
        assert_eq!(rejected.trace().steps().len(), 0);
        assert_messages(
            rejected.diagnostics(),
            &["checker.cluster_trace.fingerprint_mismatch"],
        );
    }

    struct EnvFixture {
        database: RegistrationDatabase,
        source_id: SourceId,
        module: ModuleId,
        cluster_a: CheckerRegistrationId,
        cluster_b: CheckerRegistrationId,
        pending_cluster: CheckerRegistrationId,
        malformed_cluster: CheckerRegistrationId,
        reduction: CheckerRegistrationId,
    }

    struct RejectedEnvFixture {
        database: RegistrationDatabase,
        source_id: SourceId,
        module: ModuleId,
        missing_pattern: CheckerRegistrationId,
        recovered: CheckerRegistrationId,
    }

    struct PatternFallbackEnvFixture {
        database: RegistrationDatabase,
        source_id: SourceId,
        module: ModuleId,
        cluster: CheckerRegistrationId,
    }

    trait RuleInputTestExt {
        fn with_kind(self, kind: ClusterRuleKind) -> Self;
        fn with_rule_fingerprint(self, fingerprint: &str) -> Self;
        fn with_generated_attribute(self, attribute: &str) -> Self;
    }

    impl RuleInputTestExt for ClusterRuleInput {
        fn with_kind(mut self, kind: ClusterRuleKind) -> Self {
            self.kind = kind;
            self
        }

        fn with_rule_fingerprint(mut self, fingerprint: &str) -> Self {
            self.rule_fingerprint = ClusterRuleFingerprint::new(fingerprint);
            self
        }

        fn with_generated_attribute(mut self, attribute: &str) -> Self {
            self.generated_attribute = ClusterAttributeFingerprint::new(attribute);
            self
        }
    }

    fn env_fixture() -> EnvFixture {
        env_fixture_with_activation_order(false)
    }

    fn env_fixture_with_activation_order(reverse_activations: bool) -> EnvFixture {
        let source_id = source_id();
        let module = module_id();
        let mut contributions = SourceContributionIndex::new();
        let contribution_a = contribution(&mut contributions, module.clone(), source_id, 0);
        let contribution_b = contribution(&mut contributions, module.clone(), source_id, 1);
        let contribution_pending = contribution(&mut contributions, module.clone(), source_id, 2);
        let contribution_malformed = contribution(&mut contributions, module.clone(), source_id, 3);
        let contribution_reduction = contribution(&mut contributions, module.clone(), source_id, 4);

        let mut registrations = RegistrationIndex::new();
        let cluster_a = insert_registration(
            &mut registrations,
            module.clone(),
            source_id,
            contribution_a,
            RegistrationSpec::new(
                ResolverRegistrationKind::Cluster,
                "ACluster",
                0,
                SignatureShell::Pending,
            ),
        );
        let cluster_b = insert_registration(
            &mut registrations,
            module.clone(),
            source_id,
            contribution_b,
            RegistrationSpec::new(
                ResolverRegistrationKind::Cluster,
                "BCluster",
                1,
                SignatureShell::Pending,
            ),
        );
        let pending_cluster = insert_registration(
            &mut registrations,
            module.clone(),
            source_id,
            contribution_pending,
            RegistrationSpec::new(
                ResolverRegistrationKind::Cluster,
                "PendingCluster",
                2,
                SignatureShell::Pending,
            ),
        );
        let malformed_cluster = insert_registration(
            &mut registrations,
            module.clone(),
            source_id,
            contribution_malformed,
            RegistrationSpec::new(
                ResolverRegistrationKind::Cluster,
                "MalformedCluster",
                3,
                SignatureShell::Malformed {
                    class: "recovered-target".to_owned(),
                },
            ),
        );
        let reduction = insert_registration(
            &mut registrations,
            module.clone(),
            source_id,
            contribution_reduction,
            RegistrationSpec::new(
                ResolverRegistrationKind::Reduction,
                "ReductionRule",
                4,
                SignatureShell::Pending,
            ),
        );
        contributions.add_registration(contribution_a, cluster_a);
        contributions.add_registration(contribution_b, cluster_b);
        contributions.add_registration(contribution_pending, pending_cluster);
        contributions.add_registration(contribution_malformed, malformed_cluster);
        contributions.add_registration(contribution_reduction, reduction);

        let indexes = SymbolEnvIndexes {
            registrations,
            contributions,
            ..SymbolEnvIndexes::default()
        };
        let env = SymbolEnv::new(module.clone(), indexes);
        let mut activations = vec![
            activation(
                cluster_a,
                ResolverRegistrationKind::Cluster,
                "trigger:A",
                "fingerprint:A",
            ),
            activation(
                cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:B",
                "fingerprint:B",
            ),
            activation(
                reduction,
                ResolverRegistrationKind::Reduction,
                "trigger:R",
                "fingerprint:R",
            ),
        ];
        if reverse_activations {
            activations.reverse();
        }
        let database = RegistrationDatabase::from_symbol_env(&env, activations);

        EnvFixture {
            database,
            source_id,
            module,
            cluster_a: CheckerRegistrationId::new(cluster_a.index()),
            cluster_b: CheckerRegistrationId::new(cluster_b.index()),
            pending_cluster: CheckerRegistrationId::new(pending_cluster.index()),
            malformed_cluster: CheckerRegistrationId::new(malformed_cluster.index()),
            reduction: CheckerRegistrationId::new(reduction.index()),
        }
    }

    fn rejected_env_fixture() -> RejectedEnvFixture {
        let source_id = source_id();
        let module = module_id();
        let mut contributions = SourceContributionIndex::new();
        let contribution_missing = contribution(&mut contributions, module.clone(), source_id, 0);
        let contribution_recovered = contribution(&mut contributions, module.clone(), source_id, 1);

        let mut registrations = RegistrationIndex::new();
        let missing_pattern = insert_registration(
            &mut registrations,
            module.clone(),
            source_id,
            contribution_missing,
            RegistrationSpec::new(
                ResolverRegistrationKind::Cluster,
                "MissingPatternCluster",
                0,
                SignatureShell::Pending,
            ),
        );
        let recovered = registrations.insert(
            Some(symbol_id(
                module.clone(),
                "RecoveredCluster",
                "pkg::main::RecoveredCluster",
            )),
            ResolverRegistrationKind::Cluster,
            SignatureShell::Pending,
            SemanticOrigin::new(
                source_id,
                module.clone(),
                SourceAnchor::Range(range(source_id, 1, 2)),
                vec![1],
            )
            .recovered(),
            contribution_recovered,
        );
        contributions.add_registration(contribution_missing, missing_pattern);
        contributions.add_registration(contribution_recovered, recovered);

        let indexes = SymbolEnvIndexes {
            registrations,
            contributions,
            ..SymbolEnvIndexes::default()
        };
        let env = SymbolEnv::new(module.clone(), indexes);
        let database = RegistrationDatabase::from_symbol_env(
            &env,
            [
                ActivationInput::new(
                    missing_pattern,
                    ResolverRegistrationKind::Cluster,
                    "trigger:missing",
                    " ",
                    "correctness:missing",
                    "evidence:missing",
                ),
                ActivationInput::new(
                    recovered,
                    ResolverRegistrationKind::Cluster,
                    "trigger:recovered",
                    "pattern:recovered",
                    "correctness:recovered",
                    "evidence:recovered",
                ),
            ],
        );

        RejectedEnvFixture {
            database,
            source_id,
            module,
            missing_pattern: CheckerRegistrationId::new(missing_pattern.index()),
            recovered: CheckerRegistrationId::new(recovered.index()),
        }
    }

    fn pattern_fallback_env_fixture() -> PatternFallbackEnvFixture {
        let source_id = source_id();
        let module = module_id();
        let mut contributions = SourceContributionIndex::new();
        let contribution = contribution(&mut contributions, module.clone(), source_id, 0);

        let mut registrations = RegistrationIndex::new();
        let cluster = insert_registration(
            &mut registrations,
            module.clone(),
            source_id,
            contribution,
            RegistrationSpec::new(
                ResolverRegistrationKind::Cluster,
                "PatternFallbackCluster",
                0,
                SignatureShell::Pending,
            ),
        );
        contributions.add_registration(contribution, cluster);

        let indexes = SymbolEnvIndexes {
            registrations,
            contributions,
            ..SymbolEnvIndexes::default()
        };
        let env = SymbolEnv::new(module.clone(), indexes);
        let database = RegistrationDatabase::from_symbol_env(
            &env,
            [ActivationInput::new(
                cluster,
                ResolverRegistrationKind::Cluster,
                "trigger:no-fingerprint",
                "pattern:trigger:no-fingerprint",
                "correctness:no-fingerprint",
                "evidence:no-fingerprint",
            )],
        );

        PatternFallbackEnvFixture {
            database,
            source_id,
            module,
            cluster: CheckerRegistrationId::new(cluster.index()),
        }
    }

    fn activation(
        id: ResolverRegistrationId,
        kind: ResolverRegistrationKind,
        trigger: &str,
        fingerprint: &str,
    ) -> ActivationInput {
        ActivationInput::new(
            id,
            kind,
            trigger,
            format!("pattern:{trigger}"),
            format!("correctness:{trigger}"),
            format!("evidence:{trigger}"),
        )
        .with_fingerprint(ActiveRegistrationFingerprint::new(fingerprint))
    }

    struct RegistrationSpec {
        kind: ResolverRegistrationKind,
        local: String,
        path: u32,
        shell: SignatureShell,
    }

    impl RegistrationSpec {
        fn new(
            kind: ResolverRegistrationKind,
            local: &str,
            path: u32,
            shell: SignatureShell,
        ) -> Self {
            Self {
                kind,
                local: local.to_owned(),
                path,
                shell,
            }
        }
    }

    fn insert_registration(
        registrations: &mut RegistrationIndex,
        module: ModuleId,
        source_id: SourceId,
        contribution: SourceContributionId,
        spec: RegistrationSpec,
    ) -> ResolverRegistrationId {
        registrations.insert(
            Some(symbol_id(
                module.clone(),
                &spec.local,
                &format!("pkg::main::{}", spec.local),
            )),
            spec.kind,
            spec.shell,
            SemanticOrigin::new(
                source_id,
                module,
                SourceAnchor::Range(SourceRange {
                    source_id,
                    start: spec.path as usize,
                    end: spec.path as usize + 1,
                }),
                vec![spec.path],
            ),
            contribution,
        )
    }

    fn contribution(
        contributions: &mut SourceContributionIndex,
        module: ModuleId,
        source_id: SourceId,
        offset: usize,
    ) -> SourceContributionId {
        contributions.insert(
            module,
            mizar_resolve::env::ContributionKind::LocalSource { source_id },
            SourceAnchor::Range(range(source_id, offset, offset + 1)),
        )
    }

    fn fact(
        fingerprint: &str,
        source_type: &str,
        attribute: &str,
        start: usize,
    ) -> ClusterFactInput {
        ClusterFactInput::new(
            fingerprint,
            source_type,
            attribute,
            source_type,
            range(source_id(), start, start + 1),
        )
    }

    fn rule(
        registration: CheckerRegistrationId,
        trigger: &str,
        antecedent: &str,
        generated: &str,
        start: usize,
    ) -> ClusterRuleInput {
        let fingerprint = match trigger {
            "trigger:A" => "fingerprint:A",
            "trigger:B" => "fingerprint:B",
            "trigger:R" => "fingerprint:R",
            _ => "fingerprint:unknown",
        };
        ClusterRuleInput::new(ClusterRuleDraft {
            registration,
            kind: ClusterRuleKind::Conditional,
            trigger: RegistrationTriggerKey::new(trigger),
            source_type: ClusterTypeFingerprint::new("type:T"),
            generated_attribute: ClusterAttributeFingerprint::new(format!("attr:{generated}")),
            generated_type: ClusterTypeFingerprint::new("type:T"),
            generated_fact: ClusterFactFingerprint::new(generated),
            rule_fingerprint: ClusterRuleFingerprint::new(fingerprint),
            source_range: range(source_id(), start, start + 1),
        })
        .with_antecedents(vec![ClusterFactFingerprint::new(antecedent)])
    }

    fn rule_with_types(
        registration: CheckerRegistrationId,
        trigger: &str,
        antecedent: &str,
        generated: &str,
        source_type: &str,
        generated_type: &str,
        start: usize,
    ) -> ClusterRuleInput {
        let fingerprint = match trigger {
            "trigger:A" => "fingerprint:A",
            "trigger:B" => "fingerprint:B",
            "trigger:R" => "fingerprint:R",
            _ => "fingerprint:unknown",
        };
        ClusterRuleInput::new(ClusterRuleDraft {
            registration,
            kind: ClusterRuleKind::Conditional,
            trigger: RegistrationTriggerKey::new(trigger),
            source_type: ClusterTypeFingerprint::new(source_type),
            generated_attribute: ClusterAttributeFingerprint::new(format!("attr:{generated}")),
            generated_type: ClusterTypeFingerprint::new(generated_type),
            generated_fact: ClusterFactFingerprint::new(generated),
            rule_fingerprint: ClusterRuleFingerprint::new(fingerprint),
            source_range: range(source_id(), start, start + 1),
        })
        .with_antecedents(vec![ClusterFactFingerprint::new(antecedent)])
    }

    fn fact_fingerprints(facts: &ClusterFactTable) -> Vec<&str> {
        facts
            .canonical_iter()
            .map(|(_, fact)| fact.fingerprint().as_str())
            .collect()
    }

    fn generated_facts(trace: &ResolutionTrace) -> Vec<&str> {
        trace
            .steps()
            .iter()
            .map(|step| {
                let ResolutionTraceStep::Cluster(step) = step;
                step.generated_fact().as_str()
            })
            .collect()
    }

    fn assert_messages(diagnostics: &ClusterDiagnosticTable, expected: &[&str]) {
        let actual = diagnostics
            .canonical_iter()
            .map(|(_, diagnostic)| diagnostic.message_key())
            .collect::<Vec<_>>();
        assert_eq!(actual, expected);
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "43".repeat(32)
        ))
        .unwrap();
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .unwrap()
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn symbol_id(module: ModuleId, local: &str, fqn: &str) -> SymbolId {
        SymbolId::new(
            module,
            LocalSymbolId::new(local),
            FullyQualifiedName::new(fqn),
        )
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn assert_ordered_fragments(snapshot: &str, fragments: &[&str]) {
        let mut cursor = 0;
        for fragment in fragments {
            let Some(offset) = snapshot[cursor..].find(fragment) else {
                panic!("missing ordered fragment: {fragment}\n{snapshot}");
            };
            cursor += offset + fragment.len();
        }
    }
}
