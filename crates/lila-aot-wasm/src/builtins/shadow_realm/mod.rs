//! Native ShadowRealm instances retain one strong Realm edge. Constructor
//! prototype selection and evaluation boundaries use the shared Realm owners.

mod import_value;
mod wrapping;

use crate::functions::OrdinaryDefaultPrototype;
use crate::gc_types::{
    CompletionLocals, GcLocal, GcNullability, GcOperand, RealmRecord, ShadowRealmObject,
    ShadowRealmObjectSchema, StringValue,
};
use crate::{
    CompletionKind, ControlFrameKind, ControlTarget, EmitError, Function, FunctionBuilder,
    Instruction, RuntimeErrorMessage, WasmRuntimeValueTag,
};

impl FunctionBuilder<'_> {
    /// The brand check is independent of prototype lookup, callable proxies
    /// and public properties. Both native methods acquire the receiver Realm
    /// before validating or converting their first argument.
    pub(crate) fn emit_shadow_realm_receiver_realm(
        &mut self,
        output: &CompletionLocals,
        exit: ControlTarget,
        function: &mut Function,
    ) -> Result<GcLocal<RealmRecord>, EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(function);
        self.compile_this_to_locals(&receiver, function)?;
        receiver.reference().load(function);
        function.instruction(&Instruction::RefTestNonNull(
            schema
                .reference_type::<ShadowRealmObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::SHADOW_REALM_INCOMPATIBLE_RECEIVER,
            output,
            function,
        )?;
        receiver.set_undefined(function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let record = schema.reserve_gc_local(function).initialize(
            receiver.cast_reference::<ShadowRealmObject>(schema, function),
            function,
        );
        let realm = schema.reserve_gc_local(function).initialize(
            schema
                .struct_type::<ShadowRealmObject>()
                .field(ShadowRealmObjectSchema::REALM)
                .read(&record, schema, function)
                .reference(),
            function,
        );
        record.clear(function);
        receiver.clear(function);
        Ok(realm)
    }

    pub(crate) fn compile_shadow_realm_constructor_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let new_target = schema.reserve_value_local(function);
        let prototype = schema.reserve_completion(function);
        let output = schema.reserve_completion(function);
        self.compile_new_target_to_locals(&new_target, function)?;
        output.initialize(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        new_target.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::SHADOW_REALM_REQUIRES_NEW,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);

        // GetPrototypeFromConstructor can run user code or throw. Complete it
        // before allocating and initializing the independent inner Realm.
        self.emit_get_prototype_from_constructor(
            &new_target,
            OrdinaryDefaultPrototype::ShadowRealm,
            &prototype,
            function,
        )?;
        prototype.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        output.copy_from(&prototype, function);
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let header = schema.reserve_gc_local(function).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(prototype.value()), function)?,
            function,
        );
        let realm = schema
            .reserve_gc_local(function)
            .initialize(self.emit_alloc_realm_record(0, 1, function)?, function);
        let global = self.emit_created_realm_global_object(&realm, function)?;
        global.clear(function);
        let record = schema.reserve_gc_local(function).initialize(
            schema.struct_type::<ShadowRealmObject>().construct(
                (
                    GcOperand::reference(&header, schema),
                    GcOperand::reference(&realm, schema),
                ),
                function,
            ),
            function,
        );
        output.value().set_reference(&record, schema, function);
        output.set_kind(CompletionKind::Normal, function);
        record.clear(function);
        realm.clear(function);
        header.clear(function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        output.clear(function);
        prototype.clear(function);
        new_target.clear(function);
        Ok(())
    }

    pub(crate) fn compile_shadow_realm_evaluate_builtin(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        let pending = schema.reserve_completion(function);
        let output = schema.reserve_completion(function);
        let source_parsed = schema.reserve_i32_local(function);
        output.initialize(function);
        pending.initialize(function);
        argument.set_undefined(function);
        let exit = self.open_frame(ControlFrameKind::Block, function);
        let realm = self.emit_shadow_realm_receiver_realm(&output, exit, function)?;
        self.emit_builtin_arg_to_value(0, &argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        function.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::SHADOW_REALM_SOURCE_STRING,
            &output,
            function,
        )?;
        self.emit_branch_to_target(exit, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        let source = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        let caller_realm = self.load_current_realm(function);
        self.emit_evaluate_shadow_realm_source(&realm, &source, &pending, source_parsed, function)?;
        source_parsed.load(function);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        // Parsing already produced a SyntaxError in this active method Realm.
        output.copy_from(&pending, function);
        function.instruction(&Instruction::Else);
        pending.kind().load(function);
        function.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_shadow_realm_wrapped_value(pending.value(), &caller_realm, &output, function)?;
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::SHADOW_REALM_ABRUPT,
            &output,
            function,
        )?;
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&output, function);
        caller_realm.clear(function);
        source.clear(function);
        realm.clear(function);
        schema.release_i32_local(source_parsed, function);
        output.clear(function);
        pending.clear(function);
        argument.clear(function);
        Ok(())
    }
}
