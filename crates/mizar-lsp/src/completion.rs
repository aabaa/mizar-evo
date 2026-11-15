//! Bounded lexical completion over current buffers and published owner views.

use mizar_artifact::{
    module_summary::ModuleSummary,
    store::{CanonicalJson, canonical_json_string},
};
use mizar_lexer::{ExportedSymbolShape, UserSymbolKind};
use mizar_resolve::{imports::PublishedImportView, symbols::SourceNotationView};
use serde_json::{Value, json};

use super::Server;
use crate::LspPosition;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Context {
    Types,
    Terms,
    Formulas,
    Expressions,
    Citations,
    QualifiedSymbols,
    Keywords,
}

fn byte_offset(text: &str, position: LspPosition) -> Option<usize> {
    let (mut line, mut character) = (0, 0);
    let mut chars = text.char_indices().peekable();
    while let Some((offset, ch)) = chars.next() {
        if (line, character) == (position.line, position.character) {
            return Some(offset);
        }
        if matches!(ch, '\r' | '\n') {
            if line == position.line {
                return None;
            }
            if ch == '\r' && chars.peek().is_some_and(|(_, next)| *next == '\n') {
                chars.next();
            }
            line += 1;
            character = 0;
        } else {
            character += ch.len_utf16() as u32;
        }
    }
    ((line, character) == (position.line, position.character)).then_some(text.len())
}

fn completion_context(text: &str, position: LspPosition) -> Option<(Context, String)> {
    let offset = byte_offset(text, position)?;
    // Replacing CR preserves bytes while giving line comments their raw newline boundary.
    let raw = text.replace('\r', "\n");
    let source = mizar_lexer::preprocess_source_for_lexing(&raw);
    if source.comments.iter().any(|comment| {
        comment.span.start <= offset
            && offset <= comment.span.end
            && (comment.kind == mizar_lexer::CommentKind::MultiLine
                || !raw[..offset].ends_with('\n'))
    }) {
        return None;
    }
    let (mut quote, mut escaped, mut previous) = (None, false, None);
    let current_prefix = mizar_lexer::preprocess_source_for_lexing(&raw[..offset]);
    for ch in current_prefix.lexical_text.chars() {
        if let Some(delimiter) = quote {
            if ch == '\n' {
                quote = None;
                escaped = false;
            } else if escaped {
                escaped = false;
            } else if ch == '\\' {
                escaped = true;
            } else if ch == delimiter {
                quote = None;
            }
        } else if matches!(ch, '\'' | '"') && matches!(previous, Some('(' | ',')) {
            quote = Some(ch);
        }
        if !ch.is_whitespace() {
            previous = Some(ch);
        }
    }
    if quote.is_some() {
        return None;
    }
    let mut start = text[..offset]
        .char_indices()
        .rev()
        .take_while(|(_, ch)| {
            mizar_lexer::is_identifier_continue(*ch)
                || *ch == '.'
                || (mizar_lexer::is_lexeme_run_char(*ch)
                    && !matches!(
                        ch,
                        '(' | ')' | '[' | ']' | '{' | '}' | ',' | ';' | ':' | '='
                    ))
        })
        .last()
        .map_or(offset, |(i, _)| i);
    if let Some(arrow) = text[start..offset].rfind("->") {
        start += arrow + 2;
    }
    let prefix = text[start..offset].to_owned();
    // Lex only the prefix preceding the incomplete name, after comment preprocessing.
    let prefix_source = mizar_lexer::preprocess_source_for_lexing(&raw[..start]);
    let raw = mizar_lexer::RawTokenStream::new(
        mizar_lexer::scan_raw_recoverable(&prefix_source.lexical_text).tokens,
    );
    let environment =
        mizar_lexer::build_lexical_environment(&[], &[]).expect("empty lexical environment");
    let scopes = mizar_lexer::build_scope_skeleton(&raw);
    let tokens = mizar_lexer::disambiguate(
        &raw,
        &environment,
        &mizar_lexer::ParserLexContext::general(),
        &scopes,
    )
    .tokens
    .into_iter()
    .map(|token| token.lexeme)
    .collect::<Vec<_>>();
    let tokens = tokens
        .rsplit(|token| token == ";")
        .next()
        .unwrap_or_default();
    let context = if tokens.iter().any(|token| token == "by") {
        Context::Citations
    } else {
        match tokens.last().map(String::as_str) {
            Some("be" | "being" | "->" | "qua") => Context::Types,
            Some("holds" | "that" | "not" | "and" | "or" | "implies" | "iff" | ":") => {
                Context::Formulas
            }
            Some("=" | ":=" | "equals") => Context::Terms,
            Some("(" | ",") => Context::Expressions,
            _ if tokens.is_empty() && prefix.contains('.') => Context::QualifiedSymbols,
            _ => Context::Keywords,
        }
    };
    Some((context, prefix))
}

