//! Array.fromAsync owns one typed GC continuation across intrinsic Await jobs.
use super::super::*;
use crate::emit::ControlTarget;
use crate::gc_types::*;
use crate::heap::PromiseSettlement;
use crate::objects::PropertyKeyLocals;
use lila_ir::NativeErrorKind;

mod continuation;
mod state;
use state::{AwaitPhase, IteratorMode, SourcePlan, StateValue};

/// A continuation may observe only the protocol's two result properties.
#[derive(Clone, Copy)]
enum ArrayFromAsyncIteratorResultProperty {
    Done,
    Value,
}
impl ArrayFromAsyncIteratorResultProperty {
    fn key(self) -> &'static str {
        match self {
            Self::Done => "done",
            Self::Value => "value",
        }
    }
}

impl FunctionBuilder<'_> {
    pub(crate) fn emit_array_from_async(&mut self, f: &mut Function) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let constructor = self.emit_current_function_realm_intrinsic_promise_constructor(f);
        let capability =
            self.emit_new_current_function_realm_intrinsic_promise_capability(constructor, f)?;
        let promise = s.reserve_value_local(f);
        self.emit_read_promise_capability_promise(&capability, &promise, f);
        let pending = s.reserve_completion(f);
        pending.initialize(f);
        let ctor = s.reserve_value_local(f);
        let items = s.reserve_value_local(f);
        let mapper = s.reserve_value_local(f);
        let this_argument = s.reserve_value_local(f);
        let method = s.reserve_value_local(f);
        let source = s.reserve_value_local(f);
        let target = s.reserve_value_local(f);
        let iterator_value = s.reserve_value_local(f);
        let state_slot = s
            .reserve_gc_local::<ArrayFromAsyncState, Nullable>(f)
            .initialize_null(s, f);
        let length = s.reserve_i64_local(f);
        let mode = IteratorMode::new(s, f);
        f.instruction(&Instruction::I64Const(0));
        length.store(f);
        self.compile_this_to_locals(&ctor, f)?;
        self.emit_builtin_arg_to_value(0, &items, f);
        self.emit_builtin_arg_to_value(1, &mapper, f);
        self.emit_builtin_arg_to_value(2, &this_argument, f);
        let exit = self.open_frame(ControlFrameKind::Block, f);

        mapper.tag().load(f);
        f.instruction(&Instruction::I32Const(
            WasmRuntimeValueTag::Undefined as i32,
        ));
        f.instruction(&Instruction::I32Ne);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_is_callable_i32(&mapper, f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ARRAY_FROMASYNC_MAPPER_IS_NOT_CALLABLE,
            &pending,
            f,
        )?;
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);

        self.emit_af_get_method(
            &items,
            lila_ir::WellKnownSymbol::AsyncIterator,
            &method,
            &pending,
            f,
        )?;
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        self.compile_nullish_tagged_i32(method.tag(), f)?;
        self.open_frame(ControlFrameKind::If, f);
        self.emit_af_get_method(
            &items,
            lila_ir::WellKnownSymbol::Iterator,
            &method,
            &pending,
            f,
        )?;
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        mode.set_sync(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);

        self.compile_nullish_tagged_i32(method.tag(), f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        // GetIteratorFromMethod, including the cached next Get, precedes Construct.
        let argv = self.emit_pre_evaluated_arg_vector(&[], f);
        self.emit_function_or_proxy_call_with_argv(&method, &items, &argv, &pending, f)?;
        argv.clear(f);
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        iterator_value.copy_from(pending.value(), f);
        self.emit_is_heap_object_like_tag_i32(iterator_value.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ARRAY_FROMASYNC_ITERATOR_METHOD_MUST_RETURN_OBJECT,
            &pending,
            f,
        )?;
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.emit_array_native_get(&iterator_value, "next", &pending, f)?;
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        let iterator_stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<StoredValue>()
                .from_value(&iterator_value, f),
            f,
        );
        let next_stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<StoredValue>()
                .from_value(pending.value(), f),
            f,
        );
        let complete_record = s.reserve_gc_local(f).initialize(
            s.struct_type::<IteratorRecord>().construct(
                (
                    GcOperand::reference(&iterator_stored, s),
                    GcOperand::reference(&next_stored, s),
                    GcOperand::boolean(false),
                ),
                f,
            ),
            f,
        );
        next_stored.clear(f);
        iterator_stored.clear(f);
        self.emit_af_target(&ctor, None, &target, &capability, &pending, exit, f)?;
        let state = self.emit_af_publish_state(
            &capability,
            SourcePlan::Iterable {
                items: &items,
                record: &complete_record,
                mode: &mode,
            },
            &target,
            &mapper,
            &this_argument,
            f,
        )?;
        state_slot.replace(state.load(s, f).nullable(), f);
        state.clear(f);
        complete_record.clear(f);
        f.instruction(&Instruction::Else);
        self.emit_value_to_current_function_realm_object_locals(&items, &pending, f)?;
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        source.copy_from(pending.value(), f);
        self.emit_array_native_get(&source, "length", &pending, f)?;
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        self.emit_to_length_i64_from_value_locals(pending.value(), length, &pending, f)?;
        self.emit_af_reject_abrupt(&capability, &pending, exit, f)?;
        self.emit_af_target(&ctor, Some(length), &target, &capability, &pending, exit, f)?;
        let state = self.emit_af_publish_state(
            &capability,
            SourcePlan::ArrayLike {
                object: &source,
                length,
            },
            &target,
            &mapper,
            &this_argument,
            f,
        )?;
        state_slot.replace(state.load(s, f).nullable(), f);
        state.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);

        let state = s
            .reserve_gc_local(f)
            .initialize(state_slot.load(s, f).require_non_null(f), f);
        self.emit_af_drive(&state, exit, f)?;
        state.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        // Every post-capability abrupt completion rejects this intrinsic Promise.
        self.completion().set_normal(&promise, f);
        mode.clear(s, f);
        s.release_i64_local(length, f);
        state_slot.clear(f);
        iterator_value.clear(f);
        target.clear(f);
        source.clear(f);
        method.clear(f);
        this_argument.clear(f);
        mapper.clear(f);
        items.clear(f);
        ctor.clear(f);
        pending.clear(f);
        promise.clear(f);
        capability.clear(f);
        Ok(())
    }

    fn emit_af_result_get(
        &mut self,
        result: &ValueLocals,
        property: ArrayFromAsyncIteratorResultProperty,
        pending: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_array_native_get(result, property.key(), pending, f)
    }

    fn emit_af_get_method(
        &mut self,
        items: &ValueLocals,
        symbol: lila_ir::WellKnownSymbol,
        method: &ValueLocals,
        pending: &CompletionLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let symbol = s
            .reserve_gc_local(f)
            .initialize(self.emit_well_known_symbol_reference(symbol, f)?, f);
        let key = PropertyKeyLocals::from_symbol(s, &symbol, f);
        self.emit_object_read(items, items, &key, pending, f)?;
        key.clear(f);
        symbol.clear(f);
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Normal.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        method.copy_from(pending.value(), f);
        self.compile_nullish_tagged_i32(method.tag(), f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_is_callable_i32(method, f)?;
        f.instruction(&Instruction::I32Eqz);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ARRAY_FROMASYNC_ITERATOR_METHOD_IS_NOT_CALLABLE,
            pending,
            f,
        )?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_af_target(
        &mut self,
        ctor: &ValueLocals,
        length: Option<I64Local>,
        target: &ValueLocals,
        capability: &GcLocal<PromiseCapability>,
        pending: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let number = s.reserve_value_local(f);
        let zero = s.reserve_i64_local(f);
        f.instruction(&Instruction::I64Const(0));
        zero.store(f);
        self.emit_is_constructor_i32(ctor, f);
        self.open_frame(ControlFrameKind::If, f);
        if let Some(length) = length {
            self.emit_af_number(length, &number, f);
        }
        let length_arguments = [&number];
        let argv = self.emit_pre_evaluated_arg_vector(
            if length.is_some() {
                &length_arguments
            } else {
                &[]
            },
            f,
        );
        self.emit_function_or_proxy_construct_with_argv(ctor, ctor, &argv, pending, f)?;
        argv.clear(f);
        self.emit_af_reject_abrupt(capability, pending, exit, f)?;
        target.copy_from(pending.value(), f);
        f.instruction(&Instruction::Else);
        let count = length.unwrap_or(zero);
        // ArrayCreate's RangeError belongs to the async rejection boundary.
        count.load(f);
        f.instruction(&Instruction::I64Const(u32::MAX as i64));
        f.instruction(&Instruction::I64GtU);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_runtime_error(
            NativeErrorKind::RangeError,
            RuntimeErrorMessage::INVALID_ARRAY_LENGTH,
            pending,
            f,
        )?;
        self.emit_af_reject_abrupt(capability, pending, exit, f)?;
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        let realm = self.emit_execution_realm(f);
        let prototype = s
            .reserve_gc_local(f)
            .initialize(self.emit_load_realm_array_prototype(&realm, f), f);
        let prototype_value = s.reserve_value_local(f);
        prototype_value.set_reference(&prototype, s, f);
        let array = s.reserve_gc_local(f).initialize(
            self.emit_alloc_array_payload_with_length_and_prototype(count, &prototype_value, f)?,
            f,
        );
        target.set_reference(&array, s, f);
        array.clear(f);
        prototype_value.clear(f);
        prototype.clear(f);
        realm.clear(f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i64_local(zero, f);
        number.clear(f);
        Ok(())
    }

    fn emit_af_reject_abrupt(
        &mut self,
        capability: &GcLocal<PromiseCapability>,
        pending: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_af_settle(capability, PromiseSettlement::Reject, pending.value(), f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }
    fn emit_af_settle(
        &mut self,
        capability: &GcLocal<PromiseCapability>,
        kind: PromiseSettlement,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let called = s.reserve_completion(f);
        self.emit_call_promise_capability(capability, kind, value, &called, f)?;
        called.clear(f);
        Ok(())
    }
    fn emit_af_number(&self, integer: I64Local, value: &ValueLocals, f: &mut Function) {
        integer.load(f);
        f.instruction(&Instruction::F64ConvertI64U);
        f.instruction(&Instruction::I64ReinterpretF64);
        value.scalar().store(f);
        value.set_number(value.scalar(), f);
    }
}
