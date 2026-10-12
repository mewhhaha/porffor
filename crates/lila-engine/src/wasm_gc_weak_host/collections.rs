use super::*;

use super::collection_index::Entries;

struct WeakMapResource(Mutex<Entries>);
struct WeakSetResource(Mutex<Entries>);

pub(super) fn link(linker: &mut WasmtimeLinker<WasmHostState>) -> Result<(), EngineError> {
    let row = GcHostImport::WeakMapAllocate;
    linker
        .func_wrap(
            row.module(),
            row.name(),
            |mut caller: WasmtimeCaller<'_, WasmHostState>| -> wasmtime::Result<Rooted<ExternRef>> {
                ExternRef::new(
                    &mut caller,
                    Arc::new(WeakMapResource(Mutex::new(Entries::default()))),
                )
            },
        )
        .map_err(bind_error)?;
    let row = GcHostImport::WeakSetAllocate;
    linker
        .func_wrap(
            row.module(),
            row.name(),
            |mut caller: WasmtimeCaller<'_, WasmHostState>| -> wasmtime::Result<Rooted<ExternRef>> {
                ExternRef::new(
                    &mut caller,
                    Arc::new(WeakSetResource(Mutex::new(Entries::default()))),
                )
            },
        )
        .map_err(bind_error)?;
    let row = GcHostImport::WeakMapSet;
    linker
        .func_wrap(
            row.module(),
            row.name(),
            |mut caller: WasmtimeCaller<'_, WasmHostState>,
             owner: Option<Rooted<EqRef>>,
             native: Rooted<ExternRef>,
             key: Option<Rooted<EqRef>>,
             value: Option<Rooted<EqRef>>|
             -> wasmtime::Result<()> {
                let resource = resource::<WeakMapResource>(&caller, native)?;
                let mut entries = resource.0.lock().unwrap_or_else(|e| e.into_inner());
                entries.insert(
                    &mut caller,
                    &required(owner)?,
                    &required(key)?,
                    Some(&required(value)?),
                )
            },
        )
        .map_err(bind_error)?;
    let row = GcHostImport::WeakSetAdd;
    linker
        .func_wrap(
            row.module(),
            row.name(),
            |mut caller: WasmtimeCaller<'_, WasmHostState>,
             owner: Option<Rooted<EqRef>>,
             native: Rooted<ExternRef>,
             key: Option<Rooted<EqRef>>|
             -> wasmtime::Result<()> {
                let resource = resource::<WeakSetResource>(&caller, native)?;
                let mut entries = resource.0.lock().unwrap_or_else(|e| e.into_inner());
                entries.insert(&mut caller, &required(owner)?, &required(key)?, None)
            },
        )
        .map_err(bind_error)?;
    let row = GcHostImport::WeakMapGet;
    linker
        .func_wrap(
            row.module(),
            row.name(),
            |mut caller: WasmtimeCaller<'_, WasmHostState>,
             native: Rooted<ExternRef>,
             key: Option<Rooted<EqRef>>|
             -> wasmtime::Result<Option<Rooted<EqRef>>> {
                let resource = resource::<WeakMapResource>(&caller, native)?;
                let mut entries = resource.0.lock().unwrap_or_else(|e| e.into_inner());
                let Some(index) = entries.find(&mut caller, &required(key)?)? else {
                    return Ok(None);
                };
                entries.entries[&index]
                    .value(&mut caller)?
                    .map(|v| v.as_eqref(&mut caller))
                    .transpose()
                    .map(Option::flatten)
            },
        )
        .map_err(bind_error)?;
    for (row, delete) in [
        (GcHostImport::WeakMapHas, false),
        (GcHostImport::WeakMapDelete, true),
    ] {
        linker
            .func_wrap(
                row.module(),
                row.name(),
                move |mut caller: WasmtimeCaller<'_, WasmHostState>,
                      native: Rooted<ExternRef>,
                      key: Option<Rooted<EqRef>>|
                      -> wasmtime::Result<i32> {
                    let resource = resource::<WeakMapResource>(&caller, native)?;
                    let mut entries = resource.0.lock().unwrap_or_else(|e| e.into_inner());
                    let found = entries.find(&mut caller, &required(key)?)?;
                    if delete {
                        if let Some(index) = found {
                            entries.remove(&mut caller, index)?;
                        }
                    }
                    Ok(i32::from(found.is_some()))
                },
            )
            .map_err(bind_error)?;
    }
    for (row, delete) in [
        (GcHostImport::WeakSetHas, false),
        (GcHostImport::WeakSetDelete, true),
    ] {
        linker
            .func_wrap(
                row.module(),
                row.name(),
                move |mut caller: WasmtimeCaller<'_, WasmHostState>,
                      native: Rooted<ExternRef>,
                      key: Option<Rooted<EqRef>>|
                      -> wasmtime::Result<i32> {
                    let resource = resource::<WeakSetResource>(&caller, native)?;
                    let mut entries = resource.0.lock().unwrap_or_else(|e| e.into_inner());
                    let found = entries.find(&mut caller, &required(key)?)?;
                    if delete {
                        if let Some(index) = found {
                            entries.remove(&mut caller, index)?;
                        }
                    }
                    Ok(i32::from(found.is_some()))
                },
            )
            .map_err(bind_error)?;
    }
    Ok(())
}
