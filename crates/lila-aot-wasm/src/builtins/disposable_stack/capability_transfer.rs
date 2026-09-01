use super::*;

#[must_use = "a transferred DisposableStack capability must be installed exactly once"]
pub(super) struct TransferredDisposableStackCapabilityLocals {
    entries_ptr: u32,
    entries_len: u32,
    entries_cap: u32,
}

impl<'a> FunctionBuilder<'a> {
    pub(super) fn emit_take_disposable_stack_capability(
        &mut self,
        source_record_local: u32,
        function: &mut Function,
    ) -> TransferredDisposableStackCapabilityLocals {
        let entries_ptr = self.reserve_temp_local();
        let entries_len = self.reserve_temp_local();
        let entries_cap = self.reserve_temp_local();

        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            entries_ptr,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            entries_len,
            function,
        );
        self.load_i64_to_local_from_offset(
            source_record_local,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
            entries_cap,
            function,
        );
        for offset in [
            HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
            HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
        ] {
            self.store_i64_const_at_offset(source_record_local, offset, 0, function);
        }
        self.store_i64_const_at_offset(
            source_record_local,
            HEAP_DISPOSABLE_STACK_STATE_OFFSET,
            DisposableStackState::Disposed.word(),
            function,
        );

        TransferredDisposableStackCapabilityLocals {
            entries_ptr,
            entries_len,
            entries_cap,
        }
    }

    pub(super) fn emit_install_transferred_disposable_stack_capability(
        &mut self,
        record: PendingDisposableStackRecordLocal,
        transfer: TransferredDisposableStackCapabilityLocals,
        function: &mut Function,
    ) -> PendingDisposableStackRecordLocal {
        for (offset, local) in [
            (
                HEAP_DISPOSABLE_STACK_ENTRIES_PTR_OFFSET,
                transfer.entries_ptr,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRIES_LEN_OFFSET,
                transfer.entries_len,
            ),
            (
                HEAP_DISPOSABLE_STACK_ENTRIES_CAP_OFFSET,
                transfer.entries_cap,
            ),
        ] {
            self.store_i64_local_at_offset(record.0, offset, local, function);
        }

        self.release_temp_local(transfer.entries_cap);
        self.release_temp_local(transfer.entries_len);
        self.release_temp_local(transfer.entries_ptr);
        record
    }
}
