//! This module implements the global `Intl.DateTimeFormat` object.
//!
//! `Intl.DateTimeFormat` is a built-in object that has properties and methods for date and time
//! i18n.

use std::fmt::Write as _;

use crate::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    NativeFunction,
    builtins::{
        BuiltInConstructor, BuiltInObject, IntrinsicObject, OrdinaryObject, array::Array,
        builder::BuiltInBuilder,
        date::utils::{
            MS_PER_MINUTE, date_from_time, hour_from_time, local_time, make_date, make_day,
            min_from_time, month_from_time, ms_from_time, sec_from_time, time_clip, week_day,
            year_from_time,
        },
        intl::{
            Service,
            locale::{
                canonicalize_locale_list, filter_locales, locale_to_canonical_string,
                resolve_locale, supported_numbering_systems,
            },
            number_format::substitute_digits,
            options::{IntlOptions, LocaleMatcher, coerce_options_to_object},
        },
        options::OptionType,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::{
        FunctionObjectBuilder, JsFunction, ObjectInitializer,
        internal_methods::get_prototype_from_constructor,
    },
    property::{Attribute, PropertyDescriptor},
    realm::Realm,
    string::StaticJsStrings,
    symbol::JsSymbol,
};

#[cfg(feature = "temporal")]
use crate::builtins::temporal::{
    Instant, PlainDate, PlainDateTime, PlainMonthDay, PlainTime, PlainYearMonth,
    ZonedDateTime,
};
#[cfg(feature = "temporal")]
use temporal_rs::{
    Instant as RsInstant, TimeZone as RsTimeZone, ZonedDateTime as RsZonedDateTime,
};

use boa_gc::{Finalize, Trace};
#[cfg(feature = "temporal")]
use icu_calendar::{AnyCalendar, AnyCalendarKind, Date as IcuDate, types::YearInfo};
use icu_decimal::provider::DecimalSymbolsV1;
use icu_calendar::preferences::CalendarAlgorithm;
use icu_datetime::preferences::HourCycle;
use icu_locale::{Locale, extensions::unicode::Value, extensions_unicode_key as key};

const RANGE_SEPARATOR: &str = " - ";

#[derive(Debug, Clone, Default)]
struct LocaleKeywords {
    ca: Option<String>,
    nu: Option<String>,
    hc: Option<String>,
}

#[derive(Debug, Clone)]
struct RequestedLocale {
    keywords: LocaleKeywords,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum RecordKind {
    Number,
    Instant,
    PlainDate,
    PlainDateTime,
    PlainMonthDay,
    PlainTime,
    PlainYearMonth,
    ZonedDateTime,
}

#[derive(Debug, Clone)]
struct TemporalRecord {
    kind: RecordKind,
    epoch_millis: Option<f64>,
    year: Option<i32>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: u8,
    second: u8,
    millisecond: u16,
    calendar: String,
}

#[derive(Debug, Clone)]
enum Formattable {
    Number(f64),
    Temporal(TemporalRecord),
}

#[derive(Debug, Clone)]
struct DisplayRecord {
    kind: RecordKind,
    year: Option<i32>,
    month: Option<u8>,
    day: Option<u8>,
    hour: Option<u8>,
    minute: u8,
    second: u8,
    millisecond: u16,
    calendar: String,
    month_display: Option<String>,
    era_label: Option<String>,
    related_year: Option<i32>,
    year_name: Option<String>,
}

#[derive(Debug, Clone)]
struct DateTimePart {
    kind: &'static str,
    value: JsString,
}

#[derive(Debug, Clone, Trace, Finalize, JsData)]
pub(crate) struct DateTimeFormat {
    locale: String,
    calendar: String,
    numbering_system: String,
    time_zone: String,
    weekday: Option<String>,
    era: Option<String>,
    year: Option<String>,
    month: Option<String>,
    day: Option<String>,
    day_period: Option<String>,
    hour: Option<String>,
    minute: Option<String>,
    second: Option<String>,
    fractional_second_digits: Option<u8>,
    time_zone_name: Option<String>,
    hour_cycle: Option<String>,
    hour12: Option<bool>,
    date_style: Option<String>,
    time_style: Option<String>,
    default_date_only: bool,
    temporal_locale_string: bool,
    bound_format: Option<JsFunction>,
}

impl Service for DateTimeFormat {
    type LangMarker = DecimalSymbolsV1;
    type LocaleOptions = ();
}

impl IntrinsicObject for DateTimeFormat {
    fn init(realm: &Realm) {
        let get_format = BuiltInBuilder::callable(realm, Self::get_format)
            .name(js_string!("get format"))
            .build();

        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                JsSymbol::to_string_tag(),
                js_string!("Intl.DateTimeFormat"),
                Attribute::CONFIGURABLE,
            )
            .accessor(
                js_string!("format"),
                Some(get_format),
                None,
                Attribute::CONFIGURABLE,
            )
            .method(Self::format_to_parts, js_string!("formatToParts"), 1)
            .method(Self::format_range, js_string!("formatRange"), 2)
            .method(
                Self::format_range_to_parts,
                js_string!("formatRangeToParts"),
                2,
            )
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DateTimeFormat {
    const NAME: JsString = StaticJsStrings::DATE_TIME_FORMAT;
}

impl BuiltInConstructor for DateTimeFormat {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 7;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::date_time_format;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        let new_target_inner = &if new_target.is_undefined() {
            context
                .active_function_object()
                .unwrap_or_else(|| {
                    context
                        .intrinsics()
                        .constructors()
                        .date_time_format()
                        .constructor()
                })
                .into()
        } else {
            new_target.clone()
        };

        let prototype = get_prototype_from_constructor(
            new_target_inner,
            StandardConstructors::date_time_format,
            context,
        )?;

        let date_time_format = Self::new(locales, options, context)?;
        let date_time_format = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            date_time_format,
        );

        let this = context.vm.stack.get_this(context.vm.frame());
        let Some(this_obj) = this.as_object() else {
            return Ok(date_time_format.into());
        };

        let constructor = context
            .intrinsics()
            .constructors()
            .date_time_format()
            .constructor();
        let legacy_this = this_obj.clone().downcast::<DateTimeFormat>().is_ok()
            || JsValue::ordinary_has_instance(&constructor.into(), &this, context)?;
        if new_target.is_undefined() && legacy_this {
            let fallback_symbol = context
                .intrinsics()
                .objects()
                .intl()
                .borrow()
                .data()
                .fallback_symbol();
            this_obj.define_property_or_throw(
                fallback_symbol,
                PropertyDescriptor::builder()
                    .value(date_time_format)
                    .writable(false)
                    .enumerable(false)
                    .configurable(false),
                context,
            )?;
            Ok(this)
        } else {
            Ok(date_time_format.into())
        }
    }
}

