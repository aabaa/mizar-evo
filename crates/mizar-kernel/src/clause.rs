use std::{cmp::Ordering, collections::BTreeSet};

const CLAUSE_DOMAIN_SEPARATOR: &[u8] = b"MIZAR_KERNEL_CLAUSE\0";
const TERM_SCHEMA_TAG: u8 = 1;
const ATOM_SCHEMA_TAG: u8 = 2;
const LITERAL_SCHEMA_TAG: u8 = 3;
const CLAUSE_SCHEMA_TAG: u8 = 4;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClauseProfile {
    pub schema_version: u16,
    pub encoding_version: u16,
    pub tautology_policy: TautologyPolicy,
}

impl ClauseProfile {
    #[must_use]
    pub const fn new(
        schema_version: u16,
        encoding_version: u16,
        tautology_policy: TautologyPolicy,
    ) -> Self {
        Self {
            schema_version,
            encoding_version,
            tautology_policy,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum TautologyPolicy {
    Reject,
    Marker,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ClauseValidationContext {
    pub profile: ClauseProfile,
    pub allowed_symbol_kinds: BTreeSet<SymbolKind>,
    pub known_symbol_ids: BTreeSet<SymbolKey>,
    pub canonical_variable_ids: BTreeSet<VariableId>,
    pub max_literals: usize,
    pub max_term_encoding_bytes: usize,
}

impl ClauseValidationContext {
    #[must_use]
    pub fn new(profile: ClauseProfile) -> Self {
        Self {
            profile,
            allowed_symbol_kinds: BTreeSet::new(),
            known_symbol_ids: BTreeSet::new(),
            canonical_variable_ids: BTreeSet::new(),
            max_literals: usize::MAX,
            max_term_encoding_bytes: usize::MAX,
        }
    }

    #[must_use]
    pub fn with_allowed_symbol_kind(mut self, kind: SymbolKind) -> Self {
        self.allowed_symbol_kinds.insert(kind);
        self
    }

    #[must_use]
    pub fn with_known_symbol(mut self, symbol: SymbolKey) -> Self {
        self.allowed_symbol_kinds.insert(symbol.kind);
        self.known_symbol_ids.insert(symbol);
        self
    }

    #[must_use]
    pub fn with_canonical_variable(mut self, variable: VariableId) -> Self {
        self.canonical_variable_ids.insert(variable);
        self
    }

    #[must_use]
    pub const fn with_limits(
        mut self,
        max_literals: usize,
        max_term_encoding_bytes: usize,
    ) -> Self {
        self.max_literals = max_literals;
        self.max_term_encoding_bytes = max_term_encoding_bytes;
        self
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Clause {
    profile: ClauseProfile,
    form: ClauseForm,
    literals: Vec<Literal>,
}

impl Clause {
    pub fn normalize(
        raw_literals: Vec<Literal>,
        context: &ClauseValidationContext,
    ) -> Result<Self, ClauseError> {
        validate_literal_count(raw_literals.len(), context)?;
        let mut literals = Vec::with_capacity(raw_literals.len());
        for literal in raw_literals {
            literal.validate(context)?;
            literals.push(literal);
        }

        literals.sort();
        literals.dedup();

        if literals.is_empty() {
            return Ok(Self {
                profile: context.profile.clone(),
                form: ClauseForm::Empty,
                literals,
            });
        }

        if contains_opposite_polarity_atom(&literals) {
            return match context.profile.tautology_policy {
                TautologyPolicy::Reject => Err(ClauseError::DisallowedTautology),
                TautologyPolicy::Marker => Ok(Self {
                    profile: context.profile.clone(),
                    form: ClauseForm::Tautology,
                    literals: Vec::new(),
                }),
            };
        }

        Ok(Self {
            profile: context.profile.clone(),
            form: ClauseForm::Ordinary,
            literals,
        })
    }

    pub fn from_canonical_parts(
        form: ClauseForm,
        literals: Vec<Literal>,
        context: &ClauseValidationContext,
    ) -> Result<Self, ClauseError> {
        match form {
            ClauseForm::Ordinary if literals.is_empty() => {
                return Err(ClauseError::OrdinaryEmptyPayload);
            }
            ClauseForm::Empty | ClauseForm::Tautology if !literals.is_empty() => {
                return Err(ClauseError::NonEmptyMarkerPayload { form });
            }
            ClauseForm::Tautology
                if context.profile.tautology_policy != TautologyPolicy::Marker =>
            {
                return Err(ClauseError::DisallowedTautology);
            }
            ClauseForm::Ordinary | ClauseForm::Empty | ClauseForm::Tautology => {}
        }

        validate_literal_count(literals.len(), context)?;
        for literal in &literals {
            literal.validate(context)?;
        }
        if has_duplicate_literals(&literals)? {
            return Err(ClauseError::DuplicateLiteral);
        }
        if !literals.windows(2).all(|window| window[0] < window[1]) {
            return Err(ClauseError::NonCanonicalLiteralOrder);
        }
        if form == ClauseForm::Ordinary && contains_opposite_polarity_atom(&literals) {
            return Err(ClauseError::DisallowedTautology);
        }

        Ok(Self {
            profile: context.profile.clone(),
            form,
            literals,
        })
    }

    #[must_use]
    pub const fn profile(&self) -> &ClauseProfile {
        &self.profile
    }

    #[must_use]
    pub const fn form(&self) -> ClauseForm {
        self.form
    }

    #[must_use]
    pub fn literals(&self) -> &[Literal] {
        &self.literals
    }

    #[must_use]
    pub fn render(&self) -> String {
        let mut rendered = format!(
            "clause(schema={},encoding={},tautology={:?},form={:?},literals={})",
            self.profile.schema_version,
            self.profile.encoding_version,
            self.profile.tautology_policy,
            self.form,
            self.literals.len()
        );
        for literal in &self.literals {
            rendered.push(' ');
            rendered.push_str(&literal.render());
        }
        rendered
    }

    #[must_use = "canonical hash input must be consumed or checked"]
    pub fn canonical_hash_input(&self) -> Result<Vec<u8>, ClauseError> {
        let payload_len = self.hash_payload_len()?;
        let mut payload = Vec::with_capacity(payload_len);
        payload.extend_from_slice(&self.profile.schema_version.to_be_bytes());
        payload.extend_from_slice(&self.profile.encoding_version.to_be_bytes());
        payload.push(self.profile.tautology_policy.tag());
        payload.push(self.form.tag());
        payload.extend_from_slice(&checked_literal_count(self.literals.len())?.to_be_bytes());
        for literal in &self.literals {
            payload.extend_from_slice(&literal.canonical_bytes()?);
        }

        let mut bytes = Vec::from(CLAUSE_DOMAIN_SEPARATOR);
        bytes.extend(frame(CLAUSE_SCHEMA_TAG, self.form.tag(), payload)?);
        Ok(bytes)
    }

    fn hash_payload_len(&self) -> Result<usize, ClauseError> {
        checked_literal_count(self.literals.len())?;
        let payload_len = checked_add_len(10, literal_encoding_len(&self.literals)?)?;
        checked_payload_len(payload_len)?;
        Ok(payload_len)
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ClauseForm {
    Ordinary,
    Empty,
    Tautology,
}

impl ClauseForm {
    const fn tag(self) -> u8 {
        match self {
            Self::Ordinary => 1,
            Self::Empty => 2,
            Self::Tautology => 3,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Literal {
    pub polarity: Polarity,
    pub atom: Atom,
}

impl Literal {
    #[must_use]
    pub const fn new(polarity: Polarity, atom: Atom) -> Self {
        Self { polarity, atom }
    }

    fn validate(&self, context: &ClauseValidationContext) -> Result<(), ClauseError> {
        self.atom.validate(context)
    }

    #[must_use]
    pub fn render(&self) -> String {
        let sign = match self.polarity {
            Polarity::Negative => "-",
            Polarity::Positive => "+",
        };
        format!("{sign}{}", self.atom.render())
    }

    #[must_use = "canonical bytes must be consumed or checked"]
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ClauseError> {
        let payload_len = checked_add_len(1, self.atom.canonical_len()?)?;
        checked_payload_len(payload_len)?;

        let mut payload = Vec::with_capacity(payload_len);
        payload.push(self.polarity.tag());
        payload.extend(self.atom.canonical_bytes()?);
        frame(LITERAL_SCHEMA_TAG, self.polarity.tag(), payload)
    }

    fn canonical_len(&self) -> Result<usize, ClauseError> {
        frame_len(checked_add_len(1, self.atom.canonical_len()?)?)
    }
}

impl Ord for Literal {
    fn cmp(&self, other: &Self) -> Ordering {
        self.polarity
            .rank()
            .cmp(&other.polarity.rank())
            .then_with(|| self.atom.cmp(&other.atom))
            .then_with(|| canonical_sort_bytes(self).cmp(&canonical_sort_bytes(other)))
    }
}

impl PartialOrd for Literal {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Polarity {
    Negative,
    Positive,
}

impl Polarity {
    const fn rank(self) -> u8 {
        match self {
            Self::Negative => 0,
            Self::Positive => 1,
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Negative => 1,
            Self::Positive => 2,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Atom {
    pub symbol: SymbolKey,
    pub arity: u32,
    pub arguments: Vec<Term>,
}

impl Atom {
    #[must_use]
    pub fn new(symbol: SymbolKey, arguments: Vec<Term>) -> Self {
        let arity = u32::try_from(arguments.len()).unwrap_or(u32::MAX);
        Self {
            symbol,
            arity,
            arguments,
        }
    }

    #[must_use]
    pub const fn with_arity(symbol: SymbolKey, arity: u32, arguments: Vec<Term>) -> Self {
        Self {
            symbol,
            arity,
            arguments,
        }
    }

    fn validate(&self, context: &ClauseValidationContext) -> Result<(), ClauseError> {
        validate_symbol(self.symbol, context)?;
        let actual = checked_term_count(
            self.arguments.len(),
            argument_encoding_len(&self.arguments)?,
        )?;
        if self.arity != actual {
            return Err(ClauseError::ArityMismatch {
                symbol: self.symbol,
                expected: self.arity,
                actual,
            });
        }
        for argument in &self.arguments {
            argument.validate(context)?;
        }
        Ok(())
    }

    #[must_use]
    pub fn render(&self) -> String {
        let mut rendered = format!(
            "{:?}#{}[arity={}](",
            self.symbol.kind, self.symbol.id.0, self.arity
        );
        for (index, argument) in self.arguments.iter().enumerate() {
            if index > 0 {
                rendered.push(',');
            }
            rendered.push_str(&argument.render());
        }
        rendered.push(')');
        rendered
    }

    #[must_use = "canonical bytes must be consumed or checked"]
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ClauseError> {
        let _ = self.canonical_len()?;
        let arguments_len = argument_encoding_len(&self.arguments)?;
        let payload_len = checked_add_len(self.symbol.canonical_len(), 8)?;
        let payload_len = checked_add_len(payload_len, arguments_len)?;
        checked_payload_len(payload_len)?;

        let mut payload = Vec::with_capacity(payload_len);
        payload.extend(self.symbol.canonical_bytes());
        payload.extend_from_slice(&self.arity.to_be_bytes());
        payload.extend_from_slice(
            &checked_term_count(self.arguments.len(), arguments_len)?.to_be_bytes(),
        );
        for argument in &self.arguments {
            payload.extend(argument.canonical_bytes()?);
        }
        frame(ATOM_SCHEMA_TAG, self.symbol.kind.tag(), payload)
    }

    fn canonical_len(&self) -> Result<usize, ClauseError> {
        let payload_len = checked_add_len(self.symbol.canonical_len(), 8)?;
        frame_len(checked_add_len(
            payload_len,
            argument_encoding_len(&self.arguments)?,
        )?)
    }
}

impl Ord for Atom {
    fn cmp(&self, other: &Self) -> Ordering {
        self.symbol
            .kind
            .rank()
            .cmp(&other.symbol.kind.rank())
            .then_with(|| self.symbol.id.cmp(&other.symbol.id))
            .then_with(|| self.arity.cmp(&other.arity))
            .then_with(|| argument_bytes(&self.arguments).cmp(&argument_bytes(&other.arguments)))
            .then_with(|| canonical_sort_bytes(self).cmp(&canonical_sort_bytes(other)))
    }
}

impl PartialOrd for Atom {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum Term {
    Variable(VariableId),
    Application {
        symbol: SymbolKey,
        arguments: Vec<Term>,
    },
    BinderNormalized {
        binder_id: u32,
        body: Box<Term>,
    },
    Malformed,
}

impl Term {
    fn validate(&self, context: &ClauseValidationContext) -> Result<(), ClauseError> {
        match self {
            Self::Variable(variable) => {
                if !context.canonical_variable_ids.contains(variable) {
                    return Err(ClauseError::NoncanonicalVariableId(*variable));
                }
            }
            Self::Application { symbol, arguments } => {
                validate_symbol(*symbol, context)?;
                for argument in arguments {
                    argument.validate(context)?;
                }
            }
            Self::BinderNormalized { body, .. } => body.validate(context)?,
            Self::Malformed => return Err(ClauseError::MalformedTermEncoding),
        }
        let size = self.canonical_len()?;
        if size > context.max_term_encoding_bytes {
            return Err(ClauseError::TermSizeExceeded {
                max: context.max_term_encoding_bytes,
                actual: size,
            });
        }
        Ok(())
    }

    #[must_use]
    pub fn render(&self) -> String {
        match self {
            Self::Variable(variable) => format!("v{}", variable.0),
            Self::Application { symbol, arguments } => {
                let mut rendered = format!("{:?}#{}(", symbol.kind, symbol.id.0);
                for (index, argument) in arguments.iter().enumerate() {
                    if index > 0 {
                        rendered.push(',');
                    }
                    rendered.push_str(&argument.render());
                }
                rendered.push(')');
                rendered
            }
            Self::BinderNormalized { binder_id, body } => {
                format!("binder{binder_id}({})", body.render())
            }
            Self::Malformed => "malformed".to_owned(),
        }
    }

    #[must_use = "canonical bytes must be consumed or checked"]
    pub fn canonical_bytes(&self) -> Result<Vec<u8>, ClauseError> {
        let _ = self.canonical_len()?;
        match self {
            Self::Variable(variable) => frame(TERM_SCHEMA_TAG, 1, variable.0.to_be_bytes()),
            Self::Application { symbol, arguments } => {
                let arguments_len = argument_encoding_len(arguments)?;
                let payload_len = checked_add_len(symbol.canonical_len(), 4)?;
                let payload_len = checked_add_len(payload_len, arguments_len)?;
                checked_payload_len(payload_len)?;

                let mut payload = Vec::with_capacity(payload_len);
                payload.extend(symbol.canonical_bytes());
                payload.extend_from_slice(
                    &checked_term_count(arguments.len(), arguments_len)?.to_be_bytes(),
                );
                for argument in arguments {
                    payload.extend(argument.canonical_bytes()?);
                }
                frame(TERM_SCHEMA_TAG, 2, payload)
            }
            Self::BinderNormalized { binder_id, body } => {
                let mut payload = binder_id.to_be_bytes().to_vec();
                payload.extend(body.canonical_bytes()?);
                frame(TERM_SCHEMA_TAG, 3, payload)
            }
            Self::Malformed => frame(TERM_SCHEMA_TAG, 255, []),
        }
    }

    fn canonical_len(&self) -> Result<usize, ClauseError> {
        match self {
            Self::Variable(_) => frame_len(4),
            Self::Application { symbol, arguments } => {
                let payload_len = checked_add_len(symbol.canonical_len(), 4)?;
                frame_len(checked_add_len(
                    payload_len,
                    argument_encoding_len(arguments)?,
                )?)
            }
            Self::BinderNormalized { body, .. } => {
                frame_len(checked_add_len(4, body.canonical_len()?)?)
            }
            Self::Malformed => frame_len(0),
        }
    }
}

impl Ord for Term {
    fn cmp(&self, other: &Self) -> Ordering {
        canonical_sort_bytes(self).cmp(&canonical_sort_bytes(other))
    }
}

impl PartialOrd for Term {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SymbolKey {
    pub kind: SymbolKind,
    pub id: SymbolId,
}

impl SymbolKey {
    #[must_use]
    pub const fn new(kind: SymbolKind, id: u32) -> Self {
        Self {
            kind,
            id: SymbolId(id),
        }
    }

    fn canonical_bytes(self) -> Vec<u8> {
        let mut bytes = vec![self.kind.tag()];
        bytes.extend_from_slice(&self.id.0.to_be_bytes());
        bytes
    }

    const fn canonical_len(self) -> usize {
        5
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct SymbolId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct VariableId(pub u32);

#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
#[non_exhaustive]
pub enum SymbolKind {
    Predicate,
    FunctorPredicate,
    Equality,
    BuiltinRelation,
}

impl SymbolKind {
    const fn rank(self) -> u8 {
        match self {
            Self::Predicate => 0,
            Self::FunctorPredicate => 1,
            Self::Equality => 2,
            Self::BuiltinRelation => 3,
        }
    }

    const fn tag(self) -> u8 {
        match self {
            Self::Predicate => 1,
            Self::FunctorPredicate => 2,
            Self::Equality => 3,
            Self::BuiltinRelation => 4,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
#[non_exhaustive]
pub enum ClauseError {
    MissingSymbol(SymbolKey),
    UnsupportedSymbolKind(SymbolKind),
    ArityMismatch {
        symbol: SymbolKey,
        expected: u32,
        actual: u32,
    },
    MalformedTermEncoding,
    NoncanonicalVariableId(VariableId),
    DuplicateLiteral,
    NonCanonicalLiteralOrder,
    OrdinaryEmptyPayload,
    NonEmptyMarkerPayload {
        form: ClauseForm,
    },
    DisallowedTautology,
    LiteralCountExceeded {
        max: usize,
        actual: usize,
    },
    TermSizeExceeded {
        max: usize,
        actual: usize,
    },
}

fn validate_literal_count(
    literal_count: usize,
    context: &ClauseValidationContext,
) -> Result<(), ClauseError> {
    if literal_count > context.max_literals {
        return Err(ClauseError::LiteralCountExceeded {
            max: context.max_literals,
            actual: literal_count,
        });
    }
    Ok(())
}

fn validate_symbol(
    symbol: SymbolKey,
    context: &ClauseValidationContext,
) -> Result<(), ClauseError> {
    if !context.allowed_symbol_kinds.contains(&symbol.kind) {
        return Err(ClauseError::UnsupportedSymbolKind(symbol.kind));
    }
    if !context.known_symbol_ids.contains(&symbol) {
        return Err(ClauseError::MissingSymbol(symbol));
    }
    Ok(())
}

fn contains_opposite_polarity_atom(literals: &[Literal]) -> bool {
    for (index, left) in literals.iter().enumerate() {
        for right in &literals[index + 1..] {
            if left.atom == right.atom && left.polarity != right.polarity {
                return true;
            }
        }
    }
    false
}

fn has_duplicate_literals(literals: &[Literal]) -> Result<bool, ClauseError> {
    let mut seen = BTreeSet::new();
    for literal in literals {
        if !seen.insert(literal.canonical_bytes()?) {
            return Ok(true);
        }
    }
    Ok(false)
}

fn checked_literal_count(literal_count: usize) -> Result<u32, ClauseError> {
    u32::try_from(literal_count).map_err(|_| ClauseError::LiteralCountExceeded {
        max: u32_max_usize(),
        actual: literal_count,
    })
}

fn checked_term_count(term_count: usize, encoded_bytes: usize) -> Result<u32, ClauseError> {
    u32::try_from(term_count).map_err(|_| ClauseError::TermSizeExceeded {
        max: u32_max_usize(),
        actual: encoded_bytes,
    })
}

fn checked_payload_len(payload_len: usize) -> Result<u32, ClauseError> {
    u32::try_from(payload_len).map_err(|_| ClauseError::TermSizeExceeded {
        max: u32_max_usize(),
        actual: payload_len,
    })
}

fn checked_add_len(left: usize, right: usize) -> Result<usize, ClauseError> {
    left.checked_add(right)
        .ok_or(ClauseError::TermSizeExceeded {
            max: u32_max_usize(),
            actual: usize::MAX,
        })
}

fn frame_len(payload_len: usize) -> Result<usize, ClauseError> {
    checked_payload_len(payload_len)?;
    checked_add_len(6, payload_len)
}

fn u32_max_usize() -> usize {
    usize::try_from(u32::MAX).expect("u32::MAX fits into usize on supported targets")
}

trait CanonicalSortable {
    fn canonical_bytes_for_sort(&self) -> Result<Vec<u8>, ClauseError>;
    fn fallback_sort_bytes(&self) -> Vec<u8>;
}

impl CanonicalSortable for Literal {
    fn canonical_bytes_for_sort(&self) -> Result<Vec<u8>, ClauseError> {
        self.canonical_bytes()
    }

    fn fallback_sort_bytes(&self) -> Vec<u8> {
        self.render().into_bytes()
    }
}

impl CanonicalSortable for Atom {
    fn canonical_bytes_for_sort(&self) -> Result<Vec<u8>, ClauseError> {
        self.canonical_bytes()
    }

    fn fallback_sort_bytes(&self) -> Vec<u8> {
        self.render().into_bytes()
    }
}

impl CanonicalSortable for Term {
    fn canonical_bytes_for_sort(&self) -> Result<Vec<u8>, ClauseError> {
        self.canonical_bytes()
    }

    fn fallback_sort_bytes(&self) -> Vec<u8> {
        self.render().into_bytes()
    }
}

fn canonical_sort_bytes(value: &impl CanonicalSortable) -> Vec<u8> {
    value
        .canonical_bytes_for_sort()
        .unwrap_or_else(|_| value.fallback_sort_bytes())
}

fn literal_encoding_len(literals: &[Literal]) -> Result<usize, ClauseError> {
    literals.iter().try_fold(0usize, |total, literal| {
        checked_add_len(total, literal.canonical_len()?)
    })
}

fn argument_encoding_len(arguments: &[Term]) -> Result<usize, ClauseError> {
    arguments.iter().try_fold(0usize, |total, argument| {
        checked_add_len(total, argument.canonical_len()?)
    })
}

fn argument_bytes(arguments: &[Term]) -> Vec<u8> {
    let mut bytes = Vec::new();
    for argument in arguments {
        bytes.extend(canonical_sort_bytes(argument));
    }
    bytes
}

fn frame<I>(schema_tag: u8, form_or_term_tag: u8, payload: I) -> Result<Vec<u8>, ClauseError>
where
    I: AsRef<[u8]>,
{
    let payload = payload.as_ref();
    let total_len = frame_len(payload.len())?;
    let mut bytes = Vec::with_capacity(total_len);
    bytes.push(schema_tag);
    bytes.push(form_or_term_tag);
    bytes.extend_from_slice(&checked_payload_len(payload.len())?.to_be_bytes());
    bytes.extend_from_slice(payload);
    Ok(bytes)
}

impl TautologyPolicy {
    const fn tag(self) -> u8 {
        match self {
            Self::Reject => 1,
            Self::Marker => 2,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn normalizes_valid_clauses_with_deterministic_order() {
        let context = sample_context(TautologyPolicy::Reject);
        let predicate_two = SymbolKey::new(SymbolKind::Predicate, 2);
        let predicate_one = SymbolKey::new(SymbolKind::Predicate, 1);
        let equality_one = SymbolKey::new(SymbolKind::Equality, 1);
        let raw = vec![
            lit(
                Polarity::Positive,
                predicate_two,
                vec![Term::Variable(VariableId(2))],
            ),
            lit(Polarity::Negative, equality_one, vec![]),
            lit(
                Polarity::Positive,
                predicate_one,
                vec![Term::Variable(VariableId(1))],
            ),
        ];

        let clause = Clause::normalize(raw, &context).expect("valid clause");

        assert_eq!(clause.form(), ClauseForm::Ordinary);
        assert_eq!(
            clause
                .literals()
                .iter()
                .map(Literal::render)
                .collect::<Vec<_>>(),
            [
                "-Equality#1[arity=0]()",
                "+Predicate#1[arity=1](v1)",
                "+Predicate#2[arity=1](v2)",
            ]
        );
    }

    #[test]
    fn normalizes_valid_single_literal_clause() {
        let context = sample_context(TautologyPolicy::Reject);
        let literal = lit(
            Polarity::Positive,
            SymbolKey::new(SymbolKind::Predicate, 1),
            vec![Term::Variable(VariableId(1))],
        );

        let clause = Clause::normalize(vec![literal.clone()], &context).expect("valid clause");

        assert_eq!(clause.form(), ClauseForm::Ordinary);
        assert_eq!(clause.literals(), std::slice::from_ref(&literal));
        assert!(contains_subsequence(
            &hash_input(&clause),
            &literal.canonical_bytes().expect("valid literal bytes")
        ));
    }

    #[test]
    fn symbol_kind_precedence_is_ordered_before_symbol_id() {
        let context = sample_context(TautologyPolicy::Reject);
        let predicate_high = SymbolKey::new(SymbolKind::Predicate, 9);
        let equality_low = SymbolKey::new(SymbolKind::Equality, 1);

        let clause = Clause::normalize(
            vec![
                lit(Polarity::Positive, equality_low, vec![]),
                lit(Polarity::Positive, predicate_high, vec![]),
            ],
            &context,
        )
        .expect("valid clause");

        assert_eq!(
            clause
                .literals()
                .iter()
                .map(Literal::render)
                .collect::<Vec<_>>(),
            ["+Predicate#9[arity=0]()", "+Equality#1[arity=0]()",]
        );
    }

    #[test]
    fn orders_literals_by_symbol_id_arity_and_argument_encoding() {
        let context = sample_context(TautologyPolicy::Reject);
        let predicate_one = SymbolKey::new(SymbolKind::Predicate, 1);
        let predicate_two = SymbolKey::new(SymbolKind::Predicate, 2);

        let clause = Clause::normalize(
            vec![
                lit(
                    Polarity::Positive,
                    predicate_two,
                    vec![Term::Variable(VariableId(1))],
                ),
                lit(
                    Polarity::Positive,
                    predicate_one,
                    vec![Term::Variable(VariableId(2))],
                ),
                lit(Polarity::Positive, predicate_one, vec![]),
                lit(
                    Polarity::Positive,
                    predicate_one,
                    vec![Term::Variable(VariableId(1))],
                ),
            ],
            &context,
        )
        .expect("valid clause");

        assert_eq!(
            clause
                .literals()
                .iter()
                .map(Literal::render)
                .collect::<Vec<_>>(),
            [
                "+Predicate#1[arity=0]()",
                "+Predicate#1[arity=1](v1)",
                "+Predicate#1[arity=1](v2)",
                "+Predicate#2[arity=1](v1)",
            ]
        );
    }

    #[test]
    fn term_order_matches_canonical_byte_order() {
        let symbol = SymbolKey::new(SymbolKind::Predicate, 1);
        let empty_application = Term::Application {
            symbol,
            arguments: Vec::new(),
        };
        let variable_application = Term::Application {
            symbol,
            arguments: vec![Term::Variable(VariableId(1))],
        };

        assert_eq!(
            empty_application.cmp(&variable_application),
            empty_application
                .canonical_bytes()
                .expect("empty application bytes")
                .cmp(
                    &variable_application
                        .canonical_bytes()
                        .expect("variable application bytes")
                )
        );
    }

    #[test]
    fn removes_duplicate_literals_during_normalization() {
        let context = sample_context(TautologyPolicy::Reject);
        let symbol = SymbolKey::new(SymbolKind::Predicate, 1);
        let literal = lit(
            Polarity::Positive,
            symbol,
            vec![Term::Variable(VariableId(1))],
        );

        let clause =
            Clause::normalize(vec![literal.clone(), literal], &context).expect("deduped clause");

        assert_eq!(clause.literals().len(), 1);
    }

    #[test]
    fn empty_clause_is_distinct_from_tautology() {
        let context = sample_context(TautologyPolicy::Marker);

        let clause = Clause::normalize(Vec::new(), &context).expect("empty clause");

        assert_eq!(clause.form(), ClauseForm::Empty);
        assert!(clause.literals().is_empty());
        assert!(clause.render().contains("form=Empty"));
        assert_eq!(hash_input(&clause), hash_input(&clause));
    }

    #[test]
    fn rejects_or_marks_tautologies_according_to_profile() {
        let symbol = SymbolKey::new(SymbolKind::Predicate, 1);
        let positive = lit(
            Polarity::Positive,
            symbol,
            vec![Term::Variable(VariableId(1))],
        );
        let negative = lit(
            Polarity::Negative,
            symbol,
            vec![Term::Variable(VariableId(1))],
        );

        assert_eq!(
            Clause::normalize(
                vec![positive.clone(), negative.clone()],
                &sample_context(TautologyPolicy::Reject),
            ),
            Err(ClauseError::DisallowedTautology)
        );

        let marked = Clause::normalize(
            vec![
                positive,
                negative,
                lit(
                    Polarity::Positive,
                    SymbolKey::new(SymbolKind::Predicate, 2),
                    vec![],
                ),
            ],
            &sample_context(TautologyPolicy::Marker),
        )
        .expect("tautology marker");
        assert_eq!(marked.form(), ClauseForm::Tautology);
        assert!(marked.literals().is_empty());
        assert!(marked.render().contains("literals=0"));
    }

    #[test]
    fn rejects_malformed_atoms_terms_symbols_and_variables() {
        let context = sample_context(TautologyPolicy::Reject);
        let symbol = SymbolKey::new(SymbolKind::Predicate, 1);

        assert_eq!(
            Clause::normalize(
                vec![Literal::new(
                    Polarity::Positive,
                    Atom::with_arity(symbol, 2, vec![Term::Variable(VariableId(1))]),
                )],
                &context,
            ),
            Err(ClauseError::ArityMismatch {
                symbol,
                expected: 2,
                actual: 1,
            })
        );
        assert_eq!(
            Clause::normalize(
                vec![lit(Polarity::Positive, symbol, vec![Term::Malformed])],
                &context,
            ),
            Err(ClauseError::MalformedTermEncoding)
        );
        assert_eq!(
            Clause::normalize(
                vec![lit(
                    Polarity::Positive,
                    SymbolKey::new(SymbolKind::Predicate, 77),
                    vec![],
                )],
                &context,
            ),
            Err(ClauseError::MissingSymbol(SymbolKey::new(
                SymbolKind::Predicate,
                77,
            )))
        );
        assert_eq!(
            Clause::normalize(
                vec![lit(
                    Polarity::Positive,
                    SymbolKey::new(SymbolKind::BuiltinRelation, 1),
                    vec![],
                )],
                &context,
            ),
            Err(ClauseError::UnsupportedSymbolKind(
                SymbolKind::BuiltinRelation,
            ))
        );
        assert_eq!(
            Clause::normalize(
                vec![lit(
                    Polarity::Positive,
                    symbol,
                    vec![Term::Variable(VariableId(99))],
                )],
                &context,
            ),
            Err(ClauseError::NoncanonicalVariableId(VariableId(99)))
        );
    }

    #[test]
    fn enforces_profile_payload_and_resource_bounds() {
        let context = sample_context(TautologyPolicy::Reject).with_limits(1, 16);
        let symbol = SymbolKey::new(SymbolKind::Predicate, 1);
        let literal = lit(Polarity::Positive, symbol, vec![]);

        assert_eq!(
            Clause::from_canonical_parts(ClauseForm::Ordinary, Vec::new(), &context),
            Err(ClauseError::OrdinaryEmptyPayload)
        );
        assert_eq!(
            Clause::from_canonical_parts(ClauseForm::Empty, vec![literal.clone()], &context),
            Err(ClauseError::NonEmptyMarkerPayload {
                form: ClauseForm::Empty,
            })
        );
        assert_eq!(
            Clause::from_canonical_parts(ClauseForm::Tautology, Vec::new(), &context),
            Err(ClauseError::DisallowedTautology)
        );
        assert_eq!(
            Clause::from_canonical_parts(
                ClauseForm::Tautology,
                vec![literal.clone()],
                &sample_context(TautologyPolicy::Marker),
            ),
            Err(ClauseError::NonEmptyMarkerPayload {
                form: ClauseForm::Tautology,
            })
        );
        assert_eq!(
            Clause::from_canonical_parts(
                ClauseForm::Ordinary,
                vec![literal.clone(), literal.clone()],
                &sample_context(TautologyPolicy::Reject),
            ),
            Err(ClauseError::DuplicateLiteral)
        );
        assert_eq!(
            Clause::normalize(
                vec![
                    literal.clone(),
                    lit(
                        Polarity::Positive,
                        SymbolKey::new(SymbolKind::Predicate, 2),
                        vec![]
                    ),
                ],
                &context,
            ),
            Err(ClauseError::LiteralCountExceeded { max: 1, actual: 2 })
        );
        assert_eq!(
            Clause::normalize(
                vec![lit(
                    Polarity::Positive,
                    symbol,
                    vec![Term::Application {
                        symbol,
                        arguments: vec![
                            Term::Variable(VariableId(1)),
                            Term::Variable(VariableId(2)),
                        ],
                    }],
                )],
                &context,
            ),
            Err(ClauseError::TermSizeExceeded {
                max: 16,
                actual: 35
            })
        );
    }

    #[test]
    fn rejects_noncanonical_literal_order_in_canonical_constructor() {
        let context = sample_context(TautologyPolicy::Reject);
        let first = lit(
            Polarity::Positive,
            SymbolKey::new(SymbolKind::Predicate, 2),
            vec![],
        );
        let second = lit(
            Polarity::Positive,
            SymbolKey::new(SymbolKind::Predicate, 1),
            vec![],
        );

        assert_eq!(
            Clause::from_canonical_parts(ClauseForm::Ordinary, vec![first, second], &context),
            Err(ClauseError::NonCanonicalLiteralOrder)
        );
    }

    #[test]
    fn rendering_and_hash_inputs_are_stable_and_exclude_display_data() {
        let context = sample_context(TautologyPolicy::Reject);
        let symbol_one = SymbolKey::new(SymbolKind::Predicate, 1);
        let symbol_two = SymbolKey::new(SymbolKind::Predicate, 2);
        let raw_a = vec![
            lit(
                Polarity::Positive,
                symbol_two,
                vec![Term::Variable(VariableId(2))],
            ),
            lit(
                Polarity::Negative,
                symbol_one,
                vec![Term::Variable(VariableId(1))],
            ),
        ];
        let raw_b = vec![raw_a[1].clone(), raw_a[0].clone()];

        let clause_a = Clause::normalize(raw_a, &context).expect("valid clause");
        let clause_b = Clause::normalize(raw_b.clone(), &context).expect("valid clause");

        assert_eq!(clause_a.render(), clause_b.render());
        let hash_a = hash_input(&clause_a);
        assert_eq!(hash_a, hash_input(&clause_b));
        assert!(hash_a.starts_with(CLAUSE_DOMAIN_SEPARATOR));
        assert!(
            contains_subsequence(&hash_a, &1_u16.to_be_bytes()),
            "schema/profile version participates in hash input"
        );
        assert!(
            hash_a.contains(&ClauseForm::Ordinary.tag()),
            "clause form participates in hash input"
        );
        assert_ne!(
            hash_a,
            hash_input(
                &Clause::normalize(
                    vec![
                        lit(
                            Polarity::Positive,
                            symbol_two,
                            vec![Term::Variable(VariableId(2))]
                        ),
                        lit(
                            Polarity::Negative,
                            symbol_one,
                            vec![Term::Variable(VariableId(2))]
                        ),
                    ],
                    &context,
                )
                .expect("different canonical literal bytes")
            ),
            "canonical literal bytes participate in hash input"
        );
        assert_ne!(
            hash_a,
            hash_input(
                &Clause::normalize(raw_b.clone(), &sample_context(TautologyPolicy::Marker))
                    .expect("tautology policy variant")
            ),
            "tautology policy participates in hash input"
        );
        assert_ne!(
            hash_a,
            hash_input(
                &Clause::normalize(
                    raw_b,
                    &sample_context_with_profile(ClauseProfile::new(2, 3, TautologyPolicy::Reject)),
                )
                .expect("profile version variant")
            ),
            "schema and encoding versions participate in hash input"
        );

        let marker_context = sample_context(TautologyPolicy::Marker);
        let empty = Clause::normalize(Vec::new(), &marker_context).expect("empty clause");
        let tautology = Clause::normalize(
            vec![
                lit(
                    Polarity::Positive,
                    symbol_one,
                    vec![Term::Variable(VariableId(1))],
                ),
                lit(
                    Polarity::Negative,
                    symbol_one,
                    vec![Term::Variable(VariableId(1))],
                ),
            ],
            &marker_context,
        )
        .expect("tautology marker");
        assert_ne!(
            hash_input(&empty),
            hash_input(&tautology),
            "empty and tautology forms have distinct hash inputs"
        );

        for excluded in [
            "display-name",
            "source-range",
            "/tmp/source.miz",
            "2026-06-25T00:00:00Z",
            "backend-log",
            "allocation-order",
            "worker-completion-order",
        ] {
            assert!(
                !contains_subsequence(&hash_a, excluded.as_bytes()),
                "{excluded} must not affect canonical bytes"
            );
        }
    }

    fn sample_context(policy: TautologyPolicy) -> ClauseValidationContext {
        sample_context_with_profile(ClauseProfile::new(1, 1, policy))
    }

    fn sample_context_with_profile(profile: ClauseProfile) -> ClauseValidationContext {
        let mut context = ClauseValidationContext::new(profile).with_limits(8, 256);
        for kind in [
            SymbolKind::Predicate,
            SymbolKind::FunctorPredicate,
            SymbolKind::Equality,
        ] {
            context = context.with_allowed_symbol_kind(kind);
        }
        for symbol in [
            SymbolKey::new(SymbolKind::Predicate, 1),
            SymbolKey::new(SymbolKind::Predicate, 2),
            SymbolKey::new(SymbolKind::Predicate, 9),
            SymbolKey::new(SymbolKind::FunctorPredicate, 1),
            SymbolKey::new(SymbolKind::Equality, 1),
        ] {
            context = context.with_known_symbol(symbol);
        }
        for variable in [VariableId(1), VariableId(2), VariableId(3)] {
            context = context.with_canonical_variable(variable);
        }
        context
    }

    fn lit(polarity: Polarity, symbol: SymbolKey, arguments: Vec<Term>) -> Literal {
        Literal::new(polarity, Atom::new(symbol, arguments))
    }

    fn hash_input(clause: &Clause) -> Vec<u8> {
        clause
            .canonical_hash_input()
            .expect("valid clause canonical hash input")
    }

    fn contains_subsequence(bytes: &[u8], needle: &[u8]) -> bool {
        bytes.windows(needle.len()).any(|window| window == needle)
    }
}
