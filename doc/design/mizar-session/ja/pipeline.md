# 共通パイプラインラベル

正本: [English](../en/pipeline.md)。

`src/pipeline.rs` は Build task graph が再公開する `PipelinePhase` を所有する。
非網羅 enum の順序、`as_str()` の identity 表記、`as_cli_str()` の snake-case
表記を維持する。Cache/IR の文字列ラベルは任意入力を受け続け、
`From<PipelinePhase>` は identity 表記へ変換する。診断・テスト期待値の語彙は別に保つ。

`module_work_unit_label(package, module)` は既存の Rust-debug 文字列ペアを
コロンで連結し、引用符とエスケープを維持する。Build の work unit や専用 task
identity は構築しない。Session は scheduling や phase semantics を所有しない。
互換性の固定値テストは `crates/mizar-session/tests/common_utilities.rs` に置く。
