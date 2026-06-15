use criterion::{BenchmarkId, Criterion, Throughput, criterion_group, criterion_main};
use mizar_lexer::{
    ExportRank, ExportedSymbolShape, LexicalSummaryFingerprint, ModuleId, ModuleLexicalSummary,
    ParserLexContext, ResolvedImport, SourceLineIndex, SymbolId, UserSymbolArity, UserSymbolKind,
    build_lexical_environment, build_scope_skeleton, disambiguate, preprocess_source_for_lexing,
    scan_raw,
};
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

fn active_user_symbol_lexicon(c: &mut Criterion) {
    let env = many_symbol_environment(4_096);
    let source = many_symbol_source(8_192);
    let raw = scan_raw(&source).expect("benchmark input should raw-scan");
    let scopes = build_scope_skeleton(&raw);
    let context = ParserLexContext::general();

    let mut group = c.benchmark_group("active_user_symbol_lexicon");
    group.throughput(Throughput::Bytes(source.len() as u64));

    group.bench_with_input(
        BenchmarkId::new("disambiguate_many_imported_symbols", source.len()),
        &raw,
        |b, raw| {
            b.iter(|| {
                disambiguate(
                    black_box(raw),
                    black_box(&env),
                    black_box(&context),
                    black_box(&scopes),
                )
            });
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

fn many_symbol_environment(symbols: usize) -> mizar_lexer::ActiveLexicalEnvironment {
    let mut exported_symbols = Vec::with_capacity(symbols + 3);
    exported_symbols.push(exported("+", "bench#plus", 0));
    exported_symbols.push(exported("+*", "bench#plus_star", 1));
    exported_symbols.push(exported("+*+", "bench#plus_star_plus", 2));
    for index in 0..symbols {
        exported_symbols.push(exported(
            &format!("sym{index:04}"),
            &format!("bench#sym{index:04}"),
            index as u32 + 3,
        ));
    }

    build_lexical_environment(
        &[ResolvedImport {
            module_id: ModuleId("bench.symbols".to_owned()),
        }],
        &[ModuleLexicalSummary {
            module_id: ModuleId("bench.symbols".to_owned()),
            exported_symbols,
            fingerprint: LexicalSummaryFingerprint(0x5eed),
        }],
    )
    .expect("benchmark environment should build")
}

fn exported(spelling: &str, symbol: &str, rank: u32) -> ExportedSymbolShape {
    ExportedSymbolShape {
        spelling: spelling.to_owned(),
        symbol_id: SymbolId(symbol.to_owned()),
        source_module: ModuleId("bench.symbols".to_owned()),
        export_rank: ExportRank(rank),
        kind: UserSymbolKind::Functor,
        arity: UserSymbolArity::exact(2),
        operator: None,
    }
}

fn many_symbol_source(items: usize) -> String {
    let mut source = String::with_capacity(items * 18);
    for index in 0..items {
        source.push_str("sym");
        source.push_str(&format!("{:04}", index % 4_096));
        source.push_str("+*+x ");
    }
    source
}

criterion_group!(benches, lexer_pipeline, active_user_symbol_lexicon);
criterion_main!(benches);
