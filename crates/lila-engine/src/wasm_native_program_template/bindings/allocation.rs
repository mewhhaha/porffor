use super::*;
mod payload;
pub(super) use payload::fresh_from_payload;
pub(super) fn fresh<T>(
    store: &mut impl AsContextMut<Data = T>,
    inputs: &[Val],
    outputs: &mut [Val],
    target: &ArrayType,
    width: NativeProgramArrayWidth,
    construction: NativeProgramArrayConstruction,
) -> wasmtime::Result<()> {
    let (offset, length, image) = match (construction, inputs) {
        (NativeProgramArrayConstruction::Whole, [Val::AnyRef(Some(image))]) => (0, None, *image),
        (
            NativeProgramArrayConstruction::Slice,
            [Val::I32(offset), Val::I32(length), Val::AnyRef(Some(image))],
        ) => (*offset as u32, Some(*length as u32), *image),
        _ => {
            return Err(wasmtime::Error::msg(
                "native array constructor operands differ",
            ))
        }
    };
    let image = image
        .as_array(&mut *store)?
        .ok_or_else(|| wasmtime::Error::msg("native array source is not its data image"))?;
    let bytes = image.len(&mut *store)? as usize;
    allocate(
        store, outputs, target, width, offset, length, image, 0, bytes,
    )
}

fn allocate<T>(
    store: &mut impl AsContextMut<Data = T>,
    outputs: &mut [Val],
    target: &ArrayType,
    width: NativeProgramArrayWidth,
    offset: u32,
    length: Option<u32>,
    image: wasmtime::Rooted<ArrayRef>,
    image_start: usize,
    bytes: usize,
) -> wasmtime::Result<()> {
    let offset = offset as usize;
    let unit = width.bytes();
    let length = match length {
        Some(length) => length as usize,
        None => {
            if bytes % unit != 0 {
                return Err(wasmtime::Error::msg(
                    "native whole-array data is not element-aligned",
                ));
            }
            bytes / unit
        }
    };
    let available = bytes
        .checked_sub(offset)
        .ok_or(wasmtime::Trap::ArrayOutOfBounds)?;
    if length > available / unit {
        return Err(wasmtime::Trap::ArrayOutOfBounds.into());
    }
    let mut values = Vec::new();
    values
        .try_reserve_exact(length)
        .map_err(wasmtime::Error::new)?;
    for index in 0..length {
        let base = image_start + offset + index * unit;
        let low = image.get(&mut *store, base as u32)?.unwrap_i32() as u8;
        let value = if unit == 1 {
            u16::from(low)
        } else {
            let high = image.get(&mut *store, (base + 1) as u32)?.unwrap_i32() as u8;
            u16::from_le_bytes([low, high])
        };
        values.push(Val::I32(i32::from(value)));
    }
    let pre = ArrayRefPre::new(&mut *store, target.clone());
    // Fresh allocation on every original ArrayNewData/ArrayNewFixed execution.
    let result = ArrayRef::new_fixed(&mut *store, &pre, &values)?;
    let [output] = outputs else {
        return Err(wasmtime::Error::msg(
            "native array constructor result arity differs",
        ));
    };
    *output = Val::AnyRef(Some(result.to_anyref()));
    Ok(())
}

#[cfg(test)]
mod tests;
