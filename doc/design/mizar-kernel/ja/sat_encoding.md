# Module: sat_encoding

> 正本は英語です。英語版:
> [../en/sat_encoding.md](../en/sat_encoding.md)。

## 目的

`sat_encoding` module は、checked formula/substitution evidence から deterministic SAT
problem を導出する。Architecture 15 と architecture 16 を精緻化する。

## Trust Statement

この module は trusted kernel code である。Accepted schema data から check artifact を
導出し、caller-supplied SAT clause を消費しない。

この module は no proof search、no SAT solving、no ATP search or backend
invocation、no premise selection、no overload resolution、no cluster search、no
implicit coercion insertion、no fallback inference、no acceptance from backend-reported
success alone、no source loading、no cache lookup、no artifact lookup、no wall-clock or
random-state reads、no unordered iteration dependence、no hidden reads of
mutable compiler-global state を満たす。Caller-supplied instantiated formula、SAT
clause、backend proof method、resolution trace、SMT proof object、backend log を
trusted payload として受理してはならない。

## Owned behavior

この module が所有するもの:

- checked `formula_evidence` record から task-26 formula-instantiation substitution
  side condition を validate すること;
- source formula と explicit checked substitution から instantiated formula を導出すること;
- canonical atom bytes によって SAT variable を deterministic に割り当てること;
- formulas と `final_goal` に記録された polarity で assert される target goal から
  deterministic CNF/Tseitin problem を生成すること;
- diagnostics と replay check のために read-only encoded-problem accessor と canonical
  bytes を expose すること。

この module は SAT solving、ATP encoding、premise selection、formula selection、backend
proof extraction、substitution invention、source formula projection を所有しない。

## Task-26 instantiation scope

最初の source-backed implementation は `formula_evidence::ParsedKernelEvidence`
を消費する。Formula entry は premise formula として扱い、explicit substitution
record をその named source formula に適用し、導出された instantiated formula
を asserted formula set に追加する。元の premise formula は assert されたままで、
standalone final goal は分離されたままであり、premise や `used_axioms` source
として使われない。

Task 26 は `SubstitutionPayload` の `rewrite_path` が root である formula-wide
formal-to-actual substitution を support する。Replacement term は manifest-derived
clause context と explicit `BinderContextV1` bytes に対して validate され、payload
`actual_term` records に現れる binder id も含めて検査される。Zero-frame binder context
は explicit v1 encoding を通じてのみ accept される。Empty bytes、noncanonical
frames、missing frames、unused frames は `invalid_substitution` として reject する。
Formal variable は source formula に出現していなければならず、normalized binder term
の下で capture risk があれば `invalid_substitution` として reject する。これは
explicit payload に対する deterministic replay check であり、substitution search ではない。

Task 26 では、root `rewrite_path` は source formula tree のすべての atom term
内部にある各 formal variable の非束縛出現へ substitution を適用することを意味する。
Replay は formula を canonical tree order で walk する: `Atom` arguments は左から右、
`Not` child、その後 `And` / `Or` children は保存順である。複数 replacement は
formal variable id を key とする simultaneous map として適用される。同じ payload 内の
他の entry によって replacement actual term を再帰的に rewrite しない。
Substitution record は parser がすでに要求する ascending `substitution_id` order で
消費される。各 derived formula fingerprint と canonical bytes は、instantiation 後かつ
SAT encoding 前に kernel が再計算する。

より豊かな substitution shape は、producer schema が replay に十分な formula-path
と alpha-renaming 情報を記録するまで fail-closed のままにする。Non-root term
rewrite path、local-abbreviation expansion payload、non-empty freshness witness、
non-empty free-variable constraint list は、repair または推測されず、
`invalid_substitution` として reject される。これは stub ではなく
`external_dependency_gap` / `deferred` と分類する。

## Encoding rules

最初の schema version は normalized atoms 上の propositional encoding を使う。Atom identity は
evidence profile の下での canonical `clause::Atom` byte encoding である。Variable は sorted
canonical atom-byte order で割り当て、auxiliary Tseitin variables はすべての atom variables の
後に deterministic traversal order で割り当てる。

Encoded problem はすべての premise evidence formulas と substitution-derived
instantiated formulas を true として assert する。`AssertFalseForRefutation` では
standalone target goal を false として assert する。`AssertTrueForConsistency` では
standalone target goal を true として assert する。Final-goal formula は evidence envelope
に現れるという理由だけで premise としても assert されることはない。Equivalent caller
order は同一の canonical SAT bytes を生成しなければならない。

Canonical SAT bytes は DIMACS ではなく、caller-supplied trusted payload でもない。
これは schema version、target VC、atom-variable manifest、derived formula instances、
CNF clauses を含む kernel-derived diagnostic/check-trace encoding である。

Task-26 canonical SAT bytes は domain `MIZAR_KERNEL_SAT_PROBLEM\0`、schema version
`1`、encoding version `1`、target fingerprint、sorted atom-variable manifest、
sorted assertion records、CNF clauses を使う。SAT variable は `1` から始まる
positive `u32` id である。Atom variable は sorted canonical atom bytes によって
割り当てる。Auxiliary variable は sorted assertions の pre-order Tseitin traversal
で作られた正確な順序でその後に続く。SAT literal は `(variable_id, positive_bool)`
であり、`positive_bool = true` は variable 自身、`false` はその否定を意味する。
各 clause は duplicate removal 後に `(variable_id, positive_bool)` で literal を
sort して格納する。

Tseitin clauses は traversal order で emit され、その後で各 root assertion が 1 つの
unit clause を追加する。`And(children)` では output `o` と child literals `c_i` に
対して、各 child の `(!o or c_i)`、次に `(o or !c_1 or ... or !c_n)` を emit する。
`Or(children)` では output `o` に対して、各 child の `(o or !c_i)`、次に
`(!o or c_1 or ... or c_n)` を emit する。`Not(child)` は child literal の polarity
を反転して再利用し、auxiliary variable を割り当てない。`Atom` は atom variable を
再利用する。Assertion record は assertion kind、asserted polarity、source formula id、
substitution id、formula fingerprint、canonical formula bytes で sort されるため、
equivalent caller order は同一 bytes を生成する。

Instantiated formulas と SAT clauses は kernel-derived artifacts である。
`EncodedSatProblem` fields は encoding module の外では private であり、read-only accessor
だけで expose されるため、downstream caller は SAT checking 前に target binding、
assertions、atom manifest、clauses、canonical bytes を mutate できない。これらは
diagnostic check trace として記録できるが、trusted input field ではない。

## Rejections

Malformed formula structure、unsupported formula operator、unsupported substitution
replay shape、capture risk、missing provenance、target mismatch、unsupported atom encoding、
canonical byte budget failure は SAT checking 前に reject する。Resource limits は
`resource_exhaustion`、substitution side-condition failure は `invalid_substitution`、
kernel-derived SAT material の semantic encoding failure は `invalid_sat_refutation`、
provenance / target binding failure は `missing_provenance` に map する。

## Gap classification

- `test_gap`: task 26 は stable encoding、substitution mutation rejection、
  equivalent-order determinism、target polarity、resource limits を cover する必要がある。
- `external_dependency_gap`: より豊かな quantified または theory-aware encoding は
  producer-owned formula payload と paired specs を待つ。この module はそれらを発明しない。
- `external_dependency_gap` / `deferred`: formula-path substitution、
  local-abbreviation expansion replay、alpha-renaming witness は、受理できるように
  なる前に producer-owned formula-substitution schema extension を必要とする。
