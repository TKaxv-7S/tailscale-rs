#![doc = include_str!("../README.md")]
#![no_std]

// Before adding something to this crate, please see the note in the README. In short, does it
// REALLY need to go here?

#[cfg(feature = "alloc")]
extern crate alloc;

#[cfg(any(feature = "std", test))]
extern crate std;

mod debounce;

#[cfg(feature = "tokio")]
pub use debounce::TokioDebounce;
pub use debounce::{Debounce, DebounceExt};
