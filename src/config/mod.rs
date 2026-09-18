//! The user's own files, and where they live.
//!
//! Four things they can change and one module that knows where any of it goes.
//! What they set here outlives every run, which is what separates it from
//! [`crate::typing`], and it is all theirs to hand-edit — so every loader in
//! this module is infallible by design. A missing, unreadable or corrupt file
//! means "the default", never a failure to start.

pub mod banner;
pub mod settings;
pub mod storage;
pub mod theme;
