//! # Intrusive Doubly Linked List Raw Implementation
//!
//! This module provides the [IntrusiveDLinkListRaw] struct, which manages a circular
//! doubly linked list of nodes entirely through raw pointers.
//!
//! ## Architecture
//! - Intrusive & Allocation-free: The list doesn't allocate memory for nodes. It relies on
//!   types implementing [DoublyLinkPointerRaw] to provide pointer storage inside the data itself.
//! - Circular Structure: The tail points back to the root, optimizing push and pop to O(1).
//! - Raw Pointer Model: All methods operate on NonNull pointers rather than Rust references.
//!   This prevents the creation of overlapping mutable references, satisfying strict memory model checkers like Miri.
//!
//! ## Key Invariants
//! 1. A node is considered "initialized" if its next and last pointers point to itself.
//! 2. The root_pointer inside the list structure always points to the "head" of the list.
//! 3. If len > 0, root_pointer must be Some.
//! 4. A node's is_linked state must accurately reflect whether it is part of a list.
//!

use crate::DoublyLinkPointerRaw;
use core::ptr::NonNull;

pub struct IntrusiveDLinkListRaw<T>
where
    T: DoublyLinkPointerRaw<T>,
{
    root_pointer: Option<NonNull<T>>,
    len: usize,
}

impl<T> IntrusiveDLinkListRaw<T>
where
    T: DoublyLinkPointerRaw<T>,
{
    /// Creates a new, empty intrusive doubly linked list structure.
    ///
    /// This operation is O(1) and does not perform any heap allocations.
    pub fn new() -> Self {
        Self {
            root_pointer: None,
            len: 0,
        }
    }

    /// Returns the number of nodes currently in the list via a raw pointer to the list.
    ///
    /// # Safety
    /// The `list_ptr` must point to a valid, initialized instance of `IntrusiveDLinkListRaw<T>`.
    pub fn len(list_ptr: NonNull<Self>) -> usize {
        let list_raw_ptr = list_ptr.as_ptr();

        unsafe { (*list_raw_ptr).len }
    }

    /// Returns `true` if the list contains no nodes.
    ///
    /// # Safety
    /// The `list_ptr` must point to a valid, initialized instance of `IntrusiveDLinkListRaw<T>`.
    pub fn is_empty(list_ptr: NonNull<Self>) -> bool {
        Self::len(list_ptr) == 0
    }

    /// Prepares a node for insertion by making it point to itself.
    ///
    /// In a circular intrusive list, a node must be "self-aware" before it can be
    /// linked to others. This function sets both the `next` and `last` pointers
    /// of the node to its own address and clears its link state.
    ///
    /// # Safety
    /// The `node_ptr` must be valid, well-aligned, and point to allocated memory that can be modified.
    pub fn init_node(node_ptr: NonNull<T>) {
        T::set_next(node_ptr, Some(node_ptr));
        T::set_last(node_ptr, Some(node_ptr));
        T::set_link_state(node_ptr, false);
    }

    /// Inserts a node into the circular list via raw pointers.
    ///
    /// **Note:** This is not a standard LIFO stack operation. The node is inserted into the
    /// circular structure relative to the current `root_pointer`.
    ///
    /// Returns `false` if the node is already marked as linked in a list.
    ///
    /// # Safety
    /// Both `list_ptr` and `node_ptr` must point to valid, initialized memory locations.
    /// The caller must ensure no exclusive references to these locations overlap during execution.
    pub fn push(list_ptr: NonNull<Self>, node_ptr: NonNull<T>) -> bool {
        if Self::is_node_linked(node_ptr) {
            return false;
        }

        let list_raw_ptr = list_ptr.as_ptr();

        if let Some(root) = unsafe { (*list_raw_ptr).root_pointer } {
            Self::link_to(node_ptr, root);
        } else {
            unsafe {
                (*list_raw_ptr).root_pointer = Some(node_ptr);
            }
        }
        unsafe {
            (*list_raw_ptr).len += 1;
        }
        T::set_link_state(node_ptr, true);
        true
    }

    /// Removes the current root node and advances the list's root pointer to the next node.
    ///
    /// **Note:** Shifting the root pointer to the next element in the circle means this does
    /// not follow a strict stack/LIFO order.
    ///
    /// Returns `None` if the list is empty.
    ///
    /// # Safety
    /// The `list_ptr` must be valid and dereferenceable. Any nodes unlinked during this process
    /// must also be valid memory addresses.
    pub fn pop(list_ptr: NonNull<Self>) -> Option<NonNull<T>> {
        let list_raw_ptr = list_ptr.as_ptr();

        let root_ptr = unsafe { (*list_raw_ptr).root_pointer }?;
        if Self::is_single(root_ptr) {
            unsafe { (*list_raw_ptr).root_pointer = None };
        } else {
            let next_ptr = T::get_next(root_ptr)?;

            unsafe { (*list_raw_ptr).root_pointer = Some(next_ptr) };
            Self::unlink(root_ptr);
        }

        unsafe {
            (*list_raw_ptr).len -= 1;
        }
        T::set_link_state(root_ptr, false);
        Some(root_ptr)
    }

    /// Removes a specific node from the list using raw pointers.
    ///
    /// This is an $O(1)$ operation. If the node being removed is the current root,
    /// the root pointer is automatically advanced to the next node in the circle.
    ///
    /// Returns `false` if the node is not actually part of a list, or if the list is empty.
    ///
    /// # Safety
    /// Both `list_ptr` and `node_ptr` must be valid. The node must belong to the list specified by `list_ptr`.
    pub fn remove(list_ptr: NonNull<Self>, node_ptr: NonNull<T>) -> bool {
        if !Self::is_node_linked(node_ptr) {
            return false;
        }
        let list_raw_ptr = list_ptr.as_ptr();

        let root_pointer = unsafe { (*list_raw_ptr).root_pointer };
        let Some(root_ptr) = root_pointer else {
            return false;
        };

        if root_ptr == node_ptr {
            Self::pop(list_ptr).is_some()
        } else {
            Self::unlink(node_ptr) && {
                T::set_link_state(node_ptr, false);

                unsafe {
                    (*list_raw_ptr).len -= 1;
                }
                true
            }
        }
    }

    /// Internal helper to insert `node_ptr` into the circle right after `target_ptr`.
    ///
    /// This stitches the new node between the target and whatever followed it.
    fn link_to(node_ptr: NonNull<T>, target_ptr: NonNull<T>) -> bool {
        let Some(target_nxt_ptr) = T::get_next(target_ptr) else {
            return false;
        };

        T::set_last(node_ptr, Some(target_ptr));
        T::set_next(node_ptr, Some(target_nxt_ptr));

        T::set_last(target_nxt_ptr, Some(node_ptr));
        T::set_next(target_ptr, Some(node_ptr));

        true
    }

    /// Internal helper to isolate a node from its current neighbors.
    ///
    /// After unlinking, neighbors point to each other, and `node_ptr` is reset to point to itself.
    fn unlink(node_ptr: NonNull<T>) -> bool {
        // let node = node_ptr.unsafe_mut_ref();

        let Some(nxt_ptr) = T::get_next(node_ptr) else {
            return false;
        };
        let Some(lst_ptr) = T::get_last(node_ptr) else {
            return false;
        };

        T::set_next(node_ptr, Some(node_ptr));
        T::set_last(node_ptr, Some(node_ptr));

        T::set_last(nxt_ptr, Some(lst_ptr));
        T::set_next(lst_ptr, Some(nxt_ptr));

        true
    }

    /// Checks the trait-defined state via raw pointers to see if the node is currently linked.
    fn is_node_linked(node_ptr: NonNull<T>) -> bool {
        T::is_linked(node_ptr)
    }

    /// Returns `true` if the node's `next` pointer points back to its own address.
    fn is_single(node_ptr: NonNull<T>) -> bool {
        T::get_next(node_ptr) == Some(node_ptr)
    }
}

