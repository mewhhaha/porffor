//! Process-level runtime and program cache tests.
//!
//! These tests re-execute this very test binary as fresh child processes with a
//! private `LILA_CACHE_DIR` and select the child with `--exact <module>::<test>`,
//! so they stay in their own small target: no other area's tests share the child
//! filter or the copied executable.

mod aot_date_cached_realm_choice;
mod runtime_artifact_cache;
