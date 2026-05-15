//! # Intrusive Doubly Linked List
//!
//! An efficient, zero-allocation, circular intrusive doubly linked list implementation.
//!
//! This crate provides two main flavors of intrusive lists, catering to different safety and performance needs:
//!
//! 1.  [`IntrusiveDLinkList`]: A standard Rust-style wrapper that uses `&mut self` for operations.
//! 2.  [`IntrusiveDLinkListRaw`]: A raw pointer-based implementation that avoids creating intermediate
//!     references to the list structure, ensuring compatibility with strict memory models like Miri's Tree Borrows.
//!
//! ## Features
//! - **Zero Allocation**: Nodes are managed by the caller; the list itself performs no heap allocations.
//! - **Circular Structure**: Supports seamless rotation. Note that this means the list does not follow
//!   strict LIFO (Stack) or FIFO (Queue) ordering.
//! - **Ownership Validation**: Uses a `link_state` flag to prevent nodes from being inserted into multiple lists simultaneously.
//! - **Lifetime-Bounded Iterators**: Provides safe iteration over nodes stored in the list.
//!
//! ## Example: Standard API (`IntrusiveDLinkList`)
//!
//! Recommended for most high-level use cases where you can easily manage exclusive access to the list.
//!
//! ```rust
//! use intrusive_doubly_list::{IntrusiveDLinkList, DoublyLinkPointer};
//! use core::ptr::NonNull;
//!
//! struct MyNode {
//!     next: Option<NonNull<MyNode>>,
//!     last: Option<NonNull<MyNode>>,
//!     linked: bool,
//!     value: i32,
//! }
//!
//! impl DoublyLinkPointer<MyNode> for MyNode {
//!     fn get_next(&self) -> Option<NonNull<MyNode>> { self.next }
//!     fn get_last(&self) -> Option<NonNull<MyNode>> { self.last }
//!     fn set_next(&mut self, next: Option<NonNull<MyNode>>) { self.next = next; }
//!     fn set_last(&mut self, last: Option<NonNull<MyNode>>) { self.last = last; }
//!     fn set_link_state(&mut self, state: bool) { self.linked = state; }
//!     fn is_linked(&self) -> bool { self.linked }
//! }
//!
//! let mut list = IntrusiveDLinkList::new();
//! let mut node_data = MyNode { next: None, last: None, linked: false, value: 42 };
//! let node_ptr = NonNull::from(&mut node_data);
//!
//! IntrusiveDLinkList::init_node(node_ptr);
//! list.push(node_ptr);
//!
//! assert_eq!(list.len(), 1);
//! ```
//!
//! ## Example: Raw API (`IntrusiveDLinkListRaw`)
//!
//! Essential for complex data structures (like task schedulers) where creating a `&mut` reference
//! to the list would trigger aliasing violations.
//!
//! ```rust
//! use intrusive_doubly_list::{IntrusiveDLinkListRaw, DoublyLinkPointerRaw};
//! use core::ptr::NonNull;
//!
//! struct RawNode {
//!     next: Option<NonNull<RawNode>>,
//!     last: Option<NonNull<RawNode>>,
//!     linked: bool,
//! }
//!
//! impl DoublyLinkPointerRaw<RawNode> for RawNode {
//!     fn get_next(p: NonNull<Self>) -> Option<NonNull<Self>> { unsafe { (*p.as_ptr()).next } }
//!     fn get_last(p: NonNull<Self>) -> Option<NonNull<Self>> { unsafe { (*p.as_ptr()).last } }
//!     fn set_next(p: NonNull<Self>, n: Option<NonNull<Self>>) { unsafe { (*p.as_ptr()).next = n; } }
//!     fn set_last(p: NonNull<Self>, l: Option<NonNull<Self>>) { unsafe { (*p.as_ptr()).last = l; } }
//!     fn set_link_state(p: NonNull<Self>, s: bool) { unsafe { (*p.as_ptr()).linked = s; } }
//!     fn is_linked(p: NonNull<Self>) -> bool { unsafe { (*p.as_ptr()).linked } }
//! }
//!
//! let mut list = IntrusiveDLinkListRaw::new();
//! let list_ptr = NonNull::from(&mut list);
//!
//! let mut node = RawNode { next: None, last: None, linked: false };
//! let node_ptr = NonNull::from(&mut node);
//!
//! IntrusiveDLinkListRaw::init_node(node_ptr);
//! IntrusiveDLinkListRaw::push(list_ptr, node_ptr);
//! ```
//!
//! ## Safety
//! This crate uses `unsafe` internally to manage raw pointers. Users must ensure that:
//! 1. Nodes are not dropped while they are still members of a list.
//! 2. Nodes are not moved in memory once they are linked.
//! 3. The `link_state` is correctly managed via the trait implementation to prevent multiple insertions.
//! 4. For `IntrusiveDLinkListRaw`, the caller must ensure that no exclusive references to the list or nodes
//!    overlap during operations in a way that violates Rust's memory model.

#![no_std]

mod dlink_list;
mod dlink_list_raw;
mod doubly_link_pointer;
mod doubly_link_pointer_raw;
mod ext;
mod iterator;

pub use dlink_list::IntrusiveDLinkList;
pub use dlink_list_raw::IntrusiveDLinkListRaw;
pub use doubly_link_pointer::DoublyLinkPointer;
pub use doubly_link_pointer_raw::DoublyLinkPointerRaw;
