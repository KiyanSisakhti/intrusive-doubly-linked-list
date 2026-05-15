//! # Doubly Link Pointer Raw Trait
//!
//! This trait is the core of the **raw intrusive** design. Unlike a standard list where the list
//! manages the nodes, an intrusive list requires your data structures to "volunteer" fields
//! to store the `next` and `last` pointers.
//!
//! By implementing this trait using raw `NonNull` pointers, you allow the `IntrusiveDLinkListRaw`
//! to stitch your structs together into a circular chain without triggering aliasing violations
//! in strict memory models (like Miri's Stacked/Tree Borrows).
//!
//! ## Example Implementation
//!
//! ```rust, ignore
//! use intrusive_doubly_list::DoublyLinkPointerRaw;
//! use core::ptr::NonNull;
//!
//! struct MyData {
//!     next: Option<NonNull<MyData>>,
//!     last: Option<NonNull<MyData>>,
//!     linked: bool,
//!     value: u32,
//! }
//!
//! impl DoublyLinkPointerRaw<MyData> for MyData {
//!     fn get_next(node_ptr: NonNull<MyData>) -> Option<NonNull<MyData>> {
//!         unsafe { (*node_ptr.as_ptr()).next }
//!     }
//!     fn get_last(node_ptr: NonNull<MyData>) -> Option<NonNull<MyData>> {
//!         unsafe { (*node_ptr.as_ptr()).last }
//!     }
//!     
//!     fn set_next(mut node_ptr: NonNull<MyData>, next_ptr: Option<NonNull<MyData>>) {
//!         unsafe { (*node_ptr.as_ptr()).next = next_ptr; }
//!     }
//!     fn set_last(mut node_ptr: NonNull<MyData>, last_ptr: Option<NonNull<MyData>>) {
//!         unsafe { (*node_ptr.as_ptr()).last = last_ptr; }
//!     }
//!
//!     fn set_link_state(mut node_ptr: NonNull<MyData>, state: bool) {
//!         unsafe { (*node_ptr.as_ptr()).linked = state; }
//!     }
//!     fn is_linked(node_ptr: NonNull<MyData>) -> bool {
//!         unsafe { (*node_ptr.as_ptr()).linked }
//!     }
//! }
//!

use core::ptr::NonNull;

/// A trait defining raw pointer manipulation required for an intrusive doubly linked list.
///
/// Instead of taking &self or &mut self, all methods take a `NonNull<T>` to completely
/// bypass Rust's reference aliasing rules, making it fully compatible with Miri.
pub trait DoublyLinkPointerRaw<T>
where
    T: DoublyLinkPointerRaw<T>,
{
    /// Returns the pointer to the next node in the circle.
    ///
    /// # Safety
    /// The provided node_ptr must be valid, aligned, and dereferenceable.
    fn get_next(node_ptr: NonNull<T>) -> Option<NonNull<T>>;

    //// Returns the pointer to the previous node in the circle.
    ///
    /// # Safety
    /// The provided `node_ptr` must be valid, aligned, and dereferenceable.
    fn get_last(node_ptr: NonNull<T>) -> Option<NonNull<T>>;

    /// Updates the pointer to the next node.
    ///
    /// # Safety
    /// The provided `node_ptr` must be valid, aligned, and point to a mutable memory location.
    fn set_next(node_ptr: NonNull<T>, next_ptr: Option<NonNull<T>>);

    /// Updates the pointer to the previous node.
    ///
    /// # Safety
    /// The provided `node_ptr` must be valid, aligned, and point to a mutable memory location.
    fn set_last(node_ptr: NonNull<T>, last_ptr: Option<NonNull<T>>);

    /// Sets the membership status of this node.
    ///
    /// # Safety
    /// The provided `node_ptr` must be valid, aligned, and point to a mutable memory location.
    fn set_link_state(node_ptr: NonNull<T>, state: bool);

    /// Returns `true` if this node is currently a member of a list.
    ///
    /// # Safety
    /// The provided `node_ptr` must be valid, aligned, and dereferenceable.
    fn is_linked(node_ptr: NonNull<T>) -> bool;
}
