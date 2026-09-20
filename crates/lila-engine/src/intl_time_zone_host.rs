use super::intl_host_request::CopiedIntlHostRequest;
use super::*;
use lila_intl::{
    LookupNamedTimeZone, LookupNamedTimeZoneRequest, ResolveTimeZone, ResolveTimeZoneRequest,
    TimeZoneId, MAX_TIME_ZONE_IDENTIFIER_BYTES,
};

pub(super) fn lookup_named_time_zone(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    // This boundary accepts a JavaScript string after ToString; non-ASCII and
    // ill-formed UTF-8 cannot name an IANA identifier and are ordinary rejection.
    if copied.bytes().len() > MAX_TIME_ZONE_IDENTIFIER_BYTES || !copied.bytes().is_ascii() {
        return Ok(IntlHostCallOutcome::Rejected.wire());
    }
    let text = core::str::from_utf8(copied.bytes()).expect("ASCII is UTF-8");
    let Ok(identifier) = TimeZoneId::parse(text) else {
        return Ok(IntlHostCallOutcome::Rejected.wire());
    };
    let handle = kernel.operation::<LookupNamedTimeZone>().map_err(|error| {
        wasmtime::Error::msg(format!(
            "Intl named-zone kernel capability mismatch: {error}"
        ))
    })?;
    let result = match handle.execute(LookupNamedTimeZoneRequest::new(identifier)) {
        Ok(result) => result,
        Err(_) => return Ok(IntlHostCallOutcome::Rejected.wire()),
    };
    copied.write(&mut caller, &result.encode())
}

pub(super) fn resolve_time_zone(
    mut caller: WasmtimeCaller<'_, WasmHostState>,
    request_wire: i64,
    result_wire: i64,
) -> wasmtime::Result<i64> {
    let kernel = Arc::clone(&caller.data().intl_kernel);
    let copied = CopiedIntlHostRequest::read(&mut caller, request_wire, result_wire)?;
    let request = ResolveTimeZoneRequest::decode(copied.bytes()).map_err(|error| {
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
    copied.write(&mut caller, &result.encode())
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_intl::{
        FixedTimeZoneOffset, TimeZoneEpochSeconds, TimeZoneNameStyle, TimeZoneSelection,
    };

    use crate::intl_host_probe::IntlHostProbe;

    #[test]
    fn named_lookup_capacity_query_does_not_write_and_retry_may_overlap() {
        let mut probe = IntlHostProbe::new();
        let spelling = b"etc/utc";
        probe.memory.write(&mut probe.store, 128, spelling).unwrap();
        let before = probe.memory.data(&probe.store).to_vec();
        let request = IntlHostReadSpan::new(128, spelling.len() as u32);
        let query = probe
            .invoke(
                IntlHostOp::LookupNamedTimeZone,
                request,
                IntlHostWriteSpan::new(128, 0),
            )
            .unwrap();
        assert_eq!(
            IntlHostCallOutcome::from_wire(query),
            Some(IntlHostCallOutcome::RequiredCapacity(26))
        );
        assert_eq!(probe.memory.data(&probe.store), before);
        let written = probe
            .invoke(
                IntlHostOp::LookupNamedTimeZone,
                request,
                IntlHostWriteSpan::new(128, 26),
            )
            .unwrap();
        assert_eq!(written, 26);
        let bytes = &probe.memory.data(&probe.store)[128..154];
        assert_eq!(u64::from_le_bytes(bytes[..8].try_into().unwrap()), 7);
        assert_eq!(u64::from_le_bytes(bytes[8..16].try_into().unwrap()), 3);
        assert_eq!(&bytes[16..], b"Etc/UTCUTC");
    }

    #[test]
    fn snapshot_capacity_and_overlap_use_owned_request_bytes() {
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
        let request_bytes = ResolveTimeZoneRequest::new(
            TimeZoneSelection::FixedOffset(FixedTimeZoneOffset::from_seconds(86_340).unwrap()),
            TimeZoneEpochSeconds::new(-1).unwrap(),
            Some(TimeZoneNameStyle::LongOffset),
            locale,
        )
        .encode();
        let mut probe = IntlHostProbe::new();
        probe
            .memory
            .write(&mut probe.store, 128, &request_bytes)
            .unwrap();
        let request = IntlHostReadSpan::new(128, request_bytes.len() as u32);
        let before = probe.memory.data(&probe.store).to_vec();
        let query = probe
            .invoke(
                IntlHostOp::ResolveTimeZone,
                request,
                IntlHostWriteSpan::new(128, 1),
            )
            .unwrap();
        let Some(IntlHostCallOutcome::RequiredCapacity(required)) =
            IntlHostCallOutcome::from_wire(query)
        else {
            panic!("unexpected {query}")
        };
        assert_eq!(probe.memory.data(&probe.store), before);
        assert_eq!(
            probe
                .invoke(
                    IntlHostOp::ResolveTimeZone,
                    request,
                    IntlHostWriteSpan::new(128, required)
                )
                .unwrap(),
            i64::from(required)
        );
        let bytes = &probe.memory.data(&probe.store)[128..128 + required as usize];
        assert_eq!(i64::from_le_bytes(bytes[..8].try_into().unwrap()), 86_340);
        assert_eq!(&bytes[8..], b"GMT+23:59");
    }

    #[test]
    fn lookup_rejection_is_distinct_from_malformed_snapshot_and_memory_faults() {
        let mut probe = IntlHostProbe::new();
        for spelling in [b"Missing/Zone".as_slice(), b"\xff", b"+25:00"] {
            probe.memory.write(&mut probe.store, 128, spelling).unwrap();
            let before = probe.memory.data(&probe.store).to_vec();
            assert_eq!(
                probe
                    .invoke(
                        IntlHostOp::LookupNamedTimeZone,
                        IntlHostReadSpan::new(128, spelling.len() as u32),
                        IntlHostWriteSpan::new(128, 1024)
                    )
                    .unwrap(),
                IntlHostCallOutcome::Rejected.wire()
            );
            assert_eq!(probe.memory.data(&probe.store), before);
        }
        assert!(probe
            .invoke(
                IntlHostOp::ResolveTimeZone,
                IntlHostReadSpan::new(128, 8),
                IntlHostWriteSpan::new(128, 1024)
            )
            .is_err());
        assert!(probe
            .invoke(
                IntlHostOp::LookupNamedTimeZone,
                IntlHostReadSpan::new(65_535, 2),
                IntlHostWriteSpan::new(0, 1024)
            )
            .is_err());
        probe.memory.write(&mut probe.store, 128, b"UTC").unwrap();
        assert!(probe
            .invoke(
                IntlHostOp::LookupNamedTimeZone,
                IntlHostReadSpan::new(128, 3),
                IntlHostWriteSpan::new(65_535, 1024)
            )
            .is_err());
    }
}
