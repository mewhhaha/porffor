use super::*;
use crate::gc_types::{FunctionObject, GcI32Constant, GcNullability, I32Local, ValueLocals};

/// A constructor selected from the executing builtin's immutable intrinsic
/// table. Its full value stays rooted until typed-array construction consumes it.
#[must_use]
pub(crate) struct CurrentFunctionRealmTypedArrayConstructor {
    value: ValueLocals,
}

impl CurrentFunctionRealmTypedArrayConstructor {
    pub(crate) fn value(&self) -> &ValueLocals {
        &self.value
    }
    pub(crate) fn clear(self, function: &mut Function) {
        self.value.clear(function);
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_current_function_realm_typed_array_constructor(
        &mut self,
        kind_local: I32Local,
        function: &mut Function,
    ) -> Result<CurrentFunctionRealmTypedArrayConstructor, EmitError> {
        let schema = self.runtime_schema();
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_current_function_realm(function), function);
        let value = schema.reserve_value_local(function);
        value.set_undefined(function);
        for kind in TypedArrayElementKind::ALL {
            kind_local.load(function);
            function.instruction(&Instruction::I32Const(kind.encode()));
            function.instruction(&Instruction::I32Eq);
            self.open_frame(ControlFrameKind::If, function);
            self.emit_load_non_array_realm_intrinsic(
                &realm,
                NonArrayRealmIntrinsicSlot::typed_array_constructor_identity(kind),
                &value,
                function,
            );
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        // Corrupt kinds and unpopulated intrinsic slots never select a public
        // constructor or another Realm's default.
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Function.tag()));
        function.instruction(&Instruction::I32Eq);
        value.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<FunctionObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        realm.clear(function);
        Ok(CurrentFunctionRealmTypedArrayConstructor { value })
    }
}
