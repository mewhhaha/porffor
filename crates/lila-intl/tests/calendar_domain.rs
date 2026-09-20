use icu_calendar::{
    cal::{Chinese, Dangi},
    provider::{
        chinese_based::{ChineseBasedCache, PackedChineseBasedYearInfo},
        Baked, CalendarChineseV1, CalendarDangiV1,
    },
    types::{CyclicYear, MonthCode, RataDie},
    Calendar, Date, DateDuration, Ref,
};
use icu_provider::prelude::*;
use zerovec::ZeroVec;

#[path = "calendar_domain/legacy_goldens.rs"]
mod legacy_goldens;

fn first_day<C: Calendar<Year = CyclicYear>>(calendar: &C, year: i32) -> Date<Ref<'_, C>> {
    Date::try_new_from_codes(
        None,
        year,
        MonthCode("M01".parse().unwrap()),
        1,
        Ref(calendar),
    )
    .expect("calendar first day")
}

fn assert_date<C: Calendar<Year = CyclicYear>>(calendar: &C, fixed: i64) {
    let date = Date::from_rata_die(RataDie::new(fixed), Ref(calendar));
    let year = date.cyclic_year();
    let month = date.month();
    let day = date.day_of_month().0;
    assert!((1..=60).contains(&year.year));
    assert!((12..=13).contains(&date.months_in_year()));
    assert!((1..=date.months_in_year()).contains(&month.ordinal));
    assert!(matches!(date.days_in_month(), 29 | 30));
    assert!((1..=date.days_in_month()).contains(&day));
    assert_eq!(date.to_iso().to_rata_die().to_i64_date(), fixed);
    let decoded = Date::try_new_from_codes(
        None,
        year.related_iso,
        month.standard_code,
        day,
        Ref(calendar),
    )
    .expect("converted fields construct the same date");
    assert_eq!(decoded.to_rata_die().to_i64_date(), fixed);
    for days in [-1, 1] {
        let adjacent = date.added(DateDuration::new(0, 0, 0, days));
        assert_eq!(
            adjacent.to_rata_die().to_i64_date(),
            fixed + i64::from(days)
        );
        let from_fixed = Date::from_rata_die(RataDie::new(fixed + i64::from(days)), Ref(calendar));
        assert_eq!(adjacent, from_fixed);
    }
}

fn check_extremes<C: Calendar<Year = CyclicYear>>(calendar: &C) {
    for (iso, fixed) in [
        ((-271821, 4, 19), -99_280_838),
        ((-271821, 4, 20), -99_280_837),
        ((-10000, 1, 1), -3_652_790),
        ((-2332, 3, 1), -852_050),
        ((0, 1, 1), -365),
        ((4000, 1, 1), 1_460_605),
        ((10000, 1, 1), 3_652_060),
        ((275760, 9, 13), 100_719_163),
    ] {
        assert_eq!(
            Date::try_new_iso(iso.0, iso.1, iso.2)
                .unwrap()
                .to_rata_die()
                .to_i64_date(),
            fixed
        );
        assert_date(calendar, fixed);
    }
}

#[test]
fn extreme_iso_dates_keep_valid_calendar_fields_in_both_constructors() {
    check_extremes(&Chinese::new());
    check_extremes(&Chinese::new_always_calculating());
    check_extremes(&Dangi::new());
    check_extremes(&Dangi::new_always_calculating());
}

#[test]
fn published_modern_chinese_leap_month_and_new_year_dates_are_preserved() {
    // The first three are pinned Test262 goldens; 2023/2024 come from the
    // pinned Hong Kong Observatory tables recorded in the calendar design.
    for calendar in [Chinese::new(), Chinese::new_always_calculating()] {
        for (iso, related, code, day) in [
            ((1900, 1, 1), 1899, "M12", 1),
            ((2000, 1, 1), 1999, "M11", 25),
            ((2100, 1, 1), 2099, "M11", 21),
            ((2023, 3, 21), 2023, "M02", 30),
            ((2023, 3, 22), 2023, "M02L", 1),
            ((2023, 4, 19), 2023, "M02L", 29),
            ((2023, 4, 20), 2023, "M03", 1),
            ((2024, 2, 9), 2023, "M12", 30),
            ((2024, 2, 10), 2024, "M01", 1),
        ] {
            let iso = Date::try_new_iso(iso.0, iso.1, iso.2).unwrap();
            let date = iso.to_calendar(Ref(&calendar));
            assert_eq!(date.cyclic_year().related_iso, related);
            assert_eq!(date.month().standard_code.0.as_str(), code);
            assert_eq!(date.day_of_month().0, day);
            assert_eq!(date.to_iso(), iso);
        }
    }
}

fn check_joins<C: Calendar<Year = CyclicYear>>(calendar: &C) {
    for (year, expected_fixed) in [(-3653, -1_334_565), (4704, 1_717_776)] {
        assert_eq!(
            first_day(calendar, year).to_rata_die().to_i64_date(),
            expected_fixed
        );
    }
    for year in [-3653, 1900, 2150, 4704] {
        let boundary = first_day(calendar, year).to_rata_die().to_i64_date();
        for fixed in boundary - 35..=boundary + 35 {
            assert_date(calendar, fixed);
        }
    }
}

#[test]
fn both_model_joins_and_both_cache_edges_are_reversible_day_by_day() {
    check_joins(&Chinese::new());
    check_joins(&Chinese::new_always_calculating());
    check_joins(&Dangi::new());
    check_joins(&Dangi::new_always_calculating());
}

