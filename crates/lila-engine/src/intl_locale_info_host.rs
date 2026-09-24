use super::intl_host_request::CopiedIntlHostRequest;
use super::*;
use lila_intl::{
    LocaleInfo, LocaleInfoRequest, LocaleInfoResult, SupportedValues, SupportedValuesRequest,
};

/// `Intl.Locale.prototype.get*`: decode, run the pinned provider, encode the
/// result shape fixed by the request's query. Provider failures are defects of
/// the pinned data, never JavaScript-observable outcomes.
pub(super) fn locale_info(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let request = LocaleInfoRequest::decode(copied.bytes()).map_err(|error| {
        wasmtime::Error::msg(format!("invalid Intl locale-info request: {error}"))
    })?;
    let query = request.query();
    let operation = kernel.operation::<LocaleInfo>().map_err(|error| {
        wasmtime::Error::msg(format!("Intl locale-info capability mismatch: {error}"))
    })?;
    let result: LocaleInfoResult = operation
        .execute(request)
        .map_err(|error| wasmtime::Error::msg(format!("Intl locale-info failed: {error}")))?;
    let bytes = result.encode(query).map_err(|error| {
        wasmtime::Error::msg(format!("Intl locale-info response failed: {error}"))
    })?;
    copied.write(&mut caller, &bytes)
}

/// `Intl.supportedValuesOf` after the Wasm shell has mapped `key` to its
/// closed domain; an unknown key throws RangeError before any host call.
pub(super) fn supported_values(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let request = SupportedValuesRequest::decode(copied.bytes()).map_err(|error| {
        wasmtime::Error::msg(format!("invalid Intl supported-values request: {error}"))
    })?;
    let operation = kernel.operation::<SupportedValues>().map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl supported-values capability mismatch: {error}"
        ))
    })?;
    let result = operation
        .execute(request)
        .map_err(|error| wasmtime::Error::msg(format!("Intl supported-values failed: {error}")))?;
    let bytes = result.encode().map_err(|error| {
        wasmtime::Error::msg(format!("Intl supported-values response failed: {error}"))
    })?;
    copied.write(&mut caller, &bytes)
}