fn syntax_items(context: &(Context, String)) -> Vec<Value> {
    let keywords: &[&str] = match context.0 {
        Context::Types => &["object", "set"],
        Context::Terms => &["the"],
        Context::Formulas | Context::Expressions => &["ex", "for", "not"],
        Context::Citations | Context::QualifiedSymbols | Context::Keywords => &[],
    };
    keywords.iter().filter(|word| word.starts_with(&context.1)).map(|word|
        json!({"label":word,"kind":14,"insertText":word,"insertTextFormat":1,"detail":"keyword","data":{"rank":2,"identity":word}})).collect()
}

fn accepts(context: Context, kind: UserSymbolKind) -> bool {
    match context {
        Context::QualifiedSymbols => true,
        Context::Types => matches!(
            kind,
            UserSymbolKind::Mode | UserSymbolKind::Structure | UserSymbolKind::Attribute
        ),
        Context::Terms => matches!(
            kind,
            UserSymbolKind::Functor | UserSymbolKind::Constructor | UserSymbolKind::Selector
        ),
        Context::Formulas => matches!(kind, UserSymbolKind::Predicate | UserSymbolKind::Attribute),
        Context::Expressions => matches!(
            kind,
            UserSymbolKind::Functor
                | UserSymbolKind::Constructor
                | UserSymbolKind::Selector
                | UserSymbolKind::Predicate
                | UserSymbolKind::Attribute
        ),
        _ => false,
    }
}

fn item(path: &[String], prefix: &str, rank: u8, identity: &str, detail: String) -> Option<Value> {
    // Match only published namespace boundaries; dots inside a notation piece
    // belong to that piece, rather than a namespace reconstructed from text.
    let (index, next) = path.iter().enumerate().find(|(index, next)| {
        let parent = path[..*index].join(".");
        let partial = if *index == 0 {
            Some(prefix)
        } else {
            prefix.strip_prefix(&format!("{parent}."))
        };
        partial.is_some_and(|partial| next.starts_with(partial))
    })?;
    let namespace = index + 1 < path.len();
    let access = path[..=index].join(".");
    let identity = if namespace {
        format!("namespace:{access}")
    } else {
        identity.to_owned()
    };
    let detail = if namespace {
        format!(
            "{}\nnamespace {access}",
            detail.lines().next().unwrap_or_default()
        )
    } else {
        detail
    };
    Some(
        json!({"label":next,"kind":if namespace { 9 } else { 6 },"insertText":next,"insertTextFormat":1,
        "detail":detail,"data":{"rank":rank,"identity":identity,"accessPath":access}}),
    )
}

