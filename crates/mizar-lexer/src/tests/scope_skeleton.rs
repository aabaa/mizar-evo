use super::common::*;

#[test]
fn scope_skeleton_handles_empty_stream() {
    let raw = scan_raw("").expect("empty input should raw scan");
    let skeleton = build_scope_skeleton(&raw);

    assert!(skeleton.frames.is_empty());
    assert!(skeleton.diagnostics.is_empty());
}

#[test]
fn scope_skeleton_records_simple_and_comma_separated_let_binders() {
    let source = "let x, y be set;";
    let raw = scan_raw(source).expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);

    assert_eq!(skeleton.frames.len(), 1);
    assert_eq!(skeleton.frames[0].range, SourceSpan { start: 0, end: 16 });
    assert_eq!(
        skeleton.frames[0]
            .bindings
            .iter()
            .map(|binding| binding.spelling.as_str())
            .collect::<Vec<_>>(),
        vec!["x", "y"]
    );
    assert!(skeleton.binding_overrides_symbol("x", 6));
    assert!(skeleton.binding_overrides_symbol("y", 9));
    assert!(!skeleton.binding_overrides_symbol("x", 4));
    assert!(!skeleton.binding_overrides_symbol("z", 6));
    assert!(skeleton.diagnostics.is_empty());
}

#[test]
fn scope_skeleton_records_supported_for_reserve_and_given_binders() {
    let source = "reserve A, B for set;\ngiven c being object;";
    let raw = scan_raw(source).expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);

    assert_eq!(skeleton.frames.len(), 2);
    assert_eq!(
        skeleton.frames[0]
            .bindings
            .iter()
            .map(|binding| binding.spelling.as_str())
            .collect::<Vec<_>>(),
        vec!["A", "B"]
    );
    assert_eq!(
        skeleton.frames[1]
            .bindings
            .iter()
            .map(|binding| binding.spelling.as_str())
            .collect::<Vec<_>>(),
        vec!["c"]
    );
    assert!(skeleton.binding_overrides_symbol("A", source.len() - 1));
    assert!(skeleton.binding_overrides_symbol("c", source.len() - 1));
    assert!(skeleton.diagnostics.is_empty());
}

#[test]
fn scope_skeleton_limits_for_and_given_binders_to_statement_ranges() {
    let source = "for x holds thesis;\nx;\ngiven y being object;\ny;";
    let raw = scan_raw(source).expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);

    assert_eq!(skeleton.frames.len(), 2);
    assert!(skeleton.binding_overrides_symbol("x", 6));
    assert!(!skeleton.binding_overrides_symbol("x", 21));
    assert!(skeleton.binding_overrides_symbol("y", 41));
    assert!(!skeleton.binding_overrides_symbol("y", source.len() - 1));
}

#[test]
fn scope_skeleton_separates_let_reserve_and_statement_lifetimes() {
    let source = "\
reserve R for set;
definition
let x be set;
now
let y be set;
for z holds y = z;
y;
end;
y;
end;
x;
R;";
    let raw = scan_raw(source).expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);

    let r_declaration = nth_index(source, "R", 0);
    let r_after_definition = nth_index(source, "R", 1);
    let x_inside_definition = nth_index(source, "x", 0) + 1;
    let x_after_definition = nth_index(source, "x", 1);
    let y_inside_now = nth_index(source, "y", 1);
    let y_after_now = nth_index(source, "y", 3);
    let z_inside_for = nth_index(source, "z", 1);
    let y_before_for = nth_index(source, "y", 0);

    assert!(!skeleton.binding_overrides_symbol("R", r_declaration));
    assert!(skeleton.binding_overrides_symbol("R", r_after_definition));
    assert!(skeleton.binding_overrides_symbol("x", x_inside_definition));
    assert!(!skeleton.binding_overrides_symbol("x", x_after_definition));
    assert!(skeleton.binding_overrides_symbol("y", y_inside_now));
    assert!(!skeleton.binding_overrides_symbol("y", y_after_now));
    assert!(skeleton.binding_overrides_symbol("z", z_inside_for));
    assert!(!skeleton.binding_overrides_symbol("z", y_before_for));
    assert!(skeleton.diagnostics.is_empty());
}

#[test]
fn scope_skeleton_under_approximates_block_local_reserve() {
    let source = "definition\nreserve R for set;\nR;\nend;\nR;";
    let raw = scan_raw(source).expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);

    assert!(!skeleton.binding_overrides_symbol("R", nth_index(source, "R", 1)));
    assert!(!skeleton.binding_overrides_symbol("R", nth_index(source, "R", 2)));
    assert_eq!(
        skeleton
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![ScopeSkeletonDiagnosticCode::UnsupportedBinderShape]
    );
}

