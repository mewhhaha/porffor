use super::*;
pub(crate) fn fresh_from_payload<T>(
    store: &mut impl AsContextMut<Data = T>,
    inputs: &[Val],
    outputs: &mut [Val],
    target: &ArrayType,
    width: NativeProgramArrayWidth,
    construction: NativeProgramArrayConstruction,
) -> wasmtime::Result<()> {
    use scriptw_aot_wasm::NativeProgramPayloadDirectory;
    let (offset, length, ordinal, payload) = match (construction, inputs) {
        (
            NativeProgramArrayConstruction::Whole,
            [Val::I32(ordinal), Val::AnyRef(Some(payload))],
        ) => (0, None, *ordinal as u32, *payload),
        (
            NativeProgramArrayConstruction::Slice,
            [Val::I32(offset), Val::I32(length), Val::I32(ordinal), Val::AnyRef(Some(payload))],
        ) => (
            *offset as u32,
            Some(*length as u32),
            *ordinal as u32,
            *payload,
        ),
        _ => {
            return Err(wasmtime::Error::msg(
                "native payload allocator operands differ",
            ))
        }
    };
    let image = payload
        .as_array(&mut *store)?
        .ok_or_else(|| wasmtime::Error::msg("native program payload is not its actual array"))?;
    let total = image.len(&mut *store)?;
    let footer_start = total
        .checked_sub(NativeProgramPayloadDirectory::FOOTER_BYTES as u32)
        .ok_or_else(|| wasmtime::Error::msg("native program payload has no complete footer"))?;
    let mut footer = [0; NativeProgramPayloadDirectory::FOOTER_BYTES];
    for (index, byte) in footer.iter_mut().enumerate() {
        *byte = image
            .get(&mut *store, footer_start + index as u32)?
            .unwrap_i32() as u8;
    }
    let directory =
        NativeProgramPayloadDirectory::from_footer(total, &footer).map_err(wasmtime::Error::new)?;
    let start = directory.image_row(ordinal).map_err(wasmtime::Error::new)?;
    let mut row = [0; 8];
    for (index, byte) in row.iter_mut().enumerate() {
        *byte = image.get(&mut *store, start + index as u32)?.unwrap_i32() as u8;
    }
    let span = directory.image_span(&row).map_err(wasmtime::Error::new)?;
    allocate(
        store,
        outputs,
        target,
        width,
        offset,
        length,
        image,
        span.start as usize,
        (span.end - span.start) as usize,
    )
}
