use super::*;
use lila_intl::{
    FindNamedTimeZoneTransition, FindNamedTimeZoneTransitionRequest, LookupNamedTimeZone,
    LookupNamedTimeZoneRequest, NamedTimeZoneOffset, NamedTimeZoneOffsetRequest,
    PossibleNamedTimeZoneEpochs, PossibleNamedTimeZoneEpochsRequest, ResolveTimeZone,
    ResolveTimeZoneRequest, TimeZoneId, MAX_TIME_ZONE_IDENTIFIER_BYTES,
};

pub(super) fn lookup_named_time_zone(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
) -> wasmtime::Result<Option<Vec<u8>>> {
    // This boundary accepts a JavaScript string after ToString; non-ASCII and
    // ill-formed UTF-8 cannot name an IANA identifier and are ordinary rejection.
    if payload.len() > MAX_TIME_ZONE_IDENTIFIER_BYTES || !payload.is_ascii() {
        return Ok(None);
    }
    let text = core::str::from_utf8(payload).expect("ASCII is UTF-8");
    let Ok(identifier) = TimeZoneId::parse(text) else {
        return Ok(None);
    };
    let handle = kernel.operation::<LookupNamedTimeZone>().map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl named-zone kernel capability mismatch: {error}"
        ))
    })?;
    let result = match handle.execute(LookupNamedTimeZoneRequest::new(identifier)) {
        Ok(result) => result,
        Err(lila_intl::NamedTimeZoneLookupError::Unknown(_)) => return Ok(None),
        Err(error @ lila_intl::NamedTimeZoneLookupError::UnavailableService(_)) => {
            return Err(wasmtime::Error::msg(error.to_string()));
        }
    };
    Ok(Some(result.encode()))
}

pub(super) fn resolve_time_zone(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
) -> wasmtime::Result<Option<Vec<u8>>> {
    let request = ResolveTimeZoneRequest::decode(payload).map_err(|error| {
        wasmtime::Error::msg(format!("invalid Intl time-zone snapshot request: {error}"))
    })?;
    let wants_name = request.name_style().is_some();
    let handle = kernel.operation::<ResolveTimeZone>().map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl time-zone snapshot kernel capability mismatch: {error}"
        ))
    })?;
    let result = handle.execute(request).map_err(|error| {
        wasmtime::Error::msg(format!("Intl time-zone snapshot failed: {error}"))
    })?;
    if result.display_name().is_some() != wants_name {
        return Err(wasmtime::Error::msg(
            "Intl time-zone snapshot returned an inconsistent name field",
        ));
    }
    Ok(Some(result.encode()))
}

pub(super) fn named_time_zone_offset(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
) -> wasmtime::Result<Option<Vec<u8>>> {
    let request = NamedTimeZoneOffsetRequest::decode(payload).map_err(|error| {
        wasmtime::Error::msg(format!("invalid named-zone data request: {error}"))
    })?;
    let handle = kernel.operation::<NamedTimeZoneOffset>().map_err(|error| {
        wasmtime::Error::msg(format!(
            "named-zone data kernel capability mismatch: {error}"
        ))
    })?;
    let result = handle
        .execute(request)
        .map_err(|error| wasmtime::Error::msg(format!("named-zone data query failed: {error}")))?;
    Ok(Some(result.encode()))
}

pub(super) fn possible_named_time_zone_epochs(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
) -> wasmtime::Result<Option<Vec<u8>>> {
    let request = PossibleNamedTimeZoneEpochsRequest::decode(payload).map_err(|error| {
        wasmtime::Error::msg(format!("invalid named-zone data request: {error}"))
    })?;
    let handle = kernel
        .operation::<PossibleNamedTimeZoneEpochs>()
        .map_err(|error| {
            wasmtime::Error::msg(format!(
                "named-zone data kernel capability mismatch: {error}"
            ))
        })?;
    let result = handle
        .execute(request)
        .map_err(|error| wasmtime::Error::msg(format!("named-zone data query failed: {error}")))?;
    Ok(Some(result.encode()))
}

pub(super) fn find_named_time_zone_transition(
    kernel: &IntlKernel<EmbeddedIntlProvider>,
    payload: &[u8],
) -> wasmtime::Result<Option<Vec<u8>>> {
    let request = FindNamedTimeZoneTransitionRequest::decode(payload).map_err(|error| {
        wasmtime::Error::msg(format!("invalid named-zone data request: {error}"))
    })?;
    let handle = kernel
        .operation::<FindNamedTimeZoneTransition>()
        .map_err(|error| {
            wasmtime::Error::msg(format!(
                "named-zone data kernel capability mismatch: {error}"
            ))
        })?;
    let result = handle
        .execute(request)
        .map_err(|error| wasmtime::Error::msg(format!("named-zone data query failed: {error}")))?;
    Ok(Some(result.encode()))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::intl_host_probe::IntlHostProbe;
    use lila_intl::{
        FixedTimeZoneOffset, TimeZoneEpochSeconds, TimeZoneNameStyle, TimeZoneSelection,
    };

    #[test]
    fn named_lookup_gc_response_preserves_canonical_identity() {
        let mut probe = IntlHostProbe::new();
        let bytes = probe
            .invoke(IntlHostOp::LookupNamedTimeZone, b"etc/utc")
            .unwrap()
            .expect("admitted time-zone name");
        assert_eq!(bytes.len(), 26);
        assert_eq!(u64::from_le_bytes(bytes[..8].try_into().unwrap()), 7);
        assert_eq!(u64::from_le_bytes(bytes[8..16].try_into().unwrap()), 3);
        assert_eq!(&bytes[16..], b"Etc/UTCUTC");
    }

    #[test]
    fn snapshot_gc_response_preserves_offset_and_display_name() {
        let kernel = shared_embedded_intl_kernel().unwrap();
        let locale = kernel
            .operation::<CanonicalizeLocale>()
            .unwrap()
            .execute(LocaleTransformRequest::new(
                LocaleId::parse("en-US").unwrap(),
            ))
            .unwrap()
            .locale()
            .clone();
        let request = ResolveTimeZoneRequest::new(
            TimeZoneSelection::FixedOffset(FixedTimeZoneOffset::from_seconds(86_340).unwrap()),
            TimeZoneEpochSeconds::new(-1).unwrap(),
            Some(TimeZoneNameStyle::LongOffset),
            locale,
        )
        .encode();
        let mut probe = IntlHostProbe::new();
        let bytes = probe
            .invoke(IntlHostOp::ResolveTimeZone, &request)
            .unwrap()
            .expect("valid time-zone snapshot");
        assert_eq!(i64::from_le_bytes(bytes[..8].try_into().unwrap()), 86_340);
        assert_eq!(&bytes[8..], b"GMT+23:59");
    }

    #[test]
    fn lookup_rejection_is_distinct_from_malformed_snapshot() {
        let mut probe = IntlHostProbe::new();
        for spelling in [b"Missing/Zone".as_slice(), b"\xff", b"+25:00"] {
            assert_eq!(
                probe
                    .invoke(IntlHostOp::LookupNamedTimeZone, spelling)
                    .unwrap(),
                None
            );
        }
        assert!(probe.invoke(IntlHostOp::ResolveTimeZone, &[0; 8]).is_err());
        let bytes = probe
            .invoke(IntlHostOp::LookupNamedTimeZone, b"UTC")
            .unwrap()
            .expect("valid name remains callable after rejection");
        assert_eq!(&bytes[16..], b"UTCUTC");
    }
}

#[cfg(test)]
mod exact_query_tests;
