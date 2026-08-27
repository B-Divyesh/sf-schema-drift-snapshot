//! Schema Drift Snapshot's typed snapshot, comparison, and report engine.
//!
//! The CLI is the primary interface. Library consumers can decode a versioned
//! [`model::Snapshot`], call [`diff::compare`], and render a safe Markdown
//! review with [`report::markdown`]. No API in this crate generates repair SQL.

pub mod capture;
pub mod diff;
pub mod license;
pub mod model;
pub mod redact;
pub mod report;
