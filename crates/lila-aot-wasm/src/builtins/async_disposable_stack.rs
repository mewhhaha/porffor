//! AsyncDisposableStack stores complete values in a captured private resource List.
use super::super::*;
use crate::functions::{NonArrayRealmIntrinsicSlot, OrdinaryDefaultPrototype};
use crate::gc_types::*;
use crate::objects::PropertyKeyLocals;
use lila_ir::NativeErrorKind;

mod disposal;
mod entries;

#[derive(Clone, Copy)]
pub(crate) enum AsyncDisposableStackDisposeCompletionKind {
    Normal,
    Throw,
}
impl AsyncDisposableStackDisposeCompletionKind {
    pub(crate) const fn wire_code(self) -> i32 {
        match self {
            Self::Normal => 0,
            Self::Throw => 1,
        }
    }
}

enum AdsReceiverPolicy {
    BrandOnly,
    Pending,
}

impl FunctionBuilder<'_> {
    fn emit_ads_abrupt_exit(
        &mut self,
        pending: &CompletionLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) {
        pending.kind().load(f);
        f.instruction(&Instruction::I32Const(CompletionKind::Throw.code() as i32));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        output.copy_from(pending, f);
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
    }

    fn emit_ads_type_error_if(
        &mut self,
        message: RuntimeErrorMessage,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_type_error(message, output, f)?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        Ok(())
    }

    fn emit_ads_brand(
        &mut self,
        receiver: &ValueLocals,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<GcLocal<AsyncDisposableStackObject>, EmitError> {
        let s = self.runtime_schema();
        self.emit_is_heap_object_like_tag_i32(receiver.tag(), f);
        f.instruction(&Instruction::I32Eqz);
        self.emit_ads_type_error_if(
            RuntimeErrorMessage::ASYNCDISPOSABLESTACK_METHOD_RECEIVER_IS_NOT_AN_OBJECT,
            output,
            exit,
            f,
        )?;
        receiver.reference().load(f);
        f.instruction(&Instruction::RefTestNonNull(
            s.reference_type::<AsyncDisposableStackObject>(GcNullability::NonNullable)
                .heap_type,
        ));
        f.instruction(&Instruction::I32Eqz);
        self.emit_ads_type_error_if(RuntimeErrorMessage::ASYNCDISPOSABLESTACK_METHOD_RECEIVER_DOES_NOT_HAVE_ASYNCDISPOSABLESTATE, output, exit, f)?;
        Ok(s.reserve_gc_local(f).initialize(
            receiver.cast_reference::<AsyncDisposableStackObject>(s, f),
            f,
        ))
    }

    fn emit_ads_require_pending(
        &mut self,
        stack: &GcLocal<AsyncDisposableStackObject>,
        output: &CompletionLocals,
        exit: ControlTarget,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let state = s.reserve_i32_local(f);
        s.struct_type::<AsyncDisposableStackObject>()
            .field(AsyncDisposableStackObjectSchema::STATE)
            .read(stack, s, f)
            .store(state, f);
        state.load(f);
        f.instruction(&Instruction::I32Const(
            AsyncDisposableStackState::Disposed.word() as i32,
        ));
        f.instruction(&Instruction::I32Eq);
        self.open_frame(ControlFrameKind::If, f);
        self.emit_throw_current_function_realm_error(
            NativeErrorKind::ReferenceError,
            RuntimeErrorMessage::ASYNCDISPOSABLESTACK_IS_ALREADY_DISPOSED,
            output,
            f,
        )?;
        self.emit_branch_to_target(exit, f);
        self.pop_control(ControlFrameKind::If);
        f.instruction(&Instruction::End);
        s.release_i32_local(state, f);
        Ok(())
    }

    fn emit_with_ads(
        &mut self,
        policy: AdsReceiverPolicy,
        f: &mut Function,
        consume: impl FnOnce(
            &mut Self,
            &GcLocal<AsyncDisposableStackObject>,
            &CompletionLocals,
            ControlTarget,
            &mut Function,
        ) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let receiver = s.reserve_value_local(f);
        let output = s.reserve_completion(f);
        output.initialize(f);
        self.compile_this_to_locals(&receiver, f)?;
        let exit = self.open_frame(ControlFrameKind::Block, f);
        let stack = self.emit_ads_brand(&receiver, &output, exit, f)?;
        match policy {
            AdsReceiverPolicy::BrandOnly => {}
            AdsReceiverPolicy::Pending => {
                self.emit_ads_require_pending(&stack, &output, exit, f)?
            }
        }
        consume(self, &stack, &output, exit, f)?;
        stack.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        receiver.clear(f);
        Ok(())
    }

    fn emit_empty_async_disposable_resource_stack(
        &mut self,
        f: &mut Function,
    ) -> GcLocal<AsyncDisposableResourceStack> {
        let s = self.runtime_schema();
        let entries = s.reserve_gc_local(f).initialize(
            s.array_type::<AsyncDisposableResourceTable>().fixed([], f),
            f,
        );
        let list = s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableResourceStack>()
                .construct((GcOperand::reference(&entries, s), GcOperand::i64(0)), f),
            f,
        );
        entries.clear(f);
        list
    }

    fn emit_ads_instance(
        &mut self,
        header: &GcLocal<OrdinaryObject>,
        resources: &GcLocal<AsyncDisposableResourceStack>,
        f: &mut Function,
    ) -> GcLocal<AsyncDisposableStackObject> {
        let s = self.runtime_schema();
        s.reserve_gc_local(f).initialize(
            s.struct_type::<AsyncDisposableStackObject>().construct(
                (
                    GcOperand::reference(header, s),
                    GcOperand::constant(AsyncDisposableStackState::Pending),
                    GcOperand::reference(resources, s),
                ),
                f,
            ),
            f,
        )
    }

    pub(crate) fn emit_async_disposable_stack_constructor(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let new_target = s.reserve_value_local(f);
        let value = s.reserve_value_local(f);
        let output = s.reserve_completion(f);
        self.compile_new_target_to_locals(&new_target, f)?;
        let exit = self.open_frame(ControlFrameKind::Block, f);
        new_target.tag().load(f);
        f.instruction(&Instruction::I32Const(WasmRuntimeValueTag::Undefined.tag()));
        f.instruction(&Instruction::I32Eq);
        self.emit_ads_type_error_if(
            RuntimeErrorMessage::ASYNCDISPOSABLESTACK_CONSTRUCTOR_REQUIRES_NEW,
            &output,
            exit,
            f,
        )?;
        self.emit_get_prototype_from_constructor(
            &new_target,
            OrdinaryDefaultPrototype::AsyncDisposableStack,
            &output,
            f,
        )?;
        self.emit_ads_abrupt_exit(&output, &output, exit, f);
        let header = s.reserve_gc_local(f).initialize(
            self.emit_alloc_plain_object_with_prototype(Some(output.value()), f)?,
            f,
        );
        let resources = self.emit_empty_async_disposable_resource_stack(f);
        let stack = self.emit_ads_instance(&header, &resources, f);
        value.set_reference(&stack, s, f);
        output.set_normal(&value, f);
        stack.clear(f);
        resources.clear(f);
        header.clear(f);
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        output.clear(f);
        value.clear(f);
        new_target.clear(f);
        Ok(())
    }

    pub(crate) fn emit_async_disposable_stack_move(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_with_ads(AdsReceiverPolicy::Pending, f, |b, source, output, _, f| {
            let s = b.runtime_schema();
            let header = s.reserve_gc_local(f).initialize(
                b.emit_alloc_current_function_realm_async_disposable_stack_object(f)?,
                f,
            );
            let transferred = s.reserve_gc_local(f).initialize(
                s.struct_type::<AsyncDisposableStackObject>()
                    .field(AsyncDisposableStackObjectSchema::RESOURCES)
                    .read(source, s, f)
                    .reference(),
                f,
            );
            let moved = b.emit_ads_instance(&header, &transferred, f);
            let empty = b.emit_empty_async_disposable_resource_stack(f);
            s.struct_type::<AsyncDisposableStackObject>()
                .field(AsyncDisposableStackObjectSchema::RESOURCES)
                .write(source, GcOperand::reference(&empty, s), s, f);
            s.struct_type::<AsyncDisposableStackObject>()
                .field(AsyncDisposableStackObjectSchema::STATE)
                .write(
                    source,
                    GcOperand::constant(AsyncDisposableStackState::Disposed),
                    s,
                    f,
                );
            let value = s.reserve_value_local(f);
            value.set_reference(&moved, s, f);
            output.set_normal(&value, f);
            value.clear(f);
            empty.clear(f);
            moved.clear(f);
            transferred.clear(f);
            header.clear(f);
            Ok(())
        })
    }

    pub(crate) fn emit_async_disposable_stack_disposed_getter(
        &mut self,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_with_ads(AdsReceiverPolicy::BrandOnly, f, |b, stack, output, _, f| {
            let s = b.runtime_schema();
            let state = s.reserve_i32_local(f);
            let truth = s.reserve_i32_local(f);
            s.struct_type::<AsyncDisposableStackObject>()
                .field(AsyncDisposableStackObjectSchema::STATE)
                .read(stack, s, f)
                .store(state, f);
            state.load(f);
            f.instruction(&Instruction::I32Const(
                AsyncDisposableStackState::Disposed.word() as i32,
            ));
            f.instruction(&Instruction::I32Eq);
            truth.store(f);
            output.value().set_boolean(truth, f);
            output.set_normal(output.value(), f);
            s.release_i32_local(truth, f);
            s.release_i32_local(state, f);
            Ok(())
        })
    }
}

