//! # Doubly Link Pointer Trait
//!
//! This trait is the core of the **intrusive** design. Unlike a standard list where the list
//! manages the nodes, an intrusive list requires your data structures to "volunteer" fields
//! to store the `next` and `last` pointers.
//!
//! By implementing this trait, you allow the `IntrusiveDLinkList`
//! to stitch your structs together into a circular chain without any extra memory allocations.
//!
//! ## Example Implementation
//!
//! ```rust, ignore
//! use intrusive-doubly-list::DoublyLinkPointer;
//! use core::ptr::NonNull;
//!
//! struct MyData {
//!     next: Option<NonNull<MyData>>,
//!     last: Option<NonNull<MyData>>,
//!     linked: bool,
//!     value: u32,
//! }
//!
//! impl DoublyLinkPointer<MyData> for MyData {
//!     fn get_next(&self) -> Option<NonNull<MyData>> { self.next }
//!     fn get_last(&self) -> Option<NonNull<MyData>> { self.last }
//!     
//!     fn set_next(&mut self, next: Option<NonNull<MyData>>) { self.next = next; }
//!     fn set_last(&mut self, last: Option<NonNull<MyData>>) { self.last = last; }
//!
//!     fn set_link_state(&mut self, state: bool) { self.linked = state; }
//!     fn is_linked(&self) -> bool { self.linked }
//! }
//! ```

use core::ptr::NonNull;

/// A trait defining the pointer manipulation required for an intrusive doubly linked list.
///
/// The generic parameter `T` must also implement `DoublyLinkPointer<T>`, ensuring that
/// every node in the list knows how to link to its neighbors.
pub trait DoublyLinkPointer<T>
where
    T: DoublyLinkPointer<T>,
{
    /// Returns the pointer to the next node in the circle.
    ///
    /// **Implementation Note:** This method must be a "pure" getter. It should
    /// only return the current value and must not modify the internal state
    /// of the next pointer or the node.
    fn get_next(&self) -> Option<NonNull<T>>;

    /// Returns the pointer to the previous node in the circle.
    ///
    /// **Implementation Note:** This method must be a "pure" getter. It should
    /// only return the current value and must not modify the internal state
    /// of the last pointer or the node.
    fn get_last(&self) -> Option<NonNull<T>>;

    /// Updates the pointer to the next node.
    ///
    /// # Arguments
    /// * `next_ptr` - The pointer to the node that will follow this one.
    fn set_next(&mut self, next_ptr: Option<NonNull<T>>);

    /// Updates the pointer to the previous node.
    ///
    /// # Arguments
    /// * `last_ptr` - The pointer to the node that will precede this one.
    fn set_last(&mut self, last_ptr: Option<NonNull<T>>);

    /// Sets the membership status of this node.
    ///
    /// This is used by the list to "mark" a node as owned. It is a critical
    /// safety feature to prevent a node from being added to two lists at once.
    ///
    /// # Arguments
    /// * `state` - `true` if the node is currently in a list, `false` otherwise.
    fn set_link_state(&mut self, state: bool);

    /// Returns `true` if this node is currently a member of a list.
    ///
    /// The list implementation checks this before `push` operations to avoid
    /// pointer corruption.
    fn is_linked(&self) -> bool;
}
