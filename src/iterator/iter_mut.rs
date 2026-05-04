//! # Manual: Circular Mutable Iterator Implementation
//!
//! This module implements a non-consuming, mutable iterator for a circular doubly linked list.
//!
//! ### Traversal Logic
//! 1. **Initialization**: The iterator is seeded with a `cursor` and a `start` pointer.
//!    Both usually point to the list's "root".
//! 2. **Iteration**: Each call to `next()` yields a unique mutable reference to the current node.
//! 3. **Termination**: The iterator detects the completion of a full cycle by checking
//!    if the node following the current one is the `start` node. If so, it exhausts
//!    itself by setting the `cursor` to `None`.
//!
//! ### Safety and Invariants
//! This iterator relies on the caller ensuring that the list structure remains valid
//! (no re-linking or dropping nodes) while the iterator exists.

use crate::DoublyLinkPointer;
use crate::ext::NonNullEx;
use core::marker::PhantomData;
use core::ptr::NonNull;

/// A mutable iterator over the nodes of an `IntrusiveDLinkList`.
pub struct IterMut<'a, T>
where
    T: DoublyLinkPointer<T>,
{
    /// Where we are standing right now.
    cursor: Option<NonNull<T>>,
    /// The very first person we visited, so we know when to stop.
    start: Option<NonNull<T>>,
    _marker: PhantomData<&'a mut T>,
}

impl<'a, T> IterMut<'a, T>
where
    T: DoublyLinkPointer<T> + 'a,
{
    /// Internal constructor for the iterator.
    ///
    /// # Arguments
    /// * `root` - The starting node of the list. If `None`, the iterator
    ///   will be immediately exhausted.
    pub fn new(root: Option<NonNull<T>>) -> Self {
        Self {
            cursor: root,
            start: root,
            _marker: PhantomData,
        }
    }
}

impl<'a, T> Iterator for IterMut<'a, T>
where
    T: DoublyLinkPointer<T> + 'a,
{
    type Item = &'a mut T;

    /// Advances the iterator and returns the next mutable reference.
    ///
    /// ### Implementation Steps:
    /// 1. Retrieve the current raw pointer from the `cursor`.
    /// 2. Convert the pointer to a mutable reference using `unsafe_mut_ref`.
    /// 3. Peek at the `next` pointer of the current node.
    /// 4. If the `next` pointer matches our `start` pointer, we have traversed the
    ///    entire circle; set the `cursor` to `None`.
    /// 5. Otherwise, advance the `cursor` to the `next` pointer.
    /// 6. Return the mutable reference.
    fn next(&mut self) -> Option<Self::Item> {
        let mut current_ptr = self.cursor?;
        // SAFETY: The iterator's lifetime 'a is tied to the list's mutable borrow.
        let current_ref = current_ptr.unsafe_mut_ref();
        let next_ptr = current_ref.get_next()?;

        if Some(next_ptr) == self.start {
            self.cursor = None;
        } else {
            self.cursor = Some(next_ptr);
        }

        Some(current_ref)
    }
}
