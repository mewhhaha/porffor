use super::*;
use lila_aot_wasm::GcHostValue;
use lila_ir::{RuntimeDirectEvalCaller, RuntimeDirectEvalCallerDescriptor};
use wasmtime::{AnyRef, ArrayType, AsContextMut, OwnedRooted, Rooted, StructRef, StructType};

mod invocation;
mod private_chain;

pub(super) struct ConvertedRequest {
    pub(super) caller: RuntimeDirectEvalCaller,
    pub(super) source: Vec<u16>,
    pub(super) operands: [Val; 14],
    // All caller records and value references survive compilation, nested
    // execution and allocations, even if the caller stack has no other edge.
    _roots: Vec<OwnedRooted<AnyRef>>,
}
impl ConvertedRequest {
    pub(super) fn decode(
        store: &mut impl AsContextMut<Data = WasmHostState>,
        inputs: &[Val],
        signature: &NativeDirectEvalSignature,
        execution: &WasmAgentExecutionControl,
    ) -> wasmtime::Result<Self> {
        execution.remaining().map_err(wasmtime::Error::new)?;
        let [source, descriptor, realm, lexical, variable, private, Val::I32(this_tag), Val::I64(this_scalar), Val::AnyRef(this_ref), Val::I32(target_tag), Val::I64(target_scalar), Val::AnyRef(target_ref), invocation, program] =
            inputs
        else {
            return Err(abi_error("invalid fourteen-operand direct-eval request"));
        };
        GcHostValue::check(*this_tag, *this_scalar, this_ref.is_some()).map_err(abi_error)?;
        GcHostValue::check(*target_tag, *target_scalar, target_ref.is_some()).map_err(abi_error)?;
        let mut roots = Vec::new();
        for value in inputs {
            if let Val::AnyRef(Some(reference)) = value {
                roots.push(reference.to_owned_rooted(&mut *store)?);
            }
        }
        let source = required_structure(store, source, &signature.string)?;
        required_structure(store, realm, &signature.realm)?;
        required_structure(store, program, &signature.program)?;
        nullable_structure(store, lexical, &signature.environment)?;
        nullable_structure(store, variable, &signature.environment)?;
        let private = nullable_structure(store, private, &signature.private_environment)?;
        let invocation = required_structure(store, invocation, &signature.invocation)?;
        let Val::AnyRef(Some(descriptor)) = descriptor else {
            return Err(abi_error("caller descriptor is null or not a GC reference"));
        };
        let descriptor = descriptor
            .as_array(&*store)?
            .ok_or_else(|| abi_error("caller descriptor is not an array"))?;
        if !ArrayType::eq(&descriptor.ty(&*store)?, &signature.descriptor) {
            return Err(abi_error(
                "caller descriptor has a different immutable byte type",
            ));
        }
        let mut bytes = Vec::new();
        let length = descriptor.len(&*store)? as usize;
        bytes
            .try_reserve_exact(length)
            .map_err(wasmtime::Error::new)?;
        bytes.resize(length, 0);
        descriptor.copy_to_i8_slice(&mut *store, &mut bytes)?;
        execution.remaining().map_err(wasmtime::Error::new)?;
        // Protocol/codec rejection never becomes a JavaScript SyntaxError.
        let descriptor = RuntimeDirectEvalCallerDescriptor::decode(&bytes).map_err(abi_error)?;
        let scopes = private_chain::scan(store, private, signature, execution)?;
        let caller = RuntimeDirectEvalCaller::validate(descriptor, scopes).map_err(abi_error)?;
        invocation::validate(store, &invocation, inputs, &caller, signature, execution)?;
        let source = wasm_gc_completion::string_units(store, source.to_eqref())
            .map_err(wasmtime::Error::new)?
            .into_vec();
        execution.remaining().map_err(wasmtime::Error::new)?;
        Ok(Self {
            caller,
            source,
            operands: inputs
                .try_into()
                .map_err(|_| abi_error("direct-eval operand count changed"))?,
            _roots: roots,
        })
    }
}

fn required_structure(
    store: &mut impl AsContextMut<Data = WasmHostState>,
    value: &Val,
    ty: &StructType,
) -> wasmtime::Result<Rooted<StructRef>> {
    nullable_structure(store, value, ty)?.ok_or_else(|| abi_error("required caller record is null"))
}
fn nullable_structure(
    store: &mut impl AsContextMut<Data = WasmHostState>,
    value: &Val,
    ty: &StructType,
) -> wasmtime::Result<Option<Rooted<StructRef>>> {
    let Val::AnyRef(reference) = value else {
        return Err(abi_error("caller record is not a GC reference"));
    };
    reference
        .map(|reference| {
            let value = reference
                .as_struct(&*store)?
                .ok_or_else(|| abi_error("caller record is not a core struct"))?;
            schema::peer(&value.ty(&*store)?, ty)?;
            Ok(value)
        })
        .transpose()
}
