//! Authored pinned-data and synthetic-topology controls, not Test262 results.

use super::super::tests::provider;
use super::*;
use crate::NamedTimeZoneIdentity;
use tzif::data::tzif::{DataBlock, LocalTimeTypeRecord};

fn inverse(id: &str, local_ns: i128) -> PossibleNamedTimeZoneEpochsResult {
    provider()
        .possible_epochs(
            &PossibleNamedTimeZoneEpochsRequest::new(
                TimeZoneId::parse(id).unwrap(),
                LocalTimeCoordinate::from_nanoseconds(local_ns).unwrap(),
            )
            .unwrap(),
        )
        .unwrap()
}
fn transition(id: &str, ns: i128, direction: NamedTimeZoneTransitionDirection) -> Option<i64> {
    provider()
        .find_transition(
            &FindNamedTimeZoneTransitionRequest::new(
                TimeZoneId::parse(id).unwrap(),
                TimeZoneInstant::from_nanoseconds(ns).unwrap(),
                direction,
            )
            .unwrap(),
        )
        .unwrap()
        .instant()
        .map(TimeZoneInstant::seconds)
}
fn candidates(result: &PossibleNamedTimeZoneEpochsResult) -> Vec<(i128, i32)> {
    let PossibleNamedTimeZoneEpochsResult::Candidates(values) = result else {
        panic!("expected candidates: {result:?}");
    };
    values
        .as_slice()
        .iter()
        .map(|value| (value.epoch().nanoseconds(), value.offset().seconds()))
        .collect()
}
fn synthetic(
    records: &[i64],
    times: &[i64],
    types: &[usize],
    footer_offset: i64,
) -> Result<NamedTimeZone, InvalidTimeZoneData> {
    let mut transitions = Tzif::from_bytes(jiff_tzdb::get("UTC").unwrap().1).unwrap();
    transitions.data_block2 = Some(DataBlock {
        local_time_type_records: records
            .iter()
            .map(|&seconds| LocalTimeTypeRecord {
                utoff: Seconds(seconds),
                is_dst: false,
                idx: 0,
            })
            .collect(),
        transition_times: times.iter().copied().map(Seconds).collect(),
        transition_types: types.to_vec(),
        ..DataBlock::default()
    });
    transitions.footer.as_mut().unwrap().std_info.offset = Seconds(-footer_offset);
    NamedTimeZone::from_data(
        NamedTimeZoneIdentity::from_data("UTC", "UTC").unwrap(),
        transitions,
    )
}

#[test]
fn every_explicit_boundary_roundtrips_and_only_actual_gaps_are_empty() {
    for zone in provider().zones.values() {
        let resolved = ResolvedNamedZone::new(zone);
        let block = zone.transitions.get_data_block2().unwrap();
        for t in &block.transition_times {
            let before = resolved.offset(t.0 - 1).unwrap();
            let after = resolved.offset(t.0).unwrap();
            for utc_ns in [
                i128::from(t.0) * NANOS_PER_SECOND - 1,
                i128::from(t.0) * NANOS_PER_SECOND,
                i128::from(t.0) * NANOS_PER_SECOND + 1,
            ] {
                let epoch = RawTimeZoneEpoch::from_nanoseconds(utc_ns).unwrap();
                let offset = resolved.offset(epoch.seconds()).unwrap();
                let local = LocalTimeCoordinate::from_nanoseconds(
                    utc_ns + i128::from(offset.seconds()) * NANOS_PER_SECOND,
                )
                .unwrap();
                let result = resolved.inverse(local).unwrap();
                assert!(
                    candidates(&result).contains(&(utc_ns, offset.seconds())),
                    "{} at {utc_ns}",
                    zone.identity.identifier()
                );
            }
            if after > before {
                let local_ns = i128::from(t.0) * NANOS_PER_SECOND
                    + i128::from(before.seconds()) * NANOS_PER_SECOND;
                let PossibleNamedTimeZoneEpochsResult::Gap(gap) = resolved
                    .inverse(LocalTimeCoordinate::from_nanoseconds(local_ns).unwrap())
                    .unwrap()
                else {
                    panic!("{} gap", zone.identity.identifier());
                };
                assert_eq!(
                    (gap.transition_seconds(), gap.before(), gap.after()),
                    (t.0, before, after)
                );
                // Confirm the constructor certificate at exact nearest ends;
                // Wasm still owns balanced-local inverse and policy checks.
                let lower = local_ns - 1;
                let upper = i128::from(t.0 + i64::from(after.seconds())) * NANOS_PER_SECOND;
                assert_eq!(
                    resolved
                        .candidates(LocalTimeCoordinate::from_nanoseconds(lower).unwrap())
                        .unwrap()
                        .len(),
                    1
                );
                assert_eq!(
                    resolved
                        .candidates(LocalTimeCoordinate::from_nanoseconds(upper).unwrap())
                        .unwrap()
                        .len(),
                    1
                );
            }
        }
    }
}