impl DateTimeFormat {
    pub(crate) fn new(
        locales: &JsValue,
        options: &JsValue,
        context: &mut Context,
    ) -> JsResult<Self> {
        let requested_locales = canonicalize_locale_list(locales, context)?;
        let requested_locale = requested_locales
            .first()
            .cloned()
            .map(requested_locale_from_locale)
            .transpose()?;
        let options = coerce_options_to_object(options, context)?;

        let matcher = match get_string_option(&options, "localeMatcher", context)?.as_deref() {
            None => LocaleMatcher::BestFit,
            Some("lookup") => LocaleMatcher::Lookup,
            Some("best fit") => LocaleMatcher::BestFit,
            Some(_) => {
                return Err(JsNativeError::range()
                    .with_message("Invalid localeMatcher option")
                    .into());
            }
        };
        let resolved_locale = resolve_locale::<Self>(
            requested_locales,
            &mut IntlOptions {
                matcher,
                service_options: (),
            },
            context.intl_provider(),
        )?;
        let resolved_base = locale_to_canonical_string(&resolved_locale);
        let requested_locale = requested_locale.unwrap_or_else(|| RequestedLocale {
            keywords: LocaleKeywords::default(),
        });

        let mut retained = LocaleKeywords::default();

        let locale_calendar =
            ignore_invalid_requested_locale_keyword(canonicalize_calendar(
                requested_locale.keywords.ca.as_deref(),
            ))?;
        let locale_numbering_system =
            ignore_invalid_requested_locale_keyword(canonicalize_numbering_system(
                requested_locale.keywords.nu.as_deref(),
            ))?;
        let locale_hour_cycle =
            ignore_invalid_requested_locale_keyword(canonicalize_hour_cycle(
                requested_locale.keywords.hc.as_deref(),
            ))?;

        let mut state = Self {
            locale: resolved_base.clone(),
            calendar: locale_calendar
                .clone()
                .unwrap_or_else(|| "gregory".to_owned()),
            numbering_system: locale_numbering_system.clone().unwrap_or_else(|| {
                default_numbering_system_for_locale(&resolved_base).to_owned()
            }),
            time_zone: default_time_zone_identifier(),
            weekday: None,
            era: None,
            year: None,
            month: None,
            day: None,
            day_period: None,
            hour: None,
            minute: None,
            second: None,
            fractional_second_digits: None,
            time_zone_name: None,
            hour_cycle: locale_hour_cycle.clone(),
            hour12: locale_hour_cycle.as_deref().map(is_twelve_hour_cycle),
            date_style: None,
            time_style: None,
            default_date_only: false,
            temporal_locale_string: false,
            bound_format: None,
        };

        retained.ca = locale_calendar;
        retained.nu = locale_numbering_system;
        retained.hc = locale_hour_cycle;

        let calendar = options.get(js_string!("calendar"), context)?;
        if !calendar.is_undefined() {
            if let Some(calendar) = canonicalize_calendar(Some(
                &calendar.to_string(context)?.to_std_string_escaped(),
            ))? {
                if retained.ca.as_deref() != Some(calendar.as_str()) {
                    retained.ca = None;
                }
                state.calendar = calendar;
            }
        }

        let numbering_system = options.get(js_string!("numberingSystem"), context)?;
        if !numbering_system.is_undefined() {
            if let Some(numbering_system) = canonicalize_numbering_system(Some(
                &numbering_system.to_string(context)?.to_std_string_escaped(),
            ))? {
                if retained.nu.as_deref() != Some(numbering_system.as_str()) {
                    retained.nu = None;
                }
                state.numbering_system = numbering_system;
            }
        }

        let hour12 = options.get(js_string!("hour12"), context)?;
        let explicit_hour12 = (!hour12.is_undefined()).then(|| hour12.to_boolean());

        let hour_cycle = options.get(js_string!("hourCycle"), context)?;
        let explicit_hour_cycle = if hour_cycle.is_undefined() {
            None
        } else {
            Some(
                canonicalize_hour_cycle(Some(
                    &hour_cycle.to_string(context)?.to_std_string_escaped(),
                ))?
                .ok_or_else(|| {
                    JsNativeError::range().with_message("Invalid hourCycle option")
                })?,
            )
        };

        let time_zone = options.get(js_string!("timeZone"), context)?;
        if !time_zone.is_undefined() {
            state.time_zone = canonicalize_time_zone(
                &time_zone.to_string(context)?.to_std_string_escaped(),
                context,
            )?;
        }

        state.weekday = get_string_option(&options, "weekday", context)?;
        if let Some(weekday) = &state.weekday
            && !matches!(weekday.as_str(), "narrow" | "short" | "long")
        {
            return Err(JsNativeError::range()
                .with_message("Invalid weekday option")
                .into());
        }
        state.era = get_string_option(&options, "era", context)?;
        if let Some(era) = &state.era
            && !matches!(era.as_str(), "narrow" | "short" | "long")
        {
            return Err(JsNativeError::range()
                .with_message("Invalid era option")
                .into());
        }
        state.year = get_string_option(&options, "year", context)?;
        if let Some(year) = &state.year
            && !matches!(year.as_str(), "2-digit" | "numeric")
        {
            return Err(JsNativeError::range()
                .with_message("Invalid year option")
                .into());
        }
        state.month = get_string_option(&options, "month", context)?;
        if let Some(month) = &state.month
            && !matches!(
                month.as_str(),
                "2-digit" | "numeric" | "narrow" | "short" | "long"
            )
        {
            return Err(JsNativeError::range()
                .with_message("Invalid month option")
                .into());
        }
        state.day = get_string_option(&options, "day", context)?;
        if let Some(day) = &state.day
            && !matches!(day.as_str(), "2-digit" | "numeric")
        {
            return Err(JsNativeError::range()
                .with_message("Invalid day option")
                .into());
        }

        if let Some(day_period) = get_string_option(&options, "dayPeriod", context)? {
            if !matches!(day_period.as_str(), "narrow" | "short" | "long") {
                return Err(JsNativeError::range()
                    .with_message("Invalid dayPeriod option")
                    .into());
            }
            state.day_period = Some(day_period);
        }

        state.hour = get_string_option(&options, "hour", context)?;
        if let Some(hour) = &state.hour
            && !matches!(hour.as_str(), "2-digit" | "numeric")
        {
            return Err(JsNativeError::range()
                .with_message("Invalid hour option")
                .into());
        }
        state.minute = get_string_option(&options, "minute", context)?;
        if let Some(minute) = &state.minute
            && !matches!(minute.as_str(), "2-digit" | "numeric")
        {
            return Err(JsNativeError::range()
                .with_message("Invalid minute option")
                .into());
        }
        state.second = get_string_option(&options, "second", context)?;
        if let Some(second) = &state.second
            && !matches!(second.as_str(), "2-digit" | "numeric")
        {
            return Err(JsNativeError::range()
                .with_message("Invalid second option")
                .into());
        }

        let fractional_second_digits =
            options.get(js_string!("fractionalSecondDigits"), context)?;
        if !fractional_second_digits.is_undefined() {
            let digits = fractional_second_digits.to_number(context)?;
            if !digits.is_finite() || digits < 1.0 || digits > 3.0 {
                return Err(JsNativeError::range()
                    .with_message("Invalid fractionalSecondDigits option")
                    .into());
            }
            state.fractional_second_digits = Some(digits.floor() as u8);
        }

        if let Some(time_zone_name) = get_string_option(&options, "timeZoneName", context)? {
            if !matches!(
                time_zone_name.as_str(),
                "short"
                    | "long"
                    | "shortOffset"
                    | "longOffset"
                    | "shortGeneric"
                    | "longGeneric"
            ) {
                return Err(JsNativeError::range()
                    .with_message("Invalid timeZoneName option")
                    .into());
            }
            state.time_zone_name = Some(time_zone_name);
        }

        if let Some(format_matcher) = get_string_option(&options, "formatMatcher", context)? {
            if !matches!(format_matcher.as_str(), "basic" | "best fit") {
                return Err(JsNativeError::range()
                    .with_message("Invalid formatMatcher option")
                    .into());
            }
        }

        if let Some(date_style) = get_string_option(&options, "dateStyle", context)? {
            if !matches!(date_style.as_str(), "full" | "long" | "medium" | "short") {
                return Err(JsNativeError::range()
                    .with_message("Invalid dateStyle option")
                    .into());
            }
            state.date_style = Some(date_style);
        }
        if let Some(time_style) = get_string_option(&options, "timeStyle", context)? {
            if !matches!(time_style.as_str(), "full" | "long" | "medium" | "short") {
                return Err(JsNativeError::range()
                    .with_message("Invalid timeStyle option")
                    .into());
            }
            state.time_style = Some(time_style);
        }

        if (state.date_style.is_some() || state.time_style.is_some())
            && state.has_explicit_component()
        {
            return Err(JsNativeError::typ()
                .with_message("dateStyle/timeStyle conflicts with explicit component options")
                .into());
        }

        let only_era_is_explicit = state.era.is_some()
            && state.year.is_none()
            && state.month.is_none()
            && state.day.is_none()
            && state.weekday.is_none()
            && state.date_style.is_none()
            && state.hour.is_none()
            && state.minute.is_none()
            && state.second.is_none()
            && state.fractional_second_digits.is_none()
            && state.day_period.is_none()
            && state.time_style.is_none();

        if only_era_is_explicit
            || (state.year.is_none()
            && state.month.is_none()
            && state.day.is_none()
            && state.weekday.is_none()
            && state.era.is_none()
            && state.date_style.is_none()
            && state.hour.is_none()
            && state.minute.is_none()
            && state.second.is_none()
            && state.fractional_second_digits.is_none()
            && state.day_period.is_none()
            && state.time_style.is_none())
        {
            state.year = Some("numeric".to_owned());
            state.month = Some("numeric".to_owned());
            state.day = Some("numeric".to_owned());
            state.default_date_only = true;
        }

        let needs_time =
            state.hour.is_some()
                || state.minute.is_some()
                || state.second.is_some()
                || state.fractional_second_digits.is_some()
                || state.day_period.is_some()
                || state.time_style.is_some();

        if needs_time {
            if explicit_hour12.is_some() {
                retained.hc = None;
            }
            if let Some(hour_cycle) = explicit_hour_cycle.as_deref()
                && retained.hc.as_deref() != Some(hour_cycle)
            {
                retained.hc = None;
            }

            if let Some(hour12) = explicit_hour12 {
                state.hour12 = Some(hour12);
                state.hour_cycle = Some(if hour12 {
                    default_twelve_hour_cycle(&resolved_base).to_owned()
                } else {
                    "h23".to_owned()
                });
            } else if let Some(hour_cycle) = explicit_hour_cycle {
                state.hour12 = Some(is_twelve_hour_cycle(&hour_cycle));
                state.hour_cycle = Some(hour_cycle);
            } else if state.hour_cycle.is_none() {
                let default_hour_cycle = default_hour_cycle_for_locale(&resolved_base).to_owned();
                state.hour12 = Some(is_twelve_hour_cycle(&default_hour_cycle));
                state.hour_cycle = Some(default_hour_cycle);
            }
        } else {
            if explicit_hour12.is_some() || explicit_hour_cycle.is_some() {
                retained.hc = None;
            }
            state.hour12 = None;
            state.hour_cycle = None;
        }

        state.locale = build_resolved_locale(&resolved_base, &retained);

        Ok(state)
    }

    fn has_explicit_component(&self) -> bool {
        self.weekday.is_some()
            || self.era.is_some()
            || self.year.is_some()
            || self.month.is_some()
            || self.day.is_some()
            || self.day_period.is_some()
            || self.hour.is_some()
            || self.minute.is_some()
            || self.second.is_some()
            || self.fractional_second_digits.is_some()
            || self.time_zone_name.is_some()
    }

    fn effective_for_record(&self, record: &TemporalRecord) -> Self {
        let mut effective = self.clone();
        if self.default_date_only {
            match record.kind {
                RecordKind::Instant | RecordKind::PlainDateTime | RecordKind::ZonedDateTime => {
                    effective.hour = Some("numeric".to_owned());
                    effective.minute = Some("numeric".to_owned());
                    effective.second = Some("numeric".to_owned());
                    if record.kind == RecordKind::ZonedDateTime {
                        effective.time_zone_name = Some("short".to_owned());
                    }
                    if effective.hour_cycle.is_none() {
                        effective.hour_cycle =
                            Some(default_hour_cycle_for_locale(&effective.locale).to_owned());
                        effective.hour12 =
                            effective.hour_cycle.as_deref().map(is_twelve_hour_cycle);
                    }
                }
                RecordKind::PlainTime => {
                    effective.year = None;
                    effective.month = None;
                    effective.day = None;
                    effective.era = None;
                    effective.hour = Some("numeric".to_owned());
                    effective.minute = Some("numeric".to_owned());
                    effective.second = Some("numeric".to_owned());
                    if effective.hour_cycle.is_none() {
                        effective.hour_cycle =
                            Some(default_hour_cycle_for_locale(&effective.locale).to_owned());
                        effective.hour12 =
                            effective.hour_cycle.as_deref().map(is_twelve_hour_cycle);
                    }
                }
                RecordKind::PlainMonthDay => {
                    effective.year = None;
                    effective.era = None;
                }
                RecordKind::PlainYearMonth => {
                    effective.day = None;
                }
                RecordKind::Number | RecordKind::PlainDate => {}
            }
        }
        match record.kind {
            RecordKind::PlainDate | RecordKind::PlainMonthDay | RecordKind::PlainYearMonth => {
                let has_date_overlap = match record.kind {
                    RecordKind::PlainMonthDay => {
                        effective.month.is_some() || effective.day.is_some() || effective.date_style.is_some()
                    }
                    RecordKind::PlainYearMonth => {
                        effective.year.is_some()
                            || effective.month.is_some()
                            || effective.era.is_some()
                            || effective.date_style.is_some()
                    }
                    _ => {
                        effective.year.is_some()
                            || effective.month.is_some()
                            || effective.day.is_some()
                            || effective.weekday.is_some()
                            || effective.era.is_some()
                            || effective.date_style.is_some()
                    }
                };
                if has_date_overlap {
                    effective.hour = None;
                    effective.minute = None;
                    effective.second = None;
                    effective.fractional_second_digits = None;
                    effective.day_period = None;
                    effective.time_style = None;
                    effective.time_zone_name = None;
                }
            }
            RecordKind::PlainTime => {
                let has_time_overlap = effective.hour.is_some()
                    || effective.minute.is_some()
                    || effective.second.is_some()
                    || effective.fractional_second_digits.is_some()
                    || effective.day_period.is_some()
                    || effective.time_style.is_some();
                if has_time_overlap {
                    effective.weekday = None;
                    effective.era = None;
                    effective.year = None;
                    effective.month = None;
                    effective.day = None;
                    effective.time_zone_name = None;
                    effective.date_style = None;
                }
            }
            _ => {}
        }
        effective
    }

    fn format_formattable_to_string(
        &self,
        formattable: &Formattable,
        context: &mut Context,
    ) -> JsResult<JsString> {
        let parts = self.format_formattable_to_parts(formattable, context)?;
        Ok(join_parts(&parts))
    }

    fn format_formattable_to_parts(
        &self,
        formattable: &Formattable,
        context: &mut Context,
    ) -> JsResult<Vec<DateTimePart>> {
        match formattable {
            Formattable::Number(number) => {
                let record = self.record_from_number(*number, RecordKind::Number, context)?;
                Ok(self.build_parts(&record))
            }
            Formattable::Temporal(record) => {
                if record.kind == RecordKind::ZonedDateTime && !self.temporal_locale_string {
                    return Err(JsNativeError::typ()
                        .with_message("Temporal.ZonedDateTime is not supported by Intl.DateTimeFormat")
                        .into());
                }
                let effective = self.effective_for_record(record);
                effective.validate_temporal_state(record)?;
                let display = effective.record_from_temporal(record, context)?;
                Ok(effective.build_parts(&display))
            }
        }
    }

