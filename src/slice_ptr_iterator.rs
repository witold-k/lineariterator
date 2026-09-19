// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

use std::marker::PhantomData;

/// An immutable iterator that yields raw pointers to slices of a specified width.
///
/// Yields raw pointers of type `*const [T]`. The lifetime `'a` binds the iterator
/// to the source data memory block safely.
///
/// # Example
/// ```rust
/// struct MockCrateRunner;
/// impl MockCrateRunner {
///     unsafe fn run_test() {
///         extern crate lineariterator as my_crate;
///         use my_crate::slice_ptr_iterator::SlicePtrIterator;
///
///         let data = [ 10, 20, 30, 40, 50 ];
///         // width = 2, total len = 4. Yields:, then, then [30, 40]
///         let mut iter = SlicePtrIterator::new(data.as_ptr(), 2, 4);
///
///         let w1 = iter.next().unwrap();
///         assert_eq!(unsafe { &*w1 }, &[10, 20]);
///
///         let w2 = iter.next().unwrap();
///         assert_eq!(unsafe { &*w2 }, &[20, 30]);
///     }
/// }
/// unsafe { MockCrateRunner::run_test(); }
/// ```
#[derive(Copy, Clone)]
pub struct SlicePtrIterator<'a, T> {
    ptr: *const T,
    width: usize,
    step: usize,
    windows_left: usize,
    _marker: PhantomData<&'a T>,
}

unsafe impl<'a, T: Sync> Send for SlicePtrIterator<'a, T> {}
unsafe impl<'a, T: Sync> Sync for SlicePtrIterator<'a, T> {}

/// A mutable iterator that yields raw pointers to mutable slices of a specified width.
///
/// Yields raw pointers of type `*mut [T]`.
#[derive(Copy, Clone)]
pub struct SliceMutPtrIterator<'a, T> {
    ptr: *mut T,
    width: usize,
    step: usize,
    windows_left: usize,
    _marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Send> Send for SliceMutPtrIterator<'a, T> {}
unsafe impl<'a, T: Sync> Sync for SliceMutPtrIterator<'a, T> {}

impl<'a, T> SlicePtrIterator<'a, T> {
    /// Creates a new windows pointer iterator with a default step size of `1`.
    ///
    /// # Safety
    /// - `ptr` must point to a valid, allocated region containing at least `len` objects.
    #[inline(always)]
    pub const unsafe fn new(ptr: *const T, width: usize, len: usize) -> Self {
        unsafe { Self::new_step(ptr, width, len, 1) }
    }

    /// Creates a new windows pointer iterator with a custom step size.
    ///
    /// # Safety
    /// - `ptr` must refer to an allocation containing `len` initialized `T` values.
    /// - Every yielded window must remain within that allocation and be properly aligned.
    #[inline(always)]
    pub const unsafe fn new_step(ptr: *const T, width: usize, len: usize, step: usize) -> Self {
        let actual_step = if step == 0 { 1 } else { step };
        let windows_left = if len >= width && width > 0 {
            (len - width) / actual_step + 1
        } else {
            0
        };

        Self {
            ptr,
            width,
            step: actual_step,
            windows_left,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> SliceMutPtrIterator<'a, T> {
    /// Creates a new mutable windows pointer iterator with a default step size of `1`.
    ///
    /// # Safety
    /// - `ptr` must point to an exclusively allocated region containing at least `len` objects.
    #[inline(always)]
    pub const unsafe fn new(ptr: *mut T, width: usize, len: usize) -> Self {
        unsafe { Self::new_step(ptr, width, len, 1) }
    }

    /// Creates a new mutable windows pointer iterator with a custom step size.
    ///
    /// # Safety
    /// - `ptr` must refer to an allocation containing `len` initialized, writable `T` values.
    /// - The allocation must remain exclusively accessible for lifetime `'a`.
    /// - If `step < width`, sequential windows overlap. The yielded raw pointers must not be converted
    ///   into simultaneously live overlapping `&mut [T]` references.
    #[inline(always)]
    pub const unsafe fn new_step(ptr: *mut T, width: usize, len: usize, step: usize) -> Self {
        let actual_step = if step == 0 { 1 } else { step };
        let windows_left = if len >= width && width > 0 {
            (len - width) / actual_step + 1
        } else {
            0
        };

        Self {
            ptr,
            width,
            step: actual_step,
            windows_left,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for SlicePtrIterator<'a, T> {
    type Item = *const [T];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.windows_left == 0 {
            None
        } else {
            let current = self.ptr;
            let slice_ptr = std::ptr::slice_from_raw_parts(current, self.width);

            self.ptr = self.ptr.wrapping_add(self.step);
            self.windows_left -= 1;
            Some(slice_ptr)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.windows_left, Some(self.windows_left))
    }
}

impl<'a, T> ExactSizeIterator for SlicePtrIterator<'a, T> {}

impl<'a, T> Iterator for SliceMutPtrIterator<'a, T> {
    type Item = *mut [T];

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.windows_left == 0 {
            None
        } else {
            let current = self.ptr;
            let slice_ptr = std::ptr::slice_from_raw_parts_mut(current, self.width);

            self.ptr = self.ptr.wrapping_add(self.step);
            self.windows_left -= 1;
            Some(slice_ptr)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.windows_left, Some(self.windows_left))
    }
}

impl<'a, T> ExactSizeIterator for SliceMutPtrIterator<'a, T> {}

