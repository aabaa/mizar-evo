use super::common::*;

#[test]
fn scans_empty_import_prelude() {
    let raw = scan_raw("definition\nend;").expect("source should raw scan");
    let prelude = scan_import_prelude(&raw);

    assert!(prelude.imports.is_empty());
    assert_eq!(prelude.end, 0);
    assert!(prelude.diagnostics.is_empty());
}

#[test]
fn scans_imports_aliases_and_relative_paths_from_raw_runs() {
    let raw = scan_raw("import std.algebra.group, ..common as C, .utils;")
        .expect("source should raw scan");
    let prelude = scan_import_prelude(&raw);

    let paths = prelude
        .imports
        .iter()
        .map(|import| import.path.spelling.as_str())
        .collect::<Vec<_>>();
    assert_eq!(paths, vec!["std.algebra.group", "..common", ".utils"]);
    assert_eq!(prelude.imports[1].alias.as_ref().unwrap().spelling, "C");
    assert_eq!(
        prelude.imports[1].path.relative,
        Some(RawModuleRelativePrefix::Parent)
    );
    assert_eq!(
        prelude.imports[2].path.relative,
        Some(RawModuleRelativePrefix::Current)
    );
    assert_eq!(
        prelude.end,
        "import std.algebra.group, ..common as C, .utils;".len()
    );
    assert!(prelude.diagnostics.is_empty());
}

#[test]
fn scans_contiguous_import_statements() {
    let source = "\
import std.algebra.group;
import std.topology.metric_space as Metric;
import pkg.mathcomp_mizar.algebra.ring;";
    let raw = scan_raw(source).expect("source should raw scan");
    let prelude = scan_import_prelude(&raw);

    let paths = prelude
        .imports
        .iter()
        .map(|import| import.path.spelling.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec![
            "std.algebra.group",
            "std.topology.metric_space",
            "pkg.mathcomp_mizar.algebra.ring"
        ]
    );
    assert_eq!(
        prelude.imports[1].alias.as_ref().unwrap().spelling,
        "Metric"
    );
    assert_eq!(prelude.end, source.len());
    assert!(prelude.diagnostics.is_empty());
}

#[test]
fn scans_branch_import_paths() {
    let source = "import algebra.linear.{eigen_value, jordan};";
    let raw = scan_raw(source).expect("source should raw scan");
    let prelude = scan_import_prelude(&raw);

    let paths = prelude
        .imports
        .iter()
        .map(|import| import.path.spelling.as_str())
        .collect::<Vec<_>>();
    assert_eq!(
        paths,
        vec!["algebra.linear.eigen_value", "algebra.linear.jordan"]
    );
    assert_eq!(
        prelude.imports[1].path.source_segments,
        vec![
            SourceSpan { start: 7, end: 21 },
            SourceSpan { start: 36, end: 42 },
        ]
    );
    assert_eq!(prelude.end, source.len());
    assert!(prelude.diagnostics.is_empty());
}

#[test]
fn stops_at_first_non_import_top_level_text() {
    let raw =
        scan_raw("import std.core;\ndefinition\nimport dev.late;").expect("source should raw scan");
    let prelude = scan_import_prelude(&raw);

    assert_eq!(prelude.imports.len(), 1);
    assert_eq!(prelude.imports[0].path.spelling, "std.core");
    assert_eq!(prelude.end, "import std.core;".len());
    assert!(prelude.diagnostics.is_empty());
}

#[test]
fn recovers_malformed_imports_with_diagnostics() {
    let raw = scan_raw("import std., pkg.math as ;").expect("source should raw scan");
    let prelude = scan_import_prelude(&raw);

    let paths = prelude
        .imports
        .iter()
        .map(|import| import.path.spelling.as_str())
        .collect::<Vec<_>>();
    assert_eq!(paths, vec!["std.", "pkg.math"]);
    assert_eq!(
        prelude
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            ImportPrescanDiagnosticCode::EmptyModulePathComponent,
            ImportPrescanDiagnosticCode::MissingAlias,
        ]
    );
}

#[test]
fn comma_separated_import_stub_spans_cover_each_declaration() {
    let raw = scan_raw("import std.core, pkg.math;").expect("source should raw scan");
    let prelude = scan_import_prelude(&raw);

    assert_eq!(prelude.imports[0].span, SourceSpan { start: 7, end: 15 });
    assert_eq!(prelude.imports[1].span, SourceSpan { start: 17, end: 25 });
}

#[test]
fn missing_semicolon_does_not_consume_top_level_terminator() {
    let raw = scan_raw("import std.core\ndefinition\nend;").expect("source should raw scan");
    let prelude = scan_import_prelude(&raw);

    assert_eq!(prelude.imports.len(), 1);
    assert_eq!(prelude.end, "import std.core".len());
    assert_eq!(
        prelude.diagnostics[0].code,
        ImportPrescanDiagnosticCode::MissingSemicolon
    );
}