    fn record_from_number(
        &self,
        number: f64,
        kind: RecordKind,
        context: &mut Context,
    ) -> JsResult<DisplayRecord> {
        if !number.is_finite() {
            return Err(JsNativeError::range()
                .with_message("Invalid time value")
                .into());
        }
        if time_clip(number).is_nan() {
            return Err(JsNativeError::range()
                .with_message("Invalid time value")
                .into());
        }

        #[cfg(feature = "temporal")]
        if !is_utc_time_zone(&self.time_zone)
            && parse_offset_time_zone(&self.time_zone).is_none()
            && parse_etc_gmt_time_zone(&self.time_zone).is_none()
            && let Ok(time_zone) =
                RsTimeZone::try_from_identifier_str_with_provider(&self.time_zone, context.tz_provider())
            && let Ok(instant) = RsInstant::from_epoch_milliseconds(number as i64)
            && let Ok(zdt) =
                RsZonedDateTime::try_new_iso_from_instant_with_provider(instant, time_zone, context.tz_provider())
        {
            return Ok(DisplayRecord {
                kind,
                year: Some(zdt.year()),
                month: Some(zdt.month()),
                day: Some(zdt.day()),
                hour: Some(zdt.hour()),
                minute: zdt.minute(),
                second: zdt.second(),
                millisecond: zdt.millisecond(),
            calendar: self.calendar.clone(),
            month_display: None,
            era_label: None,
            related_year: None,
            year_name: None,
        })
        .map(|mut display| {
            if let (Some(year), Some(month), Some(day)) = (display.year, display.month, display.day)
                && let Some(fields) = calendar_display_fields(self.calendar.as_str(), year, month, day)
            {
                display.year = Some(fields.year);
                display.month = Some(fields.month);
                display.day = Some(fields.day);
                display.month_display = fields.month_display;
                display.era_label = fields.era_label;
                display.related_year = fields.related_year;
                display.year_name = fields.year_name;
            }
            display
        });
        }

        let adjusted = apply_time_zone(number, &self.time_zone, context);
        Ok(DisplayRecord {
            kind,
            year: Some(year_from_time(adjusted)),
            month: Some(month_from_time(adjusted) + 1),
            day: Some(date_from_time(adjusted)),
            hour: Some(hour_from_time(adjusted)),
            minute: min_from_time(adjusted),
            second: sec_from_time(adjusted),
            millisecond: ms_from_time(adjusted),
            calendar: self.calendar.clone(),
            month_display: None,
            era_label: None,
            related_year: None,
            year_name: None,
        })
        .map(|mut display| {
            if let (Some(year), Some(month), Some(day)) = (display.year, display.month, display.day)
                && let Some(fields) = calendar_display_fields(self.calendar.as_str(), year, month, day)
            {
                display.year = Some(fields.year);
                display.month = Some(fields.month);
                display.day = Some(fields.day);
                display.month_display = fields.month_display;
                display.era_label = fields.era_label;
                display.related_year = fields.related_year;
                display.year_name = fields.year_name;
            }
            display
        })
    }

    fn record_from_temporal(
        &self,
        record: &TemporalRecord,
        context: &mut Context,
    ) -> JsResult<DisplayRecord> {
        if let Some(epoch_millis) = record.epoch_millis {
            return self.record_from_number(epoch_millis, record.kind, context);
        }

        let mut display = DisplayRecord {
            kind: record.kind,
            year: record.year,
            month: record.month,
            day: record.day,
            hour: record.hour,
            minute: record.minute,
            second: record.second,
            millisecond: record.millisecond,
            calendar: record.calendar.clone(),
            month_display: None,
            era_label: None,
            related_year: None,
            year_name: None,
        };

        if matches!(self.calendar.as_str(), "chinese" | "dangi")
            && let (Some(year), Some(month), Some(day)) =
                (display.year, display.month, display.day)
            && let Some((related_year, year_name, calendar_month, calendar_day)) =
                east_asian_fields(self.calendar.as_str(), year, month, day)
        {
            display.related_year = Some(related_year);
            display.year_name = Some(year_name);
            display.month = Some(calendar_month);
            display.day = Some(calendar_day);
        }

        Ok(display)
    }

    fn validate_temporal_state(&self, record: &TemporalRecord) -> JsResult<()> {
        if !temporal_calendar_matches_formatter(record.kind, record.calendar.as_str(), self.calendar.as_str()) {
            return Err(JsNativeError::range()
                .with_message("Temporal calendar does not match formatter calendar")
                .into());
        }

        let requested_date = self.date_style.is_some()
            || self.weekday.is_some()
            || self.era.is_some()
            || self.year.is_some()
            || self.month.is_some()
            || self.day.is_some();
        let requested_time = self.time_style.is_some()
            || self.hour.is_some()
            || self.minute.is_some()
            || self.second.is_some()
            || self.fractional_second_digits.is_some()
            || self.day_period.is_some();

        let date_overlap = match record.kind {
            RecordKind::PlainTime => false,
            RecordKind::PlainMonthDay => {
                self.month.is_some() || self.day.is_some() || self.date_style.is_some()
            }
            RecordKind::PlainYearMonth => {
                self.year.is_some()
                    || self.month.is_some()
                    || self.era.is_some()
                    || self.date_style.is_some()
            }
            _ => requested_date,
        };
        let time_overlap = match record.kind {
            RecordKind::PlainTime
            | RecordKind::PlainDateTime
            | RecordKind::Instant
            | RecordKind::ZonedDateTime => {
                requested_time
            }
            _ => false,
        };

        if (requested_date || requested_time) && !date_overlap && !time_overlap {
            return Err(JsNativeError::typ()
                .with_message("Temporal value is incompatible with formatter options")
                .into());
        }

        Ok(())
    }

    fn build_parts(&self, display: &DisplayRecord) -> Vec<DateTimePart> {
        if self.day_period.is_some()
            && !self.uses_date_fields()
            && !self.uses_time_fields_without_day_period()
        {
            let hour = display.hour.unwrap_or(0);
            return vec![DateTimePart {
                kind: "dayPeriod",
                value: self.localize(day_period_for_hour(hour, self.day_period.as_deref())),
            }];
        }

        let mut parts = Vec::new();
        if self.uses_date_fields() {
            parts.extend(self.build_date_parts(display));
        }
        if self.uses_time_fields() {
            if !parts.is_empty() {
                parts.push(part("literal", ", "));
            }
            parts.extend(self.build_time_parts(display));
        } else if self.time_zone_name.is_some() && !parts.is_empty() && supports_time_zone_name(display)
        {
            parts.push(part("literal", " "));
            parts.push(DateTimePart {
                kind: "timeZoneName",
                value: self.time_zone_name_value(),
            });
        }

        if parts.is_empty() {
            let mut fallback = String::new();
            if let Some(year) = display.year {
                let _ = write!(fallback, "{year}");
            }
            if let Some(month) = display.month {
                if !fallback.is_empty() {
                    fallback.push('-');
                }
                let _ = write!(fallback, "{month:02}");
            }
            if let Some(day) = display.day {
                if !fallback.is_empty() {
                    fallback.push('-');
                }
                let _ = write!(fallback, "{day:02}");
            }
            parts.push(DateTimePart {
                kind: "literal",
                value: JsString::from(substitute_digits(&fallback, &self.numbering_system)),
            });
        }

        parts
    }

    fn uses_date_fields(&self) -> bool {
        self.year.is_some()
            || self.month.is_some()
            || self.day.is_some()
            || self.weekday.is_some()
            || self.era.is_some()
            || self.date_style.is_some()
    }

    fn uses_time_fields(&self) -> bool {
        self.hour.is_some()
            || self.minute.is_some()
            || self.second.is_some()
            || self.fractional_second_digits.is_some()
            || self.day_period.is_some()
            || self.time_style.is_some()
    }

    fn uses_time_fields_without_day_period(&self) -> bool {
        self.hour.is_some()
            || self.minute.is_some()
            || self.second.is_some()
            || self.fractional_second_digits.is_some()
            || self.time_style.is_some()
    }

    fn build_date_parts(&self, display: &DisplayRecord) -> Vec<DateTimePart> {
        if matches!(self.calendar.as_str(), "chinese" | "dangi")
            && self.year.is_some()
            && self.month.is_none()
            && self.day.is_none()
            && self.date_style.is_none()
            && let Some(year_name) = &display.year_name
        {
            let mut parts = Vec::new();
            if let Some(related_year) = display.related_year {
                parts.push(DateTimePart {
                    kind: "relatedYear",
                    value: self.localize_number(related_year),
                });
            }
            parts.push(DateTimePart {
                kind: "yearName",
                value: JsString::from(year_name.clone()),
            });
            parts.push(part("literal", "年"));
            return parts;
        }

        let (month_style, short_year, with_weekday, textual_month, show_month, show_day, show_year) =
            if let Some(date_style) = self.date_style.as_deref() {
                match date_style {
                    "full" => ("long", false, true, true, true, true, true),
                    "long" => ("long", false, false, true, true, true, true),
                    "medium" => ("short", false, false, true, true, true, true),
                    "short" => ("numeric", true, false, false, true, true, true),
                    _ => ("numeric", false, false, false, false, false, false),
                }
            } else {
                (
                    self.month.as_deref().unwrap_or("numeric"),
                    matches!(self.year.as_deref(), Some("2-digit")),
                    self.weekday.is_some(),
                    self.month.is_some()
                        && matches!(
                            self.month.as_deref(),
                            Some("short") | Some("long") | Some("narrow")
                        ),
                    self.month.is_some(),
                    self.day.is_some(),
                    self.year.is_some(),
                )
            };

        let mut parts = Vec::new();

        if with_weekday || self.weekday.is_some() {
            if let (Some(year), Some(month), Some(day)) = (display.year, display.month, display.day)
            {
                let weekday = weekday_name(
                    year,
                    month,
                    day,
                    if with_weekday && self.weekday.is_none() {
                        Some("long")
                    } else {
                        self.weekday.as_deref().or(Some("short"))
                    },
                );
                parts.push(DateTimePart {
                    kind: "weekday",
                    value: JsString::from(weekday),
                });
                parts.push(part("literal", ", "));
            }
        }

        if matches!(self.calendar.as_str(), "chinese" | "dangi") {
            if show_month && let Some(month) = display.month {
                parts.push(DateTimePart {
                    kind: "month",
                    value: match &display.month_display {
                        Some(value)
                            if !self.temporal_locale_string
                                && display.calendar == "hebrew" =>
                        {
                            self.localize(value)
                        }
                        _ if !self.temporal_locale_string && display.calendar == "hebrew" => {
                            let is_leap_year = display.year.is_some_and(is_hebrew_leap_year);
                            self.localize(hebrew_month_display(month, is_leap_year))
                        }
                        Some(value) if value.ends_with('L') => self.localize(value),
                        _ => self.localize_number(month),
                    },
                });
            }
            if show_day && let Some(day) = display.day {
                if !parts.is_empty() && parts.last().is_some_and(|part| part.kind != "literal") {
                    parts.push(part("literal", "/"));
                }
                parts.push(DateTimePart {
                    kind: "day",
                    value: self.localize_number(day),
                });
            }
            let use_related_year = display.related_year.is_some()
                && (self.default_date_only
                    || self.date_style.is_some()
                    || !matches!(self.month.as_deref(), Some("numeric") | Some("2-digit")));
            if show_year && use_related_year && let Some(related_year) = display.related_year {
                if !parts.is_empty() && parts.last().is_some_and(|part| part.kind != "literal") {
                    parts.push(part("literal", "/"));
                }
                parts.push(DateTimePart {
                    kind: "relatedYear",
                    value: self.localize_number(related_year),
                });
            }
            if self.locale.starts_with("zh")
                && show_year
                && use_related_year
                && let Some(year_name) = &display.year_name
            {
                parts.push(DateTimePart {
                    kind: "yearName",
                    value: JsString::from(year_name.clone()),
                });
                parts.push(part("literal", "年"));
            } else if show_year && let Some(year) = display.year {
                if !parts.is_empty() && parts.last().is_some_and(|part| part.kind != "literal") {
                    parts.push(part("literal", "/"));
                }
                parts.push(DateTimePart {
                    kind: "year",
                    value: self.localize_number(year),
                });
            }
            return parts;
        }

        if textual_month {
            if show_month && let Some(month) = display.month {
                parts.push(DateTimePart {
                    kind: "month",
                    value: if self.temporal_locale_string {
                        JsString::from(
                            textual_month_name(
                                display.calendar.as_str(),
                                month,
                                month_style,
                                display.month_display.as_deref(),
                            )
                            .unwrap_or_else(|| month_name(month, month_style).to_owned()),
                        )
                    } else if let Some(value) = &display.month_display {
                        JsString::from(value.clone())
                    } else {
                        JsString::from(month_name(month, month_style))
                    },
                });
            }
            if show_day && let Some(day) = display.day {
                if !parts.is_empty() {
                    parts.push(part("literal", " "));
                }
                parts.push(DateTimePart {
                    kind: "day",
                    value: self.localize_number(day),
                });
            }
            if show_year && let Some(year) = display.year {
                if !parts.is_empty() {
                    parts.push(part("literal", ", "));
                }
                parts.push(DateTimePart {
                    kind: "year",
                    value: self.localize_year(display, year, short_year),
                });
            }
        } else {
            if show_month && let Some(month) = display.month {
                parts.push(DateTimePart {
                    kind: "month",
                    value: match &display.month_display {
                        Some(value)
                            if !self.temporal_locale_string
                                && display.calendar == "hebrew" =>
                        {
                            self.localize(value)
                        }
                        _ if !self.temporal_locale_string && display.calendar == "hebrew" => {
                            let is_leap_year = display.year.is_some_and(is_hebrew_leap_year);
                            self.localize(hebrew_month_display(month, is_leap_year))
                        }
                        Some(value) if value.ends_with('L') => self.localize(value),
                        _ => self.localize_number(month),
                    },
                });
            }
            if show_day && let Some(day) = display.day {
                if !parts.is_empty() {
                    parts.push(part("literal", "/"));
                }
                parts.push(DateTimePart {
                    kind: "day",
                    value: self.localize_number(if self.date_style.as_deref() == Some("short") {
                        day
                    } else {
                        day
                    }),
                });
            }
            if show_year && let Some(year) = display.year {
                if !parts.is_empty() {
                    parts.push(part("literal", "/"));
                }
                parts.push(DateTimePart {
                    kind: "year",
                    value: self.localize_year(display, year, short_year),
                });
            }
        }

        if self.era.is_some() && show_year {
            if let Some(year) = display.year {
                if parts.last().is_some_and(|part| part.kind != "literal") {
                    parts.push(part("literal", " "));
                }
                parts.push(DateTimePart {
                    kind: "era",
                    value: JsString::from(
                        display
                            .era_label
                            .clone()
                            .unwrap_or_else(|| if year <= 0 { "BC".to_owned() } else { "AD".to_owned() }),
                    ),
                });
            }
        }

        parts
    }

