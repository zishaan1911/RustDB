use serde::{Deserialize, Serialize};
use std::sync::atomic::{AtomicU64, Ordering};

static NEXT_TXN_ID: AtomicU64 = AtomicU64::new(1);

// Allocate the next transaction ID atomically.
pub fn next_txn_id() -> u64 {
    NEXT_TXN_ID.fetch_add(1, Ordering::Relaxed)
}

pub const INFO_XMIN_COMMITTED: u16 = 0b0000_0001;
pub const INFO_XMIN_ABORTED:   u16 = 0b0000_0010;
pub const INFO_XMAX_COMMITTED: u16 = 0b0000_0100;
pub const INFO_XMAX_ABORTED:   u16 = 0b0000_1000;
pub const INFO_HAS_TOAST:      u16 = 0b0001_0000;
pub const INFO_SELF_DELETED:   u16 = 0b0010_0000;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
pub struct RowVersion {
    pub xmin: u64,
    pub xmax: u64,
    pub cid: u32,
    pub infomask: u16,
}

impl RowVersion {
    // Create a row version for a new insert.
    pub fn new_insert(txn_id: u64, cid: u32) -> Self {
        Self {
            xmin:     txn_id,
            xmax:     u64::MAX,
            cid,
            infomask: 0,
        }
    }

    // Mark this row version as deleted.
    pub fn mark_deleted(&mut self, txn_id: u64) {
        self.xmax = txn_id;
    }

    // Return true when the row version is still live.
    pub fn is_live(&self) -> bool {
        self.xmax == u64::MAX
    }

    // Set the xmin committed flag.
    pub fn set_xmin_committed(&mut self)  { self.infomask |= INFO_XMIN_COMMITTED; }

    // Set the xmin aborted flag.
    pub fn set_xmin_aborted(&mut self)    { self.infomask |= INFO_XMIN_ABORTED;   }

    // Set the xmax committed flag.
    pub fn set_xmax_committed(&mut self)  { self.infomask |= INFO_XMAX_COMMITTED; }

    // Set the xmax aborted flag.
    pub fn set_xmax_aborted(&mut self)    { self.infomask |= INFO_XMAX_ABORTED;   }

    // Check whether xmin is committed.
    pub fn xmin_committed(&self) -> bool { self.infomask & INFO_XMIN_COMMITTED != 0 }

    // Check whether xmin is aborted.
    pub fn xmin_aborted(&self)   -> bool { self.infomask & INFO_XMIN_ABORTED   != 0 }

    // Check whether xmax is committed.
    pub fn xmax_committed(&self) -> bool { self.infomask & INFO_XMAX_COMMITTED != 0 }

    // Check whether xmax is aborted.
    pub fn xmax_aborted(&self)   -> bool { self.infomask & INFO_XMAX_ABORTED   != 0 }
}

#[derive(Debug, Clone)]
pub struct Snapshot {
    pub snapshot_txn_id: u64,
    pub active_txn_ids: Vec<u64>,
    pub xmax: u64,
}

impl Snapshot {
    // Create a new snapshot for transaction visibility.
    pub fn new(snapshot_txn_id: u64, active_txn_ids: Vec<u64>, xmax: u64) -> Self {
        Self { snapshot_txn_id, active_txn_ids, xmax }
    }

    // Determine whether a row version is visible in this snapshot.
    pub fn is_visible(&self, rv: &RowVersion) -> bool {
        if rv.xmin_aborted() {
            return false;
        }

        let xmin_visible = rv.xmin_committed()
            && rv.xmin <= self.xmax
            && !self.active_txn_ids.contains(&rv.xmin)
            || rv.xmin == self.snapshot_txn_id;

        if !xmin_visible {
            return false;
        }

        if rv.xmax == u64::MAX {
            return true;
        }

        if rv.xmax_aborted() {
            return true;
        }

        if rv.xmax == self.snapshot_txn_id {
            return false;
        }

        if rv.xmax_committed()
            && rv.xmax <= self.xmax
            && !self.active_txn_ids.contains(&rv.xmax)
        {
            return false;
        }

        true
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Build a snapshot for tests.
    fn snap(snap_id: u64, active: &[u64], xmax: u64) -> Snapshot {
        Snapshot::new(snap_id, active.to_vec(), xmax)
    }

    // Confirm committed inserts are visible.
    #[test]
    fn committed_insert_is_visible() {
        let mut rv = RowVersion::new_insert(5, 0);
        rv.set_xmin_committed();

        let snapshot = snap(10, &[], 9);
        assert!(snapshot.is_visible(&rv));
    }

    // Confirm aborted inserts are not visible.
    #[test]
    fn aborted_insert_not_visible() {
        let mut rv = RowVersion::new_insert(3, 0);
        rv.set_xmin_aborted();

        let snapshot = snap(10, &[], 9);
        assert!(!snapshot.is_visible(&rv));
    }

    // Confirm own transaction inserts remain visible.
    #[test]
    fn own_insert_visible_within_txn() {
        let rv = RowVersion::new_insert(7, 0);
        let snapshot = snap(7, &[], 6);
        assert!(snapshot.is_visible(&rv));
    }

    // Confirm committed deletes hide the row.
    #[test]
    fn committed_delete_hides_row() {
        let mut rv = RowVersion::new_insert(2, 0);
        rv.set_xmin_committed();
        rv.mark_deleted(4);
        rv.set_xmax_committed();

        let snapshot = snap(10, &[], 9);
        assert!(!snapshot.is_visible(&rv));
    }

    // Confirm aborted deletes leave the row visible.
    #[test]
    fn aborted_delete_leaves_row_visible() {
        let mut rv = RowVersion::new_insert(2, 0);
        rv.set_xmin_committed();
        rv.mark_deleted(4);
        rv.set_xmax_aborted();

        let snapshot = snap(10, &[], 9);
        assert!(snapshot.is_visible(&rv));
    }

    // Confirm rows from active writers are not visible.
    #[test]
    fn active_writer_changes_not_visible() {
        let mut rv = RowVersion::new_insert(6, 0);
        rv.set_xmin_committed();

        let snapshot = snap(10, &[6], 9);
        assert!(!snapshot.is_visible(&rv));
    }

    // Confirm transaction IDs increase monotonically.
    #[test]
    fn next_txn_id_is_monotonic() {
        let a = next_txn_id();
        let b = next_txn_id();
        let c = next_txn_id();
        assert!(a < b);
        assert!(b < c);
    }
}