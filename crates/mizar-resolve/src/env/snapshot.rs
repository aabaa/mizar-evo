use super::*;
use std::fmt::Write as _;

pub(super) fn symbol_env_snapshot_text(env: &SymbolEnv) -> String {
    let mut output = String::from("symbol-env-debug-v1\n");
    output.push_str("module: ");
    write_module_id(&mut output, &env.module_id);
    output.push('\n');
    write_import_index_snapshot(&mut output, env.imports());
    write_export_index_snapshot(&mut output, env.exports());
    write_symbol_index_snapshot(&mut output, env.symbols());
    write_label_index_snapshot(&mut output, env.labels());
    write_definition_index_snapshot(&mut output, env.definitions());
    write_overload_index_snapshot(&mut output, env.overloads());
    write_registration_index_snapshot(&mut output, env.registrations());
    write_lexical_summary_index_snapshot(&mut output, env.lexical_summaries());
    write_namespace_graph_snapshot(&mut output, env.namespace_graph());
    write_declaration_dependency_index_snapshot(&mut output, env.declaration_dependencies());
    write_contribution_index_snapshot(&mut output, env.contributions());
    write_module_summary_index_snapshot(&mut output, env.module_summaries());
    output
}

fn write_import_index_snapshot(output: &mut String, imports: &ResolvedImportIndex) {
    output.push_str("imports:\n");
    if imports.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in imports.iter() {
        let _ = write!(output, "  import#{} module=", entry.import().index());
        match entry.module() {
            Some(module) => write_module_id(output, module),
            None => output.push_str("<none>"),
        }
        output.push_str(" alias=");
        match entry.alias() {
            Some(alias) => {
                output.push('"');
                write_escaped(output, alias);
                output.push('"');
            }
            None => output.push_str("<none>"),
        }
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_export_index_snapshot(output: &mut String, exports: &ResolvedExportIndex) {
    output.push_str("exports:\n");
    if exports.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in exports.iter() {
        let _ = write!(output, "  export#{} target=", entry.export().index());
        match entry.target() {
            Some(target) => write_dependency_endpoint(output, target),
            None => output.push_str("<none>"),
        }
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_symbol_index_snapshot(output: &mut String, symbols: &SymbolIndex) {
    output.push_str("symbols:\n");
    if symbols.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in symbols.iter() {
        output.push_str("  symbol=");
        write_symbol_id(output, entry.symbol());
        let _ = write!(
            output,
            " kind={} visibility={} export={} namespace=\"",
            symbol_kind_name(entry.kind()),
            visibility_name(entry.visibility()),
            export_status_name(entry.export_status())
        );
        write_escaped(output, entry.namespace().as_str());
        output.push_str("\" spelling=\"");
        write_escaped(output, entry.primary_spelling());
        output.push_str("\" notation=");
        match entry.notation_spelling() {
            Some(spelling) => {
                output.push('"');
                write_escaped(output, spelling);
                output.push('"');
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" signature=");
        write_optional_signature_shell(output, entry.signature());
        output.push_str(" relations=");
        write_relations(output, entry.relations());
        output.push_str(" origin=");
        write_origin(output, entry.origin());
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_label_index_snapshot(output: &mut String, labels: &LabelIndex) {
    output.push_str("labels:\n");
    if labels.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in labels.iter() {
        output.push_str("  label=\"");
        write_escaped(output, entry.origin_path().as_str());
        let _ = write!(
            output,
            "\" kind={} visibility={} export={} namespace=\"",
            label_kind_name(entry.kind()),
            visibility_name(entry.visibility()),
            export_status_name(entry.export_status())
        );
        write_escaped(output, entry.namespace().as_str());
        output.push_str("\" spelling=\"");
        write_escaped(output, entry.primary_spelling());
        output.push_str("\" origin=");
        write_origin(output, entry.origin());
        let _ = writeln!(
            output,
            " contribution=contribution#{} recovery={}",
            entry.contribution().index(),
            recovery_state_name(entry.recovery())
        );
    }
}

fn write_definition_index_snapshot(output: &mut String, definitions: &DefinitionIndex) {
    output.push_str("definitions:\n");
    if definitions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in definitions.iter() {
        let _ = write!(output, "  definition#{} symbol=", entry.id().index());
        write_symbol_id(output, entry.symbol());
        let _ = write!(
            output,
            " kind={} visibility={} parameters=",
            definition_kind_name(entry.kind()),
            visibility_name(entry.visibility())
        );
        write_shell_ids(output, entry.parameters());
        output.push_str(" binders=");
        write_shell_ids(output, entry.binders());
        output.push_str(" arity=");
        match entry.arity() {
            Some(arity) => {
                let _ = write!(output, "{arity}");
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" notation=");
        match entry.notation_shape() {
            Some(shape) => {
                output.push('"');
                write_escaped(output, shape);
                output.push('"');
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" doc=");
        match entry.doc_attachment() {
            Some(doc) => {
                output.push('"');
                write_escaped(output, doc.as_str());
                output.push('"');
            }
            None => output.push_str("<none>"),
        }
        output.push_str(" conflict=");
        match entry.conflict() {
            Some(conflict) => output.push_str(declaration_conflict_class_name(conflict)),
            None => output.push_str("<none>"),
        }
        output.push_str(" dependencies=");
        write_declaration_dependency_ids(output, entry.dependencies());
        output.push_str(" signature=");
        write_optional_signature_shell(output, entry.signature());
        output.push_str(" origin=");
        write_origin(output, entry.origin());
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_overload_index_snapshot(output: &mut String, overloads: &OverloadIndex) {
    output.push_str("overloads:\n");
    if overloads.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for group in overloads.iter() {
        let _ = write!(output, "  overload#{} key=", group.id().index());
        write_overload_key(output, group.key());
        output.push_str(" candidates=");
        write_symbol_ids(output, group.candidates());
        output.push_str(" diagnostics=");
        write_diagnostic_anchor_ids(output, group.diagnostics());
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            group.contribution().index()
        );
    }
}

fn write_registration_index_snapshot(output: &mut String, registrations: &RegistrationIndex) {
    output.push_str("registrations:\n");
    if registrations.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in registrations.iter() {
        let _ = write!(output, "  registration#{} symbol=", entry.id().index());
        match entry.symbol() {
            Some(symbol) => write_symbol_id(output, symbol),
            None => output.push_str("<none>"),
        }
        let _ = write!(
            output,
            " kind={} target=",
            registration_kind_name(entry.kind())
        );
        write_signature_shell(output, entry.target());
        let _ = write!(
            output,
            " visibility={} export={} dependencies=",
            visibility_name(entry.visibility()),
            export_status_name(entry.export_status())
        );
        write_declaration_dependency_ids(output, entry.dependencies());
        output.push_str(" origin=");
        write_origin(output, entry.origin());
        let _ = writeln!(
            output,
            " contribution=contribution#{} recovery={}",
            entry.contribution().index(),
            recovery_state_name(entry.recovery())
        );
    }
}

fn write_lexical_summary_index_snapshot(
    output: &mut String,
    summaries: &ModuleLexicalSummaryIndex,
) {
    output.push_str("lexical_summaries:\n");
    if summaries.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in summaries.iter() {
        let _ = write!(output, "  lexical#{} symbol=", entry.id().index());
        write_symbol_id(output, entry.symbol());
        output.push_str(" namespace=\"");
        write_escaped(output, entry.namespace().as_str());
        output.push_str("\" spelling=\"");
        write_escaped(output, entry.spelling());
        output.push_str("\" kind=");
        output.push_str(lexical_summary_kind_name(entry.kind()));
        output.push_str(" arity=");
        match entry.arity() {
            Some(arity) => {
                let _ = write!(output, "{arity}");
            }
            None => output.push_str("<none>"),
        }
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_namespace_graph_snapshot(output: &mut String, graph: &NamespaceGraph) {
    output.push_str("namespace_graph:\n");
    output.push_str("  nodes:\n");
    if graph.node_len() == 0 {
        output.push_str("    <none>\n");
    } else {
        for node in graph.nodes() {
            let _ = write!(
                output,
                "    node#{} kind={} module=",
                node.id().index(),
                namespace_node_kind_name(node.kind())
            );
            match node.module() {
                Some(module) => write_module_id(output, module),
                None => output.push_str("<none>"),
            }
            output.push_str(" spelling=\"");
            write_escaped(output, node.spelling());
            let _ = writeln!(
                output,
                "\" contribution=contribution#{}",
                node.contribution().index()
            );
        }
    }
    output.push_str("  edges:\n");
    if graph.edge_len() == 0 {
        output.push_str("    <none>\n");
    } else {
        for edge in graph.edges() {
            let _ = write!(
                output,
                "    edge#{} from=node#{} to=node#{} kind={} anchor=",
                edge.id().index(),
                edge.from().index(),
                edge.to().index(),
                namespace_edge_kind_name(edge.kind())
            );
            write_anchor(output, edge.anchor());
            let _ = write!(
                output,
                " visibility={} target=",
                visibility_name(edge.visibility())
            );
            match edge.target() {
                Some(target) => write_namespace_target(output, target),
                None => output.push_str("<none>"),
            }
            output.push_str(" local_spelling=");
            match edge.local_spelling() {
                Some(spelling) => {
                    output.push('"');
                    write_escaped(output, spelling);
                    output.push('"');
                }
                None => output.push_str("<none>"),
            }
            let _ = writeln!(
                output,
                " contribution=contribution#{}",
                edge.contribution().index()
            );
        }
    }
}

fn write_declaration_dependency_index_snapshot(
    output: &mut String,
    dependencies: &DeclarationDependencyIndex,
) {
    output.push_str("declaration_dependencies:\n");
    if dependencies.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for dependency in dependencies.iter() {
        let _ = write!(output, "  dependency#{} source=", dependency.id().index());
        write_dependency_endpoint(output, dependency.source());
        output.push_str(" target=");
        write_dependency_endpoint(output, dependency.target());
        let _ = write!(
            output,
            " kind={} anchor=",
            declaration_dependency_kind_name(dependency.kind())
        );
        write_anchor(output, dependency.anchor());
        let _ = writeln!(
            output,
            " contribution=contribution#{}",
            dependency.contribution().index()
        );
    }
}

fn write_contribution_index_snapshot(output: &mut String, contributions: &SourceContributionIndex) {
    output.push_str("contributions:\n");
    if contributions.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for contribution in contributions.iter() {
        let _ = write!(
            output,
            "  contribution#{} module=",
            contribution.id().index()
        );
        write_module_id(output, contribution.module());
        output.push_str(" kind=");
        write_contribution_kind(output, contribution.kind());
        output.push_str(" anchor=");
        write_anchor(output, contribution.anchor());
        output.push_str(" effects=");
        write_contribution_effects(output, contribution.effects());
        output.push('\n');
    }
}

fn write_module_summary_index_snapshot(output: &mut String, summaries: &ModuleSummaryIndex) {
    output.push_str("module_summaries:\n");
    if summaries.is_empty() {
        output.push_str("  <none>\n");
        return;
    }
    for entry in summaries.iter() {
        let _ = write!(output, "  summary#{} module=", entry.id().index());
        write_module_id(output, entry.module());
        output.push_str(" identity=\"");
        write_escaped(output, entry.identity().as_str());
        let _ = writeln!(
            output,
            "\" contribution=contribution#{}",
            entry.contribution().index()
        );
    }
}

fn write_overload_key(output: &mut String, key: &OverloadKey) {
    output.push_str("{namespace=\"");
    write_escaped(output, key.namespace().as_str());
    output.push_str("\" spelling=\"");
    write_escaped(output, key.spelling());
    output.push_str("\" kind=");
    output.push_str(symbol_kind_name(key.kind()));
    output.push_str(" arity=");
    match key.arity() {
        Some(arity) => {
            let _ = write!(output, "{arity}");
        }
        None => output.push_str("<none>"),
    }
    output.push('}');
}

fn write_contribution_kind(output: &mut String, kind: &ContributionKind) {
    match kind {
        ContributionKind::LocalSource { .. } => output.push_str("local_source"),
        ContributionKind::ImportedSource { .. } => output.push_str("imported_source"),
        ContributionKind::Summary { identity } => {
            output.push_str("summary identity=\"");
            write_escaped(output, identity.as_str());
            output.push('"');
        }
        ContributionKind::Builtin { name } => {
            output.push_str("builtin name=\"");
            write_escaped(output, name);
            output.push('"');
        }
    }
}

fn write_contribution_effects(output: &mut String, effects: &ContributionEffects) {
    output.push('{');
    output.push_str("symbols=");
    write_symbol_ids(output, effects.symbols());
    output.push_str(" definitions=");
    write_definition_ids(output, effects.definitions());
    output.push_str(" overloads=");
    write_overload_group_ids(output, effects.overload_groups());
    output.push_str(" registrations=");
    write_registration_ids(output, effects.registrations());
    output.push_str(" lexical_summaries=");
    write_lexical_summary_ids(output, effects.lexical_summaries());
    output.push_str(" labels=");
    write_label_origin_paths(output, effects.labels());
    output.push_str(" namespace_edges=");
    write_namespace_edge_ids(output, effects.namespace_edges());
    output.push_str(" declaration_dependencies=");
    write_declaration_dependency_ids(output, effects.declaration_dependencies());
    output.push_str(" imports=");
    write_import_ids(output, effects.imports());
    output.push_str(" exports=");
    write_export_ids(output, effects.exports());
    output.push_str(" diagnostics=");
    write_diagnostic_anchor_ids(output, effects.diagnostics());
    output.push('}');
}

fn write_dependency_endpoint(output: &mut String, endpoint: &DependencyEndpoint) {
    match endpoint {
        DependencyEndpoint::Symbol(symbol) => {
            output.push_str("symbol=");
            write_symbol_id(output, symbol);
        }
        DependencyEndpoint::Import(import) => {
            let _ = write!(output, "import#{}", import.index());
        }
        DependencyEndpoint::Export(export) => {
            let _ = write!(output, "export#{}", export.index());
        }
        DependencyEndpoint::NamespaceEdge(edge) => {
            let _ = write!(output, "namespace_edge#{}", edge.index());
        }
        DependencyEndpoint::Label(label) => {
            output.push_str("label=\"");
            write_escaped(output, label.as_str());
            output.push('"');
        }
        DependencyEndpoint::UnresolvedName(name) => {
            let _ = write!(output, "unresolved_name#{}", name.index());
        }
        DependencyEndpoint::UnresolvedLabel(label) => {
            let _ = write!(output, "unresolved_label#{}", label.index());
        }
        DependencyEndpoint::Module(module) => {
            output.push_str("module=");
            write_module_id(output, module);
        }
    }
}

fn write_namespace_target(output: &mut String, target: &NamespaceTarget) {
    match target {
        NamespaceTarget::Module(module) => {
            output.push_str("module=");
            write_module_id(output, module);
        }
        NamespaceTarget::Symbol(symbol) => {
            output.push_str("symbol=");
            write_symbol_id(output, symbol);
        }
        NamespaceTarget::Label(label) => {
            output.push_str("label=\"");
            write_escaped(output, label.as_str());
            output.push('"');
        }
        NamespaceTarget::Unresolved(spelling) => {
            output.push_str("unresolved=\"");
            write_escaped(output, spelling);
            output.push('"');
        }
    }
}

fn write_relations(output: &mut String, relations: &[RelationMetadata]) {
    output.push('[');
    for (index, relation) in relations.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push_str(relation_kind_name(relation.kind()));
        output.push_str("->");
        write_symbol_id(output, relation.target());
    }
    output.push(']');
}

fn write_optional_signature_shell(output: &mut String, signature: Option<&SignatureShell>) {
    match signature {
        Some(signature) => write_signature_shell(output, signature),
        None => output.push_str("<none>"),
    }
}

fn write_signature_shell(output: &mut String, signature: &SignatureShell) {
    match signature {
        SignatureShell::Pending => output.push_str("pending"),
        SignatureShell::Opaque { schema, payload } => {
            output.push_str("opaque(schema=\"");
            write_escaped(output, schema);
            output.push_str("\", payload=\"");
            write_escaped(output, payload);
            output.push_str("\")");
        }
        SignatureShell::Malformed { class } => {
            output.push_str("malformed(class=\"");
            write_escaped(output, class);
            output.push_str("\")");
        }
    }
}

fn write_origin(output: &mut String, origin: &SemanticOrigin) {
    output.push('{');
    output.push_str("module=");
    write_module_id(output, origin.module_id());
    output.push_str(" anchor=");
    write_anchor(output, origin.anchor());
    output.push_str(" path=");
    write_u32_list(output, origin.structural_path());
    output.push_str(" import=");
    match origin.import_edge() {
        Some(import) => {
            let _ = write!(output, "import#{}", import.index());
        }
        None => output.push_str("<none>"),
    }
    let _ = write!(output, " recovered={}", origin.is_recovered());
    output.push('}');
}

fn write_anchor(output: &mut String, anchor: &SourceAnchor) {
    match anchor {
        SourceAnchor::Range(range) => {
            output.push_str("range(");
            write_range(output, *range);
            output.push(')');
        }
        SourceAnchor::Point { offset, .. } => {
            let _ = write!(output, "point({offset})");
        }
        SourceAnchor::Generated(origin) => {
            output.push_str("generated(");
            write_generated_span_anchor(output, origin.anchor());
            output.push_str(", reason=present)");
        }
        _ => output.push_str("unknown"),
    }
}

fn write_generated_span_anchor(output: &mut String, anchor: GeneratedSpanAnchor) {
    match anchor {
        GeneratedSpanAnchor::Range(range) => {
            output.push_str("range(");
            write_range(output, range);
            output.push(')');
        }
        GeneratedSpanAnchor::Point { offset, .. } => {
            let _ = write!(output, "point({offset})");
        }
        _ => output.push_str("unknown"),
    }
}

fn write_symbol_ids(output: &mut String, ids: &[SymbolId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        write_symbol_id(output, id);
    }
    output.push(']');
}

fn write_shell_ids(output: &mut String, ids: &[ResolverShellId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('"');
        write_escaped(output, id.as_str());
        output.push('"');
    }
    output.push(']');
}

fn write_label_origin_paths(output: &mut String, ids: &[LabelOriginPath]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        output.push('"');
        write_escaped(output, id.as_str());
        output.push('"');
    }
    output.push(']');
}

fn write_definition_ids(output: &mut String, ids: &[DefinitionId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "definition#{}", id.index());
    }
    output.push(']');
}

fn write_overload_group_ids(output: &mut String, ids: &[OverloadGroupId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "overload#{}", id.index());
    }
    output.push(']');
}

fn write_registration_ids(output: &mut String, ids: &[RegistrationId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "registration#{}", id.index());
    }
    output.push(']');
}

fn write_lexical_summary_ids(output: &mut String, ids: &[LexicalSummaryId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "lexical#{}", id.index());
    }
    output.push(']');
}

fn write_namespace_edge_ids(output: &mut String, ids: &[NamespaceEdgeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "edge#{}", id.index());
    }
    output.push(']');
}

fn write_declaration_dependency_ids(output: &mut String, ids: &[DeclarationDependencyId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "dependency#{}", id.index());
    }
    output.push(']');
}

fn write_import_ids(output: &mut String, ids: &[ResolvedImportId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "import#{}", id.index());
    }
    output.push(']');
}

fn write_export_ids(output: &mut String, ids: &[ResolvedExportId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "export#{}", id.index());
    }
    output.push(']');
}

fn write_diagnostic_anchor_ids(output: &mut String, ids: &[DiagnosticAnchorId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "diagnostic#{}", id.index());
    }
    output.push(']');
}

fn write_u32_list(output: &mut String, values: &[u32]) {
    output.push('[');
    for (index, value) in values.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "{value}");
    }
    output.push(']');
}

