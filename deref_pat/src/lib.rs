use std::ops::{Deref, DerefMut};

pub use deref_pat_macro::deref_pat;

/// Helper trait used for dereferencing operations in patterns.
///
/// Similar to [`Deref`] and [`DerefMut`], but overloaded to work on immutable as well as mutable references.
/// Also handles the special case of owned [`Box`] dereferencing.
pub trait PatDeref {
    /// The resulting type after dereferencing.
    type Target: ?Sized;

    /// Dereferences the value.
    fn pat_deref(self) -> Self::Target;
}

impl<'a, T> PatDeref for &'a T
where
    T: Deref,
{
    type Target = &'a T::Target;

    fn pat_deref(self) -> Self::Target {
        &*self
    }
}

impl<'a, T> PatDeref for &'a mut T
where
    T: DerefMut,
{
    type Target = &'a mut T::Target;

    fn pat_deref(self) -> Self::Target {
        &mut *self
    }
}

impl<T> PatDeref for Box<T> {
    type Target = T;

    fn pat_deref(self) -> Self::Target {
        *self
    }
}
