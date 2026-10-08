use super::*;
use lila_intl::{
    FormatListParts, IntlOperation, IntlOperationProvider, ListFormatOperationError,
    ListLocaleRequest, ListParts, ListSupportedLocalesRequest, ListSupportedLocalesResult,
    ListWireError, ResolveListLocale, ResolvedListLocale, SupportedListLocales,
};

pub(super) trait ListHostOperation: IntlOperation<Error = ListFormatOperationError> {
    fn decode_request(
        bytes: &[u8],
        kernel: &IntlKernel<EmbeddedIntlProvider>,
    ) -> Result<Self::Request, ListWireError>;
    fn encode_response(response: &Self::Response) -> Result<Vec<u8>, ListWireError>;
}
macro_rules! locale_operation {
    ($operation:ty, $request:ty, $response:ty) => {
        impl ListHostOperation for $operation {
            fn decode_request(
                bytes: &[u8],
                _kernel: &IntlKernel<EmbeddedIntlProvider>,
            ) -> Result<Self::Request, ListWireError> {
                <$request>::decode(bytes)
            }
            fn encode_response(response: &Self::Response) -> Result<Vec<u8>, ListWireError> {
                <$response>::encode(response)
            }
        }
    };
}
locale_operation!(ResolveListLocale, ListLocaleRequest, ResolvedListLocale);
locale_operation!(
    SupportedListLocales,
    ListSupportedLocalesRequest,
    ListSupportedLocalesResult
);
impl ListHostOperation for FormatListParts {
    fn decode_request(
        bytes: &[u8],
        kernel: &IntlKernel<EmbeddedIntlProvider>,
    ) -> Result<Self::Request, ListWireError> {
        kernel.decode_list_format_request(bytes)
    }
    fn encode_response(response: &Self::Response) -> Result<Vec<u8>, ListWireError> {
        response.encode()
    }
}

pub(super) fn call<O>(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
) -> wasmtime::Result<Option<Vec<u8>>>
where
    O: ListHostOperation,
    EmbeddedIntlProvider: IntlOperationProvider<O>,
{
    let request = O::decode_request(payload, kernel).map_err(|error| {
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
        Err(
            error @ (ListFormatOperationError::InvalidResolvedLocale
            | ListFormatOperationError::InvalidPartition
            | ListFormatOperationError::Data(_)
            | ListFormatOperationError::Resource(_)
            | ListFormatOperationError::UnavailableService(_)),
        ) => {
            return Err(wasmtime::Error::msg(format!(
                "Intl {} failed: {error}",
                O::HOST_OP.name()
            )))
        }
    };
    let bytes = O::encode_response(&response).map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl {} response failed: {error}",
            O::HOST_OP.name()
        ))
    })?;
    Ok(Some(bytes))
}
#[cfg(test)]
mod tests;

#[cfg(test)]
use lila_intl::{embedded_list_profiles, FormatListPartsRequest};
