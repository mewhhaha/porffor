//! Publication uses the declared entry's role and metadata together.

use super::*;
use crate::gc_types::{
    AsyncCallable, AsyncGeneratorCallable, ExecutableCode, ExecutableCodeEntry, ExecutableCodeKind,
    ExecutableCodeMetadata, GcStackReference, GeneratorCallable, OrdinaryCallable, RuntimeSchema,
};

impl PlannedFunctionEntry {
    pub(crate) fn emit_executable_code(
        &self,
        class_element_execution: ClassElementExecutionKind,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> Result<GcStackReference<ExecutableCode>, EmitError> {
        let (kind, standard_builtin, host_builtin) = match self.origin {
            EntryOrigin::User { .. } | EntryOrigin::EmptyOrdinaryFunction => {
                (ExecutableCodeKind::JavaScript, None, None)
            }
            EntryOrigin::StandardBuiltin(id) => {
                (ExecutableCodeKind::StandardBuiltin, Some(id), None)
            }
            EntryOrigin::HostBuiltin(id) => (ExecutableCodeKind::HostBuiltin, None, Some(id)),
            EntryOrigin::PreparedScript { .. } => {
                return Err(entry_error(
                    "a prepared Script cannot publish a callable code record",
                ))
            }
        };
        let metadata = ExecutableCodeMetadata {
            // This identifies the declaration for diagnostics. It is never a
            // value reference, dispatch index or object identity.
            source_id: i64::from(self.wasm_index),
            kind,
            standard_builtin,
            host_builtin,
            protocol: self.protocol(),
            class_element_execution,
        };
        macro_rules! publish {
            ($role:ident, $variant:ident) => {{
                let entry = schema
                    .reserve_function_local::<$role, _>(function)
                    .initialize(schema.reference_entry::<$role>(self, function)?, function);
                let published = schema.struct_type::<ExecutableCode>().publish(
                    ExecutableCodeEntry::$variant(&entry),
                    metadata,
                    function,
                )?;
                entry.clear(function);
                Ok(published)
            }};
        }
        match self.body() {
            PlannedBodyEntry::Ordinary(_) => publish!(OrdinaryCallable, Ordinary),
            PlannedBodyEntry::Generator(_) => publish!(GeneratorCallable, Generator),
            PlannedBodyEntry::Async(_) => publish!(AsyncCallable, Async),
            PlannedBodyEntry::AsyncGenerator(_) => publish!(AsyncGeneratorCallable, AsyncGenerator),
            PlannedBodyEntry::PreparedScript(_) => {
                unreachable!("Script rejected before publication")
            }
        }
    }
}
