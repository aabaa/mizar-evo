//! Syntax-free transport for source-derived evidence requests.

use crate::{
    registration_resolution::{
        ExistentialGateBaseEvidence, ExistentialGateInput, ExistentialGateRecovery,
    },
    source_attribute::{SourceAttributeChainId, SourceAttributeHandoff},
    source_type::{
        SourceTypeApplicationHandoff, SourceTypeApplicationId, SourceTypeExpressionId,
        SourceTypeHead,
    },
    type_checker::ModeExpansion,
    typed_ast::{NodeRecoveryState, TypeFactId, TypeFactTable, TypedSiteRef},
};
use mizar_resolve::{
    env::{SymbolEnv, SymbolKind},
    resolved_ast::ModuleId,
};
use mizar_session::{SourceId, SourceRange};
use std::{
    collections::{BTreeMap, BTreeSet},
    error::Error,
    fmt::{self, Write as _},
};

macro_rules! dense_id {
    ($name:ident) => {
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(usize);

        impl $name {
            pub const fn new(index: usize) -> Self {
                Self(index)
            }

            pub const fn index(self) -> usize {
                self.0
            }
        }
    };
}

dense_id!(SourceEvidenceRequestId);
dense_id!(SourceEvidenceResponseId);

/// Opaque upstream response identity.
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SourceEvidenceResponseKey(String);

impl SourceEvidenceResponseKey {
    pub fn new(value: impl Into<String>) -> Self {
        Self(value.into())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl From<&str> for SourceEvidenceResponseKey {
    fn from(value: &str) -> Self {
        Self::new(value)
    }
}

impl From<String> for SourceEvidenceResponseKey {
    fn from(value: String) -> Self {
        Self::new(value)
    }
}

/// Complete input for one source/module evidence transaction.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidenceHandoffInput {
    pub source_id: SourceId,
    pub module_id: ModuleId,
    pub requests: Vec<SourceEvidenceRequestInput>,
    pub responses: Vec<SourceEvidenceResponseInput>,
}

/// One source-derived evidence request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidenceRequestInput {
    pub owner: TypedSiteRef,
    pub site: TypedSiteRef,
    pub source_range: SourceRange,
    pub source_ordinal: usize,
    pub recovery: SourceEvidenceRecovery,
    pub kind: SourceEvidenceRequestKind,
    pub state: SourceEvidenceInputState,
    pub origin: SourceEvidenceRequestOrigin,
}