impl<T> Default for IntrusiveDLinkListRaw<T>
where
    T: DoublyLinkPointerRaw<T>,
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

    use crate::{DoublyLinkPointerRaw, IntrusiveDLinkListRaw, ext::NonNullEx};

    /// A test node structure containing intrusive link pointers and data.
    struct TestSlot {
        next: Option<NonNull<TestSlot>>,
        last: Option<NonNull<TestSlot>>,
        linked: bool,

        data: usize,
    }

    impl DoublyLinkPointerRaw<TestSlot> for TestSlot {
        fn get_next(node_ptr: NonNull<TestSlot>) -> Option<NonNull<TestSlot>> {
            node_ptr.unsafe_ref().next
        }

        fn get_last(node_ptr: NonNull<TestSlot>) -> Option<NonNull<TestSlot>> {
            node_ptr.unsafe_ref().last
        }

        fn set_next(mut node_ptr: NonNull<TestSlot>, next_ptr: Option<NonNull<TestSlot>>) {
            node_ptr.unsafe_mut_ref().next = next_ptr;
        }

        fn set_last(mut node_ptr: NonNull<TestSlot>, last_ptr: Option<NonNull<TestSlot>>) {
            node_ptr.unsafe_mut_ref().last = last_ptr;
        }

        fn set_link_state(mut node_ptr: NonNull<TestSlot>, state: bool) {
            node_ptr.unsafe_mut_ref().linked = state;
        }

        fn is_linked(node_ptr: NonNull<TestSlot>) -> bool {
            node_ptr.unsafe_ref().linked
        }
    }

    /// Basic functionality test for insertions, removals, and general list state.
    #[test]
    fn link_list_test() {
        const TEST_KEY: usize = 0xFFAF_4AB7_1CD9_5FFA;
        let slot_layout =
            Layout::from_size_align(size_of::<TestSlot>(), align_of::<TestSlot>()).unwrap();

        let mut list = IntrusiveDLinkListRaw::<TestSlot>::new();
        let list_ptr = NonNull::new(&mut list as *mut _).unwrap();

        let mut ptrs = Vec::new();

        // Push 10 allocated elements into the intrusive list
        for _ in 0..10 {
            let mut slot = allocate_slot(slot_layout);
            IntrusiveDLinkListRaw::init_node(slot);
            ptrs.push(slot);
            slot.unsafe_mut_ref().data = TEST_KEY;
            assert!(
                IntrusiveDLinkListRaw::<TestSlot>::push(list_ptr, slot),
                "IntrusiveDLinkList::push failed 0"
            );
        }

        assert!(
            IntrusiveDLinkListRaw::<TestSlot>::len(list_ptr) == 10,
            "IntrusiveDLinkListRaw::<..>::len failed"
        );

        // Test pushing and immediately removing a single element
        let slot_ptr = allocate_slot(slot_layout);
        IntrusiveDLinkListRaw::<TestSlot>::init_node(slot_ptr);

        assert!(
            IntrusiveDLinkListRaw::<TestSlot>::push(list_ptr, slot_ptr),
            "IntrusiveDLinkListRaw::<..>::push failed 1"
        );
        assert!(
            IntrusiveDLinkListRaw::<TestSlot>::remove(list_ptr, slot_ptr),
            "IntrusiveDLinkListRaw::<..>::remove failed"
        );

        // Clean up the list by popping all items
        for _ in 0..10 {
            let slot = IntrusiveDLinkListRaw::<TestSlot>::pop(list_ptr)
                .expect("IntrusiveDLinkListRaw::<..>::pop failed");
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

            let mut intrusive_dll = IntrusiveDLinkListRaw::<TestSlot>::new();
            let intrusive_dll_ptr = NonNull::new(&mut intrusive_dll as *mut _).unwrap();

            // Allocate and initialize 100 elements, inserting them into the list
            for _ in 0..100 {
                let ptr = unsafe { alloc_zeroed(slot_layout) };
                let data = ptr as usize ^ key;

                let mut slot = NonNull::new(ptr)
                    .expect("memory allocation failed")
                    .cast::<TestSlot>();

                IntrusiveDLinkListRaw::<TestSlot>::init_node(slot);
                ptrs.push(slot);
                guard.ptrs.push(slot);

                slot.unsafe_mut_ref().data = data;

                assert!(
                    IntrusiveDLinkListRaw::<TestSlot>::push(intrusive_dll_ptr, slot),
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
                    IntrusiveDLinkListRaw::<TestSlot>::remove(intrusive_dll_ptr, ptr),
                    "IntrusiveDLinkList::remove failed 0"
                );

                pop_buffer.push(ptr);
            }

            // Re-insert the removed nodes back into the list
            for ptr in pop_buffer {
                assert!(
                    IntrusiveDLinkListRaw::<TestSlot>::push(intrusive_dll_ptr, ptr),
                    "IntrusiveDLinkList::push failed 1"
                );
            }

            // Collect all elements into a HashSet for fast O(1) existence lookup
            let hash: std::collections::HashSet<NonNull<TestSlot>> = ptrs.into_iter().collect();

            // Pop everything and verify that data hasn't been corrupted
            for _ in 0..100 {
                let ptr = IntrusiveDLinkListRaw::<TestSlot>::pop(intrusive_dll_ptr)
                    .expect("IntrusiveDLinkListRaw::<..>::pop failed 0");
                assert!(hash.contains(&ptr), "IntrusiveDLinkList::pop failed 1");

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
