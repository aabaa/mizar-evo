//! On-demand resolution of handles from one retained diagnostic publication.

use mizar_diagnostics::{
    aggregator::BuildDiagnosticIndex,
    explain::{ExplanationResolution, ExplanationStore, ExplanationSubject},
    failure_record::{DiagnosticFreshness, DiagnosticPrimaryLocation, SpanFreshness},
};
use mizar_driver::{driver::CompilerDriver, request::CapturedSnapshot};
use mizar_session::{
    BuildSnapshotId, GeneratedSpanAnchor, SessionIdAllocator, SnapshotRegistry, SourceAnchor,
    SourceOriginInput, SourceRange,
};
use serde_json::{Value, json};

use crate::{Server, server::LspSnapshotFreshness};

pub(crate) fn valid_explanation_params(params: &Value) -> bool {
    params.as_object().is_some_and(|params| {
        params.len() == 2
            && params
                .get("snapshot")
                .and_then(Value::as_str)
                .is_some_and(|id| BuildSnapshotId::from_published_schema_str(id).is_ok())
            && (params.get("diagnosticId").is_some_and(Value::is_u64)
                || ["exprId", "vcId"].iter().any(|key| {
                    params
                        .get(*key)
                        .and_then(Value::as_str)
                        .is_some_and(|id| !id.is_empty())
                }))
    })
}

pub(crate) fn unavailable(reason: &str) -> Value {
    json!({"status": "unavailable", "freshness": "unavailable", "snapshot": null,
        "locations": [], "preview": null, "payload": null, "reason": reason})
}

fn stale_handle() -> Value {
    let mut result = unavailable("stale_handle");
    result["status"] = json!("stale_handle");
    result
}

impl Server {
    /// Validates normal RPC/lifecycle rules and resolves only retained published handles.
    pub fn handle_message_with_explanations<A: SessionIdAllocator>(
        &mut self,
        message: Value,
        driver: &CompilerDriver,
        snapshots: &SnapshotRegistry<A>,
        store: &ExplanationStore,
        cancelled: &dyn Fn() -> bool,
    ) -> Option<Value> {
        if message.get("method").and_then(Value::as_str) != Some("mizar/explain") {
            return self.handle_message(message);
        }
        let mut response = self.handle_message(message.clone())?;
        if response.get("result").is_none() {
            return Some(response);
        }
        let result = if cancelled() {
            None
        } else {
            let selected = self.diagnostic_host(driver, snapshots);
            let result = selected.as_ref().map_or_else(
                || Ok(unavailable("no_snapshot")),
                |(selected, _)| {
                    convert_explanation(
                        selected.diagnostics(),
                        selected.native_open_buffers().captured(),
                        store,
                        &message["params"],
                    )
                },
            );
            if cancelled() {
                None
            } else {
                Some(result.map(|mut result| {
                    let Some((selected, freshness)) = selected else {
                        return result;
                    };
                    let Some((latest, latest_freshness)) = self.diagnostic_host(driver, snapshots)
                    else {
                        return stale_handle();
                    };
                    if !selected.same_publication(&latest) {
                        return stale_handle();
                    }
                    if result["freshness"] == "current"
                        && (freshness == LspSnapshotFreshness::StaleArtifact
                            || latest_freshness == LspSnapshotFreshness::StaleArtifact)
                    {
                        result["freshness"] = json!("stale_artifact");
                    }
                    result
                }))
            }
        };
        match result {
            Some(Ok(result)) => response["result"] = result,
            result => {
                response.as_object_mut()?.remove("result");
                response["error"] = match result {
                    None => json!({"code": -32800, "message": "Request cancelled"}),
                    Some(Err(error)) => json!({"code": -32603, "message": error}),
                    Some(Ok(_)) => unreachable!(),
                };
            }
        }
        Some(response)
    }
}