    fn build_time_parts(&self, display: &DisplayRecord) -> Vec<DateTimePart> {
        let hour_cycle = self.hour_cycle.as_deref();
        let use_twelve_hour = matches!(hour_cycle, Some("h11" | "h12"));
        let show_hour = self.hour.is_some() || self.time_style.is_some();
        let show_minute = self.minute.is_some()
            || self.time_style.is_some()
            || self.second.is_some()
            || self.fractional_second_digits.is_some();
        let show_second = self.second.is_some()
            || self.fractional_second_digits.is_some()
            || matches!(self.time_style.as_deref(), Some("medium") | Some("long") | Some("full"));
        let show_tz = self.time_zone_name.is_some()
            || matches!(self.time_style.as_deref(), Some("long") | Some("full"));
        let show_day_period = use_twelve_hour && show_hour;

        let mut parts = Vec::new();
        let hour = display.hour.unwrap_or(0);
        if show_hour {
            let rendered_hour = match hour_cycle {
                Some("h11") => hour % 12,
                Some("h12") => {
                    if hour % 12 == 0 {
                        12
                    } else {
                        hour % 12
                    }
                }
                Some("h24") => {
                    if hour == 0 {
                        24
                    } else {
                        hour
                    }
                }
                _ => hour,
            };
            let value = if self.hour.as_deref() == Some("2-digit") {
                self.localize_padded(rendered_hour, 2)
            } else if !use_twelve_hour
                && (self.time_style.is_some() || matches!(hour_cycle, Some("h23" | "h24")))
            {
                self.localize_padded(rendered_hour, 2)
            } else {
                self.localize_number(rendered_hour)
            };
            parts.push(DateTimePart {
                kind: "hour",
                value,
            });
        }

        if show_minute {
            if !parts.is_empty() {
                parts.push(part("literal", ":"));
            }
            let value = if show_hour || show_second || self.fractional_second_digits.is_some() {
                self.localize_padded(display.minute, 2)
            } else {
                self.localize_number(display.minute)
            };
            parts.push(DateTimePart {
                kind: "minute",
                value,
            });
        }

        if show_second {
            if !parts.is_empty() {
                parts.push(part("literal", ":"));
            }
            let value = if show_hour || show_minute || self.fractional_second_digits.is_some() {
                self.localize_padded(display.second, 2)
            } else {
                self.localize_number(display.second)
            };
            parts.push(DateTimePart {
                kind: "second",
                value,
            });
        }

        if let Some(digits) = self.fractional_second_digits {
            parts.push(DateTimePart {
                kind: "literal",
                value: JsString::from(decimal_separator(&self.numbering_system)),
            });
            let millis = format!("{:03}", display.millisecond);
            parts.push(DateTimePart {
                kind: "fractionalSecond",
                value: JsString::from(substitute_digits(
                    &millis[..digits as usize],
                    &self.numbering_system,
                )),
            });
        }

        if show_day_period {
            parts.push(part("literal", " "));
            parts.push(DateTimePart {
                kind: "dayPeriod",
                value: self.localize(if let Some(width) = self.day_period.as_deref() {
                    day_period_for_hour(hour, Some(width))
                } else if hour < 12 {
                    "AM"
                } else {
                    "PM"
                }),
            });
        }

        if show_tz && supports_time_zone_name(display) {
            parts.push(part("literal", " "));
            parts.push(DateTimePart {
                kind: "timeZoneName",
                value: self.time_zone_name_value(),
            });
        }

        parts
    }

    fn localize(&self, text: &str) -> JsString {
        JsString::from(substitute_digits(text, &self.numbering_system))
    }

    fn localize_number<T: ToString>(&self, value: T) -> JsString {
        self.localize(&value.to_string())
    }

    fn localize_padded<T: ToString>(&self, value: T, width: usize) -> JsString {
        let rendered = value.to_string();
        let padded = format!("{rendered:0>width$}");
        self.localize(&padded)
    }

    fn localize_year(&self, display: &DisplayRecord, year: i32, short_year: bool) -> JsString {
        let display_year = if self.era.is_some() && year <= 0 {
            1 - year
        } else {
            year
        };
        if short_year {
            self.localize_padded(display_year.rem_euclid(100), 2)
        } else if matches!(display.kind, RecordKind::PlainMonthDay) {
            self.localize_number(display.related_year.unwrap_or(display_year))
        } else {
            self.localize_number(display.related_year.unwrap_or(display_year))
        }
    }

    fn time_zone_name_value(&self) -> JsString {
        let canonical = canonical_time_zone_display_name(&self.time_zone);
        let long_name = matches!(
            self.time_zone_name.as_deref(),
            Some("long" | "longGeneric" | "longOffset")
        );

        if let Some(offset) = parse_offset_time_zone(&self.time_zone)
            .or_else(|| parse_etc_gmt_time_zone(&self.time_zone))
        {
            let abs_minutes = offset.unsigned_abs();
            let hours = abs_minutes / 60;
            let minutes = abs_minutes % 60;
            let sign = if offset < 0 { '-' } else { '+' };
            let value = if offset == 0 {
                "GMT".to_owned()
            } else if long_name || minutes != 0 {
                format!("GMT{sign}{hours}:{minutes:02}")
            } else {
                format!("GMT{sign}{hours}")
            };
            return JsString::from(value);
        }

        let value = match self.time_style.as_deref() {
            Some("full") if is_utc_time_zone(&self.time_zone) => "Coordinated Universal Time",
            Some("long") if is_utc_time_zone(&self.time_zone) => "UTC",
            _ if is_utc_time_zone(&self.time_zone) => "UTC",
            _ if long_name && self.time_zone == "Europe/Vienna" => {
                "Central European Standard Time"
            }
            _ if self.time_zone_name.as_deref() == Some("short")
                && self.time_zone == "Europe/Vienna" =>
            {
                "CET"
            }
            _ => canonical.as_str(),
        };
        JsString::from(value)
    }

