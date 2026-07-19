//! Shared support for oddutils command implementations.
//!
//! This crate intentionally stays small. Utilities should remain ordinary Unix
//! programs, with shared code limited to IO, filesystem, and error handling that
//! would otherwise be repeated.

pub mod editor;
pub mod temp;
pub mod unix;
