use super::*;
use lila_intl::{
    decode_resolve_segmenter_locale_request, decode_supported_segmenter_locales_request,
    encode_resolve_segmenter_locale_response, encode_segment_utf16_response,
    encode_supported_segmenter_locales_response, SegmenterWireOperation,
};

pub(super) fn call(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
    operation: SegmenterWireOperation,
) -> wasmtime::Result<Option<Vec<u8>>> {
    macro_rules! execute {
        ($marker:ty, $request:expr, $encode:path) => {{
            let request = $request.map_err(|error| {
                wasmtime::Error::msg(format!(
                    "invalid Intl {} request: {error}",
                    operation.global_operation().name()
                ))
            })?;
            let result = kernel
                .operation::<$marker>()
                .map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "Intl {} capability mismatch: {error}",
                        operation.global_operation().name()
                    ))
                })?
                .execute(request)
                .map_err(|error| {
                    wasmtime::Error::msg(format!(
                        "Intl {} failed: {error}",
                        operation.global_operation().name()
                    ))
                })?;
            $encode(&result).map_err(|error| {
                wasmtime::Error::msg(format!(
                    "Intl {} response failed: {error}",
                    operation.global_operation().name()
                ))
            })?
        }};
    }
    let bytes = match operation {
        SegmenterWireOperation::ResolveLocale => execute!(
            lila_intl::ResolveSegmenterLocale,
            decode_resolve_segmenter_locale_request(payload),
            encode_resolve_segmenter_locale_response
        ),
        SegmenterWireOperation::SupportedLocales => execute!(
            lila_intl::SupportedSegmenterLocales,
            decode_supported_segmenter_locales_request(payload),
            encode_supported_segmenter_locales_response
        ),
        SegmenterWireOperation::SegmentUtf16 => execute!(
            lila_intl::SegmentUtf16,
            kernel.decode_segment_utf16_request(payload),
            encode_segment_utf16_response
        ),
    };
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests;
