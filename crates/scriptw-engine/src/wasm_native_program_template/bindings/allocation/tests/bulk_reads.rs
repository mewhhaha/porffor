use super::*;
use wasmtime::RootScope;

#[test]
fn byte_range_reads_check_bounds_before_changing_the_destination() {
    let engine = shared_wasm_engine().unwrap();
    let mut store = Store::new(&engine, ());
    let bytes = [0, 127, 128, 255, 0, 0xd8, 0xff, 0xdc];
    let image = array(source(&mut store, &bytes), &mut store);
    for (start, count) in [(0, 8), (2, 4), (0, 0), (8, 0)] {
        let mut copied = vec![0xa5; count];
        image
            .copy_i8_range_to_slice(&mut store, start as u32, &mut copied)
            .unwrap();
        assert_eq!(copied, bytes[start..start + count]);
    }
    for (start, count) in [(7, 2), (8, 1), (9, 0), (u32::MAX, 0)] {
        let mut copied = vec![0xa5; count];
        assert!(image
            .copy_i8_range_to_slice(&mut store, start, &mut copied)
            .is_err());
        assert_eq!(copied, vec![0xa5; count]);
    }
}

#[test]
fn byte_range_reads_reject_nonbyte_arrays_and_expired_roots() {
    let engine = shared_wasm_engine().unwrap();
    let mut store = Store::new(&engine, ());
    let word_type = ArrayType::new(&engine, FieldType::new(Mutability::Const, StorageType::I16));
    let word_pre = ArrayRefPre::new(&mut store, word_type);
    let words = ArrayRef::new_fixed(&mut store, &word_pre, &[Val::I32(0xd800)]).unwrap();
    let mut copied = [0xa5];
    assert!(words
        .copy_i8_range_to_slice(&mut store, 0, &mut copied)
        .is_err());
    assert_eq!(copied, [0xa5]);
    let byte_type = ArrayType::new(&engine, FieldType::new(Mutability::Const, StorageType::I8));
    let byte_pre = ArrayRefPre::new(&mut store, byte_type);
    let expired = {
        let mut scope = RootScope::new(&mut store);
        ArrayRef::new_from_i8_slice(&mut scope, &byte_pre, &[17]).unwrap()
    };
    assert!(expired
        .copy_i8_range_to_slice(&mut store, 0, &mut copied)
        .is_err());
    assert_eq!(copied, [0xa5]);
}

#[test]
fn bulk_array_construction_keeps_every_utf16_unit_fresh_at_an_odd_byte_offset() {
    let engine = shared_wasm_engine().unwrap();
    let mut store = Store::new(&engine, ());
    let all_bytes: Vec<u8> = (0..=u8::MAX).collect();
    let byte_image = source(&mut store, &all_bytes);
    let byte_target = ArrayType::new(&engine, FieldType::new(Mutability::Var, StorageType::I8));
    let mut out = [Val::I32(0)];
    fresh(
        &mut store,
        &[byte_image],
        &mut out,
        &byte_target,
        NativeProgramArrayWidth::I8,
        NativeProgramArrayConstruction::Whole,
    )
    .unwrap();
    let byte_result = array(out[0], &mut store);
    let copied: Vec<_> = byte_result
        .elems(&mut store)
        .unwrap()
        .map(|value| value.unwrap_i32() as u8)
        .collect();
    assert_eq!(copied, all_bytes);
    let mut bytes = vec![0xa5];
    bytes.extend((0..=u16::MAX).flat_map(u16::to_le_bytes));
    bytes.push(0x5a);
    let image = source(&mut store, &bytes);
    let target = ArrayType::new(&engine, FieldType::new(Mutability::Var, StorageType::I16));
    let mut arrays = Vec::new();
    for _ in 0..2 {
        let mut out = [Val::I32(0)];
        fresh(
            &mut store,
            &[Val::I32(1), Val::I32(65_536), image],
            &mut out,
            &target,
            NativeProgramArrayWidth::I16,
            NativeProgramArrayConstruction::Slice,
        )
        .unwrap();
        let result = array(out[0], &mut store);
        assert_eq!(result.len(&mut store).unwrap(), 65_536);
        for (expected, actual) in result.elems(&mut store).unwrap().enumerate() {
            assert_eq!(actual.unwrap_i32() as u16, expected as u16);
        }
        arrays.push(result);
    }
    assert!(!Rooted::ref_eq(&store, &arrays[0], &arrays[1]).unwrap());
    arrays[0].set(&mut store, 0, Val::I32(23)).unwrap();
    assert_eq!(arrays[1].get(&mut store, 0).unwrap().unwrap_i32(), 0);
    let original = array(image, &mut store);
    let mut copied = [0; 2];
    original
        .copy_i8_range_to_slice(&mut store, 1, &mut copied)
        .unwrap();
    assert_eq!(copied, [0, 0]);
}
