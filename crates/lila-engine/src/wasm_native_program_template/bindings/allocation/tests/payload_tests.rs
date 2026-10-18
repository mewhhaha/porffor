use super::*;
fn payload(store: &mut Store<()>, data: &[u8]) -> Val {
    let mut bytes = data.to_vec();
    bytes.extend_from_slice(&0u32.to_le_bytes());
    bytes.extend_from_slice(&(data.len() as u32).to_le_bytes());
    bytes.extend_from_slice(b"LILANP01");
    for word in [1, data.len() as u32, 1, data.len() as u32 + 8, 0] {
        bytes.extend_from_slice(&word.to_le_bytes());
    }
    source(store, &bytes)
}

#[test]
fn the_same_allocator_uses_each_actual_payload_and_returns_fresh_arrays() {
    let engine = shared_wasm_engine().unwrap();
    let mut store = Store::new(&engine, ());
    let first = payload(&mut store, &[17, 19]);
    let second = payload(&mut store, &[29, 31]);
    let target = ArrayType::new(&engine, FieldType::new(Mutability::Var, StorageType::I8));
    let mut a = [Val::I32(0)];
    let mut b = [Val::I32(0)];
    fresh_from_payload(
        &mut store,
        &[Val::I32(0), first],
        &mut a,
        &target,
        NativeProgramArrayWidth::I8,
        NativeProgramArrayConstruction::Whole,
    )
    .unwrap();
    fresh_from_payload(
        &mut store,
        &[Val::I32(0), second],
        &mut b,
        &target,
        NativeProgramArrayWidth::I8,
        NativeProgramArrayConstruction::Whole,
    )
    .unwrap();
    let a = array(a[0], &mut store);
    let b = array(b[0], &mut store);
    a.set(&mut store, 0, Val::I32(99)).unwrap();
    assert_eq!(b.get(&mut store, 0).unwrap().unwrap_i32(), 29);
    assert_eq!(
        array(first, &mut store)
            .get(&mut store, 0)
            .unwrap()
            .unwrap_i32(),
        17
    );
    assert!(!Rooted::ref_eq(&store, &a, &b).unwrap());
}

#[test]
fn payload_slices_retain_byte_offsets_and_trap_before_entering_directory_bytes() {
    let engine = shared_wasm_engine().unwrap();
    let mut store = Store::new(&engine, ());
    let data = payload(&mut store, &[0, 0xD8, 0xFF, 0xDC]);
    let target = ArrayType::new(&engine, FieldType::new(Mutability::Const, StorageType::I16));
    let mut out = [Val::I32(0)];
    fresh_from_payload(
        &mut store,
        &[Val::I32(1), Val::I32(1), Val::I32(0), data],
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
    let error = fresh_from_payload(
        &mut store,
        &[Val::I32(3), Val::I32(1), Val::I32(0), data],
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
