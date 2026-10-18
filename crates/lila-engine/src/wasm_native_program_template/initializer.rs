use super::*;
/// The raw artifact remains [] -> P. Only a checked owned-data template adds
/// its immutable payload parameter; every invocation allocates it afresh.
pub(crate) fn arguments(
    store: &mut impl AsContextMut<Data = WasmHostState>,
    template: &NativeProgramTemplate,
    initialize: &Func,
    program: &StructType,
) -> wasmtime::Result<Vec<Val>> {
    let Some(payload) = template.program_payload() else {
        return Ok(Vec::new());
    };
    let ty = initialize.ty(&*store);
    let parameters = ty.params().collect::<Vec<_>>();
    let results = ty.results().collect::<Vec<_>>();
    let [ValType::Ref(input)] = parameters.as_slice() else {
        return Err(wasmtime::Error::msg(
            "owned native initializer requires one actual payload reference",
        ));
    };
    let target = input
        .heap_type()
        .as_concrete_array()
        .cloned()
        .filter(|array| {
            !input.is_nullable()
                && array.mutability() == Mutability::Const
                && matches!(array.element_type(), StorageType::I8)
        })
        .ok_or_else(|| {
            wasmtime::Error::msg(
                "native initializer payload lacks its actual immutable byte-array type",
            )
        })?;
    if !matches!(results.as_slice(), [ValType::Ref(result)] if !result.is_nullable()
        && matches!(result.heap_type(), HeapType::ConcreteStruct(actual) if StructType::eq(actual, program)))
    {
        return Err(wasmtime::Error::msg(
            "owned native initializer does not return the actual caller P type",
        ));
    }
    let pre = ArrayRefPre::new(&mut *store, target);
    let image = ArrayRef::new_from_i8_slice(&mut *store, &pre, payload.bytes())?;
    Ok(vec![Val::AnyRef(Some(image.to_anyref()))])
}
