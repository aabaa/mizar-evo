# Architecture: ATP Backend Integration

> Canonical language: English. English canonical version: [../en/atp_backend_integration.md](../en/atp_backend_integration.md).

## 目的

外部 ATP process の起動、入力の受け渡し、timeout、portfolio execution、proof certificate の回収を定義する。
この文書は pipeline Phase 13 のうち、backend dispatch を扱う。

## 対応 backend

| Backend | Type | Format | Certificate |
|---|---|---|---|
| Vampire | ATP | TPTP | TSTP |
| E | ATP | TPTP | TSTP |
| CVC5 | SMT | SMT-LIB | LFSC / Alethe |
| Z3 | SMT | SMT-LIB | proof log / externally attested |

## 実行方針

- backend は子 process として起動する。
- problem は stdin または temporary file 経由で渡す。
- timeout は `mizar.pkg` の `[verifier].atp_timeout` を既定値にする。
- `solver: auto` では portfolio execution を行い、kernel-accepted certificate を最初に返した backend を採用する。
- 残りの backend process は終了させる。

## 結果分類

- `Proved(ProofCertificate)`
- `Disproved`
- `Timeout`
- `Unknown`
- `Error`

## Kernel との関係

backend が `Proved` を返しても、その時点では trusted result ではない。
Phase 14 の kernel certificate check が成功して初めて verified proof status になる。