fn write_symbol_id(output: &mut String, symbol: &SymbolId) {
    output.push_str("{fqn=\"");
    write_escaped(output, symbol.fqn().as_str());
    output.push_str("\" module=");
    write_module_id(output, symbol.module());
    output.push_str(" local=\"");
    write_escaped(output, symbol.local().as_str());
    output.push_str("\"}");
}

fn write_module_id(output: &mut String, module: &ModuleId) {
    write_escaped(output, module.package().as_str());
    output.push_str("::");
    write_escaped(output, module.path().as_str());
}

fn write_range(output: &mut String, range: SourceRange) {
    let _ = write!(output, "{}..{}", range.start, range.end);
}

fn write_escaped(output: &mut String, value: &str) {
    for character in value.chars() {
        for escaped in character.escape_default() {
            output.push(escaped);
        }
    }
}

fn symbol_kind_name(kind: SymbolKind) -> &'static str {
    match kind {
        SymbolKind::Predicate => "predicate",
        SymbolKind::Functor => "functor",
        SymbolKind::Mode => "mode",
        SymbolKind::Attribute => "attribute",
        SymbolKind::Structure => "structure",
        SymbolKind::Selector => "selector",
        SymbolKind::Registration => "registration",
        SymbolKind::Theorem => "theorem",
        SymbolKind::Lemma => "lemma",
        SymbolKind::Algorithm => "algorithm",
        SymbolKind::Scheme => "scheme",
        SymbolKind::Template => "template",
        SymbolKind::Synonym => "synonym",
        SymbolKind::Antonym => "antonym",
        SymbolKind::Redefinition => "redefinition",
        SymbolKind::Builtin => "builtin",
    }
}

