//! # Intrusive Doubly Linked List Implementation
//!
//! This module provides the [`IntrusiveDLinkList`] struct, which manages a circular
//! doubly linked list of nodes.
//!
//! ## Architecture
//! - **Intrusive**: The list doesn't allocate memory for nodes. Instead, it relies on
//!   the data types implementing [`DoublyLinkPointer`] to provide storage for pointers.
//! - **Circular**: The list is circular; the "tail" points back to the "root". This
//!   simplifies many operations like `push` and `pop` to constant time O(1).
//! - **Safety**: While the implementation uses `unsafe` pointers internally, it exposes
//!   a safer API by checking the `link_state` of nodes to prevent corruption caused by
//!   inserting the same node into multiple lists.
//!
//! ## Key Invariants
//! 1. A node is considered "initialized" if its next and last pointers point to itself.
//! 2. The `root_pointer` always points to the "head" of the list.
//! 3. If `len > 0`, `root_pointer` must be `Some`.
//! 4. A node's `is_linked` state must accurately reflect whether it is part of a list
//!    to prevent double-insertion bugs.

use crate::{
    DoublyLinkPointer,
    ext::NonNullEx,
    iterator::{iter::Iter, iter_mut::IterMut},
};
use core::ptr::NonNull;

pub struct IntrusiveDLinkList<T>
where
    T: DoublyLinkPointer<T>,
{
    root_pointer: Option<NonNull<T>>,
    len: usize,
}

