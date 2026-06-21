use super::*;
use mizar_syntax::{
    SurfaceFormulaConnective, SurfaceFormulaConstant, SurfaceFormulaPrefixOperator,
    SurfaceNodeKind, SurfaceOperatorAssociativity, SurfaceQuantifierKind, SurfaceTokenKind,
    SyntaxRecoveryKind,
};
use std::fmt::Write as _;

pub(super) fn resolved_ast_snapshot_text(ast: &ResolvedAst) -> String {
    let mut output = String::from("resolved-ast-debug-v1\n");
    output.push_str("module: ");
    write_module_id(&mut output, &ast.module_id);
    output.push('\n');
    let _ = writeln!(output, "root: node#{}", ast.nodes.root().index());
    output.push_str("nodes:\n");
    if ast.nodes.is_empty() {
        output.push_str("  <none>\n");
    } else {
        for (id, node) in ast.nodes.iter() {
            write_resolved_node_snapshot(&mut output, id, node);
        }
    }
    output.push_str("name_refs:\n");
    if ast.name_refs.is_empty() {
        output.push_str("  <none>\n");
    } else {
        for (id, entry) in ast.name_refs.iter() {
            write_name_ref_snapshot(&mut output, id, entry);
        }
    }
    output.push_str("label_refs:\n");
    if ast.label_refs.is_empty() {
        output.push_str("  <none>\n");
    } else {
        for (id, entry) in ast.label_refs.iter() {
            write_label_ref_snapshot(&mut output, id, entry);
        }
    }
    output.push_str("imports:\n");
    let mut import_count = 0;
    for (id, import) in ast.imports.imports() {
        import_count += 1;
        write_import_snapshot(&mut output, id, import);
    }
    if import_count == 0 {
        output.push_str("  <none>\n");
    }
    output.push_str("exports:\n");
    let mut export_count = 0;
    for (id, export) in ast.imports.exports() {
        export_count += 1;
        write_export_snapshot(&mut output, id, export);
    }
    if export_count == 0 {
        output.push_str("  <none>\n");
    }
    output.push_str("canonical_import_modules:\n");
    let canonical_modules = ast.imports.canonical_import_modules();
    if canonical_modules.is_empty() {
        output.push_str("  <none>\n");
    } else {
        for module in canonical_modules {
            output.push_str("  module=");
            write_module_id(&mut output, module);
            output.push('\n');
        }
    }
    output
}

fn write_resolved_node_snapshot(output: &mut String, id: ResolvedNodeId, node: &ResolvedNode) {
    let _ = write!(output, "  node#{} kind=", id.index());
    write_surface_node_kind(output, node.kind());
    output.push_str(" children=");
    write_node_ids(output, node.children());
    let _ = write!(
        output,
        " recovery={} resolution={} ref=",
        recovery_state_name(node.recovery()),
        node_resolution_state_name(node.resolution())
    );
    match node.reference_key() {
        Some(key) => write_node_reference_key(output, key),
        None => output.push_str("<none>"),
    }
    output.push_str(" origin=");
    write_origin(output, node.origin());
    output.push('\n');
}

fn write_name_ref_snapshot(output: &mut String, id: NameRefId, entry: &NameRefEntry) {
    let _ = write!(output, "  name#{} site=", id.index());
    write_reference_site(output, entry.site());
    let _ = write!(
        output,
        " recovery={} origin=",
        recovery_state_name(entry.recovery())
    );
    write_origin(output, entry.origin());
    output.push('\n');
    write_name_resolution_snapshot(output, entry.resolution());
}

fn write_label_ref_snapshot(output: &mut String, id: LabelRefId, entry: &LabelRefEntry) {
    let _ = write!(output, "  label#{} site=", id.index());
    write_reference_site(output, entry.site());
    let _ = write!(
        output,
        " recovery={} origin=",
        recovery_state_name(entry.recovery())
    );
    write_origin(output, entry.origin());
    output.push('\n');
    write_label_resolution_snapshot(output, entry.resolution());
}

fn write_import_snapshot(output: &mut String, id: ResolvedImportId, import: &ResolvedImport) {
    let _ = write!(
        output,
        "  import#{} owner=node#{} range=",
        id.index(),
        import.owner().index()
    );
    write_range(output, import.range());
    output.push_str(" spelling=\"");
    write_escaped(output, import.source_spelling());
    output.push('"');
    output.push_str(" alias=");
    match import.alias() {
        Some(alias) => {
            output.push('"');
            write_escaped(output, alias);
            output.push('"');
        }
        None => output.push_str("<none>"),
    }
    let _ = write!(
        output,
        " recovery={} origin=",
        recovery_state_name(import.recovery())
    );
    write_origin(output, import.origin());
    output.push('\n');
    output.push_str("    resolution=");
    write_import_resolution(output, import.resolution());
    output.push('\n');
}