#[test]
fn historical_second_offsets_aliases_and_negative_fraction_are_exact() {
    assert_eq!(candidates(&inverse("UTC", -1)), [(-1, 0)]);
    assert_eq!(
        candidates(&inverse("Africa/Monrovia", 0)),
        [(2_670_000_000_000, -2670)]
    );
    for local in [0, 1_615_689_000_123_456_789_i128, 1_636_248_600_123_456_789] {
        assert_eq!(
            inverse("America/New_York", local),
            inverse("us/eastern", local)
        );
    }
    let request = NamedTimeZoneOffsetRequest::new(
        TimeZoneId::parse("Africa/Monrovia").unwrap(),
        TimeZoneInstant::from_nanoseconds(-1).unwrap(),
    )
    .unwrap();
    assert_eq!(provider().exact_offset(&request).unwrap().seconds(), -2670);
    assert!(matches!(
        provider().exact_offset(
            &NamedTimeZoneOffsetRequest::new(
                TimeZoneId::parse("Missing/Zone").unwrap(),
                request.instant()
            )
            .unwrap()
        ),
        Err(NamedTimeZoneDataError::UnknownIdentifier(_))
    ));
}

#[test]
fn endpoint_candidates_stay_raw_and_never_become_gaps() {
    let limit = i128::from(TimeZoneInstant::limit_seconds()) * NANOS_PER_SECOND;
    let minimum = inverse("Asia/Tokyo", -limit);
    let maximum = inverse("America/New_York", limit + 1);
    assert_eq!(
        candidates(&minimum),
        [(-limit - 33_539 * NANOS_PER_SECOND, 33_539)]
    );
    assert_eq!(
        candidates(&maximum),
        [(limit + 14_400 * NANOS_PER_SECOND + 1, -14_400)]
    );
    assert!(TimeZoneInstant::from_nanoseconds(candidates(&minimum)[0].0).is_err());
    assert!(TimeZoneInstant::from_nanoseconds(candidates(&maximum)[0].0).is_err());
    // A constructor query keeps its nanosecond even when range-invalid. The
    // response is Candidates; Wasm must reject every such epoch before policy.
    assert_eq!(minimum.encode().len(), 40);
    assert_eq!(maximum.encode().len(), 40);
}

#[test]
fn midnight_skips_and_half_hour_gaps_carry_real_utc_witnesses() {
    for (id, local_seconds, utc, before, after) in [
        (
            "America/Toronto",
            -1_601_769_600_i64,
            -1_601_753_400_i64,
            -18_000,
            -14_400,
        ),
        (
            "Pacific/Apia",
            1_325_203_200,
            1_325_239_200,
            -36_000,
            50_400,
        ),
        (
            "Australia/Lord_Howe",
            1_633_227_300,
            1_633_188_600,
            37_800,
            39_600,
        ),
    ] {
        let PossibleNamedTimeZoneEpochsResult::Gap(gap) =
            inverse(id, i128::from(local_seconds) * NANOS_PER_SECOND)
        else {
            panic!("expected {id} gap");
        };
        assert_eq!(
            (
                gap.transition_seconds(),
                gap.before().seconds(),
                gap.after().seconds()
            ),
            (utc, before, after)
        );
    }
    // Toronto's date starts at 00:30 (the witness T), whereas compatible
    // midnight shifts to 01:00. GetStartOfDay must consume T directly in Wasm.
}

