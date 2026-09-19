// SPDX-License-Identifier: Apache-2.0
// Copyright (c) 2026 Witold Kaminski

//! # Non-contiguous Memory Iterators (Stride Iterators)
//!
//! This module provides highly optimized, low-level memory iterators designed to traverse
//! memory regions using fixed step sizes (strides). Unlike standard slice iterators,
//! [`NIterator`] and [`NMutIterator`] allow skipping elements (e.g., accessing every 2nd or 3rd item).
//!
//! ## Common Use Cases
//! - **Graphics & Audio:** Processing interlaced image pixels (e.g., extracting only the red channel from an RGB buffer)
//!   or handling interleaved audio channels.
//! - **Numerical Computing & Matrices:** Efficiently traversing columns in a row-major matrix layout.
//! - **Performance-Critical Code:** Minimizing branches by allowing LLVM to aggressively optimize loop structures.
//!
//! ## Safety Warranties & Invariants
//! Since these iterators operate internally with raw pointers, callers must strictly uphold
//! the conditions outlined in the `# Safety` documentation. Internally, the remaining capacity
//! is tracked via a numerical counter (`len`). The raw-pointer constructors remain unsafe:
//! callers are responsible for ensuring that every visited address is valid for the required access.

use std::marker::PhantomData;

/// An immutable iterator over a memory region with a custom step size (stride).
///
/// Yields raw pointers of type `*const T`. The lifetime `'a` ensures that the
/// underlying data source cannot be dropped or modified while this iterator exists.
///
/// # Example
/// ```rust
/// // This forces the test harness to bring your local crate into scope,
/// // automatically binding it under the name of your choice.
/// extern crate lineariterator as my_crate;
/// use my_crate::niterator::NIterator;
///
/// // Initialized array with elements to read
/// let data = [10, 20, 30, 40, 50];
///
/// // Create an iterator that yields 3 elements, skipping every second element (indices 0, 2, 4)
/// unsafe {
///     let mut iter = NIterator::new_step(data.as_ptr(), 3, 2);
///     assert_eq!(*iter.next().unwrap(), 10);
///     assert_eq!(*iter.next().unwrap(), 30);
///     assert_eq!(*iter.next().unwrap(), 50);
///     assert!(iter.next().is_none());
/// }
/// ```
#[derive(Copy, Clone)]
pub struct NIterator<'a, T> {
    ptr: *const T,
    step: isize,
    len: usize,
    _marker: PhantomData<&'a T>,
}

// Explicitly implement Send and Sync since raw pointers strip them by default.
unsafe impl<'a, T: Sync> Send for NIterator<'a, T> {}
unsafe impl<'a, T: Sync> Sync for NIterator<'a, T> {}

/// A mutable iterator over a memory region with a custom step size (stride).
///
/// Yields raw pointers of type `*mut T`. It includes an optimized [`clone_from_slice`](Self::clone_from_slice)
/// method to copy data rapidly into the target non-contiguous slots.
#[derive(Copy, Clone)]
pub struct NMutIterator<'a, T> {
    ptr: *mut T,
    step: isize,
    len: usize,
    _marker: PhantomData<&'a mut T>,
}

unsafe impl<'a, T: Send> Send for NMutIterator<'a, T> {}
unsafe impl<'a, T: Sync> Sync for NMutIterator<'a, T> {}

impl<'a, T> NIterator<'a, T> {
    /// Creates a new sequential immutable iterator (step size = 1).
    ///
    /// # Safety
    /// - `ptr` must be valid, properly aligned, and readable for the entire duration of lifetime `'a`.
    /// - The memory block stretching from `ptr` to `ptr.add(len)` must contain fully initialized objects of type `T`.
    #[inline(always)]
    pub const unsafe fn new(ptr: *const T, len: usize) -> Self {
        Self {
            ptr,
            step: 1,
            len,
            _marker: PhantomData,
        }
    }