fn append_semantic_items(
    items: &mut Vec<Value>,
    summary: &ModuleSummary,
    notation: Option<&SourceNotationView>,
    imports: Option<&PublishedImportView>,
    offsets: (usize, usize),
    context: &(Context, String),
    provenance: &str,
) {
    if let Some(notation) = notation {
        for row in notation.declarations() {
            if !row
                .activation()
                .original_input
                .map_or(row.activation().primary.end <= offsets.1, |range| {
                    range.end <= offsets.0
                })
                || !accepts(context.0, row.declaration().kind)
            {
                continue;
            }
            let declaration = row.declaration();
            let operator = notation
                .operator_events()
                .iter()
                .filter(|event| {
                    event.declaration().spelling == declaration.spelling
                        && event
                            .activation()
                            .original_input
                            .map_or(event.activation().primary.end <= offsets.1, |range| {
                                range.end <= offsets.0
                            })
                })
                .max_by_key(|event| event.activation().primary.end)
                .map_or(declaration.operator, |event| event.declaration().operator);
            let identity = format!("{:?}", row.owner().symbol());
            let detail = format!(
                "{provenance}\n{:?}; arity {:?}; operator {:?}; signature {:?}; relations {:?}",
                declaration.kind,
                declaration.arity,
                operator,
                row.owner().signature(),
                row.owner().relations()
            );
            if let Some(item) = item(
                std::slice::from_ref(&declaration.spelling),
                &context.1,
                0,
                &identity,
                detail,
            ) {
                items.push(item);
            }
        }
    }
    if context.0 == Context::Citations {
        for label in &summary.exported_labels {
            if !matches!(label.target_kind.as_str(), "theorem" | "definition") {
                continue;
            }
            let mut owners = summary
                .exported_symbols
                .iter()
                .filter(|owner| owner.fully_qualified_name == label.owner_fully_qualified_name);
            let Some(owner) = owners.next() else {
                continue;
            };
            if owners.next().is_some() || owner.source_range.end_byte > offsets.1 as u64 {
                continue;
            }
            let detail = format!(
                "{provenance}\n{}; status {:?}",
                owner.rendered_signature, owner.proof_status
            );
            if let Some(item) = item(
                std::slice::from_ref(&label.label),
                &context.1,
                0,
                &label.origin_id,
                detail,
            ) {
                items.push(item);
            }
        }
    }
    let Some(imports) = imports else {
        return;
    };
    if context.0 == Context::Citations {
        if let Some(labels) = imports.visible_labels() {
            for (_, module, label, path) in labels {
                if !matches!(label.target_kind.as_str(), "theorem" | "definition") {
                    continue;
                }
                let Some(owner) = imports
                    .summaries()
                    .iter()
                    .find(|summary| &summary.module == module)
                    .and_then(|summary| {
                        summary.exported_symbols.iter().find(|owner| {
                            owner.fully_qualified_name == label.owner_fully_qualified_name
                        })
                    })
                else {
                    continue;
                };
                let detail = format!(
                    "{provenance}\n{}; status {:?}; origin {:?}/{}",
                    owner.rendered_signature, owner.proof_status, module, label.origin_id
                );
                if let Some(item) = item(
                    &path,
                    &context.1,
                    1,
                    &format!("{module:?}/{}", label.origin_id),
                    detail,
                ) {
                    items.push(item);
                }
            }
        }
    } else if let Some(symbols) = imports.visible_symbols() {
        let summaries = imports
            .summaries()
            .iter()
            .filter_map(|summary| {
                // PublishedImportView already validated the complete lexical product.
                let shapes = summary
                    .lexical_summary
                    .contributions
                    .iter()
                    .map(|entry| {
                        ExportedSymbolShape::from_canonical_bytes(entry.payload.as_bytes())
                    })
                    .collect::<Option<Vec<_>>>()?;
                Some((&summary.module, shapes))
            })
            .collect::<Vec<_>>();
        for (_, module, row, path) in symbols {
            let Some((_, shapes)) = summaries.iter().find(|(identity, _)| *identity == module)
            else {
                continue;
            };
            let identity = canonical_json_string(&CanonicalJson::array([
                module.canonical_json().expect("published module identity"),
                CanonicalJson::string(&row.origin_id),
            ]));
            for shape in shapes.iter().filter(|shape| {
                shape.symbol_id.as_str() == identity && accepts(context.0, shape.kind)
            }) {
                let mut path = path.clone();
                if let Some(last) = path.last_mut() {
                    *last = shape.spelling.clone();
                }
                let operator = notation
                    .and_then(|view| {
                        view.operator_events()
                            .iter()
                            .filter(|event| {
                                event.declaration().spelling == shape.spelling
                                    && event.activation().original_input.map_or(
                                        event.activation().primary.end <= offsets.1,
                                        |range| range.end <= offsets.0,
                                    )
                            })
                            .max_by_key(|event| event.activation().primary.end)
                    })
                    .map_or(shape.operator, |event| event.declaration().operator);
                let detail = format!(
                    "{provenance}\n{:?}; arity {:?}; operator {:?}; declaration {}; {}; status {:?}; origin {}",
                    shape.kind,
                    shape.arity,
                    operator,
                    row.declaration_kind,
                    row.rendered_signature,
                    row.proof_status,
                    identity
                );
                if let Some(item) = item(&path, &context.1, 1, &identity, detail) {
                    items.push(item);
                }
            }
        }
    }
}

