//! Control-flow IR construction.
//!
//! Implements the phase-10 CFG construction contract specified in
//! [control_flow.md](../../../../doc/design/mizar-core/en/control_flow.md).

use crate::core_ir::{
    CoreAlgorithm, CoreAlgorithmId, CoreAlgorithmMatchArm, CoreAlgorithmStmt, CoreAlgorithmStmtId,
    CoreAlgorithmStmtKind, CoreBinder, CoreDiagnosticId, CoreFormulaId, CoreIr, CoreItemId,
    CorePlace, CoreProvenance, CoreProvenanceKey, CoreProvenancePhase, CoreSourceRef, CoreTermId,
    CoreVarRole, GhostEffectKey,
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
            CoreAlgorithmStmtKind::Assign { target, .. } => {
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
            } => Some(self.build_if(
                cursor,
                statement_id,
                &statement,
                *condition,
                then_body,
                else_body,
                loop_stack,
            )),
            CoreAlgorithmStmtKind::While {
                condition,
                invariants,
                decreasing,
                body,
            } => Some(self.build_while(
                cursor,
                statement_id,
                &statement,
                *condition,
                invariants,
                decreasing,
                body,
                loop_stack,
            )),
            CoreAlgorithmStmtKind::Match { scrutinee, arms } => Some(self.build_match(
                cursor,
                statement_id,
                &statement,
                *scrutinee,
                arms,
                loop_stack,
            )),
            CoreAlgorithmStmtKind::Return(value) => {
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
        if let Some(role) = unsupported_role {
            self.add_diagnostic(ControlFlowDiagnostic {
                kind: ControlFlowDiagnosticKind::UnsupportedLocalDeclaration { local, role },
                algorithm: self.algorithm_id,
                statement: Some(statement_id),
                source: binder.source.clone(),
                carried_core_diagnostic: None,
            });
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
        CoreItemKind, CoreItemTable, CoreNodeRef, CoreSourceMap, CoreTerm, CoreTermKind,
        CoreTermTable, CoreVarId, CoreVisibility, GeneratedOriginTable, ObligationSeedTable,
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

        fn term_var(&mut self, var: usize, start: usize) -> CoreTermId {
            let source = self.source(start, start + 1);
            let id = self.parts.terms.insert(CoreTerm::new(
                CoreTermKind::Var(CoreVarId::new(var)),
                source.clone(),
            ));
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

    #[test]
    fn control_flow_lowers_straight_line_locals_sources_and_debug_text() {
        let mut fixture = CoreFixture::new();
        let term_x = fixture.term_var(0, 10);
        let term_y = fixture.term_var(1, 11);
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
        assert_eq!(flow.diagnostics.len(), 4);
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
        let phase9 = flow
            .diagnostics
            .get(ControlFlowDiagnosticId::new(3))
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
