//! Constructor-only `%DisposableStack%` foundation.
//!
//! Real synchronous disposal members intentionally do not exist in this
//! module. See `docs/rust-rewrite/contracts/disposable-stack-construction-brand.md`.

use super::super::*;
use crate::functions::NewTargetPrototypeFallback;

/// A fully initialized, still-unpublished synchronous stack record.
///
/// Private fields and the consuming finalizer keep the sync brand, boxed
/// record and Object result publication in one operation. This is deliberately
/// non-`Copy`: constructor emission cannot duplicate or silently reuse the
/// record after publication.
#[must_use = "a pending DisposableStack record must be consumed by the instance finalizer"]
struct PendingDisposableStackRecordLocal(u32);

impl<'a> FunctionBuilder<'a> {
    /// `DisposableStack ( )`.
    pub(crate) fn emit_disposable_stack_constructor(
        &mut self,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let new_target_payload_local = self.reserve_temp_local();
        let new_target_tag_local = self.reserve_temp_local();
        let prototype_payload_local = self.reserve_temp_local();
        let prototype_tag_local = self.reserve_temp_local();
        let stack_object_local = self.reserve_temp_local();

        self.compile_new_target_to_locals(
            new_target_payload_local,
            new_target_tag_local,
            function,
        )?;
        function.instruction(&Instruction::LocalGet(new_target_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "DisposableStack constructor requires new",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);

        // This helper owns the sole observable `Get(NewTarget, "prototype")`.
        // Dynamic cross-realm Function construction remains outside the AOT
        // boundary, so primitive results use the current intrinsic global.
        self.emit_new_target_prototype_to_locals(
            DISPOSABLE_STACK_PROTOTYPE_GLOBAL_INDEX,
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
        function.instruction(&Instruction::LocalSet(stack_object_local));

        let pending_record = self.emit_alloc_pending_disposable_stack_record(function)?;
        self.emit_finalize_disposable_stack_instance(stack_object_local, pending_record, function);

        self.release_temp_local(stack_object_local);
        self.release_temp_local(prototype_tag_local);
        self.release_temp_local(prototype_payload_local);
        self.release_temp_local(new_target_tag_local);
        self.release_temp_local(new_target_payload_local);
        Ok(())
    }

    fn emit_alloc_pending_disposable_stack_record(
        &mut self,
        function: &mut Function,
    ) -> Result<PendingDisposableStackRecordLocal, EmitError> {
        let record_local = self.reserve_temp_local();

        self.emit_heap_alloc_const(HEAP_DISPOSABLE_STACK_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(record_local));
        self.store_i64_const_at_offset(
            record_local,
            HEAP_DISPOSABLE_STACK_STATE_OFFSET,
            DISPOSABLE_STACK_PENDING_STATE_WORD,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            0,
            function,
        );
        self.store_i64_const_at_offset(
            record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            0,
            function,
        );

        Ok(PendingDisposableStackRecordLocal(record_local))
    }

    fn emit_finalize_disposable_stack_instance(
        &mut self,
        object_local: u32,
        record: PendingDisposableStackRecordLocal,
        function: &mut Function,
    ) {
        self.store_i64_const_at_offset(
            object_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            OBJECT_INTERNAL_BRAND_DISPOSABLE_STACK,
            function,
        );
        self.store_i64_local_at_offset(
            object_local,
            HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
            record.0,
            function,
        );
        function.instruction(&Instruction::LocalGet(object_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));

        self.release_temp_local(record.0);
    }
}
