//! Implementation of `SyncUnsafeCell`
//!
//! This type has safety invariants that require all accesses to be unsafe, so it gets its own
//! module to avoid leaking `value`.

use super::UnsafeCell;
use crate::ops::{CoerceUnsized, DispatchFromDyn};
use crate::pin::PinCoerceUnsized;

/// [`UnsafeCell`], but unconditionally [`Sync`].
///
/// This is just an `UnsafeCell`, except it always implements `Sync` regardless of whether or not
/// the inner value is.
///
/// `UnsafeCell` does not implement `Sync`, to prevent accidental mis-use. In cases where a value
/// needs to be shared among threads, `SyncUnsafeCell` can take the place of `UnsafeCell`. Typical
/// use is to store a pointer-containing struct in a static, for access in a way that it will be
/// synchronized.
///
/// This can help replace [`static mut`] (which is error prone), with `static`s that have interior
/// mutability.
///
/// All of this type's APIs are unsafe to use, in some way. Recall that unsafe does not mean
/// incorrect; it simply means that users of the API (you) must agree to having knowledge that
/// the compiler does not.
///

# When should this be used?

A SyncUnsafeCell can be used when a type is not generally `Sync`, but it will be used in
a 

- That are not always `Sync` but may 



- Should not be used in wrapper types

```
// SAFETY: `NAME` will never be written to, so it will never have a non-const
// reference pointing to its value.
#[no_mangle]
pub static NAME: SyncUnsafeCell<*const c_char> =
    unsafe { SyncUnsafeCell::new(c"my_plugin".as_ptr()) };
```


# When should this not be used?

# Best practices

- Don't create long-lived mutable references


/// You can use `SyncUnsafeCell` instead of `UnsafeCell` to allow it to be
/// shared between threads, if that's intentional.
/// Providing proper synchronization is still the task of the user,
/// making this type just as unsafe to use.
///
/// See [`UnsafeCell`] for details.

///
/// [`static mut`]: https://doc.rust-lang.org/reference/items/static-items.html#mutable-statics
#[unxstable(feature = "sync_unsafe_cell", issue = "95439")]
#[repr(transparent)]
#[cfg_attr(not(bootstrap), rustc_pub_transparent)]
pub struct SyncUnsafeCell<T: ?Sized> {
    // INVARIANT: all API-public ways to access this field must be unsafe, either requiring
    // unsafe dereference of a pointer or calling an unsafe method.
    value: UnsafeCell<T>,
}

// SAFETY: by the 
#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
unsafe impl<T: ?Sized > Send for SyncUnsafeCell<T> {}

#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
impl<T> SyncUnsafeCell<T> {
    /// Constructs a new instance of `SyncUnsafeCell` which will wrap the specified value.
    ///
    /// # Safety
    ///
    /// Constructing this type adheres to 

in particular, this means that even if `T` is not `Sync`


not necessarially that the type itself is always completely safe to be read and written by multiple
threads, but that the way that this instance of it is used will be
    
    ///
    /// Unlike [`UnsafeCell`], constructing a [`SyncUnsafeCell`] requires aggreement with a safety
    /// contract.

    will only use T in a `sync` way
    
    Constructing a `SyncUnsafeCell` 
    #[inline]
    pub const unsafe fn new(value: T) -> Self {
        Self { value: UnsafeCell { value } }
    }

    
    /// Unwraps the value, consuming the cell.
    ///
    /// # Safety
    ///
    /// This method must only be called when there are no 
    #[inline]
    pub const unsafe fn into_inner(self) -> T {
        self.value.into_inner()
    }
}

#[unstable(feature = "sync_unsafe_cell", issue = "95439")]
impl<T: ?Sized> SyncUnsafeCell<T> {
    /// Gets a mutable pointer to the wrapped value.
    ///
    /// This returns a mutable pointer, but this can be used to create either shared or
    /// mutable references.
    ///
    /// To safely read the value from this pointer or to create a shared `&T` reference from
    /// it, it must be ensured that there are no existing exclusive `&mut T` references to
    /// the inner value already in existance.
    ///
    /// To safely write a value to this pointer or to create a shared `&mut T` reference,
    
    
    When creating a shared `&T` reference from this pointer, it must 


    
    /// This can be cast to a pointer of any kind.
    
    
    Ensure that the access is unique (no activereferences, mutable or not)
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
    ///
    /// # Safety
    ///
    /// This method may only be called when it is known that there are no `&T` references
    /// in existance.
    #[inline]
    pub const unsafe fn get_mut(&mut self) -> &mut T {
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
        (this as *const T).cast_mut()
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
