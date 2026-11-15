# 共通ハッシュユーティリティ

正本: [English](../en/hashing.md)。

`src/hashing.rs` は `Hash::to_lower_hex()` と厳密な
`Hash::from_lower_hex(&str) -> Option<Hash>` を所有する。接頭辞なし、
`Hash::BYTE_LEN` の二倍の小文字 ASCII hex のみを許す。
`lower_hex` は任意のバイト列を同じ表記へ変換する。schema とエラー変換は各所有者に残す。

`domain_hasher(domain)` は UTF-8 の長さを little-endian u64 で、続いて domain
を BLAKE3 に渡す。各 consumer の field と追加 domain の順序・書式は維持する。

`bridge_fingerprint_hash(domain, bytes)` は既存の
[VC/Kernel bridge mixer](../../architecture/ja/15.kernel_certificate_format.md)
を実装し、seed・区切り・rotation・wrapping 演算・長さ・big-endian 出力を維持する。
BLAKE3 や他の独自方式とは交換しない。衝突耐性要件の独自 fingerprint への適用は
owner 判断に残し、この関数に受理権限を与えない。旧実装由来の固定値テストは
`crates/mizar-session/tests/common_utilities.rs` に置く。
