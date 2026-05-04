//! # Intrusive Doubly Linked List
//!
//! An efficient, zero-allocation, circular intrusive doubly linked list implementation.
//!
//! ## Features
//! - **Zero Allocation**: Nodes are managed by the caller; the list itself performs no heap allocations.
//! - **Circular Structure**: Supports seamless rotation and circular traversal.
//! - **Ownership Validation**: Uses a `link_state` flag to prevent nodes from being inserted into multiple lists simultaneously.
//! - **Lifetime-Bounded Iterators**: Provides safe iteration over nodes stored in the list.
//!
//! ## Design
//! In an intrusive list, the "links" (the next and previous pointers) are stored inside the
//! data structure itself. This requires the data type to implement the [`DoublyLinkPointer`] trait.
//!
//! ## Example
//!
//! ```rust, ignore
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
//! // Usage:
//! let mut list = IntrusiveDLinkList::new();
//!
//! // Assume we have a Box or a pinned stack allocation
//! let mut node = Box::new(MyNode {
//!     next: None, last: None, linked: false, value: 42
//! });
//! let node_ptr = NonNull::new(Box::into_raw(node)).unwrap();
//!
//! // Initialize and push
//! IntrusiveDLinkList::init_node(node_ptr);
//! list.push(node_ptr);
//!
//! assert_eq!(list.len(), 1);
//! ```
//!
//! ## Safety
//! This crate uses `unsafe` internally to manage raw pointers. Users must ensure that:
//! 1. Nodes are not dropped while they are still members of a list.
//! 2. Node pointers provided to the list are valid and properly aligned.
//! 3. The `link_state` is correctly managed via the trait implementation to prevent multiple insertions.

#![no_std]

mod dlink_list;
mod doubly_link_pointer;
mod ext;
mod iterator;

pub use dlink_list::IntrusiveDLinkList;
pub use doubly_link_pointer::DoublyLinkPointer;
