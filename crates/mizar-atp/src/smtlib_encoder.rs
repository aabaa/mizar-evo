//! Deterministic SMT-LIB emission for backend-neutral ATP problems.
//!
//! This module implements the task-12 uninterpreted emitter specified in
//! [smtlib_encoder.md](../../../doc/design/mizar-atp/en/smtlib_encoder.md).
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

const UNIVERSE_SORT: &str = "mizar_universe";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[non_exhaustive]
pub enum SmtLibDialect {
    Uninterpreted,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SmtLibEncodingOutput {
    text: String,
    symbol_bindings: Vec<SmtLibSymbolBinding>,
    assertion_labels: Vec<SmtLibAssertionLabel>,
}

impl SmtLibEncodingOutput {
    pub fn text(&self) -> &str {
        &self.text
    }

    pub fn symbol_bindings(&self) -> &[SmtLibSymbolBinding] {
        &self.symbol_bindings
    }

    pub fn assertion_labels(&self) -> &[SmtLibAssertionLabel] {
        &self.assertion_labels
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SmtLibSymbolBinding {
    atp_symbol: AtpSymbolName,
    smtlib_symbol: String,
    source: AtpSymbolSource,
}

impl SmtLibSymbolBinding {
    pub const fn atp_symbol(&self) -> &AtpSymbolName {
        &self.atp_symbol
    }

    pub fn smtlib_symbol(&self) -> &str {
        &self.smtlib_symbol
    }

    pub const fn source(&self) -> &AtpSymbolSource {
        &self.source
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub struct SmtLibAssertionLabel {
    label: String,
    item: SmtLibAssertionItem,
    provenance: AtpProvenanceId,
    target_symbol: Option<AtpSymbolName>,
    negated: bool,
}

impl SmtLibAssertionLabel {
    pub fn label(&self) -> &str {
        &self.label
    }

    pub const fn item(&self) -> SmtLibAssertionItem {
        self.item
    }

    pub const fn provenance(&self) -> AtpProvenanceId {
        self.provenance
    }

    pub fn target_symbol(&self) -> Option<&AtpSymbolName> {
        self.target_symbol.as_ref()
    }

    pub const fn is_negated(&self) -> bool {
        self.negated
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
#[non_exhaustive]
pub enum SmtLibAssertionItem {
    Axiom(AtpFormulaId),
    TypeGuard(AtpTypeGuardId),
    Property(AtpPropertyId),
    NegatedConjecture(AtpFormulaId),
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SmtLibEncodingError {
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
    DuplicateSmtLibSymbol {
        symbol: String,
    },
    IllegalSmtLibSymbol {
        symbol: String,
    },
    ReservedSmtLibSymbol {
        symbol: String,
    },
    DuplicateAssertionLabel {
        label: String,
    },
}

impl fmt::Display for SmtLibEncodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnsupportedProfile { feature } => {
                write!(
                    formatter,
                    "unsupported SMT-LIB uninterpreted profile feature: {feature}"
                )
            }
            Self::MissingFormulaPayload { formula_id } => {
                write!(formatter, "missing formula payload for {formula_id:?}")
            }
            Self::EmptyFormulaList { operator } => {
                write!(
                    formatter,
                    "empty {operator} formula list is not SMT-LIB-encodable"
                )
            }
            Self::EmptyQuantifier { quantifier } => {
                write!(
                    formatter,
                    "empty {quantifier} binder list is not SMT-LIB-encodable"
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
                "SMT-LIB encoder does not support sorted binder {} : {}",
                variable.as_str(),
                sort.as_str()
            ),
            Self::FreeVariable { variable } => write!(
                formatter,
                "free variable {} is not SMT-LIB-encodable",
                variable.as_str()
            ),
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
                "native property declaration {property:?} is deferred for SMT-LIB"
            ),
            Self::DuplicateSmtLibSymbol { symbol } => {
                write!(formatter, "duplicate SMT-LIB symbol {symbol}")
            }
            Self::IllegalSmtLibSymbol { symbol } => {
                write!(formatter, "illegal SMT-LIB symbol {symbol}")
            }
            Self::ReservedSmtLibSymbol { symbol } => {
                write!(formatter, "reserved SMT-LIB symbol {symbol}")
            }
            Self::DuplicateAssertionLabel { label } => {
                write!(formatter, "duplicate SMT-LIB assertion label {label}")
            }
        }
    }
}

impl Error for SmtLibEncodingError {}

pub fn encode_smtlib(
    problem: &AtpProblem,
    dialect: SmtLibDialect,
) -> Result<SmtLibEncodingOutput, SmtLibEncodingError> {
    match dialect {
        SmtLibDialect::Uninterpreted => encode_uninterpreted(problem),
    }
}

fn encode_uninterpreted(problem: &AtpProblem) -> Result<SmtLibEncodingOutput, SmtLibEncodingError> {
    validate_uninterpreted_profile(problem)?;
    let mut context = EncodingContext::new(problem);
    context.register_fixed_sort()?;

    let mut text = String::new();
    let mut assertion_labels = Vec::new();

    writeln!(text, "(set-logic {})", context.logic()).expect("write string");
    writeln!(text, "(declare-sort {UNIVERSE_SORT} 0)").expect("write string");

    for declaration in problem
        .declarations()
        .iter()
        .filter(|declaration| declaration.kind() == AtpDeclarationKind::Function)
    {
        let name = context.mangle_symbol(
            declaration.symbol(),
            AtpDeclarationKind::Function,
            declaration.arity(),
            "function",
        )?;
        write_declaration(&mut text, &name, declaration.arity(), UNIVERSE_SORT);
    }

    for declaration in problem
        .declarations()
        .iter()
        .filter(|declaration| declaration.kind() == AtpDeclarationKind::Predicate)
    {
        let name = context.mangle_symbol(
            declaration.symbol(),
            AtpDeclarationKind::Predicate,
            declaration.arity(),
            "predicate",
        )?;
        write_declaration(&mut text, &name, declaration.arity(), "Bool");
    }

    for axiom in problem.axioms() {
        let label = context.register_assertion_label("ax", axiom.id())?;
        let formula = axiom
            .formula()
            .ok_or(SmtLibEncodingError::MissingFormulaPayload {
                formula_id: axiom.id(),
            })?;
        let rendered = render_formula(formula, &mut context, &mut Scope::default())?;
        write_assertion(&mut text, &label, &rendered);
        assertion_labels.push(SmtLibAssertionLabel {
            label,
            item: SmtLibAssertionItem::Axiom(axiom.id()),
            provenance: axiom.provenance(),
            target_symbol: None,
            negated: false,
        });
    }

    for guard in problem.type_context().guards() {
        let label = context.register_assertion_label("tg", guard.id())?;
        let rendered = render_formula(guard.formula(), &mut context, &mut Scope::default())?;
        write_assertion(&mut text, &label, &rendered);
        assertion_labels.push(SmtLibAssertionLabel {
            label,
            item: SmtLibAssertionItem::TypeGuard(guard.id()),
            provenance: guard.provenance(),
            target_symbol: None,
            negated: false,
        });
    }

    for property in problem.properties() {
        let label = context.register_assertion_label("prop", property.id())?;
        let PropertyEncoding::Axiom(formula) = property.encoding() else {
            return Err(SmtLibEncodingError::NativePropertyDeclaration {
                property: property.id(),
            });
        };
        let rendered = render_formula(formula, &mut context, &mut Scope::default())?;
        write_assertion(&mut text, &label, &rendered);
        assertion_labels.push(SmtLibAssertionLabel {
            label,
            item: SmtLibAssertionItem::Property(property.id()),
            provenance: property.provenance(),
            target_symbol: Some(property.target_symbol().clone()),
            negated: false,
        });
    }

    let conjecture = problem.conjecture();
    let label = context.register_assertion_label("neg_conj", conjecture.id())?;
    let formula = conjecture
        .formula()
        .ok_or(SmtLibEncodingError::MissingFormulaPayload {
            formula_id: conjecture.id(),
        })?;
    let rendered = render_formula(formula, &mut context, &mut Scope::default())?;
    write_assertion(&mut text, &label, &format!("(not {rendered})"));
    assertion_labels.push(SmtLibAssertionLabel {
        label,
        item: SmtLibAssertionItem::NegatedConjecture(conjecture.id()),
        provenance: conjecture.provenance(),
        target_symbol: None,
        negated: true,
    });

    writeln!(text, "(check-sat)").expect("write string");

    Ok(SmtLibEncodingOutput {
        text,
        symbol_bindings: context.symbol_bindings.into_iter().collect(),
        assertion_labels,
    })
}

fn validate_uninterpreted_profile(problem: &AtpProblem) -> Result<(), SmtLibEncodingError> {
    let profile = problem.logic_profile();
    if !profile.concrete_formats().contains(&ConcreteFormat::SmtLib) {
        return Err(SmtLibEncodingError::UnsupportedProfile {
            feature: "SMT-LIB concrete format",
        });
    }
    if profile.fragment() != LogicFragment::SmtLibUninterpreted {
        return Err(SmtLibEncodingError::UnsupportedProfile {
            feature: "non-SMT-LIB-uninterpreted logic fragment",
        });
    }
    if problem.expected_result() != ExpectedBackendResult::Unsat {
        return Err(SmtLibEncodingError::UnsupportedProfile {
            feature: "non-Unsat expected result",
        });
    }
    if profile.soft_types() != SoftTypeStrategy::GuardPredicates {
        return Err(SmtLibEncodingError::UnsupportedProfile {
            feature: "non-guard-predicate soft type strategy",
        });
    }
    Ok(())
}

fn write_declaration(output: &mut String, name: &str, arity: u32, result: &str) {
    let arguments = std::iter::repeat_n(UNIVERSE_SORT, arity as usize)
        .collect::<Vec<_>>()
        .join(" ");
    writeln!(output, "(declare-fun {name} ({arguments}) {result})").expect("write string");
}

fn write_assertion(output: &mut String, label: &str, formula: &str) {
    writeln!(output, "(assert (! {formula} :named {label}))").expect("write string");
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
    symbol_bindings: BTreeSet<SmtLibSymbolBinding>,
    assertion_labels: BTreeSet<String>,
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
            assertion_labels: BTreeSet::new(),
        }
    }

    fn logic(&self) -> &'static str {
        match self.profile_quantifiers {
            QuantifierPolicy::PropositionalOnly => "QF_UF",
            QuantifierPolicy::FirstOrder => "UF",
        }
    }

    fn register_fixed_sort(&mut self) -> Result<(), SmtLibEncodingError> {
        self.register_name(UNIVERSE_SORT.to_owned(), "fixed-sort".to_owned())
    }

    fn register_assertion_label(
        &mut self,
        prefix: &'static str,
        id: impl AssertionLabelId,
    ) -> Result<String, SmtLibEncodingError> {
        let id = id.index();
        let label = format!("{prefix}_{id}");
        validate_smtlib_symbol(&label)?;
        if !self.assertion_labels.insert(label.clone()) {
            return Err(SmtLibEncodingError::DuplicateAssertionLabel { label });
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
    ) -> Result<String, SmtLibEncodingError> {
        let (declaration, source) =
            self.symbol_signature(symbol, expected_kind, expected_arity, expected)?;
        if declaration.kind() == AtpDeclarationKind::GeneratedBinder {
            return Err(SmtLibEncodingError::InvalidDeclaration {
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
        self.symbol_bindings.insert(SmtLibSymbolBinding {
            atp_symbol: symbol.clone(),
            smtlib_symbol: name.clone(),
            source,
        });
        Ok(name)
    }

    fn mangle_binder(
        &mut self,
        binder: &AtpSymbolName,
        position: usize,
    ) -> Result<String, SmtLibEncodingError> {
        let (declaration, source) = self.symbol_signature(
            binder,
            AtpDeclarationKind::GeneratedBinder,
            0,
            "generated binder",
        )?;
        if !matches!(source, AtpSymbolSource::GeneratedBinder(_)) {
            return Err(SmtLibEncodingError::InvalidBinderSource {
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
        let name = format!("v_{}", hex(key.as_bytes()));
        self.register_name(name.clone(), format!("binder:{key}"))?;
        self.symbol_bindings.insert(SmtLibSymbolBinding {
            atp_symbol: binder.clone(),
            smtlib_symbol: name.clone(),
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
    ) -> Result<(AtpDeclaration, AtpSymbolSource), SmtLibEncodingError> {
        let declaration = self.declarations.get(symbol).cloned().ok_or_else(|| {
            SmtLibEncodingError::MissingDeclaration {
                symbol: symbol.clone(),
            }
        })?;
        let source = self.symbol_sources.get(symbol).cloned().ok_or_else(|| {
            SmtLibEncodingError::MissingSymbolMap {
                symbol: symbol.clone(),
            }
        })?;
        if declaration.kind() != expected_kind {
            return Err(SmtLibEncodingError::InvalidDeclaration {
                symbol: symbol.clone(),
                expected,
                actual: declaration.kind(),
            });
        }
        if declaration.arity() != expected_arity {
            return Err(SmtLibEncodingError::InvalidArity {
                symbol: symbol.clone(),
                expected: expected_arity,
                actual: declaration.arity(),
            });
        }
        Ok((declaration, source))
    }

    fn register_name(&mut self, name: String, owner: String) -> Result<(), SmtLibEncodingError> {
        validate_smtlib_symbol(&name)?;
        if let Some(existing_owner) = self.name_owners.get(&name) {
            if existing_owner != &owner {
                return Err(SmtLibEncodingError::DuplicateSmtLibSymbol { symbol: name });
            }
            return Ok(());
        }
        self.name_owners.insert(name, owner);
        Ok(())
    }
}

trait AssertionLabelId {
    fn index(self) -> u32;
}

impl AssertionLabelId for AtpFormulaId {
    fn index(self) -> u32 {
        self.index()
    }
}

impl AssertionLabelId for AtpPropertyId {
    fn index(self) -> u32 {
        self.index()
    }
}

impl AssertionLabelId for AtpTypeGuardId {
    fn index(self) -> u32 {
        self.index()
    }
}

fn render_formula(
    formula: &AtpFormulaTree,
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, SmtLibEncodingError> {
    match formula {
        AtpFormulaTree::True => Ok("true".to_owned()),
        AtpFormulaTree::False => Ok("false".to_owned()),
        AtpFormulaTree::Atom(atom) => render_atom(atom, context, scope),
        AtpFormulaTree::Equality { left, right } => {
            if context.profile_equality != EqualitySupport::Supported {
                return Err(SmtLibEncodingError::UnsupportedProfile {
                    feature: "equality",
                });
            }
            Ok(format!(
                "(= {} {})",
                render_term(left, context, scope)?,
                render_term(right, context, scope)?
            ))
        }
        AtpFormulaTree::Not(inner) => {
            Ok(format!("(not {})", render_formula(inner, context, scope)?))
        }
        AtpFormulaTree::And(formulas) => render_nary("and", "and", formulas, context, scope),
        AtpFormulaTree::Or(formulas) => render_nary("or", "or", formulas, context, scope),
        AtpFormulaTree::Implies(left, right) => Ok(format!(
            "(=> {} {})",
            render_formula(left, context, scope)?,
            render_formula(right, context, scope)?
        )),
        AtpFormulaTree::Forall { binders, body } => {
            render_quantifier("forall", "forall", binders, body, context, scope)
        }
        AtpFormulaTree::Exists { binders, body } => {
            render_quantifier("exists", "exists", binders, body, context, scope)
        }
    }
}

fn render_atom(
    atom: &crate::problem::AtpAtom,
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, SmtLibEncodingError> {
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
        .join(" ");
    Ok(format!("({name} {arguments})"))
}

fn render_nary(
    operator_name: &'static str,
    operator: &str,
    formulas: &[AtpFormulaTree],
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, SmtLibEncodingError> {
    if formulas.is_empty() {
        return Err(SmtLibEncodingError::EmptyFormulaList {
            operator: operator_name,
        });
    }
    let rendered = formulas
        .iter()
        .map(|formula| render_formula(formula, context, scope))
        .collect::<Result<Vec<_>, _>>()?;
    if rendered.len() == 1 {
        return Ok(rendered[0].clone());
    }
    Ok(format!("({operator} {})", rendered.join(" ")))
}

fn render_quantifier(
    quantifier_name: &'static str,
    operator: &str,
    binders: &[crate::problem::AtpBinder],
    body: &AtpFormulaTree,
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, SmtLibEncodingError> {
    if context.profile_quantifiers != QuantifierPolicy::FirstOrder {
        return Err(SmtLibEncodingError::UnsupportedProfile {
            feature: "quantifier",
        });
    }
    if binders.is_empty() {
        return Err(SmtLibEncodingError::EmptyQuantifier {
            quantifier: quantifier_name,
        });
    }

    let mut local = BTreeSet::new();
    let mut inserted = Vec::new();
    let mut names = Vec::new();
    for (position, binder) in binders.iter().enumerate() {
        if let Some(sort) = binder.sort() {
            return Err(SmtLibEncodingError::SortedBinder {
                variable: binder.variable().clone(),
                sort: sort.clone(),
            });
        }
        if !local.insert(binder.variable().clone()) {
            return Err(SmtLibEncodingError::DuplicateBinder {
                variable: binder.variable().clone(),
            });
        }
        if scope.variables.contains_key(binder.variable()) {
            return Err(SmtLibEncodingError::BinderShadowing {
                variable: binder.variable().clone(),
            });
        }
        let name = context.mangle_binder(binder.variable(), position)?;
        scope
            .variables
            .insert(binder.variable().clone(), name.clone());
        inserted.push(binder.variable().clone());
        names.push(format!("({name} {UNIVERSE_SORT})"));
    }

    let body = render_formula(body, context, scope);
    for variable in inserted {
        scope.variables.remove(&variable);
    }
    let body = body?;

    Ok(format!("({operator} ({}) {body})", names.join(" ")))
}

fn render_term(
    term: &AtpTerm,
    context: &mut EncodingContext,
    scope: &mut Scope,
) -> Result<String, SmtLibEncodingError> {
    match term {
        AtpTerm::Variable(variable) => scope.variables.get(variable).cloned().ok_or_else(|| {
            SmtLibEncodingError::FreeVariable {
                variable: variable.clone(),
            }
        }),
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
                .join(" ");
            Ok(format!("({name} {arguments})"))
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

fn validate_smtlib_symbol(symbol: &str) -> Result<(), SmtLibEncodingError> {
    let mut characters = symbol.chars();
    let Some(first) = characters.next() else {
        return Err(SmtLibEncodingError::IllegalSmtLibSymbol {
            symbol: symbol.to_owned(),
        });
    };
    let valid_first = first.is_ascii_alphabetic();
    let valid_rest =
        characters.all(|character| character == '_' || character.is_ascii_alphanumeric());
    if !valid_first || !valid_rest {
        return Err(SmtLibEncodingError::IllegalSmtLibSymbol {
            symbol: symbol.to_owned(),
        });
    }
    if reserved_smtlib_symbol(symbol) {
        return Err(SmtLibEncodingError::ReservedSmtLibSymbol {
            symbol: symbol.to_owned(),
        });
    }
    Ok(())
}

fn reserved_smtlib_symbol(symbol: &str) -> bool {
    matches!(
        symbol,
        "Bool"
            | "true"
            | "false"
            | "not"
            | "and"
            | "or"
            | "forall"
            | "exists"
            | "assert"
            | "check_sat"
            | "declare_fun"
            | "declare_sort"
            | "set_logic"
            | "let"
            | "match"
            | "par"
    )
}

#[cfg(test)]
mod tests;
