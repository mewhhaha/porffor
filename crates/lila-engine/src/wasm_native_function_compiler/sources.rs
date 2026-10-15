use super::*;
use lila_aot_wasm::{GcHostField, GcHostValue};
use wasmtime::{AsContextMut, StructType};

#[derive(Debug)]
pub(super) struct ConvertedRequest {
    pub(super) kind: DynamicFunctionKind,
    pub(super) arguments: Vec<Vec<u16>>,
    pub(super) operands: [Val; 7],
}

impl ConvertedRequest {
    pub(super) fn decode(
        store: &mut impl AsContextMut<Data = WasmHostState>,
        inputs: &[Val],
        signature: &NativeFunctionSignature,
        execution: &WasmAgentExecutionControl,
    ) -> wasmtime::Result<Self> {
        let [Val::I32(kind), Val::I32(tag), Val::I64(scalar), Val::AnyRef(target), Val::AnyRef(Some(sources)), Val::AnyRef(Some(_)), Val::AnyRef(_)] =
            inputs
        else {
            return Err(abi_error("invalid native Function request operands"));
        };
        let kind = decode_kind(*kind)?;
        GcHostValue::check(*tag, *scalar, target.is_some()).map_err(abi_error)?;
        let sources = sources
            .as_array(&*store)?
            .ok_or_else(|| abi_error("sources are not a GC array"))?;
        if !wasmtime::ArrayType::eq(&sources.ty(&*store)?, &signature.sources) {
            return Err(abi_error("source array has a different canonical type"));
        }
        let length = sources.len(&*store)?;
        let mut arguments = Vec::new();
        arguments
            .try_reserve_exact(length as usize)
            .map_err(wasmtime::Error::new)?;
        for index in 0..length {
            execution.remaining().map_err(wasmtime::Error::new)?;
            let Val::AnyRef(Some(stored)) = sources.get(&mut *store, index)? else {
                return Err(abi_error("source array entry is not a StoredValue"));
            };
            let stored = stored
                .as_struct(&*store)?
                .ok_or_else(|| abi_error("source is not a struct"))?;
            if !StructType::eq(&stored.ty(&*store)?, &signature.stored) {
                return Err(abi_error(
                    "source entry has a different canonical StoredValue type",
                ));
            }
            let mut read =
                |field: GcHostField| stored.field(&mut *store, field.definition().index() as usize);
            let Val::I32(tag) = read(GcHostField::StoredTag)? else {
                return Err(abi_error("source tag is not I32"));
            };
            let Val::I64(scalar) = read(GcHostField::StoredScalar)? else {
                return Err(abi_error("source scalar is not I64"));
            };
            let Val::AnyRef(reference) = read(GcHostField::StoredReference)? else {
                return Err(abi_error("source payload is not a GC reference"));
            };
            let checked =
                GcHostValue::check(tag, scalar, reference.is_some()).map_err(abi_error)?;
            if checked.tag() != WasmRuntimeValueTag::String {
                return Err(abi_error(
                    "Function sources must already be converted Strings",
                ));
            }
            let reference = reference.ok_or_else(|| abi_error("String source has no root"))?;
            let string = reference
                .as_struct(&*store)?
                .ok_or_else(|| abi_error("source String is not a struct"))?;
            if !StructType::eq(&string.ty(&*store)?, &signature.string) {
                return Err(abi_error("source String has a different canonical type"));
            }
            let reference = string.to_eqref();
            arguments.push(
                wasm_gc_completion::string_units(store, reference)
                    .map_err(wasmtime::Error::new)?
                    .into_vec(),
            );
        }
        let operands: [Val; 7] = inputs
            .try_into()
            .map_err(|_| abi_error("request operand count changed"))?;
        Ok(Self {
            kind,
            arguments,
            operands,
        })
    }
}

pub(super) fn decode_kind(code: i32) -> wasmtime::Result<DynamicFunctionKind> {
    match code {
        0 => Ok(DynamicFunctionKind::Ordinary),
        1 => Ok(DynamicFunctionKind::Generator),
        2 => Ok(DynamicFunctionKind::Async),
        3 => Ok(DynamicFunctionKind::AsyncGenerator),
        _ => Err(abi_error("invalid CreateDynamicFunction kind")),
    }
}
