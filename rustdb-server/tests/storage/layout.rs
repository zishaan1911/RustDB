use rustdb_server::storage::page::layout::{
    allocate_tuple, can_fit, free_space, init, is_tombstone, read_free_space_ptr, read_slot,
    read_slot_count, tombstone_slot, tuple_data, tuple_data_mut, write_free_space_ptr, write_slot,
    write_slot_count, LayoutError, HEADER_SIZE, PAGE_SIZE, SLOT_SIZE,
};

fn blank_page() -> Vec<u8> {
    vec![0; PAGE_SIZE]
}

fn fresh_page() -> Vec<u8> {
    let mut page = blank_page();
    init(&mut page).expect("page initializes");
    page
}

#[test]
fn init_sets_slot_count_to_zero() {
    let mut page = blank_page();

    init(&mut page).unwrap();

    assert_eq!(read_slot_count(&page).unwrap(), 0);
}

#[test]
fn init_sets_free_space_ptr_to_page_size() {
    let mut page = blank_page();

    init(&mut page).unwrap();

    assert_eq!(read_free_space_ptr(&page).unwrap(), PAGE_SIZE as u16);
}

#[test]
fn init_gives_maximum_free_space() {
    let mut page = blank_page();

    init(&mut page).unwrap();

    assert_eq!(free_space(&page).unwrap(), PAGE_SIZE - HEADER_SIZE);
}

#[test]
fn slot_count_round_trip() {
    let mut page = blank_page();

    write_slot_count(&mut page, 42).unwrap();

    assert_eq!(read_slot_count(&page).unwrap(), 42);
}

#[test]
fn free_space_ptr_round_trip() {
    let mut page = blank_page();

    write_free_space_ptr(&mut page, 1234).unwrap();

    assert_eq!(read_free_space_ptr(&page).unwrap(), 1234);
}

#[test]
fn free_space_decreases_after_each_slot() {
    let mut page = fresh_page();
    let fs1 = free_space(&page).unwrap();

    allocate_tuple(&mut page, 10).unwrap();
    let fs2 = free_space(&page).unwrap();

    allocate_tuple(&mut page, 20).unwrap();
    let fs3 = free_space(&page).unwrap();

    assert!(fs1 > fs2);
    assert!(fs2 > fs3);
    assert_eq!(fs1 - fs2, 10 + SLOT_SIZE);
    assert_eq!(fs2 - fs3, 20 + SLOT_SIZE);
}

#[test]
fn free_space_decreases_when_fsp_moves_down() {
    let mut page = fresh_page();
    let fs1 = free_space(&page).unwrap();
    let fsp1 = read_free_space_ptr(&page).unwrap();

    write_free_space_ptr(&mut page, fsp1 - 50).unwrap();

    let fs2 = free_space(&page).unwrap();
    assert_eq!(fs1 - fs2, 50);
}

#[test]
fn allocate_tuple_increments_slot_count() {
    let mut page = fresh_page();

    assert_eq!(read_slot_count(&page).unwrap(), 0);

    allocate_tuple(&mut page, 50).unwrap();
    assert_eq!(read_slot_count(&page).unwrap(), 1);

    allocate_tuple(&mut page, 100).unwrap();
    assert_eq!(read_slot_count(&page).unwrap(), 2);
}

#[test]
fn allocate_tuple_moves_free_space_ptr() {
    let mut page = fresh_page();
    let fsp1 = read_free_space_ptr(&page).unwrap();

    allocate_tuple(&mut page, 100).unwrap();

    let fsp2 = read_free_space_ptr(&page).unwrap();
    assert_eq!(fsp1 as usize - fsp2 as usize, 100);
}

#[test]
fn allocate_tuple_returns_correct_offset() {
    let mut page = fresh_page();

    let (idx1, offset1) = allocate_tuple(&mut page, 100).unwrap();

    assert_eq!(idx1, 0);
    assert_eq!(offset1 as usize, PAGE_SIZE - 100);
}

