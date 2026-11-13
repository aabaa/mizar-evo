# バックエンド不要のインスタンス探索

正本: [English](../en/instance_finder.md)。根拠は仕様 §21.7.1–§21.7.5 と [architecture 10](../../architecture/ja/10.atp_backend_integration.md) にある。

## API と境界

`instance_finder::find_instance_evidence(&AtpProblem, &EncodedBackendProblem, &VcKernelEvidenceHandoff, Option<[u64; 4]>, Option<&[String]>) -> Result<Option<CandidateKernelEvidence>, String>` はバックエンドなしで探索する。`CandidateKernelEvidence` は既存の `BackendCandidateEvidence` transport の再公開であり、証明の受理状態を与えない。
構造化問題は encoded 入力全体を再現し、元の local handoff の target、source formula、provenance と一致しなければならない。対応範囲は local formula evidence、先頭の全称量化された前提、ground goal であり、type guard は認証済み前提と完全に一致する重複に限る。encoded property、既存の substitution payload、未対応の式や項は fail closed とする。encoded problem 外の前提は使わない。
参考用 used-premise label は実際の前提 label へ探索を制限するだけであり、未知または conjecture の label は無効とする。backend trace、log、report から式や信頼された事実を得ない。

## 探索と上限

呼出側は [verifier-policy limits](../../mizar-proof/en/policy.md#instance-search-limits) をそのまま渡す。None は証拠を返さず、Some はゼロを含む値をそのまま扱う。4 座標は distinct observed/generated ground terms、premise-instantiation candidates（ground 前提の空置換を含む）、named equality-instance candidates、kernel/SAT checks の上限である。上限到達による探索失敗は証拠を返さない。
置換の定義域は既存の観測済み閉部分項に固定する。名前付き等式探索は有限個の置換インスタンスの部分項も使うが、置換の定義域へ追加して無制限の項閉包を生成することはしない。前提置換と reflexivity、symmetry、transitivity、function congruence、predicate congruence の名前を決定的な順序で列挙する。等式の式は渡さず、kernel が名前から構築する。
元の source/provenance と goal binding を local wire に保持する。返す候補はすべて、元の handoff context に対する通常の kernel evidence checker を通過する。利用側も通常の kernel 受理と policy 検査を維持する。
backend extraction、portfolio execution、source-language extraction の完全性、proof witness、publication、cache reuse は別作業とし、task-29 joint corpus の拒否を維持する。

## テスト

translator tests は実際の構造化・encoded 問題と local wire を用い、first-order instantiation、congruence/transitivity の等式鎖、両 encoder による決定的な backend-free 探索、上限と exhaustion、各候補の独立 kernel 検査、stale input と偽 report を検査する。この core の行列は 6 family とし、backend success report で kernel 受理を代用しない。
