//! Copy only the checked source span; every invocation allocates a fresh array.
use super::*;

pub(super) fn fresh_from_image<T>(
    store: &mut impl AsContextMut<Data = T>,
    target: &ArrayType,
    width: NativeProgramArrayWidth,
    image: wasmtime::Rooted<ArrayRef>,
    start: u32,
    length: usize,
) -> wasmtime::Result<wasmtime::Rooted<ArrayRef>> {
    let byte_count = length
        .checked_mul(width.bytes())
        .ok_or(wasmtime::Trap::ArrayOutOfBounds)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(byte_count)
        .map_err(wasmtime::Error::new)?;
    bytes.resize(byte_count, 0);
    image.copy_i8_range_to_slice(&mut *store, start, &mut bytes)?;
    let pre = ArrayRefPre::new(&mut *store, target.clone());
    match width {
        NativeProgramArrayWidth::I8 => ArrayRef::new_from_i8_slice(&mut *store, &pre, &bytes),
        NativeProgramArrayWidth::I16 => {
            let mut values = Vec::new();
            values
                .try_reserve_exact(length)
                .map_err(wasmtime::Error::new)?;
            values.extend(
                bytes
                    .chunks_exact(2)
                    .map(|pair| Val::I32(i32::from(u16::from_le_bytes([pair[0], pair[1]])))),
            );
            ArrayRef::new_fixed(&mut *store, &pre, &values)
        }
    }
}
