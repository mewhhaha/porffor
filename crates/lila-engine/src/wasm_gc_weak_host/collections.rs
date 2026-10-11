use super::*;

struct WeakMapResource(Mutex<Vec<GcEphemeron>>);
struct WeakSetResource(Mutex<Vec<GcEphemeron>>);

fn find(
    caller: &mut WasmtimeCaller<'_, WasmHostState>,
    entries: &mut Vec<GcEphemeron>,
    key: &Rooted<AnyRef>,
) -> wasmtime::Result<Option<usize>> {
    let mut index = 0;
    while index < entries.len() {
        if !entries[index].is_live(&mut *caller)? {
            entries.swap_remove(index);
        } else if entries[index].matches_key(&mut *caller, key)? {
            return Ok(Some(index));
        } else {
            index += 1;
        }
    }
    Ok(None)
}

fn insert(
    caller: &mut WasmtimeCaller<'_, WasmHostState>,
    entries: &mut Vec<GcEphemeron>,
    owner: &Rooted<AnyRef>,
    key: &Rooted<AnyRef>,
    value: Option<&Rooted<AnyRef>>,
) -> wasmtime::Result<()> {
    if let Some(index) = find(caller, entries, key)? {
        return entries[index].set_value(&mut *caller, value.map(|v| &**v));
    }
    // Reserve before registering a conditional edge: allocation failure cannot
    // leave a value-retaining slot without an entry that can delete it.
    entries.try_reserve(1).map_err(wasmtime::Error::new)?;
    entries.push(GcEphemeron::new(
        &mut *caller,
        owner,
        key,
        value.map(|v| &**v),
    )?);
    Ok(())
}

pub(super) fn link(linker: &mut WasmtimeLinker<WasmHostState>) -> Result<(), EngineError> {
    let row = GcHostImport::WeakMapAllocate;
    linker
        .func_wrap(
            row.module(),
            row.name(),
            |mut caller: WasmtimeCaller<'_, WasmHostState>| -> wasmtime::Result<Rooted<ExternRef>> {
                ExternRef::new(
                    &mut caller,
                    Arc::new(WeakMapResource(Mutex::new(Vec::new()))),
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
                    Arc::new(WeakSetResource(Mutex::new(Vec::new()))),
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
                insert(
                    &mut caller,
                    &mut entries,
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
                insert(
                    &mut caller,
                    &mut entries,
                    &required(owner)?,
                    &required(key)?,
                    None,
                )
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
                let Some(index) = find(&mut caller, &mut entries, &required(key)?)? else {
                    return Ok(None);
                };
                entries[index]
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
                    let found = find(&mut caller, &mut entries, &required(key)?)?;
                    if delete {
                        if let Some(index) = found {
                            entries[index].remove(&mut caller)?;
                            entries.swap_remove(index);
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
                    let found = find(&mut caller, &mut entries, &required(key)?)?;
                    if delete {
                        if let Some(index) = found {
                            entries[index].remove(&mut caller)?;
                            entries.swap_remove(index);
                        }
                    }
                    Ok(i32::from(found.is_some()))
                },
            )
            .map_err(bind_error)?;
    }
    Ok(())
}