    /// Creates a new immutable iterator with a custom step size (`step`).
    ///
    /// If a `step` of `0` is supplied, it automatically falls back to a safe step size of `1`
    /// to avoid deadlocks/infinite loops over the exact same memory address.
    ///
    /// # Safety
    /// - `ptr` must be valid for traversing `len` elements separated by `step` strides.
    /// - For every yielded element, the address at offset `index * step` must remain within the same
    ///   allocation, be properly aligned for `T`, and point to a fully initialized `T`.
    #[inline(always)]
    pub const unsafe fn new_step(ptr: *const T, len: usize, step: usize) -> Self {
        Self {
            ptr,
            step: if step == 0 { 1 } else { step as isize },
            len,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> NMutIterator<'a, T> {
    /// Creates a new sequential mutable iterator (step size = 1).
    ///
    /// # Safety
    /// - `ptr` must be uniquely valid, aligned, and writable for the entire duration of lifetime `'a`.
    /// - No other references or pointers may aliasingly read or write to this memory space concurrently
    ///   (upholding Rust's strict mutable exclusivity rules).
    #[inline(always)]
    pub const unsafe fn new(ptr: *mut T, len: usize) -> Self {
        Self {
            ptr,
            step: 1,
            len,
            _marker: PhantomData,
        }
    }

    /// Creates a new mutable iterator with a custom step size (`step`).
    ///
    /// If a `step` of `0` is supplied, it automatically falls back to a safe step size of `1`.
    ///
    /// # Safety
    /// - Every visited address must remain within the same allocation, be properly aligned for `T`,
    ///   and point to an initialized, writable `T` for lifetime `'a`.
    /// - The visited elements must be exclusively accessible for lifetime `'a`.
    #[inline(always)]
    pub const unsafe fn new_step(ptr: *mut T, len: usize, step: usize) -> Self {
        Self {
            ptr,
            step: if step == 0 { 1 } else { step as isize },
            len,
            _marker: PhantomData,
        }
    }

    /// Clones elements from `data` into the initialized elements targeted by this iterator.
    ///
    /// Stops when either the source slice or the iterator is exhausted. Existing target
    /// values are updated with `Clone::clone_from`, so their destructors and clone semantics
    /// are preserved.
    #[inline(always)]
    pub fn clone_from_slice(&mut self, data: &[T])
    where
        T: Clone,
    {
        let count = data.len().min(self.len);
        let mut dst = self.ptr;

        for src in data.iter().take(count) {
            unsafe {
                (&mut *dst).clone_from(src);
                dst = dst.wrapping_offset(self.step);
            }
        }

        self.ptr = dst;
        self.len -= count;
    }

    /// Copies elements from `data` into the elements targeted by this iterator.
    ///
    /// For contiguous destinations this uses `copy_nonoverlapping`; strided destinations
    /// are copied element by element. This method is restricted to `Copy` types.
    #[inline(always)]
    pub fn copy_from_slice(&mut self, data: &[T])
    where
        T: Copy,
    {
        let count = data.len().min(self.len);
        if count == 0 {
            return;
        }

        unsafe {
            if self.step == 1 {
                std::ptr::copy_nonoverlapping(data.as_ptr(), self.ptr, count);
                self.ptr = self.ptr.add(count);
            } else {
                let mut dst = self.ptr;
                for src in data.iter().take(count) {
                    dst.write(*src);
                    dst = dst.wrapping_offset(self.step);
                }
                self.ptr = dst;
            }
        }

        self.len -= count;
    }
}

impl<'a, T> Iterator for NIterator<'a, T> {
    type Item = *const T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let current = self.ptr;
            self.ptr = self.ptr.wrapping_offset(self.step);
            self.len -= 1;
            Some(current)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> ExactSizeIterator for NIterator<'a, T> {}

impl<'a, T> Iterator for NMutIterator<'a, T> {
    type Item = *mut T;

    #[inline(always)]
    fn next(&mut self) -> Option<Self::Item> {
        if self.len == 0 {
            None
        } else {
            let current = self.ptr;
            self.ptr = self.ptr.wrapping_offset(self.step);
            self.len -= 1;
            Some(current)
        }
    }

    #[inline(always)]
    fn size_hint(&self) -> (usize, Option<usize>) {
        (self.len, Some(self.len))
    }
}

impl<'a, T> ExactSizeIterator for NMutIterator<'a, T> {}

