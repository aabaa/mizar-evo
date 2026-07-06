//! Registration database, validation, and activation-gating data layer for
//! checker phase 7.

use crate::typed_ast::{
    InitialObligationDraft, InitialObligationGoal, InitialObligationId, InitialObligationKind,
    InitialObligationProvenance, InitialObligationStatus, InitialObligationTable, TypeFactId,
    TypedSiteRef,
};

use mizar_resolve::{
    env::{
        DeclarationDependencyId, ExportStatus, RegistrationId as ResolverRegistrationId,
        RegistrationKind as ResolverRegistrationKind, SignatureShell, SourceContributionId,
        SymbolEnv, Visibility,
    },
    resolved_ast::{ModuleId, RecoveryState, SemanticOrigin, SymbolId},
};
use mizar_session::{GeneratedSpanAnchor, SourceAnchor, SourceRange};
use std::{
    collections::BTreeMap,
    fmt::{self, Write as _},
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

dense_id!(CheckerRegistrationId);
dense_id!(RejectedRegistrationId);
dense_id!(RegistrationDiagnosticId);
dense_id!(ExistentialGateId);

string_key!(RegistrationTriggerKey);
string_key!(RegistrationLabelKey);
string_key!(RegistrationPatternKey);
string_key!(RegistrationParameterKey);
string_key!(AcceptedCorrectnessKey);
string_key!(ActivationEvidenceKey);
string_key!(RegistrationFingerprint);
string_key!(RegistrationTypeKey);
string_key!(RegistrationAttributeKey);
string_key!(RegistrationFunctorKey);
string_key!(RegistrationTermKey);
string_key!(RegistrationVariableKey);
string_key!(ExistentialGateGuardKey);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationDatabase {
    module_id: ModuleId,
    pending: PendingRegistrationTable,
    activated: ActivatedRegistrationIndex,
    rejected: RejectedRegistrationTable,
    diagnostics: RegistrationDiagnosticTable,
    initial_obligations: InitialObligationTable,
}

impl RegistrationDatabase {
    pub fn from_symbol_env(
        symbols: &SymbolEnv,
        activations: impl IntoIterator<Item = ActivationInput>,
    ) -> Self {
        Self::from_symbol_env_with_validation(
            symbols,
            std::iter::empty::<RegistrationValidationInput>(),
            activations,
        )
    }

    pub fn from_symbol_env_with_validation(
        symbols: &SymbolEnv,
        validations: impl IntoIterator<Item = RegistrationValidationInput>,
        activations: impl IntoIterator<Item = ActivationInput>,
    ) -> Self {
        let mut builder = RegistrationDatabaseBuilder::new(symbols.module_id().clone());
        let mut validation_inputs = validation_map(validations);
        let mut activation_inputs = activation_map(activations);

        let mut entries = symbols.registrations().iter().collect::<Vec<_>>();
        entries.sort_by_key(|entry| {
            source_order_key(entry.contribution(), entry.origin(), entry.id())
        });

        for entry in entries {
            let source = RegistrationSource::from_entry(entry);
            let validations = validation_inputs.remove(&entry.id()).unwrap_or_default();
            let inputs = activation_inputs.remove(&entry.id()).unwrap_or_default();
            builder.ingest_resolver_registration(
                entry.id(),
                entry.kind(),
                source,
                validations,
                inputs,
            );
        }

        for inputs in validation_inputs.into_values() {
            for input in inputs {
                builder.reject_unknown_validation(input);
            }
        }

        for inputs in activation_inputs.into_values() {
            for input in inputs {
                builder.reject_unknown_activation(input);
            }
        }

        builder.finish()
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn pending(&self) -> &PendingRegistrationTable {
        &self.pending
    }

    pub const fn activated(&self) -> &ActivatedRegistrationIndex {
        &self.activated
    }

    pub const fn rejected(&self) -> &RejectedRegistrationTable {
        &self.rejected
    }

    pub const fn diagnostics(&self) -> &RegistrationDiagnosticTable {
        &self.diagnostics
    }

    pub const fn initial_obligations(&self) -> &InitialObligationTable {
        &self.initial_obligations
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("registration-database-debug-v1\n");
        output.push_str("module: ");
        write_module_id(&mut output, &self.module_id);
        output.push('\n');
        write_pending(&mut output, &self.pending);
        write_activated(&mut output, &self.activated);
        write_rejected(&mut output, &self.rejected);
        write_diagnostics(&mut output, &self.diagnostics);
        write_initial_obligations(&mut output, &self.initial_obligations);
        output
    }
}

struct RegistrationDatabaseBuilder {
    module_id: ModuleId,
    pending: Vec<PendingRegistration>,
    activated: Vec<ActivatedRegistration>,
    rejected: Vec<RejectedRegistration>,
    diagnostics: RegistrationDiagnosticTable,
    initial_obligations: InitialObligationTable,
}

impl RegistrationDatabaseBuilder {
    fn new(module_id: ModuleId) -> Self {
        Self {
            module_id,
            pending: Vec::new(),
            activated: Vec::new(),
            rejected: Vec::new(),
            diagnostics: RegistrationDiagnosticTable::new(),
            initial_obligations: InitialObligationTable::new(),
        }
    }

    fn ingest_resolver_registration(
        &mut self,
        resolver_registration: ResolverRegistrationId,
        resolver_kind: ResolverRegistrationKind,
        source: RegistrationSource,
        validations: Vec<RegistrationValidationInput>,
        activations: Vec<ActivationInput>,
    ) {
        if matches!(source.target, ResolverTargetShell::Malformed { .. }) {
            self.reject(
                Some(resolver_registration),
                Some(source),
                RejectedRegistrationReason::MalformedResolverTarget,
                "checker.registration.malformed_resolver_target",
                RegistrationDiagnosticClass::MalformedResolverTarget,
                RegistrationDiagnosticSeverity::Error,
            );
            return;
        }

        match activations.as_slice() {
            [] => self.ingest_pending_validation_or_gap(
                resolver_registration,
                resolver_kind,
                source,
                validations,
            ),
            [activation] => {
                let activation = activation.clone();
                match validate_activation_companion_validations(
                    resolver_registration,
                    resolver_kind,
                    source.clone(),
                    &validations,
                ) {
                    Ok(companion_validation_kind) => match validate_activation(
                        resolver_registration,
                        resolver_kind,
                        source.clone(),
                        activation,
                        companion_validation_kind,
                    ) {
                        Ok(activated) => {
                            self.activated.push(activated);
                        }
                        Err((source, reason, message_key, class)) => {
                            let pending_source = (*source).clone();
                            self.reject(
                                Some(resolver_registration),
                                Some(*source),
                                reason,
                                message_key,
                                class,
                                RegistrationDiagnosticSeverity::Error,
                            );
                            self.ingest_pending_validation_or_gap(
                                resolver_registration,
                                resolver_kind,
                                pending_source,
                                validations,
                            );
                        }
                    },
                    Err((source, reason, message_key, class)) => self.reject(
                        Some(resolver_registration),
                        Some(*source),
                        reason,
                        message_key,
                        class,
                        RegistrationDiagnosticSeverity::Error,
                    ),
                }
            }
            _ => {
                let pending_source = source.clone();
                self.reject(
                    Some(resolver_registration),
                    Some(source),
                    RejectedRegistrationReason::DuplicateActivationInput,
                    "checker.registration.duplicate_activation_input",
                    RegistrationDiagnosticClass::InvalidActivation,
                    RegistrationDiagnosticSeverity::Error,
                );
                self.ingest_pending_validation_or_gap(
                    resolver_registration,
                    resolver_kind,
                    pending_source,
                    validations,
                );
            }
        }
    }

    fn ingest_pending_validation_or_gap(
        &mut self,
        resolver_registration: ResolverRegistrationId,
        resolver_kind: ResolverRegistrationKind,
        source: RegistrationSource,
        validations: Vec<RegistrationValidationInput>,
    ) {
        match validations.as_slice() {
            [] => self.pending_external_gap(resolver_registration, source),
            [validation] => {
                let validation = validation.clone();
                match validate_pending_registration(
                    resolver_registration,
                    resolver_kind,
                    source,
                    validation,
                    &mut self.initial_obligations,
                ) {
                    Ok(pending) => self.pending.push(pending),
                    Err((source, reason, message_key, class)) => self.reject(
                        Some(resolver_registration),
                        Some(*source),
                        reason,
                        message_key,
                        class,
                        RegistrationDiagnosticSeverity::Error,
                    ),
                }
            }
            _ => self.reject(
                Some(resolver_registration),
                Some(source),
                RejectedRegistrationReason::DuplicateValidationInput,
                "checker.registration.duplicate_validation_input",
                RegistrationDiagnosticClass::InvalidValidation,
                RegistrationDiagnosticSeverity::Error,
            ),
        }
    }

    fn pending_external_gap(
        &mut self,
        resolver_registration: ResolverRegistrationId,
        source: RegistrationSource,
    ) {
        self.diagnostics.insert(RegistrationDiagnosticDraft {
            resolver_registration: Some(resolver_registration),
            class: RegistrationDiagnosticClass::ExternalDependencyGap,
            severity: RegistrationDiagnosticSeverity::Note,
            message_key: "checker.registration.external.semantic_payload".to_owned(),
            recovery: RegistrationDiagnosticRecovery::Degraded,
        });
        self.pending.push(PendingRegistration {
            id: CheckerRegistrationId::new(resolver_registration.index()),
            resolver_registration,
            source,
            pattern_status: RegistrationPatternStatus::ExternalDependencyGap,
            status: PendingRegistrationStatus::BlockedExternalDependency,
            parameters: Vec::new(),
            obligations: Vec::new(),
        });
    }

    fn reject_unknown_activation(&mut self, input: ActivationInput) {
        self.reject(
            Some(input.resolver_registration),
            None,
            RejectedRegistrationReason::UnknownActivationOrigin,
            "checker.registration.unknown_activation_origin",
            RegistrationDiagnosticClass::InvalidActivation,
            RegistrationDiagnosticSeverity::Error,
        );
    }

    fn reject_unknown_validation(&mut self, input: RegistrationValidationInput) {
        self.reject(
            Some(input.resolver_registration),
            None,
            RejectedRegistrationReason::UnknownValidationOrigin,
            "checker.registration.unknown_validation_origin",
            RegistrationDiagnosticClass::InvalidValidation,
            RegistrationDiagnosticSeverity::Error,
        );
    }

    fn reject(
        &mut self,
        resolver_registration: Option<ResolverRegistrationId>,
        source: Option<RegistrationSource>,
        reason: RejectedRegistrationReason,
        message_key: &str,
        class: RegistrationDiagnosticClass,
        severity: RegistrationDiagnosticSeverity,
    ) {
        let id = RejectedRegistrationId::new(self.rejected.len());
        self.diagnostics.insert(RegistrationDiagnosticDraft {
            resolver_registration,
            class,
            severity,
            message_key: message_key.to_owned(),
            recovery: RegistrationDiagnosticRecovery::Degraded,
        });
        self.rejected.push(RejectedRegistration {
            id,
            resolver_registration,
            source,
            reason,
        });
    }

    fn finish(mut self) -> RegistrationDatabase {
        self.pending.sort_by_key(pending_order_key);
        self.activated.sort_by_key(activated_order_key);
        self.rejected.sort_by_key(rejected_order_key);
        RegistrationDatabase {
            module_id: self.module_id,
            pending: PendingRegistrationTable {
                entries: self.pending,
            },
            activated: ActivatedRegistrationIndex {
                entries: self.activated,
            },
            rejected: RejectedRegistrationTable {
                entries: self.rejected,
            },
            diagnostics: self.diagnostics,
            initial_obligations: self.initial_obligations,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingRegistration {
    id: CheckerRegistrationId,
    resolver_registration: ResolverRegistrationId,
    source: RegistrationSource,
    pattern_status: RegistrationPatternStatus,
    status: PendingRegistrationStatus,
    parameters: Vec<RegistrationParameterKey>,
    obligations: Vec<InitialObligationId>,
}

impl PendingRegistration {
    pub const fn id(&self) -> CheckerRegistrationId {
        self.id
    }

    pub const fn resolver_registration(&self) -> ResolverRegistrationId {
        self.resolver_registration
    }

    pub const fn source(&self) -> &RegistrationSource {
        &self.source
    }

    pub const fn pattern_status(&self) -> RegistrationPatternStatus {
        self.pattern_status
    }

    pub const fn status(&self) -> PendingRegistrationStatus {
        self.status
    }

    pub fn parameters(&self) -> &[RegistrationParameterKey] {
        &self.parameters
    }

    pub fn obligations(&self) -> &[InitialObligationId] {
        &self.obligations
    }

    pub const fn may_contribute_to_inference(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PendingRegistrationTable {
    entries: Vec<PendingRegistration>,
}

impl PendingRegistrationTable {
    pub fn iter(&self) -> impl Iterator<Item = &PendingRegistration> {
        self.entries.iter()
    }

    pub fn get(&self, id: CheckerRegistrationId) -> Option<&PendingRegistration> {
        self.entries.iter().find(|entry| entry.id() == id)
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
pub enum RegistrationPatternStatus {
    ExternalDependencyGap,
    Validated(RegistrationValidationKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum PendingRegistrationStatus {
    BlockedExternalDependency,
    AwaitingVerifierAcceptance,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivatedRegistration {
    id: CheckerRegistrationId,
    resolver_registration: ResolverRegistrationId,
    source: RegistrationSource,
    trigger: RegistrationTriggerKey,
    label: Option<RegistrationLabelKey>,
    kind: ResolverRegistrationKind,
    validation_kind: Option<RegistrationValidationKind>,
    pattern: RegistrationPatternKey,
    parameters: Vec<RegistrationParameterKey>,
    correctness: AcceptedCorrectnessKey,
    evidence: ActivationEvidenceKey,
    fingerprint: Option<RegistrationFingerprint>,
}

impl ActivatedRegistration {
    pub const fn id(&self) -> CheckerRegistrationId {
        self.id
    }

    pub const fn resolver_registration(&self) -> ResolverRegistrationId {
        self.resolver_registration
    }

    pub const fn source(&self) -> &RegistrationSource {
        &self.source
    }

    pub const fn trigger(&self) -> &RegistrationTriggerKey {
        &self.trigger
    }

    pub const fn label(&self) -> Option<&RegistrationLabelKey> {
        self.label.as_ref()
    }

    pub const fn kind(&self) -> ResolverRegistrationKind {
        self.kind
    }

    pub const fn validation_kind(&self) -> Option<RegistrationValidationKind> {
        self.validation_kind
    }

    pub const fn pattern(&self) -> &RegistrationPatternKey {
        &self.pattern
    }

    pub fn parameters(&self) -> &[RegistrationParameterKey] {
        &self.parameters
    }

    pub const fn correctness(&self) -> &AcceptedCorrectnessKey {
        &self.correctness
    }

    pub const fn evidence(&self) -> &ActivationEvidenceKey {
        &self.evidence
    }

    pub const fn fingerprint(&self) -> Option<&RegistrationFingerprint> {
        self.fingerprint.as_ref()
    }

    pub const fn may_contribute_to_inference(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivatedRegistrationIndex {
    entries: Vec<ActivatedRegistration>,
}

impl ActivatedRegistrationIndex {
    pub fn iter(&self) -> impl Iterator<Item = &ActivatedRegistration> {
        self.entries.iter()
    }

    pub fn by_trigger(&self, trigger: &RegistrationTriggerKey) -> Vec<&ActivatedRegistration> {
        self.entries
            .iter()
            .filter(|entry| entry.trigger() == trigger)
            .collect()
    }

    pub fn get(&self, id: CheckerRegistrationId) -> Option<&ActivatedRegistration> {
        self.entries.iter().find(|entry| entry.id() == id)
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedRegistration {
    id: RejectedRegistrationId,
    resolver_registration: Option<ResolverRegistrationId>,
    source: Option<RegistrationSource>,
    reason: RejectedRegistrationReason,
}

impl RejectedRegistration {
    pub const fn id(&self) -> RejectedRegistrationId {
        self.id
    }

    pub const fn resolver_registration(&self) -> Option<ResolverRegistrationId> {
        self.resolver_registration
    }

    pub const fn source(&self) -> Option<&RegistrationSource> {
        self.source.as_ref()
    }

    pub const fn reason(&self) -> RejectedRegistrationReason {
        self.reason
    }

    pub const fn may_contribute_to_inference(&self) -> bool {
        false
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RejectedRegistrationTable {
    entries: Vec<RejectedRegistration>,
}

impl RejectedRegistrationTable {
    pub fn iter(&self) -> impl Iterator<Item = &RejectedRegistration> {
        self.entries.iter()
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
pub enum RejectedRegistrationReason {
    MalformedResolverTarget,
    RecoveredResolverOrigin,
    UnknownValidationOrigin,
    UnknownActivationOrigin,
    ValidationKindMismatch,
    ActivationKindMismatch,
    MissingRegistrationLabel,
    MissingRegistrationPayload,
    MalformedRegistrationPattern,
    MissingReferencedSymbol,
    IncompatibleReferencedSymbol,
    InvalidRegistrationParameter,
    MissingCorrectnessCondition,
    MissingSourceProvenance,
    InvalidReductionOrientation,
    MissingActivationTrigger,
    MissingAcceptedPattern,
    MissingAcceptedCorrectness,
    MissingActivationEvidence,
    UnacceptedActivationEvidence,
    DuplicateValidationInput,
    DuplicateActivationInput,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationSource {
    resolver_registration: ResolverRegistrationId,
    symbol: Option<SymbolId>,
    kind: ResolverRegistrationKind,
    target: ResolverTargetShell,
    visibility: Visibility,
    export_status: ExportStatus,
    origin: SemanticOrigin,
    contribution: SourceContributionId,
    dependencies: Vec<DeclarationDependencyId>,
    recovery: RecoveryState,
}

impl RegistrationSource {
    fn from_entry(entry: &mizar_resolve::env::RegistrationEntry) -> Self {
        Self {
            resolver_registration: entry.id(),
            symbol: entry.symbol().cloned(),
            kind: entry.kind(),
            target: ResolverTargetShell::from(entry.target()),
            visibility: entry.visibility(),
            export_status: entry.export_status(),
            origin: entry.origin().clone(),
            contribution: entry.contribution(),
            dependencies: entry.dependencies().to_vec(),
            recovery: entry.recovery(),
        }
    }

    pub const fn resolver_registration(&self) -> ResolverRegistrationId {
        self.resolver_registration
    }

    pub const fn symbol(&self) -> Option<&SymbolId> {
        self.symbol.as_ref()
    }

    pub const fn kind(&self) -> ResolverRegistrationKind {
        self.kind
    }

    pub const fn target(&self) -> &ResolverTargetShell {
        &self.target
    }

    pub const fn visibility(&self) -> Visibility {
        self.visibility
    }

    pub const fn export_status(&self) -> ExportStatus {
        self.export_status
    }

    pub const fn origin(&self) -> &SemanticOrigin {
        &self.origin
    }

    pub const fn contribution(&self) -> SourceContributionId {
        self.contribution
    }

    pub fn dependencies(&self) -> &[DeclarationDependencyId] {
        &self.dependencies
    }

    pub const fn recovery(&self) -> RecoveryState {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ResolverTargetShell {
    Pending,
    Opaque { schema: String, payload: String },
    Malformed { class: String },
    Unsupported { class: String },
}

impl From<&SignatureShell> for ResolverTargetShell {
    fn from(value: &SignatureShell) -> Self {
        match value {
            SignatureShell::Pending => Self::Pending,
            SignatureShell::Opaque { schema, payload } => Self::Opaque {
                schema: schema.clone(),
                payload: payload.clone(),
            },
            SignatureShell::Malformed { class } => Self::Malformed {
                class: class.clone(),
            },
            _ => Self::Unsupported {
                class: "unsupported-signature-shell".to_owned(),
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RegistrationValidationKind {
    Existential,
    Conditional,
    Functorial,
    Reduction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationValidationInput {
    resolver_registration: ResolverRegistrationId,
    owner: TypedSiteRef,
    source_range: SourceRange,
    pattern: RegistrationValidationPattern,
    parameters: Vec<RegistrationValidationParameter>,
    referenced_symbols: Vec<RegistrationReferencedSymbol>,
    assumptions: Vec<TypeFactId>,
    correctness_goal: InitialObligationGoal,
    correctness_provenance: InitialObligationProvenance,
}

impl RegistrationValidationInput {
    pub fn new(
        resolver_registration: ResolverRegistrationId,
        owner: TypedSiteRef,
        source_range: SourceRange,
        pattern: RegistrationValidationPattern,
        correctness_goal: impl Into<InitialObligationGoal>,
        correctness_provenance: impl Into<InitialObligationProvenance>,
    ) -> Self {
        Self {
            resolver_registration,
            owner,
            source_range,
            pattern,
            parameters: Vec::new(),
            referenced_symbols: Vec::new(),
            assumptions: Vec::new(),
            correctness_goal: correctness_goal.into(),
            correctness_provenance: correctness_provenance.into(),
        }
    }

    pub const fn resolver_registration(&self) -> ResolverRegistrationId {
        self.resolver_registration
    }

    pub const fn owner(&self) -> &TypedSiteRef {
        &self.owner
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn pattern(&self) -> &RegistrationValidationPattern {
        &self.pattern
    }

    pub fn parameters(&self) -> &[RegistrationValidationParameter] {
        &self.parameters
    }

    pub fn referenced_symbols(&self) -> &[RegistrationReferencedSymbol] {
        &self.referenced_symbols
    }

    pub fn assumptions(&self) -> &[TypeFactId] {
        &self.assumptions
    }

    pub const fn correctness_goal(&self) -> &InitialObligationGoal {
        &self.correctness_goal
    }

    pub const fn correctness_provenance(&self) -> &InitialObligationProvenance {
        &self.correctness_provenance
    }

    pub fn with_parameters(
        mut self,
        parameters: impl IntoIterator<Item = RegistrationValidationParameter>,
    ) -> Self {
        self.parameters = parameters.into_iter().collect();
        self
    }

    pub fn with_referenced_symbols(
        mut self,
        referenced_symbols: impl IntoIterator<Item = RegistrationReferencedSymbol>,
    ) -> Self {
        self.referenced_symbols = referenced_symbols.into_iter().collect();
        self
    }

    pub fn with_assumptions(mut self, assumptions: impl IntoIterator<Item = TypeFactId>) -> Self {
        self.assumptions = assumptions.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum RegistrationValidationPattern {
    Existential {
        type_head: RegistrationTypeKey,
        attributes: Vec<RegistrationAttributeKey>,
    },
    Conditional {
        type_head: RegistrationTypeKey,
        antecedent: Vec<RegistrationAttributeKey>,
        consequent: Vec<RegistrationAttributeKey>,
    },
    Functorial {
        functor: RegistrationFunctorKey,
        result_type: RegistrationTypeKey,
        consequent: Vec<RegistrationAttributeKey>,
    },
    Reduction {
        lhs: RegistrationTermPattern,
        rhs: RegistrationTermPattern,
    },
}

impl RegistrationValidationPattern {
    pub const fn kind(&self) -> RegistrationValidationKind {
        match self {
            Self::Existential { .. } => RegistrationValidationKind::Existential,
            Self::Conditional { .. } => RegistrationValidationKind::Conditional,
            Self::Functorial { .. } => RegistrationValidationKind::Functorial,
            Self::Reduction { .. } => RegistrationValidationKind::Reduction,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationTermPattern {
    fingerprint: RegistrationTermKey,
    size: usize,
    free_variables: Vec<RegistrationVariableOccurrence>,
    source_range: Option<SourceRange>,
}

impl RegistrationTermPattern {
    pub fn new(
        fingerprint: impl Into<RegistrationTermKey>,
        size: usize,
        free_variables: impl IntoIterator<Item = RegistrationVariableOccurrence>,
        source_range: SourceRange,
    ) -> Self {
        Self {
            fingerprint: fingerprint.into(),
            size,
            free_variables: free_variables.into_iter().collect(),
            source_range: Some(source_range),
        }
    }

    pub fn without_source_range(
        fingerprint: impl Into<RegistrationTermKey>,
        size: usize,
        free_variables: impl IntoIterator<Item = RegistrationVariableOccurrence>,
    ) -> Self {
        Self {
            fingerprint: fingerprint.into(),
            size,
            free_variables: free_variables.into_iter().collect(),
            source_range: None,
        }
    }

    pub const fn fingerprint(&self) -> &RegistrationTermKey {
        &self.fingerprint
    }

    pub const fn size(&self) -> usize {
        self.size
    }

    pub fn free_variables(&self) -> &[RegistrationVariableOccurrence] {
        &self.free_variables
    }

    pub const fn source_range(&self) -> Option<SourceRange> {
        self.source_range
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationVariableOccurrence {
    variable: RegistrationVariableKey,
    count: usize,
}

impl RegistrationVariableOccurrence {
    pub fn new(variable: impl Into<RegistrationVariableKey>, count: usize) -> Self {
        Self {
            variable: variable.into(),
            count,
        }
    }

    pub const fn variable(&self) -> &RegistrationVariableKey {
        &self.variable
    }

    pub const fn count(&self) -> usize {
        self.count
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationValidationParameter {
    key: RegistrationParameterKey,
    typed: bool,
    visible_facts: bool,
}

impl RegistrationValidationParameter {
    pub fn new(key: impl Into<RegistrationParameterKey>) -> Self {
        Self {
            key: key.into(),
            typed: true,
            visible_facts: true,
        }
    }

    pub fn with_typed(mut self, typed: bool) -> Self {
        self.typed = typed;
        self
    }

    pub fn with_visible_facts(mut self, visible_facts: bool) -> Self {
        self.visible_facts = visible_facts;
        self
    }

    pub const fn key(&self) -> &RegistrationParameterKey {
        &self.key
    }

    pub const fn is_typed(&self) -> bool {
        self.typed
    }

    pub const fn facts_are_visible(&self) -> bool {
        self.visible_facts
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RegistrationReferencedSymbolRole {
    Attribute,
    Mode,
    Structure,
    Functor,
    Term,
    TypeHead,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationReferencedSymbol {
    role: RegistrationReferencedSymbolRole,
    symbol: Option<SymbolId>,
    compatible: bool,
}

impl RegistrationReferencedSymbol {
    pub fn compatible(role: RegistrationReferencedSymbolRole, symbol: SymbolId) -> Self {
        Self {
            role,
            symbol: Some(symbol),
            compatible: true,
        }
    }

    pub const fn missing(role: RegistrationReferencedSymbolRole) -> Self {
        Self {
            role,
            symbol: None,
            compatible: false,
        }
    }

    pub fn incompatible(role: RegistrationReferencedSymbolRole, symbol: SymbolId) -> Self {
        Self {
            role,
            symbol: Some(symbol),
            compatible: false,
        }
    }

    pub const fn role(&self) -> RegistrationReferencedSymbolRole {
        self.role
    }

    pub const fn symbol(&self) -> Option<&SymbolId> {
        self.symbol.as_ref()
    }

    pub const fn is_compatible(&self) -> bool {
        self.compatible
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActivationInput {
    resolver_registration: ResolverRegistrationId,
    kind: ResolverRegistrationKind,
    trigger: RegistrationTriggerKey,
    label: Option<RegistrationLabelKey>,
    pattern: RegistrationPatternKey,
    validation_kind: Option<RegistrationValidationKind>,
    parameters: Vec<RegistrationParameterKey>,
    correctness: AcceptedCorrectnessKey,
    evidence: ActivationEvidenceKey,
    verifier_status: ActivationVerifierStatus,
    fingerprint: Option<RegistrationFingerprint>,
}

impl ActivationInput {
    pub fn new(
        resolver_registration: ResolverRegistrationId,
        kind: ResolverRegistrationKind,
        trigger: impl Into<RegistrationTriggerKey>,
        pattern: impl Into<RegistrationPatternKey>,
        correctness: impl Into<AcceptedCorrectnessKey>,
        evidence: impl Into<ActivationEvidenceKey>,
    ) -> Self {
        Self {
            resolver_registration,
            kind,
            trigger: trigger.into(),
            label: None,
            pattern: pattern.into(),
            validation_kind: None,
            parameters: Vec::new(),
            correctness: correctness.into(),
            evidence: evidence.into(),
            verifier_status: ActivationVerifierStatus::Missing,
            fingerprint: None,
        }
    }

    pub fn accepted(
        resolver_registration: ResolverRegistrationId,
        kind: ResolverRegistrationKind,
        trigger: impl Into<RegistrationTriggerKey>,
        pattern: impl Into<RegistrationPatternKey>,
        correctness: impl Into<AcceptedCorrectnessKey>,
        evidence: impl Into<ActivationEvidenceKey>,
    ) -> Self {
        Self::new(
            resolver_registration,
            kind,
            trigger,
            pattern,
            correctness,
            evidence,
        )
        .with_verifier_status(ActivationVerifierStatus::Accepted)
    }

    pub fn with_label(mut self, label: impl Into<RegistrationLabelKey>) -> Self {
        self.label = Some(label.into());
        self
    }

    pub fn with_parameters(
        mut self,
        parameters: impl IntoIterator<Item = RegistrationParameterKey>,
    ) -> Self {
        self.parameters = parameters.into_iter().collect();
        self
    }

    pub const fn with_validation_kind(mut self, kind: RegistrationValidationKind) -> Self {
        self.validation_kind = Some(kind);
        self
    }

    pub fn with_fingerprint(mut self, fingerprint: impl Into<RegistrationFingerprint>) -> Self {
        self.fingerprint = Some(fingerprint.into());
        self
    }

    pub const fn with_verifier_status(mut self, status: ActivationVerifierStatus) -> Self {
        self.verifier_status = status;
        self
    }

    pub const fn verifier_status(&self) -> ActivationVerifierStatus {
        self.verifier_status
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ActivationVerifierStatus {
    Accepted,
    Missing,
    Rejected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistentialGateInput {
    owner: TypedSiteRef,
    source_range: SourceRange,
    pattern: RegistrationPatternKey,
    trigger: RegistrationTriggerKey,
    attributes: Vec<RegistrationAttributeKey>,
    required_guards: Vec<ExistentialGateGuardKey>,
    guard_evidence: Vec<ExistentialGateGuardEvidence>,
    candidates: Vec<ExistentialGateCandidate>,
    base_evidence: Option<ExistentialGateBaseEvidence>,
    recovery: ExistentialGateRecovery,
}

impl ExistentialGateInput {
    pub fn new(
        owner: TypedSiteRef,
        source_range: SourceRange,
        pattern: impl Into<RegistrationPatternKey>,
        trigger: impl Into<RegistrationTriggerKey>,
        attributes: impl IntoIterator<Item = RegistrationAttributeKey>,
    ) -> Self {
        Self {
            owner,
            source_range,
            pattern: pattern.into(),
            trigger: trigger.into(),
            attributes: attributes.into_iter().collect(),
            required_guards: Vec::new(),
            guard_evidence: Vec::new(),
            candidates: Vec::new(),
            base_evidence: None,
            recovery: ExistentialGateRecovery::Normal,
        }
    }

    pub const fn owner(&self) -> &TypedSiteRef {
        &self.owner
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn pattern(&self) -> &RegistrationPatternKey {
        &self.pattern
    }

    pub const fn trigger(&self) -> &RegistrationTriggerKey {
        &self.trigger
    }

    pub fn attributes(&self) -> &[RegistrationAttributeKey] {
        &self.attributes
    }

    pub fn required_guards(&self) -> &[ExistentialGateGuardKey] {
        &self.required_guards
    }

    pub fn guard_evidence(&self) -> &[ExistentialGateGuardEvidence] {
        &self.guard_evidence
    }

    pub fn candidates(&self) -> &[ExistentialGateCandidate] {
        &self.candidates
    }

    pub const fn base_evidence(&self) -> Option<&ExistentialGateBaseEvidence> {
        self.base_evidence.as_ref()
    }

    pub const fn recovery(&self) -> ExistentialGateRecovery {
        self.recovery
    }

    pub fn with_required_guards(
        mut self,
        guards: impl IntoIterator<Item = ExistentialGateGuardKey>,
    ) -> Self {
        self.required_guards = guards.into_iter().collect();
        self
    }

    pub fn with_guard_evidence(
        mut self,
        evidence: impl IntoIterator<Item = ExistentialGateGuardEvidence>,
    ) -> Self {
        self.guard_evidence = evidence.into_iter().collect();
        self
    }

    pub fn with_candidates(
        mut self,
        candidates: impl IntoIterator<Item = ExistentialGateCandidate>,
    ) -> Self {
        self.candidates = candidates.into_iter().collect();
        self
    }

    pub fn with_base_evidence(mut self, evidence: ExistentialGateBaseEvidence) -> Self {
        self.base_evidence = Some(evidence);
        self
    }

    pub const fn with_recovery(mut self, recovery: ExistentialGateRecovery) -> Self {
        self.recovery = recovery;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistentialGateCandidate {
    registration: CheckerRegistrationId,
    accepted_pattern: RegistrationPatternKey,
    accepted_correctness: AcceptedCorrectnessKey,
    activation_evidence: ActivationEvidenceKey,
    trigger: RegistrationTriggerKey,
    fingerprint: Option<RegistrationFingerprint>,
    kind: RegistrationValidationKind,
    attributes: Vec<RegistrationAttributeKey>,
}

impl ExistentialGateCandidate {
    pub fn new(
        registration: CheckerRegistrationId,
        accepted_pattern: impl Into<RegistrationPatternKey>,
        accepted_correctness: impl Into<AcceptedCorrectnessKey>,
        activation_evidence: impl Into<ActivationEvidenceKey>,
        trigger: impl Into<RegistrationTriggerKey>,
        attributes: impl IntoIterator<Item = RegistrationAttributeKey>,
    ) -> Self {
        Self {
            registration,
            accepted_pattern: accepted_pattern.into(),
            accepted_correctness: accepted_correctness.into(),
            activation_evidence: activation_evidence.into(),
            trigger: trigger.into(),
            fingerprint: None,
            kind: RegistrationValidationKind::Existential,
            attributes: attributes.into_iter().collect(),
        }
    }

    pub const fn registration(&self) -> CheckerRegistrationId {
        self.registration
    }

    pub const fn accepted_pattern(&self) -> &RegistrationPatternKey {
        &self.accepted_pattern
    }

    pub const fn accepted_correctness(&self) -> &AcceptedCorrectnessKey {
        &self.accepted_correctness
    }

    pub const fn activation_evidence(&self) -> &ActivationEvidenceKey {
        &self.activation_evidence
    }

    pub const fn trigger(&self) -> &RegistrationTriggerKey {
        &self.trigger
    }

    pub const fn fingerprint(&self) -> Option<&RegistrationFingerprint> {
        self.fingerprint.as_ref()
    }

    pub const fn kind(&self) -> RegistrationValidationKind {
        self.kind
    }

    pub fn attributes(&self) -> &[RegistrationAttributeKey] {
        &self.attributes
    }

    pub fn with_fingerprint(mut self, fingerprint: impl Into<RegistrationFingerprint>) -> Self {
        self.fingerprint = Some(fingerprint.into());
        self
    }

    pub const fn with_kind(mut self, kind: RegistrationValidationKind) -> Self {
        self.kind = kind;
        self
    }

    pub fn with_correctness(mut self, correctness: impl Into<AcceptedCorrectnessKey>) -> Self {
        self.accepted_correctness = correctness.into();
        self
    }

    pub fn with_activation_evidence(mut self, evidence: impl Into<ActivationEvidenceKey>) -> Self {
        self.activation_evidence = evidence.into();
        self
    }

    pub fn with_trigger(mut self, trigger: impl Into<RegistrationTriggerKey>) -> Self {
        self.trigger = trigger.into();
        self
    }

    pub fn with_attributes(
        mut self,
        attributes: impl IntoIterator<Item = RegistrationAttributeKey>,
    ) -> Self {
        self.attributes = attributes.into_iter().collect();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistentialGateBaseEvidence {
    kind: ExistentialGateBaseEvidenceKind,
    pattern: RegistrationPatternKey,
    coverage: ExistentialGateBaseEvidenceCoverage,
}

impl ExistentialGateBaseEvidence {
    pub fn new(
        kind: ExistentialGateBaseEvidenceKind,
        pattern: impl Into<RegistrationPatternKey>,
        coverage: ExistentialGateBaseEvidenceCoverage,
    ) -> Self {
        Self {
            kind,
            pattern: pattern.into(),
            coverage,
        }
    }

    pub const fn kind(&self) -> ExistentialGateBaseEvidenceKind {
        self.kind
    }

    pub const fn pattern(&self) -> &RegistrationPatternKey {
        &self.pattern
    }

    pub const fn coverage(&self) -> ExistentialGateBaseEvidenceCoverage {
        self.coverage
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExistentialGateBaseEvidenceKind {
    BuiltinObject,
    BuiltinSet,
    AcceptedModeApplication,
    StructureConstructorWitness,
    SchemaTypeParameter,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExistentialGateBaseEvidenceCoverage {
    Builtin,
    CompleteGuardSet,
    ZeroFieldStructure,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistentialGateGuardEvidence {
    guard: ExistentialGateGuardKey,
    fact: TypeFactId,
    visible: bool,
    consumable: bool,
}

impl ExistentialGateGuardEvidence {
    pub fn new(guard: impl Into<ExistentialGateGuardKey>, fact: TypeFactId) -> Self {
        Self {
            guard: guard.into(),
            fact,
            visible: true,
            consumable: true,
        }
    }

    pub const fn guard(&self) -> &ExistentialGateGuardKey {
        &self.guard
    }

    pub const fn fact(&self) -> TypeFactId {
        self.fact
    }

    pub const fn is_visible(&self) -> bool {
        self.visible
    }

    pub const fn is_consumable(&self) -> bool {
        self.consumable
    }

    pub const fn with_visible(mut self, visible: bool) -> Self {
        self.visible = visible;
        self
    }

    pub const fn with_consumable(mut self, consumable: bool) -> Self {
        self.consumable = consumable;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExistentialGateRecovery {
    Normal,
    Degraded,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistentialGateOutput {
    entries: Vec<ExistentialGateResult>,
    diagnostics: RegistrationDiagnosticTable,
}

impl ExistentialGateOutput {
    pub fn evaluate(
        database: &RegistrationDatabase,
        inputs: impl IntoIterator<Item = ExistentialGateInput>,
    ) -> Self {
        let mut builder = ExistentialGateBuilder::new(database);
        let mut inputs = inputs.into_iter().collect::<Vec<_>>();
        inputs.sort_by_key(existential_gate_input_order_key);
        for input in inputs {
            builder.evaluate(input);
        }
        builder.finish()
    }

    pub fn iter(&self) -> impl Iterator<Item = &ExistentialGateResult> {
        self.entries.iter()
    }

    pub const fn diagnostics(&self) -> &RegistrationDiagnosticTable {
        &self.diagnostics
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("existential-gate-debug-v1\n");
        write_existential_gates(&mut output, &self.entries);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExistentialGateResult {
    id: ExistentialGateId,
    owner: TypedSiteRef,
    source_range: SourceRange,
    pattern: RegistrationPatternKey,
    trigger: RegistrationTriggerKey,
    attributes: Vec<RegistrationAttributeKey>,
    status: ExistentialGateStatus,
    registration: Option<CheckerRegistrationId>,
    base_evidence_kind: Option<ExistentialGateBaseEvidenceKind>,
    base_evidence_coverage: Option<ExistentialGateBaseEvidenceCoverage>,
    facts: Vec<TypeFactId>,
    diagnostics: Vec<RegistrationDiagnosticId>,
}

impl ExistentialGateResult {
    pub const fn id(&self) -> ExistentialGateId {
        self.id
    }

    pub const fn owner(&self) -> &TypedSiteRef {
        &self.owner
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn pattern(&self) -> &RegistrationPatternKey {
        &self.pattern
    }

    pub const fn trigger(&self) -> &RegistrationTriggerKey {
        &self.trigger
    }

    pub fn attributes(&self) -> &[RegistrationAttributeKey] {
        &self.attributes
    }

    pub const fn status(&self) -> ExistentialGateStatus {
        self.status
    }

    pub const fn registration(&self) -> Option<CheckerRegistrationId> {
        self.registration
    }

    pub const fn base_evidence_kind(&self) -> Option<ExistentialGateBaseEvidenceKind> {
        self.base_evidence_kind
    }

    pub const fn base_evidence_coverage(&self) -> Option<ExistentialGateBaseEvidenceCoverage> {
        self.base_evidence_coverage
    }

    pub fn facts(&self) -> &[TypeFactId] {
        &self.facts
    }

    pub fn diagnostics(&self) -> &[RegistrationDiagnosticId] {
        &self.diagnostics
    }

    pub const fn may_seed_verified_facts(&self) -> bool {
        matches!(self.status, ExistentialGateStatus::Satisfied)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExistentialGateStatus {
    Satisfied,
    MissingExistential,
    BlockedGuard,
    InvalidCandidate,
    DegradedRecovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationDiagnostic {
    id: RegistrationDiagnosticId,
    resolver_registration: Option<ResolverRegistrationId>,
    class: RegistrationDiagnosticClass,
    severity: RegistrationDiagnosticSeverity,
    message_key: String,
    recovery: RegistrationDiagnosticRecovery,
}

impl RegistrationDiagnostic {
    pub const fn id(&self) -> RegistrationDiagnosticId {
        self.id
    }

    pub const fn resolver_registration(&self) -> Option<ResolverRegistrationId> {
        self.resolver_registration
    }

    pub const fn class(&self) -> RegistrationDiagnosticClass {
        self.class
    }

    pub const fn severity(&self) -> RegistrationDiagnosticSeverity {
        self.severity
    }

    pub fn message_key(&self) -> &str {
        &self.message_key
    }

    pub const fn recovery(&self) -> RegistrationDiagnosticRecovery {
        self.recovery
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RegistrationDiagnosticDraft {
    pub resolver_registration: Option<ResolverRegistrationId>,
    pub class: RegistrationDiagnosticClass,
    pub severity: RegistrationDiagnosticSeverity,
    pub message_key: String,
    pub recovery: RegistrationDiagnosticRecovery,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct RegistrationDiagnosticTable {
    entries: Vec<RegistrationDiagnostic>,
}

impl RegistrationDiagnosticTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn insert(&mut self, draft: RegistrationDiagnosticDraft) -> RegistrationDiagnosticId {
        let id = RegistrationDiagnosticId::new(self.entries.len());
        self.entries.push(RegistrationDiagnostic {
            id,
            resolver_registration: draft.resolver_registration,
            class: draft.class,
            severity: draft.severity,
            message_key: draft.message_key,
            recovery: draft.recovery,
        });
        id
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (RegistrationDiagnosticId, &RegistrationDiagnostic)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(
        &self,
    ) -> impl Iterator<Item = (RegistrationDiagnosticId, &RegistrationDiagnostic)> {
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
pub enum RegistrationDiagnosticClass {
    ExternalDependencyGap,
    MalformedResolverTarget,
    InvalidValidation,
    MissingReferencedSymbol,
    IncompatibleReferencedSymbol,
    InvalidRegistrationParameter,
    MissingCorrectnessCondition,
    MissingSourceProvenance,
    InvalidReductionOrientation,
    InvalidActivation,
    UnacceptedActivationEvidence,
    UnavailableExistentialRegistration,
    BlockedExistentialGuard,
    InvalidExistentialGateCandidate,
    Recovery,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RegistrationDiagnosticSeverity {
    Error,
    Warning,
    Note,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum RegistrationDiagnosticRecovery {
    Normal,
    Degraded,
}

type ActivationValidationError = (
    Box<RegistrationSource>,
    RejectedRegistrationReason,
    &'static str,
    RegistrationDiagnosticClass,
);

type PendingValidationError = (
    Box<RegistrationSource>,
    RejectedRegistrationReason,
    &'static str,
    RegistrationDiagnosticClass,
);
type SiteOrderKey = (usize, u8, String);
type ExistentialGateBaseEvidenceOrderKey = (u8, u8, String);
type ExistentialGateInputOrderKey = (
    String,
    usize,
    usize,
    SiteOrderKey,
    String,
    String,
    Vec<String>,
    Option<ExistentialGateBaseEvidenceOrderKey>,
);
type ExistentialGateResultOrderKey = (
    String,
    usize,
    usize,
    SiteOrderKey,
    String,
    String,
    Vec<String>,
    Option<(u8, u8)>,
    usize,
);

struct ExistentialGateBuilder<'a> {
    database: &'a RegistrationDatabase,
    entries: Vec<ExistentialGateResult>,
    diagnostics: RegistrationDiagnosticTable,
}

impl<'a> ExistentialGateBuilder<'a> {
    fn new(database: &'a RegistrationDatabase) -> Self {
        Self {
            database,
            entries: Vec::new(),
            diagnostics: RegistrationDiagnosticTable::new(),
        }
    }

    fn evaluate(&mut self, input: ExistentialGateInput) {
        let mut candidate_reports = input
            .candidates()
            .iter()
            .map(|candidate| evaluate_existential_candidate(self.database, &input, candidate))
            .collect::<Vec<_>>();
        candidate_reports.sort_by_key(existential_candidate_report_order_key);
        let base_report = input
            .base_evidence()
            .map(|evidence| evaluate_existential_base_evidence(&input, evidence));

        let status = existential_gate_status(&input, &candidate_reports, base_report.as_ref());
        let mut diagnostics = Vec::new();
        if status != ExistentialGateStatus::Satisfied {
            diagnostics.push(
                self.diagnostics.insert(RegistrationDiagnosticDraft {
                    resolver_registration: candidate_reports
                        .iter()
                        .find_map(|report| report.resolver_registration),
                    class: existential_gate_diagnostic_class(status),
                    severity: RegistrationDiagnosticSeverity::Error,
                    message_key: existential_gate_message_key(status).to_owned(),
                    recovery: RegistrationDiagnosticRecovery::Degraded,
                }),
            );
        }

        let registration = (status == ExistentialGateStatus::Satisfied)
            .then(|| {
                candidate_reports
                    .iter()
                    .find(|report| report.status == ExistentialCandidateStatus::Satisfied)
                    .map(|report| report.registration)
            })
            .flatten();
        let satisfied_base = if status == ExistentialGateStatus::Satisfied && registration.is_none()
        {
            base_report
                .as_ref()
                .filter(|report| report.status == ExistentialBaseEvidenceStatus::Satisfied)
        } else {
            None
        };
        let base_evidence_kind = satisfied_base.map(|report| report.kind);
        let base_evidence_coverage = satisfied_base.map(|report| report.coverage);
        let facts = if status == ExistentialGateStatus::Satisfied {
            visible_guard_facts(input.required_guards(), input.guard_evidence())
        } else {
            Vec::new()
        };
        let id = ExistentialGateId::new(self.entries.len());
        self.entries.push(ExistentialGateResult {
            id,
            owner: input.owner,
            source_range: input.source_range,
            pattern: input.pattern,
            trigger: input.trigger,
            attributes: canonical_attributes(input.attributes),
            status,
            registration,
            base_evidence_kind,
            base_evidence_coverage,
            facts,
            diagnostics,
        });
    }

    fn finish(mut self) -> ExistentialGateOutput {
        self.entries.sort_by_key(existential_gate_result_order_key);
        for (index, entry) in self.entries.iter_mut().enumerate() {
            entry.id = ExistentialGateId::new(index);
        }
        ExistentialGateOutput {
            entries: self.entries,
            diagnostics: self.diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExistentialCandidateReport {
    registration: CheckerRegistrationId,
    resolver_registration: Option<ResolverRegistrationId>,
    status: ExistentialCandidateStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ExistentialCandidateStatus {
    Satisfied,
    BlockedGuard,
    Invalid,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct ExistentialBaseEvidenceReport {
    kind: ExistentialGateBaseEvidenceKind,
    coverage: ExistentialGateBaseEvidenceCoverage,
    status: ExistentialBaseEvidenceStatus,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum ExistentialBaseEvidenceStatus {
    Satisfied,
    BlockedGuard,
    Invalid,
}

fn evaluate_existential_candidate(
    database: &RegistrationDatabase,
    input: &ExistentialGateInput,
    candidate: &ExistentialGateCandidate,
) -> ExistentialCandidateReport {
    let active = database.activated().get(candidate.registration());
    let Some(active) = active else {
        return ExistentialCandidateReport {
            registration: candidate.registration(),
            resolver_registration: None,
            status: ExistentialCandidateStatus::Invalid,
        };
    };
    let resolver_registration = Some(active.resolver_registration());
    if active.kind() != ResolverRegistrationKind::Cluster
        || candidate.kind() != RegistrationValidationKind::Existential
        || active.validation_kind() != Some(RegistrationValidationKind::Existential)
        || active.validation_kind() != Some(candidate.kind())
        || active.pattern() != candidate.accepted_pattern()
        || active.correctness() != candidate.accepted_correctness()
        || active.evidence() != candidate.activation_evidence()
        || active.trigger() != candidate.trigger()
        || input.trigger() != candidate.trigger()
        || !fingerprint_matches(active, candidate)
        || active.pattern() != input.pattern()
        || !candidate_covers_attributes(input.attributes(), candidate.attributes())
    {
        return ExistentialCandidateReport {
            registration: candidate.registration(),
            resolver_registration,
            status: ExistentialCandidateStatus::Invalid,
        };
    }
    if !guards_are_visible(input.required_guards(), input.guard_evidence()) {
        return ExistentialCandidateReport {
            registration: candidate.registration(),
            resolver_registration,
            status: ExistentialCandidateStatus::BlockedGuard,
        };
    }
    ExistentialCandidateReport {
        registration: candidate.registration(),
        resolver_registration,
        status: ExistentialCandidateStatus::Satisfied,
    }
}

fn evaluate_existential_base_evidence(
    input: &ExistentialGateInput,
    evidence: &ExistentialGateBaseEvidence,
) -> ExistentialBaseEvidenceReport {
    let status = if !input.attributes().is_empty()
        || evidence.pattern() != input.pattern()
        || !base_evidence_coverage_is_valid(
            evidence.kind(),
            evidence.coverage(),
            input.required_guards().is_empty(),
        ) {
        ExistentialBaseEvidenceStatus::Invalid
    } else if !guards_are_visible(input.required_guards(), input.guard_evidence()) {
        ExistentialBaseEvidenceStatus::BlockedGuard
    } else {
        ExistentialBaseEvidenceStatus::Satisfied
    };
    ExistentialBaseEvidenceReport {
        kind: evidence.kind(),
        coverage: evidence.coverage(),
        status,
    }
}

fn base_evidence_coverage_is_valid(
    kind: ExistentialGateBaseEvidenceKind,
    coverage: ExistentialGateBaseEvidenceCoverage,
    required_guards_empty: bool,
) -> bool {
    match kind {
        ExistentialGateBaseEvidenceKind::BuiltinObject
        | ExistentialGateBaseEvidenceKind::BuiltinSet => {
            coverage == ExistentialGateBaseEvidenceCoverage::Builtin && required_guards_empty
        }
        ExistentialGateBaseEvidenceKind::AcceptedModeApplication
        | ExistentialGateBaseEvidenceKind::SchemaTypeParameter => {
            coverage == ExistentialGateBaseEvidenceCoverage::CompleteGuardSet
                && !required_guards_empty
        }
        ExistentialGateBaseEvidenceKind::StructureConstructorWitness => match coverage {
            ExistentialGateBaseEvidenceCoverage::ZeroFieldStructure => required_guards_empty,
            ExistentialGateBaseEvidenceCoverage::CompleteGuardSet => !required_guards_empty,
            ExistentialGateBaseEvidenceCoverage::Builtin => false,
        },
    }
}

fn existential_gate_status(
    input: &ExistentialGateInput,
    reports: &[ExistentialCandidateReport],
    base_report: Option<&ExistentialBaseEvidenceReport>,
) -> ExistentialGateStatus {
    if input.recovery() == ExistentialGateRecovery::Degraded {
        return ExistentialGateStatus::DegradedRecovery;
    }
    if reports
        .iter()
        .any(|report| report.status == ExistentialCandidateStatus::Satisfied)
        || base_report
            .is_some_and(|report| report.status == ExistentialBaseEvidenceStatus::Satisfied)
    {
        return ExistentialGateStatus::Satisfied;
    }
    if reports
        .iter()
        .any(|report| report.status == ExistentialCandidateStatus::BlockedGuard)
        || base_report
            .is_some_and(|report| report.status == ExistentialBaseEvidenceStatus::BlockedGuard)
    {
        return ExistentialGateStatus::BlockedGuard;
    }
    if !reports.is_empty() || base_report.is_some() {
        return ExistentialGateStatus::InvalidCandidate;
    }
    ExistentialGateStatus::MissingExistential
}

fn fingerprint_matches(
    active: &ActivatedRegistration,
    candidate: &ExistentialGateCandidate,
) -> bool {
    match (active.fingerprint(), candidate.fingerprint()) {
        (Some(active), Some(candidate)) => active == candidate,
        (Some(_), None) | (None, Some(_)) => false,
        (None, None) => true,
    }
}

fn candidate_covers_attributes(
    requested: &[RegistrationAttributeKey],
    available: &[RegistrationAttributeKey],
) -> bool {
    requested
        .iter()
        .all(|attribute| available.contains(attribute))
}

fn guards_are_visible(
    required: &[ExistentialGateGuardKey],
    evidence: &[ExistentialGateGuardEvidence],
) -> bool {
    required.iter().all(|guard| {
        evidence
            .iter()
            .any(|entry| entry.guard() == guard && entry.is_visible() && entry.is_consumable())
    })
}

fn visible_guard_facts(
    required: &[ExistentialGateGuardKey],
    evidence: &[ExistentialGateGuardEvidence],
) -> Vec<TypeFactId> {
    let mut facts = required
        .iter()
        .filter_map(|guard| {
            evidence
                .iter()
                .filter(|entry| {
                    entry.guard() == guard && entry.is_visible() && entry.is_consumable()
                })
                .map(ExistentialGateGuardEvidence::fact)
                .min()
        })
        .collect::<Vec<_>>();
    facts.sort();
    facts.dedup();
    facts
}

fn canonical_attribute_key_strings(
    attributes: impl IntoIterator<Item = RegistrationAttributeKey>,
) -> Vec<String> {
    canonical_attributes(attributes)
        .into_iter()
        .map(|attribute| attribute.as_str().to_owned())
        .collect()
}

fn base_evidence_order_key(
    evidence: &ExistentialGateBaseEvidence,
) -> ExistentialGateBaseEvidenceOrderKey {
    (
        base_evidence_kind_rank(evidence.kind()),
        base_evidence_coverage_rank(evidence.coverage()),
        evidence.pattern().as_str().to_owned(),
    )
}

fn typed_site_order_key(site: &TypedSiteRef) -> SiteOrderKey {
    match site {
        TypedSiteRef::Node(node) => (node.index(), 0, String::new()),
        TypedSiteRef::Role { node, role } => (node.index(), 1, role.as_str().to_owned()),
    }
}

fn base_evidence_kind_rank(kind: ExistentialGateBaseEvidenceKind) -> u8 {
    match kind {
        ExistentialGateBaseEvidenceKind::BuiltinObject => 0,
        ExistentialGateBaseEvidenceKind::BuiltinSet => 1,
        ExistentialGateBaseEvidenceKind::AcceptedModeApplication => 2,
        ExistentialGateBaseEvidenceKind::StructureConstructorWitness => 3,
        ExistentialGateBaseEvidenceKind::SchemaTypeParameter => 4,
    }
}

fn base_evidence_coverage_rank(coverage: ExistentialGateBaseEvidenceCoverage) -> u8 {
    match coverage {
        ExistentialGateBaseEvidenceCoverage::Builtin => 0,
        ExistentialGateBaseEvidenceCoverage::CompleteGuardSet => 1,
        ExistentialGateBaseEvidenceCoverage::ZeroFieldStructure => 2,
    }
}

fn source_id_order_key(range: SourceRange) -> String {
    format!("{:?}", range.source_id)
}

fn canonical_attributes(
    attributes: impl IntoIterator<Item = RegistrationAttributeKey>,
) -> Vec<RegistrationAttributeKey> {
    let mut attributes = attributes.into_iter().collect::<Vec<_>>();
    attributes.sort();
    attributes.dedup();
    attributes
}

fn existential_gate_diagnostic_class(status: ExistentialGateStatus) -> RegistrationDiagnosticClass {
    match status {
        ExistentialGateStatus::Satisfied => RegistrationDiagnosticClass::ExternalDependencyGap,
        ExistentialGateStatus::MissingExistential => {
            RegistrationDiagnosticClass::UnavailableExistentialRegistration
        }
        ExistentialGateStatus::BlockedGuard => RegistrationDiagnosticClass::BlockedExistentialGuard,
        ExistentialGateStatus::InvalidCandidate => {
            RegistrationDiagnosticClass::InvalidExistentialGateCandidate
        }
        ExistentialGateStatus::DegradedRecovery => RegistrationDiagnosticClass::Recovery,
    }
}

fn existential_gate_message_key(status: ExistentialGateStatus) -> &'static str {
    match status {
        ExistentialGateStatus::Satisfied => "checker.registration.existential_gate_satisfied",
        ExistentialGateStatus::MissingExistential => {
            "checker.registration.unavailable_existential_registration"
        }
        ExistentialGateStatus::BlockedGuard => "checker.registration.blocked_existential_guard",
        ExistentialGateStatus::InvalidCandidate => {
            "checker.registration.invalid_existential_gate_candidate"
        }
        ExistentialGateStatus::DegradedRecovery => "checker.registration.degraded_existential_gate",
    }
}

fn validate_pending_registration(
    resolver_registration: ResolverRegistrationId,
    resolver_kind: ResolverRegistrationKind,
    source: RegistrationSource,
    validation: RegistrationValidationInput,
    obligations: &mut InitialObligationTable,
) -> Result<PendingRegistration, PendingValidationError> {
    if source.recovery() == RecoveryState::Recovered {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::RecoveredResolverOrigin,
            "checker.registration.recovered_resolver_origin",
            RegistrationDiagnosticClass::InvalidValidation,
        ));
    }
    if validation.resolver_registration != resolver_registration {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::UnknownValidationOrigin,
            "checker.registration.validation_origin_mismatch",
            RegistrationDiagnosticClass::InvalidValidation,
        ));
    }
    let kind = validation.pattern.kind();
    if resolver_kind_for_validation(kind) != resolver_kind {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::ValidationKindMismatch,
            "checker.registration.validation_kind_mismatch",
            RegistrationDiagnosticClass::InvalidValidation,
        ));
    }
    if source.symbol().is_none() {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::MissingRegistrationLabel,
            "checker.registration.missing_registration_label",
            RegistrationDiagnosticClass::InvalidValidation,
        ));
    }
    validate_referenced_symbols(&source, validation.referenced_symbols())?;
    validate_parameters(&source, validation.parameters())?;
    validate_correctness_condition(
        &source,
        validation.correctness_goal(),
        validation.correctness_provenance(),
    )?;
    validate_source_provenance(&source, validation.source_range())?;
    validate_registration_pattern(&source, validation.pattern())?;

    let obligation = obligations.insert(InitialObligationDraft {
        kind: InitialObligationKind::RegistrationCorrectness,
        owner: validation.owner().clone(),
        source_range: validation.source_range(),
        assumptions: validation.assumptions().to_vec(),
        goal: validation.correctness_goal().clone(),
        provenance: validation.correctness_provenance().clone(),
        status: InitialObligationStatus::Pending,
    });

    Ok(PendingRegistration {
        id: CheckerRegistrationId::new(resolver_registration.index()),
        resolver_registration,
        source,
        pattern_status: RegistrationPatternStatus::Validated(kind),
        status: PendingRegistrationStatus::AwaitingVerifierAcceptance,
        parameters: validation
            .parameters()
            .iter()
            .map(|parameter| parameter.key().clone())
            .collect(),
        obligations: vec![obligation],
    })
}

fn validate_referenced_symbols(
    source: &RegistrationSource,
    references: &[RegistrationReferencedSymbol],
) -> Result<(), PendingValidationError> {
    if references
        .iter()
        .any(|reference| reference.symbol().is_none())
    {
        return Err((
            Box::new(source.clone()),
            RejectedRegistrationReason::MissingReferencedSymbol,
            "checker.registration.missing_referenced_symbol",
            RegistrationDiagnosticClass::MissingReferencedSymbol,
        ));
    }
    if references
        .iter()
        .any(|reference| !reference.is_compatible())
    {
        return Err((
            Box::new(source.clone()),
            RejectedRegistrationReason::IncompatibleReferencedSymbol,
            "checker.registration.incompatible_referenced_symbol",
            RegistrationDiagnosticClass::IncompatibleReferencedSymbol,
        ));
    }
    Ok(())
}

fn validate_parameters(
    source: &RegistrationSource,
    parameters: &[RegistrationValidationParameter],
) -> Result<(), PendingValidationError> {
    if parameters
        .iter()
        .any(|parameter| key_is_missing(parameter.key().as_str()))
    {
        return Err((
            Box::new(source.clone()),
            RejectedRegistrationReason::InvalidRegistrationParameter,
            "checker.registration.invalid_parameter",
            RegistrationDiagnosticClass::InvalidRegistrationParameter,
        ));
    }
    if parameters
        .iter()
        .any(|parameter| !parameter.is_typed() || !parameter.facts_are_visible())
    {
        return Err((
            Box::new(source.clone()),
            RejectedRegistrationReason::InvalidRegistrationParameter,
            "checker.registration.invalid_parameter",
            RegistrationDiagnosticClass::InvalidRegistrationParameter,
        ));
    }
    Ok(())
}

fn validate_correctness_condition(
    source: &RegistrationSource,
    goal: &InitialObligationGoal,
    provenance: &InitialObligationProvenance,
) -> Result<(), PendingValidationError> {
    if key_is_missing(goal.as_str()) || key_is_missing(provenance.as_str()) {
        return Err((
            Box::new(source.clone()),
            RejectedRegistrationReason::MissingCorrectnessCondition,
            "checker.registration.missing_correctness_condition",
            RegistrationDiagnosticClass::MissingCorrectnessCondition,
        ));
    }
    Ok(())
}

fn validate_source_provenance(
    source: &RegistrationSource,
    source_range: SourceRange,
) -> Result<(), PendingValidationError> {
    if source_range.start > source_range.end {
        return Err((
            Box::new(source.clone()),
            RejectedRegistrationReason::MissingSourceProvenance,
            "checker.registration.missing_source_provenance",
            RegistrationDiagnosticClass::MissingSourceProvenance,
        ));
    }
    Ok(())
}

fn validate_registration_pattern(
    source: &RegistrationSource,
    pattern: &RegistrationValidationPattern,
) -> Result<(), PendingValidationError> {
    match pattern {
        RegistrationValidationPattern::Existential {
            type_head,
            attributes,
        } => {
            if key_is_missing(type_head.as_str()) || attributes.is_empty() {
                return invalid_pattern(source);
            }
            validate_attribute_keys(source, attributes)
        }
        RegistrationValidationPattern::Conditional {
            type_head,
            antecedent,
            consequent,
        } => {
            if key_is_missing(type_head.as_str()) || antecedent.is_empty() || consequent.is_empty()
            {
                return invalid_pattern(source);
            }
            validate_attribute_keys(source, antecedent)?;
            validate_attribute_keys(source, consequent)
        }
        RegistrationValidationPattern::Functorial {
            functor,
            result_type,
            consequent,
        } => {
            if key_is_missing(functor.as_str())
                || key_is_missing(result_type.as_str())
                || consequent.is_empty()
            {
                return invalid_pattern(source);
            }
            validate_attribute_keys(source, consequent)
        }
        RegistrationValidationPattern::Reduction { lhs, rhs } => {
            validate_reduction_pattern(source, lhs, rhs)
        }
    }
}

fn validate_attribute_keys(
    source: &RegistrationSource,
    attributes: &[RegistrationAttributeKey],
) -> Result<(), PendingValidationError> {
    if attributes
        .iter()
        .any(|attribute| key_is_missing(attribute.as_str()))
    {
        invalid_pattern(source)
    } else {
        Ok(())
    }
}

fn validate_reduction_pattern(
    source: &RegistrationSource,
    lhs: &RegistrationTermPattern,
    rhs: &RegistrationTermPattern,
) -> Result<(), PendingValidationError> {
    if key_is_missing(lhs.fingerprint().as_str())
        || key_is_missing(rhs.fingerprint().as_str())
        || lhs.size() == 0
        || rhs.size() == 0
    {
        return invalid_pattern(source);
    }
    if lhs.source_range().is_none() || rhs.source_range().is_none() {
        return Err((
            Box::new(source.clone()),
            RejectedRegistrationReason::MissingSourceProvenance,
            "checker.registration.missing_source_provenance",
            RegistrationDiagnosticClass::MissingSourceProvenance,
        ));
    }
    let lhs_vars = variable_occurrence_map(lhs.free_variables());
    let rhs_vars = variable_occurrence_map(rhs.free_variables());
    for (variable, rhs_count) in &rhs_vars {
        let lhs_count = lhs_vars.get(variable).copied().unwrap_or_default();
        if lhs_count == 0 || *rhs_count > lhs_count {
            return Err((
                Box::new(source.clone()),
                RejectedRegistrationReason::InvalidReductionOrientation,
                "checker.registration.invalid_reduction_orientation",
                RegistrationDiagnosticClass::InvalidReductionOrientation,
            ));
        }
    }
    if lhs.size() <= rhs.size() {
        return Err((
            Box::new(source.clone()),
            RejectedRegistrationReason::InvalidReductionOrientation,
            "checker.registration.invalid_reduction_orientation",
            RegistrationDiagnosticClass::InvalidReductionOrientation,
        ));
    }
    Ok(())
}

fn invalid_pattern(source: &RegistrationSource) -> Result<(), PendingValidationError> {
    Err((
        Box::new(source.clone()),
        RejectedRegistrationReason::MalformedRegistrationPattern,
        "checker.registration.malformed_pattern",
        RegistrationDiagnosticClass::InvalidValidation,
    ))
}

fn variable_occurrence_map(
    variables: &[RegistrationVariableOccurrence],
) -> BTreeMap<String, usize> {
    let mut map = BTreeMap::new();
    for occurrence in variables {
        *map.entry(occurrence.variable().as_str().to_owned())
            .or_default() += occurrence.count();
    }
    map
}

fn resolver_kind_for_validation(kind: RegistrationValidationKind) -> ResolverRegistrationKind {
    match kind {
        RegistrationValidationKind::Existential
        | RegistrationValidationKind::Conditional
        | RegistrationValidationKind::Functorial => ResolverRegistrationKind::Cluster,
        RegistrationValidationKind::Reduction => ResolverRegistrationKind::Reduction,
    }
}

fn validate_activation_companion_validations(
    resolver_registration: ResolverRegistrationId,
    resolver_kind: ResolverRegistrationKind,
    source: RegistrationSource,
    validations: &[RegistrationValidationInput],
) -> Result<Option<RegistrationValidationKind>, PendingValidationError> {
    match validations {
        [] => Ok(None),
        [validation] => {
            let kind = validation.pattern.kind();
            let mut scratch_obligations = InitialObligationTable::new();
            validate_pending_registration(
                resolver_registration,
                resolver_kind,
                source,
                validation.clone(),
                &mut scratch_obligations,
            )
            .map(|_| Some(kind))
        }
        _ => Err((
            Box::new(source),
            RejectedRegistrationReason::DuplicateValidationInput,
            "checker.registration.duplicate_validation_input",
            RegistrationDiagnosticClass::InvalidValidation,
        )),
    }
}

fn validate_activation(
    resolver_registration: ResolverRegistrationId,
    resolver_kind: ResolverRegistrationKind,
    source: RegistrationSource,
    activation: ActivationInput,
    companion_validation_kind: Option<RegistrationValidationKind>,
) -> Result<ActivatedRegistration, ActivationValidationError> {
    if source.recovery() == RecoveryState::Recovered {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::RecoveredResolverOrigin,
            "checker.registration.recovered_resolver_origin",
            RegistrationDiagnosticClass::InvalidActivation,
        ));
    }
    if activation.kind != resolver_kind {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::ActivationKindMismatch,
            "checker.registration.activation_kind_mismatch",
            RegistrationDiagnosticClass::InvalidActivation,
        ));
    }
    if key_is_missing(activation.trigger.as_str()) {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::MissingActivationTrigger,
            "checker.registration.missing_activation_trigger",
            RegistrationDiagnosticClass::InvalidActivation,
        ));
    }
    if key_is_missing(activation.pattern.as_str()) {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::MissingAcceptedPattern,
            "checker.registration.missing_accepted_pattern",
            RegistrationDiagnosticClass::InvalidActivation,
        ));
    }
    if key_is_missing(activation.correctness.as_str()) {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::MissingAcceptedCorrectness,
            "checker.registration.missing_accepted_correctness",
            RegistrationDiagnosticClass::InvalidActivation,
        ));
    }
    if key_is_missing(activation.evidence.as_str()) {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::MissingActivationEvidence,
            "checker.registration.missing_activation_evidence",
            RegistrationDiagnosticClass::InvalidActivation,
        ));
    }
    if activation.verifier_status != ActivationVerifierStatus::Accepted {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::UnacceptedActivationEvidence,
            "checker.registration.unaccepted_activation_evidence",
            RegistrationDiagnosticClass::UnacceptedActivationEvidence,
        ));
    }
    if let Some(validation_kind) = activation.validation_kind
        && resolver_kind_for_validation(validation_kind) != resolver_kind
    {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::ValidationKindMismatch,
            "checker.registration.validation_kind_mismatch",
            RegistrationDiagnosticClass::InvalidActivation,
        ));
    }
    if let (Some(explicit), Some(companion)) =
        (activation.validation_kind, companion_validation_kind)
        && explicit != companion
    {
        return Err((
            Box::new(source),
            RejectedRegistrationReason::ValidationKindMismatch,
            "checker.registration.validation_kind_mismatch",
            RegistrationDiagnosticClass::InvalidActivation,
        ));
    }
    let validation_kind = activation.validation_kind.or(companion_validation_kind);
    Ok(ActivatedRegistration {
        id: CheckerRegistrationId::new(resolver_registration.index()),
        resolver_registration,
        source,
        trigger: activation.trigger,
        label: activation.label,
        kind: activation.kind,
        validation_kind,
        pattern: activation.pattern,
        parameters: activation.parameters,
        correctness: activation.correctness,
        evidence: activation.evidence,
        fingerprint: activation.fingerprint,
    })
}

fn key_is_missing(value: &str) -> bool {
    value.trim().is_empty()
}

fn activation_map(
    activations: impl IntoIterator<Item = ActivationInput>,
) -> BTreeMap<ResolverRegistrationId, Vec<ActivationInput>> {
    let mut map: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for activation in activations {
        map.entry(activation.resolver_registration)
            .or_default()
            .push(activation);
    }
    for entries in map.values_mut() {
        entries.sort_by_key(activation_input_order_key);
    }
    map
}

fn validation_map(
    validations: impl IntoIterator<Item = RegistrationValidationInput>,
) -> BTreeMap<ResolverRegistrationId, Vec<RegistrationValidationInput>> {
    let mut map: BTreeMap<_, Vec<_>> = BTreeMap::new();
    for validation in validations {
        map.entry(validation.resolver_registration)
            .or_default()
            .push(validation);
    }
    for entries in map.values_mut() {
        entries.sort_by_key(validation_input_order_key);
    }
    map
}

fn source_order_key(
    contribution: SourceContributionId,
    origin: &SemanticOrigin,
    resolver_registration: ResolverRegistrationId,
) -> (usize, Vec<u32>, usize) {
    (
        contribution.index(),
        origin.structural_path().to_vec(),
        resolver_registration.index(),
    )
}

fn pending_order_key(entry: &PendingRegistration) -> (usize, Vec<u32>, usize, u8, String, usize) {
    source_registration_order_key(&entry.source)
}

fn rejected_order_key(
    entry: &RejectedRegistration,
) -> (usize, Vec<u32>, usize, u8, String, usize, u8) {
    let base = entry
        .source
        .as_ref()
        .map(source_registration_order_key)
        .unwrap_or_else(|| {
            (
                usize::MAX,
                Vec::new(),
                entry
                    .resolver_registration
                    .map_or(usize::MAX, ResolverRegistrationId::index),
                u8::MAX,
                String::new(),
                entry
                    .resolver_registration
                    .map_or(usize::MAX, ResolverRegistrationId::index),
            )
        });
    (
        base.0,
        base.1,
        base.2,
        base.3,
        base.4,
        base.5,
        rejected_reason_rank(entry.reason),
    )
}

fn source_registration_order_key(
    source: &RegistrationSource,
) -> (usize, Vec<u32>, usize, u8, String, usize) {
    (
        source.contribution.index(),
        source.origin.structural_path().to_vec(),
        source.resolver_registration.index(),
        registration_kind_rank(source.kind),
        label_fallback_key(source),
        source.resolver_registration.index(),
    )
}

fn activated_order_key(
    entry: &ActivatedRegistration,
) -> (
    String,
    String,
    String,
    Vec<u32>,
    usize,
    String,
    u8,
    String,
    usize,
) {
    (
        entry.trigger.as_str().to_owned(),
        entry
            .source
            .origin
            .module_id()
            .package()
            .as_str()
            .to_owned(),
        entry.source.origin.module_id().path().as_str().to_owned(),
        entry.source.origin.structural_path().to_vec(),
        entry.source.resolver_registration.index(),
        entry.label.as_ref().map_or_else(
            || label_fallback_key(&entry.source),
            |label| label.as_str().to_owned(),
        ),
        registration_kind_rank(entry.kind),
        entry.fingerprint.as_ref().map_or_else(
            || format!("pattern:{}", entry.pattern.as_str()),
            |fingerprint| fingerprint.as_str().to_owned(),
        ),
        entry.id.index(),
    )
}

fn activation_input_order_key(input: &ActivationInput) -> (String, String, String, u8) {
    (
        input.trigger.as_str().to_owned(),
        input
            .label
            .as_ref()
            .map_or_else(String::new, |label| label.as_str().to_owned()),
        input.pattern.as_str().to_owned(),
        input.validation_kind.map_or(255, validation_kind_rank),
    )
}

fn validation_input_order_key(input: &RegistrationValidationInput) -> (u8, String, usize) {
    (
        validation_kind_rank(input.pattern.kind()),
        input.correctness_goal.as_str().to_owned(),
        input.source_range.start,
    )
}

fn existential_gate_input_order_key(input: &ExistentialGateInput) -> ExistentialGateInputOrderKey {
    (
        source_id_order_key(input.source_range()),
        input.source_range().start,
        input.source_range().end,
        typed_site_order_key(input.owner()),
        input.pattern().as_str().to_owned(),
        input.trigger().as_str().to_owned(),
        canonical_attribute_key_strings(input.attributes().iter().cloned()),
        input.base_evidence().map(base_evidence_order_key),
    )
}

fn existential_candidate_report_order_key(
    report: &ExistentialCandidateReport,
) -> (u8, usize, Option<usize>) {
    (
        existential_candidate_status_rank(report.status),
        report.registration.index(),
        report
            .resolver_registration
            .map(ResolverRegistrationId::index),
    )
}

fn existential_gate_result_order_key(
    result: &ExistentialGateResult,
) -> ExistentialGateResultOrderKey {
    (
        source_id_order_key(result.source_range),
        result.source_range.start,
        result.source_range.end,
        typed_site_order_key(&result.owner),
        result.pattern.as_str().to_owned(),
        result.trigger.as_str().to_owned(),
        canonical_attribute_key_strings(result.attributes.iter().cloned()),
        result
            .base_evidence_kind
            .zip(result.base_evidence_coverage)
            .map(|(kind, coverage)| {
                (
                    base_evidence_kind_rank(kind),
                    base_evidence_coverage_rank(coverage),
                )
            }),
        result.id.index(),
    )
}

fn diagnostic_order_key(
    diagnostic: &RegistrationDiagnostic,
) -> (Option<usize>, u8, u8, String, usize) {
    (
        diagnostic
            .resolver_registration
            .map(ResolverRegistrationId::index),
        diagnostic_class_rank(diagnostic.class),
        diagnostic_severity_rank(diagnostic.severity),
        diagnostic.message_key.clone(),
        diagnostic.id.index(),
    )
}

fn label_fallback_key(source: &RegistrationSource) -> String {
    source.symbol.as_ref().map_or_else(
        || format!("registration#{}", source.resolver_registration.index()),
        |symbol| symbol.fqn().as_str().to_owned(),
    )
}

fn registration_kind_rank(kind: ResolverRegistrationKind) -> u8 {
    match kind {
        ResolverRegistrationKind::Cluster => 0,
        ResolverRegistrationKind::Reduction => 1,
        ResolverRegistrationKind::Identify => 2,
        ResolverRegistrationKind::Property => 3,
        _ => u8::MAX,
    }
}

fn validation_kind_rank(kind: RegistrationValidationKind) -> u8 {
    match kind {
        RegistrationValidationKind::Existential => 0,
        RegistrationValidationKind::Conditional => 1,
        RegistrationValidationKind::Functorial => 2,
        RegistrationValidationKind::Reduction => 3,
    }
}

fn rejected_reason_rank(reason: RejectedRegistrationReason) -> u8 {
    match reason {
        RejectedRegistrationReason::MalformedResolverTarget => 0,
        RejectedRegistrationReason::RecoveredResolverOrigin => 1,
        RejectedRegistrationReason::UnknownValidationOrigin => 2,
        RejectedRegistrationReason::UnknownActivationOrigin => 3,
        RejectedRegistrationReason::ValidationKindMismatch => 4,
        RejectedRegistrationReason::ActivationKindMismatch => 5,
        RejectedRegistrationReason::MissingRegistrationLabel => 6,
        RejectedRegistrationReason::MissingRegistrationPayload => 7,
        RejectedRegistrationReason::MalformedRegistrationPattern => 8,
        RejectedRegistrationReason::MissingReferencedSymbol => 9,
        RejectedRegistrationReason::IncompatibleReferencedSymbol => 10,
        RejectedRegistrationReason::InvalidRegistrationParameter => 11,
        RejectedRegistrationReason::MissingCorrectnessCondition => 12,
        RejectedRegistrationReason::MissingSourceProvenance => 13,
        RejectedRegistrationReason::InvalidReductionOrientation => 14,
        RejectedRegistrationReason::MissingActivationTrigger => 15,
        RejectedRegistrationReason::MissingAcceptedPattern => 16,
        RejectedRegistrationReason::MissingAcceptedCorrectness => 17,
        RejectedRegistrationReason::MissingActivationEvidence => 18,
        RejectedRegistrationReason::UnacceptedActivationEvidence => 19,
        RejectedRegistrationReason::DuplicateValidationInput => 20,
        RejectedRegistrationReason::DuplicateActivationInput => 21,
    }
}

fn diagnostic_class_rank(class: RegistrationDiagnosticClass) -> u8 {
    match class {
        RegistrationDiagnosticClass::ExternalDependencyGap => 0,
        RegistrationDiagnosticClass::MalformedResolverTarget => 1,
        RegistrationDiagnosticClass::InvalidValidation => 2,
        RegistrationDiagnosticClass::MissingReferencedSymbol => 3,
        RegistrationDiagnosticClass::IncompatibleReferencedSymbol => 4,
        RegistrationDiagnosticClass::InvalidRegistrationParameter => 5,
        RegistrationDiagnosticClass::MissingCorrectnessCondition => 6,
        RegistrationDiagnosticClass::MissingSourceProvenance => 7,
        RegistrationDiagnosticClass::InvalidReductionOrientation => 8,
        RegistrationDiagnosticClass::InvalidActivation => 9,
        RegistrationDiagnosticClass::UnacceptedActivationEvidence => 10,
        RegistrationDiagnosticClass::UnavailableExistentialRegistration => 11,
        RegistrationDiagnosticClass::BlockedExistentialGuard => 12,
        RegistrationDiagnosticClass::InvalidExistentialGateCandidate => 13,
        RegistrationDiagnosticClass::Recovery => 14,
    }
}

fn diagnostic_severity_rank(severity: RegistrationDiagnosticSeverity) -> u8 {
    match severity {
        RegistrationDiagnosticSeverity::Error => 0,
        RegistrationDiagnosticSeverity::Warning => 1,
        RegistrationDiagnosticSeverity::Note => 2,
    }
}

fn existential_candidate_status_rank(status: ExistentialCandidateStatus) -> u8 {
    match status {
        ExistentialCandidateStatus::Satisfied => 0,
        ExistentialCandidateStatus::BlockedGuard => 1,
        ExistentialCandidateStatus::Invalid => 2,
    }
}

fn write_pending(output: &mut String, pending: &PendingRegistrationTable) {
    output.push_str("pending:\n");
    if pending.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in pending.iter() {
        let _ = write!(
            output,
            "  pending#{} resolver=registration#{} status={} pattern={} params=",
            entry.id.index(),
            entry.resolver_registration.index(),
            pending_status_name(entry.status),
            pattern_status_name(entry.pattern_status)
        );
        write_parameter_keys(output, &entry.parameters);
        output.push_str(" obligations=");
        write_obligation_ids(output, &entry.obligations);
        output.push_str(" inference=false ");
        write_registration_source(output, &entry.source);
        output.push('\n');
    }
}

fn write_activated(output: &mut String, activated: &ActivatedRegistrationIndex) {
    output.push_str("activated:\n");
    if activated.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in activated.iter() {
        let _ = write!(
            output,
            "  active#{} resolver=registration#{} trigger=\"",
            entry.id.index(),
            entry.resolver_registration.index()
        );
        write_escaped(output, entry.trigger.as_str());
        output.push_str("\" label=");
        write_optional_key(
            output,
            entry.label.as_ref().map(RegistrationLabelKey::as_str),
        );
        output.push_str(" kind=");
        output.push_str(registration_kind_name(entry.kind));
        output.push_str(" validation_kind=");
        write_optional_validation_kind(output, entry.validation_kind);
        output.push_str(" pattern=\"");
        write_escaped(output, entry.pattern.as_str());
        output.push_str("\" params=");
        write_parameter_keys(output, &entry.parameters);
        output.push_str(" correctness=\"");
        write_escaped(output, entry.correctness.as_str());
        output.push_str("\" evidence=\"");
        write_escaped(output, entry.evidence.as_str());
        output.push_str("\" fingerprint=");
        write_optional_key(
            output,
            entry
                .fingerprint
                .as_ref()
                .map(RegistrationFingerprint::as_str),
        );
        output.push_str(" inference=true ");
        write_registration_source(output, &entry.source);
        output.push('\n');
    }
}

fn write_rejected(output: &mut String, rejected: &RejectedRegistrationTable) {
    output.push_str("rejected:\n");
    if rejected.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in rejected.iter() {
        let _ = write!(output, "  rejected#{} resolver=", entry.id.index());
        write_optional_resolver_registration(output, entry.resolver_registration);
        output.push_str(" reason=");
        output.push_str(rejected_reason_name(entry.reason));
        output.push_str(" inference=false");
        if let Some(source) = &entry.source {
            output.push(' ');
            write_registration_source(output, source);
        }
        output.push('\n');
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &RegistrationDiagnosticTable) {
    output.push_str("diagnostics:\n");
    if diagnostics.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, diagnostic) in diagnostics.canonical_iter() {
        let _ = write!(output, "  diagnostic#{} resolver=", id.index());
        write_optional_resolver_registration(output, diagnostic.resolver_registration);
        output.push_str(" class=");
        output.push_str(diagnostic_class_name(diagnostic.class));
        output.push_str(" severity=");
        output.push_str(diagnostic_severity_name(diagnostic.severity));
        output.push_str(" message=\"");
        write_escaped(output, &diagnostic.message_key);
        output.push_str("\" recovery=");
        output.push_str(diagnostic_recovery_name(diagnostic.recovery));
        output.push('\n');
    }
}

fn write_existential_gates(output: &mut String, gates: &[ExistentialGateResult]) {
    output.push_str("gates:\n");
    if gates.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for gate in gates {
        let _ = write!(
            output,
            "  gate#{} status={} seed={} registration=",
            gate.id.index(),
            existential_gate_status_name(gate.status),
            gate.may_seed_verified_facts()
        );
        write_optional_checker_registration(output, gate.registration);
        output.push_str(" base=");
        write_optional_base_evidence(output, gate.base_evidence_kind, gate.base_evidence_coverage);
        output.push_str(" pattern=\"");
        write_escaped(output, gate.pattern.as_str());
        output.push_str("\" trigger=\"");
        write_escaped(output, gate.trigger.as_str());
        output.push_str("\" attrs=");
        write_attribute_keys(output, &gate.attributes);
        output.push_str(" facts=");
        write_type_fact_ids(output, &gate.facts);
        output.push_str(" diagnostics=");
        write_registration_diagnostic_ids(output, &gate.diagnostics);
        output.push_str(" owner=");
        write_typed_site_ref(output, &gate.owner);
        output.push_str(" range=");
        write_range(output, gate.source_range);
        output.push('\n');
    }
}

fn write_initial_obligations(output: &mut String, obligations: &InitialObligationTable) {
    output.push_str("initial_obligations:\n");
    if obligations.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, obligation) in obligations.iter() {
        let _ = write!(
            output,
            "  obligation#{} kind={} status={} owner=",
            id.index(),
            initial_obligation_kind_name(obligation.kind),
            initial_obligation_status_name(obligation.status)
        );
        write_typed_site_ref(output, &obligation.owner);
        output.push_str(" range=");
        write_range(output, obligation.source_range);
        output.push_str(" assumptions=");
        write_type_fact_ids(output, &obligation.assumptions);
        output.push_str(" goal=\"");
        write_escaped(output, obligation.goal.as_str());
        output.push_str("\" provenance=\"");
        write_escaped(output, obligation.provenance.as_str());
        output.push_str("\"\n");
    }
}

fn write_registration_source(output: &mut String, source: &RegistrationSource) {
    let _ = write!(
        output,
        "source={{kind={} symbol=",
        registration_kind_name(source.kind)
    );
    write_optional_symbol(output, source.symbol.as_ref());
    output.push_str(" contribution=contribution#");
    let _ = write!(output, "{}", source.contribution.index());
    output.push_str(" visibility=");
    output.push_str(visibility_name(source.visibility));
    output.push_str(" export=");
    output.push_str(export_status_name(source.export_status));
    output.push_str(" recovery=");
    output.push_str(recovery_name(source.recovery));
    output.push_str(" deps=");
    write_dependency_ids(output, &source.dependencies);
    output.push_str(" target=");
    write_target_shell(output, &source.target);
    output.push_str(" origin=");
    write_origin(output, &source.origin);
    output.push('}');
}

fn write_origin(output: &mut String, origin: &SemanticOrigin) {
    output.push_str("origin(module=");
    write_module_id(output, origin.module_id());
    output.push_str(", path=");
    write_u32_slice(output, origin.structural_path());
    output.push_str(", anchor=");
    write_anchor(output, origin.anchor());
    if let Some(import_edge) = origin.import_edge() {
        let _ = write!(output, ", import=import#{}", import_edge.index());
    }
    let _ = write!(output, ", recovered={})", origin.is_recovered());
}

fn write_anchor(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(range) => write_range(output, *range),
        SourceAnchor::Point { offset, .. } => {
            let _ = write!(output, "point({offset})");
        }
        SourceAnchor::Generated(origin) => {
            output.push_str("generated(");
            match origin.anchor() {
                GeneratedSpanAnchor::Range(range) => write_range(output, range),
                GeneratedSpanAnchor::Point { offset, .. } => {
                    let _ = write!(output, "point({offset})");
                }
                _ => output.push_str("unknown"),
            }
            output.push_str(", reason=\"");
            write_escaped(output, origin.reason());
            output.push_str("\")");
        }
        _ => output.push_str("unknown"),
    }
}

fn write_range(output: &mut String, range: SourceRange) {
    let _ = write!(output, "range({}..{})", range.start, range.end);
}

fn write_target_shell(output: &mut String, target: &ResolverTargetShell) {
    match target {
        ResolverTargetShell::Pending => output.push_str("pending"),
        ResolverTargetShell::Opaque { schema, payload } => {
            output.push_str("opaque(schema=\"");
            write_escaped(output, schema);
            output.push_str("\", payload=\"");
            write_escaped(output, payload);
            output.push_str("\")");
        }
        ResolverTargetShell::Malformed { class } => {
            output.push_str("malformed(class=\"");
            write_escaped(output, class);
            output.push_str("\")");
        }
        ResolverTargetShell::Unsupported { class } => {
            output.push_str("unsupported(class=\"");
            write_escaped(output, class);
            output.push_str("\")");
        }
    }
}

fn write_module_id(output: &mut String, module: &ModuleId) {
    write_escaped(output, module.package().as_str());
    output.push_str("::");
    write_escaped(output, module.path().as_str());
}

fn write_optional_symbol(output: &mut String, symbol: Option<&SymbolId>) {
    if let Some(symbol) = symbol {
        output.push('"');
        write_escaped(output, symbol.fqn().as_str());
        output.push('"');
    } else {
        output.push_str("<none>");
    }
}

fn write_optional_resolver_registration(
    output: &mut String,
    registration: Option<ResolverRegistrationId>,
) {
    if let Some(registration) = registration {
        let _ = write!(output, "registration#{}", registration.index());
    } else {
        output.push_str("<none>");
    }
}

fn write_optional_checker_registration(
    output: &mut String,
    registration: Option<CheckerRegistrationId>,
) {
    if let Some(registration) = registration {
        let _ = write!(output, "checker-registration#{}", registration.index());
    } else {
        output.push_str("<none>");
    }
}

fn write_optional_base_evidence(
    output: &mut String,
    kind: Option<ExistentialGateBaseEvidenceKind>,
    coverage: Option<ExistentialGateBaseEvidenceCoverage>,
) {
    match (kind, coverage) {
        (Some(kind), Some(coverage)) => {
            output.push_str(base_evidence_kind_name(kind));
            output.push(':');
            output.push_str(base_evidence_coverage_name(coverage));
        }
        _ => output.push_str("<none>"),
    }
}

fn write_parameter_keys(output: &mut String, parameters: &[RegistrationParameterKey]) {
    output.push('[');
    for (index, parameter) in parameters.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('"');
        write_escaped(output, parameter.as_str());
        output.push('"');
    }
    output.push(']');
}

fn write_attribute_keys(output: &mut String, attributes: &[RegistrationAttributeKey]) {
    output.push('[');
    for (index, attribute) in attributes.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('"');
        write_escaped(output, attribute.as_str());
        output.push('"');
    }
    output.push(']');
}

fn write_obligation_ids(output: &mut String, obligations: &[InitialObligationId]) {
    output.push('[');
    for (index, obligation) in obligations.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "obligation#{}", obligation.index());
    }
    output.push(']');
}

fn write_registration_diagnostic_ids(
    output: &mut String,
    diagnostics: &[RegistrationDiagnosticId],
) {
    output.push('[');
    for (index, diagnostic) in diagnostics.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "diagnostic#{}", diagnostic.index());
    }
    output.push(']');
}

fn write_type_fact_ids(output: &mut String, facts: &[TypeFactId]) {
    output.push('[');
    for (index, fact) in facts.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "fact#{}", fact.index());
    }
    output.push(']');
}

fn write_dependency_ids(output: &mut String, dependencies: &[DeclarationDependencyId]) {
    output.push('[');
    for (index, dependency) in dependencies.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "dependency#{}", dependency.index());
    }
    output.push(']');
}

fn write_typed_site_ref(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node#{}", node.index());
        }
        TypedSiteRef::Role { node, role } => {
            let _ = write!(output, "node#{}:{}", node.index(), role.as_str());
        }
    }
}

fn write_u32_slice(output: &mut String, values: &[u32]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "{value}");
    }
    output.push(']');
}

fn write_optional_key(output: &mut String, value: Option<&str>) {
    if let Some(value) = value {
        output.push('"');
        write_escaped(output, value);
        output.push('"');
    } else {
        output.push_str("<none>");
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

fn visibility_name(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Public => "public",
        _ => "unknown",
    }
}

fn export_status_name(status: ExportStatus) -> &'static str {
    match status {
        ExportStatus::LocalOnly => "local_only",
        ExportStatus::Exported => "exported",
        ExportStatus::ReExported => "re_exported",
        _ => "unknown",
    }
}

fn recovery_name(recovery: RecoveryState) -> &'static str {
    match recovery {
        RecoveryState::Normal => "normal",
        RecoveryState::Recovered => "recovered",
        _ => "unknown",
    }
}

fn pattern_status_name(status: RegistrationPatternStatus) -> &'static str {
    match status {
        RegistrationPatternStatus::ExternalDependencyGap => "external_dependency_gap",
        RegistrationPatternStatus::Validated(kind) => validated_kind_name(kind),
    }
}

fn write_optional_validation_kind(output: &mut String, kind: Option<RegistrationValidationKind>) {
    if let Some(kind) = kind {
        output.push_str(validated_kind_name(kind));
    } else {
        output.push_str("<none>");
    }
}

fn validated_kind_name(kind: RegistrationValidationKind) -> &'static str {
    match kind {
        RegistrationValidationKind::Existential => "validated_existential",
        RegistrationValidationKind::Conditional => "validated_conditional",
        RegistrationValidationKind::Functorial => "validated_functorial",
        RegistrationValidationKind::Reduction => "validated_reduction",
    }
}

fn pending_status_name(status: PendingRegistrationStatus) -> &'static str {
    match status {
        PendingRegistrationStatus::BlockedExternalDependency => "blocked_external_dependency",
        PendingRegistrationStatus::AwaitingVerifierAcceptance => "awaiting_verifier_acceptance",
    }
}

fn rejected_reason_name(reason: RejectedRegistrationReason) -> &'static str {
    match reason {
        RejectedRegistrationReason::MalformedResolverTarget => "malformed_resolver_target",
        RejectedRegistrationReason::RecoveredResolverOrigin => "recovered_resolver_origin",
        RejectedRegistrationReason::UnknownValidationOrigin => "unknown_validation_origin",
        RejectedRegistrationReason::UnknownActivationOrigin => "unknown_activation_origin",
        RejectedRegistrationReason::ValidationKindMismatch => "validation_kind_mismatch",
        RejectedRegistrationReason::ActivationKindMismatch => "activation_kind_mismatch",
        RejectedRegistrationReason::MissingRegistrationLabel => "missing_registration_label",
        RejectedRegistrationReason::MissingRegistrationPayload => "missing_registration_payload",
        RejectedRegistrationReason::MalformedRegistrationPattern => {
            "malformed_registration_pattern"
        }
        RejectedRegistrationReason::MissingReferencedSymbol => "missing_referenced_symbol",
        RejectedRegistrationReason::IncompatibleReferencedSymbol => {
            "incompatible_referenced_symbol"
        }
        RejectedRegistrationReason::InvalidRegistrationParameter => {
            "invalid_registration_parameter"
        }
        RejectedRegistrationReason::MissingCorrectnessCondition => "missing_correctness_condition",
        RejectedRegistrationReason::MissingSourceProvenance => "missing_source_provenance",
        RejectedRegistrationReason::InvalidReductionOrientation => "invalid_reduction_orientation",
        RejectedRegistrationReason::MissingActivationTrigger => "missing_activation_trigger",
        RejectedRegistrationReason::MissingAcceptedPattern => "missing_accepted_pattern",
        RejectedRegistrationReason::MissingAcceptedCorrectness => "missing_accepted_correctness",
        RejectedRegistrationReason::MissingActivationEvidence => "missing_activation_evidence",
        RejectedRegistrationReason::UnacceptedActivationEvidence => {
            "unaccepted_activation_evidence"
        }
        RejectedRegistrationReason::DuplicateValidationInput => "duplicate_validation_input",
        RejectedRegistrationReason::DuplicateActivationInput => "duplicate_activation_input",
    }
}

fn diagnostic_class_name(class: RegistrationDiagnosticClass) -> &'static str {
    match class {
        RegistrationDiagnosticClass::ExternalDependencyGap => "external_dependency_gap",
        RegistrationDiagnosticClass::MalformedResolverTarget => "malformed_resolver_target",
        RegistrationDiagnosticClass::InvalidValidation => "invalid_validation",
        RegistrationDiagnosticClass::MissingReferencedSymbol => "missing_referenced_symbol",
        RegistrationDiagnosticClass::IncompatibleReferencedSymbol => {
            "incompatible_referenced_symbol"
        }
        RegistrationDiagnosticClass::InvalidRegistrationParameter => {
            "invalid_registration_parameter"
        }
        RegistrationDiagnosticClass::MissingCorrectnessCondition => "missing_correctness_condition",
        RegistrationDiagnosticClass::MissingSourceProvenance => "missing_source_provenance",
        RegistrationDiagnosticClass::InvalidReductionOrientation => "invalid_reduction_orientation",
        RegistrationDiagnosticClass::InvalidActivation => "invalid_activation",
        RegistrationDiagnosticClass::UnacceptedActivationEvidence => {
            "unaccepted_activation_evidence"
        }
        RegistrationDiagnosticClass::UnavailableExistentialRegistration => {
            "unavailable_existential_registration"
        }
        RegistrationDiagnosticClass::BlockedExistentialGuard => "blocked_existential_guard",
        RegistrationDiagnosticClass::InvalidExistentialGateCandidate => {
            "invalid_existential_gate_candidate"
        }
        RegistrationDiagnosticClass::Recovery => "recovery",
    }
}

fn diagnostic_severity_name(severity: RegistrationDiagnosticSeverity) -> &'static str {
    match severity {
        RegistrationDiagnosticSeverity::Error => "error",
        RegistrationDiagnosticSeverity::Warning => "warning",
        RegistrationDiagnosticSeverity::Note => "note",
    }
}

fn diagnostic_recovery_name(recovery: RegistrationDiagnosticRecovery) -> &'static str {
    match recovery {
        RegistrationDiagnosticRecovery::Normal => "normal",
        RegistrationDiagnosticRecovery::Degraded => "degraded",
    }
}

fn existential_gate_status_name(status: ExistentialGateStatus) -> &'static str {
    match status {
        ExistentialGateStatus::Satisfied => "satisfied",
        ExistentialGateStatus::MissingExistential => "missing_existential",
        ExistentialGateStatus::BlockedGuard => "blocked_guard",
        ExistentialGateStatus::InvalidCandidate => "invalid_candidate",
        ExistentialGateStatus::DegradedRecovery => "degraded_recovery",
    }
}

fn base_evidence_kind_name(kind: ExistentialGateBaseEvidenceKind) -> &'static str {
    match kind {
        ExistentialGateBaseEvidenceKind::BuiltinObject => "builtin_object",
        ExistentialGateBaseEvidenceKind::BuiltinSet => "builtin_set",
        ExistentialGateBaseEvidenceKind::AcceptedModeApplication => "accepted_mode_application",
        ExistentialGateBaseEvidenceKind::StructureConstructorWitness => {
            "structure_constructor_witness"
        }
        ExistentialGateBaseEvidenceKind::SchemaTypeParameter => "schema_type_parameter",
    }
}

fn base_evidence_coverage_name(coverage: ExistentialGateBaseEvidenceCoverage) -> &'static str {
    match coverage {
        ExistentialGateBaseEvidenceCoverage::Builtin => "builtin",
        ExistentialGateBaseEvidenceCoverage::CompleteGuardSet => "complete_guard_set",
        ExistentialGateBaseEvidenceCoverage::ZeroFieldStructure => "zero_field_structure",
    }
}

fn initial_obligation_kind_name(kind: InitialObligationKind) -> &'static str {
    match kind {
        InitialObligationKind::Sethood => "sethood",
        InitialObligationKind::NonEmptiness => "non_emptiness",
        InitialObligationKind::Narrowing => "narrowing",
        InitialObligationKind::RegistrationCorrectness => "registration_correctness",
    }
}

fn initial_obligation_status_name(status: InitialObligationStatus) -> &'static str {
    match status {
        InitialObligationStatus::Pending => "pending",
        InitialObligationStatus::Blocked => "blocked",
        InitialObligationStatus::Invalidated => "invalidated",
    }
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

impl fmt::Display for CheckerRegistrationId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "checker-registration#{}", self.index())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_ast::{TypeRole, TypedNodeId};
    use mizar_resolve::{
        env::{RegistrationIndex, SourceContributionIndex, SymbolEnvIndexes},
        resolved_ast::{FullyQualifiedName, LocalSymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceId,
    };

    #[test]
    fn pending_entries_never_contribute_and_keep_external_gap_diagnostics() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(&fixture.env, []);

        assert_eq!(database.pending().len(), 2);
        assert_eq!(database.activated().len(), 0);
        assert_eq!(database.rejected().len(), 1);
        assert!(
            database
                .pending()
                .iter()
                .all(|entry| !entry.may_contribute_to_inference())
        );
        assert!(
            database
                .rejected()
                .iter()
                .all(|entry| !entry.may_contribute_to_inference())
        );
        assert_eq!(
            database
                .diagnostics()
                .canonical_iter()
                .map(|(_, diagnostic)| diagnostic.message_key())
                .collect::<Vec<_>>(),
            vec![
                "checker.registration.external.semantic_payload",
                "checker.registration.external.semantic_payload",
                "checker.registration.malformed_resolver_target",
            ]
        );
    }

    #[test]
    fn activation_moves_entries_into_deterministic_trigger_order() {
        let fixture = env_fixture();
        let activations = vec![
            ActivationInput::accepted(
                fixture.cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:z",
                "pattern:z",
                "correctness:z",
                "evidence:z",
            )
            .with_label("pkg::main::ZReg")
            .with_fingerprint("fingerprint:z"),
            ActivationInput::accepted(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                "trigger:a",
                "pattern:a",
                "correctness:a",
                "evidence:a",
            )
            .with_label("pkg::main::AReg")
            .with_parameters(vec![
                RegistrationParameterKey::new("param:y"),
                RegistrationParameterKey::new("param:x"),
            ])
            .with_fingerprint("fingerprint:a"),
        ];

        let database = RegistrationDatabase::from_symbol_env(&fixture.env, activations);

        assert_eq!(database.pending().len(), 0);
        assert_eq!(database.activated().len(), 2);
        assert_eq!(database.rejected().len(), 1);
        assert!(
            database
                .activated()
                .iter()
                .all(ActivatedRegistration::may_contribute_to_inference)
        );
        let active_resolvers = database
            .activated()
            .iter()
            .map(ActivatedRegistration::resolver_registration)
            .collect::<Vec<_>>();
        assert_eq!(
            active_resolvers,
            vec![fixture.reduction_a, fixture.cluster_b]
        );
        assert_eq!(
            database
                .activated()
                .by_trigger(&RegistrationTriggerKey::new("trigger:a"))
                .len(),
            1
        );
        let first = database.activated().iter().next().unwrap();
        assert_eq!(
            first.parameters(),
            &[
                RegistrationParameterKey::new("param:y"),
                RegistrationParameterKey::new("param:x"),
            ]
        );
    }

    #[test]
    fn source_contributions_round_trip_through_all_tables() {
        let fixture = env_fixture();
        let activation = ActivationInput::accepted(
            fixture.reduction_a,
            ResolverRegistrationKind::Reduction,
            "trigger:a",
            "pattern:a",
            "correctness:a",
            "evidence:a",
        );

        let database = RegistrationDatabase::from_symbol_env(&fixture.env, [activation]);
        let active_contributions = database
            .activated()
            .iter()
            .map(|entry| entry.source().contribution())
            .collect::<Vec<_>>();
        let pending_contributions = database
            .pending()
            .iter()
            .map(|entry| entry.source().contribution())
            .collect::<Vec<_>>();
        let rejected_contributions = database
            .rejected()
            .iter()
            .filter_map(|entry| entry.source().map(RegistrationSource::contribution))
            .collect::<Vec<_>>();

        assert_eq!(active_contributions, vec![fixture.contribution_a]);
        assert_eq!(pending_contributions, vec![fixture.contribution_b]);
        assert_eq!(rejected_contributions, vec![fixture.contribution_c]);
    }

    #[test]
    fn invalid_activation_inputs_do_not_fabricate_active_records() {
        let fixture = env_fixture();
        let activation = ActivationInput::accepted(
            fixture.cluster_b,
            ResolverRegistrationKind::Reduction,
            "trigger:b",
            "pattern:b",
            "correctness:b",
            "evidence:b",
        );

        let database = RegistrationDatabase::from_symbol_env(&fixture.env, [activation]);

        assert_eq!(database.activated().len(), 0);
        assert!(
            database
                .rejected()
                .iter()
                .any(|entry| entry.reason() == RejectedRegistrationReason::ActivationKindMismatch)
        );
        assert!(
            database
                .diagnostics()
                .canonical_iter()
                .any(|(_, diagnostic)| diagnostic.message_key()
                    == "checker.registration.activation_kind_mismatch")
        );
    }

    #[test]
    fn invalid_activation_inputs_cover_all_rejection_paths() {
        let fixture = env_fixture();
        assert_invalid_activation(
            vec![ActivationInput::accepted(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                " ",
                "pattern:a",
                "correctness:a",
                "evidence:a",
            )],
            RejectedRegistrationReason::MissingActivationTrigger,
            "checker.registration.missing_activation_trigger",
        );
        assert_invalid_activation(
            vec![ActivationInput::accepted(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                "trigger:a",
                " ",
                "correctness:a",
                "evidence:a",
            )],
            RejectedRegistrationReason::MissingAcceptedPattern,
            "checker.registration.missing_accepted_pattern",
        );
        assert_invalid_activation(
            vec![ActivationInput::accepted(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                "trigger:a",
                "pattern:a",
                "",
                "evidence:a",
            )],
            RejectedRegistrationReason::MissingAcceptedCorrectness,
            "checker.registration.missing_accepted_correctness",
        );
        assert_invalid_activation(
            vec![ActivationInput::accepted(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                "trigger:a",
                "pattern:a",
                "correctness:a",
                "\t",
            )],
            RejectedRegistrationReason::MissingActivationEvidence,
            "checker.registration.missing_activation_evidence",
        );
        assert_invalid_activation(
            vec![
                ActivationInput::accepted(
                    fixture.reduction_a,
                    ResolverRegistrationKind::Reduction,
                    "trigger:a",
                    "pattern:a",
                    "correctness:a",
                    "evidence:a",
                ),
                ActivationInput::accepted(
                    fixture.reduction_a,
                    ResolverRegistrationKind::Reduction,
                    "trigger:a",
                    "pattern:b",
                    "correctness:b",
                    "evidence:b",
                ),
            ],
            RejectedRegistrationReason::DuplicateActivationInput,
            "checker.registration.duplicate_activation_input",
        );
        assert_invalid_activation(
            vec![ActivationInput::accepted(
                detached_registration_id(),
                ResolverRegistrationKind::Reduction,
                "trigger:unknown",
                "pattern:unknown",
                "correctness:unknown",
                "evidence:unknown",
            )],
            RejectedRegistrationReason::UnknownActivationOrigin,
            "checker.registration.unknown_activation_origin",
        );

        let recovered = recovered_env_fixture();
        let database = RegistrationDatabase::from_symbol_env(
            &recovered.env,
            [ActivationInput::accepted(
                recovered.registration,
                ResolverRegistrationKind::Cluster,
                "trigger:recovered",
                "pattern:recovered",
                "correctness:recovered",
                "evidence:recovered",
            )],
        );
        assert_eq!(database.activated().len(), 0);
        assert_rejection(
            &database,
            RejectedRegistrationReason::RecoveredResolverOrigin,
            "checker.registration.recovered_resolver_origin",
        );

        let no_label = no_label_env_fixture();
        let database = RegistrationDatabase::from_symbol_env_with_validation(
            &no_label.env,
            [validation(
                no_label.registration,
                existential_pattern(),
                "goal:no-label",
                "provenance:no-label",
            )],
            [],
        );
        assert_eq!(database.initial_obligations().len(), 0);
        assert_rejection(
            &database,
            RejectedRegistrationReason::MissingRegistrationLabel,
            "checker.registration.missing_registration_label",
        );
    }

    #[test]
    fn same_trigger_activation_order_is_canonical() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [
                ActivationInput::accepted(
                    fixture.cluster_b,
                    ResolverRegistrationKind::Cluster,
                    "trigger:shared",
                    "pattern:b",
                    "correctness:b",
                    "evidence:b",
                )
                .with_label("pkg::main::BReg"),
                ActivationInput::accepted(
                    fixture.reduction_a,
                    ResolverRegistrationKind::Reduction,
                    "trigger:shared",
                    "pattern:a",
                    "correctness:a",
                    "evidence:a",
                )
                .with_label("pkg::main::AReg"),
            ],
        );

        let trigger_entries = database
            .activated()
            .by_trigger(&RegistrationTriggerKey::new("trigger:shared"));
        assert_eq!(trigger_entries.len(), 2);
        assert_eq!(
            trigger_entries
                .iter()
                .map(|entry| entry.resolver_registration())
                .collect::<Vec<_>>(),
            vec![fixture.reduction_a, fixture.cluster_b]
        );
        assert_ordered_fragments(
            &database.debug_text(),
            &[
                "active#0 resolver=registration#0 trigger=\"trigger:shared\" label=\"pkg::main::AReg\"",
                "active#1 resolver=registration#1 trigger=\"trigger:shared\" label=\"pkg::main::BReg\"",
            ],
        );
    }

    #[test]
    fn debug_rendering_is_stable_and_ordered_by_checker_keys() {
        let fixture = env_fixture();
        let first = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [
                ActivationInput::accepted(
                    fixture.cluster_b,
                    ResolverRegistrationKind::Cluster,
                    "trigger:b",
                    "pattern:b",
                    "correctness:b",
                    "evidence:b",
                )
                .with_label("pkg::main::BReg"),
                ActivationInput::accepted(
                    fixture.reduction_a,
                    ResolverRegistrationKind::Reduction,
                    "trigger:a",
                    "pattern:a",
                    "correctness:a",
                    "evidence:a",
                )
                .with_label("pkg::main::AReg"),
            ],
        )
        .debug_text();
        let second = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [
                ActivationInput::accepted(
                    fixture.reduction_a,
                    ResolverRegistrationKind::Reduction,
                    "trigger:a",
                    "pattern:a",
                    "correctness:a",
                    "evidence:a",
                )
                .with_label("pkg::main::AReg"),
                ActivationInput::accepted(
                    fixture.cluster_b,
                    ResolverRegistrationKind::Cluster,
                    "trigger:b",
                    "pattern:b",
                    "correctness:b",
                    "evidence:b",
                )
                .with_label("pkg::main::BReg"),
            ],
        )
        .debug_text();

        assert_eq!(first, second);
        assert!(first.starts_with("registration-database-debug-v1\n"));
        assert_ordered_fragments(
            &first,
            &[
                "activated:\n  active#0 resolver=registration#0 trigger=\"trigger:a\"",
                "  active#1 resolver=registration#1 trigger=\"trigger:b\"",
                "rejected:\n  rejected#0 resolver=registration#2",
            ],
        );
        assert!(first.contains("visibility=public export=exported"));
    }

    #[test]
    fn validated_payloads_emit_pending_obligations_without_activation() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                fixture.cluster_b,
                existential_pattern(),
                "goal:existence",
                "provenance:existential",
            )
            .with_parameters([RegistrationValidationParameter::new("param:T")])
            .with_referenced_symbols([RegistrationReferencedSymbol::compatible(
                RegistrationReferencedSymbolRole::Attribute,
                symbol_id(module_id(), "inhabited", "pkg::main::inhabited"),
            )])],
            [],
        );

        assert_eq!(database.activated().len(), 0);
        assert_eq!(database.initial_obligations().len(), 1);
        let pending = database
            .pending()
            .get(CheckerRegistrationId::new(fixture.cluster_b.index()))
            .unwrap();
        assert_eq!(
            pending.pattern_status(),
            RegistrationPatternStatus::Validated(RegistrationValidationKind::Existential)
        );
        assert_eq!(
            pending.status(),
            PendingRegistrationStatus::AwaitingVerifierAcceptance
        );
        assert_eq!(
            pending.parameters(),
            &[RegistrationParameterKey::new("param:T")]
        );
        assert_eq!(pending.obligations(), &[InitialObligationId::new(0)]);
        assert!(!pending.may_contribute_to_inference());

        let (_, obligation) = database.initial_obligations().iter().next().unwrap();
        assert_eq!(
            obligation.kind,
            InitialObligationKind::RegistrationCorrectness
        );
        assert_eq!(obligation.status, InitialObligationStatus::Pending);
        assert_eq!(obligation.goal.as_str(), "goal:existence");
        assert!(
            !database.debug_text().contains(concat!("Vc", "Id")),
            "registration validation must not allocate proof-owned ids"
        );
    }

    #[test]
    fn kind_specific_validation_accepts_existential_conditional_functorial_and_reduction() {
        let fixture = env_fixture();
        assert_validated_kind(
            &fixture,
            fixture.cluster_b,
            existential_pattern(),
            RegistrationValidationKind::Existential,
        );
        assert_validated_kind(
            &fixture,
            fixture.cluster_b,
            conditional_pattern(),
            RegistrationValidationKind::Conditional,
        );
        assert_validated_kind(
            &fixture,
            fixture.cluster_b,
            functorial_pattern(),
            RegistrationValidationKind::Functorial,
        );
        assert_validated_kind(
            &fixture,
            fixture.reduction_a,
            valid_reduction_pattern(),
            RegistrationValidationKind::Reduction,
        );
    }

    #[test]
    fn invalid_validation_payloads_are_diagnosed_without_obligations() {
        let fixture = env_fixture();
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                existential_pattern(),
                "goal:missing-symbol",
                "provenance:missing-symbol",
            )
            .with_referenced_symbols([RegistrationReferencedSymbol::missing(
                RegistrationReferencedSymbolRole::Attribute,
            )]),
            RejectedRegistrationReason::MissingReferencedSymbol,
            "checker.registration.missing_referenced_symbol",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                existential_pattern(),
                "goal:incompatible-symbol",
                "provenance:incompatible-symbol",
            )
            .with_referenced_symbols([RegistrationReferencedSymbol::incompatible(
                RegistrationReferencedSymbolRole::Attribute,
                symbol_id(module_id(), "wrong", "pkg::main::wrong"),
            )]),
            RejectedRegistrationReason::IncompatibleReferencedSymbol,
            "checker.registration.incompatible_referenced_symbol",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                RegistrationValidationPattern::Existential {
                    type_head: RegistrationTypeKey::new("type:T"),
                    attributes: Vec::new(),
                },
                "goal:malformed",
                "provenance:malformed",
            ),
            RejectedRegistrationReason::MalformedRegistrationPattern,
            "checker.registration.malformed_pattern",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                conditional_pattern(),
                "goal:param",
                "provenance:param",
            )
            .with_parameters([RegistrationValidationParameter::new("param:x").with_typed(false)]),
            RejectedRegistrationReason::InvalidRegistrationParameter,
            "checker.registration.invalid_parameter",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                conditional_pattern(),
                "goal:param-visibility",
                "provenance:param-visibility",
            )
            .with_parameters([
                RegistrationValidationParameter::new("param:x").with_visible_facts(false)
            ]),
            RejectedRegistrationReason::InvalidRegistrationParameter,
            "checker.registration.invalid_parameter",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                functorial_pattern(),
                " ",
                "provenance:missing",
            ),
            RejectedRegistrationReason::MissingCorrectnessCondition,
            "checker.registration.missing_correctness_condition",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                valid_reduction_pattern(),
                "goal:kind",
                "provenance:kind",
            ),
            RejectedRegistrationReason::ValidationKindMismatch,
            "checker.registration.validation_kind_mismatch",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation_with_range(
                fixture.cluster_b,
                existential_pattern(),
                "goal:source-range",
                "provenance:source-range",
                SourceRange {
                    source_id: source_id(),
                    start: 10,
                    end: 9,
                },
            ),
            RejectedRegistrationReason::MissingSourceProvenance,
            "checker.registration.missing_source_provenance",
        );

        let recovered = recovered_env_fixture();
        let database = RegistrationDatabase::from_symbol_env_with_validation(
            &recovered.env,
            [validation(
                recovered.registration,
                existential_pattern(),
                "goal:recovered",
                "provenance:recovered",
            )],
            [],
        );
        assert_eq!(database.initial_obligations().len(), 0);
        assert_rejection(
            &database,
            RejectedRegistrationReason::RecoveredResolverOrigin,
            "checker.registration.recovered_resolver_origin",
        );
    }

    #[test]
    fn validation_input_routing_errors_are_diagnosed() {
        let fixture = env_fixture();
        let duplicate = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [
                validation(
                    fixture.cluster_b,
                    existential_pattern(),
                    "goal:duplicate-a",
                    "provenance:duplicate-a",
                ),
                validation(
                    fixture.cluster_b,
                    conditional_pattern(),
                    "goal:duplicate-b",
                    "provenance:duplicate-b",
                ),
            ],
            [],
        );
        assert_eq!(duplicate.initial_obligations().len(), 0);
        assert_rejection(
            &duplicate,
            RejectedRegistrationReason::DuplicateValidationInput,
            "checker.registration.duplicate_validation_input",
        );

        let unknown = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                detached_registration_id(),
                existential_pattern(),
                "goal:unknown",
                "provenance:unknown",
            )],
            [],
        );
        assert_eq!(unknown.initial_obligations().len(), 0);
        assert_rejection(
            &unknown,
            RejectedRegistrationReason::UnknownValidationOrigin,
            "checker.registration.unknown_validation_origin",
        );
    }

    #[test]
    fn malformed_kind_specific_patterns_are_rejected() {
        let fixture = env_fixture();
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                RegistrationValidationPattern::Conditional {
                    type_head: RegistrationTypeKey::new("type:T"),
                    antecedent: Vec::new(),
                    consequent: vec![RegistrationAttributeKey::new("attr:B")],
                },
                "goal:conditional-antecedent",
                "provenance:conditional-antecedent",
            ),
            RejectedRegistrationReason::MalformedRegistrationPattern,
            "checker.registration.malformed_pattern",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                RegistrationValidationPattern::Conditional {
                    type_head: RegistrationTypeKey::new("type:T"),
                    antecedent: vec![RegistrationAttributeKey::new("attr:A")],
                    consequent: Vec::new(),
                },
                "goal:conditional-consequent",
                "provenance:conditional-consequent",
            ),
            RejectedRegistrationReason::MalformedRegistrationPattern,
            "checker.registration.malformed_pattern",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                RegistrationValidationPattern::Functorial {
                    functor: RegistrationFunctorKey::new(" "),
                    result_type: RegistrationTypeKey::new("type:Result"),
                    consequent: vec![RegistrationAttributeKey::new("attr:computed")],
                },
                "goal:functorial-functor",
                "provenance:functorial-functor",
            ),
            RejectedRegistrationReason::MalformedRegistrationPattern,
            "checker.registration.malformed_pattern",
        );
        assert_invalid_validation(
            &fixture,
            fixture.cluster_b,
            validation(
                fixture.cluster_b,
                RegistrationValidationPattern::Functorial {
                    functor: RegistrationFunctorKey::new("functor:F"),
                    result_type: RegistrationTypeKey::new("type:Result"),
                    consequent: vec![RegistrationAttributeKey::new(" ")],
                },
                "goal:functorial-attribute",
                "provenance:functorial-attribute",
            ),
            RejectedRegistrationReason::MalformedRegistrationPattern,
            "checker.registration.malformed_pattern",
        );
        assert_invalid_validation(
            &fixture,
            fixture.reduction_a,
            validation(
                fixture.reduction_a,
                reduction_pattern(
                    term_pattern(" ", 2, [var("x", 1)]),
                    term_pattern("term:x", 1, [var("x", 1)]),
                ),
                "goal:reduction-key",
                "provenance:reduction-key",
            ),
            RejectedRegistrationReason::MalformedRegistrationPattern,
            "checker.registration.malformed_pattern",
        );
        assert_invalid_validation(
            &fixture,
            fixture.reduction_a,
            validation(
                fixture.reduction_a,
                reduction_pattern(
                    term_pattern("term:F(x)", 0, [var("x", 1)]),
                    term_pattern("term:x", 1, [var("x", 1)]),
                ),
                "goal:reduction-size",
                "provenance:reduction-size",
            ),
            RejectedRegistrationReason::MalformedRegistrationPattern,
            "checker.registration.malformed_pattern",
        );
    }

    #[test]
    fn reduction_validation_enforces_free_variables_size_order_and_provenance() {
        let fixture = env_fixture();
        assert_invalid_validation(
            &fixture,
            fixture.reduction_a,
            validation(
                fixture.reduction_a,
                reduction_pattern(
                    term_pattern("term:f", 2, [var("x", 1)]),
                    term_pattern("term:g", 1, [var("y", 1)]),
                ),
                "goal:free-variable",
                "provenance:free-variable",
            ),
            RejectedRegistrationReason::InvalidReductionOrientation,
            "checker.registration.invalid_reduction_orientation",
        );
        assert_invalid_validation(
            &fixture,
            fixture.reduction_a,
            validation(
                fixture.reduction_a,
                reduction_pattern(
                    term_pattern("term:f", 3, [var("x", 1)]),
                    term_pattern("term:g", 1, [var("x", 2)]),
                ),
                "goal:occurrence-count",
                "provenance:occurrence-count",
            ),
            RejectedRegistrationReason::InvalidReductionOrientation,
            "checker.registration.invalid_reduction_orientation",
        );
        assert_invalid_validation(
            &fixture,
            fixture.reduction_a,
            validation(
                fixture.reduction_a,
                reduction_pattern(
                    term_pattern("term:f", 2, [var("x", 1)]),
                    term_pattern("term:g", 2, [var("x", 1)]),
                ),
                "goal:size",
                "provenance:size",
            ),
            RejectedRegistrationReason::InvalidReductionOrientation,
            "checker.registration.invalid_reduction_orientation",
        );
        assert_invalid_validation(
            &fixture,
            fixture.reduction_a,
            validation(
                fixture.reduction_a,
                reduction_pattern(
                    RegistrationTermPattern::without_source_range("term:f", 2, [var("x", 1)]),
                    term_pattern("term:g", 1, [var("x", 1)]),
                ),
                "goal:source",
                "provenance:source",
            ),
            RejectedRegistrationReason::MissingSourceProvenance,
            "checker.registration.missing_source_provenance",
        );
    }

    #[test]
    fn accepted_activation_requires_valid_companion_validation_when_supplied() {
        let fixture = env_fixture();
        let active = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                fixture.reduction_a,
                valid_reduction_pattern(),
                "goal:accepted",
                "provenance:accepted",
            )],
            [ActivationInput::accepted(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                "trigger:a",
                "pattern:a",
                "correctness:a",
                "evidence:a",
            )],
        );
        assert_eq!(active.activated().len(), 1);
        assert_eq!(active.pending().len(), 1);
        assert_eq!(active.initial_obligations().len(), 0);

        let invalid = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                fixture.reduction_a,
                reduction_pattern(
                    term_pattern("term:f", 2, [var("x", 1)]),
                    term_pattern("term:g", 2, [var("x", 1)]),
                ),
                "goal:invalid-companion",
                "provenance:invalid-companion",
            )],
            [ActivationInput::accepted(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                "trigger:a",
                "pattern:a",
                "correctness:a",
                "evidence:a",
            )],
        );
        assert_eq!(invalid.activated().len(), 0);
        assert_eq!(invalid.initial_obligations().len(), 0);
        assert_rejection(
            &invalid,
            RejectedRegistrationReason::InvalidReductionOrientation,
            "checker.registration.invalid_reduction_orientation",
        );
    }

    #[test]
    fn unaccepted_activation_evidence_keeps_validated_registration_pending() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                fixture.reduction_a,
                valid_reduction_pattern(),
                "goal:reducibility",
                "provenance:reducibility",
            )],
            [ActivationInput::new(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                "trigger:a",
                "pattern:a",
                "correctness:a",
                "evidence:a",
            )],
        );

        assert_eq!(database.activated().len(), 0);
        assert_eq!(database.initial_obligations().len(), 1);
        assert_eq!(
            database
                .pending()
                .get(CheckerRegistrationId::new(fixture.reduction_a.index()))
                .unwrap()
                .status(),
            PendingRegistrationStatus::AwaitingVerifierAcceptance
        );
        assert_rejection(
            &database,
            RejectedRegistrationReason::UnacceptedActivationEvidence,
            "checker.registration.unaccepted_activation_evidence",
        );

        let rejected = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                fixture.reduction_a,
                valid_reduction_pattern(),
                "goal:rejected",
                "provenance:rejected",
            )],
            [ActivationInput::accepted(
                fixture.reduction_a,
                ResolverRegistrationKind::Reduction,
                "trigger:a",
                "pattern:a",
                "correctness:a",
                "evidence:a",
            )
            .with_verifier_status(ActivationVerifierStatus::Rejected)],
        );
        assert_eq!(rejected.activated().len(), 0);
        assert_eq!(rejected.initial_obligations().len(), 1);
        assert_rejection(
            &rejected,
            RejectedRegistrationReason::UnacceptedActivationEvidence,
            "checker.registration.unaccepted_activation_evidence",
        );
    }

    #[test]
    fn existential_gate_missing_and_inactive_candidates_do_not_seed_facts() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(&fixture.env, []);

        let missing = ExistentialGateOutput::evaluate(&database, [gate_input("pattern:T")]);
        let missing_gate = missing.iter().next().unwrap();
        assert_eq!(
            missing_gate.status(),
            ExistentialGateStatus::MissingExistential
        );
        assert!(!missing_gate.may_seed_verified_facts());
        assert!(missing_gate.facts().is_empty());
        assert_diagnostic(
            missing.diagnostics(),
            "checker.registration.unavailable_existential_registration",
        );

        let inactive = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:T").with_candidates([
                gate_candidate(fixture.cluster_b, "pattern:T"),
                gate_candidate(fixture.malformed_c, "pattern:T"),
            ])],
        );
        let inactive_gate = inactive.iter().next().unwrap();
        assert_eq!(
            inactive_gate.status(),
            ExistentialGateStatus::InvalidCandidate
        );
        assert!(!inactive_gate.may_seed_verified_facts());
        assert_eq!(inactive_gate.registration(), None);
        assert_diagnostic(
            inactive.diagnostics(),
            "checker.registration.invalid_existential_gate_candidate",
        );
    }

    #[test]
    fn existential_gate_pending_and_unaccepted_existentials_do_not_seed_facts() {
        let fixture = env_fixture();

        let pending = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                fixture.cluster_b,
                existential_pattern(),
                "goal:existence",
                "provenance:existence",
            )],
            [],
        );
        assert_eq!(pending.activated().len(), 0);
        assert_eq!(
            pending
                .pending()
                .get(CheckerRegistrationId::new(fixture.cluster_b.index()))
                .unwrap()
                .status(),
            PendingRegistrationStatus::AwaitingVerifierAcceptance
        );
        let pending_gate = ExistentialGateOutput::evaluate(
            &pending,
            [gate_input("pattern:T")
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:T")])],
        );
        let gate = pending_gate.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::InvalidCandidate);
        assert!(!gate.may_seed_verified_facts());
        assert_eq!(gate.registration(), None);
        assert!(gate.facts().is_empty());

        let unaccepted = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                fixture.cluster_b,
                existential_pattern(),
                "goal:unaccepted",
                "provenance:unaccepted",
            )],
            [ActivationInput::new(
                fixture.cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:existential",
                "pattern:T",
                "correctness:existential",
                "evidence:existential",
            )],
        );
        assert_eq!(unaccepted.activated().len(), 0);
        assert_rejection(
            &unaccepted,
            RejectedRegistrationReason::UnacceptedActivationEvidence,
            "checker.registration.unaccepted_activation_evidence",
        );
        let unaccepted_gate = ExistentialGateOutput::evaluate(
            &unaccepted,
            [gate_input("pattern:T")
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:T")])],
        );
        let gate = unaccepted_gate.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::InvalidCandidate);
        assert!(!gate.may_seed_verified_facts());
        assert!(gate.facts().is_empty());

        let rejected = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                fixture.cluster_b,
                existential_pattern(),
                "goal:rejected",
                "provenance:rejected",
            )],
            [ActivationInput::accepted(
                fixture.cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:existential",
                "pattern:T",
                "correctness:existential",
                "evidence:existential",
            )
            .with_verifier_status(ActivationVerifierStatus::Rejected)],
        );
        assert_eq!(rejected.activated().len(), 0);
        assert_rejection(
            &rejected,
            RejectedRegistrationReason::UnacceptedActivationEvidence,
            "checker.registration.unaccepted_activation_evidence",
        );
        let rejected_gate = ExistentialGateOutput::evaluate(
            &rejected,
            [gate_input("pattern:T")
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:T")])],
        );
        let gate = rejected_gate.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::InvalidCandidate);
        assert!(!gate.may_seed_verified_facts());
        assert!(gate.facts().is_empty());
    }

    #[test]
    fn activated_existential_gate_requires_accepted_binding_and_visible_guards() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [ActivationInput::accepted(
                fixture.cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:existential",
                "pattern:subset:X",
                "correctness:existential",
                "evidence:existential",
            )
            .with_validation_kind(RegistrationValidationKind::Existential)
            .with_fingerprint("fingerprint:existential")],
        );

        let satisfied = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:subset:X")
                .with_required_guards([ExistentialGateGuardKey::new("guard:X")])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    "guard:X",
                    TypeFactId::new(2),
                )])
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:subset:X")
                    .with_fingerprint("fingerprint:existential")])],
        );
        let gate = satisfied.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert!(gate.may_seed_verified_facts());
        assert_eq!(
            gate.registration(),
            Some(CheckerRegistrationId::new(fixture.cluster_b.index()))
        );
        assert_eq!(gate.facts(), &[TypeFactId::new(2)]);

        let duplicate_evidence = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:subset:X")
                .with_required_guards([ExistentialGateGuardKey::new("guard:X")])
                .with_guard_evidence([
                    ExistentialGateGuardEvidence::new("guard:X", TypeFactId::new(7)),
                    ExistentialGateGuardEvidence::new("guard:X", TypeFactId::new(2)),
                ])
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:subset:X")
                    .with_fingerprint("fingerprint:existential")])],
        );
        let reversed_evidence = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:subset:X")
                .with_required_guards([ExistentialGateGuardKey::new("guard:X")])
                .with_guard_evidence([
                    ExistentialGateGuardEvidence::new("guard:X", TypeFactId::new(2)),
                    ExistentialGateGuardEvidence::new("guard:X", TypeFactId::new(7)),
                ])
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:subset:X")
                    .with_fingerprint("fingerprint:existential")])],
        );
        assert_eq!(
            duplicate_evidence.debug_text(),
            reversed_evidence.debug_text()
        );
        assert_eq!(
            duplicate_evidence.iter().next().unwrap().facts(),
            &[TypeFactId::new(2)]
        );

        let hidden_guard = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:subset:X")
                .with_required_guards([ExistentialGateGuardKey::new("guard:X")])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    "guard:X",
                    TypeFactId::new(2),
                )
                .with_visible(false)])
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:subset:X")
                    .with_fingerprint("fingerprint:existential")])],
        );
        let gate = hidden_guard.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::BlockedGuard);
        assert!(!gate.may_seed_verified_facts());
        assert_diagnostic(
            hidden_guard.diagnostics(),
            "checker.registration.blocked_existential_guard",
        );

        let missing_guard = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:subset:X")
                .with_required_guards([ExistentialGateGuardKey::new("guard:X")])
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:subset:X")
                    .with_fingerprint("fingerprint:existential")])],
        );
        let gate = missing_guard.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::BlockedGuard);
        assert!(!gate.may_seed_verified_facts());
        assert!(gate.facts().is_empty());

        let non_consumable_guard = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:subset:X")
                .with_required_guards([ExistentialGateGuardKey::new("guard:X")])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    "guard:X",
                    TypeFactId::new(2),
                )
                .with_consumable(false)])
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:subset:X")
                    .with_fingerprint("fingerprint:existential")])],
        );
        let gate = non_consumable_guard.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::BlockedGuard);
        assert!(!gate.may_seed_verified_facts());
        assert!(gate.facts().is_empty());
    }

    #[test]
    fn base_inhabitation_evidence_satisfies_only_unattributed_exact_patterns() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(&fixture.env, []);

        let builtin_object = ExistentialGateOutput::evaluate(
            &database,
            [
                unattributed_gate("type:object", 10, 11).with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::BuiltinObject,
                    "type:object",
                    ExistentialGateBaseEvidenceCoverage::Builtin,
                )),
            ],
        );
        let gate = builtin_object.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert!(gate.may_seed_verified_facts());
        assert_eq!(gate.registration(), None);
        assert_eq!(
            gate.base_evidence_kind(),
            Some(ExistentialGateBaseEvidenceKind::BuiltinObject)
        );
        assert_eq!(
            gate.base_evidence_coverage(),
            Some(ExistentialGateBaseEvidenceCoverage::Builtin)
        );
        assert!(gate.facts().is_empty());

        let builtin_set = ExistentialGateOutput::evaluate(
            &database,
            [
                unattributed_gate("type:set", 12, 13).with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::BuiltinSet,
                    "type:set",
                    ExistentialGateBaseEvidenceCoverage::Builtin,
                )),
            ],
        );
        assert_eq!(
            builtin_set.iter().next().unwrap().base_evidence_kind(),
            Some(ExistentialGateBaseEvidenceKind::BuiltinSet)
        );

        let mode_guard = ExistentialGateGuardKey::new("mode:M(args:a):accepted");
        let accepted_mode = ExistentialGateOutput::evaluate(
            &database,
            [unattributed_gate("mode:M(args:a)", 14, 15)
                .with_required_guards([mode_guard.clone()])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    mode_guard.clone(),
                    TypeFactId::new(4),
                )])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::AcceptedModeApplication,
                    "mode:M(args:a)",
                    ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
                ))],
        );
        let gate = accepted_mode.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert_eq!(gate.registration(), None);
        assert_eq!(
            gate.base_evidence_kind(),
            Some(ExistentialGateBaseEvidenceKind::AcceptedModeApplication)
        );
        assert_eq!(gate.facts(), &[TypeFactId::new(4)]);
        assert!(
            accepted_mode
                .debug_text()
                .contains("base=accepted_mode_application:complete_guard_set")
        );

        let mismatched_mode_tuple = ExistentialGateOutput::evaluate(
            &database,
            [unattributed_gate("mode:M(args:a)", 16, 17)
                .with_required_guards([mode_guard.clone()])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    mode_guard,
                    TypeFactId::new(4),
                )])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::AcceptedModeApplication,
                    "mode:M(args:b)",
                    ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
                ))],
        );
        let gate = mismatched_mode_tuple.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::InvalidCandidate);
        assert!(!gate.may_seed_verified_facts());

        let attributed_gate = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("type:object").with_base_evidence(base_evidence(
                ExistentialGateBaseEvidenceKind::BuiltinObject,
                "type:object",
                ExistentialGateBaseEvidenceCoverage::Builtin,
            ))],
        );
        let gate = attributed_gate.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::InvalidCandidate);
        assert_eq!(gate.registration(), None);
        assert_eq!(gate.base_evidence_kind(), None);
        assert!(!gate.may_seed_verified_facts());
    }

    #[test]
    fn base_inhabitation_evidence_requires_complete_structure_and_schema_guards() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(&fixture.env, []);

        let zero_field_structure = ExistentialGateOutput::evaluate(
            &database,
            [
                unattributed_gate("struct:Zero", 20, 21).with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::StructureConstructorWitness,
                    "struct:Zero",
                    ExistentialGateBaseEvidenceCoverage::ZeroFieldStructure,
                )),
            ],
        );
        let gate = zero_field_structure.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert_eq!(
            gate.base_evidence_coverage(),
            Some(ExistentialGateBaseEvidenceCoverage::ZeroFieldStructure)
        );
        assert!(gate.facts().is_empty());

        let field_guard = ExistentialGateGuardKey::new("field:carrier:inhabited");
        let field_structure = ExistentialGateOutput::evaluate(
            &database,
            [unattributed_gate("struct:OneField", 22, 23)
                .with_required_guards([field_guard.clone()])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    field_guard.clone(),
                    TypeFactId::new(8),
                )])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::StructureConstructorWitness,
                    "struct:OneField",
                    ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
                ))],
        );
        let gate = field_structure.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert_eq!(gate.facts(), &[TypeFactId::new(8)]);

        let omitted_field_guards = ExistentialGateOutput::evaluate(
            &database,
            [
                unattributed_gate("struct:OneField", 24, 25).with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::StructureConstructorWitness,
                    "struct:OneField",
                    ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
                )),
            ],
        );
        let gate = omitted_field_guards.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::InvalidCandidate);
        assert!(!gate.may_seed_verified_facts());

        let hidden_field = ExistentialGateOutput::evaluate(
            &database,
            [unattributed_gate("struct:OneField", 26, 27)
                .with_required_guards([field_guard.clone()])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    field_guard.clone(),
                    TypeFactId::new(8),
                )
                .with_visible(false)])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::StructureConstructorWitness,
                    "struct:OneField",
                    ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
                ))],
        );
        let gate = hidden_field.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::BlockedGuard);
        assert!(gate.facts().is_empty());

        let non_consumable_field = ExistentialGateOutput::evaluate(
            &database,
            [unattributed_gate("struct:OneField", 28, 29)
                .with_required_guards([field_guard.clone()])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    field_guard.clone(),
                    TypeFactId::new(8),
                )
                .with_consumable(false)])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::StructureConstructorWitness,
                    "struct:OneField",
                    ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
                ))],
        );
        let gate = non_consumable_field.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::BlockedGuard);
        assert!(gate.facts().is_empty());

        let schema_guard = ExistentialGateGuardKey::new("schema:T:inhabited");
        let schema_parameter = ExistentialGateOutput::evaluate(
            &database,
            [unattributed_gate("schema:T", 30, 31)
                .with_required_guards([schema_guard.clone()])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    schema_guard.clone(),
                    TypeFactId::new(11),
                )])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::SchemaTypeParameter,
                    "schema:T",
                    ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
                ))],
        );
        let gate = schema_parameter.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert_eq!(
            gate.base_evidence_kind(),
            Some(ExistentialGateBaseEvidenceKind::SchemaTypeParameter)
        );
        assert_eq!(gate.facts(), &[TypeFactId::new(11)]);

        let schema_without_context = ExistentialGateOutput::evaluate(
            &database,
            [
                unattributed_gate("schema:T", 32, 33).with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::SchemaTypeParameter,
                    "schema:T",
                    ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
                )),
            ],
        );
        assert_eq!(
            schema_without_context.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );
    }

    #[test]
    fn base_inhabitation_evidence_rejects_invalid_coverage_combinations() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(&fixture.env, []);
        let guard = ExistentialGateGuardKey::new("guard:coverage");
        let guarded = |input: ExistentialGateInput| {
            input
                .with_required_guards([guard.clone()])
                .with_guard_evidence([ExistentialGateGuardEvidence::new(
                    guard.clone(),
                    TypeFactId::new(21),
                )])
        };
        let invalid_cases = vec![
            unattributed_gate("type:object", 40, 41).with_base_evidence(base_evidence(
                ExistentialGateBaseEvidenceKind::BuiltinObject,
                "type:object",
                ExistentialGateBaseEvidenceCoverage::CompleteGuardSet,
            )),
            guarded(unattributed_gate("type:set", 42, 43)).with_base_evidence(base_evidence(
                ExistentialGateBaseEvidenceKind::BuiltinSet,
                "type:set",
                ExistentialGateBaseEvidenceCoverage::Builtin,
            )),
            guarded(unattributed_gate("mode:M(args:a)", 44, 45)).with_base_evidence(base_evidence(
                ExistentialGateBaseEvidenceKind::AcceptedModeApplication,
                "mode:M(args:a)",
                ExistentialGateBaseEvidenceCoverage::Builtin,
            )),
            guarded(unattributed_gate("mode:M(args:a)", 46, 47)).with_base_evidence(base_evidence(
                ExistentialGateBaseEvidenceKind::AcceptedModeApplication,
                "mode:M(args:a)",
                ExistentialGateBaseEvidenceCoverage::ZeroFieldStructure,
            )),
            guarded(unattributed_gate("schema:T", 48, 49)).with_base_evidence(base_evidence(
                ExistentialGateBaseEvidenceKind::SchemaTypeParameter,
                "schema:T",
                ExistentialGateBaseEvidenceCoverage::Builtin,
            )),
            guarded(unattributed_gate("schema:T", 50, 51)).with_base_evidence(base_evidence(
                ExistentialGateBaseEvidenceKind::SchemaTypeParameter,
                "schema:T",
                ExistentialGateBaseEvidenceCoverage::ZeroFieldStructure,
            )),
            guarded(unattributed_gate("struct:OneField", 52, 53)).with_base_evidence(
                base_evidence(
                    ExistentialGateBaseEvidenceKind::StructureConstructorWitness,
                    "struct:OneField",
                    ExistentialGateBaseEvidenceCoverage::Builtin,
                ),
            ),
            guarded(unattributed_gate("struct:Zero", 54, 55)).with_base_evidence(base_evidence(
                ExistentialGateBaseEvidenceKind::StructureConstructorWitness,
                "struct:Zero",
                ExistentialGateBaseEvidenceCoverage::ZeroFieldStructure,
            )),
        ];

        for (index, input) in invalid_cases.into_iter().enumerate() {
            let output = ExistentialGateOutput::evaluate(&database, [input]);
            let gate = output.iter().next().unwrap();
            assert_eq!(
                gate.status(),
                ExistentialGateStatus::InvalidCandidate,
                "invalid coverage case {index} should reject"
            );
            assert_eq!(gate.base_evidence_kind(), None);
            assert!(gate.facts().is_empty());
            assert!(!gate.may_seed_verified_facts());
        }
    }

    #[test]
    fn existential_gate_satisfaction_source_is_candidate_or_base_evidence() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [ActivationInput::accepted(
                fixture.cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:existential",
                "type:object",
                "correctness:existential",
                "evidence:existential",
            )
            .with_validation_kind(RegistrationValidationKind::Existential)],
        );

        let candidate_over_invalid_base = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("type:object")
                .with_candidates([gate_candidate(fixture.cluster_b, "type:object")])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::BuiltinObject,
                    "type:object",
                    ExistentialGateBaseEvidenceCoverage::Builtin,
                ))],
        );
        let gate = candidate_over_invalid_base.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert_eq!(
            gate.registration(),
            Some(CheckerRegistrationId::new(fixture.cluster_b.index()))
        );
        assert_eq!(gate.base_evidence_kind(), None);

        let base_over_invalid_candidate = ExistentialGateOutput::evaluate(
            &database,
            [unattributed_gate("type:set", 56, 57)
                .with_candidates([gate_candidate(fixture.cluster_b, "type:object")])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::BuiltinSet,
                    "type:set",
                    ExistentialGateBaseEvidenceCoverage::Builtin,
                ))],
        );
        let gate = base_over_invalid_candidate.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert_eq!(gate.registration(), None);
        assert_eq!(
            gate.base_evidence_kind(),
            Some(ExistentialGateBaseEvidenceKind::BuiltinSet)
        );

        let candidate_preferred_over_valid_base = ExistentialGateOutput::evaluate(
            &database,
            [unattributed_gate("type:object", 58, 59)
                .with_candidates([gate_candidate(fixture.cluster_b, "type:object")])
                .with_base_evidence(base_evidence(
                    ExistentialGateBaseEvidenceKind::BuiltinObject,
                    "type:object",
                    ExistentialGateBaseEvidenceCoverage::Builtin,
                ))],
        );
        let gate = candidate_preferred_over_valid_base.iter().next().unwrap();
        assert_eq!(gate.status(), ExistentialGateStatus::Satisfied);
        assert_eq!(
            gate.registration(),
            Some(CheckerRegistrationId::new(fixture.cluster_b.index()))
        );
        assert_eq!(gate.base_evidence_kind(), None);
    }

    #[test]
    fn existential_gate_rejects_non_existential_or_mismatched_accepted_evidence() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [
                ActivationInput::accepted(
                    fixture.cluster_b,
                    ResolverRegistrationKind::Cluster,
                    "trigger:existential",
                    "pattern:subset:X",
                    "correctness:existential",
                    "evidence:existential",
                )
                .with_validation_kind(RegistrationValidationKind::Existential),
                ActivationInput::accepted(
                    fixture.reduction_a,
                    ResolverRegistrationKind::Reduction,
                    "trigger:reduction",
                    "pattern:reduction",
                    "correctness:reduction",
                    "evidence:reduction",
                ),
            ],
        );

        let non_existential = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:subset:X").with_candidates([
                gate_candidate(fixture.cluster_b, "pattern:subset:X")
                    .with_kind(RegistrationValidationKind::Conditional),
                gate_candidate(fixture.reduction_a, "pattern:reduction"),
            ])],
        );
        assert_eq!(
            non_existential.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );

        let parameter_mismatch = ExistentialGateOutput::evaluate(
            &database,
            [gate_input("pattern:subset:Y")
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:subset:X")])],
        );
        assert_eq!(
            parameter_mismatch.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );

        let correctness_mismatch = ExistentialGateOutput::evaluate(
            &database,
            [
                gate_input("pattern:subset:X").with_candidates([gate_candidate(
                    fixture.cluster_b,
                    "pattern:subset:X",
                )
                .with_correctness("correctness:other")]),
            ],
        );
        assert_eq!(
            correctness_mismatch.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );

        let conditional_database = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [ActivationInput::accepted(
                fixture.cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:existential",
                "pattern:subset:X",
                "correctness:existential",
                "evidence:existential",
            )
            .with_validation_kind(RegistrationValidationKind::Conditional)],
        );
        let asserted_existential = ExistentialGateOutput::evaluate(
            &conditional_database,
            [gate_input("pattern:subset:X")
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:subset:X")])],
        );
        assert_eq!(
            asserted_existential.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );

        let evidence_mismatch = ExistentialGateOutput::evaluate(
            &database,
            [
                gate_input("pattern:subset:X").with_candidates([gate_candidate(
                    fixture.cluster_b,
                    "pattern:subset:X",
                )
                .with_activation_evidence("evidence:other")]),
            ],
        );
        assert_eq!(
            evidence_mismatch.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );

        let trigger_mismatch = ExistentialGateOutput::evaluate(
            &database,
            [
                gate_input("pattern:subset:X").with_candidates([gate_candidate(
                    fixture.cluster_b,
                    "pattern:subset:X",
                )
                .with_trigger("trigger:other")]),
            ],
        );
        assert_eq!(
            trigger_mismatch.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );

        let attribute_mismatch = ExistentialGateOutput::evaluate(
            &database,
            [
                gate_input("pattern:subset:X").with_candidates([gate_candidate(
                    fixture.cluster_b,
                    "pattern:subset:X",
                )
                .with_attributes(Vec::<RegistrationAttributeKey>::new())]),
            ],
        );
        assert_eq!(
            attribute_mismatch.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );

        let fingerprinted_database = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [ActivationInput::accepted(
                fixture.cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:existential",
                "pattern:subset:X",
                "correctness:existential",
                "evidence:existential",
            )
            .with_validation_kind(RegistrationValidationKind::Existential)
            .with_fingerprint("fingerprint:existential")],
        );
        let fingerprint_mismatch = ExistentialGateOutput::evaluate(
            &fingerprinted_database,
            [
                gate_input("pattern:subset:X").with_candidates([gate_candidate(
                    fixture.cluster_b,
                    "pattern:subset:X",
                )
                .with_fingerprint("fingerprint:other")]),
            ],
        );
        assert_eq!(
            fingerprint_mismatch.iter().next().unwrap().status(),
            ExistentialGateStatus::InvalidCandidate
        );
    }

    #[test]
    fn existential_gate_degraded_recovery_and_ordering_are_deterministic() {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(
            &fixture.env,
            [ActivationInput::accepted(
                fixture.cluster_b,
                ResolverRegistrationKind::Cluster,
                "trigger:existential",
                "pattern:T",
                "correctness:existential",
                "evidence:existential",
            )
            .with_validation_kind(RegistrationValidationKind::Existential)],
        );
        let inputs = [
            gate_input_with_range("pattern:Z", 60, 61),
            gate_input_with_range("pattern:T", 50, 51)
                .with_recovery(ExistentialGateRecovery::Degraded)
                .with_candidates([gate_candidate(fixture.cluster_b, "pattern:T")]),
        ];
        let first = ExistentialGateOutput::evaluate(&database, inputs.clone()).debug_text();
        let second =
            ExistentialGateOutput::evaluate(&database, inputs.into_iter().rev()).debug_text();

        assert_eq!(first, second);
        assert_ordered_fragments(
            &first,
            &[
                "gate#0 status=degraded_recovery seed=false",
                "gate#1 status=missing_existential seed=false",
            ],
        );
        assert!(first.contains("checker.registration.degraded_existential_gate"));

        let tied_inputs = [
            gate_input_with_owner_and_attrs(
                "pattern:T",
                70,
                71,
                TypedSiteRef::Role {
                    node: TypedNodeId::new(2),
                    role: TypeRole::new("zeta_gate"),
                },
                [RegistrationAttributeKey::new("attr:Z")],
            )
            .with_candidates([gate_candidate(fixture.cluster_b, "pattern:T")]),
            gate_input_with_owner_and_attrs(
                "pattern:T",
                70,
                71,
                TypedSiteRef::Role {
                    node: TypedNodeId::new(1),
                    role: TypeRole::new("alpha_gate"),
                },
                [RegistrationAttributeKey::new("attr:A")],
            )
            .with_candidates([gate_candidate(fixture.cluster_b, "pattern:T")]),
        ];
        let first = ExistentialGateOutput::evaluate(&database, tied_inputs.clone()).debug_text();
        let second =
            ExistentialGateOutput::evaluate(&database, tied_inputs.into_iter().rev()).debug_text();
        assert_eq!(first, second);
        assert_ordered_fragments(
            &first,
            &["owner=node#1:alpha_gate", "owner=node#2:zeta_gate"],
        );
    }

    struct EnvFixture {
        env: SymbolEnv,
        reduction_a: ResolverRegistrationId,
        cluster_b: ResolverRegistrationId,
        malformed_c: ResolverRegistrationId,
        contribution_a: SourceContributionId,
        contribution_b: SourceContributionId,
        contribution_c: SourceContributionId,
    }

    struct RecoveredEnvFixture {
        env: SymbolEnv,
        registration: ResolverRegistrationId,
    }

    fn assert_invalid_activation(
        activations: Vec<ActivationInput>,
        reason: RejectedRegistrationReason,
        message_key: &str,
    ) {
        let fixture = env_fixture();
        let database = RegistrationDatabase::from_symbol_env(&fixture.env, activations);
        assert_eq!(database.activated().len(), 0);
        assert_rejection(&database, reason, message_key);
    }

    fn assert_rejection(
        database: &RegistrationDatabase,
        reason: RejectedRegistrationReason,
        message_key: &str,
    ) {
        assert!(
            database
                .rejected()
                .iter()
                .any(|entry| entry.reason() == reason),
            "missing rejected reason {reason:?} in\n{}",
            database.debug_text()
        );
        assert!(
            database
                .diagnostics()
                .canonical_iter()
                .any(|(_, diagnostic)| diagnostic.message_key() == message_key),
            "missing diagnostic {message_key} in\n{}",
            database.debug_text()
        );
    }

    fn assert_diagnostic(diagnostics: &RegistrationDiagnosticTable, message_key: &str) {
        assert!(
            diagnostics
                .canonical_iter()
                .any(|(_, diagnostic)| diagnostic.message_key() == message_key),
            "missing diagnostic {message_key}"
        );
    }

    fn assert_validated_kind(
        fixture: &EnvFixture,
        resolver_registration: ResolverRegistrationId,
        pattern: RegistrationValidationPattern,
        kind: RegistrationValidationKind,
    ) {
        let database = RegistrationDatabase::from_symbol_env_with_validation(
            &fixture.env,
            [validation(
                resolver_registration,
                pattern,
                "goal:valid",
                "provenance:valid",
            )],
            [],
        );

        let pending = database
            .pending()
            .get(CheckerRegistrationId::new(resolver_registration.index()))
            .unwrap_or_else(|| {
                panic!("missing pending registration in\n{}", database.debug_text())
            });
        assert_eq!(
            pending.pattern_status(),
            RegistrationPatternStatus::Validated(kind)
        );
        assert_eq!(pending.obligations(), &[InitialObligationId::new(0)]);
        assert_eq!(database.initial_obligations().len(), 1);
    }

    fn assert_invalid_validation(
        fixture: &EnvFixture,
        resolver_registration: ResolverRegistrationId,
        input: RegistrationValidationInput,
        reason: RejectedRegistrationReason,
        message_key: &str,
    ) {
        let database =
            RegistrationDatabase::from_symbol_env_with_validation(&fixture.env, [input], []);
        assert_eq!(
            database.initial_obligations().len(),
            0,
            "invalid validation must not emit obligations:\n{}",
            database.debug_text()
        );
        assert!(
            database
                .pending()
                .get(CheckerRegistrationId::new(resolver_registration.index()))
                .is_none(),
            "invalid validation must not leave a usable pending payload:\n{}",
            database.debug_text()
        );
        assert_rejection(&database, reason, message_key);
    }

    fn validation(
        resolver_registration: ResolverRegistrationId,
        pattern: RegistrationValidationPattern,
        goal: &str,
        provenance: &str,
    ) -> RegistrationValidationInput {
        validation_with_range(
            resolver_registration,
            pattern,
            goal,
            provenance,
            range(source_id(), 30, 31),
        )
    }

    fn validation_with_range(
        resolver_registration: ResolverRegistrationId,
        pattern: RegistrationValidationPattern,
        goal: &str,
        provenance: &str,
        source_range: SourceRange,
    ) -> RegistrationValidationInput {
        RegistrationValidationInput::new(
            resolver_registration,
            TypedSiteRef::Role {
                node: TypedNodeId::new(0),
                role: TypeRole::new("registration"),
            },
            source_range,
            pattern,
            goal,
            provenance,
        )
    }

    fn gate_input(pattern: &str) -> ExistentialGateInput {
        gate_input_with_range(pattern, 50, 51)
    }

    fn unattributed_gate(pattern: &str, start: usize, end: usize) -> ExistentialGateInput {
        gate_input_with_owner_and_attrs(
            pattern,
            start,
            end,
            TypedSiteRef::Role {
                node: TypedNodeId::new(0),
                role: TypeRole::new("base_inhabitation_gate"),
            },
            Vec::<RegistrationAttributeKey>::new(),
        )
    }

    fn gate_input_with_range(pattern: &str, start: usize, end: usize) -> ExistentialGateInput {
        gate_input_with_owner_and_attrs(
            pattern,
            start,
            end,
            TypedSiteRef::Role {
                node: TypedNodeId::new(0),
                role: TypeRole::new("existential_gate"),
            },
            [RegistrationAttributeKey::new("attr:inhabited")],
        )
    }

    fn gate_input_with_owner_and_attrs(
        pattern: &str,
        start: usize,
        end: usize,
        owner: TypedSiteRef,
        attributes: impl IntoIterator<Item = RegistrationAttributeKey>,
    ) -> ExistentialGateInput {
        ExistentialGateInput::new(
            owner,
            range(source_id(), start, end),
            pattern,
            "trigger:existential",
            attributes,
        )
    }

    fn base_evidence(
        kind: ExistentialGateBaseEvidenceKind,
        pattern: &str,
        coverage: ExistentialGateBaseEvidenceCoverage,
    ) -> ExistentialGateBaseEvidence {
        ExistentialGateBaseEvidence::new(kind, pattern, coverage)
    }

    fn gate_candidate(
        resolver_registration: ResolverRegistrationId,
        pattern: &str,
    ) -> ExistentialGateCandidate {
        ExistentialGateCandidate::new(
            CheckerRegistrationId::new(resolver_registration.index()),
            pattern,
            "correctness:existential",
            "evidence:existential",
            "trigger:existential",
            [RegistrationAttributeKey::new("attr:inhabited")],
        )
    }

    fn existential_pattern() -> RegistrationValidationPattern {
        RegistrationValidationPattern::Existential {
            type_head: RegistrationTypeKey::new("type:T"),
            attributes: vec![RegistrationAttributeKey::new("attr:inhabited")],
        }
    }

    fn conditional_pattern() -> RegistrationValidationPattern {
        RegistrationValidationPattern::Conditional {
            type_head: RegistrationTypeKey::new("type:T"),
            antecedent: vec![RegistrationAttributeKey::new("attr:A")],
            consequent: vec![RegistrationAttributeKey::new("attr:B")],
        }
    }

    fn functorial_pattern() -> RegistrationValidationPattern {
        RegistrationValidationPattern::Functorial {
            functor: RegistrationFunctorKey::new("functor:F"),
            result_type: RegistrationTypeKey::new("type:Result"),
            consequent: vec![RegistrationAttributeKey::new("attr:computed")],
        }
    }

    fn valid_reduction_pattern() -> RegistrationValidationPattern {
        reduction_pattern(
            term_pattern("term:F(x)", 2, [var("x", 1)]),
            term_pattern("term:x", 1, [var("x", 1)]),
        )
    }

    fn reduction_pattern(
        lhs: RegistrationTermPattern,
        rhs: RegistrationTermPattern,
    ) -> RegistrationValidationPattern {
        RegistrationValidationPattern::Reduction { lhs, rhs }
    }

    fn term_pattern(
        fingerprint: &str,
        size: usize,
        variables: impl IntoIterator<Item = RegistrationVariableOccurrence>,
    ) -> RegistrationTermPattern {
        RegistrationTermPattern::new(fingerprint, size, variables, range(source_id(), 40, 41))
    }

    fn var(variable: &str, count: usize) -> RegistrationVariableOccurrence {
        RegistrationVariableOccurrence::new(variable, count)
    }

    fn env_fixture() -> EnvFixture {
        let source = source_id();
        let module = module_id();
        let namespace_origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 0, 1)),
            vec![0],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution_a = contributions.insert(
            module.clone(),
            mizar_resolve::env::ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        let contribution_b = contributions.insert(
            module.clone(),
            mizar_resolve::env::ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 2)),
        );
        let contribution_c = contributions.insert(
            module.clone(),
            mizar_resolve::env::ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 2, 3)),
        );

        let mut registrations = RegistrationIndex::new();
        let reduction_a = registrations.insert(
            Some(symbol_id(module.clone(), "AReg", "pkg::main::AReg")),
            ResolverRegistrationKind::Reduction,
            SignatureShell::Opaque {
                schema: "registration-target-v1".to_owned(),
                payload: "reduce-shell".to_owned(),
            },
            namespace_origin.clone(),
            contribution_a,
        );
        let cluster_b = registrations.insert(
            Some(symbol_id(module.clone(), "BReg", "pkg::main::BReg")),
            ResolverRegistrationKind::Cluster,
            SignatureShell::Pending,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 1, 2)),
                vec![1],
            ),
            contribution_b,
        );
        let malformed_c = registrations.insert(
            Some(symbol_id(module.clone(), "CReg", "pkg::main::CReg")),
            ResolverRegistrationKind::Cluster,
            SignatureShell::Malformed {
                class: "recovered-target".to_owned(),
            },
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 2, 3)),
                vec![2],
            ),
            contribution_c,
        );
        registrations
            .get_mut(reduction_a)
            .unwrap()
            .set_visibility(Visibility::Public)
            .set_export_status(ExportStatus::Exported);
        registrations
            .get_mut(cluster_b)
            .unwrap()
            .set_visibility(Visibility::Public)
            .set_export_status(ExportStatus::ReExported);
        registrations
            .get_mut(malformed_c)
            .unwrap()
            .set_visibility(Visibility::Private)
            .set_export_status(ExportStatus::LocalOnly);

        contributions.add_registration(contribution_a, reduction_a);
        contributions.add_registration(contribution_b, cluster_b);
        contributions.add_registration(contribution_c, malformed_c);

        let indexes = SymbolEnvIndexes {
            registrations,
            contributions,
            ..SymbolEnvIndexes::default()
        };
        EnvFixture {
            env: SymbolEnv::new(module, indexes),
            reduction_a,
            cluster_b,
            malformed_c,
            contribution_a,
            contribution_b,
            contribution_c,
        }
    }

    fn recovered_env_fixture() -> RecoveredEnvFixture {
        let source = source_id();
        let module = module_id();
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 10, 11)),
            vec![10],
        )
        .recovered();
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            mizar_resolve::env::ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 10, 11)),
        );
        let mut registrations = RegistrationIndex::new();
        let registration = registrations.insert(
            Some(symbol_id(
                module.clone(),
                "RecoveredReg",
                "pkg::main::RecoveredReg",
            )),
            ResolverRegistrationKind::Cluster,
            SignatureShell::Pending,
            origin,
            contribution,
        );
        contributions.add_registration(contribution, registration);

        let indexes = SymbolEnvIndexes {
            registrations,
            contributions,
            ..SymbolEnvIndexes::default()
        };
        RecoveredEnvFixture {
            env: SymbolEnv::new(module, indexes),
            registration,
        }
    }

    fn no_label_env_fixture() -> RecoveredEnvFixture {
        let source = source_id();
        let module = module_id();
        let origin = SemanticOrigin::new(
            source,
            module.clone(),
            SourceAnchor::Range(range(source, 15, 16)),
            vec![15],
        );
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            mizar_resolve::env::ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 15, 16)),
        );
        let mut registrations = RegistrationIndex::new();
        let registration = registrations.insert(
            None,
            ResolverRegistrationKind::Cluster,
            SignatureShell::Pending,
            origin,
            contribution,
        );
        contributions.add_registration(contribution, registration);

        let indexes = SymbolEnvIndexes {
            registrations,
            contributions,
            ..SymbolEnvIndexes::default()
        };
        RecoveredEnvFixture {
            env: SymbolEnv::new(module, indexes),
            registration,
        }
    }

    fn detached_registration_id() -> ResolverRegistrationId {
        let source = source_id();
        let module = module_id();
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module.clone(),
            mizar_resolve::env::ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 20, 21)),
        );
        let mut registrations = RegistrationIndex::new();
        let mut last = None;
        for ordinal in 0..4 {
            last = Some(registrations.insert(
                Some(symbol_id(
                    module.clone(),
                    &format!("DetachedReg{ordinal}"),
                    &format!("pkg::main::DetachedReg{ordinal}"),
                )),
                ResolverRegistrationKind::Cluster,
                SignatureShell::Pending,
                SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 20 + ordinal, 21 + ordinal)),
                    vec![20 + ordinal as u32],
                ),
                contribution,
            ));
        }
        last.unwrap()
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "42".repeat(32)
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
