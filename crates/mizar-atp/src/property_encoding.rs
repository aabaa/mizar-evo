//! Backend-neutral property encoding.
//!
//! This module implements the task-8 source described by
//! [property_encoding.md](../../../doc/design/mizar-atp/en/property_encoding.md).
//! It turns explicit, already-resolved property facts into axiom-form
//! `EncodedProperty` rows plus the generated binder declarations needed by
//! those formulas. Native declarations remain deferred to later concrete
//! encoder specs.

use crate::problem::{
    AtpAtom, AtpBinder, AtpDeclaration, AtpDeclarationId, AtpDeclarationKind, AtpFormulaTree,
    AtpPayload, AtpProblemError, AtpPropertyId, AtpProvenance, AtpProvenanceId, AtpSourceBinding,
    AtpSourceRef, AtpSymbolMapEntry, AtpSymbolName, AtpSymbolSource, AtpTerm, EncodedProperty,
    EqualitySupport, LogicProfile, QuantifierPolicy,
};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt,
};

const FAMILY_COMMUTATIVITY: &str = "commutativity";
const FAMILY_SYMMETRY: &str = "symmetry";
const FAMILY_REFLEXIVITY: &str = "reflexivity";
const FAMILY_IDEMPOTENCE: &str = "idempotence";
const FAMILY_INVOLUTIVENESS: &str = "involutiveness";
const FAMILY_PROJECTIVITY: &str = "projectivity";
const FAMILY_ASYMMETRY: &str = "asymmetry";
const FAMILY_CONNECTEDNESS: &str = "connectedness";
const FAMILY_IRREFLEXIVITY: &str = "irreflexivity";

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct AtpPropertyFamily(String);

impl AtpPropertyFamily {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.trim().is_empty()
    }
}

