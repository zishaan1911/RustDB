use crc32fast::Hasher;
use thiserror::Error;

// Byte length of the CRC32 checksum appended to each WAL record.
pub const WAL_CHECKSUM_LEN: usize = 4;

#[derive(Debug, Error)]
pub enum WalChecksumError {
    #[error("WAL record checksum mismatch: stored={stored:#010x}, computed={computed:#010x}")]
    Mismatch { stored: u32, computed: u32 },

    #[error("WAL record buffer too short ({len} bytes) to contain a {WAL_CHECKSUM_LEN}-byte checksum")]
    BufferTooShort { len: usize },
}

//Lowlevel helpers

// Compute CRC32 over arbitrary bytes.
// This is the building block used by both the framing helpers below and by callers that construct checksums incrementally.
#[inline]
pub fn compute_raw(data: &[u8]) -> u32 {
    let mut h = Hasher::new();
    h.update(data);
    h.finalize()
}

// Continue an in-progress CRC32 computation.
// Useful when a WAL record is built across multiple buffers so that a single allocation is not required.
#[inline]
pub fn update_hasher(hasher: &mut Hasher, data: &[u8]) {
    hasher.update(data);
}

//Framed record helpers

//Compute the CRC32 for the payload portion of a WAL record.
pub fn compute_for_payload(payload: &[u8]) -> u32 {
    compute_raw(payload)
}

// Append a CRC32 checksum to payload, returning the framed record as a new Vec<u8>.
pub fn frame(payload: &[u8]) -> Vec<u8> {
    let crc = compute_for_payload(payload);
    let mut out = Vec::with_capacity(payload.len() + WAL_CHECKSUM_LEN);
    out.extend_from_slice(payload);
    out.extend_from_slice(&crc.to_le_bytes());
    out
}

// Append a CRC32 checksum to `buf` in-place.
// Equivalent to [frame] but avoids an allocation when the caller already owns a `Vec`.
pub fn frame_in_place(buf: &mut Vec<u8>) {
    let crc = compute_for_payload(buf);
    buf.extend_from_slice(&crc.to_le_bytes());
}

// Read the CRC32 stored in the last four bytes of a framed record.
pub fn read_stored(framed: &[u8]) -> Result<u32, WalChecksumError> {
    if framed.len() < WAL_CHECKSUM_LEN {
        return Err(WalChecksumError::BufferTooShort { len: framed.len() });
    }
    let start = framed.len() - WAL_CHECKSUM_LEN;
    let bytes: [u8; 4] = framed[start..].try_into().expect("slice is exactly 4 bytes");
    Ok(u32::from_le_bytes(bytes))
}

// Verify the CRC32 of a framed WAL record.
// Call this after reading a record from the WAL segment file and before deserialising it.
pub fn verify(framed: &[u8]) -> Result<(), WalChecksumError> {
    if framed.len() < WAL_CHECKSUM_LEN {
        return Err(WalChecksumError::BufferTooShort { len: framed.len() });
    }
    let payload_end = framed.len() - WAL_CHECKSUM_LEN;
    let stored = read_stored(framed)?;
    let computed = compute_for_payload(&framed[..payload_end]);
    if stored != computed {
        return Err(WalChecksumError::Mismatch { stored, computed });
    }
    Ok(())
}

// Return a reference to the payload portion of a framed record (everything except the trailing checksum bytes).
pub fn payload(framed: &[u8]) -> Option<&[u8]> {
    let n = framed.len().checked_sub(WAL_CHECKSUM_LEN)?;
    Some(&framed[..n])
}

#[cfg(test)]
mod tests {
    use super::*;

    // compute_raw

    #[test]
    fn compute_raw_is_deterministic() {
        let data = b"hello wal";
        assert_eq!(compute_raw(data), compute_raw(data));
    }

    #[test]
    fn compute_raw_differs_for_different_inputs() {
        assert_ne!(compute_raw(b"aaa"), compute_raw(b"aab"));
    }

    #[test]
    fn compute_raw_empty_is_defined() {
        // CRC32 of empty input is 0x00000000 per the spec.
        assert_eq!(compute_raw(b""), 0x0000_0000);
    }

    // frame

    #[test]
    fn frame_appends_four_bytes() {
        let payload = b"record body";
        let framed = frame(payload);
        assert_eq!(framed.len(), payload.len() + WAL_CHECKSUM_LEN);
    }

    #[test]
    fn frame_and_frame_in_place_agree() {
        let payload = b"txn begin";
        let via_frame = frame(payload);

        let mut buf = payload.to_vec();
        frame_in_place(&mut buf);

        assert_eq!(via_frame, buf);
    }

    #[test]
    fn frame_checksum_matches_compute_for_payload() {
        let payload = b"insert row data";
        let framed = frame(payload);
        let stored = read_stored(&framed).unwrap();
        let expected = compute_for_payload(payload);
        assert_eq!(stored, expected);
    }

    // verify

    #[test]
    fn verify_passes_for_correctly_framed_record() {
        let framed = frame(b"commit txn 42");
        assert!(verify(&framed).is_ok());
    }

    #[test]
    fn verify_fails_on_corrupted_payload() {
        let mut framed = frame(b"delete row");
        // Flip a bit in the payload.
        framed[0] ^= 0x80;
        assert!(matches!(
            verify(&framed).unwrap_err(),
            WalChecksumError::Mismatch { .. }
        ));
    }

    #[test]
    fn verify_fails_on_corrupted_checksum() {
        let mut framed = frame(b"update row");
        let last = framed.len() - 1;
        framed[last] ^= 0xFF;
        assert!(matches!(
            verify(&framed).unwrap_err(),
            WalChecksumError::Mismatch { .. }
        ));
    }

    #[test]
    fn verify_rejects_too_short_buffer() {
        let short = [0u8; 3];
        assert!(matches!(
            verify(&short).unwrap_err(),
            WalChecksumError::BufferTooShort { .. }
        ));
    }

    #[test]
    fn verify_accepts_exactly_four_bytes() {
        // if record payload is empty: the framed form is just the 4-byte
CRC of an empty slice.
        let framed = frame(b"");
        assert_eq!(framed.len(), WAL_CHECKSUM_LEN);
        assert!(verify(&framed).is_ok());
    }

    // payload helper

    #[test]
    fn payload_strips_checksum() {
        let original = b"checkpoint";
        let framed = frame(original);
        assert_eq!(payload(&framed).unwrap(), original);
    }

    #[test]
    fn payload_returns_none_for_short_buffer() {
        assert!(payload(&[0u8; 3]).is_none());
    }

    // incremental hashing

    #[test]
    fn incremental_hash_matches_single_pass() {
        let part_a = b"WAL header bytes";
        let part_b = b"WAL body bytes";

        let single = compute_raw(&[part_a, part_b].concat());

        let mut h = Hasher::new();
        update_hasher(&mut h, part_a);
        update_hasher(&mut h, part_b);
        let incremental = h.finalize();

        assert_eq!(single, incremental);
    }
}
