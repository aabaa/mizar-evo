# mizar-syntax: Surface AST

> 正本は英語です。英語版: [../en/ast.md](../en/ast.md)。

状態: planned。

## 目的

このモジュールは、`mizar-parser` が生成する、ソースの形を保った `SurfaceAst` を定義する。

## 責務

- `SurfaceAst`、`SurfaceNode`、構文ノード ID、arena の所有関係を定義する。
- ソース順、ソース範囲、回復ノードを保持する。
- モジュール、項目、項、論理式、文、証明、アルゴリズム、アノテーションを表現する。
- 解決済みシンボル ID、推論済み型、オーバーロード勝者、cluster fact、証明義務を持たない。

