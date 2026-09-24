//! Layered content cache keys for frontend pipeline outputs.
//!
//! Canonical behavior is specified in the
//! [cache-key design spec](../../../../doc/design/mizar-frontend/en/cache_key.md).

use crate::lexical_env::LexicalEnvironmentFingerprint;
use crate::lexing::{
    LexicalByteRange, ParserLexContext, ParserLexMode, ParserLexingPlan, decode_context,
    encode_context,
};
use crate::parsing::{
    OperatorAssociativity, OperatorFixity, ParserCacheKeyVersion, ParserInputs,
    StringRequiredContext,
};
use crate::preprocess::PreprocessedSource;
use crate::source::SourceUnit;
use mizar_lexer::UserSymbolKind;
use mizar_session::{Edition, Hash, ModulePath, NormalizedPath, PackageId};
use serde_json::{Value, json};
use std::sync::Arc;

const CACHE_KEYS_SCHEMA: &str = "mizar-frontend/cache-keys/v1";
const CACHE_KEYS_MAX_BYTES: usize = 16 * 1024 * 1024;

/// Version tag for source-unit cache keys.
pub const SOURCE_UNIT_CACHE_KEY_VERSION: &str = "mizar-frontend/source-unit-cache-key/v1";
/// Version tag for preprocessed-source cache keys.
pub const PREPROCESSED_SOURCE_CACHE_KEY_VERSION: &str =
    "mizar-frontend/preprocessed-source-cache-key/v1";
/// Version tag for active lexical-environment cache keys.
pub const ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION: &str =
    "mizar-frontend/active-lexical-environment-cache-key/v1";
/// Version tag for parser lexing-plan cache keys.
pub const PARSER_LEXING_PLAN_CACHE_KEY_VERSION: &str =
    "mizar-frontend/parser-lexing-plan/position-sensitive-v1";
/// Version tag for token-stream cache keys.
pub const TOKEN_STREAM_CACHE_KEY_VERSION: &str = "mizar-frontend/token-stream-cache-key/v3";
/// Version tag for surface-AST cache keys.
pub const SURFACE_AST_CACHE_KEY_VERSION: &str = "mizar-frontend/surface-ast-cache-key/v1";

/// Cache keys emitted with a complete frontend output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FrontendCacheKeys {
    /// Source-unit cache key.
    pub source: SourceUnitCacheKey,
    /// Preprocessed-source cache key.
    pub preprocessed: PreprocessedSourceCacheKey,
    /// Active lexical-environment cache key.
    pub active_lexical_environment: ActiveLexicalEnvironmentCacheKey,
    /// Token-stream cache key.
    pub tokens: TokenStreamCacheKey,
    /// Surface-AST cache key, absent when parsing produced no AST.
    pub ast: Option<SurfaceAstCacheKey>,
}

impl FrontendCacheKeys {
    /// Encodes the complete retained cache-key bundle as bounded canonical JSON.
    pub fn canonical_bytes(&self) -> Option<Vec<u8>> {
        let plan = &self.tokens.parser_lexing_plan;
        let contexts = plan
            .contexts
            .iter()
            .map(|entry| {
                (entry.range.start <= entry.range.end).then_some(())?;
                Some(json!([
                    entry.range.start,
                    entry.range.end,
                    encode_context(entry.context)?
                ]))
            })
            .collect::<Option<Vec<_>>>()?;
        let ast = match &self.ast {
            Some(ast) => json!([
                ast.version.as_ref(),
                ast.token_stream_hash.as_bytes(),
                ast.parser_version.version.as_ref(),
                ast.parser_inputs_hash.as_bytes(),
                ast.edition.as_str()
            ]),
            None => Value::Null,
        };
        let bytes = serde_json::to_vec(&json!([
            CACHE_KEYS_SCHEMA,
            [
                self.source.version.as_ref(),
                self.source.package_id.as_str(),
                self.source.module_path.as_str(),
                self.source.normalized_path.as_str(),
                self.source.source_hash.as_bytes(),
                self.source.edition.as_str()
            ],
            [
                self.preprocessed.version.as_ref(),
                self.preprocessed.source_hash.as_bytes()
            ],
            [
                self.active_lexical_environment.version.as_ref(),
                self.active_lexical_environment.fingerprint.get()
            ],
            [
                self.tokens.version.as_ref(),
                self.tokens.lexical_hash.as_bytes(),
                self.tokens.active_lexical_environment.get(),
                encode_context(self.tokens.parser_context)?,
                [
                    plan.version.as_ref(),
                    encode_context(plan.default_context)?,
                    contexts
                ]
            ],
            ast
        ]))
        .ok()?;
        (bytes.len() <= CACHE_KEYS_MAX_BYTES).then_some(bytes)
    }

