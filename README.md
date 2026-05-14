<div align="center">
<h1>Intrusive Doubly Linked-List</h1>
</div>

<div align="center">

![Test Status][git-ci]
![Lines of Code][total-lines]
![Repo Size][repo-size]


[![crate][crate-badge]][crate-link]
[![MIT licensed][license-image]][license-link]
[![Docs][docs-image]][docs-link]

</div>

---

A high-performance, **zero-allocation**, circular intrusive doubly linked list for Rust.

This crate is designed for **bare-metal (no_std)** environments where you need to manage collections of data without the overhead of a global heap allocator. By using an intrusive design, the "links" (pointers) are stored directly inside your data structures.

## Features

- **Zero Heap Allocation**: No `Box`, `Vec`, or `BTreeMap` required.
- **Bare-Metal Ready**: Works in `no_std` environments using raw pointers (`NonNull`).
- **O(1) Operations**: Push, pop, and remove operations are constant time.
- **Circular Mechanics**: Simplifies rotation and ensures the list never has a "dead end."
- **Safety Guard**: Includes a `link_state` flag to prevent a node from being corrupted by being added to two lists at once.

## Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
intrusive-doubly-list = "0.1.5"
```

## Step 1: Implement `DoublyLinkPointer`

To put your data into the list, your struct must "volunteer" fields to store the pointers. This is what makes the list **intrusive**.

```rust
use intrusive-doubly-list::DoublyLinkPointer;
use core::ptr::NonNull;

struct SensorData {
    // Intrusive links
    next: Option<NonNull<SensorData>>,
    last: Option<NonNull<SensorData>>,
    linked: bool,
    
    // Your actual data
    value: f32,
}

impl DoublyLinkPointer<SensorData> for SensorData {
    fn get_next(&self) -> Option<NonNull<SensorData>> { self.next }
    fn get_last(&self) -> Option<NonNull<SensorData>> { self.last }
    fn set_next(&mut self, next: Option<NonNull<SensorData>>) { self.next = next; }
    fn set_last(&mut self, last: Option<NonNull<SensorData>>) { self.last = last; }
    fn set_link_state(&mut self, state: bool) { self.linked = state; }
    fn is_linked(&self) -> bool { self.linked }
}
```

## Step 2: Basic Usage

In bare-metal environments, nodes often live in fixed buffers or static memory. To use them, you convert a safe mutable reference into a `NonNull<T>` pointer.

```rust
use intrusive-doubly-list::{IntrusiveDLinkList, DoublyLinkPointer};
use core::ptr::NonNull;

fn main() {
    let mut list = IntrusiveDLinkList::new();

    // 1. Create your data (e.g., on the stack)
    // IMPORTANT: The node must not be moved after being added to the list.
    let mut sensor_a = SensorData {
        next: None,
        last: None,
        linked: false,
        value: 22.5,
    };

    // 2. Convert a safe mutable reference to NonNull.
    // This creates a pointer the list can use to link the node.
    let node_ptr = NonNull::from(&mut sensor_a);

    // 3. Initialize and push
    IntrusiveDLinkList::init_node(node_ptr);
    list.push(node_ptr);

    // 4. Iterate safely
    for node in list.iter() {
        println!("Value: {}", node.value);
    }
}
```

## Advanced: Raw Operations and Pointer Aliasing

The library provides `_raw` variants for core operations: `push_raw`, `pop_raw`, and `remove_raw`. These methods take a `NonNull<IntrusiveDLinkList<T>>` instead of a standard Rust `&mut self`.

### Why use Raw Methods?

In low-level systems programming—especially when building **Intrusive Trees**, **Schedulers**, or **Async Executors**—you often face strict ownership challenges.

1. **Bypassing the Borrow Checker**: Standard Rust mutable references (`&mut T`) require exclusive access. If your list is part of a complex structure where nodes and the list container need to be accessed simultaneously via pointers, `&mut self` can be too restrictive.
2. **Avoiding Aliasing Violations**: Using `_raw` methods helps you stay compliant with Rust’s memory models (like **Stacked Borrows** or **Tree Borrows**). Converting a pointer to a reference and back can sometimes "invalidate" other existing pointers to the same memory. By staying with `NonNull`, you maintain pointer stability.
3. **Circular Dependencies**: When a node needs to remove itself from a list that it also contains a pointer to, `_raw` methods prevent the creation of conflicting mutable borrows.

### Example: Using `push_raw`

```rust
use intrusive_doubly_list::IntrusiveDLinkList;
use core::ptr::NonNull;

// Assume we have a list pointer (e.g., from a global state or tree node)
let mut list = IntrusiveDLinkList::new();
let list_ptr = NonNull::from(&mut list);

let mut data = SensorData { /* ... */ };
let node_ptr = NonNull::from(&mut data);

// Perform operation without creating a &mut reference to the list
IntrusiveDLinkList::push_raw(list_ptr, node_ptr);
```

## Important Concepts

### 1. Non-Ordered Push and Pop
This list is **not a stack**. 
- `push()` inserts the node into the circular chain relative to the current "root."
- `pop()` removes the current "root" and advances the list's internal pointer to the next neighbor.

Because the list is circular, there is no absolute "start" or "end"—only the node that the list happens to be pointing at right now. If you need a strict Last-In-First-Out (LIFO) order, a standard stack is a better choice.

### 2. Bare-Metal Safety
Since this library handles raw pointers (`NonNull<T>`), you must ensure:
1. **Memory Validity**: The data the pointer points to must remain valid as long as it is in the list. If the data is on the stack, it must not go out of scope.
2. **Initialization**: You **must** call `IntrusiveDLinkList::init_node(ptr)` before pushing a node for the first time. This ensures the node's pointers are not null and correctly point to itself.
3. **Pinning**: In many bare-metal scenarios, you should ensure your data does not move in memory once it has been linked, as the pointers inside the other nodes will become "dangling."

### 3. Why `DoublyLinkPointer`?
The trait acts as a contract. It tells the list exactly how to "stitch" your structs together. By requiring the `linked` boolean, the list can verify that a node isn't already part of another chain, preventing the most common source of memory corruption in intrusive collections.

## License

This project is licensed under the **MIT License**. See the `LICENSE` file for details.


[crate-badge]: https://img.shields.io/crates/v/intrusive-doubly-list.svg
[crate-link]: https://crates.io/crates/intrusive-doubly-list
[docs-image]: https://docs.rs/intrusive-doubly-list/badge.svg
[docs-link]: https://docs.rs/intrusive-doubly-list
[license-image]: https://img.shields.io/badge/MIT-blue.svg
[repo-size]: https://img.shields.io/github/repo-size/KiyanSisakhti/intrusive-doubly-linked-list
[total-lines]: https://aschey.tech/tokei/github/KiyanSisakhti/intrusive-doubly-linked-list
[git-ci]:https://github.com/KiyanSisakhti/intrusive-doubly-linked-list/actions/workflows/rust.yml/badge.svg?branch=main

[license-link]: #license