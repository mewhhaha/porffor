use super::*;
use lila_intl::{
    CollatorLocaleRequest, CollatorOperationError, CollatorOrdering,
    CollatorSupportedLocalesRequest, CollatorSupportedLocalesResult, CollatorWireError,
    CompareCollator, IntlOperation, IntlOperationProvider, ResolveCollatorLocale,
    ResolvedCollatorLocale, SupportedCollatorLocales,
};

pub(super) trait CollatorHostOperation:
    IntlOperation<Error = CollatorOperationError>
{
    fn decode_request(
        bytes: &[u8],
        kernel: &IntlKernel<EmbeddedIntlProvider>,
    ) -> Result<Self::Request, CollatorWireError>;
    fn encode_response(response: &Self::Response) -> Result<Vec<u8>, CollatorWireError>;
}
macro_rules! locale_operation {
    ($operation:ty, $request:ty, $response:ty) => {
        impl CollatorHostOperation for $operation {
            fn decode_request(
                bytes: &[u8],
                _kernel: &IntlKernel<EmbeddedIntlProvider>,
            ) -> Result<Self::Request, CollatorWireError> {
                <$request>::decode(bytes)
            }
            fn encode_response(response: &Self::Response) -> Result<Vec<u8>, CollatorWireError> {
                <$response>::encode(response)
            }
        }
    };
}
locale_operation!(
    ResolveCollatorLocale,
    CollatorLocaleRequest,
    ResolvedCollatorLocale
);
locale_operation!(
    SupportedCollatorLocales,
    CollatorSupportedLocalesRequest,
    CollatorSupportedLocalesResult
);
impl CollatorHostOperation for CompareCollator {
    fn decode_request(
        bytes: &[u8],
        kernel: &IntlKernel<EmbeddedIntlProvider>,
    ) -> Result<Self::Request, CollatorWireError> {
        kernel.decode_collator_compare_request(bytes)
    }
    fn encode_response(response: &Self::Response) -> Result<Vec<u8>, CollatorWireError> {
        response.encode()
    }
}

pub(super) fn call<O>(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
) -> wasmtime::Result<Option<Vec<u8>>>
where
    O: CollatorHostOperation,
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
            error @ (CollatorOperationError::InvalidConfiguration
            | CollatorOperationError::Data(_)
            | CollatorOperationError::Resource(_)
            | CollatorOperationError::UnavailableService(_)),
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
use lila_intl::{embedded_collator_profiles, CompareCollatorRequest};
