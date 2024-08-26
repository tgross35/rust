//! Implementation of `SyncUnsafeCell`

use super::UnsafeCell;
use crate::ops::{CoerceUnsized, DispatchFromDyn};
use crate::pin::PinCoerceUnsized;

/// [`UnsafeCell`], but [`Sync`].
///
/// This is just an `UnsafeCell`, except it implements `Sync`
/// if `T` implements `Sync`.
///
/// `UnsafeCell` doesn't implement `Sync`, to prevent accidental mis-use.
/// You can use `SyncUnsafeCell` instead of `UnsafeCell` to allow it to be
/// shared between threads, if that's intentional.
/// Providing proper synchronization is still the task of the user,
/// making this type just as unsafe to use.
///
/// See [`UnsafeCell`] for details.
#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
#[repr(transparent)]
#[cfg_attr(not(bootstrap), rustc_pub_transparent)]
pub struct SyncUnsafeCell<T: ?Sized> {
    value: UnsafeCell<T>,
}

#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
unsafe impl<T: ?Sized + Sync> Sync for SyncUnsafeCell<T> {}

#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
impl<T> SyncUnsafeCell<T> {
    /// Constructs a new instance of `SyncUnsafeCell` which will wrap the specified value.
    #[inline]
    pub const fn new(value: T) -> Self {
        Self { value: UnsafeCell { value } }
    }

    /// Unwraps the value, consuming the cell.
    #[inline]
    pub const fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
impl<T: ?Sized> SyncUnsafeCell<T> {
    /// Gets a mutable pointer to the wrapped value.
    ///
    /// This can be cast to a pointer of any kind.
    /// Ensure that the access is unique (no active references, mutable or not)
    /// when casting to `&mut T`, and ensure that there are no mutations
    /// or mutable aliases going on when casting to `&T`
    #[inline]
    #[rustc_never_returns_null_ptr]
    pub const fn get(&self) -> *mut T {
        self.value.get()
    }

    /// Returns a mutable reference to the underlying data.
    ///
    /// This call borrows the `SyncUnsafeCell` mutably (at compile-time) which
    /// guarantees that we possess the only reference.
    #[inline]
    pub const fn get_mut(&mut self) -> &mut T {
        self.value.get_mut()
    }

    /// Gets a mutable pointer to the wrapped value.
    ///
    /// See [`UnsafeCell::get`] for details.
    #[inline]
    pub const fn raw_get(this: *const Self) -> *mut T {
        // We can just cast the pointer from `SyncUnsafeCell<T>` to `T` because
        // of #[repr(transparent)] on both SyncUnsafeCell and UnsafeCell.
        // See UnsafeCell::raw_get.
        this as *const T as *mut T
    }
}

#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
impl<T: Default> Default for SyncUnsafeCell<T> {
    /// Creates an `SyncUnsafeCell`, with the `Default` value for T.
    fn default() -> SyncUnsafeCell<T> {
        SyncUnsafeCell::new(Default::default())
    }
}

#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
impl<T> From<T> for SyncUnsafeCell<T> {
    /// Creates a new `SyncUnsafeCell<T>` containing the given value.
    fn from(t: T) -> SyncUnsafeCell<T> {
        SyncUnsafeCell::new(t)
    }
}

#[unstable(feature = "coerce_unsized", issue = "18598")]
//#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
impl<T: CoerceUnsized<U>, U> CoerceUnsized<SyncUnsafeCell<U>> for SyncUnsafeCell<T> {}

// Allow types that wrap `SyncUnsafeCell` to also implement `DispatchFromDyn`
// and become object safe method receivers.
// Note that currently `SyncUnsafeCell` itself cannot be a method receiver
// because it does not implement Deref.
// In other words:
// `self: SyncUnsafeCell<&Self>` won't work
// `self: SyncUnsafeCellWrapper<Self>` becomes possible
#[unstable(feature = "dispatch_from_dyn", issue = "none")]
//#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
impl<T: DispatchFromDyn<U>, U> DispatchFromDyn<SyncUnsafeCell<U>> for SyncUnsafeCell<T> {}

#[unstable(feature = "pin_coerce_unsized_trait", issue = "123430")]
unsafe impl<T: ?Sized> PinCoerceUnsized for SyncUnsafeCell<T> {}
