//! The `BackendSpecOperation` to real-function join.
//!
//! `lila-ir` can prove that a catalog row carries evidence minted by a
//! `BackendSpecOperation`, but it cannot name a function in this crate. This
//! exhaustive match makes a renamed or deleted backend emitter fail name
//! resolution, and makes a new operation fail until it names a real emitter.

use lila_ir::BackendSpecOperation;

use crate::emit::FunctionBuilder;

#[allow(dead_code)]
fn backend_spec_operations_are_backed(operation: BackendSpecOperation) {
    match operation {
        BackendSpecOperation::ArraySpeciesCreate => {
            let _ = FunctionBuilder::emit_array_species_create;
        }
        BackendSpecOperation::ToPropertyDescriptor => {
            let _ = FunctionBuilder::emit_to_property_descriptor;
        }
    }
}
