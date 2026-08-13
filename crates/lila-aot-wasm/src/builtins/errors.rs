use super::super::*;
use crate::functions::FunctionRealmRevokedRoute;
use lila_ir::NativeErrorKind;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(super) enum ErrorBuiltin {
    IsError,
    Constructor(NativeErrorKind),
    PrototypeToString,
}

fn native_error_kind(name: &str) -> Result<NativeErrorKind, EmitError> {
    NativeErrorKind::from_str(name).ok_or_else(|| {
        EmitError::unsupported(format!(
            "internal wasm-aot error emitter received unknown native error name `{name}`"
        ))
    })
}

#[derive(Clone, Copy)]
pub(crate) enum NewTargetPrototypeFallback {
    CurrentGlobal,
    FunctionSnapshot(u64),
    RealmIntrinsic(u64),
}

#[must_use = "the prepared Error name must be consumed before reading message"]
struct PreparedErrorNameLocal(u32);

impl PreparedErrorNameLocal {
    fn into_local(self) -> u32 {
        self.0
    }
}

impl<'a> FunctionBuilder<'a> {
    fn emit_native_error_constructor_wrapper(
        &mut self,
        error_kind: NativeErrorKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        // Native error subclasses direct-call the shared `Error` body, so its
        // real body must be emitted — see `FunctionMetaRegistry`.
        self.functions
            .record_standard_builtin(StandardBuiltinId::ErrorConstructor);
        let error_wasm_index = self
            .functions
            .get(&StandardBuiltinId::ErrorConstructor.function_id())
            .map(|meta| meta.wasm_index)
            .ok_or_else(|| {
                EmitError::unsupported(
                    "unsupported in lila wasm-aot first slice: missing builtin meta `Error`",
                )
            })?;
        let builtin = error_kind.constructor();
        let constructor_global_index = standard_builtin_constructor_global_index(builtin)
            .ok_or_else(|| {
                EmitError::unsupported(format!(
                    "unsupported in lila wasm-aot first slice: missing constructor global for `{}`",
                    builtin.debug_name()
                ))
            })?;
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(constructor_global_index));
        function.instruction(&Instruction::LocalSet(new_target_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(new_target_tag_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::LocalGet(
            self.this_payload_local
                .expect("native error wrapper must use function ABI"),
        ));
        function.instruction(&Instruction::LocalGet(
            self.this_tag_local
                .expect("native error wrapper must use function ABI"),
        ));
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::LocalGet(self.argc_param_local()));
        function.instruction(&Instruction::LocalGet(self.argv_param_local()));
        function.instruction(&Instruction::Call(error_wasm_index));
        self.store_call_results(self.result_local, self.result_tag_local, function);

        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    pub(super) fn emit_error_builtin(
        &mut self,
        builtin: ErrorBuiltin,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match builtin {
            ErrorBuiltin::IsError => {
                let arg_payload_local = self.reserve_temp_local();
                let arg_tag_local = self.reserve_temp_local();
                let brand_local = self.reserve_temp_local();

                self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
                function.instruction(&Instruction::LocalSet(self.result_tag_local));
                self.emit_is_heap_object_like_tag_i32(arg_tag_local, function);
                function.instruction(&Instruction::If(BlockType::Empty));
                self.load_i64_to_local_from_offset(
                    arg_payload_local,
                    HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                    brand_local,
                    function,
                );
                function.instruction(&Instruction::LocalGet(brand_local));
                function.instruction(&Instruction::I64Const(OBJECT_INTERNAL_BRAND_ERROR as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::I64ExtendI32U);
                function.instruction(&Instruction::LocalSet(self.result_local));
                function.instruction(&Instruction::End);

                self.release_temp_local(brand_local);
                self.release_temp_local(arg_tag_local);
                self.release_temp_local(arg_payload_local);
            }
            ErrorBuiltin::Constructor(error_kind) => match error_kind {
                NativeErrorKind::AggregateError => {
                    let errors_arg_payload_local = self.reserve_temp_local();
                    let errors_arg_tag_local = self.reserve_temp_local();
                    let message_arg_payload_local = self.reserve_temp_local();
                    let message_arg_tag_local = self.reserve_temp_local();
                    let errors_payload_local = self.reserve_temp_local();
                    let message_payload_local = self.reserve_temp_local();
                    let prototype_payload_local = self.reserve_temp_local();
                    self.emit_builtin_arg_to_locals(
                        0,
                        errors_arg_payload_local,
                        errors_arg_tag_local,
                        function,
                    );
                    self.emit_builtin_arg_to_locals(
                        1,
                        message_arg_payload_local,
                        message_arg_tag_local,
                        function,
                    );
                    self.emit_aggregate_error_new_target_prototype_to_local(
                        prototype_payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(message_arg_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_aggregate_error_iterable_to_list_payload(
                        errors_arg_payload_local,
                        errors_arg_tag_local,
                        errors_payload_local,
                        function,
                    )?;
                    self.emit_alloc_aggregate_error_instance_from_locals(
                        None,
                        errors_payload_local,
                        prototype_payload_local,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_install_error_cause_from_arg(self.result_local, 2, function)?;
                    function.instruction(&Instruction::Else);
                    self.emit_value_to_string_payload(
                        message_arg_payload_local,
                        message_arg_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(message_payload_local));
                    self.emit_aggregate_error_iterable_to_list_payload(
                        errors_arg_payload_local,
                        errors_arg_tag_local,
                        errors_payload_local,
                        function,
                    )?;
                    self.emit_alloc_aggregate_error_instance_from_locals(
                        Some(message_payload_local),
                        errors_payload_local,
                        prototype_payload_local,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_install_error_cause_from_arg(self.result_local, 2, function)?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(prototype_payload_local);
                    self.release_temp_local(message_payload_local);
                    self.release_temp_local(errors_payload_local);
                    self.release_temp_local(message_arg_tag_local);
                    self.release_temp_local(message_arg_payload_local);
                    self.release_temp_local(errors_arg_tag_local);
                    self.release_temp_local(errors_arg_payload_local);
                    return Ok(());
                }
                NativeErrorKind::SuppressedError => {
                    let error_arg_payload_local = self.reserve_temp_local();
                    let error_arg_tag_local = self.reserve_temp_local();
                    let suppressed_arg_payload_local = self.reserve_temp_local();
                    let suppressed_arg_tag_local = self.reserve_temp_local();
                    let message_arg_payload_local = self.reserve_temp_local();
                    let message_arg_tag_local = self.reserve_temp_local();
                    let message_payload_local = self.reserve_temp_local();
                    let prototype_payload_local = self.reserve_temp_local();
                    self.emit_builtin_arg_to_locals(
                        0,
                        error_arg_payload_local,
                        error_arg_tag_local,
                        function,
                    );
                    self.emit_builtin_arg_to_locals(
                        1,
                        suppressed_arg_payload_local,
                        suppressed_arg_tag_local,
                        function,
                    );
                    self.emit_builtin_arg_to_locals(
                        2,
                        message_arg_payload_local,
                        message_arg_tag_local,
                        function,
                    );
                    self.emit_error_new_target_prototype_to_local(
                        SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
                        Some(HEAP_FUNCTION_REALM_SUPPRESSED_ERROR_PROTOTYPE_OFFSET),
                        prototype_payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(message_arg_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_alloc_suppressed_error_instance_from_locals(
                        None,
                        error_arg_payload_local,
                        error_arg_tag_local,
                        suppressed_arg_payload_local,
                        suppressed_arg_tag_local,
                        prototype_payload_local,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    self.emit_value_to_string_payload(
                        message_arg_payload_local,
                        message_arg_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalSet(message_payload_local));
                    self.emit_alloc_suppressed_error_instance_from_locals(
                        Some(message_payload_local),
                        error_arg_payload_local,
                        error_arg_tag_local,
                        suppressed_arg_payload_local,
                        suppressed_arg_tag_local,
                        prototype_payload_local,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(prototype_payload_local);
                    self.release_temp_local(message_payload_local);
                    self.release_temp_local(message_arg_tag_local);
                    self.release_temp_local(message_arg_payload_local);
                    self.release_temp_local(suppressed_arg_tag_local);
                    self.release_temp_local(suppressed_arg_payload_local);
                    self.release_temp_local(error_arg_tag_local);
                    self.release_temp_local(error_arg_payload_local);
                    return Ok(());
                }
                NativeErrorKind::EvalError
                | NativeErrorKind::RangeError
                | NativeErrorKind::ReferenceError
                | NativeErrorKind::SyntaxError
                | NativeErrorKind::TypeError
                | NativeErrorKind::URIError => {
                    self.emit_native_error_constructor_wrapper(error_kind, function)?;
                    return Ok(());
                }
                NativeErrorKind::Error => {
                    let arg_payload_local = self.reserve_temp_local();
                    let arg_tag_local = self.reserve_temp_local();
                    let message_payload_local = self.reserve_temp_local();
                    let prototype_payload_local = self.reserve_temp_local();
                    self.emit_builtin_arg_to_locals(0, arg_payload_local, arg_tag_local, function);
                    self.emit_error_new_target_prototype_to_local(
                        error_prototype_global_index(error_kind),
                        Some(error_realm_prototype_offset(error_kind)),
                        prototype_payload_local,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(arg_tag_local));
                    function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    self.emit_alloc_error_instance_from_locals(
                        prototype_payload_local,
                        None,
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_install_error_cause_from_arg(self.result_local, 1, function)?;
                    function.instruction(&Instruction::Else);
                    self.emit_value_to_string_payload(arg_payload_local, arg_tag_local, function)?;
                    function.instruction(&Instruction::LocalSet(message_payload_local));
                    self.emit_alloc_error_instance_from_locals(
                        prototype_payload_local,
                        Some(message_payload_local),
                        self.result_local,
                        self.result_tag_local,
                        function,
                    )?;
                    self.emit_install_error_cause_from_arg(self.result_local, 1, function)?;
                    function.instruction(&Instruction::End);
                    self.release_temp_local(prototype_payload_local);
                    self.release_temp_local(message_payload_local);
                    self.release_temp_local(arg_tag_local);
                    self.release_temp_local(arg_payload_local);
                }
            },
            ErrorBuiltin::PrototypeToString => {
                self.emit_error_prototype_to_string(function)?;
            }
        }
        Ok(())
    }

    fn emit_error_prototype_to_string(&mut self, function: &mut Function) -> Result<(), EmitError> {
        let receiver_payload_local = self.this_payload_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Error.prototype.toString receiver",
            )
        })?;
        let receiver_tag_local = self.this_tag_local.ok_or_else(|| {
            EmitError::unsupported(
                "unsupported in lila wasm-aot first slice: missing Error.prototype.toString receiver",
            )
        })?;

        self.emit_is_heap_object_like_tag_i32(receiver_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        let prepared_name = self.emit_error_to_string_prepare_name(
            receiver_payload_local,
            receiver_tag_local,
            function,
        )?;
        self.emit_error_to_string_message_and_result(
            receiver_payload_local,
            receiver_tag_local,
            prepared_name,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_throw_current_function_realm_type_error(
            "Error.prototype.toString receiver is not object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_error_to_string_prepare_name(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        function: &mut Function,
    ) -> Result<PreparedErrorNameLocal, EmitError> {
        // Reserve the local that crosses the phase boundary before all of this
        // phase's transient locals so the latter can be released in LIFO order.
        let name_string_local = self.reserve_temp_local();
        let name_key_local = self.reserve_temp_local();
        let name_payload_local = self.reserve_temp_local();
        let name_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(name_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            name_key_local,
            name_payload_local,
            name_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload(ERROR_NAME)));
        function.instruction(&Instruction::LocalSet(name_string_local));
        function.instruction(&Instruction::Else);
        self.emit_error_to_string_value_to_string_local(
            name_payload_local,
            name_tag_local,
            name_string_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(name_tag_local);
        self.release_temp_local(name_payload_local);
        self.release_temp_local(name_key_local);
        Ok(PreparedErrorNameLocal(name_string_local))
    }

    fn emit_error_to_string_message_and_result(
        &mut self,
        receiver_payload_local: u32,
        receiver_tag_local: u32,
        prepared_name: PreparedErrorNameLocal,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let name_string_local = prepared_name.into_local();
        let message_string_local = self.reserve_temp_local();
        let message_key_local = self.reserve_temp_local();
        let message_payload_local = self.reserve_temp_local();
        let message_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(message_key_local));
        self.emit_object_read(
            receiver_payload_local,
            receiver_tag_local,
            receiver_payload_local,
            receiver_tag_local,
            message_key_local,
            message_payload_local,
            message_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(message_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(message_string_local));
        function.instruction(&Instruction::Else);
        self.emit_error_to_string_value_to_string_local(
            message_payload_local,
            message_tag_local,
            message_string_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(message_tag_local);
        self.release_temp_local(message_payload_local);
        self.release_temp_local(message_key_local);

        let separator_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(self.strings.payload("")));
        function.instruction(&Instruction::LocalSet(separator_local));
        self.emit_string_payload_equality_i32(name_string_local, separator_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(message_string_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        self.emit_string_payload_equality_i32(message_string_local, separator_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_string_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload(": ")));
        function.instruction(&Instruction::LocalSet(separator_local));
        self.emit_concat_string_payloads_local(name_string_local, separator_local, function)?;
        function.instruction(&Instruction::LocalSet(name_string_local));
        self.emit_concat_string_payloads_local(name_string_local, message_string_local, function)?;
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(separator_local);
        self.release_temp_local(message_string_local);
        self.release_temp_local(name_string_local);
        Ok(())
    }

    fn emit_error_to_string_value_to_string_local(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        string_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let primitive = self.emit_tagged_to_primitive_locals_in_current_function_realm(
            ToPrimitiveHint::String,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_current_function_realm_primitive_to_string_local(
            primitive,
            string_payload_local,
            function,
        )
    }

    pub(crate) fn emit_alloc_error_instance_from_locals(
        &mut self,
        prototype_payload_local: u32,
        message_payload_local: Option<u32>,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_install_error_cause_from_arg(
        &mut self,
        error_object_local: u32,
        options_arg_index: usize,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let options_payload_local = self.reserve_temp_local();
        let options_tag_local = self.reserve_temp_local();
        let cause_key_local = self.reserve_temp_local();
        let has_cause_local = self.reserve_temp_local();
        let cause_payload_local = self.reserve_temp_local();
        let cause_tag_local = self.reserve_temp_local();

        self.emit_builtin_arg_to_locals(
            options_arg_index,
            options_payload_local,
            options_tag_local,
            function,
        );
        self.emit_is_heap_object_like_tag_i32(options_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("cause")));
        function.instruction(&Instruction::LocalSet(cause_key_local));
        self.emit_object_has_property_i32(
            options_payload_local,
            options_tag_local,
            cause_key_local,
            has_cause_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(has_cause_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_object_read(
            options_payload_local,
            options_tag_local,
            options_payload_local,
            options_tag_local,
            cause_key_local,
            cause_payload_local,
            cause_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            cause_payload_local,
            cause_tag_local,
            function,
        )?;
        self.emit_object_define_data(
            error_object_local,
            cause_key_local,
            cause_payload_local,
            cause_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(cause_tag_local);
        self.release_temp_local(cause_payload_local);
        self.release_temp_local(has_cause_local);
        self.release_temp_local(cause_key_local);
        self.release_temp_local(options_tag_local);
        self.release_temp_local(options_payload_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_aggregate_error_instance_from_locals(
        &mut self,
        message_payload_local: Option<u32>,
        errors_payload_local: u32,
        prototype_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("errors")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            errors_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_alloc_suppressed_error_instance_from_locals(
        &mut self,
        message_payload_local: Option<u32>,
        error_payload_local: u32,
        error_tag_local: u32,
        suppressed_payload_local: u32,
        suppressed_tag_local: u32,
        prototype_payload_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        self.emit_alloc_plain_object_with_prototype(Some(prototype_payload_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        if let Some(message_payload_local) = message_payload_local {
            function.instruction(&Instruction::I64Const(self.strings.payload("message")));
            function.instruction(&Instruction::LocalSet(key_local));
            function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
            function.instruction(&Instruction::LocalSet(value_tag_local));
            self.emit_object_define_data(
                object_local,
                key_local,
                message_payload_local,
                value_tag_local,
                function,
            )?;
        }
        function.instruction(&Instruction::I64Const(self.strings.payload("error")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            error_payload_local,
            error_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("suppressed")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            suppressed_payload_local,
            suppressed_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.release_temp_local(value_tag_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_error_new_target_prototype_to_local(
        &mut self,
        default_prototype_global_index: u32,
        fallback_realm_prototype_offset: Option<u64>,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let fallback = fallback_realm_prototype_offset
            .map(NewTargetPrototypeFallback::FunctionSnapshot)
            .unwrap_or(NewTargetPrototypeFallback::CurrentGlobal);
        let prototype_tag_local = self.reserve_temp_local();
        let result = self.emit_new_target_prototype_to_locals(
            default_prototype_global_index,
            fallback,
            prototype_payload_local,
            prototype_tag_local,
            function,
        );
        self.release_temp_local(prototype_tag_local);
        result
    }

    pub(crate) fn emit_new_target_prototype_to_locals(
        &mut self,
        default_prototype_global_index: u32,
        fallback: NewTargetPrototypeFallback,
        prototype_payload_local: u32,
        prototype_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_key_local = self.reserve_temp_local();
        let realm_source_payload_local = self.reserve_temp_local();
        let proxy_handler_payload_local = self.reserve_temp_local();
        let proxy_target_tag_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_payload_local));
        function.instruction(&Instruction::LocalSet(realm_source_payload_local));
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            new_target_payload_local,
            HEAP_OBJECT_BOXED_KIND_OFFSET,
            proxy_handler_payload_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_handler_payload_local));
        function.instruction(&Instruction::I64Const(PROXY_HANDLER_PAYLOAD_MIN as i64));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            new_target_payload_local,
            HEAP_OBJECT_BOXED_TAG_OFFSET,
            proxy_target_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(proxy_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            new_target_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            realm_source_payload_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(default_prototype_global_index));
        function.instruction(&Instruction::LocalSet(prototype_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::I64Const(self.strings.payload("prototype")));
        function.instruction(&Instruction::LocalSet(prototype_key_local));
        self.emit_object_read(
            new_target_payload_local,
            new_target_tag_local,
            new_target_payload_local,
            new_target_tag_local,
            prototype_key_local,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_is_heap_object_like_tag_i32(prototype_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Else);
        match fallback {
            NewTargetPrototypeFallback::CurrentGlobal => {
                function.instruction(&Instruction::GlobalGet(default_prototype_global_index));
                function.instruction(&Instruction::LocalSet(prototype_payload_local));
            }
            NewTargetPrototypeFallback::FunctionSnapshot(offset) => {
                self.load_i64_to_local_from_offset(
                    realm_source_payload_local,
                    offset,
                    prototype_payload_local,
                    function,
                );
            }
            NewTargetPrototypeFallback::RealmIntrinsic(offset) => {
                let prototype_realm_result = self.emit_get_function_realm(
                    new_target_payload_local,
                    new_target_tag_local,
                    function,
                );
                let prototype_realm = self.emit_route_function_realm_result(
                    prototype_realm_result,
                    FunctionRealmRevokedRoute::ThrowTypeErrorAndReturn {
                        payload_local: self.result_local,
                        tag_local: self.result_tag_local,
                    },
                    function,
                )?;
                self.emit_load_realm_intrinsic_prototype_or_global(
                    prototype_realm.index(),
                    offset,
                    default_prototype_global_index,
                    prototype_payload_local,
                    function,
                );
                self.release_resolved_function_realm_local(prototype_realm);
            }
        }
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(prototype_tag_local));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(proxy_target_tag_local);
        self.release_temp_local(proxy_handler_payload_local);
        self.release_temp_local(realm_source_payload_local);
        self.release_temp_local(prototype_key_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    pub(crate) fn emit_aggregate_error_new_target_prototype_to_local(
        &mut self,
        prototype_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_error_new_target_prototype_to_local(
            AGGREGATE_ERROR_PROTOTYPE_GLOBAL_INDEX,
            Some(HEAP_FUNCTION_REALM_AGGREGATE_ERROR_PROTOTYPE_OFFSET),
            prototype_payload_local,
            function,
        )
    }

    /// Allocate the error object for a runtime-thrown error.
    ///
    /// `message` is defined from the message, not from the name. That reads as
    /// a tautology; it is the repair. This function used to define `message`
    /// from `self.strings.payload(name)` and spell its message parameter
    /// `_message`, so not even an unused-parameter warning mentioned it, and
    /// every error the runtime threw reported `e.message === e.name`:
    ///
    /// ```text
    /// try { null.x } catch (e) { print(e.name); print(e.message); }
    /// // TypeError / TypeError     (before)
    /// // TypeError / Cannot read properties of null or undefined   (after)
    /// ```
    ///
    /// The repair is one token here and could not land alone.
    /// `StringPool::payload` takes `&self`, cannot extend the pool during
    /// emission, and panics with ``string `..` must exist in pool``; because
    /// this function never asked the pool for a message, the messages reaching
    /// only it were never required to be interned. `data.rs`'s
    /// `RUNTIME_ERROR_MESSAGE_LITERALS` is the other half, and the two are one
    /// patch: either alone is compile-time clean and run-time fatal.
    ///
    /// STANDING INSTRUCTION, unchanged in force: do **not** make the message
    /// fall back to the name when the pool lookup misses. That fallback is
    /// precisely the defect above, only harder to find — the program would run,
    /// report a plausible-looking `message`, and no test would notice. A miss
    /// must stay a named panic naming the missing string, which is what turns
    /// "someone added a message and forgot to intern it" into a one-line fix
    /// instead of an archaeology exercise.
    pub(crate) fn emit_runtime_error_object(
        &mut self,
        kind: NativeErrorKind,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let name = kind.as_str();
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(error_prototype_global_index(kind)),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(object_local));
        // [[ErrorData]]. Without it a runtime-thrown error is not an error to
        // anything that reads the internal brand: `Object.prototype.toString`
        // answered "[object Object]" and `Error.isError` answered `false`, while
        // the same class constructed by user code answered "[object Error]" and
        // `true`. Same store as `emit_alloc_error_instance_from_locals`.
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        // The message, not the name. See the doc comment: the `payload(name)`
        // that used to sit here is the T24 defect.
        function.instruction(&Instruction::I64Const(self.strings.payload(message)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    /// Publishes the two throw-diagnostic globals **together**.
    ///
    /// The name global existed alone, and an uncaught throw therefore reached
    /// the host as `TypeError: wasm-aot completion: object(handle@5397552)` — a
    /// raw linear-memory address that is not stable across builds and maps to
    /// no allocation site, so ~2,488 measured cases across ~1,743 addresses
    /// carried one bit of information between them.
    ///
    /// It is one function because the pairing is the invariant: a site that
    /// sets the name and forgets the message reports a *previous*, unrelated
    /// throw's message. `None` is the explicit "this throw carries no message"
    /// answer and clears the global; it is not the same as not calling this.
    /// The only site that may set either global without coming through here is
    /// `emit_capture_throw_error_name`, which reads both off a user-thrown
    /// value and zeroes the message on entry for the same reason.
    pub(crate) fn emit_set_thrown_error_text(
        &mut self,
        name: &str,
        message: Option<&str>,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        let message_payload = message.map_or(0, |message| self.strings.payload(message));
        function.instruction(&Instruction::I64Const(message_payload));
        function.instruction(&Instruction::GlobalSet(throw_error_message_global_index(
            self.uses_heap,
        )));
    }

    pub(crate) fn emit_throw_runtime_error(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = native_error_kind(name)?;
        self.emit_throw_runtime_error_kind(kind, message, payload_local, tag_local, function)
    }

    fn emit_throw_runtime_error_kind(
        &mut self,
        kind: NativeErrorKind,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let name = kind.as_str();
        self.emit_runtime_error_object(kind, message, payload_local, tag_local, function)?;
        self.emit_set_thrown_error_text(name, Some(message), function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Throw,
            self.strings.payload(name) as i64,
            function,
        );
        Ok(())
    }

    pub(crate) fn emit_throw_current_function_realm_error(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = native_error_kind(name)?;
        let prototype_local = self.reserve_temp_local();
        let prototype_offset = error_realm_prototype_offset(kind);

        function.instruction(&Instruction::LocalGet(self.current_env_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error_kind(kind, message, payload_local, tag_local, function)?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            self.current_env_local,
            prototype_offset,
            prototype_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(prototype_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::GlobalGet(error_prototype_global_index(kind)));
        function.instruction(&Instruction::LocalSet(prototype_local));
        function.instruction(&Instruction::End);
        self.emit_throw_runtime_error_with_prototype_local_kind(
            kind,
            message,
            prototype_local,
            payload_local,
            tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.release_temp_local(prototype_local);
        Ok(())
    }

    pub(crate) fn emit_throw_current_function_realm_type_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            TYPE_ERROR_NAME,
            message,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_throw_current_function_realm_range_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            RANGE_ERROR_NAME,
            message,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_throw_current_function_realm_uri_error(
        &mut self,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_error(
            URI_ERROR_NAME,
            message,
            payload_local,
            tag_local,
            function,
        )
    }

    pub(crate) fn emit_throw_runtime_error_with_prototype_local(
        &mut self,
        name: &str,
        message: &str,
        prototype_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let kind = native_error_kind(name)?;
        self.emit_throw_runtime_error_with_prototype_local_kind(
            kind,
            message,
            prototype_local,
            payload_local,
            tag_local,
            function,
        )
    }

    fn emit_throw_runtime_error_with_prototype_local_kind(
        &mut self,
        kind: NativeErrorKind,
        message: &str,
        prototype_local: u32,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let name = kind.as_str();
        let object_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();

        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(object_local));
        // [[ErrorData]], as in `emit_runtime_error_object`: this is the same
        // error object reached through the realm-carrying prototype instead of
        // the global one, and it was missing the brand for the same reason.
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ERROR,
            function,
        );
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(name)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        function.instruction(&Instruction::I64Const(self.strings.payload(message)));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        self.emit_object_define_data(
            object_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(tag_local));
        self.emit_set_thrown_error_text(name, Some(message), function);
        function.instruction(&Instruction::LocalGet(payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind_with_aux(
            CompletionKind::Throw,
            self.strings.payload(name) as i64,
            function,
        );

        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(key_local);
        self.release_temp_local(object_local);
        Ok(())
    }

    pub(crate) fn emit_throw_runtime_error_to_active_handler(
        &mut self,
        name: &str,
        message: &str,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_runtime_error(name, message, payload_local, tag_local, function)?;
        if !self.is_main() {
            self.emit_return_current_completion(function);
        } else if let Some(target) = self.active_throw_target() {
            self.emit_branch_to_target(target, function);
        } else {
            self.emit_return_current_completion(function);
        }
        Ok(())
    }

    /// Capture, for the host, what a `throw` of an arbitrary value was.
    ///
    /// Two globals, read together by `render_wasmtime_completion`: the error's
    /// `name` (falling back to `constructor.name`, because a `Test262Error`
    /// carries no own `name`) and its `message`. The message half is what stops
    /// an uncaught user throw from reaching the host as nothing but
    /// `object(handle@5397552)`.
    ///
    /// **Both** globals are zeroed here, not by the caller. Zeroing the name
    /// used to be the caller's job — `control_flow.rs`'s `StatementIr::Throw`
    /// and `promise.rs`'s rejection path each emitted their own
    /// `I64Const(0); GlobalSet(name)` first — and that convention is exactly
    /// how a stale value from a previous throw reaches the host at whichever
    /// call site forgets. Both globals are module-lifetime, so a forgotten
    /// clear is not a missing diagnostic but a *wrong* one, attributed to the
    /// throw being captured now. Zeroing on entry makes it impossible to
    /// forget from outside this function; the emitted instruction sequence is
    /// unchanged, because the zero moved to exactly where the callers put it.
    pub(crate) fn emit_capture_throw_error_name(
        &mut self,
        payload_local: u32,
        tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let constructor_payload_local = self.reserve_temp_local();
        let constructor_tag_local = self.reserve_temp_local();
        let name_payload_local = self.reserve_temp_local();
        let name_tag_local = self.reserve_temp_local();
        let message_payload_local = self.reserve_temp_local();
        let message_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::GlobalSet(throw_error_message_global_index(
            self.uses_heap,
        )));

        self.emit_is_heap_object_like_tag_i32(tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            payload_local,
            tag_local,
            key_local,
            name_payload_local,
            name_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_payload_local));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::End);
        // `.message`, read with the same non-calling data-property read as
        // `.name` so capturing a diagnostic can never run user code and change
        // the very completion it is describing. There is deliberately no
        // `constructor.message` fallback: `.name` has one because the error
        // classes put `name` on the prototype, whereas a missing `message` means
        // the thrown value simply has none, and inventing one would put the host
        // back to guessing.
        function.instruction(&Instruction::I64Const(self.strings.payload("message")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            payload_local,
            tag_local,
            key_local,
            message_payload_local,
            message_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(message_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(message_payload_local));
        function.instruction(&Instruction::GlobalSet(throw_error_message_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::GlobalGet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("constructor")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            payload_local,
            tag_local,
            key_local,
            constructor_payload_local,
            constructor_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(constructor_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(self.strings.payload("name")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_data_property_read_no_call(
            constructor_payload_local,
            constructor_tag_local,
            key_local,
            name_payload_local,
            name_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(name_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(name_payload_local));
        function.instruction(&Instruction::GlobalSet(throw_error_name_global_index(
            self.uses_heap,
        )));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(key_local);
        self.release_temp_local(message_tag_local);
        self.release_temp_local(message_payload_local);
        self.release_temp_local(name_tag_local);
        self.release_temp_local(name_payload_local);
        self.release_temp_local(constructor_tag_local);
        self.release_temp_local(constructor_payload_local);
        Ok(())
    }

    pub(crate) fn emit_aggregate_error_iterable_to_list_payload(
        &mut self,
        input_payload_local: u32,
        input_tag_local: u32,
        payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let key_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let iterator_payload_local = self.reserve_temp_local();
        let iterator_tag_local = self.reserve_temp_local();
        let next_payload_local = self.reserve_temp_local();
        let next_tag_local = self.reserve_temp_local();
        let result_payload_local = self.reserve_temp_local();
        let result_tag_local = self.reserve_temp_local();
        let done_payload_local = self.reserve_temp_local();
        let done_tag_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_like_snapshot_payload(
            input_payload_local,
            input_tag_local,
            payload_local,
            "AggregateError errors input must be iterable",
            function,
        )?;
        function.instruction(&Instruction::Else);
        function.instruction(&Instruction::LocalGet(input_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Arguments.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_array_like_snapshot_payload(
            input_payload_local,
            input_tag_local,
            payload_local,
            "AggregateError errors input must be iterable",
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.emit_is_heap_object_like_tag_i32(input_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.iterator"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            input_payload_local,
            input_tag_local,
            input_payload_local,
            input_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError errors input must be iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.emit_function_handle_call(
            method_payload_local,
            method_tag_local,
            Some((input_payload_local, Some(input_tag_local))),
            &[],
            iterator_payload_local,
            iterator_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(iterator_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError iterator method must return object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("next")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            iterator_payload_local,
            iterator_tag_local,
            iterator_payload_local,
            iterator_tag_local,
            key_local,
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            next_payload_local,
            next_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(next_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError iterator next must be callable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        self.emit_alloc_array_payload_with_length(index_local, payload_local, function)?;

        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.emit_function_handle_call(
            next_payload_local,
            next_tag_local,
            Some((iterator_payload_local, Some(iterator_tag_local))),
            &[],
            result_payload_local,
            result_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_is_heap_object_like_tag_i32(result_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError iterator next result must be object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(self.strings.payload("done")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            done_payload_local,
            done_tag_local,
            function,
        )?;
        self.compile_truthy_tagged_i32(done_tag_local, done_payload_local, function)?;
        function.instruction(&Instruction::BrIf(1));

        function.instruction(&Instruction::I64Const(self.strings.payload("value")));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_read(
            result_payload_local,
            result_tag_local,
            result_payload_local,
            result_tag_local,
            key_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_propagate_throw_from_locals_if_needed(
            value_payload_local,
            value_tag_local,
            function,
        )?;
        self.emit_array_write(
            payload_local,
            index_local,
            value_payload_local,
            value_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::Else);
        self.emit_throw_runtime_error(
            TYPE_ERROR_NAME,
            "AggregateError errors input must be iterable",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        self.release_temp_local(index_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(done_tag_local);
        self.release_temp_local(done_payload_local);
        self.release_temp_local(result_tag_local);
        self.release_temp_local(result_payload_local);
        self.release_temp_local(next_tag_local);
        self.release_temp_local(next_payload_local);
        self.release_temp_local(iterator_tag_local);
        self.release_temp_local(iterator_payload_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(key_local);
        Ok(())
    }
}