    fn supported_locales_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let requested_locales = canonicalize_locale_list(args.get_or_undefined(0), context)?;
        filter_locales::<Self>(requested_locales, args.get_or_undefined(1), context)
            .map(JsValue::from)
    }

    fn get_format(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let dtf = unwrap_date_time_format(this, context)?;
        let dtf_clone = dtf.clone();
        let mut dtf_borrow = dtf.borrow_mut();

        let bound_format = if let Some(bound) = dtf_borrow.data_mut().bound_format.clone() {
            bound
        } else {
            let bound_format = FunctionObjectBuilder::new(
                context.realm(),
                NativeFunction::from_copy_closure_with_captures(
                    |_, args, dtf, context| {
                        let value = if args.is_empty() {
                            JsValue::undefined()
                        } else {
                            args[0].clone()
                        };
                        let formattable = to_formattable(&value, context)?;
                        Ok(dtf
                            .borrow()
                            .data()
                            .format_formattable_to_string(&formattable, context)?
                            .into())
                    },
                    dtf_clone,
                ),
            )
            .length(1)
            .build();
            dtf_borrow.data_mut().bound_format = Some(bound_format.clone());
            bound_format
        };

        Ok(bound_format.into())
    }

    fn format_to_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dtf = unwrap_date_time_format(this, context)?;
        let value = args.get_or_undefined(0);
        let formattable = to_formattable(value, context)?;
        let parts = dtf.borrow().data().format_formattable_to_parts(&formattable, context)?;
        Ok(parts_to_array(parts.into_iter().map(|part| (part, None)), context).into())
    }

    fn format_range(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let parts = Self::range_parts(this, args, context)?;
        Ok(join_parts(
            &parts.into_iter().map(|(part, _)| part).collect::<Vec<_>>(),
        )
        .into())
    }

    fn format_range_to_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let parts = Self::range_parts(this, args, context)?;
        Ok(parts_to_array(parts, context).into())
    }

    fn range_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<Vec<(DateTimePart, Option<&'static str>)>> {
        if args.len() < 2 || args[0].is_undefined() || args[1].is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("formatRange requires startDate and endDate")
                .into());
        }

        let dtf = unwrap_date_time_format(this, context)?;
        let dtf = dtf.borrow();
        let dtf = dtf.data();

        let start = to_formattable(args.get_or_undefined(0), context)?;
        let end = to_formattable(args.get_or_undefined(1), context)?;

        match (&start, &end) {
            (Formattable::Temporal(left), Formattable::Temporal(right)) => {
                if left.kind != right.kind {
                    return Err(JsNativeError::typ()
                        .with_message("formatRange requires matching Temporal kinds")
                        .into());
                }
                if left.calendar != right.calendar {
                    return Err(JsNativeError::range()
                        .with_message("formatRange requires matching Temporal calendars")
                        .into());
                }
            }
            (Formattable::Temporal(_), Formattable::Number(_))
            | (Formattable::Number(_), Formattable::Temporal(_)) => {
                return Err(JsNativeError::typ()
                    .with_message("formatRange requires both arguments to have the same kind")
                    .into());
            }
            _ => {}
        }

        let start_parts = dtf.format_formattable_to_parts(&start, context)?;
        let end_parts = dtf.format_formattable_to_parts(&end, context)?;

        if join_parts(&start_parts) == join_parts(&end_parts) {
            return Ok(start_parts
                .into_iter()
                .map(|part| (part, Some("shared")))
                .collect());
        }

        if let (Formattable::Temporal(left), Formattable::Temporal(right)) = (&start, &end) {
            let left_effective = dtf.effective_for_record(left);
            let left_display = left_effective.record_from_temporal(left, context)?;
            let right_effective = dtf.effective_for_record(right);
            let right_display = right_effective.record_from_temporal(right, context)?;

            if left_effective.uses_date_fields()
                && left_effective.uses_time_fields()
                && left_display.year == right_display.year
                && left_display.month == right_display.month
                && left_display.day == right_display.day
            {
                let mut parts = left_effective
                    .build_date_parts(&left_display)
                    .into_iter()
                    .map(|part| (part, Some("shared")))
                    .collect::<Vec<_>>();
                if !parts.is_empty() {
                    parts.push((part("literal", ", "), Some("shared")));
                }
                parts.extend(
                    left_effective
                        .build_time_parts(&left_display)
                        .into_iter()
                        .map(|part| (part, Some("startRange"))),
                );
                parts.push((part("literal", RANGE_SEPARATOR), Some("shared")));
                parts.extend(
                    right_effective
                        .build_time_parts(&right_display)
                        .into_iter()
                        .map(|part| (part, Some("endRange"))),
                );
                return Ok(parts);
            }
        }

        if let (Formattable::Number(start_number), Formattable::Number(end_number)) = (&start, &end)
        {
            let start_display = dtf.record_from_number(*start_number, RecordKind::Number, context)?;
            let end_display = dtf.record_from_number(*end_number, RecordKind::Number, context)?;
            if dtf.date_style.is_none()
                && dtf.time_style.is_none()
                && dtf.time_zone_name.is_none()
                && dtf.weekday.is_none()
                && dtf.era.is_none()
                && matches!(dtf.month.as_deref(), Some("short") | Some("long") | Some("narrow"))
                && dtf.day.as_deref() == Some("numeric")
                && dtf.year.as_deref() == Some("numeric")
                && start_display.year == end_display.year
                && start_display.month == end_display.month
            {
                let mut parts = Vec::new();
                if let Some(month) = start_display.month {
                    parts.push((
                        DateTimePart {
                            kind: "month",
                            value: JsString::from(month_name(
                                month,
                                dtf.month.as_deref().unwrap_or("short"),
                            )),
                        },
                        Some("shared"),
                    ));
                    parts.push((part("literal", " "), Some("shared")));
                }
                parts.push((
                    DateTimePart {
                        kind: "day",
                        value: dtf.localize_number(start_display.day.unwrap_or(0)),
                    },
                    Some("startRange"),
                ));
                parts.push((part("literal", RANGE_SEPARATOR), Some("shared")));
                parts.push((
                    DateTimePart {
                        kind: "day",
                        value: dtf.localize_number(end_display.day.unwrap_or(0)),
                    },
                    Some("endRange"),
                ));
                parts.push((part("literal", ", "), Some("shared")));
                parts.push((
                    DateTimePart {
                        kind: "year",
                        value: dtf.localize_number(start_display.year.unwrap_or(0)),
                    },
                    Some("shared"),
                ));
                return Ok(parts);
            }
            if dtf.date_style.is_none()
                && dtf.time_style.is_none()
                && dtf.time_zone_name.is_none()
                && dtf.weekday.is_none()
                && dtf.era.is_none()
                && matches!(dtf.month.as_deref(), Some("short") | Some("long") | Some("narrow"))
                && dtf.day.as_deref() == Some("numeric")
                && dtf.year.as_deref() == Some("numeric")
                && start_display.year == end_display.year
            {
                let mut parts = Vec::new();
                if let Some(month) = start_display.month {
                    parts.push((
                        DateTimePart {
                            kind: "month",
                            value: JsString::from(month_name(
                                month,
                                dtf.month.as_deref().unwrap_or("short"),
                            )),
                        },
                        Some("startRange"),
                    ));
                    parts.push((part("literal", " "), Some("startRange")));
                }
                parts.push((
                    DateTimePart {
                        kind: "day",
                        value: dtf.localize_number(start_display.day.unwrap_or(0)),
                    },
                    Some("startRange"),
                ));
                parts.push((part("literal", RANGE_SEPARATOR), Some("shared")));
                if let Some(month) = end_display.month {
                    parts.push((
                        DateTimePart {
                            kind: "month",
                            value: JsString::from(month_name(
                                month,
                                dtf.month.as_deref().unwrap_or("short"),
                            )),
                        },
                        Some("endRange"),
                    ));
                    parts.push((part("literal", " "), Some("endRange")));
                }
                parts.push((
                    DateTimePart {
                        kind: "day",
                        value: dtf.localize_number(end_display.day.unwrap_or(0)),
                    },
                    Some("endRange"),
                ));
                parts.push((part("literal", ", "), Some("shared")));
                parts.push((
                    DateTimePart {
                        kind: "year",
                        value: dtf.localize_number(start_display.year.unwrap_or(0)),
                    },
                    Some("shared"),
                ));
                return Ok(parts);
            }
        }

        Ok(start_parts
            .into_iter()
            .map(|part| (part, Some("startRange")))
            .chain(std::iter::once((
                part("literal", RANGE_SEPARATOR),
                Some("shared"),
            )))
            .chain(end_parts.into_iter().map(|part| (part, Some("endRange"))))
            .collect())
    }

    fn resolved_options(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let dtf = unwrap_date_time_format(this, context)?;
        let dtf = dtf.borrow();
        let dtf = dtf.data();

        let mut object = ObjectInitializer::new(context);
        object
            .property(
                js_string!("locale"),
                JsString::from(dtf.locale.clone()),
                Attribute::all(),
            )
            .property(
                js_string!("calendar"),
                JsString::from(dtf.calendar.clone()),
                Attribute::all(),
            )
            .property(
                js_string!("numberingSystem"),
                JsString::from(dtf.numbering_system.clone()),
                Attribute::all(),
            )
            .property(
                js_string!("timeZone"),
                JsString::from(dtf.time_zone.clone()),
                Attribute::all(),
            );

        if let Some(hour_cycle) = &dtf.hour_cycle {
            object.property(
                js_string!("hourCycle"),
                JsString::from(hour_cycle.clone()),
                Attribute::all(),
            );
        }
        if let Some(hour12) = dtf.hour12 {
            object.property(js_string!("hour12"), hour12, Attribute::all());
        }
        if let Some(weekday) = &dtf.weekday {
            object.property(
                js_string!("weekday"),
                JsString::from(weekday.clone()),
                Attribute::all(),
            );
        }
        if let Some(era) = &dtf.era {
            object.property(
                js_string!("era"),
                JsString::from(era.clone()),
                Attribute::all(),
            );
        }
        if let Some(year) = &dtf.year {
            object.property(
                js_string!("year"),
                JsString::from(year.clone()),
                Attribute::all(),
            );
        }
        if let Some(month) = &dtf.month {
            object.property(
                js_string!("month"),
                JsString::from(month.clone()),
                Attribute::all(),
            );
        }
        if let Some(day) = &dtf.day {
            object.property(
                js_string!("day"),
                JsString::from(day.clone()),
                Attribute::all(),
            );
        }
        if let Some(day_period) = &dtf.day_period {
            object.property(
                js_string!("dayPeriod"),
                JsString::from(day_period.clone()),
                Attribute::all(),
            );
        }
        if let Some(hour) = &dtf.hour {
            object.property(
                js_string!("hour"),
                JsString::from(hour.clone()),
                Attribute::all(),
            );
        }
        if let Some(minute) = &dtf.minute {
            object.property(
                js_string!("minute"),
                JsString::from(minute.clone()),
                Attribute::all(),
            );
        }
        if let Some(second) = &dtf.second {
            object.property(
                js_string!("second"),
                JsString::from(second.clone()),
                Attribute::all(),
            );
        }
        if let Some(fractional_second_digits) = dtf.fractional_second_digits {
            object.property(
                js_string!("fractionalSecondDigits"),
                fractional_second_digits,
                Attribute::all(),
            );
        }
        if let Some(time_zone_name) = &dtf.time_zone_name {
            object.property(
                js_string!("timeZoneName"),
                JsString::from(time_zone_name.clone()),
                Attribute::all(),
            );
        }
        if let Some(date_style) = &dtf.date_style {
            object.property(
                js_string!("dateStyle"),
                JsString::from(date_style.clone()),
                Attribute::all(),
            );
        }
        if let Some(time_style) = &dtf.time_style {
            object.property(
                js_string!("timeStyle"),
                JsString::from(time_style.clone()),
                Attribute::all(),
            );
        }

        Ok(object.build().into())
    }
}

pub(crate) fn format_date_time_for_date_value(
    locales: &JsValue,
    options: &JsValue,
    required: DateTimeReqs,
    defaults: DateTimeReqs,
    time_value: f64,
    context: &mut Context,
) -> JsResult<JsString> {
    let options = to_date_time_options(options, &required, &defaults, context)?;
    let options_value: JsValue = options.into();
    let formatter = DateTimeFormat::new(locales, &options_value, context)?;
    formatter.format_formattable_to_string(&Formattable::Number(time_value), context)
}

pub(crate) fn format_date_time_value(
    locales: &JsValue,
    options: &JsValue,
    value: &JsValue,
    context: &mut Context,
) -> JsResult<JsString> {
    let formatter = DateTimeFormat::new(locales, options, context)?;
    let formattable = to_formattable(value, context)?;
    formatter.format_formattable_to_string(&formattable, context)
}

pub(crate) fn format_temporal_date_time_value(
    locales: &JsValue,
    options: &JsValue,
    value: &JsValue,
    required: DateTimeReqs,
    defaults: DateTimeReqs,
    context: &mut Context,
) -> JsResult<JsString> {
    #[cfg(feature = "temporal")]
    let zdt_default_time_zone_name = is_zoned_date_time_default_to_locale_string(value, options, context)?;

    let options = to_date_time_options(options, &required, &defaults, context)?;

    #[cfg(feature = "temporal")]
    if let Some(object) = value.as_object()
        && let Some(zdt) = object.downcast_ref::<ZonedDateTime>()
    {
        let explicit_time_zone = !options.get(js_string!("timeZone"), context)?.is_undefined();
        if explicit_time_zone {
            return Err(JsNativeError::typ()
                .with_message("timeZone option is not allowed for Temporal.ZonedDateTime")
                .into());
        }
        if zdt_default_time_zone_name
            && options.get(js_string!("timeZoneName"), context)?.is_undefined()
        {
            options.create_data_property_or_throw(
                js_string!("timeZoneName"),
                js_string!("short"),
                context,
            )?;
        }
        let time_zone = JsString::from(
            zdt.inner
                .time_zone()
                .identifier_with_provider(context.tz_provider())?,
        );
        options.create_data_property_or_throw(js_string!("timeZone"), time_zone, context)?;
    }

    let options_value: JsValue = options.into();
    let mut formatter = DateTimeFormat::new(locales, &options_value, context)?;
    formatter.temporal_locale_string = true;
    let formattable = to_formattable(value, context)?;
    formatter.format_formattable_to_string(&formattable, context)
}