fn write_export_snapshot(output: &mut String, id: ResolvedExportId, export: &ResolvedExport) {
    let _ = write!(
        output,
        "  export#{} owner=node#{} range=",
        id.index(),
        export.owner().index()
    );
    write_range(output, export.range());
    output.push_str(" spelling=\"");
    write_escaped(output, export.source_spelling());
    output.push('"');
    let _ = write!(
        output,
        " recovery={} origin=",
        recovery_state_name(export.recovery())
    );
    write_origin(output, export.origin());
    output.push('\n');
    output.push_str("    target=");
    write_export_target(output, export.target());
    output.push('\n');
}

fn write_name_resolution_snapshot(output: &mut String, resolution: &NameResolution) {
    match resolution {
        NameResolution::Resolved(symbol_ref) => {
            output.push_str("    resolution=resolved symbol=");
            write_symbol_id(output, symbol_ref.symbol());
            output.push_str(" range=");
            write_range(output, symbol_ref.range());
            output.push_str(" import=");
            match symbol_ref.import() {
                Some(import) => {
                    let _ = write!(output, "import#{}", import.index());
                }
                None => output.push_str("<none>"),
            }
            output.push_str(" spelling=");
            match symbol_ref.spelling() {
                Some(spelling) => {
                    output.push('"');
                    write_escaped(output, spelling);
                    output.push('"');
                }
                None => output.push_str("<none>"),
            }
            output.push('\n');
        }
        NameResolution::ResolvedBuiltin(builtin_ref) => {
            output.push_str("    resolution=builtin builtin=\"");
            write_escaped(output, builtin_ref.builtin().as_str());
            output.push_str("\" range=");
            write_range(output, builtin_ref.range());
            output.push_str(" spelling=\"");
            write_escaped(output, builtin_ref.spelling());
            output.push_str("\"\n");
        }
        NameResolution::DeferredSelector(selector_ref) => {
            let _ = write!(
                output,
                "    resolution=deferred_selector base=node#{} member=\"",
                selector_ref.base().index()
            );
            write_escaped(output, selector_ref.member());
            output.push_str("\" range=");
            write_range(output, selector_ref.range());
            output.push('\n');
        }
        NameResolution::Ambiguous(ambiguous_ref) => {
            output.push_str("    resolution=ambiguous spelling=\"");
            write_escaped(output, ambiguous_ref.spelling());
            output.push_str("\" range=");
            write_range(output, ambiguous_ref.range());
            output.push_str(" candidates=[");
            for (index, candidate) in ambiguous_ref.candidates().iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                write_symbol_id(output, candidate.symbol());
                output.push('@');
                write_range(output, candidate.range());
            }
            output.push_str("]\n");
        }
        NameResolution::Unresolved(unresolved_ref) => {
            output.push_str("    resolution=unresolved spelling=\"");
            write_escaped(output, unresolved_ref.spelling());
            output.push_str("\" lookup=");
            output.push_str(name_lookup_class_name(unresolved_ref.lookup()));
            output.push_str(" range=");
            write_range(output, unresolved_ref.range());
            output.push('\n');
        }
    }
}

fn write_label_resolution_snapshot(output: &mut String, resolution: &LabelResolution) {
    match resolution {
        LabelResolution::Resolved(label_ref) => {
            output.push_str("    resolution=resolved origin=\"");
            write_escaped(output, label_ref.origin().as_str());
            output.push_str("\" kind=");
            output.push_str(label_kind_name(label_ref.kind()));
            output.push_str(" range=");
            write_range(output, label_ref.range());
            output.push('\n');
        }
        LabelResolution::Ambiguous(ambiguous_ref) => {
            output.push_str("    resolution=ambiguous spelling=\"");
            write_escaped(output, ambiguous_ref.spelling());
            output.push_str("\" range=");
            write_range(output, ambiguous_ref.range());
            output.push_str(" candidates=[");
            for (index, candidate) in ambiguous_ref.candidates().iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                output.push('"');
                write_escaped(output, candidate.origin().as_str());
                output.push('"');
                output.push(':');
                output.push_str(label_kind_name(candidate.kind()));
                output.push('@');
                write_range(output, candidate.range());
            }
            output.push_str("]\n");
        }
        LabelResolution::Unresolved(unresolved_ref) => {
            output.push_str("    resolution=unresolved spelling=\"");
            write_escaped(output, unresolved_ref.spelling());
            output.push_str("\" expectation=");
            output.push_str(label_expectation_name(unresolved_ref.expectation()));
            output.push_str(" range=");
            write_range(output, unresolved_ref.range());
            output.push('\n');
        }
    }
}

