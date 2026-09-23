use super::intl_host_request::CopiedIntlHostRequest;
use super::*;
use lila_intl::number_format::{
    embedded_number_profiles, NumberLocaleRequest, NumberProfiles, NumberSupportedLocalesRequest,
    RangeNumberPartition, ResolvedNumberLocale, ScalarNumberPartition,
};
use lila_intl::NumberWireError;
use lila_intl::{
    FormatNumberParts, FormatNumberRangeParts, IntlOperation, IntlOperationProvider,
    ResolveNumberLocale, SupportedNumberLocales,
};
use lila_intl::{
    NumberFormatOperationError, NumberFormatRequest, NumberRangeFormatRequest,
    NumberSupportedLocalesResult,
};

/// One marker fixes the decoder, kernel request and response encoder together.
pub(super) trait NumberHostOperation:
    IntlOperation<Error = NumberFormatOperationError>
{
    fn decode_request(
        bytes: &[u8],
        profiles: &NumberProfiles,
    ) -> Result<Self::Request, NumberWireError>;
    fn encode_response(response: &Self::Response) -> Result<Vec<u8>, NumberWireError>;
}

macro_rules! number_host_operation {
    ($operation:ty, $response:ty, $bytes:ident, $profiles:ident, $decode:expr) => {
        impl NumberHostOperation for $operation {
            fn decode_request(
                $bytes: &[u8],
                $profiles: &NumberProfiles,
            ) -> Result<Self::Request, NumberWireError> {
                $decode
            }
            fn encode_response(response: &Self::Response) -> Result<Vec<u8>, NumberWireError> {
                <$response>::encode(response)
            }
        }
    };
}
number_host_operation!(
    ResolveNumberLocale,
    ResolvedNumberLocale,
    bytes,
    _profiles,
    NumberLocaleRequest::decode(bytes)
);
number_host_operation!(
    SupportedNumberLocales,
    NumberSupportedLocalesResult,
    bytes,
    _profiles,
    NumberSupportedLocalesRequest::decode(bytes)
);
number_host_operation!(
    FormatNumberParts,
    ScalarNumberPartition,
    bytes,
    profiles,
    NumberFormatRequest::decode(bytes, profiles)
);
number_host_operation!(
    FormatNumberRangeParts,
    RangeNumberPartition,
    bytes,
    profiles,
    NumberRangeFormatRequest::decode(bytes, profiles)
);

pub(super) fn call<O>(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64>
where
    O: NumberHostOperation,
    EmbeddedIntlProvider: IntlOperationProvider<O>,
{
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let profiles = embedded_number_profiles().map_err(|error| {
        wasmtime::Error::msg(format!("invalid embedded Intl number profiles: {error}"))
    })?;
    let request = O::decode_request(copied.bytes(), profiles).map_err(|error| {
        wasmtime::Error::msg(format!(
            "invalid Intl {} request: {error}",
            O::HOST_OP.name()
        ))
    })?;
    let operation = kernel.operation::<O>().map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl {} capability mismatch: {error}",
            O::HOST_OP.name()
        ))
    })?;
    let response = match operation.execute(request) {
        Ok(response) => response,
        Err(NumberFormatOperationError::NaNRangeEndpoint) => {
            return Ok(IntlHostCallOutcome::Rejected.wire());
        }
        Err(
            error
            @ (NumberFormatOperationError::Kernel(_) | NumberFormatOperationError::Numeric(_)),
        ) => {
            return Err(wasmtime::Error::msg(format!(
                "Intl {} failed: {error}",
                O::HOST_OP.name()
            )));
        }
    };
    let bytes = O::encode_response(&response).map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl {} response failed: {error}",
            O::HOST_OP.name()
        ))
    })?;
    copied.write(&mut caller, &bytes)
}

#[cfg(test)]
mod tests;