enum AdsClosure<'a> {
    SyncDispose(&'a GcLocal<AsyncDisposableStackSyncDisposeContext>),
    Disposal(&'a GcLocal<AsyncDisposableStackDisposal>, PromiseSettlement),
}
impl FunctionBuilder<'_> {
    fn emit_ads_closure(
        &mut self,
        closure: AdsClosure<'_>,
        realm: &GcLocal<RealmRecord>,
        output: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let (id, payload) = match closure {
            AdsClosure::SyncDispose(context) => (
                HostBuiltinId::AsyncDisposableStackSyncDispose.function_id(),
                BuiltinClosurePayload::AsyncDisposableStackSyncDispose(context),
            ),
            AdsClosure::Disposal(state, kind) => {
                let builtin = match kind {
                    PromiseSettlement::Fulfill => {
                        StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled
                    }
                    PromiseSettlement::Reject => {
                        StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected
                    }
                };
                (
                    builtin.function_id(),
                    BuiltinClosurePayload::AsyncDisposableStackDisposal(state),
                )
            }
        };
        let metadata = self.functions.get(&id).cloned().ok_or_else(|| {
            EmitError::unsupported("missing AsyncDisposableStack captured callback")
        })?;
        let capture = s
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(f)
            .initialize(
                s.struct_type::<BuiltinClosureCapture>()
                    .publish(payload, s, f)
                    .nullable(),
                f,
            );
        let context = self.emit_realm_function_materialization_context_from_realm(realm, f);
        let callable = s.reserve_gc_local(f).initialize(
            self.emit_function_value_payload_in_realm_with_capture(
                &metadata, &context, &capture, f,
            )?,
            f,
        );
        output.set_reference(&callable, s, f);
        callable.clear(f);
        self.release_realm_function_materialization_context(context, f);
        capture.clear(f);
        Ok(())
    }
    fn emit_ads_capture(
        &self,
        expected: BuiltinClosureCaptureKind,
        f: &mut Function,
    ) -> GcLocal<BuiltinClosureCapture> {
        let s = self.runtime_schema();
        let context = self
            .body_entry_locals()
            .expect("native AsyncDisposableStack callback entry")
            .function_context()
            .expect("native callback context");
        let capture = s.reserve_gc_local(f).initialize(
            s.struct_type::<FunctionContext>()
                .field(FunctionContextSchema::BUILTIN_CAPTURE)
                .read(context, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let kind = GcI32DomainLocal::new(s, expected, f);
        s.struct_type::<BuiltinClosureCapture>()
            .field(BuiltinClosureCaptureSchema::KIND)
            .read(&capture, s, f)
            .store_domain(&kind, f);
        kind.load(f);
        f.instruction(&Instruction::I32Const(GcI32Constant::encode(expected)));
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::Unreachable);
        f.instruction(&Instruction::End);
        kind.clear(s, f);
        capture
    }
}