/// One ordered response reference attached to a request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidenceResponseInput {
    pub request: SourceEvidenceRequestId,
    pub ordinal: usize,
    pub key: SourceEvidenceResponseKey,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceEvidenceRequestKind {
    ModeExpansion,
    StructureInhabitation,
    AttributedTypeInhabitation,
    Sethood,
    NonEmptiness,
    InheritancePath,
    CoercionViability,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceEvidenceInputState {
    Requested,
    Missing,
    Rejected,
    Supplied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceEvidenceRequestOrigin {
    SourceTypeApplication {
        application: SourceTypeApplicationId,
        expression: SourceTypeExpressionId,
        attribute_chain: Option<SourceAttributeChainId>,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceEvidenceResponseDisposition {
    Rejected,
    Supplied,
}

#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceEvidenceResponsePayload {
    ModeExpansion(ModeExpansion),
    StructureBaseEvidence(ExistentialGateBaseEvidence),
    ExistentialGate(ExistentialGateInput),
    TypeFact(TypeFactId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceEvidenceResponseProvenance {
    ExplicitInput,
    ExternalDependency,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
#[non_exhaustive]
pub enum SourceEvidenceRecovery {
    Normal,
    Degraded,
}

/// Catalog row authenticated against its exact parent request.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidenceDependencyRecord {
    key: SourceEvidenceResponseKey,
    request: SourceEvidenceRequestId,
    disposition: SourceEvidenceResponseDisposition,
    provenance: SourceEvidenceResponseProvenance,
    payload: Option<SourceEvidenceResponsePayload>,
}

impl SourceEvidenceDependencyRecord {
    pub fn new(
        key: impl Into<SourceEvidenceResponseKey>,
        request: SourceEvidenceRequestId,
        disposition: SourceEvidenceResponseDisposition,
        provenance: SourceEvidenceResponseProvenance,
        payload: Option<SourceEvidenceResponsePayload>,
    ) -> Self {
        Self {
            key: key.into(),
            request,
            disposition,
            provenance,
            payload,
        }
    }

    pub const fn key(&self) -> &SourceEvidenceResponseKey {
        &self.key
    }

    pub const fn request(&self) -> SourceEvidenceRequestId {
        self.request
    }

    pub const fn disposition(&self) -> SourceEvidenceResponseDisposition {
        self.disposition
    }

    pub const fn provenance(&self) -> SourceEvidenceResponseProvenance {
        self.provenance
    }

    pub const fn payload(&self) -> Option<&SourceEvidenceResponsePayload> {
        self.payload.as_ref()
    }
}

/// Input-only upstream dependency authentication table.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceEvidenceDependencyCatalog {
    records: Vec<SourceEvidenceDependencyRecord>,
}

impl SourceEvidenceDependencyCatalog {
    pub const fn new(records: Vec<SourceEvidenceDependencyRecord>) -> Self {
        Self { records }
    }

    pub const fn empty() -> Self {
        Self {
            records: Vec::new(),
        }
    }

    pub fn get(&self, key: &SourceEvidenceResponseKey) -> Option<&SourceEvidenceDependencyRecord> {
        self.records.iter().find(|record| record.key == *key)
    }

    pub fn iter(&self) -> impl Iterator<Item = &SourceEvidenceDependencyRecord> {
        self.records.iter()
    }

    pub const fn len(&self) -> usize {
        self.records.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.records.is_empty()
    }
}

/// Immutable validated evidence handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidenceHandoff {
    source_id: SourceId,
    module_id: ModuleId,
    requests: SourceEvidenceRequestTable,
    responses: SourceEvidenceResponseTable,
}

impl SourceEvidenceHandoff {
    pub const fn source_id(&self) -> SourceId {
        self.source_id
    }

    pub const fn module_id(&self) -> &ModuleId {
        &self.module_id
    }

    pub const fn requests(&self) -> &SourceEvidenceRequestTable {
        &self.requests
    }

    pub const fn responses(&self) -> &SourceEvidenceResponseTable {
        &self.responses
    }

    pub fn debug_text(&self) -> String {
        let mut output = String::from("source-evidence-debug-v1\n");
        output.push_str("module: ");
        output.push_str(self.module_id.path().as_str());
        output.push('\n');
        for (id, request) in self.requests.iter() {
            let _ = write!(
                output,
                "request#{} ordinal={} kind={} state={} range={}..{} owner=",
                id.index(),
                request.source_ordinal,
                request_kind_key(request.kind),
                state_key(request.state),
                request.source_range.start,
                request.source_range.end
            );
            write_site(&mut output, &request.owner);
            output.push_str(" site=");
            write_site(&mut output, &request.site);
            let _ = write!(output, " recovery={} ", recovery_key(request.recovery));
            write_origin(&mut output, &request.origin);
            output.push('\n');
        }
        for (id, response) in self.responses.iter() {
            let _ = write!(
                output,
                "response#{} request={} ordinal={} key={:?} disposition={} provenance={} payload=",
                id.index(),
                response.request.index(),
                response.ordinal,
                response.key.as_str(),
                disposition_key(response.disposition),
                provenance_key(response.provenance)
            );
            write_payload(&mut output, response.payload.as_ref());
            output.push('\n');
        }
        output
    }

    pub(crate) fn validate_installation(
        &self,
        source_id: SourceId,
        module_id: &ModuleId,
        source_type: &SourceTypeApplicationHandoff,
        source_attribute: Option<&SourceAttributeHandoff>,
        facts: &TypeFactTable,
    ) -> Result<(), SourceEvidenceError> {
        if self.source_id != source_id
            || &self.module_id != module_id
            || source_type.source_id() != source_id
            || source_type.module_id() != module_id
            || source_attribute.is_some_and(|attribute| {
                attribute.source_id() != source_id || attribute.module_id() != module_id
            })
        {
            return Err(SourceEvidenceError::EnvironmentMismatch);
        }
        validate_published_requests(self, source_type, source_attribute)?;
        validate_published_responses(self, facts)
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceEvidenceRequestTable {
    entries: Vec<SourceEvidenceRequest>,
}

impl SourceEvidenceRequestTable {
    pub fn get(&self, id: SourceEvidenceRequestId) -> Option<&SourceEvidenceRequest> {
        self.entries.get(id.index())
    }

    pub fn iter(&self) -> impl Iterator<Item = (SourceEvidenceRequestId, &SourceEvidenceRequest)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidenceRequest {
    id: SourceEvidenceRequestId,
    owner: TypedSiteRef,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceEvidenceRecovery,
    kind: SourceEvidenceRequestKind,
    state: SourceEvidenceInputState,
    origin: SourceEvidenceRequestOrigin,
}

impl SourceEvidenceRequest {
    pub const fn id(&self) -> SourceEvidenceRequestId {
        self.id
    }

    pub const fn owner(&self) -> &TypedSiteRef {
        &self.owner
    }

    pub const fn site(&self) -> &TypedSiteRef {
        &self.site
    }

    pub const fn source_range(&self) -> SourceRange {
        self.source_range
    }

    pub const fn source_ordinal(&self) -> usize {
        self.source_ordinal
    }

    pub const fn recovery(&self) -> SourceEvidenceRecovery {
        self.recovery
    }

    pub const fn kind(&self) -> SourceEvidenceRequestKind {
        self.kind
    }

    pub const fn state(&self) -> SourceEvidenceInputState {
        self.state
    }

    pub const fn origin(&self) -> &SourceEvidenceRequestOrigin {
        &self.origin
    }
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct SourceEvidenceResponseTable {
    entries: Vec<SourceEvidenceResponse>,
}

impl SourceEvidenceResponseTable {
    pub fn get(&self, id: SourceEvidenceResponseId) -> Option<&SourceEvidenceResponse> {
        self.entries.get(id.index())
    }

    pub fn iter(
        &self,
    ) -> impl Iterator<Item = (SourceEvidenceResponseId, &SourceEvidenceResponse)> {
        self.entries.iter().map(|entry| (entry.id, entry))
    }

    pub const fn len(&self) -> usize {
        self.entries.len()
    }

    pub const fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceEvidenceResponse {
    id: SourceEvidenceResponseId,
    request: SourceEvidenceRequestId,
    ordinal: usize,
    key: SourceEvidenceResponseKey,
    disposition: SourceEvidenceResponseDisposition,
    provenance: SourceEvidenceResponseProvenance,
    payload: Option<SourceEvidenceResponsePayload>,
}

impl SourceEvidenceResponse {
    pub const fn id(&self) -> SourceEvidenceResponseId {
        self.id
    }

    pub const fn request(&self) -> SourceEvidenceRequestId {
        self.request
    }

    pub const fn ordinal(&self) -> usize {
        self.ordinal
    }

    pub const fn key(&self) -> &SourceEvidenceResponseKey {
        &self.key
    }

    pub const fn disposition(&self) -> SourceEvidenceResponseDisposition {
        self.disposition
    }

    pub const fn provenance(&self) -> SourceEvidenceResponseProvenance {
        self.provenance
    }

    pub const fn payload(&self) -> Option<&SourceEvidenceResponsePayload> {
        self.payload.as_ref()
    }
}

/// Transaction validation failures. No failure publishes a partial handoff.
#[derive(Debug, Clone, PartialEq, Eq)]
#[non_exhaustive]
pub enum SourceEvidenceError {
    EnvironmentMismatch,
    InvalidSymbolHead {
        application: SourceTypeApplicationId,
    },
    InvalidSourceAttribute,
    RequestCardinalityMismatch,
    InvalidRequest {
        request: SourceEvidenceRequestId,
    },
    WrongRequestKind {
        request: SourceEvidenceRequestId,
    },
    InvalidResponse {
        response: SourceEvidenceResponseId,
    },
    EmptyResponseKey {
        response: Option<SourceEvidenceResponseId>,
    },
    DuplicateCatalogKey,
    DuplicateResponseKey {
        response: SourceEvidenceResponseId,
    },
    MissingCatalogRecord {
        response: SourceEvidenceResponseId,
    },
    CrossRequestCatalogRecord {
        response: SourceEvidenceResponseId,
    },
    StaleCatalogRecord,
    InvalidCatalogRecord,
    InvalidStateCardinality {
        request: SourceEvidenceRequestId,
    },
    InvalidPayloadKind {
        request: SourceEvidenceRequestId,
    },
    InvalidTypeFact {
        fact: TypeFactId,
    },
    InvalidExistentialGate {
        request: SourceEvidenceRequestId,
    },
}

impl fmt::Display for SourceEvidenceError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EnvironmentMismatch => {
                formatter.write_str("source evidence environment mismatch")
            }
            Self::InvalidSymbolHead { application } => write!(
                formatter,
                "source evidence application {} has an invalid symbol head",
                application.index()
            ),
            Self::InvalidSourceAttribute => {
                formatter.write_str("source evidence attribute association is invalid")
            }
            Self::RequestCardinalityMismatch => {
                formatter.write_str("source evidence request cardinality is invalid")
            }
            Self::InvalidRequest { request } => write!(
                formatter,
                "source evidence request {} is invalid",
                request.index()
            ),
            Self::WrongRequestKind { request } => write!(
                formatter,
                "source evidence request {} has the wrong kind",
                request.index()
            ),
            Self::InvalidResponse { response } => write!(
                formatter,
                "source evidence response {} is invalid",
                response.index()
            ),
            Self::EmptyResponseKey {
                response: Some(response),
            } => write!(
                formatter,
                "source evidence response {} has an empty key",
                response.index()
            ),
            Self::EmptyResponseKey { response: None } => {
                formatter.write_str("source evidence catalog has an empty key")
            }
            Self::DuplicateCatalogKey => {
                formatter.write_str("source evidence catalog has a duplicate key")
            }
            Self::DuplicateResponseKey { response } => write!(
                formatter,
                "source evidence response {} reuses a key",
                response.index()
            ),
            Self::MissingCatalogRecord { response } => write!(
                formatter,
                "source evidence response {} has no catalog record",
                response.index()
            ),
            Self::CrossRequestCatalogRecord { response } => write!(
                formatter,
                "source evidence response {} has a cross-request catalog record",
                response.index()
            ),
            Self::StaleCatalogRecord => {
                formatter.write_str("source evidence catalog has an unconsumed record")
            }
            Self::InvalidCatalogRecord => {
                formatter.write_str("source evidence catalog record is invalid")
            }
            Self::InvalidStateCardinality { request } => write!(
                formatter,
                "source evidence request {} has invalid response cardinality",
                request.index()
            ),
            Self::InvalidPayloadKind { request } => write!(
                formatter,
                "source evidence request {} has an incompatible payload",
                request.index()
            ),
            Self::InvalidTypeFact { fact } => write!(
                formatter,
                "source evidence references missing fact {}",
                fact.index()
            ),
            Self::InvalidExistentialGate { request } => write!(
                formatter,
                "source evidence request {} has an invalid existential gate",
                request.index()
            ),
        }
    }
}

impl Error for SourceEvidenceError {}

/// Validates and transactionally constructs source-evidence handoffs.
pub struct SourceEvidenceProducer;

impl SourceEvidenceProducer {
    pub fn build(
        input: SourceEvidenceHandoffInput,
        source_type: &SourceTypeApplicationHandoff,
        source_attribute: Option<&SourceAttributeHandoff>,
        symbols: &SymbolEnv,
        facts: &TypeFactTable,
        catalog: &SourceEvidenceDependencyCatalog,
    ) -> Result<SourceEvidenceHandoff, SourceEvidenceError> {
        validate_input(
            &input,
            source_type,
            source_attribute,
            symbols,
            facts,
            catalog,
        )?;

        let requests = SourceEvidenceRequestTable {
            entries: input
                .requests
                .into_iter()
                .enumerate()
                .map(|(index, row)| SourceEvidenceRequest {
                    id: SourceEvidenceRequestId::new(index),
                    owner: row.owner,
                    site: row.site,
                    source_range: row.source_range,
                    source_ordinal: row.source_ordinal,
                    recovery: row.recovery,
                    kind: row.kind,
                    state: row.state,
                    origin: row.origin,
                })
                .collect(),
        };
        let responses = SourceEvidenceResponseTable {
            entries: input
                .responses
                .into_iter()
                .enumerate()
                .map(|(index, row)| {
                    let record = catalog
                        .get(&row.key)
                        .expect("validated source-evidence catalog lookup");
                    SourceEvidenceResponse {
                        id: SourceEvidenceResponseId::new(index),
                        request: row.request,
                        ordinal: row.ordinal,
                        key: row.key,
                        disposition: record.disposition,
                        provenance: record.provenance,
                        payload: record.payload.clone(),
                    }
                })
                .collect(),
        };
        Ok(SourceEvidenceHandoff {
            source_id: input.source_id,
            module_id: input.module_id,
            requests,
            responses,
        })
    }
}

#[derive(Clone)]
struct ExpectedRequest {
    application: SourceTypeApplicationId,
    expression: SourceTypeExpressionId,
    attribute_chain: Option<SourceAttributeChainId>,
    owner: TypedSiteRef,
    site: TypedSiteRef,
    source_range: SourceRange,
    source_ordinal: usize,
    recovery: SourceEvidenceRecovery,
    kind: Option<SourceEvidenceRequestKind>,
}

fn validate_input(
    input: &SourceEvidenceHandoffInput,
    source_type: &SourceTypeApplicationHandoff,
    source_attribute: Option<&SourceAttributeHandoff>,
    symbols: &SymbolEnv,
    facts: &TypeFactTable,
    catalog: &SourceEvidenceDependencyCatalog,
) -> Result<(), SourceEvidenceError> {
    if input.source_id != source_type.source_id()
        || &input.module_id != source_type.module_id()
        || symbols.module_id() != &input.module_id
        || source_attribute.is_some_and(|attribute| {
            attribute.source_id() != input.source_id || attribute.module_id() != &input.module_id
        })
    {
        return Err(SourceEvidenceError::EnvironmentMismatch);
    }
    validate_catalog_keys(catalog)?;
    let expected = expected_requests(source_type, source_attribute, Some(symbols))?;
    let eligible = expected
        .iter()
        .filter(|request| request.kind.is_some())
        .collect::<Vec<_>>();
    if input.requests.len() != eligible.len() {
        return Err(SourceEvidenceError::RequestCardinalityMismatch);
    }
    for (index, (actual, expected)) in input.requests.iter().zip(eligible).enumerate() {
        validate_request(SourceEvidenceRequestId::new(index), actual, expected)?;
    }
    validate_input_responses(input, facts, catalog)
}

fn validate_catalog_keys(
    catalog: &SourceEvidenceDependencyCatalog,
) -> Result<(), SourceEvidenceError> {
    let mut keys = BTreeSet::new();
    for record in catalog.iter() {
        if record.key.as_str().trim().is_empty() {
            return Err(SourceEvidenceError::EmptyResponseKey { response: None });
        }
        if !keys.insert(record.key.clone()) {
            return Err(SourceEvidenceError::DuplicateCatalogKey);
        }
    }
    Ok(())
}

fn expected_requests(
    source_type: &SourceTypeApplicationHandoff,
    source_attribute: Option<&SourceAttributeHandoff>,
    symbols: Option<&SymbolEnv>,
) -> Result<Vec<ExpectedRequest>, SourceEvidenceError> {
    let mut chains_by_expression = BTreeMap::new();
    if let Some(source_attribute) = source_attribute {
        for (chain_id, chain) in source_attribute.chains().iter() {
            if chains_by_expression
                .insert(chain.expression(), (chain_id, chain))
                .is_some()
            {
                return Err(SourceEvidenceError::InvalidSourceAttribute);
            }
        }
    }

    let mut expected = Vec::with_capacity(source_type.applications().len());
    for (application_id, application) in source_type.applications().iter() {
        let expression = source_type.expressions().get(application.root()).ok_or(
            SourceEvidenceError::InvalidSymbolHead {
                application: application_id,
            },
        )?;
        let (attribute_chain, owner, site, source_range, recovery, kind) =
            if let Some((chain_id, chain)) = chains_by_expression.remove(&application.root()) {
                (
                    Some(chain_id),
                    expression.site().clone(),
                    chain.site().clone(),
                    chain.source_range(),
                    map_recovery(chain.recovery()),
                    Some(SourceEvidenceRequestKind::AttributedTypeInhabitation),
                )
            } else {
                let kind = match expression.head() {
                    SourceTypeHead::BuiltinSet | SourceTypeHead::BuiltinObject => None,
                    SourceTypeHead::Symbol {
                        symbol,
                        contribution,
                    } => {
                        if let Some(symbols) = symbols {
                            let entry = symbols.symbols().get(symbol).ok_or(
                                SourceEvidenceError::InvalidSymbolHead {
                                    application: application_id,
                                },
                            )?;
                            if entry.contribution() != *contribution {
                                return Err(SourceEvidenceError::InvalidSymbolHead {
                                    application: application_id,
                                });
                            }
                            match entry.kind() {
                                SymbolKind::Mode => Some(SourceEvidenceRequestKind::ModeExpansion),
                                SymbolKind::Structure => {
                                    Some(SourceEvidenceRequestKind::StructureInhabitation)
                                }
                                _ => {
                                    return Err(SourceEvidenceError::InvalidSymbolHead {
                                        application: application_id,
                                    });
                                }
                            }
                        } else {
                            Some(SourceEvidenceRequestKind::ModeExpansion)
                        }
                    }
                };
                (
                    None,
                    expression.site().clone(),
                    expression.head_site().clone(),
                    expression.source_range(),
                    map_recovery(expression.recovery()),
                    kind,
                )
            };
        expected.push(ExpectedRequest {
            application: application_id,
            expression: application.root(),
            attribute_chain,
            owner,
            site,
            source_range,
            source_ordinal: application.source_ordinal(),
            recovery,
            kind,
        });
    }
    if !chains_by_expression.is_empty() {
        return Err(SourceEvidenceError::InvalidSourceAttribute);
    }
    Ok(expected)
}

fn validate_request(
    id: SourceEvidenceRequestId,
    actual: &SourceEvidenceRequestInput,
    expected: &ExpectedRequest,
) -> Result<(), SourceEvidenceError> {
    if actual.kind != expected.kind.expect("eligible request has a kind") {
        return Err(SourceEvidenceError::WrongRequestKind { request: id });
    }
    let SourceEvidenceRequestOrigin::SourceTypeApplication {
        application,
        expression,
        attribute_chain,
    } = &actual.origin;
    if *application != expected.application
        || *expression != expected.expression
        || *attribute_chain != expected.attribute_chain
        || actual.owner != expected.owner
        || actual.site != expected.site
        || actual.source_range != expected.source_range
        || actual.source_ordinal != expected.source_ordinal
        || actual.recovery != expected.recovery
    {
        return Err(SourceEvidenceError::InvalidRequest { request: id });
    }
    Ok(())
}

fn validate_input_responses(
    input: &SourceEvidenceHandoffInput,
    facts: &TypeFactTable,
    catalog: &SourceEvidenceDependencyCatalog,
) -> Result<(), SourceEvidenceError> {
    let mut response_counts = vec![0usize; input.requests.len()];
    let mut previous_request = None;
    let mut consumed = BTreeSet::new();
    for (index, response) in input.responses.iter().enumerate() {
        let id = SourceEvidenceResponseId::new(index);
        let Some(request) = input.requests.get(response.request.index()) else {
            return Err(SourceEvidenceError::InvalidResponse { response: id });
        };
        if previous_request.is_some_and(|previous| response.request < previous)
            || response.ordinal != response_counts[response.request.index()]
        {
            return Err(SourceEvidenceError::InvalidResponse { response: id });
        }
        if response.key.as_str().trim().is_empty() {
            return Err(SourceEvidenceError::EmptyResponseKey { response: Some(id) });
        }
        if !consumed.insert(response.key.clone()) {
            return Err(SourceEvidenceError::DuplicateResponseKey { response: id });
        }
        let record = catalog
            .get(&response.key)
            .ok_or(SourceEvidenceError::MissingCatalogRecord { response: id })?;
        if record.request != response.request {
            return Err(SourceEvidenceError::CrossRequestCatalogRecord { response: id });
        }
        validate_record(response.request, request, record, facts)?;
        response_counts[response.request.index()] += 1;
        previous_request = Some(response.request);
    }
    for (index, (request, count)) in input.requests.iter().zip(response_counts).enumerate() {
        let valid = match request.state {
            SourceEvidenceInputState::Requested | SourceEvidenceInputState::Missing => count == 0,
            SourceEvidenceInputState::Rejected => count == 1,
            SourceEvidenceInputState::Supplied => count > 0,
        };
        if !valid {
            return Err(SourceEvidenceError::InvalidStateCardinality {
                request: SourceEvidenceRequestId::new(index),
            });
        }
    }
    if consumed.len() != catalog.len() {
        return Err(SourceEvidenceError::StaleCatalogRecord);
    }
    Ok(())
}

fn validate_record(
    request_id: SourceEvidenceRequestId,
    request: &SourceEvidenceRequestInput,
    record: &SourceEvidenceDependencyRecord,
    facts: &TypeFactTable,
) -> Result<(), SourceEvidenceError> {
    match (record.disposition, record.payload.as_ref()) {
        (SourceEvidenceResponseDisposition::Rejected, None)
            if request.state == SourceEvidenceInputState::Rejected => {}
        (SourceEvidenceResponseDisposition::Supplied, Some(payload))
            if request.state == SourceEvidenceInputState::Supplied =>
        {
            validate_payload(request_id, request, payload, facts)?;
        }
        (SourceEvidenceResponseDisposition::Rejected, Some(_) | None)
        | (SourceEvidenceResponseDisposition::Supplied, Some(_) | None) => {
            return Err(SourceEvidenceError::InvalidCatalogRecord);
        }
    }
    Ok(())
}

fn validate_payload(
    request_id: SourceEvidenceRequestId,
    request: &SourceEvidenceRequestInput,
    payload: &SourceEvidenceResponsePayload,
    facts: &TypeFactTable,
) -> Result<(), SourceEvidenceError> {
    match payload {
        SourceEvidenceResponsePayload::ModeExpansion(_)
            if request.kind == SourceEvidenceRequestKind::ModeExpansion => {}
        SourceEvidenceResponsePayload::StructureBaseEvidence(_)
            if request.kind == SourceEvidenceRequestKind::StructureInhabitation => {}
        SourceEvidenceResponsePayload::ExistentialGate(gate)
            if request.kind == SourceEvidenceRequestKind::AttributedTypeInhabitation =>
        {
            if gate.owner() != &request.owner
                || gate.source_range() != request.source_range
                || map_gate_recovery(gate.recovery()) != request.recovery
            {
                return Err(SourceEvidenceError::InvalidExistentialGate {
                    request: request_id,
                });
            }
            for evidence in gate.guard_evidence() {
                if facts.get(evidence.fact()).is_none() {
                    return Err(SourceEvidenceError::InvalidTypeFact {
                        fact: evidence.fact(),
                    });
                }
            }
        }
        SourceEvidenceResponsePayload::TypeFact(fact) => {
            if facts.get(*fact).is_none() {
                return Err(SourceEvidenceError::InvalidTypeFact { fact: *fact });
            }
        }
        _ => {
            return Err(SourceEvidenceError::InvalidPayloadKind {
                request: request_id,
            });
        }
    }
    Ok(())
}

fn validate_published_requests(
    handoff: &SourceEvidenceHandoff,
    source_type: &SourceTypeApplicationHandoff,
    source_attribute: Option<&SourceAttributeHandoff>,
) -> Result<(), SourceEvidenceError> {
    let expected = expected_requests(source_type, source_attribute, None)?;
    let eligible = expected
        .iter()
        .filter(|request| request.kind.is_some())
        .collect::<Vec<_>>();
    if handoff.requests.len() != eligible.len() {
        return Err(SourceEvidenceError::RequestCardinalityMismatch);
    }
    for ((id, actual), expected) in handoff.requests.iter().zip(eligible) {
        let SourceEvidenceRequestOrigin::SourceTypeApplication {
            application,
            expression,
            attribute_chain,
        } = &actual.origin;
        let kind_valid = match expected.attribute_chain {
            Some(_) => actual.kind == SourceEvidenceRequestKind::AttributedTypeInhabitation,
            None => matches!(
                actual.kind,
                SourceEvidenceRequestKind::ModeExpansion
                    | SourceEvidenceRequestKind::StructureInhabitation
            ),
        };
        if !kind_valid
            || *application != expected.application
            || *expression != expected.expression
            || *attribute_chain != expected.attribute_chain
            || actual.owner != expected.owner
            || actual.site != expected.site
            || actual.source_range != expected.source_range
            || actual.source_ordinal != expected.source_ordinal
            || actual.recovery != expected.recovery
        {
            return Err(SourceEvidenceError::InvalidRequest { request: id });
        }
    }
    Ok(())
}

fn validate_published_responses(
    handoff: &SourceEvidenceHandoff,
    facts: &TypeFactTable,
) -> Result<(), SourceEvidenceError> {
    let mut counts = vec![0usize; handoff.requests.len()];
    let mut previous_request = None;
    let mut keys = BTreeSet::new();
    for (id, response) in handoff.responses.iter() {
        let Some(request) = handoff.requests.get(response.request) else {
            return Err(SourceEvidenceError::InvalidResponse { response: id });
        };
        if previous_request.is_some_and(|previous| response.request < previous)
            || response.ordinal != counts[response.request.index()]
            || response.key.as_str().trim().is_empty()
            || !keys.insert(response.key.clone())
        {
            return Err(SourceEvidenceError::InvalidResponse { response: id });
        }
        let record = SourceEvidenceDependencyRecord {
            key: response.key.clone(),
            request: response.request,
            disposition: response.disposition,
            provenance: response.provenance,
            payload: response.payload.clone(),
        };
        let input = SourceEvidenceRequestInput {
            owner: request.owner.clone(),
            site: request.site.clone(),
            source_range: request.source_range,
            source_ordinal: request.source_ordinal,
            recovery: request.recovery,
            kind: request.kind,
            state: request.state,
            origin: request.origin.clone(),
        };
        validate_record(response.request, &input, &record, facts)?;
        counts[response.request.index()] += 1;
        previous_request = Some(response.request);
    }
    for ((id, request), count) in handoff.requests.iter().zip(counts) {
        let valid = match request.state {
            SourceEvidenceInputState::Requested | SourceEvidenceInputState::Missing => count == 0,
            SourceEvidenceInputState::Rejected => count == 1,
            SourceEvidenceInputState::Supplied => count > 0,
        };
        if !valid {
            return Err(SourceEvidenceError::InvalidStateCardinality { request: id });
        }
    }
    Ok(())
}

fn map_recovery(recovery: NodeRecoveryState) -> SourceEvidenceRecovery {
    match recovery {
        NodeRecoveryState::Normal => SourceEvidenceRecovery::Normal,
        NodeRecoveryState::Recovered | NodeRecoveryState::Degraded => {
            SourceEvidenceRecovery::Degraded
        }
    }
}

fn map_gate_recovery(recovery: ExistentialGateRecovery) -> SourceEvidenceRecovery {
    match recovery {
        ExistentialGateRecovery::Normal => SourceEvidenceRecovery::Normal,
        ExistentialGateRecovery::Degraded => SourceEvidenceRecovery::Degraded,
    }
}

fn request_kind_key(kind: SourceEvidenceRequestKind) -> &'static str {
    match kind {
        SourceEvidenceRequestKind::ModeExpansion => "mode-expansion",
        SourceEvidenceRequestKind::StructureInhabitation => "structure-inhabitation",
        SourceEvidenceRequestKind::AttributedTypeInhabitation => "attributed-type-inhabitation",
        SourceEvidenceRequestKind::Sethood => "sethood",
        SourceEvidenceRequestKind::NonEmptiness => "non-emptiness",
        SourceEvidenceRequestKind::InheritancePath => "inheritance-path",
        SourceEvidenceRequestKind::CoercionViability => "coercion-viability",
    }
}

fn state_key(state: SourceEvidenceInputState) -> &'static str {
    match state {
        SourceEvidenceInputState::Requested => "requested",
        SourceEvidenceInputState::Missing => "missing",
        SourceEvidenceInputState::Rejected => "rejected",
        SourceEvidenceInputState::Supplied => "supplied",
    }
}

fn recovery_key(recovery: SourceEvidenceRecovery) -> &'static str {
    match recovery {
        SourceEvidenceRecovery::Normal => "normal",
        SourceEvidenceRecovery::Degraded => "degraded",
    }
}

fn disposition_key(disposition: SourceEvidenceResponseDisposition) -> &'static str {
    match disposition {
        SourceEvidenceResponseDisposition::Rejected => "rejected",
        SourceEvidenceResponseDisposition::Supplied => "supplied",
    }
}

fn provenance_key(provenance: SourceEvidenceResponseProvenance) -> &'static str {
    match provenance {
        SourceEvidenceResponseProvenance::ExplicitInput => "explicit-input",
        SourceEvidenceResponseProvenance::ExternalDependency => "external-dependency",
    }
}

