use super::*;

/// Only these actual Await transitions can be installed by this algorithm.
#[derive(Clone, Copy)]
pub(super) enum AwaitPhase {
    Input,
    Mapper,
    Iterator,
    Close,
}
impl AwaitPhase {
    pub(super) fn stage(self) -> ArrayFromAsyncStage {
        match self {
            Self::Input => ArrayFromAsyncStage::InputValue,
            Self::Mapper => ArrayFromAsyncStage::MappedValue,
            Self::Iterator => ArrayFromAsyncStage::AsyncIteratorResult,
            Self::Close => ArrayFromAsyncStage::AsyncCloseResult,
        }
    }
}
/// Iterable modes cannot be initialized or reassigned to ArrayLike.
#[must_use = "the iterable mode belongs to a prepared source"]
pub(super) struct IteratorMode(GcI32DomainLocal<ArrayFromAsyncSourceMode>);
impl IteratorMode {
    pub(super) fn new(schema: &RuntimeSchema, f: &mut Function) -> Self {
        Self(GcI32DomainLocal::new(
            schema,
            ArrayFromAsyncSourceMode::AsyncIterator,
            f,
        ))
    }
    pub(super) fn set_sync(&self, f: &mut Function) {
        self.0
            .set_constant(ArrayFromAsyncSourceMode::SyncIterator, f);
    }
    pub(super) fn clear(self, schema: &RuntimeSchema, f: &mut Function) {
        self.0.clear(schema, f);
    }
}
/// The sole state constructor derives mode, record presence and length from
/// this plan. An iterable cannot publish a nullable record or array-like mode.
pub(super) enum SourcePlan<'a> {
    ArrayLike {
        object: &'a ValueLocals,
        length: I64Local,
    },
    Iterable {
        items: &'a ValueLocals,
        record: &'a GcLocal<IteratorRecord>,
        mode: &'a IteratorMode,
    },
}
#[derive(Clone, Copy)]
pub(super) enum StateValue {
    Source,
    Target,
    Mapper,
    ThisArgument,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_af_publish_state(
        &mut self,
        capability: &GcLocal<PromiseCapability>,
        source: SourcePlan<'_>,
        target: &ValueLocals,
        mapper: &ValueLocals,
        this_argument: &ValueLocals,
        f: &mut Function,
    ) -> Result<GcLocal<ArrayFromAsyncState>, EmitError> {
        let s = self.runtime_schema();
        let (source, length, mode, iterator) = match source {
            SourcePlan::ArrayLike { object, length } => (
                object,
                GcOperand::i64_local(length),
                GcOperand::constant(ArrayFromAsyncSourceMode::ArrayLike),
                GcOperand::null(s),
            ),
            SourcePlan::Iterable {
                items,
                record,
                mode,
            } => (
                items,
                GcOperand::i64(0),
                mode.0.operand(),
                GcOperand::nullable_reference(record, s),
            ),
        };
        let constructor = self.emit_current_function_realm_intrinsic_promise_constructor(f);
        let throwaway =
            self.emit_new_current_function_realm_intrinsic_promise_capability(constructor, f)?;
        let source_stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(source, f), f);
        let target_stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(target, f), f);
        let mapper_stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(mapper, f), f);
        let this_stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<StoredValue>().from_value(this_argument, f),
            f,
        );
        let undefined = s.reserve_value_local(f);
        undefined.set_undefined(f);
        let error_stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(&undefined, f), f);
        let state = s.reserve_gc_local(f).initialize(
            s.struct_type::<ArrayFromAsyncState>().construct(
                (
                    GcOperand::reference(capability, s),
                    GcOperand::nullable_reference(&throwaway, s),
                    GcOperand::reference(&source_stored, s),
                    GcOperand::reference(&target_stored, s),
                    GcOperand::reference(&mapper_stored, s),
                    GcOperand::reference(&this_stored, s),
                    GcOperand::i64(0),
                    length,
                    GcOperand::null(s),
                    GcOperand::null(s),
                    GcOperand::constant(ArrayFromAsyncStage::InputValue),
                    iterator,
                    mode,
                    GcOperand::reference(&error_stored, s),
                ),
                f,
            ),
            f,
        );
        // Neither callback is exposed to user code while the pair is completed.
        let capture = s.reserve_gc_local(f).initialize(
            s.struct_type::<BuiltinClosureCapture>().publish(
                BuiltinClosurePayload::ArrayFromAsync(&state),
                s,
                f,
            ),
            f,
        );
        let nullable_capture = s
            .reserve_gc_local::<BuiltinClosureCapture, Nullable>(f)
            .initialize(capture.load(s, f).nullable(), f);
        let realm = self.emit_execution_realm(f);
        let context = self.emit_realm_function_materialization_context_from_realm(&realm, f);
        for (builtin, field) in [
            (
                StandardBuiltinId::ArrayFromAsyncFulfilled,
                ArrayFromAsyncStateSchema::FULFILLED_CALLBACK,
            ),
            (
                StandardBuiltinId::ArrayFromAsyncRejected,
                ArrayFromAsyncStateSchema::REJECTED_CALLBACK,
            ),
        ] {
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported("missing Array.fromAsync continuation entry")
                })?;
            let callback = s.reserve_gc_local(f).initialize(
                self.emit_function_value_payload_in_realm_with_capture(
                    &meta,
                    &context,
                    &nullable_capture,
                    f,
                )?,
                f,
            );
            s.struct_type::<ArrayFromAsyncState>().field(field).write(
                &state,
                GcOperand::nullable_reference(&callback, s),
                s,
                f,
            );
            callback.clear(f);
        }
        self.release_realm_function_materialization_context(context, f);
        realm.clear(f);
        nullable_capture.clear(f);
        capture.clear(f);
        throwaway.clear(f);
        error_stored.clear(f);
        undefined.clear(f);
        this_stored.clear(f);
        mapper_stored.clear(f);
        target_stored.clear(f);
        source_stored.clear(f);
        Ok(state)
    }

    pub(super) fn emit_af_captured_state(&self, f: &mut Function) -> GcLocal<ArrayFromAsyncState> {
        let s = self.runtime_schema();
        let context = self
            .body_entry_locals()
            .expect("native continuation entry")
            .function_context()
            .expect("native continuation context");
        let capture = s.reserve_gc_local(f).initialize(
            s.struct_type::<FunctionContext>()
                .field(FunctionContextSchema::BUILTIN_CAPTURE)
                .read(context, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let kind = GcI32DomainLocal::new(s, BuiltinClosureCaptureKind::ArrayFromAsync, f);
        s.struct_type::<BuiltinClosureCapture>()
            .field(BuiltinClosureCaptureSchema::KIND)
            .read(&capture, s, f)
            .store_domain(&kind, f);
        kind.load(f);
        f.instruction(&Instruction::I32Const(GcI32Constant::encode(
            BuiltinClosureCaptureKind::ArrayFromAsync,
        )));
        f.instruction(&Instruction::I32Ne);
        f.instruction(&Instruction::If(BlockType::Empty));
        f.instruction(&Instruction::Unreachable);
        f.instruction(&Instruction::End);
        let state = s.reserve_gc_local(f).initialize(
            s.struct_type::<BuiltinClosureCapture>()
                .field(BuiltinClosureCaptureSchema::ARRAY_FROM_ASYNC)
                .read(&capture, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        kind.clear(s, f);
        capture.clear(f);
        state
    }
    pub(super) fn emit_af_value(
        &self,
        state: &GcLocal<ArrayFromAsyncState>,
        value: StateValue,
        output: &ValueLocals,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let field = match value {
            StateValue::Source => ArrayFromAsyncStateSchema::SOURCE,
            StateValue::Target => ArrayFromAsyncStateSchema::TARGET,
            StateValue::Mapper => ArrayFromAsyncStateSchema::MAPPER,
            StateValue::ThisArgument => ArrayFromAsyncStateSchema::THIS_ARGUMENT,
        };
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<ArrayFromAsyncState>()
                .field(field)
                .read(state, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&stored, output, s, f);
        stored.clear(f);
    }
    pub(super) fn emit_af_capability(
        &self,
        state: &GcLocal<ArrayFromAsyncState>,
        f: &mut Function,
    ) -> GcLocal<PromiseCapability> {
        let s = self.runtime_schema();
        s.reserve_gc_local(f).initialize(
            s.struct_type::<ArrayFromAsyncState>()
                .field(ArrayFromAsyncStateSchema::CAPABILITY)
                .read(state, s, f)
                .reference(),
            f,
        )
    }
    pub(super) fn emit_af_record(
        &self,
        state: &GcLocal<ArrayFromAsyncState>,
        f: &mut Function,
    ) -> GcLocal<IteratorRecord> {
        let s = self.runtime_schema();
        s.reserve_gc_local(f).initialize(
            s.struct_type::<ArrayFromAsyncState>()
                .field(ArrayFromAsyncStateSchema::ITERATOR)
                .read(state, s, f)
                .reference()
                .require_non_null(f),
            f,
        )
    }
    pub(super) fn emit_af_mode(
        &self,
        state: &GcLocal<ArrayFromAsyncState>,
        f: &mut Function,
    ) -> GcI32DomainLocal<ArrayFromAsyncSourceMode> {
        let s = self.runtime_schema();
        let mode = GcI32DomainLocal::new(s, ArrayFromAsyncSourceMode::ArrayLike, f);
        s.struct_type::<ArrayFromAsyncState>()
            .field(ArrayFromAsyncStateSchema::MODE)
            .read(state, s, f)
            .store_domain(&mode, f);
        mode
    }
    pub(super) fn emit_af_index(
        &self,
        state: &GcLocal<ArrayFromAsyncState>,
        f: &mut Function,
    ) -> I64Local {
        let s = self.runtime_schema();
        let index = s.reserve_i64_local(f);
        s.struct_type::<ArrayFromAsyncState>()
            .field(ArrayFromAsyncStateSchema::INDEX)
            .read(state, s, f)
            .store_i64(index, f);
        index
    }
    pub(super) fn emit_af_saved_error(
        &self,
        state: &GcLocal<ArrayFromAsyncState>,
        output: &ValueLocals,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let stored = s.reserve_gc_local(f).initialize(
            s.struct_type::<ArrayFromAsyncState>()
                .field(ArrayFromAsyncStateSchema::SAVED_ERROR)
                .read(state, s, f)
                .reference(),
            f,
        );
        s.struct_type::<StoredValue>()
            .read_into(&stored, output, s, f);
        stored.clear(f);
    }
    pub(super) fn emit_af_save_error(
        &self,
        state: &GcLocal<ArrayFromAsyncState>,
        error: &ValueLocals,
        f: &mut Function,
    ) {
        let s = self.runtime_schema();
        let stored = s
            .reserve_gc_local(f)
            .initialize(s.struct_type::<StoredValue>().from_value(error, f), f);
        s.struct_type::<ArrayFromAsyncState>()
            .field(ArrayFromAsyncStateSchema::SAVED_ERROR)
            .write(state, GcOperand::reference(&stored, s), s, f);
        stored.clear(f);
    }
    pub(super) fn emit_af_await(
        &mut self,
        state: &GcLocal<ArrayFromAsyncState>,
        phase: AwaitPhase,
        value: &ValueLocals,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let s = self.runtime_schema();
        let row = s.struct_type::<ArrayFromAsyncState>();
        row.field(ArrayFromAsyncStateSchema::STAGE).write(
            state,
            GcOperand::constant(phase.stage()),
            s,
            f,
        );
        let capability = s.reserve_gc_local(f).initialize(
            row.field(ArrayFromAsyncStateSchema::THROWAWAY_CAPABILITY)
                .read(state, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let fulfilled_callback = s.reserve_gc_local(f).initialize(
            row.field(ArrayFromAsyncStateSchema::FULFILLED_CALLBACK)
                .read(state, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let rejected_callback = s.reserve_gc_local(f).initialize(
            row.field(ArrayFromAsyncStateSchema::REJECTED_CALLBACK)
                .read(state, s, f)
                .reference()
                .require_non_null(f),
            f,
        );
        let fulfilled = s.reserve_value_local(f);
        fulfilled.set_reference(&fulfilled_callback, s, f);
        let rejected = s.reserve_value_local(f);
        rejected.set_reference(&rejected_callback, s, f);
        self.emit_intrinsic_await_with_handlers(value, &fulfilled, &rejected, &capability, f)?;
        rejected.clear(f);
        fulfilled.clear(f);
        rejected_callback.clear(f);
        fulfilled_callback.clear(f);
        capability.clear(f);
        Ok(())
    }
}