fn write_import_resolution(output: &mut String, resolution: &ImportResolution) {
    match resolution {
        ImportResolution::Resolved(module) => {
            output.push_str("resolved module=");
            write_module_id(output, module);
        }
        ImportResolution::Unresolved(unresolved) => {
            output.push_str("unresolved spelling=\"");
            write_escaped(output, unresolved.spelling());
            output.push_str("\" class=");
            output.push_str(import_failure_class_name(unresolved.class()));
            output.push_str(" range=");
            write_range(output, unresolved.range());
        }
        ImportResolution::Ambiguous(ambiguous) => {
            output.push_str("ambiguous candidates=[");
            for (index, module) in ambiguous.candidates().iter().enumerate() {
                if index > 0 {
                    output.push_str(", ");
                }
                write_module_id(output, module);
            }
            output.push(']');
        }
    }
}

fn write_export_target(output: &mut String, target: &ExportTarget) {
    match target {
        ExportTarget::Module(module) => {
            output.push_str("module=");
            write_module_id(output, module);
        }
        ExportTarget::ImportAlias { alias, module } => {
            output.push_str("import_alias alias=\"");
            write_escaped(output, alias);
            output.push_str("\" module=");
            write_module_id(output, module);
        }
        ExportTarget::Symbol(symbol) => {
            output.push_str("symbol=");
            write_symbol_id(output, symbol);
        }
        ExportTarget::Unresolved(unresolved) => {
            output.push_str("unresolved spelling=\"");
            write_escaped(output, unresolved.spelling());
            output.push_str("\" class=");
            output.push_str(export_failure_class_name(unresolved.class()));
            output.push_str(" range=");
            write_range(output, unresolved.range());
        }
    }
}

fn write_reference_site(output: &mut String, site: &ReferenceSite) {
    let _ = write!(output, "{{node=node#{} range=", site.node().index());
    write_range(output, site.range());
    output.push_str(" spelling=\"");
    write_escaped(output, site.spelling());
    output.push_str("\"}");
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

fn write_node_reference_key(output: &mut String, key: NodeReferenceKey) {
    match key {
        NodeReferenceKey::Name(id) => {
            let _ = write!(output, "name#{}", id.index());
        }
        NodeReferenceKey::Label(id) => {
            let _ = write!(output, "label#{}", id.index());
        }
        NodeReferenceKey::Import(id) => {
            let _ = write!(output, "import#{}", id.index());
        }
        NodeReferenceKey::Export(id) => {
            let _ = write!(output, "export#{}", id.index());
        }
    }
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

fn write_node_ids(output: &mut String, ids: &[ResolvedNodeId]) {
    output.push('[');
    for (index, id) in ids.iter().enumerate() {
        if index > 0 {
            output.push_str(", ");
        }
        let _ = write!(output, "node#{}", id.index());
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

fn recovery_state_name(recovery: RecoveryState) -> &'static str {
    match recovery {
        RecoveryState::Normal => "normal",
        RecoveryState::Recovered => "recovered",
    }
}

fn node_resolution_state_name(resolution: NodeResolutionState) -> &'static str {
    match resolution {
        NodeResolutionState::NotApplicable => "not_applicable",
        NodeResolutionState::Resolved => "resolved",
        NodeResolutionState::Unresolved => "unresolved",
        NodeResolutionState::Ambiguous => "ambiguous",
        NodeResolutionState::Deferred => "deferred",
    }
}

fn name_lookup_class_name(class: NameLookupClass) -> &'static str {
    match class {
        NameLookupClass::Module => "module",
        NameLookupClass::Namespace => "namespace",
        NameLookupClass::Symbol => "symbol",
        NameLookupClass::Builtin => "builtin",
        NameLookupClass::Selector => "selector",
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

fn label_expectation_name(expectation: LabelExpectation) -> &'static str {
    match expectation {
        LabelExpectation::ProofOrTheorem => "proof_or_theorem",
        LabelExpectation::Theorem => "theorem",
        LabelExpectation::ProofStep => "proof_step",
        LabelExpectation::Definition => "definition",
        LabelExpectation::Registration => "registration",
    }
}

fn import_failure_class_name(class: ImportFailureClass) -> &'static str {
    match class {
        ImportFailureClass::ModuleNotFound => "module_not_found",
        ImportFailureClass::RelativePathEscapesPackage => "relative_path_escapes_package",
        ImportFailureClass::RecoveredSyntax => "recovered_syntax",
    }
}

fn export_failure_class_name(class: ExportFailureClass) -> &'static str {
    match class {
        ExportFailureClass::TargetNotFound => "target_not_found",
        ExportFailureClass::NotVisible => "not_visible",
        ExportFailureClass::RecoveredSyntax => "recovered_syntax",
    }
}

