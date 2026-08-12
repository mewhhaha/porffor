//! The `EmissionSite` → real-function join.
//!
//! `lila-ir` names emitter arms in `OperationLoweringStatus::StatementEmission`
//! rows, but `lila-ir` cannot see `lila-aot-wasm` — the dependency runs
//! the other way. Without this file an `EmissionSite` variant is a `&'static
//! str` in a trench coat: it can name a function that was renamed, moved or
//! deleted, and nothing notices.
//!
//! The function below is never called. It exists so that
//!
//! - renaming or deleting an emitter arm that an `EmissionSite` claims is
//!   `E0599`/`E0433`, and
//! - adding an `EmissionSite` variant is `E0004` until it names something real.
//!
//! The guarantee is **name resolution, not signature**. It does not check that
//! the named arm emits what the row says it emits — that is ledger **L2**.

use lila_ir::EmissionSite;

use crate::emit::FunctionBuilder;

#[allow(dead_code)]
fn emission_sites_are_backed(site: EmissionSite) {
    match site {
        EmissionSite::SyncForOfIterator => {
            let _ = FunctionBuilder::compile_for_of_iterator;
        }
        EmissionSite::AsyncForOfIterator => {
            let _ = FunctionBuilder::compile_async_for_of_iterator;
        }
        EmissionSite::ArrayDestructuring => {
            let _ = FunctionBuilder::compile_array_destructure_from_value_locals;
        }
        EmissionSite::CallArgumentSpread => {
            let _ = FunctionBuilder::emit_call_args_vector;
        }
        EmissionSite::GeneratorDelegation => {
            let _ = FunctionBuilder::compile_generator_delegation;
            let _ = FunctionBuilder::compile_async_generator_delegation;
        }
    }
}
