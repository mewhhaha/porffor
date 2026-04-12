use std::{fmt, str::FromStr};

use boa_gc::{Finalize, Trace};
use icu_decimal::{preferences::NumberingSystem, provider::DecimalSymbolsV1};
use icu_locale::{
    Locale,
    extensions::unicode::{Value, key},
};
use icu_provider::DataMarkerAttributes;

use crate::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::{
        BuiltInConstructor, BuiltInObject, IntrinsicObject, OrdinaryObject,
        array::Array,
        builder::BuiltInBuilder,
        intl::{
            Service,
            locale::{canonicalize_locale_list, filter_locales, resolve_locale, validate_extension},
            number_format::{NumberFormat, NumberPart, numbering_system_is_supported},
            options::{IntlOptions, LocaleMatcher, coerce_options_to_object},
        },
        options::{OptionType, ParsableOptionType, get_option},
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
};

#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct RelativeTimeFormat {
    locale: Locale,
    numbering_system: Option<String>,
    style: RelativeTimeStyle,
    numeric: RelativeTimeNumeric,
}

impl Service for RelativeTimeFormat {
    type LangMarker = DecimalSymbolsV1;
    type LocaleOptions = RelativeTimeFormatLocaleOptions;

    fn resolve(
        locale: &mut Locale,
        options: &mut Self::LocaleOptions,
        provider: &crate::context::icu::IntlProvider,
    ) {
        let extension_numbering_system = locale
            .extensions
            .unicode
            .keywords
            .get(&key!("nu"))
            .map(ToString::to_string);
        let option_numbering_system = options.numbering_system.take().filter(|nu| {
            numbering_system_is_supported(nu)
                || Value::try_from_str(nu).is_ok_and(|nu| {
                    let Ok(nu) = NumberingSystem::try_from(nu) else {
                        return false;
                    };
                    let attr = DataMarkerAttributes::from_str_or_panic(nu.as_str());
                    validate_extension::<Self::LangMarker>(locale.id.clone(), attr, provider)
                })
        });
        let extension_numbering_system = extension_numbering_system.filter(|nu| {
            numbering_system_is_supported(nu)
                || Value::try_from_str(nu).is_ok_and(|nu| {
                    let Ok(nu) = NumberingSystem::try_from(nu) else {
                        return false;
                    };
                    let attr = DataMarkerAttributes::from_str_or_panic(nu.as_str());
                    validate_extension::<Self::LangMarker>(locale.id.clone(), attr, provider)
                })
        });

        let numbering_system = option_numbering_system
            .clone()
            .or_else(|| extension_numbering_system.clone());
        let reflect_extension = match (
            &numbering_system,
            &option_numbering_system,
            &extension_numbering_system,
        ) {
            (Some(numbering_system), Some(option), Some(extension)) => {
                numbering_system == option && option == extension
            }
            (Some(numbering_system), None, Some(extension)) => numbering_system == extension,
            _ => false,
        };
        locale.extensions.unicode.keywords.clear();
        if reflect_extension
            && let Some(nu) = numbering_system.clone()
            && let Ok(nu) = Value::try_from_str(&nu)
        {
            locale.extensions.unicode.keywords.set(key!("nu"), nu);
        }
        options.numbering_system = numbering_system;
    }
}

#[derive(Debug, Default)]
pub(crate) struct RelativeTimeFormatLocaleOptions {
    numbering_system: Option<String>,
}

