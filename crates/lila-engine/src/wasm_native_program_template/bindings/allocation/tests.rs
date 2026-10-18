use super::*;
use wasmtime::{FieldType, Rooted, Store};
fn source(store: &mut Store<()>, bytes: &[u8]) -> Val {
    let ty = ArrayType::new(
        store.engine(),
        FieldType::new(Mutability::Const, StorageType::I8),
    );
    let pre = ArrayRefPre::new(&mut *store, ty);
    Val::AnyRef(Some(
        ArrayRef::new_fixed(
            &mut *store,
            &pre,
            &bytes
                .iter()
                .map(|b| Val::I32(i32::from(*b)))
                .collect::<Vec<_>>(),
        )
        .unwrap()
        .to_anyref(),
    ))
}
fn array(value: Val, store: &mut Store<()>) -> Rooted<ArrayRef> {
    let Val::AnyRef(Some(value)) = value else {
        panic!("missing array")
    };
    value.as_array(store).unwrap().unwrap()
}
#[test]
fn extracted_arrays_remain_fresh_and_mutation_never_changes_another_invocation_or_image() {
    let engine = shared_wasm_engine().unwrap();
    let mut store = Store::new(&engine, ());
    let image = source(&mut store, &[7, 9]);
    let target = ArrayType::new(&engine, FieldType::new(Mutability::Var, StorageType::I8));
    let mut a = [Val::I32(0)];
    let mut b = [Val::I32(0)];
    fresh(
        &mut store,
        &[image],
        &mut a,
        &target,
        NativeProgramArrayWidth::I8,
        NativeProgramArrayConstruction::Whole,
    )
    .unwrap();
    fresh(
        &mut store,
        &[image],
        &mut b,
        &target,
        NativeProgramArrayWidth::I8,
        NativeProgramArrayConstruction::Whole,
    )
    .unwrap();
    let a = array(a[0], &mut store);
    let b = array(b[0], &mut store);
    assert!(!Rooted::ref_eq(&store, &a, &b).unwrap());
    a.set(&mut store, 0, Val::I32(99)).unwrap();
    assert_eq!(b.get(&mut store, 0).unwrap().unwrap_i32(), 7);
    assert_eq!(
        array(image, &mut store)
            .get(&mut store, 0)
            .unwrap()
            .unwrap_i32(),
        7
    );
}
#[test]
fn source_images_preserve_surrogate_bits_and_slice_byte_offsets_and_traps() {
    let engine = shared_wasm_engine().unwrap();
    let mut store = Store::new(&engine, ());
    let image = source(&mut store, &[0, 0xD8, 0xFF, 0xDC]);
    let target = ArrayType::new(&engine, FieldType::new(Mutability::Const, StorageType::I16));
    let mut out = [Val::I32(0)];
    fresh(
        &mut store,
        &[image],
        &mut out,
        &target,
        NativeProgramArrayWidth::I16,
        NativeProgramArrayConstruction::Whole,
    )
    .unwrap();
    let units = array(out[0], &mut store);
    assert_eq!(
        units.get(&mut store, 0).unwrap().unwrap_i32() as u16,
        0xD800
    );
    assert_eq!(
        units.get(&mut store, 1).unwrap().unwrap_i32() as u16,
        0xDCFF
    );
    fresh(
        &mut store,
        &[Val::I32(1), Val::I32(1), image],
        &mut out,
        &target,
        NativeProgramArrayWidth::I16,
        NativeProgramArrayConstruction::Slice,
    )
    .unwrap();
    assert_eq!(
        array(out[0], &mut store)
            .get(&mut store, 0)
            .unwrap()
            .unwrap_i32() as u16,
        0xFFD8
    );
    let error = fresh(
        &mut store,
        &[Val::I32(-1), Val::I32(1), image],
        &mut out,
        &target,
        NativeProgramArrayWidth::I16,
        NativeProgramArrayConstruction::Slice,
    )
    .unwrap_err();
    assert_eq!(
        error.downcast_ref::<wasmtime::Trap>(),
        Some(&wasmtime::Trap::ArrayOutOfBounds)
    );
}

mod bulk_reads;
mod payload_tests;
