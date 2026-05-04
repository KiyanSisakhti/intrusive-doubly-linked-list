//! # Manual: Circular Iterator Implementation
//!
//! This module implements a non-consuming iterator for a circular doubly linked list.
//!
//! ### Traversal Logic
//! 1. **Initialization**: The iterator stores a `cursor` (current position) and a `start`
//!    (the entry point). Both are set to the list's root.
//! 2. **Step**: On each call to `next()`, the iterator returns the node at the `cursor`.
//! 3. **Termination**: Before returning, the iterator checks if the *next* node in the
//!    chain is the `start` node. If it is, the `cursor` is set to `None`, ensuring
//!    the loop terminates after exactly one full rotation.

use core::{marker::PhantomData, ptr::NonNull};

use crate::{DoublyLinkPointer, ext::NonNullEx};

/// State management for the circular list traversal.
pub struct Iter<'a, T>
where
    T: DoublyLinkPointer<T>,
{
    /// Where we are standing right now.
    cursor: Option<NonNull<T>>,
    /// The very first person we visited, so we know when to stop.
    start: Option<NonNull<T>>,
    _marker: PhantomData<&'a T>,
}

impl<'a, T> Iter<'a, T>
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

impl<'a, T> Iterator for Iter<'a, T>
where
    T: DoublyLinkPointer<T> + 'a,
{
    type Item = &'a T;

    /// Advances the iterator and returns the next value.
    ///
    /// ### Implementation Steps:
    /// 1. Retrieve the current pointer from the `cursor`. Return `None` if empty.
    /// 2. Access the node via the `DoublyLinkPointer` trait to find its `next` neighbor.
    /// 3. Compare the neighbor to the `start` pointer:
    ///    - If they match: We have completed the circle. Set `cursor` to `None`.
    ///    - If they differ: Update `cursor` to the neighbor.
    /// 4. Return a shared reference to the node we started this step with.
    fn next(&mut self) -> Option<Self::Item> {
        let current_ptr = self.cursor?;
        let current_ref = current_ptr.unsafe_ref();
        let next_ptr = current_ref.get_next()?;

        if Some(next_ptr) == self.start {
            self.cursor = None;
        } else {
            self.cursor = Some(next_ptr);
        }

        Some(current_ref)
    }
}
