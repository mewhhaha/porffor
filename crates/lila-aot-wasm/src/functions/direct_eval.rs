//! Prepared direct eval dispatch retains source Realm, context and whole completion.

use super::*;
use crate::gc_types::{
    CompletionLocals, GcLocal, RealmRecord, StoredValue, StringValue, ValueArray, ValueLocals,
};
use lila_ir::DirectEvalContextIr;

impl FunctionBuilder<'_> {
    pub(super) fn emit_direct_eval_call(
        &mut self,
        context: &DirectEvalContextIr,
        callee_expression: &TypedExpr,
        this_expression: Option<&TypedExpr>,
        arguments: &[TypedExpr],
        continuation: &CallContinuation,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let callee = schema.reserve_value_local(function);
        let receiver = schema.reserve_value_local(function);
        receiver.set_undefined(function);
        self.compile_expr_to_value(callee_expression, &callee, function)?;
        self.emit_propagate_current_throw_if_needed(function);
        if let Some(expression) = this_expression {
            self.compile_expr_to_value(expression, &receiver, function)?;
            self.emit_propagate_current_throw_if_needed(function);
        }
        let list = self.emit_call_args_vector(arguments, function)?;
        self.emit_direct_eval_or_call_with_argv(
            context,
            &callee,
            &receiver,
            &list,
            continuation,
            output,
            function,
        )?;
        list.clear(function);
        receiver.clear(function);
        callee.clear(function);
        Ok(())
    }

    pub(crate) fn emit_direct_eval_or_call_with_argv(
        &mut self,
        context: &DirectEvalContextIr,
        callee: &ValueLocals,
        receiver: &ValueLocals,
        arguments: &GcLocal<ValueArray>,
        continuation: &CallContinuation,
        output: &ValueLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let realm = self.load_current_realm(function);
        let intrinsic = schema.reserve_value_local(function);
        self.emit_load_realm_eval_intrinsic_to_local(&realm, &intrinsic, function);
        let pending = schema.reserve_completion(function);
        pending.initialize(function);
        callee.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Function.tag()));
        function.instruction(&Instruction::I32Eq);
        callee.reference().load(function);
        intrinsic.reference().load(function);
        function.instruction(&Instruction::RefEq);
        function.instruction(&Instruction::I32And);
        self.open_frame(ControlFrameKind::If, function);
        self.emit_direct_eval_argument(context, &realm, arguments, &pending, function)?;
        function.instruction(&Instruction::Else);
        match continuation {
            CallContinuation::Continue => self.emit_function_or_proxy_call_with_argv(
                callee, receiver, arguments, &pending, function,
            )?,
            CallContinuation::Return => {
                self.emit_prepared_tail_call(callee, receiver, arguments, function)?;
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.completion().copy_from(&pending, function);
        self.emit_propagate_current_throw_if_needed(function);
        output.copy_from(pending.value(), function);
        pending.clear(function);
        intrinsic.clear(function);
        realm.clear(function);
        Ok(())
    }

    fn emit_direct_eval_argument(
        &mut self,
        context: &DirectEvalContextIr,
        realm: &GcLocal<RealmRecord>,
        arguments: &GcLocal<ValueArray>,
        pending: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let argument = schema.reserve_value_local(function);
        argument.set_undefined(function);
        schema
            .array_type::<ValueArray>()
            .length(arguments, schema, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, function);
        let zero = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        zero.store(function);
        let stored = schema.reserve_gc_local(function).initialize(
            schema
                .array_type::<ValueArray>()
                .read(arguments, zero, schema, function)
                .reference(),
            function,
        );
        schema
            .struct_type::<StoredValue>()
            .read_into(&stored, &argument, schema, function);
        stored.clear(function);
        schema.release_i32_local(zero, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        pending.set_normal(&argument, function);
        argument.tag().load(function);
        function.instruction(&Instruction::I32Const(WasmRuntimeValueTag::String.tag()));
        function.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, function);
        let source = schema.reserve_gc_local(function).initialize(
            argument.cast_reference::<StringValue>(schema, function),
            function,
        );
        self.emit_direct_eval_script_dispatch(context, realm, &source, pending, function)?;
        source.clear(function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        argument.clear(function);
        Ok(())
    }

    fn emit_direct_eval_script_dispatch(
        &mut self,
        context: &DirectEvalContextIr,
        realm: &GcLocal<RealmRecord>,
        source: &GcLocal<StringValue>,
        pending: &CompletionLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries = self
            .functions
            .prepared_scripts()
            .iter()
            .filter(|entry| entry.kind == PreparedScriptKind::DirectEval(context.clone()))
            .cloned()
            .collect::<Vec<_>>();
        let schema = self.runtime_schema();
        let ascii_case_folding = schema.reserve_i32_local(function);
        function.instruction(&Instruction::I32Const(0));
        ascii_case_folding.store(function);
        let equal = schema.reserve_i32_local(function);
        let done = self.open_frame(ControlFrameKind::Block, function);
        for entry in entries {
            let expected = schema.reserve_gc_local(function).initialize(
                self.emit_interned_string_reference(&entry.source, function)?,
                function,
            );
            self.emit_gc_string_equality(source, &expected, ascii_case_folding, equal, function);
            expected.clear(function);
            equal.load(function);
            self.open_frame(ControlFrameKind::If, function);
            let saved_realm = self.load_current_realm(function);
            self.replace_current_realm(realm, function);
            match &entry.outcome {
                PreparedScriptOutcome::DeferredSyntaxError { .. } => {
                    let prototype = schema.reserve_value_local(function);
                    self.emit_load_non_array_realm_intrinsic(
                        realm,
                        NonArrayRealmIntrinsicSlot::SyntaxErrorPrototype,
                        &prototype,
                        function,
                    );
                    let message = self.strings.source_runtime_error_message(
                        SourceRuntimeErrorMessage::PreparedScript(&entry.outcome),
                    )?;
                    self.emit_throw_runtime_error_with_prototype(
                        NativeErrorKind::SyntaxError,
                        message,
                        &prototype,
                        pending,
                        function,
                    )?;
                    prototype.clear(function);
                }
                PreparedScriptOutcome::Executable(unit) => {
                    let planned = self
                        .functions
                        .get(&unit.id.function_id())
                        .ok_or_else(|| {
                            EmitError::unsupported(
                                "prepared direct eval has no planned Script entry",
                            )
                        })?
                        .entry
                        .clone();
                    let entry = planned.prepared_script()?;
                    let variable_environment =
                        self.emit_eval_variable_environment_to_local(function);
                    let lexical_environment = schema
                        .reserve_gc_local(function)
                        .initialize(self.current_environment().load(schema, function), function);
                    let private_environment = schema.reserve_gc_local(function).initialize(
                        self.current_private_environment().load(schema, function),
                        function,
                    );
                    let invocation = self.emit_capture_direct_eval_invocation(function)?;
                    pending.store_call(
                        entry.emit_call(
                            crate::function_entry::PreparedScriptInputs::new(
                                &lexical_environment,
                                invocation.this_value(),
                                invocation.new_target(),
                                &variable_environment,
                                &private_environment,
                                invocation.context(),
                            ),
                            schema,
                            function,
                        ),
                        function,
                    );
                    self.release_direct_eval_invocation(invocation, function);
                    private_environment.clear(function);
                    lexical_environment.clear(function);
                    variable_environment.clear(function);
                }
            }
            self.replace_current_realm(&saved_realm, function);
            saved_realm.clear(function);
            self.emit_branch_to_target(done, function);
            self.pop_control(ControlFrameKind::If);
            function.instruction(&Instruction::End);
        }
        self.emit_reject_dynamic_source(lila_ir::DynamicSourceRuntimeOperation::Eval, function);
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        schema.release_i32_local(equal, function);
        schema.release_i32_local(ascii_case_folding, function);
        Ok(())
    }
}
