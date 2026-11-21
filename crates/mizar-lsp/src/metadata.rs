//! Hover projection from published expressions and their retained original source maps.

use std::sync::Arc;

use mizar_artifact::verified_artifact::VerifiedArtifact;
use mizar_driver::{driver::CompilerDriver, request::CapturedSnapshot};
use mizar_session::{SessionIdAllocator, SnapshotRegistry, SourceOriginInput, SourceRange};
use serde_json::{Value, json};

use super::{LspSnapshotFreshness, Server};
use crate::{LspPosition, lsp_range_from_open_buffer};

impl Server {
    pub(crate) fn snapshot_definition<A: SessionIdAllocator>(
        &self,
        driver: &CompilerDriver,
        snapshots: &SnapshotRegistry<A>,
        uri: &str,
        position: LspPosition,
    ) -> Option<Value> {
        let (snapshot, freshness) = self.lsp_snapshot(driver, snapshots)?;
        if freshness != LspSnapshotFreshness::Current {
            return None;
        }
        let location = artifact_definition(
            snapshot.publication(),
            snapshot.native_open_buffers().captured(),
            uri,
            position,
        )?;
        self.lsp_snapshot(driver, snapshots)
            .is_some_and(|(now, state)| {
                Arc::ptr_eq(&snapshot, &now) && state == LspSnapshotFreshness::Current
            })
            .then_some(location)
    }

    /// Reads hover metadata at coordinates of the selected publication's original source.
    /// Stale content names that source and omits a current-buffer LSP range.
    pub fn snapshot_hover<A: SessionIdAllocator>(
        &self,
        driver: &CompilerDriver,
        snapshots: &SnapshotRegistry<A>,
        uri: &str,
        position: LspPosition,
    ) -> Result<Option<Value>, String> {
        if self.initialized.is_none() || self.shutdown || self.exit_code.is_some() {
            return Ok(None);
        }
        let Some((snapshot, freshness)) = self.lsp_snapshot(driver, snapshots) else {
            return Ok(None);
        };
        let Some(mut hover) = artifact_hover(
            snapshot.publication().artifacts(),
            snapshot.native_open_buffers().captured(),
            uri,
            position,
        )?
        else {
            return Ok(None);
        };
        let current = freshness == LspSnapshotFreshness::Current
            && self
                .lsp_snapshot(driver, snapshots)
                .is_some_and(|(now, state)| {
                    Arc::ptr_eq(&snapshot, &now) && state == LspSnapshotFreshness::Current
                });
        if !current {
            hover.as_object_mut().expect("hover object").remove("range");
            let text = hover["contents"]["value"].as_str().expect("hover text");
            hover["contents"]["value"] = json!(format!(
                "stale_artifact: position and range refer to retained source\n{text}"
            ));
        }
        Ok(Some(hover))
    }

    /// Applies normal JSON-RPC/lifecycle validation, then serves published hover metadata.
    pub fn handle_message_with_metadata<A: SessionIdAllocator>(
        &mut self,
        message: Value,
        driver: &CompilerDriver,
        snapshots: &SnapshotRegistry<A>,
    ) -> Option<Value> {
        let method = message.get("method").and_then(Value::as_str);
        if !matches!(
            method,
            Some(
                "textDocument/hover"
                    | "textDocument/definition"
                    | "textDocument/completion"
                    | "textDocument/codeAction"
                    | "codeAction/resolve"
            )
        ) {
            return self.handle_message(message);
        }
        let mut response = self.handle_message(message.clone())?;
        let resolve = method == Some("codeAction/resolve");
        if response.get("result").is_none() && !(resolve && response["error"]["code"] == -32801) {
            return Some(response);
        }
        if method == Some("textDocument/definition") {
            let params = &message["params"];
            response["result"] = self
                .snapshot_definition(
                    driver,
                    snapshots,
                    params["textDocument"]["uri"].as_str()?,
                    LspPosition {
                        line: u32::try_from(params["position"]["line"].as_u64()?).ok()?,
                        character: u32::try_from(params["position"]["character"].as_u64()?).ok()?,
                    },
                )
                .unwrap_or(Value::Null);
            return Some(response);
        }
        if method == Some("textDocument/completion") {
            let params = &message["params"];
            response["result"] = json!(self.snapshot_completion(
                driver,
                snapshots,
                params["textDocument"]["uri"].as_str()?,
                LspPosition {
                    line: u32::try_from(params["position"]["line"].as_u64()?).ok()?,
                    character: u32::try_from(params["position"]["character"].as_u64()?).ok()?,
                }
            ));
            return Some(response);
        }
        if method != Some("textDocument/hover") {
            match self.snapshot_code_actions(driver, snapshots, &message["params"], resolve) {
                Ok(Some(result)) => {
                    response.as_object_mut()?.remove("error");
                    response["result"] = result;
                }
                Ok(None) => {}
                Err(error) => {
                    response.as_object_mut()?.remove("result");
                    response["error"] = json!({"code": -32603, "message": error});
                }
            }
            return Some(response);
        }
        let params = &message["params"];
        let position = LspPosition {
            line: u32::try_from(params["position"]["line"].as_u64()?).ok()?,
            character: u32::try_from(params["position"]["character"].as_u64()?).ok()?,
        };
        match self.snapshot_hover(
            driver,
            snapshots,
            params["textDocument"]["uri"].as_str()?,
            position,
        ) {
            Ok(hover) => response["result"] = hover.unwrap_or(Value::Null),
            Err(error) => {
                response.as_object_mut()?.remove("result");
                response["error"] = json!({"code": -32603, "message": error});
            }
        }
        Some(response)
    }
}