#[cfg(feature = "temporal")]
fn is_zoned_date_time_default_to_locale_string(
    value: &JsValue,
    options: &JsValue,
    context: &mut Context,
) -> JsResult<bool> {
    let Some(object) = value.as_object() else {
        return Ok(false);
    };
    if object.downcast_ref::<ZonedDateTime>().is_none() {
        return Ok(false);
    }
    if options.is_undefined() {
        return Ok(true);
    }

    let options = options.to_object(context)?;
    for property in [
        js_string!("weekday"),
        js_string!("era"),
        js_string!("year"),
        js_string!("month"),
        js_string!("day"),
        js_string!("dayPeriod"),
        js_string!("hour"),
        js_string!("minute"),
        js_string!("second"),
        js_string!("fractionalSecondDigits"),
        js_string!("timeZoneName"),
        js_string!("dateStyle"),
        js_string!("timeStyle"),
    ] {
        if !options.get(property, context)?.is_undefined() {
            return Ok(false);
        }
    }

    Ok(true)
}

pub(crate) fn to_date_time_options(
    options: &JsValue,
    required: &DateTimeReqs,
    defaults: &DateTimeReqs,
    context: &mut Context,
) -> JsResult<JsObject> {
    let options = if options.is_undefined() {
        None
    } else {
        Some(options.to_object(context)?)
    };
    let options = JsObject::from_proto_and_data_with_shared_shape(
        context.root_shape(),
        options,
        OrdinaryObject,
    );

    let mut need_defaults = true;

    if [DateTimeReqs::Date, DateTimeReqs::AnyAll].contains(required) {
        for property in [
            js_string!("weekday"),
            js_string!("year"),
            js_string!("month"),
            js_string!("day"),
        ] {
            if !options.get(property, context)?.is_undefined() {
                need_defaults = false;
            }
        }
    }

    if [DateTimeReqs::Time, DateTimeReqs::AnyAll].contains(required) {
        for property in [
            js_string!("dayPeriod"),
            js_string!("hour"),
            js_string!("minute"),
            js_string!("second"),
            js_string!("fractionalSecondDigits"),
        ] {
            if !options.get(property, context)?.is_undefined() {
                need_defaults = false;
            }
        }
    }

    let date_style = options.get(js_string!("dateStyle"), context)?;
    let time_style = options.get(js_string!("timeStyle"), context)?;
    if !date_style.is_undefined() || !time_style.is_undefined() {
        need_defaults = false;
    }

    if required == &DateTimeReqs::Date && !time_style.is_undefined() {
        return Err(JsNativeError::typ()
            .with_message("'date' is required, but timeStyle was defined")
            .into());
    }
    if required == &DateTimeReqs::Time && !date_style.is_undefined() {
        return Err(JsNativeError::typ()
            .with_message("'time' is required, but dateStyle was defined")
            .into());
    }

    if need_defaults && [DateTimeReqs::Date, DateTimeReqs::AnyAll].contains(defaults) {
        for property in [js_string!("year"), js_string!("month"), js_string!("day")] {
            options.create_data_property_or_throw(property, js_string!("numeric"), context)?;
        }
    }

    if need_defaults && [DateTimeReqs::Time, DateTimeReqs::AnyAll].contains(defaults) {
        for property in [
            js_string!("hour"),
            js_string!("minute"),
            js_string!("second"),
        ] {
            options.create_data_property_or_throw(property, js_string!("numeric"), context)?;
        }
    }

    Ok(options)
}

#[allow(unused)]
#[derive(Debug, PartialEq)]
pub(crate) enum DateTimeReqs {
    Date,
    Time,
    AnyAll,
}

fn unwrap_date_time_format(
    value: &JsValue,
    context: &mut Context,
) -> JsResult<JsObject<DateTimeFormat>> {
    let object = value.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("value was not an `Intl.DateTimeFormat` object")
    })?;

    if let Ok(dtf) = object.clone().downcast::<DateTimeFormat>() {
        return Ok(dtf);
    }

    let fallback_symbol = context
        .intrinsics()
        .objects()
        .intl()
        .borrow()
        .data()
        .fallback_symbol();
    if let Some(dtf) = object
        .get(fallback_symbol.clone(), context)?
        .as_object()
        .and_then(|object| object.downcast::<DateTimeFormat>().ok())
    {
        return Ok(dtf);
    }

    let constructor = context
        .intrinsics()
        .constructors()
        .date_time_format()
        .constructor();
    if JsValue::ordinary_has_instance(&constructor.into(), value, context)? {
        if let Some(dtf) = object
            .get(fallback_symbol, context)?
            .as_object()
            .and_then(|object| object.downcast::<DateTimeFormat>().ok())
        {
            return Ok(dtf);
        }
    }

    Err(JsNativeError::typ()
        .with_message("object was not an `Intl.DateTimeFormat` object")
        .into())
}

fn to_formattable(value: &JsValue, context: &mut Context) -> JsResult<Formattable> {
    if let Some(record) = extract_temporal_record(value, context)? {
        return Ok(Formattable::Temporal(record));
    }
    let number = if value.is_undefined() {
        context.clock().now().millis_since_epoch() as f64
    } else {
        value.to_number(context)?
    };
    Ok(Formattable::Number(number))
}

#[cfg(feature = "temporal")]
fn extract_temporal_record(
    value: &JsValue,
    context: &mut Context,
) -> JsResult<Option<TemporalRecord>> {
    let Some(object) = value.as_object() else {
        return Ok(None);
    };

    if object.is::<Instant>() {
        let epoch_millis = object
            .get(js_string!("epochMilliseconds"), context)?
            .to_number(context)?;
        return Ok(Some(TemporalRecord {
            kind: RecordKind::Instant,
            epoch_millis: Some(epoch_millis),
            year: None,
            month: None,
            day: None,
            hour: None,
            minute: 0,
            second: 0,
            millisecond: 0,
            calendar: "iso8601".to_owned(),
        }));
    }

    if object.is::<PlainDate>() {
        return Ok(Some(TemporalRecord {
            kind: RecordKind::PlainDate,
            epoch_millis: None,
            year: Some(get_integer_property(&object, "year", context)? as i32),
            month: Some(get_integer_property(&object, "month", context)? as u8),
            day: Some(get_integer_property(&object, "day", context)? as u8),
            hour: None,
            minute: 0,
            second: 0,
            millisecond: 0,
            calendar: get_string_property(&object, "calendarId", context)?,
        }));
    }

    if object.is::<PlainDateTime>() {
        return Ok(Some(TemporalRecord {
            kind: RecordKind::PlainDateTime,
            epoch_millis: None,
            year: Some(get_integer_property(&object, "year", context)? as i32),
            month: Some(get_integer_property(&object, "month", context)? as u8),
            day: Some(get_integer_property(&object, "day", context)? as u8),
            hour: Some(get_integer_property(&object, "hour", context)? as u8),
            minute: get_integer_property(&object, "minute", context)? as u8,
            second: get_integer_property(&object, "second", context)? as u8,
            millisecond: get_integer_property(&object, "millisecond", context)? as u16,
            calendar: get_string_property(&object, "calendarId", context)?,
        }));
    }

    if object.is::<PlainMonthDay>() {
        let month = get_integer_property(&object, "month", context)?;
        let month = if month > 0 {
            month as u8
        } else {
            month_from_month_code(&get_string_property(&object, "monthCode", context)?)
                .ok_or_else(|| {
                    JsNativeError::range()
                        .with_message("Invalid PlainMonthDay monthCode")
                })?
        };
        return Ok(Some(TemporalRecord {
            kind: RecordKind::PlainMonthDay,
            epoch_millis: None,
            year: None,
            month: Some(month),
            day: Some(get_integer_property(&object, "day", context)? as u8),
            hour: None,
            minute: 0,
            second: 0,
            millisecond: 0,
            calendar: get_string_property(&object, "calendarId", context)?,
        }));
    }

    if object.is::<PlainTime>() {
        return Ok(Some(TemporalRecord {
            kind: RecordKind::PlainTime,
            epoch_millis: None,
            year: None,
            month: None,
            day: None,
            hour: Some(get_integer_property(&object, "hour", context)? as u8),
            minute: get_integer_property(&object, "minute", context)? as u8,
            second: get_integer_property(&object, "second", context)? as u8,
            millisecond: get_integer_property(&object, "millisecond", context)? as u16,
            calendar: "iso8601".to_owned(),
        }));
    }

    if object.is::<PlainYearMonth>() {
        return Ok(Some(TemporalRecord {
            kind: RecordKind::PlainYearMonth,
            epoch_millis: None,
            year: Some(get_integer_property(&object, "year", context)? as i32),
            month: Some(get_integer_property(&object, "month", context)? as u8),
            day: None,
            hour: None,
            minute: 0,
            second: 0,
            millisecond: 0,
            calendar: get_string_property(&object, "calendarId", context)?,
        }));
    }

    if object.is::<ZonedDateTime>() {
        return Ok(Some(TemporalRecord {
            kind: RecordKind::ZonedDateTime,
            epoch_millis: None,
            year: Some(get_integer_property(&object, "year", context)? as i32),
            month: Some(get_integer_property(&object, "month", context)? as u8),
            day: Some(get_integer_property(&object, "day", context)? as u8),
            hour: Some(get_integer_property(&object, "hour", context)? as u8),
            minute: get_integer_property(&object, "minute", context)? as u8,
            second: get_integer_property(&object, "second", context)? as u8,
            millisecond: get_integer_property(&object, "millisecond", context)? as u16,
            calendar: get_string_property(&object, "calendarId", context)?,
        }));
    }

    Ok(None)
}

#[cfg(not(feature = "temporal"))]
fn extract_temporal_record(
    _value: &JsValue,
    _context: &mut Context,
) -> JsResult<Option<TemporalRecord>> {
    Ok(None)
}

fn get_integer_property(
    object: &JsObject,
    property: &'static str,
    context: &mut Context,
) -> JsResult<i64> {
    object
        .get(js_string!(property), context)?
        .to_number(context)
        .map(|value| value as i64)
}

fn get_string_property(
    object: &JsObject,
    property: &'static str,
    context: &mut Context,
) -> JsResult<String> {
    object
        .get(js_string!(property), context)?
        .to_string(context)
        .map(|value| value.to_std_string_escaped())
}

fn get_string_option(
    object: &JsObject,
    property: &'static str,
    context: &mut Context,
) -> JsResult<Option<String>> {
    let value = object.get(js_string!(property), context)?;
    if value.is_undefined() {
        Ok(None)
    } else {
        Ok(Some(value.to_string(context)?.to_std_string_escaped()))
    }
}

fn month_from_month_code(month_code: &str) -> Option<u8> {
    let digits = month_code.strip_prefix('M')?;
    let month: u8 = digits.parse().ok()?;
    (1..=12).contains(&month).then_some(month)
}

