use crc32fast::Hasher as Crc32Hasher;
use serde::{Deserialize, Serialize};
use std::mem;

use crate::{
    error::{Result, RustDbError},
    storage::page::RecordId,
};

pub type Lsn = u64;
pub const NULL_LSN: Lsn = 0;

#[derive(Debug, Clone, Copy)]
#[repr(C, packed)]
pub struct WalHeader {
    pub lsn: Lsn,
    pub txn_id: u64,
    pub payload_len: u32,
    pub checksum: u32,
    _reserved: [u8; 4],
}

const _: () = assert!(mem::size_of::<WalHeader>() == 32);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[repr(u8)]
pub enum WalRecordType {
    Insert     = 1,
    Update     = 2,
    Delete     = 3,
    Commit     = 4,
    Abort      = 5,
    Checkpoint = 6,
    Clr        = 7,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WalPayload {
    Insert {
        file_id: u32,
        rid: RecordId,
        after: Vec<u8>,
    },
    Update {
        file_id: u32,
        rid: RecordId,
        before: Vec<u8>,
        after: Vec<u8>,
    },
    Delete {
        file_id: u32,
        rid: RecordId,
        before: Vec<u8>,
    },
    Commit {
        prev_lsn: Lsn,
    },
    Abort {
        prev_lsn: Lsn,
    },
    Checkpoint {
        dirty_pages: Vec<DirtyPageEntry>,
        active_txns: Vec<ActiveTxnEntry>,
        prev_checkpoint_lsn: Lsn,
    },
    Clr {
        file_id: u32,
        rid: RecordId,
        undo_data: Vec<u8>,
        undo_next_lsn: Lsn,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DirtyPageEntry {
    pub file_id: u32,
    pub page_id: u32,
    pub rec_lsn: Lsn,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ActiveTxnEntry {
    pub txn_id: u64,
    pub last_lsn: Lsn,
}

#[derive(Debug, Clone)]
pub struct WalRecord {
    pub lsn: Lsn,
    pub txn_id: u64,
    pub payload: WalPayload,
}

impl TryFrom<u8> for WalRecordType {
    // Convert a raw byte into a WAL record type.
    type Error = RustDbError;

    fn try_from(v: u8) -> Result<Self> {
        match v {
            1 => Ok(Self::Insert),
            2 => Ok(Self::Update),
            3 => Ok(Self::Delete),
            4 => Ok(Self::Commit),
            5 => Ok(Self::Abort),
            6 => Ok(Self::Checkpoint),
            7 => Ok(Self::Clr),
            _ => Err(RustDbError::Internal(format!("unknown WAL record type byte: {v}"))),
        }
    }
}

impl WalRecord {
    // Create a new in-memory WAL record before writing it to disk.
    pub fn new(txn_id: u64, payload: WalPayload) -> Self {
        Self { lsn: NULL_LSN, txn_id, payload }
    }

    // Return the kind of WAL record carried by this object.
    pub fn record_type(&self) -> WalRecordType {
        match &self.payload {
            WalPayload::Insert { .. } => WalRecordType::Insert,
            WalPayload::Update { .. } => WalRecordType::Update,
            WalPayload::Delete { .. } => WalRecordType::Delete,
            WalPayload::Commit { .. } => WalRecordType::Commit,
            WalPayload::Abort { .. } => WalRecordType::Abort,
            WalPayload::Checkpoint { .. } => WalRecordType::Checkpoint,
            WalPayload::Clr { .. } => WalRecordType::Clr,
        }
    }

    // Serialize this WAL record into a byte buffer for appending.
    pub fn to_bytes(&self) -> Result<Vec<u8>> {
        let payload_bytes = bincode::serde::encode_to_vec(
            &self.payload,
            bincode::config::standard(),
        )
        .map_err(|e| RustDbError::Internal(format!("bincode encode: {e}")))?;

        let payload_len = payload_bytes.len() as u32;
        let total_len = 4 + mem::size_of::<WalHeader>() + payload_bytes.len();
        let mut buf = Vec::with_capacity(total_len);

        let mut hdr = WalHeader {
            lsn: self.lsn,
            txn_id: self.txn_id,
            payload_len,
            checksum: 0,
            _reserved: [0u8; 4],
        };

        let hdr_bytes: &[u8; 32] = unsafe { &*((&hdr) as *const WalHeader as *const [u8; 32]) };
        let mut hasher = Crc32Hasher::new();
        hasher.update(&hdr_bytes[..24]);
        hasher.update(&payload_bytes);
        hdr.checksum = hasher.finalize();

        let total_len_u32 = total_len as u32;
        buf.extend_from_slice(&total_len_u32.to_le_bytes());

        let hdr_bytes_final: &[u8; 32] = unsafe { &*((&hdr) as *const WalHeader as *const [u8; 32]) };
        buf.extend_from_slice(hdr_bytes_final);
        buf.extend_from_slice(&payload_bytes);

        debug_assert_eq!(buf.len(), total_len);
        Ok(buf)
    }

    // Parse a WAL record from bytes and verify its checksum.
    pub fn from_bytes(data: &[u8]) -> Result<(Self, usize)> {
        if data.len() < 4 {
            return Err(RustDbError::Internal("WAL: truncated length prefix".into()));
        }
        let total_len = u32::from_le_bytes(data[..4].try_into().unwrap()) as usize;

        if data.len() < total_len {
            return Err(RustDbError::Internal(format!(
                "WAL: record claims {total_len} bytes but only {} available",
                data.len()
            )));
        }

        let hdr_slice = &data[4..4 + mem::size_of::<WalHeader>()];
        let hdr: WalHeader = unsafe { (hdr_slice.as_ptr() as *const WalHeader).read_unaligned() };

        let payload_start = 4 + mem::size_of::<WalHeader>();
        let payload_end = payload_start + hdr.payload_len as usize;

        if total_len < payload_end {
            return Err(RustDbError::WalCorrupt { lsn: hdr.lsn });
        }

        let payload_bytes = &data[payload_start..payload_end];

        let mut hasher = Crc32Hasher::new();
        hasher.update(&hdr_slice[..24]);
        hasher.update(payload_bytes);
        if hasher.finalize() != hdr.checksum {
            return Err(RustDbError::WalCorrupt { lsn: hdr.lsn });
        }

        let (payload, _) = bincode::serde::decode_from_slice(
            payload_bytes,
            bincode::config::standard(),
        )
        .map_err(|e| RustDbError::Internal(format!("bincode decode: {e}")))?;

        let record = WalRecord {
            lsn: hdr.lsn,
            txn_id: hdr.txn_id,
            payload,
        };

        Ok((record, total_len))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn rid(page: u32, slot: u16) -> RecordId {
        RecordId::new(page, slot)
    }

    #[test]
    fn insert_record_roundtrip() {
        let mut rec = WalRecord::new(
            42,
            WalPayload::Insert {
                file_id: 1,
                rid: rid(5, 3),
                after: b"tuple bytes".to_vec(),
            },
        );
        rec.lsn = 100;

        let bytes = rec.to_bytes().unwrap();
        let (decoded, consumed) = WalRecord::from_bytes(&bytes).unwrap();

        assert_eq!(consumed, bytes.len());
        assert_eq!(decoded.lsn, 100);
        assert_eq!(decoded.txn_id, 42);
        assert!(matches!(decoded.record_type(), WalRecordType::Insert));

        if let WalPayload::Insert { after, rid: r, .. } = decoded.payload {
            assert_eq!(after, b"tuple bytes");
            assert_eq!(r, rid(5, 3));
        } else {
            panic!("wrong payload variant");
        }
    }

    #[test]
    fn update_record_roundtrip() {
        let mut rec = WalRecord::new(
            7,
            WalPayload::Update {
                file_id: 2,
                rid: rid(0, 0),
                before: b"old".to_vec(),
                after: b"new".to_vec(),
            },
        );
        rec.lsn = 200;

        let bytes = rec.to_bytes().unwrap();
        let (decoded, _) = WalRecord::from_bytes(&bytes).unwrap();
        assert_eq!(decoded.txn_id, 7);
        assert!(matches!(decoded.record_type(), WalRecordType::Update));
    }

    #[test]
    fn commit_record_roundtrip() {
        let mut rec = WalRecord::new(1, WalPayload::Commit { prev_lsn: 50 });
        rec.lsn = 51;
        let bytes = rec.to_bytes().unwrap();
        let (decoded, _) = WalRecord::from_bytes(&bytes).unwrap();
        assert!(matches!(decoded.record_type(), WalRecordType::Commit));
    }

    #[test]
    fn checkpoint_record_roundtrip() {
        let mut rec = WalRecord::new(
            0,
            WalPayload::Checkpoint {
                dirty_pages: vec![
                    DirtyPageEntry { file_id: 1, page_id: 3, rec_lsn: 77 },
                    DirtyPageEntry { file_id: 1, page_id: 8, rec_lsn: 90 },
                ],
                active_txns: vec![
                    ActiveTxnEntry { txn_id: 5, last_lsn: 80 },
                ],
                prev_checkpoint_lsn: 0,
            },
        );
        rec.lsn = 300;

        let bytes = rec.to_bytes().unwrap();
        let (decoded, _) = WalRecord::from_bytes(&bytes).unwrap();
        assert!(matches!(decoded.record_type(), WalRecordType::Checkpoint));

        if let WalPayload::Checkpoint { dirty_pages, active_txns, .. } = decoded.payload {
            assert_eq!(dirty_pages.len(), 2);
            assert_eq!(active_txns.len(), 1);
        } else {
            panic!("wrong variant");
        }
    }

    #[test]
    fn corrupted_record_rejected() {
        let mut rec = WalRecord::new(
            1,
            WalPayload::Insert {
                file_id: 0,
                rid: rid(1, 0),
                after: vec![0xAB; 16],
            },
        );
        rec.lsn = 1;

        let mut bytes = rec.to_bytes().unwrap();
        let last = bytes.len() - 1;
        bytes[last] ^= 0xFF;

        let result = WalRecord::from_bytes(&bytes);
        assert!(matches!(result, Err(RustDbError::WalCorrupt { .. })));
    }

    #[test]
    fn wal_header_is_32_bytes() {
        assert_eq!(mem::size_of::<WalHeader>(), 32);
    }
}
