//! Real collector and native byte-resource imports from the emitted GC ABI.

use super::wasm_shared_resource::{AsyncWaitRegistration, NativeSyncWaitResult};
use super::*;
use lila_aot_wasm::GcHostImport;
use wasmtime::{ExternRef, Rooted};

fn resource(
    caller: &WasmtimeCaller<'_, WasmHostState>,
    reference: Rooted<ExternRef>,
) -> wasmtime::Result<Arc<WasmSharedBufferResource>> {
    let resource = reference
        .data(caller)?
        .and_then(|data| data.downcast_ref::<Arc<WasmSharedBufferResource>>())
        .cloned()
        .ok_or_else(|| wasmtime::Error::msg("invalid shared byte resource host type"))?;
    let backing = caller
        .data()
        .shared_memory_backing
        .as_ref()
        .ok_or_else(|| {
            wasmtime::Error::msg("shared byte resource requires imported shared memory")
        })?;
    backing.check_resource(&resource)?;
    Ok(resource)
}

#[derive(Clone, Copy)]
enum SharedResourceScalar {
    Base,
    Length,
    Maximum,
}

pub(super) fn link(
    linker: &mut WasmtimeLinker<WasmHostState>,
    module: &WasmtimeModule,
) -> Result<(), EngineError> {
    let bind_error =
        |error| EngineError::new(format!("wasmtime GC host linker setup failed: {error}"));
    let import = GcHostImport::CollectGc;
    linker
        .func_wrap(
            import.module(),
            import.name(),
            |mut caller: WasmtimeCaller<'_, WasmHostState>| -> wasmtime::Result<()> {
                caller.gc(None)
            },
        )
        .map_err(bind_error)?;

    let import = GcHostImport::SharedBufferAllocate;
    linker
        .func_wrap(
            import.module(),
            import.name(),
            |mut caller: WasmtimeCaller<'_, WasmHostState>,
             initial: i64,
             maximum: i64,
             growable: i32|
             -> wasmtime::Result<Option<Rooted<ExternRef>>> {
                let initial = u64::try_from(initial)
                    .map_err(|_| wasmtime::Error::msg("negative shared-buffer length"))?;
                let maximum = u64::try_from(maximum)
                    .map_err(|_| wasmtime::Error::msg("negative shared-buffer maximum"))?;
                let growable = match growable {
                    0 => false,
                    1 => true,
                    _ => return Err(wasmtime::Error::msg("invalid shared-buffer growability")),
                };
                let backing = caller.data().shared_memory_backing.clone().ok_or_else(|| {
                    wasmtime::Error::msg("shared-buffer allocation requires imported shared memory")
                })?;
                let Some(resource) = backing.allocate(initial, maximum, growable)? else {
                    return Ok(None);
                };
                ExternRef::new(&mut caller, resource).map(Some)
            },
        )
        .map_err(bind_error)?;

    for (import, operation) in [
        (GcHostImport::SharedBufferBase, SharedResourceScalar::Base),
        (
            GcHostImport::SharedBufferLength,
            SharedResourceScalar::Length,
        ),
        (
            GcHostImport::SharedBufferMaximum,
            SharedResourceScalar::Maximum,
        ),
    ] {
        linker
            .func_wrap(
                import.module(),
                import.name(),
                move |caller: WasmtimeCaller<'_, WasmHostState>,
                      reference: Rooted<ExternRef>|
                      -> wasmtime::Result<i64> {
                    let resource = resource(&caller, reference)?;
                    let value = match operation {
                        SharedResourceScalar::Base => resource.base(),
                        SharedResourceScalar::Length => resource.length(),
                        SharedResourceScalar::Maximum => resource.maximum(),
                    };
                    i64::try_from(value)
                        .map_err(|_| wasmtime::Error::msg("shared resource scalar exceeds ABI"))
                },
            )
            .map_err(bind_error)?;
    }
    let import = GcHostImport::SharedBufferGrowable;
    linker
        .func_wrap(
            import.module(),
            import.name(),
            |caller: WasmtimeCaller<'_, WasmHostState>,
             reference: Rooted<ExternRef>|
             -> wasmtime::Result<i32> {
                Ok(i32::from(resource(&caller, reference)?.growable()))
            },
        )
        .map_err(bind_error)?;

    let import = GcHostImport::SharedBufferGrow;
    linker
        .func_wrap(
            import.module(),
            import.name(),
            |caller: WasmtimeCaller<'_, WasmHostState>,
             reference: Rooted<ExternRef>,
             length: i64|
             -> wasmtime::Result<i32> {
                let length = u64::try_from(length)
                    .map_err(|_| wasmtime::Error::msg("negative shared-buffer growth"))?;
                Ok(i32::from(resource(&caller, reference)?.grow(length)))
            },
        )
        .map_err(bind_error)?;

    let import = GcHostImport::AgentBroadcastResource;
    linker
        .func_wrap(
            import.module(),
            import.name(),
            |caller: WasmtimeCaller<'_, WasmHostState>,
             reference: Rooted<ExternRef>,
             id: i64|
             -> wasmtime::Result<i64> {
                let resource = resource(&caller, reference)?;
                let group = caller.data().agent_group.as_ref().ok_or_else(|| {
                    wasmtime::Error::msg("agent.broadcast requires an active agent group")
                })?;
                let sent = group.broadcast(WasmAgentBroadcast { resource, id });
                i64::try_from(sent).map_err(|_| wasmtime::Error::msg("agent count exceeds ABI"))
            },
        )
        .map_err(bind_error)?;

    let import = GcHostImport::AgentReceiveResource;
    linker.func_wrap(import.module(), import.name(),
        |mut caller: WasmtimeCaller<'_, WasmHostState>|
            -> wasmtime::Result<(Option<Rooted<ExternRef>>, i64)> {
            let commands = caller.data().agent_commands.clone().ok_or_else(|| {
                wasmtime::Error::msg("agent.receiveBroadcast may only run inside an agent")
            })?;
            let command = commands.lock().unwrap_or_else(|error| error.into_inner()).recv()
                .map_err(|error| wasmtime::Error::msg(format!("agent broadcast channel closed: {error}")))?;
            match command {
                WasmAgentCommand::Shutdown => Ok((None, 0)),
                WasmAgentCommand::Broadcast(broadcast) => {
                    let backing = caller.data().shared_memory_backing.as_ref().ok_or_else(|| {
                        wasmtime::Error::msg("agent resource requires imported shared memory")
                    })?;
                    backing.check_resource(&broadcast.resource)?;
                    // Native bytes cross Stores; JavaScript wrappers remain local.
                    Ok((Some(ExternRef::new(&mut caller, broadcast.resource)?), broadcast.id))
                }
            }
        }).map_err(bind_error)?;

    let import = GcHostImport::RegisterAsyncWaiter;
    linker
        .func_wrap(
            import.module(),
            import.name(),
            |caller: WasmtimeCaller<'_, WasmHostState>,
             reference: Rooted<ExternRef>,
             offset: i64,
             width: i32,
             expected_word: i64|
             -> wasmtime::Result<i64> {
                caller
                    .data()
                    .async_waiters
                    .register(resource(&caller, reference)?, offset, width, expected_word)
                    .map(AsyncWaitRegistration::wire)
            },
        )
        .map_err(bind_error)?;

    let import = GcHostImport::SharedBufferWait;
    linker
        .func_wrap(
            import.module(),
            import.name(),
            |caller: WasmtimeCaller<'_, WasmHostState>,
             reference: Rooted<ExternRef>,
             offset: i64,
             width: i32,
             expected: i64,
             timeout_nanos: i64|
             -> wasmtime::Result<i32> {
                caller
                    .data()
                    .async_waiters
                    .wait(
                        resource(&caller, reference)?,
                        offset,
                        width,
                        expected,
                        timeout_nanos,
                    )
                    .map(NativeSyncWaitResult::wire)
            },
        )
        .map_err(bind_error)?;

    let import = GcHostImport::NotifyAsyncWaiters;
    linker
        .func_wrap(
            import.module(),
            import.name(),
            |caller: WasmtimeCaller<'_, WasmHostState>,
             reference: Rooted<ExternRef>,
             offset: i64,
             width: i32,
             count: i64|
             -> wasmtime::Result<i64> {
                let resource = resource(&caller, reference)?;
                caller
                    .data()
                    .shared_memory_backing
                    .as_ref()
                    .ok_or_else(|| {
                        wasmtime::Error::msg("Atomics.notify requires imported shared memory")
                    })?
                    .notify_async_waiters(&resource, offset, width, count)
            },
        )
        .map_err(bind_error)?;
    wasm_gc_intl_host::link(linker, module)?;
    Ok(())
}
