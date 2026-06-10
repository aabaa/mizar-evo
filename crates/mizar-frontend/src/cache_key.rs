use crate::lexical_env::LexicalEnvironmentFingerprint;
use crate::lexing::{LexicalByteRange, ParserLexContext, ParserLexMode, ParserLexingPlan};
use crate::parsing::{
    OperatorAssociativity, ParserCacheKeyVersion, ParserInputs, StringRequiredContext,
};
use crate::preprocess::PreprocessedSource;
use crate::source::SourceUnit;
use mizar_lexer::UserSymbolKind;
use mizar_session::{Edition, Hash, ModulePath, NormalizedPath, PackageId};
use std::sync::Arc;

pub const SOURCE_UNIT_CACHE_KEY_VERSION: &str = "mizar-frontend/source-unit-cache-key/v1";
pub const PREPROCESSED_SOURCE_CACHE_KEY_VERSION: &str =
    "mizar-frontend/preprocessed-source-cache-key/v1";
pub const ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION: &str =
    "mizar-frontend/active-lexical-environment-cache-key/v1";
pub const PARSER_LEXING_PLAN_CACHE_KEY_VERSION: &str =
    "mizar-frontend/parser-lexing-plan/position-sensitive-v1";
pub const TOKEN_STREAM_CACHE_KEY_VERSION: &str = "mizar-frontend/token-stream-cache-key/v1";
pub const SURFACE_AST_CACHE_KEY_VERSION: &str = "mizar-frontend/surface-ast-cache-key/v1";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendCacheKeys {
    pub source: SourceUnitCacheKey,
    pub preprocessed: PreprocessedSourceCacheKey,
    pub active_lexical_environment: ActiveLexicalEnvironmentCacheKey,
    pub tokens: TokenStreamCacheKey,
    pub ast: Option<SurfaceAstCacheKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUnitCacheKey {
    pub version: Arc<str>,
    pub package_id: PackageId,
    pub module_path: ModulePath,
    pub normalized_path: NormalizedPath,
    pub source_hash: Hash,
    pub edition: Edition,
}

impl SourceUnitCacheKey {
    pub fn from_source(source: &SourceUnit) -> Self {
        Self {
            version: Arc::from(SOURCE_UNIT_CACHE_KEY_VERSION),
            package_id: source.package_id.clone(),
            module_path: source.module_path.clone(),
            normalized_path: source.normalized_path.clone(),
            source_hash: source.source_hash,
            edition: source.edition.clone(),
        }
    }

    pub fn stable_hash(&self) -> Hash {
        let mut hasher = stable_hasher("source-unit");
        write_str(&mut hasher, self.version.as_ref());
        write_str(&mut hasher, self.package_id.as_str());
        write_str(&mut hasher, self.module_path.as_str());
        write_str(&mut hasher, self.normalized_path.as_str());
        write_hash(&mut hasher, self.source_hash);
        write_str(&mut hasher, self.edition.as_str());
        finish_hash(hasher)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedSourceCacheKey {
    pub version: Arc<str>,
    pub source_hash: Hash,
}

impl PreprocessedSourceCacheKey {
    pub fn from_source(source: &SourceUnit) -> Self {
        Self {
            version: Arc::from(PREPROCESSED_SOURCE_CACHE_KEY_VERSION),
            source_hash: source.source_hash,
        }
    }

    pub fn stable_hash(&self) -> Hash {
        let mut hasher = stable_hasher("preprocessed-source");
        write_str(&mut hasher, self.version.as_ref());
        write_hash(&mut hasher, self.source_hash);
        finish_hash(hasher)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveLexicalEnvironmentCacheKey {
    pub version: Arc<str>,
    pub fingerprint: LexicalEnvironmentFingerprint,
}

impl ActiveLexicalEnvironmentCacheKey {
    pub fn new(fingerprint: LexicalEnvironmentFingerprint) -> Self {
        Self {
            version: Arc::from(ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION),
            fingerprint,
        }
    }

    pub fn stable_hash(&self) -> Hash {
        let mut hasher = stable_hasher("active-lexical-environment");
        write_str(&mut hasher, self.version.as_ref());
        write_u64(&mut hasher, self.fingerprint.get());
        finish_hash(hasher)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserLexingPlanCacheKey {
    pub version: Arc<str>,
    pub default_context: ParserLexContext,
    pub contexts: Vec<ParserLexingPlanContextCacheKey>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserLexingPlanContextCacheKey {
    pub range: LexicalByteRange,
    pub context: ParserLexContext,
}

impl ParserLexingPlanCacheKey {
    pub fn current() -> Self {
        Self::from_plan(&ParserLexingPlan::uniform(ParserLexContext::general()))
    }

    pub fn from_plan(plan: &ParserLexingPlan) -> Self {
        Self {
            version: Arc::from(PARSER_LEXING_PLAN_CACHE_KEY_VERSION),
            default_context: plan.default_context,
            contexts: plan
                .contexts
                .iter()
                .map(|context| ParserLexingPlanContextCacheKey {
                    range: context.range,
                    context: context.context,
                })
                .collect(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStreamCacheKey {
    pub version: Arc<str>,
    pub lexical_hash: Hash,
    pub active_lexical_environment: LexicalEnvironmentFingerprint,
    pub parser_context: ParserLexContext,
    pub parser_lexing_plan: ParserLexingPlanCacheKey,
}

impl TokenStreamCacheKey {
    pub fn new(
        preprocessed: &PreprocessedSource,
        active_lexical_environment: LexicalEnvironmentFingerprint,
        parser_context: ParserLexContext,
        parser_lexing_plan: ParserLexingPlanCacheKey,
    ) -> Self {
        Self {
            version: Arc::from(TOKEN_STREAM_CACHE_KEY_VERSION),
            lexical_hash: preprocessed.lexical_hash,
            active_lexical_environment,
            parser_context,
            parser_lexing_plan,
        }
    }

    pub fn stable_hash(&self) -> Hash {
        let mut hasher = stable_hasher("token-stream");
        write_str(&mut hasher, self.version.as_ref());
        write_hash(&mut hasher, self.lexical_hash);
        write_u64(&mut hasher, self.active_lexical_environment.get());
        write_parser_lex_context(&mut hasher, self.parser_context);
        write_str(&mut hasher, self.parser_lexing_plan.version.as_ref());
        write_parser_lex_context(&mut hasher, self.parser_lexing_plan.default_context);
        write_u64(&mut hasher, self.parser_lexing_plan.contexts.len() as u64);
        for context in &self.parser_lexing_plan.contexts {
            write_u64(&mut hasher, context.range.start as u64);
            write_u64(&mut hasher, context.range.end as u64);
            write_parser_lex_context(&mut hasher, context.context);
        }
        finish_hash(hasher)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceAstCacheKey {
    pub version: Arc<str>,
    pub token_stream_hash: Hash,
    pub parser_version: ParserCacheKeyVersion,
    pub parser_inputs_hash: Hash,
    pub edition: Edition,
}

impl SurfaceAstCacheKey {
    pub fn new(
        token_stream_hash: Hash,
        parser_version: ParserCacheKeyVersion,
        parser_inputs: &ParserInputs,
    ) -> Self {
        Self {
            version: Arc::from(SURFACE_AST_CACHE_KEY_VERSION),
            token_stream_hash,
            parser_version,
            parser_inputs_hash: parser_inputs_hash(parser_inputs),
            edition: parser_inputs.edition.clone(),
        }
    }

    pub fn stable_hash(&self) -> Hash {
        let mut hasher = stable_hasher("surface-ast");
        write_str(&mut hasher, self.version.as_ref());
        write_hash(&mut hasher, self.token_stream_hash);
        write_str(&mut hasher, self.parser_version.version.as_ref());
        write_hash(&mut hasher, self.parser_inputs_hash);
        write_str(&mut hasher, self.edition.as_str());
        finish_hash(hasher)
    }
}

fn stable_hasher(label: &str) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    write_str(&mut hasher, "mizar-frontend/cache-key/v1");
    write_str(&mut hasher, label);
    hasher
}

fn write_parser_lex_context(hasher: &mut blake3::Hasher, context: ParserLexContext) {
    write_parser_lex_mode(hasher, context.mode());
    write_user_symbol_kind_set(hasher, context);
}

fn write_parser_lex_mode(hasher: &mut blake3::Hasher, mode: ParserLexMode) {
    let mode_key = match mode {
        ParserLexMode::General => "general",
        ParserLexMode::IdentifierRequired => "identifier_required",
        ParserLexMode::Symbolic => "symbolic",
        ParserLexMode::StringRequired => "string_required",
        ParserLexMode::NamespacePath => "namespace_path",
        ParserLexMode::Recovery => "recovery",
        _ => {
            write_str(hasher, &format!("unknown:{mode:?}"));
            return;
        }
    };
    write_str(hasher, mode_key);
}

fn write_user_symbol_kind_set(hasher: &mut blake3::Hasher, context: ParserLexContext) {
    write_u64(hasher, known_user_symbol_kind_set_bits(context));
    write_str(hasher, &format!("{:?}", context.user_symbol_kinds()));
}

fn known_user_symbol_kind_set_bits(context: ParserLexContext) -> u64 {
    [
        UserSymbolKind::Functor,
        UserSymbolKind::Predicate,
        UserSymbolKind::Mode,
        UserSymbolKind::Attribute,
        UserSymbolKind::Structure,
        UserSymbolKind::Selector,
        UserSymbolKind::Constructor,
    ]
    .into_iter()
    .enumerate()
    .fold(0, |bits, (index, kind)| {
        if context.user_symbol_kinds().contains(kind) {
            bits | (1 << index)
        } else {
            bits
        }
    })
}

pub fn parser_inputs_hash(inputs: &ParserInputs) -> Hash {
    let mut hasher = stable_hasher("parser-inputs");
    write_str(&mut hasher, inputs.edition.as_str());
    write_string_required_context(&mut hasher, inputs.string_required_positions);
    write_u64(&mut hasher, inputs.operator_fixity.entries.len() as u64);
    for entry in &inputs.operator_fixity.entries {
        write_str(&mut hasher, entry.symbol_id.as_str());
        write_str(&mut hasher, entry.spelling.as_ref());
        hasher.update(&[entry.precedence]);
        write_operator_associativity(&mut hasher, entry.associativity);
    }
    finish_hash(hasher)
}

fn write_string_required_context(hasher: &mut blake3::Hasher, context: StringRequiredContext) {
    let context_key = match context {
        StringRequiredContext::None => "none",
        StringRequiredContext::PositionSensitive => "position_sensitive",
        StringRequiredContext::UniformForTest => "uniform_for_test",
    };
    write_str(hasher, context_key);
}

fn write_operator_associativity(hasher: &mut blake3::Hasher, associativity: OperatorAssociativity) {
    let associativity_key = match associativity {
        OperatorAssociativity::Left => "left",
        OperatorAssociativity::Right => "right",
        OperatorAssociativity::NonAssociative => "non_associative",
    };
    write_str(hasher, associativity_key);
}

fn write_str(hasher: &mut blake3::Hasher, value: &str) {
    hasher.update(&(value.len() as u64).to_le_bytes());
    hasher.update(value.as_bytes());
}

fn write_hash(hasher: &mut blake3::Hasher, value: Hash) {
    hasher.update(value.as_bytes());
}

fn write_u64(hasher: &mut blake3::Hasher, value: u64) {
    hasher.update(&value.to_le_bytes());
}

fn finish_hash(hasher: blake3::Hasher) -> Hash {
    Hash::from_bytes(*hasher.finalize().as_bytes())
}

#[cfg(test)]
mod tests {
    use super::{
        ActiveLexicalEnvironmentCacheKey, ParserLexingPlanCacheKey, PreprocessedSourceCacheKey,
        SourceUnitCacheKey, SurfaceAstCacheKey, TokenStreamCacheKey,
    };
    use crate::lexical_env::LexicalEnvironmentFingerprint;
    use crate::lexing::{
        LexicalByteRange, ParserLexContext, ParserLexingPlan, ParserLexingPlanContext,
    };
    use crate::parsing::{
        OperatorAssociativity, OperatorFixityEntry, OperatorFixityTable, ParserCacheKeyVersion,
        ParserInputs, StringRequiredContext,
    };
    use crate::preprocess::{LexicalSourceMap, LexicalText, PreprocessedSource, lexical_hash};
    use crate::source::SourceUnit;
    use mizar_lexer::SourcePreprocessMap;
    use mizar_session::{
        BuildSnapshotId, Edition, Hash, InMemorySessionIdAllocator, LineMap, ModulePath,
        NormalizedPath, PackageId, SessionIdAllocator, SourceId, SourceOrigin,
    };
    use std::fs;
    use std::path::PathBuf;
    use std::sync::Arc;

    #[test]
    fn source_cache_key_uses_content_identity_not_freshness_metadata() {
        let first = source_unit(SourceOrigin::OpenBuffer { version: 1 }, "display-one.miz");
        let second = source_unit(SourceOrigin::OpenBuffer { version: 2 }, "display-two.miz");

        assert_eq!(
            SourceUnitCacheKey::from_source(&first),
            SourceUnitCacheKey::from_source(&second),
            "open-buffer versions and display paths are freshness/diagnostic metadata"
        );
        assert_eq!(
            SourceUnitCacheKey::from_source(&first).stable_hash(),
            SourceUnitCacheKey::from_source(&second).stable_hash()
        );
    }

    #[test]
    fn source_cache_key_changes_for_each_content_identity_field() {
        let base = SourceUnitCacheKey::from_source(&source_unit(
            SourceOrigin::OpenBuffer { version: 1 },
            "display.miz",
        ));
        let changed_package = SourceUnitCacheKey::from_source(&SourceUnit {
            package_id: PackageId::new("other-pkg"),
            ..source_unit(SourceOrigin::OpenBuffer { version: 1 }, "display.miz")
        });
        let changed_module = SourceUnitCacheKey::from_source(&SourceUnit {
            module_path: ModulePath::new("pkg.beta"),
            ..source_unit(SourceOrigin::OpenBuffer { version: 1 }, "display.miz")
        });
        let changed_path = SourceUnitCacheKey::from_source(&SourceUnit {
            normalized_path: normalized_path("src/beta.miz"),
            ..source_unit(SourceOrigin::OpenBuffer { version: 1 }, "display.miz")
        });
        let changed_hash = SourceUnitCacheKey::from_source(&SourceUnit {
            source_hash: hash(4),
            ..source_unit(SourceOrigin::OpenBuffer { version: 1 }, "display.miz")
        });
        let changed_edition = SourceUnitCacheKey::from_source(&SourceUnit {
            edition: Edition::new("2027"),
            ..source_unit(SourceOrigin::OpenBuffer { version: 1 }, "display.miz")
        });

        for changed in [
            changed_package,
            changed_module,
            changed_path,
            changed_hash,
            changed_edition,
        ] {
            assert_ne!(base, changed);
            assert_ne!(base.stable_hash(), changed.stable_hash());
        }
    }

    #[test]
    fn comment_only_edits_invalidate_preprocess_but_preserve_token_and_ast_keys() {
        let first_source = source_unit_with_hash(hash(1));
        let second_source = source_unit_with_hash(hash(2));
        let first_preprocessed = preprocessed("definition\nend;\n");
        let second_preprocessed = preprocessed("definition\nend;\n");
        let fingerprint = LexicalEnvironmentFingerprint::new(7);

        let first_token_key = TokenStreamCacheKey::new(
            &first_preprocessed,
            fingerprint,
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::current(),
        );
        let second_token_key = TokenStreamCacheKey::new(
            &second_preprocessed,
            fingerprint,
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::current(),
        );

        assert_ne!(
            PreprocessedSourceCacheKey::from_source(&first_source),
            PreprocessedSourceCacheKey::from_source(&second_source),
            "preprocessing is keyed by source_hash"
        );
        assert_eq!(
            first_token_key, second_token_key,
            "tokenization can be reused when lexical text, environment, and parser context are unchanged"
        );
        assert_eq!(
            first_token_key.stable_hash(),
            second_token_key.stable_hash()
        );

        let parser_version = ParserCacheKeyVersion::new("parser/v1");
        assert_eq!(
            SurfaceAstCacheKey::new(
                first_token_key.stable_hash(),
                parser_version.clone(),
                &parser_inputs(Edition::new("2026")),
            ),
            SurfaceAstCacheKey::new(
                second_token_key.stable_hash(),
                parser_version,
                &parser_inputs(Edition::new("2026")),
            )
        );
    }

    #[test]
    fn import_dependency_and_parser_context_changes_invalidate_token_key() {
        let base_preprocessed = preprocessed("import alpha;\ndefinition\nend;\n");
        let base = TokenStreamCacheKey::new(
            &base_preprocessed,
            LexicalEnvironmentFingerprint::new(10),
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::current(),
        );
        let changed_environment = TokenStreamCacheKey::new(
            &base_preprocessed,
            LexicalEnvironmentFingerprint::new(11),
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::current(),
        );
        let changed_parser_context = TokenStreamCacheKey::new(
            &base_preprocessed,
            LexicalEnvironmentFingerprint::new(10),
            ParserLexContext::string_required(),
            ParserLexingPlanCacheKey::current(),
        );
        let changed_parser_plan = TokenStreamCacheKey::new(
            &base_preprocessed,
            LexicalEnvironmentFingerprint::new(10),
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::from_plan(&ParserLexingPlan::new(
                ParserLexContext::general(),
                vec![ParserLexingPlanContext::new(
                    LexicalByteRange::new(0, 6),
                    ParserLexContext::string_required(),
                )],
            )),
        );
        let changed_parser_plan_range = TokenStreamCacheKey::new(
            &base_preprocessed,
            LexicalEnvironmentFingerprint::new(10),
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::from_plan(&ParserLexingPlan::new(
                ParserLexContext::general(),
                vec![ParserLexingPlanContext::new(
                    LexicalByteRange::new(1, 6),
                    ParserLexContext::string_required(),
                )],
            )),
        );
        let changed_parser_plan_context = TokenStreamCacheKey::new(
            &base_preprocessed,
            LexicalEnvironmentFingerprint::new(10),
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::from_plan(&ParserLexingPlan::new(
                ParserLexContext::general(),
                vec![ParserLexingPlanContext::new(
                    LexicalByteRange::new(0, 6),
                    ParserLexContext::general().with_user_symbol_kinds(
                        mizar_lexer::UserSymbolKindSet::only(
                            mizar_lexer::UserSymbolKind::Predicate,
                        ),
                    ),
                )],
            )),
        );
        let changed_import_text = TokenStreamCacheKey::new(
            &preprocessed("import beta;\ndefinition\nend;\n"),
            LexicalEnvironmentFingerprint::new(10),
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::current(),
        );

        assert_ne!(
            ActiveLexicalEnvironmentCacheKey::new(LexicalEnvironmentFingerprint::new(10)),
            ActiveLexicalEnvironmentCacheKey::new(LexicalEnvironmentFingerprint::new(11))
        );
        assert_ne!(base, changed_import_text);
        assert_ne!(base.stable_hash(), changed_import_text.stable_hash());
        assert_ne!(base, changed_environment);
        assert_ne!(base.stable_hash(), changed_environment.stable_hash());
        assert_ne!(base, changed_parser_context);
        assert_ne!(base.stable_hash(), changed_parser_context.stable_hash());
        assert_ne!(base, changed_parser_plan);
        assert_ne!(base.stable_hash(), changed_parser_plan.stable_hash());
        assert_ne!(changed_parser_plan, changed_parser_plan_range);
        assert_ne!(
            changed_parser_plan.stable_hash(),
            changed_parser_plan_range.stable_hash()
        );
        assert_ne!(changed_parser_plan, changed_parser_plan_context);
        assert_ne!(
            changed_parser_plan.stable_hash(),
            changed_parser_plan_context.stable_hash()
        );
    }

    #[test]
    fn ast_cache_key_changes_with_parser_version_or_edition() {
        let token_stream_hash = hash(9);
        let base = SurfaceAstCacheKey::new(
            token_stream_hash,
            ParserCacheKeyVersion::new("parser/v1"),
            &parser_inputs(Edition::new("2026")),
        );
        let changed_token_stream = SurfaceAstCacheKey::new(
            hash(10),
            ParserCacheKeyVersion::new("parser/v1"),
            &parser_inputs(Edition::new("2026")),
        );
        let changed_parser = SurfaceAstCacheKey::new(
            token_stream_hash,
            ParserCacheKeyVersion::new("parser/v2"),
            &parser_inputs(Edition::new("2026")),
        );
        let changed_edition = SurfaceAstCacheKey::new(
            token_stream_hash,
            ParserCacheKeyVersion::new("parser/v1"),
            &parser_inputs(Edition::new("2027")),
        );
        let changed_parser_inputs = SurfaceAstCacheKey::new(
            token_stream_hash,
            ParserCacheKeyVersion::new("parser/v1"),
            &parser_inputs_with_fixity(Edition::new("2026")),
        );
        let changed_string_context = SurfaceAstCacheKey::new(
            token_stream_hash,
            ParserCacheKeyVersion::new("parser/v1"),
            &ParserInputs::new(
                Edition::new("2026"),
                OperatorFixityTable::empty(),
                StringRequiredContext::UniformForTest,
            ),
        );

        assert_ne!(base, changed_token_stream);
        assert_ne!(base.stable_hash(), changed_token_stream.stable_hash());
        assert_ne!(base, changed_parser);
        assert_ne!(base.stable_hash(), changed_parser.stable_hash());
        assert_ne!(base, changed_edition);
        assert_ne!(base.stable_hash(), changed_edition.stable_hash());
        assert_ne!(base, changed_parser_inputs);
        assert_ne!(base.stable_hash(), changed_parser_inputs.stable_hash());
        assert_ne!(base, changed_string_context);
        assert_ne!(base.stable_hash(), changed_string_context.stable_hash());
    }

    fn source_unit(origin: SourceOrigin, file_path: &str) -> SourceUnit {
        SourceUnit {
            source_id: source_id(1),
            package_id: PackageId::new("pkg"),
            module_path: ModulePath::new("pkg.alpha"),
            normalized_path: normalized_path("src/alpha.miz"),
            edition: Edition::new("2026"),
            file_path: PathBuf::from(file_path),
            source_text: Arc::from("definition\nend;\n"),
            source_hash: hash(3),
            line_map: LineMap::new(source_id(1), "definition\nend;\n"),
            loading_map: None,
            origin,
            generated_anchor: None,
        }
    }

    fn source_unit_with_hash(source_hash: Hash) -> SourceUnit {
        SourceUnit {
            source_hash,
            ..source_unit(SourceOrigin::Disk, "src/alpha.miz")
        }
    }

    fn preprocessed(text: &str) -> PreprocessedSource {
        let lexical_text = LexicalText {
            text: Arc::from(text),
        };
        PreprocessedSource {
            source_id: source_id(1),
            lexical_hash: lexical_hash(&lexical_text),
            lexical_text,
            comments: Vec::new(),
            doc_comments: Vec::new(),
            import_stubs: Vec::new(),
            source_map: LexicalSourceMap {
                source_id: source_id(1),
                lexical_text_len: text.len(),
                preprocess_map: SourcePreprocessMap {
                    segments: Vec::new(),
                },
            },
            diagnostics: Vec::new(),
        }
    }

    fn source_id(value: u8) -> SourceId {
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot_id(value))
            .unwrap()
    }

    fn snapshot_id(value: u8) -> BuildSnapshotId {
        let hex = format!("{value:02x}").repeat(32);
        BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{hex}"
        ))
        .unwrap()
    }

    fn hash(value: u8) -> Hash {
        Hash::from_bytes([value; Hash::BYTE_LEN])
    }

    fn normalized_path(path: &str) -> NormalizedPath {
        let root = std::env::temp_dir().join(format!(
            "mizar-frontend-cache-key-test-{}",
            std::process::id()
        ));
        let source_path = root.join(path);
        if let Some(parent) = source_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(&source_path, "definition\nend;\n").unwrap();
        mizar_session::normalize_path(&root, &source_path).unwrap()
    }

    fn parser_inputs(edition: Edition) -> ParserInputs {
        ParserInputs::new(
            edition,
            OperatorFixityTable::empty(),
            StringRequiredContext::None,
        )
    }

    fn parser_inputs_with_fixity(edition: Edition) -> ParserInputs {
        ParserInputs::new(
            edition,
            OperatorFixityTable {
                entries: vec![OperatorFixityEntry {
                    symbol_id: crate::lexical_env::SymbolId::new("fixture.plus"),
                    spelling: Arc::from("+"),
                    precedence: 50,
                    associativity: OperatorAssociativity::Left,
                }],
            },
            StringRequiredContext::None,
        )
    }
}
