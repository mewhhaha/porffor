const DTF_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat.rs");
const INITIALIZATION_SOURCE: &str =
    include_str!("../src/builtins/intl_datetimeformat/initialization.rs");
const ZONE_SOURCE: &str = include_str!("../src/builtins/intl_datetimeformat/time_zone.rs");
const DOMAIN_SOURCE: &str = include_str!("../../lila-intl/src/time_zone.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap_intl_date_time_format_layout.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing `{end}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

#[test]
fn named_identifier_and_fixed_offset_authorities_are_shared_validated_domains() {
    for obsolete in [
        "TzOffsetMinutes",
        "IntlDtfNamedZone",
        "INTL_DTF_NAMED_ZONES",
    ] {
        assert!(
            !DTF_SOURCE.contains(obsolete),
            "obsolete authority `{obsolete}`"
        );
    }
    let identity = bounded(
        DOMAIN_SOURCE,
        "pub struct NamedTimeZoneIdentity {",
        "impl NamedTimeZoneIdentity {",
    );
    assert!(identity.contains("identifier: TimeZoneId"));
    assert!(identity.contains("primary_identifier: TimeZoneId"));
    assert!(!identity.contains("pub "));
    let fixed = normalized(bounded(
        DOMAIN_SOURCE,
        "impl FixedTimeZoneOffset {",
        "/// Exact input domain",
    ));
    assert!(fixed.contains("pubconstMAX_HOUR:i64=23;"));
    assert!(fixed.contains("pubconstMAX_MINUTE:i64=59;"));
    assert!(fixed.contains("seconds.unsigned_abs()>Self::MAX_SECONDS||seconds%60!=0"));
    assert!(DTF_SOURCE.contains("FixedTimeZoneOffset::MAX_HOUR"));
    assert!(DTF_SOURCE.contains("FixedTimeZoneOffset::MAX_MINUTE"));
    assert!(ZONE_SOURCE.contains("IntlHostOp::LookupNamedTimeZone"));
    assert!(!ZONE_SOURCE.contains("IntlHostOp::ResolveTimeZone"));
}

#[test]
fn only_resolved_constructor_output_can_publish_all_zone_slots() {
    let lifecycle = normalized(bounded(
        DTF_SOURCE,
        "struct DtfCanonicalTimeZone {",
        "impl FunctionBuilder<'_> {",
    ))
    .replace(",)", ")");
    assert!(!lifecycle.contains("derive(Clone"));
    assert!(!lifecycle.contains("derive(Copy"));
    assert!(!normalized(bounded(
        DTF_SOURCE,
        "impl DtfCanonicalTimeZone {",
        "struct DtfResolvedTimeZone("
    ))
    .contains("fnstore("));
    for invariant in [
        "structDtfResolvedTimeZone(DtfCanonicalTimeZone);",
        "fnstore(&self,",
        "fnrelease(self,",
        "(HEAP_INTL_DTF_TIME_ZONE_OFFSET,self.0.identifier_local)",
        "(HEAP_INTL_DTF_TIME_ZONE_FIXED_SECONDS_OFFSET,self.0.fixed_seconds_local)",
        "(HEAP_INTL_DTF_TIME_ZONE_KIND_OFFSET,self.0.kind_local)",
    ] {
        assert!(lifecycle.contains(invariant), "missing `{invariant}`");
    }
    let resolver = normalized(bounded(
        DTF_SOURCE,
        "fn emit_intl_dtf_time_zone_option(",
        "fn emit_intl_dtf_parse_utc_offset(",
    ));
    assert!(resolver.contains("zone:DtfCanonicalTimeZone,"));
    assert!(resolver.contains(")->Result<DtfResolvedTimeZone,EmitError>{"));
    assert_eq!(resolver.matches("Ok(DtfResolvedTimeZone(zone))").count(), 1);
    assert_eq!(
        INITIALIZATION_SOURCE
            .matches("time_zone.store(self, record, function);")
            .count(),
        1
    );
    assert_eq!(
        INITIALIZATION_SOURCE
            .matches("time_zone.release(self);")
            .count(),
        1
    );
}

#[test]
fn provider_resolves_exact_endpoints_and_zone_kind_remains_an_untraced_scalar() {
    let input = include_str!("../src/builtins/intl_datetimeformat/provider_input.rs");
    assert!(input.contains("self.emit_temporal_epoch_nanoseconds_pair("));
    assert!(!input.contains("IntlHostOp::ResolveTimeZone"));
    let heap = normalized(HEAP_SOURCE);
    assert!(heap.contains("TimeZoneFixedSeconds"));
    assert!(heap.contains("TimeZoneKind"));
    assert!(!heap.contains("TimeZoneGmtNamePayload"));
    assert!(heap.contains(
        "name:\"time_zone_kind\",offset:HEAP_INTL_DTF_TIME_ZONE_KIND_OFFSET,width:8,pointer:false"
    ));
}
