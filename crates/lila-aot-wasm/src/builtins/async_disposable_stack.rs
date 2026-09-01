//! `%AsyncDisposableStack%` (ECMA-262 explicit-resource-management, 27.4).
//!
//! Structured after `builtins/finalization_registry.rs` — an ordinary object
//! whose internal slots live in a side record hung off
//! `HEAP_OBJECT_BOXED_PAYLOAD_OFFSET`, branded at
//! `HEAP_OBJECT_INTERNAL_BRAND_OFFSET` so every prototype method can reject a
//! foreign receiver — and after `builtins/async_iterator.rs` for the
//! promise-returning half: `disposeAsync` parks its walk state on the heap,
//! hangs that state off two builtin callbacks' `[[Environment]]` slot, and
//! re-enters the same emitted walk from either callback.
//!
//! Three spec shapes are collapsed on purpose, each because the collapse is
//! observationally identical and the alternative is a builtin function object
//! minted per call:
//!
//! * `adopt(V, onDisposeAsync)` stores `(V, onDisposeAsync)` flat with an
//!   `ADOPT` kind instead of capturing the spec's `() => onDisposeAsync(V)`
//!   closure. The disposal walk performs `Call(onDisposeAsync, undefined, « V »)`,
//!   which is what that closure does.
//! * `defer(onDisposeAsync)` likewise stores a `DEFER` kind.
//! * `GetDisposeMethod(V, async-dispose)`'s `@@dispose` fallback stores the
//!   synchronous method directly rather than the spec's promise-returning
//!   wrapper. `lower_using_declaration` already took this decision for
//!   `await using` (see the comment at `lowering.rs`'s
//!   `add_disposable_resource_statements`): awaiting the synchronous call's own
//!   result is the same observable sequence.
//!
//! What is *not* collapsed is the `Await` itself. `Dispose` awaits once per
//! entry even when there is no method to call, which is the whole content of
//! `disposeAsync/explicit-await-for-null.js` and
//! `disposeAsync/explicit-await-for-undefined.js`; and an empty stack awaits
//! zero times, which is `disposeAsync/explicit-await-skipped-when-empty.js`.
//! The walk is therefore driven by the entry list, not by whether a call
//! happened.

use super::super::*;
use crate::functions::NewTargetPrototypeFallback;

/// The disposal walk's parked state, hung off both settlement callbacks'
/// `[[Environment]]` slot. It is *not* the `AsyncDisposableStack` record: the
/// entry pointer and the descending index are snapshotted at `disposeAsync`
/// time so the walk cannot be perturbed by anything the disposers do to the
/// stack object.
const ASYNC_DISPOSABLE_STACK_DISPOSE_STATE_SIZE: u64 = 56;
const ASYNC_DISPOSABLE_STACK_DISPOSE_PROMISE_RECORD_OFFSET: u64 = 0;
const ASYNC_DISPOSABLE_STACK_DISPOSE_REALM_ENV_OFFSET: u64 = 8;
const ASYNC_DISPOSABLE_STACK_DISPOSE_ENTRIES_PTR_OFFSET: u64 = 16;
/// The index of the entry disposed *next*, counting down. `DisposeResources`
/// walks `[[DisposableResourceStack]]` in reverse list order, so the walk
/// starts at the length and stops at zero.
const ASYNC_DISPOSABLE_STACK_DISPOSE_INDEX_OFFSET: u64 = 24;
/// The closed completion kind threaded through the disposal walk. The two
/// value words below are meaningful only for `Throw`.
const ASYNC_DISPOSABLE_STACK_DISPOSE_COMPLETION_KIND_OFFSET: u64 = 32;
const ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_TAG_OFFSET: u64 = 40;
const ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_PAYLOAD_OFFSET: u64 = 48;

enum AsyncDisposableStackDisposeCompletionKind {
    Normal,
    Throw,
}

impl AsyncDisposableStackDisposeCompletionKind {
    fn word(&self) -> u64 {
        match self {
            Self::Normal => 0,
            Self::Throw => 1,
        }
    }
}

#[must_use = "a loaded disposal completion kind must be consumed by an exhaustive route"]
struct AsyncDisposableStackDisposeCompletionKindLocal(u32);

impl<'a> FunctionBuilder<'a> {
    /// `AsyncDisposableStack ( )` (27.4.1.1).
    pub(crate) fn emit_async_disposable_stack_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let stack_payload_local = self.reserve_temp_local();
        let stack_record_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_type_error(
            "AsyncDisposableStack constructor requires new",
            function,
        )?;
        function.instruction(&Instruction::End);

