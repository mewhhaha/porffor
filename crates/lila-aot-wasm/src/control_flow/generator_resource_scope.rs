//! One checked lexical resource scope owns all of its dynamic registrations.

use super::*;
use lila_ir::{
    AsyncGeneratorForOfIr, AsyncGeneratorResourceCapabilityIr,
    AsyncGeneratorResourceRegistrationIr, AsyncGeneratorResourceScopeIr, ResourceDisposalHintIr,
};

/// The complete lexical lifetime comes only from an actual checked owner.
/// Initialization and body remain the original payloads of that owner.
#[derive(Clone, Copy)]
enum MixedResourceLifetimeOwner<'a> {
    Scope(&'a AsyncGeneratorResourceScopeIr),
    ForOf {
        plan: &'a AsyncGeneratorForOfIr,
        resource: &'a lila_ir::AsyncGeneratorForOfResourceIr,
    },
    Loop(&'a lila_ir::AsyncGeneratorScopedResourceIr),
    Switch(&'a lila_ir::AsyncGeneratorScopedResourceIr),
}

#[derive(Clone, Copy)]
pub(super) struct CompleteMixedResourceLifetime<'a>(MixedResourceLifetimeOwner<'a>);

impl<'a> CompleteMixedResourceLifetime<'a> {
    fn execution(self) -> FunctionExecutionKind {
        let execution = match self.0 {
            MixedResourceLifetimeOwner::ForOf { plan, .. } => plan.execution(),
            MixedResourceLifetimeOwner::Scope(plan) => plan.execution(),
            MixedResourceLifetimeOwner::Loop(resource)
            | MixedResourceLifetimeOwner::Switch(resource) => resource.execution(),
        };
        match execution {
            lila_ir::ResumableRegionProtocolIr::Generator => FunctionExecutionKind::Generator,
            lila_ir::ResumableRegionProtocolIr::Async => FunctionExecutionKind::Async,
            lila_ir::ResumableRegionProtocolIr::AsyncGenerator => {
                FunctionExecutionKind::AsyncGenerator
            }
        }
    }
    pub(super) fn for_for_of(plan: &'a AsyncGeneratorForOfIr) -> Option<Self> {
        plan.resource()
            .map(|resource| Self(MixedResourceLifetimeOwner::ForOf { plan, resource }))
    }

    pub(super) fn for_loop(plan: &'a lila_ir::AsyncGeneratorLoopIr) -> Option<Self> {
        plan.resource()
            .map(|resource| Self(MixedResourceLifetimeOwner::Loop(resource)))
    }

    pub(super) fn for_switch(plan: &'a lila_ir::AsyncGeneratorSwitchIr) -> Option<Self> {
        plan.resource()
            .map(|resource| Self(MixedResourceLifetimeOwner::Switch(resource)))
    }

    fn binding(self) -> &'a OwnedEnvBindingIr {
        match self.0 {
            MixedResourceLifetimeOwner::Scope(plan) => plan.capability_binding(),
            MixedResourceLifetimeOwner::ForOf { resource, .. } => resource.capability_binding(),
            MixedResourceLifetimeOwner::Loop(resource)
            | MixedResourceLifetimeOwner::Switch(resource) => resource.capability_binding(),
        }
    }

    fn capability(self) -> &'a AsyncGeneratorResourceCapabilityIr {
        match self.0 {
            MixedResourceLifetimeOwner::Scope(plan) => plan.capability(),
            MixedResourceLifetimeOwner::ForOf { resource, .. } => resource.capability(),
            MixedResourceLifetimeOwner::Loop(resource)
            | MixedResourceLifetimeOwner::Switch(resource) => resource.capability(),
        }
    }

    fn entry_state(self) -> u32 {
        match self.0 {
            MixedResourceLifetimeOwner::Scope(plan) => plan.entry_state(),
            MixedResourceLifetimeOwner::ForOf { plan, .. } => plan.initialization().entry_state(),
            MixedResourceLifetimeOwner::Loop(resource)
            | MixedResourceLifetimeOwner::Switch(resource) => resource.entry_state(),
        }
    }

    fn body_end_state(self) -> u32 {
        match self.0 {
            MixedResourceLifetimeOwner::Scope(plan) => plan.body().end_state(),
            MixedResourceLifetimeOwner::ForOf { plan, .. } => plan.body().end_state(),
            MixedResourceLifetimeOwner::Loop(resource)
            | MixedResourceLifetimeOwner::Switch(resource) => resource.body_end_state(),
        }
    }

    fn exit_state(self) -> u32 {
        match self.0 {
            MixedResourceLifetimeOwner::Scope(plan) => plan.exit_state(),
            MixedResourceLifetimeOwner::ForOf { resource, .. } => resource.exit_state(),
            MixedResourceLifetimeOwner::Loop(resource)
            | MixedResourceLifetimeOwner::Switch(resource) => resource.exit_state(),
        }
    }

    fn capacity(self) -> usize {
        match self.0 {
            MixedResourceLifetimeOwner::Scope(plan) => plan.capacity(),
            MixedResourceLifetimeOwner::ForOf { .. } => 1,
            MixedResourceLifetimeOwner::Loop(resource)
            | MixedResourceLifetimeOwner::Switch(resource) => resource.capacity(),
        }
    }
}