    /// Decodes canonical storage bound to the caller's normalized path.
    pub fn from_canonical_bytes(bytes: &[u8], normalized_path: &NormalizedPath) -> Option<Self> {
        if bytes.len() > CACHE_KEYS_MAX_BYTES {
            return None;
        }
        let value: Value = serde_json::from_slice(bytes).ok()?;
        let [schema, source, preprocessed, active, tokens, ast] = fields::<6>(&value)?;
        if schema.as_str()? != CACHE_KEYS_SCHEMA {
            return None;
        }
        let [
            source_version,
            package_id,
            module_path,
            path,
            source_hash,
            source_edition,
        ] = fields::<6>(source)?;
        if path.as_str()? != normalized_path.as_str() {
            return None;
        }
        let source = SourceUnitCacheKey {
            version: Arc::from(source_version.as_str()?),
            package_id: PackageId::new(package_id.as_str()?),
            module_path: ModulePath::new(module_path.as_str()?),
            normalized_path: normalized_path.clone(),
            source_hash: decode_hash(source_hash)?,
            edition: Edition::new(source_edition.as_str()?),
        };
        let [preprocessed_version, preprocessed_hash] = fields::<2>(preprocessed)?;
        let preprocessed = PreprocessedSourceCacheKey {
            version: Arc::from(preprocessed_version.as_str()?),
            source_hash: decode_hash(preprocessed_hash)?,
        };
        let [active_version, active_fingerprint] = fields::<2>(active)?;
        let active_lexical_environment = ActiveLexicalEnvironmentCacheKey {
            version: Arc::from(active_version.as_str()?),
            fingerprint: LexicalEnvironmentFingerprint::new(active_fingerprint.as_u64()?),
        };
        let [
            token_version,
            lexical_hash,
            token_fingerprint,
            parser_context,
            plan,
        ] = fields::<5>(tokens)?;
        let [plan_version, default_context, contexts] = fields::<3>(plan)?;
        let tokens = TokenStreamCacheKey {
            version: Arc::from(token_version.as_str()?),
            lexical_hash: decode_hash(lexical_hash)?,
            active_lexical_environment: LexicalEnvironmentFingerprint::new(
                token_fingerprint.as_u64()?,
            ),
            parser_context: decode_context(parser_context)?,
            parser_lexing_plan: ParserLexingPlanCacheKey {
                version: Arc::from(plan_version.as_str()?),
                default_context: decode_context(default_context)?,
                contexts: contexts
                    .as_array()?
                    .iter()
                    .map(|entry| {
                        let [start, end, context] = fields::<3>(entry)?;
                        let start = usize::try_from(start.as_u64()?).ok()?;
                        let end = usize::try_from(end.as_u64()?).ok()?;
                        (start <= end).then_some(ParserLexingPlanContextCacheKey {
                            range: LexicalByteRange { start, end },
                            context: decode_context(context)?,
                        })
                    })
                    .collect::<Option<Vec<_>>>()?,
            },
        };
        let ast = if ast.is_null() {
            None
        } else {
            let [
                version,
                token_stream_hash,
                parser_version,
                parser_inputs_hash,
                edition,
            ] = fields::<5>(ast)?;
            Some(SurfaceAstCacheKey {
                version: Arc::from(version.as_str()?),
                token_stream_hash: decode_hash(token_stream_hash)?,
                parser_version: ParserCacheKeyVersion::new(parser_version.as_str()?),
                parser_inputs_hash: decode_hash(parser_inputs_hash)?,
                edition: Edition::new(edition.as_str()?),
            })
        };
        let keys = Self {
            source,
            preprocessed,
            active_lexical_environment,
            tokens,
            ast,
        };
        (keys.canonical_bytes()?.as_slice() == bytes).then_some(keys)
    }
}