#[test]
fn inverse_represents_three_candidates_and_close_gaps_without_spacing_assumptions() {
    let overlap = synthetic(&[0, -1800, -3600, -1800], &[1000, 2000], &[1, 2], -3600).unwrap();
    assert_eq!(overlap.offsets.len(), 3);
    let result = ResolvedNamedZone::new(&overlap)
        .inverse(LocalTimeCoordinate::new(0, 123).unwrap())
        .unwrap();
    assert_eq!(
        candidates(&result),
        [
            (123, 0),
            (1_800_000_000_123, -1800),
            (3_600_000_000_123, -3600)
        ]
    );
    assert_eq!(result.encode().len(), 16 + 3 * 24);
    let close = synthetic(&[0, 3600, 7200], &[0, 100], &[1, 2], 7200).unwrap();
    let PossibleNamedTimeZoneEpochsResult::Gap(gap) = ResolvedNamedZone::new(&close)
        .inverse(LocalTimeCoordinate::new(3700, 1).unwrap())
        .unwrap()
    else {
        panic!("close gap");
    };
    assert_eq!(
        (
            gap.transition_seconds(),
            gap.before().seconds(),
            gap.after().seconds()
        ),
        (100, 3600, 7200)
    );
}

#[test]
fn transition_direction_is_strict_at_exact_nanosecond_neighbors() {
    use NamedTimeZoneTransitionDirection::{Next, Previous};
    let t = 1_615_705_200_i128 * NANOS_PER_SECOND; // New York 2021 spring
    let previous = transition("America/New_York", t, Previous).unwrap();
    let next = transition("America/New_York", t, Next).unwrap();
    assert!(i128::from(previous) * NANOS_PER_SECOND < t);
    assert!(i128::from(next) * NANOS_PER_SECOND > t);
    assert_eq!(
        transition("America/New_York", t - 1, Next),
        Some(1_615_705_200)
    );
    assert_eq!(
        transition("America/New_York", t + 1, Previous),
        Some(1_615_705_200)
    );
    assert_eq!(
        transition("America/New_York", t - 1, Previous),
        Some(previous)
    );
    assert_eq!(transition("America/New_York", t + 1, Next), Some(next));
    let zero = synthetic(&[0, 3600], &[0], &[1], 3600).unwrap();
    for (ns, direction, expected) in [
        (-1, Next, Some(0)),
        (0, Previous, None),
        (1, Previous, Some(0)),
    ] {
        let request = FindNamedTimeZoneTransitionRequest::new(
            TimeZoneId::parse("UTC").unwrap(),
            TimeZoneInstant::from_nanoseconds(ns).unwrap(),
            direction,
        )
        .unwrap();
        assert_eq!(
            ResolvedNamedZone::new(&zero)
                .transition(&request)
                .unwrap()
                .instant()
                .map(TimeZoneInstant::seconds),
            expected
        );
    }
    // Lisbon's 1992 DST designation change leaves +01 unchanged.
    assert_ne!(
        transition("Europe/Lisbon", 717_555_600_000_000_001, Previous),
        Some(717_555_600)
    );
}

#[test]
fn seasonal_tail_boundaries_roundtrip_and_instant_edges_return_null() {
    use NamedTimeZoneTransitionDirection::{Next, Previous};
    for id in [
        "America/New_York",
        "Australia/Sydney",
        "Australia/Lord_Howe",
        "Europe/Dublin",
    ] {
        let zone = &provider().zones[&id.to_ascii_lowercase()];
        let resolved = ResolvedNamedZone::new(zone);
        let cycle = zone.tail_cycle.as_ref().unwrap();
        for &relative in cycle.relative_seconds() {
            let t = cycle.base_seconds() + relative;
            assert_eq!(
                transition(id, i128::from(t) * NANOS_PER_SECOND - 1, Next),
                Some(t)
            );
            assert_eq!(
                transition(id, i128::from(t) * NANOS_PER_SECOND + 1, Previous),
                Some(t)
            );
            assert_ne!(resolved.offset(t - 1).unwrap(), resolved.offset(t).unwrap());
        }
    }
    let limit = i128::from(TimeZoneInstant::limit_seconds()) * NANOS_PER_SECOND;
    for id in [
        "America/New_York",
        "Australia/Lord_Howe",
        "UTC",
        "Africa/Casablanca",
    ] {
        assert_eq!(transition(id, limit, Next), None);
        assert_eq!(transition(id, -limit, Previous), None);
    }
    assert_eq!(
        transition("Africa/Casablanca", 64_092_204_000_000_000_000, Next),
        None
    );
    assert!(transition("Africa/Casablanca", 64_092_204_000_000_000_000, Previous).is_some());
}

