use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mizar_lexer::{SourceLineIndex, preprocess_source_for_lexing, scan_raw};
use std::hint::black_box;

fn lexer_pipeline(c: &mut Criterion) {
    let source = large_miz_like_source(4_096);
    let preprocessed = preprocess_source_for_lexing(&source);

    let mut group = c.benchmark_group("lexer_pipeline_large_miz_like");
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_with_input(
        BenchmarkId::new("preprocess_source_for_lexing", source.len()),
        &source,
        |b, source| {
            b.iter(|| preprocess_source_for_lexing(black_box(source)));
        },
    );

    group.bench_with_input(
        BenchmarkId::new("scan_raw", preprocessed.lexical_text.len()),
        &preprocessed.lexical_text,
        |b, source| {
            b.iter(|| scan_raw(black_box(source)).expect("benchmark input should raw-scan"));
        },
    );

    group.bench_with_input(
        BenchmarkId::new("source_line_index", source.len()),
        &source,
        |b, source| {
            b.iter(|| SourceLineIndex::new(black_box(source)));
        },
    );

    group.finish();
}

fn large_miz_like_source(items: usize) -> String {
    let mut source = String::with_capacity(items * 192);
    source.push_str("::: benchmark source for lexer pipeline\n");
    source.push_str("import std.algebra.group, std.logic.basic;\n\n");

    for index in 0..items {
        source.push_str("theorem th");
        source.push_str(&index.to_string());
        source.push_str(":\n");
        source.push_str("  for x,y being set holds x = x & y = y\n");
        source.push_str("proof\n");
        source.push_str("  let x,y be set;\n");
        source.push_str("  thus x = x;\n");
        source.push_str("  thus y = y;\n");
        if index % 8 == 0 {
            source.push_str("  :: inline comment preserving line shape\n");
        }
        if index % 32 == 0 {
            source.push_str("  ::= multi-line comment\n");
            source.push_str("      with lexical-looking text alpha:=beta\n");
            source.push_str("  =::\n");
        }
        source.push_str("end;\n\n");
    }

    source
}

criterion_group!(benches, lexer_pipeline);
criterion_main!(benches);
