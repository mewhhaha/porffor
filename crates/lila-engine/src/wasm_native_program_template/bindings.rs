use super::*;
use lila_aot_wasm::{
    NativeProgramArrayConstruction, NativeProgramArrayFactory, NativeProgramArrayWidth,
};
use wasmtime::{
    ArrayRef, ArrayRefPre, ArrayType, Extern, ExternType, Func, FuncType, Global, ImportType,
    Mutability, StorageType, Val, ValType,
};
mod allocation;
#[derive(Default)]
pub(crate) struct NativeArrayConstructorCache {
    entries: Vec<(FuncType, Func)>,
}

pub(super) fn data_import(
    store: &mut impl AsContextMut<Data = WasmHostState>,
    context: &NativeSourceCompilerContext,
    template: &NativeProgramTemplate,
    import: &ImportType<'_>,
) -> wasmtime::Result<Extern> {
    if import.name().starts_with("literal.") {
        let operand = import
            .name()
            .rsplit('.')
            .next()
            .and_then(|ordinal| ordinal.parse::<usize>().ok())
            .and_then(|index| template.literal_operands().get(index))
            .filter(|operand| operand.import_name() == import.name())
            .ok_or_else(|| wasmtime::Error::msg("unknown native literal operand"))?;
        let ExternType::Global(global) = import.ty() else {
            return Err(wasmtime::Error::msg(
                "native literal operand is not a Global",
            ));
        };
        if global.mutability() != Mutability::Const || !matches!(global.content(), ValType::I32) {
            return Err(wasmtime::Error::msg(
                "native literal operand requires an immutable i32 Global",
            ));
        }
        return Ok(Extern::Global(Global::new(
            &mut *store,
            global,
            Val::I32(operand.value()),
        )?));
    }
    if let Some(image) = template
        .data_images()
        .iter()
        .find(|image| image.import_name() == import.name())
    {
        let ExternType::Global(global) = import.ty() else {
            return Err(wasmtime::Error::msg("native data image is not a Global"));
        };
        let array = array_reference(global.content())?;
        if global.mutability() != Mutability::Const
            || array.mutability() != Mutability::Const
            || !matches!(array.element_type(), StorageType::I8)
        {
            return Err(wasmtime::Error::msg(
                "native data image lacks its immutable packed-byte type",
            ));
        }
        let pre = ArrayRefPre::new(&mut *store, array);
        let bytes = ArrayRef::new_from_i8_slice(&mut *store, &pre, image.bytes())?;
        return Ok(Extern::Global(Global::new(
            &mut *store,
            global,
            Val::AnyRef(Some(bytes.to_anyref())),
        )?));
    }
    let factory = template
        .array_factories()
        .iter()
        .find(|factory| factory.import_name() == import.name())
        .ok_or_else(|| wasmtime::Error::msg("unknown native data import"))?;
    let ExternType::Func(function) = import.ty() else {
        return Err(wasmtime::Error::msg(
            "native data factory is not a function",
        ));
    };
    let target = validate_factory(factory, &function)?;
    let mut cache = context
        .array_constructors
        .lock()
        .unwrap_or_else(|p| p.into_inner());
    if let Some((_, function)) = cache
        .entries
        .iter()
        .find(|(ty, _)| FuncType::eq(ty, &function))
    {
        return Ok(Extern::Func(*function));
    }
    let execution = Arc::clone(context.execution());
    let width = factory.width();
    let construction = factory.construction();
    let callback = Func::try_new(
        &mut *store,
        function.clone(),
        move |mut caller, inputs, outputs| {
            execution.remaining().map_err(wasmtime::Error::new)?;
            allocation::fresh(&mut caller, inputs, outputs, &target, width, construction)?;
            execution.remaining().map_err(wasmtime::Error::new)?;
            Ok(())
        },
    )?;
    // The source schema has finitely many packed array types. Retain a small
    // number of actual typed allocators, never source data or mutable selectors.
    if cache.entries.len() < 8 {
        cache.entries.push((function, callback));
    }
    Ok(Extern::Func(callback))
}
fn array_reference(value: &ValType) -> wasmtime::Result<ArrayType> {
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
fn validate_factory(
    factory: &NativeProgramArrayFactory,
    function: &FuncType,
) -> wasmtime::Result<ArrayType> {
    let parameters = function.params().collect::<Vec<_>>();
    let results = function.results().collect::<Vec<_>>();
    let image = match (factory.construction(), parameters.as_slice()) {
        (NativeProgramArrayConstruction::Whole, [image]) => image,
        (NativeProgramArrayConstruction::Slice, [ValType::I32, ValType::I32, image]) => image,
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