impl<T> IntrusiveDLinkList<T>
where
    T: DoublyLinkPointer<T>,
{
    /// Creates a new, empty intrusive doubly linked list.
    ///
    /// This operation is O(1) and does not perform any heap allocations.
    pub fn new() -> Self {
        Self {
            root_pointer: None,
            len: 0,
        }
    }

    /// Returns an iterator over the nodes in the list.
    ///
    /// The iterator starts at the head of the list and follows the 'next' pointers
    /// until it has traversed the entire circle.
    pub fn iter(&self) -> Iter<'_, T> {
        Iter::new(self.root_pointer)
    }

    /// Returns a mutable iterator over the nodes in the list.
    ///
    /// The iterator starts at the head of the list and follows the 'next' pointers
    /// until it has traversed the entire circle.
    pub fn iter_mut(&mut self) -> IterMut<'_, T> {
        IterMut::new(self.root_pointer)
    }

    /// Returns the number of nodes currently in the list.
    pub fn len(&self) -> usize {
        self.len
    }

    /// Returns `true` if the list contains no nodes.
    pub fn is_empty(&self) -> bool {
        self.len == 0
    }

    /// Prepares a node for insertion by making it point to itself.
    ///
    /// In a circular intrusive list, a node must be "self-aware" before it can be
    /// linked to others. This function sets both the `next` and `last` pointers
    /// of the node to its own address.
    ///
    /// # Safety
    /// The `node_ptr` must be valid and point to a memory location that can be modified.
    pub fn init_node(mut node_ptr: NonNull<T>) {
        // let cloned_ptr = node_ptr.clone();
        let node_ref = node_ptr.unsafe_mut_ref();

        node_ref.set_next(Some(node_ptr));
        node_ref.set_last(Some(node_ptr));
    }

    /// Inserts a node into the circular list.
    ///
    /// **Note:** This is not a stack operation. The node is inserted into the
    /// circular structure relative to the current `root_pointer`. It does not
    /// guarantee a Last-In-First-Out (LIFO) order.
    ///
    /// Returns `false` if the node is already linked in a list.
    pub fn push(&mut self, mut node_ptr: NonNull<T>) -> bool {
        if Self::is_node_linked(node_ptr) {
            return false;
        }

        if let Some(root) = self.root_pointer {
            Self::link_to(node_ptr, root);
        } else {
            self.root_pointer = Some(node_ptr);
        }
        self.len += 1;
        node_ptr.unsafe_mut_ref().set_link_state(true);
        true
    }

    /// A raw version of `push` that operates on a `NonNull` pointer to the list.
    ///
    /// # Safety
    /// The caller must ensure that `list_ptr` is valid and that no other mutable
    /// references to the list exist during this operation. This is useful in
    /// complex pointer-based structures where using standard Rust borrows
    /// would trigger aliasing violations.
    pub fn push_raw(
        mut list_ptr: NonNull<IntrusiveDLinkList<T>>,
        mut node_ptr: NonNull<T>,
    ) -> bool {
        if Self::is_node_linked(node_ptr) {
            return false;
        }
        if let Some(root) = list_ptr.unsafe_ref().root_pointer {
            Self::link_to(node_ptr, root);
        } else {
            list_ptr.unsafe_mut_ref().root_pointer = Some(node_ptr);
        }

        list_ptr.unsafe_mut_ref().len += 1;
        node_ptr.unsafe_mut_ref().set_link_state(true);
        true
    }

    /// Removes the current root node and advances the list pointer to the next node.
    ///
    /// **Note:** Unlike a stack `pop`, this removes the element currently designated
    /// as the "root" and shifts the root to the next element in the circle. It does
    /// not follow a specific "last-in, first-out" order.
    ///
    /// Returns `None` if the list is empty.
    pub fn pop(&mut self) -> Option<NonNull<T>> {
        let mut root_ptr = self.root_pointer?;
        if Self::is_single(root_ptr) {
            self.root_pointer = None;
        } else {
            let next_ptr = root_ptr.unsafe_ref().get_next()?;

            self.root_pointer = Some(next_ptr);

            Self::unlink(root_ptr);
        }

        root_ptr.unsafe_mut_ref().set_link_state(false);
        self.len -= 1;
        Some(root_ptr)
    }

    /// A raw version of `pop` that operates on a `NonNull` pointer to the list.
    ///
    /// Removes the root node and returns it, shifting the list's root to the next node.
    /// This avoids creating a `&mut Self` borrow, which helps in maintaining
    /// compatibility with raw pointer-based memory models (like Miri's Tree Borrows).
    /// # Safety
    /// The caller must ensure that `list_ptr` is valid and that no other mutable
    /// references to the list exist during this operation. This is useful in
    /// complex pointer-based structures where using standard Rust borrows
    /// would trigger aliasing violations.
    pub fn pop_raw(mut list_ptr: NonNull<IntrusiveDLinkList<T>>) -> Option<NonNull<T>> {
        let mut root_ptr = list_ptr.unsafe_ref().root_pointer?;

        if Self::is_single(root_ptr) {
            list_ptr.unsafe_mut_ref().root_pointer = None;
        } else {
            let next_ptr = root_ptr.unsafe_ref().get_next()?;

            list_ptr.unsafe_mut_ref().root_pointer = Some(next_ptr);

            Self::unlink(root_ptr);
        }

        root_ptr.unsafe_mut_ref().set_link_state(false);
        list_ptr.unsafe_mut_ref().len -= 1;
        Some(root_ptr)
    }

    /// Removes a specific node from the list.
    ///
    /// This is an O(1) operation. If the node being removed is the current root,
    /// the root is advanced to the next node.
    ///
    /// Returns `false` if the node is not actually part of a list, or if the
    /// current list is empty.
    pub fn remove(&mut self, mut node_ptr: NonNull<T>) -> bool {
        if !Self::is_node_linked(node_ptr) {
            return false;
        }
        let Some(root_ptr) = self.root_pointer else {
            return false;
        };

        if root_ptr == node_ptr {
            self.pop().is_some()
        } else {
            Self::unlink(node_ptr) && {
                node_ptr.unsafe_mut_ref().set_link_state(false);
                self.len -= 1;
                true
            }
        }
    }

    /// A raw version of `remove` that operates on a `NonNull` pointer to the list.
    ///
    /// Unlinks a specific node from the list using raw pointers. This is preferred
    /// when the node and the list are part of a larger structure where formal
    /// references are difficult to obtain safely.
    ///
    /// # Safety
    /// The caller must ensure that `list_ptr` is valid and that no other mutable
    /// references to the list exist during this operation. This is useful in
    /// complex pointer-based structures where using standard Rust borrows
    /// would trigger aliasing violations.
    pub fn remove_raw(
        mut list_ptr: NonNull<IntrusiveDLinkList<T>>,
        mut node_ptr: NonNull<T>,
    ) -> bool {
        if !Self::is_node_linked(node_ptr) {
            return false;
        }
        let Some(root_ptr) = list_ptr.unsafe_ref().root_pointer else {
            return false;
        };

        if root_ptr == node_ptr {
            Self::pop_raw(list_ptr).is_some()
        } else {
            Self::unlink(node_ptr) && {
                node_ptr.unsafe_mut_ref().set_link_state(false);
                list_ptr.unsafe_mut_ref().len -= 1;
                true
            }
        }
    }

    /// Internal helper to insert `node_ptr` into the circle after `target_ptr`.
    ///
    /// This "stitches" the new node between the target and whatever was
    /// previously after the target.
    fn link_to(mut node_ptr: NonNull<T>, mut target_ptr: NonNull<T>) -> bool {
        let target_node = target_ptr.unsafe_mut_ref();
        let Some(mut target_nxt_ptr) = target_node.get_next() else {
            return false;
        };
        target_node.set_next(Some(node_ptr));

        let target_nxt_node = target_nxt_ptr.unsafe_mut_ref();
        target_nxt_node.set_last(Some(node_ptr));

        let node_mut = node_ptr.unsafe_mut_ref();
        node_mut.set_last(Some(target_ptr));
        node_mut.set_next(Some(target_nxt_ptr));

        true
    }

    /// Internal helper to remove a node from its current neighbors.
    ///
    /// After unlinking, the neighbors will point to each other, and the
    /// `node_ptr` will be reset to point to itself.
    fn unlink(mut node_ptr: NonNull<T>) -> bool {
        let node = node_ptr.unsafe_mut_ref();

        let Some(mut nxt_ptr) = node.get_next() else {
            return false;
        };
        let Some(mut lst_ptr) = node.get_last() else {
            return false;
        };

        node.set_next(Some(node_ptr));
        node.set_last(Some(node_ptr));

        nxt_ptr.unsafe_mut_ref().set_last(Some(lst_ptr));
        lst_ptr.unsafe_mut_ref().set_next(Some(nxt_ptr));

        true
    }

    /// Checks the trait-defined state to see if the node is currently in a list.
    fn is_node_linked(node_ptr: NonNull<T>) -> bool {
        node_ptr.unsafe_ref().is_linked()
    }

    /// Returns `true` if the node's `next` pointer points back to itself.
    fn is_single(node_ptr: NonNull<T>) -> bool {
        node_ptr.unsafe_ref().get_next() == Some(node_ptr)
    }
}