impl IntrinsicObject for RelativeTimeFormat {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                crate::symbol::JsSymbol::to_string_tag(),
                js_string!("Intl.RelativeTimeFormat"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::format, js_string!("format"), 2)
            .method(Self::format_to_parts, js_string!("formatToParts"), 2)
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for RelativeTimeFormat {
    const NAME: JsString = js_string!("RelativeTimeFormat");
}

impl BuiltInConstructor for RelativeTimeFormat {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::relative_time_format;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.RelativeTimeFormat` constructor without `new`")
                .into());
        }

        let rtf = Self::new(args.get_or_undefined(0), args.get_or_undefined(1), context)?;
        let prototype = get_prototype_from_constructor(
            new_target,
            StandardConstructors::relative_time_format,
            context,
        )?;
        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            rtf,
        )
        .into())
    }
}

impl RelativeTimeFormat {
    fn new(locales: &JsValue, options: &JsValue, context: &mut Context) -> JsResult<Self> {
        let requested_locales = canonicalize_locale_list(locales, context)?;
        let options = coerce_options_to_object(options, context)?;

        let matcher =
            get_option::<LocaleMatcher>(&options, js_string!("localeMatcher"), context)?
                .unwrap_or_default();
        let numbering_system =
            get_option::<UnicodeTypeSequence>(&options, js_string!("numberingSystem"), context)?;
        let requested_numbering_system = numbering_system.clone();
        let mut intl_options = IntlOptions {
            matcher,
            service_options: RelativeTimeFormatLocaleOptions {
                numbering_system: numbering_system.map(|nu| nu.0),
            },
        };
        let locale = resolve_locale::<Self>(
            requested_locales,
            &mut intl_options,
            context.intl_provider(),
        )?;

        let style =
            get_option::<RelativeTimeStyle>(&options, js_string!("style"), context)?.unwrap_or_default();
        let numeric =
            get_option::<RelativeTimeNumeric>(&options, js_string!("numeric"), context)?
                .unwrap_or_default();

        Ok(Self {
            locale,
            numbering_system: intl_options
                .service_options
                .numbering_system
                .or_else(|| {
                    requested_numbering_system
                        .map(|nu| nu.0)
                        .filter(|nu| numbering_system_is_supported(nu))
                }),
            style,
            numeric,
        })
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

    fn format(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let rtf = object
            .as_ref()
            .and_then(|object| object.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value must be an Intl.RelativeTimeFormat object")
            })?;
        let parts = rtf.partition(args, context)?;
        Ok(join_relative_parts(&parts).into())
    }

    fn format_to_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object();
        let rtf = object
            .as_ref()
            .and_then(|object| object.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value must be an Intl.RelativeTimeFormat object")
            })?;
        let parts = rtf.partition(args, context)?;
        Ok(relative_parts_to_array(parts, context).into())
    }

    fn resolved_options(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object();
        let rtf = object
            .as_ref()
            .and_then(|object| object.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value must be an Intl.RelativeTimeFormat object")
            })?;
        let options = context
            .intrinsics()
            .templates()
            .ordinary_object()
            .create(OrdinaryObject, vec![]);
        options.create_data_property_or_throw(
            js_string!("locale"),
            js_string!(rtf.locale.to_string()),
            context,
        )?;
        options.create_data_property_or_throw(
            js_string!("style"),
            rtf.style.to_js_string(),
            context,
        )?;
        options.create_data_property_or_throw(
            js_string!("numeric"),
            rtf.numeric.to_js_string(),
            context,
        )?;
        options.create_data_property_or_throw(
            js_string!("numberingSystem"),
            js_string!(
                rtf.numbering_system
                    .as_ref()
                    .map_or("latn".to_owned(), Clone::clone)
            ),
            context,
        )?;
        Ok(options.into())
    }

    fn partition(&self, args: &[JsValue], context: &mut Context) -> JsResult<Vec<RelativePart>> {
        let value = args.get_or_undefined(0).to_number(context)?;
        if !value.is_finite() {
            return Err(JsNativeError::range()
                .with_message("relative time value must be finite")
                .into());
        }
        let unit = singular_relative_time_unit(args.get_or_undefined(1), context)?;
        let negative_zero = value == 0.0 && value.is_sign_negative();

        if self.numeric == RelativeTimeNumeric::Auto
            && let Some(literal) = auto_literal(value, negative_zero, unit)
        {
            return Ok(vec![RelativePart::literal(JsString::from(literal))]);
        }

        let abs = value.abs();
        let number_parts = self.format_number_parts(abs, context)?;
        let unit_name = unit_name(locale_language(&self.locale), self.style, unit, value);
        let mut parts = Vec::new();
        if value.is_sign_negative() {
            parts.extend(number_parts.into_iter().map(|part| RelativePart {
                kind: part.kind,
                value: part.value,
                unit: Some(unit.name()),
            }));
            parts.push(RelativePart::literal(JsString::from(format!(
                " {unit_name} {}",
                past_suffix(locale_language(&self.locale))
            ))));
        } else {
            parts.push(RelativePart::literal(JsString::from(future_prefix(
                locale_language(&self.locale),
            ))));
            parts.extend(number_parts.into_iter().map(|part| RelativePart {
                kind: part.kind,
                value: part.value,
                unit: Some(unit.name()),
            }));
            parts.push(RelativePart::literal(JsString::from(format!(
                " {unit_name}"
            ))));
        }
        Ok(parts)
    }

    fn format_number_parts(
        &self,
        value: f64,
        context: &mut Context,
    ) -> JsResult<Vec<NumberPart>> {
        let options = JsObject::with_null_proto();
        if let Some(numbering_system) = &self.numbering_system {
            options.create_data_property_or_throw(
                js_string!("numberingSystem"),
                js_string!(numbering_system.clone()),
                context,
            )?;
        }
        let nf = NumberFormat::new(
            &js_string!(self.locale.to_string()).into(),
            &options.into(),
            context,
        )?;
        nf.format_value_to_parts(&JsValue::new(value), context)
    }
}

fn singular_relative_time_unit(value: &JsValue, context: &mut Context) -> JsResult<RelativeUnit> {
    let unit = value.to_string(context)?.to_std_string_escaped();
    match unit.as_str() {
        "second" | "seconds" => Ok(RelativeUnit::Second),
        "minute" | "minutes" => Ok(RelativeUnit::Minute),
        "hour" | "hours" => Ok(RelativeUnit::Hour),
        "day" | "days" => Ok(RelativeUnit::Day),
        "week" | "weeks" => Ok(RelativeUnit::Week),
        "month" | "months" => Ok(RelativeUnit::Month),
        "quarter" | "quarters" => Ok(RelativeUnit::Quarter),
        "year" | "years" => Ok(RelativeUnit::Year),
        _ => Err(JsNativeError::range()
            .with_message("invalid relative time unit")
            .into()),
    }
}

fn join_relative_parts(parts: &[RelativePart]) -> JsString {
    let mut out = String::new();
    for part in parts {
        out.push_str(&part.value.to_std_string_escaped());
    }
    JsString::from(out)
}

fn relative_parts_to_array(parts: Vec<RelativePart>, context: &mut Context) -> JsObject {
    let result = Array::array_create(0, None, context)
        .expect("creating an empty array with default proto must not fail");
    for (n, part) in parts.into_iter().enumerate() {
        let obj = context
            .intrinsics()
            .templates()
            .ordinary_object()
            .create(OrdinaryObject, vec![]);
        obj.create_data_property_or_throw(js_string!("type"), js_string!(part.kind), context)
            .expect("creating data property must not fail");
        obj.create_data_property_or_throw(js_string!("value"), part.value, context)
            .expect("creating data property must not fail");
        if let Some(unit) = part.unit {
            obj.create_data_property_or_throw(js_string!("unit"), js_string!(unit), context)
                .expect("creating data property must not fail");
        }
        result
            .create_data_property_or_throw(n, obj, context)
            .expect("creating array property must not fail");
    }
    result
}

#[derive(Clone)]
struct UnicodeTypeSequence(String);

impl OptionType for UnicodeTypeSequence {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        let value = value.to_string(context)?.to_std_string_escaped();
        if is_unicode_type_sequence(&value) {
            Ok(Self(value))
        } else {
            Err(JsNativeError::range()
                .with_message(format!("provided numbering system `{value}` is invalid"))
                .into())
        }
    }
}

fn is_unicode_type_sequence(value: &str) -> bool {
    let mut saw_subtag = false;
    for subtag in value.split('-') {
        saw_subtag = true;
        if !(3..=8).contains(&subtag.len())
            || !subtag.bytes().all(|byte| byte.is_ascii_alphanumeric())
        {
            return false;
        }
    }
    saw_subtag
}

#[derive(Debug, Clone)]
struct RelativePart {
    kind: &'static str,
    value: JsString,
    unit: Option<&'static str>,
}

impl RelativePart {
    fn literal(value: JsString) -> Self {
        Self {
            kind: "literal",
            value,
            unit: None,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum RelativeTimeStyle {
    #[default]
    Long,
    Short,
    Narrow,
}

impl RelativeTimeStyle {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Long => js_string!("long"),
            Self::Short => js_string!("short"),
            Self::Narrow => js_string!("narrow"),
        }
    }
}

impl FromStr for RelativeTimeStyle {
    type Err = ParseRelativeTimeStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            "narrow" => Ok(Self::Narrow),
            _ => Err(ParseRelativeTimeStyleError),
        }
    }
}

impl ParsableOptionType for RelativeTimeStyle {}

#[derive(Debug)]
struct ParseRelativeTimeStyleError;

impl fmt::Display for ParseRelativeTimeStyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid relative time style")
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum RelativeTimeNumeric {
    #[default]
    Always,
    Auto,
}

impl RelativeTimeNumeric {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Always => js_string!("always"),
            Self::Auto => js_string!("auto"),
        }
    }
}

impl FromStr for RelativeTimeNumeric {
    type Err = ParseRelativeTimeNumericError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "always" => Ok(Self::Always),
            "auto" => Ok(Self::Auto),
            _ => Err(ParseRelativeTimeNumericError),
        }
    }
}

impl ParsableOptionType for RelativeTimeNumeric {}

#[derive(Debug)]
struct ParseRelativeTimeNumericError;

impl fmt::Display for ParseRelativeTimeNumericError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid relative time numeric option")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum RelativeUnit {
    Second,
    Minute,
    Hour,
    Day,
    Week,
    Month,
    Quarter,
    Year,
}

impl RelativeUnit {
    fn name(self) -> &'static str {
        match self {
            Self::Second => "second",
            Self::Minute => "minute",
            Self::Hour => "hour",
            Self::Day => "day",
            Self::Week => "week",
            Self::Month => "month",
            Self::Quarter => "quarter",
            Self::Year => "year",
        }
    }
}

fn locale_language(locale: &Locale) -> &str {
    locale.id.language.as_str()
}

fn future_prefix(language: &str) -> &'static str {
    if language == "pl" { "za " } else { "in " }
}

fn past_suffix(language: &str) -> &'static str {
    if language == "pl" { "temu" } else { "ago" }
}

fn auto_literal(value: f64, negative_zero: bool, unit: RelativeUnit) -> Option<&'static str> {
    if negative_zero {
        return auto_literal_for_key(0, unit);
    }
    if value == -1.0 {
        return auto_literal_for_key(-1, unit);
    }
    if value == 0.0 {
        return auto_literal_for_key(0, unit);
    }
    if value == 1.0 {
        return auto_literal_for_key(1, unit);
    }
    None
}

fn auto_literal_for_key(key: i8, unit: RelativeUnit) -> Option<&'static str> {
    match (unit, key) {
        (RelativeUnit::Second, 0) => Some("now"),
        (RelativeUnit::Minute, 0) => Some("this minute"),
        (RelativeUnit::Hour, 0) => Some("this hour"),
        (RelativeUnit::Day, -1) => Some("yesterday"),
        (RelativeUnit::Day, 0) => Some("today"),
        (RelativeUnit::Day, 1) => Some("tomorrow"),
        (RelativeUnit::Week, -1) => Some("last week"),
        (RelativeUnit::Week, 0) => Some("this week"),
        (RelativeUnit::Week, 1) => Some("next week"),
        (RelativeUnit::Month, -1) => Some("last month"),
        (RelativeUnit::Month, 0) => Some("this month"),
        (RelativeUnit::Month, 1) => Some("next month"),
        (RelativeUnit::Quarter, -1) => Some("last quarter"),
        (RelativeUnit::Quarter, 0) => Some("this quarter"),
        (RelativeUnit::Quarter, 1) => Some("next quarter"),
        (RelativeUnit::Year, -1) => Some("last year"),
        (RelativeUnit::Year, 0) => Some("this year"),
        (RelativeUnit::Year, 1) => Some("next year"),
        _ => None,
    }
}

fn unit_name(language: &str, style: RelativeTimeStyle, unit: RelativeUnit, value: f64) -> &'static str {
    if language == "pl" {
        return polish_unit_name(style, unit, value);
    }
    english_unit_name(style, unit, value)
}

fn english_unit_name(style: RelativeTimeStyle, unit: RelativeUnit, value: f64) -> &'static str {
    let singular = value.abs() == 1.0;
    match style {
        RelativeTimeStyle::Long => match (unit, singular) {
            (RelativeUnit::Second, true) => "second",
            (RelativeUnit::Minute, true) => "minute",
            (RelativeUnit::Hour, true) => "hour",
            (RelativeUnit::Day, true) => "day",
            (RelativeUnit::Week, true) => "week",
            (RelativeUnit::Month, true) => "month",
            (RelativeUnit::Quarter, true) => "quarter",
            (RelativeUnit::Year, true) => "year",
            (RelativeUnit::Second, false) => "seconds",
            (RelativeUnit::Minute, false) => "minutes",
            (RelativeUnit::Hour, false) => "hours",
            (RelativeUnit::Day, false) => "days",
            (RelativeUnit::Week, false) => "weeks",
            (RelativeUnit::Month, false) => "months",
            (RelativeUnit::Quarter, false) => "quarters",
            (RelativeUnit::Year, false) => "years",
        },
        RelativeTimeStyle::Short | RelativeTimeStyle::Narrow => match (unit, singular) {
            (RelativeUnit::Second, _) => "sec.",
            (RelativeUnit::Minute, _) => "min.",
            (RelativeUnit::Hour, _) => "hr.",
            (RelativeUnit::Day, true) => "day",
            (RelativeUnit::Day, false) => "days",
            (RelativeUnit::Week, _) => "wk.",
            (RelativeUnit::Month, _) => "mo.",
            (RelativeUnit::Quarter, true) => "qtr.",
            (RelativeUnit::Quarter, false) => "qtrs.",
            (RelativeUnit::Year, _) => "yr.",
        },
    }
}

fn polish_unit_name(style: RelativeTimeStyle, unit: RelativeUnit, value: f64) -> &'static str {
    let category = polish_plural_category(value);
    match style {
        RelativeTimeStyle::Long => match unit {
            RelativeUnit::Second => regular_polish("sekund", category),
            RelativeUnit::Minute => regular_polish("minut", category),
            RelativeUnit::Hour => regular_polish("godzin", category),
            RelativeUnit::Day => match category {
                PolishPlural::One => "dzień",
                PolishPlural::Other => "dnia",
                _ => "dni",
            },
            RelativeUnit::Week => match category {
                PolishPlural::One => "tydzień",
                PolishPlural::Few => "tygodnie",
                PolishPlural::Other => "tygodnia",
                _ => "tygodni",
            },
            RelativeUnit::Month => match category {
                PolishPlural::One => "miesiąc",
                PolishPlural::Few => "miesiące",
                PolishPlural::Other => "miesiąca",
                _ => "miesięcy",
            },
            RelativeUnit::Quarter => match category {
                PolishPlural::One => "kwartał",
                PolishPlural::Few => "kwartały",
                PolishPlural::Other => "kwartału",
                _ => "kwartałów",
            },
            RelativeUnit::Year => match category {
                PolishPlural::One => "rok",
                PolishPlural::Few => "lata",
                PolishPlural::Other => "roku",
                _ => "lat",
            },
        },
        RelativeTimeStyle::Short => match unit {
            RelativeUnit::Second => "sek.",
            RelativeUnit::Minute => "min",
            RelativeUnit::Hour => "godz.",
            RelativeUnit::Day => match category {
                PolishPlural::One => "dzień",
                PolishPlural::Other => "dnia",
                _ => "dni",
            },
            RelativeUnit::Week => match category {
                PolishPlural::One => "tydz.",
                _ => "tyg.",
            },
            RelativeUnit::Month => "mies.",
            RelativeUnit::Quarter => "kw.",
            RelativeUnit::Year => match category {
                PolishPlural::One => "rok",
                PolishPlural::Few => "lata",
                PolishPlural::Other => "roku",
                _ => "lat",
            },
        },
        RelativeTimeStyle::Narrow => match unit {
            RelativeUnit::Second => "s",
            RelativeUnit::Minute => "min",
            RelativeUnit::Hour => "g.",
            RelativeUnit::Day => match category {
                PolishPlural::One => "dzień",
                PolishPlural::Other => "dnia",
                _ => "dni",
            },
            RelativeUnit::Week => match category {
                PolishPlural::One => "tydz.",
                _ => "tyg.",
            },
            RelativeUnit::Month => "mies.",
            RelativeUnit::Quarter => "kw.",
            RelativeUnit::Year => match category {
                PolishPlural::One => "rok",
                PolishPlural::Few => "lata",
                PolishPlural::Other => "roku",
                _ => "lat",
            },
        },
    }
}

fn regular_polish(stem: &'static str, category: PolishPlural) -> &'static str {
    match (stem, category) {
        ("sekund", PolishPlural::One) => "sekundę",
        ("sekund", PolishPlural::Few | PolishPlural::Other) => "sekundy",
        ("sekund", PolishPlural::Many) => "sekund",
        ("minut", PolishPlural::One) => "minutę",
        ("minut", PolishPlural::Few | PolishPlural::Other) => "minuty",
        ("minut", PolishPlural::Many) => "minut",
        ("godzin", PolishPlural::One) => "godzinę",
        ("godzin", PolishPlural::Few | PolishPlural::Other) => "godziny",
        ("godzin", PolishPlural::Many) => "godzin",
        _ => stem,
    }
}

#[derive(Debug, Clone, Copy)]
enum PolishPlural {
    One,
    Few,
    Many,
    Other,
}

fn polish_plural_category(value: f64) -> PolishPlural {
    let abs = value.abs();
    if abs.fract() != 0.0 {
        return PolishPlural::Other;
    }
    let n = abs as i64;
    if n == 1 {
        return PolishPlural::One;
    }
    let mod10 = n % 10;
    let mod100 = n % 100;
    if (2..=4).contains(&mod10) && !(12..=14).contains(&mod100) {
        PolishPlural::Few
    } else {
        PolishPlural::Many
    }
}
