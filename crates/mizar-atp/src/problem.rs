//! Backend-neutral ATP problem data shapes.
//!
//! This module implements the task-3 data model specified in
//! [problem.md](../../../doc/design/mizar-atp/en/problem.md). The problem layer
//! is candidate-production input only: it does not run backends, call the
//! kernel, check SAT, accept proofs, or make backend proof material trusted.

use mizar_session::Hash;
use mizar_vc::vc_ir::VcId;
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
};

macro_rules! dense_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(u32);

        impl $name {
            pub const fn new(index: u32) -> Self {
                Self(index)
            }

            pub const fn index(self) -> u32 {
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

            pub fn is_empty(&self) -> bool {
                self.0.trim().is_empty()
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

dense_id!(AtpDeclarationId);
dense_id!(AtpFormulaId);
dense_id!(AtpPropertyId);
dense_id!(AtpProvenanceId);
dense_id!(AtpTypeGuardId);

string_key!(AtpSymbolName);
string_key!(AtpSourceBinding);
string_key!(AtpDiagnosticKey);
string_key!(AtpDiagnosticMessage);
string_key!(AtpProfileName);
string_key!(AtpPayload);
string_key!(AtpRequiredProofStatus);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct AtpProblemId(Hash);

impl AtpProblemId {
    pub const fn hash(self) -> Hash {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtpFingerprint {
    algorithm_id: u8,
    digest: Vec<u8>,
}

impl AtpFingerprint {
    pub fn new(algorithm_id: u8, digest: Vec<u8>) -> Result<Self, AtpProblemError> {
        if digest.is_empty() {
            return Err(AtpProblemError::EmptyFingerprint { algorithm_id });
        }
        Ok(Self {
            algorithm_id,
            digest,
        })
    }

    pub const fn algorithm_id(&self) -> u8 {
        self.algorithm_id
    }

    pub fn digest(&self) -> &[u8] {
        &self.digest
    }

    fn render(&self) -> String {
        format!("{}:{}", self.algorithm_id, hex(&self.digest))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpTargetBinding {
    fingerprint: AtpFingerprint,
    producer_binding: AtpSourceBinding,
}

impl AtpTargetBinding {
    pub fn new(
        fingerprint: AtpFingerprint,
        producer_binding: impl Into<AtpSourceBinding>,
    ) -> Result<Self, AtpProblemError> {
        let producer_binding = producer_binding.into();
        if producer_binding.is_empty() {
            return Err(AtpProblemError::EmptyField {
                field: "target_binding.producer_binding",
            });
        }
        Ok(Self {
            fingerprint,
            producer_binding,
        })
    }

    pub const fn fingerprint(&self) -> &AtpFingerprint {
        &self.fingerprint
    }

    pub const fn producer_binding(&self) -> &AtpSourceBinding {
        &self.producer_binding
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LogicProfile {
    name: AtpProfileName,
    fragment: LogicFragment,
    equality: EqualitySupport,
    quantifiers: QuantifierPolicy,
    soft_types: SoftTypeStrategy,
    native_properties: NativePropertySupport,
    concrete_formats: BTreeSet<ConcreteFormat>,
}

impl LogicProfile {
    pub fn try_new(
        name: impl Into<AtpProfileName>,
        fragment: LogicFragment,
        equality: EqualitySupport,
        quantifiers: QuantifierPolicy,
        soft_types: SoftTypeStrategy,
        native_properties: NativePropertySupport,
        concrete_formats: BTreeSet<ConcreteFormat>,
    ) -> Result<Self, AtpProblemError> {
        let name = name.into();
        if name.is_empty() {
            return Err(AtpProblemError::InvalidLogicProfile {
                reason: "empty profile name",
            });
        }
        if concrete_formats.is_empty() {
            return Err(AtpProblemError::InvalidLogicProfile {
                reason: "no concrete encoder format",
            });
        }
        Ok(Self {
            name,
            fragment,
            equality,
            quantifiers,
            soft_types,
            native_properties,
            concrete_formats,
        })
    }

    pub const fn name(&self) -> &AtpProfileName {
        &self.name
    }

    pub const fn fragment(&self) -> LogicFragment {
        self.fragment
    }

    pub const fn equality(&self) -> EqualitySupport {
        self.equality
    }

    pub const fn quantifiers(&self) -> QuantifierPolicy {
        self.quantifiers
    }

    pub const fn soft_types(&self) -> SoftTypeStrategy {
        self.soft_types
    }

    pub const fn native_properties(&self) -> NativePropertySupport {
        self.native_properties
    }

    pub fn concrete_formats(&self) -> &BTreeSet<ConcreteFormat> {
        &self.concrete_formats
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LogicFragment {
    Fof,
    TffLike,
    SmtLibUninterpreted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum EqualitySupport {
    Unsupported,
    Supported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum QuantifierPolicy {
    PropositionalOnly,
    FirstOrder,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SoftTypeStrategy {
    BackendSorts,
    GuardPredicates,
    SortsAndGuards,
}

impl SoftTypeStrategy {
    const fn requires_type_guards(self) -> bool {
        matches!(self, Self::GuardPredicates | Self::SortsAndGuards)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NativePropertySupport {
    Unsupported,
    Supported,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ConcreteFormat {
    Tptp,
    SmtLib,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ExpectedBackendResult {
    Unsat,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpProblemParts {
    pub vc_id: VcId,
    pub target_binding: AtpTargetBinding,
    pub logic_profile: LogicProfile,
    pub expected_result: ExpectedBackendResult,
    pub declarations: Vec<AtpDeclaration>,
    pub axioms: Vec<AtpFormula>,
    pub conjecture: AtpFormula,
    pub type_context: AtpTypeContext,
    pub properties: Vec<EncodedProperty>,
    pub symbol_map: Vec<AtpSymbolMapEntry>,
    pub provenance: Vec<AtpProvenance>,
    pub diagnostics: Vec<AtpDiagnostic>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpProblem {
    problem_id: AtpProblemId,
    vc_id: VcId,
    target_binding: AtpTargetBinding,
    logic_profile: LogicProfile,
    expected_result: ExpectedBackendResult,
    declarations: Vec<AtpDeclaration>,
    axioms: Vec<AtpFormula>,
    conjecture: AtpFormula,
    type_context: AtpTypeContext,
    properties: Vec<EncodedProperty>,
    symbol_map: Vec<AtpSymbolMapEntry>,
    provenance: Vec<AtpProvenance>,
    diagnostics: Vec<AtpDiagnostic>,
}

impl AtpProblem {
    pub fn try_new(parts: AtpProblemParts) -> Result<Self, AtpProblemError> {
        let normalized = normalize_parts(parts)?;
        let canonical = render_problem_body(&normalized, RenderMode::SemanticIdentity);
        let problem_id = AtpProblemId(stable_hash(canonical.as_bytes()));
        Ok(Self {
            problem_id,
            vc_id: normalized.vc_id,
            target_binding: normalized.target_binding,
            logic_profile: normalized.logic_profile,
            expected_result: normalized.expected_result,
            declarations: normalized.declarations,
            axioms: normalized.axioms,
            conjecture: normalized.conjecture,
            type_context: normalized.type_context,
            properties: normalized.properties,
            symbol_map: normalized.symbol_map,
            provenance: normalized.provenance,
            diagnostics: normalized.diagnostics,
        })
    }

    pub const fn problem_id(&self) -> AtpProblemId {
        self.problem_id
    }

    pub const fn vc_id(&self) -> VcId {
        self.vc_id
    }

    pub const fn target_binding(&self) -> &AtpTargetBinding {
        &self.target_binding
    }

    pub const fn logic_profile(&self) -> &LogicProfile {
        &self.logic_profile
    }

    pub const fn expected_result(&self) -> ExpectedBackendResult {
        self.expected_result
    }

    pub fn declarations(&self) -> &[AtpDeclaration] {
        &self.declarations
    }

    pub fn axioms(&self) -> &[AtpFormula] {
        &self.axioms
    }

    pub const fn conjecture(&self) -> &AtpFormula {
        &self.conjecture
    }

    pub const fn type_context(&self) -> &AtpTypeContext {
        &self.type_context
    }

    pub fn properties(&self) -> &[EncodedProperty] {
        &self.properties
    }

    pub fn symbol_map(&self) -> &[AtpSymbolMapEntry] {
        &self.symbol_map
    }

    pub fn provenance(&self) -> &[AtpProvenance] {
        &self.provenance
    }

    pub fn diagnostics(&self) -> &[AtpDiagnostic] {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("atp-problem-debug-v1\n");
        writeln!(
            &mut output,
            "problem-id: {}",
            hex(self.problem_id.hash().as_bytes())
        )
        .expect("write string");
        output.push_str(&render_problem_body(self, RenderMode::Debug));
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpDeclaration {
    id: AtpDeclarationId,
    kind: AtpDeclarationKind,
    symbol: AtpSymbolName,
    arity: u32,
    provenance: AtpProvenanceId,
}

impl AtpDeclaration {
    pub fn new(
        id: AtpDeclarationId,
        kind: AtpDeclarationKind,
        symbol: impl Into<AtpSymbolName>,
        arity: u32,
        provenance: AtpProvenanceId,
    ) -> Self {
        Self {
            id,
            kind,
            symbol: symbol.into(),
            arity,
            provenance,
        }
    }

    pub const fn id(&self) -> AtpDeclarationId {
        self.id
    }

    pub const fn kind(&self) -> AtpDeclarationKind {
        self.kind
    }

    pub const fn symbol(&self) -> &AtpSymbolName {
        &self.symbol
    }

    pub const fn arity(&self) -> u32 {
        self.arity
    }

    pub const fn provenance(&self) -> AtpProvenanceId {
        self.provenance
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AtpDeclarationKind {
    Sort,
    Function,
    Predicate,
    GeneratedBinder,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpFormula {
    id: AtpFormulaId,
    formula: Option<AtpFormulaTree>,
    provenance: AtpProvenanceId,
}

impl AtpFormula {
    pub fn new(id: AtpFormulaId, formula: AtpFormulaTree, provenance: AtpProvenanceId) -> Self {
        Self {
            id,
            formula: Some(formula),
            provenance,
        }
    }

    pub const fn missing(id: AtpFormulaId, provenance: AtpProvenanceId) -> Self {
        Self {
            id,
            formula: None,
            provenance,
        }
    }

    pub const fn id(&self) -> AtpFormulaId {
        self.id
    }

    pub const fn formula(&self) -> Option<&AtpFormulaTree> {
        self.formula.as_ref()
    }

    pub const fn provenance(&self) -> AtpProvenanceId {
        self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AtpFormulaTree {
    True,
    False,
    Atom(AtpAtom),
    Equality {
        left: AtpTerm,
        right: AtpTerm,
    },
    Not(Box<AtpFormulaTree>),
    And(Vec<AtpFormulaTree>),
    Or(Vec<AtpFormulaTree>),
    Implies(Box<AtpFormulaTree>, Box<AtpFormulaTree>),
    Forall {
        binders: Vec<AtpBinder>,
        body: Box<AtpFormulaTree>,
    },
    Exists {
        binders: Vec<AtpBinder>,
        body: Box<AtpFormulaTree>,
    },
}

impl AtpFormulaTree {
    fn render(&self) -> String {
        match self {
            Self::True => "true".to_owned(),
            Self::False => "false".to_owned(),
            Self::Atom(atom) => atom.render(),
            Self::Equality { left, right } => format!("(= {} {})", left.render(), right.render()),
            Self::Not(formula) => format!("(not {})", formula.render()),
            Self::And(formulas) => render_formula_list("and", formulas),
            Self::Or(formulas) => render_formula_list("or", formulas),
            Self::Implies(left, right) => {
                format!("(implies {} {})", left.render(), right.render())
            }
            Self::Forall { binders, body } => render_quantifier("forall", binders, body),
            Self::Exists { binders, body } => render_quantifier("exists", binders, body),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtpAtom {
    predicate: AtpSymbolName,
    arguments: Vec<AtpTerm>,
}

impl AtpAtom {
    pub fn new(predicate: impl Into<AtpSymbolName>, arguments: Vec<AtpTerm>) -> Self {
        Self {
            predicate: predicate.into(),
            arguments,
        }
    }

    pub const fn predicate(&self) -> &AtpSymbolName {
        &self.predicate
    }

    pub fn arguments(&self) -> &[AtpTerm] {
        &self.arguments
    }

    fn render(&self) -> String {
        if self.arguments.is_empty() {
            return render_string(self.predicate.as_str());
        }
        let arguments = self
            .arguments
            .iter()
            .map(AtpTerm::render)
            .collect::<Vec<_>>()
            .join(" ");
        format!("({} {arguments})", render_string(self.predicate.as_str()))
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AtpTerm {
    Variable(AtpSymbolName),
    Function {
        function: AtpSymbolName,
        arguments: Vec<AtpTerm>,
    },
}

impl AtpTerm {
    fn render(&self) -> String {
        match self {
            Self::Variable(variable) => render_string(variable.as_str()),
            Self::Function {
                function,
                arguments,
            } => {
                if arguments.is_empty() {
                    render_string(function.as_str())
                } else {
                    let arguments = arguments
                        .iter()
                        .map(Self::render)
                        .collect::<Vec<_>>()
                        .join(" ");
                    format!("({} {arguments})", render_string(function.as_str()))
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtpBinder {
    variable: AtpSymbolName,
    sort: Option<AtpSymbolName>,
}

impl AtpBinder {
    pub fn new(variable: impl Into<AtpSymbolName>, sort: Option<AtpSymbolName>) -> Self {
        Self {
            variable: variable.into(),
            sort,
        }
    }

    pub const fn variable(&self) -> &AtpSymbolName {
        &self.variable
    }

    pub const fn sort(&self) -> Option<&AtpSymbolName> {
        self.sort.as_ref()
    }

    fn render(&self) -> String {
        match &self.sort {
            Some(sort) => format!(
                "{}:{}",
                render_string(self.variable.as_str()),
                render_string(sort.as_str())
            ),
            None => render_string(self.variable.as_str()),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpTypeContext {
    guards: Vec<AtpTypeGuard>,
}

impl AtpTypeContext {
    pub fn new(guards: Vec<AtpTypeGuard>) -> Self {
        Self { guards }
    }

    pub fn guards(&self) -> &[AtpTypeGuard] {
        &self.guards
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpTypeGuard {
    id: AtpTypeGuardId,
    formula: AtpFormulaTree,
    provenance: AtpProvenanceId,
}

impl AtpTypeGuard {
    pub fn new(id: AtpTypeGuardId, formula: AtpFormulaTree, provenance: AtpProvenanceId) -> Self {
        Self {
            id,
            formula,
            provenance,
        }
    }

    pub const fn id(&self) -> AtpTypeGuardId {
        self.id
    }

    pub const fn formula(&self) -> &AtpFormulaTree {
        &self.formula
    }

    pub const fn provenance(&self) -> AtpProvenanceId {
        self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct EncodedProperty {
    id: AtpPropertyId,
    target_symbol: AtpSymbolName,
    encoding: PropertyEncoding,
    provenance: AtpProvenanceId,
}

impl EncodedProperty {
    pub fn axiom(
        id: AtpPropertyId,
        target_symbol: impl Into<AtpSymbolName>,
        formula: AtpFormulaTree,
        provenance: AtpProvenanceId,
    ) -> Self {
        Self {
            id,
            target_symbol: target_symbol.into(),
            encoding: PropertyEncoding::Axiom(formula),
            provenance,
        }
    }

    pub fn native_declaration(
        id: AtpPropertyId,
        target_symbol: impl Into<AtpSymbolName>,
        declaration: AtpDeclarationId,
        provenance: AtpProvenanceId,
    ) -> Self {
        Self {
            id,
            target_symbol: target_symbol.into(),
            encoding: PropertyEncoding::NativeDeclaration(declaration),
            provenance,
        }
    }

    pub const fn id(&self) -> AtpPropertyId {
        self.id
    }

    pub const fn target_symbol(&self) -> &AtpSymbolName {
        &self.target_symbol
    }

    pub const fn encoding(&self) -> &PropertyEncoding {
        &self.encoding
    }

    pub const fn provenance(&self) -> AtpProvenanceId {
        self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum PropertyEncoding {
    Axiom(AtpFormulaTree),
    NativeDeclaration(AtpDeclarationId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpSymbolMapEntry {
    backend_symbol: AtpSymbolName,
    source: AtpSymbolSource,
}

impl AtpSymbolMapEntry {
    pub fn new(backend_symbol: impl Into<AtpSymbolName>, source: AtpSymbolSource) -> Self {
        Self {
            backend_symbol: backend_symbol.into(),
            source,
        }
    }

    pub const fn backend_symbol(&self) -> &AtpSymbolName {
        &self.backend_symbol
    }

    pub const fn source(&self) -> &AtpSymbolSource {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AtpSymbolSource {
    MizarSymbol(AtpSourceBinding),
    GeneratedBinder(AtpSourceBinding),
    TypeGuard(AtpTypeGuardId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpProvenance {
    id: AtpProvenanceId,
    source: AtpSourceRef,
    payload: AtpPayload,
}

impl AtpProvenance {
    pub fn new(id: AtpProvenanceId, source: AtpSourceRef, payload: impl Into<AtpPayload>) -> Self {
        Self {
            id,
            source,
            payload: payload.into(),
        }
    }

    pub const fn id(&self) -> AtpProvenanceId {
        self.id
    }

    pub const fn source(&self) -> &AtpSourceRef {
        &self.source
    }

    pub const fn payload(&self) -> &AtpPayload {
        &self.payload
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AtpSourceRef {
    LocalHypothesis(AtpSourceBinding),
    CitedPremise(AtpSourceBinding),
    GeneratedVcFact(AtpSourceBinding),
    ImportedAxiom {
        package: AtpSourceBinding,
        module: AtpSourceBinding,
        item: AtpSourceBinding,
        statement_fingerprint: AtpFingerprint,
        required_status: AtpRequiredProofStatus,
        context_requirement: AtpSourceBinding,
    },
    ImportedTheorem {
        package: AtpSourceBinding,
        module: AtpSourceBinding,
        item: AtpSourceBinding,
        statement_fingerprint: AtpFingerprint,
        required_status: AtpRequiredProofStatus,
        context_requirement: AtpSourceBinding,
    },
    CheckerOwnedFact(AtpSourceBinding),
    TypeFact(AtpSourceBinding),
    EncodedProperty(AtpSourceBinding),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpDiagnostic {
    key: AtpDiagnosticKey,
    message: AtpDiagnosticMessage,
}

impl AtpDiagnostic {
    pub fn new(key: impl Into<AtpDiagnosticKey>, message: impl Into<AtpDiagnosticMessage>) -> Self {
        Self {
            key: key.into(),
            message: message.into(),
        }
    }

    pub const fn key(&self) -> &AtpDiagnosticKey {
        &self.key
    }

    pub const fn message(&self) -> &AtpDiagnosticMessage {
        &self.message
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AtpProblemError {
    EmptyFingerprint {
        algorithm_id: u8,
    },
    EmptyField {
        field: &'static str,
    },
    InvalidLogicProfile {
        reason: &'static str,
    },
    MissingFormulaPayload {
        formula_id: AtpFormulaId,
    },
    MissingProvenance {
        owner: &'static str,
        provenance_id: AtpProvenanceId,
    },
    MissingSymbolMap {
        symbol: AtpSymbolName,
    },
    MissingTypeContextBinding {
        strategy: SoftTypeStrategy,
    },
    DuplicateId {
        section: &'static str,
        id: u32,
    },
    DuplicateSymbolMap {
        symbol: AtpSymbolName,
    },
    DuplicateDeclarationSymbol {
        symbol: AtpSymbolName,
    },
    MissingDeclaration {
        declaration: AtpDeclarationId,
    },
    MissingDeclarationSymbol {
        symbol: AtpSymbolName,
    },
    InvalidSymbolDeclaration {
        symbol: AtpSymbolName,
        expected: &'static str,
        actual: AtpDeclarationKind,
    },
    InvalidSymbolArity {
        symbol: AtpSymbolName,
        expected: u32,
        actual: u32,
    },
    MissingTypeGuard {
        type_guard: AtpTypeGuardId,
    },
    UnsupportedProfileFeature {
        feature: &'static str,
    },
}

impl fmt::Display for AtpProblemError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyFingerprint { algorithm_id } => {
                write!(
                    formatter,
                    "empty fingerprint digest for algorithm {algorithm_id}"
                )
            }
            Self::EmptyField { field } => write!(formatter, "empty required field {field}"),
            Self::InvalidLogicProfile { reason } => {
                write!(formatter, "invalid logic profile: {reason}")
            }
            Self::MissingFormulaPayload { formula_id } => {
                write!(formatter, "missing formula payload for {formula_id:?}")
            }
            Self::MissingProvenance {
                owner,
                provenance_id,
            } => write!(
                formatter,
                "missing provenance {provenance_id:?} for {owner}"
            ),
            Self::MissingSymbolMap { symbol } => {
                write!(formatter, "missing symbol-map row for {}", symbol.as_str())
            }
            Self::MissingTypeContextBinding { strategy } => {
                write!(formatter, "missing type-context binding for {strategy:?}")
            }
            Self::DuplicateId { section, id } => {
                write!(formatter, "duplicate id {id} in section {section}")
            }
            Self::DuplicateSymbolMap { symbol } => {
                write!(
                    formatter,
                    "duplicate symbol-map row for {}",
                    symbol.as_str()
                )
            }
            Self::DuplicateDeclarationSymbol { symbol } => {
                write!(
                    formatter,
                    "duplicate declaration for symbol {}",
                    symbol.as_str()
                )
            }
            Self::MissingDeclaration { declaration } => {
                write!(formatter, "missing declaration {declaration:?}")
            }
            Self::MissingDeclarationSymbol { symbol } => {
                write!(
                    formatter,
                    "missing declaration for symbol {}",
                    symbol.as_str()
                )
            }
            Self::InvalidSymbolDeclaration {
                symbol,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "invalid declaration for symbol {}: expected {expected}, got {actual:?}",
                    symbol.as_str()
                )
            }
            Self::InvalidSymbolArity {
                symbol,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "invalid arity for symbol {}: expected {expected}, got {actual}",
                    symbol.as_str()
                )
            }
            Self::MissingTypeGuard { type_guard } => {
                write!(formatter, "missing type guard {type_guard:?}")
            }
            Self::UnsupportedProfileFeature { feature } => {
                write!(formatter, "unsupported profile feature {feature}")
            }
        }
    }
}

impl Error for AtpProblemError {}

fn normalize_parts(mut parts: AtpProblemParts) -> Result<AtpProblemParts, AtpProblemError> {
    if parts.expected_result != ExpectedBackendResult::Unsat {
        return Err(AtpProblemError::UnsupportedProfileFeature {
            feature: "non-unsat expected result",
        });
    }

    sort_by_id(
        &mut parts.provenance,
        |entry| entry.id().index(),
        "provenance",
    )?;
    sort_symbol_map(&mut parts.symbol_map)?;
    sort_by_id(
        &mut parts.declarations,
        |entry| entry.id().index(),
        "declarations",
    )?;
    sort_by_id(&mut parts.axioms, |entry| entry.id().index(), "axioms")?;
    sort_by_id(
        &mut parts.type_context.guards,
        |entry| entry.id().index(),
        "type-guards",
    )?;
    sort_by_id(
        &mut parts.properties,
        |entry| entry.id().index(),
        "properties",
    )?;
    parts.diagnostics.sort_by(|left, right| {
        left.key
            .cmp(&right.key)
            .then_with(|| left.message.cmp(&right.message))
    });

    let provenance_ids = parts
        .provenance
        .iter()
        .map(AtpProvenance::id)
        .collect::<BTreeSet<_>>();
    let type_guard_ids = parts
        .type_context
        .guards
        .iter()
        .map(AtpTypeGuard::id)
        .collect::<BTreeSet<_>>();
    let symbol_map = parts
        .symbol_map
        .iter()
        .map(|entry| entry.backend_symbol.clone())
        .collect::<BTreeSet<_>>();
    let declaration_ids = parts
        .declarations
        .iter()
        .map(AtpDeclaration::id)
        .collect::<BTreeSet<_>>();
    validate_provenance(&parts.provenance)?;
    validate_symbol_map(&parts.symbol_map, &type_guard_ids)?;
    validate_declarations(&parts.declarations, &provenance_ids, &symbol_map)?;
    let declaration_map = declaration_map(&parts.declarations);
    validate_formula_id_namespaces(&parts.axioms, parts.conjecture.id())?;
    validate_formulas(
        &parts.axioms,
        "axiom",
        &parts.logic_profile,
        &provenance_ids,
        &symbol_map,
        &declaration_map,
    )?;
    validate_formula(
        &parts.conjecture,
        "conjecture",
        &parts.logic_profile,
        &provenance_ids,
        &symbol_map,
        &declaration_map,
    )?;
    validate_type_context(
        &parts.type_context,
        &parts.logic_profile,
        &provenance_ids,
        &symbol_map,
        &declaration_map,
    )?;
    validate_properties(
        &parts.properties,
        &parts.logic_profile,
        &provenance_ids,
        &symbol_map,
        &declaration_map,
        &declaration_ids,
    )?;

    Ok(parts)
}

fn sort_by_id<T, F>(items: &mut [T], id: F, section: &'static str) -> Result<(), AtpProblemError>
where
    F: Fn(&T) -> u32,
{
    items.sort_by_key(&id);
    for pair in items.windows(2) {
        if id(&pair[0]) == id(&pair[1]) {
            return Err(AtpProblemError::DuplicateId {
                section,
                id: id(&pair[0]),
            });
        }
    }
    Ok(())
}

fn sort_symbol_map(entries: &mut [AtpSymbolMapEntry]) -> Result<(), AtpProblemError> {
    entries.sort_by(|left, right| left.backend_symbol.cmp(&right.backend_symbol));
    for pair in entries.windows(2) {
        if pair[0].backend_symbol == pair[1].backend_symbol {
            return Err(AtpProblemError::DuplicateSymbolMap {
                symbol: pair[0].backend_symbol.clone(),
            });
        }
    }
    Ok(())
}

fn validate_provenance(provenance: &[AtpProvenance]) -> Result<(), AtpProblemError> {
    for entry in provenance {
        if entry.payload.is_empty() {
            return Err(AtpProblemError::EmptyField {
                field: "provenance.payload",
            });
        }
        validate_source_ref(entry.source())?;
    }
    Ok(())
}

fn validate_symbol_map(
    entries: &[AtpSymbolMapEntry],
    type_guards: &BTreeSet<AtpTypeGuardId>,
) -> Result<(), AtpProblemError> {
    for entry in entries {
        if entry.backend_symbol.is_empty() {
            return Err(AtpProblemError::EmptyField {
                field: "symbol_map.backend_symbol",
            });
        }
        validate_symbol_source(entry.source(), type_guards)?;
    }
    Ok(())
}

fn validate_source_ref(source: &AtpSourceRef) -> Result<(), AtpProblemError> {
    match source {
        AtpSourceRef::LocalHypothesis(binding)
        | AtpSourceRef::CitedPremise(binding)
        | AtpSourceRef::GeneratedVcFact(binding)
        | AtpSourceRef::CheckerOwnedFact(binding)
        | AtpSourceRef::TypeFact(binding)
        | AtpSourceRef::EncodedProperty(binding) => {
            require_nonempty_binding(binding, "provenance.source")
        }
        AtpSourceRef::ImportedAxiom {
            package,
            module,
            item,
            required_status,
            context_requirement,
            ..
        }
        | AtpSourceRef::ImportedTheorem {
            package,
            module,
            item,
            required_status,
            context_requirement,
            ..
        } => {
            require_nonempty_binding(package, "imported.package")?;
            require_nonempty_binding(module, "imported.module")?;
            require_nonempty_binding(item, "imported.item")?;
            require_nonempty_binding(required_status, "imported.required_status")?;
            require_nonempty_binding(context_requirement, "imported.context_requirement")
        }
    }
}

fn validate_symbol_source(
    source: &AtpSymbolSource,
    type_guards: &BTreeSet<AtpTypeGuardId>,
) -> Result<(), AtpProblemError> {
    match source {
        AtpSymbolSource::MizarSymbol(binding) | AtpSymbolSource::GeneratedBinder(binding) => {
            require_nonempty_binding(binding, "symbol_map.source")
        }
        AtpSymbolSource::TypeGuard(id) => {
            if type_guards.contains(id) {
                Ok(())
            } else {
                Err(AtpProblemError::MissingTypeGuard { type_guard: *id })
            }
        }
    }
}

fn require_nonempty_binding(
    binding: &impl BindingLike,
    field: &'static str,
) -> Result<(), AtpProblemError> {
    if binding.is_empty() {
        Err(AtpProblemError::EmptyField { field })
    } else {
        Ok(())
    }
}

trait BindingLike {
    fn is_empty(&self) -> bool;
}

impl BindingLike for AtpSourceBinding {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

impl BindingLike for AtpRequiredProofStatus {
    fn is_empty(&self) -> bool {
        self.is_empty()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeclarationSignature {
    kind: AtpDeclarationKind,
    arity: u32,
}

fn declaration_map(
    declarations: &[AtpDeclaration],
) -> BTreeMap<AtpSymbolName, DeclarationSignature> {
    declarations
        .iter()
        .map(|declaration| {
            (
                declaration.symbol.clone(),
                DeclarationSignature {
                    kind: declaration.kind,
                    arity: declaration.arity,
                },
            )
        })
        .collect()
}

fn validate_formula_id_namespaces(
    axioms: &[AtpFormula],
    conjecture_id: AtpFormulaId,
) -> Result<(), AtpProblemError> {
    if axioms.iter().any(|formula| formula.id() == conjecture_id) {
        Err(AtpProblemError::DuplicateId {
            section: "formulas",
            id: conjecture_id.index(),
        })
    } else {
        Ok(())
    }
}

fn validate_declarations(
    declarations: &[AtpDeclaration],
    provenance: &BTreeSet<AtpProvenanceId>,
    symbol_map: &BTreeSet<AtpSymbolName>,
) -> Result<(), AtpProblemError> {
    let mut seen_symbols = BTreeSet::new();
    for declaration in declarations {
        if declaration.symbol.is_empty() {
            return Err(AtpProblemError::EmptyField {
                field: "declaration.symbol",
            });
        }
        if !seen_symbols.insert(declaration.symbol.clone()) {
            return Err(AtpProblemError::DuplicateDeclarationSymbol {
                symbol: declaration.symbol.clone(),
            });
        }
        require_provenance("declaration", declaration.provenance, provenance)?;
        require_symbol(&declaration.symbol, symbol_map)?;
    }
    Ok(())
}

fn validate_formulas(
    formulas: &[AtpFormula],
    owner: &'static str,
    profile: &LogicProfile,
    provenance: &BTreeSet<AtpProvenanceId>,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<(), AtpProblemError> {
    for formula in formulas {
        validate_formula(
            formula,
            owner,
            profile,
            provenance,
            symbol_map,
            declarations,
        )?;
    }
    Ok(())
}

fn validate_formula(
    formula: &AtpFormula,
    owner: &'static str,
    profile: &LogicProfile,
    provenance: &BTreeSet<AtpProvenanceId>,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<(), AtpProblemError> {
    let Some(tree) = formula.formula() else {
        return Err(AtpProblemError::MissingFormulaPayload {
            formula_id: formula.id(),
        });
    };
    require_provenance(owner, formula.provenance(), provenance)?;
    validate_formula_tree(tree, profile, symbol_map, declarations)
}

fn validate_type_context(
    context: &AtpTypeContext,
    profile: &LogicProfile,
    provenance: &BTreeSet<AtpProvenanceId>,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<(), AtpProblemError> {
    if profile.soft_types().requires_type_guards() && context.guards.is_empty() {
        return Err(AtpProblemError::MissingTypeContextBinding {
            strategy: profile.soft_types(),
        });
    }
    for guard in &context.guards {
        require_provenance("type-guard", guard.provenance(), provenance)?;
        validate_formula_tree(guard.formula(), profile, symbol_map, declarations)?;
    }
    Ok(())
}

fn validate_properties(
    properties: &[EncodedProperty],
    profile: &LogicProfile,
    provenance: &BTreeSet<AtpProvenanceId>,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declaration_map: &BTreeMap<AtpSymbolName, DeclarationSignature>,
    declaration_ids: &BTreeSet<AtpDeclarationId>,
) -> Result<(), AtpProblemError> {
    for property in properties {
        require_provenance("property", property.provenance(), provenance)?;
        require_declared_symbol(property.target_symbol(), symbol_map, declaration_map)?;
        match property.encoding() {
            PropertyEncoding::Axiom(formula) => {
                validate_formula_tree(formula, profile, symbol_map, declaration_map)?;
            }
            PropertyEncoding::NativeDeclaration(declaration) => {
                if profile.native_properties() != NativePropertySupport::Supported {
                    return Err(AtpProblemError::UnsupportedProfileFeature {
                        feature: "native property declaration",
                    });
                }
                if !declaration_ids.contains(declaration) {
                    return Err(AtpProblemError::MissingDeclaration {
                        declaration: *declaration,
                    });
                }
            }
        }
    }
    Ok(())
}

fn require_provenance(
    owner: &'static str,
    provenance_id: AtpProvenanceId,
    provenance: &BTreeSet<AtpProvenanceId>,
) -> Result<(), AtpProblemError> {
    if provenance.contains(&provenance_id) {
        Ok(())
    } else {
        Err(AtpProblemError::MissingProvenance {
            owner,
            provenance_id,
        })
    }
}

fn require_symbol(
    symbol: &AtpSymbolName,
    symbol_map: &BTreeSet<AtpSymbolName>,
) -> Result<(), AtpProblemError> {
    if symbol_map.contains(symbol) {
        Ok(())
    } else {
        Err(AtpProblemError::MissingSymbolMap {
            symbol: symbol.clone(),
        })
    }
}

fn require_declared_symbol(
    symbol: &AtpSymbolName,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<DeclarationSignature, AtpProblemError> {
    require_symbol(symbol, symbol_map)?;
    declarations
        .get(symbol)
        .copied()
        .ok_or_else(|| AtpProblemError::MissingDeclarationSymbol {
            symbol: symbol.clone(),
        })
}

fn require_symbol_signature(
    symbol: &AtpSymbolName,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
    expected_kind: AtpDeclarationKind,
    expected_arity: u32,
    expected: &'static str,
) -> Result<(), AtpProblemError> {
    let actual = require_declared_symbol(symbol, symbol_map, declarations)?;
    if actual.kind != expected_kind {
        return Err(AtpProblemError::InvalidSymbolDeclaration {
            symbol: symbol.clone(),
            expected,
            actual: actual.kind,
        });
    }
    if actual.arity != expected_arity {
        return Err(AtpProblemError::InvalidSymbolArity {
            symbol: symbol.clone(),
            expected: expected_arity,
            actual: actual.arity,
        });
    }
    Ok(())
}

fn validate_formula_tree(
    formula: &AtpFormulaTree,
    profile: &LogicProfile,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<(), AtpProblemError> {
    match formula {
        AtpFormulaTree::True | AtpFormulaTree::False => Ok(()),
        AtpFormulaTree::Atom(atom) => {
            require_symbol_signature(
                atom.predicate(),
                symbol_map,
                declarations,
                AtpDeclarationKind::Predicate,
                atom.arguments().len() as u32,
                "predicate",
            )?;
            for argument in atom.arguments() {
                validate_term(argument, symbol_map, declarations)?;
            }
            Ok(())
        }
        AtpFormulaTree::Equality { left, right } => {
            if profile.equality() != EqualitySupport::Supported {
                return Err(AtpProblemError::UnsupportedProfileFeature {
                    feature: "equality",
                });
            }
            validate_term(left, symbol_map, declarations)?;
            validate_term(right, symbol_map, declarations)
        }
        AtpFormulaTree::Not(inner) => {
            validate_formula_tree(inner, profile, symbol_map, declarations)
        }
        AtpFormulaTree::And(formulas) | AtpFormulaTree::Or(formulas) => {
            if formulas.is_empty() {
                return Err(AtpProblemError::MissingFormulaPayload {
                    formula_id: AtpFormulaId::new(0),
                });
            }
            for inner in formulas {
                validate_formula_tree(inner, profile, symbol_map, declarations)?;
            }
            Ok(())
        }
        AtpFormulaTree::Implies(left, right) => {
            validate_formula_tree(left, profile, symbol_map, declarations)?;
            validate_formula_tree(right, profile, symbol_map, declarations)
        }
        AtpFormulaTree::Forall { binders, body } | AtpFormulaTree::Exists { binders, body } => {
            if profile.quantifiers() != QuantifierPolicy::FirstOrder {
                return Err(AtpProblemError::UnsupportedProfileFeature {
                    feature: "quantifier",
                });
            }
            if binders.is_empty() {
                return Err(AtpProblemError::MissingFormulaPayload {
                    formula_id: AtpFormulaId::new(0),
                });
            }
            for binder in binders {
                require_symbol_signature(
                    binder.variable(),
                    symbol_map,
                    declarations,
                    AtpDeclarationKind::GeneratedBinder,
                    0,
                    "generated binder",
                )?;
                if let Some(sort) = binder.sort() {
                    require_symbol_signature(
                        sort,
                        symbol_map,
                        declarations,
                        AtpDeclarationKind::Sort,
                        0,
                        "sort",
                    )?;
                }
            }
            validate_formula_tree(body, profile, symbol_map, declarations)
        }
    }
}

fn validate_term(
    term: &AtpTerm,
    symbol_map: &BTreeSet<AtpSymbolName>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
) -> Result<(), AtpProblemError> {
    match term {
        AtpTerm::Variable(variable) => require_symbol_signature(
            variable,
            symbol_map,
            declarations,
            AtpDeclarationKind::GeneratedBinder,
            0,
            "generated binder",
        ),
        AtpTerm::Function {
            function,
            arguments,
        } => {
            require_symbol_signature(
                function,
                symbol_map,
                declarations,
                AtpDeclarationKind::Function,
                arguments.len() as u32,
                "function",
            )?;
            for argument in arguments {
                validate_term(argument, symbol_map, declarations)?;
            }
            Ok(())
        }
    }
}

enum RenderMode {
    SemanticIdentity,
    Debug,
}

trait ProblemView {
    fn vc_id(&self) -> VcId;
    fn target_binding(&self) -> &AtpTargetBinding;
    fn logic_profile(&self) -> &LogicProfile;
    fn expected_result(&self) -> ExpectedBackendResult;
    fn declarations(&self) -> &[AtpDeclaration];
    fn axioms(&self) -> &[AtpFormula];
    fn conjecture(&self) -> &AtpFormula;
    fn type_context(&self) -> &AtpTypeContext;
    fn properties(&self) -> &[EncodedProperty];
    fn symbol_map(&self) -> &[AtpSymbolMapEntry];
    fn provenance(&self) -> &[AtpProvenance];
    fn diagnostics(&self) -> &[AtpDiagnostic];
}

impl ProblemView for AtpProblem {
    fn vc_id(&self) -> VcId {
        self.vc_id
    }

    fn target_binding(&self) -> &AtpTargetBinding {
        &self.target_binding
    }

    fn logic_profile(&self) -> &LogicProfile {
        &self.logic_profile
    }

    fn expected_result(&self) -> ExpectedBackendResult {
        self.expected_result
    }

    fn declarations(&self) -> &[AtpDeclaration] {
        &self.declarations
    }

    fn axioms(&self) -> &[AtpFormula] {
        &self.axioms
    }

    fn conjecture(&self) -> &AtpFormula {
        &self.conjecture
    }

    fn type_context(&self) -> &AtpTypeContext {
        &self.type_context
    }

    fn properties(&self) -> &[EncodedProperty] {
        &self.properties
    }

    fn symbol_map(&self) -> &[AtpSymbolMapEntry] {
        &self.symbol_map
    }

    fn provenance(&self) -> &[AtpProvenance] {
        &self.provenance
    }

    fn diagnostics(&self) -> &[AtpDiagnostic] {
        &self.diagnostics
    }
}

impl ProblemView for AtpProblemParts {
    fn vc_id(&self) -> VcId {
        self.vc_id
    }

    fn target_binding(&self) -> &AtpTargetBinding {
        &self.target_binding
    }

    fn logic_profile(&self) -> &LogicProfile {
        &self.logic_profile
    }

    fn expected_result(&self) -> ExpectedBackendResult {
        self.expected_result
    }

    fn declarations(&self) -> &[AtpDeclaration] {
        &self.declarations
    }

    fn axioms(&self) -> &[AtpFormula] {
        &self.axioms
    }

    fn conjecture(&self) -> &AtpFormula {
        &self.conjecture
    }

    fn type_context(&self) -> &AtpTypeContext {
        &self.type_context
    }

    fn properties(&self) -> &[EncodedProperty] {
        &self.properties
    }

    fn symbol_map(&self) -> &[AtpSymbolMapEntry] {
        &self.symbol_map
    }

    fn provenance(&self) -> &[AtpProvenance] {
        &self.provenance
    }

    fn diagnostics(&self) -> &[AtpDiagnostic] {
        &self.diagnostics
    }
}

fn render_problem_body(problem: &impl ProblemView, mode: RenderMode) -> String {
    let mut output = String::new();
    writeln!(&mut output, "vc-id: {:?}", problem.vc_id()).expect("write string");
    writeln!(
        &mut output,
        "target: {}; producer={}",
        problem.target_binding().fingerprint().render(),
        render_string(problem.target_binding().producer_binding().as_str())
    )
    .expect("write string");
    write_logic_profile(&mut output, problem.logic_profile());
    writeln!(
        &mut output,
        "expected-result: {:?}",
        problem.expected_result()
    )
    .expect("write string");
    write_declarations(&mut output, problem.declarations());
    write_formulas(&mut output, "axioms", problem.axioms());
    write_formula(&mut output, "conjecture", problem.conjecture());
    write_type_context(&mut output, problem.type_context());
    write_properties(&mut output, problem.properties());
    write_symbol_map(&mut output, problem.symbol_map());
    write_provenance(&mut output, problem.provenance());
    if matches!(mode, RenderMode::Debug) {
        write_diagnostics(&mut output, problem.diagnostics());
    }
    output
}

fn write_logic_profile(output: &mut String, profile: &LogicProfile) {
    writeln!(
        output,
        "logic-profile: name={}; fragment={:?}; equality={:?}; quantifiers={:?}; soft-types={:?}; native-properties={:?}; formats={:?}",
        render_string(profile.name().as_str()),
        profile.fragment(),
        profile.equality(),
        profile.quantifiers(),
        profile.soft_types(),
        profile.native_properties(),
        profile.concrete_formats()
    )
    .expect("write string");
}

fn write_declarations(output: &mut String, declarations: &[AtpDeclaration]) {
    writeln!(output, "[declarations]").expect("write string");
    for declaration in declarations {
        writeln!(
            output,
            "{}: kind={:?}; symbol={}; arity={}; provenance={}",
            declaration.id().index(),
            declaration.kind(),
            render_string(declaration.symbol().as_str()),
            declaration.arity(),
            declaration.provenance().index()
        )
        .expect("write string");
    }
}

fn write_formulas(output: &mut String, section: &str, formulas: &[AtpFormula]) {
    writeln!(output, "[{section}]").expect("write string");
    for formula in formulas {
        write_formula(output, "formula", formula);
    }
}

fn write_formula(output: &mut String, label: &str, formula: &AtpFormula) {
    let payload = formula
        .formula()
        .map_or_else(|| "<missing>".to_owned(), AtpFormulaTree::render);
    writeln!(
        output,
        "{label} {}: provenance={}; formula={payload}",
        formula.id().index(),
        formula.provenance().index()
    )
    .expect("write string");
}

fn write_type_context(output: &mut String, context: &AtpTypeContext) {
    writeln!(output, "[type-context]").expect("write string");
    for guard in context.guards() {
        writeln!(
            output,
            "{}: provenance={}; formula={}",
            guard.id().index(),
            guard.provenance().index(),
            guard.formula().render()
        )
        .expect("write string");
    }
}

fn write_properties(output: &mut String, properties: &[EncodedProperty]) {
    writeln!(output, "[properties]").expect("write string");
    for property in properties {
        let encoding = match property.encoding() {
            PropertyEncoding::Axiom(formula) => format!("axiom:{}", formula.render()),
            PropertyEncoding::NativeDeclaration(declaration) => {
                format!("native-declaration:{}", declaration.index())
            }
        };
        writeln!(
            output,
            "{}: target={}; provenance={}; encoding={encoding}",
            property.id().index(),
            render_string(property.target_symbol().as_str()),
            property.provenance().index()
        )
        .expect("write string");
    }
}

fn write_symbol_map(output: &mut String, entries: &[AtpSymbolMapEntry]) {
    writeln!(output, "[symbol-map]").expect("write string");
    for entry in entries {
        writeln!(
            output,
            "{}: {}",
            render_string(entry.backend_symbol().as_str()),
            render_symbol_source(entry.source())
        )
        .expect("write string");
    }
}

fn write_provenance(output: &mut String, provenance: &[AtpProvenance]) {
    writeln!(output, "[provenance]").expect("write string");
    for entry in provenance {
        writeln!(
            output,
            "{}: source={}; payload={}",
            entry.id().index(),
            render_source_ref(entry.source()),
            render_string(entry.payload().as_str())
        )
        .expect("write string");
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &[AtpDiagnostic]) {
    writeln!(output, "[diagnostics:non-semantic]").expect("write string");
    for diagnostic in diagnostics {
        writeln!(
            output,
            "{}: {}",
            render_string(diagnostic.key().as_str()),
            render_string(diagnostic.message().as_str())
        )
        .expect("write string");
    }
}

fn render_formula_list(operator: &str, formulas: &[AtpFormulaTree]) -> String {
    let body = formulas
        .iter()
        .map(AtpFormulaTree::render)
        .collect::<Vec<_>>()
        .join(" ");
    format!("({operator} {body})")
}

fn render_quantifier(operator: &str, binders: &[AtpBinder], body: &AtpFormulaTree) -> String {
    let binders = binders
        .iter()
        .map(AtpBinder::render)
        .collect::<Vec<_>>()
        .join(" ");
    format!("({operator} ({binders}) {})", body.render())
}

fn render_symbol_source(source: &AtpSymbolSource) -> String {
    match source {
        AtpSymbolSource::MizarSymbol(binding) => {
            format!("mizar:{}", render_string(binding.as_str()))
        }
        AtpSymbolSource::GeneratedBinder(binding) => {
            format!("generated-binder:{}", render_string(binding.as_str()))
        }
        AtpSymbolSource::TypeGuard(id) => format!("type-guard:{}", id.index()),
    }
}

fn render_source_ref(source: &AtpSourceRef) -> String {
    match source {
        AtpSourceRef::LocalHypothesis(binding) => {
            format!("local-hypothesis:{}", render_string(binding.as_str()))
        }
        AtpSourceRef::CitedPremise(binding) => {
            format!("cited-premise:{}", render_string(binding.as_str()))
        }
        AtpSourceRef::GeneratedVcFact(binding) => {
            format!("generated-vc-fact:{}", render_string(binding.as_str()))
        }
        AtpSourceRef::ImportedAxiom {
            package,
            module,
            item,
            statement_fingerprint,
            required_status,
            context_requirement,
        } => format!(
            "imported-axiom:package={};module={};item={};statement={};required-status={};context={}",
            render_string(package.as_str()),
            render_string(module.as_str()),
            render_string(item.as_str()),
            statement_fingerprint.render(),
            render_string(required_status.as_str()),
            render_string(context_requirement.as_str())
        ),
        AtpSourceRef::ImportedTheorem {
            package,
            module,
            item,
            statement_fingerprint,
            required_status,
            context_requirement,
        } => format!(
            "imported-theorem:package={};module={};item={};statement={};required-status={};context={}",
            render_string(package.as_str()),
            render_string(module.as_str()),
            render_string(item.as_str()),
            statement_fingerprint.render(),
            render_string(required_status.as_str()),
            render_string(context_requirement.as_str())
        ),
        AtpSourceRef::CheckerOwnedFact(binding) => {
            format!("checker-owned:{}", render_string(binding.as_str()))
        }
        AtpSourceRef::TypeFact(binding) => format!("type-fact:{}", render_string(binding.as_str())),
        AtpSourceRef::EncodedProperty(binding) => {
            format!("encoded-property:{}", render_string(binding.as_str()))
        }
    }
}

fn render_string(value: &str) -> String {
    format!("{}:{}", value.len(), hex(value.as_bytes()))
}

fn stable_hash(bytes: &[u8]) -> Hash {
    let mut lanes = [
        0x6d_69_7a_61_72_2d_61_74_u64,
        0x70_2d_70_72_6f_62_6c_65_u64,
        0x6d_2d_69_64_2d_76_31_u64,
        0x63_61_6e_6f_6e_69_63_u64,
    ];

    for (index, byte) in bytes.iter().copied().enumerate() {
        let lane = index % lanes.len();
        let mixed_index = (index as u64).rotate_left((lane as u32) + 3);
        lanes[lane] ^= u64::from(byte)
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(mixed_index);
        lanes[lane] = lanes[lane]
            .rotate_left(13 + lane as u32)
            .wrapping_mul(0x1000_0000_01b3);
    }

    lanes[0] ^= bytes.len() as u64;
    lanes[1] ^= (bytes.len() as u64).rotate_left(19);
    lanes[2] ^= lanes[0].rotate_left(7);
    lanes[3] ^= lanes[1].rotate_left(11);

    let mut output = [0_u8; Hash::BYTE_LEN];
    for (chunk, lane) in output.chunks_exact_mut(8).zip(lanes) {
        chunk.copy_from_slice(&lane.to_be_bytes());
    }
    Hash::from_bytes(output)
}

fn hex(bytes: &[u8]) -> String {
    let mut encoded = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(&mut encoded, "{byte:02x}").expect("write string");
    }
    encoded
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn constructs_minimal_problem_and_renders_unsat_contract() {
        let problem = minimal_problem();

        assert_eq!(problem.vc_id(), VcId::new(7));
        assert_eq!(problem.expected_result(), ExpectedBackendResult::Unsat);
        assert_eq!(problem.declarations().len(), 2);
        assert_eq!(problem.axioms().len(), 1);
        assert!(problem.debug_text().contains("expected-result: Unsat"));
        assert!(problem.debug_text().contains("[diagnostics:non-semantic]"));
    }

    #[test]
    fn deterministic_identity_sorts_shuffled_inputs_and_excludes_diagnostics() {
        let first = populated_problem(false);
        let second = populated_problem(true);
        let different_diagnostic =
            AtpProblem::try_new(populated_parts_with_diagnostic(false, "z-note"))
                .expect("diagnostic variant");

        assert_eq!(first.problem_id(), second.problem_id());
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.problem_id(), different_diagnostic.problem_id());
        assert_ne!(first.debug_text(), different_diagnostic.debug_text());
        assert_eq!(
            first
                .declarations()
                .iter()
                .map(AtpDeclaration::id)
                .collect::<Vec<_>>(),
            vec![AtpDeclarationId::new(1), AtpDeclarationId::new(2)]
        );
        assert_eq!(
            first
                .symbol_map()
                .iter()
                .map(|entry| entry.backend_symbol().as_str())
                .collect::<Vec<_>>(),
            vec!["P", "a", "guard1", "x"]
        );
        assert_eq!(
            first
                .axioms()
                .iter()
                .map(AtpFormula::id)
                .collect::<Vec<_>>(),
            vec![AtpFormulaId::new(1), AtpFormulaId::new(3)]
        );
        assert_eq!(
            first
                .type_context()
                .guards()
                .iter()
                .map(AtpTypeGuard::id)
                .collect::<Vec<_>>(),
            vec![AtpTypeGuardId::new(1), AtpTypeGuardId::new(2)]
        );
        assert_eq!(
            first
                .properties()
                .iter()
                .map(EncodedProperty::id)
                .collect::<Vec<_>>(),
            vec![AtpPropertyId::new(1), AtpPropertyId::new(2)]
        );
        assert!(first.debug_text().contains("diagnostics:non-semantic"));
    }

    #[test]
    fn rejects_missing_required_inputs_fail_closed() {
        let mut parts = minimal_parts();
        parts.declarations[0] = AtpDeclaration::new(
            AtpDeclarationId::new(1),
            AtpDeclarationKind::Predicate,
            "P",
            1,
            AtpProvenanceId::new(99),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingProvenance {
                owner: "declaration",
                provenance_id: AtpProvenanceId::new(99)
            }
        );

        assert_eq!(
            AtpFingerprint::new(18, Vec::new()).unwrap_err(),
            AtpProblemError::EmptyFingerprint { algorithm_id: 18 }
        );
        assert_eq!(
            AtpTargetBinding::new(
                AtpFingerprint::new(18, b"target".to_vec()).expect("fingerprint"),
                ""
            )
            .unwrap_err(),
            AtpProblemError::EmptyField {
                field: "target_binding.producer_binding"
            }
        );

        let mut parts = minimal_parts();
        parts.symbol_map.clear();
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingSymbolMap {
                symbol: AtpSymbolName::new("P")
            }
        );

        let mut parts = minimal_parts();
        parts.logic_profile = profile_with_soft_types(SoftTypeStrategy::GuardPredicates);
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingTypeContextBinding {
                strategy: SoftTypeStrategy::GuardPredicates
            }
        );
    }

    #[test]
    fn symbol_references_fail_closed_for_each_problem_section() {
        let mut parts = minimal_parts();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            atom("Q", vec![constant("a")]),
            AtpProvenanceId::new(2),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingSymbolMap {
                symbol: AtpSymbolName::new("Q")
            }
        );

        let mut parts = minimal_parts();
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "Q",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:Q")),
        ));
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            atom("Q", vec![constant("a")]),
            AtpProvenanceId::new(2),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingDeclarationSymbol {
                symbol: AtpSymbolName::new("Q")
            }
        );

        let mut parts = minimal_parts();
        parts
            .symbol_map
            .retain(|entry| entry.backend_symbol().as_str() != "a");
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingSymbolMap {
                symbol: AtpSymbolName::new("a")
            }
        );

        let mut parts = minimal_parts();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            atom(
                "P",
                vec![AtpTerm::Function {
                    function: AtpSymbolName::new("f"),
                    arguments: vec![constant("a")],
                }],
            ),
            AtpProvenanceId::new(2),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingSymbolMap {
                symbol: AtpSymbolName::new("f")
            }
        );

        let mut parts = minimal_parts();
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "f",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("fun:f")),
        ));
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            atom(
                "P",
                vec![AtpTerm::Function {
                    function: AtpSymbolName::new("f"),
                    arguments: vec![constant("a")],
                }],
            ),
            AtpProvenanceId::new(2),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingDeclarationSymbol {
                symbol: AtpSymbolName::new("f")
            }
        );

        let mut parts = minimal_parts();
        parts.logic_profile = profile_first_order();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", None)],
                body: Box::new(AtpFormulaTree::True),
            },
            AtpProvenanceId::new(2),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingSymbolMap {
                symbol: AtpSymbolName::new("x")
            }
        );

        let mut parts = minimal_parts();
        parts.logic_profile = profile_first_order();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", None)],
                body: Box::new(AtpFormulaTree::True),
            },
            AtpProvenanceId::new(2),
        );
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "x",
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingDeclarationSymbol {
                symbol: AtpSymbolName::new("x")
            }
        );

        let mut parts = minimal_parts();
        parts.logic_profile = profile_first_order();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
                body: Box::new(AtpFormulaTree::True),
            },
            AtpProvenanceId::new(2),
        );
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "x",
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
        ));
        parts.declarations.push(AtpDeclaration::new(
            AtpDeclarationId::new(3),
            AtpDeclarationKind::GeneratedBinder,
            "x",
            0,
            AtpProvenanceId::new(2),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingSymbolMap {
                symbol: AtpSymbolName::new("S")
            }
        );

        let mut parts = minimal_parts();
        parts.logic_profile = profile_first_order();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
                body: Box::new(AtpFormulaTree::True),
            },
            AtpProvenanceId::new(2),
        );
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "x",
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
        ));
        parts.declarations.push(AtpDeclaration::new(
            AtpDeclarationId::new(3),
            AtpDeclarationKind::GeneratedBinder,
            "x",
            0,
            AtpProvenanceId::new(2),
        ));
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "S",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("sort:S")),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingDeclarationSymbol {
                symbol: AtpSymbolName::new("S")
            }
        );

        let mut parts = minimal_parts();
        parts.properties.push(EncodedProperty::axiom(
            AtpPropertyId::new(1),
            "property-target",
            AtpFormulaTree::True,
            AtpProvenanceId::new(2),
        ));
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "property-target",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("property-target")),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingDeclarationSymbol {
                symbol: AtpSymbolName::new("property-target")
            }
        );

        let mut parts = minimal_parts();
        parts.properties.push(EncodedProperty::axiom(
            AtpPropertyId::new(1),
            "missing-property-target",
            AtpFormulaTree::True,
            AtpProvenanceId::new(2),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingSymbolMap {
                symbol: AtpSymbolName::new("missing-property-target")
            }
        );
    }

    #[test]
    fn every_problem_formula_source_requires_existing_provenance() {
        let mut parts = minimal_parts();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(99),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingProvenance {
                owner: "axiom",
                provenance_id: AtpProvenanceId::new(99)
            }
        );

        let mut parts = minimal_parts();
        parts.conjecture = AtpFormula::new(
            AtpFormulaId::new(2),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(99),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingProvenance {
                owner: "conjecture",
                provenance_id: AtpProvenanceId::new(99)
            }
        );

        let mut parts = populated_parts(false);
        parts.type_context.guards[0] = AtpTypeGuard::new(
            AtpTypeGuardId::new(2),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(99),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingProvenance {
                owner: "type-guard",
                provenance_id: AtpProvenanceId::new(99)
            }
        );

        let mut parts = populated_parts(false);
        parts.properties[0] = EncodedProperty::axiom(
            AtpPropertyId::new(2),
            "P",
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(99),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingProvenance {
                owner: "property",
                provenance_id: AtpProvenanceId::new(99)
            }
        );
    }

    #[test]
    fn provenance_and_symbol_map_sources_fail_closed_when_incomplete() {
        let mut parts = minimal_parts();
        parts.provenance[0] = AtpProvenance::new(
            AtpProvenanceId::new(1),
            AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("")),
            "payload",
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::EmptyField {
                field: "provenance.source"
            }
        );

        for (source, field) in [
            (
                AtpSourceRef::ImportedAxiom {
                    package: AtpSourceBinding::new(""),
                    module: AtpSourceBinding::new("mod"),
                    item: AtpSourceBinding::new("ax"),
                    statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                        .expect("fingerprint"),
                    required_status: AtpRequiredProofStatus::new("kernel-checked"),
                    context_requirement: AtpSourceBinding::new("ctx"),
                },
                "imported.package",
            ),
            (
                AtpSourceRef::ImportedAxiom {
                    package: AtpSourceBinding::new("pkg"),
                    module: AtpSourceBinding::new(""),
                    item: AtpSourceBinding::new("ax"),
                    statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                        .expect("fingerprint"),
                    required_status: AtpRequiredProofStatus::new("kernel-checked"),
                    context_requirement: AtpSourceBinding::new("ctx"),
                },
                "imported.module",
            ),
            (
                AtpSourceRef::ImportedAxiom {
                    package: AtpSourceBinding::new("pkg"),
                    module: AtpSourceBinding::new("mod"),
                    item: AtpSourceBinding::new(""),
                    statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                        .expect("fingerprint"),
                    required_status: AtpRequiredProofStatus::new("kernel-checked"),
                    context_requirement: AtpSourceBinding::new("ctx"),
                },
                "imported.item",
            ),
            (
                AtpSourceRef::ImportedAxiom {
                    package: AtpSourceBinding::new("pkg"),
                    module: AtpSourceBinding::new("mod"),
                    item: AtpSourceBinding::new("ax"),
                    statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                        .expect("fingerprint"),
                    required_status: AtpRequiredProofStatus::new(""),
                    context_requirement: AtpSourceBinding::new("ctx"),
                },
                "imported.required_status",
            ),
            (
                AtpSourceRef::ImportedAxiom {
                    package: AtpSourceBinding::new("pkg"),
                    module: AtpSourceBinding::new("mod"),
                    item: AtpSourceBinding::new("ax"),
                    statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                        .expect("fingerprint"),
                    required_status: AtpRequiredProofStatus::new("kernel-checked"),
                    context_requirement: AtpSourceBinding::new(""),
                },
                "imported.context_requirement",
            ),
        ] {
            let mut parts = minimal_parts();
            parts.provenance[0] = AtpProvenance::new(AtpProvenanceId::new(1), source, "payload");
            assert_eq!(
                AtpProblem::try_new(parts).unwrap_err(),
                AtpProblemError::EmptyField { field }
            );
        }

        let mut parts = minimal_parts();
        parts.provenance[0] = AtpProvenance::new(
            AtpProvenanceId::new(1),
            AtpSourceRef::ImportedTheorem {
                package: AtpSourceBinding::new("pkg"),
                module: AtpSourceBinding::new("mod"),
                item: AtpSourceBinding::new("thm"),
                statement_fingerprint: AtpFingerprint::new(2, b"stmt".to_vec())
                    .expect("fingerprint"),
                required_status: AtpRequiredProofStatus::new("kernel-checked"),
                context_requirement: AtpSourceBinding::new(""),
            },
            "payload",
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::EmptyField {
                field: "imported.context_requirement"
            }
        );

        let mut parts = minimal_parts();
        parts.symbol_map[0] =
            AtpSymbolMapEntry::new("P", AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("")));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::EmptyField {
                field: "symbol_map.source"
            }
        );

        let mut parts = populated_parts(false);
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "guard-missing",
            AtpSymbolSource::TypeGuard(AtpTypeGuardId::new(99)),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingTypeGuard {
                type_guard: AtpTypeGuardId::new(99)
            }
        );
    }

    #[test]
    fn rejects_missing_formula_payloads_and_duplicate_ids() {
        let mut parts = minimal_parts();
        parts.axioms[0] = AtpFormula::missing(AtpFormulaId::new(1), AtpProvenanceId::new(2));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingFormulaPayload {
                formula_id: AtpFormulaId::new(1)
            }
        );

        let mut parts = minimal_parts();
        parts.conjecture = AtpFormula::missing(AtpFormulaId::new(2), AtpProvenanceId::new(3));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingFormulaPayload {
                formula_id: AtpFormulaId::new(2)
            }
        );

        let mut parts = minimal_parts();
        parts.axioms.push(AtpFormula::new(
            AtpFormulaId::new(1),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(2),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::DuplicateId {
                section: "axioms",
                id: 1
            }
        );

        let mut parts = minimal_parts();
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "P",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("duplicate")),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::DuplicateSymbolMap {
                symbol: AtpSymbolName::new("P")
            }
        );

        let mut parts = minimal_parts();
        parts.declarations.push(AtpDeclaration::new(
            AtpDeclarationId::new(3),
            AtpDeclarationKind::Predicate,
            "P",
            1,
            AtpProvenanceId::new(1),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::DuplicateDeclarationSymbol {
                symbol: AtpSymbolName::new("P")
            }
        );

        let mut parts = minimal_parts();
        parts.conjecture = AtpFormula::new(
            AtpFormulaId::new(1),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(3),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::DuplicateId {
                section: "formulas",
                id: 1
            }
        );
    }

    #[test]
    fn symbol_declarations_must_match_formula_kind_and_arity() {
        let mut parts = minimal_parts();
        parts.declarations[0] = AtpDeclaration::new(
            AtpDeclarationId::new(1),
            AtpDeclarationKind::Function,
            "P",
            1,
            AtpProvenanceId::new(1),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::InvalidSymbolDeclaration {
                symbol: AtpSymbolName::new("P"),
                expected: "predicate",
                actual: AtpDeclarationKind::Function
            }
        );

        let mut parts = minimal_parts();
        parts.declarations[0] = AtpDeclaration::new(
            AtpDeclarationId::new(1),
            AtpDeclarationKind::Predicate,
            "P",
            0,
            AtpProvenanceId::new(1),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::InvalidSymbolArity {
                symbol: AtpSymbolName::new("P"),
                expected: 1,
                actual: 0
            }
        );

        let mut parts = minimal_parts();
        parts.declarations[1] = AtpDeclaration::new(
            AtpDeclarationId::new(2),
            AtpDeclarationKind::Predicate,
            "a",
            0,
            AtpProvenanceId::new(1),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::InvalidSymbolDeclaration {
                symbol: AtpSymbolName::new("a"),
                expected: "function",
                actual: AtpDeclarationKind::Predicate
            }
        );

        let mut parts = minimal_parts();
        parts.declarations[1] = AtpDeclaration::new(
            AtpDeclarationId::new(2),
            AtpDeclarationKind::Function,
            "a",
            1,
            AtpProvenanceId::new(1),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::InvalidSymbolArity {
                symbol: AtpSymbolName::new("a"),
                expected: 0,
                actual: 1
            }
        );

        let mut parts = minimal_parts();
        parts.logic_profile = profile_first_order();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
                body: Box::new(atom("P", vec![variable("x")])),
            },
            AtpProvenanceId::new(2),
        );
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "x",
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
        ));
        parts.declarations.push(AtpDeclaration::new(
            AtpDeclarationId::new(3),
            AtpDeclarationKind::Predicate,
            "x",
            0,
            AtpProvenanceId::new(2),
        ));
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "S",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("sort:S")),
        ));
        parts.declarations.push(AtpDeclaration::new(
            AtpDeclarationId::new(4),
            AtpDeclarationKind::Sort,
            "S",
            0,
            AtpProvenanceId::new(2),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::InvalidSymbolDeclaration {
                symbol: AtpSymbolName::new("x"),
                expected: "generated binder",
                actual: AtpDeclarationKind::Predicate
            }
        );

        let mut parts = minimal_parts();
        parts.logic_profile = profile_first_order();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
                body: Box::new(AtpFormulaTree::True),
            },
            AtpProvenanceId::new(2),
        );
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "x",
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
        ));
        parts.declarations.push(AtpDeclaration::new(
            AtpDeclarationId::new(3),
            AtpDeclarationKind::GeneratedBinder,
            "x",
            0,
            AtpProvenanceId::new(2),
        ));
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "S",
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("sort:S")),
        ));
        parts.declarations.push(AtpDeclaration::new(
            AtpDeclarationId::new(4),
            AtpDeclarationKind::Predicate,
            "S",
            0,
            AtpProvenanceId::new(2),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::InvalidSymbolDeclaration {
                symbol: AtpSymbolName::new("S"),
                expected: "sort",
                actual: AtpDeclarationKind::Predicate
            }
        );
    }

    #[test]
    fn unsupported_profile_limitations_are_classified_separately() {
        let mut parts = minimal_parts();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Equality {
                left: constant("a"),
                right: constant("a"),
            },
            AtpProvenanceId::new(2),
        );
        parts.logic_profile = profile_without_equality();
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::UnsupportedProfileFeature {
                feature: "equality"
            }
        );

        let mut parts = minimal_parts();
        parts.axioms[0] = AtpFormula::new(
            AtpFormulaId::new(1),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", None)],
                body: Box::new(atom("P", vec![variable("x")])),
            },
            AtpProvenanceId::new(2),
        );
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "x",
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::UnsupportedProfileFeature {
                feature: "quantifier"
            }
        );
    }

    #[test]
    fn invalid_logic_profile_is_rejected_before_problem_construction() {
        assert_eq!(
            LogicProfile::try_new(
                "",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::PropositionalOnly,
                SoftTypeStrategy::BackendSorts,
                NativePropertySupport::Unsupported,
                BTreeSet::from([ConcreteFormat::Tptp]),
            )
            .unwrap_err(),
            AtpProblemError::InvalidLogicProfile {
                reason: "empty profile name"
            }
        );
        assert_eq!(
            LogicProfile::try_new(
                "empty-formats",
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::PropositionalOnly,
                SoftTypeStrategy::BackendSorts,
                NativePropertySupport::Unsupported,
                BTreeSet::new(),
            )
            .unwrap_err(),
            AtpProblemError::InvalidLogicProfile {
                reason: "no concrete encoder format"
            }
        );
    }

    #[test]
    fn expected_backend_result_has_only_unsat_variant() {
        let source = include_str!("problem.rs");
        let start = source
            .find("pub enum ExpectedBackendResult")
            .expect("expected-result enum");
        let tail = &source[start..];
        let end = tail.find("\n}\n").expect("expected-result enum end");
        assert_eq!(
            &tail[..end + 3],
            "pub enum ExpectedBackendResult {\n    Unsat,\n}\n"
        );
    }

    #[test]
    fn canonical_identity_length_prefixes_string_payloads() {
        let mut first = minimal_parts();
        first.provenance[0] = AtpProvenance::new(
            AtpProvenanceId::new(1),
            AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("ctx:1")),
            "alpha\nbeta",
        );
        let first = AtpProblem::try_new(first).expect("newline payload problem");

        let mut second = minimal_parts();
        second.provenance[0] = AtpProvenance::new(
            AtpProvenanceId::new(1),
            AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("ctx:1")),
            "alpha\\nbeta",
        );
        let second = AtpProblem::try_new(second).expect("escaped payload problem");

        assert_ne!(first.problem_id(), second.problem_id());
        assert!(first.debug_text().contains("616c7068610a62657461"));
    }

    #[test]
    fn properties_and_type_guards_require_provenance_and_supported_profiles() {
        let problem = populated_problem(false);
        assert_eq!(problem.properties().len(), 2);
        assert_eq!(problem.type_context().guards().len(), 2);

        let mut parts = populated_parts(false);
        parts.properties[0] = EncodedProperty::native_declaration(
            AtpPropertyId::new(2),
            "P",
            AtpDeclarationId::new(1),
            AtpProvenanceId::new(2),
        );
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::UnsupportedProfileFeature {
                feature: "native property declaration"
            }
        );

        let mut parts = minimal_parts();
        parts.logic_profile = profile_with_native_properties();
        parts.properties.push(EncodedProperty::native_declaration(
            AtpPropertyId::new(1),
            "P",
            AtpDeclarationId::new(1),
            AtpProvenanceId::new(2),
        ));
        let problem = AtpProblem::try_new(parts).expect("native property declaration");
        assert_eq!(problem.properties().len(), 1);

        let mut parts = minimal_parts();
        parts.logic_profile = profile_with_native_properties();
        parts.properties.push(EncodedProperty::native_declaration(
            AtpPropertyId::new(1),
            "P",
            AtpDeclarationId::new(99),
            AtpProvenanceId::new(2),
        ));
        assert_eq!(
            AtpProblem::try_new(parts).unwrap_err(),
            AtpProblemError::MissingDeclaration {
                declaration: AtpDeclarationId::new(99)
            }
        );
    }

    #[test]
    fn public_problem_rendering_excludes_prohibited_trusted_material() {
        let problem = populated_problem(false);
        let rendered = format!("{:?}\n{}", problem, problem.debug_text());
        for prohibited in [
            "SAT clause",
            "backend log",
            "backend used_axioms",
            "proof method",
            "accepted proof status",
        ] {
            assert!(
                !rendered.contains(prohibited),
                "prohibited trusted material leaked: {prohibited}"
            );
        }
    }

    fn minimal_problem() -> AtpProblem {
        AtpProblem::try_new(minimal_parts()).expect("minimal ATP problem")
    }

    fn populated_problem(reverse: bool) -> AtpProblem {
        AtpProblem::try_new(populated_parts(reverse)).expect("populated ATP problem")
    }

    fn minimal_parts() -> AtpProblemParts {
        let provenance = vec![
            provenance(
                1,
                AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("ctx:1")),
            ),
            provenance(
                2,
                AtpSourceRef::CitedPremise(AtpSourceBinding::new("premise:1")),
            ),
            provenance(
                3,
                AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:7")),
            ),
        ];
        AtpProblemParts {
            vc_id: VcId::new(7),
            target_binding: target_binding(),
            logic_profile: profile(),
            expected_result: ExpectedBackendResult::Unsat,
            declarations: vec![
                AtpDeclaration::new(
                    AtpDeclarationId::new(1),
                    AtpDeclarationKind::Predicate,
                    "P",
                    1,
                    AtpProvenanceId::new(1),
                ),
                AtpDeclaration::new(
                    AtpDeclarationId::new(2),
                    AtpDeclarationKind::Function,
                    "a",
                    0,
                    AtpProvenanceId::new(1),
                ),
            ],
            axioms: vec![AtpFormula::new(
                AtpFormulaId::new(1),
                atom("P", vec![constant("a")]),
                AtpProvenanceId::new(2),
            )],
            conjecture: AtpFormula::new(
                AtpFormulaId::new(2),
                atom("P", vec![constant("a")]),
                AtpProvenanceId::new(3),
            ),
            type_context: AtpTypeContext::new(Vec::new()),
            properties: Vec::new(),
            symbol_map: vec![
                AtpSymbolMapEntry::new(
                    "P",
                    AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P")),
                ),
                AtpSymbolMapEntry::new(
                    "a",
                    AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("const:a")),
                ),
            ],
            provenance,
            diagnostics: vec![AtpDiagnostic::new("note", "fixture diagnostic")],
        }
    }

    fn populated_parts(reverse: bool) -> AtpProblemParts {
        populated_parts_with_diagnostic(reverse, "a-note")
    }

    fn populated_parts_with_diagnostic(reverse: bool, diagnostic_key: &str) -> AtpProblemParts {
        let mut parts = minimal_parts();
        parts.logic_profile = profile_with_soft_types(SoftTypeStrategy::GuardPredicates);
        parts.axioms.push(AtpFormula::new(
            AtpFormulaId::new(3),
            atom("P", vec![constant("a")]),
            AtpProvenanceId::new(2),
        ));
        parts.type_context = AtpTypeContext::new(vec![
            AtpTypeGuard::new(
                AtpTypeGuardId::new(2),
                atom("P", vec![constant("a")]),
                AtpProvenanceId::new(1),
            ),
            AtpTypeGuard::new(
                AtpTypeGuardId::new(1),
                atom("P", vec![constant("a")]),
                AtpProvenanceId::new(1),
            ),
        ]);
        parts.properties = vec![
            EncodedProperty::axiom(
                AtpPropertyId::new(2),
                "P",
                atom("P", vec![constant("a")]),
                AtpProvenanceId::new(2),
            ),
            EncodedProperty::axiom(
                AtpPropertyId::new(1),
                "P",
                AtpFormulaTree::Implies(
                    Box::new(atom("P", vec![constant("a")])),
                    Box::new(atom("P", vec![constant("a")])),
                ),
                AtpProvenanceId::new(2),
            ),
        ];
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "x",
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new("binder:x")),
        ));
        parts.symbol_map.push(AtpSymbolMapEntry::new(
            "guard1",
            AtpSymbolSource::TypeGuard(AtpTypeGuardId::new(1)),
        ));
        parts.diagnostics = vec![AtpDiagnostic::new(
            diagnostic_key,
            "non semantic diagnostic",
        )];
        if reverse {
            parts.declarations.reverse();
            parts.provenance.reverse();
            parts.symbol_map.reverse();
            parts.axioms.reverse();
            parts.properties.reverse();
            parts.type_context.guards.reverse();
        }
        parts
    }

    fn target_binding() -> AtpTargetBinding {
        AtpTargetBinding::new(
            AtpFingerprint::new(18, b"target-vc-7".to_vec()).expect("fingerprint"),
            AtpSourceBinding::new("vc:7"),
        )
        .expect("target binding")
    }

    fn profile() -> LogicProfile {
        profile_with_soft_types(SoftTypeStrategy::BackendSorts)
    }

    fn profile_with_soft_types(soft_types: SoftTypeStrategy) -> LogicProfile {
        LogicProfile::try_new(
            "fof-fixture",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::PropositionalOnly,
            soft_types,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("profile")
    }

    fn profile_first_order() -> LogicProfile {
        LogicProfile::try_new(
            "fof-first-order",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::BackendSorts,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("profile")
    }

    fn profile_with_native_properties() -> LogicProfile {
        LogicProfile::try_new(
            "fof-native-properties",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::PropositionalOnly,
            SoftTypeStrategy::BackendSorts,
            NativePropertySupport::Supported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("profile")
    }

    fn profile_without_equality() -> LogicProfile {
        LogicProfile::try_new(
            "fof-no-equality",
            LogicFragment::Fof,
            EqualitySupport::Unsupported,
            QuantifierPolicy::PropositionalOnly,
            SoftTypeStrategy::BackendSorts,
            NativePropertySupport::Unsupported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("profile")
    }

    fn provenance(id: u32, source: AtpSourceRef) -> AtpProvenance {
        AtpProvenance::new(
            AtpProvenanceId::new(id),
            source,
            format!("provenance-payload-{id}"),
        )
    }

    fn atom(predicate: &str, arguments: Vec<AtpTerm>) -> AtpFormulaTree {
        AtpFormulaTree::Atom(AtpAtom::new(predicate, arguments))
    }

    fn variable(name: &str) -> AtpTerm {
        AtpTerm::Variable(AtpSymbolName::new(name))
    }

    fn constant(name: &str) -> AtpTerm {
        AtpTerm::Function {
            function: AtpSymbolName::new(name),
            arguments: Vec::new(),
        }
    }
}
