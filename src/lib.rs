//! pklith: materialize hk guardrails from what a repository's files require.
//!
//! One module per spec node (`src/<node>/SPEC.md`), added as each is built.

pub mod catalog;
pub mod cli;
pub mod cover;
pub mod detect;
pub mod r#gen;
pub mod hook;
pub mod lay;
pub mod legacy;
pub mod map;
pub mod proc;
pub mod registry;
pub mod report;
pub mod rule;
pub mod scan;
pub mod seed;
