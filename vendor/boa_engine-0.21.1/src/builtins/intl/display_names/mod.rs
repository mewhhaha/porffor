use std::{fmt, str::FromStr};

use boa_gc::{Finalize, Trace};
use icu_decimal::provider::DecimalSymbolsV1;
use icu_locale::Locale;

use crate::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::{
        BuiltInConstructor, BuiltInObject, IntrinsicObject, OrdinaryObject,
        builder::BuiltInBuilder,
        intl::{
            Service,
            locale::{canonicalize_locale_list, filter_locales, resolve_locale},
            options::{IntlOptions, LocaleMatcher},
        },
        options::{OptionType, ParsableOptionType, get_option, get_options_object},
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
    symbol::JsSymbol,
};

#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct DisplayNames {
    locale: Locale,
    style: DisplayNamesStyle,
    typ: DisplayNamesType,
    fallback: DisplayNamesFallback,
    language_display: DisplayNamesLanguageDisplay,
}

impl Service for DisplayNames {
    type LangMarker = DecimalSymbolsV1;
    type LocaleOptions = ();
}

impl IntrinsicObject for DisplayNames {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                JsSymbol::to_string_tag(),
                js_string!("Intl.DisplayNames"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::of, js_string!("of"), 1)
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DisplayNames {
    const NAME: JsString = js_string!("DisplayNames");
}

impl BuiltInConstructor for DisplayNames {
    const CONSTRUCTOR_ARGUMENTS: usize = 2;
    const PROTOTYPE_STORAGE_SLOTS: usize = 3;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::display_names;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.DisplayNames` constructor without `new`")
                .into());
        }

        // The custom newTarget prototype is observed before option validation.
        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::display_names, context)?;
        let display_names = Self::new(args.get_or_undefined(0), args.get_or_undefined(1), context)?;

        Ok(JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            display_names,
        )
        .into())
    }
}

impl DisplayNames {
    fn new(locales: &JsValue, options: &JsValue, context: &mut Context) -> JsResult<Self> {
        let requested_locales = canonicalize_locale_list(locales, context)?;
        let options = get_options_object(options)?;
        let matcher =
            get_option::<LocaleMatcher>(&options, js_string!("localeMatcher"), context)?
                .unwrap_or_default();
        let style =
            get_option::<DisplayNamesStyle>(&options, js_string!("style"), context)?
                .unwrap_or_default();
        let typ = get_option::<DisplayNamesType>(&options, js_string!("type"), context)?
            .ok_or_else(|| {
                JsNativeError::typ().with_message("Intl.DisplayNames requires a type option")
            })?;
        let fallback =
            get_option::<DisplayNamesFallback>(&options, js_string!("fallback"), context)?
                .unwrap_or_default();
        let language_display = get_option::<DisplayNamesLanguageDisplay>(
            &options,
            js_string!("languageDisplay"),
            context,
        )?
        .unwrap_or_default();

        let locale = resolve_locale::<Self>(
            requested_locales,
            &mut IntlOptions {
                matcher,
                ..Default::default()
            },
            context.intl_provider(),
        )?;

        Ok(Self {
            locale,
            style,
            typ,
            fallback,
            language_display,
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

    fn of(this: &JsValue, args: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        let object = this.as_object();
        let display_names = object
            .as_ref()
            .and_then(|object| object.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value must be an Intl.DisplayNames object")
            })?;
        let code = args.get_or_undefined(0).to_string(context)?.to_std_string_escaped();
        let canonical = canonical_code_for_display_names(display_names.typ, &code)?;
        if let Some(display) = display_name(display_names.typ, &canonical) {
            return Ok(js_string!(display).into());
        }
        match display_names.fallback {
            DisplayNamesFallback::Code => Ok(js_string!(canonical).into()),
            DisplayNamesFallback::None => Ok(JsValue::undefined()),
        }
    }

    fn resolved_options(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object();
        let display_names = object
            .as_ref()
            .and_then(|object| object.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value must be an Intl.DisplayNames object")
            })?;
        let options = context
            .intrinsics()
            .templates()
            .ordinary_object()
            .create(OrdinaryObject, vec![]);
        options.create_data_property_or_throw(
            js_string!("locale"),
            js_string!(display_names.locale.to_string()),
            context,
        )?;
        options.create_data_property_or_throw(
            js_string!("style"),
            display_names.style.to_js_string(),
            context,
        )?;
        options.create_data_property_or_throw(
            js_string!("type"),
            display_names.typ.to_js_string(),
            context,
        )?;
        options.create_data_property_or_throw(
            js_string!("fallback"),
            display_names.fallback.to_js_string(),
            context,
        )?;
        if display_names.typ == DisplayNamesType::Language {
            options.create_data_property_or_throw(
                js_string!("languageDisplay"),
                display_names.language_display.to_js_string(),
                context,
            )?;
        }
        Ok(options.into())
    }
}

fn canonical_code_for_display_names(
    typ: DisplayNamesType,
    code: &str,
) -> JsResult<String> {
    match typ {
        DisplayNamesType::Language => canonical_language_code(code),
        DisplayNamesType::Region => canonical_region_code(code),
        DisplayNamesType::Script => canonical_script_code(code),
        DisplayNamesType::Currency => canonical_currency_code(code),
        DisplayNamesType::Calendar => canonical_calendar_code(code),
        DisplayNamesType::DateTimeField => canonical_date_time_field_code(code),
    }
}

fn display_name(typ: DisplayNamesType, canonical: &str) -> Option<&'static str> {
    if typ == DisplayNamesType::Calendar {
        return match canonical {
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
            | "roc" => Some("calendar"),
            _ => None,
        };
    }

    if typ != DisplayNamesType::DateTimeField {
        return None;
    }
    match canonical {
        "era" => Some("era"),
        "year" => Some("year"),
        "quarter" => Some("quarter"),
        "month" => Some("month"),
        "weekOfYear" => Some("week"),
        "weekday" => Some("day of the week"),
        "day" => Some("day"),
        "dayPeriod" => Some("AM/PM"),
        "hour" => Some("hour"),
        "minute" => Some("minute"),
        "second" => Some("second"),
        "timeZoneName" => Some("time zone"),
        _ => None,
    }
}

fn canonical_language_code(code: &str) -> JsResult<String> {
    if code == "root" || code.contains('_') {
        return Err(invalid_code());
    }
    let subtags: Vec<&str> = code.split('-').collect();
    if subtags.is_empty() || subtags.iter().any(|subtag| subtag.is_empty()) {
        return Err(invalid_code());
    }
    let language = subtags[0];
    if !is_alpha(language) || !matches!(language.len(), 2 | 3 | 5..=8) {
        return Err(invalid_code());
    }

    let mut index = 1;
    let mut out = vec![language.to_ascii_lowercase()];
    if let Some(script) = subtags.get(index)
        && script.len() == 4
        && is_alpha(script)
    {
        out.push(to_ascii_titlecase(script));
        index += 1;
    }
    if let Some(region) = subtags.get(index)
        && is_region_subtag(region)
    {
        out.push(region.to_ascii_uppercase());
        index += 1;
    }

    let mut variants = Vec::<String>::new();
    while let Some(variant) = subtags.get(index) {
        if !is_variant_subtag(variant) {
            return Err(invalid_code());
        }
        let variant = variant.to_ascii_lowercase();
        if variants.contains(&variant) {
            return Err(invalid_code());
        }
        variants.push(variant.clone());
        out.push(variant);
        index += 1;
    }

    Ok(out.join("-"))
}

fn canonical_region_code(code: &str) -> JsResult<String> {
    if is_region_subtag(code) {
        Ok(code.to_ascii_uppercase())
    } else {
        Err(invalid_code())
    }
}

fn canonical_script_code(code: &str) -> JsResult<String> {
    if code.len() == 4 && is_alpha(code) {
        Ok(to_ascii_titlecase(code))
    } else {
        Err(invalid_code())
    }
}

fn canonical_currency_code(code: &str) -> JsResult<String> {
    if code.len() == 3 && is_alpha(code) {
        Ok(code.to_ascii_uppercase())
    } else {
        Err(invalid_code())
    }
}

fn canonical_calendar_code(code: &str) -> JsResult<String> {
    if is_unicode_type_sequence(code) {
        Ok(code.to_ascii_lowercase())
    } else {
        Err(invalid_code())
    }
}

fn canonical_date_time_field_code(code: &str) -> JsResult<String> {
    match code {
        "era" | "year" | "quarter" | "month" | "weekOfYear" | "weekday" | "day"
        | "dayPeriod" | "hour" | "minute" | "second" | "timeZoneName" => Ok(code.to_owned()),
        _ => Err(invalid_code()),
    }
}

fn is_region_subtag(code: &str) -> bool {
    (code.len() == 2 && is_alpha(code)) || (code.len() == 3 && is_digit(code))
}

fn is_variant_subtag(code: &str) -> bool {
    ((5..=8).contains(&code.len()) && is_alnum(code))
        || (code.len() == 4 && code.as_bytes()[0].is_ascii_digit() && is_alnum(code))
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

fn is_alpha(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphabetic())
}

fn is_digit(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_digit())
}

fn is_alnum(value: &str) -> bool {
    value.bytes().all(|byte| byte.is_ascii_alphanumeric())
}

fn to_ascii_titlecase(value: &str) -> String {
    let mut chars = value.chars();
    let first = chars
        .next()
        .expect("script subtags are checked to be non-empty");
    let mut out = first.to_ascii_uppercase().to_string();
    out.push_str(&chars.as_str().to_ascii_lowercase());
    out
}

fn invalid_code() -> crate::JsError {
    JsNativeError::range()
        .with_message("invalid display names code")
        .into()
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum DisplayNamesStyle {
    Narrow,
    Short,
    #[default]
    Long,
}

impl DisplayNamesStyle {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Narrow => js_string!("narrow"),
            Self::Short => js_string!("short"),
            Self::Long => js_string!("long"),
        }
    }
}

impl FromStr for DisplayNamesStyle {
    type Err = ParseDisplayNamesStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "narrow" => Ok(Self::Narrow),
            "short" => Ok(Self::Short),
            "long" => Ok(Self::Long),
            _ => Err(ParseDisplayNamesStyleError),
        }
    }
}

impl ParsableOptionType for DisplayNamesStyle {}

#[derive(Debug)]
struct ParseDisplayNamesStyleError;

impl fmt::Display for ParseDisplayNamesStyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid display names style")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum DisplayNamesType {
    Language,
    Region,
    Script,
    Currency,
    Calendar,
    DateTimeField,
}

impl DisplayNamesType {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Language => js_string!("language"),
            Self::Region => js_string!("region"),
            Self::Script => js_string!("script"),
            Self::Currency => js_string!("currency"),
            Self::Calendar => js_string!("calendar"),
            Self::DateTimeField => js_string!("dateTimeField"),
        }
    }
}

impl FromStr for DisplayNamesType {
    type Err = ParseDisplayNamesTypeError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "language" => Ok(Self::Language),
            "region" => Ok(Self::Region),
            "script" => Ok(Self::Script),
            "currency" => Ok(Self::Currency),
            "calendar" => Ok(Self::Calendar),
            "dateTimeField" => Ok(Self::DateTimeField),
            _ => Err(ParseDisplayNamesTypeError),
        }
    }
}

impl OptionType for DisplayNamesType {
    fn from_value(value: JsValue, context: &mut Context) -> JsResult<Self> {
        value
            .to_string(context)?
            .to_std_string_escaped()
            .parse::<Self>()
            .map_err(|err| JsNativeError::range().with_message(err.to_string()).into())
    }
}

#[derive(Debug)]
struct ParseDisplayNamesTypeError;

impl fmt::Display for ParseDisplayNamesTypeError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid display names type")
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum DisplayNamesFallback {
    #[default]
    Code,
    None,
}

impl DisplayNamesFallback {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Code => js_string!("code"),
            Self::None => js_string!("none"),
        }
    }
}

impl FromStr for DisplayNamesFallback {
    type Err = ParseDisplayNamesFallbackError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "code" => Ok(Self::Code),
            "none" => Ok(Self::None),
            _ => Err(ParseDisplayNamesFallbackError),
        }
    }
}

impl ParsableOptionType for DisplayNamesFallback {}

#[derive(Debug)]
struct ParseDisplayNamesFallbackError;

impl fmt::Display for ParseDisplayNamesFallbackError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid display names fallback")
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum DisplayNamesLanguageDisplay {
    #[default]
    Dialect,
    Standard,
}

impl DisplayNamesLanguageDisplay {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Dialect => js_string!("dialect"),
            Self::Standard => js_string!("standard"),
        }
    }
}

impl FromStr for DisplayNamesLanguageDisplay {
    type Err = ParseDisplayNamesLanguageDisplayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "dialect" => Ok(Self::Dialect),
            "standard" => Ok(Self::Standard),
            _ => Err(ParseDisplayNamesLanguageDisplayError),
        }
    }
}

impl ParsableOptionType for DisplayNamesLanguageDisplay {}

#[derive(Debug)]
struct ParseDisplayNamesLanguageDisplayError;

impl fmt::Display for ParseDisplayNamesLanguageDisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid language display option")
    }
}
