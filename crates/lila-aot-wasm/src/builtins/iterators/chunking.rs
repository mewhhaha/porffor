//! `Iterator.prototype.chunks ( chunkSize )` and
//! `Iterator.prototype.windows ( windowSize [ , undersized ] )`
//! (proposal-iterator-chunking).
//!
//! Both create an Iterator Helper whose closure is compiled here as an
//! explicit state machine rather than a generator body. The `[[GeneratorState]]`
//! of the helper is a hidden number slot over [`ChunkingHelperState`]; the
//! `[[UnderlyingIterators]]` list is the single `iterated` record, stored as
//! its `[[Iterator]]` and `[[NextMethod]]`.
//!
//! The closure's only resumption point that can observe a `return()` is the
//! `IfAbruptCloseIterator` after yielding a full chunk or window. The final
//! yield of a partial chunk (and of an `"allow-partial"` window) is a plain
//! `? Yield`, after which both `next()` and `return()` complete the generator
//! without touching the underlying iterator, which is exactly the observable
//! behaviour of an already completed helper; that yield therefore records
//! `Completed` directly.

use super::direct_record::{ArgumentErrorKind, TaggedLocals};
use super::*;

/// The proposal method that created a chunking helper. The brand selects it
/// again when `%IteratorHelperPrototype%` dispatches `next` and `return`.
#[derive(Clone, Copy, PartialEq, Eq)]
enum IteratorChunkingKind {
    Chunks,
    Windows,
}

impl IteratorChunkingKind {
    const ALL: [Self; 2] = [Self::Chunks, Self::Windows];

    const fn brand(self) -> u64 {
        match self {
            Self::Chunks => OBJECT_INTERNAL_BRAND_ITERATOR_CHUNKS_HELPER,
            Self::Windows => OBJECT_INTERNAL_BRAND_ITERATOR_WINDOWS_HELPER,
        }
    }

    const fn property_name(self) -> &'static str {
        match self {
            Self::Chunks => "chunks",
            Self::Windows => "windows",
        }
    }

    const fn method(self) -> &'static str {
        match self {
            Self::Chunks => "Iterator.prototype.chunks",
            Self::Windows => "Iterator.prototype.windows",
        }
    }

    const fn non_object_message(self) -> &'static str {
        match self {
            Self::Chunks => "Iterator.prototype.chunks called on non-object",
            Self::Windows => "Iterator.prototype.windows called on non-object",
        }
    }

    const fn size_not_integral_message(self) -> &'static str {
        match self {
            Self::Chunks => "Iterator.prototype.chunks chunkSize must be an integral Number",
            Self::Windows => "Iterator.prototype.windows windowSize must be an integral Number",
        }
    }

    const fn size_out_of_range_message(self) -> &'static str {
        match self {
            Self::Chunks => "Iterator.prototype.chunks chunkSize must be between 1 and 2^32 - 1",
            Self::Windows => "Iterator.prototype.windows windowSize must be between 1 and 2^32 - 1",
        }
    }

    const fn incompatible_receiver_message(self) -> &'static str {
        match self {
            Self::Chunks => "Iterator chunks helper called on incompatible receiver",
            Self::Windows => "Iterator windows helper called on incompatible receiver",
        }
    }

    const fn running_message(self) -> &'static str {
        match self {
            Self::Chunks => "Iterator chunks helper is already running",
            Self::Windows => "Iterator windows helper is already running",
        }
    }

    const fn next_result_not_object_message(self) -> &'static str {
        match self {
            Self::Chunks => "Iterator chunks helper next result must be object",
            Self::Windows => "Iterator windows helper next result must be object",
        }
    }

    const fn pool_strings(self) -> [&'static str; 8] {
        [
            self.property_name(),
            self.method(),
            self.non_object_message(),
            self.size_not_integral_message(),
            self.size_out_of_range_message(),
            self.incompatible_receiver_message(),
            self.running_message(),
            self.next_result_not_object_message(),
        ]
    }
}

const WINDOWS_UNDERSIZED_MESSAGE: &str =
    "Iterator.prototype.windows undersized must be \"only-full\" or \"allow-partial\"";

/// The two accepted `undersized` spellings of `Iterator.prototype.windows`.
#[derive(Clone, Copy)]
enum WindowsUndersized {
    OnlyFull,
    AllowPartial,
}

