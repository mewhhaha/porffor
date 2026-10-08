use super::*;
use lila_intl::{
    DateTimeFormatError, DateTimeFormatRequest, DateTimeLocaleRequest, DateTimeLocaleResult,
    DateTimeParts, DateTimePlanRequest, DateTimePlanResult, DateTimeRangeParts,
    DateTimeRangeRequest, DateTimeSupportedLocalesRequest, DateTimeSupportedLocalesResult,
    DateTimeWireError, FormatDateTimeParts, FormatDateTimeRangeParts, IntlOperation,
    IntlOperationProvider, ResolveDateTimeLocale, SelectDateTimeFormat, SupportedDateTimeLocales,
};

/// The operation marker fixes both codecs, so dispatch cannot decode one
/// operation's request and execute another operation with a compatible shape.
pub(super) trait DateTimeHostOperation: IntlOperation<Error = DateTimeFormatError> {
    fn decode_request(bytes: &[u8]) -> Result<Self::Request, DateTimeWireError>;
    fn encode_response(response: &Self::Response) -> Result<Vec<u8>, DateTimeWireError>;
}

macro_rules! date_time_host_operation {
    ($operation:ty, $request:ty, $response:ty) => {
        impl DateTimeHostOperation for $operation {
            fn decode_request(bytes: &[u8]) -> Result<Self::Request, DateTimeWireError> {
                <$request>::decode(bytes)
            }
            fn encode_response(response: &Self::Response) -> Result<Vec<u8>, DateTimeWireError> {
                <$response>::encode(response)
            }
        }
    };
}
date_time_host_operation!(
    ResolveDateTimeLocale,
    DateTimeLocaleRequest,
    DateTimeLocaleResult
);
date_time_host_operation!(
    SupportedDateTimeLocales,
    DateTimeSupportedLocalesRequest,
    DateTimeSupportedLocalesResult
);
date_time_host_operation!(
    SelectDateTimeFormat,
    DateTimePlanRequest,
    DateTimePlanResult
);
date_time_host_operation!(FormatDateTimeParts, DateTimeFormatRequest, DateTimeParts);
date_time_host_operation!(
    FormatDateTimeRangeParts,
    DateTimeRangeRequest,
    DateTimeRangeParts
);

pub(super) fn call<O>(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
) -> wasmtime::Result<Option<Vec<u8>>>
where
    O: DateTimeHostOperation,
    EmbeddedIntlProvider: IntlOperationProvider<O>,
{
    let request = O::decode_request(payload).map_err(|error| {
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
        Err(DateTimeFormatError::UnavailableFormat | DateTimeFormatError::InputKindMismatch) => {
            return Ok(None);
        }
        Err(
            error @ (DateTimeFormatError::InvalidRequest(_)
            | DateTimeFormatError::InvalidPlan(_)
            | DateTimeFormatError::InvalidProfile(_)
            | DateTimeFormatError::UnavailableTimeZone(_)
            | DateTimeFormatError::UnavailableService(_)),
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
    Ok(Some(bytes))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intl_host_probe::IntlHostProbe;
    use lila_intl::{
        CanonicalLocaleId, DateTimeComponents, DateTimeDefaults, DateTimeExactInput,
        DateTimeFormatMatcher, DateTimeHourCyclePreference, DateTimeInput, DateTimeLocaleMatcher,
        DateTimeNumericWidth, DateTimeRequired, DateTimeStyleSelection, DateTimeValueKind,
        TimeZoneId, TimeZoneSelection,
    };

    fn resolve_request() -> DateTimeLocaleRequest {
        DateTimeLocaleRequest {
            requested: vec![CanonicalLocaleId::from_data("en-US").unwrap()],
            matcher: DateTimeLocaleMatcher::Lookup,
            calendar: None,
            numbering_system: None,
            hour_cycle: DateTimeHourCyclePreference::Default,
        }
    }

    fn gc_response<O: DateTimeHostOperation>(probe: &mut IntlHostProbe, bytes: &[u8]) -> Vec<u8> {
        probe
            .invoke(O::HOST_OP, bytes)
            .unwrap()
            .expect("accepted Intl request")
    }

    #[test]
    fn every_date_time_operation_returns_an_owned_gc_response() {
        let mut probe = IntlHostProbe::new();
        let bytes =
            gc_response::<ResolveDateTimeLocale>(&mut probe, &resolve_request().encode().unwrap());
        let locale = DateTimeLocaleResult::decode(&bytes).unwrap();
        assert_eq!(locale.locale.as_str(), "en-US");
        let bytes = gc_response::<SupportedDateTimeLocales>(
            &mut probe,
            &DateTimeSupportedLocalesRequest {
                requested: vec![
                    CanonicalLocaleId::from_data("en-US").unwrap(),
                    CanonicalLocaleId::from_data("zxx").unwrap(),
                ],
                matcher: DateTimeLocaleMatcher::Lookup,
            }
            .encode()
            .unwrap(),
        );
        assert_eq!(
            DateTimeSupportedLocalesResult::decode(&bytes)
                .unwrap()
                .locales,
            vec![CanonicalLocaleId::from_data("en-US").unwrap()]
        );
        let bytes = gc_response::<SelectDateTimeFormat>(
            &mut probe,
            &DateTimePlanRequest {
                locale,
                time_zone: TimeZoneSelection::Named(TimeZoneId::parse("UTC").unwrap()),
                selection: DateTimeStyleSelection::Components(DateTimeComponents {
                    year: Some(DateTimeNumericWidth::Numeric),
                    ..DateTimeComponents::default()
                }),
                matcher: DateTimeFormatMatcher::Basic,
                required: DateTimeRequired::Any,
                defaults: DateTimeDefaults::Date,
            }
            .encode()
            .unwrap(),
        );
        let plan = DateTimePlanResult::decode(&bytes).unwrap().plan;
        let input = DateTimeInput::Exact(
            DateTimeExactInput::new(DateTimeValueKind::Instant, -1, 999_999_999).unwrap(),
        );
        let bytes = gc_response::<FormatDateTimeParts>(
            &mut probe,
            &DateTimeFormatRequest {
                plan: plan.clone(),
                input,
            }
            .encode()
            .unwrap(),
        );
        let parts = DateTimeParts::decode(&bytes).unwrap();
        assert_eq!(parts.to_formatted_string(), "1969");
        let bytes = gc_response::<FormatDateTimeRangeParts>(
            &mut probe,
            &DateTimeRangeRequest {
                plan,
                start: input,
                end: input,
            }
            .encode()
            .unwrap(),
        );
        let range = DateTimeRangeParts::decode(&bytes).unwrap();
        assert_eq!(range.to_formatted_string(), "1969");
        assert!(range
            .parts
            .iter()
            .all(|part| part.source == lila_intl::DateTimeRangeSource::Shared));
    }

    #[test]
    fn malformed_date_time_messages_trap_at_the_gc_boundary() {
        let mut probe = IntlHostProbe::new();
        let valid = resolve_request().encode().unwrap();
        for bytes in [&valid[..valid.len() - 1], &[]] {
            assert!(probe
                .invoke(IntlHostOp::ResolveDateTimeLocale, &bytes)
                .is_err());
        }

        assert!(probe
            .invoke(IntlHostOp::SelectDateTimeFormat, &valid)
            .is_err());
    }
}
