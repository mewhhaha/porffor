const ZONED_DATE_TIME_SOURCE: &str = include_str!("../src/builtins/temporal.rs");
const ERA_SOURCE: &str = include_str!("../src/builtins/temporal_plain_date.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

#[test]
fn property_bag_reads_options_after_year_and_before_algorithmic_validation() {
    let property_bag = bounded(
        ZONED_DATE_TIME_SOURCE,
        "    fn emit_temporal_zoned_date_time_from_property_bag(",
        "    fn emit_temporal_regulate_property_bag_date_time(",
    );
    let time_zone_read = "self.strings.payload(\"timeZone\")";
    let time_zone_conversion = "        self.emit_temporal_zoned_date_time_time_zone(";
    let year_conversion = "        self.emit_temporal_property_bag_integer(\n            argument_payload_local,\n            argument_tag_local,\n            \"year\",";
    let options_read = "        self.emit_temporal_zoned_date_time_options(";
    let resolver = "        let resolved_year = self.emit_temporal_resolve_era_to_year(";
    let requires_year = "        self.emit_throw_current_function_realm_type_error(\n            \"Temporal.ZonedDateTime property bag requires year\",";
    let requires_day = "        self.emit_throw_current_function_realm_type_error(\n            \"Temporal.ZonedDateTime property bag requires day\",";
    let requires_time_zone = "        self.emit_throw_current_function_realm_type_error(\n            \"Temporal.ZonedDateTime property bag requires timeZone\",";

    assert_eq!(property_bag.matches(year_conversion).count(), 1);
    assert_eq!(property_bag.matches(options_read).count(), 1);
    assert_eq!(property_bag.matches(resolver).count(), 1);
    for requirement in [requires_year, requires_day, requires_time_zone] {
        assert_eq!(
            property_bag.matches(requirement).count(),
            1,
            "`{requirement}`"
        );
    }
    assert_eq!(property_bag.matches(time_zone_read).count(), 1);
    assert_eq!(property_bag.matches(time_zone_conversion).count(), 1);

    let year_conversion_start = property_bag.find(year_conversion).unwrap();
    let year_conversion_end = year_conversion_start
        + property_bag[year_conversion_start..]
            .find("\n        )?;")
            .expect("year conversion completion")
        + "\n        )?;".len();
    let options_offset = property_bag.find(options_read).unwrap();
    assert!(
        year_conversion_end < options_offset,
        "the options read must start after the year conversion completes"
    );
    assert!(
        !property_bag[year_conversion_end..options_offset].contains("emit_temporal_property_bag_"),
        "year must remain the final property-bag conversion before options"
    );

    assert_before(property_bag, time_zone_read, requires_time_zone);
    assert_before(property_bag, requires_time_zone, time_zone_conversion);
    assert_before(property_bag, time_zone_conversion, year_conversion);
    for later_operation in [resolver, requires_year, requires_day] {
        assert_before(property_bag, options_read, later_operation);
    }
    assert_before(property_bag, resolver, requires_year);
    assert_before(property_bag, requires_year, requires_day);
}

#[test]
fn option_reader_releases_its_five_scratch_locals_in_reverse_reservation_order() {
    let option_reader = bounded(
        ZONED_DATE_TIME_SOURCE,
        "    fn emit_temporal_zoned_date_time_options(",
        "    pub(crate) fn emit_temporal_zoned_date_time_constructor(",
    );
    let reservations = option_reader
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("let ")?
                .strip_suffix(" = self.reserve_temp_local();")
        })
        .collect::<Vec<_>>();
    let releases = option_reader
        .lines()
        .filter_map(|line| {
            line.trim()
                .strip_prefix("self.release_temp_local(")?
                .strip_suffix(");")
        })
        .collect::<Vec<_>>();

    assert_eq!(
        reservations,
        [
            "property_key_local",
            "option_payload_local",
            "option_tag_local",
            "expected_payload_local",
            "recognized_local",
        ]
    );
    assert_eq!(
        releases,
        reservations.iter().rev().copied().collect::<Vec<_>>()
    );
}

