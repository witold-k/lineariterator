// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use lineariterator::slice_ptr_iterator::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_slice_ptr_iterator_basic() {
        let data = [1, 2, 3, 4, 5];
        let width = 3;

        let it = unsafe {
            SlicePtrIterator::new(data.as_ptr(), width, data.len())
        };

        let mut results = Vec::new();

        for ptr in it {
            let slice = unsafe { &*ptr };
            results.push(slice.to_vec());
        }

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
    fn test_slice_mut_ptr_iterator_basic() {
        let mut data = [10, 20, 30, 40];
        let width = 2;

        let it = unsafe {
            SliceMutPtrIterator::new(data.as_mut_ptr(), width, data.len())
        };

        let mut results = Vec::new();

        for ptr in it {
            let slice = unsafe { &mut *ptr };
            results.push(slice.to_vec());
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
    fn test_empty_iteration_when_width_exceeds_len() {
        let data = [1, 2];
        let width = 3;

        let mut it = unsafe {
            SlicePtrIterator::new(data.as_ptr(), width, data.len())
        };

        // Es darf keine Panic geben und der erste Aufruf muss None liefern
        assert!(it.next().is_none(), "Iterator must be empty when width > len");
    }

    #[test]
    fn test_single_window() {
        let data = [7, 8, 9];
        let width = 3;

        let mut it = unsafe {
            SlicePtrIterator::new(data.as_ptr(), width, data.len())
        };

        let first = it.next().unwrap();
        let slice = unsafe { &*first };
        assert_eq!(slice, &[7, 8, 9]);

        assert!(it.next().is_none());
    }

    #[test]
    fn test_pointer_progression() {
        let data = [1, 2, 3, 4];
        let width = 2;

        let mut it = unsafe {
            SlicePtrIterator::new(data.as_ptr(), width, data.len())
        };

        let p1 = it.next().unwrap();
        let p2 = it.next().unwrap();
        let p3 = it.next().unwrap();

        assert!(it.next().is_none());

        unsafe {
            assert_eq!(p1, std::ptr::slice_from_raw_parts(data.as_ptr(), 2));
            assert_eq!(p2, std::ptr::slice_from_raw_parts(data.as_ptr().add(1), 2));
            assert_eq!(p3, std::ptr::slice_from_raw_parts(data.as_ptr().add(2), 2));
        }
    }

    #[test]
    fn test_slice_mut_ptr_iterator_mutation() {
        let mut data = [1, 2, 3, 4];
        let width = 2;

        let it = unsafe {
            SliceMutPtrIterator::new(data.as_mut_ptr(), width, data.len())
        };

        for ptr in it {
            let slice = unsafe { &mut *ptr };
            for x in slice {
                *x += 10;
            }
        }

        // Windows:
        // [1,2] → [11,12]
        // [2,3] → [12,13]
        // [3,4] → [13,14]
        //
        // Final data:
        // [11,12,13,14]

        assert_eq!(data, [11, 22, 23, 14]);
    }

}

