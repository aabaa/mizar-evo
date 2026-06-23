//! Core IR data shapes.
//!
//! Implements the data-layer contract specified in
//! [core_ir.md](../../../../doc/design/mizar-core/en/core_ir.md).

use mizar_resolve::resolved_ast::{ModuleId, SymbolId};
use mizar_session::{SourceId, SourceRange};
use std::{
    collections::BTreeMap,
    error::Error,
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

dense_id!(CoreItemId);
dense_id!(CoreTermId);
dense_id!(CoreFormulaId);
dense_id!(CoreDefinitionId);
dense_id!(CoreProofId);
dense_id!(CoreProofNodeId);
dense_id!(CoreAlgorithmId);
dense_id!(CoreAlgorithmStmtId);
dense_id!(GeneratedOriginId);
dense_id!(ObligationSeedId);
dense_id!(CoreDiagnosticId);
dense_id!(CoreVarId);

string_key!(CoreTypePredicate);
string_key!(CoreVisibility);
string_key!(CoreVarRole);
string_key!(GeneratedOriginKey);
string_key!(CoreProvenanceKey);
string_key!(CoreDiagnosticMessageKey);
string_key!(LocalProofOrProgramPath);
string_key!(NormalizedSemanticOrigin);
string_key!(CoreLabelRef);
string_key!(CorePlace);
string_key!(GhostEffectKey);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreIr {
    source_id: SourceId,
    module_id: ModuleId,
    items: CoreItemTable,
    terms: CoreTermTable,
    formulas: CoreFormulaTable,
    definitions: CoreDefinitionTable,
    proofs: CoreProofTable,
    proof_nodes: CoreProofNodeTable,
    algorithms: CoreAlgorithmTable,
    algorithm_statements: CoreAlgorithmStmtTable,
    generated: GeneratedOriginTable,
    obligation_seeds: ObligationSeedTable,
    source_map: CoreSourceMap,
    diagnostics: CoreDiagnosticTable,
}

impl CoreIr {
    pub fn try_new(mut parts: CoreIrParts) -> Result<Self, CoreIrError> {
        normalize_core_ir_parts(&mut parts);
        validate_core_ir(&parts)?;
        Ok(Self {
            source_id: parts.source_id,
            module_id: parts.module_id,
            items: parts.items,
            terms: parts.terms,
            formulas: parts.formulas,
            definitions: parts.definitions,
            proofs: parts.proofs,
            proof_nodes: parts.proof_nodes,
            algorithms: parts.algorithms,
            algorithm_statements: parts.algorithm_statements,
            generated: parts.generated,
            obligation_seeds: parts.obligation_seeds,
            source_map: parts.source_map,
            diagnostics: parts.diagnostics,
        })
    }

    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn items(&self) -> &CoreItemTable {
        &self.items
    }

    pub const fn terms(&self) -> &CoreTermTable {
        &self.terms
    }

    pub const fn formulas(&self) -> &CoreFormulaTable {
        &self.formulas
    }

    pub const fn definitions(&self) -> &CoreDefinitionTable {
        &self.definitions
    }

    pub const fn proofs(&self) -> &CoreProofTable {
        &self.proofs
    }

    pub const fn proof_nodes(&self) -> &CoreProofNodeTable {
        &self.proof_nodes
    }

    pub const fn algorithms(&self) -> &CoreAlgorithmTable {
        &self.algorithms
    }

    pub const fn algorithm_statements(&self) -> &CoreAlgorithmStmtTable {
        &self.algorithm_statements
    }

    pub const fn generated(&self) -> &GeneratedOriginTable {
        &self.generated
    }

    pub const fn obligation_seeds(&self) -> &ObligationSeedTable {
        &self.obligation_seeds
    }

    pub const fn source_map(&self) -> &CoreSourceMap {
        &self.source_map
    }

    pub const fn diagnostics(&self) -> &CoreDiagnosticTable {
        &self.diagnostics
    }

    pub fn has_error_nodes(&self) -> bool {
        self.terms
            .iter()
            .any(|(_, term)| matches!(term.kind, CoreTermKind::Error(_)))
            || self
                .formulas
                .iter()
                .any(|(_, formula)| matches!(formula.kind, CoreFormulaKind::Error(_)))
            || self
                .proof_nodes
                .iter()
                .any(|(_, node)| matches!(node.kind, CoreProofNodeKind::Error(_)))
            || self
                .algorithm_statements
                .iter()
                .any(|(_, stmt)| matches!(stmt.kind, CoreAlgorithmStmtKind::Error(_)))
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("core-ir-debug-v1\n");
        write_header(&mut output, self.source_id, &self.module_id);
        write_table(&mut output, "items", self.items.iter());
        write_table(&mut output, "terms", self.terms.iter());
        write_table(&mut output, "formulas", self.formulas.iter());
        write_table(&mut output, "definitions", self.definitions.iter());
        write_table(&mut output, "proofs", self.proofs.iter());
        write_table(&mut output, "proof-nodes", self.proof_nodes.iter());
        write_table(&mut output, "algorithms", self.algorithms.iter());
        write_table(
            &mut output,
            "algorithm-statements",
            self.algorithm_statements.iter(),
        );
        write_table(&mut output, "generated", self.generated.iter());
        write_table(
            &mut output,
            "obligation-seeds",
            self.obligation_seeds.iter(),
        );
        write_diagnostics(&mut output, &self.diagnostics);
        write_source_map(&mut output, &self.source_map);
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreIrParts {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub items: CoreItemTable,
    pub terms: CoreTermTable,
    pub formulas: CoreFormulaTable,
    pub definitions: CoreDefinitionTable,
    pub proofs: CoreProofTable,
    pub proof_nodes: CoreProofNodeTable,
    pub algorithms: CoreAlgorithmTable,
    pub algorithm_statements: CoreAlgorithmStmtTable,
    pub generated: GeneratedOriginTable,
    pub obligation_seeds: ObligationSeedTable,
    pub source_map: CoreSourceMap,
    pub diagnostics: CoreDiagnosticTable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreItem {
    pub symbol: SymbolId,
    pub kind: CoreItemKind,
    pub visibility: CoreVisibility,
    pub status: CoreItemStatus,
    pub dependencies: Vec<CoreItemId>,
    pub source: CoreSourceRef,
    pub diagnostics: Vec<CoreDiagnosticId>,
}

impl CoreItem {
    pub fn new(
        symbol: SymbolId,
        kind: CoreItemKind,
        visibility: impl Into<CoreVisibility>,
        source: CoreSourceRef,
    ) -> Self {
        Self {
            symbol,
            kind,
            visibility: visibility.into(),
            status: CoreItemStatus::Valid,
            dependencies: Vec::new(),
            source,
            diagnostics: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreItemKind {
    Structure,
    Mode,
    Attribute,
    Predicate,
    Functor,
    Theorem,
    Lemma,
    Scheme,
    Registration,
    Reduction,
    GeneratedDefinition,
    Algorithm,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoreItemStatus {
    Valid,
    Partial,
    Skipped,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreTerm {
    pub kind: CoreTermKind,
    pub source: CoreSourceRef,
}

impl CoreTerm {
    pub const fn new(kind: CoreTermKind, source: CoreSourceRef) -> Self {
        Self { kind, source }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreTermKind {
    Var(CoreVarId),
    Const(SymbolId),
    Apply {
        functor: SymbolId,
        args: Vec<CoreTermId>,
    },
    Select {
        selector: SymbolId,
        base: CoreTermId,
    },
    Tuple(Vec<CoreTermId>),
    SetEnum(Vec<CoreTermId>),
    Generated {
        origin: GeneratedOriginId,
        args: Vec<CoreTermId>,
    },
    Error(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreFormula {
    pub kind: CoreFormulaKind,
    pub source: CoreSourceRef,
}

impl CoreFormula {
    pub const fn new(kind: CoreFormulaKind, source: CoreSourceRef) -> Self {
        Self { kind, source }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreFormulaKind {
    True,
    False,
    Atom {
        predicate: SymbolId,
        args: Vec<CoreTermId>,
    },
    Equals {
        left: CoreTermId,
        right: CoreTermId,
    },
    TypePred {
        subject: CoreTermId,
        ty: CoreTypePredicate,
    },
    Not(CoreFormulaId),
    And(Vec<CoreFormulaId>),
    Or(Vec<CoreFormulaId>),
    Implies {
        premise: CoreFormulaId,
        conclusion: CoreFormulaId,
    },
    Iff {
        left: CoreFormulaId,
        right: CoreFormulaId,
    },
    Forall {
        binders: Vec<CoreBinder>,
        body: CoreFormulaId,
    },
    Exists {
        binders: Vec<CoreBinder>,
        body: CoreFormulaId,
    },
    Error(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreBinder {
    pub var: CoreVarId,
    pub role: CoreVarRole,
    pub ty_guard: Option<CoreFormulaId>,
    pub source_name: Option<String>,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreDefinition {
    pub item: CoreItemId,
    pub symbol: SymbolId,
    pub params: Vec<CoreBinder>,
    pub body: DefinitionBody,
    pub expansion: ExpansionPolicy,
    pub correctness: Vec<ObligationSeedId>,
    pub generated_dependencies: Vec<GeneratedOriginId>,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DefinitionBody {
    Term(CoreTermId),
    Formula(CoreFormulaId),
    Guarded(Vec<GuardedDefinitionBranch>),
    Algorithm(CoreAlgorithmId),
    Unavailable(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GuardedDefinitionBranch {
    pub guard: CoreFormulaId,
    pub body: DefinitionBranchBody,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum DefinitionBranchBody {
    Term(CoreTermId),
    Formula(CoreFormulaId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ExpansionPolicy {
    Opaque,
    Transparent,
    Reducible { registration: SymbolId },
    Computable { algorithm: SymbolId },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreProof {
    pub item: CoreItemId,
    pub proposition: CoreFormulaId,
    pub root: CoreProofNodeId,
    pub status: CoreProofStatus,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoreProofStatus {
    Open,
    Assumed,
    Conditional,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreProofNode {
    pub kind: CoreProofNodeKind,
    pub source: CoreSourceRef,
    pub diagnostics: Vec<CoreDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreProofNodeKind {
    IntroduceBinder {
        binder: CoreBinder,
        child: CoreProofNodeId,
    },
    Assume {
        label: Option<CoreLabelRef>,
        formula: CoreFormulaId,
        child: CoreProofNodeId,
    },
    Step {
        label: Option<CoreLabelRef>,
        formula: CoreFormulaId,
        justification: CoreJustification,
    },
    Branch {
        kind: ProofBranchKind,
        children: Vec<CoreProofNodeId>,
    },
    TerminalGoal(ObligationSeedId),
    Error(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreJustification {
    pub citations: Vec<CoreCitation>,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreCitation {
    Label(CoreLabelRef),
    Symbol(SymbolId),
    Generated(GeneratedOriginId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ProofBranchKind {
    Cases,
    Suppose,
    Now,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreAlgorithm {
    pub item: CoreItemId,
    pub symbol: SymbolId,
    pub params: Vec<CoreBinder>,
    pub result: Option<CoreBinder>,
    pub contracts: CoreContractSet,
    pub statements: Vec<CoreAlgorithmStmtId>,
    pub ghost_effects: Vec<GhostEffectKey>,
    pub source: CoreSourceRef,
    pub diagnostics: Vec<CoreDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreContractSet {
    pub requires: Vec<CoreFormulaId>,
    pub ensures: Vec<CoreFormulaId>,
    pub invariants: Vec<CoreFormulaId>,
    pub assertions: Vec<CoreFormulaId>,
    pub decreasing: Vec<CoreTermId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreAlgorithmStmt {
    pub owner: CoreAlgorithmId,
    pub kind: CoreAlgorithmStmtKind,
    pub source: CoreSourceRef,
    pub diagnostics: Vec<CoreDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreAlgorithmStmtKind {
    Let {
        binder: CoreBinder,
        value: Option<CoreTermId>,
        ghost: bool,
    },
    Assign {
        target: CorePlace,
        value: CoreTermId,
    },
    Assert {
        formula: CoreFormulaId,
    },
    If {
        condition: CoreFormulaId,
        then_body: Vec<CoreAlgorithmStmtId>,
        else_body: Vec<CoreAlgorithmStmtId>,
    },
    While {
        condition: CoreFormulaId,
        invariants: Vec<CoreFormulaId>,
        decreasing: Vec<CoreTermId>,
        body: Vec<CoreAlgorithmStmtId>,
    },
    Match {
        scrutinee: CoreTermId,
        arms: Vec<CoreAlgorithmMatchArm>,
    },
    Return(Option<CoreTermId>),
    Break,
    Continue,
    Pick {
        binder: CoreBinder,
        witness_ty: Option<CoreFormulaId>,
        ghost: bool,
    },
    Error(CoreDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreAlgorithmMatchArm {
    pub pattern: CoreProvenanceKey,
    pub body: Vec<CoreAlgorithmStmtId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedOrigin {
    pub owner: CoreItemId,
    pub kind: GeneratedOriginKind,
    pub key: GeneratedOriginKey,
    pub params: Vec<CoreVarId>,
    pub evidence: Vec<CoreProvenance>,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GeneratedOriginKind {
    StableChoice,
    FraenkelComprehension,
    LocalAbbreviation,
    TypePredicate,
    ErrorPlaceholder,
    AlgorithmPick,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationSeed {
    pub owner: CoreItemId,
    pub kind: ObligationSeedKind,
    pub goal: Option<CoreFormulaId>,
    pub context: Vec<CoreFormulaId>,
    pub local_path: LocalProofOrProgramPath,
    pub label: Option<CoreLabelRef>,
    pub semantic_origin: NormalizedSemanticOrigin,
    pub provenance: Vec<CoreProvenance>,
    pub source: CoreSourceRef,
    pub core_refs: Vec<CoreNodeRef>,
    pub status: ObligationSeedStatus,
    pub diagnostics: Vec<CoreDiagnosticId>,
}

impl ObligationSeed {
    pub fn canonical_key(&self) -> ObligationSeedCanonicalKey {
        ObligationSeedCanonicalKey {
            owner: self.owner,
            source: self.source.sort_key(),
            local_path: self.local_path.clone(),
            kind: self.kind.sort_name(),
            label: self.label.clone(),
            semantic_origin: self.semantic_origin.clone(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ObligationSeedCanonicalKey {
    owner: CoreItemId,
    source: CoreSourceSortKey,
    local_path: LocalProofOrProgramPath,
    kind: &'static str,
    label: Option<CoreLabelRef>,
    semantic_origin: NormalizedSemanticOrigin,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ObligationSeedKind {
    TheoremProof,
    DefinitionCorrectness,
    CheckerInitial,
    GeneratedNonEmptiness,
    GeneratedSethood,
    FraenkelMembershipAxiom,
    AlgorithmContract,
    AlgorithmTermination,
    GhostErasure,
}

impl ObligationSeedKind {
    const fn sort_name(&self) -> &'static str {
        match self {
            Self::TheoremProof => "theorem-proof",
            Self::DefinitionCorrectness => "definition-correctness",
            Self::CheckerInitial => "checker-initial",
            Self::GeneratedNonEmptiness => "generated-non-emptiness",
            Self::GeneratedSethood => "generated-sethood",
            Self::FraenkelMembershipAxiom => "fraenkel-membership-axiom",
            Self::AlgorithmContract => "algorithm-contract",
            Self::AlgorithmTermination => "algorithm-termination",
            Self::GhostErasure => "ghost-erasure",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ObligationSeedStatus {
    Active,
    Skipped,
    Deferred,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CoreSourceMap {
    pub item_sources: BTreeMap<CoreItemId, CoreSourceRef>,
    pub term_sources: BTreeMap<CoreTermId, CoreSourceRef>,
    pub formula_sources: BTreeMap<CoreFormulaId, CoreSourceRef>,
    pub definition_sources: BTreeMap<CoreDefinitionId, CoreSourceRef>,
    pub proof_sources: BTreeMap<CoreProofNodeId, CoreSourceRef>,
    pub algorithm_sources: BTreeMap<CoreAlgorithmStmtId, CoreSourceRef>,
    pub generated_sources: BTreeMap<GeneratedOriginId, CoreSourceRef>,
    pub obligation_sources: BTreeMap<ObligationSeedId, CoreSourceRef>,
}

impl CoreSourceMap {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreSourceRef {
    pub anchor: CoreSourceAnchor,
    pub provenance: Vec<CoreProvenance>,
}

impl CoreSourceRef {
    pub fn direct(range: SourceRange) -> Self {
        Self {
            anchor: CoreSourceAnchor::SourceRange(range),
            provenance: Vec::new(),
        }
    }

    pub fn generated(generated_from: GeneratedFrom) -> Self {
        Self {
            anchor: CoreSourceAnchor::GeneratedFrom(generated_from),
            provenance: Vec::new(),
        }
    }

    pub fn with_provenance(mut self, provenance: Vec<CoreProvenance>) -> Self {
        self.provenance = provenance;
        normalize_provenance(&mut self.provenance);
        self
    }

    pub const fn is_mapped(&self) -> bool {
        matches!(
            self.anchor,
            CoreSourceAnchor::SourceRange(_) | CoreSourceAnchor::GeneratedFrom(_)
        )
    }

    fn sort_key(&self) -> CoreSourceSortKey {
        match &self.anchor {
            CoreSourceAnchor::SourceRange(range) => CoreSourceSortKey {
                kind: 0,
                start: range.start,
                end: range.end,
                owner: None,
                origin_kind: String::new(),
                key: String::new(),
                reason: String::new(),
            },
            CoreSourceAnchor::GeneratedFrom(generated_from) => CoreSourceSortKey {
                kind: 1,
                start: 0,
                end: 0,
                owner: Some(generated_from.owner.clone()),
                origin_kind: format!("{:?}", generated_from.kind),
                key: generated_from.key.as_str().to_owned(),
                reason: generated_from.reason.as_str().to_owned(),
            },
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreSourceAnchor {
    SourceRange(SourceRange),
    GeneratedFrom(GeneratedFrom),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct GeneratedFrom {
    pub owner: CoreNodeRef,
    pub kind: GeneratedOriginKind,
    pub key: GeneratedOriginKey,
    pub reason: CoreProvenanceKey,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CoreSourceSortKey {
    kind: u8,
    start: usize,
    end: usize,
    owner: Option<CoreNodeRef>,
    origin_kind: String,
    key: String,
    reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct CoreProvenance {
    pub phase: CoreProvenancePhase,
    pub key: CoreProvenanceKey,
}

impl CoreProvenance {
    pub fn new(phase: CoreProvenancePhase, key: impl Into<CoreProvenanceKey>) -> Self {
        Self {
            phase,
            key: key.into(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoreProvenancePhase {
    Resolver,
    Checker,
    Erasure,
    View,
    Template,
    ProofSkeleton,
    Generated,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CoreDiagnostic {
    pub class: CoreDiagnosticClass,
    pub severity: CoreDiagnosticSeverity,
    pub recovery: CoreDiagnosticRecovery,
    pub message_key: CoreDiagnosticMessageKey,
    pub primary_source: CoreSourceRef,
    pub related: Vec<CoreSourceRef>,
    pub owner: Option<CoreNodeRef>,
}

impl CoreDiagnostic {
    pub fn error(
        class: CoreDiagnosticClass,
        message_key: impl Into<CoreDiagnosticMessageKey>,
        primary_source: CoreSourceRef,
    ) -> Self {
        Self {
            class,
            severity: CoreDiagnosticSeverity::Error,
            recovery: CoreDiagnosticRecovery::Fatal,
            message_key: message_key.into(),
            primary_source,
            related: Vec::new(),
            owner: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoreDiagnosticClass {
    UnresolvedSemanticInput,
    InvalidErasure,
    UnsupportedLowering,
    MalformedProofSkeleton,
    SourceMapping,
    AlgorithmShell,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoreDiagnosticSeverity {
    Note,
    Warning,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum CoreDiagnosticRecovery {
    Recovered,
    Partial,
    Fatal,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum CoreNodeRef {
    Item(CoreItemId),
    Term(CoreTermId),
    Formula(CoreFormulaId),
    Definition(CoreDefinitionId),
    Proof(CoreProofId),
    ProofNode(CoreProofNodeId),
    Algorithm(CoreAlgorithmId),
    AlgorithmStmt(CoreAlgorithmStmtId),
    Generated(GeneratedOriginId),
    ObligationSeed(ObligationSeedId),
    Diagnostic(CoreDiagnosticId),
}

impl CoreNodeRef {
    fn validate(&self, parts: &CoreIrParts) -> Result<(), CoreIrError> {
        match self {
            Self::Item(id) => validate_index("item", id.index(), parts.items.len()),
            Self::Term(id) => validate_index("term", id.index(), parts.terms.len()),
            Self::Formula(id) => validate_index("formula", id.index(), parts.formulas.len()),
            Self::Definition(id) => {
                validate_index("definition", id.index(), parts.definitions.len())
            }
            Self::Proof(id) => validate_index("proof", id.index(), parts.proofs.len()),
            Self::ProofNode(id) => {
                validate_index("proof node", id.index(), parts.proof_nodes.len())
            }
            Self::Algorithm(id) => validate_index("algorithm", id.index(), parts.algorithms.len()),
            Self::AlgorithmStmt(id) => validate_index(
                "algorithm statement",
                id.index(),
                parts.algorithm_statements.len(),
            ),
            Self::Generated(id) => validate_index("generated", id.index(), parts.generated.len()),
            Self::ObligationSeed(id) => {
                validate_index("obligation seed", id.index(), parts.obligation_seeds.len())
            }
            Self::Diagnostic(id) => {
                validate_index("diagnostic", id.index(), parts.diagnostics.len())
            }
        }
    }
}

macro_rules! table {
    ($table:ident, $id:ident, $entry:ty) => {
        #[derive(Debug, Clone, PartialEq, Eq, Default)]
        pub struct $table {
            entries: Vec<$entry>,
        }

        impl $table {
            pub fn new() -> Self {
                Self::default()
            }

            pub fn insert(&mut self, entry: $entry) -> $id {
                let id = $id::new(self.entries.len());
                self.entries.push(entry);
                id
            }

            pub fn get(&self, id: $id) -> Option<&$entry> {
                self.entries.get(id.index())
            }

            pub fn get_mut(&mut self, id: $id) -> Option<&mut $entry> {
                self.entries.get_mut(id.index())
            }

            pub fn iter(&self) -> impl Iterator<Item = ($id, &$entry)> {
                self.entries
                    .iter()
                    .enumerate()
                    .map(|(index, entry)| ($id::new(index), entry))
            }

            pub fn iter_mut(&mut self) -> impl Iterator<Item = ($id, &mut $entry)> {
                self.entries
                    .iter_mut()
                    .enumerate()
                    .map(|(index, entry)| ($id::new(index), entry))
            }

            pub fn len(&self) -> usize {
                self.entries.len()
            }

            pub fn is_empty(&self) -> bool {
                self.entries.is_empty()
            }
        }
    };
}

table!(CoreItemTable, CoreItemId, CoreItem);
table!(CoreTermTable, CoreTermId, CoreTerm);
table!(CoreFormulaTable, CoreFormulaId, CoreFormula);
table!(CoreDefinitionTable, CoreDefinitionId, CoreDefinition);
table!(CoreProofTable, CoreProofId, CoreProof);
table!(CoreProofNodeTable, CoreProofNodeId, CoreProofNode);
table!(CoreAlgorithmTable, CoreAlgorithmId, CoreAlgorithm);
table!(
    CoreAlgorithmStmtTable,
    CoreAlgorithmStmtId,
    CoreAlgorithmStmt
);
table!(GeneratedOriginTable, GeneratedOriginId, GeneratedOrigin);
table!(CoreDiagnosticTable, CoreDiagnosticId, CoreDiagnostic);

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ObligationSeedTable {
    entries: Vec<ObligationSeed>,
}

impl ObligationSeedTable {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert(&mut self, seed: ObligationSeed) -> ObligationSeedId {
        let id = ObligationSeedId::new(self.entries.len());
        self.entries.push(seed);
        id
    }

    pub fn get(&self, id: ObligationSeedId) -> Option<&ObligationSeed> {
        self.entries.get(id.index())
    }

    pub fn get_mut(&mut self, id: ObligationSeedId) -> Option<&mut ObligationSeed> {
        self.entries.get_mut(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (ObligationSeedId, &ObligationSeed)> {
        self.entries
            .iter()
            .enumerate()
            .map(|(index, entry)| (ObligationSeedId::new(index), entry))
    }

    pub fn iter_mut(&mut self) -> impl Iterator<Item = (ObligationSeedId, &mut ObligationSeed)> {
        self.entries
            .iter_mut()
            .enumerate()
            .map(|(index, entry)| (ObligationSeedId::new(index), entry))
    }

    pub fn canonical_order(&self) -> Vec<ObligationSeedId> {
        let mut ids = self.iter().map(|(id, _)| id).collect::<Vec<_>>();
        ids.sort_by(|left, right| {
            self.get(*left)
                .expect("left id from table iteration")
                .canonical_key()
                .cmp(
                    &self
                        .get(*right)
                        .expect("right id from table iteration")
                        .canonical_key(),
                )
                .then_with(|| left.cmp(right))
        });
        ids
    }

    pub fn len(&self) -> usize {
        self.entries.len()
    }

    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum CoreIrError {
    InvalidReference {
        table: &'static str,
        index: usize,
        len: usize,
    },
    MissingSourceMap {
        table: &'static str,
        index: usize,
    },
    UnmappedSource {
        table: &'static str,
        index: usize,
    },
    SourceMapMismatch {
        table: &'static str,
        index: usize,
    },
    InvalidSourceMapEntry {
        table: &'static str,
        index: usize,
        len: usize,
    },
    InvalidSourceRange {
        table: &'static str,
        index: usize,
        reason: &'static str,
    },
    StableChoiceGeneratedTerm {
        term: CoreTermId,
        origin: GeneratedOriginId,
    },
    StatementOwnerMismatch {
        statement: CoreAlgorithmStmtId,
        expected: CoreAlgorithmId,
        actual: CoreAlgorithmId,
    },
    DuplicateGeneratedOrigin {
        first: GeneratedOriginId,
        duplicate: GeneratedOriginId,
    },
    MissingGeneratedOriginForSource {
        table: &'static str,
        index: usize,
        kind: GeneratedOriginKind,
        key: GeneratedOriginKey,
    },
    ErrorDiagnosticMismatch {
        table: &'static str,
        index: usize,
        diagnostic: CoreDiagnosticId,
    },
    ActiveObligationSeedWithoutGoal {
        seed: ObligationSeedId,
    },
    InactiveObligationSeedWithoutReason {
        seed: ObligationSeedId,
    },
}

impl fmt::Display for CoreIrError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidReference { table, index, len } => {
                write!(
                    formatter,
                    "invalid {table} reference {index}; table length is {len}"
                )
            }
            Self::MissingSourceMap { table, index } => {
                write!(
                    formatter,
                    "missing source-map entry for {table} row {index}"
                )
            }
            Self::UnmappedSource { table, index } => {
                write!(formatter, "unmapped source for {table} row {index}")
            }
            Self::SourceMapMismatch { table, index } => {
                write!(
                    formatter,
                    "source-map entry for {table} row {index} differs from the node source"
                )
            }
            Self::InvalidSourceMapEntry { table, index, len } => {
                write!(
                    formatter,
                    "invalid source-map entry for {table} row {index}; table length is {len}"
                )
            }
            Self::InvalidSourceRange {
                table,
                index,
                reason,
            } => {
                write!(
                    formatter,
                    "invalid source range for {table} row {index}: {reason}"
                )
            }
            Self::StableChoiceGeneratedTerm { term, origin } => {
                write!(
                    formatter,
                    "term {} uses generated stable-choice origin {} instead of an Apply node",
                    term.index(),
                    origin.index()
                )
            }
            Self::StatementOwnerMismatch {
                statement,
                expected,
                actual,
            } => {
                write!(
                    formatter,
                    "algorithm statement {} is listed under algorithm {} but owned by algorithm {}",
                    statement.index(),
                    expected.index(),
                    actual.index()
                )
            }
            Self::DuplicateGeneratedOrigin { first, duplicate } => {
                write!(
                    formatter,
                    "generated origin {} duplicates generated origin {}",
                    duplicate.index(),
                    first.index()
                )
            }
            Self::MissingGeneratedOriginForSource {
                table,
                index,
                kind,
                key,
            } => {
                write!(
                    formatter,
                    "{table} row {index} has GeneratedFrom({kind:?}, {}) without a matching generated origin",
                    key.as_str()
                )
            }
            Self::ErrorDiagnosticMismatch {
                table,
                index,
                diagnostic,
            } => {
                write!(
                    formatter,
                    "{table} error node {index} is not explained by diagnostic {}",
                    diagnostic.index()
                )
            }
            Self::ActiveObligationSeedWithoutGoal { seed } => {
                write!(
                    formatter,
                    "active obligation seed {} has no goal",
                    seed.index()
                )
            }
            Self::InactiveObligationSeedWithoutReason { seed } => {
                write!(
                    formatter,
                    "inactive obligation seed {} has no diagnostic or provenance reason",
                    seed.index()
                )
            }
        }
    }
}

impl Error for CoreIrError {}

fn normalize_core_ir_parts(parts: &mut CoreIrParts) {
    for (_, item) in parts.items.iter_mut() {
        normalize_source_ref(&mut item.source);
        item.diagnostics.sort();
    }
    for (_, term) in parts.terms.iter_mut() {
        normalize_source_ref(&mut term.source);
    }
    for (_, formula) in parts.formulas.iter_mut() {
        normalize_source_ref(&mut formula.source);
    }
    for (_, definition) in parts.definitions.iter_mut() {
        for binder in &mut definition.params {
            normalize_binder(binder);
        }
        normalize_source_ref(&mut definition.source);
    }
    for (_, proof) in parts.proofs.iter_mut() {
        normalize_source_ref(&mut proof.source);
    }
    for (_, node) in parts.proof_nodes.iter_mut() {
        normalize_source_ref(&mut node.source);
        normalize_proof_node(&mut node.kind);
        node.diagnostics.sort();
    }
    for (_, algorithm) in parts.algorithms.iter_mut() {
        for binder in &mut algorithm.params {
            normalize_binder(binder);
        }
        if let Some(result) = &mut algorithm.result {
            normalize_binder(result);
        }
        normalize_source_ref(&mut algorithm.source);
        algorithm.diagnostics.sort();
    }
    for (_, statement) in parts.algorithm_statements.iter_mut() {
        normalize_source_ref(&mut statement.source);
        normalize_algorithm_statement(&mut statement.kind);
        statement.diagnostics.sort();
    }
    for (_, generated) in parts.generated.iter_mut() {
        normalize_provenance(&mut generated.evidence);
        normalize_source_ref(&mut generated.source);
    }
    for (_, seed) in parts.obligation_seeds.iter_mut() {
        normalize_provenance(&mut seed.provenance);
        normalize_source_ref(&mut seed.source);
        seed.diagnostics.sort();
    }
    for source in parts.source_map.item_sources.values_mut() {
        normalize_source_ref(source);
    }
    for source in parts.source_map.term_sources.values_mut() {
        normalize_source_ref(source);
    }
    for source in parts.source_map.formula_sources.values_mut() {
        normalize_source_ref(source);
    }
    for source in parts.source_map.definition_sources.values_mut() {
        normalize_source_ref(source);
    }
    for source in parts.source_map.proof_sources.values_mut() {
        normalize_source_ref(source);
    }
    for source in parts.source_map.algorithm_sources.values_mut() {
        normalize_source_ref(source);
    }
    for source in parts.source_map.generated_sources.values_mut() {
        normalize_source_ref(source);
    }
    for source in parts.source_map.obligation_sources.values_mut() {
        normalize_source_ref(source);
    }
    for (_, diagnostic) in parts.diagnostics.iter_mut() {
        normalize_source_ref(&mut diagnostic.primary_source);
        for related in &mut diagnostic.related {
            normalize_source_ref(related);
        }
        diagnostic.related.sort_by(source_ref_cmp);
    }
}

fn normalize_proof_node(kind: &mut CoreProofNodeKind) {
    match kind {
        CoreProofNodeKind::IntroduceBinder { binder, .. } => normalize_binder(binder),
        CoreProofNodeKind::Step { justification, .. } => {
            normalize_source_ref(&mut justification.source);
        }
        CoreProofNodeKind::Assume { .. }
        | CoreProofNodeKind::Branch { .. }
        | CoreProofNodeKind::TerminalGoal(_)
        | CoreProofNodeKind::Error(_) => {}
    }
}

fn normalize_algorithm_statement(kind: &mut CoreAlgorithmStmtKind) {
    match kind {
        CoreAlgorithmStmtKind::Let { binder, .. } | CoreAlgorithmStmtKind::Pick { binder, .. } => {
            normalize_binder(binder);
        }
        CoreAlgorithmStmtKind::Assign { .. }
        | CoreAlgorithmStmtKind::Assert { .. }
        | CoreAlgorithmStmtKind::If { .. }
        | CoreAlgorithmStmtKind::While { .. }
        | CoreAlgorithmStmtKind::Match { .. }
        | CoreAlgorithmStmtKind::Return(_)
        | CoreAlgorithmStmtKind::Break
        | CoreAlgorithmStmtKind::Continue
        | CoreAlgorithmStmtKind::Error(_) => {}
    }
}

fn normalize_binder(binder: &mut CoreBinder) {
    normalize_source_ref(&mut binder.source);
}

fn normalize_source_ref(source: &mut CoreSourceRef) {
    normalize_provenance(&mut source.provenance);
}

fn normalize_provenance(provenance: &mut [CoreProvenance]) {
    provenance.sort();
}

fn source_ref_cmp(left: &CoreSourceRef, right: &CoreSourceRef) -> std::cmp::Ordering {
    left.sort_key()
        .cmp(&right.sort_key())
        .then_with(|| left.provenance.cmp(&right.provenance))
}

fn validate_core_ir(parts: &CoreIrParts) -> Result<(), CoreIrError> {
    for (id, item) in parts.items.iter() {
        validate_source_entry(
            "item",
            id.index(),
            &item.source,
            parts.source_map.item_sources.get(&id),
            parts,
        )?;
        for dependency in &item.dependencies {
            validate_index("item", dependency.index(), parts.items.len())?;
        }
        validate_diagnostics(&item.diagnostics, parts)?;
    }

    for (id, term) in parts.terms.iter() {
        validate_source_entry(
            "term",
            id.index(),
            &term.source,
            parts.source_map.term_sources.get(&id),
            parts,
        )?;
        validate_term(id, term, parts)?;
    }

    for (id, formula) in parts.formulas.iter() {
        validate_source_entry(
            "formula",
            id.index(),
            &formula.source,
            parts.source_map.formula_sources.get(&id),
            parts,
        )?;
        validate_formula(id, formula, parts)?;
    }

    for (id, definition) in parts.definitions.iter() {
        validate_source_entry(
            "definition",
            id.index(),
            &definition.source,
            parts.source_map.definition_sources.get(&id),
            parts,
        )?;
        validate_index("item", definition.item.index(), parts.items.len())?;
        validate_binders(&definition.params, parts)?;
        validate_definition_body(&definition.body, parts)?;
        for seed in &definition.correctness {
            validate_index(
                "obligation seed",
                seed.index(),
                parts.obligation_seeds.len(),
            )?;
        }
        for generated in &definition.generated_dependencies {
            validate_index("generated", generated.index(), parts.generated.len())?;
        }
    }

    for (id, proof) in parts.proofs.iter() {
        validate_source("proof", id.index(), &proof.source, parts)?;
        validate_index("item", proof.item.index(), parts.items.len())?;
        validate_index("formula", proof.proposition.index(), parts.formulas.len())?;
        validate_index("proof node", proof.root.index(), parts.proof_nodes.len())?;
    }

    for (id, node) in parts.proof_nodes.iter() {
        validate_source_entry(
            "proof",
            id.index(),
            &node.source,
            parts.source_map.proof_sources.get(&id),
            parts,
        )?;
        validate_proof_node(id, node, parts)?;
        validate_diagnostics(&node.diagnostics, parts)?;
    }

    for (id, algorithm) in parts.algorithms.iter() {
        validate_source("algorithm", id.index(), &algorithm.source, parts)?;
        validate_index("item", algorithm.item.index(), parts.items.len())?;
        validate_binders(&algorithm.params, parts)?;
        if let Some(result) = &algorithm.result {
            validate_binder(result, parts)?;
        }
        validate_contracts(&algorithm.contracts, parts)?;
        for statement in &algorithm.statements {
            validate_statement_owner(*statement, id, parts)?;
        }
        validate_diagnostics(&algorithm.diagnostics, parts)?;
    }

    for (id, statement) in parts.algorithm_statements.iter() {
        validate_source_entry(
            "algorithm statement",
            id.index(),
            &statement.source,
            parts.source_map.algorithm_sources.get(&id),
            parts,
        )?;
        validate_index("algorithm", statement.owner.index(), parts.algorithms.len())?;
        validate_algorithm_statement(
            id,
            statement.owner,
            &statement.source,
            &statement.kind,
            parts,
        )?;
        validate_diagnostics(&statement.diagnostics, parts)?;
    }

    for (id, generated) in parts.generated.iter() {
        validate_source_entry(
            "generated",
            id.index(),
            &generated.source,
            parts.source_map.generated_sources.get(&id),
            parts,
        )?;
        validate_index("item", generated.owner.index(), parts.items.len())?;
    }

    validate_generated_origin_uniqueness(parts)?;

    for (id, seed) in parts.obligation_seeds.iter() {
        validate_source_entry(
            "obligation seed",
            id.index(),
            &seed.source,
            parts.source_map.obligation_sources.get(&id),
            parts,
        )?;
        validate_obligation_seed(id, seed, parts)?;
    }

    for (id, diagnostic) in parts.diagnostics.iter() {
        validate_source("diagnostic", id.index(), &diagnostic.primary_source, parts)?;
        if let Some(owner) = &diagnostic.owner {
            owner.validate(parts)?;
        }
        for related in &diagnostic.related {
            validate_source("diagnostic related", id.index(), related, parts)?;
        }
    }

    validate_source_map_domain(
        "item",
        parts.source_map.item_sources.keys().map(|id| id.index()),
        parts.items.len(),
    )?;
    validate_source_map_domain(
        "term",
        parts.source_map.term_sources.keys().map(|id| id.index()),
        parts.terms.len(),
    )?;
    validate_source_map_domain(
        "formula",
        parts.source_map.formula_sources.keys().map(|id| id.index()),
        parts.formulas.len(),
    )?;
    validate_source_map_domain(
        "definition",
        parts
            .source_map
            .definition_sources
            .keys()
            .map(|id| id.index()),
        parts.definitions.len(),
    )?;
    validate_source_map_domain(
        "proof",
        parts.source_map.proof_sources.keys().map(|id| id.index()),
        parts.proof_nodes.len(),
    )?;
    validate_source_map_domain(
        "algorithm statement",
        parts
            .source_map
            .algorithm_sources
            .keys()
            .map(|id| id.index()),
        parts.algorithm_statements.len(),
    )?;
    validate_source_map_domain(
        "generated",
        parts
            .source_map
            .generated_sources
            .keys()
            .map(|id| id.index()),
        parts.generated.len(),
    )?;
    validate_source_map_domain(
        "obligation seed",
        parts
            .source_map
            .obligation_sources
            .keys()
            .map(|id| id.index()),
        parts.obligation_seeds.len(),
    )?;

    Ok(())
}

fn validate_term(id: CoreTermId, term: &CoreTerm, parts: &CoreIrParts) -> Result<(), CoreIrError> {
    match &term.kind {
        CoreTermKind::Var(_) | CoreTermKind::Const(_) => {}
        CoreTermKind::Apply { args, .. }
        | CoreTermKind::Tuple(args)
        | CoreTermKind::SetEnum(args) => {
            for arg in args {
                validate_index("term", arg.index(), parts.terms.len())?;
            }
        }
        CoreTermKind::Select { base, .. } => {
            validate_index("term", base.index(), parts.terms.len())?;
        }
        CoreTermKind::Generated { origin, args } => {
            validate_index("generated", origin.index(), parts.generated.len())?;
            if matches!(
                parts.generated.get(*origin).map(|origin| &origin.kind),
                Some(GeneratedOriginKind::StableChoice)
            ) {
                return Err(CoreIrError::StableChoiceGeneratedTerm {
                    term: id,
                    origin: *origin,
                });
            }
            for arg in args {
                validate_index("term", arg.index(), parts.terms.len())?;
            }
        }
        CoreTermKind::Error(diagnostic) => {
            validate_error_diagnostic(
                "term",
                id.index(),
                CoreNodeRef::Term(id),
                &term.source,
                *diagnostic,
                parts,
            )?;
        }
    }
    Ok(())
}

fn validate_formula(
    id: CoreFormulaId,
    formula: &CoreFormula,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    match &formula.kind {
        CoreFormulaKind::True | CoreFormulaKind::False => {}
        CoreFormulaKind::Atom { args, .. } => {
            for arg in args {
                validate_index("term", arg.index(), parts.terms.len())?;
            }
        }
        CoreFormulaKind::Equals { left, right } => {
            validate_index("term", left.index(), parts.terms.len())?;
            validate_index("term", right.index(), parts.terms.len())?;
        }
        CoreFormulaKind::TypePred { subject, .. } => {
            validate_index("term", subject.index(), parts.terms.len())?;
        }
        CoreFormulaKind::Not(id) => validate_index("formula", id.index(), parts.formulas.len())?,
        CoreFormulaKind::And(ids) | CoreFormulaKind::Or(ids) => {
            for id in ids {
                validate_index("formula", id.index(), parts.formulas.len())?;
            }
        }
        CoreFormulaKind::Implies {
            premise,
            conclusion,
        } => {
            validate_index("formula", premise.index(), parts.formulas.len())?;
            validate_index("formula", conclusion.index(), parts.formulas.len())?;
        }
        CoreFormulaKind::Iff { left, right } => {
            validate_index("formula", left.index(), parts.formulas.len())?;
            validate_index("formula", right.index(), parts.formulas.len())?;
        }
        CoreFormulaKind::Forall { binders, body } | CoreFormulaKind::Exists { binders, body } => {
            validate_binders(binders, parts)?;
            validate_index("formula", body.index(), parts.formulas.len())?;
        }
        CoreFormulaKind::Error(diagnostic) => {
            validate_error_diagnostic(
                "formula",
                id.index(),
                CoreNodeRef::Formula(id),
                &formula.source,
                *diagnostic,
                parts,
            )?;
        }
    }
    Ok(())
}

fn validate_definition_body(body: &DefinitionBody, parts: &CoreIrParts) -> Result<(), CoreIrError> {
    match body {
        DefinitionBody::Term(term) => validate_index("term", term.index(), parts.terms.len()),
        DefinitionBody::Formula(formula) => {
            validate_index("formula", formula.index(), parts.formulas.len())
        }
        DefinitionBody::Guarded(branches) => {
            for branch in branches {
                validate_index("formula", branch.guard.index(), parts.formulas.len())?;
                match branch.body {
                    DefinitionBranchBody::Term(term) => {
                        validate_index("term", term.index(), parts.terms.len())?;
                    }
                    DefinitionBranchBody::Formula(formula) => {
                        validate_index("formula", formula.index(), parts.formulas.len())?;
                    }
                }
            }
            Ok(())
        }
        DefinitionBody::Algorithm(algorithm) => {
            validate_index("algorithm", algorithm.index(), parts.algorithms.len())
        }
        DefinitionBody::Unavailable(diagnostic) => {
            validate_index("diagnostic", diagnostic.index(), parts.diagnostics.len())
        }
    }
}

fn validate_proof_node(
    id: CoreProofNodeId,
    node: &CoreProofNode,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    match &node.kind {
        CoreProofNodeKind::IntroduceBinder { binder, child } => {
            validate_binder(binder, parts)?;
            validate_index("proof node", child.index(), parts.proof_nodes.len())?;
        }
        CoreProofNodeKind::Assume { formula, child, .. } => {
            validate_index("formula", formula.index(), parts.formulas.len())?;
            validate_index("proof node", child.index(), parts.proof_nodes.len())?;
        }
        CoreProofNodeKind::Step {
            formula,
            justification,
            ..
        } => {
            validate_index("formula", formula.index(), parts.formulas.len())?;
            for citation in &justification.citations {
                validate_citation(citation, parts)?;
            }
            validate_source(
                "justification",
                formula.index(),
                &justification.source,
                parts,
            )?;
        }
        CoreProofNodeKind::Branch { children, .. } => {
            for child in children {
                validate_index("proof node", child.index(), parts.proof_nodes.len())?;
            }
        }
        CoreProofNodeKind::TerminalGoal(seed) => {
            validate_index(
                "obligation seed",
                seed.index(),
                parts.obligation_seeds.len(),
            )?;
        }
        CoreProofNodeKind::Error(diagnostic) => {
            validate_error_diagnostic(
                "proof",
                id.index(),
                CoreNodeRef::ProofNode(id),
                &node.source,
                *diagnostic,
                parts,
            )?;
        }
    }
    Ok(())
}

fn validate_algorithm_statement(
    id: CoreAlgorithmStmtId,
    owner: CoreAlgorithmId,
    source: &CoreSourceRef,
    statement: &CoreAlgorithmStmtKind,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    match statement {
        CoreAlgorithmStmtKind::Let { binder, value, .. } => {
            validate_binder(binder, parts)?;
            if let Some(value) = value {
                validate_index("term", value.index(), parts.terms.len())?;
            }
        }
        CoreAlgorithmStmtKind::Assign { value, .. } => {
            validate_index("term", value.index(), parts.terms.len())?;
        }
        CoreAlgorithmStmtKind::Assert { formula } => {
            validate_index("formula", formula.index(), parts.formulas.len())?;
        }
        CoreAlgorithmStmtKind::If {
            condition,
            then_body,
            else_body,
        } => {
            validate_index("formula", condition.index(), parts.formulas.len())?;
            validate_statements_owned(then_body, owner, parts)?;
            validate_statements_owned(else_body, owner, parts)?;
        }
        CoreAlgorithmStmtKind::While {
            condition,
            invariants,
            decreasing,
            body,
        } => {
            validate_index("formula", condition.index(), parts.formulas.len())?;
            validate_formulas(invariants, parts)?;
            validate_terms(decreasing, parts)?;
            validate_statements_owned(body, owner, parts)?;
        }
        CoreAlgorithmStmtKind::Match { scrutinee, arms } => {
            validate_index("term", scrutinee.index(), parts.terms.len())?;
            for arm in arms {
                validate_statements_owned(&arm.body, owner, parts)?;
            }
        }
        CoreAlgorithmStmtKind::Return(term) => {
            if let Some(term) = term {
                validate_index("term", term.index(), parts.terms.len())?;
            }
        }
        CoreAlgorithmStmtKind::Break | CoreAlgorithmStmtKind::Continue => {}
        CoreAlgorithmStmtKind::Pick {
            binder, witness_ty, ..
        } => {
            validate_binder(binder, parts)?;
            if let Some(witness_ty) = witness_ty {
                validate_index("formula", witness_ty.index(), parts.formulas.len())?;
            }
        }
        CoreAlgorithmStmtKind::Error(diagnostic) => {
            validate_error_diagnostic(
                "algorithm statement",
                id.index(),
                CoreNodeRef::AlgorithmStmt(id),
                source,
                *diagnostic,
                parts,
            )?;
        }
    }
    Ok(())
}

fn validate_obligation_seed(
    id: ObligationSeedId,
    seed: &ObligationSeed,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    validate_index("item", seed.owner.index(), parts.items.len())?;
    if let Some(goal) = seed.goal {
        validate_index("formula", goal.index(), parts.formulas.len())?;
    } else if seed.status == ObligationSeedStatus::Active {
        return Err(CoreIrError::ActiveObligationSeedWithoutGoal { seed: id });
    } else if seed.provenance.is_empty() && seed.diagnostics.is_empty() {
        return Err(CoreIrError::InactiveObligationSeedWithoutReason { seed: id });
    }
    validate_formulas(&seed.context, parts)?;
    for core_ref in &seed.core_refs {
        core_ref.validate(parts)?;
    }
    validate_diagnostics(&seed.diagnostics, parts)?;
    Ok(())
}

fn validate_contracts(contracts: &CoreContractSet, parts: &CoreIrParts) -> Result<(), CoreIrError> {
    validate_formulas(&contracts.requires, parts)?;
    validate_formulas(&contracts.ensures, parts)?;
    validate_formulas(&contracts.invariants, parts)?;
    validate_formulas(&contracts.assertions, parts)?;
    validate_terms(&contracts.decreasing, parts)
}

fn validate_citation(citation: &CoreCitation, parts: &CoreIrParts) -> Result<(), CoreIrError> {
    match citation {
        CoreCitation::Label(_) | CoreCitation::Symbol(_) => Ok(()),
        CoreCitation::Generated(id) => {
            validate_index("generated", id.index(), parts.generated.len())
        }
    }
}

fn validate_binders(binders: &[CoreBinder], parts: &CoreIrParts) -> Result<(), CoreIrError> {
    for binder in binders {
        validate_binder(binder, parts)?;
    }
    Ok(())
}

fn validate_binder(binder: &CoreBinder, parts: &CoreIrParts) -> Result<(), CoreIrError> {
    validate_source("binder", binder.var.index(), &binder.source, parts)?;
    if let Some(guard) = binder.ty_guard {
        validate_index("formula", guard.index(), parts.formulas.len())?;
    }
    Ok(())
}

fn validate_terms(ids: &[CoreTermId], parts: &CoreIrParts) -> Result<(), CoreIrError> {
    for id in ids {
        validate_index("term", id.index(), parts.terms.len())?;
    }
    Ok(())
}

fn validate_formulas(ids: &[CoreFormulaId], parts: &CoreIrParts) -> Result<(), CoreIrError> {
    for id in ids {
        validate_index("formula", id.index(), parts.formulas.len())?;
    }
    Ok(())
}

fn validate_statement_owner(
    id: CoreAlgorithmStmtId,
    expected: CoreAlgorithmId,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    validate_index(
        "algorithm statement",
        id.index(),
        parts.algorithm_statements.len(),
    )?;
    let actual = parts
        .algorithm_statements
        .get(id)
        .expect("validated statement id")
        .owner;
    if actual == expected {
        Ok(())
    } else {
        Err(CoreIrError::StatementOwnerMismatch {
            statement: id,
            expected,
            actual,
        })
    }
}

fn validate_statements_owned(
    ids: &[CoreAlgorithmStmtId],
    owner: CoreAlgorithmId,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    for id in ids {
        validate_statement_owner(*id, owner, parts)?;
    }
    Ok(())
}

fn validate_diagnostics(ids: &[CoreDiagnosticId], parts: &CoreIrParts) -> Result<(), CoreIrError> {
    for id in ids {
        validate_index("diagnostic", id.index(), parts.diagnostics.len())?;
    }
    Ok(())
}

fn validate_error_diagnostic(
    table: &'static str,
    index: usize,
    owner: CoreNodeRef,
    source: &CoreSourceRef,
    diagnostic: CoreDiagnosticId,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    validate_index("diagnostic", diagnostic.index(), parts.diagnostics.len())?;
    let diagnostic_row = parts
        .diagnostics
        .get(diagnostic)
        .expect("validated diagnostic id");
    if diagnostic_explains_node(diagnostic_row, &owner, source) {
        Ok(())
    } else {
        Err(CoreIrError::ErrorDiagnosticMismatch {
            table,
            index,
            diagnostic,
        })
    }
}

fn diagnostic_explains_node(
    diagnostic: &CoreDiagnostic,
    owner: &CoreNodeRef,
    source: &CoreSourceRef,
) -> bool {
    diagnostic.owner.as_ref() == Some(owner)
        || diagnostic.primary_source == *source
        || generated_from_owner(&diagnostic.primary_source) == Some(owner)
}

fn generated_from_owner(source: &CoreSourceRef) -> Option<&CoreNodeRef> {
    match &source.anchor {
        CoreSourceAnchor::SourceRange(_) => None,
        CoreSourceAnchor::GeneratedFrom(generated_from) => Some(&generated_from.owner),
    }
}

fn validate_source_entry(
    table: &'static str,
    index: usize,
    expected: &CoreSourceRef,
    actual: Option<&CoreSourceRef>,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    let Some(actual) = actual else {
        return Err(CoreIrError::MissingSourceMap { table, index });
    };
    if actual != expected {
        return Err(CoreIrError::SourceMapMismatch { table, index });
    }
    validate_source(table, index, expected, parts)
}

fn validate_source(
    table: &'static str,
    index: usize,
    source: &CoreSourceRef,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    match &source.anchor {
        CoreSourceAnchor::SourceRange(range) => {
            if range.source_id != parts.source_id {
                return Err(CoreIrError::InvalidSourceRange {
                    table,
                    index,
                    reason: "range belongs to another source",
                });
            }
            if range.start > range.end {
                return Err(CoreIrError::InvalidSourceRange {
                    table,
                    index,
                    reason: "start is after end",
                });
            }
        }
        CoreSourceAnchor::GeneratedFrom(generated_from) => {
            generated_from.owner.validate(parts)?;
            validate_generated_from(table, index, generated_from, parts)?;
        }
    }
    Ok(())
}

fn validate_generated_from(
    table: &'static str,
    index: usize,
    generated_from: &GeneratedFrom,
    parts: &CoreIrParts,
) -> Result<(), CoreIrError> {
    let CoreNodeRef::Item(owner) = &generated_from.owner else {
        return Ok(());
    };
    let has_origin = parts.generated.iter().any(|(_, origin)| {
        origin.owner == *owner
            && origin.kind == generated_from.kind
            && origin.key == generated_from.key
    });
    if has_origin {
        Ok(())
    } else {
        Err(CoreIrError::MissingGeneratedOriginForSource {
            table,
            index,
            kind: generated_from.kind,
            key: generated_from.key.clone(),
        })
    }
}

fn validate_generated_origin_uniqueness(parts: &CoreIrParts) -> Result<(), CoreIrError> {
    let mut seen =
        BTreeMap::<(CoreItemId, GeneratedOriginKind, GeneratedOriginKey), GeneratedOriginId>::new();
    for (id, origin) in parts.generated.iter() {
        let key = (origin.owner, origin.kind, origin.key.clone());
        if let Some(first) = seen.insert(key, id) {
            return Err(CoreIrError::DuplicateGeneratedOrigin {
                first,
                duplicate: id,
            });
        }
    }
    Ok(())
}

fn validate_source_map_domain<I>(
    table: &'static str,
    indices: I,
    len: usize,
) -> Result<(), CoreIrError>
where
    I: IntoIterator<Item = usize>,
{
    for index in indices {
        if index >= len {
            return Err(CoreIrError::InvalidSourceMapEntry { table, index, len });
        }
    }
    Ok(())
}

fn validate_index(table: &'static str, index: usize, len: usize) -> Result<(), CoreIrError> {
    if index < len {
        Ok(())
    } else {
        Err(CoreIrError::InvalidReference { table, index, len })
    }
}

fn write_header(output: &mut String, source_id: SourceId, module_id: &ModuleId) {
    let _ = writeln!(output, "source: {source_id:?}");
    let _ = writeln!(output, "module: {module_id:?}");
}

fn write_table<I, Id, Entry>(output: &mut String, name: &str, iter: I)
where
    I: IntoIterator<Item = (Id, Entry)>,
    Id: fmt::Debug,
    Entry: fmt::Debug,
{
    let _ = writeln!(output, "[{name}]");
    for (id, entry) in iter {
        let _ = writeln!(output, "{id:?}: {entry:?}");
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &CoreDiagnosticTable) {
    let _ = writeln!(output, "[diagnostics]");
    let mut rows = diagnostics.iter().collect::<Vec<_>>();
    rows.sort_by(|(left_id, left), (right_id, right)| {
        diagnostic_sort_key(*left_id, left).cmp(&diagnostic_sort_key(*right_id, right))
    });
    for (id, diagnostic) in rows {
        let _ = writeln!(output, "{id:?}: {diagnostic:?}");
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct CoreDiagnosticSortKey {
    source: CoreSourceSortKey,
    owner: Option<CoreNodeRef>,
    class: CoreDiagnosticClass,
    message_key: CoreDiagnosticMessageKey,
    id: CoreDiagnosticId,
}

fn diagnostic_sort_key(id: CoreDiagnosticId, diagnostic: &CoreDiagnostic) -> CoreDiagnosticSortKey {
    CoreDiagnosticSortKey {
        source: diagnostic.primary_source.sort_key(),
        owner: diagnostic.owner.clone(),
        class: diagnostic.class,
        message_key: diagnostic.message_key.clone(),
        id,
    }
}

fn write_source_map(output: &mut String, source_map: &CoreSourceMap) {
    let _ = writeln!(output, "[source-map]");
    let _ = writeln!(output, "items: {:?}", source_map.item_sources);
    let _ = writeln!(output, "terms: {:?}", source_map.term_sources);
    let _ = writeln!(output, "formulas: {:?}", source_map.formula_sources);
    let _ = writeln!(output, "definitions: {:?}", source_map.definition_sources);
    let _ = writeln!(output, "proofs: {:?}", source_map.proof_sources);
    let _ = writeln!(output, "algorithms: {:?}", source_map.algorithm_sources);
    let _ = writeln!(output, "generated: {:?}", source_map.generated_sources);
    let _ = writeln!(output, "obligations: {:?}", source_map.obligation_sources);
}

#[cfg(test)]
mod tests {
    use super::*;
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
    };

    fn source_id_for(hex_pair: &str) -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            hex_pair.repeat(32)
        ))
        .expect("valid snapshot id");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id")
    }

    fn source_id() -> SourceId {
        source_id_for("00")
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn symbol(name: &str) -> SymbolId {
        let module = module_id();
        SymbolId::new(
            module,
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::main::{name}")),
        )
    }

    fn direct(source_id: SourceId, start: usize, end: usize) -> CoreSourceRef {
        CoreSourceRef::direct(range(source_id, start, end))
    }

    fn minimal_parts() -> CoreIrParts {
        let source_id = source_id();
        let source = direct(source_id, 0, 5);
        let theorem = symbol("Th1");
        let predicate = symbol("P");

        let mut items = CoreItemTable::new();
        let item = items.insert(CoreItem::new(
            theorem,
            CoreItemKind::Theorem,
            "public",
            source.clone(),
        ));

        let mut terms = CoreTermTable::new();
        let term = terms.insert(CoreTerm::new(
            CoreTermKind::Var(CoreVarId::new(0)),
            source.clone(),
        ));

        let mut formulas = CoreFormulaTable::new();
        let formula = formulas.insert(CoreFormula::new(
            CoreFormulaKind::Atom {
                predicate,
                args: vec![term],
            },
            source.clone(),
        ));

        let mut seeds = ObligationSeedTable::new();
        let seed = seeds.insert(ObligationSeed {
            owner: item,
            kind: ObligationSeedKind::TheoremProof,
            goal: Some(formula),
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("proof/0"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Th1"),
            provenance: vec![CoreProvenance::new(
                CoreProvenancePhase::ProofSkeleton,
                "terminal",
            )],
            source: source.clone(),
            core_refs: vec![CoreNodeRef::Formula(formula)],
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        });

        let mut proof_nodes = CoreProofNodeTable::new();
        let proof_node = proof_nodes.insert(CoreProofNode {
            kind: CoreProofNodeKind::TerminalGoal(seed),
            source: source.clone(),
            diagnostics: Vec::new(),
        });

        let mut proofs = CoreProofTable::new();
        proofs.insert(CoreProof {
            item,
            proposition: formula,
            root: proof_node,
            status: CoreProofStatus::Open,
            source: source.clone(),
        });

        let mut source_map = CoreSourceMap::new();
        source_map.item_sources.insert(item, source.clone());
        source_map.term_sources.insert(term, source.clone());
        source_map.formula_sources.insert(formula, source.clone());
        source_map.proof_sources.insert(proof_node, source.clone());
        source_map.obligation_sources.insert(seed, source.clone());

        CoreIrParts {
            source_id,
            module_id: module_id(),
            items,
            terms,
            formulas,
            definitions: CoreDefinitionTable::new(),
            proofs,
            proof_nodes,
            algorithms: CoreAlgorithmTable::new(),
            algorithm_statements: CoreAlgorithmStmtTable::new(),
            generated: GeneratedOriginTable::new(),
            obligation_seeds: seeds,
            source_map,
            diagnostics: CoreDiagnosticTable::new(),
        }
    }

    fn assert_invalid_reference(parts: CoreIrParts, table: &'static str, index: usize) {
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::InvalidReference {
                table: actual_table,
                index: actual_index,
                ..
            }) if actual_table == table && actual_index == index
        ));
    }

    fn assert_ordered(text: &str, first: &str, second: &str) {
        let first_index = text
            .find(first)
            .unwrap_or_else(|| panic!("missing {first}"));
        let second_index = text
            .find(second)
            .unwrap_or_else(|| panic!("missing {second}"));
        assert!(
            first_index < second_index,
            "{first} should appear before {second}"
        );
    }

    fn assert_seed_before(
        order: &[ObligationSeedId],
        first: ObligationSeedId,
        second: ObligationSeedId,
    ) {
        let first_index = order
            .iter()
            .position(|id| *id == first)
            .unwrap_or_else(|| panic!("missing seed {first:?}"));
        let second_index = order
            .iter()
            .position(|id| *id == second)
            .unwrap_or_else(|| panic!("missing seed {second:?}"));
        assert!(
            first_index < second_index,
            "{first:?} should appear before {second:?}"
        );
    }

    fn generated_source(
        owner: CoreNodeRef,
        kind: GeneratedOriginKind,
        key: &str,
        reason: &str,
    ) -> CoreSourceRef {
        CoreSourceRef::generated(GeneratedFrom {
            owner,
            kind,
            key: GeneratedOriginKey::new(key),
            reason: CoreProvenanceKey::new(reason),
        })
    }

    fn expect_nested_owner_mismatch(
        nesting: impl FnOnce(CoreAlgorithmStmtId) -> CoreAlgorithmStmtKind,
    ) {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 96, 97);
        let algorithm = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("OwnerAlgorithm"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: Vec::new(),
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let other_algorithm = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("OtherAlgorithm"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: Vec::new(),
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let nested = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: other_algorithm,
            kind: CoreAlgorithmStmtKind::Break,
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let root = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: algorithm,
            kind: nesting(nested),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        parts
            .algorithms
            .get_mut(algorithm)
            .expect("algorithm")
            .statements = vec![root];
        parts
            .source_map
            .algorithm_sources
            .insert(nested, source.clone());
        parts.source_map.algorithm_sources.insert(root, source);

        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::StatementOwnerMismatch {
                statement: actual_statement,
                expected,
                actual
            }) if actual_statement == nested && expected == algorithm && actual == other_algorithm
        ));
    }

    #[test]
    fn minimal_core_ir_constructs_and_renders_deterministically() {
        let core = CoreIr::try_new(minimal_parts()).expect("valid core ir");
        let first = core.debug_text();
        let second = core.debug_text();

        assert_eq!(first, second);
        assert!(first.starts_with("core-ir-debug-v1\n"));
        assert!(first.contains("[items]"));
        assert!(first.contains("[obligation-seeds]"));
    }

    #[test]
    fn dense_ids_are_stable_for_inserted_nodes() {
        let source_id = source_id();
        let source = direct(source_id, 0, 1);
        let mut terms = CoreTermTable::new();

        let first = terms.insert(CoreTerm::new(
            CoreTermKind::Var(CoreVarId::new(0)),
            source.clone(),
        ));
        let second = terms.insert(CoreTerm::new(CoreTermKind::Var(CoreVarId::new(1)), source));

        assert_eq!(first.index(), 0);
        assert_eq!(second.index(), 1);
        assert_eq!(
            terms.iter().map(|(id, _)| id.index()).collect::<Vec<_>>(),
            vec![0, 1]
        );
    }

    #[test]
    fn validation_rejects_invalid_references_across_core_tables() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 6, 7);
        let mut item = CoreItem::new(
            symbol("BadDependency"),
            CoreItemKind::Theorem,
            "public",
            source.clone(),
        );
        item.dependencies.push(CoreItemId::new(99));
        let item_id = parts.items.insert(item);
        parts.source_map.item_sources.insert(item_id, source);
        assert_invalid_reference(parts, "item", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 7, 8);
        let term = parts.terms.insert(CoreTerm::new(
            CoreTermKind::Generated {
                origin: GeneratedOriginId::new(99),
                args: Vec::new(),
            },
            source.clone(),
        ));
        parts.source_map.term_sources.insert(term, source);
        assert_invalid_reference(parts, "generated", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 6, 7);
        let invalid = parts.formulas.insert(CoreFormula::new(
            CoreFormulaKind::Atom {
                predicate: symbol("Q"),
                args: vec![CoreTermId::new(99)],
            },
            source.clone(),
        ));
        parts.source_map.formula_sources.insert(invalid, source);
        assert_invalid_reference(parts, "term", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 8, 9);
        let definition = parts.definitions.insert(CoreDefinition {
            item: CoreItemId::new(0),
            symbol: symbol("BadDefinition"),
            params: Vec::new(),
            body: DefinitionBody::Term(CoreTermId::new(0)),
            expansion: ExpansionPolicy::Opaque,
            correctness: vec![ObligationSeedId::new(99)],
            generated_dependencies: Vec::new(),
            source: source.clone(),
        });
        parts
            .source_map
            .definition_sources
            .insert(definition, source);
        assert_invalid_reference(parts, "obligation seed", 99);

        let mut parts = minimal_parts();
        let proof_source = direct(parts.source_id, 9, 10);
        parts.proofs.insert(CoreProof {
            item: CoreItemId::new(0),
            proposition: CoreFormulaId::new(0),
            root: CoreProofNodeId::new(99),
            status: CoreProofStatus::Open,
            source: proof_source,
        });
        assert_invalid_reference(parts, "proof node", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 10, 11);
        let node = parts.proof_nodes.insert(CoreProofNode {
            kind: CoreProofNodeKind::TerminalGoal(ObligationSeedId::new(99)),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        parts.source_map.proof_sources.insert(node, source);
        assert_invalid_reference(parts, "obligation seed", 99);

        let mut parts = minimal_parts();
        let algorithm_source = direct(parts.source_id, 11, 12);
        parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("BadAlgorithm"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: vec![CoreAlgorithmStmtId::new(99)],
            ghost_effects: Vec::new(),
            source: algorithm_source,
            diagnostics: Vec::new(),
        });
        assert_invalid_reference(parts, "algorithm statement", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 12, 13);
        let statement = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: CoreAlgorithmId::new(99),
            kind: CoreAlgorithmStmtKind::Break,
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        parts.source_map.algorithm_sources.insert(statement, source);
        assert_invalid_reference(parts, "algorithm", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 13, 14);
        let generated = parts.generated.insert(GeneratedOrigin {
            owner: CoreItemId::new(99),
            kind: GeneratedOriginKind::FraenkelComprehension,
            key: GeneratedOriginKey::new("fraenkel_bad"),
            params: Vec::new(),
            evidence: Vec::new(),
            source: source.clone(),
        });
        parts.source_map.generated_sources.insert(generated, source);
        assert_invalid_reference(parts, "item", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 14, 15);
        let seed = parts.obligation_seeds.insert(ObligationSeed {
            owner: CoreItemId::new(0),
            kind: ObligationSeedKind::TheoremProof,
            goal: Some(CoreFormulaId::new(0)),
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("proof/bad-ref"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Th1"),
            provenance: vec![CoreProvenance::new(
                CoreProvenancePhase::ProofSkeleton,
                "bad",
            )],
            source: source.clone(),
            core_refs: vec![CoreNodeRef::Term(CoreTermId::new(99))],
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        });
        parts.source_map.obligation_sources.insert(seed, source);
        assert_invalid_reference(parts, "term", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 15, 16);
        let term = parts.terms.insert(CoreTerm::new(
            CoreTermKind::Error(CoreDiagnosticId::new(99)),
            source.clone(),
        ));
        parts.source_map.term_sources.insert(term, source);
        assert_invalid_reference(parts, "diagnostic", 99);

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 16, 17);
        let mut diagnostic = CoreDiagnostic::error(
            CoreDiagnosticClass::MalformedProofSkeleton,
            "bad-owner",
            source,
        );
        diagnostic.owner = Some(CoreNodeRef::Term(CoreTermId::new(99)));
        parts.diagnostics.insert(diagnostic);
        assert_invalid_reference(parts, "term", 99);
    }

    #[test]
    fn validation_rejects_source_map_and_anchor_drift() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 8, 9);
        let term = parts
            .terms
            .insert(CoreTerm::new(CoreTermKind::Const(symbol("c")), source));

        assert!(matches!(
            CoreIr::try_new(parts.clone()),
            Err(CoreIrError::MissingSourceMap { table: "term", index }) if index == term.index()
        ));

        let mut mismatched = minimal_parts();
        mismatched
            .source_map
            .term_sources
            .insert(CoreTermId::new(0), direct(mismatched.source_id, 20, 21));
        assert!(matches!(
            CoreIr::try_new(mismatched),
            Err(CoreIrError::SourceMapMismatch {
                table: "term",
                index: 0
            })
        ));

        let mut extra = minimal_parts();
        extra
            .source_map
            .term_sources
            .insert(CoreTermId::new(99), direct(extra.source_id, 21, 22));
        assert!(matches!(
            CoreIr::try_new(extra),
            Err(CoreIrError::InvalidSourceMapEntry {
                table: "term",
                index: 99,
                ..
            })
        ));

        let mut invalid_owner = minimal_parts();
        let bad_generated_source = generated_source(
            CoreNodeRef::Term(CoreTermId::new(99)),
            GeneratedOriginKind::LocalAbbreviation,
            "bad-source",
            "invalid generated owner",
        );
        let term = invalid_owner.terms.insert(CoreTerm::new(
            CoreTermKind::Const(symbol("bad_source")),
            bad_generated_source.clone(),
        ));
        invalid_owner
            .source_map
            .term_sources
            .insert(term, bad_generated_source);
        assert_invalid_reference(invalid_owner, "term", 99);

        let mut invalid_range = minimal_parts();
        let bad_range = direct(invalid_range.source_id, 30, 29);
        let term = invalid_range.terms.insert(CoreTerm::new(
            CoreTermKind::Const(symbol("bad_range")),
            bad_range.clone(),
        ));
        invalid_range
            .source_map
            .term_sources
            .insert(term, bad_range);
        assert!(matches!(
            CoreIr::try_new(invalid_range),
            Err(CoreIrError::InvalidSourceRange {
                table: "term",
                reason: "start is after end",
                ..
            })
        ));

        let mut stable_choice_term = minimal_parts();
        let source = direct(stable_choice_term.source_id, 32, 33);
        let origin = stable_choice_term.generated.insert(GeneratedOrigin {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::StableChoice,
            key: GeneratedOriginKey::new("choice_T"),
            params: Vec::new(),
            evidence: Vec::new(),
            source: source.clone(),
        });
        stable_choice_term
            .source_map
            .generated_sources
            .insert(origin, source.clone());
        let term = stable_choice_term.terms.insert(CoreTerm::new(
            CoreTermKind::Generated {
                origin,
                args: Vec::new(),
            },
            source.clone(),
        ));
        stable_choice_term
            .source_map
            .term_sources
            .insert(term, source);
        assert!(matches!(
            CoreIr::try_new(stable_choice_term),
            Err(CoreIrError::StableChoiceGeneratedTerm {
                term: actual_term,
                origin: actual_origin
            }) if actual_term == term && actual_origin == origin
        ));

        let choice_source = generated_source(
            CoreNodeRef::Item(CoreItemId::new(0)),
            GeneratedOriginKind::StableChoice,
            "choice_T",
            "stable choice",
        );
        let generated = parts.terms.insert(CoreTerm::new(
            CoreTermKind::Apply {
                functor: symbol("choice_T"),
                args: Vec::new(),
            },
            choice_source.clone(),
        ));
        parts
            .source_map
            .term_sources
            .insert(term, direct(parts.source_id, 8, 9));
        parts
            .source_map
            .term_sources
            .insert(generated, choice_source.clone());
        let origin = parts.generated.insert(GeneratedOrigin {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::StableChoice,
            key: GeneratedOriginKey::new("choice_T"),
            params: Vec::new(),
            evidence: Vec::new(),
            source: choice_source.clone(),
        });
        parts
            .source_map
            .generated_sources
            .insert(origin, choice_source);

        assert!(CoreIr::try_new(parts).is_ok());
    }

    #[test]
    fn validation_rejects_non_term_source_map_drift() {
        let mut parts = minimal_parts();
        parts
            .source_map
            .item_sources
            .insert(CoreItemId::new(0), direct(parts.source_id, 32, 33));
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::SourceMapMismatch {
                table: "item",
                index: 0
            })
        ));

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 33, 34);
        let definition = parts.definitions.insert(CoreDefinition {
            item: CoreItemId::new(0),
            symbol: symbol("MissingDefinitionSource"),
            params: Vec::new(),
            body: DefinitionBody::Term(CoreTermId::new(0)),
            expansion: ExpansionPolicy::Opaque,
            correctness: Vec::new(),
            generated_dependencies: Vec::new(),
            source,
        });
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::MissingSourceMap {
                table: "definition",
                index
            }) if index == definition.index()
        ));

        let mut parts = minimal_parts();
        parts
            .source_map
            .formula_sources
            .remove(&CoreFormulaId::new(0));
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::MissingSourceMap {
                table: "formula",
                index: 0
            })
        ));

        let mut parts = minimal_parts();
        parts
            .source_map
            .formula_sources
            .insert(CoreFormulaId::new(0), direct(parts.source_id, 34, 35));
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::SourceMapMismatch {
                table: "formula",
                index: 0
            })
        ));

        let mut parts = minimal_parts();
        parts
            .source_map
            .proof_sources
            .insert(CoreProofNodeId::new(0), direct(parts.source_id, 35, 36));
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::SourceMapMismatch {
                table: "proof",
                index: 0
            })
        ));

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 36, 37);
        let algorithm = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("MissingStatementSource"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: vec![CoreAlgorithmStmtId::new(0)],
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let statement = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: algorithm,
            kind: CoreAlgorithmStmtKind::Break,
            source,
            diagnostics: Vec::new(),
        });
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::MissingSourceMap {
                table: "algorithm statement",
                index
            }) if index == statement.index()
        ));

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 37, 38);
        let algorithm = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("MismatchedStatementSource"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: vec![CoreAlgorithmStmtId::new(0)],
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let statement = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: algorithm,
            kind: CoreAlgorithmStmtKind::Break,
            source,
            diagnostics: Vec::new(),
        });
        parts
            .source_map
            .algorithm_sources
            .insert(statement, direct(parts.source_id, 38, 39));
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::SourceMapMismatch {
                table: "algorithm statement",
                index
            }) if index == statement.index()
        ));

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 39, 40);
        let generated = parts.generated.insert(GeneratedOrigin {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("missing-source"),
            params: Vec::new(),
            evidence: Vec::new(),
            source,
        });
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::MissingSourceMap {
                table: "generated",
                index
            }) if index == generated.index()
        ));

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 40, 41);
        let generated = parts.generated.insert(GeneratedOrigin {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("mismatched-source"),
            params: Vec::new(),
            evidence: Vec::new(),
            source,
        });
        parts
            .source_map
            .generated_sources
            .insert(generated, direct(parts.source_id, 41, 42));
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::SourceMapMismatch {
                table: "generated",
                index
            }) if index == generated.index()
        ));

        let mut parts = minimal_parts();
        parts.source_map.generated_sources.insert(
            GeneratedOriginId::new(99),
            generated_source(
                CoreNodeRef::Item(CoreItemId::new(0)),
                GeneratedOriginKind::LocalAbbreviation,
                "extra",
                "extra",
            ),
        );
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::MissingGeneratedOriginForSource {
                table: "generated",
                index: 99,
                ..
            }) | Err(CoreIrError::InvalidSourceMapEntry {
                table: "generated",
                index: 99,
                ..
            })
        ));

        let mut parts = minimal_parts();
        parts
            .source_map
            .obligation_sources
            .insert(ObligationSeedId::new(0), direct(parts.source_id, 35, 36));
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::SourceMapMismatch {
                table: "obligation seed",
                index: 0
            })
        ));
    }

    #[test]
    fn validation_rejects_algorithm_owner_and_generated_origin_drift() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 36, 37);
        let algorithm_a = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("AlgorithmA"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: vec![CoreAlgorithmStmtId::new(0)],
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let algorithm_b = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("AlgorithmB"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: Vec::new(),
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let statement = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: algorithm_b,
            kind: CoreAlgorithmStmtKind::Break,
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        parts.source_map.algorithm_sources.insert(statement, source);
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::StatementOwnerMismatch {
                statement: actual_statement,
                expected,
                actual
            }) if actual_statement == statement && expected == algorithm_a && actual == algorithm_b
        ));

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 37, 38);
        let algorithm = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("AlgorithmNested"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: vec![CoreAlgorithmStmtId::new(0)],
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let other_algorithm = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("AlgorithmOther"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: Vec::new(),
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let nested = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: other_algorithm,
            kind: CoreAlgorithmStmtKind::Break,
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        let root = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: algorithm,
            kind: CoreAlgorithmStmtKind::If {
                condition: CoreFormulaId::new(0),
                then_body: vec![nested],
                else_body: Vec::new(),
            },
            source: source.clone(),
            diagnostics: Vec::new(),
        });
        parts
            .algorithms
            .get_mut(algorithm)
            .expect("algorithm")
            .statements = vec![root];
        parts
            .source_map
            .algorithm_sources
            .insert(nested, source.clone());
        parts.source_map.algorithm_sources.insert(root, source);
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::StatementOwnerMismatch {
                statement: actual_statement,
                expected,
                actual
            }) if actual_statement == nested && expected == algorithm && actual == other_algorithm
        ));

        expect_nested_owner_mismatch(|nested| CoreAlgorithmStmtKind::While {
            condition: CoreFormulaId::new(0),
            invariants: Vec::new(),
            decreasing: Vec::new(),
            body: vec![nested],
        });
        expect_nested_owner_mismatch(|nested| CoreAlgorithmStmtKind::Match {
            scrutinee: CoreTermId::new(0),
            arms: vec![CoreAlgorithmMatchArm {
                pattern: CoreProvenanceKey::new("case"),
                body: vec![nested],
            }],
        });

        let mut parts = minimal_parts();
        let generated = generated_source(
            CoreNodeRef::Item(CoreItemId::new(0)),
            GeneratedOriginKind::LocalAbbreviation,
            "missing-origin",
            "term source",
        );
        let term = parts.terms.insert(CoreTerm::new(
            CoreTermKind::Apply {
                functor: symbol("missing_origin"),
                args: Vec::new(),
            },
            generated.clone(),
        ));
        parts.source_map.term_sources.insert(term, generated);
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::MissingGeneratedOriginForSource {
                table: "term",
                index,
                ..
            }) if index == term.index()
        ));

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 38, 39);
        let first = parts.generated.insert(GeneratedOrigin {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("dup"),
            params: Vec::new(),
            evidence: Vec::new(),
            source: source.clone(),
        });
        let duplicate = parts.generated.insert(GeneratedOrigin {
            owner: CoreItemId::new(0),
            kind: GeneratedOriginKind::LocalAbbreviation,
            key: GeneratedOriginKey::new("dup"),
            params: Vec::new(),
            evidence: Vec::new(),
            source: source.clone(),
        });
        parts
            .source_map
            .generated_sources
            .insert(first, source.clone());
        parts.source_map.generated_sources.insert(duplicate, source);
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::DuplicateGeneratedOrigin {
                first: actual_first,
                duplicate: actual_duplicate
            }) if actual_first == first && actual_duplicate == duplicate
        ));
    }

    #[test]
    fn obligation_seed_goal_invariants_are_enforced() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 10, 11);
        let active_without_goal = parts.obligation_seeds.insert(ObligationSeed {
            owner: CoreItemId::new(0),
            kind: ObligationSeedKind::TheoremProof,
            goal: None,
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("proof/1"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Th1"),
            provenance: Vec::new(),
            source: source.clone(),
            core_refs: Vec::new(),
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        });
        parts
            .source_map
            .obligation_sources
            .insert(active_without_goal, source);

        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::ActiveObligationSeedWithoutGoal { seed }) if seed == active_without_goal
        ));

        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 11, 12);
        let skipped_without_reason = parts.obligation_seeds.insert(ObligationSeed {
            owner: CoreItemId::new(0),
            kind: ObligationSeedKind::AlgorithmTermination,
            goal: None,
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("program/loop/0"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Th1.loop"),
            provenance: Vec::new(),
            source: source.clone(),
            core_refs: Vec::new(),
            status: ObligationSeedStatus::Skipped,
            diagnostics: Vec::new(),
        });
        parts
            .source_map
            .obligation_sources
            .insert(skipped_without_reason, source);

        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::InactiveObligationSeedWithoutReason { seed })
                if seed == skipped_without_reason
        ));
    }

    #[test]
    fn obligation_seed_canonical_order_uses_paths_and_origins() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 12, 13);
        let late = parts.obligation_seeds.insert(ObligationSeed {
            owner: CoreItemId::new(0),
            kind: ObligationSeedKind::DefinitionCorrectness,
            goal: Some(CoreFormulaId::new(0)),
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("z"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new("late"),
            provenance: vec![CoreProvenance::new(CoreProvenancePhase::Generated, "late")],
            source: source.clone(),
            core_refs: Vec::new(),
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        });
        let early = parts.obligation_seeds.insert(ObligationSeed {
            owner: CoreItemId::new(0),
            kind: ObligationSeedKind::DefinitionCorrectness,
            goal: Some(CoreFormulaId::new(0)),
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("a"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new("early"),
            provenance: vec![CoreProvenance::new(CoreProvenancePhase::Generated, "early")],
            source: source.clone(),
            core_refs: Vec::new(),
            status: ObligationSeedStatus::Active,
            diagnostics: Vec::new(),
        });
        parts
            .source_map
            .obligation_sources
            .insert(late, source.clone());
        parts.source_map.obligation_sources.insert(early, source);

        let core = CoreIr::try_new(parts).expect("valid core ir");
        let order = core.obligation_seeds().canonical_order();

        assert_seed_before(&order, early, late);
    }

    #[test]
    fn obligation_seed_canonical_order_covers_all_sort_keys() {
        let mut parts = minimal_parts();
        let source_id = parts.source_id;
        let source = direct(source_id, 50, 51);
        let mut second_item = CoreItem::new(
            symbol("Second"),
            CoreItemKind::Lemma,
            "public",
            source.clone(),
        );
        second_item.status = CoreItemStatus::Partial;
        let item1 = parts.items.insert(second_item);
        parts.source_map.item_sources.insert(item1, source.clone());

        let insert_seed = |parts: &mut CoreIrParts,
                           owner: CoreItemId,
                           source: CoreSourceRef,
                           path: &str,
                           kind: ObligationSeedKind,
                           label: Option<&str>,
                           origin: &str| {
            let seed = parts.obligation_seeds.insert(ObligationSeed {
                owner,
                kind,
                goal: Some(CoreFormulaId::new(0)),
                context: Vec::new(),
                local_path: LocalProofOrProgramPath::new(path),
                label: label.map(CoreLabelRef::new),
                semantic_origin: NormalizedSemanticOrigin::new(origin),
                provenance: vec![CoreProvenance::new(CoreProvenancePhase::Generated, origin)],
                source: source.clone(),
                core_refs: Vec::new(),
                status: ObligationSeedStatus::Active,
                diagnostics: Vec::new(),
            });
            parts.source_map.obligation_sources.insert(seed, source);
            seed
        };

        let owner_late = insert_seed(
            &mut parts,
            item1,
            direct(source_id, 60, 61),
            "k",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "owner",
        );
        let owner_early = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 60, 61),
            "k",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "owner",
        );
        let source_late = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 62, 63),
            "source",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "source",
        );
        let source_early = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 61, 62),
            "source",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "source",
        );
        let path_late = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 64, 65),
            "z",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "path",
        );
        let path_early = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 64, 65),
            "a",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "path",
        );
        let kind_late = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 66, 67),
            "kind",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "kind",
        );
        let kind_early = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 66, 67),
            "kind",
            ObligationSeedKind::GhostErasure,
            Some("A"),
            "kind",
        );
        let label_late = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 68, 69),
            "label",
            ObligationSeedKind::TheoremProof,
            Some("B"),
            "label",
        );
        let label_early = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 68, 69),
            "label",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "label",
        );
        let origin_late = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 70, 71),
            "origin",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "z-origin",
        );
        let origin_early = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 70, 71),
            "origin",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "a-origin",
        );
        let dense_early = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 72, 73),
            "tie",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "tie",
        );
        let dense_late = insert_seed(
            &mut parts,
            CoreItemId::new(0),
            direct(source_id, 72, 73),
            "tie",
            ObligationSeedKind::TheoremProof,
            Some("A"),
            "tie",
        );

        let core = CoreIr::try_new(parts).expect("valid core ir");
        let order = core.obligation_seeds().canonical_order();

        assert_seed_before(&order, owner_early, owner_late);
        assert_seed_before(&order, source_early, source_late);
        assert_seed_before(&order, path_early, path_late);
        assert_seed_before(&order, kind_early, kind_late);
        assert_seed_before(&order, label_early, label_late);
        assert_seed_before(&order, origin_early, origin_late);
        assert_seed_before(&order, dense_early, dense_late);
    }

    #[test]
    fn obligation_seed_status_and_metadata_are_preserved() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 20, 21).with_provenance(vec![CoreProvenance::new(
            CoreProvenancePhase::Generated,
            "generated-source",
        )]);
        let diagnostic = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::AlgorithmShell,
            "deferred-contract",
            source.clone(),
        ));
        let provenance = CoreProvenance::new(CoreProvenancePhase::Checker, "contract-post");
        let deferred = parts.obligation_seeds.insert(ObligationSeed {
            owner: CoreItemId::new(0),
            kind: ObligationSeedKind::AlgorithmContract,
            goal: None,
            context: vec![CoreFormulaId::new(0)],
            local_path: LocalProofOrProgramPath::new("program/post/0"),
            label: Some(CoreLabelRef::new("A1")),
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Th1.post"),
            provenance: vec![provenance.clone()],
            source: source.clone(),
            core_refs: vec![
                CoreNodeRef::Item(CoreItemId::new(0)),
                CoreNodeRef::Formula(CoreFormulaId::new(0)),
                CoreNodeRef::Diagnostic(diagnostic),
            ],
            status: ObligationSeedStatus::Deferred,
            diagnostics: vec![diagnostic],
        });
        parts
            .source_map
            .obligation_sources
            .insert(deferred, source.clone());
        let error_source = direct(parts.source_id, 21, 22);
        let error_seed = parts.obligation_seeds.insert(ObligationSeed {
            owner: CoreItemId::new(0),
            kind: ObligationSeedKind::GeneratedSethood,
            goal: None,
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("generated/error"),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Th1.generated"),
            provenance: Vec::new(),
            source: error_source.clone(),
            core_refs: vec![CoreNodeRef::Diagnostic(diagnostic)],
            status: ObligationSeedStatus::Error,
            diagnostics: vec![diagnostic],
        });
        parts
            .source_map
            .obligation_sources
            .insert(error_seed, error_source);

        let core = CoreIr::try_new(parts).expect("deferred obligation has explicit reason");
        let seed = core
            .obligation_seeds()
            .get(deferred)
            .expect("inserted seed");

        assert_eq!(seed.goal, None);
        assert_eq!(seed.context, vec![CoreFormulaId::new(0)]);
        assert_eq!(seed.local_path.as_str(), "program/post/0");
        assert_eq!(seed.label.as_ref().map(CoreLabelRef::as_str), Some("A1"));
        assert_eq!(seed.semantic_origin.as_str(), "pkg::main::Th1.post");
        assert_eq!(seed.provenance, vec![provenance]);
        assert_eq!(seed.source, source);
        assert_eq!(seed.status, ObligationSeedStatus::Deferred);
        assert_eq!(seed.diagnostics, vec![diagnostic]);
        assert!(
            core.obligation_seeds()
                .canonical_order()
                .contains(&deferred)
        );
        assert_eq!(
            core.obligation_seeds()
                .get(error_seed)
                .expect("error seed")
                .status,
            ObligationSeedStatus::Error
        );
    }

    #[test]
    fn debug_rendering_includes_ordered_tables_maps_and_recovery_statuses() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 30, 31);
        let mut partial_item = CoreItem::new(
            symbol("PartialItem"),
            CoreItemKind::Lemma,
            "public",
            source.clone(),
        );
        partial_item.status = CoreItemStatus::Partial;
        let item = parts.items.insert(partial_item);
        parts.source_map.item_sources.insert(item, source.clone());
        let second_term = parts.terms.insert(CoreTerm::new(
            CoreTermKind::Const(symbol("c2")),
            source.clone(),
        ));
        parts
            .source_map
            .term_sources
            .insert(second_term, source.clone());
        let diagnostic = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::UnsupportedLowering,
            "skipped",
            source.clone(),
        ));
        let skipped = parts.obligation_seeds.insert(ObligationSeed {
            owner: CoreItemId::new(0),
            kind: ObligationSeedKind::GhostErasure,
            goal: None,
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new("debug/skipped"),
            label: Some(CoreLabelRef::new("S1")),
            semantic_origin: NormalizedSemanticOrigin::new("pkg::main::Th1.debug"),
            provenance: vec![CoreProvenance::new(CoreProvenancePhase::Erasure, "ghost")],
            source: source.clone(),
            core_refs: vec![CoreNodeRef::Item(item), CoreNodeRef::Term(second_term)],
            status: ObligationSeedStatus::Skipped,
            diagnostics: vec![diagnostic],
        });
        parts.source_map.obligation_sources.insert(skipped, source);

        let core = CoreIr::try_new(parts).expect("debug fixture is valid");
        let text = core.debug_text();
        let source_map = &text[text.find("[source-map]").expect("source map section")..];

        assert_ordered(&text, "CoreTermId(0)", "CoreTermId(1)");
        assert_ordered(source_map, "CoreTermId(0)", "CoreTermId(1)");
        assert!(text.contains("Partial"));
        assert!(text.contains("Skipped"));
        assert!(text.contains("CoreProvenance"));
        assert!(text.contains("debug/skipped"));
        assert!(text.contains("S1"));
    }

    #[test]
    fn provenance_and_diagnostics_are_canonicalized_for_debug() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 80, 81).with_provenance(vec![
            CoreProvenance::new(CoreProvenancePhase::Generated, "omega"),
            CoreProvenance::new(CoreProvenancePhase::Resolver, "alpha"),
        ]);
        let term = parts.terms.insert(CoreTerm::new(
            CoreTermKind::Const(symbol("ordered_source")),
            source.clone(),
        ));
        parts.source_map.term_sources.insert(term, source);
        let late_source = direct(parts.source_id, 90, 91);
        let late = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::UnsupportedLowering,
            "same-source-key",
            late_source,
        ));
        let early_source = direct(parts.source_id, 82, 83);
        let early = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::UnsupportedLowering,
            "same-source-key",
            early_source,
        ));
        let owner_source = direct(parts.source_id, 100, 101);
        let mut owner_b = CoreDiagnostic::error(
            CoreDiagnosticClass::InvalidErasure,
            "same-owner-key",
            owner_source.clone(),
        );
        owner_b.owner = Some(CoreNodeRef::Term(CoreTermId::new(0)));
        let owner_b = parts.diagnostics.insert(owner_b);
        let mut owner_a = CoreDiagnostic::error(
            CoreDiagnosticClass::InvalidErasure,
            "same-owner-key",
            owner_source,
        );
        owner_a.owner = Some(CoreNodeRef::Item(CoreItemId::new(0)));
        let owner_a = parts.diagnostics.insert(owner_a);
        let class_source = direct(parts.source_id, 102, 103);
        let class_late = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::UnsupportedLowering,
            "same-class-key",
            class_source.clone(),
        ));
        let class_early = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::InvalidErasure,
            "same-class-key",
            class_source,
        ));
        let message_source = direct(parts.source_id, 104, 105);
        let message_late = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::SourceMapping,
            "z-message",
            message_source.clone(),
        ));
        let message_early = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::SourceMapping,
            "a-message",
            message_source,
        ));
        let dense_source = direct(parts.source_id, 106, 107);
        let dense_early = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::AlgorithmShell,
            "same-message",
            dense_source.clone(),
        ));
        let dense_late = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::AlgorithmShell,
            "same-message",
            dense_source,
        ));
        let mut related = CoreDiagnostic::error(
            CoreDiagnosticClass::SourceMapping,
            "related-order",
            direct(parts.source_id, 108, 109),
        );
        related.related = vec![
            direct(parts.source_id, 113, 114).with_provenance(vec![CoreProvenance::new(
                CoreProvenancePhase::Generated,
                "late-related",
            )]),
            direct(parts.source_id, 110, 111).with_provenance(vec![CoreProvenance::new(
                CoreProvenancePhase::Resolver,
                "early-related",
            )]),
        ];
        let related = parts.diagnostics.insert(related);

        let core = CoreIr::try_new(parts).expect("valid ordered debug fixture");
        let term_source = &core.terms().get(term).expect("term").source;
        assert_eq!(
            term_source.provenance,
            vec![
                CoreProvenance::new(CoreProvenancePhase::Resolver, "alpha"),
                CoreProvenance::new(CoreProvenancePhase::Generated, "omega"),
            ]
        );

        let text = core.debug_text();
        assert_ordered(&text, "Resolver", "Generated");
        let diagnostics = &text[text.find("[diagnostics]").expect("diagnostics section")..];
        assert_ordered(diagnostics, &format!("{early:?}:"), &format!("{late:?}:"));
        assert_ordered(
            diagnostics,
            &format!("{owner_a:?}:"),
            &format!("{owner_b:?}:"),
        );
        assert_ordered(
            diagnostics,
            &format!("{class_early:?}:"),
            &format!("{class_late:?}:"),
        );
        assert_ordered(
            diagnostics,
            &format!("{message_early:?}:"),
            &format!("{message_late:?}:"),
        );
        assert_ordered(
            diagnostics,
            &format!("{dense_early:?}:"),
            &format!("{dense_late:?}:"),
        );
        let related_row = core.diagnostics().get(related).expect("related diagnostic");
        assert_eq!(related_row.related[0].sort_key().start, 110);
        assert_eq!(related_row.related[1].sort_key().start, 113);
        assert_ordered(diagnostics, "early-related", "late-related");
    }

    #[test]
    fn error_node_diagnostics_must_explain_the_failed_site() {
        let mut parts = minimal_parts();
        let term_source = direct(parts.source_id, 92, 93);
        let unrelated_source = direct(parts.source_id, 94, 95);
        let diagnostic = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::UnsupportedLowering,
            "unrelated",
            unrelated_source,
        ));
        let term = parts.terms.insert(CoreTerm::new(
            CoreTermKind::Error(diagnostic),
            term_source.clone(),
        ));
        parts.source_map.term_sources.insert(term, term_source);

        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::ErrorDiagnosticMismatch {
                table: "term",
                index,
                diagnostic: actual_diagnostic
            }) if index == term.index() && actual_diagnostic == diagnostic
        ));

        let mut parts = minimal_parts();
        let formula_source = direct(parts.source_id, 93, 94);
        let unrelated_source = direct(parts.source_id, 94, 95);
        let diagnostic = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::UnsupportedLowering,
            "unrelated-formula",
            unrelated_source,
        ));
        let formula = parts.formulas.insert(CoreFormula::new(
            CoreFormulaKind::Error(diagnostic),
            formula_source.clone(),
        ));
        parts
            .source_map
            .formula_sources
            .insert(formula, formula_source);
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::ErrorDiagnosticMismatch {
                table: "formula",
                index,
                diagnostic: actual_diagnostic
            }) if index == formula.index() && actual_diagnostic == diagnostic
        ));

        let mut parts = minimal_parts();
        let proof_source = direct(parts.source_id, 95, 96);
        let unrelated_source = direct(parts.source_id, 96, 97);
        let diagnostic = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::MalformedProofSkeleton,
            "unrelated-proof",
            unrelated_source,
        ));
        let node = parts.proof_nodes.insert(CoreProofNode {
            kind: CoreProofNodeKind::Error(diagnostic),
            source: proof_source.clone(),
            diagnostics: Vec::new(),
        });
        parts.source_map.proof_sources.insert(node, proof_source);
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::ErrorDiagnosticMismatch {
                table: "proof",
                index,
                diagnostic: actual_diagnostic
            }) if index == node.index() && actual_diagnostic == diagnostic
        ));

        let mut parts = minimal_parts();
        let statement_source = direct(parts.source_id, 97, 98);
        let unrelated_source = direct(parts.source_id, 98, 99);
        let diagnostic = parts.diagnostics.insert(CoreDiagnostic::error(
            CoreDiagnosticClass::AlgorithmShell,
            "unrelated-statement",
            unrelated_source,
        ));
        let algorithm = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("BadStatementDiagnostic"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: vec![CoreAlgorithmStmtId::new(0)],
            ghost_effects: Vec::new(),
            source: statement_source.clone(),
            diagnostics: Vec::new(),
        });
        let statement = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: algorithm,
            kind: CoreAlgorithmStmtKind::Error(diagnostic),
            source: statement_source.clone(),
            diagnostics: Vec::new(),
        });
        parts
            .source_map
            .algorithm_sources
            .insert(statement, statement_source);
        assert!(matches!(
            CoreIr::try_new(parts),
            Err(CoreIrError::ErrorDiagnosticMismatch {
                table: "algorithm statement",
                index,
                diagnostic: actual_diagnostic
            }) if index == statement.index() && actual_diagnostic == diagnostic
        ));
    }

    #[test]
    fn error_nodes_remain_explicit_and_non_verified() {
        let mut parts = minimal_parts();
        let source = direct(parts.source_id, 40, 41);
        let diagnostic_id = CoreDiagnosticId::new(parts.diagnostics.len());
        let error_term_id = CoreTermId::new(parts.terms.len());
        let mut diagnostic = CoreDiagnostic::error(
            CoreDiagnosticClass::UnsupportedLowering,
            "unsupported",
            source.clone(),
        );
        diagnostic.owner = Some(CoreNodeRef::Term(error_term_id));
        let diagnostic = parts.diagnostics.insert(diagnostic);

        let error_term = parts.terms.insert(CoreTerm::new(
            CoreTermKind::Error(diagnostic),
            source.clone(),
        ));
        assert_eq!(error_term, error_term_id);
        parts
            .source_map
            .term_sources
            .insert(error_term, source.clone());

        let error_formula = parts.formulas.insert(CoreFormula::new(
            CoreFormulaKind::Error(diagnostic),
            source.clone(),
        ));
        parts
            .source_map
            .formula_sources
            .insert(error_formula, source.clone());

        let proof_node = parts.proof_nodes.insert(CoreProofNode {
            kind: CoreProofNodeKind::Error(diagnostic),
            source: source.clone(),
            diagnostics: vec![diagnostic],
        });
        parts
            .source_map
            .proof_sources
            .insert(proof_node, source.clone());

        let algorithm_id = CoreAlgorithmId::new(parts.algorithms.len());
        let statement_id = CoreAlgorithmStmtId::new(parts.algorithm_statements.len());
        let algorithm = parts.algorithms.insert(CoreAlgorithm {
            item: CoreItemId::new(0),
            symbol: symbol("RecoveringAlgorithm"),
            params: Vec::new(),
            result: None,
            contracts: CoreContractSet::default(),
            statements: vec![statement_id],
            ghost_effects: Vec::new(),
            source: source.clone(),
            diagnostics: vec![diagnostic],
        });
        assert_eq!(algorithm, algorithm_id);
        let statement = parts.algorithm_statements.insert(CoreAlgorithmStmt {
            owner: algorithm,
            kind: CoreAlgorithmStmtKind::Error(diagnostic),
            source: source.clone(),
            diagnostics: vec![diagnostic],
        });
        assert_eq!(statement, statement_id);
        parts
            .source_map
            .algorithm_sources
            .insert(statement, source.clone());

        let mut skipped_item = CoreItem::new(
            symbol("SkippedItem"),
            CoreItemKind::Theorem,
            "public",
            source.clone(),
        );
        skipped_item.status = CoreItemStatus::Skipped;
        skipped_item.diagnostics.push(diagnostic);
        let item = parts.items.insert(skipped_item);
        parts.source_map.item_sources.insert(item, source);

        let core = CoreIr::try_new(parts).expect("error nodes are valid recovery data");

        assert!(core.has_error_nodes());
        assert!(matches!(
            core.formulas().get(error_formula).expect("formula").kind,
            CoreFormulaKind::Error(id) if id == diagnostic_id
        ));
        assert!(matches!(
            core.proof_nodes().get(proof_node).expect("proof node").kind,
            CoreProofNodeKind::Error(id) if id == diagnostic
        ));
        assert!(matches!(
            core.algorithm_statements()
                .get(statement)
                .expect("statement")
                .kind,
            CoreAlgorithmStmtKind::Error(id) if id == diagnostic
        ));
        assert_eq!(
            core.diagnostics()
                .get(diagnostic)
                .expect("diagnostic")
                .owner,
            Some(CoreNodeRef::Term(error_term))
        );
        assert_eq!(
            core.items().get(item).expect("item").status,
            CoreItemStatus::Skipped
        );
    }
}