impl WindowsUndersized {
    const ALL: [Self; 2] = [Self::OnlyFull, Self::AllowPartial];

    const fn spelling(self) -> &'static str {
        match self {
            Self::OnlyFull => "only-full",
            Self::AllowPartial => "allow-partial",
        }
    }

    const fn allows_partial(self) -> bool {
        match self {
            Self::OnlyFull => false,
            Self::AllowPartial => true,
        }
    }
}

/// The hidden slots of a chunking helper. Both kinds share the layout; only
/// `windows` carries a persistent buffer and an `undersized` mode.
#[derive(Clone, Copy)]
enum ChunkingSlot {
    Iterator,
    NextMethod,
    Size,
    State,
    Buffer,
    AllowPartial,
}

impl ChunkingSlot {
    const ALL: [Self; 6] = [
        Self::Iterator,
        Self::NextMethod,
        Self::Size,
        Self::State,
        Self::Buffer,
        Self::AllowPartial,
    ];

    const fn key(self) -> &'static str {
        match self {
            Self::Iterator => "$IteratorChunkingIterator",
            Self::NextMethod => "$IteratorChunkingNext",
            Self::Size => "$IteratorChunkingSize",
            Self::State => "$IteratorChunkingState",
            Self::Buffer => "$IteratorChunkingBuffer",
            Self::AllowPartial => "$IteratorChunkingAllowPartial",
        }
    }
}

/// The `[[GeneratorState]]` of a chunking helper, persisted as a number word.
#[derive(Clone, Copy)]
enum ChunkingHelperState {
    SuspendedStart,
    SuspendedYield,
    Executing,
    Completed,
}

impl ChunkingHelperState {
    const fn word(self) -> u64 {
        match self {
            Self::SuspendedStart => 0,
            Self::SuspendedYield => 1,
            Self::Executing => 2,
            Self::Completed => 3,
        }
    }
}

/// Every string the chunking emitters intern, walked by the string pool so a
/// message added to a `match` above cannot miss the pool.
pub(super) fn iterator_chunking_pool_strings() -> impl Iterator<Item = &'static str> {
    IteratorChunkingKind::ALL
        .into_iter()
        .flat_map(IteratorChunkingKind::pool_strings)
        .chain(WindowsUndersized::ALL.map(WindowsUndersized::spelling))
        .chain(ChunkingSlot::ALL.map(ChunkingSlot::key))
        .chain([WINDOWS_UNDERSIZED_MESSAGE])
}

impl<'a> FunctionBuilder<'a> {
    pub(crate) fn emit_iterator_prototype_chunks(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_chunking_create(IteratorChunkingKind::Chunks, function)
    }

    pub(crate) fn emit_iterator_prototype_windows(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_chunking_create(IteratorChunkingKind::Windows, function)
    }

    pub(crate) fn emit_iterator_chunks_next(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_chunking_next(IteratorChunkingKind::Chunks, function)
    }

    pub(crate) fn emit_iterator_windows_next(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_chunking_next(IteratorChunkingKind::Windows, function)
    }

    pub(crate) fn emit_iterator_chunks_return(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_chunking_return(IteratorChunkingKind::Chunks, function)
    }

    pub(crate) fn emit_iterator_windows_return(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_iterator_chunking_return(IteratorChunkingKind::Windows, function)
    }

    fn emit_iterator_chunking_create(
        &mut self,
        kind: IteratorChunkingKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver = self.direct_iterator_receiver(kind.method())?;
        // 1-2. If O is not an Object, throw a TypeError exception.
        self.emit_require_object_receiver(receiver, kind.non_object_message(), function)?;
        // 3. iterated = { [[Iterator]]: O, [[NextMethod]]: undefined, [[Done]]: false }.
        let close = self.reserve_iterator_close_on_throw_locals(receiver);
        let [size_payload_local, size_tag_local, size_local, allow_partial_local, undersized_payload_local, undersized_tag_local, spelling_local, next_payload_local, next_tag_local, prototype_local, helper_local, buffer_local, buffer_tag_local, zero_local] =
            self.reserve_temp_locals();

        // 4-5. chunkSize / windowSize validation, closing `iterated`.
        self.emit_builtin_arg_to_locals(0, size_payload_local, size_tag_local, function);
        self.emit_chunking_size_validation(
            kind,
            TaggedLocals {
                payload: size_payload_local,
                tag: size_tag_local,
            },
            size_local,
            close,
            function,
        )?;
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(allow_partial_local));
        match kind {
            IteratorChunkingKind::Chunks => {}
            // windows 6-7. undersized defaults to "only-full"; anything but the
            // two spellings is a TypeError that closes `iterated`.
            IteratorChunkingKind::Windows => self.emit_windows_undersized_validation(
                TaggedLocals {
                    payload: undersized_payload_local,
                    tag: undersized_tag_local,
                },
                spelling_local,
                allow_partial_local,
                close,
                function,
            )?,
        }

