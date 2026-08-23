mod array;
mod array_from_async;
mod async_disposable_stack;
mod async_iterator;
mod atomics;
mod bigint;
mod binary_data;
mod boolean;
mod bootstrap;
mod collections;
mod date;
mod decimal;
mod disposable_stack;
mod errors;
mod finalization_registry;
mod function;
mod global_numeric;
mod host;
mod intl;
mod intl_datetimeformat;
pub(crate) use intl_datetimeformat::intl_date_time_format_pool_strings;
mod iterators;
mod json;
mod math;
mod number;
mod object;
mod promise;
mod proxy;
mod reflect;
mod regexp;
mod standard;
mod string;
mod symbol;
mod temporal;
mod temporal_duration;
mod temporal_duration_methods;
mod temporal_instant;
/// The unvalidated epoch-nanosecond local pair, re-exported for `date.rs`:
/// `Date.prototype.toTemporalInstant` shares the millisecond widening.
pub(crate) use temporal_instant::UnvalidatedEpochNanoseconds;
mod temporal_options;
mod temporal_plain_date;
/// The calendar table, re-exported for `data.rs`: the string pool derives the
/// interned calendar spellings *and* every era spelling by walking
/// `TemporalCalendarId::ALL -> eras() -> spellings()`, which is exactly the
/// table `emit_temporal_resolve_era_to_year` matches an incoming `era`
/// against. `Era::code()` is `Era::spellings()[0]`, so an alias such as `ad`
/// or `bc` added to that table is interned, accepted by
/// `CalendarResolveFields` and excluded from the `era` accessor's answer
/// without a second edit anywhere — and a *calendar* added with a complete
/// `eras()` is interned without an edit here at all.
pub(crate) use temporal_plain_date::TemporalCalendarId;
/// The `DifferenceTemporal*` guard messages, re-exported for `data.rs` for the
/// same reason as the calendar table: the string pool derives them by walking
/// `TemporalDifferenceGuard::ALL -> message()`, gated on
/// `emitting_builtins()`, instead of repeating the five literals. A message
/// spelled at an emitter and not interned is a *compile-time panic* in every
/// full bootstrap (`string ... must exist in pool`), which is how batch 6 took
/// 24 `lila-aot-wasm --lib` tests down with two new `&str` literals.
pub(crate) use temporal_plain_date::TemporalDifferenceGuard;
mod temporal_plain_date_methods;
mod temporal_plain_date_time;
mod temporal_plain_date_time_methods;
mod temporal_plain_month_day;
mod temporal_plain_time;
mod temporal_plain_time_methods;
mod temporal_plain_year_month;
mod temporal_plain_year_month_methods;
/// `Temporal.ZonedDateTime.prototype.{add,subtract,until,since,withCalendar}`.
///
/// Split from `temporal.rs` on the same boundary
/// `temporal_plain_date_time_methods` is split from
/// `temporal_plain_date_time`: record/constructor/accessors on one side,
/// prototype method bodies on the other. `check-module-boundaries.sh` requires
/// both, so the split cannot silently collapse back.
mod temporal_zoned_date_time_methods;
/// The two closed direction domains of that surface. `add`/`subtract` and
/// `until`/`since` are adjacent arms of one `match` in `standard.rs`; as
/// `bool`s a transposition compiled and silently inverted the operation.
pub(crate) use temporal_zoned_date_time_methods::{
    ZonedDateTimeArithmetic, ZonedDateTimeDifference,
};
mod uri;
mod weak_ref;
pub(crate) const ECMASCRIPT_NON_ASCII_WHITESPACE_UTF8: [&[u8]; 19] = [
    &[0xC2, 0xA0],       // U+00A0
    &[0xE1, 0x9A, 0x80], // U+1680
    &[0xE2, 0x80, 0x80], // U+2000
    &[0xE2, 0x80, 0x81], // U+2001
    &[0xE2, 0x80, 0x82], // U+2002
    &[0xE2, 0x80, 0x83], // U+2003
    &[0xE2, 0x80, 0x84], // U+2004
    &[0xE2, 0x80, 0x85], // U+2005
    &[0xE2, 0x80, 0x86], // U+2006
    &[0xE2, 0x80, 0x87], // U+2007
    &[0xE2, 0x80, 0x88], // U+2008
    &[0xE2, 0x80, 0x89], // U+2009
    &[0xE2, 0x80, 0x8A], // U+200A
    &[0xE2, 0x80, 0xA8], // U+2028
    &[0xE2, 0x80, 0xA9], // U+2029
    &[0xE2, 0x80, 0xAF], // U+202F
    &[0xE2, 0x81, 0x9F], // U+205F
    &[0xE3, 0x80, 0x80], // U+3000
    &[0xEF, 0xBB, 0xBF], // U+FEFF
];
pub(crate) const ARRAY_ITERATOR_KIND_VALUES: u64 = 0;
pub(crate) const ARRAY_ITERATOR_KIND_KEYS: u64 = 1;
pub(crate) const ARRAY_ITERATOR_KIND_ENTRIES: u64 = 2;
pub(crate) const NUMBER_TO_PRECISION_CASES: &[(f64, i64, &str)] = &[
    (f64::INFINITY, 1000, "Infinity"),
    (f64::NEG_INFINITY, 1000, "-Infinity"),
    (
        3.0,
        100,
        "3.000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000000",
    ),
    (3.0, 1, "3"),
    (10.0, 1, "1e+1"),
    (11.0, 1, "1e+1"),
    (17.0, 1, "2e+1"),
    (19.0, 1, "2e+1"),
    (20.0, 1, "2e+1"),
    (100.0, 1, "1e+2"),
    (1000.0, 1, "1e+3"),
    (10000.0, 1, "1e+4"),
    (100000.0, 1, "1e+5"),
    (100.0, 2, "1.0e+2"),
    (1000.0, 2, "1.0e+3"),
    (10000.0, 2, "1.0e+4"),
    (100000.0, 2, "1.0e+5"),
    (1000.0, 3, "1.00e+3"),
    (10000.0, 3, "1.00e+4"),
    (100000.0, 3, "1.00e+5"),
    (42.0, 1, "4e+1"),
    (-42.0, 1, "-4e+1"),
    (1.2345e27, 1, "1e+27"),
    (1.2345e27, 2, "1.2e+27"),
    (1.2345e27, 3, "1.23e+27"),
    (1.2345e27, 4, "1.234e+27"),
    (1.2345e27, 5, "1.2345e+27"),
    (1.2345e27, 6, "1.23450e+27"),
    (1.2345e27, 7, "1.234500e+27"),
    (1.2345e27, 16, "1.234500000000000e+27"),
    (1.2345e27, 17, "1.2345000000000000e+27"),
    (1.2345e27, 18, "1.23449999999999996e+27"),
    (1.2345e27, 19, "1.234499999999999962e+27"),
    (1.2345e27, 20, "1.2344999999999999618e+27"),
    (1.2345e27, 21, "1.23449999999999996184e+27"),
    (-1.2345e27, 1, "-1e+27"),
    (-1.2345e27, 2, "-1.2e+27"),
    (-1.2345e27, 3, "-1.23e+27"),
    (-1.2345e27, 4, "-1.234e+27"),
    (-1.2345e27, 5, "-1.2345e+27"),
    (-1.2345e27, 6, "-1.23450e+27"),
    (-1.2345e27, 7, "-1.234500e+27"),
    (-1.2345e27, 16, "-1.234500000000000e+27"),
    (-1.2345e27, 17, "-1.2345000000000000e+27"),
    (-1.2345e27, 18, "-1.23449999999999996e+27"),
    (-1.2345e27, 19, "-1.234499999999999962e+27"),
    (-1.2345e27, 20, "-1.2344999999999999618e+27"),
    (-1.2345e27, 21, "-1.23449999999999996184e+27"),
    (1e21, 1, "1e+21"),
    (1e21, 2, "1.0e+21"),
    (1e21, 3, "1.00e+21"),
    (1e21, 4, "1.000e+21"),
    (1e21, 5, "1.0000e+21"),
    (1e21, 6, "1.00000e+21"),
    (1e21, 7, "1.000000e+21"),
    (1e21, 16, "1.000000000000000e+21"),
    (1e21, 17, "1.0000000000000000e+21"),
    (1e21, 18, "1.00000000000000000e+21"),
    (1e21, 19, "1.000000000000000000e+21"),
    (1e21, 20, "1.0000000000000000000e+21"),
    (1e21, 21, "1.00000000000000000000e+21"),
    (1e-21, 1, "1e-21"),
    (1e-21, 2, "1.0e-21"),
    (1e-21, 3, "1.00e-21"),
    (1e-21, 4, "1.000e-21"),
    (1e-21, 5, "1.0000e-21"),
    (1e-21, 6, "1.00000e-21"),
    (1e-21, 7, "1.000000e-21"),
    (1e-21, 16, "9.999999999999999e-22"),
    (1e-21, 17, "9.9999999999999991e-22"),
    (1e-21, 18, "9.99999999999999908e-22"),
    (1e-21, 19, "9.999999999999999075e-22"),
    (1e-21, 20, "9.9999999999999990754e-22"),
    (1e-21, 21, "9.99999999999999907537e-22"),
    (1e-8, 1, "1e-8"),
    (-1e-8, 1, "-1e-8"),
];
