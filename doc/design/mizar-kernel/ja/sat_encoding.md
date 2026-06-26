# Module: sat_encoding

> 正本は英語です。英語版:
> [../en/sat_encoding.md](../en/sat_encoding.md)。

## 目的

`sat_encoding` module は、checked formula/substitution evidence から deterministic SAT
problem を導出する。Architecture 15 と architecture 16 を精緻化する。

## Trust Statement

この module は trusted kernel code である。Accepted schema data から check artifact を
導出し、caller-supplied SAT clause を消費しない。

この module は proof search、premise selection、ATP search、backend invocation、
overload resolution、cluster search、implicit coercion insertion、fallback inference、
source loading、cache lookup、artifact lookup、wall-clock / random-state read、
unordered iteration dependence、mutable compiler-global state の hidden read を行っては
ならない。

## Owned behavior

この module が所有するもの:

- checked `formula_evidence` と `substitution_checker` result を消費して substitution
  side condition を validate すること;
- source formula と explicit checked substitution から instantiated formula を導出すること;
- canonical atom bytes によって SAT variable を deterministic に割り当てること;
- formulas と negated target goal から deterministic CNF/Tseitin problem を生成すること;
- diagnostics と replay check のために canonical bytes を expose すること。

この module は SAT solving、ATP encoding、premise selection、formula selection、backend
proof extraction を所有しない。

## Encoding rules

最初の schema version は normalized atoms 上の propositional encoding を使う。Atom identity は
evidence profile の下での canonical `clause::Atom` byte encoding である。Variable は sorted
canonical atom-byte order で割り当て、auxiliary Tseitin variables はすべての atom variables の
後に deterministic traversal order で割り当てる。

Encoded problem はすべての premise evidence formulas を true として assert し、standalone target
goal を refutation のため false として assert する。Final-goal formula は evidence envelope に現れる
という理由だけで premise としても assert されることはない。Equivalent caller order は同一の
canonical SAT bytes を生成しなければならない。

Instantiated formulas と SAT clauses は kernel-derived artifacts である。Diagnostic check
trace として記録できるが、trusted input field ではない。

## Rejections

Malformed formula structure、unsupported formula operator、inconsistent substitution report、
missing provenance、target mismatch、unsupported atom encoding、canonical byte budget failure は
SAT checking 前に reject する。Resource limits は `resource_exhaustion`、semantic encoding
failure は owning evidence field に応じて `invalid_sat_refutation` または `missing_provenance`
に map する。

## Gap classification

- `test_gap`: task 26 は stable encoding、substitution mutation rejection、
  equivalent-order determinism、target polarity、resource limits を cover する必要がある。
- `external_dependency_gap`: より豊かな quantified または theory-aware encoding は
  producer-owned formula payload と paired specs を待つ。この module はそれらを発明しない。