fn visibility_name(visibility: Visibility) -> &'static str {
    match visibility {
        Visibility::Private => "private",
        Visibility::Public => "public",
    }
}

fn export_status_name(status: ExportStatus) -> &'static str {
    match status {
        ExportStatus::LocalOnly => "local_only",
        ExportStatus::Exported => "exported",
        ExportStatus::ReExported => "re_exported",
    }
}

fn lexical_summary_kind_name(kind: LexicalSummaryKind) -> &'static str {
    match kind {
        LexicalSummaryKind::Notation => "notation",
        LexicalSummaryKind::Selector => "selector",
        LexicalSummaryKind::Constructor => "constructor",
    }
}

fn label_kind_name(kind: LabelKind) -> &'static str {
    match kind {
        LabelKind::Theorem => "theorem",
        LabelKind::Definition => "definition",
        LabelKind::ProofStep => "proof_step",
        LabelKind::Registration => "registration",
    }
}

fn definition_kind_name(kind: DefinitionKind) -> &'static str {
    match kind {
        DefinitionKind::Predicate => "predicate",
        DefinitionKind::Functor => "functor",
        DefinitionKind::Mode => "mode",
        DefinitionKind::Attribute => "attribute",
        DefinitionKind::Structure => "structure",
        DefinitionKind::Registration => "registration",
        DefinitionKind::Theorem => "theorem",
        DefinitionKind::Lemma => "lemma",
        DefinitionKind::Algorithm => "algorithm",
        DefinitionKind::Scheme => "scheme",
        DefinitionKind::Template => "template",
        DefinitionKind::Synonym => "synonym",
        DefinitionKind::Antonym => "antonym",
        DefinitionKind::Redefinition => "redefinition",
        DefinitionKind::Selector => "selector",
    }
}

