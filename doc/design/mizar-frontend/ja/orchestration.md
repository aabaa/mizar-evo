# mizar-frontend: Orchestration

> 正本は英語です。英語版: [../en/orchestration.md](../en/orchestration.md)。

状態: planned。

## 目的

このモジュールは、`FrontendOutput` を生成するフェーズ 1-3 の coordinator を定義する。

## 責務

- ソース読み込み、字句解析器の前処理ヘルパー、インポート事前走査、アクティブ字句環境構築、トークン曖昧性解消、パーサー呼び出しを統制する。
- `mizar-parser` を呼び出して `mizar-syntax::SurfaceAst` を生成する。
- 字句診断と構文診断を決定的な順序で統合する。
- AST ノード定義や文法ロジックを所有せずに `FrontendOutput` を公開する。

