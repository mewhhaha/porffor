//! Only declaration-owned layouts and field projections enter this boundary.
use super::*;
use scriptw_aot_wasm::{GcHostField, GcHostLayout};
use wasmtime::{ArrayType, FieldType, HeapType, Mutability, StorageType, StructType, ValType};

pub(super) fn structure(
    value: &ValType,
    nullable: bool,
    layout: GcHostLayout,
) -> wasmtime::Result<StructType> {
    let ValType::Ref(reference) = value else {
        return Err(abi_error("expected a concrete struct reference"));
    };
    if reference.is_nullable() != nullable {
        return Err(abi_error("wrong struct reference nullability"));
    }
    let ty = reference
        .heap_type()
        .as_concrete_struct()
        .ok_or_else(|| abi_error("expected a concrete core struct"))?;
    if layout.field_count() != Some(ty.fields().len() as u32) {
        return Err(abi_error("core struct differs from declared field count"));
    }
    Ok(ty.clone())
}
pub(super) fn array(
    value: &ValType,
    nullable: bool,
    layout: GcHostLayout,
) -> wasmtime::Result<ArrayType> {
    let ValType::Ref(reference) = value else {
        return Err(abi_error("expected a concrete array reference"));
    };
    if reference.is_nullable() != nullable {
        return Err(abi_error("wrong array reference nullability"));
    }
    let ty = reference
        .heap_type()
        .as_concrete_array()
        .ok_or_else(|| abi_error("expected a concrete core array"))?;
    if layout.array_mutable() != Some(ty.mutability() == Mutability::Var)
        || !wasm_gc_completion::storage_matches(
            layout
                .array_element()
                .ok_or_else(|| abi_error("layout is not an array"))?,
            &ty.element_type(),
        )
    {
        return Err(abi_error(
            "core array differs from declared storage or mutability",
        ));
    }
    Ok(ty.clone())
}
pub(super) fn field(ty: &StructType, name: GcHostField) -> wasmtime::Result<FieldType> {
    let definition = name.definition();
    if definition.layout().field_count() != Some(ty.fields().len() as u32) {
        return Err(abi_error("field owner differs from declared layout"));
    }
    let actual = ty
        .field(definition.index() as usize)
        .ok_or_else(|| abi_error("missing declared field"))?;
    if (actual.mutability() == Mutability::Var) != definition.mutable()
        || !wasm_gc_completion::storage_matches(definition.storage(), actual.element_type())
    {
        return Err(abi_error(
            "field storage or mutability differs from declaration",
        ));
    }
    Ok(actual)
}
pub(super) fn field_structure(
    ty: &StructType,
    name: GcHostField,
    nullable: bool,
    layout: GcHostLayout,
) -> wasmtime::Result<StructType> {
    let actual = field(ty, name)?;
    let StorageType::ValType(value) = actual.element_type() else {
        return Err(abi_error("declared struct field has no value reference"));
    };
    structure(value, nullable, layout)
}
pub(super) fn field_array(
    ty: &StructType,
    name: GcHostField,
    nullable: bool,
    layout: GcHostLayout,
) -> wasmtime::Result<ArrayType> {
    let actual = field(ty, name)?;
    let StorageType::ValType(value) = actual.element_type() else {
        return Err(abi_error("declared array field has no value reference"));
    };
    array(value, nullable, layout)
}
pub(super) fn peer(actual: &StructType, expected: &StructType) -> wasmtime::Result<()> {
    if !StructType::eq(actual, expected) {
        return Err(abi_error("core references have different canonical types"));
    }
    Ok(())
}
pub(super) fn eq_reference(value: &ValType) -> wasmtime::Result<()> {
    if !matches!(value, ValType::Ref(reference) if reference.is_nullable() && matches!(reference.heap_type(), HeapType::Eq))
    {
        return Err(abi_error("JavaScript value payload is not nullable eqref"));
    }
    Ok(())
}