#[derive(Clone, Copy)]
enum ResourceNativeCapability {
    Sync,
    Async,
}

/// The table kind and original invocation cell come only from a whole checked
/// source scope. A registration hint cannot choose its own capability model.
#[derive(Clone)]
pub(crate) struct CheckedAsyncGeneratorResourceOwner {
    binding: OwnedEnvBindingIr,
    kind: ResourceNativeCapability,
}

impl CheckedAsyncGeneratorResourceOwner {
    fn for_lifetime(plan: CompleteMixedResourceLifetime<'_>) -> Self {
        Self {
            binding: plan.binding().clone(),
            kind: match plan.capability() {
                AsyncGeneratorResourceCapabilityIr::Sync(_) => ResourceNativeCapability::Sync,
                AsyncGeneratorResourceCapabilityIr::Async(_) => ResourceNativeCapability::Async,
            },
        }
    }
}

impl FunctionBuilder<'_> {
    pub(super) fn compile_async_generator_resource_scope(
        &mut self,
        plan: &AsyncGeneratorResourceScopeIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let previous_environment_owner = self.checked_async_generator_environment_owner;
        if plan.execution() == lila_ir::ResumableRegionProtocolIr::AsyncGenerator {
            self.checked_async_generator_environment_owner = Some(
                CheckedAsyncGeneratorEnvironmentOwner::for_resource_scope(plan),
            );
        }
        let result = self.compile_complete_mixed_resource_lifetime_then(
            CompleteMixedResourceLifetime(MixedResourceLifetimeOwner::Scope(plan)),
            function,
            |builder, _, function| {
                builder.compile_resumable_block_contents(
                    plan.body().block(),
                    plan.entry_state(),
                    true,
                    function,
                )
            },
        );
        self.checked_async_generator_environment_owner = previous_environment_owner;
        result
    }

    pub(super) fn compile_complete_mixed_resource_lifetime_then(
        &mut self,
        plan: CompleteMixedResourceLifetime<'_>,
        function: &mut Function,
        body: impl FnOnce(&mut Self, ControlTarget, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        let previous = self.checked_async_generator_resource_owner.take();
        self.checked_async_generator_resource_owner =
            Some(CheckedAsyncGeneratorResourceOwner::for_lifetime(plan));
        let result = self.emit_complete_mixed_resource_lifetime(plan, function, body);
        self.checked_async_generator_resource_owner = previous;
        result
    }

    fn emit_complete_mixed_resource_lifetime(
        &mut self,
        plan: CompleteMixedResourceLifetime<'_>,
        function: &mut Function,
        body: impl FnOnce(&mut Self, ControlTarget, &mut Function) -> Result<(), EmitError>,
    ) -> Result<(), EmitError> {
        if !self
            .current_function_meta()
            .is_some_and(|meta| meta.protocol().execution_kind() == plan.execution())
            || !self
                .owned_env_bindings
                .iter()
                .any(|binding| binding == plan.binding())
        {
            return Err(EmitError::unsupported(
                "compiler invariant: whole resource scope requires its exact checked activation",
            ));
        }
        let binding = self
            .activation_owned_binding_storage(&plan.binding().name)
            .ok_or_else(|| {
                EmitError::unsupported("whole resource scope lost its invocation cell")
            })?;
        self.emit_resumable_state_in_range(plan.entry_state(), plan.exit_state(), false, function)?;
        self.open_frame(ControlFrameKind::If, function);
        self.push_scope();
        let outer = self.open_frame(ControlFrameKind::Block, function);
        let disposal = self.open_frame(ControlFrameKind::Block, function);
        self.finally_stack.push(disposal);
        self.emit_resumable_state_in_range(
            plan.entry_state(),
            plan.body_end_state(),
            true,
            function,
        )?;
        self.open_frame(ControlFrameKind::If, function);
        self.emit_resumable_state_equals(plan.entry_state(), function)?;
        self.open_frame(ControlFrameKind::If, function);
        match plan.capability() {
            AsyncGeneratorResourceCapabilityIr::Sync(_) => {
                let capability = self.initialize_empty_activation_sync_dispose_capability(
                    &ActivationSyncDisposeCapabilityStorage { binding },
                    plan.capacity(),
                    function,
                )?;
                capability.entries.clear(function);
                capability.record.clear(function);
            }
            AsyncGeneratorResourceCapabilityIr::Async(_) => {
                let capability = self.initialize_empty_activation_async_dispose_capability(
                    &ActivationAsyncDisposeCapabilityStorage { binding },
                    plan.capacity(),
                    function,
                )?;
                self.release_active_activation_async_dispose_capability(capability, function);
            }
        }
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        // The callback is the owner's original initialization/body payload.
        // Its exact resumable tape skips already registered entries.
        body(self, disposal, function)?;
        self.emit_branch_to_target(disposal, function);
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        self.finally_stack.pop();
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        match plan.capability() {
            AsyncGeneratorResourceCapabilityIr::Sync(_) => {
                let detached = self.detach_activation_sync_dispose_capability(
                    ActivationSyncDisposeCapabilityStorage { binding },
                    function,
                )?;
                let resources = self.load_detached_activation_sync_disposable_resources(
                    &detached,
                    plan.capacity(),
                    function,
                );
                self.release_detached_activation_sync_dispose_capability(detached, function);
                let pending = self.capture_pending_sync_dispose_completion(function);
                self.set_completion_kind(CompletionKind::Normal, function);
                self.emit_set_resumable_resume_point(plan.exit_state(), function)?;
                self.emit_save_resumable_environment(function)?;
                self.consume_sync_disposable_resources(
                    pending,
                    resources,
                    SyncDisposeCompletionContinuation::Dispatch,
                    function,
                )?;
            }
            AsyncGeneratorResourceCapabilityIr::Async(capability) => {
                let finalizer = capability.finalizer();
                self.emit_resumable_finalizer_needs_pending_completion(
                    finalizer.dispose_state(),
                    function,
                )?;
                self.open_frame(ControlFrameKind::If, function);
                let pending = self.begin_async_dispose_pending_completion(function)?;
                self.set_completion_kind(CompletionKind::Normal, function);
                let disposing = self.begin_activation_async_dispose_capability(
                    ActivationAsyncDisposeCapabilityStorage { binding },
                    finalizer,
                    function,
                )?;
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                let owner=match plan.execution() {
                    FunctionExecutionKind::Async=>ActivationAsyncDisposeOwner::AsyncCompleteIterator(capability),
                    FunctionExecutionKind::AsyncGenerator=>ActivationAsyncDisposeOwner::AsyncGenerator(capability),
                    FunctionExecutionKind::Ordinary|FunctionExecutionKind::Generator=>return Err(EmitError::unsupported("awaited resource disposal requires the checked async iterator protocol")),
                };
                self.consume_activation_async_dispose_capability(
                    &owner,
                    disposing,
                    pending,
                    finalizer,
                    ActivationAsyncDisposeCompletionContinuation::CompleteMixedScope,
                    function,
                )?;
            }
        }
        self.pop_control(ControlFrameKind::Block);
        function.instruction(&Instruction::End);
        let _ = outer;
        self.pop_scope();
        self.pop_control(ControlFrameKind::If);
        function.instruction(&Instruction::End);
        Ok(())
    }

    pub(super) fn compile_async_generator_resource_registration(
        &mut self,
        registration: &AsyncGeneratorResourceRegistrationIr,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let owner = self
            .checked_async_generator_resource_owner
            .as_ref()
            .filter(|owner| &owner.binding == registration.capability_binding())
            .cloned()
            .ok_or_else(|| {
                EmitError::unsupported(
                "compiler invariant: resource registration requires its exact enclosing capability",
            )
            })?;
        let capability_binding = self
            .activation_owned_binding_storage(&owner.binding.name)
            .ok_or_else(|| {
                EmitError::unsupported("resource registration lost its invocation cell")
            })?;
        let target = self
            .lookup_current_scope_binding(registration.binding_name())
            .or_else(|| self.lookup_binding(registration.binding_name()))
            .or_else(|| self.activation_owned_binding_storage(registration.binding_name()))
            .ok_or_else(|| {
                EmitError::unsupported("resource registration lost its original const binding")
            })?;
        let saved = self.save_statement_list_value(function);
        match (owner.kind, registration.hint()) {
            (ResourceNativeCapability::Sync, ResourceDisposalHintIr::Sync) => {
                let capability = self.load_activation_sync_dispose_capability(
                    &ActivationSyncDisposeCapabilityStorage {
                        binding: capability_binding,
                    },
                    function,
                )?;
                let acquired = self.reserve_sync_disposable_resource_locals(function);
                self.compile_expr_to_value(registration.initializer(), &acquired.value, function)?;
                self.emit_propagate_current_throw_if_needed(function);
                self.acquire_sync_disposable_resource_from_locals(&acquired, function)?;
                self.append_activation_sync_disposable_resource(&capability, &acquired, function);
                self.write_binding_from_locals(target, &acquired.value, function);
                self.release_sync_disposable_resource_locals(acquired, function);
                capability.entries.clear(function);
                capability.record.clear(function);
            }
            (ResourceNativeCapability::Async, ResourceDisposalHintIr::Async) => {
                let capability = self.load_activation_async_dispose_capability(
                    &ActivationAsyncDisposeCapabilityStorage {
                        binding: capability_binding,
                    },
                    function,
                )?;
                let acquired = self.reserve_async_disposable_resource_locals(function);
                self.compile_expr_to_value(registration.initializer(), &acquired.value, function)?;
                self.emit_propagate_current_throw_if_needed(function);
                self.acquire_async_disposable_resource_from_locals(&acquired, function)?;
                self.append_activation_async_disposable_resource(&capability, &acquired, function);
                self.write_binding_from_locals(target, &acquired.value, function);
                self.release_async_disposable_resource_locals(acquired, function);
                self.release_active_activation_async_dispose_capability(capability, function);
            }
            (ResourceNativeCapability::Async, ResourceDisposalHintIr::Sync) => {
                let capability = self.load_activation_async_dispose_capability(
                    &ActivationAsyncDisposeCapabilityStorage {
                        binding: capability_binding,
                    },
                    function,
                )?;
                let acquired = self.reserve_sync_disposable_resource_locals(function);
                self.compile_expr_to_value(registration.initializer(), &acquired.value, function)?;
                self.emit_propagate_current_throw_if_needed(function);
                self.acquire_sync_disposable_resource_from_locals(&acquired, function)?;
                acquired.registered.load(function);
                self.open_frame(ControlFrameKind::If, function);
                let mixed = self.reserve_async_disposable_resource_locals(function);
                mixed
                    .kind
                    .set_constant(ActivationAsyncDisposeEntryKind::SyncMethod, function);
                mixed.value.copy_from(&acquired.value, function);
                mixed.method.copy_from(&acquired.method, function);
                self.append_activation_async_disposable_resource(&capability, &mixed, function);
                self.release_async_disposable_resource_locals(mixed, function);
                self.pop_control(ControlFrameKind::If);
                function.instruction(&Instruction::End);
                self.write_binding_from_locals(target, &acquired.value, function);
                self.release_sync_disposable_resource_locals(acquired, function);
                self.release_active_activation_async_dispose_capability(capability, function);
            }
            (ResourceNativeCapability::Sync, ResourceDisposalHintIr::Async) => {
                return Err(EmitError::unsupported(
                    "compiler invariant: async registration in a synchronous capability",
                ))
            }
        }
        self.restore_statement_list_value(saved, function)
    }
}
