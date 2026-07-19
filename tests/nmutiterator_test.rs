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