fn fields<const N: usize>(value: &Value) -> Option<&[Value; N]> {
    value.as_array()?.as_slice().try_into().ok()
}

fn decode_hash(value: &Value) -> Option<Hash> {
    let values = fields::<{ Hash::BYTE_LEN }>(value)?;
    let mut bytes = [0; Hash::BYTE_LEN];
    for (byte, value) in bytes.iter_mut().zip(values) {
        *byte = u8::try_from(value.as_u64()?).ok()?;
    }
    Some(Hash::from_bytes(bytes))
}

/// Content key for source-unit identity and text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceUnitCacheKey {
    /// Version tag for this key shape.
    pub version: Arc<str>,
    /// Package id of the source unit.
    pub package_id: PackageId,
    /// Module path of the source unit.
    pub module_path: ModulePath,
    /// Normalized source path.
    pub normalized_path: NormalizedPath,
    /// Hash of loaded source text.
    pub source_hash: Hash,
    /// Source edition.
    pub edition: Edition,
}

impl SourceUnitCacheKey {
    /// Builds a source-unit cache key from a source unit.
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

    /// Computes the stable hash for this cache key.
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

/// Content key for preprocessing output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PreprocessedSourceCacheKey {
    /// Version tag for this key shape.
    pub version: Arc<str>,
    /// Hash of loaded source text.
    pub source_hash: Hash,
}

impl PreprocessedSourceCacheKey {
    /// Builds a preprocessed-source cache key from a source unit.
    pub fn from_source(source: &SourceUnit) -> Self {
        Self {
            version: Arc::from(PREPROCESSED_SOURCE_CACHE_KEY_VERSION),
            source_hash: source.source_hash,
        }
    }

    /// Computes the stable hash for this cache key.
    pub fn stable_hash(&self) -> Hash {
        let mut hasher = stable_hasher("preprocessed-source");
        write_str(&mut hasher, self.version.as_ref());
        write_hash(&mut hasher, self.source_hash);
        finish_hash(hasher)
    }
}

/// Content key for an active lexical environment.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActiveLexicalEnvironmentCacheKey {
    /// Version tag for this key shape.
    pub version: Arc<str>,
    /// Fingerprint of the active lexical environment.
    pub fingerprint: LexicalEnvironmentFingerprint,
}

impl ActiveLexicalEnvironmentCacheKey {
    /// Creates an active lexical-environment cache key.
    pub fn new(fingerprint: LexicalEnvironmentFingerprint) -> Self {
        Self {
            version: Arc::from(ACTIVE_LEXICAL_ENVIRONMENT_CACHE_KEY_VERSION),
            fingerprint,
        }
    }

    /// Computes the stable hash for this cache key.
    pub fn stable_hash(&self) -> Hash {
        let mut hasher = stable_hasher("active-lexical-environment");
        write_str(&mut hasher, self.version.as_ref());
        write_u64(&mut hasher, self.fingerprint.get());
        finish_hash(hasher)
    }
}

/// Content key for a parser lexing plan.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserLexingPlanCacheKey {
    /// Version tag for this key shape.
    pub version: Arc<str>,
    /// Default parser lexing context.
    pub default_context: ParserLexContext,
    /// Position-sensitive context overrides.
    pub contexts: Vec<ParserLexingPlanContextCacheKey>,
}

