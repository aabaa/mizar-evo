//! Deterministic TPTP emission for backend-neutral ATP problems.
//!
//! This module implements the task-10 FOF emitter specified in
//! [tptp_encoder.md](../../../doc/design/mizar-atp/en/tptp_encoder.md).
//! The emitted text is untrusted backend input only; it is not kernel
//! evidence, not a SAT problem, and not accepted proof material.

use crate::problem::{
    AtpDeclaration, AtpDeclarationKind, AtpFormulaId, AtpFormulaTree, AtpProblem, AtpPropertyId,
    AtpProvenanceId, AtpSymbolName, AtpSymbolSource, AtpTerm, AtpTypeGuardId, ConcreteFormat,
    EqualitySupport, ExpectedBackendResult, LogicFragment, PropertyEncoding, QuantifierPolicy,
    SoftTypeStrategy,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum TptpDialect {
    Fof,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TptpEncodingOutput {
    text: String,
    symbol_bindings: Vec<TptpSymbolBinding>,
    formula_labels: Vec<TptpFormulaLabel>,
}

impl TptpEncodingOutput {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn symbol_bindings(&self) -> &[TptpSymbolBinding] {
        &self.symbol_bindings
    }

    pub fn formula_labels(&self) -> &[TptpFormulaLabel] {
        &self.formula_labels
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TptpSymbolBinding {
    atp_symbol: AtpSymbolName,
    tptp_name: String,
    source: AtpSymbolSource,
}

impl TptpSymbolBinding {
    pub const fn atp_symbol(&self) -> &AtpSymbolName {
        &self.atp_symbol
    }

    pub fn tptp_name(&self) -> &str {
        &self.tptp_name
    }

    pub const fn source(&self) -> &AtpSymbolSource {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct TptpFormulaLabel {
    label: String,
    item: TptpFormulaItem,
    provenance: AtpProvenanceId,
    target_symbol: Option<AtpSymbolName>,
}

impl TptpFormulaLabel {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub const fn item(&self) -> TptpFormulaItem {
        self.item
    }

    pub const fn provenance(&self) -> AtpProvenanceId {
        self.provenance
    }

    pub fn target_symbol(&self) -> Option<&AtpSymbolName> {
        self.target_symbol.as_ref()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum TptpFormulaItem {
    Axiom(AtpFormulaId),
    TypeGuard(AtpTypeGuardId),
    Property(AtpPropertyId),
    Conjecture(AtpFormulaId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum TptpEncodingError {
    UnsupportedProfile {
        feature: &'static str,
    },
    MissingFormulaPayload {
        formula_id: AtpFormulaId,
    },
    EmptyFormulaList {
        operator: &'static str,
    },
    EmptyQuantifier {
        quantifier: &'static str,
    },
    MissingDeclaration {
        symbol: AtpSymbolName,
    },
    MissingSymbolMap {
        symbol: AtpSymbolName,
    },
    InvalidDeclaration {
        symbol: AtpSymbolName,
        expected: &'static str,
        actual: AtpDeclarationKind,
    },
    InvalidArity {
        symbol: AtpSymbolName,
        expected: u32,
        actual: u32,
    },
    SortedBinder {
        variable: AtpSymbolName,
        sort: AtpSymbolName,
    },
    FreeVariable {
        variable: AtpSymbolName,
    },
    DuplicateBinder {
        variable: AtpSymbolName,
    },
    BinderShadowing {
        variable: AtpSymbolName,
    },
    InvalidBinderSource {
        variable: AtpSymbolName,
    },
    NativePropertyDeclaration {
        property: AtpPropertyId,
    },
    DuplicateTptpName {
        name: String,
    },
    IllegalTptpName {
        name: String,
    },
    ReservedTptpName {
        name: String,
    },
    DuplicateFormulaLabel {
        label: String,
    },
}

impl fmt::Display for TptpEncodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProfile { feature } => {
                write!(formatter, "unsupported TPTP FOF profile feature: {feature}")
            }
            Self::MissingFormulaPayload { formula_id } => {
                write!(formatter, "missing formula payload for {formula_id:?}")
            }
            Self::EmptyFormulaList { operator } => {
                write!(
                    formatter,
                    "empty {operator} formula list is not TPTP-encodable"
                )
            }
            Self::EmptyQuantifier { quantifier } => {
                write!(
                    formatter,
                    "empty {quantifier} binder list is not TPTP-encodable"
                )
            }
            Self::MissingDeclaration { symbol } => {
                write!(
                    formatter,
                    "missing declaration for symbol {}",
                    symbol.as_str()
                )
            }
            Self::MissingSymbolMap { symbol } => {
                write!(
                    formatter,
                    "missing symbol-map row for symbol {}",
                    symbol.as_str()
                )
            }
            Self::InvalidDeclaration {
                symbol,
                expected,
                actual,
            } => write!(
                formatter,
                "symbol {} must be declared as {expected}, found {actual:?}",
                symbol.as_str()
            ),
            Self::InvalidArity {
                symbol,
                expected,
                actual,
            } => write!(
                formatter,
                "symbol {} arity mismatch: expected {expected}, found {actual}",
                symbol.as_str()
            ),
            Self::SortedBinder { variable, sort } => write!(
                formatter,
                "FOF encoder does not support sorted binder {} : {}",
                variable.as_str(),
                sort.as_str()
            ),
            Self::FreeVariable { variable } => {
                write!(
                    formatter,
                    "free variable {} is not TPTP-encodable",
                    variable.as_str()
                )
            }
            Self::DuplicateBinder { variable } => write!(
                formatter,
                "duplicate binder {} in one quantifier",
                variable.as_str()
            ),
            Self::BinderShadowing { variable } => {
                write!(
                    formatter,
                    "nested binder shadowing for {}",
                    variable.as_str()
                )
            }
            Self::InvalidBinderSource { variable } => write!(
                formatter,
                "binder {} must have a generated-binder symbol-map source",
                variable.as_str()
            ),
            Self::NativePropertyDeclaration { property } => write!(
                formatter,
                "native property declaration {property:?} is deferred for TPTP FOF"
            ),
            Self::DuplicateTptpName { name } => {
                write!(formatter, "duplicate TPTP name {name}")
            }
            Self::IllegalTptpName { name } => {
                write!(formatter, "illegal TPTP name {name}")
            }
            Self::ReservedTptpName { name } => {
                write!(formatter, "reserved TPTP name {name}")
            }
            Self::DuplicateFormulaLabel { label } => {
                write!(formatter, "duplicate TPTP formula label {label}")
            }
        }
    }
}

impl Error for TptpEncodingError {}

pub fn encode_tptp(
    problem: &AtpProblem,
    dialect: TptpDialect,
) -> Result<TptpEncodingOutput, TptpEncodingError> {
    match dialect {
        TptpDialect::Fof => encode_fof(problem),
    }
}

fn encode_fof(problem: &AtpProblem) -> Result<TptpEncodingOutput, TptpEncodingError> {
    validate_fof_profile(problem)?;
    let mut context = EncodingContext::new(problem);
    let mut text = String::new();
    let mut formula_labels = Vec::new();

    for axiom in problem.axioms() {
        let label = context.register_formula_label("ax", axiom.id())?;
        let formula = axiom
            .formula()
            .ok_or(TptpEncodingError::MissingFormulaPayload {
                formula_id: axiom.id(),
            })?;
        let rendered = render_formula(formula, &mut context, &mut Scope::default())?;
        write_entry(&mut text, &label, "axiom", &rendered);
        formula_labels.push(TptpFormulaLabel {
            label,
            item: TptpFormulaItem::Axiom(axiom.id()),
            provenance: axiom.provenance(),
            target_symbol: None,
        });
    }

    for guard in problem.type_context().guards() {
        let label = context.register_formula_label("tg", guard.id())?;
        let rendered = render_formula(guard.formula(), &mut context, &mut Scope::default())?;
        write_entry(&mut text, &label, "axiom", &rendered);
        formula_labels.push(TptpFormulaLabel {
            label,
            item: TptpFormulaItem::TypeGuard(guard.id()),
            provenance: guard.provenance(),
            target_symbol: None,
        });
    }

    for property in problem.properties() {
        let label = context.register_formula_label("prop", property.id())?;
        let PropertyEncoding::Axiom(formula) = property.encoding() else {
            return Err(TptpEncodingError::NativePropertyDeclaration {
                property: property.id(),
            });
        };
        let rendered = render_formula(formula, &mut context, &mut Scope::default())?;
        write_entry(&mut text, &label, "axiom", &rendered);
        formula_labels.push(TptpFormulaLabel {
            label,
            item: TptpFormulaItem::Property(property.id()),
            provenance: property.provenance(),
            target_symbol: Some(property.target_symbol().clone()),
        });
    }

    let conjecture = problem.conjecture();
    let label = context.register_formula_label("conj", conjecture.id())?;
    let formula = conjecture
        .formula()
        .ok_or(TptpEncodingError::MissingFormulaPayload {
            formula_id: conjecture.id(),
        })?;
    let rendered = render_formula(formula, &mut context, &mut Scope::default())?;
    write_entry(&mut text, &label, "conjecture", &rendered);
    formula_labels.push(TptpFormulaLabel {
        label,
        item: TptpFormulaItem::Conjecture(conjecture.id()),
        provenance: conjecture.provenance(),
        target_symbol: None,
    });

    Ok(TptpEncodingOutput {
        text,
        symbol_bindings: context.symbol_bindings.into_iter().collect(),
        formula_labels,
    })
}

fn validate_fof_profile(problem: &AtpProblem) -> Result<(), TptpEncodingError> {
    let profile = problem.logic_profile();
    if !profile.concrete_formats().contains(&ConcreteFormat::Tptp) {
        return Err(TptpEncodingError::UnsupportedProfile {
            feature: "TPTP concrete format",
        });
    }
    if profile.fragment() != LogicFragment::Fof {
        return Err(TptpEncodingError::UnsupportedProfile {
            feature: "non-FOF logic fragment",
        });
    }
    if problem.expected_result() != ExpectedBackendResult::Unsat {
        return Err(TptpEncodingError::UnsupportedProfile {
            feature: "non-Unsat expected result",
        });
    }
    if profile.soft_types() != SoftTypeStrategy::GuardPredicates {
        return Err(TptpEncodingError::UnsupportedProfile {
            feature: "non-guard-predicate soft type strategy",
        });
    }
    Ok(())
}

fn write_entry(output: &mut String, label: &str, role: &str, formula: &str) {
    writeln!(output, "fof({label}, {role}, {formula}).").expect("write string");
}

#[derive(Default)]
struct Scope {
    variables: BTreeMap<AtpSymbolName, String>,
}

struct EncodingContext {
    profile_equality: EqualitySupport,
    profile_quantifiers: QuantifierPolicy,
    declarations: BTreeMap<AtpSymbolName, AtpDeclaration>,
    symbol_sources: BTreeMap<AtpSymbolName, AtpSymbolSource>,
    name_owners: BTreeMap<String, String>,
    symbol_bindings: BTreeSet<TptpSymbolBinding>,
    formula_labels: BTreeSet<String>,
}

impl EncodingContext {
    fn new(problem: &AtpProblem) -> Self {
        Self {
            profile_equality: problem.logic_profile().equality(),
            profile_quantifiers: problem.logic_profile().quantifiers(),
            declarations: problem
                .declarations()
                .iter()
                .cloned()
                .map(|declaration| (declaration.symbol().clone(), declaration))
                .collect(),
            symbol_sources: problem
                .symbol_map()
                .iter()
                .map(|entry| (entry.backend_symbol().clone(), entry.source().clone()))
                .collect(),
            name_owners: BTreeMap::new(),
            symbol_bindings: BTreeSet::new(),
            formula_labels: BTreeSet::new(),
        }
    }

    fn register_formula_label(
        &mut self,
        prefix: &'static str,
        id: impl FormulaLabelId,
    ) -> Result<String, TptpEncodingError> {
        let id = id.index();
        let label = format!("{prefix}_{id}");
        validate_tptp_name(&label)?;
        if !self.formula_labels.insert(label.clone()) {
            return Err(TptpEncodingError::DuplicateFormulaLabel { label });
        }
        self.register_name(label.clone(), format!("label:{prefix}:{id}"))?;
        Ok(label)
    }

    fn mangle_symbol(
        &mut self,
        symbol: &AtpSymbolName,
        expected_kind: AtpDeclarationKind,
        expected_arity: u32,
        expected: &'static str,
    ) -> Result<String, TptpEncodingError> {
        let (declaration, source) =
            self.symbol_signature(symbol, expected_kind, expected_arity, expected)?;
        if declaration.kind() == AtpDeclarationKind::GeneratedBinder {
            return Err(TptpEncodingError::InvalidDeclaration {
                symbol: symbol.clone(),
                expected,
                actual: declaration.kind(),
            });
        }
        let key = length_delimited_fields(&[
            "symbol".to_owned(),
            declaration_kind_key(declaration.kind()).to_owned(),
            declaration.arity().to_string(),
            symbol_source_key(&source),
            declaration.symbol().as_str().to_owned(),
        ]);
        let name = format!("m_{}", hex(key.as_bytes()));
        self.register_name(name.clone(), format!("symbol:{key}"))?;
        self.symbol_bindings.insert(TptpSymbolBinding {
            atp_symbol: symbol.clone(),
            tptp_name: name.clone(),
            source,
        });
        Ok(name)
    }

    fn mangle_binder(
        &mut self,
        binder: &AtpSymbolName,
        position: usize,
    ) -> Result<String, TptpEncodingError> {
        let (declaration, source) = self.symbol_signature(
            binder,
            AtpDeclarationKind::GeneratedBinder,
            0,
            "generated binder",
        )?;
        if !matches!(source, AtpSymbolSource::GeneratedBinder(_)) {
            return Err(TptpEncodingError::InvalidBinderSource {
                variable: binder.clone(),
            });
        }
        let key = length_delimited_fields(&[
            "binder".to_owned(),
            symbol_source_key(&source),
            declaration.id().index().to_string(),
            declaration.arity().to_string(),
            position.to_string(),
        ]);
        let name = format!("V_{}", hex(key.as_bytes()));
        self.register_name(name.clone(), format!("binder:{key}"))?;
        self.symbol_bindings.insert(TptpSymbolBinding {
            atp_symbol: binder.clone(),
            tptp_name: name.clone(),
            source,
        });
        Ok(name)
    }

    fn symbol_signature(
        &self,
        symbol: &AtpSymbolName,
        expected_kind: AtpDeclarationKind,
        expected_arity: u32,
        expected: &'static str,
    ) -> Result<(AtpDeclaration, AtpSymbolSource), TptpEncodingError> {
        let declaration = self.declarations.get(symbol).cloned().ok_or_else(|| {
            TptpEncodingError::MissingDeclaration {
                symbol: symbol.clone(),
            }
        })?;
        let source = self.symbol_sources.get(symbol).cloned().ok_or_else(|| {
            TptpEncodingError::MissingSymbolMap {
                symbol: symbol.clone(),
            }
        })?;
        if declaration.kind() != expected_kind {
            return Err(TptpEncodingError::InvalidDeclaration {
                symbol: symbol.clone(),
                expected,
                actual: declaration.kind(),
            });
        }
        if declaration.arity() != expected_arity {
            return Err(TptpEncodingError::InvalidArity {
                symbol: symbol.clone(),
                expected: expected_arity,
                actual: declaration.arity(),
            });
        }
        Ok((declaration, source))
    }

    fn register_name(&mut self, name: String, owner: String) -> Result<(), TptpEncodingError> {
        validate_tptp_name(&name)?;
        if let Some(existing_owner) = self.name_owners.get(&name) {
            if existing_owner != &owner {
                return Err(TptpEncodingError::DuplicateTptpName { name });
            }
            return Ok(());
        }
        self.name_owners.insert(name, owner);
        Ok(())
    }
}

trait FormulaLabelId {
    fn index(self) -> u32;
}

impl FormulaLabelId for AtpFormulaId {
    fn index(self) -> u32 {
        self.index()
    }
}

impl FormulaLabelId for AtpPropertyId {
    fn index(self) -> u32 {
        self.index()
    }
}

impl FormulaLabelId for AtpTypeGuardId {
    fn index(self) -> u32 {
        self.index()
    }
}

fn render_formula(
    formula: &AtpFormulaTree,
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, TptpEncodingError> {
    match formula {
        AtpFormulaTree::True => Ok("$true".to_owned()),
        AtpFormulaTree::False => Ok("$false".to_owned()),
        AtpFormulaTree::Atom(atom) => render_atom(atom, context, scope),
        AtpFormulaTree::Equality { left, right } => {
            if context.profile_equality != EqualitySupport::Supported {
                return Err(TptpEncodingError::UnsupportedProfile {
                    feature: "equality",
                });
            }
            Ok(format!(
                "({} = {})",
                render_term(left, context, scope)?,
                render_term(right, context, scope)?
            ))
        }
        AtpFormulaTree::Not(inner) => Ok(format!("~({})", render_formula(inner, context, scope)?)),
        AtpFormulaTree::And(formulas) => render_nary("and", "&", formulas, context, scope),
        AtpFormulaTree::Or(formulas) => render_nary("or", "|", formulas, context, scope),
        AtpFormulaTree::Implies(left, right) => Ok(format!(
            "({} => {})",
            render_formula(left, context, scope)?,
            render_formula(right, context, scope)?
        )),
        AtpFormulaTree::Forall { binders, body } => {
            render_quantifier("forall", "!", binders, body, context, scope)
        }
        AtpFormulaTree::Exists { binders, body } => {
            render_quantifier("exists", "?", binders, body, context, scope)
        }
    }
}

fn render_atom(
    atom: &crate::problem::AtpAtom,
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, TptpEncodingError> {
    let name = context.mangle_symbol(
        atom.predicate(),
        AtpDeclarationKind::Predicate,
        atom.arguments().len() as u32,
        "predicate",
    )?;
    if atom.arguments().is_empty() {
        return Ok(name);
    }
    let arguments = atom
        .arguments()
        .iter()
        .map(|argument| render_term(argument, context, scope))
        .collect::<Result<Vec<_>, _>>()?
        .join(", ");
    Ok(format!("{name}({arguments})"))
}

fn render_nary(
    operator_name: &'static str,
    operator: &str,
    formulas: &[AtpFormulaTree],
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, TptpEncodingError> {
    if formulas.is_empty() {
        return Err(TptpEncodingError::EmptyFormulaList {
            operator: operator_name,
        });
    }
    let rendered = formulas
        .iter()
        .map(|formula| render_formula(formula, context, scope))
        .collect::<Result<Vec<_>, _>>()?;
    if rendered.len() == 1 {
        return Ok(format!("({})", rendered[0]));
    }
    Ok(format!("({})", rendered.join(&format!(" {operator} "))))
}

fn render_quantifier(
    quantifier_name: &'static str,
    operator: &str,
    binders: &[crate::problem::AtpBinder],
    body: &AtpFormulaTree,
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, TptpEncodingError> {
    if context.profile_quantifiers != QuantifierPolicy::FirstOrder {
        return Err(TptpEncodingError::UnsupportedProfile {
            feature: "quantifier",
        });
    }
    if binders.is_empty() {
        return Err(TptpEncodingError::EmptyQuantifier {
            quantifier: quantifier_name,
        });
    }

    let mut local = BTreeSet::new();
    let mut inserted = Vec::new();
    let mut names = Vec::new();
    for (position, binder) in binders.iter().enumerate() {
        if let Some(sort) = binder.sort() {
            return Err(TptpEncodingError::SortedBinder {
                variable: binder.variable().clone(),
                sort: sort.clone(),
            });
        }
        if !local.insert(binder.variable().clone()) {
            return Err(TptpEncodingError::DuplicateBinder {
                variable: binder.variable().clone(),
            });
        }
        if scope.variables.contains_key(binder.variable()) {
            return Err(TptpEncodingError::BinderShadowing {
                variable: binder.variable().clone(),
            });
        }
        let name = context.mangle_binder(binder.variable(), position)?;
        scope
            .variables
            .insert(binder.variable().clone(), name.clone());
        inserted.push(binder.variable().clone());
        names.push(name);
    }

    let body = render_formula(body, context, scope);
    for variable in inserted {
        scope.variables.remove(&variable);
    }
    let body = body?;

    Ok(format!("{operator} [{}] : ({body})", names.join(", "))).map(|body| format!("({body})"))
}

fn render_term(
    term: &AtpTerm,
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, TptpEncodingError> {
    match term {
        AtpTerm::Variable(variable) => {
            scope
                .variables
                .get(variable)
                .cloned()
                .ok_or_else(|| TptpEncodingError::FreeVariable {
                    variable: variable.clone(),
                })
        }
        AtpTerm::Function {
            function,
            arguments,
        } => {
            let name = context.mangle_symbol(
                function,
                AtpDeclarationKind::Function,
                arguments.len() as u32,
                "function",
            )?;
            if arguments.is_empty() {
                return Ok(name);
            }
            let arguments = arguments
                .iter()
                .map(|argument| render_term(argument, context, scope))
                .collect::<Result<Vec<_>, _>>()?
                .join(", ");
            Ok(format!("{name}({arguments})"))
        }
    }
}

fn declaration_kind_key(kind: AtpDeclarationKind) -> &'static str {
    match kind {
        AtpDeclarationKind::Sort => "sort",
        AtpDeclarationKind::Function => "function",
        AtpDeclarationKind::Predicate => "predicate",
        AtpDeclarationKind::GeneratedBinder => "generated-binder",
    }
}

fn symbol_source_key(source: &AtpSymbolSource) -> String {
    match source {
        AtpSymbolSource::MizarSymbol(binding) => {
            length_delimited_fields(&["mizar-symbol".to_owned(), binding.as_str().to_owned()])
        }
        AtpSymbolSource::GeneratedBinder(binding) => {
            length_delimited_fields(&["generated-binder".to_owned(), binding.as_str().to_owned()])
        }
        AtpSymbolSource::TypeGuard(id) => {
            length_delimited_fields(&["type-guard".to_owned(), id.index().to_string()])
        }
    }
}

fn length_delimited_fields(fields: &[String]) -> String {
    let mut output = String::new();
    for field in fields {
        write!(output, "{}:{field};", field.len()).expect("write string");
    }
    output
}

fn hex(bytes: &[u8]) -> String {
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        write!(output, "{byte:02x}").expect("write string");
    }
    output
}

fn validate_tptp_name(name: &str) -> Result<(), TptpEncodingError> {
    let mut characters = name.chars();
    let Some(first) = characters.next() else {
        return Err(TptpEncodingError::IllegalTptpName {
            name: name.to_owned(),
        });
    };
    let valid_first = first.is_ascii_lowercase() || first.is_ascii_uppercase();
    let valid_rest =
        characters.all(|character| character == '_' || character.is_ascii_alphanumeric());
    if !valid_first || !valid_rest {
        return Err(TptpEncodingError::IllegalTptpName {
            name: name.to_owned(),
        });
    }
    if reserved_tptp_name(name) {
        return Err(TptpEncodingError::ReservedTptpName {
            name: name.to_owned(),
        });
    }
    Ok(())
}

fn reserved_tptp_name(name: &str) -> bool {
    matches!(
        name,
        "fof"
            | "cnf"
            | "tff"
            | "thf"
            | "tcf"
            | "include"
            | "axiom"
            | "hypothesis"
            | "definition"
            | "assumption"
            | "lemma"
            | "theorem"
            | "conjecture"
            | "negated_conjecture"
            | "plain"
            | "type"
            | "unknown"
            | "$true"
            | "$false"
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::{
        AtpAtom, AtpBinder, AtpDeclarationId, AtpDiagnostic, AtpFingerprint, AtpFormula,
        AtpFormulaTree, AtpPayload, AtpProblemParts, AtpProvenance, AtpSourceBinding, AtpSourceRef,
        AtpSymbolMapEntry, AtpTargetBinding, AtpTypeContext, AtpTypeGuard, EncodedProperty,
        NativePropertySupport,
    };
    use std::collections::BTreeSet;

    #[test]
    fn emits_golden_fof_entries_in_section_order() {
        let problem = AtpProblem::try_new(populated_parts(false, "diag-a")).expect("problem");
        let output = encode_tptp(&problem, TptpDialect::Fof).expect("tptp output");
        let predicate = tptp_name_for(output.symbol_bindings(), "P");
        let constant = tptp_name_for(output.symbol_bindings(), "a1");
        let binder = tptp_name_for(output.symbol_bindings(), "x");

        assert_eq!(
            output.text(),
            format!(
                concat!(
                    "fof(ax_1, axiom, {predicate}({constant})).\n",
                    "fof(tg_1, axiom, {predicate}({constant})).\n",
                    "fof(prop_1, axiom, (! [{binder}] : (({predicate}({binder}) => {predicate}({binder}))))).\n",
                    "fof(conj_2, conjecture, ({constant} = {constant})).\n"
                ),
                predicate = predicate,
                constant = constant,
                binder = binder
            )
        );
        assert!(output.text().ends_with('\n'));
        assert_eq!(output.text().matches('\n').count(), 4);

        let labels = output
            .formula_labels()
            .iter()
            .map(|label| {
                (
                    label.label(),
                    label.item(),
                    label.provenance().index(),
                    label.target_symbol().map(AtpSymbolName::as_str),
                )
            })
            .collect::<Vec<_>>();
        assert_eq!(
            labels,
            [
                (
                    "ax_1",
                    TptpFormulaItem::Axiom(AtpFormulaId::new(1)),
                    2,
                    None
                ),
                (
                    "tg_1",
                    TptpFormulaItem::TypeGuard(AtpTypeGuardId::new(1)),
                    4,
                    None
                ),
                (
                    "prop_1",
                    TptpFormulaItem::Property(AtpPropertyId::new(1)),
                    6,
                    Some("P")
                ),
                (
                    "conj_2",
                    TptpFormulaItem::Conjecture(AtpFormulaId::new(2)),
                    3,
                    None
                )
            ]
        );
    }

    #[test]
    fn output_is_deterministic_and_ignores_diagnostics() {
        let problem_a =
            AtpProblem::try_new(populated_parts(false, "fof(raw_diagnostic)")).expect("problem a");
        let problem_b =
            AtpProblem::try_new(populated_parts(true, "$true\nraw_diagnostic")).expect("problem b");

        let output_a = encode_tptp(&problem_a, TptpDialect::Fof).expect("output a");
        let output_b = encode_tptp(&problem_b, TptpDialect::Fof).expect("output b");

        assert_eq!(output_a.text(), output_b.text());
        assert_eq!(output_a.symbol_bindings(), output_b.symbol_bindings());
        assert_eq!(output_a.formula_labels(), output_b.formula_labels());
        for prohibited in [
            "fof(raw_diagnostic)",
            "$true\nraw_diagnostic",
            "diagnostic-key-z",
            "diagnostic-key-a",
            "diagnostic order must not matter",
        ] {
            assert!(
                !output_a.text().contains(prohibited),
                "semantic TPTP text leaked diagnostic payload `{prohibited}`"
            );
            assert!(
                !output_b.text().contains(prohibited),
                "semantic TPTP text leaked diagnostic payload `{prohibited}`"
            );
        }
    }

    #[test]
    fn renders_all_formula_shapes() {
        let mut parts = base_parts();
        parts.logic_profile = profile(
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::GuardPredicates,
            BTreeSet::from([ConcreteFormat::Tptp]),
        );
        parts.conjecture = AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Exists {
                binders: vec![AtpBinder::new("y", None)],
                body: Box::new(AtpFormulaTree::And(vec![
                    AtpFormulaTree::True,
                    AtpFormulaTree::False,
                    AtpFormulaTree::Not(Box::new(atom("P", vec![variable("y")]))),
                    AtpFormulaTree::Or(vec![
                        atom("P", vec![function("f", vec![constant("a1")])]),
                        AtpFormulaTree::Equality {
                            left: variable("y"),
                            right: constant("a1"),
                        },
                    ]),
                ])),
            },
            AtpProvenanceId::new(3),
        );
        add_binder(&mut parts, 6, "y", "binder:y");
        add_function(&mut parts, 7, "f", 1, "fun:f");
        let problem = AtpProblem::try_new(parts).expect("problem");

        let text = encode_tptp(&problem, TptpDialect::Fof)
            .expect("output")
            .text()
            .to_owned();
        let output = encode_tptp(&problem, TptpDialect::Fof).expect("output");
        let predicate = tptp_name_for(output.symbol_bindings(), "P");
        let constant = tptp_name_for(output.symbol_bindings(), "a1");
        let binder = tptp_name_for(output.symbol_bindings(), "y");
        let function = tptp_name_for(output.symbol_bindings(), "f");

        assert_eq!(
            text,
            format!(
                concat!(
                    "fof(ax_1, axiom, {predicate}({constant})).\n",
                    "fof(tg_1, axiom, {predicate}({constant})).\n",
                    "fof(conj_2, conjecture, (? [{binder}] : (($true & $false & ~({predicate}({binder})) & ({predicate}({function}({constant})) | ({binder} = {constant})))))).\n"
                ),
                predicate = predicate,
                constant = constant,
                binder = binder,
                function = function
            )
        );
    }

    #[test]
    fn rejects_unsupported_profiles_and_native_properties() {
        for (parts, feature) in [
            (
                {
                    let mut parts = base_parts();
                    parts.logic_profile = profile(
                        LogicFragment::Fof,
                        EqualitySupport::Supported,
                        QuantifierPolicy::PropositionalOnly,
                        SoftTypeStrategy::GuardPredicates,
                        BTreeSet::from([ConcreteFormat::SmtLib]),
                    );
                    parts
                },
                "TPTP concrete format",
            ),
            (
                {
                    let mut parts = base_parts();
                    parts.logic_profile = profile(
                        LogicFragment::TffLike,
                        EqualitySupport::Supported,
                        QuantifierPolicy::PropositionalOnly,
                        SoftTypeStrategy::GuardPredicates,
                        BTreeSet::from([ConcreteFormat::Tptp]),
                    );
                    parts
                },
                "non-FOF logic fragment",
            ),
            (
                {
                    let mut parts = base_parts();
                    parts.logic_profile = profile(
                        LogicFragment::Fof,
                        EqualitySupport::Supported,
                        QuantifierPolicy::PropositionalOnly,
                        SoftTypeStrategy::BackendSorts,
                        BTreeSet::from([ConcreteFormat::Tptp]),
                    );
                    parts
                },
                "non-guard-predicate soft type strategy",
            ),
            (
                {
                    let mut parts = base_parts();
                    parts.logic_profile = profile(
                        LogicFragment::Fof,
                        EqualitySupport::Supported,
                        QuantifierPolicy::PropositionalOnly,
                        SoftTypeStrategy::SortsAndGuards,
                        BTreeSet::from([ConcreteFormat::Tptp]),
                    );
                    parts
                },
                "non-guard-predicate soft type strategy",
            ),
        ] {
            let problem = AtpProblem::try_new(parts).expect("profile-compatible problem");
            assert_eq!(
                encode_tptp(&problem, TptpDialect::Fof),
                Err(TptpEncodingError::UnsupportedProfile { feature })
            );
        }

        let mut parts = base_parts();
        parts.logic_profile = profile_with_native_property_support();
        parts
            .declarations
            .push(declaration(8, AtpDeclarationKind::Function, "native", 0));
        parts.symbol_map.push(symbol("native", "native:decl"));
        parts.properties = vec![EncodedProperty::native_declaration(
            AtpPropertyId::new(1),
            "P",
            AtpDeclarationId::new(8),
            AtpProvenanceId::new(6),
        )];
        let problem = AtpProblem::try_new(parts).expect("native property problem");
        assert_eq!(
            encode_tptp(&problem, TptpDialect::Fof),
            Err(TptpEncodingError::NativePropertyDeclaration {
                property: AtpPropertyId::new(1)
            })
        );
    }

    #[test]
    fn rejects_scope_and_sorted_binder_errors() {
        let mut free = base_parts();
        free.declarations
            .push(declaration(5, AtpDeclarationKind::GeneratedBinder, "x", 0));
        free.symbol_map.push(generated_binder("x", "binder:x"));
        free.conjecture = AtpFormula::new(
            AtpFormulaId::new(2),
            atom("P", vec![variable("x")]),
            AtpProvenanceId::new(3),
        );
        let problem = AtpProblem::try_new(free).expect("free variable problem");
        assert_eq!(
            encode_tptp(&problem, TptpDialect::Fof),
            Err(TptpEncodingError::FreeVariable {
                variable: AtpSymbolName::new("x")
            })
        );

        let mut duplicate = base_parts();
        duplicate.logic_profile = first_order_profile();
        add_binder(&mut duplicate, 5, "x", "binder:x");
        duplicate.conjecture = AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", None), AtpBinder::new("x", None)],
                body: Box::new(atom("P", vec![variable("x")])),
            },
            AtpProvenanceId::new(3),
        );
        let problem = AtpProblem::try_new(duplicate).expect("duplicate binder problem");
        assert_eq!(
            encode_tptp(&problem, TptpDialect::Fof),
            Err(TptpEncodingError::DuplicateBinder {
                variable: AtpSymbolName::new("x")
            })
        );

        let mut shadow = base_parts();
        shadow.logic_profile = first_order_profile();
        add_binder(&mut shadow, 5, "x", "binder:x");
        shadow.conjecture = AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", None)],
                body: Box::new(AtpFormulaTree::Exists {
                    binders: vec![AtpBinder::new("x", None)],
                    body: Box::new(atom("P", vec![variable("x")])),
                }),
            },
            AtpProvenanceId::new(3),
        );
        let problem = AtpProblem::try_new(shadow).expect("shadowing problem");
        assert_eq!(
            encode_tptp(&problem, TptpDialect::Fof),
            Err(TptpEncodingError::BinderShadowing {
                variable: AtpSymbolName::new("x")
            })
        );

        let mut sorted = base_parts();
        sorted.logic_profile = first_order_profile();
        add_binder(&mut sorted, 5, "x", "binder:x");
        add_sort(&mut sorted, 9, "S", "sort:S");
        sorted.conjecture = AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", Some(AtpSymbolName::new("S")))],
                body: Box::new(atom("P", vec![variable("x")])),
            },
            AtpProvenanceId::new(3),
        );
        let problem = AtpProblem::try_new(sorted).expect("sorted binder problem");
        assert_eq!(
            encode_tptp(&problem, TptpDialect::Fof),
            Err(TptpEncodingError::SortedBinder {
                variable: AtpSymbolName::new("x"),
                sort: AtpSymbolName::new("S")
            })
        );
    }

    #[test]
    fn rejects_private_rendering_failure_cases() {
        let problem = AtpProblem::try_new(base_parts()).expect("problem");
        let mut context = EncodingContext::new(&problem);
        assert_eq!(
            render_formula(
                &AtpFormulaTree::And(Vec::new()),
                &mut context,
                &mut Scope::default()
            ),
            Err(TptpEncodingError::EmptyFormulaList { operator: "and" })
        );
        assert_eq!(
            render_formula(
                &AtpFormulaTree::Forall {
                    binders: Vec::new(),
                    body: Box::new(AtpFormulaTree::True),
                },
                &mut context,
                &mut Scope::default()
            ),
            Err(TptpEncodingError::UnsupportedProfile {
                feature: "quantifier"
            })
        );

        let mut first_order = EncodingContext::new(
            &AtpProblem::try_new({
                let mut parts = base_parts();
                parts.logic_profile = first_order_profile();
                parts
            })
            .expect("first-order problem"),
        );
        assert_eq!(
            render_formula(
                &AtpFormulaTree::Forall {
                    binders: Vec::new(),
                    body: Box::new(AtpFormulaTree::True),
                },
                &mut first_order,
                &mut Scope::default()
            ),
            Err(TptpEncodingError::EmptyQuantifier {
                quantifier: "forall"
            })
        );

        let mut no_equality = EncodingContext::new(
            &AtpProblem::try_new({
                let mut parts = base_parts();
                parts.logic_profile = profile(
                    LogicFragment::Fof,
                    EqualitySupport::Unsupported,
                    QuantifierPolicy::PropositionalOnly,
                    SoftTypeStrategy::GuardPredicates,
                    BTreeSet::from([ConcreteFormat::Tptp]),
                );
                parts.conjecture = AtpFormula::new(
                    AtpFormulaId::new(2),
                    atom("P", vec![constant("a1")]),
                    AtpProvenanceId::new(3),
                );
                parts
            })
            .expect("no-equality problem"),
        );
        assert_eq!(
            render_formula(
                &AtpFormulaTree::Equality {
                    left: constant("a1"),
                    right: constant("a1"),
                },
                &mut no_equality,
                &mut Scope::default()
            ),
            Err(TptpEncodingError::UnsupportedProfile {
                feature: "equality"
            })
        );
    }

    #[test]
    fn rejects_encoder_validation_failures_fail_closed() {
        let problem = AtpProblem::try_new(base_parts()).expect("problem");

        let mut missing_declaration = EncodingContext::new(&problem);
        assert_eq!(
            render_formula(
                &atom("missing", Vec::new()),
                &mut missing_declaration,
                &mut Scope::default()
            ),
            Err(TptpEncodingError::MissingDeclaration {
                symbol: AtpSymbolName::new("missing")
            })
        );

        let mut missing_symbol_map = EncodingContext::new(&problem);
        missing_symbol_map
            .symbol_sources
            .remove(&AtpSymbolName::new("P"));
        assert_eq!(
            render_formula(
                &atom("P", vec![constant("a1")]),
                &mut missing_symbol_map,
                &mut Scope::default()
            ),
            Err(TptpEncodingError::MissingSymbolMap {
                symbol: AtpSymbolName::new("P")
            })
        );

        let mut invalid_kind = EncodingContext::new(&problem);
        invalid_kind.declarations.insert(
            AtpSymbolName::new("P"),
            declaration(1, AtpDeclarationKind::Function, "P", 1),
        );
        assert_eq!(
            render_formula(
                &atom("P", vec![constant("a1")]),
                &mut invalid_kind,
                &mut Scope::default()
            ),
            Err(TptpEncodingError::InvalidDeclaration {
                symbol: AtpSymbolName::new("P"),
                expected: "predicate",
                actual: AtpDeclarationKind::Function
            })
        );

        let mut invalid_arity = EncodingContext::new(&problem);
        invalid_arity.declarations.insert(
            AtpSymbolName::new("P"),
            declaration(1, AtpDeclarationKind::Predicate, "P", 2),
        );
        assert_eq!(
            render_formula(
                &atom("P", vec![constant("a1")]),
                &mut invalid_arity,
                &mut Scope::default()
            ),
            Err(TptpEncodingError::InvalidArity {
                symbol: AtpSymbolName::new("P"),
                expected: 1,
                actual: 2
            })
        );

        let mut invalid_binder_source = base_parts();
        invalid_binder_source.logic_profile = first_order_profile();
        invalid_binder_source.declarations.push(declaration(
            5,
            AtpDeclarationKind::GeneratedBinder,
            "x",
            0,
        ));
        invalid_binder_source
            .symbol_map
            .push(symbol("x", "not-generated-binder:x"));
        invalid_binder_source.conjecture = AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", None)],
                body: Box::new(atom("P", vec![variable("x")])),
            },
            AtpProvenanceId::new(3),
        );
        let problem = AtpProblem::try_new(invalid_binder_source)
            .expect("problem with non-generated binder source");
        assert_eq!(
            encode_tptp(&problem, TptpDialect::Fof),
            Err(TptpEncodingError::InvalidBinderSource {
                variable: AtpSymbolName::new("x")
            })
        );
    }

    #[test]
    fn raw_name_injection_is_mangled_out_of_symbol_positions() {
        let raw_predicate = "$true\nfof-injection";
        let raw_constant = "axiom, conjecture";
        let raw_binder = "Upper lower $false\npunctuation, whitespace";
        let mut parts = base_parts();
        parts.logic_profile = first_order_profile();
        parts.declarations = vec![
            declaration(1, AtpDeclarationKind::Predicate, raw_predicate, 1),
            declaration(2, AtpDeclarationKind::Function, raw_constant, 0),
            declaration(5, AtpDeclarationKind::GeneratedBinder, raw_binder, 0),
        ];
        parts.symbol_map = vec![
            symbol(raw_predicate, "pred:raw"),
            symbol(raw_constant, "const:raw"),
            generated_binder(raw_binder, "binder:raw"),
        ];
        parts.axioms = vec![AtpFormula::new(
            AtpFormulaId::new(1),
            atom(raw_predicate, vec![constant(raw_constant)]),
            AtpProvenanceId::new(2),
        )];
        parts.type_context = AtpTypeContext::new(vec![AtpTypeGuard::new(
            AtpTypeGuardId::new(1),
            atom(raw_predicate, vec![constant(raw_constant)]),
            AtpProvenanceId::new(4),
        )]);
        parts.conjecture = AtpFormula::new(
            AtpFormulaId::new(2),
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new(raw_binder, None)],
                body: Box::new(atom(raw_predicate, vec![variable(raw_binder)])),
            },
            AtpProvenanceId::new(3),
        );
        let problem = AtpProblem::try_new(parts).expect("raw-name problem");

        let output = encode_tptp(&problem, TptpDialect::Fof).expect("output");

        assert!(!output.text().contains(raw_predicate));
        assert!(!output.text().contains(raw_constant));
        assert!(!output.text().contains(raw_binder));
        assert!(output.symbol_bindings().iter().any(|binding| {
            binding.atp_symbol().as_str() == raw_binder && binding.tptp_name().starts_with("V_")
        }));
        assert!(output.symbol_bindings().iter().any(|binding| {
            binding.atp_symbol().as_str() == raw_predicate && binding.tptp_name().starts_with("m_")
        }));
        assert!(output.symbol_bindings().iter().any(|binding| {
            binding.atp_symbol().as_str() == raw_constant && binding.tptp_name().starts_with("m_")
        }));
        for binding in output.symbol_bindings() {
            assert!(output.text().contains(binding.tptp_name()));
        }
    }

    #[test]
    fn tracks_symbol_metadata_without_source_payload_in_text() {
        let problem = AtpProblem::try_new(populated_parts(false, "diag")).expect("problem");
        let output = encode_tptp(&problem, TptpDialect::Fof).expect("output");

        assert!(output.symbol_bindings().iter().any(|binding| {
            binding.atp_symbol().as_str() == "P"
                && binding.tptp_name().starts_with("m_")
                && matches!(binding.source(), AtpSymbolSource::MizarSymbol(_))
        }));
        assert!(output.symbol_bindings().iter().any(|binding| {
            binding.atp_symbol().as_str() == "x"
                && binding.tptp_name().starts_with("V_")
                && matches!(binding.source(), AtpSymbolSource::GeneratedBinder(_))
        }));
        assert!(!output.text().contains("pred:P"));
        assert!(!output.text().contains("binder:x"));
    }

    #[test]
    fn duplicate_name_and_illegal_name_checks_fail_closed() {
        let problem = AtpProblem::try_new(base_parts()).expect("problem");
        let mut context = EncodingContext::new(&problem);
        context
            .register_name("m_abc".to_owned(), "owner:1".to_owned())
            .expect("first owner");
        assert_eq!(
            context.register_name("m_abc".to_owned(), "owner:2".to_owned()),
            Err(TptpEncodingError::DuplicateTptpName {
                name: "m_abc".to_owned()
            })
        );
        assert_eq!(
            validate_tptp_name("fof"),
            Err(TptpEncodingError::ReservedTptpName {
                name: "fof".to_owned()
            })
        );
        assert_eq!(
            validate_tptp_name("bad-name"),
            Err(TptpEncodingError::IllegalTptpName {
                name: "bad-name".to_owned()
            })
        );
    }

    fn populated_parts(reverse: bool, diagnostic: &str) -> AtpProblemParts {
        let mut parts = base_parts();
        add_binder(&mut parts, 5, "x", "binder:x");
        parts.logic_profile = first_order_profile();
        parts.properties = vec![EncodedProperty::axiom(
            AtpPropertyId::new(1),
            "P",
            AtpFormulaTree::Forall {
                binders: vec![AtpBinder::new("x", None)],
                body: Box::new(AtpFormulaTree::Implies(
                    Box::new(atom("P", vec![variable("x")])),
                    Box::new(atom("P", vec![variable("x")])),
                )),
            },
            AtpProvenanceId::new(6),
        )];
        parts.diagnostics = vec![
            AtpDiagnostic::new("diagnostic-key-z", diagnostic),
            AtpDiagnostic::new("diagnostic-key-a", "diagnostic order must not matter"),
        ];
        if reverse {
            parts.declarations.reverse();
            parts.axioms.reverse();
            parts.symbol_map.reverse();
            parts.provenance.reverse();
            parts.properties.reverse();
            parts.type_context =
                AtpTypeContext::new(parts.type_context.guards().iter().cloned().rev().collect());
            parts.diagnostics.reverse();
        }
        parts
    }

    fn base_parts() -> AtpProblemParts {
        AtpProblemParts {
            vc_id: mizar_vc::vc_ir::VcId::new(21),
            target_binding: target_binding(),
            logic_profile: profile(
                LogicFragment::Fof,
                EqualitySupport::Supported,
                QuantifierPolicy::PropositionalOnly,
                SoftTypeStrategy::GuardPredicates,
                BTreeSet::from([ConcreteFormat::Tptp]),
            ),
            expected_result: ExpectedBackendResult::Unsat,
            declarations: vec![
                declaration(1, AtpDeclarationKind::Predicate, "P", 1),
                declaration(2, AtpDeclarationKind::Function, "a1", 0),
            ],
            axioms: vec![AtpFormula::new(
                AtpFormulaId::new(1),
                atom("P", vec![constant("a1")]),
                AtpProvenanceId::new(2),
            )],
            conjecture: AtpFormula::new(
                AtpFormulaId::new(2),
                AtpFormulaTree::Equality {
                    left: constant("a1"),
                    right: constant("a1"),
                },
                AtpProvenanceId::new(3),
            ),
            type_context: AtpTypeContext::new(vec![AtpTypeGuard::new(
                AtpTypeGuardId::new(1),
                atom("P", vec![constant("a1")]),
                AtpProvenanceId::new(4),
            )]),
            properties: Vec::new(),
            symbol_map: vec![symbol("P", "pred:P"), symbol("a1", "const:a1")],
            provenance: vec![
                provenance(
                    1,
                    AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("decls")),
                ),
                provenance(
                    2,
                    AtpSourceRef::CitedPremise(AtpSourceBinding::new("premise:1")),
                ),
                provenance(
                    3,
                    AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
                ),
                provenance(4, AtpSourceRef::TypeFact(AtpSourceBinding::new("type:1"))),
                provenance(
                    5,
                    AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("binder:source")),
                ),
                provenance(
                    6,
                    AtpSourceRef::EncodedProperty(AtpSourceBinding::new("property:1")),
                ),
            ],
            diagnostics: Vec::new(),
        }
    }

    fn target_binding() -> AtpTargetBinding {
        AtpTargetBinding::new(
            AtpFingerprint::new(18, b"target-vc-21".to_vec()).expect("fingerprint"),
            AtpSourceBinding::new("vc:21"),
        )
        .expect("target binding")
    }

    fn first_order_profile() -> crate::problem::LogicProfile {
        profile(
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::FirstOrder,
            SoftTypeStrategy::GuardPredicates,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
    }

    fn profile_with_native_property_support() -> crate::problem::LogicProfile {
        crate::problem::LogicProfile::try_new(
            "native-property-fixture",
            LogicFragment::Fof,
            EqualitySupport::Supported,
            QuantifierPolicy::PropositionalOnly,
            SoftTypeStrategy::GuardPredicates,
            NativePropertySupport::Supported,
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("profile")
    }

    fn profile(
        fragment: LogicFragment,
        equality: EqualitySupport,
        quantifiers: QuantifierPolicy,
        soft_types: SoftTypeStrategy,
        concrete_formats: BTreeSet<ConcreteFormat>,
    ) -> crate::problem::LogicProfile {
        crate::problem::LogicProfile::try_new(
            "tptp-fixture",
            fragment,
            equality,
            quantifiers,
            soft_types,
            NativePropertySupport::Unsupported,
            concrete_formats,
        )
        .expect("profile")
    }

    fn add_binder(parts: &mut AtpProblemParts, id: u32, symbol_name: &str, source: &str) {
        parts.declarations.push(declaration(
            id,
            AtpDeclarationKind::GeneratedBinder,
            symbol_name,
            0,
        ));
        parts.symbol_map.push(generated_binder(symbol_name, source));
    }

    fn add_function(
        parts: &mut AtpProblemParts,
        id: u32,
        symbol_name: &str,
        arity: u32,
        source: &str,
    ) {
        parts.declarations.push(declaration(
            id,
            AtpDeclarationKind::Function,
            symbol_name,
            arity,
        ));
        parts.symbol_map.push(symbol(symbol_name, source));
    }

    fn add_sort(parts: &mut AtpProblemParts, id: u32, symbol_name: &str, source: &str) {
        parts
            .declarations
            .push(declaration(id, AtpDeclarationKind::Sort, symbol_name, 0));
        parts.symbol_map.push(symbol(symbol_name, source));
    }

    fn declaration(id: u32, kind: AtpDeclarationKind, symbol: &str, arity: u32) -> AtpDeclaration {
        AtpDeclaration::new(
            crate::problem::AtpDeclarationId::new(id),
            kind,
            symbol,
            arity,
            AtpProvenanceId::new(1),
        )
    }

    fn symbol(symbol_name: &str, source: &str) -> crate::problem::AtpSymbolMapEntry {
        AtpSymbolMapEntry::new(
            symbol_name,
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new(source)),
        )
    }

    fn generated_binder(symbol_name: &str, source: &str) -> crate::problem::AtpSymbolMapEntry {
        AtpSymbolMapEntry::new(
            symbol_name,
            AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new(source)),
        )
    }

    fn provenance(id: u32, source: AtpSourceRef) -> AtpProvenance {
        AtpProvenance::new(
            AtpProvenanceId::new(id),
            source,
            AtpPayload::new(format!("payload:{id}")),
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

    fn function(name: &str, arguments: Vec<AtpTerm>) -> AtpTerm {
        AtpTerm::Function {
            function: AtpSymbolName::new(name),
            arguments,
        }
    }

    fn tptp_name_for<'a>(bindings: &'a [TptpSymbolBinding], symbol: &str) -> &'a str {
        bindings
            .iter()
            .find(|binding| binding.atp_symbol().as_str() == symbol)
            .map(TptpSymbolBinding::tptp_name)
            .expect("symbol binding exists")
    }
}
