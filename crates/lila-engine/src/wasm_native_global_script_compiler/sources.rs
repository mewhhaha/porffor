use super::*;
use lila_ir::RuntimeGlobalScriptKind;
use wasmtime::{AsContextMut, OwnedRooted, StructRef, StructType};

pub(super) struct ConvertedRequest {
    pub(super) kind: RuntimeGlobalScriptKind,
    pub(super) source: Vec<u16>,
    pub(super) operands: [Val; 4],
    // Holds the actual P1 owner across nested compilation/GC for the request.
    _calling_program: OwnedRooted<StructRef>,
}
impl ConvertedRequest {
    pub(super) fn decode(
        store: &mut impl AsContextMut<Data = WasmHostState>,
        inputs: &[Val],
        signature: &NativeGlobalScriptSignature,
        execution: &WasmAgentExecutionControl,
    ) -> wasmtime::Result<Self> {
        execution.remaining().map_err(wasmtime::Error::new)?;
        let [Val::I32(kind), Val::AnyRef(Some(source)), Val::AnyRef(Some(realm)), Val::AnyRef(Some(program))] =
            inputs
        else {
            return Err(abi_error(
                "global source requires kind, String, Realm and actual calling P",
            ));
        };
        let kind = decode_kind(*kind)?;
        let source = source
            .as_struct(&*store)?
            .ok_or_else(|| abi_error("source is not a String struct"))?;
        let realm = realm
            .as_struct(&*store)?
            .ok_or_else(|| abi_error("target Realm is not a core struct"))?;
        let program = program
            .as_struct(&*store)?
            .ok_or_else(|| abi_error("calling P is not a core struct"))?;
        if !StructType::eq(&source.ty(&*store)?, &signature.string)
            || !StructType::eq(&realm.ty(&*store)?, &signature.realm)
            || !StructType::eq(&program.ty(&*store)?, &signature.program)
        {
            return Err(abi_error(
                "request GC operand has a different canonical core type",
            ));
        }
        let calling_program = program.to_owned_rooted(&mut *store)?;
        let source = wasm_gc_completion::string_units(store, source.to_eqref())
            .map_err(wasmtime::Error::new)?
            .into_vec();
        execution.remaining().map_err(wasmtime::Error::new)?;
        let operands = inputs
            .try_into()
            .map_err(|_| abi_error("request operand count changed"))?;
        Ok(Self {
            kind,
            source,
            operands,
            _calling_program: calling_program,
        })
    }
}
pub(super) fn decode_kind(kind: i32) -> wasmtime::Result<RuntimeGlobalScriptKind> {
    match kind {
        0 => Ok(RuntimeGlobalScriptKind::IndirectEval),
        1 => Ok(RuntimeGlobalScriptKind::RealmScript),
        _ => Err(abi_error(
            "global source kind is neither indirect eval nor Realm Script",
        )),
    }
}