        // Set iterated to ? GetIteratorDirect(O).
        self.emit_get_iterator_direct_next(
            receiver,
            TaggedLocals {
                payload: next_payload_local,
                tag: next_tag_local,
            },
            close.key_local,
            function,
        )?;

        // CreateIteratorFromClosure(closure, "Iterator Helper",
        // %IteratorHelperPrototype%, « [[UnderlyingIterators]] »).
        self.emit_load_function_defining_realm_iterator_helper_prototype(
            self.current_env_local,
            prototype_local,
            function,
        );
        self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?;
        function.instruction(&Instruction::LocalSet(helper_local));
        self.store_i64_const_at_offset(
            helper_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            kind.brand(),
            function,
        );
        self.emit_object_define_local_data(
            helper_local,
            ChunkingSlot::Iterator.key(),
            receiver.payload,
            receiver.tag,
            function,
        )?;
        self.emit_object_define_local_data(
            helper_local,
            ChunkingSlot::NextMethod.key(),
            next_payload_local,
            next_tag_local,
            function,
        )?;
        self.emit_object_define_number_data_from_i64_local(
            helper_local,
            ChunkingSlot::Size.key(),
            size_local,
            function,
        )?;
        self.emit_chunking_state_store(
            helper_local,
            ChunkingHelperState::SuspendedStart,
            function,
        )?;
        match kind {
            IteratorChunkingKind::Chunks => {}
            IteratorChunkingKind::Windows => {
                self.emit_object_define_bool_data_from_local(
                    helper_local,
                    ChunkingSlot::AllowPartial.key(),
                    allow_partial_local,
                    function,
                )?;
                self.emit_chunking_empty_array(zero_local, buffer_local, function)?;
                function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
                function.instruction(&Instruction::LocalSet(buffer_tag_local));
                self.emit_object_define_local_data(
                    helper_local,
                    ChunkingSlot::Buffer.key(),
                    buffer_local,
                    buffer_tag_local,
                    function,
                )?;
            }
        }
        function.instruction(&Instruction::LocalGet(helper_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_locals([
            size_payload_local,
            size_tag_local,
            size_local,
            allow_partial_local,
            undersized_payload_local,
            undersized_tag_local,
            spelling_local,
            next_payload_local,
            next_tag_local,
            prototype_local,
            helper_local,
            buffer_local,
            buffer_tag_local,
            zero_local,
        ]);
        self.release_iterator_close_on_throw_locals(close);
        Ok(())
    }

    /// Steps 4-5: a TypeError unless the size is an integral Number, then a
    /// RangeError unless it lies in [1, 2^32 - 1]. Neither coerces.
    fn emit_chunking_size_validation(
        &mut self,
        kind: IteratorChunkingKind,
        size: TaggedLocals,
        size_local: u32,
        close: IteratorCloseOnThrowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(size.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Number.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_closing_direct_iterator(
            ArgumentErrorKind::Type,
            kind.size_not_integral_message(),
            close,
            function,
        )?;
        function.instruction(&Instruction::End);

        // IsIntegralNumber: `x - x` is NaN exactly for NaN and the infinities,
        // and `trunc(x) != x` for every other non-integral value.
        function.instruction(&Instruction::LocalGet(size.payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::LocalGet(size.payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Sub);
        function.instruction(&Instruction::F64Const(Ieee64::from(0.0)));
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::LocalGet(size.payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Trunc);
        function.instruction(&Instruction::LocalGet(size.payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Ne);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_closing_direct_iterator(
            ArgumentErrorKind::Type,
            kind.size_not_integral_message(),
            close,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(size.payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(1.0)));
        function.instruction(&Instruction::F64Lt);
        function.instruction(&Instruction::LocalGet(size.payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::F64Const(Ieee64::from(4_294_967_295.0)));
        function.instruction(&Instruction::F64Gt);
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_closing_direct_iterator(
            ArgumentErrorKind::Range,
            kind.size_out_of_range_message(),
            close,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(size.payload));
        function.instruction(&Instruction::F64ReinterpretI64);
        function.instruction(&Instruction::I64TruncF64U);
        function.instruction(&Instruction::LocalSet(size_local));
        Ok(())
    }

    /// windows steps 6-7, over the second argument. Sets `allow_partial_local`
    /// to 1 exactly for `"allow-partial"`.
    fn emit_windows_undersized_validation(
        &mut self,
        undersized: TaggedLocals,
        spelling_local: u32,
        allow_partial_local: u32,
        close: IteratorCloseOnThrowLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_builtin_arg_to_locals(1, undersized.payload, undersized.tag, function);
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(undersized.tag));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::BrIf(0));
        function.instruction(&Instruction::LocalGet(undersized.tag));
        function.instruction(&Instruction::I64Const(ValueKind::String.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        for mode in WindowsUndersized::ALL {
            function.instruction(&Instruction::I64Const(
                self.strings.payload(mode.spelling()),
            ));
            function.instruction(&Instruction::LocalSet(spelling_local));
            self.emit_string_payload_equality_i32(undersized.payload, spelling_local, function);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::I64Const(i64::from(mode.allows_partial())));
            function.instruction(&Instruction::LocalSet(allow_partial_local));
            // Out of this `if`, the String `if`, and the validation block.
            function.instruction(&Instruction::Br(2));
            function.instruction(&Instruction::End);
        }
        function.instruction(&Instruction::End);
        self.emit_throw_closing_direct_iterator(
            ArgumentErrorKind::Type,
            WINDOWS_UNDERSIZED_MESSAGE,
            close,
            function,
        )?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// A fresh empty Array in the running builtin's Realm, where
    /// `CreateArrayFromList` allocates. `length_local` is left at zero, which
    /// the callers rely on as their element count or index.
    fn emit_chunking_empty_array(
        &mut self,
        length_local: u32,
        array_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(length_local));
        self.emit_alloc_array_payload_with_length_in_current_function_realm(
            length_local,
            array_local,
            function,
        )
    }

    fn emit_chunking_state_load(
        &mut self,
        helper_payload_local: u32,
        state_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_read_number_slot_to_i64_local(
            helper_payload_local,
            ChunkingSlot::State.key(),
            state_local,
            function,
        )
    }

    fn emit_chunking_state_store(
        &mut self,
        helper_payload_local: u32,
        state: ChunkingHelperState,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_define_number_data_from_i64_const(
            helper_payload_local,
            ChunkingSlot::State.key(),
            state.word(),
            function,
        )
    }

    /// Records `Completed` while a throw is pending, then returns the throw.
    fn emit_chunking_complete_and_rethrow(
        &mut self,
        helper_payload_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let [saved_payload_local, saved_tag_local, saved_completion_local, saved_aux_local] =
            self.reserve_temp_locals();
        self.save_current_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_chunking_state_store(
            helper_payload_local,
            ChunkingHelperState::Completed,
            function,
        )?;
        self.restore_saved_completion(
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
            function,
        );
        self.emit_return_current_completion(function);
        self.release_temp_locals([
            saved_payload_local,
            saved_tag_local,
            saved_completion_local,
            saved_aux_local,
        ]);
        Ok(())
    }

    /// GeneratorValidate for `next` and `return`: a TypeError while the
    /// helper is executing, `{ undefined, true }` once it has completed.
    fn emit_chunking_validate_resumable(
        &mut self,
        kind: IteratorChunkingKind,
        state_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            ChunkingHelperState::Executing.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            kind.running_message(),
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            ChunkingHelperState::Completed.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_return_done_iterator_result(function)?;
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `Yield(CreateArrayFromList(buffer))` with `array_local` already the
    /// fresh array, recording `state` as the helper's state across the yield.
    fn emit_chunking_yield_array(
        &mut self,
        helper_payload_local: u32,
        array_local: u32,
        state: ChunkingHelperState,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_chunking_state_store(helper_payload_local, state, function)?;
        let [array_tag_local] = self.reserve_temp_locals();
        function.instruction(&Instruction::I64Const(ValueKind::Array.tag() as i64));
        function.instruction(&Instruction::LocalSet(array_tag_local));
        self.emit_iterator_result_object_from_locals(
            array_local,
            array_tag_local,
            false,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        self.release_temp_locals([array_tag_local]);
        Ok(())
    }

    fn emit_chunking_slot_read(
        &mut self,
        helper: TaggedLocals,
        slot: ChunkingSlot,
        key_local: u32,
        present_local: u32,
        value: TaggedLocals,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::I64Const(
            self.strings.static_builtin_property_key_payload(slot.key()),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_object_own_data_field_read(
            helper.payload,
            helper.tag,
            key_local,
            present_local,
            value.payload,
            value.tag,
            function,
        );
    }

    /// `%IteratorHelperPrototype%.next` for a chunking helper: resumes the
    /// closure until it yields the next chunk or window, or completes.
    fn emit_iterator_chunking_next(
        &mut self,
        kind: IteratorChunkingKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper = self.direct_iterator_receiver(kind.method())?;
        self.emit_iterator_helper_brand_check(
            helper.payload,
            helper.tag,
            kind.brand(),
            kind.incompatible_receiver_message(),
            function,
        )?;
        let [state_local, key_local, present_local, iterator_payload_local, iterator_tag_local, next_payload_local, next_tag_local, size_local, allow_partial_local, allow_partial_tag_local, buffer_local, buffer_tag_local, count_local, index_local, element_payload_local, element_tag_local, copy_local] =
            self.reserve_temp_locals();
        let step = self.reserve_direct_step_locals();
        let iterator = TaggedLocals {
            payload: iterator_payload_local,
            tag: iterator_tag_local,
        };
        let next = TaggedLocals {
            payload: next_payload_local,
            tag: next_tag_local,
        };

        self.emit_chunking_state_load(helper.payload, state_local, function)?;
        self.emit_chunking_validate_resumable(kind, state_local, function)?;
        self.emit_chunking_state_store(helper.payload, ChunkingHelperState::Executing, function)?;
        self.emit_chunking_slot_read(
            helper,
            ChunkingSlot::Iterator,
            key_local,
            present_local,
            iterator,
            function,
        );
        self.emit_chunking_slot_read(
            helper,
            ChunkingSlot::NextMethod,
            key_local,
            present_local,
            next,
            function,
        );
        self.emit_object_read_number_slot_to_i64_local(
            helper.payload,
            ChunkingSlot::Size.key(),
            size_local,
            function,
        )?;
        match kind {
            IteratorChunkingKind::Chunks => {
                // The buffer never outlives one resumption: every full chunk is
                // yielded and replaced, and exhaustion completes the helper.
                self.emit_chunking_empty_array(count_local, buffer_local, function)?;
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::LocalSet(allow_partial_local));
            }
            IteratorChunkingKind::Windows => {
                self.emit_chunking_slot_read(
                    helper,
                    ChunkingSlot::Buffer,
                    key_local,
                    present_local,
                    TaggedLocals {
                        payload: buffer_local,
                        tag: buffer_tag_local,
                    },
                    function,
                );
                self.load_i64_to_local_from_offset(
                    buffer_local,
                    HEAP_LEN_OFFSET,
                    count_local,
                    function,
                );
                self.emit_chunking_slot_read(
                    helper,
                    ChunkingSlot::AllowPartial,
                    key_local,
                    present_local,
                    TaggedLocals {
                        payload: allow_partial_local,
                        tag: allow_partial_tag_local,
                    },
                    function,
                );
            }
        }

        let element = TaggedLocals {
            payload: element_payload_local,
            tag: element_tag_local,
        };
        self.emit_capturing_throws(function, |builder, function| {
            function.instruction(&Instruction::Loop(BlockType::Empty));
            // Let value be ? IteratorStepValue(iterated).
            builder.emit_direct_iterator_step_done_i32(
                iterator,
                next,
                &step,
                kind.next_result_not_object_message(),
                function,
            )?;
            function.instruction(&Instruction::If(BlockType::Empty));
            builder.emit_chunking_exhausted(
                kind,
                helper,
                buffer_local,
                count_local,
                size_local,
                allow_partial_local,
                function,
            )?;
            function.instruction(&Instruction::End);
            builder.emit_direct_iterator_step_value(&step, function)?;
            let value = step.value();
            match kind {
                IteratorChunkingKind::Chunks => {
                    // Append value to buffer.
                    builder.emit_array_write(
                        buffer_local,
                        count_local,
                        value.payload,
                        value.tag,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(count_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(count_local));
                    // If the number of elements in buffer is chunkSize, yield it.
                    function.instruction(&Instruction::LocalGet(count_local));
                    function.instruction(&Instruction::LocalGet(size_local));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    builder.emit_chunking_yield_array(
                        helper.payload,
                        buffer_local,
                        ChunkingHelperState::SuspendedYield,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                }
                IteratorChunkingKind::Windows => {
                    // If the number of elements in buffer is windowSize, remove
                    // the first element; then append value.
                    function.instruction(&Instruction::LocalGet(count_local));
                    function.instruction(&Instruction::LocalGet(size_local));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    builder.emit_windows_shift_append(
                        buffer_local,
                        count_local,
                        index_local,
                        copy_local,
                        element,
                        value,
                        function,
                    )?;
                    function.instruction(&Instruction::Else);
                    builder.emit_array_write(
                        buffer_local,
                        count_local,
                        value.payload,
                        value.tag,
                        function,
                    )?;
                    function.instruction(&Instruction::LocalGet(count_local));
                    function.instruction(&Instruction::I64Const(1));
                    function.instruction(&Instruction::I64Add);
                    function.instruction(&Instruction::LocalSet(count_local));
                    function.instruction(&Instruction::End);
                    // If the number of elements in buffer is windowSize, yield
                    // CreateArrayFromList(buffer): a fresh copy, because the
                    // buffer keeps sliding.
                    function.instruction(&Instruction::LocalGet(count_local));
                    function.instruction(&Instruction::LocalGet(size_local));
                    function.instruction(&Instruction::I64Eq);
                    function.instruction(&Instruction::If(BlockType::Empty));
                    builder.emit_array_copy_prefix(
                        buffer_local,
                        count_local,
                        index_local,
                        copy_local,
                        element,
                        function,
                    )?;
                    builder.emit_chunking_yield_array(
                        helper.payload,
                        copy_local,
                        ChunkingHelperState::SuspendedYield,
                        function,
                    )?;
                    function.instruction(&Instruction::End);
                }
            }
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            Ok(())
        })?;
        // Only a throw reaches here: an abrupt closure completes the helper
        // without closing `iterated`.
        self.emit_chunking_complete_and_rethrow(helper.payload, function)?;

        self.release_direct_step_locals(step);
        self.release_temp_locals([
            state_local,
            key_local,
            present_local,
            iterator_payload_local,
            iterator_tag_local,
            next_payload_local,
            next_tag_local,
            size_local,
            allow_partial_local,
            allow_partial_tag_local,
            buffer_local,
            buffer_tag_local,
            count_local,
            index_local,
            element_payload_local,
            element_tag_local,
            copy_local,
        ]);
        Ok(())
    }

    /// `If value is done`: chunks yields a non-empty buffer and windows an
    /// `"allow-partial"` buffer shorter than windowSize, both as the closure's
    /// last yield; otherwise the closure returns. Either way the helper is
    /// complete afterwards (see the module note).
    #[allow(clippy::too_many_arguments)]
    fn emit_chunking_exhausted(
        &mut self,
        kind: IteratorChunkingKind,
        helper: TaggedLocals,
        buffer_local: u32,
        count_local: u32,
        size_local: u32,
        allow_partial_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        match kind {
            IteratorChunkingKind::Chunks => {
                function.instruction(&Instruction::LocalGet(count_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
            }
            IteratorChunkingKind::Windows => {
                function.instruction(&Instruction::LocalGet(allow_partial_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::LocalGet(count_local));
                function.instruction(&Instruction::I64Const(0));
                function.instruction(&Instruction::I64Ne);
                function.instruction(&Instruction::I32And);
                function.instruction(&Instruction::LocalGet(count_local));
                function.instruction(&Instruction::LocalGet(size_local));
                function.instruction(&Instruction::I64LtU);
                function.instruction(&Instruction::I32And);
            }
        }
        function.instruction(&Instruction::If(BlockType::Empty));
        // The buffer was never exposed: a chunk buffer is fresh per
        // resumption, and a windows buffer is yielded only once it is full.
        self.emit_chunking_yield_array(
            helper.payload,
            buffer_local,
            ChunkingHelperState::Completed,
            function,
        )?;
        function.instruction(&Instruction::End);
        self.emit_chunking_state_store(helper.payload, ChunkingHelperState::Completed, function)?;
        self.emit_return_done_iterator_result(function)
    }

    /// Removes the first of `count` elements of `buffer` and appends `value`,
    /// in place. The windows buffer is private to the helper, so sliding it
    /// is unobservable.
    #[allow(clippy::too_many_arguments)]
    fn emit_windows_shift_append(
        &mut self,
        buffer_local: u32,
        count_local: u32,
        index_local: u32,
        target_index_local: u32,
        element: TaggedLocals,
        value: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            buffer_local,
            index_local,
            element.payload,
            element.tag,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(target_index_local));
        self.emit_array_write(
            buffer_local,
            target_index_local,
            element.payload,
            element.tag,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(target_index_local));
        self.emit_array_write(
            buffer_local,
            target_index_local,
            value.payload,
            value.tag,
            function,
        )
    }

    /// `CreateArrayFromList` over the first `count` elements of the dense
    /// private `buffer`, into `copy_local`.
    fn emit_array_copy_prefix(
        &mut self,
        buffer_local: u32,
        count_local: u32,
        index_local: u32,
        copy_local: u32,
        element: TaggedLocals,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_chunking_empty_array(index_local, copy_local, function)?;
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(count_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        self.emit_array_read(
            buffer_local,
            index_local,
            element.payload,
            element.tag,
            function,
        );
        self.emit_array_write(
            copy_local,
            index_local,
            element.payload,
            element.tag,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// `%IteratorHelperPrototype%.return` for a chunking helper.
    ///
    /// From suspended-start the helper completes before `IteratorCloseAll`;
    /// from suspended-yield the resumed `IfAbruptCloseIterator` closes
    /// `iterated` while the helper is still executing. Both close exactly the
    /// underlying iterator and complete the helper, even when closing throws.
    fn emit_iterator_chunking_return(
        &mut self,
        kind: IteratorChunkingKind,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let helper = self.direct_iterator_receiver(kind.method())?;
        self.emit_iterator_helper_brand_check(
            helper.payload,
            helper.tag,
            kind.brand(),
            kind.incompatible_receiver_message(),
            function,
        )?;
        let [state_local, key_local, present_local, iterator_payload_local, iterator_tag_local, return_payload_local, return_tag_local, close_result_payload_local, close_result_tag_local] =
            self.reserve_temp_locals();
        let iterator = TaggedLocals {
            payload: iterator_payload_local,
            tag: iterator_tag_local,
        };

        self.emit_chunking_state_load(helper.payload, state_local, function)?;
        self.emit_chunking_validate_resumable(kind, state_local, function)?;
        self.emit_chunking_slot_read(
            helper,
            ChunkingSlot::Iterator,
            key_local,
            present_local,
            iterator,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            ChunkingHelperState::SuspendedYield.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_chunking_state_store(helper.payload, ChunkingHelperState::Executing, function)?;
        function.instruction(&Instruction::Else);
        self.emit_chunking_state_store(helper.payload, ChunkingHelperState::Completed, function)?;
        function.instruction(&Instruction::End);
        self.emit_capturing_throws(function, |builder, function| {
            builder.emit_iterator_close(
                iterator.payload,
                iterator.tag,
                key_local,
                return_payload_local,
                return_tag_local,
                close_result_payload_local,
                close_result_tag_local,
                function,
            )
        })?;
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_chunking_complete_and_rethrow(helper.payload, function)?;
        function.instruction(&Instruction::End);
        self.emit_chunking_state_store(helper.payload, ChunkingHelperState::Completed, function)?;
        self.emit_return_done_iterator_result(function)?;

        self.release_temp_locals([
            state_local,
            key_local,
            present_local,
            iterator_payload_local,
            iterator_tag_local,
            return_payload_local,
            return_tag_local,
            close_result_payload_local,
            close_result_tag_local,
        ]);
        Ok(())
    }
}