fn write_site(output: &mut String, site: &TypedSiteRef) {
    match site {
        TypedSiteRef::Node(node) => {
            let _ = write!(output, "node#{}", node.index());
        }
        TypedSiteRef::Role { node, role } => {
            let _ = write!(output, "node#{}:{}", node.index(), role.as_str());
        }
    }
}

fn write_origin(output: &mut String, origin: &SourceEvidenceRequestOrigin) {
    match origin {
        SourceEvidenceRequestOrigin::SourceTypeApplication {
            application,
            expression,
            attribute_chain,
        } => {
            let _ = write!(
                output,
                "origin=source-type(application={},expression={},attribute-chain=",
                application.index(),
                expression.index()
            );
            if let Some(chain) = attribute_chain {
                let _ = write!(output, "{}", chain.index());
            } else {
                output.push_str("none");
            }
            output.push(')');
        }
    }
}

fn write_payload(output: &mut String, payload: Option<&SourceEvidenceResponsePayload>) {
    match payload {
        None => output.push_str("none"),
        Some(SourceEvidenceResponsePayload::ModeExpansion(_)) => output.push_str("mode-expansion"),
        Some(SourceEvidenceResponsePayload::StructureBaseEvidence(evidence)) => {
            let _ = write!(
                output,
                "structure-base(kind={:?},pattern={:?},coverage={:?})",
                evidence.kind(),
                evidence.pattern().as_str(),
                evidence.coverage()
            );
        }
        Some(SourceEvidenceResponsePayload::ExistentialGate(gate)) => {
            output.push_str("existential-gate(owner=");
            write_site(output, gate.owner());
            let _ = write!(
                output,
                ",range={}..{},recovery={})",
                gate.source_range().start,
                gate.source_range().end,
                recovery_key(map_gate_recovery(gate.recovery()))
            );
        }
        Some(SourceEvidenceResponsePayload::TypeFact(fact)) => {
            let _ = write!(output, "type-fact#{}", fact.index());
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        binding_env::{
            BinderIdentity, BindingContextDraft, BindingContextId, BindingContextLayer,
            BindingContextOwner, BindingContextRecovery, BindingContextTable,
            BindingDiagnosticTable, BindingDraft, BindingEnv, BindingEnvParts, BindingId,
            BindingKind, BindingRecoveryState, BindingStatus, BindingTable, BindingTypeSite,
            CapturedFreeVariables,
        },
        registration_resolution::ExistentialGateGuardEvidence,
        source_attribute::{
            SourceAttributeChainInput, SourceAttributeHandoffInput, SourceAttributeInput,
            SourceAttributePolarityInput, SourceAttributeProducer,
        },
        source_type::{
            SourceTypeApplicationForm, SourceTypeApplicationInput, SourceTypeExpressionInput,
            SourceTypeHandoffInput, SourceTypeProducer,
        },
        type_checker::{ModeExpansion, TypeExpressionInput, TypeHeadInput},
        typed_ast::{
            CoercionTable, FactProvenance, FactStatus, InitialObligationTable,
            LocalTypeContextTable, Polarity, TypeDiagnosticTable, TypeFactDraft, TypePredicateRef,
            TypeRole, TypeRuleId, TypeTable, TypedArena, TypedAst, TypedAstParts, TypedNode,
            TypedNodeId,
        },
    };
    use mizar_resolve::{
        env::{ContributionKind, NamespacePath, SymbolEntry, SymbolEnvIndexes},
        resolved_ast::{FullyQualifiedName, LocalSymbolId, SemanticOrigin, SymbolId},
    };
    use mizar_session::{
        BuildSnapshotId, InMemorySessionIdAllocator, ModulePath, PackageId,
        SessionIdAllocator as _, SourceAnchor,
    };

    struct Fixture {
        source: SourceId,
        module: ModuleId,
        symbols: SymbolEnv,
        bindings: BindingEnv,
        source_type: SourceTypeApplicationHandoff,
        arena: TypedArena,
        owner: TypedSiteRef,
        site: TypedSiteRef,
        source_range: SourceRange,
        facts: TypeFactTable,
        fact: TypeFactId,
        kind: SourceEvidenceRequestKind,
        application: SourceTypeApplicationId,
        expression: SourceTypeExpressionId,
        source_ordinal: usize,
        attribute_symbol: SymbolId,
        attribute_contribution: mizar_resolve::env::SourceContributionId,
    }

    fn source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "e5".repeat(32)
        ))
        .expect("snapshot");
        InMemorySessionIdAllocator::new()
            .next_source_id(snapshot)
            .expect("source")
    }

    fn other_source_id() -> SourceId {
        let snapshot = BuildSnapshotId::from_published_schema_str(&format!(
            "mizar-session-build-snapshot-v1:{}",
            "e5".repeat(32)
        ))
        .expect("snapshot");
        let allocator = InMemorySessionIdAllocator::new();
        allocator.next_source_id(snapshot).expect("first source");
        allocator.next_source_id(snapshot).expect("second source")
    }

    fn module() -> ModuleId {
        ModuleId::new(PackageId::new("pkg"), ModulePath::new("source.evidence"))
    }

    fn range(source: SourceId, start: usize, end: usize) -> SourceRange {
        SourceRange {
            source_id: source,
            start,
            end,
        }
    }

    fn role(node: usize, value: &str) -> TypedSiteRef {
        TypedSiteRef::Role {
            node: TypedNodeId::new(node),
            role: TypeRole::new(value),
        }
    }

    fn fixture(symbol_kind: Option<SymbolKind>) -> Fixture {
        fixture_with_leading_builtin(symbol_kind, false)
    }

    fn fixture_with_leading_builtin(
        symbol_kind: Option<SymbolKind>,
        leading_builtin: bool,
    ) -> Fixture {
        let source = source_id();
        let module = module();
        let source_ordinal = usize::from(leading_builtin);
        let application = SourceTypeApplicationId::new(source_ordinal);
        let expression = SourceTypeExpressionId::new(source_ordinal);
        let range_start = 10 + source_ordinal * 20;
        let source_range = range(source, range_start, range_start + 10);
        let owner = role(source_ordinal, "expression");
        let site = role(source_ordinal, "head");
        let arena_nodes = (0..=source_ordinal)
            .map(|index| {
                TypedNode::new(
                    "source-evidence-expression",
                    SourceAnchor::Range(range(source, 10 + index * 20, 20 + index * 20)),
                )
            })
            .collect();
        let arena = TypedArena::try_new(Some(TypedNodeId::new(source_ordinal)), arena_nodes)
            .expect("arena");

        let mut indexes = SymbolEnvIndexes::default();
        let (head, head_spelling, kind) = if let Some(symbol_kind) = symbol_kind {
            let contribution = indexes.contributions.insert(
                module.clone(),
                ContributionKind::LocalSource { source_id: source },
                SourceAnchor::Range(range(source, 1, 5)),
            );
            let symbol = SymbolId::new(
                module.clone(),
                LocalSymbolId::new("head"),
                FullyQualifiedName::new(format!("{}::Head", module.path().as_str())),
            );
            indexes.symbols.insert(SymbolEntry::new(
                symbol.clone(),
                symbol_kind,
                NamespacePath::new(module.path().as_str()),
                "Head",
                SemanticOrigin::new(
                    source,
                    module.clone(),
                    SourceAnchor::Range(range(source, 1, 5)),
                    vec![0],
                ),
                contribution,
            ));
            indexes
                .contributions
                .add_symbol(contribution, symbol.clone());
            let request_kind = match symbol_kind {
                SymbolKind::Mode => SourceEvidenceRequestKind::ModeExpansion,
                SymbolKind::Structure => SourceEvidenceRequestKind::StructureInhabitation,
                _ => panic!("fixture only admits a source type symbol"),
            };
            (
                SourceTypeHead::Symbol {
                    symbol,
                    contribution,
                },
                "Head",
                request_kind,
            )
        } else {
            (
                SourceTypeHead::BuiltinSet,
                "set",
                SourceEvidenceRequestKind::ModeExpansion,
            )
        };
        let attribute_contribution = indexes.contributions.insert(
            module.clone(),
            ContributionKind::LocalSource { source_id: source },
            SourceAnchor::Range(range(source, 1, 5)),
        );
        let attribute_symbol = SymbolId::new(
            module.clone(),
            LocalSymbolId::new("attribute"),
            FullyQualifiedName::new(format!("{}::Attribute", module.path().as_str())),
        );
        indexes.symbols.insert(SymbolEntry::new(
            attribute_symbol.clone(),
            SymbolKind::Attribute,
            NamespacePath::new(module.path().as_str()),
            head_spelling,
            SemanticOrigin::new(
                source,
                module.clone(),
                SourceAnchor::Range(range(source, 1, 5)),
                vec![1],
            ),
            attribute_contribution,
        ));
        indexes
            .contributions
            .add_symbol(attribute_contribution, attribute_symbol.clone());
        let symbols = SymbolEnv::new(module.clone(), indexes);

        let mut contexts = BindingContextTable::new();
        let binding_ids = (0..=source_ordinal).map(BindingId::new).collect::<Vec<_>>();
        contexts.insert(BindingContextDraft {
            owner: BindingContextOwner::Module,
            parent: None,
            layer: BindingContextLayer::Module,
            lexical_scope: None,
            bindings: binding_ids.clone(),
            visible_bindings: binding_ids,
            recovery: BindingContextRecovery::Normal,
        });
        let mut bindings = BindingTable::new();
        for index in 0..=source_ordinal {
            let declaration_range = range(source, index * 2, index * 2 + 1);
            let spelling = format!("x{index}");
            bindings.insert(BindingDraft {
                spelling: spelling.clone(),
                kind: BindingKind::ReservedVariable,
                identity: BinderIdentity::ReservedVariable {
                    spelling,
                    declaration_range,
                },
                owner_context: BindingContextId::new(0),
                declaration_range,
                visible_after_ordinal: index,
                type_site: BindingTypeSite::Source(range(source, 10 + index * 20, 20 + index * 20)),
                status: BindingStatus::Reserved,
                captured: CapturedFreeVariables::default(),
                diagnostics: Vec::new(),
                recovery: BindingRecoveryState::Normal,
            });
        }
        let bindings = BindingEnv::try_new(BindingEnvParts {
            source_id: source,
            module_id: module.clone(),
            contexts,
            bindings,
            diagnostics: BindingDiagnosticTable::new(),
        })
        .expect("bindings");

        let mut applications = Vec::new();
        let mut expressions = Vec::new();
        if leading_builtin {
            applications.push(SourceTypeApplicationInput {
                binding: BindingId::new(0),
                source_ordinal: 0,
                root: SourceTypeExpressionId::new(0),
            });
            expressions.push(SourceTypeExpressionInput {
                source_id: source,
                module_id: module.clone(),
                site: role(0, "expression"),
                source_range: range(source, 10, 20),
                spelling: head_spelling.to_owned(),
                head_site: role(0, "head"),
                head_range: range(source, 10, 10 + head_spelling.len()),
                head_spelling: head_spelling.to_owned(),
                form: SourceTypeApplicationForm::Bare,
                head: head.clone(),
                recovery: NodeRecoveryState::Normal,
            });
        }
        applications.push(SourceTypeApplicationInput {
            binding: BindingId::new(source_ordinal),
            source_ordinal,
            root: expression,
        });
        expressions.push(SourceTypeExpressionInput {
            source_id: source,
            module_id: module.clone(),
            site: owner.clone(),
            source_range,
            spelling: head_spelling.to_owned(),
            head_site: site.clone(),
            head_range: range(source, range_start, range_start + head_spelling.len()),
            head_spelling: head_spelling.to_owned(),
            form: SourceTypeApplicationForm::Bare,
            head,
            recovery: NodeRecoveryState::Normal,
        });
        let source_type = SourceTypeProducer::build(
            SourceTypeHandoffInput {
                source_id: source,
                module_id: module.clone(),
                applications,
                expressions,
                arguments: Vec::new(),
            },
            &bindings,
            &symbols,
            &arena,
        )
        .expect("source type");

        let mut facts = TypeFactTable::new();
        let fact = facts.insert(TypeFactDraft {
            subject: owner.clone(),
            predicate: TypePredicateRef::new("source-evidence-fact"),
            polarity: Polarity::Positive,
            provenance: FactProvenance::Inferred(TypeRuleId::new("source-evidence-test")),
            status: FactStatus::Known,
        });
        Fixture {
            source,
            module,
            symbols,
            bindings,
            source_type,
            arena,
            owner,
            site,
            source_range,
            facts,
            fact,
            kind,
            application,
            expression,
            source_ordinal,
            attribute_symbol,
            attribute_contribution,
        }
    }

    fn source_attribute(fixture: &Fixture) -> SourceAttributeHandoff {
        let spelling = fixture
            .source_type
            .expressions()
            .get(fixture.expression)
            .expect("expression")
            .spelling()
            .to_owned();
        SourceAttributeProducer::build(
            SourceAttributeHandoffInput {
                source_id: fixture.source,
                module_id: fixture.module.clone(),
                chains: vec![SourceAttributeChainInput {
                    expression: fixture.expression,
                    source_ordinal: 0,
                    site: fixture.owner.clone(),
                    source_range: fixture.source_range,
                    spelling: spelling.clone(),
                    recovery: NodeRecoveryState::Normal,
                }],
                attributes: vec![SourceAttributeInput {
                    chain: SourceAttributeChainId::new(0),
                    ordinal: 0,
                    site: role(fixture.source_ordinal, "attribute"),
                    source_range: fixture.source_range,
                    spelling: spelling.clone(),
                    target_site: role(fixture.source_ordinal, "attribute-target"),
                    target_range: fixture.source_range,
                    target_spelling: spelling,
                    recovery: NodeRecoveryState::Normal,
                    symbol: fixture.attribute_symbol.clone(),
                    contribution: fixture.attribute_contribution,
                    polarity: SourceAttributePolarityInput::Positive,
                }],
                qualifiers: Vec::new(),
                argument_groups: Vec::new(),
                arguments: Vec::new(),
            },
            &fixture.source_type,
            &fixture.bindings,
            &fixture.symbols,
            &fixture.arena,
        )
        .expect("source attribute")
    }

    fn replacement_symbols(
        fixture: &Fixture,
        kind: SymbolKind,
        wrong_contribution: bool,
    ) -> SymbolEnv {
        let SourceTypeHead::Symbol { symbol, .. } = fixture
            .source_type
            .expressions()
            .get(fixture.expression)
            .expect("expression")
            .head()
        else {
            panic!("symbol fixture");
        };
        let mut indexes = SymbolEnvIndexes::default();
        let expected = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 5)),
        );
        let alternate = indexes.contributions.insert(
            fixture.module.clone(),
            ContributionKind::LocalSource {
                source_id: fixture.source,
            },
            SourceAnchor::Range(range(fixture.source, 1, 5)),
        );
        let entry_contribution = if wrong_contribution {
            alternate
        } else {
            expected
        };
        indexes.symbols.insert(SymbolEntry::new(
            symbol.clone(),
            kind,
            NamespacePath::new(fixture.module.path().as_str()),
            "Head",
            SemanticOrigin::new(
                fixture.source,
                fixture.module.clone(),
                SourceAnchor::Range(range(fixture.source, 1, 5)),
                vec![0],
            ),
            entry_contribution,
        ));
        indexes
            .contributions
            .add_symbol(entry_contribution, symbol.clone());
        SymbolEnv::new(fixture.module.clone(), indexes)
    }

    fn attributed_request(
        fixture: &Fixture,
        state: SourceEvidenceInputState,
    ) -> SourceEvidenceRequestInput {
        SourceEvidenceRequestInput {
            owner: fixture.owner.clone(),
            site: fixture.owner.clone(),
            source_range: fixture.source_range,
            source_ordinal: fixture.source_ordinal,
            recovery: SourceEvidenceRecovery::Normal,
            kind: SourceEvidenceRequestKind::AttributedTypeInhabitation,
            state,
            origin: SourceEvidenceRequestOrigin::SourceTypeApplication {
                application: fixture.application,
                expression: fixture.expression,
                attribute_chain: Some(SourceAttributeChainId::new(0)),
            },
        }
    }

    fn request(fixture: &Fixture, state: SourceEvidenceInputState) -> SourceEvidenceRequestInput {
        SourceEvidenceRequestInput {
            owner: fixture.owner.clone(),
            site: fixture.site.clone(),
            source_range: fixture.source_range,
            source_ordinal: fixture.source_ordinal,
            recovery: SourceEvidenceRecovery::Normal,
            kind: fixture.kind,
            state,
            origin: SourceEvidenceRequestOrigin::SourceTypeApplication {
                application: fixture.application,
                expression: fixture.expression,
                attribute_chain: None,
            },
        }
    }

    fn input(
        fixture: &Fixture,
        state: SourceEvidenceInputState,
        responses: Vec<SourceEvidenceResponseInput>,
    ) -> SourceEvidenceHandoffInput {
        SourceEvidenceHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            requests: vec![request(fixture, state)],
            responses,
        }
    }

    fn record(
        key: &str,
        request: usize,
        disposition: SourceEvidenceResponseDisposition,
        payload: Option<SourceEvidenceResponsePayload>,
    ) -> SourceEvidenceDependencyRecord {
        SourceEvidenceDependencyRecord::new(
            key,
            SourceEvidenceRequestId::new(request),
            disposition,
            SourceEvidenceResponseProvenance::ExplicitInput,
            payload,
        )
    }

    fn response(request: usize, ordinal: usize, key: &str) -> SourceEvidenceResponseInput {
        SourceEvidenceResponseInput {
            request: SourceEvidenceRequestId::new(request),
            ordinal,
            key: SourceEvidenceResponseKey::new(key),
        }
    }

    fn build(
        fixture: &Fixture,
        input: SourceEvidenceHandoffInput,
        catalog: SourceEvidenceDependencyCatalog,
    ) -> Result<SourceEvidenceHandoff, SourceEvidenceError> {
        SourceEvidenceProducer::build(
            input,
            &fixture.source_type,
            None,
            &fixture.symbols,
            &fixture.facts,
            &catalog,
        )
    }

    #[test]
    fn all_transport_states_publish_dense_deterministic_rows() {
        let fixture = fixture(Some(SymbolKind::Mode));
        for state in [
            SourceEvidenceInputState::Requested,
            SourceEvidenceInputState::Missing,
        ] {
            let first = build(
                &fixture,
                input(&fixture, state, Vec::new()),
                SourceEvidenceDependencyCatalog::empty(),
            )
            .expect("empty-response state");
            let second = build(
                &fixture,
                input(&fixture, state, Vec::new()),
                SourceEvidenceDependencyCatalog::empty(),
            )
            .expect("deterministic state");
            assert_eq!(first, second);
            assert_eq!(first.debug_text(), second.debug_text());
            assert_eq!(first.requests().len(), 1);
            assert!(first.responses().is_empty());
        }

        let rejected = build(
            &fixture,
            input(
                &fixture,
                SourceEvidenceInputState::Rejected,
                vec![response(0, 0, "rejected")],
            ),
            SourceEvidenceDependencyCatalog::new(vec![record(
                "rejected",
                0,
                SourceEvidenceResponseDisposition::Rejected,
                None,
            )]),
        )
        .expect("rejected");
        assert_eq!(
            rejected
                .responses()
                .get(SourceEvidenceResponseId::new(0))
                .map(SourceEvidenceResponse::disposition),
            Some(SourceEvidenceResponseDisposition::Rejected)
        );

        let supplied = build(
            &fixture,
            input(
                &fixture,
                SourceEvidenceInputState::Supplied,
                vec![response(0, 0, "fact-a"), response(0, 1, "fact-b")],
            ),
            SourceEvidenceDependencyCatalog::new(vec![
                record(
                    "fact-a",
                    0,
                    SourceEvidenceResponseDisposition::Supplied,
                    Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                ),
                record(
                    "fact-b",
                    0,
                    SourceEvidenceResponseDisposition::Supplied,
                    Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                ),
            ]),
        )
        .expect("supplied");
        assert_eq!(supplied.responses().len(), 2);
        assert_eq!(
            supplied
                .responses()
                .iter()
                .map(|(id, row)| (id.index(), row.request().index(), row.ordinal()))
                .collect::<Vec<_>>(),
            vec![(0, 0, 0), (1, 0, 1)]
        );
        assert!(supplied.debug_text().contains("payload=type-fact#0"));
    }

    #[test]
    fn exact_request_association_and_kind_are_authenticated() {
        let mode = fixture(Some(SymbolKind::Mode));
        build(
            &mode,
            input(&mode, SourceEvidenceInputState::Missing, Vec::new()),
            SourceEvidenceDependencyCatalog::empty(),
        )
        .expect("mode");

        let structure = fixture(Some(SymbolKind::Structure));
        build(
            &structure,
            input(&structure, SourceEvidenceInputState::Missing, Vec::new()),
            SourceEvidenceDependencyCatalog::empty(),
        )
        .expect("structure");

        let mut wrong_kind = input(&mode, SourceEvidenceInputState::Missing, Vec::new());
        wrong_kind.requests[0].kind = SourceEvidenceRequestKind::StructureInhabitation;
        assert!(matches!(
            build(&mode, wrong_kind, SourceEvidenceDependencyCatalog::empty()),
            Err(SourceEvidenceError::WrongRequestKind { .. })
        ));

        for mutate in 0..6 {
            let mut corrupt = input(&mode, SourceEvidenceInputState::Missing, Vec::new());
            match mutate {
                0 => corrupt.requests[0].owner = mode.site.clone(),
                1 => corrupt.requests[0].site = mode.owner.clone(),
                2 => corrupt.requests[0].source_range.end += 1,
                3 => corrupt.requests[0].source_ordinal = 1,
                4 => corrupt.requests[0].recovery = SourceEvidenceRecovery::Degraded,
                5 => {
                    corrupt.requests[0].origin =
                        SourceEvidenceRequestOrigin::SourceTypeApplication {
                            application: SourceTypeApplicationId::new(0),
                            expression: SourceTypeExpressionId::new(99),
                            attribute_chain: None,
                        };
                }
                _ => unreachable!(),
            }
            assert!(matches!(
                build(&mode, corrupt, SourceEvidenceDependencyCatalog::empty()),
                Err(SourceEvidenceError::InvalidRequest { .. })
            ));
        }

        let mut wrong_source = input(&mode, SourceEvidenceInputState::Missing, Vec::new());
        wrong_source.source_id = other_source_id();
        assert!(matches!(
            build(
                &mode,
                wrong_source,
                SourceEvidenceDependencyCatalog::empty()
            ),
            Err(SourceEvidenceError::EnvironmentMismatch)
        ));
        let mut wrong_module = input(&mode, SourceEvidenceInputState::Missing, Vec::new());
        wrong_module.module_id =
            ModuleId::new(PackageId::new("pkg"), ModulePath::new("wrong.module"));
        assert!(matches!(
            build(
                &mode,
                wrong_module,
                SourceEvidenceDependencyCatalog::empty()
            ),
            Err(SourceEvidenceError::EnvironmentMismatch)
        ));

        for requests in [
            Vec::new(),
            vec![
                request(&mode, SourceEvidenceInputState::Missing),
                request(&mode, SourceEvidenceInputState::Missing),
            ],
        ] {
            let mut wrong_cardinality = input(&mode, SourceEvidenceInputState::Missing, Vec::new());
            wrong_cardinality.requests = requests;
            assert!(matches!(
                build(
                    &mode,
                    wrong_cardinality,
                    SourceEvidenceDependencyCatalog::empty()
                ),
                Err(SourceEvidenceError::RequestCardinalityMismatch)
            ));
        }

        let valid_input = input(&mode, SourceEvidenceInputState::Missing, Vec::new());
        for symbols in [
            SymbolEnv::new(mode.module.clone(), SymbolEnvIndexes::default()),
            replacement_symbols(&mode, SymbolKind::Mode, true),
            replacement_symbols(&mode, SymbolKind::Predicate, false),
        ] {
            assert!(matches!(
                SourceEvidenceProducer::build(
                    valid_input.clone(),
                    &mode.source_type,
                    None,
                    &symbols,
                    &mode.facts,
                    &SourceEvidenceDependencyCatalog::empty(),
                ),
                Err(SourceEvidenceError::InvalidSymbolHead { .. })
            ));
        }

        let builtin = fixture(None);
        assert!(matches!(
            build(
                &builtin,
                input(&builtin, SourceEvidenceInputState::Missing, Vec::new()),
                SourceEvidenceDependencyCatalog::empty()
            ),
            Err(SourceEvidenceError::RequestCardinalityMismatch)
        ));
    }

    #[test]
    fn catalog_parentage_uniqueness_and_full_consumption_are_atomic() {
        let fixture = fixture(Some(SymbolKind::Mode));
        let supplied = input(
            &fixture,
            SourceEvidenceInputState::Supplied,
            vec![response(0, 0, "key")],
        );

        assert!(matches!(
            build(
                &fixture,
                supplied.clone(),
                SourceEvidenceDependencyCatalog::empty()
            ),
            Err(SourceEvidenceError::MissingCatalogRecord { .. })
        ));
        assert!(matches!(
            build(
                &fixture,
                supplied.clone(),
                SourceEvidenceDependencyCatalog::new(vec![record(
                    "key",
                    1,
                    SourceEvidenceResponseDisposition::Supplied,
                    Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                )])
            ),
            Err(SourceEvidenceError::CrossRequestCatalogRecord { .. })
        ));
        assert!(matches!(
            build(
                &fixture,
                supplied.clone(),
                SourceEvidenceDependencyCatalog::new(vec![
                    record(
                        "key",
                        0,
                        SourceEvidenceResponseDisposition::Supplied,
                        Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                    ),
                    record(
                        "stale",
                        0,
                        SourceEvidenceResponseDisposition::Supplied,
                        Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                    ),
                ])
            ),
            Err(SourceEvidenceError::StaleCatalogRecord)
        ));
        assert!(matches!(
            build(
                &fixture,
                supplied,
                SourceEvidenceDependencyCatalog::new(vec![
                    record(
                        "key",
                        0,
                        SourceEvidenceResponseDisposition::Supplied,
                        Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                    ),
                    record(
                        "key",
                        0,
                        SourceEvidenceResponseDisposition::Supplied,
                        Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                    ),
                ])
            ),
            Err(SourceEvidenceError::DuplicateCatalogKey)
        ));

        let duplicate_response = input(
            &fixture,
            SourceEvidenceInputState::Supplied,
            vec![response(0, 0, "key"), response(0, 1, "key")],
        );
        assert!(matches!(
            build(
                &fixture,
                duplicate_response,
                SourceEvidenceDependencyCatalog::new(vec![record(
                    "key",
                    0,
                    SourceEvidenceResponseDisposition::Supplied,
                    Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                )])
            ),
            Err(SourceEvidenceError::DuplicateResponseKey { .. })
        ));
    }

    #[test]
    fn state_disposition_payload_and_response_cardinality_must_agree() {
        let fixture = fixture(Some(SymbolKind::Mode));
        for state in [
            SourceEvidenceInputState::Requested,
            SourceEvidenceInputState::Missing,
        ] {
            assert!(
                build(
                    &fixture,
                    input(&fixture, state, vec![response(0, 0, "unexpected")]),
                    SourceEvidenceDependencyCatalog::new(vec![record(
                        "unexpected",
                        0,
                        SourceEvidenceResponseDisposition::Rejected,
                        None,
                    )]),
                )
                .is_err()
            );
        }

        assert!(matches!(
            build(
                &fixture,
                input(
                    &fixture,
                    SourceEvidenceInputState::Rejected,
                    vec![response(0, 0, "a"), response(0, 1, "b")],
                ),
                SourceEvidenceDependencyCatalog::new(vec![
                    record("a", 0, SourceEvidenceResponseDisposition::Rejected, None,),
                    record("b", 0, SourceEvidenceResponseDisposition::Rejected, None,),
                ]),
            ),
            Err(SourceEvidenceError::InvalidStateCardinality { .. })
        ));
        assert!(matches!(
            build(
                &fixture,
                input(&fixture, SourceEvidenceInputState::Supplied, Vec::new()),
                SourceEvidenceDependencyCatalog::empty(),
            ),
            Err(SourceEvidenceError::InvalidStateCardinality { .. })
        ));

        let cases = [
            (
                SourceEvidenceInputState::Rejected,
                SourceEvidenceResponseDisposition::Supplied,
                Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
            ),
            (
                SourceEvidenceInputState::Supplied,
                SourceEvidenceResponseDisposition::Rejected,
                None,
            ),
            (
                SourceEvidenceInputState::Rejected,
                SourceEvidenceResponseDisposition::Rejected,
                Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
            ),
            (
                SourceEvidenceInputState::Supplied,
                SourceEvidenceResponseDisposition::Supplied,
                None,
            ),
        ];
        for (state, disposition, payload) in cases {
            assert!(matches!(
                build(
                    &fixture,
                    input(&fixture, state, vec![response(0, 0, "mismatch")]),
                    SourceEvidenceDependencyCatalog::new(vec![record(
                        "mismatch",
                        0,
                        disposition,
                        payload,
                    )]),
                ),
                Err(SourceEvidenceError::InvalidCatalogRecord)
            ));
        }
    }

    #[test]
    fn cardinality_order_fact_and_gate_corruption_fail_closed() {
        let fixture = fixture(Some(SymbolKind::Mode));
        assert!(matches!(
            build(
                &fixture,
                input(&fixture, SourceEvidenceInputState::Rejected, Vec::new()),
                SourceEvidenceDependencyCatalog::empty()
            ),
            Err(SourceEvidenceError::InvalidStateCardinality { .. })
        ));

        let reordered = input(
            &fixture,
            SourceEvidenceInputState::Supplied,
            vec![response(0, 1, "key")],
        );
        assert!(matches!(
            build(
                &fixture,
                reordered,
                SourceEvidenceDependencyCatalog::new(vec![record(
                    "key",
                    0,
                    SourceEvidenceResponseDisposition::Supplied,
                    Some(SourceEvidenceResponsePayload::TypeFact(fixture.fact)),
                )])
            ),
            Err(SourceEvidenceError::InvalidResponse { .. })
        ));

        let dangling_fact = TypeFactId::new(99);
        assert!(matches!(
            build(
                &fixture,
                input(
                    &fixture,
                    SourceEvidenceInputState::Supplied,
                    vec![response(0, 0, "fact")],
                ),
                SourceEvidenceDependencyCatalog::new(vec![record(
                    "fact",
                    0,
                    SourceEvidenceResponseDisposition::Supplied,
                    Some(SourceEvidenceResponsePayload::TypeFact(dangling_fact)),
                )])
            ),
            Err(SourceEvidenceError::InvalidTypeFact { fact }) if fact == dangling_fact
        ));

        let gate = ExistentialGateInput::new(
            fixture.owner.clone(),
            fixture.source_range,
            "pattern",
            "trigger",
            Vec::new(),
        );
        assert!(matches!(
            build(
                &fixture,
                input(
                    &fixture,
                    SourceEvidenceInputState::Supplied,
                    vec![response(0, 0, "gate")],
                ),
                SourceEvidenceDependencyCatalog::new(vec![record(
                    "gate",
                    0,
                    SourceEvidenceResponseDisposition::Supplied,
                    Some(SourceEvidenceResponsePayload::ExistentialGate(gate)),
                )])
            ),
            Err(SourceEvidenceError::InvalidPayloadKind { .. })
        ));

        let attributed = SourceEvidenceRequestInput {
            kind: SourceEvidenceRequestKind::AttributedTypeInhabitation,
            ..request(&fixture, SourceEvidenceInputState::Supplied)
        };
        let wrong_gate = ExistentialGateInput::new(
            fixture.site.clone(),
            fixture.source_range,
            "pattern",
            "trigger",
            Vec::new(),
        );
        assert!(matches!(
            validate_payload(
                SourceEvidenceRequestId::new(0),
                &attributed,
                &SourceEvidenceResponsePayload::ExistentialGate(wrong_gate),
                &fixture.facts,
            ),
            Err(SourceEvidenceError::InvalidExistentialGate { .. })
        ));

        let wrong_range = ExistentialGateInput::new(
            fixture.owner.clone(),
            range(
                fixture.source,
                fixture.source_range.start,
                fixture.source_range.end - 1,
            ),
            "pattern",
            "trigger",
            Vec::new(),
        );
        assert!(matches!(
            validate_payload(
                SourceEvidenceRequestId::new(0),
                &attributed,
                &SourceEvidenceResponsePayload::ExistentialGate(wrong_range),
                &fixture.facts,
            ),
            Err(SourceEvidenceError::InvalidExistentialGate { .. })
        ));

        let wrong_recovery = ExistentialGateInput::new(
            fixture.owner.clone(),
            fixture.source_range,
            "pattern",
            "trigger",
            Vec::new(),
        )
        .with_recovery(ExistentialGateRecovery::Degraded);
        assert!(matches!(
            validate_payload(
                SourceEvidenceRequestId::new(0),
                &attributed,
                &SourceEvidenceResponsePayload::ExistentialGate(wrong_recovery),
                &fixture.facts,
            ),
            Err(SourceEvidenceError::InvalidExistentialGate { .. })
        ));

        let dangling_guard = TypeFactId::new(99);
        let gate_with_dangling_guard = ExistentialGateInput::new(
            fixture.owner.clone(),
            fixture.source_range,
            "pattern",
            "trigger",
            Vec::new(),
        )
        .with_guard_evidence([ExistentialGateGuardEvidence::new("guard", dangling_guard)]);
        assert!(matches!(
            validate_payload(
                SourceEvidenceRequestId::new(0),
                &attributed,
                &SourceEvidenceResponsePayload::ExistentialGate(gate_with_dangling_guard),
                &fixture.facts,
            ),
            Err(SourceEvidenceError::InvalidTypeFact { fact }) if fact == dangling_guard
        ));
    }

    #[test]
    fn attributed_chain_uses_application_ordinal_and_keeps_dense_request_order() {
        let fixture = fixture_with_leading_builtin(Some(SymbolKind::Mode), true);
        let source_attribute = source_attribute(&fixture);
        let leading_request = SourceEvidenceRequestInput {
            owner: role(0, "expression"),
            site: role(0, "head"),
            source_range: range(fixture.source, 10, 20),
            source_ordinal: 0,
            recovery: SourceEvidenceRecovery::Normal,
            kind: SourceEvidenceRequestKind::ModeExpansion,
            state: SourceEvidenceInputState::Missing,
            origin: SourceEvidenceRequestOrigin::SourceTypeApplication {
                application: SourceTypeApplicationId::new(0),
                expression: SourceTypeExpressionId::new(0),
                attribute_chain: None,
            },
        };
        let missing_input = SourceEvidenceHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            requests: vec![
                leading_request.clone(),
                attributed_request(&fixture, SourceEvidenceInputState::Missing),
            ],
            responses: Vec::new(),
        };
        let missing = SourceEvidenceProducer::build(
            missing_input,
            &fixture.source_type,
            Some(&source_attribute),
            &fixture.symbols,
            &fixture.facts,
            &SourceEvidenceDependencyCatalog::empty(),
        )
        .expect("attributed builtin request");
        assert_eq!(
            missing
                .requests()
                .iter()
                .map(|(id, request)| (id.index(), request.source_ordinal()))
                .collect::<Vec<_>>(),
            vec![(0, 0), (1, 1)]
        );
        assert_eq!(
            source_attribute
                .chains()
                .get(SourceAttributeChainId::new(0))
                .map(|chain| chain.source_ordinal()),
            Some(0)
        );

        let gate = ExistentialGateInput::new(
            fixture.owner.clone(),
            fixture.source_range,
            "pattern",
            "trigger",
            Vec::new(),
        );
        let supplied_input = SourceEvidenceHandoffInput {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            requests: vec![
                leading_request,
                attributed_request(&fixture, SourceEvidenceInputState::Supplied),
            ],
            responses: vec![response(1, 0, "gate")],
        };
        let handoff = SourceEvidenceProducer::build(
            supplied_input,
            &fixture.source_type,
            Some(&source_attribute),
            &fixture.symbols,
            &fixture.facts,
            &SourceEvidenceDependencyCatalog::new(vec![record(
                "gate",
                1,
                SourceEvidenceResponseDisposition::Supplied,
                Some(SourceEvidenceResponsePayload::ExistentialGate(gate)),
            )]),
        )
        .expect("attributed gate");
        assert_eq!(
            handoff
                .requests()
                .get(SourceEvidenceRequestId::new(1))
                .map(SourceEvidenceRequest::kind),
            Some(SourceEvidenceRequestKind::AttributedTypeInhabitation)
        );

        let builtin = self::fixture(None);
        let builtin_attribute = self::source_attribute(&builtin);
        SourceEvidenceProducer::build(
            SourceEvidenceHandoffInput {
                source_id: builtin.source,
                module_id: builtin.module.clone(),
                requests: vec![attributed_request(
                    &builtin,
                    SourceEvidenceInputState::Missing,
                )],
                responses: Vec::new(),
            },
            &builtin.source_type,
            Some(&builtin_attribute),
            &builtin.symbols,
            &builtin.facts,
            &SourceEvidenceDependencyCatalog::empty(),
        )
        .expect("attributed builtin request");
    }

    #[test]
    fn mode_payload_and_typed_ast_installation_preserve_the_handoff() {
        let fixture = fixture(Some(SymbolKind::Mode));
        let expansion = ModeExpansion::new(
            TypeExpressionInput::new(
                fixture.owner.clone(),
                fixture.source_range,
                "set",
                TypeHeadInput::BuiltinSet,
            ),
            Vec::new(),
        );
        let handoff = build(
            &fixture,
            input(
                &fixture,
                SourceEvidenceInputState::Supplied,
                vec![response(0, 0, "mode")],
            ),
            SourceEvidenceDependencyCatalog::new(vec![record(
                "mode",
                0,
                SourceEvidenceResponseDisposition::Supplied,
                Some(SourceEvidenceResponsePayload::ModeExpansion(expansion)),
            )]),
        )
        .expect("mode payload");

        let typed = TypedAst::try_new(TypedAstParts {
            source_id: fixture.source,
            module_id: fixture.module.clone(),
            resolved_root: None,
            source_context: None,
            source_type: Some(fixture.source_type.clone()),
            source_attribute: None,
            nodes: fixture.arena.clone(),
            contexts: LocalTypeContextTable::new(),
            types: TypeTable::new(),
            facts: fixture.facts.clone(),
            coercions: CoercionTable::new(),
            initial_obligations: InitialObligationTable::new(),
            diagnostics: TypeDiagnosticTable::new(),
        })
        .expect("typed AST")
        .with_source_evidence(handoff.clone())
        .expect("source evidence installation");
        assert_eq!(typed.source_evidence(), Some(&handoff));
        assert!(typed.debug_text().contains("source-evidence-debug-v1\n"));
        assert!(matches!(
            typed.clone().with_source_evidence(handoff),
            Err(crate::typed_ast::TypedAstError::InvalidSourceEvidence)
        ));
    }
}
