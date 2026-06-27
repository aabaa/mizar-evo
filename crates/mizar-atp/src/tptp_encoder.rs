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
mod tests;
