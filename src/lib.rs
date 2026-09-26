//! pklith: materialize hk guardrails from what a repository's files require.
//!
//! One module per spec node (`src/<node>/SPEC.md`), added as each is built.

pub mod catalog;
pub mod cli;
pub mod cover;
pub mod proc;
pub mod registry;
pub mod report;
pub mod scan;
