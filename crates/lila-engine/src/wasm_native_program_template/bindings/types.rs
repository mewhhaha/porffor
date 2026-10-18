use super::*;
pub(super) fn array_reference(value: &ValType) -> wasmtime::Result<ArrayType> {
    let ValType::Ref(reference) = value else {
        return Err(wasmtime::Error::msg(
            "native data requires an array reference",
        ));
    };
    if reference.is_nullable() {
        return Err(wasmtime::Error::msg(
            "native data references must be non-null",
        ));
    }
    reference
        .heap_type()
        .as_concrete_array()
        .cloned()
        .ok_or_else(|| wasmtime::Error::msg("native data requires its actual concrete array type"))
}
pub(super) fn validate_factory(
    factory: &NativeProgramArrayFactory,
    function: &FuncType,
    owned: bool,
) -> wasmtime::Result<ArrayType> {
    let parameters = function.params().collect::<Vec<_>>();
    let results = function.results().collect::<Vec<_>>();
    let image = match (factory.construction(), owned, parameters.as_slice()) {
        (NativeProgramArrayConstruction::Whole, false, [image]) => image,
        (NativeProgramArrayConstruction::Slice, false, [ValType::I32, ValType::I32, image]) => {
            image
        }
        (NativeProgramArrayConstruction::Whole, true, [ValType::I32, image]) => image,
        (
            NativeProgramArrayConstruction::Slice,
            true,
            [ValType::I32, ValType::I32, ValType::I32, image],
        ) => image,
        _ => {
            return Err(wasmtime::Error::msg(
                "native array factory has a different parameter shape",
            ))
        }
    };
    let image = array_reference(image)?;
    if image.mutability() != Mutability::Const || !matches!(image.element_type(), StorageType::I8) {
        return Err(wasmtime::Error::msg(
            "native array factory data must be immutable bytes",
        ));
    }
    let [result] = results.as_slice() else {
        return Err(wasmtime::Error::msg(
            "native array factory requires exactly one result",
        ));
    };
    let target = array_reference(result)?;
    if !matches!(
        (factory.width(), target.element_type()),
        (NativeProgramArrayWidth::I8, StorageType::I8)
            | (NativeProgramArrayWidth::I16, StorageType::I16)
    ) {
        return Err(wasmtime::Error::msg(
            "native array factory packed storage differs",
        ));
    }
    Ok(target)
}