pub(crate) fn convert_explanation(
    index: &BuildDiagnosticIndex,
    captured: &CapturedSnapshot,
    store: &ExplanationStore,
    params: &Value,
) -> Result<Value, String> {
    if !valid_explanation_params(params) {
        return Err("invalid explanation params".into());
    }
    if index.snapshot() != captured.snapshot().id {
        return Err("explanation index snapshot mismatch".into());
    }
    let snapshot = index
        .snapshot()
        .to_published_schema_string()
        .map_err(|error| format!("invalid explanation snapshot: {error:?}"))?;
    if params["snapshot"] != snapshot {
        return Ok(stale_handle());
    }
    let mut result = unavailable("target_unavailable");
    result["snapshot"] = json!(snapshot);
    result["freshness"] = json!("current");
    let mut matches = index.records().iter().filter(|record| {
        if let Some(id) = params["diagnosticId"].as_u64() {
            return record.handle().id.get() == id;
        }
        record
            .explanation()
            .is_some_and(|handle| match handle.subject() {
                ExplanationSubject::Expression(id) => params["exprId"].as_str() == Some(id),
                ExplanationSubject::VerificationCondition(id) => {
                    params["vcId"].as_str() == Some(id)
                }
                _ => false,
            })
    });
    let Some(record) = matches.next() else {
        return Ok(result);
    };
    if matches.next().is_some() {
        return Ok(result);
    }
    let Some(handle) = record.explanation() else {
        return Ok(result);
    };
    if record.handle().snapshot != index.snapshot()
        || record.freshness()
            != &(DiagnosticFreshness::Current {
                source_snapshot: index.snapshot(),
            })
    {
        return Err("explanation diagnostic is not current for its index".into());
    }
    let resolution = store.resolve(handle, Some(index.snapshot()));
    if matches!(resolution, ExplanationResolution::Stale { .. }) {
        return Ok(stale_handle());
    }
    let DiagnosticPrimaryLocation::Span(span) = record.primary_location() else {
        return Err("unsupported explanation primary location".into());
    };
    if span.freshness() != SpanFreshness::Current {
        return Err("explanation span is not current".into());
    }
    let range = match span.anchor() {
        SourceAnchor::Range(range) => *range,
        SourceAnchor::Point { source_id, offset } => SourceRange {
            source_id: *source_id,
            start: *offset,
            end: *offset,
        },
        SourceAnchor::Generated(origin) => match origin.anchor() {
            GeneratedSpanAnchor::Range(range) => range,
            GeneratedSpanAnchor::Point { source_id, offset } => SourceRange {
                source_id,
                start: offset,
                end: offset,
            },
            _ => return Err("unsupported generated explanation anchor".into()),
        },
        _ => return Err("unsupported explanation anchor".into()),
    };
    let sources = captured
        .native_sources()
        .ok_or("missing explanation native sources")?;
    let mut matching = sources
        .iter()
        .filter(|(_, loaded, _)| loaded.source_id == range.source_id);
    let (_, loaded, input) = matching.next().ok_or("unknown explanation source")?;
    if matching.next().is_some() {
        return Err("ambiguous explanation source".into());
    }
    let SourceOriginInput::OpenBuffer {
        uri,
        actual_version,
        ..
    } = &input.origin
    else {
        return Err("explanation source is not an OpenBuffer".into());
    };
    if sources
        .iter()
        .filter(|(_, _, input)| {
            matches!(
                &input.origin, SourceOriginInput::OpenBuffer { uri: other, .. } if other == uri
            )
        })
        .count()
        != 1
    {
        return Err("ambiguous explanation URI".into());
    }
    let version = i32::try_from(*actual_version).map_err(|_| "explanation version overflow")?;
    let range = crate::lsp_range_from_open_buffer(
        &loaded.line_map,
        loaded.loading_map.as_ref(),
        &input.origin,
        range,
    )
    .map_err(|error| format!("invalid explanation source map: {error:?}"))?;
    result["locations"] = json!([{"uri": uri, "version": version, "range": {
        "start": {"line": range.start.line, "character": range.start.character},
        "end": {"line": range.end.line, "character": range.end.character}}}]);
    result["preview"] = handle.preview().map_or(Value::Null, |preview| json!({
        "format": preview.format().as_str(), "text": preview.text(), "truncated": preview.truncated()
    }));
    match resolution {
        ExplanationResolution::Available(payload) => {
            result["status"] = json!("available");
            result
                .as_object_mut()
                .expect("explanation object")
                .remove("reason");
            result["payload"] = json!({"format": payload.format().as_str(), "text": payload.text(),
            "truncated": payload.truncated(), "summaryHash": payload.summary_hash().map(|hash| {
                hash.as_bytes().iter().map(|byte| format!("{byte:02x}")).collect::<String>()
            })});
        }
        ExplanationResolution::Missing { reason } => result["reason"] = json!(reason.as_str()),
        ExplanationResolution::Unavailable { reason } => result["reason"] = json!(reason),
        _ => return Err("unsupported explanation resolution".into()),
    }
    Ok(result)
}