#[test]
fn allocate_tuple_returns_sequential_slot_indices() {
    let mut page = fresh_page();

    let (idx1, _) = allocate_tuple(&mut page, 50).unwrap();
    let (idx2, _) = allocate_tuple(&mut page, 60).unwrap();
    let (idx3, _) = allocate_tuple(&mut page, 70).unwrap();

    assert_eq!(idx1, 0);
    assert_eq!(idx2, 1);
    assert_eq!(idx3, 2);
}

#[test]
fn allocate_tuple_errors_when_full() {
    let mut page = fresh_page();

    let result = allocate_tuple(&mut page, PAGE_SIZE);

    assert!(result.is_err());
}

#[test]
fn can_fit_returns_true_when_space_available() {
    let mut page = fresh_page();

    assert!(can_fit(&page, 100).unwrap());
    assert!(can_fit(&page, 1000).unwrap());

    allocate_tuple(&mut page, 100).unwrap();
    assert!(can_fit(&page, 100).unwrap());
}

#[test]
fn can_fit_returns_false_when_space_exhausted() {
    let mut page = fresh_page();
    let max_allocatable = PAGE_SIZE - HEADER_SIZE - SLOT_SIZE;

    allocate_tuple(&mut page, max_allocatable - 10).unwrap();

    assert!(!can_fit(&page, 100).unwrap());
}

#[test]
fn write_then_read_slot_round_trips() {
    let mut page = fresh_page();

    write_slot_count(&mut page, 1).unwrap();
    write_slot(&mut page, 0, 100, 200).unwrap();

    assert_eq!(read_slot(&page, 0).unwrap(), (100, 200));
}

#[test]
fn read_slot_out_of_range_errors() {
    let mut page = fresh_page();

    write_slot_count(&mut page, 2).unwrap();

    assert!(read_slot(&page, 5).is_err());
}

#[test]
fn write_slot_out_of_range_errors() {
    let mut page = fresh_page();

    write_slot_count(&mut page, 2).unwrap();

    assert!(write_slot(&mut page, 5, 100, 200).is_err());
}

#[test]
fn tombstone_slot_zeroes_length() {
    let mut page = fresh_page();
    let (idx, _) = allocate_tuple(&mut page, 50).unwrap();

    tombstone_slot(&mut page, idx).unwrap();

    let (_offset, length) = read_slot(&page, idx).unwrap();
    assert_eq!(length, 0);
}

#[test]
fn tombstone_preserves_offset() {
    let mut page = fresh_page();
    let (idx, offset) = allocate_tuple(&mut page, 50).unwrap();

    tombstone_slot(&mut page, idx).unwrap();

    let (read_offset, _) = read_slot(&page, idx).unwrap();
    assert_eq!(read_offset, offset);
}

#[test]
fn is_tombstone_false_for_live_slot() {
    let mut page = fresh_page();
    let (idx, _) = allocate_tuple(&mut page, 50).unwrap();

    assert!(!is_tombstone(&page, idx).unwrap());
}

#[test]
fn tuple_data_returns_written_bytes() {
    let mut page = fresh_page();
    let (idx, _) = allocate_tuple(&mut page, 10).unwrap();

    tuple_data_mut(&mut page, idx)
        .unwrap()
        .copy_from_slice(b"0123456789");

    assert_eq!(tuple_data(&page, idx).unwrap(), b"0123456789");
}

#[test]
fn tuple_data_mut_allows_in_place_write() {
    let mut page = fresh_page();
    let (idx, _) = allocate_tuple(&mut page, 5).unwrap();

    tuple_data_mut(&mut page, idx)
        .unwrap()
        .copy_from_slice(b"HELLO");

    assert_eq!(tuple_data(&page, idx).unwrap(), b"HELLO");
}

