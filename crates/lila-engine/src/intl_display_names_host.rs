use super::*;
use lila_intl::{
    decode_display_names_locale_request, decode_display_names_supported_request,
    encode_display_name_response, encode_display_names_locale_response,
    encode_display_names_supported_response, DisplayNamesError, DisplayNamesWireOperation,
};

pub(super) fn call(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
    operation: DisplayNamesWireOperation,
) -> wasmtime::Result<Option<Vec<u8>>> {
    macro_rules! execute {
        ($marker:ty, $request:expr, $encode:path) => {{
            let request = $request.map_err(|error| {
                wasmtime::Error::msg(format!(
                    "invalid Intl {} request: {error}",
                    operation.global_operation().name()
                ))
            })?;
            let handle = kernel.operation::<$marker>().map_err(|error| {
                wasmtime::Error::msg(format!(
                    "Intl {} capability mismatch: {error}",
                    operation.global_operation().name()
                ))
            })?;
            let result = match handle.execute(request) {
                Ok(result) => result,
                Err(DisplayNamesError::InvalidCode) => return Ok(None),
                Err(
                    error @ (DisplayNamesError::InvalidConfiguration
                    | DisplayNamesError::InvalidResolvedLocale
                    | DisplayNamesError::InvalidData(_)
                    | DisplayNamesError::Resource(_)
                    | DisplayNamesError::UnavailableService(_)),
                ) => {
                    return Err(wasmtime::Error::msg(format!(
                        "Intl {} failed: {error}",
                        operation.global_operation().name()
                    )));
                }
            };
            $encode(&result).map_err(|error| {
                wasmtime::Error::msg(format!(
                    "Intl {} response failed: {error}",
                    operation.global_operation().name()
                ))
            })?
        }};
    }
    let bytes = match operation {
        DisplayNamesWireOperation::ResolveLocale => execute!(
            lila_intl::ResolveDisplayNamesLocale,
            decode_display_names_locale_request(payload),
            encode_display_names_locale_response
        ),
        DisplayNamesWireOperation::SupportedLocales => execute!(
            lila_intl::SupportedDisplayNamesLocales,
            decode_display_names_supported_request(payload),
            encode_display_names_supported_response
        ),
        DisplayNamesWireOperation::DisplayName => execute!(
            lila_intl::DisplayName,
            kernel.decode_display_name_request(payload),
            encode_display_name_response
        ),
    };
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests;
