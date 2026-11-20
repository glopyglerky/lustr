// Original rustc-stable-hash 0.1.2; MIT OR Apache-2.0.
// core namespace and module paths adapted; original hashing operations retained.
//! A stable hashing algorithm used by rustc

#![deny(clippy::missing_safety_doc)]
#![deny(unsafe_op_in_unsafe_fn)]

mod int_overflow;
mod sip128;
mod stable_hasher;

/// Hashers collection
pub mod hashers {
    #[doc(inline)]
    pub use super::sip128::{SipHasher128, SipHasher128Hash};

    /// Stable 128-bits Sip Hasher
    ///
    /// [`StableHasher`] version of [`SipHasher128`].
    ///
    /// [`StableHasher`]: super::StableHasher
    pub type StableSipHasher128 = super::StableHasher<SipHasher128>;
}

#[doc(inline)]
pub use stable_hasher::StableHasher;

#[doc(inline)]
pub use stable_hasher::FromStableHash;

#[doc(inline)]
pub use stable_hasher::ExtendedHasher;

#[doc(inline)]
pub use hashers::{SipHasher128Hash, StableSipHasher128};
