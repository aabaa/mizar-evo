# Architecture: ATP Interface Protocol

> Canonical language: English. English canonical version: [../en/atp_interface_protocol.md](../en/atp_interface_protocol.md).

## 目的

`VcIr` / `AtpProblem` を外部 ATP が理解できる concrete problem format へ変換する方針を定義する。
この文書は pipeline Phase 13 のうち、problem translation / encoding を扱う。

## 対応フォーマット

| Format | Target ATPs | Use Case |
|---|---|---|
| TPTP FOF/TFF | Vampire, E | 一階述語論理、等式推論、大きめの axiom set |
| SMT-LIB 2 | CVC5, Z3 | 算術、SMT theory、等式と算術の混合 |

## エンコーディング方針

- Mizar の soft type は prover format に応じて sort または guard predicate として表現する。
- functor / predicate / mode は backend-safe な symbol name に正規化する。
- property は backend が native に扱える場合は native support を使い、そうでなければ axiom として渡す。
- `by` citation と local hypotheses から premise set を構成する。

## `AtpProblem`

`AtpProblem` は backend-neutral な prover input であり、まだ TPTP / SMT-LIB text ではない。
1 つの `VcIr` から backend profile ごとに複数の `AtpProblem` が生成される可能性がある。

## 非責務

この文書は external process の起動、timeout、portfolio execution、certificate 回収を扱わない。
それらは [atp_backend_integration.md](./atp_backend_integration.md) が扱う。
