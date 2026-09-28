use super::intl_host_request::CopiedIntlHostRequest;
use super::*;
use lila_intl::{CollatorCompareRequest, CollatorLocaleRequest};
use lila_intl::{CompareCollator, ResolveCollatorLocale};

pub(super) fn call(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    operation: IntlHostOp,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let bytes = match operation {
        IntlHostOp::ResolveCollatorLocale => {
            let request = CollatorLocaleRequest::decode(copied.bytes()).map_err(|error| {
                wasmtime::Error::msg(format!("invalid Collator locale request: {error}"))
            })?;
            let provider = kernel
                .operation::<ResolveCollatorLocale>()
                .map_err(|error| {
                    wasmtime::Error::msg(format!("Collator locale capability mismatch: {error}"))
                })?;
            let response = match provider.execute(request) {
                Ok(response) => response,
                Err(
                    lila_intl::IntlServiceError::UnsupportedLocale
                    | lila_intl::IntlServiceError::InvalidLocale
                    | lila_intl::IntlServiceError::InvalidOption,
                ) => {
                    return Ok(IntlHostCallOutcome::Rejected.wire());
                }
                Err(error) => {
                    return Err(wasmtime::Error::msg(format!(
                        "Collator locale resolution failed: {error}"
                    )));
                }
            };
            response.encode().map_err(|error| {
                wasmtime::Error::msg(format!("Collator locale response failed: {error}"))
            })?
        }
        IntlHostOp::CompareCollator => {
            let request = CollatorCompareRequest::decode(copied.bytes()).map_err(|error| {
                wasmtime::Error::msg(format!("invalid Collator comparison request: {error}"))
            })?;
            let provider = kernel.operation::<CompareCollator>().map_err(|error| {
                wasmtime::Error::msg(format!("Collator comparison capability mismatch: {error}"))
            })?;
            let result = provider.execute(request).map_err(|error| {
                wasmtime::Error::msg(format!("Collator comparison failed: {error}"))
            })?;
            lila_intl::encode_collator_comparison(result).map_err(|error| {
                wasmtime::Error::msg(format!("Collator comparison response failed: {error}"))
            })?
        }
        _ => {
            return Err(wasmtime::Error::msg(
                "non-Collator operation dispatched to Collator host",
            ));
        }
    };
    copied.write(&mut caller, &bytes)
}