fn artifact_definition(
    publication: &mizar_driver::events::SnapshotPublication,
    captured: &CapturedSnapshot,
    uri: &str,
    position: LspPosition,
) -> Option<Value> {
    let sources = captured.native_sources()?;
    let mut queried = sources.iter().filter(|(_, _, input)| {
        matches!(&input.origin, SourceOriginInput::OpenBuffer { uri: original, .. } if original == uri)
    });
    let (_, source, input) = queried.next()?;
    if queried.next().is_some() {
        return None;
    }
    let SourceOriginInput::OpenBuffer { text, .. } = &input.origin else {
        return None;
    };
    let (mut line, mut character) = (0usize, 0usize);
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        if (line, character) == (position.line as usize, position.character as usize) {
            break;
        }
        if matches!(ch, '\r' | '\n') {
            if line == position.line as usize {
                return None;
            }
            if ch == '\r' && chars.peek() == Some(&'\n') {
                chars.next();
            }
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16();
        }
    }
    if (line, character) != (position.line as usize, position.character as usize) {
        return None;
    }
    // Each binding uses only this retained publication and its original maps.
    let binding = |index: usize| {
        let artifact = publication.artifacts().get(index)?;
        let metadata = publication.source_metadata().get(index)?.as_ref()?;
        let summary = metadata.summary();
        let mut originals = sources.iter().filter(|(_, loaded, input)| {
            loaded.normalized_path.as_str() == artifact.source_file
                && input.package_id.as_str() == artifact.module.package_id
                && input.module_path.as_str() == artifact.module.module_path
        });
        let (_, loaded, input) = originals.next()?;
        let mut manifests = publication.manifests().iter().flat_map(|manifest| {
            manifest.manifest.modules.iter().filter_map(move |entry| {
                (entry.module == artifact.module).then_some((manifest, entry))
            })
        });
        let (manifest, entry) = manifests.next()?;
        if originals.next().is_some()
            || manifests.next().is_some()
            || !matches!(input.origin, SourceOriginInput::OpenBuffer { .. })
            || artifact.source_hash != loaded.source_hash
            || artifact
                .provenance
                .captured_source
                .as_ref()
                .is_some_and(|source| {
                    source != &(loaded.source_id, loaded.origin.clone(), loaded.source_hash)
                })
            || metadata.identity().is_some_and(|identity| {
                identity.artifact() != artifact.as_ref()
                    || identity.publication_hash() != manifest.artifact_hash
            })
            || summary.module != artifact.module
            || summary.source_hash != loaded.source_hash
            || entry.source_file != artifact.source_file
            || entry.source_hash != loaded.source_hash
            || entry.interface_hash.digest != artifact.interface_hash
            || entry.implementation_hash.digest != artifact.implementation_hash
            || entry.module_summary_interface_hash.as_ref()?.digest != summary.interface_hash
        {
            return None;
        }
        Some((artifact, metadata, loaded, input, manifest.artifact_hash))
    };
    let mut matching = publication
        .artifacts()
        .iter()
        .enumerate()
        .filter(|(_, artifact)| {
            artifact.source_file == source.normalized_path.as_str()
                && artifact.module.package_id == input.package_id.as_str()
                && artifact.module.module_path == input.module_path.as_str()
        });
    let (index, _) = matching.next()?;
    if matching.next().is_some() {
        return None;
    }
    let (artifact, metadata, loaded, input, publication_hash) = binding(index)?;
    let projection = metadata.direct_by()?;
    if projection.module() != &artifact.module
        || projection.publication_hash() != publication_hash
        || projection.captured_source()
            != &(loaded.source_id, loaded.origin.clone(), loaded.source_hash)
        || projection.loading_map() != loaded.loading_map.as_ref()
    {
        return None;
    }
    let coordinate = |position: LspPosition| (position.line, position.character);
    let mut selected = None;
    for occurrence in projection.occurrences() {
        let range = lsp_range_from_open_buffer(
            &loaded.line_map,
            loaded.loading_map.as_ref(),
            &input.origin,
            occurrence.target_range,
        )
        .ok()?;
        if coordinate(range.start) <= coordinate(position)
            && coordinate(position) < coordinate(range.end)
        {
            if selected.is_some() {
                return None;
            }
            selected = Some(occurrence);
        }
    }
    let mizar_artifact::direct_by::DirectByTarget::Theorem(endpoint) = &selected?.target else {
        return None;
    };
    let mut targets = publication
        .artifacts()
        .iter()
        .enumerate()
        .filter(|(_, target)| target.module == endpoint.module);
    let (index, _) = targets.next()?;
    if targets.next().is_some() {
        return None;
    }
    let (target, target_metadata, target_source, target_input, target_hash) = binding(index)?;
    let imports = if endpoint.module == artifact.module {
        None
    } else {
        let imports = metadata.imports()?;
        if imports.source_id() != loaded.source_id
            || imports.source_hash() != loaded.source_hash
            || imports.module().package().as_str() != artifact.module.package_id
            || imports.module().path().as_str() != artifact.module.module_path
        {
            return None;
        }
        Some(imports)
    };
    definition_export(
        target,
        target_metadata.summary(),
        target_hash,
        endpoint,
        imports,
    )?;
    let SourceOriginInput::OpenBuffer { uri, .. } = &target_input.origin else {
        return None;
    };
    if uri.is_empty()
        || sources.iter().filter(|(_, _, input)| {
            matches!(&input.origin, SourceOriginInput::OpenBuffer { uri: original, .. } if original == uri)
        }).count() != 1
    {
        return None;
    }
    let range = lsp_range_from_open_buffer(
        &target_source.line_map,
        target_source.loading_map.as_ref(),
        &target_input.origin,
        SourceRange {
            source_id: target_source.source_id,
            start: usize::try_from(endpoint.export.source_range.start_byte).ok()?,
            end: usize::try_from(endpoint.export.source_range.end_byte).ok()?,
        },
    )
    .ok()?;
    Some(json!({"uri": uri, "range": {
        "start": {"line": range.start.line, "character": range.start.character},
        "end": {"line": range.end.line, "character": range.end.character}
    }}))
}

