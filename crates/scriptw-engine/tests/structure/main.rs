//! Consolidated integration tests: source/structure guards, compiler identity and rooted-observation tests.
//!
//! One binary per area bounds the number of linked Wasmtime test executables
//! while keeping per-process memory bounded.

mod compiler_fingerprint;
mod compiler_inspection;
mod engine_error_wasmtime_policy_authority_structure;
mod execution_backend_routing_structure;
mod module_entry_source_authority_structure;
mod oracle_entry_syntax;
mod rooted_completion_graph;
mod wasm_execution_mode_structure;
mod wasm_top_level_completion_kind_structure;
