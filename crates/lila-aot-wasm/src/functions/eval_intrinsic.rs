//! The direct-eval identity is the defining Realm's once-published callable.

use super::*;
use crate::gc_types::{
    FunctionObject, GcLocal, GcNullability, IntrinsicTable, RealmRecord, RealmRecordSchema,
    StoredValue, ValueLocals,
};

impl FunctionBuilder<'_> {
    pub(crate) fn emit_initialize_realm_eval_intrinsic(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        eval: &GcLocal<FunctionObject>,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        let intrinsics = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<RealmRecord>()
                .field(RealmRecordSchema::INTRINSICS)
                .read(realm, schema, function)
                .reference(),
            function,
        );
        let index = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(
            NonArrayRealmIntrinsicSlot::EvalFunction.gc_index() as i32,
        ));
        index.store(function);
        let previous = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<IntrinsicTable>()
                .read(&intrinsics, index, schema, function)
                .reference(),
            function,
        );
        let value = schema.reserve_value_local(function);
        schema
            .struct_type::<StoredValue>()
            .read_into(&previous, &value, schema, function);
        value.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        value.set_reference(eval, schema, function);
        self.emit_store_non_array_realm_intrinsic(
            realm,
            NonArrayRealmIntrinsicSlot::EvalFunction,
            &value,
            function,
        );
        value.clear(function);
        previous.clear(function);
        intrinsics.clear(function);
        schema.release_i32_local(index, function);
    }

    pub(crate) fn emit_load_realm_eval_intrinsic_to_local(
        &mut self,
        realm: &GcLocal<RealmRecord>,
        result: &ValueLocals,
        function: &mut Function,
    ) {
        let schema = self.runtime_schema();
        self.emit_load_non_array_realm_intrinsic(
            realm,
            NonArrayRealmIntrinsicSlot::EvalFunction,
            result,
            function,
        );
        result.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<FunctionObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        result.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Function.tag()));
        function.instruction(&Instruction::I32Eq);
        function.instruction(&Instruction::I32And);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        function.instruction(&Instruction::Unreachable);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
    }
}
