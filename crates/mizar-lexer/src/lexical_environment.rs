use crate::raw_lexer::is_user_symbol_spelling;
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
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveLexicalEnvironment {
    pub reserved_words: ReservedWordTable,
    pub reserved_symbols: ReservedSymbolTable,
    pub user_symbols: UserSymbolIndex,
    pub fingerprint: LexicalEnvironmentFingerprint,
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

            user_symbols.insert(UserSymbolCandidate {
                spelling: exported.spelling.clone(),
                symbol_id: exported.symbol_id.clone(),
                source_module: exported.source_module.clone(),
                imported_module: import.module_id.clone(),
                import_ordinal,
                export_rank: exported.export_rank,
                kind: exported.kind,
                arity: exported.arity,
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

    pub fn longest_user_symbol_at(&self, input: &str, start: usize) -> Vec<UserSymbolCandidate> {
        self.user_symbols.longest_at(input, start)
    }
}
impl UserSymbolIndex {
    pub fn visible_symbol(&self, spelling: &str) -> Option<&UserSymbolCandidate> {
        self.symbols_by_spelling
            .get(spelling)
            .and_then(|candidates| candidates.last())
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
    Ok(())
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
