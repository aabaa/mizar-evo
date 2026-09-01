# 監査: 意味論ブリッジ・フロントエンドギャップ台帳（2026年9月・監査1）

> 正典は英語版
> [../en/semantic_bridge_frontend_gaps.md](../en/semantic_bridge_frontend_gaps.md)
> です。本書はポインタのみです（2026年9月のステータス文書言語方針による）。

- 概要: 監査1コーパス120件の実フロントエンド検証で特定した8種のギャップ
  （G1 ローカル記法シンボルの使用箇所、G2 記号スペル字句、G3 `then` 連結、
  G4 synonym/antonym、G5 `qua`+構造体型での**パーサpanic**、G6 従属ローカル
  mode 使用、G7 空正当化の非一貫性、G9 既活性スペルのパターン位置）と、
  それにブロックされる29ソースの台帳。
- 機械可読版: `tests/coverage/audit1_frontend_gaps.tsv`
