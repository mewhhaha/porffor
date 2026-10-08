//! Actual native helper records share cached iterator protocol and whole completion.
use super::*;
use crate::control_flow::OwnedSyncIterator;
use crate::functions::NonArrayRealmIntrinsicSlot;
use crate::objects::PropertyKeyLocals;

mod concat;
mod lazy;
mod protocol;
mod terminal;
mod zip;
pub(super) use zip::IteratorZipInput;

#[derive(Clone, Copy)]
pub(super) enum IteratorHelperOperation {
    Next,
    Return,
}
#[derive(Clone, Copy)]
pub(super) enum LazyIteratorKind {
    Map,
    Filter,
    FlatMap,
    Take,
    Drop,
}
#[derive(Clone, Copy)]
pub(super) enum IteratorTerminalKind {
    ToArray,
    ForEach,
    Every,
    Some,
    Find,
    Reduce,
}

#[derive(Clone, Copy)]
enum HelperKind {
    Map,
    Filter,
    FlatMap,
    Take,
    Drop,
    Concat,
    Zip,
}
#[derive(Clone, Copy)]
enum HelperFlag {
    Done,
    Executing,
    Started,
}
enum HelperState {
    Map(GcLocal<IteratorMapHelper>),
    Filter(GcLocal<IteratorFilterHelper>),
    FlatMap(GcLocal<IteratorFlatMapHelper>),
    Take(GcLocal<IteratorTakeHelper>),
    Drop(GcLocal<IteratorDropHelper>),
    Concat(GcLocal<IteratorConcatHelper>),
    Zip(GcLocal<IteratorZipHelper>),
}
impl HelperKind {
    const ALL: &'static [Self] = &[
        Self::Map,
        Self::Filter,
        Self::FlatMap,
        Self::Take,
        Self::Drop,
        Self::Concat,
        Self::Zip,
    ];
    fn creator(self) -> StandardBuiltinId {
        match self {
            Self::Map => StandardBuiltinId::IteratorPrototypeMap,
            Self::Filter => StandardBuiltinId::IteratorPrototypeFilter,
            Self::FlatMap => StandardBuiltinId::IteratorPrototypeFlatMap,
            Self::Take => StandardBuiltinId::IteratorPrototypeTake,
            Self::Drop => StandardBuiltinId::IteratorPrototypeDrop,
            Self::Concat => StandardBuiltinId::IteratorConcat,
            Self::Zip => StandardBuiltinId::IteratorZip,
        }
    }
    fn entry(self, operation: IteratorHelperOperation) -> StandardBuiltinId {
        match (self, operation) {
            (Self::Map, IteratorHelperOperation::Next) => StandardBuiltinId::IteratorMapNext,
            (Self::Map, IteratorHelperOperation::Return) => StandardBuiltinId::IteratorMapReturn,
            (Self::Filter, IteratorHelperOperation::Next) => StandardBuiltinId::IteratorFilterNext,
            (Self::Filter, IteratorHelperOperation::Return) => {
                StandardBuiltinId::IteratorFilterReturn
            }
            (Self::FlatMap, IteratorHelperOperation::Next) => {
                StandardBuiltinId::IteratorFlatMapNext
            }
            (Self::FlatMap, IteratorHelperOperation::Return) => {
                StandardBuiltinId::IteratorFlatMapReturn
            }
            (Self::Take, IteratorHelperOperation::Next) => StandardBuiltinId::IteratorTakeNext,
            (Self::Take, IteratorHelperOperation::Return) => StandardBuiltinId::IteratorTakeReturn,
            (Self::Drop, IteratorHelperOperation::Next) => StandardBuiltinId::IteratorDropNext,
            (Self::Drop, IteratorHelperOperation::Return) => StandardBuiltinId::IteratorDropReturn,
            (Self::Concat, IteratorHelperOperation::Next) => StandardBuiltinId::IteratorConcatNext,
            (Self::Concat, IteratorHelperOperation::Return) => {
                StandardBuiltinId::IteratorConcatReturn
            }
            (Self::Zip, IteratorHelperOperation::Next) => StandardBuiltinId::IteratorZipNext,
            (Self::Zip, IteratorHelperOperation::Return) => StandardBuiltinId::IteratorZipReturn,
        }
    }
    fn running_message(self) -> RuntimeErrorMessage {
        match self {
            Self::Map => RuntimeErrorMessage::ITERATOR_MAP_HELPER_IS_ALREADY_RUNNING,
            Self::Filter => RuntimeErrorMessage::ITERATOR_FILTER_HELPER_IS_ALREADY_RUNNING,
            Self::FlatMap => RuntimeErrorMessage::ITERATOR_FLATMAP_HELPER_IS_ALREADY_RUNNING,
            Self::Take => RuntimeErrorMessage::ITERATOR_TAKE_HELPER_IS_ALREADY_RUNNING,
            Self::Drop => RuntimeErrorMessage::ITERATOR_DROP_HELPER_IS_ALREADY_RUNNING,
            Self::Concat => RuntimeErrorMessage::ITERATOR_CONCAT_HELPER_IS_ALREADY_RUNNING,
            Self::Zip => RuntimeErrorMessage::ITERATOR_ZIP_HELPER_IS_ALREADY_RUNNING,
        }
    }
    fn test(self, value: &ValueLocals, schema: &RuntimeSchema, f: &mut Function) {
        value.reference().load(f);
        let heap = match self {
            Self::Map => {
                schema
                    .reference_type::<IteratorMapHelper>(GcNullability::NonNullable)
                    .heap_type
            }
            Self::Filter => {
                schema
                    .reference_type::<IteratorFilterHelper>(GcNullability::NonNullable)
                    .heap_type
            }
            Self::FlatMap => {
                schema
                    .reference_type::<IteratorFlatMapHelper>(GcNullability::NonNullable)
                    .heap_type
            }
            Self::Take => {
                schema
                    .reference_type::<IteratorTakeHelper>(GcNullability::NonNullable)
                    .heap_type
            }
            Self::Drop => {
                schema
                    .reference_type::<IteratorDropHelper>(GcNullability::NonNullable)
                    .heap_type
            }
            Self::Concat => {
                schema
                    .reference_type::<IteratorConcatHelper>(GcNullability::NonNullable)
                    .heap_type
            }
            Self::Zip => {
                schema
                    .reference_type::<IteratorZipHelper>(GcNullability::NonNullable)
                    .heap_type
            }
        };
        f.instruction(&Instruction::RefTestNonNull(heap));
    }
    fn cast(self, value: &ValueLocals, schema: &RuntimeSchema, f: &mut Function) -> HelperState {
        match self {
            Self::Map => HelperState::Map(
                schema
                    .reserve_gc_local(f)
                    .initialize(value.cast_reference::<IteratorMapHelper>(schema, f), f),
            ),
            Self::Filter => HelperState::Filter(
                schema
                    .reserve_gc_local(f)
                    .initialize(value.cast_reference::<IteratorFilterHelper>(schema, f), f),
            ),
            Self::FlatMap => HelperState::FlatMap(
                schema
                    .reserve_gc_local(f)
                    .initialize(value.cast_reference::<IteratorFlatMapHelper>(schema, f), f),
            ),
            Self::Take => HelperState::Take(
                schema
                    .reserve_gc_local(f)
                    .initialize(value.cast_reference::<IteratorTakeHelper>(schema, f), f),
            ),
            Self::Drop => HelperState::Drop(
                schema
                    .reserve_gc_local(f)
                    .initialize(value.cast_reference::<IteratorDropHelper>(schema, f), f),
            ),
            Self::Concat => HelperState::Concat(
                schema
                    .reserve_gc_local(f)
                    .initialize(value.cast_reference::<IteratorConcatHelper>(schema, f), f),
            ),
            Self::Zip => HelperState::Zip(
                schema
                    .reserve_gc_local(f)
                    .initialize(value.cast_reference::<IteratorZipHelper>(schema, f), f),
            ),
        }
    }
}
impl HelperState {
    fn read_flag(
        &self,
        flag: HelperFlag,
        into: I32Local,
        schema: &RuntimeSchema,
        f: &mut Function,
    ) {
        match self {
            Self::Map(state) => match flag {
                HelperFlag::Done => schema
                    .field(IteratorMapHelperSchema::DONE)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Executing => schema
                    .field(IteratorMapHelperSchema::EXECUTING)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Started => schema
                    .field(IteratorMapHelperSchema::STARTED)
                    .read(state, schema, f)
                    .store(into, f),
            },
            Self::Filter(state) => match flag {
                HelperFlag::Done => schema
                    .field(IteratorFilterHelperSchema::DONE)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Executing => schema
                    .field(IteratorFilterHelperSchema::EXECUTING)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Started => schema
                    .field(IteratorFilterHelperSchema::STARTED)
                    .read(state, schema, f)
                    .store(into, f),
            },
            Self::FlatMap(state) => match flag {
                HelperFlag::Done => schema
                    .field(IteratorFlatMapHelperSchema::DONE)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Executing => schema
                    .field(IteratorFlatMapHelperSchema::EXECUTING)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Started => schema
                    .field(IteratorFlatMapHelperSchema::STARTED)
                    .read(state, schema, f)
                    .store(into, f),
            },
            Self::Take(state) => match flag {
                HelperFlag::Done => schema
                    .field(IteratorTakeHelperSchema::DONE)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Executing => schema
                    .field(IteratorTakeHelperSchema::EXECUTING)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Started => schema
                    .field(IteratorTakeHelperSchema::STARTED)
                    .read(state, schema, f)
                    .store(into, f),
            },
            Self::Drop(state) => match flag {
                HelperFlag::Done => schema
                    .field(IteratorDropHelperSchema::DONE)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Executing => schema
                    .field(IteratorDropHelperSchema::EXECUTING)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Started => schema
                    .field(IteratorDropHelperSchema::STARTED)
                    .read(state, schema, f)
                    .store(into, f),
            },
            Self::Concat(state) => match flag {
                HelperFlag::Done => schema
                    .field(IteratorConcatHelperSchema::DONE)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Executing => schema
                    .field(IteratorConcatHelperSchema::EXECUTING)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Started => schema
                    .field(IteratorConcatHelperSchema::STARTED)
                    .read(state, schema, f)
                    .store(into, f),
            },
            Self::Zip(state) => match flag {
                HelperFlag::Done => schema
                    .field(IteratorZipHelperSchema::DONE)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Executing => schema
                    .field(IteratorZipHelperSchema::EXECUTING)
                    .read(state, schema, f)
                    .store(into, f),
                HelperFlag::Started => schema
                    .field(IteratorZipHelperSchema::STARTED)
                    .read(state, schema, f)
                    .store(into, f),
            },
        }
    }
    fn write_flag(&self, flag: HelperFlag, value: bool, schema: &RuntimeSchema, f: &mut Function) {
        match self {
            Self::Map(state) => {
                match flag {
                    HelperFlag::Done => schema.field(IteratorMapHelperSchema::DONE).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                    HelperFlag::Executing => schema
                        .field(IteratorMapHelperSchema::EXECUTING)
                        .write(state, GcOperand::boolean(value), schema, f),
                    HelperFlag::Started => schema.field(IteratorMapHelperSchema::STARTED).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                }
            }
            Self::Filter(state) => {
                match flag {
                    HelperFlag::Done => schema.field(IteratorFilterHelperSchema::DONE).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                    HelperFlag::Executing => schema
                        .field(IteratorFilterHelperSchema::EXECUTING)
                        .write(state, GcOperand::boolean(value), schema, f),
                    HelperFlag::Started => schema.field(IteratorFilterHelperSchema::STARTED).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                }
            }
            Self::FlatMap(state) => {
                match flag {
                    HelperFlag::Done => schema.field(IteratorFlatMapHelperSchema::DONE).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                    HelperFlag::Executing => schema
                        .field(IteratorFlatMapHelperSchema::EXECUTING)
                        .write(state, GcOperand::boolean(value), schema, f),
                    HelperFlag::Started => schema
                        .field(IteratorFlatMapHelperSchema::STARTED)
                        .write(state, GcOperand::boolean(value), schema, f),
                }
            }
            Self::Take(state) => {
                match flag {
                    HelperFlag::Done => schema.field(IteratorTakeHelperSchema::DONE).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                    HelperFlag::Executing => schema
                        .field(IteratorTakeHelperSchema::EXECUTING)
                        .write(state, GcOperand::boolean(value), schema, f),
                    HelperFlag::Started => schema.field(IteratorTakeHelperSchema::STARTED).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                }
            }
            Self::Drop(state) => {
                match flag {
                    HelperFlag::Done => schema.field(IteratorDropHelperSchema::DONE).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                    HelperFlag::Executing => schema
                        .field(IteratorDropHelperSchema::EXECUTING)
                        .write(state, GcOperand::boolean(value), schema, f),
                    HelperFlag::Started => schema.field(IteratorDropHelperSchema::STARTED).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                }
            }
            Self::Concat(state) => {
                match flag {
                    HelperFlag::Done => schema.field(IteratorConcatHelperSchema::DONE).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                    HelperFlag::Executing => schema
                        .field(IteratorConcatHelperSchema::EXECUTING)
                        .write(state, GcOperand::boolean(value), schema, f),
                    HelperFlag::Started => schema.field(IteratorConcatHelperSchema::STARTED).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                }
            }
            Self::Zip(state) => {
                match flag {
                    HelperFlag::Done => schema.field(IteratorZipHelperSchema::DONE).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                    HelperFlag::Executing => schema
                        .field(IteratorZipHelperSchema::EXECUTING)
                        .write(state, GcOperand::boolean(value), schema, f),
                    HelperFlag::Started => schema.field(IteratorZipHelperSchema::STARTED).write(
                        state,
                        GcOperand::boolean(value),
                        schema,
                        f,
                    ),
                }
            }
        }
    }
    fn realm(&self, schema: &RuntimeSchema, f: &mut Function) -> GcLocal<RealmRecord> {
        let reference = match self {
            Self::Map(state) => schema
                .field(IteratorMapHelperSchema::REALM)
                .read(state, schema, f)
                .reference(),
            Self::Filter(state) => schema
                .field(IteratorFilterHelperSchema::REALM)
                .read(state, schema, f)
                .reference(),
            Self::FlatMap(state) => schema
                .field(IteratorFlatMapHelperSchema::REALM)
                .read(state, schema, f)
                .reference(),
            Self::Take(state) => schema
                .field(IteratorTakeHelperSchema::REALM)
                .read(state, schema, f)
                .reference(),
            Self::Drop(state) => schema
                .field(IteratorDropHelperSchema::REALM)
                .read(state, schema, f)
                .reference(),
            Self::Concat(state) => schema
                .field(IteratorConcatHelperSchema::REALM)
                .read(state, schema, f)
                .reference(),
            Self::Zip(state) => schema
                .field(IteratorZipHelperSchema::REALM)
                .read(state, schema, f)
                .reference(),
        };
        schema.reserve_gc_local(f).initialize(reference, f)
    }
    fn clear(self, f: &mut Function) {
        match self {
            Self::Map(state) => state.clear(f),
            Self::Filter(state) => state.clear(f),
            Self::FlatMap(state) => state.clear(f),
            Self::Take(state) => state.clear(f),
            Self::Drop(state) => state.clear(f),
            Self::Concat(state) => state.clear(f),
            Self::Zip(state) => state.clear(f),
        }
    }
}
impl FunctionBuilder<'_> {
    pub(super) fn emit_native_iterator_helper_dispatch(
        &mut self,
        operation: IteratorHelperOperation,
        f: &mut Function,
    ) -> Result<(), EmitError> {
        let schema = self.runtime_schema();
        let receiver = schema.reserve_value_local(f);
        self.compile_this_to_locals(&receiver, f)?;
        let output = schema.reserve_completion(f);
        output.initialize(f);
        let flag = schema.reserve_i32_local(f);
        let exit = self.open_frame(ControlFrameKind::Block, f);
        for &kind in HelperKind::ALL {
            let needed = self
                .runtime_bootstrap_plan
                .should_initialize_standard_builtin(kind.creator())
                || matches!(kind, HelperKind::Zip)
                    && self
                        .runtime_bootstrap_plan
                        .should_initialize_standard_builtin(StandardBuiltinId::IteratorZipKeyed);
            if !needed {
                continue;
            }
            kind.test(&receiver, schema, f);
            self.open_frame(ControlFrameKind::If, f);
            let state = kind.cast(&receiver, schema, f);
            state.read_flag(HelperFlag::Executing, flag, schema, f);
            flag.load(f);
            self.emit_helper_type_error_if(kind.running_message(), &output, exit, f)?;
            state.read_flag(HelperFlag::Done, flag, schema, f);
            flag.load(f);
            self.open_frame(ControlFrameKind::If, f);
            self.emit_helper_done_result(&output, f)?;
            self.emit_branch_to_target(exit, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
            let realm = match operation {
                IteratorHelperOperation::Next => state.realm(schema, f),
                IteratorHelperOperation::Return => {
                    let realm = schema
                        .reserve_gc_local(f)
                        .initialize(self.emit_current_function_realm(f), f);
                    state.read_flag(HelperFlag::Started, flag, schema, f);
                    flag.load(f);
                    self.open_frame(ControlFrameKind::If, f);
                    let saved = state.realm(schema, f);
                    realm.replace(saved.load(schema, f), f);
                    saved.clear(f);
                    self.pop_control(ControlFrameKind::If);
                    f.instruction(&Instruction::End);
                    realm
                }
            };
            let builtin = kind.entry(operation);
            let meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "missing native iterator helper entry {}",
                        builtin.debug_name()
                    ))
                })?;
            let context = self.emit_realm_function_materialization_context_from_realm(&realm, f);
            let callable = schema.reserve_gc_local(f).initialize(
                self.emit_function_value_payload_in_realm(&meta, &context, f)?,
                f,
            );
            let callee = schema.reserve_value_local(f);
            callee.set_reference(&callable, schema, f);
            let argv = self.emit_pre_evaluated_arg_vector(&[], f);
            self.emit_function_or_proxy_call_with_argv(&callee, &receiver, &argv, &output, f)?;
            argv.clear(f);
            callee.clear(f);
            callable.clear(f);
            self.release_realm_function_materialization_context(context, f);
            realm.clear(f);
            state.clear(f);
            self.emit_branch_to_target(exit, f);
            self.pop_control(ControlFrameKind::If);
            f.instruction(&Instruction::End);
        }
        self.emit_throw_current_function_realm_type_error(
            RuntimeErrorMessage::ITERATOR_HELPER_CALLED_ON_INCOMPATIBLE_RECEIVER,
            &output,
            f,
        )?;
        self.pop_control(ControlFrameKind::Block);
        f.instruction(&Instruction::End);
        self.completion().copy_from(&output, f);
        schema.release_i32_local(flag, f);
        output.clear(f);
        receiver.clear(f);
        Ok(())
    }
}
