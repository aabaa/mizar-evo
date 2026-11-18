//! Isolated opaque math transport and local asset readiness.

use std::fmt::Write;

use crate::{doc_build::DocBuildError, markdown::ParsedComments};

/// Emits source-ordered fragments; provenance remains in `parsed` at each ordinal.
pub fn opaque_math_fragments(
    parsed: &ParsedComments<'_, '_>,
) -> Result<Vec<String>, DocBuildError> {
    let refuse = || DocBuildError::Binding("opaque math source binding mismatch");
    if !parsed.available() || parsed.blocks.len() != parsed.attachments.blocks.len() {
        return Err(refuse());
    }
    let mut fragments = Vec::new();
    for (block_ordinal, block) in parsed.blocks.iter().enumerate() {
        let mut cursor = 0;
        for (occurrence, math) in block.math.iter().enumerate() {
            if math.range.start < cursor
                || math.range.start >= math.range.end
                || block.text.get(math.range.clone()).is_none()
                || block.fallback
            {
                return Err(refuse());
            }
            cursor = math.range.end;
            let mut payload = String::new();
            let mut end = math.range.start;
            for segment in &math.content {
                if segment.start < end || segment.end > math.range.end {
                    return Err(refuse());
                }
                payload.push_str(block.text.get(segment.clone()).ok_or_else(refuse)?);
                end = segment.end;
            }
            let tag = if math.display { "div" } else { "span" };
            let mut html = format!(
                "<{tag} class=\"mizar-math\" data-mizar-block=\"{block_ordinal}\" data-mizar-occurrence=\"{occurrence}\" data-mizar-display=\"{}\" data-mizar-tex=\"",
                math.display
            );
            for byte in payload.as_bytes() {
                write!(html, "{byte:02x}").expect("writing to String cannot fail");
            }
            html.push_str("\"><span role=\"status\">Mathematics pending</span><code>");
            for ch in payload.chars() {
                match ch {
                    '&' => html.push_str("&amp;"),
                    '<' => html.push_str("&lt;"),
                    '>' => html.push_str("&gt;"),
                    '"' => html.push_str("&quot;"),
                    '\'' => html.push_str("&#39;"),
                    ch if ch.is_control() && ch != '\n' && ch != '\t' => {
                        write!(html, "\\u{{{:x}}}", u32::from(ch))
                            .expect("writing to String cannot fail");
                    }
                    ch => html.push(ch),
                }
            }
            writeln!(html, "</code></{tag}>").expect("writing to String cannot fail");
            fragments.push(html);
        }
    }
    Ok(fragments)
}

/// Refuses incomplete, duplicated, extra or changed local asset/configuration bytes.
pub fn validate_math_assets(assets: &[(&str, &[u8])]) -> Result<(), DocBuildError> {
    let expected = crate::math::bundled_math_assets();
    if assets.len() != expected.len()
        || expected.iter().any(|(path, bytes)| {
            let mut found = assets.iter().filter(|(candidate, _)| candidate == path);
            !found.next().is_some_and(|(_, supplied)| supplied == bytes) || found.next().is_some()
        })
    {
        return Err(DocBuildError::Binding("opaque math local assets mismatch"));
    }
    Ok(())
}
