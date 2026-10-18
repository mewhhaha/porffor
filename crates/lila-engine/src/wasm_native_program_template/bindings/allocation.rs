use super::*;
mod bulk;
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
    let start = image_start
        .checked_add(offset)
        .and_then(|start| u32::try_from(start).ok())
        .ok_or(wasmtime::Trap::ArrayOutOfBounds)?;
    let result = bulk::fresh_from_image(store, target, width, image, start, length)?;
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
