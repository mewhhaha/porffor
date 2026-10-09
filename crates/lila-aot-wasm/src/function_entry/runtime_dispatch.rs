//! The concrete callable owns its typed executable references.

use super::*;
use crate::gc_types::{
    AsyncCallable, AsyncGeneratorCallable, ExecutableCode, ExecutableCodeSchema, FunctionLocal,
    FunctionObject, FunctionObjectSchema, GcCallResult, GcLocal, GeneratorCallable,
    OrdinaryCallable, RuntimeSchema,
};

pub(crate) struct RuntimeFunctionEntryDispatch {
    callable: GcLocal<FunctionObject>,
    code: GcLocal<ExecutableCode>,
}

macro_rules! runtime_entry {
    ($entry:ident, $role:ident, $inputs:ident) => {
        pub(crate) struct $entry {
            callable: GcLocal<FunctionObject>,
            code: FunctionLocal<$role>,
        }
        impl $entry {
            pub(crate) fn function_object(&self) -> &GcLocal<FunctionObject> {
                &self.callable
            }
            pub(crate) fn emit_call(
                &self,
                inputs: $inputs<'_>,
                schema: &RuntimeSchema,
                function: &mut Function,
            ) -> GcCallResult {
                inputs.emit(&self.callable, schema, function);
                schema.call_reference(&self.code, function)
            }
            fn clear(self, function: &mut Function) {
                self.code.clear(function);
                self.callable.clear(function);
            }
        }
    };
}
runtime_entry!(
    RuntimeOrdinaryBodyEntry,
    OrdinaryCallable,
    OrdinaryBodyInputs
);
runtime_entry!(
    RuntimeGeneratorBodyEntry,
    GeneratorCallable,
    GeneratorBodyInputs
);
runtime_entry!(RuntimeAsyncBodyEntry, AsyncCallable, AsyncBodyInputs);
runtime_entry!(
    RuntimeAsyncGeneratorBodyEntry,
    AsyncGeneratorCallable,
    AsyncGeneratorBodyInputs
);

impl RuntimeOrdinaryBodyEntry {
    pub(crate) fn emit_return_call(
        &self,
        inputs: OrdinaryBodyInputs<'_>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) {
        inputs.emit(&self.callable, schema, function);
        schema.return_call_ordinary_reference(&self.code, function);
    }
}

impl RuntimeFunctionEntryDispatch {
    /// Called only after ValueLocals has provided a concrete FunctionObject.
    pub(crate) fn from_function(
        callable: &GcLocal<FunctionObject>,
        schema: &RuntimeSchema,
        function: &mut Function,
    ) -> Self {
        let callable = schema
            .reserve_gc_local(function)
            .initialize(callable.load(schema, function), function);
        let code = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<FunctionObject>()
                .field(FunctionObjectSchema::CODE)
                .read(&callable, schema, function)
                .reference(),
            function,
        );
        Self { callable, code }
    }

    pub(crate) fn release(self, function: &mut Function) {
        self.code.clear(function);
        self.callable.clear(function);
    }
}

macro_rules! runtime_family {
    ($method:ident, $entry:ident, $role:ident, $field:ident) => {
        impl RuntimeFunctionEntryDispatch {
            pub(crate) fn $method(
                &self,
                emitter: &mut FunctionBuilder<'_>,
                function: &mut Function,
                emit: impl FnOnce(
                    &$entry,
                    &mut FunctionBuilder<'_>,
                    &mut Function,
                ) -> Result<(), EmitError>,
            ) -> Result<(), EmitError> {
                let schema = emitter.runtime_schema();
                // The nullable code field is the branch predicate. A reference
                // becomes non-null only within the branch that proved it exists.
                schema
                    .struct_type::<ExecutableCode>()
                    .field(ExecutableCodeSchema::$field)
                    .read(&self.code, schema, function)
                    .callable();
                function.instruction(&Instruction::RefIsNull);
                function.instruction(&Instruction::I32Eqz);
                emitter.open_frame(ControlFrameKind::If, function);
                let code = schema
                    .reserve_function_local::<$role, _>(function)
                    .initialize(
                        schema
                            .struct_type::<ExecutableCode>()
                            .field(ExecutableCodeSchema::$field)
                            .read(&self.code, schema, function)
                            .callable()
                            .require_non_null(function),
                        function,
                    );
                let callable = schema
                    .reserve_gc_local(function)
                    .initialize(self.callable.load(schema, function), function);
                let entry = $entry { callable, code };
                emit(&entry, emitter, function)?;
                entry.clear(function);
                emitter.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                Ok(())
            }
        }
    };
}
runtime_family!(
    ordinary,
    RuntimeOrdinaryBodyEntry,
    OrdinaryCallable,
    ORDINARY_ENTRY
);
runtime_family!(
    generator,
    RuntimeGeneratorBodyEntry,
    GeneratorCallable,
    GENERATOR_ENTRY
);
runtime_family!(
    asynchronous,
    RuntimeAsyncBodyEntry,
    AsyncCallable,
    ASYNC_ENTRY
);
runtime_family!(
    async_generator,
    RuntimeAsyncGeneratorBodyEntry,
    AsyncGeneratorCallable,
    ASYNC_GENERATOR_ENTRY
);