fn year_signature<C: Calendar<Year = CyclicYear>>(
    first: &Date<Ref<'_, C>>,
) -> Vec<(MonthCode, u8)> {
    (0..first.months_in_year())
        .map(|ordinal| {
            let date = (*first).added(DateDuration::new(0, i32::from(ordinal), 0, 0));
            (date.month().standard_code, date.days_in_month())
        })
        .collect()
}

fn check_retained_interval<C: Calendar<Year = CyclicYear>>(cached: &C, calculated: &C) {
    let mut previous_end = None;
    for year in -3654..=4705 {
        let first = first_day(calculated, year);
        let fixed = first.to_rata_die().to_i64_date();
        if let Some(previous_end) = previous_end {
            assert_eq!(previous_end, fixed, "retained interval join {year}");
        }
        previous_end = Some(fixed + i64::from(first.days_in_year()));
        let cached_first = first_day(cached, year);
        assert_eq!(
            cached_first.to_rata_die(),
            first.to_rata_die(),
            "cache start {year}"
        );
        assert_eq!(
            year_signature(&cached_first),
            year_signature(&first),
            "cache fields {year}"
        );
    }
}

#[test]
fn the_entire_retained_interval_and_baked_caches_share_exact_year_records() {
    check_retained_interval(&Chinese::new(), &Chinese::new_always_calculating());
    check_retained_interval(&Dangi::new(), &Dangi::new_always_calculating());
}

fn check_full_domain<C: Calendar<Year = CyclicYear>>(calendar: &C) {
    let mut previous_end = None;
    for year in -271_822..=275_761 {
        let first = first_day(calendar, year);
        let fixed = first.to_rata_die().to_i64_date();
        if let Some(previous_end) = previous_end {
            assert_eq!(previous_end, fixed, "full-domain join {year}");
        }
        let mut total = 0_u16;
        let signature = year_signature(&first);
        assert!(matches!(signature.len(), 12 | 13));
        for (_, days) in signature {
            assert!(matches!(days, 29 | 30));
            total += u16::from(days);
        }
        assert_eq!(first.days_in_year(), total);
        assert_eq!(first.day_of_month().0, 1);
        assert_eq!(first.cyclic_year().related_iso, year);
        previous_end = Some(fixed + i64::from(total));
    }
}

#[test]
fn chinese_canonical_api_has_continuous_years_across_the_full_temporal_domain() {
    check_full_domain(&Chinese::new_always_calculating());
}

#[test]
fn dangi_canonical_api_has_continuous_years_across_the_full_temporal_domain() {
    check_full_domain(&Dangi::new_always_calculating());
}

struct CalendarCacheProvider(ChineseBasedCache<'static>);

impl DataProvider<CalendarChineseV1> for CalendarCacheProvider {
    fn load(&self, _: DataRequest) -> Result<DataResponse<CalendarChineseV1>, DataError> {
        Ok(DataResponse {
            metadata: Default::default(),
            payload: DataPayload::from_owned(self.0.clone()),
        })
    }
}

impl DataProvider<CalendarDangiV1> for CalendarCacheProvider {
    fn load(&self, _: DataRequest) -> Result<DataResponse<CalendarDangiV1>, DataError> {
        Ok(DataResponse {
            metadata: Default::default(),
            payload: DataPayload::from_owned(self.0.clone()),
        })
    }
}

#[test]
fn custom_provider_corruption_is_rejected_at_calendar_construction() {
    Chinese::try_new_unstable(&Baked).expect("valid Chinese baked cache");
    Dangi::try_new_unstable(&Baked).expect("valid Dangi baked cache");
    let response: DataResponse<CalendarChineseV1> = Baked.load(Default::default()).unwrap();
    let cache = response.payload.get();
    let packed = cache
        .data
        .get((2023 - cache.first_related_iso_year) as usize)
        .unwrap();
    let valid_provider = CalendarCacheProvider(ChineseBasedCache {
        first_related_iso_year: 2023,
        data: ZeroVec::alloc_from_slice(&[packed]),
    });
    Chinese::try_new_unstable(&valid_provider).expect("valid one-year cache");
    for corrupt in [
        PackedChineseBasedYearInfo(packed.0, packed.1, 255),
        PackedChineseBasedYearInfo(packed.0, (packed.1 & 31) | 32, packed.2 & !1),
        PackedChineseBasedYearInfo(packed.0, packed.1, packed.2 + 2),
    ] {
        let provider = CalendarCacheProvider(ChineseBasedCache {
            first_related_iso_year: 2023,
            data: ZeroVec::alloc_from_slice(&[corrupt]),
        });
        assert!(Chinese::try_new_unstable(&provider).is_err());
    }
}

#[test]
fn dangi_custom_provider_corruption_is_rejected_at_calendar_construction() {
    let response: DataResponse<CalendarDangiV1> = Baked.load(Default::default()).unwrap();
    let cache = response.payload.get();
    let packed = cache
        .data
        .get((2023 - cache.first_related_iso_year) as usize)
        .unwrap();
    let valid_provider = CalendarCacheProvider(ChineseBasedCache {
        first_related_iso_year: 2023,
        data: ZeroVec::alloc_from_slice(&[packed]),
    });
    Dangi::try_new_unstable(&valid_provider).expect("valid one-year Dangi cache");
    for corrupt in [
        PackedChineseBasedYearInfo(packed.0, packed.1, 255),
        PackedChineseBasedYearInfo(packed.0, (packed.1 & 31) | 32, packed.2 & !1),
        PackedChineseBasedYearInfo(packed.0, packed.1, packed.2 + 2),
    ] {
        let provider = CalendarCacheProvider(ChineseBasedCache {
            first_related_iso_year: 2023,
            data: ZeroVec::alloc_from_slice(&[corrupt]),
        });
        assert!(Dangi::try_new_unstable(&provider).is_err());
    }
}