#[test]
fn era_slots_stay_live_across_options_until_the_resolver_consumes_them() {
    let property_bag = bounded(
        ZONED_DATE_TIME_SOURCE,
        "    fn emit_temporal_zoned_date_time_from_property_bag(",
        "    fn emit_temporal_regulate_property_bag_date_time(",
    );
    let reservation_prologue = property_bag
        .split_once("        function.instruction(")
        .expect("property-bag reservation prologue")
        .0;
    assert_eq!(
        reservation_prologue
            .matches("self.reserve_temporal_era_slots()")
            .count(),
        1
    );
    assert!(
        reservation_prologue
            .trim_end()
            .ends_with("let era_slots = self.reserve_temporal_era_slots();"),
        "era slots must be the final persistent reservation"
    );

    let options_arguments = bounded(
        property_bag,
        "        self.emit_temporal_zoned_date_time_options(",
        "        )?;",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        options_arguments,
        [
            "options_payload_local,",
            "options_tag_local,",
            "offset_option_local,",
            "overflow_option_local,",
            "function,",
        ]
    );

    let era_read = "        let era = self.emit_temporal_read_era_fields(";
    assert_eq!(property_bag.matches(era_read).count(), 1);
    let era_read_arguments = bounded(property_bag, era_read, "        )?;")
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        era_read_arguments,
        [
            "era_slots,",
            "argument_payload_local,",
            "argument_tag_local,",
            "calendar_payload_local,",
            "function,",
        ]
    );
    assert_before(
        property_bag,
        era_read,
        "        self.emit_temporal_zoned_date_time_options(",
    );
    assert_before(
        property_bag,
        "        self.emit_temporal_zoned_date_time_options(",
        "        let resolved_year = self.emit_temporal_resolve_era_to_year(",
    );

    let option_reader = bounded(
        ZONED_DATE_TIME_SOURCE,
        "    fn emit_temporal_zoned_date_time_options(",
        "    pub(crate) fn emit_temporal_zoned_date_time_constructor(",
    );
    for forbidden_consumer in [
        "TemporalEraSlots",
        "TemporalEraLocals",
        "era_slots",
        "emit_temporal_resolve_era_to_year",
    ] {
        assert!(
            !option_reader.contains(forbidden_consumer),
            "the options reader must not consume `{forbidden_consumer}`"
        );
    }

    let resolver_arguments = bounded(
        property_bag,
        "        let resolved_year = self.emit_temporal_resolve_era_to_year(",
        "        )?;",
    )
    .lines()
    .map(str::trim)
    .filter(|line| !line.is_empty())
    .collect::<Vec<_>>();
    assert_eq!(
        resolver_arguments,
        [
            "era,",
            "calendar_payload_local,",
            "year_local,",
            "present_local,",
            "function,",
        ]
    );

    let resolver = bounded(
        ERA_SOURCE,
        "    pub(crate) fn emit_temporal_resolve_era_to_year(",
        "    pub(crate) fn emit_temporal_resolved_year_default_to(",
    );
    assert!(resolver.contains("era: TemporalEraLocals,"));
    assert!(!resolver.contains("era: &TemporalEraLocals,"));
    let (before_era_locals, _) = ERA_SOURCE
        .split_once("pub(crate) struct TemporalEraLocals {")
        .expect("TemporalEraLocals declaration");
    let era_locals_attributes = before_era_locals
        .rsplit_once("\n\n")
        .expect("TemporalEraLocals attributes")
        .1;
    assert!(era_locals_attributes.contains("#[must_use]"));
    assert!(!era_locals_attributes.contains("derive"));
    assert!(!ERA_SOURCE.contains("impl Clone for TemporalEraLocals"));
    assert!(!ERA_SOURCE.contains("impl Copy for TemporalEraLocals"));
    assert_eq!(
        resolver
            .matches("let TemporalEraLocals {\n            era_payload_local,\n            era_present_local,\n            era_year_local,\n            era_year_present_local,\n        } = era;")
            .count(),
        1
    );
    assert_eq!(
        resolver
            .matches("for local in [\n            era_year_present_local,\n            era_year_local,\n            era_present_local,\n            era_payload_local,\n        ] {\n            self.release_temp_local(local);\n        }")
            .count(),
        1,
        "the consuming resolver must release the era slots in reverse order"
    );
}
