# lineariterator

Low-level Rust iterators for strided element traversal and fixed-width windows over contiguous memory.

The crate provides both raw-pointer iterators and reference-based wrappers. It is intended for cases where traversal patterns such as strides or sliding windows need to be expressed explicitly and with minimal abstraction overhead.

> **Status:** experimental. The raw-pointer APIs require the caller to uphold their documented safety invariants. Safe immutable windows use the standard `Iterator` trait; safe mutable overlapping windows use a small dependency-free lending iterator API.

## Features

- strided traversal over elements
- immutable and mutable raw-pointer iterators
- fixed-width sliding windows with configurable step size
- immutable reference windows over safe Rust slices
- mutable overlapping reference windows through a GAT-based lending iterator
- exact-size iteration for the standard iterator types
- no external runtime dependencies

## Modules

### `niterator`

Strided traversal over individual elements.

- `NIterator<'a, T>` yields `*const T`
- `NMutIterator<'a, T>` yields `*mut T`
- `NMutIterator::clone_from_slice()` copies values into strided destinations

Example:

```rust
use lineariterator::niterator::NIterator;

let data = [10, 20, 30, 40, 50];

unsafe {
    let mut iter = NIterator::new_step(data.as_ptr(), 3, 2);

    assert_eq!(*iter.next().unwrap(), 10);
    assert_eq!(*iter.next().unwrap(), 30);
    assert_eq!(*iter.next().unwrap(), 50);
    assert!(iter.next().is_none());
}
```

### `slice_ptr_iterator`

Low-level fixed-width window iteration using raw slice pointers.

- `SlicePtrIterator<'a, T>` yields `*const [T]`
- `SliceMutPtrIterator<'a, T>` yields `*mut [T]`
- configurable window width and step size

Example:

```rust
use lineariterator::slice_ptr_iterator::SlicePtrIterator;

let data = [1, 2, 3, 4, 5];

unsafe {
    let mut iter = SlicePtrIterator::new(data.as_ptr(), 3, data.len());

    assert_eq!(&*iter.next().unwrap(), &[1, 2, 3]);
    assert_eq!(&*iter.next().unwrap(), &[2, 3, 4]);
    assert_eq!(&*iter.next().unwrap(), &[3, 4, 5]);
    assert!(iter.next().is_none());
}
```

### `slice_ref_iterator`

Reference-based wrappers around the pointer-window iterator.

- `SliceRefIterator<'a, T>` yields immutable slice windows
- `SliceMutRefIterator<'a, T>` yields mutable windows through `LendingIterator`; each window borrows the iterator, preventing simultaneous overlapping `&mut` references

Immutable example:

```rust
use lineariterator::slice_ref_iterator::SliceRefIterator;

let data = [10, 20, 30, 40, 50];
let windows: Vec<_> = SliceRefIterator::new(&data, 2).collect();

assert_eq!(windows[0], &[10, 20]);
assert_eq!(windows[1], &[20, 30]);
assert_eq!(windows[2], &[30, 40]);
assert_eq!(windows[3], &[40, 50]);
```

Mutable overlapping windows use the lending API:

```rust
use lineariterator::slice_ref_iterator::{LendingIterator, SliceMutRefIterator};

let mut data = [1, 2, 3, 4];
let mut windows = SliceMutRefIterator::new(&mut data, 2);

while let Some(window) = windows.next() {
    window[0] += 10;
}

assert_eq!(data, [11, 12, 13, 4]);
```

## Safety

The raw-pointer iterators are intentionally low-level. Their constructors are `unsafe` because the caller must guarantee that all addresses visited by the iterator are valid for the required access and remain valid for the iterator's lifetime.

Special care is required when using the raw mutable pointer iterators:

- aliased mutable access must not be created
- pointer arithmetic must remain within the allocation
- all yielded pointers must only be dereferenced while valid
- overlapping mutable windows must never be converted into simultaneously live mutable references

`SliceMutRefIterator` keeps overlapping mutable windows safe by implementing `LendingIterator` rather than `Iterator`. The lifetime of each returned `&mut [T]` is tied to the borrow of the iterator, so the iterator cannot advance while that window is still in use.

## Testing

Run the test suite with:

```bash
cargo test
```

Run Clippy with warnings denied:

```bash
cargo clippy -- -D warnings
```

Or use the project helper:

```bash
just build
```

## Design goals

- explicit traversal semantics
- small implementation
- predictable iterator state
- minimal dependencies
- clear separation between raw-pointer primitives and safe wrappers
- safety contracts documented at every unsafe boundary
- consistent test layout: tests live under `tests/`, mirror the relative `src/` hierarchy where relevant, and use the source filename with a `_test.rs` suffix

## License

Apache-2.0