fn definition_export<'a>(
    artifact: &VerifiedArtifact,
    summary: &'a mizar_artifact::module_summary::ModuleSummary,
    publication_hash: mizar_session::Hash,
    endpoint: &mizar_artifact::direct_by::DirectByEndpoint,
    imports: Option<&mizar_resolve::imports::PublishedImportView>,
) -> Option<&'a mizar_artifact::module_summary::ExportedSymbolSummary> {
    use mizar_artifact::{
        module_summary::ProofStatusSummary,
        verified_artifact::{ExportProofStatus, ExportVisibility},
    };
    let export = &endpoint.export;
    let mut exports = artifact
        .exports
        .iter()
        .filter(|row| row.origin_id == export.origin_id);
    let mut rows = summary
        .exported_symbols
        .iter()
        .filter(|row| row.origin_id == export.origin_id);
    let row = rows.next()?;
    if endpoint.module != artifact.module
        || summary.module != artifact.module
        || summary.source_hash != artifact.source_hash
        || endpoint.publication_hash != publication_hash
        || export.export_kind != "theorem"
        || exports.next()? != export
        || exports.next().is_some()
        || rows.next().is_some()
        || row.fully_qualified_name != export.fully_qualified_name
        || row.namespace_path != export.namespace_path
        || row.visibility
            != match export.visibility {
                ExportVisibility::Public => "public",
                ExportVisibility::Reexported => "reexported",
                _ => return None,
            }
        || row.declaration_kind != export.export_kind
        || row.source_range != export.source_range
        || row.rendered_signature != export.rendered_signature
        || row.interface_fingerprint != export.interface_fingerprint.digest
        || row.proof_status
            != match export.proof_status {
                Some(ExportProofStatus::Accepted) => Some(ProofStatusSummary::Accepted),
                Some(ExportProofStatus::NotAccepted) => Some(ProofStatusSummary::NotAccepted),
                Some(ExportProofStatus::NotRequired) => Some(ProofStatusSummary::NotRequired),
                None => None,
                _ => return None,
            }
    {
        return None;
    }
    if let Some(imports) = imports
        && (!imports
            .summaries()
            .iter()
            .any(|candidate| candidate == summary)
            || !imports
                .visible_symbols()?
                .iter()
                .any(|(_, module, visible, _)| *module == &endpoint.module && *visible == row))
    {
        return None;
    }
    Some(row)
}