fn write_surface_node_kind(output: &mut String, kind: &SurfaceNodeKind) {
    match kind {
        SurfaceNodeKind::Root => output.push_str("Root"),
        SurfaceNodeKind::Token(token) => {
            output.push_str("Token kind=");
            output.push_str(surface_token_kind_name(token.kind));
            output.push_str(" text=\"");
            write_escaped(output, token.text.as_ref());
            output.push('"');
        }
        SurfaceNodeKind::InfixExpression(operator) => {
            output.push_str("InfixExpression spelling=\"");
            write_escaped(output, operator.spelling.as_ref());
            let _ = write!(
                output,
                "\" precedence={} associativity={}",
                operator.precedence,
                surface_operator_associativity_name(operator.associativity)
            );
        }
        SurfaceNodeKind::PrefixExpression(operator) => {
            output.push_str("PrefixExpression spelling=\"");
            write_escaped(output, operator.spelling.as_ref());
            let _ = write!(output, "\" precedence={}", operator.precedence);
        }
        SurfaceNodeKind::PostfixExpression(operator) => {
            output.push_str("PostfixExpression spelling=\"");
            write_escaped(output, operator.spelling.as_ref());
            let _ = write!(output, "\" precedence={}", operator.precedence);
        }
        SurfaceNodeKind::FormulaExpression => output.push_str("FormulaExpression"),
        SurfaceNodeKind::BuiltinPredicateApplication => {
            output.push_str("BuiltinPredicateApplication");
        }
        SurfaceNodeKind::IsAssertion => output.push_str("IsAssertion"),
        SurfaceNodeKind::AttributeTestChain => output.push_str("AttributeTestChain"),
        SurfaceNodeKind::PredicateApplication => output.push_str("PredicateApplication"),
        SurfaceNodeKind::PredicateSegment => output.push_str("PredicateSegment"),
        SurfaceNodeKind::PredicateHead => output.push_str("PredicateHead"),
        SurfaceNodeKind::InlinePredicateApplication => {
            output.push_str("InlinePredicateApplication");
        }
        SurfaceNodeKind::PrefixFormula(operator) => {
            let _ = write!(
                output,
                "PrefixFormula operator={}",
                surface_formula_prefix_operator_name(*operator)
            );
        }
        SurfaceNodeKind::BinaryFormula(operator) => {
            let _ = write!(
                output,
                "BinaryFormula connective={} repeated={}",
                surface_formula_connective_name(operator.connective),
                operator.repeated
            );
        }
        SurfaceNodeKind::ParenthesizedFormula => output.push_str("ParenthesizedFormula"),
        SurfaceNodeKind::QuantifiedFormula(quantifier) => {
            let _ = write!(
                output,
                "QuantifiedFormula quantifier={}",
                surface_quantifier_kind_name(*quantifier)
            );
        }
        SurfaceNodeKind::QuantifierVariableSegment => output.push_str("QuantifierVariableSegment"),
        SurfaceNodeKind::FormulaConstant(constant) => {
            let _ = write!(
                output,
                "FormulaConstant constant={}",
                surface_formula_constant_name(*constant)
            );
        }
        SurfaceNodeKind::ErrorRecovery(kind) => {
            let _ = write!(
                output,
                "ErrorRecovery kind={}",
                syntax_recovery_kind_name(*kind)
            );
        }
        SurfaceNodeKind::CompilationUnit => output.push_str("CompilationUnit"),
        SurfaceNodeKind::ItemList => output.push_str("ItemList"),
        SurfaceNodeKind::PlaceholderItem => output.push_str("PlaceholderItem"),
        SurfaceNodeKind::ImportItem => output.push_str("ImportItem"),
        SurfaceNodeKind::ImportAliasDecl => output.push_str("ImportAliasDecl"),
        SurfaceNodeKind::ModuleBranchImport => output.push_str("ModuleBranchImport"),
        SurfaceNodeKind::ExportItem => output.push_str("ExportItem"),
        SurfaceNodeKind::VisibilityMarker => output.push_str("VisibilityMarker"),
        SurfaceNodeKind::VisibleItem => output.push_str("VisibleItem"),
        SurfaceNodeKind::ReserveItem => output.push_str("ReserveItem"),
        SurfaceNodeKind::ReserveSegment => output.push_str("ReserveSegment"),
        SurfaceNodeKind::TypeExpression => output.push_str("TypeExpression"),
        SurfaceNodeKind::AttributeChain => output.push_str("AttributeChain"),
        SurfaceNodeKind::AttributeRef => output.push_str("AttributeRef"),
        SurfaceNodeKind::ParameterPrefix => output.push_str("ParameterPrefix"),
        SurfaceNodeKind::TypeHead => output.push_str("TypeHead"),
        SurfaceNodeKind::TypeArguments => output.push_str("TypeArguments"),
        SurfaceNodeKind::TemplateLoci => output.push_str("TemplateLoci"),
        SurfaceNodeKind::TemplateLocus => output.push_str("TemplateLocus"),
        SurfaceNodeKind::TemplateArguments => output.push_str("TemplateArguments"),
        SurfaceNodeKind::TemplateArgument => output.push_str("TemplateArgument"),
        SurfaceNodeKind::AlgorithmDefinition => output.push_str("AlgorithmDefinition"),
        SurfaceNodeKind::AlgorithmParameters => output.push_str("AlgorithmParameters"),
        SurfaceNodeKind::AlgorithmBody => output.push_str("AlgorithmBody"),
        SurfaceNodeKind::AlgorithmStatementList => output.push_str("AlgorithmStatementList"),
        SurfaceNodeKind::VariableDeclaration => output.push_str("VariableDeclaration"),
        SurfaceNodeKind::VariableBinding => output.push_str("VariableBinding"),
        SurfaceNodeKind::AssignmentStatement => output.push_str("AssignmentStatement"),
        SurfaceNodeKind::Lvalue => output.push_str("Lvalue"),
        SurfaceNodeKind::SnapshotStatement => output.push_str("SnapshotStatement"),
        SurfaceNodeKind::ReturnStatement => output.push_str("ReturnStatement"),
        SurfaceNodeKind::ClaimBlockItem => output.push_str("ClaimBlockItem"),
        SurfaceNodeKind::IfStatement => output.push_str("IfStatement"),
        SurfaceNodeKind::WhileStatement => output.push_str("WhileStatement"),
        SurfaceNodeKind::ForRangeStatement => output.push_str("ForRangeStatement"),
        SurfaceNodeKind::ForCollectionStatement => output.push_str("ForCollectionStatement"),
        SurfaceNodeKind::MatchStatement => output.push_str("MatchStatement"),
        SurfaceNodeKind::MatchCase => output.push_str("MatchCase"),
        SurfaceNodeKind::MatchEnding => output.push_str("MatchEnding"),
        SurfaceNodeKind::BreakStatement => output.push_str("BreakStatement"),
        SurfaceNodeKind::ContinueStatement => output.push_str("ContinueStatement"),
        SurfaceNodeKind::AlgorithmTerminationClause => {
            output.push_str("AlgorithmTerminationClause")
        }
        SurfaceNodeKind::AlgorithmRequiresClause => output.push_str("AlgorithmRequiresClause"),
        SurfaceNodeKind::AlgorithmEnsuresClause => output.push_str("AlgorithmEnsuresClause"),
        SurfaceNodeKind::AlgorithmDecreasingClause => output.push_str("AlgorithmDecreasingClause"),
        SurfaceNodeKind::LoopInvariantClause => output.push_str("LoopInvariantClause"),
        SurfaceNodeKind::LoopDecreasingClause => output.push_str("LoopDecreasingClause"),
        SurfaceNodeKind::AssertStatement => output.push_str("AssertStatement"),
        SurfaceNodeKind::TermList => output.push_str("TermList"),
        SurfaceNodeKind::Annotation => output.push_str("Annotation"),
        SurfaceNodeKind::LibraryAnnotation => output.push_str("LibraryAnnotation"),
        SurfaceNodeKind::AnnotationLabelList => output.push_str("AnnotationLabelList"),
        SurfaceNodeKind::AnnotationLabel => output.push_str("AnnotationLabel"),
        SurfaceNodeKind::AnnotationArgumentList => output.push_str("AnnotationArgumentList"),
        SurfaceNodeKind::AnnotationArgument => output.push_str("AnnotationArgument"),
        SurfaceNodeKind::ProofHintOptionList => output.push_str("ProofHintOptionList"),
        SurfaceNodeKind::ProofHintOption => output.push_str("ProofHintOption"),
        SurfaceNodeKind::StandaloneDiagnosticAnnotation => {
            output.push_str("StandaloneDiagnosticAnnotation");
        }
        SurfaceNodeKind::AnnotatedStatement => output.push_str("AnnotatedStatement"),
        SurfaceNodeKind::AnnotatedAlgorithmStatement => {
            output.push_str("AnnotatedAlgorithmStatement")
        }
        SurfaceNodeKind::AnnotatedDefinitionContent => {
            output.push_str("AnnotatedDefinitionContent")
        }
        SurfaceNodeKind::AnnotatedRegistrationContent => {
            output.push_str("AnnotatedRegistrationContent")
        }
        SurfaceNodeKind::TermPlaceholder => output.push_str("TermPlaceholder"),
        SurfaceNodeKind::TermExpression => output.push_str("TermExpression"),
        SurfaceNodeKind::TermReference => output.push_str("TermReference"),
        SurfaceNodeKind::NumeralTerm => output.push_str("NumeralTerm"),
        SurfaceNodeKind::ItTerm => output.push_str("ItTerm"),
        SurfaceNodeKind::ParenthesizedTerm => output.push_str("ParenthesizedTerm"),
        SurfaceNodeKind::ChoiceTerm => output.push_str("ChoiceTerm"),
        SurfaceNodeKind::ApplicationTerm => output.push_str("ApplicationTerm"),
        SurfaceNodeKind::StructureConstructor => output.push_str("StructureConstructor"),
        SurfaceNodeKind::FieldArgument => output.push_str("FieldArgument"),
        SurfaceNodeKind::SetEnumeration => output.push_str("SetEnumeration"),
        SurfaceNodeKind::SetComprehension => output.push_str("SetComprehension"),
        SurfaceNodeKind::ComprehensionVariableSegment => {
            output.push_str("ComprehensionVariableSegment")
        }
        SurfaceNodeKind::StatementItem => output.push_str("StatementItem"),
        SurfaceNodeKind::LetStatement => output.push_str("LetStatement"),
        SurfaceNodeKind::QualifiedVariableSegment => output.push_str("QualifiedVariableSegment"),
        SurfaceNodeKind::AssumptionStatement => output.push_str("AssumptionStatement"),
        SurfaceNodeKind::Proposition => output.push_str("Proposition"),
        SurfaceNodeKind::ConditionList => output.push_str("ConditionList"),
        SurfaceNodeKind::GivenStatement => output.push_str("GivenStatement"),
        SurfaceNodeKind::TakeStatement => output.push_str("TakeStatement"),
        SurfaceNodeKind::Witness => output.push_str("Witness"),
        SurfaceNodeKind::SetStatement => output.push_str("SetStatement"),
        SurfaceNodeKind::Equating => output.push_str("Equating"),
        SurfaceNodeKind::CompactStatement => output.push_str("CompactStatement"),
        SurfaceNodeKind::JustificationClause => output.push_str("JustificationClause"),
        SurfaceNodeKind::ReferenceList => output.push_str("ReferenceList"),
        SurfaceNodeKind::Reference => output.push_str("Reference"),
        SurfaceNodeKind::QualifiedReference => output.push_str("QualifiedReference"),
        SurfaceNodeKind::GroupedReference => output.push_str("GroupedReference"),
        SurfaceNodeKind::GroupedReferenceItem => output.push_str("GroupedReferenceItem"),
        SurfaceNodeKind::BulkReference => output.push_str("BulkReference"),
        SurfaceNodeKind::ComputationJustification => output.push_str("ComputationJustification"),
        SurfaceNodeKind::ComputationOption => output.push_str("ComputationOption"),
        SurfaceNodeKind::ConsiderStatement => output.push_str("ConsiderStatement"),
        SurfaceNodeKind::ReconsiderStatement => output.push_str("ReconsiderStatement"),
        SurfaceNodeKind::ReconsiderItem => output.push_str("ReconsiderItem"),
        SurfaceNodeKind::ConclusionStatement => output.push_str("ConclusionStatement"),
        SurfaceNodeKind::ThenStatement => output.push_str("ThenStatement"),
        SurfaceNodeKind::IterativeEqualityStatement => {
            output.push_str("IterativeEqualityStatement")
        }
        SurfaceNodeKind::IterativeEqualityStep => output.push_str("IterativeEqualityStep"),
        SurfaceNodeKind::NowStatement => output.push_str("NowStatement"),
        SurfaceNodeKind::HerebyStatement => output.push_str("HerebyStatement"),
        SurfaceNodeKind::CaseReasoningStatement => output.push_str("CaseReasoningStatement"),
        SurfaceNodeKind::CaseItem => output.push_str("CaseItem"),
        SurfaceNodeKind::SupposeItem => output.push_str("SupposeItem"),
        SurfaceNodeKind::InlineFunctorDefinition => output.push_str("InlineFunctorDefinition"),
        SurfaceNodeKind::InlinePredicateDefinition => output.push_str("InlinePredicateDefinition"),
        SurfaceNodeKind::TypedParameter => output.push_str("TypedParameter"),
        SurfaceNodeKind::TheoremItem => output.push_str("TheoremItem"),
        SurfaceNodeKind::LemmaItem => output.push_str("LemmaItem"),
        SurfaceNodeKind::ProofBlock => output.push_str("ProofBlock"),
        SurfaceNodeKind::DefinitionBlockItem => output.push_str("DefinitionBlockItem"),
        SurfaceNodeKind::DefinitionParameter => output.push_str("DefinitionParameter"),
        SurfaceNodeKind::TemplateParameter => output.push_str("TemplateParameter"),
        SurfaceNodeKind::AttributeDefinition => output.push_str("AttributeDefinition"),
        SurfaceNodeKind::AttributePattern => output.push_str("AttributePattern"),
        SurfaceNodeKind::FormulaDefiniens => output.push_str("FormulaDefiniens"),
        SurfaceNodeKind::FormulaCase => output.push_str("FormulaCase"),
        SurfaceNodeKind::CorrectnessCondition => output.push_str("CorrectnessCondition"),
        SurfaceNodeKind::PredicateDefinition => output.push_str("PredicateDefinition"),
        SurfaceNodeKind::PredicatePattern => output.push_str("PredicatePattern"),
        SurfaceNodeKind::FunctorDefinition => output.push_str("FunctorDefinition"),
        SurfaceNodeKind::FunctorPattern => output.push_str("FunctorPattern"),
        SurfaceNodeKind::TermDefiniens => output.push_str("TermDefiniens"),
        SurfaceNodeKind::TermCase => output.push_str("TermCase"),
        SurfaceNodeKind::ModeDefinition => output.push_str("ModeDefinition"),
        SurfaceNodeKind::ModePattern => output.push_str("ModePattern"),
        SurfaceNodeKind::ModeProperty => output.push_str("ModeProperty"),
        SurfaceNodeKind::AttributeRedefinition => output.push_str("AttributeRedefinition"),
        SurfaceNodeKind::PredicateRedefinition => output.push_str("PredicateRedefinition"),
        SurfaceNodeKind::FunctorRedefinition => output.push_str("FunctorRedefinition"),
        SurfaceNodeKind::CoherenceCondition => output.push_str("CoherenceCondition"),
        SurfaceNodeKind::NotationAlias => output.push_str("NotationAlias"),
        SurfaceNodeKind::NotationPattern => output.push_str("NotationPattern"),
        SurfaceNodeKind::PropertyClause => output.push_str("PropertyClause"),
        SurfaceNodeKind::StructureDefinition => output.push_str("StructureDefinition"),
        SurfaceNodeKind::StructurePattern => output.push_str("StructurePattern"),
        SurfaceNodeKind::StructureField => output.push_str("StructureField"),
        SurfaceNodeKind::StructureProperty => output.push_str("StructureProperty"),
        SurfaceNodeKind::InheritanceDefinition => output.push_str("InheritanceDefinition"),
        SurfaceNodeKind::InheritanceTarget => output.push_str("InheritanceTarget"),
        SurfaceNodeKind::FieldRedefinition => output.push_str("FieldRedefinition"),
        SurfaceNodeKind::PropertyRedefinition => output.push_str("PropertyRedefinition"),
        SurfaceNodeKind::RegistrationBlockItem => output.push_str("RegistrationBlockItem"),
        SurfaceNodeKind::RegistrationParameter => output.push_str("RegistrationParameter"),
        SurfaceNodeKind::ExistentialRegistration => output.push_str("ExistentialRegistration"),
        SurfaceNodeKind::ConditionalRegistration => output.push_str("ConditionalRegistration"),
        SurfaceNodeKind::FunctorialRegistration => output.push_str("FunctorialRegistration"),
        SurfaceNodeKind::ReductionRegistration => output.push_str("ReductionRegistration"),
        SurfaceNodeKind::SelectorAccess => output.push_str("SelectorAccess"),
        SurfaceNodeKind::StructureUpdate => output.push_str("StructureUpdate"),
        SurfaceNodeKind::FieldUpdate => output.push_str("FieldUpdate"),
        SurfaceNodeKind::QuaExpression => output.push_str("QuaExpression"),
        SurfaceNodeKind::ModulePath => output.push_str("ModulePath"),
        SurfaceNodeKind::NamespacePath => output.push_str("NamespacePath"),
        SurfaceNodeKind::QualifiedSymbol => output.push_str("QualifiedSymbol"),
        SurfaceNodeKind::PathSegment => output.push_str("PathSegment"),
        SurfaceNodeKind::RelativePrefix => output.push_str("RelativePrefix"),
        _ => output.push_str("UnknownSurfaceNodeKind"),
    }
}