        // `NewTargetPrototypeFallback::CurrentGlobal` rather than
        // `RealmIntrinsic`: the only case that can tell the two apart is
        // `proto-from-ctor-realm.js`, which needs the cross-realm `Function`
        // constructor and is a declared policy case on this backend. Choosing
        // the global keeps `%AsyncDisposableStack.prototype%` out of the
        // realm-intrinsics record.
        self.emit_new_target_prototype_to_locals(
            ASYNC_DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
            NewTargetPrototypeFallback::CurrentGlobal,
            prototype_payload_local,
            prototype_tag_local,
            function,
        )?;
        self.emit_alloc_plain_object_with_prototype_and_tag(
            Some(prototype_payload_local),
            Some(prototype_tag_local),
            None,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(stack_payload_local));
        self.emit_heap_alloc_const(HEAP_ASYNC_DISPOSABLE_STACK_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(stack_record_local));
        self.emit_heap_alloc_const(
            MIN_HEAP_CAPACITY * HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
            AsyncDisposableStackState::Pending.word(),
            function,
        );
        self.store_i64_local_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            MIN_HEAP_CAPACITY,
            function,
        );
        self.store_i64_const_at_offset(
            stack_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ASYNC_DISPOSABLE_STACK,
            function,
        );
        self.store_i64_local_at_offset(
            stack_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            stack_record_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(stack_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(stack_record_local);
        self.release_temp_local(stack_payload_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    /// `AsyncDisposableStack.prototype.use ( value )` (27.4.3.7).
    pub(crate) fn emit_async_disposable_stack_use(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let key_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();

        self.emit_async_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.emit_async_disposable_stack_require_pending(stack_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);

        function.instruction(&Instruction::I64Const(
            AsyncDisposableStackEntryKind::Empty.word() as i64,
        ));
        function.instruction(&Instruction::LocalSet(kind_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(method_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(method_tag_local));

        // `CreateDisposableResource(V, async-dispose)`: a null or undefined
        // resource keeps both slots undefined and is still appended, so the
        // walk still awaits once for it.
        self.emit_async_disposable_stack_is_nullish_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_is_heap_object_like_tag_i32(value_tag_local, function);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_type_error(
            "AsyncDisposableStack.prototype.use value is not an object",
            function,
        )?;
        function.instruction(&Instruction::End);

        // `GetDisposeMethod(V, async-dispose)`: `@@asyncDispose` first, and only
        // if that yields undefined is `@@dispose` read at all — the read order
        // and the read *count* are both asserted
        // (`use/gets-value-Symbol.asyncDispose-property-once.js`,
        // `use/gets-value-Symbol.dispose-property-after-trying-Symbol.asyncDispose.js`).
        function.instruction(&Instruction::I64Const(
            self.strings
                .property_key_symbol_payload("Symbol.asyncDispose"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_async_disposable_stack_get_method(
            value_payload_local,
            value_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(
            self.strings.property_key_symbol_payload("Symbol.dispose"),
        ));
        function.instruction(&Instruction::LocalSet(key_local));
        self.emit_async_disposable_stack_get_method(
            value_payload_local,
            value_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(method_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_type_error(
            "AsyncDisposableStack.prototype.use value is not disposable",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::I64Const(
            AsyncDisposableStackEntryKind::Use.word() as i64,
        ));
        function.instruction(&Instruction::LocalSet(kind_local));
        function.instruction(&Instruction::End);

        self.emit_async_disposable_stack_push_entry(
            stack_record_local,
            kind_local,
            value_payload_local,
            value_tag_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(kind_local);
        self.release_temp_local(key_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    /// `AsyncDisposableStack.prototype.adopt ( value, onDisposeAsync )`
    /// (27.4.3.1).
    pub(crate) fn emit_async_disposable_stack_adopt(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();

        self.emit_async_disposable_stack_record_from_receiver(stack_record_local, function)?;
        // Step order is observable: `adopt/throws-if-disposed.js` passes an
        // acceptable callback and still wants the ReferenceError, so the
        // disposed check precedes the callable check.
        self.emit_async_disposable_stack_require_pending(stack_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, value_payload_local, value_tag_local, function);
        self.emit_builtin_arg_to_locals(1, method_payload_local, method_tag_local, function);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_type_error(
            "AsyncDisposableStack.prototype.adopt onDisposeAsync is not callable",
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(
            AsyncDisposableStackEntryKind::Adopt.word() as i64,
        ));
        function.instruction(&Instruction::LocalSet(kind_local));
        self.emit_async_disposable_stack_push_entry(
            stack_record_local,
            kind_local,
            value_payload_local,
            value_tag_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;

        function.instruction(&Instruction::LocalGet(value_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(value_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(kind_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    /// `AsyncDisposableStack.prototype.defer ( onDisposeAsync )` (27.4.3.3).
    pub(crate) fn emit_async_disposable_stack_defer(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();

        self.emit_async_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.emit_async_disposable_stack_require_pending(stack_record_local, function)?;
        self.emit_builtin_arg_to_locals(0, method_payload_local, method_tag_local, function);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_type_error(
            "AsyncDisposableStack.prototype.defer onDisposeAsync is not callable",
            function,
        )?;
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(value_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(value_tag_local));
        function.instruction(&Instruction::I64Const(
            AsyncDisposableStackEntryKind::Defer.word() as i64,
        ));
        function.instruction(&Instruction::LocalSet(kind_local));
        self.emit_async_disposable_stack_push_entry(
            stack_record_local,
            kind_local,
            value_payload_local,
            value_tag_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(kind_local);
        self.release_temp_local(method_tag_local);
        self.release_temp_local(method_payload_local);
        self.release_temp_local(value_tag_local);
        self.release_temp_local(value_payload_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    /// `AsyncDisposableStack.prototype.move ( )` (27.4.3.6).
    ///
    /// The new stack is created from `%AsyncDisposableStack.prototype%`
    /// directly, not from the receiver's prototype:
    /// `move/still-returns-new-asyncdisposablestack-when-subclassed.js` asserts
    /// the result is *not* an instance of the subclass.
    pub(crate) fn emit_async_disposable_stack_move(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let moved_payload_local = self.reserve_temp_local();
        let moved_record_local = self.reserve_temp_local();
        let copied_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();

        self.emit_async_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.emit_async_disposable_stack_require_pending(stack_record_local, function)?;

        self.emit_alloc_plain_object_with_prototype(
            None,
            Some(ASYNC_DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX),
            function,
        )?;
        function.instruction(&Instruction::LocalSet(moved_payload_local));
        self.emit_heap_alloc_const(HEAP_ASYNC_DISPOSABLE_STACK_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(moved_record_local));
        self.store_i64_const_at_offset(
            moved_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
            AsyncDisposableStackState::Pending.word(),
            function,
        );
        for offset in [
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(stack_record_local, offset, copied_local, function);
            self.store_i64_local_at_offset(moved_record_local, offset, copied_local, function);
        }
        self.store_i64_const_at_offset(
            moved_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_ASYNC_DISPOSABLE_STACK,
            function,
        );
        self.store_i64_local_at_offset(
            moved_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            moved_record_local,
            function,
        );

        // The receiver keeps a *fresh* empty DisposeCapability rather than
        // aliasing the moved one, then goes to `disposed`.
        // `move/does-not-dispose-resources.js` requires that nothing runs here.
        self.emit_heap_alloc_const(
            MIN_HEAP_CAPACITY * HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE,
            function,
        )?;
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        self.store_i64_local_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            MIN_HEAP_CAPACITY,
            function,
        );
        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
            AsyncDisposableStackState::Disposed.word(),
            function,
        );

        function.instruction(&Instruction::LocalGet(moved_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(entries_ptr_local);
        self.release_temp_local(copied_local);
        self.release_temp_local(moved_record_local);
        self.release_temp_local(moved_payload_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    /// `get AsyncDisposableStack.prototype.disposed` (27.4.3.5).
    pub(crate) fn emit_async_disposable_stack_disposed_getter(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let stack_record_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();

        self.emit_async_disposable_stack_record_from_receiver(stack_record_local, function)?;
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            AsyncDisposableStackState::Disposed.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I64ExtendI32U);
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Boolean.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(state_local);
        self.release_temp_local(stack_record_local);
        Ok(())
    }

    /// `AsyncDisposableStack.prototype.disposeAsync ( )` (27.4.3.4).
    ///
    /// Every failure mode here *rejects* rather than throws — the method is
    /// specified as returning a promise before it inspects the receiver at all,
    /// which is what `disposeAsync/this-not-object-rejects.js` and
    /// `disposeAsync/this-does-not-have-internal-asyncdisposablestate-rejects.js`
    /// distinguish from the synchronous members' TypeErrors.
    pub(crate) fn emit_async_disposable_stack_dispose_async(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let promise_constructor_payload_local = self.reserve_temp_local();
        let promise_constructor_tag_local = self.reserve_temp_local();
        let capability_record_local = self.reserve_temp_local();
        let promise_payload_local = self.reserve_temp_local();
        let promise_tag_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();
        let stack_record_local = self.reserve_temp_local();
        let state_local = self.reserve_temp_local();
        let dispose_state_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();

        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(promise_constructor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(promise_constructor_tag_local));
        self.emit_new_promise_capability(
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            capability_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        self.load_i64_to_local_from_offset(
            promise_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            promise_record_local,
            function,
        );

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "AsyncDisposableStack method receiver is not an object",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_async_disposable_stack_reject_current_throw_and_return(
            promise_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_ASYNC_DISPOSABLE_STACK as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "AsyncDisposableStack method receiver does not have [[AsyncDisposableState]]",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_async_disposable_stack_reject_current_throw_and_return(
            promise_record_local,
            promise_payload_local,
            promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            stack_record_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
            state_local,
            function,
        );
        // Already disposed: resolve with undefined and, critically, do not walk
        // the entry list again. This is the whole content of
        // `does-not-reinvoke-disposers-if-already-disposed.js` and, because the
        // state flips *before* the first await below, of
        // `does-not-reinvoke-disposers-if-dispose-already-started.js`.
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            AsyncDisposableStackState::Disposed.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_settle_undefined(
            promise_record_local,
            PromiseSettlement::Fulfill,
            function,
        )?;
        self.emit_async_disposable_stack_return_promise(
            promise_payload_local,
            promise_tag_local,
            function,
        );
        function.instruction(&Instruction::End);
        self.store_i64_const_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
            AsyncDisposableStackState::Disposed.word(),
            function,
        );

        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.emit_heap_alloc_const(ASYNC_DISPOSABLE_STACK_DISPOSE_STATE_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(dispose_state_local));
        self.store_i64_local_at_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_PROMISE_RECORD_OFFSET,
            promise_record_local,
            function,
        );
        self.store_i64_local_at_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_REALM_ENV_OFFSET,
            self.current_env_local,
            function,
        );
        self.store_i64_local_at_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.store_i64_local_at_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_INDEX_OFFSET,
            entries_len_local,
            function,
        );
        self.emit_store_async_disposable_stack_dispose_completion_kind(
            dispose_state_local,
            AsyncDisposableStackDisposeCompletionKind::Normal,
            function,
        );
        for offset in [
            ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_TAG_OFFSET,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_PAYLOAD_OFFSET,
        ] {
            self.store_i64_const_at_offset(dispose_state_local, offset, 0, function);
        }

        self.emit_async_disposable_stack_dispose_step(dispose_state_local, function)?;
        self.emit_async_disposable_stack_return_promise(
            promise_payload_local,
            promise_tag_local,
            function,
        );

        for local in [
            entries_len_local,
            entries_ptr_local,
            dispose_state_local,
            state_local,
            stack_record_local,
            receiver_brand_local,
            receiver_tag_local,
            receiver_payload_local,
            promise_record_local,
            promise_tag_local,
            promise_payload_local,
            capability_record_local,
            promise_constructor_tag_local,
            promise_constructor_payload_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// The `Await` fulfilment handler for one disposal step: resume the walk.
    pub(crate) fn emit_async_disposable_stack_dispose_async_fulfilled(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let dispose_state_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(dispose_state_local));
        self.emit_async_disposable_stack_restore_realm_env(dispose_state_local, function);
        self.emit_async_disposable_stack_dispose_step(dispose_state_local, function)?;
        self.emit_async_disposable_stack_return_undefined(function);

        self.release_temp_local(dispose_state_local);
        Ok(())
    }

    /// The `Await` rejection handler: fold the reason into the walk's threaded
    /// completion, then resume. `DisposeResources` does not stop at the first
    /// error — `rejects-with-suppressederror-if-multiple-errors-during-disposal.js`
    /// requires all three disposers to have run.
    pub(crate) fn emit_async_disposable_stack_dispose_async_rejected(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let dispose_state_local = self.reserve_temp_local();
        let reason_payload_local = self.reserve_temp_local();
        let reason_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(0));
        function.instruction(&Instruction::LocalSet(dispose_state_local));
        self.emit_async_disposable_stack_restore_realm_env(dispose_state_local, function);
        self.emit_builtin_arg_to_locals(0, reason_payload_local, reason_tag_local, function);
        self.emit_async_disposable_stack_record_error(
            dispose_state_local,
            reason_payload_local,
            reason_tag_local,
            function,
        )?;
        self.emit_async_disposable_stack_dispose_step(dispose_state_local, function)?;
        self.emit_async_disposable_stack_return_undefined(function);

        self.release_temp_local(reason_tag_local);
        self.release_temp_local(reason_payload_local);
        self.release_temp_local(dispose_state_local);
        Ok(())
    }

    /// `DisposeResources` (27.3.1.5), as one resumable step emitted into all
    /// three of `disposeAsync` and its two settlement callbacks.
    ///
    /// It runs disposers until one of them yields a value to await, arms that
    /// await against the same parked state, and stops; or it runs out of
    /// entries and settles the outer promise. A disposer that throws
    /// *synchronously* never suspends, so a stack of three synchronously
    /// throwing `defer`s is walked entirely inside one call — which is exactly
    /// the `rejects-with-suppressederror-...` case, and is why the aggregation
    /// lives here rather than only in the rejection handler.
    fn emit_async_disposable_stack_dispose_step(
        &mut self,
        dispose_state_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let suspended_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let entries_ptr_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();
        let kind_local = self.reserve_temp_local();
        let value_payload_local = self.reserve_temp_local();
        let value_tag_local = self.reserve_temp_local();
        let method_payload_local = self.reserve_temp_local();
        let method_tag_local = self.reserve_temp_local();
        let awaited_payload_local = self.reserve_temp_local();
        let awaited_tag_local = self.reserve_temp_local();
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        let promise_constructor_payload_local = self.reserve_temp_local();
        let promise_constructor_tag_local = self.reserve_temp_local();
        let throwaway_capability_local = self.reserve_temp_local();
        let throwaway_promise_payload_local = self.reserve_temp_local();
        let throwaway_promise_tag_local = self.reserve_temp_local();
        let fulfilled_payload_local = self.reserve_temp_local();
        let rejected_payload_local = self.reserve_temp_local();
        let callback_tag_local = self.reserve_temp_local();
        let promise_record_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(suspended_local));

        // Branch depths inside the loop body are counted from here:
        //   [0] = Loop  (continue with the next entry)
        //   [1] = Block (walk finished; fall through to the settle below)
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_INDEX_OFFSET,
            index_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::BrIf(1));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Sub);
        function.instruction(&Instruction::LocalSet(index_local));
        self.store_i64_local_at_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_INDEX_OFFSET,
            index_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
            kind_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
            value_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
            value_payload_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
            method_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            entry_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
            method_payload_local,
            function,
        );

        // `Dispose(V, hint, method)` step 1: with no method there is no call,
        // and `result` is undefined — but step 3 still awaits it.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(awaited_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(awaited_tag_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(error_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(error_tag_local));
        // `adopt` and `defer` entries call their callback with an undefined
        // receiver; only a `use` entry passes the resource as `this`.
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));

        // The dispatch is DERIVED from `AsyncDisposableStackEntryKind::ALL`,
        // not transcribed. The chain's last arm is an emitted `Else`, so a
        // fifth entry kind added to a transcribed chain would have been
        // disposed silently as a `defer`; here it extends the chain instead,
        // and `dispose_call` makes forgetting its shape an `error[E0004]`.
        //
        // Emitted shape for the four kinds of today — instruction for
        // instruction what the hand-written chain emitted:
        //
        //   kind != Empty ; If                      // the no-call kinds
        //     kind == Use ; If   <call: resource receiver, no args>
        //     Else
        //       kind == Adopt ; If  <call: undefined receiver, « V »>
        //       Else                <call: undefined receiver, « »>
        //       End
        //     End
        //   End
        let mut no_call_kinds = Vec::new();
        let mut calling_kinds = Vec::new();
        for kind in AsyncDisposableStackEntryKind::ALL {
            match kind.dispose_call() {
                None => no_call_kinds.push(kind),
                Some(call) => calling_kinds.push((kind, call)),
            }
        }

        // The throw test after the chain reads `completion_local`
        // UNCONDITIONALLY, but the call that writes it is inside the guard
        // below. An `EMPTY` entry — `use(null)` / `use(undefined)`, i.e.
        // `explicit-await-for-null.js` and `explicit-await-for-undefined.js` —
        // performs no call and would otherwise test whatever the previous
        // emitter happened to leave there. Every producer reachable today
        // leaves `Normal` (`emit_new_promise_capability` and
        // `emit_intrinsic_await_reactions` both end with it, and the
        // synchronous-throw arm resets before its `Br`), so this is latent
        // rather than live — but that is a property of every helper this file
        // happens to call, not of this walk. Establishing it here makes the
        // no-call path structural.
        self.set_completion_kind(CompletionKind::Normal, function);

        // Guard: skip the whole chain for any kind that performs no call. With
        // more than one such kind the tests fold with `I32And`; `I64Ne` already
        // yields i32.
        for (index, kind) in no_call_kinds.iter().enumerate() {
            function.instruction(&Instruction::LocalGet(kind_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Ne);
            if index > 0 {
                function.instruction(&Instruction::I32And);
            }
        }
        let has_no_call_guard = !no_call_kinds.is_empty();
        if has_no_call_guard {
            function.instruction(&Instruction::If(BlockType::Empty));
        }

        let no_arguments: [(u32, u32); 0] = [];
        let resource_argument = [(value_payload_local, value_tag_local)];
        let last_calling_index = calling_kinds.len().saturating_sub(1);
        for (index, (kind, call)) in calling_kinds.iter().enumerate() {
            // Every kind but the last gets an explicit comparison; the last is
            // the `Else` the preceding one opened.
            if index < last_calling_index {
                function.instruction(&Instruction::LocalGet(kind_local));
                function.instruction(&Instruction::I64Const(kind.word() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
            }
            let (this_payload_local, this_tag_local, arguments): (u32, u32, &[(u32, u32)]) =
                match call {
                    AsyncDisposableStackDisposeCall::ResourceReceiver => {
                        (value_payload_local, value_tag_local, &no_arguments)
                    }
                    AsyncDisposableStackDisposeCall::UndefinedReceiverWithResourceArgument => (
                        undefined_payload_local,
                        undefined_tag_local,
                        &resource_argument,
                    ),
                    AsyncDisposableStackDisposeCall::UndefinedReceiverNoArguments => {
                        (undefined_payload_local, undefined_tag_local, &no_arguments)
                    }
                };
            self.emit_function_or_proxy_call_leave_throw_completion(
                method_payload_local,
                method_tag_local,
                this_payload_local,
                this_tag_local,
                arguments,
                awaited_payload_local,
                awaited_tag_local,
                function,
            )?;
            if index < last_calling_index {
                function.instruction(&Instruction::Else);
            }
        }
        for _ in 0..last_calling_index {
            function.instruction(&Instruction::End);
        }
        if has_no_call_guard {
            function.instruction(&Instruction::End);
        }

        // A synchronous throw is folded into the threaded completion and the
        // walk continues with the next entry, without awaiting.
        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(error_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(error_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_async_disposable_stack_record_error(
            dispose_state_local,
            error_payload_local,
            error_tag_local,
            function,
        )?;
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);

        for (builtin, callback_payload_local) in [
            (
                StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled,
                fulfilled_payload_local,
            ),
            (
                StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected,
                rejected_payload_local,
            ),
        ] {
            let callback_meta = self
                .functions
                .get(&builtin.function_id())
                .cloned()
                .ok_or_else(|| {
                    EmitError::unsupported(format!(
                        "unsupported in lila wasm-aot first slice: missing builtin meta `{}`",
                        builtin.debug_name()
                    ))
                })?;
            self.emit_function_value_payload(&callback_meta, function)?;
            function.instruction(&Instruction::LocalSet(callback_payload_local));
            self.store_i64_local_at_offset(
                callback_payload_local,
                HEAP_FUNCTION_ENV_HANDLE_OFFSET,
                dispose_state_local,
                function,
            );
        }
        function.instruction(&Instruction::GlobalGet(PROMISE_CONSTRUCTOR_GLOBAL_INDEX));
        function.instruction(&Instruction::LocalSet(promise_constructor_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(promise_constructor_tag_local));
        self.emit_new_promise_capability(
            promise_constructor_payload_local,
            promise_constructor_tag_local,
            throwaway_capability_local,
            throwaway_promise_payload_local,
            throwaway_promise_tag_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(ValueKind::Function.tag() as i64));
        function.instruction(&Instruction::LocalSet(callback_tag_local));
        self.emit_intrinsic_await_with_handlers(
            awaited_payload_local,
            awaited_tag_local,
            fulfilled_payload_local,
            callback_tag_local,
            rejected_payload_local,
            callback_tag_local,
            throwaway_capability_local,
            function,
        )?;
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::LocalSet(suspended_local));
        function.instruction(&Instruction::Br(1));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(suspended_local));
        function.instruction(&Instruction::I64Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_PROMISE_RECORD_OFFSET,
            promise_record_local,
            function,
        );
        let completion_kind = self.emit_load_async_disposable_stack_dispose_completion_kind(
            dispose_state_local,
            function,
        );
        self.emit_async_disposable_stack_dispose_completion_kind_is(
            completion_kind,
            AsyncDisposableStackDisposeCompletionKind::Normal,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_settle_undefined(
            promise_record_local,
            PromiseSettlement::Fulfill,
            function,
        )?;
        function.instruction(&Instruction::Else);
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_TAG_OFFSET,
            error_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_PAYLOAD_OFFSET,
            error_payload_local,
            function,
        );
        self.emit_settle_promise_record(
            promise_record_local,
            PromiseSettlement::Reject,
            error_payload_local,
            error_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);

        for local in [
            promise_record_local,
            callback_tag_local,
            rejected_payload_local,
            fulfilled_payload_local,
            throwaway_promise_tag_local,
            throwaway_promise_payload_local,
            throwaway_capability_local,
            promise_constructor_tag_local,
            promise_constructor_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            error_tag_local,
            error_payload_local,
            awaited_tag_local,
            awaited_payload_local,
            method_tag_local,
            method_payload_local,
            value_tag_local,
            value_payload_local,
            kind_local,
            entry_local,
            entries_ptr_local,
            index_local,
            suspended_local,
        ] {
            self.release_temp_local(local);
        }
        Ok(())
    }

    /// Fold one error into the walk's threaded completion.
    ///
    /// The *new* error becomes `[[Error]]` and the error already carried becomes
    /// `[[Suppressed]]`, which is `DisposeResources` step 1.b.i: walking three
    /// throwing disposers in reverse order therefore yields
    /// `SuppressedError { error: first-registered, suppressed: SuppressedError { … } }`,
    /// the exact nesting
    /// `rejects-with-suppressederror-if-multiple-errors-during-disposal.js`
    /// asserts. A single error is stored unwrapped, which is the other half of
    /// the pair (`rejects-with-error-as-is-if-only-one-error-during-disposal.js`).
    fn emit_async_disposable_stack_record_error(
        &mut self,
        dispose_state_local: u32,
        error_payload_local: u32,
        error_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let suppressed_payload_local = self.reserve_temp_local();
        let suppressed_tag_local = self.reserve_temp_local();
        let prototype_local = self.reserve_temp_local();
        let combined_payload_local = self.reserve_temp_local();
        let combined_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(error_payload_local));
        function.instruction(&Instruction::LocalSet(combined_payload_local));
        function.instruction(&Instruction::LocalGet(error_tag_local));
        function.instruction(&Instruction::LocalSet(combined_tag_local));
        let completion_kind = self.emit_load_async_disposable_stack_dispose_completion_kind(
            dispose_state_local,
            function,
        );
        self.emit_async_disposable_stack_dispose_completion_kind_is(
            completion_kind,
            AsyncDisposableStackDisposeCompletionKind::Throw,
            function,
        );
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_TAG_OFFSET,
            suppressed_tag_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_PAYLOAD_OFFSET,
            suppressed_payload_local,
            function,
        );
        function.instruction(&Instruction::GlobalGet(
            SUPPRESSED_ERROR_PROTOTYPE_GLOBAL_INDEX,
        ));
        function.instruction(&Instruction::LocalSet(prototype_local));
        self.emit_alloc_suppressed_error_instance_from_locals(
            None,
            error_payload_local,
            error_tag_local,
            suppressed_payload_local,
            suppressed_tag_local,
            prototype_local,
            combined_payload_local,
            combined_tag_local,
            function,
        )?;
        function.instruction(&Instruction::End);

        self.store_i64_local_at_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_TAG_OFFSET,
            combined_tag_local,
            function,
        );
        self.store_i64_local_at_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_ERROR_PAYLOAD_OFFSET,
            combined_payload_local,
            function,
        );
        self.emit_store_async_disposable_stack_dispose_completion_kind(
            dispose_state_local,
            AsyncDisposableStackDisposeCompletionKind::Throw,
            function,
        );

        self.release_temp_local(combined_tag_local);
        self.release_temp_local(combined_payload_local);
        self.release_temp_local(prototype_local);
        self.release_temp_local(suppressed_tag_local);
        self.release_temp_local(suppressed_payload_local);
        Ok(())
    }

    fn emit_store_async_disposable_stack_dispose_completion_kind(
        &mut self,
        dispose_state_local: u32,
        completion_kind: AsyncDisposableStackDisposeCompletionKind,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_COMPLETION_KIND_OFFSET,
            completion_kind.word(),
            function,
        );
    }

    fn emit_load_async_disposable_stack_dispose_completion_kind(
        &mut self,
        dispose_state_local: u32,
        function: &mut Function,
    ) -> AsyncDisposableStackDisposeCompletionKindLocal {
        let completion_kind_local = self.reserve_temp_local();
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_COMPLETION_KIND_OFFSET,
            completion_kind_local,
            function,
        );
        for completion_kind in [
            AsyncDisposableStackDisposeCompletionKind::Normal,
            AsyncDisposableStackDisposeCompletionKind::Throw,
        ] {
            function.instruction(&Instruction::LocalGet(completion_kind_local));
            function.instruction(&Instruction::I64Const(completion_kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
        }
        function.instruction(&Instruction::I32Or);
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::Unreachable);
        function.instruction(&Instruction::End);
        AsyncDisposableStackDisposeCompletionKindLocal(completion_kind_local)
    }

    fn emit_async_disposable_stack_dispose_completion_kind_is(
        &mut self,
        completion_kind_local: AsyncDisposableStackDisposeCompletionKindLocal,
        expected: AsyncDisposableStackDisposeCompletionKind,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(completion_kind_local.0));
        function.instruction(&Instruction::I64Const(expected.word() as i64));
        function.instruction(&Instruction::I64Eq);
        self.release_temp_local(completion_kind_local.0);
    }

    /// `GetMethod(V, key)` (7.3.11) written into `method_*`.
    ///
    /// `undefined` and `null` both yield `undefined`; anything else that is not
    /// callable is a TypeError, which is why
    /// `use/throws-if-value-Symbol.asyncDispose-property-not-callable.js` must
    /// *not* fall through to the `@@dispose` lookup.
    fn emit_async_disposable_stack_get_method(
        &mut self,
        value_payload_local: u32,
        value_tag_local: u32,
        key_local: u32,
        method_payload_local: u32,
        method_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_object_read_without_throw_propagation(
            value_payload_local,
            value_tag_local,
            value_payload_local,
            value_tag_local,
            key_local,
            method_payload_local,
            method_tag_local,
            function,
        )?;
        self.emit_return_current_completion_if_throw(function);
        self.emit_async_disposable_stack_is_nullish_i32(method_tag_local, function);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(method_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(method_tag_local));
        function.instruction(&Instruction::Else);
        self.emit_is_callable_i32(method_tag_local, method_payload_local, function)?;
        function.instruction(&Instruction::I32Eqz);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_type_error(
            "AsyncDisposableStack.prototype.use dispose method is not callable",
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        Ok(())
    }

    /// Append one `DisposableResource` to the receiver's
    /// `[[DisposableResourceStack]]`, doubling the backing allocation when it is
    /// full. Copied from `emit_finalization_registry_register`'s growth loop.
    fn emit_async_disposable_stack_push_entry(
        &mut self,
        stack_record_local: u32,
        kind_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        method_payload_local: u32,
        method_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let entries_ptr_local = self.reserve_temp_local();
        let entries_len_local = self.reserve_temp_local();
        let entries_cap_local = self.reserve_temp_local();
        let new_cap_local = self.reserve_temp_local();
        let allocation_size_local = self.reserve_temp_local();
        let new_entries_ptr_local = self.reserve_temp_local();
        let index_local = self.reserve_temp_local();
        let old_entry_local = self.reserve_temp_local();
        let new_entry_local = self.reserve_temp_local();
        let copied_value_local = self.reserve_temp_local();
        let entry_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            entries_ptr_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );
        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            entries_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(entries_cap_local));
        function.instruction(&Instruction::I64Const(2));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(new_cap_local));
        function.instruction(&Instruction::LocalGet(new_cap_local));
        function.instruction(&Instruction::I64Const(
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::LocalSet(allocation_size_local));
        self.emit_heap_alloc_from_local(allocation_size_local, function)?;
        function.instruction(&Instruction::LocalSet(new_entries_ptr_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Block(BlockType::Empty));
        function.instruction(&Instruction::Loop(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64GeU);
        function.instruction(&Instruction::BrIf(1));
        for (entries_base_local, indexed_entry_local) in [
            (entries_ptr_local, old_entry_local),
            (new_entries_ptr_local, new_entry_local),
        ] {
            function.instruction(&Instruction::LocalGet(entries_base_local));
            function.instruction(&Instruction::LocalGet(index_local));
            function.instruction(&Instruction::I64Const(
                HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE as i64,
            ));
            function.instruction(&Instruction::I64Mul);
            function.instruction(&Instruction::I64Add);
            function.instruction(&Instruction::LocalSet(indexed_entry_local));
        }
        for offset in [
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
        ] {
            self.load_i64_to_local_from_offset(
                old_entry_local,
                offset,
                copied_value_local,
                function,
            );
            self.store_i64_local_at_offset(new_entry_local, offset, copied_value_local, function);
        }
        function.instruction(&Instruction::LocalGet(index_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(index_local));
        function.instruction(&Instruction::Br(0));
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::End);
        self.store_i64_local_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            new_entries_ptr_local,
            function,
        );
        self.store_i64_local_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            new_cap_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(new_entries_ptr_local));
        function.instruction(&Instruction::LocalSet(entries_ptr_local));
        function.instruction(&Instruction::End);

        function.instruction(&Instruction::LocalGet(entries_ptr_local));
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_SIZE as i64,
        ));
        function.instruction(&Instruction::I64Mul);
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entry_local));
        for (offset, value_local) in [
            (HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_KIND_OFFSET, kind_local),
            (
                HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_TAG_OFFSET,
                value_tag_local,
            ),
            (
                HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_VALUE_PAYLOAD_OFFSET,
                value_payload_local,
            ),
            (
                HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_TAG_OFFSET,
                method_tag_local,
            ),
            (
                HEAP_ASYNC_DISPOSABLE_STACK_ENTRY_METHOD_PAYLOAD_OFFSET,
                method_payload_local,
            ),
        ] {
            self.store_i64_local_at_offset(entry_local, offset, value_local, function);
        }
        function.instruction(&Instruction::LocalGet(entries_len_local));
        function.instruction(&Instruction::I64Const(1));
        function.instruction(&Instruction::I64Add);
        function.instruction(&Instruction::LocalSet(entries_len_local));
        self.store_i64_local_at_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            entries_len_local,
            function,
        );

        self.release_temp_local(entry_local);
        self.release_temp_local(copied_value_local);
        self.release_temp_local(new_entry_local);
        self.release_temp_local(old_entry_local);
        self.release_temp_local(index_local);
        self.release_temp_local(new_entries_ptr_local);
        self.release_temp_local(allocation_size_local);
        self.release_temp_local(new_cap_local);
        self.release_temp_local(entries_cap_local);
        self.release_temp_local(entries_len_local);
        self.release_temp_local(entries_ptr_local);
        Ok(())
    }

    /// The `[[AsyncDisposableState]]` brand check shared by every synchronous
    /// member. `disposeAsync` deliberately does *not* use it: it must reject
    /// rather than throw.
    fn emit_async_disposable_stack_record_from_receiver(
        &mut self,
        stack_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let receiver_payload_local = self.reserve_temp_local();
        let receiver_tag_local = self.reserve_temp_local();
        let receiver_brand_local = self.reserve_temp_local();

        self.compile_this_to_locals(receiver_payload_local, receiver_tag_local, function)?;
        function.instruction(&Instruction::LocalGet(receiver_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_type_error(
            "AsyncDisposableStack method receiver is not an object",
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_ASYNC_DISPOSABLE_STACK as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_async_disposable_stack_type_error(
            "AsyncDisposableStack method receiver does not have [[AsyncDisposableState]]",
            function,
        )?;
        function.instruction(&Instruction::End);
        self.load_i64_to_local_from_offset(
            receiver_payload_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            stack_record_local,
            function,
        );

        self.release_temp_local(receiver_brand_local);
        self.release_temp_local(receiver_tag_local);
        self.release_temp_local(receiver_payload_local);
        Ok(())
    }

    /// `If asyncDisposableStack.[[AsyncDisposableState]] is disposed, throw a
    /// **ReferenceError**` — not a TypeError. Every
    /// `prototype/*/throws-if-disposed.js` names `ReferenceError` explicitly.
    fn emit_async_disposable_stack_require_pending(
        &mut self,
        stack_record_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let state_local = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            stack_record_local,
            HEAP_ASYNC_DISPOSABLE_STACK_STATE_OFFSET,
            state_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(state_local));
        function.instruction(&Instruction::I64Const(
            AsyncDisposableStackState::Disposed.word() as i64,
        ));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_error(
            REFERENCE_ERROR_NAME,
            "AsyncDisposableStack is already disposed",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        self.release_temp_local(state_local);
        Ok(())
    }

    fn emit_async_disposable_stack_is_nullish_i32(
        &mut self,
        tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::LocalGet(tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Null.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::I32Or);
    }

    fn emit_async_disposable_stack_restore_realm_env(
        &mut self,
        dispose_state_local: u32,
        function: &mut Function,
    ) {
        self.load_i64_to_local_from_offset(
            dispose_state_local,
            ASYNC_DISPOSABLE_STACK_DISPOSE_REALM_ENV_OFFSET,
            self.current_env_local,
            function,
        );
    }

    fn emit_async_disposable_stack_settle_undefined(
        &mut self,
        promise_record_local: u32,
        settlement: PromiseSettlement,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        self.emit_settle_promise_record(
            promise_record_local,
            settlement,
            undefined_payload_local,
            undefined_tag_local,
            function,
        )?;

        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        Ok(())
    }

    fn emit_async_disposable_stack_reject_current_throw_and_return(
        &mut self,
        promise_record_local: u32,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let error_payload_local = self.reserve_temp_local();
        let error_tag_local = self.reserve_temp_local();

        function.instruction(&Instruction::LocalGet(self.completion_local));
        function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        function.instruction(&Instruction::LocalGet(self.result_local));
        function.instruction(&Instruction::LocalSet(error_payload_local));
        function.instruction(&Instruction::LocalGet(self.result_tag_local));
        function.instruction(&Instruction::LocalSet(error_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_settle_promise_record(
            promise_record_local,
            PromiseSettlement::Reject,
            error_payload_local,
            error_tag_local,
            function,
        )?;
        self.emit_async_disposable_stack_return_promise(
            promise_payload_local,
            promise_tag_local,
            function,
        );
        function.instruction(&Instruction::End);

        self.release_temp_local(error_tag_local);
        self.release_temp_local(error_payload_local);
        Ok(())
    }

    fn emit_async_disposable_stack_return_promise(
        &mut self,
        promise_payload_local: u32,
        promise_tag_local: u32,
        function: &mut Function,
    ) {
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
        self.emit_return_current_completion(function);
    }

    fn emit_async_disposable_stack_return_undefined(&mut self, function: &mut Function) {
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.set_completion_kind(CompletionKind::Normal, function);
    }

    fn emit_async_disposable_stack_type_error(
        &mut self,
        message: &'static str,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        self.emit_throw_current_function_realm_type_error(
            message,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        Ok(())
    }
}
