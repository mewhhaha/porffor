use super::*;
use scriptw_aot_wasm::GcHostField;
use scriptw_ir::RuntimeDirectEvalPrivateScope;
use std::collections::BTreeSet;

pub(super) fn scan(
    store: &mut impl AsContextMut<Data = WasmHostState>,
    mut current: Option<Rooted<StructRef>>,
    signature: &NativeDirectEvalSignature,
    execution: &WasmAgentExecutionControl,
) -> wasmtime::Result<Vec<RuntimeDirectEvalPrivateScope>> {
    let mut scopes = Vec::new();
    let mut seen = BTreeSet::new();
    while let Some(environment) = current {
        execution.remaining().map_err(wasmtime::Error::new)?;
        schema::peer(&environment.ty(&*store)?, &signature.private_environment)?;
        let Val::I64(scope) = environment.field(
            &mut *store,
            GcHostField::PrivateEnvironmentClassScope
                .definition()
                .index() as usize,
        )?
        else {
            return Err(abi_error(
                "private environment scope is not the declared integer",
            ));
        };
        let scope = u32::try_from(scope).map_err(|_| {
            abi_error("private environment scope is outside the IR identity domain")
        })?;
        // Repeated identity also rejects a cycle, without raw GC addresses.
        if !seen.insert(scope) {
            return Err(abi_error("private chain repeats a scope or cycle"));
        }
        let Val::AnyRef(Some(names)) = environment.field(
            &mut *store,
            GcHostField::PrivateEnvironmentNames.definition().index() as usize,
        )?
        else {
            return Err(abi_error("private environment has no actual name table"));
        };
        let names = names
            .as_array(&*store)?
            .ok_or_else(|| abi_error("private names are not a GC array"))?;
        if !ArrayType::eq(&names.ty(&*store)?, &signature.private_names) {
            return Err(abi_error("private table has a different canonical type"));
        }
        scopes.push(RuntimeDirectEvalPrivateScope::new(
            scope,
            names.len(&*store)?,
        ));
        let parent = environment.field(
            &mut *store,
            GcHostField::PrivateEnvironmentParent.definition().index() as usize,
        )?;
        current = nullable_structure(store, &parent, &signature.private_environment)?;
    }
    execution.remaining().map_err(wasmtime::Error::new)?;
    Ok(scopes)
}
