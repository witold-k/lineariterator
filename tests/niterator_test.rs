// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use lineariterator::niterator::NMutIterator;

/// Tests basic iteration of NMutIterator.
///
/// # Steps
/// 1. Create a mutable array of integers.
/// 2. Get a mutable pointer to the start of the array.
/// 3. Create an NMutIterator using the pointer and the length of the array.
/// 4. Collect the values from the iterator into a vector.
/// 5. Assert that the collected values match the original array.
#[test]
fn test_nmutiterator_basic_iteration() {
    let mut data = [10, 20, 30, 40];
    let ptr = data.as_mut_ptr();

    let iter = unsafe { NMutIterator::new(ptr, data.len()) };

    let mut collected = Vec::new();
    for p in iter {
        unsafe { collected.push(*p) }
    }

    assert_eq!(collected, vec![10, 20, 30, 40]);
}


#[test]
fn test_nmutiterator_step() {
    let mut data = [10, 20, 30, 40, 50];
    let mut iter = unsafe { NMutIterator::new_step(data.as_mut_ptr(), 3, 2) };

    let values: Vec<_> = iter.by_ref().map(|ptr| unsafe { *ptr }).collect();
    assert_eq!(values, vec![10, 30, 50]);
    assert_eq!(iter.len(), 0);
}

#[test]
fn test_nmutiterator_zero_step_falls_back_to_one() {
    let mut data = [1, 2, 3];
    let iter = unsafe { NMutIterator::new_step(data.as_mut_ptr(), data.len(), 0) };

    let values: Vec<_> = iter.map(|ptr| unsafe { *ptr }).collect();
    assert_eq!(values, vec![1, 2, 3]);
}

#[test]
fn test_nmutiterator_clone_from_slice_contiguous() {
    let mut data = [0, 0, 0, 0];
    let mut iter = unsafe { NMutIterator::new(data.as_mut_ptr(), data.len()) };

    iter.clone_from_slice(&[1, 2, 3]);

    assert_eq!(data, [1, 2, 3, 0]);
    assert_eq!(iter.len(), 1);
}

#[test]
fn test_nmutiterator_clone_from_slice_strided() {
    let mut data = [0, 0, 0, 0, 0];
    let mut iter = unsafe { NMutIterator::new_step(data.as_mut_ptr(), 3, 2) };

    iter.clone_from_slice(&[1, 2, 3, 4]);

    assert_eq!(data, [1, 0, 2, 0, 3]);
    assert_eq!(iter.len(), 0);
}

#[test]
fn test_nmutiterator_zero_sized_type() {
    let mut data = [(); 4];
    let mut iter = unsafe { NMutIterator::new(data.as_mut_ptr(), data.len()) };

    assert_eq!(iter.len(), 4);
    assert_eq!(iter.by_ref().count(), 4);
    assert_eq!(iter.len(), 0);
}
