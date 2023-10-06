//! # The alloc prelude

/// The first version of the alloc prelude.
pub mod v1 {
    pub use crate::borrow::ToOwned;
    pub use crate::boxed::Box;
    pub use crate::string::String;
    pub use crate::string::ToString;
    pub use crate::vec::Vec;
}

/// The 2024 version of the prelude of the alloc prelude.
#[unstable(feature = "prelude_2024", issue = "none")]
pub mod rust_2024 {
    pub use super::v1::*;
}
