# Doc11 Math Model 定義提案
Canonical language: English; [英語正本](../en/math.md)。
状態: 非規範的提案。以下の Doc11 固有 Q1–Q12 は上記 Q6/Q7/Q12 の限定採用を除き UNSELECTED（未選択）であり、どちらの案も実装・規範的採用を許可しない。承認済み [Doc4 decisions](./doc_build.md#owner-q-decisions)、未選択の [Doc8 Q9–Q18](./commonmark.md#未選択の所有者-q-decisions)、[Doc9 Q1–Q10](./links.md#未選択の所有者-q-決定) とは別の識別子であり、いずれも変更しない。

上記の未選択表記や以下の候補表記に対し、この限定採用状態を優先する。

## 採用済み opaque client runtime
Doc8 の実 inline/display occurrence に限り Doc11 Q6/Q7/Q12 の opaque-client 部分を採用する。[英語の所有定義](../en/math.md#adopted-opaque-client-runtime)が正本であり、symbolic formula、annotation、番号/参照、共有 build diagnostic は未選択のままとする。payload/provenance は [transport](./render.md#adopted-opaque-math-transport) で保存し、build 時 TeX 検証は行わない。
上流 npm `katex@0.19.0` の同一配布から JS/CSS/参照全 font と MIT license を改変せず同梱する。manifest に上流 archive と各 asset の SHA-256 を記録し、設定/client も同じ集合に束縛する。欠落・改変・release 混在・設定不一致は実行前に拒否し、local subtree 以外を取得しない。
固定設定は output `htmlAndMathml`、throwOnError true、trust false、strict ignore、maxExpand 1000、maxSize 10、globalGroup/leqno/fleqn/colorIsTextColor false、minRuleThickness 0、errorColor #cc0000。displayMode は Doc8 分類を使い、各呼出しに独立した空 macro 辞書と detached node を与える。UTF-8 payload 上限 65536 bytes は client でのみ適用し、超過は安全な fallback とする。
各 occurrence を独立に処理し、失敗時は textContent に元 payload と可視の Mathematics unavailable status を残す。JS 不在時も pending literal が読める。隣の処理を継続し、build diagnostic は追加しない。
同じ入力の fragment/asset bytes と terminal state/MathML を含む serialized DOM を、Linux x86_64、headless Chrome 151.0.7922.75、en-US、UTC、固定 asset/設定、font ready 後で比較する。pixel 等価や他 browser はこの主張に含めない。

## Authority And Fixed Boundary
spec [21.3](../../../spec/ja/21.source_code_annotation_and_atp.md#213-latex-式の注釈)、[24.3](../../../spec/ja/24.documentation_generation.md#243-数式のレンダリング)、[24.4](../../../spec/ja/24.documentation_generation.md#244-html出力) が注釈優先・既定変換・番号・client-side KaTeX を定める。architecture13 [Step3](../../architecture/en/13.documentation_and_extraction.md#step-3-resolve-links-and-render-math)、[決定性](../../architecture/en/13.documentation_and_extraction.md#outputs-are-deterministic)、internal05 [artifact inputs](../../internal/en/05.documentation_extraction.md#artifact-inputs) が consumer の所有境界を定める。この定義は限定承認済み [Doc4 inputs](./doc_build.md#index-inputs-and-later-completion) に依存し、Doc8 parsing/採用には依存しない。
functor・predicate・mode・attribute 定義の注釈は、定理文・型シグネチャ・数式表示の各 symbol occurrence に優先表記を与える。注釈のない symbol は元の名前を verbatim な等幅表記で保ち、周囲の式構造には spec24.3 の11行すべての既定変換を適用する。注釈は対応 occurrence の自動変換を上書きし、無関係の構造は上書きしない。spec24.4 の item page は requires/ensures も数式表示するが、comment tag から形式契約を推測しない。
spec24.3 は inline `$formula$`、独立行の `$$formula$$` 中央 display、定理文 display equation の自動番号、item 文書 page 内 display 連番を使う `(chapter.section.item)`、番号の hyperlink、記載済み `$$\text{see }(24.3.1)$$` 参照形式を定める。認識・delimiter eligibility・Markdown 干渉・不整合 delimiter 警告は Doc8 の選択に残し、ここでは分類済み occurrence だけを受け取る。
generator は build 中に LaTeX 構文を検証しない。不正 LaTeX は変更せず rendering engine に渡り、出力中の rendering error になり得る。static HTML は client-side KaTeX、`assets/katex/` の local library、server-side 処理なしを守る。提案 repertoire は generator allowlist や新しい言語制約ではない。

## Candidate Inputs And Formula Strategy
以下は owner review 用の Q1–Q12 の組合せ案①のみを記述する。Doc4 は完全な package/version/lock/module/edition identity、選択 publication hash、exports/origins、存在する保存 projection を保持する。reader の integrity/schema/binding refusal は維持する。raw signature・source range・proof evidence は構造化された描画可能な式、完全な annotation metadata、resolved symbol occurrence を保証しない。
案①は symbol annotation・parameter/binder identity・resolved occurrence・signature/statement/形式契約の構造について、Artifact owner が定義する publication-bound presentation projection を要求する。既知の注釈不在と metadata 不明を区別し、dependency を含む参照 symbol を定義元 publication に結ぶ。missing・incomplete・available-empty・conflicting record は別状態に保つ。source 再解析・signature parsing・名前による overload 解決・架空の empty projection は許さない。artifact field/schema や producer 実装は提案せず、別途承認が必要である。
parser-neutral comment handoff は raw LaTeX payload、inline/display 分類、source 順 occurrence identity、完全な attached-target provenance、mapped source segment を保つ。payload は認識済み delimiter token を除き、内部の綴り・空白・改行を保持し、ここでは TeX trim・entity decode・macro 展開をしない。採用済み Doc8 の順序付き raw interior segment を用い、CommonMark interpreted text は payload にしない。認識済み empty payload は occurrence 不在と区別し、追加の Doc11 build-time 構文警告を出す根拠にしない。
案①は保存 annotation を対応 resolved occurrence の opaque TeX fragment として挿入し、文字を変数 placeholder として走査せず、内部 symbol 名を再帰展開しない。これは literal insertion の具体的代替案であり、採用済み引数置換ではない。Factorial の `n!` と Gcd の `\gcd(a,b)` に対する実引数名変更・nested call・binder capture は Q2 の未解決事項である。spec の定理例は満たせなければならず、literal insertion が意図した利用を満たせないなら、template 構文を捏造したり例を変更したりせず owner 明確化を待つ。
完全な unannotated 構造では、案①は producer の順序付き expression/binder child を走査し、spec24.3 の表どおりに算術・除算・冪・剰余・比較・所属・包含・全称/存在量化を適用する。TeX brace で operand grouping を保持し、必要箇所の数学的括弧で tree grouping を保つ。operand 順と binder identity を保持し、代数的簡約・変数名変更・論理正規化はしない。unannotated leaf の symbol 名は verbatim 等幅で表示し、TeX escape は可視の綴りの保存だけに使う。正確な grouping・identifier encoding は Q3/Q4 に残し、新しい演算子変換を暗黙採用しない。
表外構造・未対応 projection variant・dependency notation 不明は readiness gap であり、架空の式や数学的誤りの証拠ではない。案①は存在する authoritative display text を escaped plain text として保ち、数式 rendering 不可を示し、presentation diagnostic を提案する。text もなければ statement を捏造せず不明を示す。明示的 empty annotation は empty payload として渡し、symbol 名へ黙って置換しない。incomplete annotation metadata は既知の注釈不在にしない。劣化方針と publication scope は Q5 が gate する。

## Candidate KaTeX And Numbering Handoff
generator repertoire 案は表の TeX construct、grouping、括弧、名前・参照を保持する monospace/text escape に限る。著者の annotation/comment payload は制限しない文字列として渡し、実際の対応 command と未対応/不正 command の挙動は選択 KaTeX runtime が担い、第二の generator TeX parser は置かない。build-time KaTeX 実行・TeX compile・構文事前検証・command 拒否・修復を導入しない。
opaque client 実行は上記採用済み runtime に従う。残る generated formula/numbering 入力には独立の判断が必要である。
案①は各 theorem-statement display occurrence に、同じ式の重複も含む page-local な安定 source/structure ordinal を付け、inline は counter を消費しない。comment display と非 theorem display は、この案では自動番号なしの display に保つ。chapter/section は page/publication に結び付く owner 指定 allocation から取得し、FQN 分解や例の定数から推測しない。allocation 不在は番号捏造ではなく numbering readiness を止める。Q8/Q9 に scope/allocation の代替案を残し、必須 theorem 番号と形式を保つ。
案①は将来 parser の明示的 math-reference 分類からだけ番号参照を認識し、TeX substring を走査しない。publication-bound number-to-display registry から番号 target を得て、重複番号・missing target は first-match link にせず未解決に保つ。Q10 が分類・曖昧/劣化方針・参照 namespace を gate し、認識は Doc8、page/anchor allocation・href encoding は emission 前に Doc9/Doc13–15 が定義する。Doc9 page/anchor のどの案も採用しない。

## Unselected Owner Q-Decisions
上記 Q6/Q7/Q12 の opaque-client 部分を除き、各行は UNSELECTED（未選択）であり、該当 Doc11/12 または後続 rendering を gate する。採用前に整合的な組合せと残る正確な定義が必要である。解決が言語仕様意図を変える場合は停止して明示的な仕様権限を求め、以下の案はその変更を許可しない。

| Question / affected gate | Unselected ① | Unselected ② |
|---|---|---|
| Q1 — 完全な formula/annotation input | 上記の completeness/resolved identity を持つ Artifact-owned structured presentation projection を要求。 | 指定 producer の別途認証済み publication-bound presentation sidecar を要求し validation/completeness を先に定義。両案とも raw-signature parsing・新しい意味解析は禁止。 |
| Q2 — annotation 適用 | 上記 opaque literal insertion。実引数名変更/nested call と spec 例の整合を採用前に確認。 | arity・capture avoidance・literal variable 区別・escape を含む構造化 parameter/binder-aware annotation 適用を定義。既存 string 構文だけでは template grammar は定まらない。 |
| Q3 — 自動 grouping・拡張構文 | 上記の構造保存 table-only traversal。他構造は unavailable とし、採用前に括弧を厳密定義。 | 実装前に完全な precedence/拡張構造描画定義を math owner に要求。全 table mapping と annotation 優先は維持。 |
| Q4 — verbatim identifier 描画 | Unicode/TeX 特殊文字の正確な TeX text/monospace escape map を定義し可視綴りを保存。 | unannotated 名を mixed text/math presentation の別 escaped monospace text node で運び、必須数学的構造を失わない composition を定義。 |
| Q5 — 不完全/未対応 formula の劣化 | 影響 formula のみ authoritative plain text または明示的 unavailable に劣化し、上記 warning を提案。 | 完全 input/rendering rule が揃うまで影響 mathematical page output を gate。proof validity を変えず refusal scope・diagnostic を定義。両案とも架空 TeX を engine に渡さない。 |
| Q6 — KaTeX release/repertoire/configuration | 上記 immutable release/assets と隔離した safe option/macro manifest を同梱。正確な pin・対応 command repertoire・全 option は採用前提。 | 別の明示的 client-side KaTeX configuration/repertoire 提案を採用前に要求。両案とも server renderer・build-time 構文検証・入力 allowlist は不可。 |
| Q7 — client failure/transport | client render failure を隔離し escaped payload と visible error marker を保持。正確な DOM encoding/fallback・safe option を Doc13–15 と定義。 | client failure 時は別 marker なしの escaped literal payload を保持し、accessible presentation/error feedback を render owner と定義。両案とも不正 LaTeX pass-through と緩やかな劣化を維持。 |
| Q8 — 番号 scope/order | theorem-statement display のみ item page ごとの producer/source occurrence 順に番号を付け、重複も数える。 | comment・他 item-page display も同じ page sequence に含め、追加 scope を採用前に定義。必須 theorem-statement 番号は両案で維持。 |
| Q9 — chapter/section allocation | page planning と共に明示的 publication-bound chapter/section allocation を要求。 | collision/completeness validation を持つ render-owner allocation registry を要求。両案とも FQN 綴りから推測せず `(chapter.section.item)` を変更しない。 |
| Q10 — 番号参照/target | parser 分類参照と publication-bound number registry を要求。未解決/曖昧参照は text を保持し、warning を提案。 | render-owned validated formula-reference registry を要求し、不完全/曖昧 target では該当参照 emission を gate。Doc8/Doc9 案を決めず認識・destination encoding を共同定義。 |
| Q11 — presentation diagnostic/map | mapped occurrence segment と stable full-identity tie を使う deterministic Documentation draft。build-time invalid-LaTeX warning は推定しない。 | 完全な mapped provenance が揃うまで diagnostic 生成 path を gate。unmapped location/refusal 方針を shared diagnostic owner と定義。両案とも offset 捏造・code 割当てなし。 |
| Q12 — 再現性境界 | 同じ pin assets/configuration と同じ browser/runtime environment で generated byte/client result を一致させ、browser pixel 同値は除く。 | generated-byte 再現性と別途列挙した browser compatibility matrix を要求し、client 同値基準を採用前に定義。両案とも source 順だけから cross-browser pixel 決定性を主張しない。 |

## Determinism And Adoption
案①は Doc4 full-identity/module/FQN 順と semantic child/source 順を別々に保つ。同じ bound input、lock/toolchain/schema context、option、engine asset、allocation registry なら worker/enumeration 順に依存せず math payload・番号・generated byte が一致し、重複も別の安定 occurrence として保持する。clock・network lookup・random id・ambient macro state は出力を供給しない。render-time failure は build diagnostic ではなく、asset/configuration 不在は deployment/readiness failure であって invalid-LaTeX 指摘ではない。
presentation draft は既存 stage/file/range diagnostic 順に stable publication/target/occurrence tie と保持 mapped segment を使う。map 不在は利用可能な authoritative provenance を使い、架空の連続 span を作らない。internal05 [diagnostic owner](../../internal/en/05.documentation_extraction.md#diagnostics) が実装前に意味・位置・順・profile 挙動を定義し、W0400–W0404 は attachment warning に保つ。明示的 documentation profile だけが documentation warning を昇格でき、phase0–15 検証・proof status・extraction acceptance は変えない。
上記 opaque-client runtime 以外の Doc11 は提案に保つ。Doc12 には整合的 owner 採用、承認済み complete input producer、正確な formula/annotation・KaTeX 定義、番号/参照 interface、shared diagnostic が必要であり、独立の Doc8 依存も保つ。symbolic-math code・schema・diagnostic 割当て・整合的採用・Doc8/Doc9 決定・coverage/owner credit・Doc4/5/crate 全体の完了は主張しない。