#[test]
fn tuple_data_errors_on_tombstone() {
    let mut page = fresh_page();
    let (idx, _) = allocate_tuple(&mut page, 50).unwrap();

    tombstone_slot(&mut page, idx).unwrap();

    assert!(tuple_data(&page, idx).is_err());
}

#[test]
fn multiple_tuples_do_not_overlap() {
    let mut page = fresh_page();

    let (idx1, _) = allocate_tuple(&mut page, 10).unwrap();
    let (idx2, _) = allocate_tuple(&mut page, 20).unwrap();
    let (idx3, _) = allocate_tuple(&mut page, 15).unwrap();

    tuple_data_mut(&mut page, idx1)
        .unwrap()
        .copy_from_slice(b"0123456789");
    tuple_data_mut(&mut page, idx2)
        .unwrap()
        .copy_from_slice(b"01234567890123456789");
    tuple_data_mut(&mut page, idx3)
        .unwrap()
        .copy_from_slice(b"012345678901234");

    assert_eq!(tuple_data(&page, idx1).unwrap(), b"0123456789");
    assert_eq!(tuple_data(&page, idx2).unwrap(), b"01234567890123456789");
    assert_eq!(tuple_data(&page, idx3).unwrap(), b"012345678901234");
}

#[test]
fn free_space_after_multiple_allocations_is_consistent() {
    let mut page = fresh_page();
    let fs_initial = free_space(&page).unwrap();

    allocate_tuple(&mut page, 100).unwrap();
    let fs_after_1 = free_space(&page).unwrap();
    assert_eq!(fs_initial - fs_after_1, 100 + SLOT_SIZE);

    allocate_tuple(&mut page, 200).unwrap();
    let fs_after_2 = free_space(&page).unwrap();
    assert_eq!(fs_after_1 - fs_after_2, 200 + SLOT_SIZE);

    allocate_tuple(&mut page, 50).unwrap();
    let fs_after_3 = free_space(&page).unwrap();
    assert_eq!(fs_after_2 - fs_after_3, 50 + SLOT_SIZE);
}

#[test]
fn all_entry_points_reject_wrong_size_buffer() {
    let small_page = vec![0; 100];
    let large_page = vec![0; 16384];

    assert!(read_slot_count(&small_page).is_err());
    assert!(read_slot_count(&large_page).is_err());
    assert!(read_free_space_ptr(&small_page).is_err());
    assert!(read_free_space_ptr(&large_page).is_err());
    assert!(free_space(&small_page).is_err());
    assert!(free_space(&large_page).is_err());
    assert!(init(&mut small_page.clone()).is_err());
    assert!(init(&mut large_page.clone()).is_err());
}

#[test]
fn init_sets_empty_page_metadata() {
    let page = fresh_page();

    assert_eq!(read_slot_count(&page), Ok(0));
    assert_eq!(read_free_space_ptr(&page), Ok(PAGE_SIZE as u16));
    assert_eq!(free_space(&page), Ok(PAGE_SIZE - HEADER_SIZE));
}

#[test]
fn allocate_tuple_appends_slot_and_moves_tuple_area_backwards() {
    let mut page = fresh_page();

    let (first_slot, first_offset) = allocate_tuple(&mut page, 12).expect("first tuple fits");
    let (second_slot, second_offset) = allocate_tuple(&mut page, 7).expect("second tuple fits");

    assert_eq!(first_slot, 0);
    assert_eq!(second_slot, 1);
    assert_eq!(first_offset as usize, PAGE_SIZE - 12);
    assert_eq!(second_offset as usize, PAGE_SIZE - 12 - 7);
    assert_eq!(read_slot_count(&page), Ok(2));
    assert_eq!(read_slot(&page, first_slot), Ok((first_offset, 12)));
    assert_eq!(read_slot(&page, second_slot), Ok((second_offset, 7)));
    assert_eq!(
        free_space(&page),
        Ok(PAGE_SIZE - HEADER_SIZE - (2 * SLOT_SIZE) - 19)
    );
}

