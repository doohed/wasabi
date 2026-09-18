//! What a test is made of, and how a run went.
//!
//! Everything here is about one run: the words it deals, what the settings do
//! to them, and the readings taken while they're typed. Nothing in this module
//! outlives the test it describes — what survives a run lives in
//! [`crate::scores`], and what the user chose lives in [`crate::config`].

pub mod misses;
pub mod modifiers;
pub mod timeline;
pub mod word;
pub mod wordlist;