fn declaration_conflict_class_name(class: &DeclarationConflictClass) -> &'static str {
    match class {
        DeclarationConflictClass::DuplicateSpelling => "duplicate_spelling",
        DeclarationConflictClass::IllegalOverloadGroup => "illegal_overload_group",
        DeclarationConflictClass::RecoveredShell => "recovered_shell",
    }
}

fn registration_kind_name(kind: RegistrationKind) -> &'static str {
    match kind {
        RegistrationKind::Cluster => "cluster",
        RegistrationKind::Identify => "identify",
        RegistrationKind::Reduction => "reduction",
        RegistrationKind::Property => "property",
    }
}

fn relation_kind_name(kind: RelationKind) -> &'static str {
    match kind {
        RelationKind::Synonym => "synonym",
        RelationKind::Antonym => "antonym",
        RelationKind::Redefinition => "redefinition",
    }
}

fn namespace_node_kind_name(kind: NamespaceNodeKind) -> &'static str {
    match kind {
        NamespaceNodeKind::Module => "module",
        NamespaceNodeKind::Alias => "alias",
        NamespaceNodeKind::Segment => "segment",
        NamespaceNodeKind::BuiltinPrelude => "builtin_prelude",
        NamespaceNodeKind::Unresolved => "unresolved",
    }
}

