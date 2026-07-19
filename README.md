# lineariterator

Contiguous and non-contiguous memory layouts are iterated via hardware-optimized strides.
Low-level raw pointer performance bridges directly into safe Rust abstractions.
Zero runtime overhead and maximum LLVM auto-vectorization are guaranteed across all operations.

## Architectural Overview

                      +------------------------+
                      |     Standard Slice     |
                      +-----------+------------+
                                  |
                  +---------------+---------------+
                  |                               |
                  v                               v
      +-----------------------+       +-----------------------+
      |      niterator        |       |  slice_ptr_iterator   |
      |                       |       |                       |
      |  - Stride / Step      |       |  - Fixed-width window |
      |  - Points to T        |       |  - Points to [T]      |
      +-----------------------+       +-----------+-----------+
                                                  |
                                                  | Wrapped by (Zero Cost)
                                                  v
                                      +-----------------------+
                                      |  slice_ref_iterator   |
                                      |                       |
                                      |  - Yields &[T] / &mut |
                                      +-----------------------+

### 1. Zero-Branch Hot Paths (windows_left)
Memory address limits are traditionally compared at runtime via (self.ptr >= self.end). This mechanism forces defensive compiler boundaries and hurts hardware pipeline prediction layout states.
Total iteration capacity is calculated exactly once during constructor initialization phase blocks. The hot path loop (next()) checks against an integer zero (windows_left == 0). Branching stalls collapse into single-cycle pipeline-friendly decrements.

### 2. Strict Logic Isolation
Traversal mechanics live exclusively in raw pointer structures (slice_ptr_iterator). Index offsets, custom strides, and size calculations are centralized here. Safe reference wrappers (slice_ref_iterator) compose these pointer structures as inner elements. Raw types convert into native references at zero runtime cost.

### 3. LLVM Register Pinning and ZST Proofing
* Unchecked Asserts: The reference wrappers utilize core::hint::assert_unchecked internally. This informs LLVM that inner pointer addresses cannot evaluate to null. Redundant fallback branches are completely stripped from compiled assembly.
* Zero-Sized Types: Iteration states are tracked via numerical counts rather than absolute addresses. Types with a size of zero (such as ()) iterate correctly without triggering infinite loop traps.

---

## Module Breakdown and Documentation

### 1. niterator - Non-contiguous Stride Traversal
Multidimensional arrays, interlaced audio channels, and matrix layouts use this module. It allows moving through element blocks with customized step sizes.

* NIterator<'a, T>: Yields raw immutable element pointers (*const T).
* NMutIterator<'a, T>: Yields raw mutable element pointers (*mut T). Includes a fast .clone_from_slice(&[T]) routine which uses sequential memcpy when step == 1.

### 2. slice_ptr_iterator - Low-Level Pointer Windows
Chunked slice pointers are yielded from an allocated block without safe conversions.

* SlicePtrIterator<'a, T>: Yields raw slice layout pointers (*const [T]).
* SliceMutPtrIterator<'a, T>: Yields mutable raw slice layout pointers (*mut [T]).

### 3. slice_ref_iterator - Safe Reference Windows
Underlying slice_ptr_iterator elements are wrapped at zero runtime compilation cost.

* SliceRefIterator<'a, T>: Yields safe immutable slice segments (&'a [T]).
* SliceMutRefIterator<'a, T>: Yields safe disjoint mutable slice segments (&'a mut [T]).

---

## Usage Examples

### Stride Iteration (niterator)
```rust
use lineariterator::niterator::NIterator;

let data = [10, 20, 30, 40, 50];

unsafe {
    // Read 3 elements, skipping every second element (stride step = 2)
    let mut iter = NIterator::new_step(data.as_ptr(), 3, 2);

    assert_eq!(*iter.next().unwrap(), 10);
    assert_eq!(*iter.next().unwrap(), 30);
    assert_eq!(*iter.next().unwrap(), 50);
    assert!(iter.next().is_none());
}
```

### Sliding Reference Windows (slice_ref_iterator)
```rust
use lineariterator::slice_ref_iterator::SliceRefIterator;

let data = [10, 20, 30, 40, 50];
// Window width = 2, Step stride = 1
let mut iter = SliceRefIterator::new(&data, 2);

assert_eq!(iter.next().unwrap(), &[10, 20]);
assert_eq!(iter.next().unwrap(), &[20, 30]);
assert_eq!(iter.next().unwrap(), &[30, 40]);
assert_eq!(iter.next().unwrap(), &[40, 50]);
assert!(iter.next().is_none());
```

---

## Critical Safety Invariants

Crate mechanics work directly at the raw metal layer of memory. These invariants must be respected to guarantee safe execution paths:

### Mutable Overlapping Overrides
Ensure your step stride size matches or exceeds window width (step >= width) when initializing custom configurations (new_step) under SliceMutRefIterator.

Consecutive elements will overlap in memory if step < width (such as the default sliding setup inside SliceMutRefIterator::new which defaults to step = 1).
* The Rule: The yielded mutable reference must be completely dropped before calling .next() again.
* The Violation: Storing multiple overlapping mutable windows concurrently violates Rust's mutable exclusivity law, causing instant Undefined Behavior (UB).

---

## Performance Profiling

ExactSizeIterator and FusedIterator traits are implemented natively across all types. Methods like .count() or .size_hint() run at constant O(1) time complexity.
Sequential element loop evaluation is skipped completely.

Benchmark against the standard library via:
```bash
cargo bench
```

Execute structural memory validation tests via:
```bash
cargo test
```