fn artifact_hover(
    artifacts: &[Arc<VerifiedArtifact>],
    captured: &CapturedSnapshot,
    uri: &str,
    position: LspPosition,
) -> Result<Option<Value>, String> {
    let mut sources = captured.native_sources().unwrap_or_default().iter().filter(|(_, _, input)| {
        matches!(&input.origin, SourceOriginInput::OpenBuffer { uri: original, .. } if original == uri)
    });
    let Some((_, loaded, input)) = sources.next() else {
        return Ok(None);
    };
    if sources.next().is_some() {
        return Err("Ambiguous metadata source association".into());
    }
    let SourceOriginInput::OpenBuffer {
        actual_version,
        text,
        ..
    } = &input.origin
    else {
        unreachable!("source filtered above")
    };
    let (mut line, mut character) = (0, 0);
    let mut scalars = text.chars().peekable();
    while let Some(scalar) = scalars.next() {
        if (line, character) == (position.line as usize, position.character as usize) {
            break;
        }
        if matches!(scalar, '\r' | '\n') {
            if scalar == '\r' && scalars.peek() == Some(&'\n') {
                scalars.next();
            }
            line += 1;
            character = 0;
        } else {
            character += scalar.len_utf16();
        }
    }
    if (line, character) != (position.line as usize, position.character as usize) {
        return Ok(None);
    }
    let mut matching = artifacts.iter().filter(|artifact| {
        artifact.source_file == loaded.normalized_path.as_str()
            && artifact.module.package_id == input.package_id.as_str()
            && artifact.module.module_path == input.module_path.as_str()
    });
    let Some(artifact) = matching.next() else {
        return Ok(None);
    };
    if matching.next().is_some() {
        return Err("Ambiguous metadata artifact association".into());
    }
    let coordinate = |position: LspPosition| (position.line, position.character);
    let mut selected = None;
    for expression in &artifact.expressions {
        let native = expression.source_range;
        let range = lsp_range_from_open_buffer(
            &loaded.line_map,
            loaded.loading_map.as_ref(),
            &input.origin,
            SourceRange {
                source_id: loaded.source_id,
                start: usize::try_from(native.start_byte).map_err(|error| error.to_string())?,
                end: usize::try_from(native.end_byte).map_err(|error| error.to_string())?,
            },
        )
        .map_err(|error| format!("Invalid metadata source range: {error:?}"))?;
        if coordinate(range.start) <= coordinate(position)
            && coordinate(position) < coordinate(range.end)
        {
            let key = (
                native.end_byte - native.start_byte,
                &expression.expression_id,
            );
            if selected
                .as_ref()
                .is_none_or(|(previous, _, _)| key < *previous)
            {
                selected = Some((key, expression, range));
            }
        }
    }
    let Some((_, expression, range)) = selected else {
        return Ok(None);
    };
    let mut lines = vec![
        format!("Source: {uri} (version {actual_version})"),
        format!("Expression: {}", expression.expression_id),
        format!("Surface: {}", expression.rendered_surface),
    ];
    for (label, value) in [
        ("Type", expression.inferred_type.as_ref()),
        ("Symbol", expression.resolved_symbol.as_ref()),
        ("Thesis", expression.active_thesis.as_ref()),
    ] {
        if let Some(value) = value {
            lines.push(format!("{label}: {value}"));
        }
    }
    for coercion in &expression.inserted_coercions {
        lines.push(format!("Coercion: {coercion}"));
    }
    if let Some(overload) = &expression.overload_resolution {
        lines.push(format!("Overload root: {}", overload.root_symbol));
        lines.push(format!(
            "Selected candidate: {}",
            overload.selected_candidate
        ));
        for refinement in &overload.active_refinements {
            lines.push(format!("Refinement: {refinement}"));
        }
        if let Some(coercion) = &overload.coercion_summary {
            lines.push(format!("Overload coercion: {coercion}"));
        }
    }
    Ok(Some(json!({
        "contents": {"kind": "plaintext", "value": lines.join("\n")},
        "range": {"start": {"line": range.start.line, "character": range.start.character},
            "end": {"line": range.end.line, "character": range.end.character}}
    })))
}

#[cfg(test)]
#[path = "metadata_tests.rs"]
mod tests;
