//! Dereference struct fields in patterns.
//!
//! Provides the [`deref_pat`] macro as well as the [`PatDeref`] helper trait.
//!
//! Fields containing [`Box`], [`Rc`](std::rc::Rc), [`String`], [`Vec`] and everything else that implements [`Deref`] and [`DerefMut`] are supported.
//! In the special case of [`Box`] even matching on owned values is supported.
//!
//! # Usage
//! ```
//! use deref_pat::deref_pat;
//!
//! # struct Foo { string: String };
//! # let foo: Foo = Foo { string: "foo".into() };
//! deref_pat! {
//!     if let Foo { #[deref] string: bound @ "foo" } = &foo {
//!         assert_eq!(bound, "foo");
//!     } else {
//!         panic!("did not match");
//!     }
//! }
//! ```
//!
//! The generated code looks something like:
//! ```
//! # use deref_pat::PatDeref;
//! # struct Foo { string: String };
//! # let foo: Foo = Foo { string: "foo".into() };
//! if let Some(bound) = {
//!     let mut result = None;
//!     if let Foo { string } = &foo {
//!         if let bound @ "foo" = PatDeref::pat_deref(string) {
//!             result = Some(bound);
//!         }
//!     }
//!     result
//! } {
//!     assert_eq!(bound, "foo");
//! } else {
//!     panic!("did not match");
//! }
//! ```
//!
//! # Notes
//! - The `deref_pat` crate must be in scope under this exact name. Supplying a custom name is not yet supported.
//! - Only supports `if let` expressions. `match` expression are not yet supported.

pub use deref_pat_macro::deref_pat;
use std::ops::{Deref, DerefMut};

/// Helper trait used for dereferencing operations in patterns.
///
/// Similar to [`Deref`] and [`DerefMut`],
/// but works on immutable as well as mutable references to values implementing [`Deref`] and [`DerefMut`].
/// Also handles owned [`Box`] dereferencing as a special case.
///
/// # Usage
/// ```
/// use deref_pat::PatDeref;
///
/// # type T = ();
/// let mut boxed: Box<T> = Default::default();
/// let ref_deref: &T = PatDeref::pat_deref(&boxed);
/// let mut_deref: &mut T = PatDeref::pat_deref(&mut boxed);
/// let owned_deref: T = PatDeref::pat_deref(boxed);
/// ```
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