#[test]
fn scope_skeleton_pairs_nested_block_ranges() {
    let source = "definition\nlet x be set;\nproof\nnow\nlet y be set;\nend;\nend;\nend;";
    let raw = scan_raw(source).expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);

    assert_eq!(skeleton.frames.len(), 3);
    assert_eq!(
        skeleton
            .frames
            .iter()
            .map(|frame| frame.range)
            .collect::<Vec<_>>(),
        vec![
            SourceSpan { start: 0, end: 62 },
            SourceSpan { start: 25, end: 57 },
            SourceSpan { start: 31, end: 52 },
        ]
    );
    assert!(skeleton.binding_overrides_symbol("x", 25));
    assert!(skeleton.binding_overrides_symbol("x", 61));
    assert!(skeleton.binding_overrides_symbol("y", 51));
    assert!(!skeleton.binding_overrides_symbol("y", 52));
    assert!(skeleton.diagnostics.is_empty());
}

#[test]
fn scope_skeleton_records_proof_case_suppose_and_algorithm_shapes() {
    let source = "\
definition
proof
given g being object;
consider c being object such that c = c;
set s = c;
reconsider rc = c as object;
take tk = c;
deffunc F(object) = c;
defpred P[object] means c = c;
case
let k be set;
end;
suppose c = c;
let sp be set;
end;
end;
end;
algorithm
do
var a, b = (c, d);
const n = 1;
ghost var gv;
ghost const gc = 2;
for i = 0 to 2 do
var inner;
end;
for item in Items processed Seen do
var consumed;
end;
end;
end;";
    let raw = scan_raw(source).expect("source should raw scan");
    let skeleton = build_scope_skeleton(&raw);

    assert_eq!(
        skeleton
            .blocks
            .iter()
            .map(|block| block.kind)
            .collect::<Vec<_>>(),
        vec![
            LexicalBlockKind::Definition,
            LexicalBlockKind::Proof,
            LexicalBlockKind::Case,
            LexicalBlockKind::Suppose,
            LexicalBlockKind::Algorithm,
            LexicalBlockKind::Do,
            LexicalBlockKind::Do,
            LexicalBlockKind::Do,
        ]
    );
    assert!(skeleton.binding_overrides_symbol("g", nth_index(source, "object", 0)));
    assert!(!skeleton.binding_overrides_symbol("g", nth_index(source, "consider", 0)));
    assert!(skeleton.binding_overrides_symbol("c", nth_index(source, "deffunc", 0)));
    assert!(skeleton.binding_overrides_symbol("F", nth_index(source, "defpred", 0)));
    assert!(skeleton.binding_overrides_symbol("a", nth_index(source, "const", 0)));
    assert!(skeleton.binding_overrides_symbol("gv", nth_index(source, "for i", 0)));
    assert!(skeleton.binding_overrides_symbol("i", nth_index(source, "inner", 0)));
    assert!(!skeleton.binding_overrides_symbol("i", nth_index(source, "for item", 0)));
    assert!(skeleton.binding_overrides_symbol("Seen", nth_index(source, "consumed", 0)));
    assert!(!skeleton.binding_overrides_symbol("Seen", source.len()));
    assert!(skeleton.diagnostics.is_empty());
}

#[test]
fn scope_skeleton_under_approximates_malformed_binders() {
    let raw = scan_raw("let , x be set;\nfor + y holds thesis;").expect("source should scan");
    let skeleton = build_scope_skeleton(&raw);

    assert!(skeleton.frames.is_empty());
    assert_eq!(
        skeleton
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            ScopeSkeletonDiagnosticCode::MalformedBinderList,
            ScopeSkeletonDiagnosticCode::UnsupportedBinderShape,
        ]
    );
}

#[test]
fn scope_skeleton_reports_recoverable_block_diagnostics_deterministically() {
    let raw = scan_raw("end;\ndefinition\nlet x be set;").expect("source should scan");
    let first = build_scope_skeleton(&raw);
    let second = build_scope_skeleton(&raw);

    assert_eq!(first, second);
    assert_eq!(
        first
            .diagnostics
            .iter()
            .map(|diagnostic| diagnostic.code)
            .collect::<Vec<_>>(),
        vec![
            ScopeSkeletonDiagnosticCode::UnmatchedEnd,
            ScopeSkeletonDiagnosticCode::MissingEnd,
        ]
    );
    assert!(first.binding_overrides_symbol("x", 27));
}
