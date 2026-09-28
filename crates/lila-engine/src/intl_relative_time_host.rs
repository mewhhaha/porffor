use super::intl_host_request::CopiedIntlHostRequest;
use super::*;
use lila_intl::{
    FormatRelativeTime, IntlServiceError, RelativeTimeFormatRequest, RelativeTimeLocaleRequest,
    ResolveRelativeTimeLocale,
};

/// JavaScript conversion and branding are emitted in Wasm. These host calls
/// receive only validated primitive snapshots and return provider data.
pub(super) fn resolve_locale(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let request = RelativeTimeLocaleRequest::decode(copied.bytes()).map_err(|error| {
        wasmtime::Error::msg(format!(
            "invalid RelativeTimeFormat locale request: {error}"
        ))
    })?;
    let provider = kernel
        .operation::<ResolveRelativeTimeLocale>()
        .map_err(|error| {
            wasmtime::Error::msg(format!(
                "RelativeTimeFormat locale capability mismatch: {error}"
            ))
        })?;
    let response = match provider.execute(request) {
        Ok(response) => response,
        Err(error) => return provider_error(error),
    };
    let bytes = response.encode().map_err(|error| {
        wasmtime::Error::msg(format!(
            "invalid RelativeTimeFormat locale response: {error}"
        ))
    })?;
    copied.write(&mut caller, &bytes)
}

pub(super) fn format(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let request = RelativeTimeFormatRequest::decode(copied.bytes()).map_err(|error| {
        wasmtime::Error::msg(format!(
            "invalid RelativeTimeFormat formatting request: {error}"
        ))
    })?;
    let provider = kernel.operation::<FormatRelativeTime>().map_err(|error| {
        wasmtime::Error::msg(format!("RelativeTimeFormat capability mismatch: {error}"))
    })?;
    let response = match provider.execute(request) {
        Ok(response) => response,
        Err(error) => return provider_error(error),
    };
    let bytes = response.encode().map_err(|error| {
        wasmtime::Error::msg(format!(
            "invalid RelativeTimeFormat parts response: {error}"
        ))
    })?;
    copied.write(&mut caller, &bytes)
}

fn provider_error(error: IntlServiceError) -> wasmtime::Result<i64> {
    match error {
        IntlServiceError::UnsupportedLocale
        | IntlServiceError::InvalidLocale
        | IntlServiceError::InvalidNumber
        | IntlServiceError::InvalidOption => Ok(IntlHostCallOutcome::Rejected.wire()),
        IntlServiceError::MissingRelativeTimePattern
        | IntlServiceError::InvalidRelativeTimeProfile(_)
        | IntlServiceError::InvalidWire
        | IntlServiceError::Provider(_) => Err(wasmtime::Error::msg(format!(
            "RelativeTimeFormat provider failed: {error}"
        ))),
    }
}
