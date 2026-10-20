// Adapted from the pinned ICU4X vendor tests. See vendor/icu_calendar-2.0.6/LICENSE.
// Retained Chinese/Dangi assertions from the pinned ICU4X vendor source.
// Expected values and assertions are unchanged; only private-access calls use public APIs.
#![allow(unused_imports)]
pub(super) mod chinese {
    use calendrical_calculations::{iso::fixed_from_iso, rata_die::RataDie};
    use icu_calendar::{
        cal::{Chinese, Dangi},
        types::MonthCode,
        Date, Iso, Ref,
    };
    use tinystr::tinystr;
    fn do_twice(
        chinese_calculating: &Chinese,
        chinese_cached: &Chinese,
        test: impl Fn(Ref<Chinese>, &'static str),
    ) {
        test(Ref(chinese_calculating), "calculating");
        test(Ref(chinese_cached), "cached");
    }
    #[test]
    pub(crate) fn test_chinese_from_rd() {
        #[derive(Debug)]
        struct TestCase {
            rd: i64,
            expected_year: i32,
            expected_month: u8,
            expected_day: u8,
        }

        let cases = [
            TestCase {
                rd: -964192,
                expected_year: -2,
                expected_month: 1,
                expected_day: 1,
            },
            TestCase {
                rd: -963838,
                expected_year: -1,
                expected_month: 1,
                expected_day: 1,
            },
            TestCase {
                rd: -963129,
                expected_year: 0,
                expected_month: 13,
                expected_day: 1,
            },
            TestCase {
                rd: -963100,
                expected_year: 0,
                expected_month: 13,
                expected_day: 30,
            },
            TestCase {
                rd: -963099,
                expected_year: 1,
                expected_month: 1,
                expected_day: 1,
            },
            TestCase {
                rd: 738700,
                expected_year: 4660,
                expected_month: 6,
                expected_day: 12,
            },
            TestCase {
                rd: fixed_from_iso(2319, 2, 20).to_i64_date(),
                expected_year: 2319 + 2636,
                expected_month: 13,
                expected_day: 30,
            },
            TestCase {
                rd: fixed_from_iso(2319, 2, 21).to_i64_date(),
                expected_year: 2319 + 2636 + 1,
                expected_month: 1,
                expected_day: 1,
            },
            TestCase {
                rd: 738718,
                expected_year: 4660,
                expected_month: 6,
                expected_day: 30,
            },
            TestCase {
                rd: 738747,
                expected_year: 4660,
                expected_month: 7,
                expected_day: 29,
            },
            TestCase {
                rd: 738748,
                expected_year: 4660,
                expected_month: 8,
                expected_day: 1,
            },
            TestCase {
                rd: 738865,
                expected_year: 4660,
                expected_month: 11,
                expected_day: 29,
            },
            TestCase {
                rd: 738895,
                expected_year: 4660,
                expected_month: 12,
                expected_day: 29,
            },
            TestCase {
                rd: 738925,
                expected_year: 4660,
                expected_month: 13,
                expected_day: 30,
            },
        ];

        let chinese_calculating = Chinese::new_always_calculating();
        let chinese_cached = Chinese::new();
        for case in cases {
            let rata_die = RataDie::new(case.rd);

            do_twice(
                &chinese_calculating,
                &chinese_cached,
                |chinese, calendar_type| {
                    let chinese = Date::from_rata_die(rata_die, chinese);
                    assert_eq!(
                        case.expected_year,
                        chinese.extended_year(),
                        "[{calendar_type}] Chinese from RD failed, case: {case:?}"
                    );
                    assert_eq!(
                        case.expected_month,
                        chinese.month().ordinal,
                        "[{calendar_type}] Chinese from RD failed, case: {case:?}"
                    );
                    assert_eq!(
                        case.expected_day,
                        chinese.day_of_month().0,
                        "[{calendar_type}] Chinese from RD failed, case: {case:?}"
                    );
                },
            );
        }
    }
    #[test]
    pub(crate) fn test_rd_from_chinese() {
        #[derive(Debug)]
        struct TestCase {
            year: i32,
            month: u8,
            day: u8,
            expected: i64,
        }

        let cases = [
            TestCase {
                year: 2023,
                month: 6,
                day: 6,
                // June 23 2023
                expected: 738694,
            },
            TestCase {
                year: -2636,
                month: 1,
                day: 1,
                expected: -963099,
            },
        ];

        let chinese_calculating = Chinese::new_always_calculating();
        let chinese_cached = Chinese::new();
        for case in cases {
            do_twice(
                &chinese_calculating,
                &chinese_cached,
                |chinese, calendar_type| {
                    let date = Date::try_new_chinese_with_calendar(
                        case.year, case.month, case.day, chinese,
                    )
                    .unwrap();
                    let rd = date.to_rata_die().to_i64_date();
                    let expected = case.expected;
                    assert_eq!(rd, expected, "[{calendar_type}] RD from Chinese failed, with expected: {expected} and calculated: {rd}, for test case: {case:?}");
                },
            );
        }
    }
    #[test]
    pub(crate) fn test_rd_chinese_roundtrip() {
        let mut rd = -1963020;
        let max_rd = 1963020;
        let mut iters = 0;
        let max_iters = 560;
        let chinese_calculating = Chinese::new_always_calculating();
        let chinese_cached = Chinese::new();
        while rd < max_rd && iters < max_iters {
            let rata_die = RataDie::new(rd);

            do_twice(
                &chinese_calculating,
                &chinese_cached,
                |chinese, calendar_type| {
                    let chinese = Date::from_rata_die(rata_die, chinese);
                    let result = chinese.to_rata_die();
                    assert_eq!(result, rata_die, "[{calendar_type}] Failed roundtrip RD -> Chinese -> RD for RD: {rata_die:?}, with calculated: {result:?} from Chinese date:\n{chinese:?}");
                },
            );
            rd += 7043;
            iters += 1;
        }
    }
    #[test]
    pub(crate) fn test_chinese_epoch() {
        let iso = Date::try_new_iso(-2636, 2, 15).unwrap();

        do_twice(
            &Chinese::new_always_calculating(),
            &Chinese::new(),
            |chinese, _calendar_type| {
                let chinese = iso.to_calendar(chinese);

                assert_eq!(chinese.cyclic_year().related_iso, -2636);
                assert_eq!(chinese.month().ordinal, 1);
                assert_eq!(chinese.month().standard_code.0, "M01");
                assert_eq!(chinese.day_of_month().0, 1);
                assert_eq!(chinese.cyclic_year().year, 1);
                assert_eq!(chinese.cyclic_year().related_iso, -2636);
            },
        )
    }
    #[test]
    pub(crate) fn test_iso_to_chinese_negative_years() {
        #[derive(Debug)]
        struct TestCase {
            iso_year: i32,
            iso_month: u8,
            iso_day: u8,
            expected_year: i32,
            expected_month: u8,
            expected_day: u8,
        }

        let cases = [
            TestCase {
                iso_year: -2636,
                iso_month: 2,
                iso_day: 14,
                expected_year: -2637,
                expected_month: 13,
                expected_day: 30,
            },
            TestCase {
                iso_year: -2636,
                iso_month: 1,
                iso_day: 15,
                expected_year: -2637,
                expected_month: 12,
                expected_day: 30,
            },
        ];

        let chinese_calculating = Chinese::new_always_calculating();
        let chinese_cached = Chinese::new();

        for case in cases {
            let iso = Date::try_new_iso(case.iso_year, case.iso_month, case.iso_day).unwrap();
            do_twice(
                &chinese_calculating,
                &chinese_cached,
                |chinese, calendar_type| {
                    let chinese = iso.to_calendar(chinese);
                    assert_eq!(
                        case.expected_year,
                        chinese.cyclic_year().related_iso,
                        "[{calendar_type}] ISO to Chinese failed for case: {case:?}"
                    );
                    assert_eq!(
                        case.expected_month,
                        chinese.month().ordinal,
                        "[{calendar_type}] ISO to Chinese failed for case: {case:?}"
                    );
                    assert_eq!(
                        case.expected_day,
                        chinese.day_of_month().0,
                        "[{calendar_type}] ISO to Chinese failed for case: {case:?}"
                    );
                },
            );
        }
    }
    #[test]
    pub(crate) fn test_chinese_leap_months() {
        let expected = [
            (1933, 6),
            (1938, 8),
            (1984, 11),
            (2009, 6),
            (2017, 7),
            (2028, 6),
        ];
        let chinese_calculating = Chinese::new_always_calculating();
        let chinese_cached = Chinese::new();

        for case in expected {
            let year = case.0;
            let expected_month = case.1;
            let iso = Date::try_new_iso(year, 6, 1).unwrap();
            do_twice(
                &chinese_calculating,
                &chinese_cached,
                |chinese, calendar_type| {
                    let chinese_date = iso.to_calendar(chinese);
                    assert!(
                        chinese_date.is_in_leap_year(),
                        "[{calendar_type}] {year} should be a leap year"
                    );
                    let new_year = Date::try_new_chinese_with_calendar(year, 1, 1, chinese)
                        .unwrap()
                        .to_rata_die();
                    assert_eq!(
                        expected_month,
                        calendrical_calculations::chinese_based::get_leap_month_from_new_year::<
                            calendrical_calculations::chinese_based::Chinese,
                        >(new_year),
                        "[{calendar_type}] {year} have leap month {expected_month}"
                    );
                },
            );
        }
    }
    #[test]
    pub(crate) fn test_month_days() {
        let year = 2023;
        let calendar = Chinese::new_always_calculating();
        let cases = [
            (1, 29),
            (2, 30),
            (3, 29),
            (4, 29),
            (5, 30),
            (6, 30),
            (7, 29),
            (8, 30),
            (9, 30),
            (10, 29),
            (11, 30),
            (12, 29),
            (13, 30),
        ];
        for case in cases {
            let days_in_month =
                Date::try_new_chinese_with_calendar(year, case.0, 1, Ref(&calendar))
                    .unwrap()
                    .days_in_month();
            assert_eq!(
                case.1, days_in_month,
                "month_days test failed for case: {case:?}"
            );
        }
    }
    #[test]
    pub(crate) fn test_ordinal_to_month_code() {
        #[derive(Debug)]
        struct TestCase {
            year: i32,
            month: u8,
            day: u8,
            expected_code: &'static str,
        }

        let cases = [
            TestCase {
                year: 2023,
                month: 1,
                day: 9,
                expected_code: "M12",
            },
            TestCase {
                year: 2023,
                month: 2,
                day: 9,
                expected_code: "M01",
            },
            TestCase {
                year: 2023,
                month: 3,
                day: 9,
                expected_code: "M02",
            },
            TestCase {
                year: 2023,
                month: 4,
                day: 9,
                expected_code: "M02L",
            },
            TestCase {
                year: 2023,
                month: 5,
                day: 9,
                expected_code: "M03",
            },
            TestCase {
                year: 2023,
                month: 6,
                day: 9,
                expected_code: "M04",
            },
            TestCase {
                year: 2023,
                month: 7,
                day: 9,
                expected_code: "M05",
            },
            TestCase {
                year: 2023,
                month: 8,
                day: 9,
                expected_code: "M06",
            },
            TestCase {
                year: 2023,
                month: 9,
                day: 9,
                expected_code: "M07",
            },
            TestCase {
                year: 2023,
                month: 10,
                day: 9,
                expected_code: "M08",
            },
            TestCase {
                year: 2023,
                month: 11,
                day: 9,
                expected_code: "M09",
            },
            TestCase {
                year: 2023,
                month: 12,
                day: 9,
                expected_code: "M10",
            },
            TestCase {
                year: 2024,
                month: 1,
                day: 9,
                expected_code: "M11",
            },
            TestCase {
                year: 2024,
                month: 2,
                day: 9,
                expected_code: "M12",
            },
            TestCase {
                year: 2024,
                month: 2,
                day: 10,
                expected_code: "M01",
            },
        ];

        let chinese_calculating = Chinese::new_always_calculating();
        let chinese_cached = Chinese::new();

        for case in cases {
            let iso = Date::try_new_iso(case.year, case.month, case.day).unwrap();
            do_twice(
                &chinese_calculating,
                &chinese_cached,
                |chinese, calendar_type| {
                    let chinese = iso.to_calendar(chinese);
                    let result_code = chinese.month().standard_code.0;
                    let expected_code = case.expected_code.to_string();
                    assert_eq!(
                        expected_code, result_code,
                        "[{calendar_type}] Month codes did not match for test case: {case:?}"
                    );
                },
            );
        }
    }
    #[test]
    pub(crate) fn test_month_code_to_ordinal() {
        // construct using ::default() to force recomputation
        let year = 2023;
        let calendar = Chinese::new_always_calculating();
        let codes = [
            (1, tinystr!(4, "M01")),
            (2, tinystr!(4, "M02")),
            (3, tinystr!(4, "M02L")),
            (4, tinystr!(4, "M03")),
            (5, tinystr!(4, "M04")),
            (6, tinystr!(4, "M05")),
            (7, tinystr!(4, "M06")),
            (8, tinystr!(4, "M07")),
            (9, tinystr!(4, "M08")),
            (10, tinystr!(4, "M09")),
            (11, tinystr!(4, "M10")),
            (12, tinystr!(4, "M11")),
            (13, tinystr!(4, "M12")),
        ];
        for ordinal_code_pair in codes {
            let code = MonthCode(ordinal_code_pair.1);
            let ordinal = Date::try_new_from_codes(None, year, code, 1, Ref(&calendar))
                .ok()
                .map(|date| date.month().ordinal);
            assert_eq!(
                ordinal,
                Some(ordinal_code_pair.0),
                "Code to ordinal failed for year: {}, code: {code}",
                year
            );
        }
    }
    #[test]
    pub(crate) fn check_invalid_month_code_to_ordinal() {
        let non_leap_year = 4659;
        let leap_year = 4660;
        let invalid_codes = [
            (non_leap_year, tinystr!(4, "M2")),
            (leap_year, tinystr!(4, "M0")),
            (non_leap_year, tinystr!(4, "J01")),
            (leap_year, tinystr!(4, "3M")),
            (non_leap_year, tinystr!(4, "M04L")),
            (leap_year, tinystr!(4, "M04L")),
            (non_leap_year, tinystr!(4, "M13")),
            (leap_year, tinystr!(4, "M13")),
        ];
        for (year, code) in invalid_codes {
            // construct using ::default() to force recomputation
            let calendar = Chinese::new_always_calculating();
            let code = MonthCode(code);
            let ordinal = Date::try_new_from_codes(None, year, code, 1, Ref(&calendar))
                .ok()
                .map(|date| date.month().ordinal);
            assert_eq!(
                ordinal, None,
                "Invalid month code failed for year: {}, code: {code}",
                year
            );
        }
    }
    #[test]
    pub(crate) fn test_iso_chinese_roundtrip() {
        let chinese_calculating = Chinese::new_always_calculating();
        let chinese_cached = Chinese::new();

        for i in -1000..=1000 {
            let year = i;
            let month = i as u8 % 12 + 1;
            let day = i as u8 % 28 + 1;
            let iso = Date::try_new_iso(year, month, day).unwrap();
            do_twice(
                &chinese_calculating,
                &chinese_cached,
                |chinese, calendar_type| {
                    let chinese = iso.to_calendar(chinese);
                    let result = chinese.to_calendar(Iso);
                    assert_eq!(iso, result, "[{calendar_type}] ISO to Chinese roundtrip failed!\nIso: {iso:?}\nChinese: {chinese:?}\nResult: {result:?}");
                },
            );
        }
    }
    #[test]
    pub(crate) fn test_consistent_with_icu() {
        #[derive(Debug)]
        struct TestCase {
            iso_year: i32,
            iso_month: u8,
            iso_day: u8,
            expected_rel_iso: i32,
            expected_cyclic: u8,
            expected_month: u8,
            expected_day: u8,
        }

        let cases = [
            TestCase {
                iso_year: -2332,
                iso_month: 3,
                iso_day: 1,
                expected_rel_iso: -2332,
                expected_cyclic: 5,
                expected_month: 1,
                expected_day: 16,
            },
            TestCase {
                iso_year: -2332,
                iso_month: 2,
                iso_day: 15,
                expected_rel_iso: -2332,
                expected_cyclic: 5,
                expected_month: 1,
                expected_day: 1,
            },
            TestCase {
                // This test case fails to match ICU
                iso_year: -2332,
                iso_month: 2,
                iso_day: 14,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 13,
                expected_day: 30,
            },
            TestCase {
                // This test case fails to match ICU
                iso_year: -2332,
                iso_month: 1,
                iso_day: 17,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 13,
                expected_day: 2,
            },
            TestCase {
                // This test case fails to match ICU
                iso_year: -2332,
                iso_month: 1,
                iso_day: 16,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 13,
                expected_day: 1,
            },
            TestCase {
                iso_year: -2332,
                iso_month: 1,
                iso_day: 15,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 12,
                expected_day: 29,
            },
            TestCase {
                iso_year: -2332,
                iso_month: 1,
                iso_day: 1,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 12,
                expected_day: 15,
            },
            TestCase {
                iso_year: -2333,
                iso_month: 1,
                iso_day: 16,
                expected_rel_iso: -2334,
                expected_cyclic: 3,
                expected_month: 12,
                expected_day: 19,
            },
        ];

        let chinese_calculating = Chinese::new_always_calculating();
        let chinese_cached = Chinese::new();

        for case in cases {
            let iso = Date::try_new_iso(case.iso_year, case.iso_month, case.iso_day).unwrap();

            do_twice(
                &chinese_calculating,
                &chinese_cached,
                |chinese, calendar_type| {
                    let chinese = iso.to_calendar(chinese);
                    let chinese_rel_iso = chinese.cyclic_year().related_iso;
                    let chinese_cyclic = chinese.cyclic_year().year;
                    let chinese_month = chinese.month().ordinal;
                    let chinese_day = chinese.day_of_month().0;

                    assert_eq!(
                        chinese_rel_iso, case.expected_rel_iso,
                        "[{calendar_type}] Related ISO failed for test case: {case:?}"
                    );
                    assert_eq!(
                        chinese_cyclic, case.expected_cyclic,
                        "[{calendar_type}] Cyclic year failed for test case: {case:?}"
                    );
                    assert_eq!(
                        chinese_month, case.expected_month,
                        "[{calendar_type}] Month failed for test case: {case:?}"
                    );
                    assert_eq!(
                        chinese_day, case.expected_day,
                        "[{calendar_type}] Day failed for test case: {case:?}"
                    );
                },
            );
        }
    }
}

pub(super) mod dangi {
    use calendrical_calculations::{iso::fixed_from_iso, rata_die::RataDie};
    use icu_calendar::{
        cal::{Chinese, Dangi},
        types::MonthCode,
        Date, Iso, Ref,
    };
    use tinystr::tinystr;
    fn do_twice(
        dangi_calculating: &Dangi,
        dangi_cached: &Dangi,
        test: impl Fn(Ref<Dangi>, &'static str),
    ) {
        test(Ref(dangi_calculating), "calculating");
        test(Ref(dangi_cached), "cached");
    }
    fn check_cyclic_and_rel_iso(year: i32) {
        let iso = Date::try_new_iso(year, 6, 6).unwrap();
        let chinese = iso.to_calendar(Chinese::new_always_calculating());
        let dangi = iso.to_calendar(Dangi::new_always_calculating());
        let chinese_year = chinese.cyclic_year();
        let korean_year = dangi.cyclic_year();
        assert_eq!(
            chinese_year, korean_year,
            "Cyclic year failed for year: {year}"
        );
        let chinese_rel_iso = chinese_year.related_iso;
        let korean_rel_iso = korean_year.related_iso;
        assert_eq!(
            chinese_rel_iso, korean_rel_iso,
            "Rel. ISO year equality failed for year: {year}"
        );
        assert_eq!(korean_rel_iso, year, "Dangi Rel. ISO failed!");
    }
    #[test]
    pub(crate) fn test_cyclic_same_as_chinese_near_present_day() {
        for year in 1923..=2123 {
            check_cyclic_and_rel_iso(year);
        }
    }
    #[test]
    pub(crate) fn test_cyclic_same_as_chinese_near_rd_zero() {
        for year in -100..=100 {
            check_cyclic_and_rel_iso(year);
        }
    }
    #[test]
    pub(crate) fn test_iso_to_dangi_roundtrip() {
        let mut rd = -1963020;
        let max_rd = 1963020;
        let mut iters = 0;
        let max_iters = 560;
        let dangi_calculating = Dangi::new_always_calculating();
        let dangi_cached = Dangi::new();
        while rd < max_rd && iters < max_iters {
            let rata_die = RataDie::new(rd);
            let iso = Date::from_rata_die(rata_die, Iso);
            do_twice(&dangi_calculating, &dangi_cached, |dangi, calendar_type| {
                let korean = iso.to_calendar(dangi);
                let result = korean.to_calendar(Iso);
                assert_eq!(
                    iso, result,
                    "[{calendar_type}] Failed roundtrip ISO -> Dangi -> ISO for RD: {rd}"
                );
            });

            rd += 7043;
            iters += 1;
        }
    }
    #[test]
    pub(crate) fn test_dangi_consistent_with_icu() {
        // Test cases for this test are derived from existing ICU Intl.DateTimeFormat. If there is a bug in ICU,
        // these test cases may be affected, and this calendar's output may not be entirely valid.

        // There are a number of test cases which do not match ICU for dates very far in the past or future,
        // see #3709.

        #[derive(Debug)]
        struct TestCase {
            iso_year: i32,
            iso_month: u8,
            iso_day: u8,
            expected_rel_iso: i32,
            expected_cyclic: u8,
            expected_month: u8,
            expected_day: u8,
        }

        let cases = [
            TestCase {
                // #3709: This test case fails to match ICU
                iso_year: 4321,
                iso_month: 1,
                iso_day: 23,
                expected_rel_iso: 4320,
                expected_cyclic: 57,
                expected_month: 13,
                expected_day: 12,
            },
            TestCase {
                iso_year: 3649,
                iso_month: 9,
                iso_day: 20,
                expected_rel_iso: 3649,
                expected_cyclic: 46,
                expected_month: 9,
                expected_day: 1,
            },
            TestCase {
                iso_year: 3333,
                iso_month: 3,
                iso_day: 3,
                expected_rel_iso: 3333,
                expected_cyclic: 30,
                expected_month: 1,
                expected_day: 25,
            },
            TestCase {
                iso_year: 3000,
                iso_month: 3,
                iso_day: 30,
                expected_rel_iso: 3000,
                expected_cyclic: 57,
                expected_month: 3,
                expected_day: 3,
            },
            TestCase {
                iso_year: 2772,
                iso_month: 7,
                iso_day: 27,
                expected_rel_iso: 2772,
                expected_cyclic: 9,
                expected_month: 7,
                expected_day: 5,
            },
            TestCase {
                iso_year: 2525,
                iso_month: 2,
                iso_day: 25,
                expected_rel_iso: 2525,
                expected_cyclic: 2,
                expected_month: 2,
                expected_day: 3,
            },
            TestCase {
                iso_year: 2345,
                iso_month: 3,
                iso_day: 21,
                expected_rel_iso: 2345,
                expected_cyclic: 2,
                expected_month: 2,
                expected_day: 17,
            },
            TestCase {
                iso_year: 2222,
                iso_month: 2,
                iso_day: 22,
                expected_rel_iso: 2222,
                expected_cyclic: 59,
                expected_month: 1,
                expected_day: 11,
            },
            TestCase {
                iso_year: 2167,
                iso_month: 6,
                iso_day: 22,
                expected_rel_iso: 2167,
                expected_cyclic: 4,
                expected_month: 5,
                expected_day: 6,
            },
            TestCase {
                iso_year: 2121,
                iso_month: 2,
                iso_day: 12,
                expected_rel_iso: 2120,
                expected_cyclic: 17,
                expected_month: 13,
                expected_day: 25,
            },
            TestCase {
                iso_year: 2080,
                iso_month: 12,
                iso_day: 31,
                expected_rel_iso: 2080,
                expected_cyclic: 37,
                expected_month: 12,
                expected_day: 21,
            },
            TestCase {
                iso_year: 2030,
                iso_month: 3,
                iso_day: 20,
                expected_rel_iso: 2030,
                expected_cyclic: 47,
                expected_month: 2,
                expected_day: 17,
            },
            TestCase {
                iso_year: 2027,
                iso_month: 2,
                iso_day: 7,
                expected_rel_iso: 2027,
                expected_cyclic: 44,
                expected_month: 1,
                expected_day: 1,
            },
            TestCase {
                iso_year: 2023,
                iso_month: 7,
                iso_day: 1,
                expected_rel_iso: 2023,
                expected_cyclic: 40,
                expected_month: 6,
                expected_day: 14,
            },
            TestCase {
                iso_year: 2022,
                iso_month: 3,
                iso_day: 1,
                expected_rel_iso: 2022,
                expected_cyclic: 39,
                expected_month: 1,
                expected_day: 29,
            },
            TestCase {
                iso_year: 2021,
                iso_month: 2,
                iso_day: 1,
                expected_rel_iso: 2020,
                expected_cyclic: 37,
                expected_month: 13,
                expected_day: 20,
            },
            TestCase {
                iso_year: 2016,
                iso_month: 3,
                iso_day: 30,
                expected_rel_iso: 2016,
                expected_cyclic: 33,
                expected_month: 2,
                expected_day: 22,
            },
            TestCase {
                iso_year: 2016,
                iso_month: 7,
                iso_day: 30,
                expected_rel_iso: 2016,
                expected_cyclic: 33,
                expected_month: 6,
                expected_day: 27,
            },
            TestCase {
                iso_year: 2015,
                iso_month: 9,
                iso_day: 22,
                expected_rel_iso: 2015,
                expected_cyclic: 32,
                expected_month: 8,
                expected_day: 10,
            },
            TestCase {
                iso_year: 2013,
                iso_month: 10,
                iso_day: 1,
                expected_rel_iso: 2013,
                expected_cyclic: 30,
                expected_month: 8,
                expected_day: 27,
            },
            TestCase {
                iso_year: 2010,
                iso_month: 2,
                iso_day: 1,
                expected_rel_iso: 2009,
                expected_cyclic: 26,
                expected_month: 13,
                expected_day: 18,
            },
            TestCase {
                iso_year: 2000,
                iso_month: 8,
                iso_day: 30,
                expected_rel_iso: 2000,
                expected_cyclic: 17,
                expected_month: 8,
                expected_day: 2,
            },
            TestCase {
                iso_year: 1990,
                iso_month: 11,
                iso_day: 11,
                expected_rel_iso: 1990,
                expected_cyclic: 7,
                expected_month: 10,
                expected_day: 24,
            },
            TestCase {
                iso_year: 1970,
                iso_month: 6,
                iso_day: 10,
                expected_rel_iso: 1970,
                expected_cyclic: 47,
                expected_month: 5,
                expected_day: 7,
            },
            TestCase {
                iso_year: 1970,
                iso_month: 1,
                iso_day: 1,
                expected_rel_iso: 1969,
                expected_cyclic: 46,
                expected_month: 11,
                expected_day: 24,
            },
            TestCase {
                iso_year: 1941,
                iso_month: 12,
                iso_day: 7,
                expected_rel_iso: 1941,
                expected_cyclic: 18,
                expected_month: 11,
                expected_day: 19,
            },
            TestCase {
                iso_year: 1812,
                iso_month: 5,
                iso_day: 4,
                expected_rel_iso: 1812,
                expected_cyclic: 9,
                expected_month: 3,
                expected_day: 24,
            },
            TestCase {
                iso_year: 1655,
                iso_month: 6,
                iso_day: 15,
                expected_rel_iso: 1655,
                expected_cyclic: 32,
                expected_month: 5,
                expected_day: 12,
            },
            TestCase {
                iso_year: 1333,
                iso_month: 3,
                iso_day: 10,
                expected_rel_iso: 1333,
                expected_cyclic: 10,
                expected_month: 2,
                expected_day: 16,
            },
            TestCase {
                iso_year: 1000,
                iso_month: 10,
                iso_day: 10,
                expected_rel_iso: 1000,
                expected_cyclic: 37,
                expected_month: 9,
                expected_day: 5,
            },
            TestCase {
                iso_year: 842,
                iso_month: 2,
                iso_day: 15,
                expected_rel_iso: 841,
                expected_cyclic: 58,
                expected_month: 13,
                expected_day: 28,
            },
            TestCase {
                iso_year: 101,
                iso_month: 1,
                iso_day: 10,
                expected_rel_iso: 100,
                expected_cyclic: 37,
                expected_month: 12,
                expected_day: 24,
            },
            TestCase {
                iso_year: -1,
                iso_month: 3,
                iso_day: 28,
                expected_rel_iso: -1,
                expected_cyclic: 56,
                expected_month: 2,
                expected_day: 25,
            },
            TestCase {
                iso_year: -3,
                iso_month: 2,
                iso_day: 28,
                expected_rel_iso: -3,
                expected_cyclic: 54,
                expected_month: 2,
                expected_day: 5,
            },
            TestCase {
                iso_year: -365,
                iso_month: 7,
                iso_day: 24,
                expected_rel_iso: -365,
                expected_cyclic: 52,
                expected_month: 6,
                expected_day: 24,
            },
            TestCase {
                iso_year: -999,
                iso_month: 9,
                iso_day: 9,
                expected_rel_iso: -999,
                expected_cyclic: 18,
                expected_month: 7,
                expected_day: 27,
            },
            TestCase {
                iso_year: -1500,
                iso_month: 1,
                iso_day: 5,
                expected_rel_iso: -1501,
                expected_cyclic: 56,
                expected_month: 12,
                expected_day: 2,
            },
            TestCase {
                iso_year: -2332,
                iso_month: 3,
                iso_day: 1,
                expected_rel_iso: -2332,
                expected_cyclic: 5,
                expected_month: 1,
                expected_day: 16,
            },
            TestCase {
                iso_year: -2332,
                iso_month: 2,
                iso_day: 15,
                expected_rel_iso: -2332,
                expected_cyclic: 5,
                expected_month: 1,
                expected_day: 1,
            },
            TestCase {
                // #3709: This test case fails to match ICU
                iso_year: -2332,
                iso_month: 2,
                iso_day: 14,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 13,
                expected_day: 30,
            },
            TestCase {
                // #3709: This test case fails to match ICU
                iso_year: -2332,
                iso_month: 1,
                iso_day: 17,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 13,
                expected_day: 2,
            },
            TestCase {
                // #3709: This test case fails to match ICU
                iso_year: -2332,
                iso_month: 1,
                iso_day: 16,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 13,
                expected_day: 1,
            },
            TestCase {
                iso_year: -2332,
                iso_month: 1,
                iso_day: 15,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 12,
                expected_day: 29,
            },
            TestCase {
                iso_year: -2332,
                iso_month: 1,
                iso_day: 1,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 12,
                expected_day: 15,
            },
            TestCase {
                iso_year: -2333,
                iso_month: 1,
                iso_day: 16,
                expected_rel_iso: -2334,
                expected_cyclic: 3,
                expected_month: 12,
                expected_day: 19,
            },
            TestCase {
                iso_year: -2333,
                iso_month: 1,
                iso_day: 27,
                expected_rel_iso: -2333,
                expected_cyclic: 4,
                expected_month: 1,
                expected_day: 1,
            },
            TestCase {
                iso_year: -2333,
                iso_month: 1,
                iso_day: 26,
                expected_rel_iso: -2334,
                expected_cyclic: 3,
                expected_month: 12,
                expected_day: 29,
            },
            TestCase {
                iso_year: -2600,
                iso_month: 9,
                iso_day: 16,
                expected_rel_iso: -2600,
                expected_cyclic: 37,
                expected_month: 8,
                expected_day: 16,
            },
            TestCase {
                iso_year: -2855,
                iso_month: 2,
                iso_day: 3,
                expected_rel_iso: -2856,
                expected_cyclic: 21,
                expected_month: 12,
                expected_day: 30,
            },
            TestCase {
                // #3709: This test case fails to match ICU
                iso_year: -3000,
                iso_month: 5,
                iso_day: 15,
                expected_rel_iso: -3000,
                expected_cyclic: 57,
                expected_month: 4,
                expected_day: 1,
            },
            TestCase {
                // #3709: This test case fails to match ICU
                iso_year: -3649,
                iso_month: 9,
                iso_day: 20,
                expected_rel_iso: -3649,
                expected_cyclic: 8,
                expected_month: 8,
                expected_day: 10,
            },
            TestCase {
                // #3709: This test case fails to match ICU
                iso_year: -3649,
                iso_month: 3,
                iso_day: 30,
                expected_rel_iso: -3649,
                expected_cyclic: 8,
                expected_month: 2,
                expected_day: 14,
            },
            TestCase {
                // #3709: This test case fails to match ICU
                iso_year: -3650,
                iso_month: 3,
                iso_day: 30,
                expected_rel_iso: -3650,
                expected_cyclic: 7,
                expected_month: 3,
                expected_day: 3,
            },
        ];

        let dangi_calculating = Dangi::new_always_calculating();
        let dangi_cached = Dangi::new();

        for case in cases {
            let iso = Date::try_new_iso(case.iso_year, case.iso_month, case.iso_day).unwrap();
            do_twice(&dangi_calculating, &dangi_cached, |dangi, calendar_type| {
                let dangi = iso.to_calendar(dangi);
                let dangi_cyclic = dangi.cyclic_year();
                let dangi_month = dangi.month().ordinal;
                let dangi_day = dangi.day_of_month().0;

                assert_eq!(
                    dangi_cyclic.related_iso, case.expected_rel_iso,
                    "[{calendar_type}] Related ISO failed for test case: {case:?}"
                );
                assert_eq!(
                    dangi_cyclic.year, case.expected_cyclic,
                    "[{calendar_type}] Cyclic year failed for test case: {case:?}"
                );
                assert_eq!(
                    dangi_month, case.expected_month,
                    "[{calendar_type}] Month failed for test case: {case:?}"
                );
                assert_eq!(
                    dangi_day, case.expected_day,
                    "[{calendar_type}] Day failed for test case: {case:?}"
                );
            });
        }
    }
}

pub(super) mod continuity {
    use icu_calendar::*;
    fn check_continuity<A: AsCalendar>(mut date: Date<A>) {
        let one_day_duration = DateDuration::<A::Calendar>::new(0, 0, 0, 1);

        let mut rata_die = date.to_rata_die();
        let mut weekday = date.day_of_week();
        let mut year = date.year();
        let mut is_in_leap_year = date.is_in_leap_year();

        for _ in 0..(366 * 20) {
            let next_date = date.added(one_day_duration);
            let next_rata_die = next_date.to_iso().to_rata_die();
            assert_eq!(next_rata_die, rata_die + 1, "{next_date:?}");
            let next_weekday = next_date.day_of_week();
            let next_year = next_date.year();
            let next_is_in_leap_year = next_date.is_in_leap_year();
            assert_eq!(
                (next_weekday as usize) % 7,
                (weekday as usize + 1) % 7,
                "{next_date:?}"
            );
            if year == next_year {
                assert_eq!(is_in_leap_year, next_is_in_leap_year, "{next_date:?}");
            }
            date = next_date;
            rata_die = next_rata_die;
            weekday = next_weekday;
            year = next_year;
            is_in_leap_year = next_is_in_leap_year;
        }
    }
    fn check_every_250_days<A: AsCalendar>(mut date: Date<A>) {
        let one_thousand_days_duration = DateDuration::<A::Calendar>::new(0, 0, 0, 250);

        let mut rata_die = date.to_rata_die();

        for _ in 0..2000 {
            let next_date = date.added(one_thousand_days_duration);
            let next_iso = next_date.to_iso();
            let next_rata_die = next_iso.to_rata_die();
            assert_eq!(next_rata_die, rata_die + 250, "{next_date:?}");
            let next_date_roundtrip = next_iso.to_calendar(Ref(next_date.calendar()));
            assert_eq!(next_date, next_date_roundtrip, "{next_date:?}");
            date = next_date;
            rata_die = next_rata_die;
        }
    }
    #[test]
    pub(crate) fn test_chinese_continuity() {
        let cal = icu_calendar::cal::Chinese::new();
        let cal = Ref(&cal);
        let date = Date::try_new_chinese_with_calendar(-10, 1, 1, cal);
        check_continuity(date.unwrap());
        let date = Date::try_new_chinese_with_calendar(-300, 1, 1, cal);
        check_every_250_days(date.unwrap());
        let date = Date::try_new_chinese_with_calendar(-10000, 1, 1, cal);
        check_every_250_days(date.unwrap());
    }
    #[test]
    pub(crate) fn test_dangi_continuity() {
        let cal = icu_calendar::cal::Dangi::new();
        let cal = Ref(&cal);
        let date = Date::try_new_dangi_with_calendar(-10, 1, 1, cal);
        check_continuity(date.unwrap());
        let date = Date::try_new_dangi_with_calendar(-300, 1, 1, cal);
        check_every_250_days(date.unwrap());
    }
}

pub(super) mod any_calendar {
    use icu_calendar::types::{self, MonthCode};
    use icu_calendar::*;
    use tinystr::tinystr;
    fn single_test_roundtrip(
        calendar: Ref<AnyCalendar>,
        era: Option<(&str, Option<u8>)>,
        year: i32,
        month_code: &str,
        day: u8,
    ) {
        let month = types::MonthCode(month_code.parse().expect("month code must parse"));

        let date = Date::try_new_from_codes(era.map(|x| x.0), year, month, day, calendar)
            .unwrap_or_else(|e| {
                panic!(
                    "Failed to construct date for {} with {era:?}, {year}, {month}, {day}: {e:?}",
                    calendar.debug_name(),
                )
            });

        let roundtrip_year = date.year();
        // FIXME: these APIs should be improved
        let roundtrip_year = roundtrip_year.era_year_or_related_iso();
        let roundtrip_month = date.month().standard_code;
        let roundtrip_day = date.day_of_month().0;

        assert_eq!(
            (year, month, day),
            (roundtrip_year, roundtrip_month, roundtrip_day),
            "Failed to roundtrip for calendar {}",
            calendar.debug_name()
        );

        if let Some((era_code, era_index)) = era {
            let roundtrip_era_year = date.year().era().expect("year type should be era");
            assert_eq!(
                (era_code, era_index),
                (
                    roundtrip_era_year.era.as_str(),
                    roundtrip_era_year.era_index
                ),
                "Failed to roundtrip era for calendar {}",
                calendar.debug_name()
            )
        }

        let iso = date.to_iso();
        let reconstructed = Date::new_from_iso(iso, calendar);
        assert_eq!(
            date, reconstructed,
            "Failed to roundtrip via iso with {era:?}, {year}, {month}, {day}"
        )
    }
    fn single_test_error(
        calendar: Ref<AnyCalendar>,
        era: Option<(&str, Option<u8>)>,
        year: i32,
        month_code: &str,
        day: u8,
        error: DateError,
    ) {
        let month = types::MonthCode(month_code.parse().expect("month code must parse"));

        let date = Date::try_new_from_codes(era.map(|x| x.0), year, month, day, calendar);
        assert_eq!(
            date,
            Err(error),
            "Construction with {era:?}, {year}, {month}, {day} did not return {error:?}"
        )
    }
    #[test]
    pub(crate) fn chinese_and_dangi_construction() {
        let chinese = AnyCalendar::new(AnyCalendarKind::Chinese);
        let chinese = Ref(&chinese);
        let dangi = AnyCalendar::new(AnyCalendarKind::Dangi);
        let dangi = Ref(&dangi);
        single_test_roundtrip(chinese, None, 400, "M02", 5);
        single_test_roundtrip(chinese, None, 4660, "M07", 29);
        single_test_roundtrip(chinese, None, -100, "M11", 12);
        single_test_error(
            chinese,
            None,
            4658,
            "M13",
            1,
            DateError::UnknownMonthCode(MonthCode(tinystr!(4, "M13"))),
        );

        single_test_roundtrip(dangi, None, 400, "M02", 5);
        single_test_roundtrip(dangi, None, 4660, "M08", 29);
        single_test_roundtrip(dangi, None, -1300, "M11", 12);
        single_test_error(
            dangi,
            None,
            10393,
            "M00L",
            1,
            DateError::UnknownMonthCode(MonthCode(tinystr!(4, "M00L"))),
        );
    }
}

pub(super) mod documentation {
    #[test]
    pub(crate) fn chinese_doc_1() {
        use icu_calendar::{cal::Chinese, Date};

        let chinese = Chinese::new();
        let chinese_date = Date::try_new_chinese_with_calendar(2023, 6, 6, chinese)
            .expect("Failed to initialize Chinese Date instance.");

        assert_eq!(chinese_date.cyclic_year().related_iso, 2023);
        assert_eq!(chinese_date.cyclic_year().year, 40);
        assert_eq!(chinese_date.month().ordinal, 6);
        assert_eq!(chinese_date.day_of_month().0, 6);
    }
    #[test]
    pub(crate) fn chinese_doc_2() {
        use icu_calendar::{cal::Chinese, Date};

        let chinese = Chinese::new_always_calculating();

        let date_chinese = Date::try_new_chinese_with_calendar(2023, 6, 11, chinese)
            .expect("Failed to initialize Chinese Date instance.");

        assert_eq!(date_chinese.cyclic_year().related_iso, 2023);
        assert_eq!(date_chinese.cyclic_year().year, 40);
        assert_eq!(date_chinese.month().ordinal, 6);
        assert_eq!(date_chinese.day_of_month().0, 11);
    }
    #[test]
    pub(crate) fn dangi_doc_1() {
        use icu_calendar::cal::Dangi;
        use icu_calendar::Date;

        let dangi = Dangi::new();
        let dangi_date = Date::try_new_dangi_with_calendar(2023, 6, 6, dangi)
            .expect("Failed to initialize Dangi Date instance.");

        assert_eq!(dangi_date.cyclic_year().related_iso, 2023);
        assert_eq!(dangi_date.cyclic_year().year, 40);
        assert_eq!(dangi_date.month().ordinal, 6);
        assert_eq!(dangi_date.day_of_month().0, 6);
    }
    #[test]
    pub(crate) fn dangi_doc_2() {
        use icu_calendar::cal::{Chinese, Dangi};
        use icu_calendar::Date;
        use tinystr::tinystr;

        let iso_a = Date::try_new_iso(2012, 4, 23).unwrap();
        let dangi_a = iso_a.to_calendar(Dangi::new());
        let chinese_a = iso_a.to_calendar(Chinese::new());

        assert_eq!(dangi_a.month().standard_code.0, tinystr!(4, "M03L"));
        assert_eq!(chinese_a.month().standard_code.0, tinystr!(4, "M04"));

        let iso_b = Date::try_new_iso(2012, 5, 23).unwrap();
        let dangi_b = iso_b.to_calendar(Dangi::new());
        let chinese_b = iso_b.to_calendar(Chinese::new());

        assert_eq!(dangi_b.month().standard_code.0, tinystr!(4, "M04"));
        assert_eq!(chinese_b.month().standard_code.0, tinystr!(4, "M04L"));
    }
    #[test]
    pub(crate) fn dangi_doc_3() {
        use icu_calendar::cal::Dangi;
        use icu_calendar::Date;

        let dangi = Dangi::new();

        let date_dangi = Date::try_new_dangi_with_calendar(2023, 6, 18, dangi)
            .expect("Failed to initialize Dangi Date instance.");

        assert_eq!(date_dangi.cyclic_year().related_iso, 2023);
        assert_eq!(date_dangi.cyclic_year().year, 40);
        assert_eq!(date_dangi.month().ordinal, 6);
        assert_eq!(date_dangi.day_of_month().0, 18);
    }
}