fn surface_token_kind_name(kind: SurfaceTokenKind) -> &'static str {
    match kind {
        SurfaceTokenKind::Identifier => "Identifier",
        SurfaceTokenKind::ReservedWord => "ReservedWord",
        SurfaceTokenKind::ReservedSymbol => "ReservedSymbol",
        SurfaceTokenKind::Numeral => "Numeral",
        SurfaceTokenKind::LexemeRun => "LexemeRun",
        SurfaceTokenKind::UserSymbol => "UserSymbol",
        SurfaceTokenKind::AnnotationMarker => "AnnotationMarker",
        SurfaceTokenKind::StringLiteral => "StringLiteral",
        SurfaceTokenKind::ErrorRecovery => "ErrorRecovery",
        SurfaceTokenKind::Unknown => "Unknown",
        _ => "UnknownTokenKind",
    }
}

fn surface_operator_associativity_name(
    associativity: SurfaceOperatorAssociativity,
) -> &'static str {
    match associativity {
        SurfaceOperatorAssociativity::Left => "Left",
        SurfaceOperatorAssociativity::Right => "Right",
        SurfaceOperatorAssociativity::NonAssociative => "NonAssociative",
    }
}

fn surface_formula_prefix_operator_name(operator: SurfaceFormulaPrefixOperator) -> &'static str {
    match operator {
        SurfaceFormulaPrefixOperator::Not => "Not",
    }
}

