use super::*;

#[must_use = "the detached List must be installed exactly once"]
pub(super) struct TransferredDisposableStackCapabilityLocals {
    resources: GcLocal<DisposableResourceStack>,
}

impl FunctionBuilder<'_> {
    pub(super) fn emit_take_disposable_stack_capability(
        &mut self,
        source: &PendingDisposableStack<'_>,
        f: &mut Function,
    ) -> TransferredDisposableStackCapabilityLocals {
        let s = self.runtime_schema();
        let resources = s.reserve_gc_local(f).initialize(
            s.struct_type::<DisposableStack>()
                .field(DisposableStackSchema::RESOURCE_STACK)
                .read(source.0, s, f)
                .reference(),
            f,
        );
        let empty = self.emit_empty_disposable_resource_stack(f);
        s.struct_type::<DisposableStack>()
            .field(DisposableStackSchema::RESOURCE_STACK)
            .write(source.0, GcOperand::reference(&empty, s), s, f);
        s.struct_type::<DisposableStack>()
            .field(DisposableStackSchema::STATE)
            .write(
                source.0,
                GcOperand::constant(DisposableStackState::Disposed),
                s,
                f,
            );
        empty.clear(f);
        TransferredDisposableStackCapabilityLocals { resources }
    }

    pub(super) fn emit_install_transferred_disposable_stack_capability(
        &mut self,
        pending: PendingDisposableStackRecordLocal,
        transfer: TransferredDisposableStackCapabilityLocals,
        f: &mut Function,
    ) -> PendingDisposableStackRecordLocal {
        pending.resources.clear(f);
        PendingDisposableStackRecordLocal {
            header: pending.header,
            resources: transfer.resources,
        }
    }
}
