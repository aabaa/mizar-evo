//! Type-expression normalization for checker phase 6.

use crate::typed_ast::{
    DiagnosticRecoveryState, NormalizedTypeId, TypeDiagnosticClass, TypeDiagnosticDraft,
    TypeDiagnosticId, TypeDiagnosticSeverity, TypeDiagnosticTable, TypeEntryActual, TypeEntryDraft,
    TypeProvenance, TypeRuleId, TypeStatus, TypeTable, TypedSiteRef,
};
use mizar_resolve::{
    env::{SymbolEnv, SymbolKind},
    resolved_ast::SymbolId,
};
use mizar_session::SourceRange;
use std::{
    collections::{BTreeMap, BTreeSet},
    fmt::Write as _,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeNormalizationOutput {
    normalized_types: NormalizedTypeTable,
    type_entries: TypeTable,
    diagnostics: TypeDiagnosticTable,
}

impl TypeNormalizationOutput {
    pub const fn normalized_types(&self) -> &NormalizedTypeTable {
        &self.normalized_types
    }

    pub const fn type_entries(&self) -> &TypeTable {
        &self.type_entries
    }

    pub const fn diagnostics(&self) -> &TypeDiagnosticTable {
        &self.diagnostics
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("type-normalization-debug-v1\n");
        write_normalized_types(&mut output, &self.normalized_types);
        write_type_entries(&mut output, &self.type_entries);
        write_diagnostics(&mut output, &self.diagnostics);
        output
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct TypeNormalizer {
    mode_expansions: BTreeMap<SymbolId, ModeExpansion>,
}

impl TypeNormalizer {
    pub fn new(mode_expansions: impl IntoIterator<Item = (SymbolId, ModeExpansion)>) -> Self {
        let mode_expansions = mode_expansions.into_iter().collect();
        Self { mode_expansions }
    }

    pub fn normalize(
        &self,
        symbols: &SymbolEnv,
        inputs: impl IntoIterator<Item = TypeExpressionInput>,
    ) -> TypeNormalizationOutput {
        let mut state = NormalizationState {
            symbols,
            mode_expansions: &self.mode_expansions,
            normalized_types: NormalizedTypeTable::new(),
            type_entries: TypeTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        };
        let mut seen_sites = BTreeSet::new();

        for input in inputs {
            let site = input.site.clone();
            let range = input.source_range;
            if !seen_sites.insert(site.clone()) {
                state.diagnostic(
                    Some(site),
                    range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.duplicate_site",
                    DiagnosticRecoveryState::Degraded,
                );
                continue;
            }

            let id = state.normalize_input(input);
            let normalized_status = state
                .normalized_types
                .get(id)
                .map(|normalized| normalized.status)
                .unwrap_or(NormalizedTypeStatus::Error);
            let (status, provenance) = match normalized_status {
                NormalizedTypeStatus::Known => (
                    TypeStatus::Known,
                    TypeProvenance::Inferred(TypeRuleId::new("type-expression-normalization")),
                ),
                NormalizedTypeStatus::Degraded => (
                    TypeStatus::Unknown,
                    TypeProvenance::Recovery(state.recovery_diagnostic(
                        site.clone(),
                        range,
                        "checker.type.recovery",
                    )),
                ),
                NormalizedTypeStatus::Error => (
                    TypeStatus::Error,
                    TypeProvenance::Recovery(state.recovery_diagnostic(
                        site.clone(),
                        range,
                        "checker.type.recovery",
                    )),
                ),
            };
            state.type_entries.insert(TypeEntryDraft {
                owner: site,
                expected: None,
                actual: TypeEntryActual::Known(id),
                status,
                provenance,
            });
        }
        let (normalized_types, type_remap) = state.normalized_types.into_canonical();
        let type_entries = remap_type_table(state.type_entries, &type_remap);

        TypeNormalizationOutput {
            normalized_types,
            type_entries,
            diagnostics: state.diagnostics,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeExpressionInput {
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub spelling: String,
    pub head: TypeHeadInput,
    pub args: Vec<TypeExpressionInput>,
    pub attributes: Vec<AttributeInput>,
}

impl TypeExpressionInput {
    pub fn new(
        site: TypedSiteRef,
        source_range: SourceRange,
        spelling: impl Into<String>,
        head: TypeHeadInput,
    ) -> Self {
        Self {
            site,
            source_range,
            spelling: spelling.into(),
            head,
            args: Vec::new(),
            attributes: Vec::new(),
        }
    }

    pub fn with_args(mut self, args: Vec<TypeExpressionInput>) -> Self {
        self.args = args;
        self
    }

    pub fn with_attributes(mut self, attributes: Vec<AttributeInput>) -> Self {
        self.attributes = attributes;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeHeadInput {
    BuiltinObject,
    BuiltinSet,
    Symbol(SymbolId),
    Unresolved(String),
    Ambiguous(Vec<SymbolId>),
    Unsupported(String),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeInput {
    pub symbol: SymbolId,
    pub polarity: AttributePolarity,
    pub args: Vec<TypeExpressionInput>,
    pub source_range: SourceRange,
    pub spelling: String,
}

impl AttributeInput {
    pub fn new(
        symbol: SymbolId,
        polarity: AttributePolarity,
        source_range: SourceRange,
        spelling: impl Into<String>,
    ) -> Self {
        Self {
            symbol,
            polarity,
            args: Vec::new(),
            source_range,
            spelling: spelling.into(),
        }
    }

    pub fn with_args(mut self, args: Vec<TypeExpressionInput>) -> Self {
        self.args = args;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AttributePolarity {
    Positive,
    Negative,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModeExpansion {
    pub radix: TypeExpressionInput,
    pub attributes: Vec<AttributeInput>,
}

impl ModeExpansion {
    pub fn new(radix: TypeExpressionInput, attributes: Vec<AttributeInput>) -> Self {
        Self { radix, attributes }
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct NormalizedTypeTable {
    entries: Vec<NormalizedType>,
    ids_by_key: BTreeMap<NormalizedTypeKey, NormalizedTypeId>,
}

impl NormalizedTypeTable {
    pub const fn new() -> Self {
        Self {
            entries: Vec::new(),
            ids_by_key: BTreeMap::new(),
        }
    }

    fn intern(&mut self, draft: NormalizedTypeDraft) -> NormalizedTypeId {
        let key = NormalizedTypeKey::from(&draft);
        if let Some(id) = self.ids_by_key.get(&key) {
            let entry = &mut self.entries[id.index()];
            entry.source = canonical_source(entry.source.clone(), draft.source);
            entry.status = merge_status(entry.status, draft.status);
            return *id;
        }

        let id = NormalizedTypeId::new(self.entries.len());
        self.entries.push(NormalizedType {
            id,
            head: draft.head,
            args: draft.args,
            attributes: draft.attributes,
            source: draft.source,
            status: draft.status,
        });
        self.ids_by_key.insert(key, id);
        id
    }

    pub fn get(&self, id: NormalizedTypeId) -> Option<&NormalizedType> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (NormalizedTypeId, &NormalizedType)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub fn canonical_iter(&self) -> impl Iterator<Item = (NormalizedTypeId, &NormalizedType)> {
        self.ids_by_key
            .values()
            .copied()
            .map(|id| (id, &self.entries[id.index()]))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NormalizedType {
    pub id: NormalizedTypeId,
    pub head: TypeHeadRef,
    pub args: Vec<NormalizedTypeId>,
    pub attributes: AttributeSet,
    pub source: TypeSource,
    pub status: NormalizedTypeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct NormalizedTypeDraft {
    head: TypeHeadRef,
    args: Vec<NormalizedTypeId>,
    attributes: AttributeSet,
    source: TypeSource,
    status: NormalizedTypeStatus,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeHeadRef {
    BuiltinObject,
    BuiltinSet,
    Mode(SymbolId),
    Structure(SymbolId),
    Error(TypeHeadErrorKind),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum TypeHeadErrorKind {
    Unknown,
    WrongKind,
    Ambiguous,
    Unsupported,
    Recovery,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AttributeSet {
    positive: Vec<AttributeInstance>,
    negative: Vec<AttributeInstance>,
}

impl AttributeSet {
    pub const fn empty() -> Self {
        Self {
            positive: Vec::new(),
            negative: Vec::new(),
        }
    }

    pub fn positive(&self) -> &[AttributeInstance] {
        &self.positive
    }

    pub fn negative(&self) -> &[AttributeInstance] {
        &self.negative
    }

    fn is_empty(&self) -> bool {
        self.positive.is_empty() && self.negative.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AttributeInstance {
    pub symbol: SymbolId,
    pub args: Vec<NormalizedTypeId>,
    pub source_range: SourceRangeKey,
    pub spelling: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TypeSource {
    pub spelling: String,
    pub range: SourceRange,
}

impl TypeSource {
    pub fn new(spelling: impl Into<String>, range: SourceRange) -> Self {
        Self {
            spelling: spelling.into(),
            range,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum NormalizedTypeStatus {
    Known,
    Degraded,
    Error,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceRangeKey {
    pub start: usize,
    pub end: usize,
}

impl From<SourceRange> for SourceRangeKey {
    fn from(range: SourceRange) -> Self {
        Self {
            start: range.start,
            end: range.end,
        }
    }
}

struct NormalizationState<'a> {
    symbols: &'a SymbolEnv,
    mode_expansions: &'a BTreeMap<SymbolId, ModeExpansion>,
    normalized_types: NormalizedTypeTable,
    type_entries: TypeTable,
    diagnostics: TypeDiagnosticTable,
}

impl NormalizationState<'_> {
    fn normalize_input(&mut self, input: TypeExpressionInput) -> NormalizedTypeId {
        let TypeExpressionInput {
            site,
            source_range,
            spelling,
            head,
            args,
            attributes,
        } = input;

        let mut status = NormalizedTypeStatus::Known;
        let normalized_args = self.normalize_type_args(args, &site, source_range, &mut status);
        let (head, head_failed) = self.normalize_head(&site, source_range, head);
        if head_failed {
            status = degrade(status);
        }

        let mode_expansion = match &head {
            TypeHeadRef::Mode(symbol) => self.mode_expansions.get(symbol).cloned(),
            _ => None,
        };

        if matches!(head, TypeHeadRef::Mode(_)) && mode_expansion.is_none() {
            self.diagnostic(
                Some(site.clone()),
                source_range,
                TypeDiagnosticClass::TypeExpression,
                TypeDiagnosticSeverity::Note,
                "checker.type.external.mode_expansion_payload",
                DiagnosticRecoveryState::Degraded,
            );
            status = degrade(status);
        }

        let mut all_attributes = attributes;
        if let Some(expansion) = mode_expansion {
            let radix_id = self.normalize_input(expansion.radix);
            let (radix_head, radix_args, radix_attributes, radix_status) =
                match self.normalized_types.get(radix_id).cloned() {
                    Some(normalized) => (
                        normalized.head,
                        normalized.args,
                        normalized.attributes,
                        normalized.status,
                    ),
                    None => (
                        {
                            self.recovery_diagnostic(
                                site.clone(),
                                source_range,
                                "checker.type.recovery.mode_radix",
                            );
                            TypeHeadRef::Error(TypeHeadErrorKind::Recovery)
                        },
                        Vec::new(),
                        AttributeSet::empty(),
                        NormalizedTypeStatus::Error,
                    ),
                };
            if radix_status != NormalizedTypeStatus::Known {
                status = degrade(status);
            }
            if !normalized_args.is_empty() {
                self.diagnostic(
                    Some(site.clone()),
                    source_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.wrong_mode_arity",
                    DiagnosticRecoveryState::Degraded,
                );
                status = degrade(status);
            };
            all_attributes.extend(expansion.attributes);
            let extra_attributes =
                self.normalize_attributes(all_attributes, &site, source_range, &mut status);
            let attribute_set = self.merge_attribute_sets(
                radix_attributes,
                extra_attributes,
                &site,
                source_range,
                &mut status,
            );
            return finish_type(
                &mut self.normalized_types,
                NormalizedTypeDraft {
                    head: radix_head,
                    args: radix_args,
                    attributes: attribute_set,
                    source: TypeSource::new(spelling, source_range),
                    status,
                },
            );
        }

        if !normalized_args.is_empty() {
            match head {
                TypeHeadRef::BuiltinObject | TypeHeadRef::BuiltinSet => {
                    self.diagnostic(
                        Some(site.clone()),
                        source_range,
                        TypeDiagnosticClass::TypeExpression,
                        TypeDiagnosticSeverity::Error,
                        "checker.type.wrong_builtin_arity",
                        DiagnosticRecoveryState::Degraded,
                    );
                    status = degrade(status);
                }
                TypeHeadRef::Structure(_) => {
                    self.diagnostic(
                        Some(site.clone()),
                        source_range,
                        TypeDiagnosticClass::TypeExpression,
                        TypeDiagnosticSeverity::Error,
                        "checker.type.wrong_structure_arity",
                        DiagnosticRecoveryState::Degraded,
                    );
                    status = degrade(status);
                }
                TypeHeadRef::Mode(_) | TypeHeadRef::Error(_) => {}
            }
        }

        let attribute_set =
            self.normalize_attributes(all_attributes, &site, source_range, &mut status);
        finish_type(
            &mut self.normalized_types,
            NormalizedTypeDraft {
                head,
                args: normalized_args,
                attributes: attribute_set,
                source: TypeSource::new(spelling, source_range),
                status,
            },
        )
    }

    fn normalize_type_args(
        &mut self,
        args: Vec<TypeExpressionInput>,
        site: &TypedSiteRef,
        source_range: SourceRange,
        status: &mut NormalizedTypeStatus,
    ) -> Vec<NormalizedTypeId> {
        args.into_iter()
            .map(|arg| {
                let id = self.normalize_input(arg);
                if self
                    .normalized_types
                    .get(id)
                    .is_some_and(|normalized| normalized.status != NormalizedTypeStatus::Known)
                {
                    *status = degrade(*status);
                    self.diagnostic(
                        Some(site.clone()),
                        source_range,
                        TypeDiagnosticClass::TypeExpression,
                        TypeDiagnosticSeverity::Error,
                        "checker.type.argument_degraded",
                        DiagnosticRecoveryState::Degraded,
                    );
                }
                id
            })
            .collect()
    }

    fn normalize_head(
        &mut self,
        site: &TypedSiteRef,
        source_range: SourceRange,
        head: TypeHeadInput,
    ) -> (TypeHeadRef, bool) {
        match head {
            TypeHeadInput::BuiltinObject => (TypeHeadRef::BuiltinObject, false),
            TypeHeadInput::BuiltinSet => (TypeHeadRef::BuiltinSet, false),
            TypeHeadInput::Symbol(symbol) => match self.symbols.symbols().get(&symbol) {
                Some(entry) => match entry.kind() {
                    SymbolKind::Mode => (TypeHeadRef::Mode(symbol), false),
                    SymbolKind::Structure => (TypeHeadRef::Structure(symbol), false),
                    _ => {
                        self.diagnostic(
                            Some(site.clone()),
                            source_range,
                            TypeDiagnosticClass::TypeExpression,
                            TypeDiagnosticSeverity::Error,
                            "checker.type.wrong_head_kind",
                            DiagnosticRecoveryState::Degraded,
                        );
                        (TypeHeadRef::Error(TypeHeadErrorKind::WrongKind), true)
                    }
                },
                None => {
                    self.diagnostic(
                        Some(site.clone()),
                        source_range,
                        TypeDiagnosticClass::TypeExpression,
                        TypeDiagnosticSeverity::Error,
                        "checker.type.unknown_head",
                        DiagnosticRecoveryState::Degraded,
                    );
                    (TypeHeadRef::Error(TypeHeadErrorKind::Unknown), true)
                }
            },
            TypeHeadInput::Unresolved(_) => {
                self.diagnostic(
                    Some(site.clone()),
                    source_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.unknown_head",
                    DiagnosticRecoveryState::Degraded,
                );
                (TypeHeadRef::Error(TypeHeadErrorKind::Unknown), true)
            }
            TypeHeadInput::Ambiguous(_) => {
                self.diagnostic(
                    Some(site.clone()),
                    source_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.ambiguous_head",
                    DiagnosticRecoveryState::Degraded,
                );
                (TypeHeadRef::Error(TypeHeadErrorKind::Ambiguous), true)
            }
            TypeHeadInput::Unsupported(_) => {
                self.diagnostic(
                    Some(site.clone()),
                    source_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.unsupported_payload",
                    DiagnosticRecoveryState::Degraded,
                );
                (TypeHeadRef::Error(TypeHeadErrorKind::Unsupported), true)
            }
        }
    }

    fn normalize_attributes(
        &mut self,
        attributes: Vec<AttributeInput>,
        site: &TypedSiteRef,
        source_range: SourceRange,
        status: &mut NormalizedTypeStatus,
    ) -> AttributeSet {
        let mut positive = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();
        let mut negative = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();

        for attribute in attributes {
            let AttributeInput {
                symbol,
                polarity,
                args,
                source_range: attribute_range,
                spelling,
            } = attribute;

            if !matches!(
                self.symbols
                    .symbols()
                    .get(&symbol)
                    .map(|entry| entry.kind()),
                Some(SymbolKind::Attribute)
            ) {
                self.diagnostic(
                    Some(site.clone()),
                    attribute_range,
                    TypeDiagnosticClass::TypeExpression,
                    TypeDiagnosticSeverity::Error,
                    "checker.type.wrong_attribute_kind",
                    DiagnosticRecoveryState::Degraded,
                );
                *status = degrade(*status);
            }

            let args = self.normalize_type_args(args, site, source_range, status);
            let instance = AttributeInstance {
                symbol,
                args,
                source_range: attribute_range.into(),
                spelling,
            };
            let key = AttributeSemanticKey {
                symbol: instance.symbol.clone(),
                args: instance.args.clone(),
            };
            match polarity {
                AttributePolarity::Positive => {
                    insert_canonical_attribute(&mut positive, key, instance);
                }
                AttributePolarity::Negative => {
                    insert_canonical_attribute(&mut negative, key, instance);
                }
            }
        }

        let contradictions = positive
            .keys()
            .filter(|key| negative.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        for _ in contradictions {
            self.diagnostic(
                Some(site.clone()),
                source_range,
                TypeDiagnosticClass::TypeExpression,
                TypeDiagnosticSeverity::Error,
                "checker.type.contradictory_attribute",
                DiagnosticRecoveryState::Degraded,
            );
            *status = degrade(*status);
        }

        AttributeSet {
            positive: positive.into_values().collect(),
            negative: negative.into_values().collect(),
        }
    }

    fn merge_attribute_sets(
        &mut self,
        base: AttributeSet,
        extra: AttributeSet,
        site: &TypedSiteRef,
        source_range: SourceRange,
        status: &mut NormalizedTypeStatus,
    ) -> AttributeSet {
        let mut positive = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();
        let mut negative = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();
        for instance in base.positive.into_iter().chain(extra.positive) {
            insert_canonical_attribute(&mut positive, attribute_semantic_key(&instance), instance);
        }
        for instance in base.negative.into_iter().chain(extra.negative) {
            insert_canonical_attribute(&mut negative, attribute_semantic_key(&instance), instance);
        }
        let contradictions = positive
            .keys()
            .filter(|key| negative.contains_key(*key))
            .cloned()
            .collect::<Vec<_>>();
        for _ in contradictions {
            self.diagnostic(
                Some(site.clone()),
                source_range,
                TypeDiagnosticClass::TypeExpression,
                TypeDiagnosticSeverity::Error,
                "checker.type.contradictory_attribute",
                DiagnosticRecoveryState::Degraded,
            );
            *status = degrade(*status);
        }
        AttributeSet {
            positive: positive.into_values().collect(),
            negative: negative.into_values().collect(),
        }
    }

    fn recovery_diagnostic(
        &mut self,
        site: TypedSiteRef,
        source_range: SourceRange,
        message_key: &str,
    ) -> TypeDiagnosticId {
        self.diagnostic(
            Some(site),
            source_range,
            TypeDiagnosticClass::Recovery,
            TypeDiagnosticSeverity::Note,
            message_key,
            DiagnosticRecoveryState::Recovery,
        )
    }

    fn diagnostic(
        &mut self,
        owner: Option<TypedSiteRef>,
        source_range: SourceRange,
        class: TypeDiagnosticClass,
        severity: TypeDiagnosticSeverity,
        message_key: &str,
        recovery: DiagnosticRecoveryState,
    ) -> TypeDiagnosticId {
        self.diagnostics.insert(TypeDiagnosticDraft {
            owner,
            source_range,
            class,
            severity,
            message_key: message_key.to_owned(),
            recovery,
        })
    }
}

fn finish_type(
    normalized_types: &mut NormalizedTypeTable,
    draft: NormalizedTypeDraft,
) -> NormalizedTypeId {
    normalized_types.intern(draft)
}

fn degrade(status: NormalizedTypeStatus) -> NormalizedTypeStatus {
    match status {
        NormalizedTypeStatus::Known | NormalizedTypeStatus::Degraded => {
            NormalizedTypeStatus::Degraded
        }
        NormalizedTypeStatus::Error => NormalizedTypeStatus::Error,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct AttributeSemanticKey {
    symbol: SymbolId,
    args: Vec<NormalizedTypeId>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct NormalizedTypeKey {
    head: TypeHeadRef,
    args: Vec<NormalizedTypeId>,
    positive: Vec<AttributeSemanticKey>,
    negative: Vec<AttributeSemanticKey>,
}

impl From<&NormalizedTypeDraft> for NormalizedTypeKey {
    fn from(draft: &NormalizedTypeDraft) -> Self {
        Self {
            head: draft.head.clone(),
            args: draft.args.clone(),
            positive: draft
                .attributes
                .positive
                .iter()
                .map(attribute_semantic_key)
                .collect(),
            negative: draft
                .attributes
                .negative
                .iter()
                .map(attribute_semantic_key)
                .collect(),
        }
    }
}

impl From<&NormalizedType> for NormalizedTypeKey {
    fn from(normalized: &NormalizedType) -> Self {
        Self {
            head: normalized.head.clone(),
            args: normalized.args.clone(),
            positive: normalized
                .attributes
                .positive
                .iter()
                .map(attribute_semantic_key)
                .collect(),
            negative: normalized
                .attributes
                .negative
                .iter()
                .map(attribute_semantic_key)
                .collect(),
        }
    }
}

fn attribute_semantic_key(instance: &AttributeInstance) -> AttributeSemanticKey {
    AttributeSemanticKey {
        symbol: instance.symbol.clone(),
        args: instance.args.clone(),
    }
}

fn insert_canonical_attribute(
    attributes: &mut BTreeMap<AttributeSemanticKey, AttributeInstance>,
    key: AttributeSemanticKey,
    instance: AttributeInstance,
) {
    let should_insert = match attributes.get(&key) {
        Some(current) => {
            attribute_instance_order_key(&instance) < attribute_instance_order_key(current)
        }
        None => true,
    };
    if should_insert {
        attributes.insert(key, instance);
    }
}

fn attribute_instance_order_key(instance: &AttributeInstance) -> (SourceRangeKey, &str) {
    (instance.source_range, instance.spelling.as_str())
}

fn canonical_source(left: TypeSource, right: TypeSource) -> TypeSource {
    if type_source_order_key(&right) < type_source_order_key(&left) {
        right
    } else {
        left
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
struct TypeSourceOrderKey {
    source_id: String,
    range: SourceRangeKey,
    spelling: String,
}

fn type_source_order_key(source: &TypeSource) -> TypeSourceOrderKey {
    TypeSourceOrderKey {
        source_id: format!("{:?}", source.range.source_id),
        range: source.range.into(),
        spelling: source.spelling.clone(),
    }
}

fn merge_status(left: NormalizedTypeStatus, right: NormalizedTypeStatus) -> NormalizedTypeStatus {
    if status_rank(right) > status_rank(left) {
        right
    } else {
        left
    }
}

fn status_rank(status: NormalizedTypeStatus) -> u8 {
    match status {
        NormalizedTypeStatus::Known => 0,
        NormalizedTypeStatus::Degraded => 1,
        NormalizedTypeStatus::Error => 2,
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StructuralTypeKey {
    head: TypeHeadRef,
    args: Vec<StructuralTypeKey>,
    positive: Vec<StructuralAttributeKey>,
    negative: Vec<StructuralAttributeKey>,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
struct StructuralAttributeKey {
    symbol: SymbolId,
    args: Vec<StructuralTypeKey>,
}

impl NormalizedTypeTable {
    fn into_canonical(self) -> (Self, BTreeMap<NormalizedTypeId, NormalizedTypeId>) {
        let mut memo = BTreeMap::new();
        let mut by_key = BTreeMap::<StructuralTypeKey, Vec<NormalizedTypeId>>::new();
        for (old_id, _) in self.iter() {
            by_key
                .entry(structural_type_key(old_id, &self.entries, &mut memo))
                .or_default()
                .push(old_id);
        }

        let mut remap = BTreeMap::new();
        let groups = by_key.values().cloned().collect::<Vec<_>>();
        for (new_index, old_ids) in groups.iter().enumerate() {
            let new_id = NormalizedTypeId::new(new_index);
            for old_id in old_ids {
                remap.insert(*old_id, new_id);
            }
        }

        let mut entries = Vec::new();
        let mut ids_by_key = BTreeMap::new();
        for (new_index, old_ids) in groups.into_iter().enumerate() {
            let representative = old_ids
                .iter()
                .copied()
                .min_by(|left, right| {
                    type_source_order_key(&self.entries[left.index()].source)
                        .cmp(&type_source_order_key(&self.entries[right.index()].source))
                })
                .expect("canonical groups are never empty");
            let mut entry = self.entries[representative.index()].clone();
            entry.id = NormalizedTypeId::new(new_index);
            entry.args = entry
                .args
                .into_iter()
                .map(|id| remapped_type_id(id, &remap))
                .collect();
            entry.attributes = canonical_attribute_set_for_group(&old_ids, &self.entries, &remap);
            entry.source = canonical_source_for_group(&old_ids, &self.entries);
            entry.status = merged_status_for_group(&old_ids, &self.entries);
            ids_by_key.insert(NormalizedTypeKey::from(&entry), entry.id);
            entries.push(entry);
        }

        (
            Self {
                entries,
                ids_by_key,
            },
            remap,
        )
    }
}

fn canonical_attribute_set_for_group(
    old_ids: &[NormalizedTypeId],
    entries: &[NormalizedType],
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> AttributeSet {
    let mut positive = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();
    let mut negative = BTreeMap::<AttributeSemanticKey, AttributeInstance>::new();

    for old_id in old_ids {
        let entry = &entries[old_id.index()];
        for attribute in entry.attributes.positive.iter().cloned() {
            let attribute = remap_attribute_instance(attribute, remap);
            insert_canonical_attribute(
                &mut positive,
                attribute_semantic_key(&attribute),
                attribute,
            );
        }
        for attribute in entry.attributes.negative.iter().cloned() {
            let attribute = remap_attribute_instance(attribute, remap);
            insert_canonical_attribute(
                &mut negative,
                attribute_semantic_key(&attribute),
                attribute,
            );
        }
    }

    AttributeSet {
        positive: positive.into_values().collect(),
        negative: negative.into_values().collect(),
    }
}

fn canonical_source_for_group(
    old_ids: &[NormalizedTypeId],
    entries: &[NormalizedType],
) -> TypeSource {
    let mut sources = old_ids
        .iter()
        .map(|old_id| entries[old_id.index()].source.clone());
    let mut source = sources.next().expect("canonical groups are never empty");
    for candidate in sources {
        source = canonical_source(source, candidate);
    }
    source
}

fn merged_status_for_group(
    old_ids: &[NormalizedTypeId],
    entries: &[NormalizedType],
) -> NormalizedTypeStatus {
    old_ids
        .iter()
        .fold(NormalizedTypeStatus::Known, |status, old_id| {
            merge_status(status, entries[old_id.index()].status)
        })
}

fn structural_type_key(
    id: NormalizedTypeId,
    entries: &[NormalizedType],
    memo: &mut BTreeMap<NormalizedTypeId, StructuralTypeKey>,
) -> StructuralTypeKey {
    if let Some(key) = memo.get(&id) {
        return key.clone();
    }
    let entry = &entries[id.index()];
    let key = StructuralTypeKey {
        head: entry.head.clone(),
        args: entry
            .args
            .iter()
            .map(|arg| structural_type_key(*arg, entries, memo))
            .collect(),
        positive: entry
            .attributes
            .positive
            .iter()
            .map(|attribute| structural_attribute_key(attribute, entries, memo))
            .collect(),
        negative: entry
            .attributes
            .negative
            .iter()
            .map(|attribute| structural_attribute_key(attribute, entries, memo))
            .collect(),
    };
    memo.insert(id, key.clone());
    key
}

fn structural_attribute_key(
    attribute: &AttributeInstance,
    entries: &[NormalizedType],
    memo: &mut BTreeMap<NormalizedTypeId, StructuralTypeKey>,
) -> StructuralAttributeKey {
    StructuralAttributeKey {
        symbol: attribute.symbol.clone(),
        args: attribute
            .args
            .iter()
            .map(|arg| structural_type_key(*arg, entries, memo))
            .collect(),
    }
}

fn remap_type_table(
    type_entries: TypeTable,
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> TypeTable {
    let mut remapped = TypeTable::new();
    for (_, entry) in type_entries.iter() {
        remapped.insert(TypeEntryDraft {
            owner: entry.owner.clone(),
            expected: entry.expected.map(|id| remapped_type_id(id, remap)),
            actual: remap_type_actual(entry.actual, remap),
            status: entry.status,
            provenance: entry.provenance.clone(),
        });
    }
    remapped
}

fn remap_type_actual(
    actual: TypeEntryActual,
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> TypeEntryActual {
    match actual {
        TypeEntryActual::Known(id) => TypeEntryActual::Known(remapped_type_id(id, remap)),
        TypeEntryActual::CandidateSet(id) => TypeEntryActual::CandidateSet(id),
        TypeEntryActual::Absent => TypeEntryActual::Absent,
    }
}

fn remap_attribute_instance(
    mut attribute: AttributeInstance,
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> AttributeInstance {
    attribute.args = attribute
        .args
        .into_iter()
        .map(|id| remapped_type_id(id, remap))
        .collect();
    attribute
}

fn remapped_type_id(
    id: NormalizedTypeId,
    remap: &BTreeMap<NormalizedTypeId, NormalizedTypeId>,
) -> NormalizedTypeId {
    remap.get(&id).copied().unwrap_or(id)
}

fn write_normalized_types(output: &mut String, types: &NormalizedTypeTable) {
    output.push_str("normalized_types:\n");
    if types.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (id, ty) in types.canonical_iter() {
        let _ = write!(output, "  normalized_type#{} head=", id.index());
        write_head(output, &ty.head);
        output.push_str(" args=");
        write_type_ids(output, &ty.args);
        output.push_str(" attributes=");
        write_attributes(output, &ty.attributes);
        let _ = writeln!(
            output,
            " status={} source=\"{}\" range={}..{}",
            normalized_status_name(ty.status),
            escaped_display(&ty.source.spelling),
            ty.source.range.start,
            ty.source.range.end
        );
    }
}

fn write_type_entries(output: &mut String, types: &TypeTable) {
    output.push_str("type_entries:\n");
    if types.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for (ordinal, (_, entry)) in types.canonical_iter().enumerate() {
        let _ = write!(
            output,
            "  type_entry#{} owner={} status={:?} actual=",
            ordinal,
            site_key(&entry.owner),
            entry.status
        );
        match entry.actual {
            TypeEntryActual::Known(id) => {
                let _ = write!(output, "normalized_type#{}", id.index());
            }
            TypeEntryActual::CandidateSet(id) => {
                let _ = write!(output, "candidate_set#{}", id.index());
            }
            TypeEntryActual::Absent => output.push_str("<absent>"),
        }
        output.push('\n');
    }
}

fn write_diagnostics(output: &mut String, diagnostics: &TypeDiagnosticTable) {
    output.push_str("diagnostics:\n");
    if diagnostics.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    let mut diagnostics = diagnostics
        .iter()
        .map(|(_, diagnostic)| diagnostic)
        .collect::<Vec<_>>();
    diagnostics.sort_by_key(|diagnostic| diagnostic_debug_key(diagnostic));
    for (ordinal, diagnostic) in diagnostics.into_iter().enumerate() {
        let _ = write!(output, "  diagnostic#{} owner=", ordinal);
        match &diagnostic.owner {
            Some(owner) => output.push_str(&site_key(owner)),
            None => output.push_str("<none>"),
        }
        let _ = writeln!(
            output,
            " range={}..{} class={:?} severity={:?} message_key=\"{}\" recovery={:?}",
            diagnostic.source_range.start,
            diagnostic.source_range.end,
            diagnostic.class,
            diagnostic.severity,
            escaped_display(&diagnostic.message_key),
            diagnostic.recovery
        );
    }
}

fn diagnostic_debug_key(
    diagnostic: &crate::typed_ast::TypeDiagnostic,
) -> (
    String,
    usize,
    usize,
    TypeDiagnosticClass,
    TypeDiagnosticSeverity,
    String,
    DiagnosticRecoveryState,
) {
    (
        diagnostic
            .owner
            .as_ref()
            .map(site_key)
            .unwrap_or_else(|| "<none>".to_owned()),
        diagnostic.source_range.start,
        diagnostic.source_range.end,
        diagnostic.class,
        diagnostic.severity,
        diagnostic.message_key.clone(),
        diagnostic.recovery,
    )
}

fn write_head(output: &mut String, head: &TypeHeadRef) {
    match head {
        TypeHeadRef::BuiltinObject => output.push_str("builtin_object"),
        TypeHeadRef::BuiltinSet => output.push_str("builtin_set"),
        TypeHeadRef::Mode(symbol) => {
            output.push_str("mode=");
            write_symbol_id(output, symbol);
        }
        TypeHeadRef::Structure(symbol) => {
            output.push_str("structure=");
            write_symbol_id(output, symbol);
        }
        TypeHeadRef::Error(kind) => {
            let _ = write!(output, "error={}", type_head_error_kind_name(*kind));
        }
    }
}

fn type_head_error_kind_name(kind: TypeHeadErrorKind) -> &'static str {
    match kind {
        TypeHeadErrorKind::Unknown => "unknown",
        TypeHeadErrorKind::WrongKind => "wrong_kind",
        TypeHeadErrorKind::Ambiguous => "ambiguous",
        TypeHeadErrorKind::Unsupported => "unsupported",
        TypeHeadErrorKind::Recovery => "recovery",
    }
}

fn write_attributes(output: &mut String, attributes: &AttributeSet) {
    if attributes.is_empty() {
        output.push_str("[]");
        return;
    }
    output.push('[');
    let mut first = true;
    for (polarity, instances) in [
        ("positive", attributes.positive.as_slice()),
        ("negative", attributes.negative.as_slice()),
    ] {
        for instance in instances {
            if !first {
                output.push_str(", ");
            }
            first = false;
            let _ = write!(output, "{polarity}:");
            write_symbol_id(output, &instance.symbol);
            output.push('(');
            write_type_ids(output, &instance.args);
            let _ = write!(
                output,
                ", range={}..{}, spelling=\"{}\")",
                instance.source_range.start,
                instance.source_range.end,
                escaped_display(&instance.spelling)
            );
        }
    }
    output.push(']');
}

fn write_type_ids(output: &mut String, ids: &[NormalizedTypeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "normalized_type#{}", id.index());
    }
    output.push(']');
}

fn write_symbol_id(output: &mut String, symbol: &SymbolId) {
    output.push_str("{fqn=\"");
    write_escaped(output, symbol.fqn().as_str());
    output.push_str("\" local=\"");
    write_escaped(output, symbol.local().as_str());
    output.push_str("\"}");
}

fn site_key(site: &TypedSiteRef) -> String {
    match site {
        TypedSiteRef::Node(node) => format!("node#{}", node.index()),
        TypedSiteRef::Role { node, role } => format!("node#{}:{}", node.index(), role.as_str()),
    }
}

fn normalized_status_name(status: NormalizedTypeStatus) -> &'static str {
    match status {
        NormalizedTypeStatus::Known => "known",
        NormalizedTypeStatus::Degraded => "degraded",
        NormalizedTypeStatus::Error => "error",
    }
}

fn escaped_display(value: &str) -> String {
    let mut escaped = String::new();
    write_escaped(&mut escaped, value);
    escaped
}

fn write_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        match character {
            '\\' => output.push_str("\\\\"),
            '"' => output.push_str("\\\""),
            '\n' => output.push_str("\\n"),
            '\r' => output.push_str("\\r"),
            '\t' => output.push_str("\\t"),
            character if character.is_control() => {
                let _ = write!(output, "\\u{{{:x}}}", character as u32);
            }
            character => output.push(character),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::typed_ast::{TypeRole, TypedNodeId};
    use mizar_resolve::{
        env::{
            ContributionKind, DefinitionIndex, ExportStatus, LabelIndex, ModuleLexicalSummaryIndex,
            ModuleSummaryIndex, NamespaceGraph, NamespacePath, OverloadIndex, RegistrationIndex,
            ResolvedExportIndex, ResolvedImportIndex, SourceContributionIndex, SymbolEntry,
            SymbolEnvIndexes, SymbolIndex, Visibility,
        },
        resolved_ast::{FullyQualifiedName, LocalSymbolId, ModuleId, SemanticOrigin},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId, SessionIdAllocator,
        SourceAnchor, SourceId,
    };

    #[test]
    fn attributes_are_sorted_deduplicated_and_contradictions_are_diagnosed() {
        let source = source_id();
        let attr_a = symbol_id("AttrA/1", "pkg::main::AttrA/1");
        let attr_b = symbol_id("AttrB/1", "pkg::main::AttrB/1");
        let symbols = symbol_env(vec![
            symbol_entry(attr_b.clone(), SymbolKind::Attribute),
            symbol_entry(attr_a.clone(), SymbolKind::Attribute),
        ]);
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 10),
            "set attr",
            TypeHeadInput::BuiltinSet,
        )
        .with_attributes(vec![
            AttributeInput::new(
                attr_b.clone(),
                AttributePolarity::Positive,
                range(source, 8, 9),
                "B",
            ),
            AttributeInput::new(
                attr_a.clone(),
                AttributePolarity::Negative,
                range(source, 4, 5),
                "non A",
            ),
            AttributeInput::new(
                attr_a.clone(),
                AttributePolarity::Positive,
                range(source, 2, 3),
                "A",
            ),
            AttributeInput::new(
                attr_b,
                AttributePolarity::Positive,
                range(source, 6, 7),
                "B",
            ),
        ]);

        let output = TypeNormalizer::default().normalize(&symbols, [input]);
        let normalized = output
            .normalized_types()
            .get(NormalizedTypeId::new(0))
            .unwrap();

        assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
        assert_eq!(normalized.attributes.positive().len(), 2);
        assert_eq!(normalized.attributes.negative().len(), 1);
        assert_eq!(
            normalized
                .attributes
                .positive()
                .iter()
                .map(|attribute| attribute.symbol.local().as_str())
                .collect::<Vec<_>>(),
            vec!["AttrA/1", "AttrB/1"]
        );
        assert_eq!(output.diagnostics().len(), 2);
        assert!(
            output
                .debug_text()
                .contains("checker.type.contradictory_attribute")
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.contradictory_attribute"),
            vec![(0, 10)]
        );
    }

    #[test]
    fn attribute_arguments_and_duplicate_ranges_are_canonicalized() {
        let source = source_id();
        let attr = symbol_id("Attr/1", "pkg::main::Attr/1");
        let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 50),
            "attributed set",
            TypeHeadInput::BuiltinSet,
        )
        .with_attributes(vec![
            AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 30, 35),
                "Attr set late",
            )
            .with_args(vec![TypeExpressionInput::new(
                site(1),
                range(source, 36, 39),
                "set",
                TypeHeadInput::BuiltinSet,
            )]),
            AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 20, 25),
                "Attr object",
            )
            .with_args(vec![TypeExpressionInput::new(
                site(2),
                range(source, 26, 32),
                "object",
                TypeHeadInput::BuiltinObject,
            )]),
            AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 10, 15),
                "Attr set early",
            )
            .with_args(vec![TypeExpressionInput::new(
                site(3),
                range(source, 16, 19),
                "set",
                TypeHeadInput::BuiltinSet,
            )]),
        ]);

        let output = TypeNormalizer::default().normalize(&symbols, [input]);
        let normalized = output
            .type_entries()
            .iter()
            .find_map(|(_, entry)| match entry.actual {
                TypeEntryActual::Known(id) if entry.owner == site(0) => {
                    output.normalized_types().get(id)
                }
                _ => None,
            })
            .unwrap();

        assert_eq!(normalized.status, NormalizedTypeStatus::Known);
        assert_eq!(normalized.attributes.positive().len(), 2);
        assert_eq!(
            normalized
                .attributes
                .positive()
                .iter()
                .map(|attribute| attribute.args.clone())
                .collect::<Vec<_>>(),
            vec![
                vec![NormalizedTypeId::new(0)],
                vec![NormalizedTypeId::new(1)]
            ]
        );
        assert_eq!(normalized.attributes.positive()[1].source_range.start, 10);
        assert_eq!(normalized.attributes.positive()[1].source_range.end, 15);
    }

    #[test]
    fn equivalent_inputs_have_order_independent_debug_rendering() {
        let (source, alternate_source) = source_ids_pair();
        let symbols = symbol_env(Vec::new());
        let known_later = TypeExpressionInput::new(
            site(2),
            range(source, 20, 23),
            "set shared",
            TypeHeadInput::BuiltinSet,
        );
        let known_earlier = TypeExpressionInput::new(
            site(1),
            range(alternate_source, 20, 23),
            "set shared",
            TypeHeadInput::BuiltinSet,
        );
        let bad_later = TypeExpressionInput::new(
            site(4),
            range(source, 40, 47),
            "missing later",
            TypeHeadInput::Unresolved("missing".to_owned()),
        );
        let bad_earlier = TypeExpressionInput::new(
            site(3),
            range(source, 30, 37),
            "missing earlier",
            TypeHeadInput::Unresolved("missing".to_owned()),
        );

        let first = TypeNormalizer::default().normalize(
            &symbols,
            [
                known_later.clone(),
                bad_later.clone(),
                known_earlier.clone(),
                bad_earlier.clone(),
            ],
        );
        let second = TypeNormalizer::default().normalize(
            &symbols,
            [bad_earlier, known_earlier, bad_later, known_later],
        );

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.normalized_types().len(), 2);
        assert!(first.debug_text().contains("source=\"set shared\""));
        assert!(first.debug_text().contains("error=unknown"));
        assert!(!first.debug_text().contains("error=diagnostic#"));
        assert_eq!(
            first
                .normalized_types()
                .get(NormalizedTypeId::new(0))
                .unwrap()
                .source
                .range
                .source_id,
            second
                .normalized_types()
                .get(NormalizedTypeId::new(0))
                .unwrap()
                .source
                .range
                .source_id
        );
    }

    #[test]
    fn builtins_structures_and_recursive_arguments_have_deterministic_ids() {
        let source = source_id();
        let structure = symbol_id("Struct/0", "pkg::main::Struct/0");
        let mode = symbol_id("Mode/1", "pkg::main::Mode/1");
        let symbols = symbol_env(vec![
            symbol_entry(structure.clone(), SymbolKind::Structure),
            symbol_entry(mode.clone(), SymbolKind::Mode),
        ]);
        let arg = TypeExpressionInput::new(
            site(1),
            range(source, 2, 5),
            "set",
            TypeHeadInput::BuiltinSet,
        );
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 6),
            "Mode of set",
            TypeHeadInput::Symbol(mode.clone()),
        )
        .with_args(vec![arg]);
        let structure_input = TypeExpressionInput::new(
            site(2),
            range(source, 7, 13),
            "Struct",
            TypeHeadInput::Symbol(structure),
        );
        let object_input = TypeExpressionInput::new(
            site(3),
            range(source, 14, 20),
            "object",
            TypeHeadInput::BuiltinObject,
        );
        let mode_with_object_arg = TypeExpressionInput::new(
            site(4),
            range(source, 21, 35),
            "Mode of object",
            TypeHeadInput::Symbol(mode.clone()),
        )
        .with_args(vec![TypeExpressionInput::new(
            site(5),
            range(source, 29, 35),
            "object",
            TypeHeadInput::BuiltinObject,
        )]);

        let first = TypeNormalizer::default().normalize(
            &symbols,
            [
                input.clone(),
                structure_input.clone(),
                object_input.clone(),
                mode_with_object_arg.clone(),
            ],
        );
        let second = TypeNormalizer::default().normalize(
            &symbols,
            [input, structure_input, object_input, mode_with_object_arg],
        );

        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.normalized_types().len(), 5);
        assert!(matches!(
            first
                .normalized_types()
                .get(NormalizedTypeId::new(0))
                .unwrap()
                .head,
            TypeHeadRef::BuiltinObject
        ));
        assert!(matches!(
            first
                .normalized_types()
                .get(NormalizedTypeId::new(1))
                .unwrap()
                .head,
            TypeHeadRef::BuiltinSet
        ));
        assert!(matches!(
            first.normalized_types().get(NormalizedTypeId::new(2)).unwrap().head,
            TypeHeadRef::Mode(ref actual) if actual == &mode
        ));
        assert!(
            first
                .normalized_types()
                .iter()
                .any(|(_, normalized)| matches!(normalized.head, TypeHeadRef::Structure(_)))
        );
        let mode_arg_sets = first
            .normalized_types()
            .iter()
            .filter_map(|(id, normalized)| match normalized.head {
                TypeHeadRef::Mode(ref actual) if actual == &mode => Some((id, &normalized.args)),
                _ => None,
            })
            .collect::<Vec<_>>();
        assert_eq!(mode_arg_sets.len(), 2);
        assert_ne!(mode_arg_sets[0].0, mode_arg_sets[1].0);
        assert_ne!(mode_arg_sets[0].1, mode_arg_sets[1].1);
        assert!(
            mode_arg_sets
                .iter()
                .any(|(_, args)| args.as_slice() == [NormalizedTypeId::new(0)])
        );
        assert!(
            mode_arg_sets
                .iter()
                .any(|(_, args)| args.as_slice() == [NormalizedTypeId::new(1)])
        );
        assert_eq!(
            first
                .type_entries()
                .canonical_iter()
                .map(|(id, _)| id.index())
                .collect::<Vec<_>>(),
            vec![0, 1, 2, 3]
        );
        assert!(
            first
                .debug_text()
                .starts_with("type-normalization-debug-v1\n")
        );
        assert!(!first.debug_text().contains("SourceId"));
        assert!(!first.debug_text().contains(concat!("V", "cId")));
        assert!(!first.debug_text().contains(concat!("Proof", "Witness")));
        assert!(!first.debug_text().contains("accepted_verifier_status"));
        assert!(!first.debug_text().contains(concat!("inserted", "_qua")));
        assert!(
            !first
                .debug_text()
                .contains(concat!("active", "_refinement"))
        );
        assert!(!first.debug_text().contains(concat!("overload", "_root")));
    }

    #[test]
    fn mode_expansion_provider_unfolds_radix_and_attributes_idempotently() {
        let source = source_id();
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let attr = symbol_id("Expandable/0", "pkg::main::Expandable/0");
        let symbols = symbol_env(vec![
            symbol_entry(mode.clone(), SymbolKind::Mode),
            symbol_entry(attr.clone(), SymbolKind::Attribute),
        ]);
        let expansion = ModeExpansion::new(
            TypeExpressionInput::new(
                site(10),
                range(source, 20, 23),
                "set",
                TypeHeadInput::BuiltinSet,
            ),
            vec![AttributeInput::new(
                attr.clone(),
                AttributePolarity::Positive,
                range(source, 24, 25),
                "expandable",
            )],
        );
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 4),
            "Mode",
            TypeHeadInput::Symbol(mode.clone()),
        );
        let repeated = TypeExpressionInput::new(
            site(1),
            range(source, 5, 9),
            "Mode",
            TypeHeadInput::Symbol(mode.clone()),
        );

        let first = TypeNormalizer::new([(mode.clone(), expansion.clone())])
            .normalize(&symbols, [input.clone(), repeated.clone()]);
        let second =
            TypeNormalizer::new([(mode, expansion)]).normalize(&symbols, [input, repeated]);
        assert_eq!(first.debug_text(), second.debug_text());
        assert_eq!(first.normalized_types().len(), 2);
        assert_eq!(
            first
                .type_entries()
                .canonical_iter()
                .map(|(_, entry)| entry.actual)
                .collect::<Vec<_>>(),
            vec![
                TypeEntryActual::Known(NormalizedTypeId::new(1)),
                TypeEntryActual::Known(NormalizedTypeId::new(1)),
            ]
        );
        let expanded = first
            .normalized_types()
            .canonical_iter()
            .map(|(_, normalized)| normalized)
            .find(|normalized| !normalized.attributes.positive().is_empty())
            .unwrap();

        assert!(matches!(expanded.head, TypeHeadRef::BuiltinSet));
        assert_eq!(expanded.attributes.positive()[0].symbol, attr);
        assert_eq!(first.diagnostics().len(), 0);
    }

    #[test]
    fn mode_expansion_preserves_radix_arguments_and_attributes() {
        let source = source_id();
        let outer = symbol_id("Outer/0", "pkg::main::Outer/0");
        let inner = symbol_id("Inner/1", "pkg::main::Inner/1");
        let base_attr = symbol_id("Base/0", "pkg::main::Base/0");
        let extra_attr = symbol_id("Extra/0", "pkg::main::Extra/0");
        let symbols = symbol_env(vec![
            symbol_entry(outer.clone(), SymbolKind::Mode),
            symbol_entry(inner.clone(), SymbolKind::Mode),
            symbol_entry(base_attr.clone(), SymbolKind::Attribute),
            symbol_entry(extra_attr.clone(), SymbolKind::Attribute),
        ]);
        let radix = TypeExpressionInput::new(
            site(10),
            range(source, 20, 30),
            "Inner of object",
            TypeHeadInput::Symbol(inner.clone()),
        )
        .with_args(vec![TypeExpressionInput::new(
            site(11),
            range(source, 29, 35),
            "object",
            TypeHeadInput::BuiltinObject,
        )])
        .with_attributes(vec![AttributeInput::new(
            base_attr.clone(),
            AttributePolarity::Positive,
            range(source, 36, 37),
            "base",
        )]);
        let expansion = ModeExpansion::new(
            radix,
            vec![AttributeInput::new(
                extra_attr.clone(),
                AttributePolarity::Positive,
                range(source, 38, 39),
                "extra",
            )],
        );
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 5),
            "Outer",
            TypeHeadInput::Symbol(outer.clone()),
        );

        let output = TypeNormalizer::new([(outer, expansion)]).normalize(&symbols, [input]);
        let expanded = output
            .type_entries()
            .iter()
            .find_map(|(_, entry)| match entry.actual {
                TypeEntryActual::Known(id) if entry.owner == site(0) => {
                    output.normalized_types().get(id)
                }
                _ => None,
            })
            .unwrap();

        assert!(matches!(expanded.head, TypeHeadRef::Mode(ref actual) if actual == &inner));
        assert_eq!(expanded.args.len(), 1);
        assert_eq!(
            expanded
                .attributes
                .positive()
                .iter()
                .map(|attribute| attribute.symbol.local().as_str())
                .collect::<Vec<_>>(),
            vec!["Base/0", "Extra/0"]
        );
    }

    #[test]
    fn mode_expansion_with_arguments_reports_wrong_mode_arity() {
        let source = source_id();
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let symbols = symbol_env(vec![symbol_entry(mode.clone(), SymbolKind::Mode)]);
        let expansion = ModeExpansion::new(
            TypeExpressionInput::new(
                site(10),
                range(source, 20, 23),
                "set",
                TypeHeadInput::BuiltinSet,
            ),
            Vec::new(),
        );
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 12),
            "Mode of object",
            TypeHeadInput::Symbol(mode.clone()),
        )
        .with_args(vec![TypeExpressionInput::new(
            site(1),
            range(source, 8, 14),
            "object",
            TypeHeadInput::BuiltinObject,
        )]);

        let output = TypeNormalizer::new([(mode, expansion)]).normalize(&symbols, [input]);
        let normalized = output
            .type_entries()
            .iter()
            .find_map(|(_, entry)| match entry.actual {
                TypeEntryActual::Known(id) if entry.owner == site(0) => {
                    output.normalized_types().get(id)
                }
                _ => None,
            })
            .unwrap();

        assert!(matches!(normalized.head, TypeHeadRef::BuiltinSet));
        assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_mode_arity"),
            vec![(0, 12)]
        );
    }

    #[test]
    fn missing_mode_payload_degrades_without_cluster_repair() {
        let source = source_id();
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let symbols = symbol_env(vec![symbol_entry(mode.clone(), SymbolKind::Mode)]);
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 4),
            "Mode",
            TypeHeadInput::Symbol(mode.clone()),
        );

        let output = TypeNormalizer::default().normalize(&symbols, [input]);
        let normalized = output
            .normalized_types()
            .get(NormalizedTypeId::new(0))
            .unwrap();

        assert!(matches!(normalized.head, TypeHeadRef::Mode(ref actual) if actual == &mode));
        assert_eq!(normalized.status, NormalizedTypeStatus::Degraded);
        assert!(
            output
                .debug_text()
                .contains("checker.type.external.mode_expansion_payload")
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.external.mode_expansion_payload"),
            vec![(0, 4)]
        );
        assert!(!output.debug_text().contains("cluster"));
        assert!(!output.debug_text().contains("registration"));
    }

    #[test]
    fn unknown_wrong_kind_ambiguous_and_unsupported_heads_are_partial_entries() {
        let source = source_id();
        let attr = symbol_id("Attr/0", "pkg::main::Attr/0");
        let candidate_a = symbol_id("A/0", "pkg::main::A/0");
        let candidate_b = symbol_id("B/0", "pkg::main::B/0");
        let symbols = symbol_env(vec![symbol_entry(attr.clone(), SymbolKind::Attribute)]);
        let inputs = [
            TypeExpressionInput::new(
                site(0),
                range(source, 0, 1),
                "missing",
                TypeHeadInput::Unresolved("missing".to_owned()),
            ),
            TypeExpressionInput::new(
                site(1),
                range(source, 2, 3),
                "Attr",
                TypeHeadInput::Symbol(attr),
            ),
            TypeExpressionInput::new(
                site(2),
                range(source, 4, 5),
                "ambiguous",
                TypeHeadInput::Ambiguous(vec![candidate_b, candidate_a]),
            ),
            TypeExpressionInput::new(
                site(3),
                range(source, 6, 7),
                "unsupported",
                TypeHeadInput::Unsupported("raw".to_owned()),
            ),
        ];

        let output = TypeNormalizer::default().normalize(&symbols, inputs);

        assert_eq!(output.type_entries().len(), 4);
        assert!(
            output
                .type_entries()
                .iter()
                .all(|(_, entry)| entry.status == TypeStatus::Unknown)
        );
        for expected in [
            "checker.type.unknown_head",
            "checker.type.wrong_head_kind",
            "checker.type.ambiguous_head",
            "checker.type.unsupported_payload",
            "checker.type.recovery",
        ] {
            assert!(output.debug_text().contains(expected), "{expected}");
        }
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.unknown_head"),
            vec![(0, 1)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_head_kind"),
            vec![(2, 3)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.ambiguous_head"),
            vec![(4, 5)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.unsupported_payload"),
            vec![(6, 7)]
        );
    }

    #[test]
    fn structure_arity_and_wrong_attribute_kind_emit_degraded_diagnostics() {
        let source = source_id();
        let structure = symbol_id("Struct/0", "pkg::main::Struct/0");
        let mode = symbol_id("Mode/0", "pkg::main::Mode/0");
        let wrong_attribute = symbol_id("Wrong/0", "pkg::main::Wrong/0");
        let symbols = symbol_env(vec![
            symbol_entry(structure, SymbolKind::Structure),
            symbol_entry(mode.clone(), SymbolKind::Mode),
            symbol_entry(wrong_attribute.clone(), SymbolKind::Mode),
        ]);
        let input = TypeExpressionInput::new(
            site(0),
            range(source, 0, 6),
            "Struct of Mode",
            TypeHeadInput::Symbol(symbol_id("Struct/0", "pkg::main::Struct/0")),
        )
        .with_args(vec![TypeExpressionInput::new(
            site(1),
            range(source, 7, 11),
            "Mode",
            TypeHeadInput::Symbol(mode),
        )])
        .with_attributes(vec![AttributeInput::new(
            wrong_attribute,
            AttributePolarity::Positive,
            range(source, 12, 13),
            "Wrong",
        )]);
        let builtin_with_arg = TypeExpressionInput::new(
            site(2),
            range(source, 14, 20),
            "set of Mode",
            TypeHeadInput::BuiltinSet,
        )
        .with_args(vec![TypeExpressionInput::new(
            site(3),
            range(source, 21, 25),
            "Mode",
            TypeHeadInput::Symbol(symbol_id("Mode/0", "pkg::main::Mode/0")),
        )]);

        let output = TypeNormalizer::default().normalize(&symbols, [input, builtin_with_arg]);

        assert!(
            output
                .debug_text()
                .contains("checker.type.wrong_structure_arity")
        );
        assert!(
            output
                .debug_text()
                .contains("checker.type.wrong_attribute_kind")
        );
        assert!(
            output
                .debug_text()
                .contains("checker.type.wrong_builtin_arity")
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_structure_arity"),
            vec![(0, 6)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_attribute_kind"),
            vec![(12, 13)]
        );
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.wrong_builtin_arity"),
            vec![(14, 20)]
        );
    }

    #[test]
    fn duplicate_type_expression_sites_are_diagnosed_without_duplicate_entries() {
        let source = source_id();
        let symbols = symbol_env(Vec::new());
        let first = TypeExpressionInput::new(
            site(0),
            range(source, 0, 3),
            "set",
            TypeHeadInput::BuiltinSet,
        );
        let duplicate = TypeExpressionInput::new(
            site(0),
            range(source, 4, 7),
            "object",
            TypeHeadInput::BuiltinObject,
        );

        let output = TypeNormalizer::default().normalize(&symbols, [first, duplicate]);

        assert_eq!(output.type_entries().len(), 1);
        assert_eq!(
            diagnostic_ranges(&output, "checker.type.duplicate_site"),
            vec![(4, 7)]
        );
    }

    fn diagnostic_ranges(
        output: &TypeNormalizationOutput,
        message_key: &str,
    ) -> Vec<(usize, usize)> {
        output
            .diagnostics()
            .iter()
            .filter_map(|(_, diagnostic)| {
                (diagnostic.message_key == message_key)
                    .then_some((diagnostic.source_range.start, diagnostic.source_range.end))
            })
            .collect()
    }

    fn symbol_env(entries: Vec<SymbolEntry>) -> SymbolEnv {
        let mut symbols = SymbolIndex::new();
        for entry in entries {
            symbols.insert(entry);
        }
        let mut contributions = SourceContributionIndex::new();
        let source = source_id();
        contributions.insert(
            module_id(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        SymbolEnv::new(
            module_id(),
            SymbolEnvIndexes {
                imports: ResolvedImportIndex::new(),
                exports: ResolvedExportIndex::new(),
                symbols,
                labels: LabelIndex::new(),
                definitions: DefinitionIndex::new(),
                overloads: OverloadIndex::new(),
                registrations: RegistrationIndex::new(),
                lexical_summaries: ModuleLexicalSummaryIndex::new(),
                namespace_graph: NamespaceGraph::new(),
                declaration_dependencies: Default::default(),
                contributions,
                module_summaries: ModuleSummaryIndex::new(),
            },
        )
    }

    fn symbol_entry(symbol: SymbolId, kind: SymbolKind) -> SymbolEntry {
        let source = source_id();
        let mut contributions = SourceContributionIndex::new();
        let contribution = contributions.insert(
            module_id(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 0, 1)),
        );
        SymbolEntry::new(
            symbol,
            kind,
            NamespacePath::new("main"),
            "symbol",
            SemanticOrigin::new(
                source,
                module_id(),
                SourceAnchor::Range(range(source, 0, 1)),
                Vec::new(),
            ),
            contribution,
        )
        .with_visibility(Visibility::Public)
        .with_export_status(ExportStatus::Exported)
    }

    fn site(index: usize) -> TypedSiteRef {
        TypedSiteRef::Role {
            node: TypedNodeId::new(index),
            role: TypeRole::new("type"),
        }
    }

    fn source_id() -> SourceId {
        source_ids_pair().0
    }

    fn source_ids_pair() -> (SourceId, SourceId) {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "17".repeat(32)
        ))
        .unwrap();
        let allocator = InMemorySessionIdAllocator::new();
        (
            allocator.next_source_id(snapshot).unwrap(),
            allocator.next_source_id(snapshot).unwrap(),
        )
    }

    fn range(source_id: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id,
            start,
            end,
        }
    }

    fn module_id() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("main"))
    }

    fn symbol_id(local: &str, fqn: &str) -> SymbolId {
        SymbolId::new(
            module_id(),
            LocalSymbolId::new(local),
            FullyQualifiedName::new(fqn),
        )
    }
}
