//! Checked framing shared by stage storage and canonical payloads.

pub(super) fn checked_size(domain: &[u8], fields: &[&[u8]], limit: usize) -> Option<usize> {
    let mut size = domain.len();
    for field in fields {
        u64::try_from(field.len()).ok()?;
        size = size.checked_add(8)?.checked_add(field.len())?;
        if size > limit {
            return None;
        }
    }
    Some(size)
}

pub(super) fn encode(domain: &[u8], fields: &[&[u8]], limit: usize) -> Option<Vec<u8>> {
    let size = checked_size(domain, fields, limit)?;
    let mut bytes = Vec::with_capacity(size);
    bytes.extend_from_slice(domain);
    for field in fields {
        let length = u64::try_from(field.len()).ok()?;
        bytes.extend_from_slice(&length.to_le_bytes());
        bytes.extend_from_slice(field);
    }
    Some(bytes)
}

pub(super) fn decode<'a, const N: usize>(
    bytes: &'a [u8],
    domain: &[u8],
    limit: usize,
) -> Option<[&'a [u8]; N]> {
    if bytes.len() > limit {
        return None;
    }
    let mut remaining = bytes.strip_prefix(domain)?;
    let mut fields = [&[][..]; N];
    for field in &mut fields {
        let length = remaining
            .get(..8)
            .and_then(|bytes| <[u8; 8]>::try_from(bytes).ok())
            .map(u64::from_le_bytes)
            .and_then(|length| usize::try_from(length).ok())?;
        remaining = &remaining[8..];
        *field = remaining.get(..length)?;
        remaining = &remaining[length..];
    }
    remaining.is_empty().then_some(fields)
}

#[cfg(test)]
mod tests {
    use super::{checked_size, decode, encode};

    #[test]
    fn encoding_matches_literal_little_endian_frames() {
        let expected = [
            b'F', 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, b'a', b'b', b'c',
        ];
        assert_eq!(
            encode(b"F", &[b"", b"abc"], expected.len()),
            Some(expected.to_vec())
        );
    }

    #[test]
    fn decoding_accepts_literal_frame_at_limit_and_rejects_over_limit() {
        let frame = [
            b'F', 0, 0, 0, 0, 0, 0, 0, 0, 3, 0, 0, 0, 0, 0, 0, 0, b'a', b'b', b'c',
        ];
        assert_eq!(
            decode::<2>(&frame, b"F", frame.len()),
            Some([&b""[..], &b"abc"[..]])
        );
        assert_eq!(decode::<2>(&frame, b"F", frame.len() - 1), None);
    }

    #[test]
    fn malformed_frames_are_rejected() {
        let wrong_domain = *b"X";
        let truncated_length = [b'F', 0, 0, 0];
        let truncated_body = [b'F', 3, 0, 0, 0, 0, 0, 0, 0, b'a', b'b'];
        let overflowing_length = [b'F', 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff];
        let trailing_data = [b'F', 0, 0, 0, 0, 0, 0, 0, 0, b'x'];

        assert_eq!(decode::<1>(&wrong_domain, b"F", 1), None);
        assert_eq!(
            decode::<1>(&truncated_length, b"F", truncated_length.len()),
            None
        );
        assert_eq!(
            decode::<1>(&truncated_body, b"F", truncated_body.len()),
            None
        );
        assert_eq!(
            decode::<1>(&overflowing_length, b"F", overflowing_length.len()),
            None
        );
        assert_eq!(decode::<1>(&trailing_data, b"F", trailing_data.len()), None);
    }

    #[test]
    fn encoding_rejects_frames_over_the_caller_limit() {
        assert_eq!(checked_size(b"F", &[b"abc"], 11), None);
        assert_eq!(checked_size(b"F", &[b"abc"], 12), Some(12));
        assert_eq!(encode(b"F", &[b"abc"], 11), None);
        assert_eq!(
            encode(b"F", &[b"abc"], 12),
            Some(vec![b'F', 3, 0, 0, 0, 0, 0, 0, 0, b'a', b'b', b'c',])
        );
    }
}