impl From<&str> for AtpPropertyFamily {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for AtpPropertyFamily {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AtpPropertyTargetKind {
    Function,
    Predicate,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum AtpPropertyEncodingStrategy {
    Axiom,
    NativeDeclaration,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpPropertyBinderSort {
    pub symbol: AtpSymbolName,
    pub source: AtpSourceBinding,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpPropertyProjection {
    pub source_property: AtpSourceBinding,
    pub family: AtpPropertyFamily,
    pub target_symbol: AtpSymbolName,
    pub target_source: AtpSourceBinding,
    pub target_kind: AtpPropertyTargetKind,
    pub target_arity: u32,
    pub binder_sort: Option<AtpPropertyBinderSort>,
    pub provenance_payload: AtpPayload,
    pub encoding_strategy: AtpPropertyEncodingStrategy,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpPropertyEncodingInput {
    pub logic_profile: LogicProfile,
    pub existing_declarations: Vec<AtpDeclaration>,
    pub existing_symbol_map: Vec<AtpSymbolMapEntry>,
    pub existing_provenance: Vec<AtpProvenance>,
    pub property_projections: Vec<AtpPropertyProjection>,
    pub next_property_id: AtpPropertyId,
    pub next_declaration_id: AtpDeclarationId,
    pub next_provenance_id: AtpProvenanceId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AtpPropertyEncodingBundle {
    properties: Vec<EncodedProperty>,
    declarations: Vec<AtpDeclaration>,
    symbol_map: Vec<AtpSymbolMapEntry>,
    provenance: Vec<AtpProvenance>,
}

impl AtpPropertyEncodingBundle {
    pub fn properties(&self) -> &[EncodedProperty] {
        &self.properties
    }

    pub fn declarations(&self) -> &[AtpDeclaration] {
        &self.declarations
    }

    pub fn symbol_map(&self) -> &[AtpSymbolMapEntry] {
        &self.symbol_map
    }

    pub fn provenance(&self) -> &[AtpProvenance] {
        &self.provenance
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum AtpPropertyEncodingError {
    EmptyField {
        field: &'static str,
    },
    UnsupportedFamily {
        family: AtpPropertyFamily,
    },
    UnsupportedProfileFeature {
        feature: &'static str,
    },
    NativeDeclarationDeferred,
    InvalidPropertyTarget {
        family: AtpPropertyFamily,
        expected_kind: AtpPropertyTargetKind,
        expected_arity: u32,
        actual_kind: AtpPropertyTargetKind,
        actual_arity: u32,
    },
    MissingSymbolMap {
        symbol: AtpSymbolName,
    },
    MissingDeclarationSymbol {
        symbol: AtpSymbolName,
    },
    InvalidSymbolDeclaration {
        symbol: AtpSymbolName,
        expected: &'static str,
        actual: AtpDeclarationKind,
    },
    InvalidSymbolArity {
        symbol: AtpSymbolName,
        expected: u32,
        actual: u32,
    },
    InvalidSymbolSource {
        symbol: AtpSymbolName,
        expected: AtpSymbolSource,
        actual: AtpSymbolSource,
    },
    DuplicateSymbolSource {
        source: AtpSymbolSource,
    },
    DuplicateSourceProperty {
        source_property: AtpSourceBinding,
    },
    DuplicateEncodedProperty {
        identity: String,
    },
    DuplicateGeneratedSymbol {
        symbol: AtpSymbolName,
    },
    DuplicateId {
        section: &'static str,
        id: u32,
    },
    IdExhausted {
        section: &'static str,
    },
    ProblemInvariant {
        error: AtpProblemError,
    },
}

impl fmt::Display for AtpPropertyEncodingError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyField { field } => write!(formatter, "empty property field: {field}"),
            Self::UnsupportedFamily { family } => {
                write!(
                    formatter,
                    "unsupported property family: {}",
                    family.as_str()
                )
            }
            Self::UnsupportedProfileFeature { feature } => {
                write!(formatter, "unsupported property profile feature: {feature}")
            }
            Self::NativeDeclarationDeferred => {
                formatter.write_str("native property declarations are deferred")
            }
            Self::InvalidPropertyTarget {
                family,
                expected_kind,
                expected_arity,
                actual_kind,
                actual_arity,
            } => write!(
                formatter,
                "invalid property target for {}: expected {:?}/{expected_arity}, got {:?}/{actual_arity}",
                family.as_str(),
                expected_kind,
                actual_kind
            ),
            Self::MissingSymbolMap { symbol } => {
                write!(formatter, "missing symbol-map row for {}", symbol.as_str())
            }
            Self::MissingDeclarationSymbol { symbol } => {
                write!(formatter, "missing declaration for {}", symbol.as_str())
            }
            Self::InvalidSymbolDeclaration {
                symbol,
                expected,
                actual,
            } => write!(
                formatter,
                "invalid declaration for {}: expected {expected}, got {actual:?}",
                symbol.as_str()
            ),
            Self::InvalidSymbolArity {
                symbol,
                expected,
                actual,
            } => write!(
                formatter,
                "invalid arity for {}: expected {expected}, got {actual}",
                symbol.as_str()
            ),
            Self::InvalidSymbolSource {
                symbol,
                expected,
                actual,
            } => write!(
                formatter,
                "invalid symbol source for {}: expected {expected:?}, got {actual:?}",
                symbol.as_str()
            ),
            Self::DuplicateSymbolSource { source } => {
                write!(
                    formatter,
                    "duplicate symbol-map source identity: {source:?}"
                )
            }
            Self::DuplicateSourceProperty { source_property } => write!(
                formatter,
                "duplicate source property identity: {}",
                source_property.as_str()
            ),
            Self::DuplicateEncodedProperty { identity } => {
                write!(formatter, "duplicate encoded property identity: {identity}")
            }
            Self::DuplicateGeneratedSymbol { symbol } => {
                write!(formatter, "duplicate generated symbol: {}", symbol.as_str())
            }
            Self::DuplicateId { section, id } => {
                write!(formatter, "duplicate generated id in {section}: {id}")
            }
            Self::IdExhausted { section } => {
                write!(formatter, "generated id space exhausted for {section}")
            }
            Self::ProblemInvariant { error } => write!(formatter, "{error}"),
        }
    }
}

impl Error for AtpPropertyEncodingError {}

pub fn encode_properties(
    input: AtpPropertyEncodingInput,
) -> Result<AtpPropertyEncodingBundle, AtpPropertyEncodingError> {
    let declaration_map = declaration_map(&input.existing_declarations)?;
    let symbol_map = symbol_source_map(&input.existing_symbol_map)?;
    let existing_symbol_sources = symbol_map.values().cloned().collect::<BTreeSet<_>>();
    let declaration_ids = input
        .existing_declarations
        .iter()
        .map(AtpDeclaration::id)
        .collect::<BTreeSet<_>>();
    let provenance_ids = input
        .existing_provenance
        .iter()
        .map(AtpProvenance::id)
        .collect::<BTreeSet<_>>();

    let mut projections = Vec::new();
    let mut source_identities = BTreeSet::new();
    let mut encoded_identities = BTreeSet::new();
    for projection in input.property_projections {
        validate_projection_fields(&projection)?;
        if projection.encoding_strategy == AtpPropertyEncodingStrategy::NativeDeclaration {
            return Err(AtpPropertyEncodingError::NativeDeclarationDeferred);
        }
        let family = SupportedPropertyFamily::parse(&projection.family)?;
        let signature = family.signature();
        validate_projection_signature(&projection, signature)?;
        validate_profile(&input.logic_profile, family)?;
        validate_target_symbol(&projection, signature, &declaration_map, &symbol_map)?;
        validate_binder_sort(&projection.binder_sort, &declaration_map, &symbol_map)?;

        if !source_identities.insert(projection.source_property.clone()) {
            return Err(AtpPropertyEncodingError::DuplicateSourceProperty {
                source_property: projection.source_property,
            });
        }
        let encoded_identity = encoded_identity_key(&projection, family);
        if !encoded_identities.insert(encoded_identity.clone()) {
            return Err(AtpPropertyEncodingError::DuplicateEncodedProperty {
                identity: encoded_identity,
            });
        }
        projections.push((property_sort_key(&projection, family), family, projection));
    }

    projections.sort_by(|left, right| left.0.cmp(&right.0));

    let mut properties = Vec::new();
    let mut declarations = Vec::new();
    let mut symbol_entries = Vec::new();
    let mut provenance = Vec::new();
    let mut generated_symbols = BTreeSet::new();
    let mut generated_sources = BTreeSet::new();

    for (property_offset, (identity, family, projection)) in projections.into_iter().enumerate() {
        let property_id = property_id(input.next_property_id, property_offset)?;
        let property_provenance_id =
            provenance_id(input.next_provenance_id, provenance.len(), "provenance")?;
        ensure_unused_provenance_id(property_provenance_id, &provenance_ids)?;
        provenance.push(AtpProvenance::new(
            property_provenance_id,
            AtpSourceRef::EncodedProperty(property_source_binding(&projection)),
            projection.provenance_payload.clone(),
        ));

        let mut binder_symbols = Vec::new();
        let mut binders = Vec::new();
        for binder_position in 0..family.binder_count() {
            let binder_symbol = binder_symbol(&identity, binder_position);
            ensure_fresh_generated_symbol(&binder_symbol, &symbol_map, &mut generated_symbols)?;
            let binder_provenance_id =
                provenance_id(input.next_provenance_id, provenance.len(), "provenance")?;
            ensure_unused_provenance_id(binder_provenance_id, &provenance_ids)?;
            let binder_declaration_id =
                declaration_id(input.next_declaration_id, declarations.len())?;
            ensure_unused_declaration_id(binder_declaration_id, &declaration_ids)?;
            let binder_source = binder_source_binding(&projection, binder_position);
            let binder_symbol_source = AtpSymbolSource::GeneratedBinder(binder_source.clone());
            ensure_fresh_generated_source(
                &binder_symbol_source,
                &existing_symbol_sources,
                &mut generated_sources,
            )?;

            provenance.push(AtpProvenance::new(
                binder_provenance_id,
                AtpSourceRef::EncodedProperty(binder_source.clone()),
                binder_payload(&identity, binder_position),
            ));
            declarations.push(AtpDeclaration::new(
                binder_declaration_id,
                AtpDeclarationKind::GeneratedBinder,
                binder_symbol.clone(),
                0,
                binder_provenance_id,
            ));
            symbol_entries.push(AtpSymbolMapEntry::new(
                binder_symbol.clone(),
                binder_symbol_source,
            ));
            binders.push(AtpBinder::new(
                binder_symbol.clone(),
                projection
                    .binder_sort
                    .as_ref()
                    .map(|sort| sort.symbol.clone()),
            ));
            binder_symbols.push(binder_symbol);
        }

        let formula = property_formula(family, &projection.target_symbol, &binder_symbols, binders);
        properties.push(EncodedProperty::axiom(
            property_id,
            projection.target_symbol,
            formula,
            property_provenance_id,
        ));
    }

    Ok(AtpPropertyEncodingBundle {
        properties,
        declarations,
        symbol_map: symbol_entries,
        provenance,
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
enum SupportedPropertyFamily {
    Commutativity,
    Symmetry,
    Reflexivity,
    Idempotence,
    Involutiveness,
    Projectivity,
    Asymmetry,
    Connectedness,
    Irreflexivity,
}

impl SupportedPropertyFamily {
    fn parse(family: &AtpPropertyFamily) -> Result<Self, AtpPropertyEncodingError> {
        if family.is_empty() {
            return Err(AtpPropertyEncodingError::EmptyField {
                field: "property.family",
            });
        }
        match family.as_str() {
            FAMILY_COMMUTATIVITY => Ok(Self::Commutativity),
            FAMILY_SYMMETRY => Ok(Self::Symmetry),
            FAMILY_REFLEXIVITY => Ok(Self::Reflexivity),
            FAMILY_IDEMPOTENCE => Ok(Self::Idempotence),
            FAMILY_INVOLUTIVENESS => Ok(Self::Involutiveness),
            FAMILY_PROJECTIVITY => Ok(Self::Projectivity),
            FAMILY_ASYMMETRY => Ok(Self::Asymmetry),
            FAMILY_CONNECTEDNESS => Ok(Self::Connectedness),
            FAMILY_IRREFLEXIVITY => Ok(Self::Irreflexivity),
            _ => Err(AtpPropertyEncodingError::UnsupportedFamily {
                family: family.clone(),
            }),
        }
    }

    const fn as_str(self) -> &'static str {
        match self {
            Self::Commutativity => FAMILY_COMMUTATIVITY,
            Self::Symmetry => FAMILY_SYMMETRY,
            Self::Reflexivity => FAMILY_REFLEXIVITY,
            Self::Idempotence => FAMILY_IDEMPOTENCE,
            Self::Involutiveness => FAMILY_INVOLUTIVENESS,
            Self::Projectivity => FAMILY_PROJECTIVITY,
            Self::Asymmetry => FAMILY_ASYMMETRY,
            Self::Connectedness => FAMILY_CONNECTEDNESS,
            Self::Irreflexivity => FAMILY_IRREFLEXIVITY,
        }
    }

    const fn signature(self) -> PropertySignature {
        match self {
            Self::Commutativity | Self::Idempotence => {
                PropertySignature::new(AtpPropertyTargetKind::Function, 2, true)
            }
            Self::Involutiveness | Self::Projectivity => {
                PropertySignature::new(AtpPropertyTargetKind::Function, 1, true)
            }
            Self::Symmetry | Self::Reflexivity | Self::Asymmetry | Self::Irreflexivity => {
                PropertySignature::new(AtpPropertyTargetKind::Predicate, 2, false)
            }
            Self::Connectedness => {
                PropertySignature::new(AtpPropertyTargetKind::Predicate, 2, true)
            }
        }
    }

    const fn binder_count(self) -> usize {
        match self {
            Self::Commutativity | Self::Symmetry | Self::Asymmetry | Self::Connectedness => 2,
            Self::Reflexivity
            | Self::Idempotence
            | Self::Involutiveness
            | Self::Projectivity
            | Self::Irreflexivity => 1,
        }
    }
}

#[derive(Debug, Clone, Copy)]
struct PropertySignature {
    kind: AtpPropertyTargetKind,
    arity: u32,
    requires_equality: bool,
}

impl PropertySignature {
    const fn new(kind: AtpPropertyTargetKind, arity: u32, requires_equality: bool) -> Self {
        Self {
            kind,
            arity,
            requires_equality,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct DeclarationSignature {
    kind: AtpDeclarationKind,
    arity: u32,
}

fn validate_projection_fields(
    projection: &AtpPropertyProjection,
) -> Result<(), AtpPropertyEncodingError> {
    if projection.source_property.is_empty() {
        return Err(AtpPropertyEncodingError::EmptyField {
            field: "property.source_property",
        });
    }
    if projection.target_symbol.is_empty() {
        return Err(AtpPropertyEncodingError::EmptyField {
            field: "property.target_symbol",
        });
    }
    if projection.target_source.is_empty() {
        return Err(AtpPropertyEncodingError::EmptyField {
            field: "property.target_source",
        });
    }
    if projection.provenance_payload.is_empty() {
        return Err(AtpPropertyEncodingError::EmptyField {
            field: "property.provenance_payload",
        });
    }
    if let Some(sort) = &projection.binder_sort {
        if sort.symbol.is_empty() {
            return Err(AtpPropertyEncodingError::EmptyField {
                field: "property.binder_sort.symbol",
            });
        }
        if sort.source.is_empty() {
            return Err(AtpPropertyEncodingError::EmptyField {
                field: "property.binder_sort.source",
            });
        }
    }
    Ok(())
}

fn validate_projection_signature(
    projection: &AtpPropertyProjection,
    signature: PropertySignature,
) -> Result<(), AtpPropertyEncodingError> {
    if projection.target_kind != signature.kind || projection.target_arity != signature.arity {
        Err(AtpPropertyEncodingError::InvalidPropertyTarget {
            family: projection.family.clone(),
            expected_kind: signature.kind,
            expected_arity: signature.arity,
            actual_kind: projection.target_kind,
            actual_arity: projection.target_arity,
        })
    } else {
        Ok(())
    }
}

fn validate_profile(
    profile: &LogicProfile,
    family: SupportedPropertyFamily,
) -> Result<(), AtpPropertyEncodingError> {
    if profile.quantifiers() != QuantifierPolicy::FirstOrder {
        return Err(AtpPropertyEncodingError::UnsupportedProfileFeature {
            feature: "quantifier",
        });
    }
    if family.signature().requires_equality && profile.equality() != EqualitySupport::Supported {
        return Err(AtpPropertyEncodingError::UnsupportedProfileFeature {
            feature: "equality",
        });
    }
    Ok(())
}

fn validate_target_symbol(
    projection: &AtpPropertyProjection,
    signature: PropertySignature,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
    symbol_map: &BTreeMap<AtpSymbolName, AtpSymbolSource>,
) -> Result<(), AtpPropertyEncodingError> {
    validate_symbol_source(
        &projection.target_symbol,
        AtpSymbolSource::MizarSymbol(projection.target_source.clone()),
        symbol_map,
    )?;
    let declaration = declarations.get(&projection.target_symbol).ok_or_else(|| {
        AtpPropertyEncodingError::MissingDeclarationSymbol {
            symbol: projection.target_symbol.clone(),
        }
    })?;
    let expected_declaration = match signature.kind {
        AtpPropertyTargetKind::Function => AtpDeclarationKind::Function,
        AtpPropertyTargetKind::Predicate => AtpDeclarationKind::Predicate,
    };
    if declaration.kind != expected_declaration {
        return Err(AtpPropertyEncodingError::InvalidSymbolDeclaration {
            symbol: projection.target_symbol.clone(),
            expected: match signature.kind {
                AtpPropertyTargetKind::Function => "function",
                AtpPropertyTargetKind::Predicate => "predicate",
            },
            actual: declaration.kind,
        });
    }
    if declaration.arity != signature.arity {
        return Err(AtpPropertyEncodingError::InvalidSymbolArity {
            symbol: projection.target_symbol.clone(),
            expected: signature.arity,
            actual: declaration.arity,
        });
    }
    Ok(())
}

fn validate_binder_sort(
    sort: &Option<AtpPropertyBinderSort>,
    declarations: &BTreeMap<AtpSymbolName, DeclarationSignature>,
    symbol_map: &BTreeMap<AtpSymbolName, AtpSymbolSource>,
) -> Result<(), AtpPropertyEncodingError> {
    let Some(sort) = sort else {
        return Ok(());
    };
    validate_symbol_source(
        &sort.symbol,
        AtpSymbolSource::MizarSymbol(sort.source.clone()),
        symbol_map,
    )?;
    let declaration = declarations.get(&sort.symbol).ok_or_else(|| {
        AtpPropertyEncodingError::MissingDeclarationSymbol {
            symbol: sort.symbol.clone(),
        }
    })?;
    if declaration.kind != AtpDeclarationKind::Sort {
        return Err(AtpPropertyEncodingError::InvalidSymbolDeclaration {
            symbol: sort.symbol.clone(),
            expected: "sort",
            actual: declaration.kind,
        });
    }
    if declaration.arity != 0 {
        return Err(AtpPropertyEncodingError::InvalidSymbolArity {
            symbol: sort.symbol.clone(),
            expected: 0,
            actual: declaration.arity,
        });
    }
    Ok(())
}

fn validate_symbol_source(
    symbol: &AtpSymbolName,
    expected: AtpSymbolSource,
    symbol_map: &BTreeMap<AtpSymbolName, AtpSymbolSource>,
) -> Result<(), AtpPropertyEncodingError> {
    let actual =
        symbol_map
            .get(symbol)
            .ok_or_else(|| AtpPropertyEncodingError::MissingSymbolMap {
                symbol: symbol.clone(),
            })?;
    if actual == &expected {
        Ok(())
    } else {
        Err(AtpPropertyEncodingError::InvalidSymbolSource {
            symbol: symbol.clone(),
            expected,
            actual: actual.clone(),
        })
    }
}

fn declaration_map(
    declarations: &[AtpDeclaration],
) -> Result<BTreeMap<AtpSymbolName, DeclarationSignature>, AtpPropertyEncodingError> {
    let mut result = BTreeMap::new();
    for declaration in declarations {
        if declaration.symbol().is_empty() {
            return Err(AtpPropertyEncodingError::EmptyField {
                field: "declaration.symbol",
            });
        }
        if result
            .insert(
                declaration.symbol().clone(),
                DeclarationSignature {
                    kind: declaration.kind(),
                    arity: declaration.arity(),
                },
            )
            .is_some()
        {
            return Err(AtpPropertyEncodingError::ProblemInvariant {
                error: AtpProblemError::DuplicateDeclarationSymbol {
                    symbol: declaration.symbol().clone(),
                },
            });
        }
    }
    Ok(result)
}

fn symbol_source_map(
    entries: &[AtpSymbolMapEntry],
) -> Result<BTreeMap<AtpSymbolName, AtpSymbolSource>, AtpPropertyEncodingError> {
    let mut result = BTreeMap::new();
    let mut seen_sources = BTreeMap::new();
    for entry in entries {
        if entry.backend_symbol().is_empty() {
            return Err(AtpPropertyEncodingError::EmptyField {
                field: "symbol_map.backend_symbol",
            });
        }
        if seen_sources
            .insert(entry.source().clone(), entry.backend_symbol().clone())
            .is_some()
        {
            return Err(AtpPropertyEncodingError::DuplicateSymbolSource {
                source: entry.source().clone(),
            });
        }
        if result
            .insert(entry.backend_symbol().clone(), entry.source().clone())
            .is_some()
        {
            return Err(AtpPropertyEncodingError::ProblemInvariant {
                error: AtpProblemError::DuplicateSymbolMap {
                    symbol: entry.backend_symbol().clone(),
                },
            });
        }
    }
    Ok(result)
}

fn ensure_fresh_generated_symbol(
    symbol: &AtpSymbolName,
    existing_symbols: &BTreeMap<AtpSymbolName, AtpSymbolSource>,
    generated_symbols: &mut BTreeSet<AtpSymbolName>,
) -> Result<(), AtpPropertyEncodingError> {
    if existing_symbols.contains_key(symbol) || !generated_symbols.insert(symbol.clone()) {
        Err(AtpPropertyEncodingError::DuplicateGeneratedSymbol {
            symbol: symbol.clone(),
        })
    } else {
        Ok(())
    }
}

fn ensure_fresh_generated_source(
    source: &AtpSymbolSource,
    existing_sources: &BTreeSet<AtpSymbolSource>,
    generated_sources: &mut BTreeSet<AtpSymbolSource>,
) -> Result<(), AtpPropertyEncodingError> {
    if existing_sources.contains(source) || !generated_sources.insert(source.clone()) {
        Err(AtpPropertyEncodingError::DuplicateSymbolSource {
            source: source.clone(),
        })
    } else {
        Ok(())
    }
}

fn ensure_unused_declaration_id(
    id: AtpDeclarationId,
    existing_ids: &BTreeSet<AtpDeclarationId>,
) -> Result<(), AtpPropertyEncodingError> {
    if existing_ids.contains(&id) {
        Err(AtpPropertyEncodingError::DuplicateId {
            section: "declarations",
            id: id.index(),
        })
    } else {
        Ok(())
    }
}

fn ensure_unused_provenance_id(
    id: AtpProvenanceId,
    existing_ids: &BTreeSet<AtpProvenanceId>,
) -> Result<(), AtpPropertyEncodingError> {
    if existing_ids.contains(&id) {
        Err(AtpPropertyEncodingError::DuplicateId {
            section: "provenance",
            id: id.index(),
        })
    } else {
        Ok(())
    }
}

fn property_formula(
    family: SupportedPropertyFamily,
    target_symbol: &AtpSymbolName,
    binder_symbols: &[AtpSymbolName],
    binders: Vec<AtpBinder>,
) -> AtpFormulaTree {
    let a = variable(&binder_symbols[0]);
    let body = match family {
        SupportedPropertyFamily::Commutativity => {
            let b = variable(&binder_symbols[1]);
            equality(
                function(target_symbol, vec![a.clone(), b.clone()]),
                function(target_symbol, vec![b, a]),
            )
        }
        SupportedPropertyFamily::Symmetry => {
            let b = variable(&binder_symbols[1]);
            implies(
                atom(target_symbol, vec![a.clone(), b.clone()]),
                atom(target_symbol, vec![b, a]),
            )
        }
        SupportedPropertyFamily::Reflexivity => atom(target_symbol, vec![a.clone(), a]),
        SupportedPropertyFamily::Idempotence => {
            equality(function(target_symbol, vec![a.clone(), a.clone()]), a)
        }
        SupportedPropertyFamily::Involutiveness => equality(
            function(
                target_symbol,
                vec![function(target_symbol, vec![a.clone()])],
            ),
            a,
        ),
        SupportedPropertyFamily::Projectivity => equality(
            function(
                target_symbol,
                vec![function(target_symbol, vec![a.clone()])],
            ),
            function(target_symbol, vec![a]),
        ),
        SupportedPropertyFamily::Asymmetry => {
            let b = variable(&binder_symbols[1]);
            implies(
                atom(target_symbol, vec![a.clone(), b.clone()]),
                AtpFormulaTree::Not(Box::new(atom(target_symbol, vec![b, a]))),
            )
        }
        SupportedPropertyFamily::Connectedness => {
            let b = variable(&binder_symbols[1]);
            implies(
                AtpFormulaTree::Not(Box::new(equality(a.clone(), b.clone()))),
                AtpFormulaTree::Or(vec![
                    atom(target_symbol, vec![a.clone(), b.clone()]),
                    atom(target_symbol, vec![b, a]),
                ]),
            )
        }
        SupportedPropertyFamily::Irreflexivity => {
            AtpFormulaTree::Not(Box::new(atom(target_symbol, vec![a.clone(), a])))
        }
    };
    AtpFormulaTree::Forall {
        binders,
        body: Box::new(body),
    }
}

fn equality(left: AtpTerm, right: AtpTerm) -> AtpFormulaTree {
    AtpFormulaTree::Equality { left, right }
}

fn implies(left: AtpFormulaTree, right: AtpFormulaTree) -> AtpFormulaTree {
    AtpFormulaTree::Implies(Box::new(left), Box::new(right))
}

fn atom(predicate: &AtpSymbolName, arguments: Vec<AtpTerm>) -> AtpFormulaTree {
    AtpFormulaTree::Atom(AtpAtom::new(predicate.clone(), arguments))
}

fn variable(name: &AtpSymbolName) -> AtpTerm {
    AtpTerm::Variable(name.clone())
}

fn function(function: &AtpSymbolName, arguments: Vec<AtpTerm>) -> AtpTerm {
    AtpTerm::Function {
        function: function.clone(),
        arguments,
    }
}

fn property_id(
    base: AtpPropertyId,
    offset: usize,
) -> Result<AtpPropertyId, AtpPropertyEncodingError> {
    Ok(AtpPropertyId::new(add_id_offset(
        base.index(),
        offset,
        "properties",
    )?))
}

fn declaration_id(
    base: AtpDeclarationId,
    offset: usize,
) -> Result<AtpDeclarationId, AtpPropertyEncodingError> {
    Ok(AtpDeclarationId::new(add_id_offset(
        base.index(),
        offset,
        "declarations",
    )?))
}

fn provenance_id(
    base: AtpProvenanceId,
    offset: usize,
    section: &'static str,
) -> Result<AtpProvenanceId, AtpPropertyEncodingError> {
    Ok(AtpProvenanceId::new(add_id_offset(
        base.index(),
        offset,
        section,
    )?))
}

fn add_id_offset(
    base: u32,
    offset: usize,
    section: &'static str,
) -> Result<u32, AtpPropertyEncodingError> {
    let offset =
        u32::try_from(offset).map_err(|_| AtpPropertyEncodingError::IdExhausted { section })?;
    base.checked_add(offset)
        .ok_or(AtpPropertyEncodingError::IdExhausted { section })
}

fn property_source_binding(projection: &AtpPropertyProjection) -> AtpSourceBinding {
    AtpSourceBinding::new(format!(
        "{}#target:{}",
        projection.source_property.as_str(),
        projection.target_source.as_str()
    ))
}

fn binder_source_binding(projection: &AtpPropertyProjection, position: usize) -> AtpSourceBinding {
    AtpSourceBinding::new(format!(
        "{}#target:{}#binder:{position}",
        projection.source_property.as_str(),
        projection.target_source.as_str()
    ))
}

fn binder_payload(identity: &str, position: usize) -> AtpPayload {
    AtpPayload::new(format!("property-binder:{identity}:position:{position}"))
}

fn binder_symbol(identity: &str, position: usize) -> AtpSymbolName {
    AtpSymbolName::new(format!(
        "prop_binder_{}_{position}",
        hex(identity.as_bytes())
    ))
}

fn property_sort_key(
    projection: &AtpPropertyProjection,
    family: SupportedPropertyFamily,
) -> String {
    let mut key = String::new();
    push_key_field(&mut key, "source", projection.source_property.as_str());
    push_key_field(&mut key, "family", family.as_str());
    push_key_field(&mut key, "target-source", projection.target_source.as_str());
    push_key_field(
        &mut key,
        "target-kind",
        target_kind_name(projection.target_kind),
    );
    push_key_field(
        &mut key,
        "target-arity",
        &projection.target_arity.to_string(),
    );
    push_key_field(
        &mut key,
        "strategy",
        encoding_strategy_name(projection.encoding_strategy),
    );
    if let Some(sort) = &projection.binder_sort {
        push_key_field(&mut key, "sort-source", sort.source.as_str());
        push_key_field(&mut key, "sort-symbol", sort.symbol.as_str());
    }
    key
}

fn encoded_identity_key(
    projection: &AtpPropertyProjection,
    family: SupportedPropertyFamily,
) -> String {
    let mut key = String::new();
    push_key_field(&mut key, "family", family.as_str());
    push_key_field(&mut key, "target-source", projection.target_source.as_str());
    push_key_field(
        &mut key,
        "target-kind",
        target_kind_name(projection.target_kind),
    );
    push_key_field(
        &mut key,
        "target-arity",
        &projection.target_arity.to_string(),
    );
    push_key_field(
        &mut key,
        "strategy",
        encoding_strategy_name(projection.encoding_strategy),
    );
    if let Some(sort) = &projection.binder_sort {
        push_key_field(&mut key, "sort-source", sort.source.as_str());
        push_key_field(&mut key, "sort-symbol", sort.symbol.as_str());
    }
    key
}

fn push_key_field(key: &mut String, label: &str, value: &str) {
    key.push_str(label);
    key.push('=');
    key.push_str(&value.len().to_string());
    key.push(':');
    key.push_str(value);
    key.push(';');
}

const fn target_kind_name(kind: AtpPropertyTargetKind) -> &'static str {
    match kind {
        AtpPropertyTargetKind::Function => "function",
        AtpPropertyTargetKind::Predicate => "predicate",
    }
}

const fn encoding_strategy_name(strategy: AtpPropertyEncodingStrategy) -> &'static str {
    match strategy {
        AtpPropertyEncodingStrategy::Axiom => "axiom",
        AtpPropertyEncodingStrategy::NativeDeclaration => "native-declaration",
    }
}

fn hex(bytes: &[u8]) -> String {
    const DIGITS: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(char::from(DIGITS[usize::from(byte >> 4)]));
        output.push(char::from(DIGITS[usize::from(byte & 0x0f)]));
    }
    output
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::problem::{
        AtpDiagnostic, AtpFingerprint, AtpFormula, AtpFormulaId, AtpProblem, AtpProblemParts,
        AtpTargetBinding, AtpTypeContext, ConcreteFormat, ExpectedBackendResult, LogicFragment,
        NativePropertySupport, SoftTypeStrategy,
    };

    #[test]
    fn encodes_every_supported_family_as_axiom_form() {
        for fixture in family_fixtures() {
            let bundle =
                encode_properties(input_for(vec![fixture.projection()])).expect("property bundle");

            assert_eq!(bundle.properties().len(), 1);
            assert_eq!(bundle.declarations().len(), fixture.binder_count);
            assert_eq!(bundle.symbol_map().len(), fixture.binder_count);
            assert_eq!(bundle.provenance().len(), fixture.binder_count + 1);
            assert_eq!(bundle.properties()[0].id(), AtpPropertyId::new(10));
            assert_eq!(
                bundle.properties()[0].target_symbol().as_str(),
                fixture.target
            );
            assert_eq!(
                bundle
                    .declarations()
                    .iter()
                    .map(AtpDeclaration::kind)
                    .collect::<Vec<_>>(),
                vec![AtpDeclarationKind::GeneratedBinder; fixture.binder_count]
            );

            let formula = match bundle.properties()[0].encoding() {
                crate::problem::PropertyEncoding::Axiom(formula) => formula,
                crate::problem::PropertyEncoding::NativeDeclaration(_) => {
                    panic!("task 8 must emit axiom-form properties")
                }
            };
            let AtpFormulaTree::Forall { binders, body } = formula else {
                panic!("property must be universally quantified");
            };
            assert_eq!(binders.len(), fixture.binder_count);
            assert!(binders.iter().all(|binder| binder.sort().is_none()));
            let binder_symbols = binders
                .iter()
                .map(|binder| binder.variable().clone())
                .collect::<Vec<_>>();
            assert_eq!(
                binder_symbols
                    .iter()
                    .map(|symbol| symbol.as_str().to_owned())
                    .collect::<Vec<_>>(),
                bundle
                    .declarations()
                    .iter()
                    .map(|declaration| declaration.symbol().as_str().to_owned())
                    .collect::<Vec<_>>()
            );
            assert_eq!(
                body.as_ref(),
                &fixture.expected_body(&AtpSymbolName::new(fixture.target), &binder_symbols)
            );
        }
    }

    #[test]
    fn generated_binders_are_declared_traceable_and_canonical_under_shuffle() {
        let first = encode_properties(input_for(vec![
            projection(
                "prop:symmetry",
                FAMILY_SYMMETRY,
                "P2",
                "pred:P2",
                AtpPropertyTargetKind::Predicate,
                2,
            ),
            projection(
                "prop:idempotence",
                FAMILY_IDEMPOTENCE,
                "F2",
                "fun:F2",
                AtpPropertyTargetKind::Function,
                2,
            ),
        ]))
        .expect("property bundle");
        let second = encode_properties(input_for(vec![
            projection(
                "prop:idempotence",
                FAMILY_IDEMPOTENCE,
                "F2",
                "fun:F2",
                AtpPropertyTargetKind::Function,
                2,
            ),
            projection(
                "prop:symmetry",
                FAMILY_SYMMETRY,
                "P2",
                "pred:P2",
                AtpPropertyTargetKind::Predicate,
                2,
            ),
        ]))
        .expect("property bundle");

        assert_eq!(first, second);
        assert_eq!(
            first
                .properties()
                .iter()
                .map(EncodedProperty::id)
                .collect::<Vec<_>>(),
            vec![AtpPropertyId::new(10), AtpPropertyId::new(11)]
        );
        assert!(
            first
                .symbol_map()
                .iter()
                .all(|entry| matches!(entry.source(), AtpSymbolSource::GeneratedBinder(_)))
        );
        assert!(first.provenance().iter().all(|entry| {
            matches!(entry.source(), AtpSourceRef::EncodedProperty(_))
                && !entry.payload().is_empty()
        }));
        assert!(
            first.declarations()[0].symbol().as_str().ends_with("_0"),
            "first generated binder records position 0"
        );
        assert!(
            first.declarations()[1].symbol().as_str().ends_with("_1"),
            "second generated binder records position 1"
        );
    }

    #[test]
    fn property_bundle_integrates_with_atp_problem_parts() {
        let bundle = encode_properties(input_for(vec![projection(
            "prop:connectedness",
            FAMILY_CONNECTEDNESS,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]))
        .expect("property bundle");
        let mut parts = minimal_parts();
        parts
            .declarations
            .extend(bundle.declarations().iter().cloned());
        parts.symbol_map.extend(bundle.symbol_map().iter().cloned());
        parts.provenance.extend(bundle.provenance().iter().cloned());
        parts.properties.extend(bundle.properties().iter().cloned());

        let problem = AtpProblem::try_new(parts).expect("problem with encoded property");
        assert_eq!(problem.properties().len(), 1);
        assert!(problem.debug_text().contains("[properties]"));
        assert!(problem.debug_text().contains("encoding=axiom:"));
        assert!(!problem.debug_text().contains("native-declaration"));
    }

    #[test]
    fn rejects_wrong_target_kind_and_arity_for_function_and_predicate_groups() {
        let error = encode_properties(input_for(vec![projection(
            "prop:bad-function-kind",
            FAMILY_COMMUTATIVITY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]))
        .unwrap_err();
        assert_eq!(
            error,
            AtpPropertyEncodingError::InvalidPropertyTarget {
                family: AtpPropertyFamily::new(FAMILY_COMMUTATIVITY),
                expected_kind: AtpPropertyTargetKind::Function,
                expected_arity: 2,
                actual_kind: AtpPropertyTargetKind::Predicate,
                actual_arity: 2,
            }
        );

        let error = encode_properties(input_for(vec![projection(
            "prop:bad-predicate-arity",
            FAMILY_REFLEXIVITY,
            "P1",
            "pred:P1",
            AtpPropertyTargetKind::Predicate,
            1,
        )]))
        .unwrap_err();
        assert_eq!(
            error,
            AtpPropertyEncodingError::InvalidPropertyTarget {
                family: AtpPropertyFamily::new(FAMILY_REFLEXIVITY),
                expected_kind: AtpPropertyTargetKind::Predicate,
                expected_arity: 2,
                actual_kind: AtpPropertyTargetKind::Predicate,
                actual_arity: 1,
            }
        );
    }

    #[test]
    fn rejects_missing_target_declaration_symbol_map_and_provenance_payload() {
        let mut input = input_for(vec![projection(
            "prop:missing-declaration",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        input
            .existing_declarations
            .retain(|declaration| declaration.symbol().as_str() != "P2");
        assert_eq!(
            encode_properties(input).unwrap_err(),
            AtpPropertyEncodingError::MissingDeclarationSymbol {
                symbol: AtpSymbolName::new("P2"),
            }
        );

        let mut input = input_for(vec![projection(
            "prop:missing-symbol",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        input
            .existing_symbol_map
            .retain(|entry| entry.backend_symbol().as_str() != "P2");
        assert_eq!(
            encode_properties(input).unwrap_err(),
            AtpPropertyEncodingError::MissingSymbolMap {
                symbol: AtpSymbolName::new("P2"),
            }
        );

        let mut bad_payload = projection(
            "prop:empty-payload",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        );
        bad_payload.provenance_payload = AtpPayload::new("");
        assert_eq!(
            encode_properties(input_for(vec![bad_payload])).unwrap_err(),
            AtpPropertyEncodingError::EmptyField {
                field: "property.provenance_payload",
            }
        );

        let mut id_collision = input_for(vec![projection(
            "prop:id-collision",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        id_collision.next_provenance_id = AtpProvenanceId::new(1);
        assert_eq!(
            encode_properties(id_collision).unwrap_err(),
            AtpPropertyEncodingError::DuplicateId {
                section: "provenance",
                id: 1,
            }
        );
    }

    #[test]
    fn rejects_mismatched_existing_target_declaration_and_source_identity() {
        let mut wrong_declaration_kind = input_for(vec![projection(
            "prop:wrong-declaration-kind",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        replace_declaration(
            &mut wrong_declaration_kind.existing_declarations,
            "P2",
            AtpDeclarationKind::Function,
            2,
        );
        assert_eq!(
            encode_properties(wrong_declaration_kind).unwrap_err(),
            AtpPropertyEncodingError::InvalidSymbolDeclaration {
                symbol: AtpSymbolName::new("P2"),
                expected: "predicate",
                actual: AtpDeclarationKind::Function,
            }
        );

        let mut wrong_declaration_arity = input_for(vec![projection(
            "prop:wrong-declaration-arity",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        replace_declaration(
            &mut wrong_declaration_arity.existing_declarations,
            "P2",
            AtpDeclarationKind::Predicate,
            1,
        );
        assert_eq!(
            encode_properties(wrong_declaration_arity).unwrap_err(),
            AtpPropertyEncodingError::InvalidSymbolArity {
                symbol: AtpSymbolName::new("P2"),
                expected: 2,
                actual: 1,
            }
        );

        let mut wrong_source = input_for(vec![projection(
            "prop:wrong-source",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        replace_symbol_source(&mut wrong_source.existing_symbol_map, "P2", "pred:other");
        assert_eq!(
            encode_properties(wrong_source).unwrap_err(),
            AtpPropertyEncodingError::InvalidSymbolSource {
                symbol: AtpSymbolName::new("P2"),
                expected: AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P2")),
                actual: AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:other")),
            }
        );

        let mut duplicate_source = input_for(vec![projection(
            "prop:duplicate-symbol-source",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        duplicate_source
            .existing_symbol_map
            .push(AtpSymbolMapEntry::new(
                "P2_alias",
                AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P2")),
            ));
        assert_eq!(
            encode_properties(duplicate_source).unwrap_err(),
            AtpPropertyEncodingError::DuplicateSymbolSource {
                source: AtpSymbolSource::MizarSymbol(AtpSourceBinding::new("pred:P2")),
            }
        );

        let mut duplicate_generated_binder_source = input_for(vec![projection(
            "prop:generated-source-collision",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        duplicate_generated_binder_source
            .existing_symbol_map
            .push(AtpSymbolMapEntry::new(
                "preexisting_binder",
                AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new(
                    "prop:generated-source-collision#target:pred:P2#binder:0",
                )),
            ));
        assert_eq!(
            encode_properties(duplicate_generated_binder_source).unwrap_err(),
            AtpPropertyEncodingError::DuplicateSymbolSource {
                source: AtpSymbolSource::GeneratedBinder(AtpSourceBinding::new(
                    "prop:generated-source-collision#target:pred:P2#binder:0",
                )),
            }
        );
    }

    #[test]
    fn rejects_profile_without_quantifiers_or_required_equality() {
        let mut no_quantifier = input_for(vec![projection(
            "prop:no-quantifier",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        no_quantifier.logic_profile = profile(QuantifierPolicy::PropositionalOnly, true, false);
        assert_eq!(
            encode_properties(no_quantifier).unwrap_err(),
            AtpPropertyEncodingError::UnsupportedProfileFeature {
                feature: "quantifier",
            }
        );

        let mut no_function_equality = input_for(vec![projection(
            "prop:no-function-equality",
            FAMILY_COMMUTATIVITY,
            "F2",
            "fun:F2",
            AtpPropertyTargetKind::Function,
            2,
        )]);
        no_function_equality.logic_profile = profile(QuantifierPolicy::FirstOrder, false, false);
        assert_eq!(
            encode_properties(no_function_equality).unwrap_err(),
            AtpPropertyEncodingError::UnsupportedProfileFeature {
                feature: "equality",
            }
        );

        let mut no_connectedness_equality = input_for(vec![projection(
            "prop:no-connectedness-equality",
            FAMILY_CONNECTEDNESS,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        )]);
        no_connectedness_equality.logic_profile =
            profile(QuantifierPolicy::FirstOrder, false, false);
        assert_eq!(
            encode_properties(no_connectedness_equality).unwrap_err(),
            AtpPropertyEncodingError::UnsupportedProfileFeature {
                feature: "equality",
            }
        );
    }

    #[test]
    fn native_declaration_requests_are_deferred_even_when_profile_supports_them() {
        let mut native = projection(
            "prop:native",
            FAMILY_SYMMETRY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        );
        native.encoding_strategy = AtpPropertyEncodingStrategy::NativeDeclaration;
        let mut input = input_for(vec![native]);
        input.logic_profile = profile(QuantifierPolicy::FirstOrder, true, true);

        assert_eq!(
            encode_properties(input).unwrap_err(),
            AtpPropertyEncodingError::NativeDeclarationDeferred
        );
    }

    #[test]
    fn rejects_unsupported_family_and_duplicate_identities() {
        assert_eq!(
            encode_properties(input_for(vec![projection(
                "prop:assoc",
                "associativity",
                "F2",
                "fun:F2",
                AtpPropertyTargetKind::Function,
                2,
            )]))
            .unwrap_err(),
            AtpPropertyEncodingError::UnsupportedFamily {
                family: AtpPropertyFamily::new("associativity"),
            }
        );

        assert_eq!(
            encode_properties(input_for(vec![
                projection(
                    "prop:duplicate",
                    FAMILY_SYMMETRY,
                    "P2",
                    "pred:P2",
                    AtpPropertyTargetKind::Predicate,
                    2,
                ),
                projection(
                    "prop:duplicate",
                    FAMILY_ASYMMETRY,
                    "P2",
                    "pred:P2",
                    AtpPropertyTargetKind::Predicate,
                    2,
                ),
            ]))
            .unwrap_err(),
            AtpPropertyEncodingError::DuplicateSourceProperty {
                source_property: AtpSourceBinding::new("prop:duplicate"),
            }
        );

        let duplicate_encoded = encode_properties(input_for(vec![
            projection(
                "prop:symmetry-a",
                FAMILY_SYMMETRY,
                "P2",
                "pred:P2",
                AtpPropertyTargetKind::Predicate,
                2,
            ),
            projection(
                "prop:symmetry-b",
                FAMILY_SYMMETRY,
                "P2",
                "pred:P2",
                AtpPropertyTargetKind::Predicate,
                2,
            ),
        ]))
        .unwrap_err();
        assert!(matches!(
            duplicate_encoded,
            AtpPropertyEncodingError::DuplicateEncodedProperty { .. }
        ));
    }

    #[test]
    fn binder_sort_is_validated_and_attached_to_generated_binders() {
        let mut sorted = projection(
            "prop:sorted-reflexivity",
            FAMILY_REFLEXIVITY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        );
        sorted.binder_sort = Some(AtpPropertyBinderSort {
            symbol: AtpSymbolName::new("S"),
            source: AtpSourceBinding::new("sort:S"),
        });

        let bundle = encode_properties(input_for(vec![sorted])).expect("sorted property bundle");
        let crate::problem::PropertyEncoding::Axiom(AtpFormulaTree::Forall { binders, .. }) =
            bundle.properties()[0].encoding()
        else {
            panic!("expected sorted quantified property");
        };
        assert_eq!(binders[0].sort().map(AtpSymbolName::as_str), Some("S"));

        let mut bad_sort = projection(
            "prop:bad-sort",
            FAMILY_REFLEXIVITY,
            "P2",
            "pred:P2",
            AtpPropertyTargetKind::Predicate,
            2,
        );
        bad_sort.binder_sort = Some(AtpPropertyBinderSort {
            symbol: AtpSymbolName::new("missing-sort"),
            source: AtpSourceBinding::new("sort:missing"),
        });
        assert_eq!(
            encode_properties(input_for(vec![bad_sort])).unwrap_err(),
            AtpPropertyEncodingError::MissingSymbolMap {
                symbol: AtpSymbolName::new("missing-sort"),
            }
        );
    }

    struct FamilyFixture {
        family: &'static str,
        target: &'static str,
        target_source: &'static str,
        target_kind: AtpPropertyTargetKind,
        target_arity: u32,
        binder_count: usize,
        expected_body: fn(&AtpSymbolName, &[AtpSymbolName]) -> AtpFormulaTree,
    }

    impl FamilyFixture {
        fn projection(&self) -> AtpPropertyProjection {
            projection(
                &format!("prop:{}", self.family),
                self.family,
                self.target,
                self.target_source,
                self.target_kind,
                self.target_arity,
            )
        }

        fn expected_body(
            &self,
            target_symbol: &AtpSymbolName,
            binder_symbols: &[AtpSymbolName],
        ) -> AtpFormulaTree {
            (self.expected_body)(target_symbol, binder_symbols)
        }
    }

    fn family_fixtures() -> Vec<FamilyFixture> {
        vec![
            FamilyFixture {
                family: FAMILY_COMMUTATIVITY,
                target: "F2",
                target_source: "fun:F2",
                target_kind: AtpPropertyTargetKind::Function,
                target_arity: 2,
                binder_count: 2,
                expected_body: expected_commutativity,
            },
            FamilyFixture {
                family: FAMILY_SYMMETRY,
                target: "P2",
                target_source: "pred:P2",
                target_kind: AtpPropertyTargetKind::Predicate,
                target_arity: 2,
                binder_count: 2,
                expected_body: expected_symmetry,
            },
            FamilyFixture {
                family: FAMILY_REFLEXIVITY,
                target: "P2",
                target_source: "pred:P2",
                target_kind: AtpPropertyTargetKind::Predicate,
                target_arity: 2,
                binder_count: 1,
                expected_body: expected_reflexivity,
            },
            FamilyFixture {
                family: FAMILY_IDEMPOTENCE,
                target: "F2",
                target_source: "fun:F2",
                target_kind: AtpPropertyTargetKind::Function,
                target_arity: 2,
                binder_count: 1,
                expected_body: expected_idempotence,
            },
            FamilyFixture {
                family: FAMILY_INVOLUTIVENESS,
                target: "F1",
                target_source: "fun:F1",
                target_kind: AtpPropertyTargetKind::Function,
                target_arity: 1,
                binder_count: 1,
                expected_body: expected_involutiveness,
            },
            FamilyFixture {
                family: FAMILY_PROJECTIVITY,
                target: "F1",
                target_source: "fun:F1",
                target_kind: AtpPropertyTargetKind::Function,
                target_arity: 1,
                binder_count: 1,
                expected_body: expected_projectivity,
            },
            FamilyFixture {
                family: FAMILY_ASYMMETRY,
                target: "P2",
                target_source: "pred:P2",
                target_kind: AtpPropertyTargetKind::Predicate,
                target_arity: 2,
                binder_count: 2,
                expected_body: expected_asymmetry,
            },
            FamilyFixture {
                family: FAMILY_CONNECTEDNESS,
                target: "P2",
                target_source: "pred:P2",
                target_kind: AtpPropertyTargetKind::Predicate,
                target_arity: 2,
                binder_count: 2,
                expected_body: expected_connectedness,
            },
            FamilyFixture {
                family: FAMILY_IRREFLEXIVITY,
                target: "P2",
                target_source: "pred:P2",
                target_kind: AtpPropertyTargetKind::Predicate,
                target_arity: 2,
                binder_count: 1,
                expected_body: expected_irreflexivity,
            },
        ]
    }

    fn expected_commutativity(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        let b = variable(&binders[1]);
        equality(
            function(target, vec![a.clone(), b.clone()]),
            function(target, vec![b, a]),
        )
    }

    fn expected_symmetry(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        let b = variable(&binders[1]);
        implies(
            atom(target, vec![a.clone(), b.clone()]),
            atom(target, vec![b, a]),
        )
    }

    fn expected_reflexivity(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        atom(target, vec![a.clone(), a])
    }

    fn expected_idempotence(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        equality(function(target, vec![a.clone(), a.clone()]), a)
    }

    fn expected_involutiveness(
        target: &AtpSymbolName,
        binders: &[AtpSymbolName],
    ) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        equality(function(target, vec![function(target, vec![a.clone()])]), a)
    }

    fn expected_projectivity(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        equality(
            function(target, vec![function(target, vec![a.clone()])]),
            function(target, vec![a]),
        )
    }

    fn expected_asymmetry(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        let b = variable(&binders[1]);
        implies(
            atom(target, vec![a.clone(), b.clone()]),
            AtpFormulaTree::Not(Box::new(atom(target, vec![b, a]))),
        )
    }

    fn expected_connectedness(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        let b = variable(&binders[1]);
        implies(
            AtpFormulaTree::Not(Box::new(equality(a.clone(), b.clone()))),
            AtpFormulaTree::Or(vec![
                atom(target, vec![a.clone(), b.clone()]),
                atom(target, vec![b, a]),
            ]),
        )
    }

    fn expected_irreflexivity(target: &AtpSymbolName, binders: &[AtpSymbolName]) -> AtpFormulaTree {
        let a = variable(&binders[0]);
        AtpFormulaTree::Not(Box::new(atom(target, vec![a.clone(), a])))
    }

    fn input_for(property_projections: Vec<AtpPropertyProjection>) -> AtpPropertyEncodingInput {
        AtpPropertyEncodingInput {
            logic_profile: profile(QuantifierPolicy::FirstOrder, true, false),
            existing_declarations: base_declarations(),
            existing_symbol_map: base_symbol_map(),
            existing_provenance: base_provenance(),
            property_projections,
            next_property_id: AtpPropertyId::new(10),
            next_declaration_id: AtpDeclarationId::new(100),
            next_provenance_id: AtpProvenanceId::new(200),
        }
    }

    fn projection(
        source_property: &str,
        family: &str,
        target_symbol: &str,
        target_source: &str,
        target_kind: AtpPropertyTargetKind,
        target_arity: u32,
    ) -> AtpPropertyProjection {
        AtpPropertyProjection {
            source_property: AtpSourceBinding::new(source_property),
            family: AtpPropertyFamily::new(family),
            target_symbol: AtpSymbolName::new(target_symbol),
            target_source: AtpSourceBinding::new(target_source),
            target_kind,
            target_arity,
            binder_sort: None,
            provenance_payload: AtpPayload::new(format!("payload:{source_property}")),
            encoding_strategy: AtpPropertyEncodingStrategy::Axiom,
        }
    }

    fn minimal_parts() -> AtpProblemParts {
        let mut declarations = base_declarations();
        declarations.retain(|declaration| {
            matches!(
                declaration.symbol().as_str(),
                "Q" | "P2" | "F1" | "F2" | "S"
            )
        });
        let mut symbol_map = base_symbol_map();
        symbol_map.retain(|entry| {
            matches!(
                entry.backend_symbol().as_str(),
                "Q" | "P2" | "F1" | "F2" | "S"
            )
        });
        AtpProblemParts {
            vc_id: mizar_vc::vc_ir::VcId::new(11),
            target_binding: AtpTargetBinding::new(
                AtpFingerprint::new(18, b"target-vc-11".to_vec()).expect("fingerprint"),
                AtpSourceBinding::new("vc:11"),
            )
            .expect("target binding"),
            logic_profile: profile(QuantifierPolicy::FirstOrder, true, false),
            expected_result: ExpectedBackendResult::Unsat,
            declarations,
            axioms: vec![AtpFormula::new(
                AtpFormulaId::new(1),
                AtpFormulaTree::Atom(AtpAtom::new("Q", Vec::new())),
                AtpProvenanceId::new(2),
            )],
            conjecture: AtpFormula::new(
                AtpFormulaId::new(2),
                AtpFormulaTree::Atom(AtpAtom::new("Q", Vec::new())),
                AtpProvenanceId::new(3),
            ),
            type_context: AtpTypeContext::new(Vec::new()),
            properties: Vec::new(),
            symbol_map,
            provenance: base_provenance(),
            diagnostics: vec![AtpDiagnostic::new("note", "property fixture")],
        }
    }

    fn base_declarations() -> Vec<AtpDeclaration> {
        vec![
            declaration(1, AtpDeclarationKind::Predicate, "Q", 0),
            declaration(2, AtpDeclarationKind::Predicate, "P2", 2),
            declaration(3, AtpDeclarationKind::Predicate, "P1", 1),
            declaration(4, AtpDeclarationKind::Function, "F2", 2),
            declaration(5, AtpDeclarationKind::Function, "F1", 1),
            declaration(6, AtpDeclarationKind::Sort, "S", 0),
        ]
    }

    fn declaration(id: u32, kind: AtpDeclarationKind, symbol: &str, arity: u32) -> AtpDeclaration {
        AtpDeclaration::new(
            AtpDeclarationId::new(id),
            kind,
            symbol,
            arity,
            AtpProvenanceId::new(1),
        )
    }

    fn base_symbol_map() -> Vec<AtpSymbolMapEntry> {
        vec![
            symbol("Q", "pred:Q"),
            symbol("P2", "pred:P2"),
            symbol("P1", "pred:P1"),
            symbol("F2", "fun:F2"),
            symbol("F1", "fun:F1"),
            symbol("S", "sort:S"),
        ]
    }

    fn symbol(symbol: &str, source: &str) -> AtpSymbolMapEntry {
        AtpSymbolMapEntry::new(
            symbol,
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new(source)),
        )
    }

    fn replace_declaration(
        declarations: &mut [AtpDeclaration],
        symbol: &str,
        kind: AtpDeclarationKind,
        arity: u32,
    ) {
        let declaration = declarations
            .iter_mut()
            .find(|declaration| declaration.symbol().as_str() == symbol)
            .expect("fixture declaration exists");
        *declaration = AtpDeclaration::new(
            declaration.id(),
            kind,
            symbol,
            arity,
            declaration.provenance(),
        );
    }

    fn replace_symbol_source(entries: &mut [AtpSymbolMapEntry], symbol: &str, source: &str) {
        let entry = entries
            .iter_mut()
            .find(|entry| entry.backend_symbol().as_str() == symbol)
            .expect("fixture symbol-map row exists");
        *entry = AtpSymbolMapEntry::new(
            symbol,
            AtpSymbolSource::MizarSymbol(AtpSourceBinding::new(source)),
        );
    }

    fn base_provenance() -> Vec<AtpProvenance> {
        vec![
            AtpProvenance::new(
                AtpProvenanceId::new(1),
                AtpSourceRef::LocalHypothesis(AtpSourceBinding::new("decls")),
                "decl payload",
            ),
            AtpProvenance::new(
                AtpProvenanceId::new(2),
                AtpSourceRef::CitedPremise(AtpSourceBinding::new("premise:1")),
                "axiom payload",
            ),
            AtpProvenance::new(
                AtpProvenanceId::new(3),
                AtpSourceRef::GeneratedVcFact(AtpSourceBinding::new("goal:1")),
                "goal payload",
            ),
        ]
    }

    fn profile(
        quantifiers: QuantifierPolicy,
        equality: bool,
        native_properties: bool,
    ) -> LogicProfile {
        LogicProfile::try_new(
            "property-fixture",
            LogicFragment::Fof,
            if equality {
                EqualitySupport::Supported
            } else {
                EqualitySupport::Unsupported
            },
            quantifiers,
            SoftTypeStrategy::BackendSorts,
            if native_properties {
                NativePropertySupport::Supported
            } else {
                NativePropertySupport::Unsupported
            },
            BTreeSet::from([ConcreteFormat::Tptp]),
        )
        .expect("profile")
    }
}
