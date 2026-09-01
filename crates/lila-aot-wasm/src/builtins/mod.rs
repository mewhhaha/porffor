mod array;
mod array_from_async;
mod async_disposable_stack;
mod async_iterator;
mod atomics;
mod bigint;
mod binary_data;
pub(crate) use binary_data::{TypedArrayAccessorKind, TypedArrayViewLocals, TypedArrayWitnessUse};
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
pub(crate) use iterators::ArrayIteratorKind;
mod json;
mod math;
mod number;
mod object;
mod promise;
pub(crate) use promise::{AsyncExecutionRealmContext, AsyncGeneratorCompleteStepKind};
mod proxy;
mod reflect;
mod regexp;
mod standard;
mod string;
pub(crate) use string::StringNormalizationForm;
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
mod uri;
mod weak_ref;