fn sort_items(mut items: Vec<Value>, prefix: &str) -> Vec<Value> {
    let key = |item: &Value| {
        (
            !item["label"]
                .as_str()
                .is_some_and(|label| label == prefix || prefix.ends_with(&format!(".{label}"))),
            item["data"]["rank"].as_u64(),
            item["label"].as_str().unwrap_or_default().to_owned(),
            item["data"]["identity"]
                .as_str()
                .unwrap_or_default()
                .to_owned(),
            item.to_string(),
        )
    };
    items.sort_by_key(&key);
    items.dedup();
    for (index, item) in items.iter_mut().enumerate() {
        item["sortText"] = json!(format!("{index:08}"));
    }
    items
}

impl Server {
    pub(super) fn completion(&self, uri: &str, position: LspPosition) -> Vec<Value> {
        self.document(uri)
            .and_then(|(_, text)| completion_context(text, position))
            .map_or_else(Vec::new, |context| {
                sort_items(syntax_items(&context), &context.1)
            })
    }

    /// Adds metadata from one retained publication, using its original source coordinates.
    pub(super) fn snapshot_completion<A: mizar_session::SessionIdAllocator>(
        &self,
        driver: &mizar_driver::driver::CompilerDriver,
        snapshots: &mizar_session::SnapshotRegistry<A>,
        uri: &str,
        position: LspPosition,
    ) -> Vec<Value> {
        if self.initialized.is_none() || self.shutdown || self.exit_code.is_some() {
            return Vec::new();
        }
        let Some(context) = self
            .document(uri)
            .and_then(|(_, text)| completion_context(text, position))
        else {
            return Vec::new();
        };
        let mut items = syntax_items(&context);
        let Some((snapshot, freshness)) = self.lsp_snapshot(driver, snapshots) else {
            return sort_items(items, &context.1);
        };
        let publication = snapshot.publication();
        let mut sources = snapshot.native_open_buffers().captured().native_sources().unwrap_or_default().iter().filter(|(_, _, input)| matches!(&input.origin, mizar_session::SourceOriginInput::OpenBuffer { uri: original, .. } if original == uri));
        let Some((_, loaded, input)) = sources.next() else {
            return sort_items(items, &context.1);
        };
        if sources.next().is_some() {
            return sort_items(items, &context.1);
        }
        let mizar_session::SourceOriginInput::OpenBuffer {
            text,
            actual_version,
            ..
        } = &input.origin
        else {
            unreachable!("filtered open buffer");
        };
        let Some(raw_offset) = byte_offset(text, position) else {
            return sort_items(items, &context.1);
        };
        let native_offset = if let Some(map) = &loaded.loading_map {
            // The loading map is monotonic; choose the last loaded boundary at this raw cursor.
            let (mut low, mut high) = (0, loaded.text.len());
            while low < high {
                let middle = low + (high - low).div_ceil(2);
                if map
                    .original_offset_for_loaded(loaded.source_id, middle)
                    .is_ok_and(|offset| offset <= raw_offset)
                {
                    low = middle;
                } else {
                    high = middle - 1;
                }
            }
            low
        } else {
            raw_offset
        };
        let mut artifacts = publication
            .artifacts()
            .iter()
            .enumerate()
            .filter(|(_, artifact)| {
                artifact.source_file == loaded.normalized_path.as_str()
                    && artifact.module.package_id == input.package_id.as_str()
                    && artifact.module.module_path == input.module_path.as_str()
            });
        let Some((index, _)) = artifacts.next() else {
            return sort_items(items, &context.1);
        };
        if artifacts.next().is_some() {
            return sort_items(items, &context.1);
        }
        let Some(Some(metadata)) = publication.source_metadata().get(index) else {
            return sort_items(items, &context.1);
        };
        let current = freshness == super::LspSnapshotFreshness::Current
            && self
                .lsp_snapshot(driver, snapshots)
                .is_some_and(|(now, state)| {
                    std::sync::Arc::ptr_eq(&snapshot, &now)
                        && state == super::LspSnapshotFreshness::Current
                });
        let provenance = format!(
            "{}Source: {uri} (version {actual_version})",
            if current { "" } else { "stale_artifact: " }
        );
        append_semantic_items(
            &mut items,
            metadata.summary(),
            metadata.notation(),
            metadata.imports(),
            (raw_offset, native_offset),
            &context,
            &provenance,
        );
        sort_items(items, &context.1)
    }
}

#[cfg(test)]
#[path = "completion_tests.rs"]
mod tests;
