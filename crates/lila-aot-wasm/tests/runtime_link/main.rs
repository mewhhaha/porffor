//! Consolidated integration tests. The linked runtime/program split: runtime imports, rooted
//! snapshots, host import indices, runtime errors and string data.
//!
//! One binary per area: every integration-test binary links the emitter and the ICU data, so
//! a few area targets cost far less disk and link time than one target per file. Each module
//! is one former test file; run just one with `-- <module>::`.

#[path = "../fixtures/linked_bodies.rs"]
mod linked_bodies;

mod host_import_function_indices_structure;
mod pooled_string_data;
mod rooted_snapshot_exports;
mod runtime_error_active_handler_structure;
mod runtime_import_reachability;
mod runtime_regexp_entry_kind_structure;
mod thrown_error_diagnostic_kind_authority_structure;
mod unhandled_rejection_reporting_structure;