fn namespace_edge_kind_name(kind: NamespaceEdgeKind) -> &'static str {
    match kind {
        NamespaceEdgeKind::Import => "import",
        NamespaceEdgeKind::Export => "export",
        NamespaceEdgeKind::ReExport => "re_export",
        NamespaceEdgeKind::Segment => "segment",
        NamespaceEdgeKind::BuiltinPrelude => "builtin_prelude",
        NamespaceEdgeKind::Unresolved => "unresolved",
    }
}

fn declaration_dependency_kind_name(kind: DeclarationDependencyKind) -> &'static str {
    match kind {
        DeclarationDependencyKind::Import => "import",
        DeclarationDependencyKind::ReExport => "re_export",
        DeclarationDependencyKind::SignatureMention => "signature_mention",
        DeclarationDependencyKind::SynonymTarget => "synonym_target",
        DeclarationDependencyKind::AntonymTarget => "antonym_target",
        DeclarationDependencyKind::RedefinitionTarget => "redefinition_target",
        DeclarationDependencyKind::RegistrationMention => "registration_mention",
        DeclarationDependencyKind::LabelCitation => "label_citation",
    }
}

fn recovery_state_name(recovery: RecoveryState) -> &'static str {
    match recovery {
        RecoveryState::Normal => "normal",
        RecoveryState::Recovered => "recovered",
    }
}