/// Cache-key representation of one parser lexing-plan context override.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParserLexingPlanContextCacheKey {
    /// Lexical byte range covered by this context.
    pub range: LexicalByteRange,
    /// Parser lexing context active in the range.
    pub context: ParserLexContext,
}

impl ParserLexingPlanCacheKey {
    /// Returns the current uniform general parser lexing-plan key.
    pub fn current() -> Self {
        Self::from_plan(&ParserLexingPlan::uniform(ParserLexContext::general()))
    }

    /// Builds a cache key from a parser lexing plan.
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

/// Content key for token-stream output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TokenStreamCacheKey {
    /// Version tag for this key shape.
    pub version: Arc<str>,
    /// Stable hash of lexical text.
    pub lexical_hash: Hash,
    /// Fingerprint of the active lexical environment.
    pub active_lexical_environment: LexicalEnvironmentFingerprint,
    /// Default parser lexing context used for tokenization.
    pub parser_context: ParserLexContext,
    /// Position-sensitive parser lexing-plan key.
    pub parser_lexing_plan: ParserLexingPlanCacheKey,
}

impl TokenStreamCacheKey {
    /// Creates a token-stream cache key from tokenization inputs.
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

    /// Computes the stable hash for this cache key.
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

/// Content key for surface-AST output.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SurfaceAstCacheKey {
    /// Version tag for this key shape.
    pub version: Arc<str>,
    /// Stable hash of the token-stream cache key.
    pub token_stream_hash: Hash,
    /// Parser seam cache-key version.
    pub parser_version: ParserCacheKeyVersion,
    /// Stable hash of parser inputs.
    pub parser_inputs_hash: Hash,
    /// Source edition used for parsing.
    pub edition: Edition,
}

impl SurfaceAstCacheKey {
    /// Creates a surface-AST cache key from parser inputs.
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