impl<T> Default for IntrusiveDLinkList<T>
where
    T: DoublyLinkPointer<T>,
{
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {

    extern crate std;
    use std::vec::Vec;

    use std::{
        alloc::{Layout, alloc_zeroed, dealloc},
        ptr::NonNull,
    };

    use crate::{DoublyLinkPointer, IntrusiveDLinkList, ext::NonNullEx};

    /// A test node structure containing intrusive link pointers and data.
    struct TestSlot {
        next: Option<NonNull<TestSlot>>,
        last: Option<NonNull<TestSlot>>,
        linked: bool,

        data: usize,
    }

    impl DoublyLinkPointer<TestSlot> for TestSlot {
        fn get_next(&self) -> Option<NonNull<TestSlot>> {
            self.next
        }

        fn get_last(&self) -> Option<NonNull<TestSlot>> {
            self.last
        }

        fn set_next(&mut self, next_ptr: Option<NonNull<TestSlot>>) {
            self.next = next_ptr;
        }

        fn set_last(&mut self, last_ptr: Option<NonNull<TestSlot>>) {
            self.last = last_ptr
        }

        fn set_link_state(&mut self, state: bool) {
            self.linked = state;
        }

        fn is_linked(&self) -> bool {
            self.linked
        }
    }

    /// Basic functionality test for insertions, removals, and general list state.
    #[test]
    fn link_list_test() {
        const TEST_KEY: usize = 0xFFAF_4AB7_1CD9_5FFA;
        let slot_layout =
            Layout::from_size_align(size_of::<TestSlot>(), align_of::<TestSlot>()).unwrap();

        let mut list = IntrusiveDLinkList::<TestSlot>::new();

        let mut ptrs = Vec::new();

        // Push 10 allocated elements into the intrusive list
        for _ in 0..10 {
            let mut slot = allocate_slot(slot_layout);
            IntrusiveDLinkList::init_node(slot);
            ptrs.push(slot);
            slot.unsafe_mut_ref().data = TEST_KEY;
            assert!(list.push(slot), "IntrusiveDLinkList::push failed 0");
        }

        let list_ptr = NonNull::from_ref(&list);

        for _ in 0..10 {
            let mut slot = allocate_slot(slot_layout);
            IntrusiveDLinkList::init_node(slot);
            ptrs.push(slot);
            slot.unsafe_mut_ref().data = TEST_KEY;
            assert!(
                IntrusiveDLinkList::push_raw(list_ptr, slot),
                "IntrusiveDLinkList::push failed 1"
            );
        }

        assert!(list.len() == 20, "IntrusiveDLinkList::len failed");

        // Test pushing and immediately removing a single element
        let slot_ptr = allocate_slot(slot_layout);
        IntrusiveDLinkList::init_node(slot_ptr);

        assert!(list.push(slot_ptr), "IntrusiveDLinkList::push failed 1");
        assert!(list.remove(slot_ptr), "IntrusiveDLinkList::remove failed");

        // Clean up the list by popping all items
        for _ in 0..10 {
            let slot = list.pop().expect("IntrusiveDLinkList::pop failed");
            assert_eq!(
                slot.unsafe_ref().data,
                TEST_KEY,
                "IntrusiveDLinkList structural testing failed"
            );
        }

        // Deallocate all manually allocated nodes to prevent memory leaks
        for ptr in ptrs.drain(..) {
            unsafe {
                dealloc(ptr.as_ptr().cast(), slot_layout);
            }
        }
        unsafe {
            dealloc(slot_ptr.as_ptr().cast(), slot_layout);
        }
    }

    /// Advanced property-based test using Bolero.
    /// It fuzzes the operations by performing bulk pushes, arbitrary removals,
    /// re-insertions, and structural integrity checks.
    #[cfg(not(miri))]
    #[test]
    fn link_list_test_bolero() {
        use bolero::check;
        use std::collections::HashSet;

        let slot_layout =
            Layout::from_size_align(size_of::<TestSlot>(), align_of::<TestSlot>()).unwrap();

        check!().with_generator(0..usize::MAX).for_each(|key| {
            let mut ptrs = Vec::with_capacity(100);

            /// RAII guard to guarantee all allocated memory is properly deallocated,
            /// even if a panic occurs during the test execution.
            struct DeallocGuard {
                ptrs: Vec<NonNull<TestSlot>>,
                layout: Layout,
            }

            impl Drop for DeallocGuard {
                fn drop(&mut self) {
                    for &ptr in &self.ptrs {
                        unsafe {
                            dealloc(ptr.as_ptr().cast(), self.layout);
                        }
                    }
                }
            }

            let mut guard = DeallocGuard {
                ptrs: Vec::with_capacity(100),
                layout: slot_layout,
            };

            let mut intrusive_dll = IntrusiveDLinkList::<TestSlot>::new();

            // Allocate and initialize 100 elements, inserting them into the list
            for _ in 0..100 {
                let ptr = unsafe { alloc_zeroed(slot_layout) };
                let data = ptr as usize ^ key;

                let mut slot = NonNull::new(ptr)
                    .expect("memory allocation failed")
                    .cast::<TestSlot>();

                IntrusiveDLinkList::<TestSlot>::init_node(slot);
                ptrs.push(slot);
                guard.ptrs.push(slot);

                slot.unsafe_mut_ref().data = data;

                assert!(
                    intrusive_dll.push(slot),
                    "IntrusiveDLinkList::push failed 0"
                );
            }

            // Remove every other node from the list and collect them in a buffer
            let mut pop_buffer = Vec::with_capacity(50);
            for i in 0..100usize {
                if (i % 2) != 0 {
                    continue;
                }
                let ptr = ptrs[i];

                assert!(
                    intrusive_dll.remove(ptr),
                    "IntrusiveDLinkList::remove failed 0"
                );

                pop_buffer.push(ptr);
            }

            // Re-insert the removed nodes back into the list
            for ptr in pop_buffer {
                assert!(intrusive_dll.push(ptr), "IntrusiveDLinkList::push failed 1");
            }

            // Collect all elements into a HashSet for fast O(1) existence lookup
            let hash: HashSet<NonNull<TestSlot>> = ptrs.into_iter().collect();

            // Validate that all nodes currently in the list are valid and linked
            for slot in intrusive_dll.iter() {
                let ptr = NonNull::from_ref(slot);
                assert!(hash.contains(&ptr), "IntrusiveDLinkList::iter failed");
                assert!(
                    ptr.unsafe_ref().is_linked(),
                    "IntrusiveDLinkList linked check failed"
                );
            }

            // Pop everything and verify that data hasn't been corrupted
            for _ in 0..100 {
                let ptr = intrusive_dll.pop().expect("IntrusiveDLinkList::pop failed");

                let d = ptr.as_ptr() as usize ^ key;
                let data = ptr.unsafe_ref().data;

                assert!(data == d, "IntrusiveDLinkList structural testing failed");
            }
        });
    }

    /// Helper function to allocate raw zeroed memory for a single TestSlot node.
    fn allocate_slot(mem_lay: Layout) -> NonNull<TestSlot> {
        let ptr = unsafe { alloc_zeroed(mem_lay) };
        NonNull::new(ptr)
            .expect("Failed to allocate test slot")
            .cast()
    }
}