fn surface_formula_connective_name(connective: SurfaceFormulaConnective) -> &'static str {
    match connective {
        SurfaceFormulaConnective::And => "And",
        SurfaceFormulaConnective::Or => "Or",
        SurfaceFormulaConnective::Implies => "Implies",
        SurfaceFormulaConnective::Iff => "Iff",
    }
}

fn surface_quantifier_kind_name(quantifier: SurfaceQuantifierKind) -> &'static str {
    match quantifier {
        SurfaceQuantifierKind::Universal => "Universal",
        SurfaceQuantifierKind::Existential => "Existential",
    }
}

fn surface_formula_constant_name(constant: SurfaceFormulaConstant) -> &'static str {
    match constant {
        SurfaceFormulaConstant::Thesis => "Thesis",
        SurfaceFormulaConstant::Contradiction => "Contradiction",
    }
}

fn syntax_recovery_kind_name(kind: SyntaxRecoveryKind) -> &'static str {
    match kind {
        SyntaxRecoveryKind::ErrorToken => "ErrorToken",
        SyntaxRecoveryKind::MissingEnd => "MissingEnd",
        SyntaxRecoveryKind::MissingStringLiteral => "MissingStringLiteral",
        SyntaxRecoveryKind::MissingItem => "MissingItem",
        SyntaxRecoveryKind::MissingTypeExpression => "MissingTypeExpression",
        SyntaxRecoveryKind::MissingTerm => "MissingTerm",
        SyntaxRecoveryKind::MissingFormula => "MissingFormula",
        SyntaxRecoveryKind::MissingStatement => "MissingStatement",
        SyntaxRecoveryKind::MissingProofStep => "MissingProofStep",
        SyntaxRecoveryKind::MissingAnnotationArgument => "MissingAnnotationArgument",
        SyntaxRecoveryKind::SkippedToken => "SkippedToken",
        SyntaxRecoveryKind::UnmatchedOpeningDelimiter => "UnmatchedOpeningDelimiter",
        SyntaxRecoveryKind::UnmatchedClosingDelimiter => "UnmatchedClosingDelimiter",
        SyntaxRecoveryKind::MalformedAnnotation => "MalformedAnnotation",
        _ => "UnknownRecoveryKind",
    }
}
