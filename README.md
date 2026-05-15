<div align="center">
  <h1>Intrusive Doubly Linked List</h1>
  <p><strong>A high-performance, zero-allocation, circular intrusive doubly linked list for Rust.</strong></p>

  [![crate][crate-badge]][crate-link]
  ![Test Status][git-ci]
  ![Lines of Code][total-lines]

  ![Repo Size][repo-size]
  [![MIT licensed][license-image]][license-link]
  [![Docs][docs-image]][docs-link]
</div>

---

## 📖 Table of Contents
- [Introduction](#-introduction)
- [Why Intrusive?](#-why-intrusive)
- [Key Features](#-key-features)
- [Installation](#-installation)
- [Quick Start](#-quick-start)
  - [1. Define Your Node](#1-define-your-node)
  - [2. Basic Operations](#2-basic-operations)
- [⚠️ Important: Not a Stack or Queue (LIFO/FIFO)](#️-important-not-a-stack-or-queue-lifofifo)
- [Advanced: Avoiding Aliasing with Raw API](#-advanced-avoiding-aliasing-with-raw-api)
- [Safety & Invariants](#-safety--invariants)
- [Performance](#-performance)
- [Contributing](#-contributing)
- [License](#-license)

---

## 🌟 Introduction

`intrusive-doubly-list` provides a robust, `no_std` compatible, circular intrusive doubly linked list. Unlike standard collections, it allows you to manage memory manually, making it ideal for:
- **Embedded Systems** (where heap is scarce or non-existent).
- **Kernel Development** (tracking tasks, memory pages).
- **Custom Memory Managers** (slab allocators, buddy systems).
- **High-Performance Schedulers**.

---

## 🤔 Why Intrusive?

In a standard `std::collections::LinkedList<T>`, each insertion requires a heap allocation for a wrapper node:
```rust
// Standard: List owns a Node which wraps your Data
// [List] -> [Node { Data, Next, Last }] -> [Node { Data, Next, Last }]
```

In an **intrusive** list, the links are stored **inside** your data structure:
```rust
// Intrusive: Data *is* the Node
// [List] -> [Data { Next, Last, ...fields }] -> [Data { Next, Last, ...fields }]
```

### Advantages:
1. **Zero Allocation**: No `Box` or `Vec` needed during list operations.
2. **Locality**: Data and links are stored together, improving cache hits.
3. **Flexible Ownership**: A single object can be part of a list while being owned by something else (e.g., a static array or a custom pool).
4. **O(1) Removal**: You can remove a node from the list in constant time if you have a pointer to the node itself, without searching the list.

---

## ✨ Key Features

- **Circular Design**: The list is circular (tail connects back to head), simplifying many edge cases and enabling infinite rotation.
- **Ownership Validation**: Integrated `link_state` flag prevents common bugs where a node is accidentally added to two lists at once.
- **Multiple APIs**:
    - **Safe-ish Wrapper (`IntrusiveDLinkList`)**: Uses standard Rust references for ease of use.
    - **Raw API (`IntrusiveDLinkListRaw`)**: Operates entirely on `NonNull` pointers, strictly adhering to Miri's memory models (Tree/Stacked Borrows) by avoiding reference aliasing.
- **Iterator Support**: Safe, lifetime-bounded iterators (`iter()` and `iter_mut()`).
- **Completely `no_std`**: No standard library or allocator required.

---

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
intrusive-doubly-list = "0.2.0"
```

---

## 🚀 Quick Start

### 1. Define Your Node
Implement the `DoublyLinkPointer` trait for your struct.

```rust
use intrusive_doubly_list::DoublyLinkPointer;
use core::ptr::NonNull;

struct Task {
    name: &'static str,
    // Intrusive links
    next: Option<NonNull<Task>>,
    last: Option<NonNull<Task>>,
    linked: bool,
}

impl DoublyLinkPointer<Task> for Task {
    fn get_next(&self) -> Option<NonNull<Task>> { self.next }
    fn get_last(&self) -> Option<NonNull<Task>> { self.last }
    fn set_next(&mut self, next: Option<NonNull<Task>>) { self.next = next; }
    fn set_last(&mut self, last: Option<NonNull<Task>>) { self.last = last; }
    fn set_link_state(&mut self, state: bool) { self.linked = state; }
    fn is_linked(&self) -> bool { self.linked }
}
```

### 2. Basic Operations

```rust
use intrusive_doubly_list::IntrusiveDLinkList;
use core::ptr::NonNull;

fn main() {
    let mut list = IntrusiveDLinkList::new();

    // Nodes can live on the stack, but they MUST NOT MOVE while linked
    let mut t1 = Task { name: "Task 1", next: None, last: None, linked: false };
    let mut t2 = Task { name: "Task 2", next: None, last: None, linked: false };

    let p1 = NonNull::from(&mut t1);
    let p2 = NonNull::from(&mut t2);

    // Initialize nodes (critical step!)
    IntrusiveDLinkList::init_node(p1);
    IntrusiveDLinkList::init_node(p2);

    // Push into the list
    list.push(p1);
    list.push(p2);

    println!("List length: {}", list.len());

    // Iterate
    for task in list.iter() {
        println!("Running: {}", task.name);
    }

    // Remove a specific node
    list.remove(p1);

    // Pop the current root
    if let Some(node_ptr) = list.pop() {
        let node = unsafe { node_ptr.as_ref() };
        println!("Popped: {}", node.name);
    }
}
```

---

## ⚠️ Important: Not a Stack or Queue (LIFO/FIFO)

Unlike `std::collections::VecDeque`, this list is **circular** and does not have a concept of "front" or "back" in the traditional linear sense.

- **`push()`** inserts the node into the circular chain relative to the current "root."
- **`pop()`** removes the **current root** and then advances the root pointer to the next node in the circle.

**This is not a LIFO (Stack) or FIFO (Queue).** If you push three elements and then pop three elements, the order of return depends on the internal circular rotation. If you require a specific order, you must manage the root pointer or use a non-circular list implementation.

---

## 🛡️ Advanced: Avoiding Aliasing with Raw API

Rust's borrow checker is very strict about mutable references (`&mut T`). In complex intrusive structures (like a tree where every node is also in a list), creating a `&mut list` can sometimes invalidate existing pointers to nodes.

To solve this, crates provide `IntrusiveDLinkListRaw` and the `DoublyLinkPointerRaw` trait. These operate entirely on raw pointers, making them safe for use with **Miri** and complex pointer-heavy code.

```rust
use intrusive_doubly_list::{IntrusiveDLinkListRaw, DoublyLinkPointerRaw};
use core::ptr::NonNull;

// 1. Implement DoublyLinkPointerRaw (uses raw pointers in methods)
impl DoublyLinkPointerRaw<Task> for Task {
    fn get_next(p: NonNull<Self>) -> Option<NonNull<Self>> { unsafe { (*p.as_ptr()).next } }
    // ... implement other methods using unsafe raw pointer access ...
    # fn get_last(p: NonNull<Self>) -> Option<NonNull<Self>> { unsafe { (*p.as_ptr()).last } }
    # fn set_next(p: NonNull<Self>, n: Option<NonNull<Self>>) { unsafe { (*p.as_ptr()).next = n; } }
    # fn set_last(p: NonNull<Self>, l: Option<NonNull<Self>>) { unsafe { (*p.as_ptr()).last = l; } }
    # fn set_link_state(p: NonNull<Self>, s: bool) { unsafe { (*p.as_ptr()).linked = s; } }
    # fn is_linked(p: NonNull<Self>) -> bool { unsafe { (*p.as_ptr()).linked } }
}

fn main() {
    let mut list = IntrusiveDLinkListRaw::<Task>::new();
    let list_ptr = NonNull::from(&mut list);

    let mut t1 = Task { /* ... */ };
    let p1 = NonNull::from(&mut t1);

    IntrusiveDLinkListRaw::init_node(p1);
    IntrusiveDLinkListRaw::push(list_ptr, p1);
    
    let popped = IntrusiveDLinkListRaw::pop(list_ptr);
}
```

---

## ⚠️ Safety & Invariants

Working with intrusive lists requires care. By using this crate, you agree to uphold the following:

1.  **Stability**: Once a node is added to a list, it **must not be moved** in memory (e.g., don't push a stack-allocated node into a `Vec` or return it from a function).
2.  **Lifetime**: A node must not be dropped/deallocated while it is still a member of a list.
3.  **Initialization**: Always call `init_node` before the first time you `push` a node.
4.  **Single Membership**: A node cannot be in two lists at the same time. The `link_state` flag helps catch this, but the trait implementation must correctly manage it.

---

## ⚡ Performance

- **`push`**: $O(1)$
- **`pop`**: $O(1)$
- **`remove`**: $O(1)$
- **`len`**: $O(1)$ (tracked via internal counter)
- **Memory**: 0 extra bytes allocated on heap. The list structure itself is only 16-24 bytes (on 64-bit).

---

## 🤝 Contributing

Contributions are welcome! Whether it's optimization, documentation, or new features, feel free to open an issue or a PR.

1. Fork the repo.
2. Create your feature branch.
3. Ensure all tests pass (including Miri).
4. Submit a PR.

```bash
# Run tests
cargo test
# Run Miri
cargo miri test
```

---

## ⚖️ License

Licensed under the **MIT License**. See [LICENSE](LICENSE) for details.




[crate-badge]: https://img.shields.io/crates/v/intrusive-doubly-list.svg
[crate-link]: https://crates.io/crates/intrusive-doubly-list
[docs-image]: https://docs.rs/intrusive-doubly-list/badge.svg
[docs-link]: https://docs.rs/intrusive-doubly-list
[license-image]: https://img.shields.io/badge/MIT-blue.svg
[repo-size]: https://img.shields.io/github/repo-size/KiyanSisakhti/intrusive-doubly-linked-list
[total-lines]: https://aschey.tech/tokei/github/KiyanSisakhti/intrusive-doubly-linked-list
[git-ci]:https://github.com/KiyanSisakhti/intrusive-doubly-linked-list/actions/workflows/rust.yml/badge.svg?branch=main

[license-link]: #license
