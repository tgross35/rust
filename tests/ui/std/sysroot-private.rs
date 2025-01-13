//! Test that private dependencies of `std` that live in the sysroot do not reach through to
//! diagnostics.
//!
//! This test would be more robust if we could patch the sysroot with an "evil" crate that
//! provided known types that we control; however, this would effectively require rebuilding
//! `std` (or patching crate metadata). So, this test relies on what is currently public API
//! of `std`'s dependencies, but may not be robust against dependency upgrades/changes.

//@ revisions: default rustc_private_enabled

// Enabling `rustc_private` should `std`'s dependencies accessible, so they should show up
// in diagnostics. NB: not all diagnostics are affected by this.
#![cfg_attr(rustc_private_enabled, feature(rustc_private))]
#![crate_type = "lib"]

trait Trait { type Bar; }

// Attempt to get a suggestion for `addr2line::LookupContinuation`, which has member `Buf`
// Note that the suggestion only happens when `rustc_private` is enabled.
type AssociatedTy = dyn Trait<Buf = i32, Bar = i32>;
//~^ ERROR associated type `Buf` not found
//[rustc_private_enabled]~| NOTE there is an associated type `Buf` in the trait `addr2line::lookup::LookupContinuation`

// Attempt to get a suggestion for `hashbrown::Equivalent`
trait Trait2<K>: Equivalent<K> {}
//~^ ERROR cannot find trait
//~| NOTE not found

// Attempt to get a suggestion for `hashbrown::Equivalent::equivalent`
fn trait_member<T>(val: &T, key: &K) -> bool {
    //~^ ERROR cannot find type `K`
    //~| NOTE similarly named
    val.equivalent(key)
}

// Attempt to get a suggestion for `memchr::memchr2`
fn free_function(buf: &[u8]) -> Option<usize> {
    memchr2(b'a', b'b', buf)
    //~^ ERROR cannot find function
    //~| NOTE not found
}