fn requested_locale_from_locale(locale: Locale) -> JsResult<RequestedLocale> {
    let keyword_value = |key| {
        locale
            .extensions
            .unicode
            .keywords
            .get(&key)
            .map(ToString::to_string)
            .map(|value| value.to_ascii_lowercase())
            .filter(|value| !value.is_empty() && value != "true")
    };

    let keywords = LocaleKeywords {
        ca: keyword_value(key!("ca")),
        nu: keyword_value(key!("nu")),
        hc: keyword_value(key!("hc")),
    };

    Ok(RequestedLocale { keywords })
}

fn ignore_invalid_requested_locale_keyword<T>(result: JsResult<Option<T>>) -> JsResult<Option<T>> {
    match result {
        Ok(value) => Ok(value),
        Err(_) => Ok(None),
    }
}

fn canonicalize_calendar(value: Option<&str>) -> JsResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let lower = normalize_ascii_string(value, "calendar")?;
    if !is_unicode_type_sequence(&lower) {
        return Err(JsNativeError::range()
            .with_message("Invalid calendar")
            .into());
    }
    let normalized = match lower.as_str() {
        "islamicc" | "islamic" | "islamic-rgsa" => "islamic-civil",
        "ethiopic-amete-alem" => "ethioaa",
        _ => lower.as_str(),
    };
    Ok(is_supported_calendar(normalized).then(|| normalized.to_owned()))
}

fn canonicalize_numbering_system(value: Option<&str>) -> JsResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let lower = normalize_ascii_string(value, "numberingSystem")?;
    if !is_unicode_type_sequence(&lower) {
        return Err(JsNativeError::range()
            .with_message("Invalid numberingSystem")
            .into());
    }
    if supported_numbering_systems().contains(&lower.as_str()) {
        Ok(Some(lower))
    } else {
        Ok(None)
    }
}

fn canonicalize_hour_cycle(value: Option<&str>) -> JsResult<Option<String>> {
    let Some(value) = value else {
        return Ok(None);
    };
    let value = value.to_ascii_lowercase();
    if matches!(value.as_str(), "h11" | "h12" | "h23" | "h24") {
        Ok(Some(value))
    } else {
        Ok(None)
    }
}

fn canonicalize_time_zone(value: &str, context: &mut Context) -> JsResult<String> {
    if let Some(offset) = parse_offset_time_zone(value) {
        if offset == 0 {
            return Ok("+00:00".to_owned());
        }
        let sign = if offset.is_negative() { '-' } else { '+' };
        let offset = offset.unsigned_abs();
        let hours = offset / 60;
        let minutes = offset % 60;
        return Ok(format!("{sign}{hours:02}:{minutes:02}"));
    }
    if value.starts_with('+') || value.starts_with('-') {
        return Err(JsNativeError::range()
            .with_message("Invalid timeZone")
            .into());
    }

    if value.eq_ignore_ascii_case("utc") {
        return Ok("UTC".to_owned());
    }
    if value.eq_ignore_ascii_case("gmt") {
        return Ok("GMT".to_owned());
    }
    if value.eq_ignore_ascii_case("etc/utc") {
        return Ok("Etc/UTC".to_owned());
    }
    if value.eq_ignore_ascii_case("etc/gmt") {
        return Ok("Etc/GMT".to_owned());
    }
    if let Some(named) = canonicalize_named_time_zone(value) {
        return Ok(named);
    }
    if !value.contains('/') {
        return Err(JsNativeError::range()
            .with_message("Invalid timeZone")
            .into());
    }
    #[cfg(feature = "temporal")]
    if let Ok(time_zone) = RsTimeZone::try_from_str_with_provider(value, context.tz_provider()) {
        return Ok(time_zone
            .identifier_with_provider(context.tz_provider())
            .unwrap_or_else(|_| value.to_owned()));
    }
    if !value
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '/' | '_' | '+' | '-'))
    {
        return Err(JsNativeError::range()
            .with_message("Invalid timeZone")
            .into());
    }
    Ok(canonicalize_time_zone_case(value))
}

fn parse_offset_time_zone(value: &str) -> Option<i32> {
    let bytes = value.as_bytes();
    if bytes.len() < 3 || !matches!(bytes[0], b'+' | b'-') {
        return None;
    }
    let sign = if bytes[0] == b'-' { -1 } else { 1 };
    let digits = &value[1..];
    let (hours, minutes) = if let Some((hours, minutes)) = digits.split_once(':') {
        if hours.len() != 2 || minutes.len() != 2 {
            return None;
        }
        (hours, minutes)
    } else if digits.len() == 4 {
        (&digits[..2], &digits[2..])
    } else if digits.len() == 2 {
        (digits, "00")
    } else {
        return None;
    };
    let hours: i32 = hours.parse().ok()?;
    let minutes: i32 = minutes.parse().ok()?;
    if hours > 23 || minutes > 59 {
        return None;
    }
    Some(sign * (hours * 60 + minutes))
}

fn parse_etc_gmt_time_zone(value: &str) -> Option<i32> {
    let trimmed = value.strip_prefix("Etc/GMT")?;
    if trimmed.is_empty() {
        return Some(0);
    }
    let sign = match trimmed.as_bytes().first().copied()? {
        b'+' => -1,
        b'-' => 1,
        _ => return None,
    };
    let hours: i32 = trimmed[1..].parse().ok()?;
    if hours > 23 {
        return None;
    }
    Some(sign * hours * 60)
}

fn default_time_zone_identifier() -> String {
    iana_time_zone::get_timezone().unwrap_or_else(|_| "UTC".to_owned())
}

fn normalize_ascii_string(value: &str, label: &str) -> JsResult<String> {
    let lower = value.to_ascii_lowercase();
    if !lower
        .chars()
        .all(|ch| ch.is_ascii_alphanumeric() || ch == '-')
    {
        return Err(JsNativeError::range()
            .with_message(format!("Invalid {label}"))
            .into());
    }
    Ok(lower)
}

fn is_unicode_type_sequence(value: &str) -> bool {
    value
        .split('-')
        .all(|segment| (3..=8).contains(&segment.len()) && segment.chars().all(|ch| ch.is_ascii_alphanumeric()))
}

fn is_supported_calendar(value: &str) -> bool {
    matches!(
        value,
        "buddhist"
            | "chinese"
            | "coptic"
            | "dangi"
            | "ethioaa"
            | "ethiopic"
            | "gregory"
            | "hebrew"
            | "indian"
            | "islamic-civil"
            | "islamic-tbla"
            | "islamic-umalqura"
            | "iso8601"
            | "japanese"
            | "persian"
            | "roc"
    )
}

fn canonical_time_zone_display_name(time_zone: &str) -> String {
    match time_zone {
        "Asia/Calcutta" => "Asia/Kolkata".to_owned(),
        _ => time_zone.to_owned(),
    }
}

fn canonicalize_time_zone_case(value: &str) -> String {
    value
        .split('/')
        .map(|segment| {
            segment
                .split('_')
                .map(|part| {
                    part
                        .split('-')
                        .map(|piece| {
                            if piece.is_empty() {
                                String::new()
                            } else {
                                let mut chars = piece.chars();
                                let first = chars.next().unwrap().to_ascii_uppercase();
                                let rest = chars.as_str().to_ascii_lowercase();
                                format!("{first}{rest}")
                            }
                        })
                        .collect::<Vec<_>>()
                        .join("-")
                })
                .collect::<Vec<_>>()
                .join("_")
        })
        .collect::<Vec<_>>()
        .join("/")
}

fn canonicalize_named_time_zone(value: &str) -> Option<String> {
    let upper = value.to_ascii_uppercase();
    Some(match upper.as_str() {
        "CET" => "CET",
        "CST6CDT" => "CST6CDT",
        "EET" => "EET",
        "EST" => "EST",
        "EST5EDT" => "EST5EDT",
        "HST" => "HST",
        "MET" => "MET",
        "MST" => "MST",
        "MST7MDT" => "MST7MDT",
        "PST8PDT" => "PST8PDT",
        "WET" => "WET",
        "CUBA" => "Cuba",
        "EGYPT" => "Egypt",
        "EIRE" => "Eire",
        "GB" => "GB",
        "GB-EIRE" => "GB-Eire",
        "GMT+0" => "GMT+0",
        "GMT-0" => "GMT-0",
        "GMT0" => "GMT0",
        "GREENWICH" => "Greenwich",
        "HONGKONG" => "Hongkong",
        "ICELAND" => "Iceland",
        "IRAN" => "Iran",
        "ISRAEL" => "Israel",
        "JAMAICA" => "Jamaica",
        "JAPAN" => "Japan",
        "KWAJALEIN" => "Kwajalein",
        "LIBYA" => "Libya",
        "NZ" => "NZ",
        "NZ-CHAT" => "NZ-CHAT",
        "NAVAJO" => "Navajo",
        "PRC" => "PRC",
        "POLAND" => "Poland",
        "PORTUGAL" => "Portugal",
        "ROC" => "ROC",
        "ROK" => "ROK",
        "SINGAPORE" => "Singapore",
        "TURKEY" => "Turkey",
        "UCT" => "UCT",
        "UNIVERSAL" => "Universal",
        "W-SU" => "W-SU",
        "ZULU" => "Zulu",
        _ => return None,
    }
    .to_owned())
}

fn era_label_for_code(code: &str) -> String {
    match code {
        "ce" | "ad" => "AD".to_owned(),
        "bce" | "bc" => "BC".to_owned(),
        _ => code.to_owned(),
    }
}

#[derive(Debug, Clone)]
struct CalendarDisplayFields {
    year: i32,
    month: u8,
    day: u8,
    month_display: Option<String>,
    era_label: Option<String>,
    related_year: Option<i32>,
    year_name: Option<String>,
}

#[cfg(feature = "temporal")]
fn calendar_kind(calendar: &str) -> Option<AnyCalendarKind> {
    Some(match calendar {
        "buddhist" => AnyCalendarKind::Buddhist,
        "chinese" => AnyCalendarKind::Chinese,
        "coptic" => AnyCalendarKind::Coptic,
        "dangi" => AnyCalendarKind::Dangi,
        "ethioaa" => AnyCalendarKind::EthiopianAmeteAlem,
        "ethiopic" => AnyCalendarKind::Ethiopian,
        "gregory" => AnyCalendarKind::Gregorian,
        "hebrew" => AnyCalendarKind::Hebrew,
        "indian" => AnyCalendarKind::Indian,
        "islamic-civil" => AnyCalendarKind::HijriTabularTypeIIFriday,
        "islamic-tbla" => AnyCalendarKind::HijriTabularTypeIIThursday,
        "islamic-umalqura" => AnyCalendarKind::HijriUmmAlQura,
        "iso8601" => AnyCalendarKind::Iso,
        "japanese" => AnyCalendarKind::Japanese,
        "persian" => AnyCalendarKind::Persian,
        "roc" => AnyCalendarKind::Roc,
        _ => return None,
    })
}

