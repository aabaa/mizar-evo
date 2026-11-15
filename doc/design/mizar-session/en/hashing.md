# Shared hash utilities

Canonical language: English; [Japanese companion](../ja/hashing.md).

`src/hashing.rs` owns `Hash::to_lower_hex()` and strict
`Hash::from_lower_hex(&str) -> Option<Hash>`: exactly twice `Hash::BYTE_LEN`
lowercase ASCII hex digits, without prefixes. `lower_hex` encodes arbitrary
bytes using the same spelling. Callers retain their own schema/error adapters.

`domain_hasher(domain)` initializes BLAKE3 with a little-endian u64 UTF-8
byte length followed by the domain bytes. Consumer-owned fields and additional
framed domains retain their original order and encoding.

`bridge_fingerprint_hash(domain, bytes)` implements the existing
[VC/Kernel bridge mixer](../../architecture/en/15.kernel_certificate_format.md).
Its distinct seeds, separator, rotations, wrapping arithmetic, lengths and
big-endian output remain unchanged. It is not interchangeable with BLAKE3 or
the other custom mixers. Applicability of the architecture collision-resistance
requirement to custom fingerprints remains an owner decision; this utility
adds no acceptance authority. Predecessor golden vectors live in
`crates/mizar-session/tests/common_utilities.rs`.
