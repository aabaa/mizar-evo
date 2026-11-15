//! Hover projection from published expressions and their retained original source maps.

use std::sync::Arc;

use mizar_artifact::verified_artifact::VerifiedArtifact;
use mizar_driver::{driver::CompilerDriver, request::CapturedSnapshot};
use mizar_session::{SessionIdAllocator, SnapshotRegistry, SourceOriginInput, SourceRange};
use serde_json::{Value, json};

use super::{LspSnapshotFreshness, Server};
use crate::{LspPosition, lsp_range_from_open_buffer};

impl Server {
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
        if message.get("method").and_then(Value::as_str) != Some("textDocument/hover") {
            return self.handle_message(message);
        }
        let mut response = self.handle_message(message.clone())?;
        if response.get("result").is_none() {
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
