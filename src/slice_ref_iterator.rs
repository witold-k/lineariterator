// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

//! # Module: Slice Reference Iterator
//!
//! ## Core Responsibility
//! This module implements iterators for borrowing fixed-width windows from a slice.
//! Immutable windows use the standard `Iterator` trait. Mutable windows use
//! [`LendingIterator`] so overlapping windows can be exposed without allowing
//! simultaneously live overlapping mutable references.

use crate::slice_ptr_iterator::{SlicePtrIterator, SliceMutPtrIterator};
use std::marker::PhantomData;
use std::iter::FusedIterator;

/// An iterator whose yielded item may borrow from the iterator itself.
///
/// Unlike `Iterator`, the lifetime of an item is tied to the mutable borrow
/// used for the call to `LendingIterator::next`. This makes overlapping
/// mutable windows safe: the current window must stop being used before the
/// iterator can advance to the next one.
pub trait LendingIterator {
    type Item<'b>
    where
        Self: 'b;

    fn next(&mut self) -> Option<Self::Item<'_>>;
}

/// An immutable iterator that yields references to slice windows.
///
/// # Example
/// ```rust
/// struct MockRunner;
/// impl MockRunner {
///     fn run() {
///         extern crate lineariterator as my_crate;
///         use my_crate::slice_ref_iterator::SliceRefIterator;
///
///         let data = [10, 20, 30, 40, 50];
///         let mut iter = SliceRefIterator::new(&data, 2);
///
///         assert_eq!(iter.next().unwrap(), &[10, 20]);
///         assert_eq!(iter.next().unwrap(), &[20, 30]);
///         assert_eq!(iter.next().unwrap(), &[30, 40]);
///         assert_eq!(iter.next().unwrap(), &[40, 50]);
///         assert!(iter.next().is_none());
///     }
/// }
/// MockRunner::run();
/// ```
pub struct SliceRefIterator<'a, T> {
    inner: SlicePtrIterator<'a, T>,
    _marker: PhantomData<&'a [T]>,
}

unsafe impl<'a, T: Sync> Send for SliceRefIterator<'a, T> {}

unsafe impl<'a, T: Sync> Sync for SliceRefIterator<'a, T> {}

/// A mutable window iterator over a slice.
///
/// The borrow returned by `next` prevents advancing the iterator while that window
/// is still in use:
///
/// ```compile_fail
/// use lineariterator::slice_ref_iterator::{LendingIterator, SliceMutRefIterator};
///
/// let mut data = [1, 2, 3];
/// let mut iter = SliceMutRefIterator::new(&mut data, 2);
/// let first = iter.next().unwrap();
/// let _second = iter.next().unwrap();
/// first[0] = 10;
/// ```
///
/// Consecutive windows may overlap when the step is smaller than the window width.
/// Unlike a standard `Iterator`, this type implements [`LendingIterator`], tying each
/// returned mutable window to the borrow of the iterator. The iterator therefore cannot
/// advance while a previously yielded mutable window is still in use.
///
/// # Example
/// ```rust
/// struct MockRunner;
/// impl MockRunner {
///     fn run() {
///         extern crate lineariterator as my_crate;
///         use my_crate::slice_ref_iterator::{LendingIterator, SliceMutRefIterator};
///
///         let mut data = [10, 20, 30, 40];
///         let mut iter = SliceMutRefIterator::new(&mut data, 2);
///
///         let w1 = iter.next().unwrap();
///         w1[0] = 99;
///
///         assert_eq!(data, [99, 20, 30, 40]);
///     }
/// }
/// MockRunner::run();
/// ```
pub struct SliceMutRefIterator<'a, T> {
    inner: SliceMutPtrIterator<'a, T>,
    _marker: PhantomData<&'a mut [T]>,
}

unsafe impl<'a, T: Send> Send for SliceMutRefIterator<'a, T> {}
unsafe impl<'a, T: Sync> Sync for SliceMutRefIterator<'a, T> {}

impl<'a, T> SliceRefIterator<'a, T> {
    /// Creates a new immutable window iterator over a slice with a default step size of `1`.
    #[inline(always)]
    pub fn new(slice: &'a [T], width: usize) -> Self {
        Self::new_step(slice, width, 1)
    }

    /// Creates a new immutable window iterator over a slice with a custom step size.
    #[inline(always)]
    pub fn new_step(slice: &'a [T], width: usize, step: usize) -> Self {
        Self {
            // Safety: The raw pointer from the slice is valid, aligned, and initialized for its length.
            inner: unsafe { SlicePtrIterator::new_step(slice.as_ptr(), width, slice.len(), step) },
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for SliceRefIterator<'a, T> {
    type Item = &'a [T];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        let ptr_item = self.inner.next()?;
        // The pointer originates from the borrowed input slice and the inner iterator
        // only yields windows that fit within that slice.
        unsafe { Some(&*ptr_item) }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        self.inner.size_hint()
    }

    #[inline(always)]
    fn count(self) -> usize {
        self.inner.size_hint().0
    }
}

impl<'a, T> ExactSizeIterator for SliceRefIterator<'a, T> {}
impl<'a, T> FusedIterator for SliceRefIterator<'a, T> {}


impl<'a, T> SliceMutRefIterator<'a, T> {
    /// Creates a new mutable window iterator over a slice with a default step size of `1`.
    #[inline(always)]
    pub fn new(slice: &'a mut [T], width: usize) -> Self {
        Self::new_step(slice, width, 1)
    }

    /// Creates a new mutable window iterator over a slice with a custom step size.
    #[inline(always)]
    pub fn new_step(slice: &'a mut [T], width: usize, step: usize) -> Self {
        Self {
            // Safety: The mutable pointer from the slice is valid and uniquely un-aliased.
            inner: unsafe { SliceMutPtrIterator::new_step(slice.as_mut_ptr(), width, slice.len(), step) },
            _marker: PhantomData,
        }
    }
}

impl<'a, T> LendingIterator for SliceMutRefIterator<'a, T> {
    type Item<'b> = &'b mut [T]
    where
        Self: 'b;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item<'_>> {
        let ptr_item = self.inner.next()?;
        // Safety: the returned reference is tied to the mutable borrow of self.
        // The iterator therefore cannot advance while that reference is in use,
        // even when consecutive windows overlap.
        unsafe { Some(&mut *ptr_item) }
    }
}

