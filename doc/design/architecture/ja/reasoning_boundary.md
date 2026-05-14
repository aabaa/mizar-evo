# Architecture: Reasoning Boundary

> Canonical language: English. English canonical version: [../en/reasoning_boundary.md](../en/reasoning_boundary.md).

## 目的

Mizar Evo において、Mizar 側、外部 ATP 側、kernel 側がそれぞれどの推論を担当するかを定義する。
この文書は pipeline Phase 12-14 を扱う。

## 対象フェーズ

| Phase | Responsibility |
|---|---|
| 12. Pre-ATP Discharge | Mizar 側で閉じられる obligation を処理する |
| 13. ATP Translation / Dispatch | open VC を ATP problem に変換し、外部 prover に渡す |
| 14. Kernel Certificate Check | ATP certificate を独立検証し、proof status を確定する |

## Mizar 側の責務

- 型検査、subtyping、coercion、sethood 判定
- cluster / registration resolution
- overload resolution
- definitional expansion の境界管理
- surface syntax から core IR への elaboration
- trivial obligation や computation で閉じられる VC の処理

## ATP 側に委譲する責務

- `by` justification に対応する一階述語論理の探索
- 等式推論
- cited premises と local hypotheses に基づく goal の証明探索
- solver-specific proof certificate の生成

## Kernel 側の責務

- ATP certificate / witness を検証する
- ATP result を盲目的に信頼しない
- de Bruijn criterion に沿って trusted boundary を小さく保つ

## 採用方針

型・名前・registration・overload は Mizar 側で確定し、論理探索は ATP に委譲する hybrid approach を採用する。
ATP の成功は kernel certificate check を通るまで受理しない。
