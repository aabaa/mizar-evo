//! Control-flow IR construction.
//!
//! Implements the phase-10 CFG construction contract specified in
//! [control_flow.md](../../../../doc/design/mizar-core/en/control_flow.md).

use crate::core_ir::{
    CoreAlgorithm, CoreAlgorithmId, CoreAlgorithmMatchArm, CoreAlgorithmStmt, CoreAlgorithmStmtId,
    CoreAlgorithmStmtKind, CoreBinder, CoreDiagnosticId, CoreFormulaId, CoreFormulaKind, CoreIr,
    CoreItemId, CoreNodeRef, CorePlace, CoreProvenance, CoreProvenanceKey, CoreProvenancePhase,
    CoreSourceAnchor, CoreSourceRef, CoreTermId, CoreTermKind, CoreVarId, CoreVarRole,
    GhostEffectKey, LocalProofOrProgramPath, NormalizedSemanticOrigin, ObligationSeed,
    ObligationSeedId, ObligationSeedKind, ObligationSeedStatus,
};
use mizar_resolve::resolved_ast::SymbolId;
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

dense_id!(ControlFlowId);
dense_id!(BasicBlockId);
dense_id!(LocalId);
dense_id!(LoopId);
dense_id!(ControlFlowExitId);
dense_id!(ProgramContextId);
dense_id!(ContextFactId);
dense_id!(AssignmentEffectId);
dense_id!(CallSiteId);
dense_id!(ControlFlowDiagnosticId);
dense_id!(ObligationHandoffId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowOutput {
    pub flows: ControlFlowTable,
    pub flow_map: BTreeMap<CoreAlgorithmId, ControlFlowId>,
}

impl ControlFlowOutput {
    pub fn debug_text(&self) -> String {
        let mut output = String::from("control-flow-output-debug-v1\n");
        writeln!(&mut output, "flow-map: {:?}", self.flow_map).expect("write string");
        for (id, flow) in self.flows.iter() {
            writeln!(&mut output, "[flow {id:?}]").expect("write string");
            output.push_str(&flow.debug_text());
        }
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationSeedHandoff {
    pub entries: ObligationHandoffTable,
    pub source_map: BTreeMap<ObligationHandoffId, CoreSourceRef>,
}

impl ObligationSeedHandoff {
    pub fn debug_text(&self) -> String {
        let mut output = String::from("obligation-seed-handoff-debug-v1\n");
        write_table(&mut output, "entries", self.entries.iter());
        writeln!(&mut output, "source-map: {:?}", self.source_map).expect("write string");
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObligationHandoffEntry {
    pub seed: ObligationSeed,
    pub origin: ObligationHandoffOrigin,
    pub flow_site: Option<ControlFlowObligationSite>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum ObligationHandoffOrigin {
    ExistingCore {
        seed: ObligationSeedId,
    },
    FlowDerived {
        flow: ControlFlowId,
        algorithm: CoreAlgorithmId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct ControlFlowObligationSite {
    pub kind: ControlFlowObligationSiteKind,
    pub ordinal: usize,
    pub statement: Option<CoreAlgorithmStmtId>,
    pub block: Option<BasicBlockId>,
    pub loop_id: Option<LoopId>,
    pub exit: Option<ControlFlowExitId>,
    pub local: Option<LocalId>,
    pub assignment_effect: Option<AssignmentEffectId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ControlFlowObligationSiteKind {
    Requires,
    Ensures,
    AlgorithmAssertion,
    StatementAssertion,
    AlgorithmInvariant,
    LoopInvariant,
    TerminationMeasure,
    PartialTermination,
    GhostPick,
    GhostAssignment,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowIr {
    pub algorithm: CoreAlgorithmId,
    pub item: CoreItemId,
    pub symbol: SymbolId,
    pub entry: BasicBlockId,
    pub blocks: ControlFlowBlockTable,
    pub locals: ControlFlowLocalTable,
    pub contexts: ProgramContextTable,
    pub context_facts: ContextFactTable,
    pub assignment_effects: AssignmentEffectTable,
    pub call_sites: CallSiteTable,
    pub contracts: ControlFlowContractSet,
    pub loops: ControlFlowLoopTable,
    pub exits: ControlFlowExitTable,
    pub ghost_effects: ControlFlowGhostTable,
    pub termination: ControlFlowTerminationPlan,
    pub source_map: ControlFlowSourceMap,
    pub diagnostics: ControlFlowDiagnosticTable,
}

impl ControlFlowIr {
    pub fn debug_text(&self) -> String {
        let mut output = String::from("control-flow-ir-debug-v1\n");
        writeln!(&mut output, "algorithm: {:?}", self.algorithm).expect("write string");
        writeln!(&mut output, "item: {:?}", self.item).expect("write string");
        writeln!(&mut output, "symbol: {:?}", self.symbol).expect("write string");
        writeln!(&mut output, "entry: {:?}", self.entry).expect("write string");
        write_table(&mut output, "blocks", self.blocks.iter());
        write_table(&mut output, "locals", self.locals.iter());
        write_table(&mut output, "contexts", self.contexts.iter());
        write_table(&mut output, "context-facts", self.context_facts.iter());
        write_table(
            &mut output,
            "assignment-effects",
            self.assignment_effects.iter(),
        );
        write_table(&mut output, "call-sites", self.call_sites.iter());
        write_table(&mut output, "loops", self.loops.iter());
        write_table(&mut output, "exits", self.exits.iter());
        write_table(&mut output, "diagnostics", self.diagnostics.iter());
        writeln!(&mut output, "contracts: {:?}", self.contracts).expect("write string");
        writeln!(&mut output, "ghost-effects: {:?}", self.ghost_effects).expect("write string");
        writeln!(&mut output, "termination: {:?}", self.termination).expect("write string");
        writeln!(&mut output, "source-map: {:?}", self.source_map).expect("write string");
        output
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowBlock {
    pub algorithm: CoreAlgorithmId,
    pub statements: Vec<CoreAlgorithmStmtId>,
    pub terminator: ControlFlowTerminator,
    pub context_in: ProgramContextId,
    pub context_out: Vec<ProgramContextId>,
    pub reachable: Reachability,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ControlFlowTerminator {
    Goto(BasicBlockId),
    Branch {
        condition: CoreFormulaId,
        then_block: BasicBlockId,
        else_block: BasicBlockId,
    },
    Switch {
        scrutinee: CoreTermId,
        arms: Vec<ControlFlowSwitchArm>,
        join: Option<BasicBlockId>,
    },
    Return(Option<CoreTermId>),
    Break {
        loop_id: LoopId,
        target: BasicBlockId,
    },
    Continue {
        loop_id: LoopId,
        target: BasicBlockId,
    },
    Unreachable,
    Error(ControlFlowDiagnosticId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowSwitchArm {
    pub pattern: CoreProvenanceKey,
    pub arm_index: usize,
    pub block: BasicBlockId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum Reachability {
    Reachable,
    Unreachable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowLocal {
    pub algorithm: CoreAlgorithmId,
    pub binder: CoreBinder,
    pub kind: LocalKind,
    pub declaration: LocalDeclaration,
    pub mutability: LocalMutability,
    pub ghost: bool,
    pub initialized_at: Option<CoreAlgorithmStmtId>,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalKind {
    Parameter,
    Result,
    Let,
    Pick { witness_ty: Option<CoreFormulaId> },
    HiddenLoopValue,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LocalDeclaration {
    Parameter,
    Result,
    Var,
    Const,
    GhostVar,
    GhostConst,
    PickRuntime,
    PickGhost,
    HiddenLoopValue,
    Unsupported(CoreVarRole),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum LocalMutability {
    Immutable,
    Mutable,
    Unknown,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProgramContext {
    pub definitely_initialized: Vec<LocalId>,
    pub maybe_assigned: Vec<CorePlace>,
    pub available_facts: Vec<ContextFactId>,
    pub assignment_effects: Vec<AssignmentEffectId>,
    pub call_effects: Vec<CallSiteId>,
    pub path_conditions: Vec<CoreFormulaId>,
    pub active_invariants: Vec<CoreFormulaId>,
    pub loop_stack: Vec<LoopId>,
    pub ghost_visible: Vec<LocalId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContextFact {
    pub formula: CoreFormulaId,
    pub source: CoreSourceRef,
    pub kind: ContextFactKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum ContextFactKind {
    Requirement,
    LocalInitializer,
    Assertion,
    BranchCondition,
    LoopInvariant,
    CallEnsures,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssignmentEffect {
    pub statement: CoreAlgorithmStmtId,
    pub target: AssignmentEffectTarget,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssignmentEffectTarget {
    Local(LocalId),
    Place(CorePlace),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CallSite {
    pub statement: CoreAlgorithmStmtId,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ControlFlowContractSet {
    pub requires: Vec<ContractSite>,
    pub ensures: Vec<ContractSite>,
    pub calls: Vec<CallSiteId>,
    pub assertions: Vec<AssertionSite>,
    pub loop_invariants: Vec<LoopInvariantSite>,
    pub decreasing: Vec<TerminationMeasureSite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ContractSite {
    pub kind: ContractSiteKind,
    pub formula: CoreFormulaId,
    pub placement: ContractSitePlacement,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContractSiteKind {
    Requires,
    Ensures,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ContractSitePlacement {
    Entry {
        block: BasicBlockId,
        context: ProgramContextId,
    },
    Return {
        block: BasicBlockId,
        exit: ControlFlowExitId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AssertionSite {
    pub formula: CoreFormulaId,
    pub placement: AssertionPlacement,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AssertionPlacement {
    AlgorithmContract {
        block: BasicBlockId,
        context: ProgramContextId,
    },
    Statement {
        statement: CoreAlgorithmStmtId,
        block: BasicBlockId,
        successor_context: ProgramContextId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LoopInvariantSite {
    pub formula: CoreFormulaId,
    pub placement: LoopInvariantPlacement,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LoopInvariantPlacement {
    AlgorithmContract {
        block: BasicBlockId,
        context: ProgramContextId,
    },
    Header {
        loop_id: LoopId,
        block: BasicBlockId,
    },
    NormalBackedge {
        loop_id: LoopId,
        from: BasicBlockId,
        to: BasicBlockId,
    },
    BreakExit {
        loop_id: LoopId,
        exit: ControlFlowExitId,
    },
    ContinueExit {
        loop_id: LoopId,
        exit: ControlFlowExitId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminationMeasureSite {
    pub term: CoreTermId,
    pub placement: TerminationMeasurePlacement,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminationMeasurePlacement {
    AlgorithmHeader {
        block: BasicBlockId,
    },
    LoopHeader {
        loop_id: LoopId,
        header: BasicBlockId,
    },
    ContinueEdge {
        loop_id: LoopId,
        exit: ControlFlowExitId,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowLoop {
    pub algorithm: CoreAlgorithmId,
    pub header: BasicBlockId,
    pub body: BasicBlockId,
    pub exit: BasicBlockId,
    pub condition: CoreFormulaId,
    pub invariants: Vec<CoreFormulaId>,
    pub decreasing: Vec<CoreTermId>,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowExit {
    pub algorithm: CoreAlgorithmId,
    pub statement: Option<CoreAlgorithmStmtId>,
    pub from: BasicBlockId,
    pub target: Option<BasicBlockId>,
    pub kind: ControlFlowExitKind,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ControlFlowExitKind {
    Return,
    Break { loop_id: LoopId },
    Continue { loop_id: LoopId },
    Error { diagnostic: ControlFlowDiagnosticId },
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ControlFlowGhostTable {
    pub declared_algorithm_effects: Vec<GhostEffectKey>,
    pub local_visibility: BTreeMap<LocalId, GhostVisibility>,
    pub runtime_assignment_effects: Vec<AssignmentEffectId>,
    pub ghost_assignment_effects: Vec<AssignmentEffectId>,
    pub runtime_pick_locals: Vec<LocalId>,
    pub ghost_pick_locals: Vec<LocalId>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum GhostVisibility {
    Runtime,
    GhostOnly,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ControlFlowTerminationPlan {
    pub algorithm_decreasing: Vec<CoreTermId>,
    pub loop_decreasing: BTreeMap<LoopId, Vec<CoreTermId>>,
    pub partial_sites: Vec<TerminationSite>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TerminationSite {
    pub kind: TerminationSiteKind,
    pub source: CoreSourceRef,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TerminationSiteKind {
    Algorithm,
    Loop(LoopId),
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ControlFlowSourceMap {
    pub algorithm_sources: BTreeMap<CoreAlgorithmId, CoreSourceRef>,
    pub block_sources: BTreeMap<BasicBlockId, CoreSourceRef>,
    pub statement_placements: BTreeMap<CoreAlgorithmStmtId, ControlFlowStatementPlacement>,
    pub local_sources: BTreeMap<LocalId, CoreSourceRef>,
    pub loop_sources: BTreeMap<LoopId, CoreSourceRef>,
    pub exit_sources: BTreeMap<ControlFlowExitId, CoreSourceRef>,
    pub diagnostic_sources: BTreeMap<ControlFlowDiagnosticId, CoreSourceRef>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ControlFlowStatementPlacement {
    Block {
        block: BasicBlockId,
    },
    Terminator {
        block: BasicBlockId,
    },
    LoopHeader {
        loop_id: LoopId,
        header: BasicBlockId,
    },
    SwitchArm {
        block: BasicBlockId,
        arm_index: usize,
    },
    LocalBinding {
        local: LocalId,
        block: BasicBlockId,
    },
    Checkpoint {
        block: BasicBlockId,
    },
    ErrorSite {
        block: BasicBlockId,
        diagnostic: ControlFlowDiagnosticId,
    },
}

impl ControlFlowStatementPlacement {
    const fn block(&self) -> BasicBlockId {
        match self {
            Self::Block { block }
            | Self::Terminator { block }
            | Self::LoopHeader { header: block, .. }
            | Self::SwitchArm { block, .. }
            | Self::LocalBinding { block, .. }
            | Self::Checkpoint { block }
            | Self::ErrorSite { block, .. } => *block,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ControlFlowDiagnostic {
    pub kind: ControlFlowDiagnosticKind,
    pub algorithm: CoreAlgorithmId,
    pub statement: Option<CoreAlgorithmStmtId>,
    pub source: CoreSourceRef,
    pub carried_core_diagnostic: Option<CoreDiagnosticId>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum ControlFlowDiagnosticKind {
    UnsupportedLocalDeclaration { local: LocalId, role: CoreVarRole },
    IllegalBreak,
    IllegalContinue,
    Phase9Error,
    UnreachableStatement { block: BasicBlockId },
    UseBeforeAssignment { local: LocalId, var: CoreVarId },
    FlowDiagnostic,
}

table!(ControlFlowTable, ControlFlowId, ControlFlowIr);
table!(ControlFlowBlockTable, BasicBlockId, ControlFlowBlock);
table!(ControlFlowLocalTable, LocalId, ControlFlowLocal);
table!(ProgramContextTable, ProgramContextId, ProgramContext);
table!(ContextFactTable, ContextFactId, ContextFact);
table!(AssignmentEffectTable, AssignmentEffectId, AssignmentEffect);
table!(CallSiteTable, CallSiteId, CallSite);
table!(ControlFlowLoopTable, LoopId, ControlFlowLoop);
table!(ControlFlowExitTable, ControlFlowExitId, ControlFlowExit);
table!(
    ControlFlowDiagnosticTable,
    ControlFlowDiagnosticId,
    ControlFlowDiagnostic
);
table!(
    ObligationHandoffTable,
    ObligationHandoffId,
    ObligationHandoffEntry
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct BlockCursor {
    block: BasicBlockId,
    context: ProgramContextId,
    reachable: Reachability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct LoopTarget {
    loop_id: LoopId,
    header: BasicBlockId,
    exit: BasicBlockId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct LocalUse {
    local: LocalId,
    var: CoreVarId,
    source: CoreSourceRef,
}

pub fn build_control_flow_ir(core: &CoreIr) -> ControlFlowOutput {
    let mut flows = ControlFlowTable::new();
    let mut flow_map = BTreeMap::new();

    for (algorithm_id, algorithm) in core.algorithms().iter() {
        let flow = FlowBuilder::new(core, algorithm_id, algorithm).finish();
        let flow_id = flows.insert(flow);
        flow_map.insert(algorithm_id, flow_id);
    }

    ControlFlowOutput { flows, flow_map }
}

pub fn build_obligation_seed_handoff(
    core: &CoreIr,
    control_flow: &ControlFlowOutput,
) -> ObligationSeedHandoff {
    let mut entries = Vec::new();

    for seed_id in core.obligation_seeds().canonical_order() {
        let seed = core
            .obligation_seeds()
            .get(seed_id)
            .expect("canonical seed id")
            .clone();
        entries.push(ObligationHandoffEntry {
            seed,
            origin: ObligationHandoffOrigin::ExistingCore { seed: seed_id },
            flow_site: None,
        });
    }

    for (flow_id, flow) in control_flow.flows.iter() {
        collect_flow_obligation_entries(&mut entries, flow_id, flow);
    }

    entries.sort_by(handoff_entry_cmp);

    let mut table = ObligationHandoffTable::new();
    let mut source_map = BTreeMap::new();
    for entry in entries {
        let source = entry.seed.source.clone();
        let id = table.insert(entry);
        source_map.insert(id, source);
    }

    ObligationSeedHandoff {
        entries: table,
        source_map,
    }
}

fn collect_flow_obligation_entries(
    entries: &mut Vec<ObligationHandoffEntry>,
    flow_id: ControlFlowId,
    flow: &ControlFlowIr,
) {
    for (ordinal, site) in flow.contracts.requires.iter().enumerate() {
        let (block, statement) = match site.placement {
            ContractSitePlacement::Entry { block, .. } => (Some(block), None),
            ContractSitePlacement::Return { block, exit } => (
                Some(block),
                flow.exits
                    .get(exit)
                    .and_then(|flow_exit| flow_exit.statement),
            ),
        };
        entries.push(flow_obligation_entry(
            flow_id,
            flow,
            ObligationSeedKind::AlgorithmContract,
            FlowSeedPayload {
                goal: Some(site.formula),
                term: None,
                source: site.source.clone(),
                local_path: format!(
                    "program/{}/contract/requires/{ordinal}",
                    flow.algorithm.index()
                ),
                semantic_origin: format!("flow:{}:requires:{ordinal}", flow.algorithm.index()),
                provenance: format!("flow-handoff:requires:{ordinal}"),
                statement,
                site: ControlFlowObligationSite {
                    kind: ControlFlowObligationSiteKind::Requires,
                    ordinal,
                    statement,
                    block,
                    loop_id: None,
                    exit: None,
                    local: None,
                    assignment_effect: None,
                },
            },
        ));
    }

    for (ordinal, site) in flow.contracts.ensures.iter().enumerate() {
        let (block, exit, statement) = match site.placement {
            ContractSitePlacement::Return { block, exit } => (
                Some(block),
                Some(exit),
                flow.exits
                    .get(exit)
                    .and_then(|flow_exit| flow_exit.statement),
            ),
            ContractSitePlacement::Entry { block, .. } => (Some(block), None, None),
        };
        entries.push(flow_obligation_entry(
            flow_id,
            flow,
            ObligationSeedKind::AlgorithmContract,
            FlowSeedPayload {
                goal: Some(site.formula),
                term: None,
                source: site.source.clone(),
                local_path: format!(
                    "program/{}/contract/ensures/{ordinal}",
                    flow.algorithm.index()
                ),
                semantic_origin: format!("flow:{}:ensures:{ordinal}", flow.algorithm.index()),
                provenance: format!("flow-handoff:ensures:{ordinal}"),
                statement,
                site: ControlFlowObligationSite {
                    kind: ControlFlowObligationSiteKind::Ensures,
                    ordinal,
                    statement,
                    block,
                    loop_id: None,
                    exit,
                    local: None,
                    assignment_effect: None,
                },
            },
        ));
    }

    for (ordinal, site) in flow.contracts.assertions.iter().enumerate() {
        let (kind, statement, block) = match site.placement {
            AssertionPlacement::AlgorithmContract { block, .. } => (
                ControlFlowObligationSiteKind::AlgorithmAssertion,
                None,
                Some(block),
            ),
            AssertionPlacement::Statement {
                statement, block, ..
            } => (
                ControlFlowObligationSiteKind::StatementAssertion,
                Some(statement),
                Some(block),
            ),
        };
        entries.push(flow_obligation_entry(
            flow_id,
            flow,
            ObligationSeedKind::AlgorithmContract,
            FlowSeedPayload {
                goal: Some(site.formula),
                term: None,
                source: site.source.clone(),
                local_path: format!("program/{}/assertion/{ordinal}", flow.algorithm.index()),
                semantic_origin: format!("flow:{}:assertion:{ordinal}", flow.algorithm.index()),
                provenance: format!("flow-handoff:assertion:{ordinal}"),
                statement,
                site: ControlFlowObligationSite {
                    kind,
                    ordinal,
                    statement,
                    block,
                    loop_id: None,
                    exit: None,
                    local: None,
                    assignment_effect: None,
                },
            },
        ));
    }

    for (ordinal, site) in flow.contracts.loop_invariants.iter().enumerate() {
        let (kind, block, loop_id, exit) = match site.placement {
            LoopInvariantPlacement::AlgorithmContract { block, .. } => (
                ControlFlowObligationSiteKind::AlgorithmInvariant,
                Some(block),
                None,
                None,
            ),
            LoopInvariantPlacement::Header { loop_id, block } => (
                ControlFlowObligationSiteKind::LoopInvariant,
                Some(block),
                Some(loop_id),
                None,
            ),
            LoopInvariantPlacement::NormalBackedge { loop_id, from, .. } => (
                ControlFlowObligationSiteKind::LoopInvariant,
                Some(from),
                Some(loop_id),
                None,
            ),
            LoopInvariantPlacement::BreakExit { loop_id, exit }
            | LoopInvariantPlacement::ContinueExit { loop_id, exit } => (
                ControlFlowObligationSiteKind::LoopInvariant,
                flow.exits.get(exit).map(|flow_exit| flow_exit.from),
                Some(loop_id),
                Some(exit),
            ),
        };
        entries.push(flow_obligation_entry(
            flow_id,
            flow,
            ObligationSeedKind::AlgorithmContract,
            FlowSeedPayload {
                goal: Some(site.formula),
                term: None,
                source: site.source.clone(),
                local_path: format!("program/{}/invariant/{ordinal}", flow.algorithm.index()),
                semantic_origin: format!("flow:{}:invariant:{ordinal}", flow.algorithm.index()),
                provenance: format!("flow-handoff:invariant:{ordinal}"),
                statement: None,
                site: ControlFlowObligationSite {
                    kind,
                    ordinal,
                    statement: None,
                    block,
                    loop_id,
                    exit,
                    local: None,
                    assignment_effect: None,
                },
            },
        ));
    }

    for (ordinal, site) in flow.contracts.decreasing.iter().enumerate() {
        let (block, loop_id, exit) = match site.placement {
            TerminationMeasurePlacement::AlgorithmHeader { block } => (Some(block), None, None),
            TerminationMeasurePlacement::LoopHeader { loop_id, header } => {
                (Some(header), Some(loop_id), None)
            }
            TerminationMeasurePlacement::ContinueEdge { loop_id, exit } => (
                flow.exits.get(exit).map(|flow_exit| flow_exit.from),
                Some(loop_id),
                Some(exit),
            ),
        };
        entries.push(flow_obligation_entry(
            flow_id,
            flow,
            ObligationSeedKind::AlgorithmTermination,
            FlowSeedPayload {
                goal: None,
                term: Some(site.term),
                source: site.source.clone(),
                local_path: format!(
                    "program/{}/termination/measure/{ordinal}",
                    flow.algorithm.index()
                ),
                semantic_origin: format!(
                    "flow:{}:termination:measure:{ordinal}",
                    flow.algorithm.index()
                ),
                provenance: format!("flow-handoff:termination-measure:{ordinal}"),
                statement: None,
                site: ControlFlowObligationSite {
                    kind: ControlFlowObligationSiteKind::TerminationMeasure,
                    ordinal,
                    statement: None,
                    block,
                    loop_id,
                    exit,
                    local: None,
                    assignment_effect: None,
                },
            },
        ));
    }

    for (ordinal, site) in flow.termination.partial_sites.iter().enumerate() {
        let (block, loop_id) = match site.kind {
            TerminationSiteKind::Algorithm => (Some(flow.entry), None),
            TerminationSiteKind::Loop(loop_id) => (
                flow.loops.get(loop_id).map(|flow_loop| flow_loop.header),
                Some(loop_id),
            ),
        };
        entries.push(flow_obligation_entry(
            flow_id,
            flow,
            ObligationSeedKind::AlgorithmTermination,
            FlowSeedPayload {
                goal: None,
                term: None,
                source: site.source.clone(),
                local_path: format!(
                    "program/{}/termination/partial/{ordinal}",
                    flow.algorithm.index()
                ),
                semantic_origin: format!(
                    "flow:{}:termination:partial:{ordinal}",
                    flow.algorithm.index()
                ),
                provenance: format!("flow-handoff:partial-termination:{ordinal}"),
                statement: None,
                site: ControlFlowObligationSite {
                    kind: ControlFlowObligationSiteKind::PartialTermination,
                    ordinal,
                    statement: None,
                    block,
                    loop_id,
                    exit: None,
                    local: None,
                    assignment_effect: None,
                },
            },
        ));
    }

    for (ordinal, local) in flow
        .ghost_effects
        .ghost_pick_locals
        .iter()
        .copied()
        .enumerate()
    {
        let Some(local_row) = flow.locals.get(local) else {
            continue;
        };
        entries.push(flow_obligation_entry(
            flow_id,
            flow,
            ObligationSeedKind::GhostErasure,
            FlowSeedPayload {
                goal: None,
                term: None,
                source: local_row.source.clone(),
                local_path: format!("program/{}/ghost/pick/{ordinal}", flow.algorithm.index()),
                semantic_origin: format!("flow:{}:ghost:pick:{ordinal}", flow.algorithm.index()),
                provenance: format!("flow-handoff:ghost-pick:{ordinal}"),
                statement: local_row.initialized_at,
                site: ControlFlowObligationSite {
                    kind: ControlFlowObligationSiteKind::GhostPick,
                    ordinal,
                    statement: local_row.initialized_at,
                    block: local_row
                        .initialized_at
                        .and_then(|statement| flow.source_map.statement_placements.get(&statement))
                        .map(ControlFlowStatementPlacement::block),
                    loop_id: None,
                    exit: None,
                    local: Some(local),
                    assignment_effect: None,
                },
            },
        ));
    }

    for (ordinal, effect) in flow
        .ghost_effects
        .ghost_assignment_effects
        .iter()
        .copied()
        .enumerate()
    {
        let Some(effect_row) = flow.assignment_effects.get(effect) else {
            continue;
        };
        let local = match effect_row.target {
            AssignmentEffectTarget::Local(local) => Some(local),
            AssignmentEffectTarget::Place(_) => None,
        };
        entries.push(flow_obligation_entry(
            flow_id,
            flow,
            ObligationSeedKind::GhostErasure,
            FlowSeedPayload {
                goal: None,
                term: None,
                source: effect_row.source.clone(),
                local_path: format!(
                    "program/{}/ghost/assignment/{ordinal}",
                    flow.algorithm.index()
                ),
                semantic_origin: format!(
                    "flow:{}:ghost:assignment:{ordinal}",
                    flow.algorithm.index()
                ),
                provenance: format!("flow-handoff:ghost-assignment:{ordinal}"),
                statement: Some(effect_row.statement),
                site: ControlFlowObligationSite {
                    kind: ControlFlowObligationSiteKind::GhostAssignment,
                    ordinal,
                    statement: Some(effect_row.statement),
                    block: flow
                        .source_map
                        .statement_placements
                        .get(&effect_row.statement)
                        .map(ControlFlowStatementPlacement::block),
                    loop_id: None,
                    exit: None,
                    local,
                    assignment_effect: Some(effect),
                },
            },
        ));
    }
}

struct FlowSeedPayload {
    goal: Option<CoreFormulaId>,
    term: Option<CoreTermId>,
    source: CoreSourceRef,
    local_path: String,
    semantic_origin: String,
    provenance: String,
    statement: Option<CoreAlgorithmStmtId>,
    site: ControlFlowObligationSite,
}

fn flow_obligation_entry(
    flow_id: ControlFlowId,
    flow: &ControlFlowIr,
    kind: ObligationSeedKind,
    payload: FlowSeedPayload,
) -> ObligationHandoffEntry {
    let source = synthetic_source(&payload.source, payload.provenance.clone());
    let mut provenance = source.provenance.clone();
    provenance.push(CoreProvenance::new(
        CoreProvenancePhase::Generated,
        payload.provenance,
    ));
    provenance.sort();
    provenance.dedup();
    let mut core_refs = vec![
        CoreNodeRef::Item(flow.item),
        CoreNodeRef::Algorithm(flow.algorithm),
    ];
    if let Some(goal) = payload.goal {
        core_refs.push(CoreNodeRef::Formula(goal));
    }
    if let Some(term) = payload.term {
        core_refs.push(CoreNodeRef::Term(term));
    }
    if let Some(statement) = payload.statement {
        core_refs.push(CoreNodeRef::AlgorithmStmt(statement));
    }
    sort_and_dedup(&mut core_refs);

    ObligationHandoffEntry {
        seed: ObligationSeed {
            owner: flow.item,
            kind,
            goal: payload.goal,
            context: Vec::new(),
            local_path: LocalProofOrProgramPath::new(payload.local_path),
            label: None,
            semantic_origin: NormalizedSemanticOrigin::new(payload.semantic_origin),
            provenance,
            source,
            core_refs,
            status: ObligationSeedStatus::Deferred,
            diagnostics: Vec::new(),
        },
        origin: ObligationHandoffOrigin::FlowDerived {
            flow: flow_id,
            algorithm: flow.algorithm,
        },
        flow_site: Some(payload.site),
    }
}

fn handoff_entry_cmp(
    left: &ObligationHandoffEntry,
    right: &ObligationHandoffEntry,
) -> std::cmp::Ordering {
    left.seed
        .canonical_key()
        .cmp(&right.seed.canonical_key())
        .then_with(|| left.origin.cmp(&right.origin))
        .then_with(|| left.flow_site.cmp(&right.flow_site))
}

struct FlowBuilder<'a> {
    core: &'a CoreIr,
    algorithm_id: CoreAlgorithmId,
    algorithm: &'a CoreAlgorithm,
    flow: ControlFlowIr,
}

impl<'a> FlowBuilder<'a> {
    fn new(core: &'a CoreIr, algorithm_id: CoreAlgorithmId, algorithm: &'a CoreAlgorithm) -> Self {
        let mut flow = ControlFlowIr {
            algorithm: algorithm_id,
            item: algorithm.item,
            symbol: algorithm.symbol.clone(),
            entry: BasicBlockId::new(0),
            blocks: ControlFlowBlockTable::new(),
            locals: ControlFlowLocalTable::new(),
            contexts: ProgramContextTable::new(),
            context_facts: ContextFactTable::new(),
            assignment_effects: AssignmentEffectTable::new(),
            call_sites: CallSiteTable::new(),
            contracts: ControlFlowContractSet::default(),
            loops: ControlFlowLoopTable::new(),
            exits: ControlFlowExitTable::new(),
            ghost_effects: ControlFlowGhostTable {
                declared_algorithm_effects: algorithm.ghost_effects.clone(),
                ..ControlFlowGhostTable::default()
            },
            termination: ControlFlowTerminationPlan {
                algorithm_decreasing: algorithm.contracts.decreasing.clone(),
                ..ControlFlowTerminationPlan::default()
            },
            source_map: ControlFlowSourceMap::default(),
            diagnostics: ControlFlowDiagnosticTable::new(),
        };
        flow.source_map
            .algorithm_sources
            .insert(algorithm_id, algorithm.source.clone());

        Self {
            core,
            algorithm_id,
            algorithm,
            flow,
        }
    }

    fn finish(mut self) -> ControlFlowIr {
        let mut initialized = Vec::new();
        let mut ghost_visible = Vec::new();
        for binder in &self.algorithm.params {
            let local = self.add_local(ControlFlowLocal {
                algorithm: self.algorithm_id,
                binder: binder.clone(),
                kind: LocalKind::Parameter,
                declaration: LocalDeclaration::Parameter,
                mutability: LocalMutability::Immutable,
                ghost: false,
                initialized_at: None,
                source: binder.source.clone(),
            });
            initialized.push(local);
        }
        if let Some(result) = &self.algorithm.result {
            self.add_local(ControlFlowLocal {
                algorithm: self.algorithm_id,
                binder: result.clone(),
                kind: LocalKind::Result,
                declaration: LocalDeclaration::Result,
                mutability: LocalMutability::Immutable,
                ghost: false,
                initialized_at: None,
                source: result.source.clone(),
            });
        }

        sort_and_dedup(&mut initialized);
        sort_and_dedup(&mut ghost_visible);
        let entry_context = self.add_context(ProgramContext {
            definitely_initialized: initialized,
            maybe_assigned: Vec::new(),
            available_facts: Vec::new(),
            assignment_effects: Vec::new(),
            call_effects: Vec::new(),
            path_conditions: Vec::new(),
            active_invariants: Vec::new(),
            loop_stack: Vec::new(),
            ghost_visible,
        });
        let entry = self.add_block(
            self.algorithm.source.clone(),
            Reachability::Reachable,
            entry_context,
        );
        debug_assert_eq!(entry, BasicBlockId::new(0));
        self.flow.entry = entry;
        self.attach_entry_requires(entry, entry_context);
        self.attach_entry_contract_payloads(entry, entry_context);
        self.attach_algorithm_termination(entry);

        let cursor = BlockCursor {
            block: entry,
            context: entry_context,
            reachable: Reachability::Reachable,
        };
        if let Some(exit) = self.build_sequence(&self.algorithm.statements.clone(), cursor, &[])
            && exit.reachable == Reachability::Reachable
            && self.block_is_open(exit.block)
        {
            self.terminate(
                exit.block,
                ControlFlowTerminator::Return(None),
                vec![exit.context],
            );
            self.add_exit(ControlFlowExit {
                algorithm: self.algorithm_id,
                statement: None,
                from: exit.block,
                target: None,
                kind: ControlFlowExitKind::Return,
                source: self
                    .flow
                    .blocks
                    .get(exit.block)
                    .expect("fallthrough block")
                    .source
                    .clone(),
            });
        }
        self.attach_return_ensures();
        self.sort_diagnostics();
        self.flow
    }

    fn build_sequence(
        &mut self,
        statements: &[CoreAlgorithmStmtId],
        initial: BlockCursor,
        loop_stack: &[LoopTarget],
    ) -> Option<BlockCursor> {
        let mut current = Some(initial);
        let mut last_context = initial.context;
        let mut last_reachable = initial.reachable;

        for statement in statements {
            let cursor = match current {
                Some(cursor) => cursor,
                None => {
                    let source = self.statement(*statement).source.clone();
                    let block = self.add_block(
                        synthetic_source(&source, "unreachable"),
                        Reachability::Unreachable,
                        last_context,
                    );
                    BlockCursor {
                        block,
                        context: last_context,
                        reachable: Reachability::Unreachable,
                    }
                }
            };
            if cursor.reachable == Reachability::Unreachable {
                self.add_diagnostic(ControlFlowDiagnostic {
                    kind: ControlFlowDiagnosticKind::UnreachableStatement {
                        block: cursor.block,
                    },
                    algorithm: self.algorithm_id,
                    statement: Some(*statement),
                    source: self.statement(*statement).source.clone(),
                    carried_core_diagnostic: None,
                });
            }
            let next = self.build_statement(cursor, *statement, loop_stack);
            if let Some(cursor) = next {
                last_context = cursor.context;
                last_reachable = cursor.reachable;
                current = Some(cursor);
            } else {
                current = None;
                last_reachable = Reachability::Unreachable;
            }
        }

        current.map(|mut cursor| {
            cursor.reachable = last_reachable;
            cursor
        })
    }

    fn build_statement(
        &mut self,
        cursor: BlockCursor,
        statement_id: CoreAlgorithmStmtId,
        loop_stack: &[LoopTarget],
    ) -> Option<BlockCursor> {
        let statement = self.statement(statement_id).clone();
        match &statement.kind {
            CoreAlgorithmStmtKind::Let {
                binder,
                value,
                ghost,
            } => Some(self.build_let(cursor, statement_id, &statement, binder, *value, *ghost)),
            CoreAlgorithmStmtKind::Assign { target, value } => {
                if cursor.reachable == Reachability::Reachable {
                    self.check_term_uses(*value, cursor.context, statement_id);
                }
                self.flow.source_map.statement_placements.insert(
                    statement_id,
                    ControlFlowStatementPlacement::Block {
                        block: cursor.block,
                    },
                );
                self.append_statement(cursor.block, statement_id);
                let effect = self.add_assignment_effect(AssignmentEffect {
                    statement: statement_id,
                    target: AssignmentEffectTarget::Place(target.clone()),
                    source: statement.source.clone(),
                });
                let mut context = self.context(cursor.context).clone();
                push_unique(&mut context.maybe_assigned, target.clone());
                push_unique(&mut context.assignment_effects, effect);
                let context = self.add_context(context);
                Some(BlockCursor { context, ..cursor })
            }
            CoreAlgorithmStmtKind::Assert { formula } => {
                if cursor.reachable == Reachability::Reachable {
                    self.check_formula_uses(*formula, cursor.context, statement_id);
                }
                self.flow.source_map.statement_placements.insert(
                    statement_id,
                    ControlFlowStatementPlacement::Checkpoint {
                        block: cursor.block,
                    },
                );
                self.append_statement(cursor.block, statement_id);
                let fact = self.add_context_fact(ContextFact {
                    formula: *formula,
                    source: self.formula_source(*formula),
                    kind: ContextFactKind::Assertion,
                });
                let mut context = self.context(cursor.context).clone();
                push_unique(&mut context.available_facts, fact);
                let context = self.add_context(context);
                self.flow.contracts.assertions.push(AssertionSite {
                    formula: *formula,
                    placement: AssertionPlacement::Statement {
                        statement: statement_id,
                        block: cursor.block,
                        successor_context: context,
                    },
                    source: statement.source.clone(),
                });
                Some(BlockCursor { context, ..cursor })
            }
            CoreAlgorithmStmtKind::If {
                condition,
                then_body,
                else_body,
            } => {
                if cursor.reachable == Reachability::Reachable {
                    self.check_formula_uses(*condition, cursor.context, statement_id);
                }
                Some(self.build_if(
                    cursor,
                    statement_id,
                    &statement,
                    *condition,
                    then_body,
                    else_body,
                    loop_stack,
                ))
            }
            CoreAlgorithmStmtKind::While {
                condition,
                invariants,
                decreasing,
                body,
            } => {
                if cursor.reachable == Reachability::Reachable {
                    self.check_formula_uses(*condition, cursor.context, statement_id);
                    for invariant in invariants {
                        self.check_formula_uses(*invariant, cursor.context, statement_id);
                    }
                    for term in decreasing {
                        self.check_term_uses(*term, cursor.context, statement_id);
                    }
                }
                Some(self.build_while(
                    cursor,
                    statement_id,
                    &statement,
                    *condition,
                    invariants,
                    decreasing,
                    body,
                    loop_stack,
                ))
            }
            CoreAlgorithmStmtKind::Match { scrutinee, arms } => {
                if cursor.reachable == Reachability::Reachable {
                    self.check_term_uses(*scrutinee, cursor.context, statement_id);
                }
                Some(self.build_match(
                    cursor,
                    statement_id,
                    &statement,
                    *scrutinee,
                    arms,
                    loop_stack,
                ))
            }
            CoreAlgorithmStmtKind::Return(value) => {
                if cursor.reachable == Reachability::Reachable
                    && let Some(value) = value
                {
                    self.check_term_uses(*value, cursor.context, statement_id);
                }
                self.flow.source_map.statement_placements.insert(
                    statement_id,
                    ControlFlowStatementPlacement::Terminator {
                        block: cursor.block,
                    },
                );
                self.terminate(
                    cursor.block,
                    ControlFlowTerminator::Return(*value),
                    vec![cursor.context],
                );
                self.add_exit(ControlFlowExit {
                    algorithm: self.algorithm_id,
                    statement: Some(statement_id),
                    from: cursor.block,
                    target: None,
                    kind: ControlFlowExitKind::Return,
                    source: statement.source,
                });
                None
            }
            CoreAlgorithmStmtKind::Break => {
                self.build_break_or_continue(cursor, statement_id, &statement, loop_stack, true)
            }
            CoreAlgorithmStmtKind::Continue => {
                self.build_break_or_continue(cursor, statement_id, &statement, loop_stack, false)
            }
            CoreAlgorithmStmtKind::Pick {
                binder,
                witness_ty,
                ghost,
            } => Some(self.build_pick(
                cursor,
                statement_id,
                &statement,
                binder,
                *witness_ty,
                *ghost,
            )),
            CoreAlgorithmStmtKind::Error(diagnostic) => {
                let flow_diagnostic = self.add_diagnostic(ControlFlowDiagnostic {
                    kind: ControlFlowDiagnosticKind::Phase9Error,
                    algorithm: self.algorithm_id,
                    statement: Some(statement_id),
                    source: statement.source.clone(),
                    carried_core_diagnostic: Some(*diagnostic),
                });
                self.flow.source_map.statement_placements.insert(
                    statement_id,
                    ControlFlowStatementPlacement::ErrorSite {
                        block: cursor.block,
                        diagnostic: flow_diagnostic,
                    },
                );
                self.terminate(
                    cursor.block,
                    ControlFlowTerminator::Error(flow_diagnostic),
                    Vec::new(),
                );
                self.add_exit(ControlFlowExit {
                    algorithm: self.algorithm_id,
                    statement: Some(statement_id),
                    from: cursor.block,
                    target: None,
                    kind: ControlFlowExitKind::Error {
                        diagnostic: flow_diagnostic,
                    },
                    source: statement.source,
                });
                None
            }
        }
    }

    fn build_let(
        &mut self,
        cursor: BlockCursor,
        statement_id: CoreAlgorithmStmtId,
        statement: &CoreAlgorithmStmt,
        binder: &CoreBinder,
        value: Option<CoreTermId>,
        ghost: bool,
    ) -> BlockCursor {
        self.append_statement(cursor.block, statement_id);
        let (declaration, mutability, local_ghost, unsupported_role) =
            local_declaration_from_role(&binder.role, ghost);
        let initialized_at = value.map(|_| statement_id);
        let local = self.add_local(ControlFlowLocal {
            algorithm: self.algorithm_id,
            binder: binder.clone(),
            kind: LocalKind::Let,
            declaration,
            mutability,
            ghost: local_ghost,
            initialized_at,
            source: binder.source.clone(),
        });
        self.flow.source_map.statement_placements.insert(
            statement_id,
            ControlFlowStatementPlacement::LocalBinding {
                local,
                block: cursor.block,
            },
        );
        if let Some(role) = unsupported_role {
            self.add_diagnostic(ControlFlowDiagnostic {
                kind: ControlFlowDiagnosticKind::UnsupportedLocalDeclaration { local, role },
                algorithm: self.algorithm_id,
                statement: Some(statement_id),
                source: binder.source.clone(),
                carried_core_diagnostic: None,
            });
        }
        if cursor.reachable == Reachability::Reachable
            && let Some(value) = value
        {
            self.check_term_uses(value, cursor.context, statement_id);
        }
        let mut context = self.context(cursor.context).clone();
        if initialized_at.is_some() {
            push_unique(&mut context.definitely_initialized, local);
            let effect = self.add_assignment_effect(AssignmentEffect {
                statement: statement_id,
                target: AssignmentEffectTarget::Local(local),
                source: statement.source.clone(),
            });
            push_unique(&mut context.assignment_effects, effect);
        }
        if local_ghost {
            push_unique(&mut context.ghost_visible, local);
        }
        let context = self.add_context(context);
        BlockCursor { context, ..cursor }
    }

    fn build_pick(
        &mut self,
        cursor: BlockCursor,
        statement_id: CoreAlgorithmStmtId,
        statement: &CoreAlgorithmStmt,
        binder: &CoreBinder,
        witness_ty: Option<CoreFormulaId>,
        ghost: bool,
    ) -> BlockCursor {
        self.append_statement(cursor.block, statement_id);
        let declaration = if ghost {
            LocalDeclaration::PickGhost
        } else {
            LocalDeclaration::PickRuntime
        };
        let local = self.add_local(ControlFlowLocal {
            algorithm: self.algorithm_id,
            binder: binder.clone(),
            kind: LocalKind::Pick { witness_ty },
            declaration,
            mutability: LocalMutability::Immutable,
            ghost,
            initialized_at: Some(statement_id),
            source: binder.source.clone(),
        });
        self.flow.source_map.statement_placements.insert(
            statement_id,
            ControlFlowStatementPlacement::LocalBinding {
                local,
                block: cursor.block,
            },
        );
        if cursor.reachable == Reachability::Reachable
            && let Some(witness_ty) = witness_ty
        {
            self.check_formula_uses_with_bound(
                witness_ty,
                cursor.context,
                statement_id,
                BTreeSet::from([binder.var]),
            );
        }
        let effect = self.add_assignment_effect(AssignmentEffect {
            statement: statement_id,
            target: AssignmentEffectTarget::Local(local),
            source: statement.source.clone(),
        });
        let mut context = self.context(cursor.context).clone();
        push_unique(&mut context.definitely_initialized, local);
        push_unique(&mut context.assignment_effects, effect);
        if ghost {
            push_unique(&mut context.ghost_visible, local);
        }
        let context = self.add_context(context);
        BlockCursor { context, ..cursor }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_if(
        &mut self,
        cursor: BlockCursor,
        statement_id: CoreAlgorithmStmtId,
        statement: &CoreAlgorithmStmt,
        condition: CoreFormulaId,
        then_body: &[CoreAlgorithmStmtId],
        else_body: &[CoreAlgorithmStmtId],
        loop_stack: &[LoopTarget],
    ) -> BlockCursor {
        let mut then_context = self.context(cursor.context).clone();
        push_unique(&mut then_context.path_conditions, condition);
        let then_context = self.add_context(then_context);
        let else_context = self.add_context(self.context(cursor.context).clone());
        let then_block = self.add_block(
            synthetic_source(&statement.source, "if-then"),
            cursor.reachable,
            then_context,
        );
        let else_block = self.add_block(
            synthetic_source(&statement.source, "if-else"),
            cursor.reachable,
            else_context,
        );
        self.flow.source_map.statement_placements.insert(
            statement_id,
            ControlFlowStatementPlacement::Terminator {
                block: cursor.block,
            },
        );
        self.terminate(
            cursor.block,
            ControlFlowTerminator::Branch {
                condition,
                then_block,
                else_block,
            },
            vec![then_context, else_context],
        );

        let then_exit = self.build_sequence(
            then_body,
            BlockCursor {
                block: then_block,
                context: then_context,
                reachable: cursor.reachable,
            },
            loop_stack,
        );
        let else_exit = self.build_sequence(
            else_body,
            BlockCursor {
                block: else_block,
                context: else_context,
                reachable: cursor.reachable,
            },
            loop_stack,
        );
        let normal_exits = [then_exit, else_exit]
            .into_iter()
            .flatten()
            .filter(|exit| exit.reachable == Reachability::Reachable)
            .collect::<Vec<_>>();
        let join_context = self.join_context(
            normal_exits
                .iter()
                .map(|exit| exit.context)
                .collect::<Vec<_>>()
                .as_slice(),
            cursor.context,
        );
        let join_reachable = if normal_exits
            .iter()
            .any(|exit| exit.reachable == Reachability::Reachable)
        {
            Reachability::Reachable
        } else {
            Reachability::Unreachable
        };
        let join_block = self.add_block(
            synthetic_source(&statement.source, "if-join"),
            join_reachable,
            join_context,
        );
        for exit in normal_exits {
            self.terminate(
                exit.block,
                ControlFlowTerminator::Goto(join_block),
                vec![exit.context],
            );
        }
        BlockCursor {
            block: join_block,
            context: join_context,
            reachable: join_reachable,
        }
    }

    #[allow(clippy::too_many_arguments)]
    fn build_while(
        &mut self,
        cursor: BlockCursor,
        statement_id: CoreAlgorithmStmtId,
        statement: &CoreAlgorithmStmt,
        condition: CoreFormulaId,
        invariants: &[CoreFormulaId],
        decreasing: &[CoreTermId],
        body: &[CoreAlgorithmStmtId],
        loop_stack: &[LoopTarget],
    ) -> BlockCursor {
        let invariant_facts = invariants
            .iter()
            .copied()
            .map(|formula| {
                self.add_context_fact(ContextFact {
                    formula,
                    source: self.formula_source(formula),
                    kind: ContextFactKind::LoopInvariant,
                })
            })
            .collect::<Vec<_>>();
        let mut header_context = self.context(cursor.context).clone();
        header_context
            .active_invariants
            .extend(invariants.iter().copied());
        header_context
            .available_facts
            .extend(invariant_facts.iter().copied());
        let header_context = self.add_context(header_context);
        let mut body_context = self.context(cursor.context).clone();
        push_unique(&mut body_context.path_conditions, condition);
        body_context
            .active_invariants
            .extend(invariants.iter().copied());
        body_context
            .available_facts
            .extend(invariant_facts.iter().copied());
        let mut condition_exit_context = self.context(cursor.context).clone();
        condition_exit_context
            .active_invariants
            .extend(invariants.iter().copied());
        condition_exit_context
            .available_facts
            .extend(invariant_facts.iter().copied());
        let condition_exit_context = self.add_context(condition_exit_context);

        let header_block = self.add_block(
            synthetic_source(&statement.source, "while-header"),
            cursor.reachable,
            header_context,
        );
        let body_block = self.add_block(
            synthetic_source(&statement.source, "while-body"),
            cursor.reachable,
            ProgramContextId::new(usize::MAX),
        );
        let exit_block = self.add_block(
            synthetic_source(&statement.source, "while-exit"),
            cursor.reachable,
            condition_exit_context,
        );
        let loop_id = self.add_loop(ControlFlowLoop {
            algorithm: self.algorithm_id,
            header: header_block,
            body: body_block,
            exit: exit_block,
            condition,
            invariants: invariants.to_vec(),
            decreasing: decreasing.to_vec(),
            source: statement.source.clone(),
        });
        body_context.loop_stack.push(loop_id);
        let body_context = self.add_context(body_context);
        self.flow
            .blocks
            .get_mut(body_block)
            .expect("body block")
            .context_in = body_context;
        self.flow.source_map.statement_placements.insert(
            statement_id,
            ControlFlowStatementPlacement::LoopHeader {
                loop_id,
                header: header_block,
            },
        );
        self.flow
            .termination
            .loop_decreasing
            .insert(loop_id, decreasing.to_vec());
        for invariant in invariants {
            self.flow.contracts.loop_invariants.push(LoopInvariantSite {
                formula: *invariant,
                placement: LoopInvariantPlacement::Header {
                    loop_id,
                    block: header_block,
                },
                source: self.formula_source(*invariant),
            });
        }
        if decreasing.is_empty() {
            self.flow.termination.partial_sites.push(TerminationSite {
                kind: TerminationSiteKind::Loop(loop_id),
                source: statement.source.clone(),
            });
        } else {
            for term in decreasing {
                self.flow.contracts.decreasing.push(TerminationMeasureSite {
                    term: *term,
                    placement: TerminationMeasurePlacement::LoopHeader {
                        loop_id,
                        header: header_block,
                    },
                    source: self.term_source(*term),
                });
            }
        }
        self.terminate(
            cursor.block,
            ControlFlowTerminator::Goto(header_block),
            vec![header_context],
        );
        self.terminate(
            header_block,
            ControlFlowTerminator::Branch {
                condition,
                then_block: body_block,
                else_block: exit_block,
            },
            vec![body_context, condition_exit_context],
        );

        let mut nested_loops = loop_stack.to_vec();
        nested_loops.push(LoopTarget {
            loop_id,
            header: header_block,
            exit: exit_block,
        });
        let body_exit = self.build_sequence(
            body,
            BlockCursor {
                block: body_block,
                context: body_context,
                reachable: cursor.reachable,
            },
            &nested_loops,
        );
        if let Some(exit) = body_exit {
            self.terminate(
                exit.block,
                ControlFlowTerminator::Goto(header_block),
                vec![exit.context],
            );
        }
        if let Some(exit) = body_exit
            && exit.reachable == Reachability::Reachable
        {
            for invariant in invariants {
                self.flow.contracts.loop_invariants.push(LoopInvariantSite {
                    formula: *invariant,
                    placement: LoopInvariantPlacement::NormalBackedge {
                        loop_id,
                        from: exit.block,
                        to: header_block,
                    },
                    source: self.formula_source(*invariant),
                });
            }
        }
        let mut exit_contexts = vec![condition_exit_context];
        if let Some(exit) = body_exit
            && exit.reachable == Reachability::Reachable
        {
            exit_contexts.push(exit.context);
        }
        for (_, exit) in self.flow.exits.iter() {
            if matches!(
                exit.kind,
                ControlFlowExitKind::Break {
                    loop_id: current_loop
                } if current_loop == loop_id
            ) && exit.target == Some(exit_block)
            {
                let break_block = self.flow.blocks.get(exit.from).expect("break block");
                if break_block.reachable == Reachability::Reachable {
                    exit_contexts.extend(break_block.context_out.iter().copied());
                    let exit_id = self
                        .flow
                        .exits
                        .iter()
                        .find_map(|(id, candidate)| (candidate == exit).then_some(id))
                        .expect("exit from iteration");
                    for invariant in invariants {
                        self.flow.contracts.loop_invariants.push(LoopInvariantSite {
                            formula: *invariant,
                            placement: LoopInvariantPlacement::BreakExit {
                                loop_id,
                                exit: exit_id,
                            },
                            source: self.formula_source(*invariant),
                        });
                    }
                }
            }
            if matches!(
                exit.kind,
                ControlFlowExitKind::Continue {
                    loop_id: current_loop
                } if current_loop == loop_id
            ) && exit.target == Some(header_block)
            {
                let continue_block = self.flow.blocks.get(exit.from).expect("continue block");
                if continue_block.reachable == Reachability::Reachable {
                    exit_contexts.extend(continue_block.context_out.iter().copied());
                    let exit_id = self
                        .flow
                        .exits
                        .iter()
                        .find_map(|(id, candidate)| (candidate == exit).then_some(id))
                        .expect("exit from iteration");
                    for invariant in invariants {
                        self.flow.contracts.loop_invariants.push(LoopInvariantSite {
                            formula: *invariant,
                            placement: LoopInvariantPlacement::ContinueExit {
                                loop_id,
                                exit: exit_id,
                            },
                            source: self.formula_source(*invariant),
                        });
                    }
                    for term in decreasing {
                        self.flow.contracts.decreasing.push(TerminationMeasureSite {
                            term: *term,
                            placement: TerminationMeasurePlacement::ContinueEdge {
                                loop_id,
                                exit: exit_id,
                            },
                            source: self.term_source(*term),
                        });
                    }
                }
            }
        }
        let loop_exit_context = self.join_context(&exit_contexts, condition_exit_context);
        self.flow
            .blocks
            .get_mut(exit_block)
            .expect("loop exit block")
            .context_in = loop_exit_context;
        BlockCursor {
            block: exit_block,
            context: loop_exit_context,
            reachable: cursor.reachable,
        }
    }

    fn build_match(
        &mut self,
        cursor: BlockCursor,
        statement_id: CoreAlgorithmStmtId,
        statement: &CoreAlgorithmStmt,
        scrutinee: CoreTermId,
        arms: &[CoreAlgorithmMatchArm],
        loop_stack: &[LoopTarget],
    ) -> BlockCursor {
        let mut arm_cursors = Vec::new();
        let mut switch_contexts = Vec::new();
        let mut switch_arms = Vec::new();
        for (arm_index, arm) in arms.iter().enumerate() {
            let context = self.add_context(self.context(cursor.context).clone());
            let block = self.add_block(
                synthetic_source(&statement.source, format!("match-arm-{arm_index}")),
                cursor.reachable,
                context,
            );
            switch_arms.push(ControlFlowSwitchArm {
                pattern: arm.pattern.clone(),
                arm_index,
                block,
            });
            switch_contexts.push(context);
            arm_cursors.push((
                arm,
                BlockCursor {
                    block,
                    context,
                    reachable: cursor.reachable,
                },
            ));
        }
        self.flow.source_map.statement_placements.insert(
            statement_id,
            ControlFlowStatementPlacement::Terminator {
                block: cursor.block,
            },
        );

        let mut normal_exits = Vec::new();
        for (arm, arm_cursor) in arm_cursors {
            if let Some(exit) = self.build_sequence(&arm.body, arm_cursor, loop_stack)
                && exit.reachable == Reachability::Reachable
            {
                normal_exits.push(exit);
            }
        }
        let join_context = self.join_context(
            normal_exits
                .iter()
                .map(|exit| exit.context)
                .collect::<Vec<_>>()
                .as_slice(),
            cursor.context,
        );
        let join_reachable = if normal_exits
            .iter()
            .any(|exit| exit.reachable == Reachability::Reachable)
        {
            Reachability::Reachable
        } else {
            Reachability::Unreachable
        };
        let join_block = self.add_block(
            synthetic_source(&statement.source, "match-join"),
            join_reachable,
            join_context,
        );
        self.terminate(
            cursor.block,
            ControlFlowTerminator::Switch {
                scrutinee,
                arms: switch_arms,
                join: Some(join_block),
            },
            switch_contexts,
        );
        for exit in normal_exits {
            self.terminate(
                exit.block,
                ControlFlowTerminator::Goto(join_block),
                vec![exit.context],
            );
        }
        BlockCursor {
            block: join_block,
            context: join_context,
            reachable: join_reachable,
        }
    }

    fn build_break_or_continue(
        &mut self,
        cursor: BlockCursor,
        statement_id: CoreAlgorithmStmtId,
        statement: &CoreAlgorithmStmt,
        loop_stack: &[LoopTarget],
        is_break: bool,
    ) -> Option<BlockCursor> {
        if let Some(target) = loop_stack.last() {
            self.flow.source_map.statement_placements.insert(
                statement_id,
                ControlFlowStatementPlacement::Terminator {
                    block: cursor.block,
                },
            );
            let (terminator, exit_kind, target_block) = if is_break {
                (
                    ControlFlowTerminator::Break {
                        loop_id: target.loop_id,
                        target: target.exit,
                    },
                    ControlFlowExitKind::Break {
                        loop_id: target.loop_id,
                    },
                    target.exit,
                )
            } else {
                (
                    ControlFlowTerminator::Continue {
                        loop_id: target.loop_id,
                        target: target.header,
                    },
                    ControlFlowExitKind::Continue {
                        loop_id: target.loop_id,
                    },
                    target.header,
                )
            };
            self.terminate(cursor.block, terminator, vec![cursor.context]);
            self.add_exit(ControlFlowExit {
                algorithm: self.algorithm_id,
                statement: Some(statement_id),
                from: cursor.block,
                target: Some(target_block),
                kind: exit_kind,
                source: statement.source.clone(),
            });
            None
        } else {
            let diagnostic = self.add_diagnostic(ControlFlowDiagnostic {
                kind: if is_break {
                    ControlFlowDiagnosticKind::IllegalBreak
                } else {
                    ControlFlowDiagnosticKind::IllegalContinue
                },
                algorithm: self.algorithm_id,
                statement: Some(statement_id),
                source: statement.source.clone(),
                carried_core_diagnostic: None,
            });
            self.flow.source_map.statement_placements.insert(
                statement_id,
                ControlFlowStatementPlacement::ErrorSite {
                    block: cursor.block,
                    diagnostic,
                },
            );
            self.terminate(
                cursor.block,
                ControlFlowTerminator::Error(diagnostic),
                Vec::new(),
            );
            self.add_exit(ControlFlowExit {
                algorithm: self.algorithm_id,
                statement: Some(statement_id),
                from: cursor.block,
                target: None,
                kind: ControlFlowExitKind::Error { diagnostic },
                source: statement.source.clone(),
            });
            None
        }
    }

    fn add_block(
        &mut self,
        source: CoreSourceRef,
        reachable: Reachability,
        context_in: ProgramContextId,
    ) -> BasicBlockId {
        let id = self.flow.blocks.insert(ControlFlowBlock {
            algorithm: self.algorithm_id,
            statements: Vec::new(),
            terminator: ControlFlowTerminator::Unreachable,
            context_in,
            context_out: Vec::new(),
            reachable,
            source: source.clone(),
        });
        self.flow.source_map.block_sources.insert(id, source);
        id
    }

    fn add_local(&mut self, local: ControlFlowLocal) -> LocalId {
        let source = local.source.clone();
        let ghost = local.ghost;
        let is_pick = matches!(local.kind, LocalKind::Pick { .. });
        let id = self.flow.locals.insert(local);
        self.flow.source_map.local_sources.insert(id, source);
        self.flow.ghost_effects.local_visibility.insert(
            id,
            if ghost {
                GhostVisibility::GhostOnly
            } else {
                GhostVisibility::Runtime
            },
        );
        if is_pick {
            if ghost {
                push_unique(&mut self.flow.ghost_effects.ghost_pick_locals, id);
            } else {
                push_unique(&mut self.flow.ghost_effects.runtime_pick_locals, id);
            }
        }
        id
    }

    fn add_context(&mut self, mut context: ProgramContext) -> ProgramContextId {
        sort_and_dedup(&mut context.definitely_initialized);
        sort_and_dedup(&mut context.maybe_assigned);
        sort_and_dedup(&mut context.available_facts);
        sort_and_dedup(&mut context.assignment_effects);
        sort_and_dedup(&mut context.call_effects);
        sort_and_dedup(&mut context.active_invariants);
        sort_and_dedup(&mut context.ghost_visible);
        self.flow.contexts.insert(context)
    }

    fn add_context_fact(&mut self, fact: ContextFact) -> ContextFactId {
        self.flow.context_facts.insert(fact)
    }

    fn push_context_fact(&mut self, context: ProgramContextId, fact: ContextFactId) {
        let context = self
            .flow
            .contexts
            .get_mut(context)
            .expect("builder context id");
        push_unique(&mut context.available_facts, fact);
        sort_and_dedup(&mut context.available_facts);
    }

    fn add_loop(&mut self, loop_record: ControlFlowLoop) -> LoopId {
        let source = loop_record.source.clone();
        let id = self.flow.loops.insert(loop_record);
        self.flow.source_map.loop_sources.insert(id, source);
        id
    }

    fn add_exit(&mut self, exit: ControlFlowExit) -> ControlFlowExitId {
        let source = exit.source.clone();
        let id = self.flow.exits.insert(exit);
        self.flow.source_map.exit_sources.insert(id, source);
        id
    }

    fn add_diagnostic(&mut self, diagnostic: ControlFlowDiagnostic) -> ControlFlowDiagnosticId {
        let source = diagnostic.source.clone();
        let id = self.flow.diagnostics.insert(diagnostic);
        self.flow.source_map.diagnostic_sources.insert(id, source);
        id
    }

    fn sort_diagnostics(&mut self) {
        if self.flow.diagnostics.len() <= 1 {
            return;
        }
        let mut diagnostics = self
            .flow
            .diagnostics
            .entries
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, diagnostic)| (ControlFlowDiagnosticId::new(index), diagnostic))
            .collect::<Vec<_>>();
        diagnostics.sort_by_key(|(old_id, diagnostic)| {
            (
                diagnostic_source_sort_key(&diagnostic.source),
                diagnostic.algorithm,
                self.diagnostic_block(*old_id, diagnostic),
                diagnostic_class_rank(&diagnostic.kind),
                old_id.index(),
            )
        });

        let remap = diagnostics
            .iter()
            .enumerate()
            .map(|(new_index, (old_id, _))| (*old_id, ControlFlowDiagnosticId::new(new_index)))
            .collect::<BTreeMap<_, _>>();
        self.flow.diagnostics.entries = diagnostics
            .into_iter()
            .map(|(_, diagnostic)| diagnostic)
            .collect();
        self.remap_diagnostic_ids(&remap);
    }

    fn diagnostic_block(
        &self,
        old_id: ControlFlowDiagnosticId,
        diagnostic: &ControlFlowDiagnostic,
    ) -> BasicBlockId {
        if let ControlFlowDiagnosticKind::UnreachableStatement { block } = diagnostic.kind {
            return block;
        }
        diagnostic
            .statement
            .and_then(|statement| {
                self.flow
                    .source_map
                    .statement_placements
                    .get(&statement)
                    .map(ControlFlowStatementPlacement::block)
            })
            .or_else(|| {
                self.flow.blocks.iter().find_map(|(block_id, block)| {
                    matches!(block.terminator, ControlFlowTerminator::Error(id) if id == old_id)
                        .then_some(block_id)
                })
            })
            .unwrap_or(BasicBlockId::new(usize::MAX))
    }

    fn remap_diagnostic_ids(
        &mut self,
        remap: &BTreeMap<ControlFlowDiagnosticId, ControlFlowDiagnosticId>,
    ) {
        for (_, block) in self.flow.blocks.iter_mut() {
            if let ControlFlowTerminator::Error(diagnostic) = &mut block.terminator
                && let Some(new_id) = remap.get(diagnostic)
            {
                *diagnostic = *new_id;
            }
        }
        for placement in self.flow.source_map.statement_placements.values_mut() {
            if let ControlFlowStatementPlacement::ErrorSite { diagnostic, .. } = placement
                && let Some(new_id) = remap.get(diagnostic)
            {
                *diagnostic = *new_id;
            }
        }
        for (_, exit) in self.flow.exits.iter_mut() {
            if let ControlFlowExitKind::Error { diagnostic } = &mut exit.kind
                && let Some(new_id) = remap.get(diagnostic)
            {
                *diagnostic = *new_id;
            }
        }
        self.flow.source_map.diagnostic_sources = self
            .flow
            .source_map
            .diagnostic_sources
            .iter()
            .filter_map(|(old_id, source)| {
                remap.get(old_id).map(|new_id| (*new_id, source.clone()))
            })
            .collect();
    }

    fn add_assignment_effect(&mut self, effect: AssignmentEffect) -> AssignmentEffectId {
        let visibility = match &effect.target {
            AssignmentEffectTarget::Local(local) => self
                .flow
                .ghost_effects
                .local_visibility
                .get(local)
                .copied()
                .unwrap_or(GhostVisibility::Runtime),
            AssignmentEffectTarget::Place(_) => GhostVisibility::Runtime,
        };
        let id = self.flow.assignment_effects.insert(effect);
        match visibility {
            GhostVisibility::Runtime => {
                push_unique(&mut self.flow.ghost_effects.runtime_assignment_effects, id);
            }
            GhostVisibility::GhostOnly => {
                push_unique(&mut self.flow.ghost_effects.ghost_assignment_effects, id);
            }
        }
        id
    }

    fn append_statement(&mut self, block: BasicBlockId, statement: CoreAlgorithmStmtId) {
        self.flow
            .blocks
            .get_mut(block)
            .expect("block from builder")
            .statements
            .push(statement);
    }

    fn terminate(
        &mut self,
        block: BasicBlockId,
        terminator: ControlFlowTerminator,
        context_out: Vec<ProgramContextId>,
    ) {
        let block = self.flow.blocks.get_mut(block).expect("block from builder");
        block.terminator = terminator;
        block.context_out = context_out;
    }

    fn block_is_open(&self, block: BasicBlockId) -> bool {
        matches!(
            self.flow
                .blocks
                .get(block)
                .expect("block from builder")
                .terminator,
            ControlFlowTerminator::Unreachable
        )
    }

    fn attach_entry_requires(&mut self, entry: BasicBlockId, entry_context: ProgramContextId) {
        for formula in self.algorithm.contracts.requires.clone() {
            let source = self.formula_source(formula);
            let fact = self.add_context_fact(ContextFact {
                formula,
                source: source.clone(),
                kind: ContextFactKind::Requirement,
            });
            self.push_context_fact(entry_context, fact);
            self.flow.contracts.requires.push(ContractSite {
                kind: ContractSiteKind::Requires,
                formula,
                placement: ContractSitePlacement::Entry {
                    block: entry,
                    context: entry_context,
                },
                source,
            });
        }
    }

    fn attach_entry_contract_payloads(
        &mut self,
        entry: BasicBlockId,
        entry_context: ProgramContextId,
    ) {
        for formula in self.algorithm.contracts.assertions.clone() {
            self.flow.contracts.assertions.push(AssertionSite {
                formula,
                placement: AssertionPlacement::AlgorithmContract {
                    block: entry,
                    context: entry_context,
                },
                source: self.formula_source(formula),
            });
        }
        for formula in self.algorithm.contracts.invariants.clone() {
            self.flow.contracts.loop_invariants.push(LoopInvariantSite {
                formula,
                placement: LoopInvariantPlacement::AlgorithmContract {
                    block: entry,
                    context: entry_context,
                },
                source: self.formula_source(formula),
            });
        }
    }

    fn attach_algorithm_termination(&mut self, entry: BasicBlockId) {
        if self.algorithm.contracts.decreasing.is_empty() {
            self.flow.termination.partial_sites.push(TerminationSite {
                kind: TerminationSiteKind::Algorithm,
                source: self.algorithm.source.clone(),
            });
            return;
        }
        for term in self.algorithm.contracts.decreasing.clone() {
            self.flow.contracts.decreasing.push(TerminationMeasureSite {
                term,
                placement: TerminationMeasurePlacement::AlgorithmHeader { block: entry },
                source: self.term_source(term),
            });
        }
    }

    fn attach_return_ensures(&mut self) {
        let returns = self
            .flow
            .exits
            .iter()
            .filter_map(|(exit_id, exit)| {
                matches!(exit.kind, ControlFlowExitKind::Return).then_some((exit_id, exit.from))
            })
            .collect::<Vec<_>>();
        for (exit, block) in returns {
            for formula in self.algorithm.contracts.ensures.clone() {
                self.flow.contracts.ensures.push(ContractSite {
                    kind: ContractSiteKind::Ensures,
                    formula,
                    placement: ContractSitePlacement::Return { block, exit },
                    source: self.formula_source(formula),
                });
            }
        }
    }

    fn formula_source(&self, formula: CoreFormulaId) -> CoreSourceRef {
        self.core
            .source_map()
            .formula_sources
            .get(&formula)
            .cloned()
            .unwrap_or_else(|| {
                self.core
                    .formulas()
                    .get(formula)
                    .expect("validated CoreIr formula id")
                    .source
                    .clone()
            })
    }

    fn term_source(&self, term: CoreTermId) -> CoreSourceRef {
        self.core
            .source_map()
            .term_sources
            .get(&term)
            .cloned()
            .unwrap_or_else(|| {
                self.core
                    .terms()
                    .get(term)
                    .expect("validated CoreIr term id")
                    .source
                    .clone()
            })
    }

    fn check_term_uses(
        &mut self,
        term: CoreTermId,
        context: ProgramContextId,
        statement: CoreAlgorithmStmtId,
    ) {
        self.check_term_uses_with_bound(term, context, statement, BTreeSet::new());
    }

    fn check_term_uses_with_bound(
        &mut self,
        term: CoreTermId,
        context: ProgramContextId,
        statement: CoreAlgorithmStmtId,
        bound_vars: BTreeSet<CoreVarId>,
    ) {
        let mut uses = Vec::new();
        self.collect_term_uses(term, &bound_vars, &mut uses);
        self.emit_use_before_assignment_diagnostics(uses, context, statement);
    }

    fn check_formula_uses(
        &mut self,
        formula: CoreFormulaId,
        context: ProgramContextId,
        statement: CoreAlgorithmStmtId,
    ) {
        self.check_formula_uses_with_bound(formula, context, statement, BTreeSet::new());
    }

    fn check_formula_uses_with_bound(
        &mut self,
        formula: CoreFormulaId,
        context: ProgramContextId,
        statement: CoreAlgorithmStmtId,
        bound_vars: BTreeSet<CoreVarId>,
    ) {
        let mut uses = Vec::new();
        self.collect_formula_uses(formula, &bound_vars, &mut uses);
        self.emit_use_before_assignment_diagnostics(uses, context, statement);
    }

    fn emit_use_before_assignment_diagnostics(
        &mut self,
        uses: Vec<LocalUse>,
        context: ProgramContextId,
        statement: CoreAlgorithmStmtId,
    ) {
        let initialized = self.context(context).definitely_initialized.clone();
        let mut emitted = Vec::new();
        for local_use in uses {
            if initialized.contains(&local_use.local)
                || emitted.iter().any(|emitted| emitted == &local_use)
            {
                continue;
            }
            self.add_diagnostic(ControlFlowDiagnostic {
                kind: ControlFlowDiagnosticKind::UseBeforeAssignment {
                    local: local_use.local,
                    var: local_use.var,
                },
                algorithm: self.algorithm_id,
                statement: Some(statement),
                source: local_use.source.clone(),
                carried_core_diagnostic: None,
            });
            emitted.push(local_use);
        }
    }

    fn collect_formula_uses(
        &self,
        formula: CoreFormulaId,
        bound_vars: &BTreeSet<CoreVarId>,
        uses: &mut Vec<LocalUse>,
    ) {
        let formula = self
            .core
            .formulas()
            .get(formula)
            .expect("validated CoreIr formula id");
        match &formula.kind {
            CoreFormulaKind::True | CoreFormulaKind::False | CoreFormulaKind::Error(_) => {}
            CoreFormulaKind::Atom { args, .. } => {
                for arg in args {
                    self.collect_term_uses(*arg, bound_vars, uses);
                }
            }
            CoreFormulaKind::Equals { left, right } => {
                self.collect_term_uses(*left, bound_vars, uses);
                self.collect_term_uses(*right, bound_vars, uses);
            }
            CoreFormulaKind::TypePred { subject, .. } => {
                self.collect_term_uses(*subject, bound_vars, uses);
            }
            CoreFormulaKind::Not(inner) => self.collect_formula_uses(*inner, bound_vars, uses),
            CoreFormulaKind::And(items) | CoreFormulaKind::Or(items) => {
                for item in items {
                    self.collect_formula_uses(*item, bound_vars, uses);
                }
            }
            CoreFormulaKind::Implies {
                premise,
                conclusion,
            } => {
                self.collect_formula_uses(*premise, bound_vars, uses);
                self.collect_formula_uses(*conclusion, bound_vars, uses);
            }
            CoreFormulaKind::Iff { left, right } => {
                self.collect_formula_uses(*left, bound_vars, uses);
                self.collect_formula_uses(*right, bound_vars, uses);
            }
            CoreFormulaKind::Forall { binders, body }
            | CoreFormulaKind::Exists { binders, body } => {
                let mut scoped = bound_vars.clone();
                for binder in binders {
                    scoped.insert(binder.var);
                    if let Some(guard) = binder.ty_guard {
                        self.collect_formula_uses(guard, &scoped, uses);
                    }
                }
                self.collect_formula_uses(*body, &scoped, uses);
            }
        }
    }

    fn collect_term_uses(
        &self,
        term: CoreTermId,
        bound_vars: &BTreeSet<CoreVarId>,
        uses: &mut Vec<LocalUse>,
    ) {
        let term_record = self
            .core
            .terms()
            .get(term)
            .expect("validated CoreIr term id");
        match &term_record.kind {
            CoreTermKind::Var(var) => {
                if !bound_vars.contains(var)
                    && let Some(local) = self.local_for_var(*var)
                {
                    uses.push(LocalUse {
                        local,
                        var: *var,
                        source: self.term_source(term),
                    });
                }
            }
            CoreTermKind::Const(_) | CoreTermKind::Error(_) => {}
            CoreTermKind::Apply { args, .. }
            | CoreTermKind::Tuple(args)
            | CoreTermKind::SetEnum(args)
            | CoreTermKind::Generated { args, .. } => {
                for arg in args {
                    self.collect_term_uses(*arg, bound_vars, uses);
                }
            }
            CoreTermKind::Select { base, .. } => self.collect_term_uses(*base, bound_vars, uses),
        }
    }

    fn local_for_var(&self, var: CoreVarId) -> Option<LocalId> {
        self.flow.locals.iter().find_map(|(id, local)| {
            (local.binder.var == var && !matches!(local.kind, LocalKind::HiddenLoopValue))
                .then_some(id)
        })
    }

    fn join_context(
        &mut self,
        contexts: &[ProgramContextId],
        fallback: ProgramContextId,
    ) -> ProgramContextId {
        if contexts.is_empty() {
            return self.add_context(self.context(fallback).clone());
        }
        let first = self.context(contexts[0]).clone();
        let mut joined = ProgramContext {
            definitely_initialized: first.definitely_initialized,
            maybe_assigned: first.maybe_assigned,
            available_facts: first.available_facts,
            assignment_effects: first.assignment_effects,
            call_effects: first.call_effects,
            path_conditions: Vec::new(),
            active_invariants: first.active_invariants,
            loop_stack: first.loop_stack,
            ghost_visible: first.ghost_visible,
        };
        for context in &contexts[1..] {
            let context = self.context(*context);
            joined.definitely_initialized = intersection(
                &joined.definitely_initialized,
                &context.definitely_initialized,
            );
            joined.available_facts =
                intersection(&joined.available_facts, &context.available_facts);
            joined.active_invariants =
                intersection(&joined.active_invariants, &context.active_invariants);
            joined.loop_stack = intersection(&joined.loop_stack, &context.loop_stack);
            joined.ghost_visible = intersection(&joined.ghost_visible, &context.ghost_visible);
            extend_unique(
                &mut joined.maybe_assigned,
                context.maybe_assigned.iter().cloned(),
            );
            extend_unique(
                &mut joined.assignment_effects,
                context.assignment_effects.iter().copied(),
            );
            extend_unique(
                &mut joined.call_effects,
                context.call_effects.iter().copied(),
            );
        }
        self.add_context(joined)
    }

    fn statement(&self, id: CoreAlgorithmStmtId) -> &CoreAlgorithmStmt {
        self.core
            .algorithm_statements()
            .get(id)
            .expect("validated CoreIr statement id")
    }

    fn context(&self, id: ProgramContextId) -> &ProgramContext {
        self.flow.contexts.get(id).expect("builder context id")
    }
}

fn local_declaration_from_role(
    role: &CoreVarRole,
    statement_ghost: bool,
) -> (LocalDeclaration, LocalMutability, bool, Option<CoreVarRole>) {
    match (role.as_str(), statement_ghost) {
        ("local:var", false) => (LocalDeclaration::Var, LocalMutability::Mutable, false, None),
        ("local:var", true) => (
            LocalDeclaration::GhostVar,
            LocalMutability::Mutable,
            true,
            None,
        ),
        ("local:const", false) => (
            LocalDeclaration::Const,
            LocalMutability::Immutable,
            false,
            None,
        ),
        ("local:const", true) => (
            LocalDeclaration::GhostConst,
            LocalMutability::Immutable,
            true,
            None,
        ),
        ("local:ghost-var", _) => (
            LocalDeclaration::GhostVar,
            LocalMutability::Mutable,
            true,
            None,
        ),
        ("local:ghost-const", _) => (
            LocalDeclaration::GhostConst,
            LocalMutability::Immutable,
            true,
            None,
        ),
        _ => (
            LocalDeclaration::Unsupported(role.clone()),
            LocalMutability::Unknown,
            statement_ghost,
            Some(role.clone()),
        ),
    }
}

fn synthetic_source(source: &CoreSourceRef, role: impl Into<CoreProvenanceKey>) -> CoreSourceRef {
    let mut provenance = source.provenance.clone();
    provenance.push(CoreProvenance::new(CoreProvenancePhase::Generated, role));
    source.clone().with_provenance(provenance)
}

fn push_unique<T>(values: &mut Vec<T>, value: T)
where
    T: Ord + Clone,
{
    if !values.contains(&value) {
        values.push(value);
    }
}

fn extend_unique<T, I>(values: &mut Vec<T>, iter: I)
where
    T: Ord + Clone,
    I: IntoIterator<Item = T>,
{
    for value in iter {
        push_unique(values, value);
    }
}

fn sort_and_dedup<T: Ord>(values: &mut Vec<T>) {
    values.sort();
    values.dedup();
}

fn diagnostic_source_sort_key(
    source: &CoreSourceRef,
) -> (u8, usize, usize, String, String, String, String) {
    match &source.anchor {
        CoreSourceAnchor::SourceRange(range) => (
            0,
            range.start,
            range.end,
            String::new(),
            String::new(),
            String::new(),
            String::new(),
        ),
        CoreSourceAnchor::GeneratedFrom(generated_from) => (
            1,
            0,
            0,
            format!("{:?}", generated_from.owner),
            format!("{:?}", generated_from.kind),
            generated_from.key.as_str().to_owned(),
            generated_from.reason.as_str().to_owned(),
        ),
    }
}

const fn diagnostic_class_rank(kind: &ControlFlowDiagnosticKind) -> u8 {
    match kind {
        ControlFlowDiagnosticKind::UnsupportedLocalDeclaration { .. } => 0,
        ControlFlowDiagnosticKind::IllegalBreak => 1,
        ControlFlowDiagnosticKind::IllegalContinue => 2,
        ControlFlowDiagnosticKind::Phase9Error => 3,
        ControlFlowDiagnosticKind::UnreachableStatement { .. } => 4,
        ControlFlowDiagnosticKind::UseBeforeAssignment { .. } => 5,
        ControlFlowDiagnosticKind::FlowDiagnostic => 6,
    }
}

fn intersection<T>(left: &[T], right: &[T]) -> Vec<T>
where
    T: Ord + Clone,
{
    let right = right.iter().collect::<BTreeSet<_>>();
    left.iter()
        .filter(|value| right.contains(value))
        .cloned()
        .collect()
}

fn write_table<Id, Entry, I>(output: &mut String, name: &str, iter: I)
where
    Id: std::fmt::Debug,
    Entry: std::fmt::Debug,
    I: IntoIterator<Item = (Id, Entry)>,
{
    let _ = writeln!(output, "[{name}]");
    for (id, entry) in iter {
        let _ = writeln!(output, "{id:?}: {entry:?}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core_ir::{
        CoreAlgorithmTable, CoreContractSet, CoreDiagnostic, CoreDiagnosticClass,
        CoreDiagnosticTable, CoreFormula, CoreFormulaKind, CoreFormulaTable, CoreIrParts, CoreItem,
        CoreItemKind, CoreItemTable, CoreLabelRef, CoreNodeRef, CoreSourceMap, CoreTerm,
        CoreTermKind, CoreTermTable, CoreVarId, CoreVisibility, GeneratedOrigin, GeneratedOriginId,
        GeneratedOriginKey, GeneratedOriginKind, GeneratedOriginTable, ObligationSeedTable,
    };
    use mizar_resolve::resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId};
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceId, SourceRange,
    };

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "15".repeat(32)
        ))
        .expect("valid snapshot id");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source id")
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("core_flow"))
    }

    fn symbol(name: &str) -> SymbolId {
        SymbolId::new(
            module_id(),
            LocalSymbolId::new(name),
            FullyQualifiedName::new(format!("pkg::core_flow::{name}")),
        )
    }

    fn direct(source_id: SourceId, start: usize, end: usize) -> CoreSourceRef {
        CoreSourceRef::direct(range(source_id, start, end))
    }

    struct CoreFixture {
        source_id: SourceId,
        algorithm_id: CoreAlgorithmId,
        item: CoreItemId,
        parts: CoreIrParts,
    }

    impl CoreFixture {
        fn new() -> Self {
            let source_id = source_id();
            let item_source = direct(source_id, 0, 1);
            let mut items = CoreItemTable::new();
            let item = items.insert(CoreItem::new(
                symbol("AlgorithmOwner"),
                CoreItemKind::Algorithm,
                CoreVisibility::new("public"),
                item_source.clone(),
            ));
            let mut source_map = CoreSourceMap::new();
            source_map.item_sources.insert(item, item_source);
            Self {
                source_id,
                algorithm_id: CoreAlgorithmId::new(0),
                item,
                parts: CoreIrParts {
                    source_id,
                    module_id: module_id(),
                    items,
                    terms: CoreTermTable::new(),
                    formulas: CoreFormulaTable::new(),
                    definitions: Default::default(),
                    proofs: Default::default(),
                    proof_nodes: Default::default(),
                    algorithms: CoreAlgorithmTable::new(),
                    algorithm_statements: Default::default(),
                    generated: GeneratedOriginTable::new(),
                    obligation_seeds: ObligationSeedTable::new(),
                    source_map,
                    diagnostics: CoreDiagnosticTable::new(),
                },
            }
        }

        fn source(&self, start: usize, end: usize) -> CoreSourceRef {
            direct(self.source_id, start, end)
        }

        fn generated_origin(
            &mut self,
            kind: GeneratedOriginKind,
            key: &str,
            start: usize,
        ) -> GeneratedOriginId {
            let source = self.source(start, start + 1);
            let id = self.parts.generated.insert(GeneratedOrigin {
                owner: self.item,
                kind,
                key: GeneratedOriginKey::new(key),
                functor: None,
                params: Vec::new(),
                evidence: vec![CoreProvenance::new(
                    CoreProvenancePhase::Generated,
                    format!("{key}:evidence"),
                )],
                source: source.clone(),
            });
            self.parts.source_map.generated_sources.insert(id, source);
            id
        }

        fn diagnostic(&mut self, message_key: &str, start: usize) -> CoreDiagnosticId {
            let source = self.source(start, start + 1);
            self.parts.diagnostics.insert(CoreDiagnostic::error(
                CoreDiagnosticClass::UnsupportedLowering,
                message_key,
                source,
            ))
        }

        fn term_var(&mut self, var: usize, start: usize) -> CoreTermId {
            self.term(CoreTermKind::Var(CoreVarId::new(var)), start)
        }

        fn term(&mut self, kind: CoreTermKind, start: usize) -> CoreTermId {
            let source = self.source(start, start + 1);
            let id = self.parts.terms.insert(CoreTerm::new(kind, source.clone()));
            self.parts.source_map.term_sources.insert(id, source);
            id
        }

        fn formula(&mut self, kind: CoreFormulaKind, start: usize) -> CoreFormulaId {
            let source = self.source(start, start + 1);
            let id = self
                .parts
                .formulas
                .insert(CoreFormula::new(kind, source.clone()));
            self.parts.source_map.formula_sources.insert(id, source);
            id
        }

        fn binder(&self, var: usize, role: &str, start: usize) -> CoreBinder {
            CoreBinder {
                var: CoreVarId::new(var),
                role: CoreVarRole::new(role),
                ty_guard: None,
                source_name: Some(format!("v{var}")),
                source: self.source(start, start + 1),
            }
        }

        fn stmt(&mut self, kind: CoreAlgorithmStmtKind, start: usize) -> CoreAlgorithmStmtId {
            let source = self.source(start, start + 1);
            let id = self.parts.algorithm_statements.insert(CoreAlgorithmStmt {
                owner: self.algorithm_id,
                kind,
                source: source.clone(),
                diagnostics: Vec::new(),
            });
            self.parts.source_map.algorithm_sources.insert(id, source);
            id
        }

        fn obligation_seed(
            &mut self,
            kind: ObligationSeedKind,
            goal: Option<CoreFormulaId>,
            path: &str,
            label: Option<&str>,
            origin: &str,
            start: usize,
        ) -> ObligationSeedId {
            let source = self.source(start, start + 1);
            let seed = self.parts.obligation_seeds.insert(ObligationSeed {
                owner: self.item,
                kind,
                goal,
                context: goal.into_iter().collect(),
                local_path: LocalProofOrProgramPath::new(path),
                label: label.map(CoreLabelRef::new),
                semantic_origin: NormalizedSemanticOrigin::new(origin),
                provenance: vec![CoreProvenance::new(CoreProvenancePhase::Checker, origin)],
                source: source.clone(),
                core_refs: goal
                    .map(|formula| {
                        vec![CoreNodeRef::Item(self.item), CoreNodeRef::Formula(formula)]
                    })
                    .unwrap_or_else(|| vec![CoreNodeRef::Item(self.item)]),
                status: ObligationSeedStatus::Active,
                diagnostics: Vec::new(),
            });
            self.parts
                .source_map
                .obligation_sources
                .insert(seed, source);
            seed
        }

        fn enrich_obligation_seed(
            &mut self,
            seed: ObligationSeedId,
            status: ObligationSeedStatus,
            diagnostics: Vec<CoreDiagnosticId>,
            extra_refs: Vec<CoreNodeRef>,
            extra_provenance: Vec<CoreProvenance>,
        ) {
            let row = self
                .parts
                .obligation_seeds
                .get_mut(seed)
                .expect("obligation seed");
            row.status = status;
            row.diagnostics = diagnostics;
            row.core_refs.extend(extra_refs);
            sort_and_dedup(&mut row.core_refs);
            row.provenance.extend(extra_provenance);
            sort_and_dedup(&mut row.provenance);
        }

        fn error_stmt(&mut self, start: usize) -> CoreAlgorithmStmtId {
            let source = self.source(start, start + 1);
            let diagnostic = self.parts.diagnostics.insert(CoreDiagnostic::error(
                CoreDiagnosticClass::AlgorithmShell,
                "phase9-error",
                source.clone(),
            ));
            let id = self.parts.algorithm_statements.insert(CoreAlgorithmStmt {
                owner: self.algorithm_id,
                kind: CoreAlgorithmStmtKind::Error(diagnostic),
                source: source.clone(),
                diagnostics: vec![diagnostic],
            });
            self.parts
                .diagnostics
                .get_mut(diagnostic)
                .expect("diagnostic")
                .owner = Some(CoreNodeRef::AlgorithmStmt(id));
            self.parts.source_map.algorithm_sources.insert(id, source);
            id
        }

        fn finish(
            self,
            params: Vec<CoreBinder>,
            result: Option<CoreBinder>,
            statements: Vec<CoreAlgorithmStmtId>,
        ) -> CoreIr {
            self.finish_with_contracts(params, result, statements, CoreContractSet::default())
        }

        fn finish_with_contracts(
            mut self,
            params: Vec<CoreBinder>,
            result: Option<CoreBinder>,
            statements: Vec<CoreAlgorithmStmtId>,
            contracts: CoreContractSet,
        ) -> CoreIr {
            let algorithm = CoreAlgorithm {
                item: self.item,
                symbol: symbol("AlgorithmOwner"),
                params,
                result,
                contracts,
                statements,
                ghost_effects: Vec::new(),
                source: self.source(1, 2),
                diagnostics: Vec::new(),
            };
            let inserted = self.parts.algorithms.insert(algorithm);
            assert_eq!(inserted, self.algorithm_id);
            CoreIr::try_new(self.parts).expect("valid fixture core ir")
        }
    }

    fn empty_control_flow_output() -> ControlFlowOutput {
        ControlFlowOutput {
            flows: ControlFlowTable::new(),
            flow_map: BTreeMap::new(),
        }
    }

    fn only_flow(output: &ControlFlowOutput) -> &ControlFlowIr {
        output
            .flows
            .get(ControlFlowId::new(0))
            .expect("single flow")
    }

    fn context_has_fact(
        flow: &ControlFlowIr,
        context: ProgramContextId,
        formula: CoreFormulaId,
        source: &CoreSourceRef,
        kind: ContextFactKind,
    ) -> bool {
        flow.contexts
            .get(context)
            .expect("context")
            .available_facts
            .iter()
            .any(|fact| {
                let fact = flow.context_facts.get(*fact).expect("context fact");
                fact.formula == formula && fact.source == *source && fact.kind == kind
            })
    }

    fn flow_handoff_entries(
        handoff: &ObligationSeedHandoff,
    ) -> Vec<(ObligationHandoffId, &ObligationHandoffEntry)> {
        handoff
            .entries
            .iter()
            .filter(|(_, entry)| {
                matches!(entry.origin, ObligationHandoffOrigin::FlowDerived { .. })
            })
            .collect()
    }

    fn source_map_matches_seed(
        handoff: &ObligationSeedHandoff,
        id: ObligationHandoffId,
        entry: &ObligationHandoffEntry,
    ) -> bool {
        handoff.source_map.get(&id) == Some(&entry.seed.source)
    }

    fn assert_flow_seed_core_refs(
        flow: &ControlFlowIr,
        entry: &ObligationHandoffEntry,
        extra_refs: &[CoreNodeRef],
    ) {
        let mut expected = vec![
            CoreNodeRef::Item(flow.item),
            CoreNodeRef::Algorithm(flow.algorithm),
        ];
        expected.extend_from_slice(extra_refs);
        sort_and_dedup(&mut expected);
        assert_eq!(entry.seed.core_refs, expected);
    }

    fn flow_entry_by_kind<'a>(
        entries: &'a [(ObligationHandoffId, &ObligationHandoffEntry)],
        kind: ControlFlowObligationSiteKind,
        ordinal: usize,
    ) -> &'a ObligationHandoffEntry {
        entries
            .iter()
            .find(|(_, entry)| {
                entry
                    .flow_site
                    .as_ref()
                    .is_some_and(|site| site.kind == kind && site.ordinal == ordinal)
            })
            .map(|(_, entry)| *entry)
            .unwrap_or_else(|| panic!("missing {kind:?} ordinal {ordinal}"))
    }

    fn assert_flow_seed_source(
        handoff: &ObligationSeedHandoff,
        entry: &ObligationHandoffEntry,
        expected: &CoreSourceRef,
        expected_generated_key: &str,
    ) {
        let handoff_id = handoff
            .entries
            .iter()
            .find_map(|(id, candidate)| (candidate == entry).then_some(id))
            .expect("entry id");
        assert_eq!(entry.seed.source.anchor, expected.anchor);
        assert_eq!(
            handoff.source_map.get(&handoff_id),
            Some(&entry.seed.source)
        );
        assert_eq!(
            entry.seed.source.provenance,
            vec![CoreProvenance::new(
                CoreProvenancePhase::Generated,
                expected_generated_key
            )]
        );
        let generated_seed_keys = entry
            .seed
            .provenance
            .iter()
            .filter_map(|provenance| {
                (provenance.phase == CoreProvenancePhase::Generated)
                    .then_some(provenance.key.as_str())
            })
            .collect::<Vec<_>>();
        assert_eq!(generated_seed_keys, vec![expected_generated_key]);
    }

    #[test]
    fn obligation_handoff_preserves_existing_core_seeds() {
        let mut fixture = CoreFixture::new();
        let theorem_goal = fixture.formula(CoreFormulaKind::True, 10);
        let definition_goal = fixture.formula(CoreFormulaKind::False, 11);
        let checker_goal = fixture.formula(
            CoreFormulaKind::Iff {
                left: theorem_goal,
                right: definition_goal,
            },
            12,
        );
        let theorem_seed = fixture.obligation_seed(
            ObligationSeedKind::TheoremProof,
            Some(theorem_goal),
            "proof/terminal/0",
            Some("Thesis"),
            "pkg::main::AlgorithmOwner.proof.terminal",
            20,
        );
        let definition_seed = fixture.obligation_seed(
            ObligationSeedKind::DefinitionCorrectness,
            Some(definition_goal),
            "definition/correctness/0",
            None,
            "pkg::main::AlgorithmOwner.definition.correctness",
            21,
        );
        let checker_seed = fixture.obligation_seed(
            ObligationSeedKind::CheckerInitial,
            Some(checker_goal),
            "checker/initial/0",
            Some("C1"),
            "pkg::main::AlgorithmOwner.checker.initial",
            22,
        );
        let generated_comprehension = fixture.generated_origin(
            GeneratedOriginKind::FraenkelComprehension,
            "fraenkel:comprehension:0",
            23,
        );
        let deferred_diagnostic = fixture.diagnostic("generated-sethood-deferred", 24);
        let generated_deferred_seed = fixture.obligation_seed(
            ObligationSeedKind::GeneratedSethood,
            Some(checker_goal),
            "generated/fraenkel/sethood/0",
            None,
            "pkg::main::AlgorithmOwner.generated.fraenkel.sethood",
            25,
        );
        fixture.enrich_obligation_seed(
            generated_deferred_seed,
            ObligationSeedStatus::Deferred,
            vec![deferred_diagnostic],
            vec![
                CoreNodeRef::Generated(generated_comprehension),
                CoreNodeRef::Diagnostic(deferred_diagnostic),
            ],
            vec![CoreProvenance::new(
                CoreProvenancePhase::Generated,
                "fraenkel:comprehension:0",
            )],
        );
        let generated_choice =
            fixture.generated_origin(GeneratedOriginKind::StableChoice, "choice:0", 26);
        let error_diagnostic = fixture.diagnostic("generated-nonempty-error", 27);
        let generated_error_seed = fixture.obligation_seed(
            ObligationSeedKind::GeneratedNonEmptiness,
            None,
            "generated/choice/nonempty/0",
            None,
            "pkg::main::AlgorithmOwner.generated.choice.nonempty",
            28,
        );
        fixture.enrich_obligation_seed(
            generated_error_seed,
            ObligationSeedStatus::Error,
            vec![error_diagnostic],
            vec![
                CoreNodeRef::Generated(generated_choice),
                CoreNodeRef::Diagnostic(error_diagnostic),
            ],
            vec![CoreProvenance::new(
                CoreProvenancePhase::Generated,
                "choice:0",
            )],
        );
        let owner_item = fixture.item;
        let core = fixture.finish(Vec::new(), None, Vec::new());

        let handoff = build_obligation_seed_handoff(&core, &empty_control_flow_output());
        assert_eq!(handoff.entries.len(), 5);
        assert_eq!(handoff.source_map.len(), handoff.entries.len());
        let expected_existing_seeds = [
            theorem_seed,
            definition_seed,
            checker_seed,
            generated_deferred_seed,
            generated_error_seed,
        ];

        for (handoff_id, entry) in handoff.entries.iter() {
            let ObligationHandoffOrigin::ExistingCore { seed } = entry.origin else {
                panic!("existing core seed expected");
            };
            assert!(expected_existing_seeds.contains(&seed));
            let original = core.obligation_seeds().get(seed).expect("original seed");
            assert_eq!(&entry.seed, original);
            assert_eq!(
                handoff.source_map.get(&handoff_id),
                Some(&core.source_map().obligation_sources[&seed])
            );
            assert!(entry.flow_site.is_none());
        }

        let theorem = handoff
            .entries
            .iter()
            .map(|(_, entry)| entry)
            .find(|entry| entry.seed.kind == ObligationSeedKind::TheoremProof)
            .expect("theorem seed");
        assert_eq!(
            theorem.seed.label.as_ref().map(CoreLabelRef::as_str),
            Some("Thesis")
        );
        assert_eq!(theorem.seed.local_path.as_str(), "proof/terminal/0");
        assert!(
            theorem
                .seed
                .core_refs
                .contains(&CoreNodeRef::Formula(theorem_goal))
        );

        let generated_deferred = handoff
            .entries
            .iter()
            .map(|(_, entry)| entry)
            .find(|entry| {
                matches!(
                    entry.origin,
                    ObligationHandoffOrigin::ExistingCore { seed }
                        if seed == generated_deferred_seed
                )
            })
            .expect("generated deferred seed");
        assert_eq!(
            generated_deferred.seed.kind,
            ObligationSeedKind::GeneratedSethood
        );
        assert_eq!(
            generated_deferred.seed.status,
            ObligationSeedStatus::Deferred
        );
        assert_eq!(
            generated_deferred.seed.diagnostics,
            vec![deferred_diagnostic]
        );
        assert!(
            generated_deferred
                .seed
                .provenance
                .contains(&CoreProvenance::new(
                    CoreProvenancePhase::Generated,
                    "fraenkel:comprehension:0"
                ))
        );
        let mut expected_deferred_refs = vec![
            CoreNodeRef::Item(owner_item),
            CoreNodeRef::Formula(checker_goal),
            CoreNodeRef::Generated(generated_comprehension),
            CoreNodeRef::Diagnostic(deferred_diagnostic),
        ];
        sort_and_dedup(&mut expected_deferred_refs);
        assert_eq!(generated_deferred.seed.core_refs, expected_deferred_refs);

        let generated_error = handoff
            .entries
            .iter()
            .map(|(_, entry)| entry)
            .find(|entry| {
                matches!(
                    entry.origin,
                    ObligationHandoffOrigin::ExistingCore { seed } if seed == generated_error_seed
                )
            })
            .expect("generated error seed");
        assert_eq!(
            generated_error.seed.kind,
            ObligationSeedKind::GeneratedNonEmptiness
        );
        assert_eq!(generated_error.seed.status, ObligationSeedStatus::Error);
        assert_eq!(generated_error.seed.diagnostics, vec![error_diagnostic]);
        assert!(
            generated_error
                .seed
                .provenance
                .contains(&CoreProvenance::new(
                    CoreProvenancePhase::Generated,
                    "choice:0"
                ))
        );
        let mut expected_error_refs = vec![
            CoreNodeRef::Item(owner_item),
            CoreNodeRef::Generated(generated_choice),
            CoreNodeRef::Diagnostic(error_diagnostic),
        ];
        sort_and_dedup(&mut expected_error_refs);
        assert_eq!(generated_error.seed.core_refs, expected_error_refs);
    }

    #[test]
    fn obligation_handoff_emits_flow_contract_termination_and_ghost_seeds() {
        let mut fixture = CoreFixture::new();
        let term = fixture.term_var(0, 10);
        let requires = fixture.formula(CoreFormulaKind::True, 11);
        let ensures = fixture.formula(CoreFormulaKind::False, 12);
        let statement_assertion = fixture.formula(
            CoreFormulaKind::Equals {
                left: term,
                right: term,
            },
            13,
        );
        let contract_assertion = fixture.formula(CoreFormulaKind::True, 14);
        let contract_invariant = fixture.formula(CoreFormulaKind::False, 15);
        let decreasing = fixture.term_var(0, 16);
        let ghost_pick_binder = fixture.binder(1, "ghost-var", 17);
        let ghost_let_binder = fixture.binder(2, "ghost-const", 18);
        let ghost_pick = fixture.stmt(
            CoreAlgorithmStmtKind::Pick {
                binder: ghost_pick_binder,
                witness_ty: Some(requires),
                ghost: true,
            },
            20,
        );
        let ghost_let = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: ghost_let_binder,
                value: Some(term),
                ghost: true,
            },
            21,
        );
        let assert_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::Assert {
                formula: statement_assertion,
            },
            22,
        );
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(Some(term)), 23);
        let core = fixture.finish_with_contracts(
            Vec::new(),
            None,
            vec![ghost_pick, ghost_let, assert_stmt, return_stmt],
            CoreContractSet {
                requires: vec![requires],
                ensures: vec![ensures],
                invariants: vec![contract_invariant],
                assertions: vec![contract_assertion],
                decreasing: vec![decreasing],
            },
        );
        let first_flow = build_control_flow_ir(&core);
        let second_flow = build_control_flow_ir(&core);
        let flow = only_flow(&first_flow);

        let first = build_obligation_seed_handoff(&core, &first_flow);
        let second = build_obligation_seed_handoff(&core, &second_flow);
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.source_map.len(), first.entries.len());

        let flow_entries = flow_handoff_entries(&first);
        assert_eq!(flow_entries.len(), 9);
        assert!(
            flow_entries
                .iter()
                .all(|(id, entry)| source_map_matches_seed(&first, *id, entry))
        );
        assert!(flow_entries.iter().all(|(_, entry)| {
            matches!(
                entry.origin,
                ObligationHandoffOrigin::FlowDerived {
                    flow,
                    algorithm
                } if flow == ControlFlowId::new(0) && algorithm == CoreAlgorithmId::new(0)
            )
        }));
        assert!(
            flow_entries
                .iter()
                .all(|(_, entry)| entry.seed.status == ObligationSeedStatus::Deferred)
        );
        assert!(
            flow_entries
                .iter()
                .all(|(_, entry)| entry.seed.context.is_empty())
        );
        assert!(
            flow_entries
                .iter()
                .all(|(_, entry)| entry.seed.diagnostics.is_empty())
        );

        let site_kinds = flow_entries
            .iter()
            .map(|(_, entry)| entry.flow_site.as_ref().expect("flow site").kind)
            .collect::<Vec<_>>();
        for expected in [
            ControlFlowObligationSiteKind::Requires,
            ControlFlowObligationSiteKind::Ensures,
            ControlFlowObligationSiteKind::AlgorithmAssertion,
            ControlFlowObligationSiteKind::StatementAssertion,
            ControlFlowObligationSiteKind::AlgorithmInvariant,
            ControlFlowObligationSiteKind::TerminationMeasure,
            ControlFlowObligationSiteKind::GhostPick,
            ControlFlowObligationSiteKind::GhostAssignment,
        ] {
            assert!(
                site_kinds.contains(&expected),
                "missing site kind {expected:?}"
            );
        }
        assert_eq!(
            site_kinds
                .iter()
                .filter(|kind| **kind == ControlFlowObligationSiteKind::GhostAssignment)
                .count(),
            2
        );

        let requires_seed =
            flow_entry_by_kind(&flow_entries, ControlFlowObligationSiteKind::Requires, 0);
        assert_eq!(
            requires_seed.seed.kind,
            ObligationSeedKind::AlgorithmContract
        );
        assert_eq!(requires_seed.seed.goal, Some(requires));
        assert_eq!(
            requires_seed.seed.semantic_origin.as_str(),
            "flow:0:requires:0"
        );
        assert_flow_seed_source(
            &first,
            requires_seed,
            &core.source_map().formula_sources[&requires],
            "flow-handoff:requires:0",
        );
        assert_flow_seed_core_refs(flow, requires_seed, &[CoreNodeRef::Formula(requires)]);
        assert_eq!(
            requires_seed.seed.local_path.as_str(),
            "program/0/contract/requires/0"
        );
        assert_eq!(
            requires_seed.flow_site.as_ref().expect("requires site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::Requires,
                ordinal: 0,
                statement: None,
                block: Some(BasicBlockId::new(0)),
                loop_id: None,
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let ensures_seed =
            flow_entry_by_kind(&flow_entries, ControlFlowObligationSiteKind::Ensures, 0);
        assert_eq!(ensures_seed.seed.goal, Some(ensures));
        assert_eq!(
            ensures_seed.seed.semantic_origin.as_str(),
            "flow:0:ensures:0"
        );
        assert_flow_seed_source(
            &first,
            ensures_seed,
            &core.source_map().formula_sources[&ensures],
            "flow-handoff:ensures:0",
        );
        assert_flow_seed_core_refs(
            flow,
            ensures_seed,
            &[
                CoreNodeRef::Formula(ensures),
                CoreNodeRef::AlgorithmStmt(return_stmt),
            ],
        );
        assert_eq!(
            ensures_seed.seed.local_path.as_str(),
            "program/0/contract/ensures/0"
        );
        let (return_exit_id, return_exit) = flow
            .exits
            .iter()
            .find(|(_, exit)| exit.statement == Some(return_stmt))
            .expect("return exit");
        assert_eq!(return_exit.kind, ControlFlowExitKind::Return);
        assert_eq!(
            ensures_seed.flow_site.as_ref().expect("ensures site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::Ensures,
                ordinal: 0,
                statement: Some(return_stmt),
                block: Some(return_exit.from),
                loop_id: None,
                exit: Some(return_exit_id),
                local: None,
                assignment_effect: None,
            }
        );

        let algorithm_assertion_seed = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::AlgorithmAssertion,
            0,
        );
        assert_eq!(algorithm_assertion_seed.seed.goal, Some(contract_assertion));
        assert_eq!(
            algorithm_assertion_seed.seed.semantic_origin.as_str(),
            "flow:0:assertion:0"
        );
        assert_eq!(
            algorithm_assertion_seed.seed.local_path.as_str(),
            "program/0/assertion/0"
        );
        assert_flow_seed_source(
            &first,
            algorithm_assertion_seed,
            &core.source_map().formula_sources[&contract_assertion],
            "flow-handoff:assertion:0",
        );
        assert_flow_seed_core_refs(
            flow,
            algorithm_assertion_seed,
            &[CoreNodeRef::Formula(contract_assertion)],
        );
        assert_eq!(
            algorithm_assertion_seed
                .flow_site
                .as_ref()
                .expect("algorithm assertion site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::AlgorithmAssertion,
                ordinal: 0,
                statement: None,
                block: Some(BasicBlockId::new(0)),
                loop_id: None,
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let statement_assertion_seed = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::StatementAssertion,
            1,
        );
        assert_eq!(
            statement_assertion_seed.seed.goal,
            Some(statement_assertion)
        );
        assert_eq!(
            statement_assertion_seed.seed.semantic_origin.as_str(),
            "flow:0:assertion:1"
        );
        assert_eq!(
            statement_assertion_seed.seed.local_path.as_str(),
            "program/0/assertion/1"
        );
        assert_flow_seed_source(
            &first,
            statement_assertion_seed,
            &core.source_map().algorithm_sources[&assert_stmt],
            "flow-handoff:assertion:1",
        );
        assert_flow_seed_core_refs(
            flow,
            statement_assertion_seed,
            &[
                CoreNodeRef::Formula(statement_assertion),
                CoreNodeRef::AlgorithmStmt(assert_stmt),
            ],
        );
        assert_eq!(
            statement_assertion_seed
                .flow_site
                .as_ref()
                .expect("statement assertion site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::StatementAssertion,
                ordinal: 1,
                statement: Some(assert_stmt),
                block: Some(BasicBlockId::new(0)),
                loop_id: None,
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let algorithm_invariant_seed = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::AlgorithmInvariant,
            0,
        );
        assert_eq!(algorithm_invariant_seed.seed.goal, Some(contract_invariant));
        assert_eq!(
            algorithm_invariant_seed.seed.semantic_origin.as_str(),
            "flow:0:invariant:0"
        );
        assert_eq!(
            algorithm_invariant_seed.seed.local_path.as_str(),
            "program/0/invariant/0"
        );
        assert_flow_seed_source(
            &first,
            algorithm_invariant_seed,
            &core.source_map().formula_sources[&contract_invariant],
            "flow-handoff:invariant:0",
        );
        assert_flow_seed_core_refs(
            flow,
            algorithm_invariant_seed,
            &[CoreNodeRef::Formula(contract_invariant)],
        );
        assert_eq!(
            algorithm_invariant_seed
                .flow_site
                .as_ref()
                .expect("algorithm invariant site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::AlgorithmInvariant,
                ordinal: 0,
                statement: None,
                block: Some(BasicBlockId::new(0)),
                loop_id: None,
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let termination_seed = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::TerminationMeasure,
            0,
        );
        assert_eq!(
            termination_seed.seed.kind,
            ObligationSeedKind::AlgorithmTermination
        );
        assert_eq!(termination_seed.seed.goal, None);
        assert_eq!(
            termination_seed.seed.semantic_origin.as_str(),
            "flow:0:termination:measure:0"
        );
        assert_flow_seed_source(
            &first,
            termination_seed,
            &core.source_map().term_sources[&decreasing],
            "flow-handoff:termination-measure:0",
        );
        assert_flow_seed_core_refs(flow, termination_seed, &[CoreNodeRef::Term(decreasing)]);
        assert_eq!(
            termination_seed.seed.local_path.as_str(),
            "program/0/termination/measure/0"
        );
        assert_eq!(
            termination_seed
                .flow_site
                .as_ref()
                .expect("termination site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::TerminationMeasure,
                ordinal: 0,
                statement: None,
                block: Some(BasicBlockId::new(0)),
                loop_id: None,
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let ghost_pick_seed =
            flow_entry_by_kind(&flow_entries, ControlFlowObligationSiteKind::GhostPick, 0);
        let expected_ghost_pick_local = flow.ghost_effects.ghost_pick_locals[0];
        assert_eq!(flow.ghost_effects.ghost_pick_locals, vec![LocalId::new(0)]);
        assert_eq!(expected_ghost_pick_local, LocalId::new(0));
        let expected_ghost_pick_row = flow
            .locals
            .get(expected_ghost_pick_local)
            .expect("ghost pick local row");
        assert_eq!(ghost_pick_seed.seed.kind, ObligationSeedKind::GhostErasure);
        assert_eq!(ghost_pick_seed.seed.goal, None);
        assert_eq!(
            ghost_pick_seed.seed.semantic_origin.as_str(),
            "flow:0:ghost:pick:0"
        );
        assert_eq!(
            ghost_pick_seed.seed.local_path.as_str(),
            "program/0/ghost/pick/0"
        );
        assert_flow_seed_source(
            &first,
            ghost_pick_seed,
            &expected_ghost_pick_row.source,
            "flow-handoff:ghost-pick:0",
        );
        assert_flow_seed_core_refs(
            flow,
            ghost_pick_seed,
            &[CoreNodeRef::AlgorithmStmt(ghost_pick)],
        );
        assert_eq!(
            ghost_pick_seed.flow_site.as_ref().expect("ghost pick site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::GhostPick,
                ordinal: 0,
                statement: Some(ghost_pick),
                block: Some(BasicBlockId::new(0)),
                loop_id: None,
                exit: None,
                local: Some(LocalId::new(0)),
                assignment_effect: None,
            }
        );
        let ghost_assignment_pick = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::GhostAssignment,
            0,
        );
        let expected_pick_effect = flow
            .assignment_effects
            .iter()
            .find_map(|(effect_id, effect)| {
                (effect.statement == ghost_pick).then_some((effect_id, effect))
            })
            .expect("ghost pick assignment effect");
        let expected_pick_local = match expected_pick_effect.1.target {
            AssignmentEffectTarget::Local(local) => local,
            AssignmentEffectTarget::Place(_) => panic!("ghost pick should assign a local"),
        };
        assert_eq!(
            ghost_assignment_pick.seed.local_path.as_str(),
            "program/0/ghost/assignment/0"
        );
        assert_eq!(
            ghost_assignment_pick.seed.semantic_origin.as_str(),
            "flow:0:ghost:assignment:0"
        );
        assert_flow_seed_source(
            &first,
            ghost_assignment_pick,
            &core.source_map().algorithm_sources[&ghost_pick],
            "flow-handoff:ghost-assignment:0",
        );
        assert_flow_seed_core_refs(
            flow,
            ghost_assignment_pick,
            &[CoreNodeRef::AlgorithmStmt(ghost_pick)],
        );
        assert_eq!(
            ghost_assignment_pick
                .flow_site
                .as_ref()
                .expect("ghost assignment site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::GhostAssignment,
                ordinal: 0,
                statement: Some(ghost_pick),
                block: Some(BasicBlockId::new(0)),
                loop_id: None,
                exit: None,
                local: Some(expected_pick_local),
                assignment_effect: Some(expected_pick_effect.0),
            }
        );
        let ghost_assignment_let = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::GhostAssignment,
            1,
        );
        let expected_let_effect = flow
            .assignment_effects
            .iter()
            .find_map(|(effect_id, effect)| {
                (effect.statement == ghost_let).then_some((effect_id, effect))
            })
            .expect("ghost let assignment effect");
        let expected_let_local = match expected_let_effect.1.target {
            AssignmentEffectTarget::Local(local) => local,
            AssignmentEffectTarget::Place(_) => panic!("ghost let should assign a local"),
        };
        assert_eq!(
            ghost_assignment_let.seed.local_path.as_str(),
            "program/0/ghost/assignment/1"
        );
        assert_eq!(
            ghost_assignment_let.seed.semantic_origin.as_str(),
            "flow:0:ghost:assignment:1"
        );
        assert_flow_seed_source(
            &first,
            ghost_assignment_let,
            &core.source_map().algorithm_sources[&ghost_let],
            "flow-handoff:ghost-assignment:1",
        );
        assert_flow_seed_core_refs(
            flow,
            ghost_assignment_let,
            &[CoreNodeRef::AlgorithmStmt(ghost_let)],
        );
        assert_eq!(
            ghost_assignment_let
                .flow_site
                .as_ref()
                .expect("ghost assignment site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::GhostAssignment,
                ordinal: 1,
                statement: Some(ghost_let),
                block: Some(BasicBlockId::new(0)),
                loop_id: None,
                exit: None,
                local: Some(expected_let_local),
                assignment_effect: Some(expected_let_effect.0),
            }
        );
        assert!(
            flow_entries
                .iter()
                .filter(|(_, entry)| entry.seed.kind == ObligationSeedKind::GhostErasure)
                .all(
                    |(_, entry)| !entry.seed.semantic_origin.as_str().contains("VcId")
                        && !entry
                            .seed
                            .semantic_origin
                            .as_str()
                            .contains("ObligationAnchor")
                )
        );
    }

    #[test]
    fn obligation_handoff_covers_loop_partial_and_combined_ordering() {
        let mut fixture = CoreFixture::new();
        let existing_goal = fixture.formula(CoreFormulaKind::True, 5);
        let early_seed = fixture.obligation_seed(
            ObligationSeedKind::CheckerInitial,
            Some(existing_goal),
            "checker/early",
            Some("Early"),
            "pkg::main::AlgorithmOwner.checker.early",
            0,
        );
        let term = fixture.term_var(0, 10);
        let condition = fixture.formula(CoreFormulaKind::True, 11);
        let continue_invariant = fixture.formula(CoreFormulaKind::False, 12);
        let partial_invariant = fixture.formula(CoreFormulaKind::True, 13);
        let loop_decreasing = fixture.term_var(0, 14);
        let continue_stmt = fixture.stmt(CoreAlgorithmStmtKind::Continue, 20);
        let continue_loop = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition,
                invariants: vec![continue_invariant],
                decreasing: vec![loop_decreasing],
                body: vec![continue_stmt],
            },
            21,
        );
        let normal_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("loop_target"),
                value: term,
            },
            30,
        );
        let partial_loop = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition,
                invariants: vec![partial_invariant],
                decreasing: Vec::new(),
                body: vec![normal_assign],
            },
            31,
        );
        let late_seed = fixture.obligation_seed(
            ObligationSeedKind::TheoremProof,
            Some(existing_goal),
            "proof/late",
            Some("Late"),
            "pkg::main::AlgorithmOwner.proof.late",
            90,
        );
        let core = fixture.finish(Vec::new(), None, vec![continue_loop, partial_loop]);
        let flow_output = build_control_flow_ir(&core);
        let flow = only_flow(&flow_output);
        let handoff = build_obligation_seed_handoff(&core, &flow_output);

        assert_eq!(handoff.entries.len(), 10);
        assert_eq!(handoff.source_map.len(), handoff.entries.len());
        for (id, entry) in handoff.entries.iter() {
            assert!(source_map_matches_seed(&handoff, id, entry));
        }
        let ordered_origins = handoff
            .entries
            .iter()
            .map(|(_, entry)| &entry.origin)
            .collect::<Vec<_>>();
        assert!(matches!(
            ordered_origins.first(),
            Some(ObligationHandoffOrigin::ExistingCore { seed }) if *seed == early_seed
        ));
        assert!(matches!(
            ordered_origins.last(),
            Some(ObligationHandoffOrigin::ExistingCore { seed }) if *seed == late_seed
        ));
        let (continue_exit_id, _) = flow
            .exits
            .iter()
            .find(|(_, exit)| exit.statement == Some(continue_stmt))
            .expect("continue exit");
        let ordered_signature = handoff
            .entries
            .iter()
            .map(|(_, entry)| match &entry.origin {
                ObligationHandoffOrigin::ExistingCore { seed } => {
                    format!("core:{}:{}", seed.index(), entry.seed.local_path.as_str())
                }
                ObligationHandoffOrigin::FlowDerived { flow, algorithm } => {
                    let site = entry.flow_site.as_ref().expect("flow site");
                    format!(
                        "flow:{}:{}:{:?}:{}:stmt={:?}:block={:?}:loop={:?}:exit={:?}:local={:?}:effect={:?}",
                        flow.index(),
                        algorithm.index(),
                        site.kind,
                        entry.seed.local_path.as_str(),
                        site.statement.map(CoreAlgorithmStmtId::index),
                        site.block.map(BasicBlockId::index),
                        site.loop_id.map(LoopId::index),
                        site.exit.map(ControlFlowExitId::index),
                        site.local.map(LocalId::index),
                        site.assignment_effect.map(AssignmentEffectId::index),
                    )
                }
            })
            .collect::<Vec<_>>();
        let loop0 = flow.loops.get(LoopId::new(0)).expect("loop 0");
        let loop1 = flow.loops.get(LoopId::new(1)).expect("loop 1");
        let algorithm_partial_block = flow.entry.index();
        let partial_loop_block = loop1.header.index();
        let header0 = loop0.header.index();
        let body0 = loop0.body.index();
        let header1 = loop1.header.index();
        let body1 = loop1.body.index();
        let continue_exit_index = continue_exit_id.index();
        assert_eq!(
            ordered_signature,
            vec![
                format!("core:{}:checker/early", early_seed.index()),
                format!(
                    "flow:0:0:PartialTermination:program/0/termination/partial/0:stmt=None:block=Some({algorithm_partial_block}):loop=None:exit=None:local=None:effect=None"
                ),
                format!(
                    "flow:0:0:LoopInvariant:program/0/invariant/0:stmt=None:block=Some({header0}):loop=Some(0):exit=None:local=None:effect=None"
                ),
                format!(
                    "flow:0:0:LoopInvariant:program/0/invariant/1:stmt=None:block=Some({body0}):loop=Some(0):exit=Some({continue_exit_index}):local=None:effect=None"
                ),
                format!(
                    "flow:0:0:LoopInvariant:program/0/invariant/2:stmt=None:block=Some({header1}):loop=Some(1):exit=None:local=None:effect=None"
                ),
                format!(
                    "flow:0:0:LoopInvariant:program/0/invariant/3:stmt=None:block=Some({body1}):loop=Some(1):exit=None:local=None:effect=None"
                ),
                format!(
                    "flow:0:0:TerminationMeasure:program/0/termination/measure/0:stmt=None:block=Some({header0}):loop=Some(0):exit=None:local=None:effect=None"
                ),
                format!(
                    "flow:0:0:TerminationMeasure:program/0/termination/measure/1:stmt=None:block=Some({body0}):loop=Some(0):exit=Some({continue_exit_index}):local=None:effect=None"
                ),
                format!(
                    "flow:0:0:PartialTermination:program/0/termination/partial/1:stmt=None:block=Some({partial_loop_block}):loop=Some(1):exit=None:local=None:effect=None"
                ),
                format!("core:{}:proof/late", late_seed.index()),
            ]
        );

        let flow_entries = flow_handoff_entries(&handoff);
        assert_eq!(flow_entries.len(), 8);
        assert!(
            flow_entries
                .iter()
                .all(|(_, entry)| entry.seed.status == ObligationSeedStatus::Deferred)
        );
        assert!(
            flow_entries
                .iter()
                .all(|(_, entry)| entry.seed.context.is_empty())
        );
        assert!(
            flow_entries
                .iter()
                .all(|(id, entry)| source_map_matches_seed(&handoff, *id, entry))
        );
        assert_eq!(
            flow_entries
                .iter()
                .filter(|(_, entry)| entry
                    .flow_site
                    .as_ref()
                    .is_some_and(|site| site.kind == ControlFlowObligationSiteKind::LoopInvariant))
                .count(),
            4
        );
        assert_eq!(
            flow_entries
                .iter()
                .filter(|(_, entry)| entry.flow_site.as_ref().is_some_and(|site| {
                    site.kind == ControlFlowObligationSiteKind::TerminationMeasure
                }))
                .count(),
            2
        );
        assert_eq!(
            flow_entries
                .iter()
                .filter(|(_, entry)| entry.flow_site.as_ref().is_some_and(|site| {
                    site.kind == ControlFlowObligationSiteKind::PartialTermination
                }))
                .count(),
            2
        );

        let algorithm_partial_entry = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::PartialTermination,
            0,
        );
        assert_eq!(
            algorithm_partial_entry.seed.kind,
            ObligationSeedKind::AlgorithmTermination
        );
        assert_eq!(algorithm_partial_entry.seed.goal, None);
        assert_eq!(
            algorithm_partial_entry.seed.semantic_origin.as_str(),
            "flow:0:termination:partial:0"
        );
        assert_flow_seed_core_refs(flow, algorithm_partial_entry, &[]);
        assert_flow_seed_source(
            &handoff,
            algorithm_partial_entry,
            &core
                .algorithms()
                .get(flow.algorithm)
                .expect("algorithm")
                .source,
            "flow-handoff:partial-termination:0",
        );
        assert_eq!(
            algorithm_partial_entry
                .flow_site
                .as_ref()
                .expect("algorithm partial site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::PartialTermination,
                ordinal: 0,
                statement: None,
                block: Some(flow.entry),
                loop_id: None,
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let header_invariant_entry = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::LoopInvariant,
            0,
        );
        assert_eq!(header_invariant_entry.seed.goal, Some(continue_invariant));
        assert_eq!(
            header_invariant_entry.seed.semantic_origin.as_str(),
            "flow:0:invariant:0"
        );
        assert_flow_seed_core_refs(
            flow,
            header_invariant_entry,
            &[CoreNodeRef::Formula(continue_invariant)],
        );
        assert_flow_seed_source(
            &handoff,
            header_invariant_entry,
            &core.source_map().formula_sources[&continue_invariant],
            "flow-handoff:invariant:0",
        );
        assert_eq!(
            header_invariant_entry
                .flow_site
                .as_ref()
                .expect("header invariant site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::LoopInvariant,
                ordinal: 0,
                statement: None,
                block: Some(loop0.header),
                loop_id: Some(LoopId::new(0)),
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let continue_invariant_ordinal = flow
            .contracts
            .loop_invariants
            .iter()
            .position(|site| {
                matches!(
                    site.placement,
                    LoopInvariantPlacement::ContinueExit { loop_id, exit }
                        if loop_id == LoopId::new(0) && exit == continue_exit_id
                )
            })
            .expect("continue invariant site");
        let continue_invariant_entry = flow_entries
            .iter()
            .find(|(_, entry)| {
                entry.flow_site.as_ref().is_some_and(|site| {
                    site.kind == ControlFlowObligationSiteKind::LoopInvariant
                        && site.loop_id == Some(LoopId::new(0))
                        && site.exit == Some(continue_exit_id)
                })
            })
            .map(|(_, entry)| *entry)
            .expect("continue invariant handoff");
        assert_eq!(continue_invariant_entry.seed.goal, Some(continue_invariant));
        assert_eq!(
            continue_invariant_entry.seed.semantic_origin.as_str(),
            format!("flow:0:invariant:{continue_invariant_ordinal}")
        );
        assert_flow_seed_core_refs(
            flow,
            continue_invariant_entry,
            &[CoreNodeRef::Formula(continue_invariant)],
        );
        assert_flow_seed_source(
            &handoff,
            continue_invariant_entry,
            &core.source_map().formula_sources[&continue_invariant],
            &format!("flow-handoff:invariant:{continue_invariant_ordinal}"),
        );
        assert_eq!(
            continue_invariant_entry.seed.local_path.as_str(),
            format!("program/0/invariant/{continue_invariant_ordinal}")
        );
        assert_eq!(
            continue_invariant_entry
                .flow_site
                .as_ref()
                .expect("continue invariant site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::LoopInvariant,
                ordinal: continue_invariant_ordinal,
                statement: None,
                block: Some(loop0.body),
                loop_id: Some(LoopId::new(0)),
                exit: Some(continue_exit_id),
                local: None,
                assignment_effect: None,
            }
        );

        let partial_header_invariant_entry = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::LoopInvariant,
            2,
        );
        assert_eq!(
            partial_header_invariant_entry.seed.goal,
            Some(partial_invariant)
        );
        assert_eq!(
            partial_header_invariant_entry.seed.semantic_origin.as_str(),
            "flow:0:invariant:2"
        );
        assert_flow_seed_core_refs(
            flow,
            partial_header_invariant_entry,
            &[CoreNodeRef::Formula(partial_invariant)],
        );
        assert_flow_seed_source(
            &handoff,
            partial_header_invariant_entry,
            &core.source_map().formula_sources[&partial_invariant],
            "flow-handoff:invariant:2",
        );
        assert_eq!(
            partial_header_invariant_entry
                .flow_site
                .as_ref()
                .expect("partial header invariant site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::LoopInvariant,
                ordinal: 2,
                statement: None,
                block: Some(loop1.header),
                loop_id: Some(LoopId::new(1)),
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let normal_backedge_invariant_entry = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::LoopInvariant,
            3,
        );
        assert_eq!(
            normal_backedge_invariant_entry.seed.goal,
            Some(partial_invariant)
        );
        assert_eq!(
            normal_backedge_invariant_entry
                .seed
                .semantic_origin
                .as_str(),
            "flow:0:invariant:3"
        );
        assert_flow_seed_core_refs(
            flow,
            normal_backedge_invariant_entry,
            &[CoreNodeRef::Formula(partial_invariant)],
        );
        assert_flow_seed_source(
            &handoff,
            normal_backedge_invariant_entry,
            &core.source_map().formula_sources[&partial_invariant],
            "flow-handoff:invariant:3",
        );
        assert_eq!(
            normal_backedge_invariant_entry
                .flow_site
                .as_ref()
                .expect("normal backedge invariant site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::LoopInvariant,
                ordinal: 3,
                statement: None,
                block: Some(loop1.body),
                loop_id: Some(LoopId::new(1)),
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let loop_header_measure_entry = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::TerminationMeasure,
            0,
        );
        assert_eq!(
            loop_header_measure_entry.seed.kind,
            ObligationSeedKind::AlgorithmTermination
        );
        assert_eq!(loop_header_measure_entry.seed.goal, None);
        assert_eq!(
            loop_header_measure_entry.seed.semantic_origin.as_str(),
            "flow:0:termination:measure:0"
        );
        assert_flow_seed_core_refs(
            flow,
            loop_header_measure_entry,
            &[CoreNodeRef::Term(loop_decreasing)],
        );
        assert_flow_seed_source(
            &handoff,
            loop_header_measure_entry,
            &core.source_map().term_sources[&loop_decreasing],
            "flow-handoff:termination-measure:0",
        );
        assert_eq!(
            loop_header_measure_entry
                .flow_site
                .as_ref()
                .expect("loop header measure site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::TerminationMeasure,
                ordinal: 0,
                statement: None,
                block: Some(loop0.header),
                loop_id: Some(LoopId::new(0)),
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        let continue_measure_ordinal = flow
            .contracts
            .decreasing
            .iter()
            .position(|site| {
                matches!(
                    site.placement,
                    TerminationMeasurePlacement::ContinueEdge { loop_id, exit }
                        if loop_id == LoopId::new(0) && exit == continue_exit_id
                )
            })
            .expect("continue termination measure");
        let continue_measure_entry = flow_entries
            .iter()
            .find(|(_, entry)| {
                entry.flow_site.as_ref().is_some_and(|site| {
                    site.kind == ControlFlowObligationSiteKind::TerminationMeasure
                        && site.loop_id == Some(LoopId::new(0))
                        && site.exit == Some(continue_exit_id)
                })
            })
            .map(|(_, entry)| *entry)
            .expect("continue termination handoff");
        assert_eq!(
            continue_measure_entry.seed.kind,
            ObligationSeedKind::AlgorithmTermination
        );
        assert_eq!(continue_measure_entry.seed.goal, None);
        assert_eq!(
            continue_measure_entry.seed.semantic_origin.as_str(),
            format!("flow:0:termination:measure:{continue_measure_ordinal}")
        );
        assert_flow_seed_core_refs(
            flow,
            continue_measure_entry,
            &[CoreNodeRef::Term(loop_decreasing)],
        );
        assert_flow_seed_source(
            &handoff,
            continue_measure_entry,
            &core.source_map().term_sources[&loop_decreasing],
            &format!("flow-handoff:termination-measure:{continue_measure_ordinal}"),
        );
        assert_eq!(
            continue_measure_entry.seed.local_path.as_str(),
            format!("program/0/termination/measure/{continue_measure_ordinal}")
        );
        assert_eq!(
            continue_measure_entry
                .flow_site
                .as_ref()
                .expect("continue termination site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::TerminationMeasure,
                ordinal: continue_measure_ordinal,
                statement: None,
                block: Some(loop0.body),
                loop_id: Some(LoopId::new(0)),
                exit: Some(continue_exit_id),
                local: None,
                assignment_effect: None,
            }
        );

        let partial_loop_ordinal = flow
            .termination
            .partial_sites
            .iter()
            .position(|site| {
                matches!(site.kind, TerminationSiteKind::Loop(loop_id) if loop_id == LoopId::new(1))
            })
            .expect("partial loop site");
        let partial_loop_entry = flow_entries
            .iter()
            .find(|(_, entry)| {
                entry.flow_site.as_ref().is_some_and(|site| {
                    site.kind == ControlFlowObligationSiteKind::PartialTermination
                        && site.loop_id == Some(LoopId::new(1))
                })
            })
            .map(|(_, entry)| *entry)
            .expect("partial loop handoff");
        assert_eq!(
            partial_loop_entry.seed.kind,
            ObligationSeedKind::AlgorithmTermination
        );
        assert_eq!(partial_loop_entry.seed.goal, None);
        assert_eq!(
            partial_loop_entry.seed.semantic_origin.as_str(),
            format!("flow:0:termination:partial:{partial_loop_ordinal}")
        );
        assert_flow_seed_core_refs(flow, partial_loop_entry, &[]);
        assert_flow_seed_source(
            &handoff,
            partial_loop_entry,
            &core.source_map().algorithm_sources[&partial_loop],
            &format!("flow-handoff:partial-termination:{partial_loop_ordinal}"),
        );
        assert_eq!(
            partial_loop_entry.seed.local_path.as_str(),
            format!("program/0/termination/partial/{partial_loop_ordinal}")
        );
        assert_eq!(
            partial_loop_entry
                .flow_site
                .as_ref()
                .expect("partial loop site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::PartialTermination,
                ordinal: partial_loop_ordinal,
                statement: None,
                block: Some(flow.loops.get(LoopId::new(1)).expect("partial loop").header),
                loop_id: Some(LoopId::new(1)),
                exit: None,
                local: None,
                assignment_effect: None,
            }
        );

        assert!(flow_entries.iter().all(|(_, entry)| {
            !entry.seed.semantic_origin.as_str().contains("VcId")
                && !entry
                    .seed
                    .semantic_origin
                    .as_str()
                    .contains("ObligationAnchor")
                && !entry.seed.semantic_origin.as_str().contains("SourceRange")
        }));
    }

    #[test]
    fn obligation_handoff_covers_break_exit_loop_invariant() {
        let mut fixture = CoreFixture::new();
        let condition = fixture.formula(CoreFormulaKind::True, 10);
        let break_invariant = fixture.formula(CoreFormulaKind::False, 11);
        let algorithm_decreasing = fixture.term_var(0, 12);
        let loop_decreasing = fixture.term_var(0, 13);
        let break_stmt = fixture.stmt(CoreAlgorithmStmtKind::Break, 20);
        let break_loop = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition,
                invariants: vec![break_invariant],
                decreasing: vec![loop_decreasing],
                body: vec![break_stmt],
            },
            21,
        );
        let core = fixture.finish_with_contracts(
            Vec::new(),
            None,
            vec![break_loop],
            CoreContractSet {
                decreasing: vec![algorithm_decreasing],
                ..CoreContractSet::default()
            },
        );
        let flow_output = build_control_flow_ir(&core);
        let flow = only_flow(&flow_output);
        let handoff = build_obligation_seed_handoff(&core, &flow_output);
        let flow_entries = flow_handoff_entries(&handoff);

        assert_eq!(flow_entries.len(), 4);
        assert!(
            flow_entries
                .iter()
                .all(|(_, entry)| entry.seed.status == ObligationSeedStatus::Deferred)
        );
        assert!(
            flow_entries
                .iter()
                .all(|(_, entry)| entry.seed.context.is_empty())
        );
        assert!(
            flow_entries
                .iter()
                .all(|(id, entry)| source_map_matches_seed(&handoff, *id, entry))
        );
        let (break_exit_id, break_exit) = flow
            .exits
            .iter()
            .find(|(_, exit)| {
                exit.statement == Some(break_stmt)
                    && matches!(exit.kind, ControlFlowExitKind::Break { .. })
            })
            .expect("break exit");
        assert!(matches!(
            break_exit.kind,
            ControlFlowExitKind::Break {
                loop_id
            } if loop_id == LoopId::new(0)
        ));

        let break_invariant_ordinal = flow
            .contracts
            .loop_invariants
            .iter()
            .position(|site| {
                matches!(
                    site.placement,
                    LoopInvariantPlacement::BreakExit { loop_id, exit }
                        if loop_id == LoopId::new(0) && exit == break_exit_id
                )
            })
            .expect("break invariant site");
        let break_invariant_entry = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::LoopInvariant,
            break_invariant_ordinal,
        );
        assert_eq!(break_invariant_entry.seed.goal, Some(break_invariant));
        assert_eq!(
            break_invariant_entry.seed.semantic_origin.as_str(),
            format!("flow:0:invariant:{break_invariant_ordinal}")
        );
        assert_eq!(
            break_invariant_entry.seed.local_path.as_str(),
            format!("program/0/invariant/{break_invariant_ordinal}")
        );
        assert_flow_seed_core_refs(
            flow,
            break_invariant_entry,
            &[CoreNodeRef::Formula(break_invariant)],
        );
        assert_flow_seed_source(
            &handoff,
            break_invariant_entry,
            &core.source_map().formula_sources[&break_invariant],
            &format!("flow-handoff:invariant:{break_invariant_ordinal}"),
        );
        assert_eq!(
            break_invariant_entry
                .flow_site
                .as_ref()
                .expect("break invariant site"),
            &ControlFlowObligationSite {
                kind: ControlFlowObligationSiteKind::LoopInvariant,
                ordinal: break_invariant_ordinal,
                statement: None,
                block: Some(break_exit.from),
                loop_id: Some(LoopId::new(0)),
                exit: Some(break_exit_id),
                local: None,
                assignment_effect: None,
            }
        );

        let algorithm_measure = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::TerminationMeasure,
            0,
        );
        assert_eq!(
            algorithm_measure.seed.semantic_origin.as_str(),
            "flow:0:termination:measure:0"
        );
        assert_flow_seed_core_refs(
            flow,
            algorithm_measure,
            &[CoreNodeRef::Term(algorithm_decreasing)],
        );
        assert_flow_seed_source(
            &handoff,
            algorithm_measure,
            &core.source_map().term_sources[&algorithm_decreasing],
            "flow-handoff:termination-measure:0",
        );

        let loop_measure = flow_entry_by_kind(
            &flow_entries,
            ControlFlowObligationSiteKind::TerminationMeasure,
            1,
        );
        assert_eq!(
            loop_measure.seed.semantic_origin.as_str(),
            "flow:0:termination:measure:1"
        );
        assert_flow_seed_core_refs(flow, loop_measure, &[CoreNodeRef::Term(loop_decreasing)]);
        assert_flow_seed_source(
            &handoff,
            loop_measure,
            &core.source_map().term_sources[&loop_decreasing],
            "flow-handoff:termination-measure:1",
        );
    }

    #[test]
    fn control_flow_lowers_straight_line_locals_sources_and_debug_text() {
        let mut fixture = CoreFixture::new();
        let term_x = fixture.term_var(0, 10);
        let term_y = fixture.term_var(0, 11);
        let condition = fixture.formula(CoreFormulaKind::True, 12);
        let param = fixture.binder(0, "param", 20);
        let result = fixture.binder(1, "result", 21);
        let let_binder = fixture.binder(2, "local:var", 30);
        let let_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: let_binder.clone(),
                value: Some(term_x),
                ghost: false,
            },
            31,
        );
        let runtime_pick_binder = fixture.binder(3, "local:const", 32);
        let runtime_pick = fixture.stmt(
            CoreAlgorithmStmtKind::Pick {
                binder: runtime_pick_binder.clone(),
                witness_ty: Some(condition),
                ghost: false,
            },
            33,
        );
        let ghost_pick_binder = fixture.binder(4, "local:const", 34);
        let ghost_pick = fixture.stmt(
            CoreAlgorithmStmtKind::Pick {
                binder: ghost_pick_binder.clone(),
                witness_ty: Some(condition),
                ghost: true,
            },
            35,
        );
        let ghost_let_binder = fixture.binder(5, "local:const", 36);
        let ghost_let = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: ghost_let_binder.clone(),
                value: Some(term_y),
                ghost: true,
            },
            37,
        );
        let unsupported_binder = fixture.binder(6, "checker:future-local", 38);
        let unsupported = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: unsupported_binder.clone(),
                value: None,
                ghost: false,
            },
            39,
        );
        let assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("result"),
                value: term_y,
            },
            40,
        );
        let assert_stmt = fixture.stmt(CoreAlgorithmStmtKind::Assert { formula: condition }, 41);
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(Some(term_y)), 42);
        let core = fixture.finish(
            vec![param.clone()],
            Some(result.clone()),
            vec![
                let_stmt,
                runtime_pick,
                ghost_pick,
                ghost_let,
                unsupported,
                assign,
                assert_stmt,
                return_stmt,
            ],
        );

        let first = build_control_flow_ir(&core);
        let second = build_control_flow_ir(&core);
        assert_eq!(first, second);
        assert_eq!(first.debug_text(), second.debug_text());
        let flow = only_flow(&first);
        assert!(flow.debug_text().contains("control-flow-ir-debug-v1"));
        assert_eq!(flow.locals.len(), 7);
        assert!(
            flow.locals
                .iter()
                .all(|(_, local)| local.kind != LocalKind::HiddenLoopValue)
        );
        let param_local = flow.locals.get(LocalId::new(0)).expect("param local");
        assert_eq!(param_local.kind, LocalKind::Parameter);
        assert_eq!(param_local.declaration, LocalDeclaration::Parameter);
        assert_eq!(param_local.mutability, LocalMutability::Immutable);
        assert_eq!(param_local.initialized_at, None);
        assert_eq!(param_local.source, param.source);
        assert_eq!(
            flow.source_map.local_sources[&LocalId::new(0)],
            param.source
        );
        let result_local = flow.locals.get(LocalId::new(1)).expect("result local");
        assert_eq!(result_local.kind, LocalKind::Result);
        assert_eq!(result_local.declaration, LocalDeclaration::Result);
        assert_eq!(result_local.mutability, LocalMutability::Immutable);
        assert_eq!(result_local.initialized_at, None);
        assert_eq!(result_local.source, result.source);
        assert_eq!(
            flow.source_map.local_sources[&LocalId::new(1)],
            result.source
        );
        let let_local = flow.locals.get(LocalId::new(2)).expect("let local");
        assert_eq!(let_local.kind, LocalKind::Let);
        assert_eq!(let_local.declaration, LocalDeclaration::Var);
        assert_eq!(let_local.mutability, LocalMutability::Mutable);
        assert_eq!(let_local.initialized_at, Some(let_stmt));
        assert_eq!(
            flow.source_map.local_sources[&LocalId::new(2)],
            let_binder.source
        );
        let runtime_pick_local = flow
            .locals
            .get(LocalId::new(3))
            .expect("runtime pick local");
        assert_eq!(
            runtime_pick_local.kind,
            LocalKind::Pick {
                witness_ty: Some(condition)
            }
        );
        assert_eq!(
            runtime_pick_local.declaration,
            LocalDeclaration::PickRuntime
        );
        assert_eq!(runtime_pick_local.mutability, LocalMutability::Immutable);
        assert_eq!(runtime_pick_local.initialized_at, Some(runtime_pick));
        assert!(!runtime_pick_local.ghost);
        assert_eq!(
            flow.source_map.local_sources[&LocalId::new(3)],
            runtime_pick_binder.source
        );
        let ghost_local = flow.locals.get(LocalId::new(4)).expect("ghost pick");
        assert_eq!(
            ghost_local.kind,
            LocalKind::Pick {
                witness_ty: Some(condition)
            }
        );
        assert_eq!(ghost_local.declaration, LocalDeclaration::PickGhost);
        assert_eq!(ghost_local.mutability, LocalMutability::Immutable);
        assert_eq!(ghost_local.initialized_at, Some(ghost_pick));
        assert!(ghost_local.ghost);
        assert_eq!(
            flow.source_map.local_sources[&LocalId::new(4)],
            ghost_pick_binder.source
        );
        let ghost_let_local = flow.locals.get(LocalId::new(5)).expect("ghost let");
        assert_eq!(ghost_let_local.kind, LocalKind::Let);
        assert_eq!(ghost_let_local.declaration, LocalDeclaration::GhostConst);
        assert_eq!(ghost_let_local.mutability, LocalMutability::Immutable);
        assert_eq!(ghost_let_local.initialized_at, Some(ghost_let));
        assert!(ghost_let_local.ghost);
        assert_eq!(
            flow.source_map.local_sources[&LocalId::new(5)],
            ghost_let_binder.source
        );
        let unsupported_local = flow.locals.get(LocalId::new(6)).expect("unsupported");
        assert!(matches!(
            &unsupported_local.declaration,
            LocalDeclaration::Unsupported(role) if role.as_str() == "checker:future-local"
        ));
        assert_eq!(unsupported_local.mutability, LocalMutability::Unknown);
        assert_eq!(
            flow.source_map.local_sources[&LocalId::new(6)],
            unsupported_binder.source
        );
        assert_eq!(flow.diagnostics.len(), 1);
        assert!(matches!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(0))
                .expect("diagnostic")
                .kind,
            ControlFlowDiagnosticKind::UnsupportedLocalDeclaration { local, .. }
                if local == LocalId::new(6)
        ));
        let entry_block = flow.blocks.get(flow.entry).expect("entry block");
        assert_eq!(
            entry_block.source,
            core.algorithms()
                .get(flow.algorithm)
                .expect("algorithm")
                .source
        );
        assert_eq!(
            flow.source_map.block_sources[&flow.entry],
            entry_block.source
        );
        assert_eq!(
            entry_block.statements,
            vec![
                let_stmt,
                runtime_pick,
                ghost_pick,
                ghost_let,
                unsupported,
                assign,
                assert_stmt
            ]
        );
        let ControlFlowTerminator::Return(Some(return_value)) = entry_block.terminator else {
            panic!("expected return terminator");
        };
        assert_eq!(return_value, term_y);
        assert!(matches!(
            flow.source_map.statement_placements[&let_stmt],
            ControlFlowStatementPlacement::LocalBinding {
                local,
                block
            } if local == LocalId::new(2) && block == BasicBlockId::new(0)
        ));
        assert!(matches!(
            flow.source_map.statement_placements[&assign],
            ControlFlowStatementPlacement::Block {
                block
            } if block == BasicBlockId::new(0)
        ));
        assert!(matches!(
            flow.source_map.statement_placements[&assert_stmt],
            ControlFlowStatementPlacement::Checkpoint {
                block
            } if block == BasicBlockId::new(0)
        ));
        assert!(matches!(
            flow.source_map.statement_placements[&return_stmt],
            ControlFlowStatementPlacement::Terminator {
                block
            } if block == BasicBlockId::new(0)
        ));
        let entry_context = flow
            .contexts
            .get(flow.blocks.get(flow.entry).expect("entry").context_in)
            .expect("entry context");
        assert_eq!(entry_context.definitely_initialized, vec![LocalId::new(0)]);
        let return_context = flow
            .contexts
            .get(entry_block.context_out[0])
            .expect("return context");
        assert_eq!(
            return_context.definitely_initialized,
            vec![
                LocalId::new(0),
                LocalId::new(2),
                LocalId::new(3),
                LocalId::new(4),
                LocalId::new(5)
            ]
        );
        assert_eq!(
            return_context.maybe_assigned,
            vec![CorePlace::new("result")]
        );
        assert_eq!(
            return_context.ghost_visible,
            vec![LocalId::new(4), LocalId::new(5)]
        );
        assert_eq!(return_context.assignment_effects.len(), 5);
        assert_eq!(
            flow.ghost_effects.runtime_pick_locals,
            vec![LocalId::new(3)]
        );
        assert_eq!(flow.ghost_effects.ghost_pick_locals, vec![LocalId::new(4)]);
        assert_eq!(
            flow.ghost_effects.runtime_assignment_effects,
            vec![
                AssignmentEffectId::new(0),
                AssignmentEffectId::new(1),
                AssignmentEffectId::new(4)
            ]
        );
        assert_eq!(
            flow.ghost_effects.ghost_assignment_effects,
            vec![AssignmentEffectId::new(2), AssignmentEffectId::new(3)]
        );
        assert!(flow.call_sites.is_empty());
        assert!(flow.contracts.calls.is_empty());
    }

    #[test]
    fn control_flow_attaches_contracts_assertions_and_algorithm_termination() {
        let mut fixture = CoreFixture::new();
        let term = fixture.term_var(0, 10);
        let requires = fixture.formula(CoreFormulaKind::True, 11);
        let ensures = fixture.formula(CoreFormulaKind::False, 12);
        let assertion = fixture.formula(
            CoreFormulaKind::Equals {
                left: term,
                right: term,
            },
            13,
        );
        let decreasing = fixture.term_var(1, 14);
        let contract_assertion = fixture.formula(CoreFormulaKind::True, 15);
        let contract_invariant = fixture.formula(CoreFormulaKind::False, 16);
        let assert_stmt = fixture.stmt(CoreAlgorithmStmtKind::Assert { formula: assertion }, 20);
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(Some(term)), 21);
        let core = fixture.finish_with_contracts(
            Vec::new(),
            None,
            vec![assert_stmt, return_stmt],
            CoreContractSet {
                requires: vec![requires],
                ensures: vec![ensures],
                invariants: vec![contract_invariant],
                assertions: vec![contract_assertion],
                decreasing: vec![decreasing],
            },
        );

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.contracts.requires.len(), 1);
        let require_site = &flow.contracts.requires[0];
        assert_eq!(require_site.kind, ContractSiteKind::Requires);
        assert_eq!(require_site.formula, requires);
        assert_eq!(
            require_site.source,
            core.source_map().formula_sources[&requires]
        );
        let entry_context_id = flow.blocks.get(flow.entry).expect("entry").context_in;
        assert!(matches!(
            require_site.placement,
            ContractSitePlacement::Entry { block, context }
                if block == flow.entry && context == entry_context_id
        ));
        let entry_context = flow.contexts.get(entry_context_id).expect("entry context");
        let require_fact = entry_context.available_facts[0];
        let require_fact = flow.context_facts.get(require_fact).expect("requires fact");
        assert_eq!(require_fact.formula, requires);
        assert_eq!(
            require_fact.source,
            core.source_map().formula_sources[&requires]
        );
        assert_eq!(require_fact.kind, ContextFactKind::Requirement);

        assert_eq!(flow.contracts.assertions.len(), 2);
        let contract_assertion_site = flow
            .contracts
            .assertions
            .iter()
            .find(|site| {
                site.formula == contract_assertion
                    && matches!(
                        site.placement,
                        AssertionPlacement::AlgorithmContract { block, context }
                            if block == flow.entry && context == entry_context_id
                    )
            })
            .expect("algorithm contract assertion site");
        assert_eq!(
            contract_assertion_site.source,
            core.source_map().formula_sources[&contract_assertion]
        );
        let assertion_site = flow
            .contracts
            .assertions
            .iter()
            .find(|site| {
                matches!(
                    site.placement,
                    AssertionPlacement::Statement { statement, .. } if statement == assert_stmt
                )
            })
            .expect("statement assertion site");
        assert_eq!(assertion_site.formula, assertion);
        assert_eq!(
            assertion_site.source,
            core.source_map().algorithm_sources[&assert_stmt]
        );
        assert!(matches!(
            assertion_site.placement,
            AssertionPlacement::Statement {
                block,
                successor_context,
                ..
            } if block == flow.entry
                && successor_context == flow.blocks.get(flow.entry).expect("entry").context_out[0]
                && context_has_fact(
                    flow,
                    successor_context,
                    assertion,
                    &core.source_map().formula_sources[&assertion],
                    ContextFactKind::Assertion
                )
        ));

        assert_eq!(flow.contracts.loop_invariants.len(), 1);
        let contract_invariant_site = &flow.contracts.loop_invariants[0];
        assert_eq!(contract_invariant_site.formula, contract_invariant);
        assert_eq!(
            contract_invariant_site.source,
            core.source_map().formula_sources[&contract_invariant]
        );
        assert!(matches!(
            contract_invariant_site.placement,
            LoopInvariantPlacement::AlgorithmContract { block, context }
                if block == flow.entry && context == entry_context_id
        ));

        assert_eq!(flow.contracts.ensures.len(), 1);
        let ensure_site = &flow.contracts.ensures[0];
        assert_eq!(ensure_site.kind, ContractSiteKind::Ensures);
        assert_eq!(ensure_site.formula, ensures);
        assert_eq!(
            ensure_site.source,
            core.source_map().formula_sources[&ensures]
        );
        assert!(matches!(
            ensure_site.placement,
            ContractSitePlacement::Return { block, exit }
                if block == flow.entry
                    && matches!(
                        flow.exits.get(exit).expect("return exit").statement,
                        Some(statement) if statement == return_stmt
                    )
        ));

        assert_eq!(flow.contracts.decreasing.len(), 1);
        let measure = &flow.contracts.decreasing[0];
        assert_eq!(measure.term, decreasing);
        assert_eq!(measure.source, core.source_map().term_sources[&decreasing]);
        assert!(matches!(
            measure.placement,
            TerminationMeasurePlacement::AlgorithmHeader { block } if block == flow.entry
        ));
        assert!(flow.termination.partial_sites.is_empty());
    }

    #[test]
    fn control_flow_attaches_implicit_ensures_and_partial_algorithm_termination() {
        let mut fixture = CoreFixture::new();
        let term = fixture.term_var(0, 10);
        let ensures = fixture.formula(CoreFormulaKind::True, 11);
        let assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("result"),
                value: term,
            },
            20,
        );
        let core = fixture.finish_with_contracts(
            Vec::new(),
            None,
            vec![assign],
            CoreContractSet {
                ensures: vec![ensures],
                ..CoreContractSet::default()
            },
        );

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.contracts.ensures.len(), 1);
        let implicit_exit = flow
            .exits
            .iter()
            .find(|(_, exit)| exit.statement.is_none())
            .map(|(id, _)| id)
            .expect("implicit return exit");
        assert!(matches!(
            flow.contracts.ensures[0].placement,
            ContractSitePlacement::Return { exit, .. } if exit == implicit_exit
        ));
        assert!(flow.termination.partial_sites.iter().any(|site| {
            matches!(site.kind, TerminationSiteKind::Algorithm)
                && site.source
                    == core
                        .algorithms()
                        .get(flow.algorithm)
                        .expect("algorithm")
                        .source
        }));
        assert!(flow.call_sites.is_empty());
        assert!(flow.contracts.calls.is_empty());
    }

    #[test]
    fn control_flow_lowers_if_with_and_without_else() {
        let mut fixture = CoreFixture::new();
        let term = fixture.term_var(0, 10);
        let condition = fixture.formula(CoreFormulaKind::True, 11);
        let assign_then = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("then"),
                value: term,
            },
            20,
        );
        let assert_else = fixture.stmt(CoreAlgorithmStmtKind::Assert { formula: condition }, 21);
        let first_if = fixture.stmt(
            CoreAlgorithmStmtKind::If {
                condition,
                then_body: vec![assign_then],
                else_body: vec![assert_else],
            },
            22,
        );
        let assign_absent_else = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("absent_else"),
                value: term,
            },
            23,
        );
        let second_if = fixture.stmt(
            CoreAlgorithmStmtKind::If {
                condition,
                then_body: vec![assign_absent_else],
                else_body: Vec::new(),
            },
            24,
        );
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(None), 25);
        let core = fixture.finish(Vec::new(), None, vec![first_if, second_if, return_stmt]);

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.blocks.len(), 7);
        let branch_count = flow
            .blocks
            .iter()
            .filter(|(_, block)| matches!(block.terminator, ControlFlowTerminator::Branch { .. }))
            .count();
        assert_eq!(branch_count, 2);
        let first_branch = flow.blocks.get(BasicBlockId::new(0)).expect("first block");
        let ControlFlowTerminator::Branch {
            condition: actual_condition,
            then_block,
            else_block,
        } = first_branch.terminator
        else {
            panic!("expected branch");
        };
        assert_eq!(actual_condition, condition);
        assert_eq!(then_block, BasicBlockId::new(1));
        assert_eq!(else_block, BasicBlockId::new(2));
        assert_eq!(first_branch.context_out.len(), 2);
        let then_context = flow
            .contexts
            .get(first_branch.context_out[0])
            .expect("then context");
        assert_eq!(then_context.path_conditions, vec![condition]);
        let else_context = flow
            .contexts
            .get(first_branch.context_out[1])
            .expect("else context");
        assert!(else_context.path_conditions.is_empty());
        assert_eq!(
            flow.blocks.get(then_block).expect("then").statements,
            vec![assign_then]
        );
        assert_eq!(
            flow.blocks.get(else_block).expect("else").statements,
            vec![assert_else]
        );
        assert!(matches!(
            flow.blocks.get(then_block).expect("then").terminator,
            ControlFlowTerminator::Goto(target) if target == BasicBlockId::new(3)
        ));
        assert!(matches!(
            flow.blocks.get(else_block).expect("else").terminator,
            ControlFlowTerminator::Goto(target) if target == BasicBlockId::new(3)
        ));
        let first_join = flow.blocks.get(BasicBlockId::new(3)).expect("first join");
        let ControlFlowTerminator::Branch {
            then_block,
            else_block,
            ..
        } = first_join.terminator
        else {
            panic!("expected second branch at first join");
        };
        assert_eq!(then_block, BasicBlockId::new(4));
        assert_eq!(else_block, BasicBlockId::new(5));
        assert_eq!(
            flow.blocks
                .get(BasicBlockId::new(4))
                .expect("second then")
                .statements,
            vec![assign_absent_else]
        );
        assert!(
            flow.blocks
                .get(BasicBlockId::new(5))
                .expect("absent else")
                .statements
                .is_empty()
        );
        assert!(matches!(
            flow.blocks
                .get(BasicBlockId::new(5))
                .expect("absent else")
                .terminator,
            ControlFlowTerminator::Goto(target) if target == BasicBlockId::new(6)
        ));
        assert!(matches!(
            flow.source_map.statement_placements[&second_if],
            ControlFlowStatementPlacement::Terminator { .. }
        ));
        assert!(matches!(
            flow.blocks
                .get(BasicBlockId::new(6))
                .expect("second join")
                .terminator,
            ControlFlowTerminator::Return(None)
        ));
    }

    #[test]
    fn control_flow_lowers_while_break_continue_without_hidden_locals() {
        let mut fixture = CoreFixture::new();
        let condition = fixture.formula(CoreFormulaKind::True, 10);
        let continue_stmt = fixture.stmt(CoreAlgorithmStmtKind::Continue, 20);
        let break_stmt = fixture.stmt(CoreAlgorithmStmtKind::Break, 21);
        let branch = fixture.stmt(
            CoreAlgorithmStmtKind::If {
                condition,
                then_body: vec![continue_stmt],
                else_body: vec![break_stmt],
            },
            22,
        );
        let while_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition,
                invariants: vec![condition],
                decreasing: Vec::new(),
                body: vec![branch],
            },
            23,
        );
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(None), 24);
        let core = fixture.finish(Vec::new(), None, vec![while_stmt, return_stmt]);

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.blocks.len(), 7);
        assert_eq!(flow.loops.len(), 1);
        let loop_record = flow.loops.get(LoopId::new(0)).expect("loop");
        assert_eq!(loop_record.header, BasicBlockId::new(1));
        assert_eq!(loop_record.body, BasicBlockId::new(2));
        assert_eq!(loop_record.exit, BasicBlockId::new(3));
        assert_eq!(
            flow.source_map.block_sources[&loop_record.header],
            flow.blocks.get(loop_record.header).expect("header").source
        );
        assert_eq!(
            flow.source_map.block_sources[&loop_record.body],
            flow.blocks.get(loop_record.body).expect("body").source
        );
        assert_eq!(
            flow.source_map.block_sources[&loop_record.exit],
            flow.blocks.get(loop_record.exit).expect("exit").source
        );
        assert_eq!(loop_record.condition, condition);
        assert_eq!(loop_record.invariants, vec![condition]);
        assert!(
            flow.locals
                .iter()
                .all(|(_, local)| local.kind != LocalKind::HiddenLoopValue)
        );
        assert!(matches!(
            flow.source_map.statement_placements[&while_stmt],
            ControlFlowStatementPlacement::LoopHeader {
                loop_id,
                header
            } if loop_id == LoopId::new(0) && header == BasicBlockId::new(1)
        ));
        assert!(matches!(
            flow.blocks
                .get(BasicBlockId::new(0))
                .expect("preheader")
                .terminator,
            ControlFlowTerminator::Goto(target) if target == BasicBlockId::new(1)
        ));
        let header = flow.blocks.get(loop_record.header).expect("header");
        let ControlFlowTerminator::Branch {
            condition: header_condition,
            then_block,
            else_block,
        } = header.terminator
        else {
            panic!("expected loop header branch");
        };
        assert_eq!(header_condition, condition);
        assert_eq!(then_block, loop_record.body);
        assert_eq!(else_block, loop_record.exit);
        let body_context = flow
            .contexts
            .get(header.context_out[0])
            .expect("body context");
        assert_eq!(body_context.path_conditions, vec![condition]);
        assert_eq!(body_context.active_invariants, vec![condition]);
        assert_eq!(body_context.loop_stack, vec![LoopId::new(0)]);
        let invariant_source = core.source_map().formula_sources[&condition].clone();
        assert!(context_has_fact(
            flow,
            header.context_in,
            condition,
            &invariant_source,
            ContextFactKind::LoopInvariant
        ));
        assert!(context_has_fact(
            flow,
            header.context_out[0],
            condition,
            &invariant_source,
            ContextFactKind::LoopInvariant
        ));
        assert!(context_has_fact(
            flow,
            header.context_out[1],
            condition,
            &invariant_source,
            ContextFactKind::LoopInvariant
        ));
        let body_block = flow.blocks.get(loop_record.body).expect("body block");
        let ControlFlowTerminator::Branch {
            then_block,
            else_block,
            ..
        } = body_block.terminator
        else {
            panic!("expected body if branch");
        };
        assert_eq!(then_block, BasicBlockId::new(4));
        assert_eq!(else_block, BasicBlockId::new(5));
        assert!(matches!(
            flow.blocks
                .get(BasicBlockId::new(4))
                .expect("continue block")
                .terminator,
            ControlFlowTerminator::Continue { loop_id, target }
                if loop_id == LoopId::new(0) && target == BasicBlockId::new(1)
        ));
        assert!(matches!(
            flow.blocks
                .get(BasicBlockId::new(5))
                .expect("break block")
                .terminator,
            ControlFlowTerminator::Break { loop_id, target }
                if loop_id == LoopId::new(0) && target == BasicBlockId::new(3)
        ));
        let unreachable_join = flow
            .blocks
            .get(BasicBlockId::new(6))
            .expect("unreachable body join");
        assert_eq!(unreachable_join.reachable, Reachability::Unreachable);
        assert!(matches!(
            unreachable_join.terminator,
            ControlFlowTerminator::Goto(target) if target == BasicBlockId::new(1)
        ));
        let (continue_exit_id, continue_exit) = flow
            .exits
            .iter()
            .find(|(_, exit)| exit.statement == Some(continue_stmt))
            .expect("continue exit");
        assert_eq!(continue_exit.target, Some(loop_record.header));
        assert!(matches!(
            continue_exit.kind,
            ControlFlowExitKind::Continue {
                loop_id
            } if loop_id == LoopId::new(0)
        ));
        let (break_exit_id, break_exit) = flow
            .exits
            .iter()
            .find(|(_, exit)| exit.statement == Some(break_stmt))
            .expect("break exit");
        assert_eq!(break_exit.target, Some(loop_record.exit));
        assert!(context_has_fact(
            flow,
            flow.blocks
                .get(continue_exit.from)
                .expect("continue block")
                .context_out[0],
            condition,
            &invariant_source,
            ContextFactKind::LoopInvariant
        ));
        assert!(context_has_fact(
            flow,
            flow.blocks
                .get(break_exit.from)
                .expect("break block")
                .context_out[0],
            condition,
            &invariant_source,
            ContextFactKind::LoopInvariant
        ));
        assert!(context_has_fact(
            flow,
            flow.blocks
                .get(loop_record.exit)
                .expect("loop exit")
                .context_in,
            condition,
            &invariant_source,
            ContextFactKind::LoopInvariant
        ));
        assert!(flow.termination.partial_sites.iter().any(|site| {
            matches!(site.kind, TerminationSiteKind::Loop(loop_id) if loop_id == LoopId::new(0))
                && site.source == loop_record.source
        }));
        assert!(flow.contracts.loop_invariants.iter().any(|site| {
            site.formula == condition
                && matches!(
                    site.placement,
                    LoopInvariantPlacement::Header { loop_id, block }
                        if loop_id == LoopId::new(0) && block == loop_record.header
                )
        }));
        assert!(flow.contracts.loop_invariants.iter().any(|site| {
            site.formula == condition
                && matches!(
                    site.placement,
                    LoopInvariantPlacement::BreakExit { loop_id, exit }
                        if loop_id == LoopId::new(0) && exit == break_exit_id
                )
        }));
        assert!(flow.contracts.loop_invariants.iter().any(|site| {
            site.formula == condition
                && matches!(
                    site.placement,
                    LoopInvariantPlacement::ContinueExit { loop_id, exit }
                        if loop_id == LoopId::new(0) && exit == continue_exit_id
                )
        }));
    }

    #[test]
    fn control_flow_attaches_loop_invariants_and_decreasing_sites() {
        let mut fixture = CoreFixture::new();
        let term = fixture.term_var(0, 10);
        let first_decreasing = fixture.term_var(1, 11);
        let second_decreasing = fixture.term_var(2, 12);
        let condition = fixture.formula(CoreFormulaKind::True, 13);
        let first_invariant = fixture.formula(CoreFormulaKind::False, 14);
        let second_invariant = fixture.formula(CoreFormulaKind::True, 15);
        let normal_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("normal_loop"),
                value: term,
            },
            20,
        );
        let normal_loop = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition,
                invariants: vec![first_invariant],
                decreasing: vec![first_decreasing],
                body: vec![normal_assign],
            },
            21,
        );
        let continue_stmt = fixture.stmt(CoreAlgorithmStmtKind::Continue, 22);
        let continue_loop = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition,
                invariants: vec![second_invariant],
                decreasing: vec![second_decreasing],
                body: vec![continue_stmt],
            },
            23,
        );
        let core = fixture.finish(Vec::new(), None, vec![normal_loop, continue_loop]);

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.loops.len(), 2);
        let first_loop = flow.loops.get(LoopId::new(0)).expect("first loop");
        let second_loop = flow.loops.get(LoopId::new(1)).expect("second loop");
        let (continue_exit_id, continue_exit) = flow
            .exits
            .iter()
            .find(|(_, exit)| exit.statement == Some(continue_stmt))
            .expect("continue exit");
        assert_eq!(continue_exit.from, second_loop.body);
        assert!(context_has_fact(
            flow,
            flow.blocks
                .get(first_loop.header)
                .expect("first header")
                .context_in,
            first_invariant,
            &core.source_map().formula_sources[&first_invariant],
            ContextFactKind::LoopInvariant
        ));
        assert!(context_has_fact(
            flow,
            flow.blocks
                .get(first_loop.exit)
                .expect("first loop exit")
                .context_in,
            first_invariant,
            &core.source_map().formula_sources[&first_invariant],
            ContextFactKind::LoopInvariant
        ));
        assert!(context_has_fact(
            flow,
            flow.blocks
                .get(continue_exit.from)
                .expect("continue block")
                .context_out[0],
            second_invariant,
            &core.source_map().formula_sources[&second_invariant],
            ContextFactKind::LoopInvariant
        ));
        assert!(flow.contracts.loop_invariants.iter().any(|site| {
            site.formula == first_invariant
                && site.source == core.source_map().formula_sources[&first_invariant]
                && matches!(
                    site.placement,
                    LoopInvariantPlacement::NormalBackedge { loop_id, from, to }
                        if loop_id == LoopId::new(0)
                            && from == first_loop.body
                            && to == first_loop.header
                )
        }));
        assert!(flow.contracts.loop_invariants.iter().any(|site| {
            site.formula == second_invariant
                && site.source == core.source_map().formula_sources[&second_invariant]
                && matches!(
                    site.placement,
                    LoopInvariantPlacement::ContinueExit { loop_id, exit }
                        if loop_id == LoopId::new(1) && exit == continue_exit_id
                )
        }));
        assert!(flow.contracts.decreasing.iter().any(|site| {
            site.term == first_decreasing
                && site.source == core.source_map().term_sources[&first_decreasing]
                && matches!(
                    site.placement,
                    TerminationMeasurePlacement::LoopHeader {
                        loop_id,
                        header
                    } if loop_id == LoopId::new(0) && header == first_loop.header
                )
        }));
        assert!(flow.contracts.decreasing.iter().any(|site| {
            site.term == second_decreasing
                && site.source == core.source_map().term_sources[&second_decreasing]
                && matches!(
                    site.placement,
                    TerminationMeasurePlacement::ContinueEdge {
                        loop_id,
                        exit
                    } if loop_id == LoopId::new(1) && exit == continue_exit_id
                )
        }));
        assert!(
            flow.termination
                .partial_sites
                .iter()
                .any(|site| { matches!(site.kind, TerminationSiteKind::Algorithm) })
        );
    }

    #[test]
    fn control_flow_closes_fallthrough_and_joins_break_contexts() {
        let mut fixture = CoreFixture::new();
        let term = fixture.term_var(0, 10);
        let condition = fixture.formula(CoreFormulaKind::True, 11);
        let inside_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("inside_loop"),
                value: term,
            },
            20,
        );
        let break_stmt = fixture.stmt(CoreAlgorithmStmtKind::Break, 21);
        let while_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition,
                invariants: Vec::new(),
                decreasing: Vec::new(),
                body: vec![inside_assign, break_stmt],
            },
            22,
        );
        let after_loop_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("after_loop"),
                value: term,
            },
            23,
        );
        let core = fixture.finish(Vec::new(), None, vec![while_stmt, after_loop_assign]);

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        let loop_record = flow.loops.get(LoopId::new(0)).expect("loop");
        let loop_exit_context = flow
            .contexts
            .get(
                flow.blocks
                    .get(loop_record.exit)
                    .expect("loop exit")
                    .context_in,
            )
            .expect("loop exit context");
        assert_eq!(
            loop_exit_context.maybe_assigned,
            vec![CorePlace::new("inside_loop")]
        );

        let exit_block = flow.blocks.get(loop_record.exit).expect("exit block");
        assert_eq!(exit_block.statements, vec![after_loop_assign]);
        assert!(matches!(
            exit_block.terminator,
            ControlFlowTerminator::Return(None)
        ));
        let final_context = flow
            .contexts
            .get(exit_block.context_out[0])
            .expect("fallthrough context");
        assert_eq!(
            final_context.maybe_assigned,
            vec![CorePlace::new("after_loop"), CorePlace::new("inside_loop")]
        );
        let implicit_return = flow
            .exits
            .iter()
            .find(|(_, exit)| exit.statement.is_none())
            .map(|(_, exit)| exit)
            .expect("implicit fallthrough return");
        assert_eq!(implicit_return.from, loop_record.exit);
        assert!(matches!(implicit_return.kind, ControlFlowExitKind::Return));
    }

    #[test]
    fn control_flow_joins_loop_carried_contexts_and_skips_unreachable_joins() {
        let mut fixture = CoreFixture::new();
        let term = fixture.term_var(0, 10);
        let condition = fixture.formula(CoreFormulaKind::True, 11);
        let loop_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("loop_carried"),
                value: term,
            },
            20,
        );
        let while_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition,
                invariants: Vec::new(),
                decreasing: Vec::new(),
                body: vec![loop_assign],
            },
            21,
        );
        let unreachable_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("unreachable"),
                value: term,
            },
            22,
        );
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(None), 23);
        let branch = fixture.stmt(
            CoreAlgorithmStmtKind::If {
                condition,
                then_body: vec![return_stmt, unreachable_assign],
                else_body: Vec::new(),
            },
            24,
        );
        let after_branch_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("after_branch"),
                value: term,
            },
            25,
        );
        let core = fixture.finish(
            Vec::new(),
            None,
            vec![while_stmt, branch, after_branch_assign],
        );

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        let loop_record = flow.loops.get(LoopId::new(0)).expect("loop");
        let loop_exit_context = flow
            .contexts
            .get(
                flow.blocks
                    .get(loop_record.exit)
                    .expect("loop exit")
                    .context_in,
            )
            .expect("loop exit context");
        assert_eq!(
            loop_exit_context.maybe_assigned,
            vec![CorePlace::new("loop_carried")]
        );

        let implicit_return = flow
            .exits
            .iter()
            .find(|(_, exit)| exit.statement.is_none())
            .map(|(_, exit)| exit)
            .expect("implicit fallthrough return");
        let final_context = flow
            .contexts
            .get(
                flow.blocks
                    .get(implicit_return.from)
                    .expect("implicit return block")
                    .context_out[0],
            )
            .expect("final context");
        assert_eq!(
            final_context.maybe_assigned,
            vec![
                CorePlace::new("after_branch"),
                CorePlace::new("loop_carried")
            ]
        );
        assert!(
            !final_context
                .maybe_assigned
                .contains(&CorePlace::new("unreachable"))
        );
    }

    #[test]
    fn control_flow_lowers_match_arms_in_source_order() {
        let mut fixture = CoreFixture::new();
        let term = fixture.term_var(0, 10);
        let first_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("first"),
                value: term,
            },
            20,
        );
        let second_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("second"),
                value: term,
            },
            21,
        );
        let match_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::Match {
                scrutinee: term,
                arms: vec![
                    CoreAlgorithmMatchArm {
                        pattern: CoreProvenanceKey::new("case:first"),
                        body: vec![first_assign],
                    },
                    CoreAlgorithmMatchArm {
                        pattern: CoreProvenanceKey::new("case:second"),
                        body: vec![second_assign],
                    },
                ],
            },
            22,
        );
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(None), 23);
        let core = fixture.finish(Vec::new(), None, vec![match_stmt, return_stmt]);

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.blocks.len(), 4);
        let (switch_block, arms, join) = flow
            .blocks
            .iter()
            .find_map(|(id, block)| match &block.terminator {
                ControlFlowTerminator::Switch { arms, join, .. } => Some((id, arms, join)),
                _ => None,
            })
            .expect("switch");
        assert_eq!(switch_block, BasicBlockId::new(0));
        assert_eq!(
            arms.iter().map(|arm| &arm.pattern).collect::<Vec<_>>(),
            vec![
                &CoreProvenanceKey::new("case:first"),
                &CoreProvenanceKey::new("case:second")
            ]
        );
        assert_eq!(
            arms.iter().map(|arm| arm.block).collect::<Vec<_>>(),
            vec![BasicBlockId::new(1), BasicBlockId::new(2)]
        );
        assert_eq!(*join, Some(BasicBlockId::new(3)));
        assert_eq!(
            flow.blocks
                .get(BasicBlockId::new(1))
                .expect("first arm")
                .statements,
            vec![first_assign]
        );
        assert_eq!(
            flow.blocks
                .get(BasicBlockId::new(2))
                .expect("second arm")
                .statements,
            vec![second_assign]
        );
        assert!(matches!(
            flow.blocks
                .get(BasicBlockId::new(1))
                .expect("first arm")
                .terminator,
            ControlFlowTerminator::Goto(target) if target == BasicBlockId::new(3)
        ));
        assert!(matches!(
            flow.blocks
                .get(BasicBlockId::new(3))
                .expect("join")
                .terminator,
            ControlFlowTerminator::Return(None)
        ));
        assert!(flow.diagnostics.is_empty());
    }

    #[test]
    fn control_flow_records_use_before_assignment_diagnostics() {
        let mut fixture = CoreFixture::new();
        let uninitialized_use = fixture.term_var(2, 31);
        let self_use = fixture.term_var(3, 41);
        let shadowed_use = fixture.term_var(2, 51);
        let shadow_guard = fixture.formula(
            CoreFormulaKind::Equals {
                left: shadowed_use,
                right: shadowed_use,
            },
            52,
        );
        let mut shadowing_binder = fixture.binder(2, "local:const", 53);
        shadowing_binder.ty_guard = Some(shadow_guard);
        let quantified = fixture.formula(
            CoreFormulaKind::Forall {
                binders: vec![shadowing_binder],
                body: shadow_guard,
            },
            54,
        );
        let uninitialized_binder = fixture.binder(2, "local:var", 20);
        let declare_uninitialized = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: uninitialized_binder,
                value: None,
                ghost: false,
            },
            21,
        );
        let use_uninitialized = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("result"),
                value: uninitialized_use,
            },
            32,
        );
        let self_binder = fixture.binder(3, "local:const", 40);
        let self_initializing_let = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: self_binder,
                value: Some(self_use),
                ghost: false,
            },
            42,
        );
        let shadowed_assert = fixture.stmt(
            CoreAlgorithmStmtKind::Assert {
                formula: quantified,
            },
            55,
        );
        let core = fixture.finish(
            Vec::new(),
            None,
            vec![
                declare_uninitialized,
                use_uninitialized,
                self_initializing_let,
                shadowed_assert,
            ],
        );

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.diagnostics.len(), 2);
        let first = flow
            .diagnostics
            .get(ControlFlowDiagnosticId::new(0))
            .expect("first use-before");
        assert!(matches!(
            first.kind,
            ControlFlowDiagnosticKind::UseBeforeAssignment { local, var }
                if local == LocalId::new(0) && var == CoreVarId::new(2)
        ));
        assert_eq!(first.statement, Some(use_uninitialized));
        assert_eq!(
            first.source,
            core.source_map().term_sources[&uninitialized_use]
        );
        let second = flow
            .diagnostics
            .get(ControlFlowDiagnosticId::new(1))
            .expect("self use-before");
        assert!(matches!(
            second.kind,
            ControlFlowDiagnosticKind::UseBeforeAssignment { local, var }
                if local == LocalId::new(1) && var == CoreVarId::new(3)
        ));
        assert_eq!(second.statement, Some(self_initializing_let));
        assert_eq!(second.source, core.source_map().term_sources[&self_use]);
        assert!(
            flow.diagnostics
                .iter()
                .all(|(_, diagnostic)| diagnostic.statement != Some(shadowed_assert))
        );
        for (diagnostic_id, diagnostic) in flow.diagnostics.iter() {
            assert_eq!(
                flow.source_map.diagnostic_sources[&diagnostic_id],
                diagnostic.source
            );
            assert!(diagnostic.carried_core_diagnostic.is_none());
        }
    }

    #[test]
    fn control_flow_checks_use_before_at_all_statement_owned_sites() {
        let mut fixture = CoreFixture::new();
        let uninitialized_binder = fixture.binder(0, "local:var", 10);
        let declare_uninitialized = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: uninitialized_binder,
                value: None,
                ghost: false,
            },
            11,
        );

        let tuple_late = fixture.term_var(0, 31);
        let tuple_early = fixture.term_var(0, 30);
        let tuple = fixture.term(CoreTermKind::Tuple(vec![tuple_late, tuple_early]), 32);
        let assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("result"),
                value: tuple,
            },
            33,
        );

        let let_value = fixture.term_var(0, 40);
        let let_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: fixture.binder(1, "local:const", 39),
                value: Some(let_value),
                ghost: false,
            },
            41,
        );

        let pick_use = fixture.term_var(0, 42);
        let pick_witness = fixture.formula(
            CoreFormulaKind::Equals {
                left: pick_use,
                right: pick_use,
            },
            43,
        );
        let pick = fixture.stmt(
            CoreAlgorithmStmtKind::Pick {
                binder: fixture.binder(2, "local:const", 44),
                witness_ty: Some(pick_witness),
                ghost: false,
            },
            45,
        );

        let if_use = fixture.term_var(0, 46);
        let if_condition = fixture.formula(
            CoreFormulaKind::Equals {
                left: if_use,
                right: if_use,
            },
            47,
        );
        let if_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::If {
                condition: if_condition,
                then_body: Vec::new(),
                else_body: Vec::new(),
            },
            48,
        );

        let while_condition_use = fixture.term_var(0, 49);
        let while_condition = fixture.formula(
            CoreFormulaKind::Equals {
                left: while_condition_use,
                right: while_condition_use,
            },
            50,
        );
        let while_invariant_use = fixture.term_var(0, 51);
        let while_invariant = fixture.formula(
            CoreFormulaKind::Equals {
                left: while_invariant_use,
                right: while_invariant_use,
            },
            52,
        );
        let while_decreasing = fixture.term_var(0, 53);
        let while_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::While {
                condition: while_condition,
                invariants: vec![while_invariant],
                decreasing: vec![while_decreasing],
                body: Vec::new(),
            },
            54,
        );

        let match_scrutinee = fixture.term_var(0, 55);
        let match_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::Match {
                scrutinee: match_scrutinee,
                arms: vec![CoreAlgorithmMatchArm {
                    pattern: CoreProvenanceKey::new("case:only"),
                    body: Vec::new(),
                }],
            },
            56,
        );

        let assert_use = fixture.term_var(0, 57);
        let assert_formula = fixture.formula(
            CoreFormulaKind::Equals {
                left: assert_use,
                right: assert_use,
            },
            58,
        );
        let assert_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::Assert {
                formula: assert_formula,
            },
            59,
        );

        let return_use = fixture.term_var(0, 60);
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(Some(return_use)), 61);
        let core = fixture.finish(
            Vec::new(),
            None,
            vec![
                declare_uninitialized,
                assign,
                let_stmt,
                pick,
                if_stmt,
                while_stmt,
                match_stmt,
                assert_stmt,
                return_stmt,
            ],
        );

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        let use_before = flow
            .diagnostics
            .iter()
            .map(|(_, diagnostic)| diagnostic)
            .collect::<Vec<_>>();
        assert_eq!(use_before.len(), 11);
        assert!(use_before.iter().all(|diagnostic| {
            matches!(
                diagnostic.kind,
                ControlFlowDiagnosticKind::UseBeforeAssignment { local, var }
                    if local == LocalId::new(0) && var == CoreVarId::new(0)
            )
        }));
        assert!(use_before.iter().all(|diagnostic| {
            !matches!(diagnostic.kind, ControlFlowDiagnosticKind::FlowDiagnostic)
                && diagnostic.carried_core_diagnostic.is_none()
        }));
        assert_eq!(
            use_before
                .iter()
                .map(|diagnostic| diagnostic.statement)
                .collect::<Vec<_>>(),
            vec![
                Some(assign),
                Some(assign),
                Some(let_stmt),
                Some(pick),
                Some(if_stmt),
                Some(while_stmt),
                Some(while_stmt),
                Some(while_stmt),
                Some(match_stmt),
                Some(assert_stmt),
                Some(return_stmt),
            ]
        );
        assert_eq!(
            use_before
                .iter()
                .map(|diagnostic| diagnostic.source.clone())
                .collect::<Vec<_>>(),
            vec![
                core.source_map().term_sources[&tuple_early].clone(),
                core.source_map().term_sources[&tuple_late].clone(),
                core.source_map().term_sources[&let_value].clone(),
                core.source_map().term_sources[&pick_use].clone(),
                core.source_map().term_sources[&if_use].clone(),
                core.source_map().term_sources[&while_condition_use].clone(),
                core.source_map().term_sources[&while_invariant_use].clone(),
                core.source_map().term_sources[&while_decreasing].clone(),
                core.source_map().term_sources[&match_scrutinee].clone(),
                core.source_map().term_sources[&assert_use].clone(),
                core.source_map().term_sources[&return_use].clone(),
            ]
        );
    }

    #[test]
    fn control_flow_quantifier_use_before_scope_is_left_to_right() {
        let mut fixture = CoreFixture::new();
        let local_binder = fixture.binder(6, "local:var", 10);
        let declare_uninitialized = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: local_binder,
                value: None,
                ghost: false,
            },
            11,
        );
        let later_binder_use = fixture.term_var(6, 20);
        let first_guard = fixture.formula(
            CoreFormulaKind::Equals {
                left: later_binder_use,
                right: later_binder_use,
            },
            21,
        );
        let earlier_binder_use = fixture.term_var(5, 22);
        let second_guard = fixture.formula(
            CoreFormulaKind::Equals {
                left: earlier_binder_use,
                right: earlier_binder_use,
            },
            23,
        );
        let body_use = fixture.term_var(6, 24);
        let body = fixture.formula(
            CoreFormulaKind::Equals {
                left: body_use,
                right: body_use,
            },
            25,
        );
        let mut first_binder = fixture.binder(5, "local:const", 26);
        first_binder.ty_guard = Some(first_guard);
        let mut second_binder = fixture.binder(6, "local:const", 27);
        second_binder.ty_guard = Some(second_guard);
        let quantified = fixture.formula(
            CoreFormulaKind::Exists {
                binders: vec![first_binder, second_binder],
                body,
            },
            28,
        );
        let assert_stmt = fixture.stmt(
            CoreAlgorithmStmtKind::Assert {
                formula: quantified,
            },
            29,
        );
        let core = fixture.finish(Vec::new(), None, vec![declare_uninitialized, assert_stmt]);

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.diagnostics.len(), 1);
        let diagnostic = flow
            .diagnostics
            .get(ControlFlowDiagnosticId::new(0))
            .expect("later binder guard diagnostic");
        assert!(matches!(
            diagnostic.kind,
            ControlFlowDiagnosticKind::UseBeforeAssignment { local, var }
                if local == LocalId::new(0) && var == CoreVarId::new(6)
        ));
        assert_eq!(diagnostic.statement, Some(assert_stmt));
        assert_eq!(
            diagnostic.source,
            core.source_map().term_sources[&later_binder_use]
        );
    }

    #[test]
    fn control_flow_records_unreachable_statement_diagnostics() {
        let mut fixture = CoreFixture::new();
        let uninitialized_use = fixture.term_var(0, 30);
        let binder = fixture.binder(0, "local:var", 20);
        let declare_uninitialized = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder,
                value: None,
                ghost: false,
            },
            21,
        );
        let return_stmt = fixture.stmt(CoreAlgorithmStmtKind::Return(None), 22);
        let unreachable_assign = fixture.stmt(
            CoreAlgorithmStmtKind::Assign {
                target: CorePlace::new("result"),
                value: uninitialized_use,
            },
            31,
        );
        let true_formula = fixture.formula(CoreFormulaKind::True, 32);
        let unreachable_assert = fixture.stmt(
            CoreAlgorithmStmtKind::Assert {
                formula: true_formula,
            },
            33,
        );
        let core = fixture.finish(
            Vec::new(),
            None,
            vec![
                declare_uninitialized,
                return_stmt,
                unreachable_assign,
                unreachable_assert,
            ],
        );

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.diagnostics.len(), 2);
        let unreachable_assign_block =
            flow.source_map.statement_placements[&unreachable_assign].block();
        let unreachable_assert_block =
            flow.source_map.statement_placements[&unreachable_assert].block();
        let first = flow
            .diagnostics
            .get(ControlFlowDiagnosticId::new(0))
            .expect("unreachable assign");
        assert!(matches!(
            first.kind,
            ControlFlowDiagnosticKind::UnreachableStatement { block }
                if block == unreachable_assign_block
        ));
        assert_eq!(first.statement, Some(unreachable_assign));
        assert_eq!(
            first.source,
            core.source_map().algorithm_sources[&unreachable_assign]
        );
        let second = flow
            .diagnostics
            .get(ControlFlowDiagnosticId::new(1))
            .expect("unreachable assert");
        assert!(matches!(
            second.kind,
            ControlFlowDiagnosticKind::UnreachableStatement { block }
                if block == unreachable_assert_block
        ));
        assert_eq!(second.statement, Some(unreachable_assert));
        assert_eq!(
            second.source,
            core.source_map().algorithm_sources[&unreachable_assert]
        );
        assert!(flow.diagnostics.iter().all(|(_, diagnostic)| {
            !matches!(
                diagnostic.kind,
                ControlFlowDiagnosticKind::UseBeforeAssignment { .. }
            )
        }));
    }

    #[test]
    fn control_flow_records_structural_error_diagnostics_in_order() {
        let mut fixture = CoreFixture::new();
        let unsupported_binder = fixture.binder(0, "checker:future-local", 18);
        let unsupported = fixture.stmt(
            CoreAlgorithmStmtKind::Let {
                binder: unsupported_binder.clone(),
                value: None,
                ghost: false,
            },
            19,
        );
        let break_stmt = fixture.stmt(CoreAlgorithmStmtKind::Break, 20);
        let continue_stmt = fixture.stmt(CoreAlgorithmStmtKind::Continue, 21);
        let error_stmt = fixture.error_stmt(22);
        let core = fixture.finish(
            Vec::new(),
            None,
            vec![unsupported, break_stmt, continue_stmt, error_stmt],
        );

        let output = build_control_flow_ir(&core);
        let flow = only_flow(&output);
        assert_eq!(flow.diagnostics.len(), 6);
        let core_error_diagnostic = match core
            .algorithm_statements()
            .get(error_stmt)
            .expect("error stmt")
            .kind
        {
            CoreAlgorithmStmtKind::Error(diagnostic) => diagnostic,
            _ => panic!("expected phase9 error statement"),
        };
        assert!(matches!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(0))
                .expect("unsupported")
                .kind,
            ControlFlowDiagnosticKind::UnsupportedLocalDeclaration {
                local,
                ..
            } if local == LocalId::new(0)
        ));
        assert!(matches!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(1))
                .expect("illegal break")
                .kind,
            ControlFlowDiagnosticKind::IllegalBreak
        ));
        assert!(matches!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(2))
                .expect("illegal continue")
                .kind,
            ControlFlowDiagnosticKind::IllegalContinue
        ));
        assert!(matches!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(3))
                .expect("unreachable continue")
                .kind,
            ControlFlowDiagnosticKind::UnreachableStatement { .. }
        ));
        assert!(matches!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(4))
                .expect("phase9")
                .kind,
            ControlFlowDiagnosticKind::Phase9Error
        ));
        let phase9 = flow
            .diagnostics
            .get(ControlFlowDiagnosticId::new(4))
            .expect("phase9");
        assert!(matches!(
            phase9.kind,
            ControlFlowDiagnosticKind::Phase9Error
        ));
        assert_eq!(phase9.statement, Some(error_stmt));
        assert_eq!(phase9.carried_core_diagnostic, Some(core_error_diagnostic));
        for (diagnostic_id, diagnostic) in flow.diagnostics.iter() {
            assert_eq!(
                flow.source_map.diagnostic_sources[&diagnostic_id],
                diagnostic.source
            );
        }
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(0))
                .expect("unsupported")
                .statement,
            Some(unsupported)
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(0))
                .expect("unsupported")
                .source,
            unsupported_binder.source
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(1))
                .expect("break")
                .statement,
            Some(break_stmt)
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(1))
                .expect("break")
                .source,
            core.source_map().algorithm_sources[&break_stmt]
        );
        assert!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(1))
                .expect("break")
                .carried_core_diagnostic
                .is_none()
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(2))
                .expect("continue")
                .statement,
            Some(continue_stmt)
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(2))
                .expect("continue")
                .source,
            core.source_map().algorithm_sources[&continue_stmt]
        );
        assert!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(2))
                .expect("continue")
                .carried_core_diagnostic
                .is_none()
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(3))
                .expect("unreachable continue")
                .statement,
            Some(continue_stmt)
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(3))
                .expect("unreachable continue")
                .source,
            core.source_map().algorithm_sources[&continue_stmt]
        );
        assert!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(3))
                .expect("unreachable continue")
                .carried_core_diagnostic
                .is_none()
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(5))
                .expect("unreachable phase9")
                .statement,
            Some(error_stmt)
        );
        assert_eq!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(5))
                .expect("unreachable phase9")
                .source,
            core.source_map().algorithm_sources[&error_stmt]
        );
        assert!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(5))
                .expect("unreachable phase9")
                .carried_core_diagnostic
                .is_none()
        );
        assert_eq!(
            phase9.source,
            core.source_map().algorithm_sources[&error_stmt]
        );
        assert!(
            flow.diagnostics
                .get(ControlFlowDiagnosticId::new(0))
                .expect("unsupported")
                .carried_core_diagnostic
                .is_none()
        );
        assert!(matches!(
            flow.source_map.statement_placements[&break_stmt],
            ControlFlowStatementPlacement::ErrorSite {
                diagnostic,
                ..
            } if diagnostic == ControlFlowDiagnosticId::new(1)
        ));
        assert_eq!(
            flow.source_map.exit_sources[&ControlFlowExitId::new(0)],
            core.source_map().algorithm_sources[&break_stmt]
        );
        assert_eq!(
            flow.source_map.exit_sources[&ControlFlowExitId::new(1)],
            core.source_map().algorithm_sources[&continue_stmt]
        );
        assert_eq!(
            flow.source_map.exit_sources[&ControlFlowExitId::new(2)],
            core.source_map().algorithm_sources[&error_stmt]
        );
        let error_terminators = flow
            .blocks
            .iter()
            .filter(|(_, block)| matches!(block.terminator, ControlFlowTerminator::Error(_)))
            .count();
        assert_eq!(error_terminators, 3);
    }
}
