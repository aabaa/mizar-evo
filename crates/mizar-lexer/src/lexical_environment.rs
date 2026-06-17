use crate::raw_lexer::{
    RawTokenKind, RawTokenStream, is_identifier_continue, is_identifier_start,
    is_user_symbol_spelling,
};
use crate::source::{SourcePos, SourceSpan};
use crate::tables::{
    RESERVED_SYMBOLS, RESERVED_WORDS, ReservedSymbolTable, ReservedWordTable, is_reserved_symbol,
    is_reserved_word,
};
use std::collections::BTreeMap;
use std::error::Error;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ModuleId(pub String);

impl ModuleId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SymbolId(pub String);

impl SymbolId {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ExportRank(pub u32);

impl ExportRank {
    pub const fn new(value: u32) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u32 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum UserSymbolKind {
    Functor,
    Predicate,
    Mode,
    Attribute,
    Structure,
    Selector,
    Constructor,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserSymbolArity {
    pub minimum: u16,
    pub maximum: Option<u16>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UserSymbolKindSet(u16);

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LexicalSummaryFingerprint(pub u64);

impl LexicalSummaryFingerprint {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LexicalEnvironmentFingerprint(pub u64);

impl LexicalEnvironmentFingerprint {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedImport {
    pub module_id: ModuleId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleLexicalSummary {
    pub module_id: ModuleId,
    pub exported_symbols: Vec<ExportedSymbolShape>,
    pub fingerprint: LexicalSummaryFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExportedSymbolShape {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub export_rank: ExportRank,
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
    pub operator: Option<ExportedOperatorMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveLexicalEnvironment {
    pub reserved_words: ReservedWordTable,
    pub reserved_symbols: ReservedSymbolTable,
    pub user_symbols: UserSymbolIndex,
    pub fingerprint: LexicalEnvironmentFingerprint,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct LocalLexicalDeclarations {
    pub user_symbols: Vec<LocalUserSymbolDeclaration>,
    pub operator_declarations: Vec<LocalOperatorDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalUserSymbolDeclaration {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub export_rank: ExportRank,
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
    pub declared_at: SourceSpan,
    pub activation_start: SourcePos,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LocalOperatorDeclaration {
    pub spelling: String,
    pub source_module: ModuleId,
    pub declared_at: SourceSpan,
    pub activation_start: SourcePos,
    pub operator: Option<ExportedOperatorMetadata>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct UserSymbolIndex {
    symbols_by_spelling: BTreeMap<String, Vec<UserSymbolCandidate>>,
    trie_root: UserSymbolTrieNode,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct UserSymbolTrieNode {
    children: BTreeMap<u8, UserSymbolTrieNode>,
    candidates: Vec<UserSymbolCandidate>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UserSymbolCandidate {
    pub spelling: String,
    pub symbol_id: SymbolId,
    pub source_module: ModuleId,
    pub imported_module: ModuleId,
    pub import_ordinal: usize,
    pub export_rank: ExportRank,
    pub kind: UserSymbolKind,
    pub arity: UserSymbolArity,
    pub operator: Option<ExportedOperatorMetadata>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ExportedOperatorMetadata {
    pub fixity: ExportedOperatorFixity,
    pub precedence: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportedOperatorFixity {
    Prefix,
    Infix(ExportedOperatorAssociativity),
    Postfix,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExportedOperatorAssociativity {
    Left,
    Right,
    NonAssociative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum LexicalEnvironmentError {
    MissingModuleSummary {
        module_id: ModuleId,
    },
    InconsistentDuplicateSummary {
        module_id: ModuleId,
    },
    InvalidUserSymbolSpelling {
        spelling: String,
        module_id: ModuleId,
    },
    InvalidUserSymbolArity {
        spelling: String,
        module_id: ModuleId,
        arity: UserSymbolArity,
    },
    InvalidOperatorMetadata {
        spelling: String,
        module_id: ModuleId,
        kind: UserSymbolKind,
        arity: UserSymbolArity,
        operator: ExportedOperatorMetadata,
    },
    ReservedWordCollision {
        spelling: String,
        module_id: ModuleId,
    },
    ReservedSymbolCollision {
        spelling: String,
        module_id: ModuleId,
    },
    UserSymbolImportConflict {
        spelling: String,
        earlier_import: ModuleId,
        later_import: ModuleId,
    },
}

impl fmt::Display for LexicalEnvironmentError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingModuleSummary { module_id } => {
                write!(f, "missing lexical summary for module `{}`", module_id.0)
            }
            Self::InconsistentDuplicateSummary { module_id } => {
                write!(
                    f,
                    "inconsistent duplicate lexical summary for module `{}`",
                    module_id.0
                )
            }
            Self::InvalidUserSymbolSpelling {
                spelling,
                module_id,
            } => write!(
                f,
                "invalid user symbol spelling `{spelling}` exported by module `{}`",
                module_id.0
            ),
            Self::InvalidUserSymbolArity {
                spelling,
                module_id,
                arity,
            } => write!(
                f,
                "invalid arity {:?} for user symbol `{spelling}` exported by module `{}`",
                arity, module_id.0
            ),
            Self::InvalidOperatorMetadata {
                spelling,
                module_id,
                kind,
                arity,
                operator,
            } => write!(
                f,
                "invalid operator metadata {:?} for {:?} symbol `{spelling}` with arity {:?} exported by module `{}`",
                operator, kind, arity, module_id.0
            ),
            Self::ReservedWordCollision {
                spelling,
                module_id,
            } => write!(
                f,
                "user symbol `{spelling}` exported by module `{}` collides with a reserved word",
                module_id.0
            ),
            Self::ReservedSymbolCollision {
                spelling,
                module_id,
            } => write!(
                f,
                "user symbol `{spelling}` exported by module `{}` collides with a reserved symbol",
                module_id.0
            ),
            Self::UserSymbolImportConflict {
                spelling,
                earlier_import,
                later_import,
            } => write!(
                f,
                "user symbol `{spelling}` exported by module `{}` conflicts with earlier import from module `{}`",
                later_import.0, earlier_import.0
            ),
        }
    }
}

impl Error for LexicalEnvironmentError {}
impl UserSymbolKind {
    const fn tag(self) -> u8 {
        match self {
            Self::Functor => 0,
            Self::Predicate => 1,
            Self::Mode => 2,
            Self::Attribute => 3,
            Self::Structure => 4,
            Self::Selector => 5,
            Self::Constructor => 6,
        }
    }

    const fn bit(self) -> u16 {
        1 << self.tag()
    }
}

impl UserSymbolArity {
    pub const fn exact(value: u16) -> Self {
        Self {
            minimum: value,
            maximum: Some(value),
        }
    }

    pub const fn range(minimum: u16, maximum: u16) -> Self {
        Self {
            minimum,
            maximum: Some(maximum),
        }
    }

    pub const fn at_least(minimum: u16) -> Self {
        Self {
            minimum,
            maximum: None,
        }
    }

    fn is_valid(self) -> bool {
        self.maximum
            .map(|maximum| self.minimum <= maximum)
            .unwrap_or(true)
    }
}

impl UserSymbolKindSet {
    pub const fn empty() -> Self {
        Self(0)
    }

    pub const fn all() -> Self {
        Self(
            (1 << UserSymbolKind::Functor.tag())
                | (1 << UserSymbolKind::Predicate.tag())
                | (1 << UserSymbolKind::Mode.tag())
                | (1 << UserSymbolKind::Attribute.tag())
                | (1 << UserSymbolKind::Structure.tag())
                | (1 << UserSymbolKind::Selector.tag())
                | (1 << UserSymbolKind::Constructor.tag()),
        )
    }

    pub const fn only(kind: UserSymbolKind) -> Self {
        Self(kind.bit())
    }

    pub fn from_slice(kinds: &[UserSymbolKind]) -> Self {
        let mut set = Self::empty();
        for kind in kinds {
            set.insert(*kind);
        }
        set
    }

    pub fn insert(&mut self, kind: UserSymbolKind) {
        self.0 |= kind.bit();
    }

    pub fn contains(self, kind: UserSymbolKind) -> bool {
        self.0 & kind.bit() != 0
    }
}

pub fn build_lexical_environment(
    imports: &[ResolvedImport],
    summaries: &[ModuleLexicalSummary],
) -> Result<ActiveLexicalEnvironment, LexicalEnvironmentError> {
    let summaries_by_module = index_module_lexical_summaries(summaries)?;
    let mut user_symbols = UserSymbolIndex::default();
    let mut fingerprint = StableFingerprint::new();

    fingerprint.write_str("mizar-lexer.active-lexical-environment.v2");
    fingerprint.write_str("reserved-words");
    for word in RESERVED_WORDS {
        fingerprint.write_str(word);
    }
    fingerprint.write_str("reserved-symbols");
    for symbol in RESERVED_SYMBOLS {
        fingerprint.write_str(symbol);
    }

    for (import_ordinal, import) in imports.iter().enumerate() {
        let summary = summaries_by_module.get(&import.module_id).ok_or_else(|| {
            LexicalEnvironmentError::MissingModuleSummary {
                module_id: import.module_id.clone(),
            }
        })?;

        fingerprint.write_usize(import_ordinal);
        fingerprint.write_str(&import.module_id.0);
        fingerprint.write_u64(summary.fingerprint.0);

        for exported in &summary.exported_symbols {
            validate_exported_symbol_shape(exported)?;
            fingerprint.write_str(&exported.spelling);
            fingerprint.write_str(&exported.symbol_id.0);
            fingerprint.write_str(&exported.source_module.0);
            fingerprint.write_u64(u64::from(exported.export_rank.0));
            fingerprint.write_byte(exported.kind.tag());
            fingerprint.write_u64(u64::from(exported.arity.minimum));
            match exported.arity.maximum {
                Some(maximum) => {
                    fingerprint.write_byte(1);
                    fingerprint.write_u64(u64::from(maximum));
                }
                None => fingerprint.write_byte(0),
            }
            match exported.operator {
                Some(operator) => {
                    fingerprint.write_byte(1);
                    fingerprint.write_byte(operator_fixity_tag(operator.fixity));
                    if let ExportedOperatorFixity::Infix(associativity) = operator.fixity {
                        fingerprint.write_byte(operator_associativity_tag(associativity));
                    }
                    fingerprint.write_byte(operator.precedence);
                }
                None => fingerprint.write_byte(0),
            }

            user_symbols.insert(UserSymbolCandidate {
                spelling: exported.spelling.clone(),
                symbol_id: exported.symbol_id.clone(),
                source_module: exported.source_module.clone(),
                imported_module: import.module_id.clone(),
                import_ordinal,
                export_rank: exported.export_rank,
                kind: exported.kind,
                arity: exported.arity,
                operator: exported.operator,
            })?;
        }
    }

    Ok(ActiveLexicalEnvironment {
        reserved_words: RESERVED_WORDS,
        reserved_symbols: RESERVED_SYMBOLS,
        user_symbols,
        fingerprint: LexicalEnvironmentFingerprint(fingerprint.finish()),
    })
}

pub fn collect_local_lexical_declarations(
    raw: &RawTokenStream,
    current_module: ModuleId,
) -> LocalLexicalDeclarations {
    LocalDeclarationCollector::new(raw, current_module).collect()
}

impl ActiveLexicalEnvironment {
    pub fn reserved_word(&self, spelling: &str) -> Option<&'static str> {
        self.reserved_words
            .iter()
            .copied()
            .find(|word| *word == spelling)
    }

    pub fn reserved_symbol(&self, spelling: &str) -> Option<&'static str> {
        self.reserved_symbols
            .iter()
            .copied()
            .find(|symbol| *symbol == spelling)
    }

    pub fn user_symbol(&self, spelling: &str) -> Option<&UserSymbolCandidate> {
        self.user_symbols.visible_symbol(spelling)
    }

    pub fn visible_user_symbols(&self) -> Vec<&UserSymbolCandidate> {
        self.user_symbols.visible_symbols()
    }

    pub fn user_symbols_at(
        &self,
        spelling: &str,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate> {
        let mut candidates = self.user_symbols.visible_candidates(spelling);
        candidates.extend(local_declarations.user_symbols(spelling, position));
        sort_user_symbol_candidates(&mut candidates);
        candidates
    }

    pub fn visible_user_symbols_at(
        &self,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate> {
        let mut symbols = self
            .visible_user_symbols()
            .into_iter()
            .cloned()
            .collect::<Vec<_>>();
        symbols.extend(local_declarations.visible_user_symbols(position));
        sort_user_symbol_candidates(&mut symbols);
        symbols
    }

    pub fn longest_user_symbol_at(&self, input: &str, start: usize) -> Vec<UserSymbolCandidate> {
        self.user_symbols.longest_at(input, start)
    }

    pub fn longest_user_symbol_at_position(
        &self,
        input: &str,
        start: usize,
        position: SourcePos,
        local_declarations: &LocalLexicalDeclarations,
    ) -> Vec<UserSymbolCandidate> {
        let imported = self.longest_user_symbol_at(input, start);
        let local = local_declarations.longest_user_symbol_at(input, start, position);

        match (imported.first(), local.first()) {
            (None, None) => Vec::new(),
            (Some(_), None) => imported,
            (None, Some(_)) => local,
            (Some(imported_candidate), Some(local_candidate)) => {
                match imported_candidate
                    .spelling
                    .len()
                    .cmp(&local_candidate.spelling.len())
                {
                    std::cmp::Ordering::Greater => imported,
                    std::cmp::Ordering::Less => local,
                    std::cmp::Ordering::Equal => {
                        let mut candidates = imported;
                        candidates.extend(local);
                        sort_user_symbol_candidates(&mut candidates);
                        candidates
                    }
                }
            }
        }
    }
}
impl LocalLexicalDeclarations {
    pub fn empty() -> Self {
        Self::default()
    }

    pub fn user_symbols(&self, spelling: &str, position: SourcePos) -> Vec<UserSymbolCandidate> {
        let mut candidates = self
            .user_symbols
            .iter()
            .filter(|declaration| {
                declaration.spelling == spelling && declaration.activation_start <= position
            })
            .map(LocalUserSymbolDeclaration::candidate)
            .collect::<Vec<_>>();
        sort_user_symbol_candidates(&mut candidates);
        candidates
    }

    pub fn visible_user_symbols(&self, position: SourcePos) -> Vec<UserSymbolCandidate> {
        let mut candidates = self
            .user_symbols
            .iter()
            .filter(|declaration| declaration.activation_start <= position)
            .map(LocalUserSymbolDeclaration::candidate)
            .collect::<Vec<_>>();
        sort_user_symbol_candidates(&mut candidates);
        candidates
    }

    pub fn longest_user_symbol_at(
        &self,
        input: &str,
        start: usize,
        position: SourcePos,
    ) -> Vec<UserSymbolCandidate> {
        let Some(rest) = input.get(start..) else {
            return Vec::new();
        };

        let mut longest_len = 0;
        let mut candidates = Vec::new();
        for declaration in &self.user_symbols {
            if declaration.activation_start > position || !rest.starts_with(&declaration.spelling) {
                continue;
            }
            match declaration.spelling.len().cmp(&longest_len) {
                std::cmp::Ordering::Greater => {
                    longest_len = declaration.spelling.len();
                    candidates.clear();
                    candidates.push(declaration.candidate());
                }
                std::cmp::Ordering::Equal => candidates.push(declaration.candidate()),
                std::cmp::Ordering::Less => {}
            }
        }
        sort_user_symbol_candidates(&mut candidates);
        candidates
    }
}

impl LocalUserSymbolDeclaration {
    fn candidate(&self) -> UserSymbolCandidate {
        UserSymbolCandidate {
            spelling: self.spelling.clone(),
            symbol_id: self.symbol_id.clone(),
            source_module: self.source_module.clone(),
            imported_module: self.source_module.clone(),
            import_ordinal: usize::MAX,
            export_rank: self.export_rank,
            kind: self.kind,
            arity: self.arity,
            operator: None,
        }
    }
}

impl UserSymbolIndex {
    pub fn visible_symbol(&self, spelling: &str) -> Option<&UserSymbolCandidate> {
        self.symbols_by_spelling
            .get(spelling)
            .and_then(|candidates| candidates.last())
    }

    pub fn visible_candidates(&self, spelling: &str) -> Vec<UserSymbolCandidate> {
        self.symbols_by_spelling
            .get(spelling)
            .map(|candidates| visible_user_symbol_candidates(candidates))
            .unwrap_or_default()
    }

    pub fn visible_symbols(&self) -> Vec<&UserSymbolCandidate> {
        let mut symbols = self
            .symbols_by_spelling
            .values()
            .flat_map(|candidates| {
                let visible_import = candidates.last().map(|candidate| candidate.import_ordinal);
                candidates.iter().filter(move |candidate| {
                    visible_import.is_some_and(|import| candidate.import_ordinal == import)
                })
            })
            .collect::<Vec<_>>();
        symbols.sort_by(|left, right| {
            left.spelling
                .cmp(&right.spelling)
                .then_with(|| left.symbol_id.cmp(&right.symbol_id))
        });
        symbols
    }

    pub fn longest_at(&self, input: &str, start: usize) -> Vec<UserSymbolCandidate> {
        let Some(rest) = input.get(start..) else {
            return Vec::new();
        };

        let mut node = &self.trie_root;
        let mut longest_candidates = None;
        for byte in rest.as_bytes() {
            let Some(child) = node.children.get(byte) else {
                break;
            };
            node = child;
            if !node.candidates.is_empty() {
                longest_candidates = Some(node.candidates.as_slice());
            }
        }

        longest_candidates
            .map(visible_user_symbol_candidates)
            .unwrap_or_default()
    }

    fn insert(&mut self, candidate: UserSymbolCandidate) -> Result<(), LexicalEnvironmentError> {
        let spelling = candidate.spelling.clone();
        let candidates = self
            .symbols_by_spelling
            .entry(spelling.clone())
            .or_default();
        if let Some(previous) = candidates
            .iter()
            .find(|previous| previous.import_ordinal != candidate.import_ordinal)
        {
            return Err(LexicalEnvironmentError::UserSymbolImportConflict {
                spelling: candidate.spelling,
                earlier_import: previous.imported_module.clone(),
                later_import: candidate.imported_module,
            });
        }

        candidates.push(candidate);
        sort_user_symbol_candidates(candidates);
        self.sync_trie_terminal(&spelling);
        Ok(())
    }

    fn sync_trie_terminal(&mut self, spelling: &str) {
        let mut node = &mut self.trie_root;
        for byte in spelling.bytes() {
            node = node.children.entry(byte).or_default();
        }
        node.candidates = self
            .symbols_by_spelling
            .get(spelling)
            .expect("inserted spelling should be present")
            .clone();
    }
}

fn visible_user_symbol_candidates(
    spelling_candidates: &[UserSymbolCandidate],
) -> Vec<UserSymbolCandidate> {
    let Some(visible_import) = spelling_candidates
        .last()
        .map(|candidate| candidate.import_ordinal)
    else {
        return Vec::new();
    };
    let mut candidates = spelling_candidates
        .iter()
        .filter(|candidate| candidate.import_ordinal == visible_import)
        .cloned()
        .collect::<Vec<_>>();

    candidates.sort_by(|left, right| {
        right
            .import_ordinal
            .cmp(&left.import_ordinal)
            .then_with(|| left.spelling.cmp(&right.spelling))
            .then_with(|| left.symbol_id.cmp(&right.symbol_id))
    });
    candidates
}

fn sort_user_symbol_candidates(candidates: &mut [UserSymbolCandidate]) {
    candidates.sort_by(|left, right| {
        left.import_ordinal
            .cmp(&right.import_ordinal)
            .then_with(|| left.export_rank.cmp(&right.export_rank))
            .then_with(|| left.kind.cmp(&right.kind))
            .then_with(|| left.arity.cmp(&right.arity))
            .then_with(|| left.source_module.cmp(&right.source_module))
            .then_with(|| left.symbol_id.cmp(&right.symbol_id))
    });
}

fn operator_fixity_tag(fixity: ExportedOperatorFixity) -> u8 {
    match fixity {
        ExportedOperatorFixity::Prefix => 0,
        ExportedOperatorFixity::Infix(_) => 1,
        ExportedOperatorFixity::Postfix => 2,
    }
}

fn operator_associativity_tag(associativity: ExportedOperatorAssociativity) -> u8 {
    match associativity {
        ExportedOperatorAssociativity::Left => 0,
        ExportedOperatorAssociativity::Right => 1,
        ExportedOperatorAssociativity::NonAssociative => 2,
    }
}

#[cfg(test)]
impl UserSymbolIndex {
    pub(crate) fn trie_node_count(&self) -> usize {
        fn count(node: &UserSymbolTrieNode) -> usize {
            1 + node.children.values().map(count).sum::<usize>()
        }

        count(&self.trie_root)
    }

    pub(crate) fn spelling_count(&self) -> usize {
        self.symbols_by_spelling.len()
    }
}

fn index_module_lexical_summaries(
    summaries: &[ModuleLexicalSummary],
) -> Result<BTreeMap<ModuleId, &ModuleLexicalSummary>, LexicalEnvironmentError> {
    let mut summaries_by_module = BTreeMap::new();
    for summary in summaries {
        if let Some(previous) = summaries_by_module.insert(summary.module_id.clone(), summary)
            && previous != summary
        {
            return Err(LexicalEnvironmentError::InconsistentDuplicateSummary {
                module_id: summary.module_id.clone(),
            });
        }
    }
    Ok(summaries_by_module)
}

fn validate_exported_symbol_shape(
    exported: &ExportedSymbolShape,
) -> Result<(), LexicalEnvironmentError> {
    if !is_user_symbol_spelling(&exported.spelling) {
        return Err(LexicalEnvironmentError::InvalidUserSymbolSpelling {
            spelling: exported.spelling.clone(),
            module_id: exported.source_module.clone(),
        });
    }
    if !exported.arity.is_valid() {
        return Err(LexicalEnvironmentError::InvalidUserSymbolArity {
            spelling: exported.spelling.clone(),
            module_id: exported.source_module.clone(),
            arity: exported.arity,
        });
    }
    if is_reserved_word(&exported.spelling) {
        return Err(LexicalEnvironmentError::ReservedWordCollision {
            spelling: exported.spelling.clone(),
            module_id: exported.source_module.clone(),
        });
    }
    if is_reserved_symbol(&exported.spelling) && exported.spelling != "." {
        return Err(LexicalEnvironmentError::ReservedSymbolCollision {
            spelling: exported.spelling.clone(),
            module_id: exported.source_module.clone(),
        });
    }
    if let Some(operator) = exported.operator {
        let expected_arity = match operator.fixity {
            ExportedOperatorFixity::Prefix | ExportedOperatorFixity::Postfix => 1,
            ExportedOperatorFixity::Infix(_) => 2,
        };
        if exported.kind != UserSymbolKind::Functor
            || exported.arity != UserSymbolArity::exact(expected_arity)
        {
            return Err(LexicalEnvironmentError::InvalidOperatorMetadata {
                spelling: exported.spelling.clone(),
                module_id: exported.source_module.clone(),
                kind: exported.kind,
                arity: exported.arity,
                operator,
            });
        }
    }
    Ok(())
}

struct LocalDeclarationCollector {
    pieces: Vec<DeclarationPiece>,
    current_module: ModuleId,
    next_rank: u32,
    user_symbols: Vec<LocalUserSymbolDeclaration>,
    operator_declarations: Vec<LocalOperatorDeclaration>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DeclarationPiece {
    text: String,
    span: SourceSpan,
    kind: DeclarationPieceKind,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DeclarationPieceKind {
    Word,
    Numeral,
    StringLiteral,
    Symbol,
}

impl LocalDeclarationCollector {
    fn new(raw: &RawTokenStream, current_module: ModuleId) -> Self {
        Self {
            pieces: declaration_pieces(raw),
            current_module,
            next_rank: 0,
            user_symbols: Vec::new(),
            operator_declarations: Vec::new(),
        }
    }

    fn collect(mut self) -> LocalLexicalDeclarations {
        for index in 0..self.pieces.len() {
            if self.is_redefinition(index) {
                continue;
            }
            match self.pieces[index].text.as_str() {
                "pred" => self.collect_pattern_declaration(
                    index,
                    UserSymbolKind::Predicate,
                    PatternTerminator::Word("means"),
                ),
                "func" => self.collect_pattern_declaration(
                    index,
                    UserSymbolKind::Functor,
                    PatternTerminator::Symbol("->"),
                ),
                "mode" => self.collect_mode_declaration(index),
                "attr" => self.collect_attribute_declaration(index),
                "struct" => self.collect_structure_declaration(index),
                "synonym" | "antonym" => self.collect_alias_declaration(index),
                "infix_operator" | "prefix_operator" | "postfix_operator" => {
                    self.collect_operator_declaration(index)
                }
                _ => {}
            }
        }

        LocalLexicalDeclarations {
            user_symbols: self.user_symbols,
            operator_declarations: self.operator_declarations,
        }
    }

    fn is_redefinition(&self, index: usize) -> bool {
        index > 0 && self.pieces[index - 1].text == "redefine"
    }

    fn collect_pattern_declaration(
        &mut self,
        keyword_index: usize,
        kind: UserSymbolKind,
        terminator: PatternTerminator,
    ) {
        let Some(completion) = self.item_completion(keyword_index) else {
            return;
        };
        let Some(colon) = self.find_symbol(keyword_index + 1, completion.piece_index, ":") else {
            return;
        };
        let Some(pattern_end) = self.find_terminator(colon + 1, completion.piece_index, terminator)
        else {
            return;
        };
        let pattern = &self.pieces[colon + 1..pattern_end];
        let symbol_selections = select_symbol_spelling_pieces(pattern);
        if symbol_selections.is_empty() {
            let Some(selection) = select_notation_spelling(pattern) else {
                return;
            };
            let arity = pattern_arity(pattern, selection.relative_index);
            let absolute_index = colon + 1 + selection.relative_index;
            self.push_user_symbol(absolute_index, kind, arity, completion.activation_start);
        } else {
            let arity = pattern_arity_without_symbol_pieces(pattern);
            for selection in symbol_selections {
                let absolute_index = colon + 1 + selection.relative_index;
                self.push_user_symbol(absolute_index, kind, arity, completion.activation_start);
            }
        }
    }

    fn collect_mode_declaration(&mut self, keyword_index: usize) {
        let Some(completion) = self.item_completion(keyword_index) else {
            return;
        };
        let Some(colon) = self.find_symbol(keyword_index + 1, completion.piece_index, ":") else {
            return;
        };
        let name_end = self
            .find_word(colon + 1, completion.piece_index, "is")
            .unwrap_or(completion.piece_index);
        let Some(selection) = select_constructor_spelling(&self.pieces[colon + 1..name_end]) else {
            return;
        };
        self.push_user_symbol(
            colon + 1 + selection.relative_index,
            UserSymbolKind::Mode,
            UserSymbolArity::exact(0),
            completion.activation_start,
        );
    }

    fn collect_attribute_declaration(&mut self, keyword_index: usize) {
        let Some(completion) = self.item_completion(keyword_index) else {
            return;
        };
        let Some(colon) = self.find_symbol(keyword_index + 1, completion.piece_index, ":") else {
            return;
        };
        let Some(is_index) = self.find_word(colon + 1, completion.piece_index, "is") else {
            return;
        };
        let name_end = self
            .find_word(is_index + 1, completion.piece_index, "means")
            .unwrap_or(completion.piece_index);
        let Some(selection) = select_constructor_spelling(&self.pieces[is_index + 1..name_end])
        else {
            return;
        };
        self.push_user_symbol(
            is_index + 1 + selection.relative_index,
            UserSymbolKind::Attribute,
            UserSymbolArity::exact(1),
            completion.activation_start,
        );
    }

    fn collect_structure_declaration(&mut self, keyword_index: usize) {
        let Some(completion) = self.struct_completion(keyword_index) else {
            return;
        };
        let name_end = self
            .find_word(keyword_index + 1, completion.piece_index, "where")
            .unwrap_or(completion.piece_index);
        let Some(selection) =
            select_constructor_spelling(&self.pieces[keyword_index + 1..name_end])
        else {
            return;
        };
        self.push_user_symbol(
            keyword_index + 1 + selection.relative_index,
            UserSymbolKind::Structure,
            UserSymbolArity::exact(0),
            completion.activation_start,
        );
    }

    fn collect_alias_declaration(&mut self, keyword_index: usize) {
        let Some(completion) = self.item_completion(keyword_index) else {
            return;
        };
        let Some(for_index) = self.find_word(keyword_index + 1, completion.piece_index, "for")
        else {
            return;
        };
        let pattern = &self.pieces[keyword_index + 1..for_index];
        let symbol_selections = select_symbol_spelling_pieces(pattern);
        let kind = if !symbol_selections.is_empty() {
            UserSymbolKind::Functor
        } else {
            UserSymbolKind::Mode
        };
        if symbol_selections.is_empty() {
            let Some(selection) = select_notation_spelling(pattern) else {
                return;
            };
            let arity = pattern_arity(pattern, selection.relative_index);
            self.push_user_symbol(
                keyword_index + 1 + selection.relative_index,
                kind,
                arity,
                completion.activation_start,
            );
        } else {
            let arity = pattern_arity_without_symbol_pieces(pattern);
            for selection in symbol_selections {
                self.push_user_symbol(
                    keyword_index + 1 + selection.relative_index,
                    kind,
                    arity,
                    completion.activation_start,
                );
            }
        }
    }

    fn collect_operator_declaration(&mut self, keyword_index: usize) {
        let Some(completion) = self.item_completion(keyword_index) else {
            return;
        };
        let operator_spelling = self.pieces[keyword_index + 1..completion.piece_index]
            .iter()
            .find(|piece| piece.kind == DeclarationPieceKind::StringLiteral)
            .and_then(|piece| {
                unquote_string_literal(&piece.text).map(|spelling| (piece, spelling))
            });
        let Some((piece, spelling)) = operator_spelling else {
            return;
        };
        let operator = self.operator_metadata(keyword_index, completion.piece_index);
        self.operator_declarations.push(LocalOperatorDeclaration {
            spelling,
            source_module: self.current_module.clone(),
            declared_at: piece.span,
            activation_start: completion.activation_start,
            operator,
        });
    }

    fn operator_metadata(
        &self,
        keyword_index: usize,
        item_end: usize,
    ) -> Option<ExportedOperatorMetadata> {
        let keyword = self.pieces[keyword_index].text.as_str();
        let precedence = self.pieces[keyword_index + 1..item_end]
            .iter()
            .rev()
            .find(|piece| piece.kind == DeclarationPieceKind::Numeral)
            .and_then(|piece| piece.text.parse::<u8>().ok())?;
        let fixity = match keyword {
            "prefix_operator" => ExportedOperatorFixity::Prefix,
            "postfix_operator" => ExportedOperatorFixity::Postfix,
            "infix_operator" => {
                let associativity = self.pieces[keyword_index + 1..item_end].iter().find_map(
                    |piece| match piece.text.as_str() {
                        "left" => Some(ExportedOperatorAssociativity::Left),
                        "right" => Some(ExportedOperatorAssociativity::Right),
                        "none" => Some(ExportedOperatorAssociativity::NonAssociative),
                        _ => None,
                    },
                )?;
                ExportedOperatorFixity::Infix(associativity)
            }
            _ => return None,
        };
        Some(ExportedOperatorMetadata { fixity, precedence })
    }

    fn push_user_symbol(
        &mut self,
        piece_index: usize,
        kind: UserSymbolKind,
        arity: UserSymbolArity,
        activation_start: SourcePos,
    ) {
        let piece = &self.pieces[piece_index];
        if !is_user_symbol_spelling(&piece.text)
            || is_reserved_word(&piece.text)
            || (is_reserved_symbol(&piece.text) && piece.text != ".")
            || !arity.is_valid()
        {
            return;
        }

        let rank = ExportRank(self.next_rank);
        self.next_rank += 1;
        self.user_symbols.push(LocalUserSymbolDeclaration {
            spelling: piece.text.clone(),
            symbol_id: SymbolId::new(format!(
                "{}#local:{}:{}",
                self.current_module.as_str(),
                rank.get(),
                piece.text
            )),
            source_module: self.current_module.clone(),
            export_rank: rank,
            kind,
            arity,
            declared_at: piece.span,
            activation_start,
        });
    }

    fn item_completion(&self, keyword_index: usize) -> Option<ItemCompletion> {
        if self.pieces[keyword_index].text == "struct" {
            return self.struct_completion(keyword_index);
        }
        let semicolon = self.find_symbol(keyword_index + 1, self.pieces.len(), ";")?;
        let completion = ItemCompletion {
            piece_index: semicolon,
            activation_start: self.pieces[semicolon].span.end,
        };
        Some(self.extend_completion_through_declaration_trail(completion))
    }

    fn struct_completion(&self, keyword_index: usize) -> Option<ItemCompletion> {
        let mut cursor = keyword_index + 1;
        while cursor < self.pieces.len() {
            if self.pieces[cursor].text == "end"
                && self
                    .pieces
                    .get(cursor + 1)
                    .is_some_and(|piece| piece.text == ";")
            {
                return Some(ItemCompletion {
                    piece_index: cursor + 1,
                    activation_start: self.pieces[cursor + 1].span.end,
                });
            }
            cursor += 1;
        }
        self.item_completion_fallback(keyword_index)
    }

    fn extend_completion_through_declaration_trail(
        &self,
        mut completion: ItemCompletion,
    ) -> ItemCompletion {
        while self
            .pieces
            .get(completion.piece_index + 1)
            .is_some_and(|piece| is_declaration_trailing_keyword(&piece.text))
        {
            let trail_start = completion.piece_index + 1;
            let next_semicolon = if self
                .find_word(trail_start, self.pieces.len(), "proof")
                .is_some_and(|proof| {
                    self.find_symbol(trail_start, self.pieces.len(), ";")
                        .is_some_and(|semicolon| proof < semicolon)
                }) {
                self.find_end_semicolon(trail_start)
            } else {
                self.find_symbol(trail_start, self.pieces.len(), ";")
            };

            let Some(semicolon) = next_semicolon else {
                break;
            };
            completion = ItemCompletion {
                piece_index: semicolon,
                activation_start: self.pieces[semicolon].span.end,
            };
        }
        completion
    }

    fn item_completion_fallback(&self, keyword_index: usize) -> Option<ItemCompletion> {
        let semicolon = self.find_symbol(keyword_index + 1, self.pieces.len(), ";")?;
        Some(ItemCompletion {
            piece_index: semicolon,
            activation_start: self.pieces[semicolon].span.end,
        })
    }

    fn find_symbol(&self, start: usize, end: usize, symbol: &str) -> Option<usize> {
        self.pieces[start..end]
            .iter()
            .position(|piece| piece.kind == DeclarationPieceKind::Symbol && piece.text == symbol)
            .map(|index| start + index)
    }

    fn find_word(&self, start: usize, end: usize, word: &str) -> Option<usize> {
        self.pieces[start..end]
            .iter()
            .position(|piece| piece.kind == DeclarationPieceKind::Word && piece.text == word)
            .map(|index| start + index)
    }

    fn find_terminator(
        &self,
        start: usize,
        end: usize,
        terminator: PatternTerminator,
    ) -> Option<usize> {
        self.pieces[start..end]
            .iter()
            .position(|piece| match terminator {
                PatternTerminator::Word(word) => {
                    piece.kind == DeclarationPieceKind::Word && piece.text == word
                }
                PatternTerminator::Symbol(symbol) => {
                    piece.kind == DeclarationPieceKind::Symbol && piece.text == symbol
                }
            })
            .map(|index| start + index)
    }

    fn find_end_semicolon(&self, start: usize) -> Option<usize> {
        let mut depth = 0usize;
        for cursor in start..self.pieces.len() {
            let piece = &self.pieces[cursor];
            if piece.kind != DeclarationPieceKind::Word {
                continue;
            }
            if is_trailing_proof_block_opener(&piece.text) {
                depth += 1;
                continue;
            }
            if piece.text == "end" && depth > 0 {
                depth -= 1;
                if depth == 0
                    && self.pieces.get(cursor + 1).is_some_and(|next| {
                        next.kind == DeclarationPieceKind::Symbol && next.text == ";"
                    })
                {
                    return Some(cursor + 1);
                }
            }
        }
        None
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct ItemCompletion {
    piece_index: usize,
    activation_start: SourcePos,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum PatternTerminator {
    Word(&'static str),
    Symbol(&'static str),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SpellingSelection {
    relative_index: usize,
}

fn select_notation_spelling(pieces: &[DeclarationPiece]) -> Option<SpellingSelection> {
    if pieces
        .first()
        .is_some_and(|piece| piece.kind == DeclarationPieceKind::Word)
        && pieces
            .get(1)
            .is_some_and(|piece| piece.kind == DeclarationPieceKind::Symbol && piece.text == "(")
    {
        return Some(SpellingSelection { relative_index: 0 });
    }

    pieces
        .iter()
        .position(|piece| {
            piece.kind == DeclarationPieceKind::Symbol && !is_declaration_delimiter(&piece.text)
        })
        .or_else(|| {
            let word_indexes = pieces
                .iter()
                .enumerate()
                .filter(|(_, piece)| {
                    piece.kind == DeclarationPieceKind::Word && !is_reserved_word(&piece.text)
                })
                .map(|(index, _)| index)
                .collect::<Vec<_>>();
            match word_indexes.as_slice() {
                [] => None,
                [only] => Some(*only),
                [first, second] => {
                    if second_word_is_likely_infix_notation(
                        &pieces[*first],
                        &pieces[*second],
                        false,
                    ) {
                        Some(*second)
                    } else {
                        Some(*first)
                    }
                }
                [first, second, rest @ ..] => {
                    if second_word_is_likely_infix_notation(
                        &pieces[*first],
                        &pieces[*second],
                        !rest.is_empty(),
                    ) {
                        Some(*second)
                    } else {
                        Some(*first)
                    }
                }
            }
        })
        .map(|relative_index| SpellingSelection { relative_index })
}

fn select_symbol_spelling_pieces(pieces: &[DeclarationPiece]) -> Vec<SpellingSelection> {
    pieces
        .iter()
        .enumerate()
        .filter(|(_, piece)| {
            piece.kind == DeclarationPieceKind::Symbol && !is_declaration_delimiter(&piece.text)
        })
        .map(|(relative_index, _)| SpellingSelection { relative_index })
        .collect()
}

fn second_word_is_likely_infix_notation(
    first: &DeclarationPiece,
    second: &DeclarationPiece,
    has_right_locus: bool,
) -> bool {
    let first_text = first.text.as_str();
    let second_text = second.text.as_str();
    first_text.len() == 1
        && first_text
            .chars()
            .next()
            .is_some_and(|ch| ch.is_ascii_lowercase() || ch == '_')
        && (has_right_locus
            || second_text.len() > 1
            || second_text.contains('_')
            || second_text.contains('\'')
            || second_text
                .chars()
                .next()
                .is_some_and(|ch| ch.is_ascii_uppercase()))
}

fn select_constructor_spelling(pieces: &[DeclarationPiece]) -> Option<SpellingSelection> {
    pieces
        .iter()
        .position(|piece| {
            matches!(
                piece.kind,
                DeclarationPieceKind::Word | DeclarationPieceKind::Symbol
            ) && !is_declaration_delimiter(&piece.text)
                && !matches!(
                    piece.text.as_str(),
                    "of" | "over" | "where" | "is" | "means"
                )
        })
        .map(|relative_index| SpellingSelection { relative_index })
}

fn pattern_arity(pieces: &[DeclarationPiece], spelling_index: usize) -> UserSymbolArity {
    let arity = pieces
        .iter()
        .enumerate()
        .filter(|(index, piece)| {
            *index != spelling_index
                && piece.kind == DeclarationPieceKind::Word
                && !is_reserved_word(&piece.text)
        })
        .count();
    UserSymbolArity::exact(arity.min(u16::MAX as usize) as u16)
}

fn pattern_arity_without_symbol_pieces(pieces: &[DeclarationPiece]) -> UserSymbolArity {
    let arity = pieces
        .iter()
        .filter(|piece| piece.kind == DeclarationPieceKind::Word && !is_reserved_word(&piece.text))
        .count();
    UserSymbolArity::exact(arity.min(u16::MAX as usize) as u16)
}

fn declaration_pieces(raw: &RawTokenStream) -> Vec<DeclarationPiece> {
    let mut pieces = Vec::new();
    for token in &raw.tokens {
        match token.kind {
            RawTokenKind::Layout | RawTokenKind::Error => {}
            RawTokenKind::AnnotationMarker => pieces.push(DeclarationPiece {
                text: token.lexeme.clone(),
                span: token.span,
                kind: DeclarationPieceKind::Symbol,
            }),
            RawTokenKind::LexemeRun | RawTokenKind::NumeralLike => {
                split_declaration_run(&token.lexeme, token.span.start, &mut pieces);
            }
        }
    }
    pieces
}

fn split_declaration_run(run: &str, base: SourcePos, pieces: &mut Vec<DeclarationPiece>) {
    let mut cursor = 0;
    while cursor < run.len() {
        let rest = &run[cursor..];
        if let Some((symbol, len)) = structural_symbol_prefix(rest) {
            pieces.push(DeclarationPiece {
                text: symbol.to_owned(),
                span: SourceSpan::new(base + cursor, base + cursor + len),
                kind: DeclarationPieceKind::Symbol,
            });
            cursor += len;
            continue;
        }
        if let Some(len) = string_literal_piece_len(rest) {
            pieces.push(DeclarationPiece {
                text: rest[..len].to_owned(),
                span: SourceSpan::new(base + cursor, base + cursor + len),
                kind: DeclarationPieceKind::StringLiteral,
            });
            cursor += len;
            continue;
        }
        if let Some(len) = declaration_identifier_prefix_len(rest) {
            pieces.push(DeclarationPiece {
                text: rest[..len].to_owned(),
                span: SourceSpan::new(base + cursor, base + cursor + len),
                kind: DeclarationPieceKind::Word,
            });
            cursor += len;
            continue;
        }
        if let Some(len) = declaration_numeral_prefix_len(rest) {
            pieces.push(DeclarationPiece {
                text: rest[..len].to_owned(),
                span: SourceSpan::new(base + cursor, base + cursor + len),
                kind: DeclarationPieceKind::Numeral,
            });
            cursor += len;
            continue;
        }

        let len = declaration_symbol_prefix_len(rest);
        pieces.push(DeclarationPiece {
            text: rest[..len].to_owned(),
            span: SourceSpan::new(base + cursor, base + cursor + len),
            kind: DeclarationPieceKind::Symbol,
        });
        cursor += len;
    }
}

fn declaration_identifier_prefix_len(value: &str) -> Option<usize> {
    let mut chars = value.char_indices();
    let (_, first) = chars.next()?;
    if !is_identifier_start(first) {
        return None;
    }
    let mut end = first.len_utf8();
    for (index, ch) in chars {
        if !is_identifier_continue(ch) {
            break;
        }
        end = index + ch.len_utf8();
    }
    Some(end)
}

fn declaration_numeral_prefix_len(value: &str) -> Option<usize> {
    let mut end = 0;
    for (index, ch) in value.char_indices() {
        if !ch.is_ascii_digit() {
            break;
        }
        end = index + ch.len_utf8();
    }
    (end > 0).then_some(end)
}

fn declaration_symbol_prefix_len(value: &str) -> usize {
    let mut end = 0;
    for (index, ch) in value.char_indices() {
        if structural_symbol_prefix(&value[index..]).is_some()
            || ch == '"'
            || ch == '\''
            || is_identifier_start(ch)
            || ch.is_ascii_digit()
        {
            break;
        }
        end = index + ch.len_utf8();
    }
    if end == 0 {
        value
            .chars()
            .next()
            .map(char::len_utf8)
            .expect("declaration symbol prefix is called with non-empty input")
    } else {
        end
    }
}

fn structural_symbol_prefix(value: &str) -> Option<(&'static str, usize)> {
    const STRUCTURAL_SYMBOLS: &[&str] = &[
        "...", "->", ":=", "<>", ".{", ".*", ".=", "..", "@[", "[:", ":]", ":", ";", ",", "(", ")",
        "[", "]", "{", "}",
    ];
    STRUCTURAL_SYMBOLS
        .iter()
        .copied()
        .find(|symbol| value.starts_with(symbol))
        .map(|symbol| (symbol, symbol.len()))
}

fn string_literal_piece_len(value: &str) -> Option<usize> {
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    let mut escaped = false;
    for (index, ch) in value[quote.len_utf8()..].char_indices() {
        if escaped {
            escaped = false;
            continue;
        }
        if ch == '\\' {
            escaped = true;
            continue;
        }
        if ch == quote {
            return Some(quote.len_utf8() + index + ch.len_utf8());
        }
    }
    Some(quote.len_utf8())
}

fn unquote_string_literal(value: &str) -> Option<String> {
    let quote = value.chars().next()?;
    if quote != '"' && quote != '\'' {
        return None;
    }
    if !value.ends_with(quote) || value.len() < quote.len_utf8() * 2 {
        return None;
    }
    let body = &value[quote.len_utf8()..value.len() - quote.len_utf8()];
    let mut output = String::new();
    let mut escaped = false;
    for ch in body.chars() {
        if escaped {
            if !matches!(ch, '"' | '\'' | '\\') {
                return None;
            }
            output.push(ch);
            escaped = false;
        } else if ch == '\\' {
            escaped = true;
        } else {
            output.push(ch);
        }
    }
    (!escaped).then_some(output)
}

fn is_declaration_delimiter(value: &str) -> bool {
    matches!(
        value,
        ":" | ";" | "," | "(" | ")" | "[" | "]" | "{" | "}" | "->" | ":="
    )
}

fn is_declaration_trailing_keyword(value: &str) -> bool {
    matches!(
        value,
        "asymmetry"
            | "coherence"
            | "commutativity"
            | "compatibility"
            | "connectedness"
            | "consistency"
            | "existence"
            | "idempotence"
            | "involutiveness"
            | "irreflexivity"
            | "projectivity"
            | "reducibility"
            | "reduction"
            | "reflexivity"
            | "sethood"
            | "symmetry"
            | "uniqueness"
    )
}

fn is_trailing_proof_block_opener(value: &str) -> bool {
    matches!(value, "case" | "hereby" | "now" | "proof" | "suppose")
}

struct StableFingerprint {
    value: u64,
}

impl StableFingerprint {
    fn new() -> Self {
        Self {
            value: 0xcbf29ce484222325,
        }
    }

    fn write_str(&mut self, value: &str) {
        self.write_usize(value.len());
        for byte in value.as_bytes() {
            self.write_byte(*byte);
        }
    }

    fn write_usize(&mut self, value: usize) {
        self.write_u64(value as u64);
    }

    fn write_u64(&mut self, value: u64) {
        for byte in value.to_le_bytes() {
            self.write_byte(byte);
        }
    }

    fn write_byte(&mut self, byte: u8) {
        self.value ^= u64::from(byte);
        self.value = self.value.wrapping_mul(0x100000001b3);
    }

    fn finish(self) -> u64 {
        self.value
    }
}
