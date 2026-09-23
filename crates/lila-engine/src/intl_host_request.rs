use super::*;

/// The request is owned before any result can be written, including overlapping spans.
pub(super) struct CopiedIntlHostRequest {
    memory: WasmtimeMemory,
    bytes: Vec<u8>,
    result: IntlHostWriteSpan,
}

impl CopiedIntlHostRequest {
    pub(super) fn read(
        caller: &mut WasmtimeCaller<'_, WasmHostState>,
        request_wire: i64,
        result_wire: i64,
    ) -> wasmtime::Result<Self> {
        let memory = match caller.get_export("memory") {
            Some(WasmtimeExtern::Memory(memory)) => memory,
            _ => {
                return Err(wasmtime::Error::msg(
                    "Intl host call requires exported private memory",
                ))
            }
        };
        let request = IntlHostReadSpan::from_wire(request_wire);
        let offset = usize::try_from(request.offset()).expect("u32 fits the host address space");
        let length = usize::try_from(request.length()).expect("u32 fits the host address space");
        let end = offset.checked_add(length).ok_or_else(|| {
            wasmtime::Error::msg("Intl request memory range overflows the host address space")
        })?;
        let source = memory.data(&*caller).get(offset..end).ok_or_else(|| {
            wasmtime::Error::msg(format!(
                "Intl request memory at {offset} for {length} bytes is out of bounds"
            ))
        })?;
        let mut bytes = Vec::new();
        bytes.try_reserve_exact(length).map_err(|error| {
            wasmtime::Error::msg(format!("could not allocate Intl request buffer: {error}"))
        })?;
        bytes.extend_from_slice(source);
        Ok(Self {
            memory,
            bytes,
            result: IntlHostWriteSpan::from_wire(result_wire),
        })
    }

    pub(super) fn bytes(&self) -> &[u8] {
        &self.bytes
    }

    pub(super) fn write(
        self,
        caller: &mut WasmtimeCaller<'_, WasmHostState>,
        bytes: &[u8],
    ) -> wasmtime::Result<i64> {
        let length = u32::try_from(bytes.len())
            .map_err(|_| wasmtime::Error::msg("Intl result does not fit the wire length"))?;
        if length > self.result.capacity() {
            return Ok(IntlHostCallOutcome::RequiredCapacity(length).wire());
        }
        let offset =
            usize::try_from(self.result.offset()).expect("u32 fits the host address space");
        self.memory.write(caller, offset, bytes).map_err(|error| {
            wasmtime::Error::msg(format!(
                "failed to write Intl result memory at {offset} for {length} bytes: {error}"
            ))
        })?;
        Ok(IntlHostCallOutcome::Written(length).wire())
    }
}