#[test]
fn tuple_data_round_trips_multiple_records_without_overlap() {
    let mut page = fresh_page();

    let (alpha_slot, _) = allocate_tuple(&mut page, 5).expect("alpha tuple fits");
    let (beta_slot, _) = allocate_tuple(&mut page, 9).expect("beta tuple fits");
    let (gamma_slot, _) = allocate_tuple(&mut page, 4).expect("gamma tuple fits");

    tuple_data_mut(&mut page, alpha_slot)
        .expect("alpha tuple is writable")
        .copy_from_slice(b"alpha");
    tuple_data_mut(&mut page, beta_slot)
        .expect("beta tuple is writable")
        .copy_from_slice(b"beta-data");
    tuple_data_mut(&mut page, gamma_slot)
        .expect("gamma tuple is writable")
        .copy_from_slice(b"gamm");

    assert_eq!(
        tuple_data(&page, alpha_slot).expect("alpha tuple readable"),
        b"alpha"
    );
    assert_eq!(
        tuple_data(&page, beta_slot).expect("beta tuple readable"),
        b"beta-data"
    );
    assert_eq!(
        tuple_data(&page, gamma_slot).expect("gamma tuple readable"),
        b"gamm"
    );
}

#[test]
fn tombstone_marks_slot_unreadable_but_preserves_offset() {
    let mut page = fresh_page();
    let (slot, offset) = allocate_tuple(&mut page, 16).expect("tuple fits");

    tombstone_slot(&mut page, slot).expect("slot can be tombstoned");

    assert_eq!(read_slot(&page, slot), Ok((offset, 0)));
    assert_eq!(is_tombstone(&page, slot), Ok(true));
    assert_eq!(tuple_data(&page, slot), Err(LayoutError::TombstoneSlot));
    assert_eq!(
        tuple_data_mut(&mut page, slot),
        Err(LayoutError::TombstoneSlot)
    );
}

#[test]
fn can_fit_accounts_for_new_slot_metadata() {
    let mut page = fresh_page();
    let exact_fit_len = PAGE_SIZE - HEADER_SIZE - SLOT_SIZE;

    assert_eq!(can_fit(&page, exact_fit_len), Ok(true));
    assert_eq!(can_fit(&page, exact_fit_len + 1), Ok(false));

    allocate_tuple(&mut page, exact_fit_len).expect("exactly fitting tuple allocates");

    assert_eq!(can_fit(&page, 0), Ok(false));
    assert_eq!(
        allocate_tuple(&mut page, 0),
        Err(LayoutError::NoSpace { data_len: 0 })
    );
}

#[test]
fn slot_operations_reject_out_of_range_index() {
    let mut page = fresh_page();
    write_slot_count(&mut page, 1).expect("test slot count set");

    assert_eq!(
        read_slot(&page, 1),
        Err(LayoutError::SlotOutOfRange {
            index: 1,
            slot_count: 1,
        })
    );
    assert_eq!(
        write_slot(&mut page, 1, 128, 8),
        Err(LayoutError::SlotOutOfRange {
            index: 1,
            slot_count: 1,
        })
    );
}

#[test]
fn entry_points_reject_wrong_page_size() {
    let mut short_page = vec![0; PAGE_SIZE - 1];

    assert_eq!(
        init(&mut short_page),
        Err(LayoutError::PageSizeMismatch {
            expected: PAGE_SIZE,
            got: PAGE_SIZE - 1,
        })
    );
    assert_eq!(
        read_slot_count(&short_page),
        Err(LayoutError::PageSizeMismatch {
            expected: PAGE_SIZE,
            got: PAGE_SIZE - 1,
        })
    );
    assert_eq!(
        allocate_tuple(&mut short_page, 1),
        Err(LayoutError::PageSizeMismatch {
            expected: PAGE_SIZE,
            got: PAGE_SIZE - 1,
        })
    );
}
