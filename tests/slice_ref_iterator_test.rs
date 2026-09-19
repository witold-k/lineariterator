// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use lineariterator::slice_ref_iterator::*;

#[test]
fn test_slice_ref_iterator_basic() {
    let data = [1, 2, 3, 4, 5];
    let width = 3;

    let it = SliceRefIterator::new(&data, width);

    let results: Vec<_> = it.map(|s| s.to_vec()).collect();

    assert_eq!(
        results,
        vec![
            vec![1, 2, 3],
            vec![2, 3, 4],
            vec![3, 4, 5],
        ]
    );
}

#[test]
fn test_slice_mut_ref_iterator_basic() {
    let mut data = [10, 20, 30, 40];
    let width = 2;

    let mut it = SliceMutRefIterator::new(&mut data, width);

    let mut results = Vec::new();
    while let Some(window) = LendingIterator::next(&mut it) {
        results.push(window.to_vec());
    }

    assert_eq!(
        results,
        vec![
            vec![10, 20],
            vec![20, 30],
            vec![30, 40],
        ]
    );
}

#[test]
fn test_single_window() {
    let data = [7, 8, 9];
    let width = 3;

    let mut it = SliceRefIterator::new(&data, width);

    assert_eq!(it.next().unwrap(), &[7, 8, 9]);
    assert!(it.next().is_none());
}

#[test]
fn test_slice_mut_ref_iterator_progressive_mutation() {
    let mut data = [5, 6, 7, 8];
    let width = 3;

    let mut it = SliceMutRefIterator::new(&mut data, width);

    while let Some(window) = LendingIterator::next(&mut it) {
        window[0] *= 2; // double the first element of each window
    }

    // Windows:
    // [5,6,7] -> double 5 -> [10,6,7]
    // [6,7,8] -> double 6 -> [10,12,7]
    //
    // Final data:
    // [10,12,7,8]

    assert_eq!(data, [10, 12, 7, 8]);
}



#[test]
fn test_slice_ref_iterator_custom_step() {
    let data = [1, 2, 3, 4, 5, 6];
    let results: Vec<_> = SliceRefIterator::new_step(&data, 2, 2)
        .map(|window| window.to_vec())
        .collect();

    assert_eq!(results, vec![vec![1, 2], vec![3, 4], vec![5, 6]]);
}

#[test]
fn test_slice_ref_iterator_zero_width_is_empty() {
    let data = [1, 2, 3];
    let mut iter = SliceRefIterator::new(&data, 0);

    assert_eq!(iter.len(), 0);
    assert!(iter.next().is_none());
}

#[test]
fn test_slice_mut_ref_iterator_overlapping_windows() {
    let mut data = [1, 2, 3, 4];
    let mut iter = SliceMutRefIterator::new(&mut data, 2);

    while let Some(window) = LendingIterator::next(&mut iter) {
        window[0] += 10;
    }

    assert_eq!(data, [11, 12, 13, 4]);
}

#[test]
fn test_slice_mut_ref_iterator_custom_step() {
    let mut data = [1, 2, 3, 4, 5, 6];
    let mut iter = SliceMutRefIterator::new_step(&mut data, 2, 2);

    while let Some(window) = LendingIterator::next(&mut iter) {
        window[0] *= 10;
    }

    assert_eq!(data, [10, 2, 30, 4, 50, 6]);
}

#[test]
fn test_slice_ref_iterator_zero_sized_type() {
    let data = [(); 4];
    let iter = SliceRefIterator::new(&data, 2);

    assert_eq!(iter.count(), 3);
}