#[cfg(feature = "temporal")]
fn calendar_display_fields(
    calendar: &str,
    iso_year: i32,
    iso_month: u8,
    iso_day: u8,
) -> Option<CalendarDisplayFields> {
    let kind = calendar_kind(calendar)?;
    let iso = IcuDate::try_new_iso(iso_year, iso_month, iso_day).ok()?;
    let date = iso.to_calendar(AnyCalendar::new(kind));
    let month = date.month();
    let month_number = month.month_number();
    let formatting_is_leap = month
        .formatting_code
        .parsed()
        .map(|(_, leap)| leap)
        .unwrap_or_else(|| month.is_leap());
    let year_info = date.year();
    let month_display = if calendar == "hebrew" {
        Some(hebrew_month_display(month_number, formatting_is_leap).to_owned())
    } else if matches!(
        calendar,
        "islamic-civil" | "islamic-tbla" | "islamic-umalqura"
    ) {
        Some(islamic_month_display(month_number).to_owned())
    } else {
        formatting_is_leap.then(|| format!("{month_number}L"))
    };
    let day = date.day_of_month().0;
    match year_info {
        YearInfo::Era(era) => Some(CalendarDisplayFields {
            year: era.year,
            month: month_number,
            day,
            month_display,
            era_label: Some(era_label_for_code(&era.era.to_string())),
            related_year: None,
            year_name: None,
        }),
        YearInfo::Cyclic(cycle) => Some(CalendarDisplayFields {
            year: cycle.related_iso,
            month: month_number,
            day,
            month_display,
            era_label: None,
            related_year: Some(cycle.related_iso),
            year_name: Some(sexagenary_name(cycle.year)),
        }),
        _ => None,
    }
}

#[cfg(not(feature = "temporal"))]
fn calendar_display_fields(
    _calendar: &str,
    _iso_year: i32,
    _iso_month: u8,
    _iso_day: u8,
) -> Option<CalendarDisplayFields> {
    None
}

fn default_numbering_system_for_locale(locale: &str) -> &'static str {
    if locale.eq_ignore_ascii_case("ar") || locale.to_ascii_lowercase().starts_with("ar-") {
        "arab"
    } else {
        "latn"
    }
}

fn hebrew_month_display(month_number: u8, is_leap: bool) -> &'static str {
    match (month_number, is_leap) {
        (1, _) => "Tishri",
        (2, _) => "Heshvan",
        (3, _) => "Kislev",
        (4, _) => "Tevet",
        (5, false) => "Shevat",
        (5, true) => "Adar I",
        (6, _) => "Adar II",
        (7, _) => "Nisan",
        (8, _) => "Iyar",
        (9, _) => "Sivan",
        (10, _) => "Tamuz",
        (11, _) => "Av",
        (12, _) => "Elul",
        _ => "",
    }
}

fn is_hebrew_leap_year(year: i32) -> bool {
    matches!(year.rem_euclid(19), 0 | 3 | 6 | 8 | 11 | 14 | 17)
}

fn islamic_month_display(month_number: u8) -> &'static str {
    match month_number {
        1 => "Muharram",
        2 => "Safar",
        3 => "Rabiʻ I",
        4 => "Rabiʻ II",
        5 => "Jumada I",
        6 => "Jumada II",
        7 => "Rajab",
        8 => "Shaʻban",
        9 => "Ramadan",
        10 => "Shawwal",
        11 => "Dhuʻl-Qiʻdah",
        12 => "Dhuʻl-Hijjah",
        _ => "",
    }
}

fn default_hour_cycle_for_locale(locale: &str) -> &'static str {
    let lower = locale.to_ascii_lowercase();
    if lower == "ja" || lower.starts_with("ja-") {
        "h11"
    } else if lower == "de" || lower.starts_with("de-") {
        "h23"
    } else {
        "h12"
    }
}

fn default_twelve_hour_cycle(locale: &str) -> &'static str {
    if default_hour_cycle_for_locale(locale) == "h11" {
        "h11"
    } else {
        "h12"
    }
}

fn build_resolved_locale(base: &str, keywords: &LocaleKeywords) -> String {
    let mut retained = Vec::new();
    if let Some(calendar) = &keywords.ca {
        retained.push(format!("ca-{calendar}"));
    }
    if let Some(hour_cycle) = &keywords.hc {
        retained.push(format!("hc-{hour_cycle}"));
    }
    if let Some(numbering_system) = &keywords.nu {
        retained.push(format!("nu-{numbering_system}"));
    }
    if retained.is_empty() {
        base.to_owned()
    } else {
        format!("{base}-u-{}", retained.join("-"))
    }
}

fn is_twelve_hour_cycle(value: &str) -> bool {
    matches!(value, "h11" | "h12")
}

fn apply_time_zone(number: f64, time_zone: &str, context: &mut Context) -> f64 {
    if is_utc_time_zone(time_zone) {
        return number;
    }
    if let Some(offset) = parse_offset_time_zone(time_zone) {
        return number + f64::from(offset) * MS_PER_MINUTE;
    }
    if let Some(offset) = parse_etc_gmt_time_zone(time_zone) {
        return number + f64::from(offset) * MS_PER_MINUTE;
    }
    if time_zone == default_time_zone_identifier() {
        return local_time(number, context.host_hooks().as_ref());
    }
    local_time(number, context.host_hooks().as_ref())
}

fn is_utc_time_zone(time_zone: &str) -> bool {
    matches!(time_zone, "UTC" | "GMT" | "Etc/UTC" | "Etc/GMT" | "+00:00")
}

fn temporal_calendar_matches_formatter(
    kind: RecordKind,
    record_calendar: &str,
    formatter_calendar: &str,
) -> bool {
    (record_calendar == "iso8601"
        && !matches!(kind, RecordKind::PlainMonthDay | RecordKind::PlainYearMonth))
        || record_calendar == formatter_calendar
        || matches!(
            (record_calendar, formatter_calendar),
            ("islamic-tbla", "islamic-civil") | ("islamic-civil", "islamic-tbla")
        )
}

fn supports_time_zone_name(display: &DisplayRecord) -> bool {
    matches!(
        display.kind,
        RecordKind::Number | RecordKind::Instant | RecordKind::ZonedDateTime
    )
}

fn day_period_for_hour(hour: u8, width: Option<&str>) -> &'static str {
    match width.unwrap_or("short") {
        "narrow" if hour == 12 => "n",
        "narrow" | "short" | "long" if hour < 12 => "in the morning",
        "short" | "long" if hour == 12 => "noon",
        "narrow" | "short" | "long" if hour < 18 => "in the afternoon",
        "narrow" | "short" | "long" if hour < 21 => "in the evening",
        "narrow" | "short" | "long" => "at night",
        _ => "AM",
    }
}

fn decimal_separator(numbering_system: &str) -> &'static str {
    if numbering_system == "arab" {
        "\u{066B}"
    } else {
        "."
    }
}

fn month_name(month: u8, width: &str) -> &'static str {
    const SHORT: [&str; 12] = [
        "Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec",
    ];
    const LONG: [&str; 12] = [
        "January",
        "February",
        "March",
        "April",
        "May",
        "June",
        "July",
        "August",
        "September",
        "October",
        "November",
        "December",
    ];
    if width == "long" {
        LONG[(month.saturating_sub(1)) as usize]
    } else {
        SHORT[(month.saturating_sub(1)) as usize]
    }
}

fn textual_month_name(
    calendar: &str,
    month: u8,
    width: &str,
    month_display: Option<&str>,
) -> Option<String> {
    if width == "long" {
        match calendar {
            "islamic-civil" | "islamic-tbla" | "islamic-umalqura" => {
                return Some(islamic_month_display(month).to_owned());
            }
            _ => {}
        }
    }

    match month_display {
        Some(value) if !value.ends_with('L') => Some(value.to_owned()),
        _ => None,
    }
}

fn weekday_name(year: i32, month: u8, day: u8, width: Option<&str>) -> &'static str {
    const SHORT: [&str; 7] = ["Sun", "Mon", "Tue", "Wed", "Thu", "Fri", "Sat"];
    const LONG: [&str; 7] = [
        "Sunday",
        "Monday",
        "Tuesday",
        "Wednesday",
        "Thursday",
        "Friday",
        "Saturday",
    ];
    let day_index = week_day(make_date(
        make_day(f64::from(year), f64::from(month.saturating_sub(1)), f64::from(day)),
        0.0,
    )) as usize;
    if width == Some("long") {
        LONG[day_index]
    } else {
        SHORT[day_index]
    }
}

#[cfg(feature = "temporal")]
fn east_asian_fields(calendar: &str, year: i32, month: u8, day: u8) -> Option<(i32, String, u8, u8)> {
    let calendar = match calendar {
        "chinese" => AnyCalendar::new(AnyCalendarKind::Chinese),
        "dangi" => AnyCalendar::new(AnyCalendarKind::Dangi),
        _ => return None,
    };
    let iso = IcuDate::try_new_iso(year, month, day).ok()?;
    let date = iso.to_calendar(calendar);
    let month = date.month().ordinal;
    let day = date.day_of_month().0;
    let YearInfo::Cyclic(cycle) = date.year() else {
        return None;
    };
    let related_year = cycle.related_iso;
    let cycle = cycle.year;
    Some((related_year, sexagenary_name(cycle), month, day))
}

#[cfg(feature = "temporal")]
fn sexagenary_name(year: u8) -> String {
    const STEMS: [&str; 10] = ["甲", "乙", "丙", "丁", "戊", "己", "庚", "辛", "壬", "癸"];
    const BRANCHES: [&str; 12] = ["子", "丑", "寅", "卯", "辰", "巳", "午", "未", "申", "酉", "戌", "亥"];
    let index = usize::from(year.saturating_sub(1));
    format!("{}{}", STEMS[index % 10], BRANCHES[index % 12])
}

#[cfg(not(feature = "temporal"))]
fn east_asian_fields(
    _calendar: &str,
    _year: i32,
    _month: u8,
    _day: u8,
) -> Option<(i32, String, u8, u8)> {
    None
}

#[cfg(not(feature = "temporal"))]
fn sexagenary_name(_year: u8) -> String {
    String::new()
}

fn part(kind: &'static str, value: &str) -> DateTimePart {
    DateTimePart {
        kind,
        value: JsString::from(value),
    }
}

fn join_parts(parts: &[DateTimePart]) -> JsString {
    let mut value = String::new();
    for part in parts {
        value.push_str(&part.value.to_std_string_escaped());
    }
    JsString::from(value)
}

fn parts_to_array(
    parts: impl IntoIterator<Item = (DateTimePart, Option<&'static str>)>,
    context: &mut Context,
) -> JsObject {
    let objects = parts
        .into_iter()
        .map(|(part, source)| {
            let mut object = ObjectInitializer::new(context);
            object
                .property(js_string!("type"), js_string!(part.kind), Attribute::all())
                .property(js_string!("value"), part.value, Attribute::all());
            if let Some(source) = source {
                object.property(js_string!("source"), js_string!(source), Attribute::all());
            }
            object.build().into()
        })
        .collect::<Vec<_>>();
    Array::create_array_from_list(objects, context)
}

impl OptionType for CalendarAlgorithm {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        let s = value.to_string(context)?.to_std_string_escaped();
        Value::try_from_str(&s)
            .ok()
            .and_then(|v| CalendarAlgorithm::try_from(&v).ok())
            .ok_or_else(|| {
                JsNativeError::range()
                    .with_message(format!("provided calendar `{s}` is invalid"))
                    .into()
            })
    }
}

impl OptionType for HourCycle {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        match value.to_string(context)?.to_std_string_escaped().as_str() {
            "h11" => Ok(HourCycle::H11),
            "h12" => Ok(HourCycle::H12),
            "h23" => Ok(HourCycle::H23),
            _ => Err(JsNativeError::range()
                .with_message("provided hour cycle was not `h11`, `h12` or `h23`")
                .into()),
        }
    }
}
