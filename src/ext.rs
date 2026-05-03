//! Helper trait to simplify converting `NonNull` pointers into shared or mutable references.

use core::ptr::NonNull;

pub trait NonNullEx<T> {
    type Type;

    fn unsafe_ref<'a>(&self) -> &'a Self::Type;
    fn unsafe_mut_ref<'a>(&mut self) -> &'a mut Self::Type;
}

impl<T> NonNullEx<T> for NonNull<T> {
    type Type = T;

    fn unsafe_ref<'a>(&self) -> &'a Self::Type {
        unsafe { self.as_ref() }
    }

    fn unsafe_mut_ref<'a>(&mut self) -> &'a mut Self::Type {
        unsafe { self.as_mut() }
    }
}
