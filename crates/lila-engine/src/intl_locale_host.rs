use super::*;
use lila_intl::{IntlOperation, IntlOperationProvider, LocaleTransformResult};

pub(super) fn wasm_intl_locale_call<O>(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_span_wire: i64,
    result_span_wire: i64,
) -> wasmtime::Result<i64>
where
    O: IntlOperation<
        Request = LocaleTransformRequest,
        Response = LocaleTransformResult,
        Error = LocaleTransformError,
    >,
    EmbeddedLocaleProvider: IntlOperationProvider<O>,
{
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let memory = match caller.get_export("memory") {
        Some(WasmtimeExtern::Memory(memory)) => memory,
        _ => {
            return Err(wasmtime::Error::msg(
                "Intl host call requires exported private memory",
            ));
        }
    };
    let request_span = IntlHostReadSpan::from_wire(request_span_wire);
    let result_span = IntlHostWriteSpan::from_wire(result_span_wire);
    let request_length = usize::try_from(request_span.length())
        .expect("u32 Intl request length fits the host address space");
    let request_offset = usize::try_from(request_span.offset())
        .expect("u32 Intl request offset fits the host address space");
    let request_end = request_offset.checked_add(request_length).ok_or_else(|| {
        wasmtime::Error::msg("Intl request memory range overflows the host address space")
    })?;
    let request_memory = memory
        .data(&caller)
        .get(request_offset..request_end)
        .ok_or_else(|| {
            wasmtime::Error::msg(format!(
                "Intl request memory at {request_offset} for {request_length} bytes is out of bounds"
            ))
        })?;
    let mut request_bytes = Vec::new();
    request_bytes
        .try_reserve_exact(request_length)
        .map_err(|error| {
            wasmtime::Error::msg(format!("could not allocate Intl request buffer: {error}"))
        })?;
    request_bytes.extend_from_slice(request_memory);
    let request_text = String::from_utf8(request_bytes).map_err(|error| {
        wasmtime::Error::msg(format!("Intl locale request is not UTF-8: {error}"))
    })?;
    let locale = LocaleId::parse(request_text.into_boxed_str()).map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl locale request crossed the host ABI without structural validation: {error}"
        ))
    })?;
    let handle = kernel.operation::<O>().map_err(|error| {
        wasmtime::Error::msg(format!("Intl locale kernel capability mismatch: {error}"))
    })?;
    let result = match handle.execute(LocaleTransformRequest::new(locale)) {
        Ok(result) => result,
        Err(LocaleTransformError::Unsupported(_)) => {
            return Ok(IntlHostCallOutcome::Rejected.wire());
        }
        Err(LocaleTransformError::InvalidProviderResult(error)) => {
            return Err(wasmtime::Error::msg(format!(
                "Intl locale provider returned invalid canonical data: {error}"
            )));
        }
    };
    let result_bytes = result.locale().as_str().as_bytes();
    let result_length = u32::try_from(result_bytes.len()).map_err(|_| {
        wasmtime::Error::msg("Intl canonical locale result does not fit the wire length")
    })?;
    if result_length > result_span.capacity() {
        return Ok(IntlHostCallOutcome::RequiredCapacity(result_length).wire());
    }
    let result_offset = usize::try_from(result_span.offset())
        .expect("u32 Intl result offset fits the host address space");
    memory
        .write(&mut caller, result_offset, result_bytes)
        .map_err(|error| {
            wasmtime::Error::msg(format!(
                "failed to write Intl result memory at {result_offset} for {result_length} bytes: {error}"
            ))
        })?;
    Ok(IntlHostCallOutcome::Written(result_length).wire())
}