    /// Computes the stable hash for this cache key.
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

/// Computes a stable hash for parser inputs.
pub fn parser_inputs_hash(inputs: &ParserInputs) -> Hash {
    let mut hasher = stable_hasher("parser-inputs");
    write_str(&mut hasher, inputs.edition.as_str());
    write_string_required_context(&mut hasher, inputs.string_required_positions);
    write_u64(&mut hasher, inputs.operator_fixity.entries.len() as u64);
    for entry in &inputs.operator_fixity.entries {
        write_str(&mut hasher, entry.spelling.as_ref());
        write_operator_fixity(&mut hasher, entry.fixity);
        hasher.update(&[entry.precedence]);
        write_u64(&mut hasher, entry.active_from as u64);
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

fn write_operator_fixity(hasher: &mut blake3::Hasher, fixity: OperatorFixity) {
    match fixity {
        OperatorFixity::Prefix => write_str(hasher, "prefix"),
        OperatorFixity::Infix(associativity) => {
            write_str(hasher, "infix");
            write_operator_associativity(hasher, associativity);
        }
        OperatorFixity::Postfix => write_str(hasher, "postfix"),
    }
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
        ActiveLexicalEnvironmentCacheKey, FrontendCacheKeys, ParserLexingPlanCacheKey,
        ParserLexingPlanContextCacheKey, PreprocessedSourceCacheKey, SourceUnitCacheKey,
        SurfaceAstCacheKey, TOKEN_STREAM_CACHE_KEY_VERSION, TokenStreamCacheKey,
        parser_inputs_hash,
    };
    use crate::lexical_env::LexicalEnvironmentFingerprint;
    use crate::lexing::{
        LexicalByteRange, ParserLexContext, ParserLexingPlan, ParserLexingPlanContext,
    };
    use crate::parsing::{
        OperatorAssociativity, OperatorFixity, OperatorFixityEntry, OperatorFixityTable,
        ParserCacheKeyVersion, ParserInputs, StringRequiredContext,
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

    const RETAINED_KEYS_WIRE: &str = r#"["mizar-frontend/cache-keys/v1",["future/source","pkg-v99","opaque.mod","src/alpha.miz",[1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1,1],"2099"],["future/preprocess",[2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2,2]],["future/active",23],["future/tokens",[3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3,3],29,[5,[0,1,2,3,4,5,6]],["future/plan",[0,[]],[[8,12,[1,[0]]],[2,6,[2,[2]]],[2,6,[2,[2]]],[0,0,[3,[3]]],[5,9,[4,[4]]],[3,7,[5,[5]]]]]],["future/ast",[4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4,4],"future/parser",[5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5,5],"2098"]]"#;

    #[test]
    fn retained_cache_keys_fixed_wire_decodes_every_field_and_reencodes_identically() {
        use mizar_lexer::{UserSymbolKind, UserSymbolKindSet};

        let path = normalized_path("src/alpha.miz");
        let all_kinds = UserSymbolKindSet::from_slice(&[
            UserSymbolKind::Functor,
            UserSymbolKind::Predicate,
            UserSymbolKind::Mode,
            UserSymbolKind::Attribute,
            UserSymbolKind::Structure,
            UserSymbolKind::Selector,
            UserSymbolKind::Constructor,
        ]);
        let expected = FrontendCacheKeys {
            source: SourceUnitCacheKey {
                version: Arc::from("future/source"),
                package_id: PackageId::new("pkg-v99"),
                module_path: ModulePath::new("opaque.mod"),
                normalized_path: path.clone(),
                source_hash: hash(1),
                edition: Edition::new("2099"),
            },
            preprocessed: PreprocessedSourceCacheKey {
                version: Arc::from("future/preprocess"),
                source_hash: hash(2),
            },
            active_lexical_environment: ActiveLexicalEnvironmentCacheKey {
                version: Arc::from("future/active"),
                fingerprint: LexicalEnvironmentFingerprint::new(23),
            },
            tokens: TokenStreamCacheKey {
                version: Arc::from("future/tokens"),
                lexical_hash: hash(3),
                active_lexical_environment: LexicalEnvironmentFingerprint::new(29),
                parser_context: ParserLexContext::recovery().with_user_symbol_kinds(all_kinds),
                parser_lexing_plan: ParserLexingPlanCacheKey {
                    version: Arc::from("future/plan"),
                    default_context: ParserLexContext::general()
                        .with_user_symbol_kinds(UserSymbolKindSet::from_slice(&[])),
                    contexts: vec![
                        ParserLexingPlanContextCacheKey {
                            range: LexicalByteRange::new(8, 12),
                            context: ParserLexContext::identifier_required()
                                .with_user_symbol_kinds(UserSymbolKindSet::only(
                                    UserSymbolKind::Functor,
                                )),
                        },
                        ParserLexingPlanContextCacheKey {
                            range: LexicalByteRange::new(2, 6),
                            context: ParserLexContext::symbolic().with_user_symbol_kinds(
                                UserSymbolKindSet::only(UserSymbolKind::Mode),
                            ),
                        },
                        ParserLexingPlanContextCacheKey {
                            range: LexicalByteRange::new(2, 6),
                            context: ParserLexContext::symbolic().with_user_symbol_kinds(
                                UserSymbolKindSet::only(UserSymbolKind::Mode),
                            ),
                        },
                        ParserLexingPlanContextCacheKey {
                            range: LexicalByteRange::new(0, 0),
                            context: ParserLexContext::string_required().with_user_symbol_kinds(
                                UserSymbolKindSet::only(UserSymbolKind::Attribute),
                            ),
                        },
                        ParserLexingPlanContextCacheKey {
                            range: LexicalByteRange::new(5, 9),
                            context: ParserLexContext::namespace_path().with_user_symbol_kinds(
                                UserSymbolKindSet::only(UserSymbolKind::Structure),
                            ),
                        },
                        ParserLexingPlanContextCacheKey {
                            range: LexicalByteRange::new(3, 7),
                            context: ParserLexContext::recovery().with_user_symbol_kinds(
                                UserSymbolKindSet::only(UserSymbolKind::Selector),
                            ),
                        },
                    ],
                },
            },
            ast: Some(SurfaceAstCacheKey {
                version: Arc::from("future/ast"),
                token_stream_hash: hash(4),
                parser_version: ParserCacheKeyVersion::new("future/parser"),
                parser_inputs_hash: hash(5),
                edition: Edition::new("2098"),
            }),
        };
        let decoded = FrontendCacheKeys::from_canonical_bytes(RETAINED_KEYS_WIRE.as_bytes(), &path)
            .expect("fixed canonical wire must decode");
        assert_eq!(decoded, expected);
        assert_eq!(
            decoded.canonical_bytes().as_deref(),
            Some(RETAINED_KEYS_WIRE.as_bytes())
        );
        assert_eq!(decoded.source.stable_hash(), expected.source.stable_hash());
        assert_eq!(
            decoded.preprocessed.stable_hash(),
            expected.preprocessed.stable_hash()
        );
        assert_eq!(
            decoded.active_lexical_environment.stable_hash(),
            expected.active_lexical_environment.stable_hash()
        );
        assert_eq!(decoded.tokens.stable_hash(), expected.tokens.stable_hash());
        assert_eq!(
            decoded.ast.as_ref().unwrap().stable_hash(),
            expected.ast.as_ref().unwrap().stable_hash()
        );
        for (tag, kind) in [
            (1, UserSymbolKind::Predicate),
            (6, UserSymbolKind::Constructor),
        ] {
            let mut wire: serde_json::Value = serde_json::from_str(RETAINED_KEYS_WIRE).unwrap();
            wire[4][3] = serde_json::json!([0, [tag]]);
            let bytes = serde_json::to_vec(&wire).unwrap();
            let keys = FrontendCacheKeys::from_canonical_bytes(&bytes, &path).unwrap();
            assert_eq!(
                keys.tokens.parser_context,
                ParserLexContext::general().with_user_symbol_kinds(UserSymbolKindSet::only(kind))
            );
            assert_eq!(keys.canonical_bytes(), Some(bytes));
        }
        assert_ne!(decoded.source.source_hash, decoded.preprocessed.source_hash);
        assert_ne!(
            decoded.active_lexical_environment.fingerprint,
            decoded.tokens.active_lexical_environment
        );
        assert_ne!(
            decoded.ast.as_ref().unwrap().token_stream_hash,
            decoded.tokens.stable_hash()
        );
    }

    #[test]
    fn retained_cache_keys_absent_ast_and_maximum_integer_round_trip() {
        let path = normalized_path("src/alpha.miz");
        let mut value: serde_json::Value = serde_json::from_str(RETAINED_KEYS_WIRE).unwrap();
        value[5] = serde_json::Value::Null;
        value[3][1] = serde_json::json!(u64::MAX);
        value[4][2] = serde_json::json!(u64::MAX);
        value[4][4][2][0][0] = serde_json::json!(usize::MAX);
        value[4][4][2][0][1] = serde_json::json!(usize::MAX);
        let bytes = serde_json::to_vec(&value).unwrap();
        let decoded = FrontendCacheKeys::from_canonical_bytes(&bytes, &path).unwrap();
        assert!(decoded.ast.is_none());
        assert_eq!(
            decoded.active_lexical_environment.fingerprint.get(),
            u64::MAX
        );
        assert_eq!(decoded.tokens.active_lexical_environment.get(), u64::MAX);
        assert_eq!(
            decoded.tokens.parser_lexing_plan.contexts[0].range.start,
            usize::MAX
        );
        assert_eq!(
            decoded.tokens.parser_lexing_plan.contexts[0].range.end,
            usize::MAX
        );
        assert_eq!(decoded.canonical_bytes(), Some(bytes));
    }

    #[test]
    fn retained_cache_keys_reject_bad_shape_width_tags_ranges_and_path() {
        let path = normalized_path("src/alpha.miz");
        let base: serde_json::Value = serde_json::from_str(RETAINED_KEYS_WIRE).unwrap();
        let mut cases = Vec::new();
        let mut value = base.clone();
        value[0] = serde_json::json!("other/schema");
        cases.push(("schema", value));
        let mut value = base.clone();
        value[1].as_array_mut().unwrap().pop();
        cases.push(("source arity", value));
        let mut value = base.clone();
        value[2][1].as_array_mut().unwrap().pop();
        cases.push(("short hash", value));
        let mut value = base.clone();
        value[5][1][0] = serde_json::json!(256);
        cases.push(("hash byte overflow", value));
        let mut value = base.clone();
        value[3][1] = serde_json::json!(-1);
        cases.push(("negative fingerprint", value));
        let mut value = base.clone();
        value[4][4][2][0][0] = serde_json::json!(-1);
        cases.push(("negative range", value));
        let mut value = base.clone();
        value[4][3][0] = serde_json::json!(6);
        cases.push(("unknown parser mode", value));
        let mut value = base.clone();
        value[4][3][1] = serde_json::json!([7]);
        cases.push(("unknown user kind", value));
        let mut value = base.clone();
        value[4][3][1] = serde_json::json!([1, 0]);
        cases.push(("unordered user kinds", value));
        let mut value = base.clone();
        value[4][4][1][1] = serde_json::json!([0, 0]);
        cases.push(("duplicate user kind", value));
        let mut value = base.clone();
        value[4][4][2][0][1] = serde_json::json!(7);
        cases.push(("reversed lexical range", value));
        let mut value = base.clone();
        value[1][3] = serde_json::json!("src/other.miz");
        cases.push(("path mismatch", value));
        for (label, value) in cases {
            let bytes = serde_json::to_vec(&value).unwrap();
            assert!(
                FrontendCacheKeys::from_canonical_bytes(&bytes, &path).is_none(),
                "{label}"
            );
        }
        assert!(
            FrontendCacheKeys::from_canonical_bytes(
                format!(" {RETAINED_KEYS_WIRE}").as_bytes(),
                &path
            )
            .is_none()
        );
        assert!(FrontendCacheKeys::from_canonical_bytes(b"[", &path).is_none());
        assert!(FrontendCacheKeys::from_canonical_bytes(&[0xff], &path).is_none());
        let mut invalid_typed =
            FrontendCacheKeys::from_canonical_bytes(RETAINED_KEYS_WIRE.as_bytes(), &path).unwrap();
        invalid_typed.tokens.parser_lexing_plan.contexts[0].range =
            LexicalByteRange { start: 13, end: 12 };
        assert!(invalid_typed.canonical_bytes().is_none());
        let too_large_integer = RETAINED_KEYS_WIRE.replacen(
            "\"future/active\",23",
            "\"future/active\",18446744073709551616",
            1,
        );
        assert!(
            FrontendCacheKeys::from_canonical_bytes(too_large_integer.as_bytes(), &path).is_none()
        );
    }

    #[test]
    fn retained_cache_keys_accept_exactly_sixteen_mib_and_reject_more() {
        let path = normalized_path("src/alpha.miz");
        let mut value: serde_json::Value = serde_json::from_str(RETAINED_KEYS_WIRE).unwrap();
        value[1][1] = serde_json::json!("");
        let overhead = serde_json::to_vec(&value).unwrap().len();
        let limit = 16 * 1024 * 1024;
        value[1][1] = serde_json::Value::String("x".repeat(limit - overhead));
        let exact = serde_json::to_vec(&value).unwrap();
        assert_eq!(exact.len(), limit);
        let decoded = FrontendCacheKeys::from_canonical_bytes(&exact, &path).unwrap();
        assert_eq!(decoded.canonical_bytes(), Some(exact));
        value[1][1] = serde_json::Value::String("x".repeat(limit - overhead + 1));
        let oversized = serde_json::to_vec(&value).unwrap();
        assert_eq!(oversized.len(), limit + 1);
        assert!(FrontendCacheKeys::from_canonical_bytes(&oversized, &path).is_none());
        let mut decoded = decoded;
        decoded.source.package_id = PackageId::new("x".repeat(limit - overhead + 1));
        assert!(decoded.canonical_bytes().is_none());
    }

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
    fn token_stream_cache_key_version_is_bumped_for_local_notation_aliases() {
        assert_eq!(
            TOKEN_STREAM_CACHE_KEY_VERSION,
            "mizar-frontend/token-stream-cache-key/v3"
        );

        let mut current = TokenStreamCacheKey::new(
            &preprocessed(
                "definition\n  let X, Y be set;\n  antonym X idalias Y for X idbase Y;\nend;\n",
            ),
            LexicalEnvironmentFingerprint::new(7),
            ParserLexContext::general(),
            ParserLexingPlanCacheKey::current(),
        );
        let current_hash = current.stable_hash();
        current.version = Arc::from("mizar-frontend/token-stream-cache-key/v2");
        assert_ne!(current_hash, current.stable_hash());
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

    #[test]
    fn parser_inputs_hash_changes_with_operator_fixity_kind() {
        let prefix =
            parser_inputs_with_operator_fixity(Edition::new("2026"), "op", OperatorFixity::Prefix);
        let postfix =
            parser_inputs_with_operator_fixity(Edition::new("2026"), "op", OperatorFixity::Postfix);
        let left_infix = parser_inputs_with_operator_fixity(
            Edition::new("2026"),
            "op",
            OperatorFixity::Infix(OperatorAssociativity::Left),
        );
        let right_infix = parser_inputs_with_operator_fixity(
            Edition::new("2026"),
            "op",
            OperatorFixity::Infix(OperatorAssociativity::Right),
        );
        let left_infix_higher_precedence = parser_inputs_with_operator_fixity_and_precedence(
            Edition::new("2026"),
            "op",
            OperatorFixity::Infix(OperatorAssociativity::Left),
            51,
        );

        assert_ne!(parser_inputs_hash(&prefix), parser_inputs_hash(&postfix));
        assert_ne!(parser_inputs_hash(&prefix), parser_inputs_hash(&left_infix));
        assert_ne!(
            parser_inputs_hash(&left_infix),
            parser_inputs_hash(&right_infix)
        );
        assert_ne!(
            parser_inputs_hash(&left_infix),
            parser_inputs_hash(&left_infix_higher_precedence)
        );
    }

    #[test]
    fn parser_inputs_hash_changes_with_operator_fixity_activation() {
        let mut later = parser_inputs_with_fixity(Edition::new("2026"));
        later.operator_fixity.entries[0].active_from = 128;

        assert_ne!(
            parser_inputs_hash(&parser_inputs_with_fixity(Edition::new("2026"))),
            parser_inputs_hash(&later)
        );
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
        parser_inputs_with_operator_fixity(
            edition,
            "+",
            OperatorFixity::Infix(OperatorAssociativity::Left),
        )
    }

    fn parser_inputs_with_operator_fixity(
        edition: Edition,
        spelling: &str,
        fixity: OperatorFixity,
    ) -> ParserInputs {
        parser_inputs_with_operator_fixity_and_precedence(edition, spelling, fixity, 50)
    }

    fn parser_inputs_with_operator_fixity_and_precedence(
        edition: Edition,
        spelling: &str,
        fixity: OperatorFixity,
        precedence: u8,
    ) -> ParserInputs {
        ParserInputs::new(
            edition,
            OperatorFixityTable {
                entries: vec![OperatorFixityEntry {
                    spelling: Arc::from(spelling),
                    fixity,
                    precedence,
                    active_from: 0,
                }],
            },
            StringRequiredContext::None,
        )
    }
}
