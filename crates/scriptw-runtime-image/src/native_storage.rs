//! Private storage framing only; the unchanged native manifest still admits
//! source, target, Engine policy and the exact decoded native SHA-256.
use sha2::{Digest, Sha256};
use std::io::Read;

/// Verified decoded Cargo storage. Private fields prevent a caller from
/// substituting bytes or a self-reported digest at Engine's admission boundary.
pub struct NativeRuntimeBytes {
    bytes: Box<[u8]>,
    digest: [u8; 32],
}
impl NativeRuntimeBytes {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
    pub fn digest(&self) -> &[u8; 32] {
        &self.digest
    }
}

pub(super) const MAGIC: &[u8; 8] = b"SWRTZ001";
pub(super) const HEADER_BYTES: usize = 48;
pub(super) const MAX_RAW_BYTES: usize = 256 * 1024 * 1024;

pub(super) fn decode(encoded: &[u8]) -> Option<NativeRuntimeBytes> {
    if encoded.get(..8)? != MAGIC {
        return None;
    }
    let length = usize::try_from(u64::from_le_bytes(encoded.get(8..16)?.try_into().ok()?)).ok()?;
    if length == 0 || length > MAX_RAW_BYTES {
        return None;
    }
    let expected: [u8; 32] = encoded.get(16..HEADER_BYTES)?.try_into().ok()?;
    let mut decoder = zstd::stream::read::Decoder::new(encoded.get(HEADER_BYTES..)?).ok()?;
    // Output bounds alone do not cap a malformed frame's history window.
    // Cargo uses level 1; its roundtrip also enforces this 8 MiB window bound.
    decoder.window_log_max(23).ok()?;
    let mut bytes = Vec::with_capacity(length);
    // A corrupted length cannot make an unbounded decoder allocation. One
    // extra byte distinguishes exact output from a truncated admitted prefix.
    decoder
        .take(length as u64 + 1)
        .read_to_end(&mut bytes)
        .ok()?;
    let digest: [u8; 32] = Sha256::digest(&bytes).into();
    if bytes.len() != length || digest != expected {
        return None;
    }
    Some(NativeRuntimeBytes {
        bytes: bytes.into_boxed_slice(),
        digest,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(raw: &[u8]) -> Vec<u8> {
        let mut image = MAGIC.to_vec();
        image.extend_from_slice(&(raw.len() as u64).to_le_bytes());
        image.extend_from_slice(&Sha256::digest(raw));
        image.extend_from_slice(&zstd::stream::encode_all(raw, 1).unwrap());
        image
    }

    #[test]
    fn exact_decoded_bytes_and_rejected_container_corruption() {
        let raw = b"Cargo-owned native image, including \0 bytes";
        let encoded = encode(raw);
        assert_eq!(decode(&encoded).unwrap().bytes(), raw);
        for index in [0, 8, 16, HEADER_BYTES] {
            let mut changed = encoded.clone();
            changed[index] ^= 1;
            assert!(decode(&changed).is_none(), "corrupted offset {index}");
        }
        for end in [0, 7, 15, 47, encoded.len() - 1] {
            assert!(decode(&encoded[..end]).is_none(), "truncated at {end}");
        }
        for length in [0, raw.len() as u64 - 1, raw.len() as u64 + 1, u64::MAX] {
            let mut changed = encoded.clone();
            changed[8..16].copy_from_slice(&length.to_le_bytes());
            assert!(decode(&changed).is_none(), "wrong raw length {length}");
        }
        let mut trailing = encoded;
        trailing.push(0xff);
        assert!(decode(&trailing).is_none());
    }

    #[test]
    fn oversized_frame_window_is_rejected_before_decompression() {
        let raw = b"bounded native storage";
        // Zstd frame without a content-size field, one final raw block.
        let mut frame = vec![0x28, 0xb5, 0x2f, 0xfd, 0, 0x50];
        frame.extend_from_slice(&((raw.len() as u32 * 8) | 1).to_le_bytes()[..3]);
        frame.extend_from_slice(raw);
        let wrap = |frame: &[u8]| {
            let mut bytes = MAGIC.to_vec();
            bytes.extend_from_slice(&(raw.len() as u64).to_le_bytes());
            bytes.extend_from_slice(&Sha256::digest(raw));
            bytes.extend_from_slice(frame);
            bytes
        };
        assert_eq!(decode(&wrap(&frame)).unwrap().bytes(), raw);
        frame[5] = 0x88; // 128 MiB history request with the same small output.
        assert!(decode(&wrap(&frame)).is_none());
    }
}
