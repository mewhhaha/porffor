use super::intl_host_request::CopiedIntlHostRequest;
use super::*;
use lila_intl::{
    PluralCategory, PluralCategoryRequest, PluralRulesLocaleRequest, ResolvePluralRulesLocale,
    SelectPluralCategory,
};

pub(super) fn call(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    operation: IntlHostOp,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let bytes = match operation {
        IntlHostOp::ResolvePluralRulesLocale => {
            let request = PluralRulesLocaleRequest::decode(copied.bytes()).map_err(|error| {
                wasmtime::Error::msg(format!("invalid PluralRules locale request: {error}"))
            })?;
            let provider = kernel
                .operation::<ResolvePluralRulesLocale>()
                .map_err(|error| {
                    wasmtime::Error::msg(format!("PluralRules locale capability mismatch: {error}"))
                })?;
            let response = match provider.execute(request) {
                Ok(response) => response,
                Err(
                    lila_intl::IntlServiceError::UnsupportedLocale
                    | lila_intl::IntlServiceError::InvalidLocale
                    | lila_intl::IntlServiceError::InvalidOption,
                ) => return Ok(IntlHostCallOutcome::Rejected.wire()),
                Err(error) => {
                    return Err(wasmtime::Error::msg(format!(
                        "PluralRules locale resolution failed: {error}"
                    )));
                }
            };
            response.encode().map_err(|error| {
                wasmtime::Error::msg(format!("PluralRules locale response failed: {error}"))
            })?
        }
        IntlHostOp::SelectPluralCategory => {
            let request = PluralCategoryRequest::decode(copied.bytes()).map_err(|error| {
                wasmtime::Error::msg(format!("invalid PluralRules selection request: {error}"))
            })?;
            let provider = kernel.operation::<SelectPluralCategory>().map_err(|error| {
                wasmtime::Error::msg(format!("PluralRules capability mismatch: {error}"))
            })?;
            let result: PluralCategory = match provider.execute(request) {
                Ok(result) => result,
                Err(lila_intl::IntlServiceError::InvalidOption | lila_intl::IntlServiceError::InvalidNumber) => {
                    return Ok(IntlHostCallOutcome::Rejected.wire());
                }
                Err(error) => {
                    return Err(wasmtime::Error::msg(format!(
                        "PluralRules selection failed: {error}"
                    )));
                }
            };
            result.encode().map_err(|error| {
                wasmtime::Error::msg(format!("PluralRules selection response failed: {error}"))
            })?
        }
        _ => {
            return Err(wasmtime::Error::msg(
                "non-PluralRules operation dispatched to PluralRules host",
            ));
        }
    };
    copied.write(&mut caller, &bytes)
}
