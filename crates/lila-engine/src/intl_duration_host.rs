use super::*;
use lila_intl::{
    encode_duration_response, DurationError, DurationHostOp, DurationResponse, DurationWireError,
    DurationWireRequest,
};

pub(super) fn call(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
    operation: DurationHostOp,
) -> wasmtime::Result<Option<Vec<u8>>> {
    // Own the complete request before admitting any primitive or writing output.
    let request = match kernel.decode_duration_request(operation, payload) {
        Ok(request) => request,
        Err(DurationWireError::Rejected(
            DurationError::InvalidRecord | DurationError::InvalidBounds,
        )) => return Ok(None),
        Err(
            error @ (DurationWireError::Malformed(_)
            | DurationWireError::Resource(_)
            | DurationWireError::Rejected(
                DurationError::InvalidOptions
                | DurationError::InvalidLocale
                | DurationError::InvalidProfile
                | DurationError::Resource(_)
                | DurationError::Number(_)
                | DurationError::List(_)
                | DurationError::UnavailableService(_),
            )),
        ) => {
            return Err(wasmtime::Error::msg(format!(
                "invalid Intl {} request: {error}",
                operation.global_operation().name()
            )));
        }
    };
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
        DurationWireRequest::Resolve(request) => {
            DurationResponse::Resolved(execute!(lila_intl::ResolveDurationFormatLocale, request))
        }
        DurationWireRequest::SupportedLocales(request) => DurationResponse::SupportedLocales(
            execute!(lila_intl::SupportedDurationFormatLocales, request),
        ),
        DurationWireRequest::Parts(request) => {
            DurationResponse::Parts(execute!(lila_intl::PartitionDurationFormat, request))
        }
    };
    let bytes = encode_duration_response(&response).map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl {} response failed: {error}",
            operation.global_operation().name()
        ))
    })?;
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests;