#[test]
fn ambiguous_nearest_gap_endpoint_prevents_catalogue_publication() {
    // The raw jump used to be a valid data-only witness. Its lower endpoint
    // has two inverse epochs, so the stronger catalogue certificate rejects
    // the whole synthetic zone before any gap can become queryable.
    assert!(synthetic(&[0, -100, 100], &[0, 100], &[1, 2], 100).is_err());
}

#[test]
fn conflicting_gap_topology_prevents_catalogue_publication() {
    // Both T0[0,100) and T20[-80,220) contain local50, and the T20 interval is
    // partly filled by another UTC segment. Complete interval certification
    // rejects before either gap can be exposed to a policy consumer.
    assert!(synthetic(&[0, 100, -100, 200], &[0, 10, 20], &[1, 2, 3], 200).is_err());
}

#[test]
fn selected_actual_alias_tzif_keeps_exact_dst_inverse_tail_and_transition_queries() {
    let id = crate::CustomProfileId::parse("selected-exact-zone-queries").unwrap();
    let full =
        crate::NamedTimeZoneDataImage::for_profile(crate::IntlDataProfile::Custom(id.clone()))
            .unwrap();
    let image = crate::NamedTimeZoneDataImage::for_custom_projection(
        &id,
        &[TimeZoneId::parse("America/New_York").unwrap()],
    )
    .unwrap();
    for name in ["America/New_York", "US/Eastern", "Etc/UTC", "UTC"] {
        let identifier = TimeZoneId::parse(name).unwrap();
        for seconds in [-2_208_988_800, 1_615_705_200, 1_636_264_800, 4_102_444_800] {
            let offset = NamedTimeZoneOffsetRequest::new(
                identifier.clone(),
                TimeZoneInstant::new(seconds, 17).unwrap(),
            )
            .unwrap();
            assert_eq!(
                image.zones_ref().exact_offset(&offset).unwrap(),
                full.zones_ref().exact_offset(&offset).unwrap()
            );
            for direction in [
                NamedTimeZoneTransitionDirection::Next,
                NamedTimeZoneTransitionDirection::Previous,
            ] {
                let request = FindNamedTimeZoneTransitionRequest::new(
                    identifier.clone(),
                    offset.instant(),
                    direction,
                )
                .unwrap();
                assert_eq!(
                    image.zones_ref().find_transition(&request).unwrap(),
                    full.zones_ref().find_transition(&request).unwrap()
                );
            }
        }
        for seconds in [1_615_689_000, 1_636_248_600, 4_102_444_800] {
            let request = PossibleNamedTimeZoneEpochsRequest::new(
                identifier.clone(),
                LocalTimeCoordinate::new(seconds, 27).unwrap(),
            )
            .unwrap();
            assert_eq!(
                image.zones_ref().possible_epochs(&request).unwrap(),
                full.zones_ref().possible_epochs(&request).unwrap()
            );
        }
    }
    let unavailable = NamedTimeZoneOffsetRequest::new(
        TimeZoneId::parse("Europe/Paris").unwrap(),
        TimeZoneInstant::new(0, 0).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        image.zones_ref().exact_offset(&unavailable),
        Err(NamedTimeZoneDataError::UnavailableIdentifier(_))
    ));
    let unknown = NamedTimeZoneOffsetRequest::new(
        TimeZoneId::parse("Unknown/Zone").unwrap(),
        TimeZoneInstant::new(0, 0).unwrap(),
    )
    .unwrap();
    assert!(matches!(
        image.zones_ref().exact_offset(&unknown),
        Err(NamedTimeZoneDataError::UnknownIdentifier(_))
    ));
}
