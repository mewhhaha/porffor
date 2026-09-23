use super::intl_host_request::CopiedIntlHostRequest;
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
    EmbeddedIntlProvider: IntlOperationProvider<O>,
{
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_span_wire, result_span_wire)?;
    let request_text = core::str::from_utf8(copied.bytes()).map_err(|error| {
        wasmtime::Error::msg(format!("Intl locale request is not UTF-8: {error}"))
    })?;
    let locale = LocaleId::parse(request_text).map_err(|error| {
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
    copied.write(&mut caller, result.locale().as_str().as_bytes())
}
