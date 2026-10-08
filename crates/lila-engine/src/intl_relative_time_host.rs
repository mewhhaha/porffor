use super::*;
#[cfg(test)]
use lila_intl::embedded_relative_profiles;
use lila_intl::number_format::NumberSupportedLocalesRequest;
#[cfg(test)]
use lila_intl::number_format::{embedded_number_profiles_arc, PartitionLimits};
use lila_intl::{
    encode_relative_response, FormatRelativeTimePartsRequest, NumberSupportedLocalesResult,
    RelativeHostOp, RelativeRequest, RelativeResponse,
};

pub(super) fn call(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
    operation: RelativeHostOp,
) -> wasmtime::Result<Option<Vec<u8>>> {
    let request = kernel
        .decode_relative_time_request(operation, payload)
        .map_err(|error| {
            wasmtime::Error::msg(format!(
                "invalid Intl {} request: {error}",
                operation.global_operation().name()
            ))
        })?;
    macro_rules! execute {
        ($marker:ty, $request:expr) => {{
            kernel
                .operation::<$marker>()
                .map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "Intl {} capability mismatch: {error}",
                        operation.global_operation().name()
                    ))
                })?
                .execute($request)
                .map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "Intl {} failed: {error}",
                        operation.global_operation().name()
                    ))
                })?
        }};
    }
    let response = match request {
        RelativeRequest::Resolve(request) => {
            RelativeResponse::Resolved(execute!(lila_intl::ResolveRelativeTimeLocale, request))
        }
        RelativeRequest::Supported { requested, matcher } => {
            let NumberSupportedLocalesResult { locales } = execute!(
                lila_intl::SupportedRelativeTimeLocales,
                NumberSupportedLocalesRequest { requested, matcher }
            );
            RelativeResponse::Supported(locales)
        }
        RelativeRequest::Parts {
            configuration,
            value,
            unit,
        } => RelativeResponse::Parts(execute!(
            lila_intl::FormatRelativeTimeParts,
            FormatRelativeTimePartsRequest::new(configuration, value, unit)
        )),
    };
    let bytes = encode_relative_response(&response).map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl {} response failed: {error}",
            operation.global_operation().name()
        ))
    })?;
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests;
