//! All types and interfaces to support multiple backends.
//!
//! This is independent of `client`, `server`, and `bridge`.

#[allow(unsafe_code)]
mod arena;
#[allow(unsafe_code)]
mod buffer;
#[allow(unsafe_code)]
mod closure;
#[forbid(unsafe_code)]
mod fxhash;
#[forbid(unsafe_code)]
pub(crate) mod handle;
#[allow(unsafe_code)]
pub(crate) mod selfless_reify;

pub(crate) use arena::Arena;
pub(crate) use buffer::Buffer;
pub(crate) use closure::Closure;
pub(crate) use fxhash::FxHashMap;
pub(crate) use handle::Handle;
