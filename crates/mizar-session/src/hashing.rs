//! Shared hash encodings and domain initialization.

use crate::Hash;

impl Hash {
    #[must_use]
    pub fn to_lower_hex(self) -> String {
        lower_hex(self.as_bytes())
    }

    #[must_use]
    pub fn from_lower_hex(value: &str) -> Option<Self> {
        if value.len() != Self::BYTE_LEN * 2 {
            return None;
        }
        let mut bytes = [0; Self::BYTE_LEN];
        for (index, pair) in value.as_bytes().as_chunks::<2>().0.iter().enumerate() {
            let decode = |byte| match byte {
                b'0'..=b'9' => Some(byte - b'0'),
                b'a'..=b'f' => Some(byte - b'a' + 10),
                _ => None,
            };
            bytes[index] = (decode(pair[0])? << 4) | decode(pair[1])?;
        }
        Some(Self::from_bytes(bytes))
    }
}

#[must_use]
pub fn lower_hex(bytes: &[u8]) -> String {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::with_capacity(bytes.len() * 2);
    for byte in bytes {
        output.push(HEX[(byte >> 4) as usize] as char);
        output.push(HEX[(byte & 0x0f) as usize] as char);
    }
    output
}

#[must_use]
pub fn domain_hasher(domain: &str) -> blake3::Hasher {
    let mut hasher = blake3::Hasher::new();
    hasher.update(&(domain.len() as u64).to_le_bytes());
    hasher.update(domain.as_bytes());
    hasher
}

#[must_use]
pub fn bridge_fingerprint_hash(domain: &str, bytes: &[u8]) -> Hash {
    let mut lanes = [
        0x6d_69_7a_61_72_2d_76_63_u64,
        0x70_72_6f_6f_66_2d_69_64_u64,
        0x74_61_73_6b_32_30_2d_76_u64,
        0x66_69_6e_67_65_72_2d_31_u64,
    ];

    for (index, byte) in domain
        .as_bytes()
        .iter()
        .copied()
        .chain([0])
        .chain(bytes.iter().copied())
        .enumerate()
    {
        let lane = index % lanes.len();
        let mixed_index = (index as u64).rotate_left((lane as u32) + 1);
        lanes[lane] ^= u64::from(byte)
            .wrapping_add(0x9e37_79b9_7f4a_7c15)
            .wrapping_add(mixed_index);
        lanes[lane] = lanes[lane]
            .rotate_left(11 + lane as u32)
            .wrapping_mul(0x1000_0000_01b3);
    }

    lanes[0] ^= bytes.len() as u64;
    lanes[1] ^= (domain.len() as u64).rotate_left(17);
    lanes[2] ^= lanes[0].rotate_left(7);
    lanes[3] ^= lanes[1].rotate_left(13);

    let mut output = [0_u8; Hash::BYTE_LEN];
    for (chunk, lane) in output.as_chunks_mut::<8>().0.iter_mut().zip(lanes) {
        chunk.copy_from_slice(&lane.to_be_bytes());
    }
    Hash::from_bytes(output)
}
