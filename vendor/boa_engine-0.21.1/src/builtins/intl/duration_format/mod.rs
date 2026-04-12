use std::{fmt, str::FromStr};

use boa_gc::{Finalize, Trace};
use icu_decimal::{preferences::NumberingSystem, provider::DecimalSymbolsV1};
use icu_list::{
    ListFormatter, ListFormatterPreferences,
    options::{ListFormatterOptions, ListLength},
};
use icu_locale::{
    Locale,
    extensions::unicode::{Value, key},
};
use icu_provider::DataMarkerAttributes;
use temporal_rs::Duration as InnerDuration;

use crate::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsValue,
    builtins::{
        BuiltInConstructor, BuiltInObject, IntrinsicObject, OrdinaryObject,
        array::Array,
        builder::BuiltInBuilder,
        intl::{
            Service,
            locale::{canonicalize_locale_list, filter_locales, resolve_locale, validate_extension},
            number_format::{
                NumberFormat, NumberPart, UnitDisplay as NumberUnitDisplay,
                numbering_system_is_supported,
            },
            options::{IntlOptions, LocaleMatcher},
        },
        options::{ParsableOptionType, get_option, get_options_object},
        temporal::to_temporal_duration,
    },
    context::intrinsics::{Intrinsics, StandardConstructor, StandardConstructors},
    js_string,
    object::internal_methods::get_prototype_from_constructor,
    property::Attribute,
    realm::Realm,
};

#[derive(Debug, Clone, Trace, Finalize, JsData)]
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct DurationFormat {
    locale: Locale,
    numbering_system: Option<Value>,
    style: DurationStyle,
    units: [DurationUnitOptions; 10],
    fractional_digits: Option<u8>,
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
struct DurationUnitOptions {
    style: UnitStyle,
    display: UnitDisplay,
}

impl Service for DurationFormat {
    type LangMarker = DecimalSymbolsV1;
    type LocaleOptions = DurationFormatLocaleOptions;

    fn resolve(
        locale: &mut Locale,
        options: &mut Self::LocaleOptions,
        provider: &crate::context::icu::IntlProvider,
    ) {
        let extension_numbering_system = locale.extensions.unicode.keywords.get(&key!("nu")).cloned();
        let option_numbering_system = options.numbering_system.take().filter(|nu| {
            NumberingSystem::try_from(nu.clone()).is_ok_and(|nu| {
                numbering_system_is_supported(nu.as_str()) || {
                    let attr = DataMarkerAttributes::from_str_or_panic(nu.as_str());
                    validate_extension::<Self::LangMarker>(locale.id.clone(), attr, provider)
                }
            })
        });
        let extension_numbering_system = extension_numbering_system.filter(|nu| {
            NumberingSystem::try_from(nu.clone()).is_ok_and(|nu| {
                numbering_system_is_supported(nu.as_str()) || {
                    let attr = DataMarkerAttributes::from_str_or_panic(nu.as_str());
                    validate_extension::<Self::LangMarker>(locale.id.clone(), attr, provider)
                }
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
        {
            locale.extensions.unicode.keywords.set(key!("nu"), nu);
        }
        options.numbering_system = numbering_system;
    }
}

#[derive(Debug, Default)]
pub(crate) struct DurationFormatLocaleOptions {
    numbering_system: Option<Value>,
}

impl IntrinsicObject for DurationFormat {
    fn init(realm: &Realm) {
        BuiltInBuilder::from_standard_constructor::<Self>(realm)
            .static_method(
                Self::supported_locales_of,
                js_string!("supportedLocalesOf"),
                1,
            )
            .property(
                crate::symbol::JsSymbol::to_string_tag(),
                js_string!("Intl.DurationFormat"),
                Attribute::CONFIGURABLE,
            )
            .method(Self::format, js_string!("format"), 1)
            .method(Self::format_to_parts, js_string!("formatToParts"), 1)
            .method(Self::resolved_options, js_string!("resolvedOptions"), 0)
            .build();
    }

    fn get(intrinsics: &Intrinsics) -> JsObject {
        Self::STANDARD_CONSTRUCTOR(intrinsics.constructors()).constructor()
    }
}

impl BuiltInObject for DurationFormat {
    const NAME: JsString = js_string!("DurationFormat");
}

impl BuiltInConstructor for DurationFormat {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 4;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::duration_format;

    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if new_target.is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("cannot call `Intl.DurationFormat` constructor without `new`")
                .into());
        }

        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);
        let requested_locales = canonicalize_locale_list(locales, context)?;
        let options = get_options_object(options)?;

        let matcher =
            get_option::<LocaleMatcher>(&options, js_string!("localeMatcher"), context)?
                .unwrap_or_default();
        let numbering_system =
            get_option::<NumberingSystem>(&options, js_string!("numberingSystem"), context)?;
        let requested_numbering_system = numbering_system.clone();
        let mut intl_options = IntlOptions {
            matcher,
            service_options: DurationFormatLocaleOptions {
                numbering_system: numbering_system.map(Value::from),
            },
        };
        let locale = resolve_locale::<Self>(
            requested_locales,
            &mut intl_options,
            context.intl_provider(),
        )?;

        let style =
            get_option::<DurationStyle>(&options, js_string!("style"), context)?.unwrap_or_default();
        let units = read_unit_options(&options, style, context)?;
        let fractional_digits = get_fractional_digits(&options, context)?;

        let prototype =
            get_prototype_from_constructor(new_target, StandardConstructors::duration_format, context)?;
        let duration_format = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            Self {
                locale,
                numbering_system: intl_options
                    .service_options
                    .numbering_system
                    .or_else(|| {
                        requested_numbering_system
                            .filter(|nu| numbering_system_is_supported(nu.as_str()))
                            .map(Value::from)
                    }),
                style,
                units,
                fractional_digits,
            },
        );
        Ok(duration_format.into())
    }
}

impl DurationFormat {
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
        let df = object
            .as_ref()
            .and_then(|object| object.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value must be an Intl.DurationFormat object")
            })?;
        let duration = to_temporal_duration(args.get_or_undefined(0), context)?;
        let parts = df.partition_duration(duration, context)?;
        Ok(join_duration_parts(&parts).into())
    }

    fn format_to_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object();
        let df = object
            .as_ref()
            .and_then(|object| object.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value must be an Intl.DurationFormat object")
            })?;
        let duration = to_temporal_duration(args.get_or_undefined(0), context)?;
        let parts = df.partition_duration(duration, context)?;
        Ok(duration_parts_to_array(parts, context).into())
    }

    fn resolved_options(
        this: &JsValue,
        _: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let object = this.as_object();
        let df = object
            .as_ref()
            .and_then(|object| object.downcast_ref::<Self>())
            .ok_or_else(|| {
                JsNativeError::typ()
                    .with_message("this value must be an Intl.DurationFormat object")
            })?;
        let options = context
            .intrinsics()
            .templates()
            .ordinary_object()
            .create(OrdinaryObject, vec![]);

        options.create_data_property_or_throw(
            js_string!("locale"),
            js_string!(df.locale.to_string()),
            context,
        )?;
        options.create_data_property_or_throw(
            js_string!("numberingSystem"),
            js_string!(
                df.numbering_system
                    .as_ref()
                    .map_or("latn".to_owned(), ToString::to_string)
            ),
            context,
        )?;
        options.create_data_property_or_throw(
            js_string!("style"),
            df.style.to_js_string(),
            context,
        )?;

        for (i, unit) in DURATION_UNITS.iter().enumerate() {
            let opts = df.units[i];
            options.create_data_property_or_throw(
                js_string!(unit.plural),
                opts.style.to_js_string(),
                context,
            )?;
            options.create_data_property_or_throw(
                js_string!(unit.display_name),
                opts.display.to_js_string(),
                context,
            )?;
        }

        if let Some(fractional_digits) = df.fractional_digits {
            options.create_data_property_or_throw(
                js_string!("fractionalDigits"),
                fractional_digits,
                context,
            )?;
        }

        Ok(options.into())
    }

    pub(crate) fn format_duration_to_string(
        locales: &JsValue,
        options: &JsValue,
        duration: InnerDuration,
        context: &mut Context,
    ) -> JsResult<JsString> {
        let df = Self::new(locales, options, context)?;
        let parts = df.partition_duration(duration, context)?;
        Ok(join_duration_parts(&parts))
    }

    fn new(locales: &JsValue, options: &JsValue, context: &mut Context) -> JsResult<Self> {
        let requested_locales = canonicalize_locale_list(locales, context)?;
        let options = get_options_object(options)?;
        let matcher =
            get_option::<LocaleMatcher>(&options, js_string!("localeMatcher"), context)?
                .unwrap_or_default();
        let numbering_system =
            get_option::<NumberingSystem>(&options, js_string!("numberingSystem"), context)?;
        let mut intl_options = IntlOptions {
            matcher,
            service_options: DurationFormatLocaleOptions {
                numbering_system: numbering_system.map(Value::from),
            },
        };
        let locale = resolve_locale::<Self>(
            requested_locales,
            &mut intl_options,
            context.intl_provider(),
        )?;
        let style =
            get_option::<DurationStyle>(&options, js_string!("style"), context)?.unwrap_or_default();
        let units = read_unit_options(&options, style, context)?;
        let fractional_digits = get_fractional_digits(&options, context)?;
        Ok(Self {
            locale,
            numbering_system: intl_options.service_options.numbering_system,
            style,
            units,
            fractional_digits,
        })
    }

    fn partition_duration(
        &self,
        duration: InnerDuration,
        context: &mut Context,
    ) -> JsResult<Vec<DurationPart>> {
        let values = duration_values(&duration);
        let negative = values.iter().any(|value| *value < 0);
        let mut result: Vec<Vec<DurationPart>> = Vec::new();
        let mut need_separator = false;
        let mut display_negative_sign = true;

        for (index, unit) in DURATION_UNITS.iter().enumerate() {
            let mut value = values[index];
            let style = self.units[index].style;
            let display = self.units[index].display;
            let mut done = false;
            let mut value_string = value.to_string();
            let mut min_fraction_digits = None;
            let mut max_fraction_digits = None;
            let mut rounding_trunc = false;

            if matches!(unit.plural, "seconds" | "milliseconds" | "microseconds") {
                let next_style = self.units[index + 1].style;
                if next_style == UnitStyle::Numeric {
                    let exponent = match unit.plural {
                        "seconds" => 9,
                        "milliseconds" => 6,
                        _ => 3,
                    };
                    value_string = duration_to_fractional(&values, exponent);
                    value = if value_string == "0" { 0 } else { 1 };
                    min_fraction_digits = Some(self.fractional_digits.unwrap_or(0));
                    max_fraction_digits = Some(self.fractional_digits.unwrap_or(9));
                    rounding_trunc = true;
                    done = true;
                }
            }

            let later_time_nonzero = values[UnitIndex::Seconds as usize] != 0
                || values[UnitIndex::Milliseconds as usize] != 0
                || values[UnitIndex::Microseconds as usize] != 0
                || values[UnitIndex::Nanoseconds as usize] != 0;
            let display_required = unit.plural == "minutes"
                && ((need_separator
                    && (self.units[UnitIndex::Seconds as usize].display == UnitDisplay::Always
                        || later_time_nonzero))
                    || (!need_separator
                        && matches!(style, UnitStyle::Numeric | UnitStyle::TwoDigit)
                        && values[UnitIndex::Seconds as usize] != 0
                        && values[UnitIndex::Milliseconds as usize] == 0
                        && values[UnitIndex::Microseconds as usize] == 0
                        && values[UnitIndex::Nanoseconds as usize] == 0));

            if value != 0 || display == UnitDisplay::Always || display_required {
                let sign_display_never = if display_negative_sign {
                    display_negative_sign = false;
                    if value == 0 && negative {
                        value_string = "-0".to_owned();
                    }
                    false
                } else {
                    true
                };

                let mut list = if need_separator {
                    let mut list = result.pop().unwrap_or_default();
                    list.push(DurationPart::literal(js_string!(":")));
                    list
                } else {
                    Vec::new()
                };
                let number_parts = self.format_number_parts(
                    unit.singular,
                    style,
                    &value_string,
                    sign_display_never,
                    min_fraction_digits,
                    max_fraction_digits,
                    rounding_trunc,
                    context,
                )?;
                list.extend(number_parts.into_iter().map(|part| DurationPart {
                    kind: part.kind,
                    value: part.value,
                    unit: Some(unit.singular),
                }));

                if !need_separator {
                    if matches!(style, UnitStyle::TwoDigit | UnitStyle::Numeric) {
                        need_separator = true;
                    }
                    result.push(list);
                } else {
                    result.push(list);
                }
            }

            if done {
                break;
            }
        }

        self.list_format_parts(result, context)
    }

    #[allow(clippy::too_many_arguments)]
    fn format_number_parts(
        &self,
        unit: &'static str,
        style: UnitStyle,
        value: &str,
        sign_display_never: bool,
        min_fraction_digits: Option<u8>,
        max_fraction_digits: Option<u8>,
        rounding_trunc: bool,
        context: &mut Context,
    ) -> JsResult<Vec<NumberPart>> {
        let options = JsObject::with_null_proto();
        if let Some(numbering_system) = &self.numbering_system {
            options.create_data_property_or_throw(
                js_string!("numberingSystem"),
                js_string!(numbering_system.to_string()),
                context,
            )?;
        }
        if style == UnitStyle::TwoDigit {
            options.create_data_property_or_throw(js_string!("minimumIntegerDigits"), 2, context)?;
        }
        if matches!(style, UnitStyle::Numeric | UnitStyle::TwoDigit) {
            options.create_data_property_or_throw(js_string!("useGrouping"), false, context)?;
        } else {
            options.create_data_property_or_throw(js_string!("style"), js_string!("unit"), context)?;
            options.create_data_property_or_throw(js_string!("unit"), js_string!(unit), context)?;
            options.create_data_property_or_throw(
                js_string!("unitDisplay"),
                style.to_number_unit_display().to_js_string(),
                context,
            )?;
        }
        if sign_display_never {
            options.create_data_property_or_throw(
                js_string!("signDisplay"),
                js_string!("never"),
                context,
            )?;
        }
        if let Some(min) = min_fraction_digits {
            options.create_data_property_or_throw(js_string!("minimumFractionDigits"), min, context)?;
        }
        if let Some(max) = max_fraction_digits {
            options.create_data_property_or_throw(js_string!("maximumFractionDigits"), max, context)?;
        }
        if rounding_trunc {
            options.create_data_property_or_throw(
                js_string!("roundingMode"),
                js_string!("trunc"),
                context,
            )?;
        }

        let nf = NumberFormat::new(
            &js_string!(self.locale.to_string()).into(),
            &options.into(),
            context,
        )?;
        let value = if value == "-0" {
            JsValue::new(-0.0)
        } else {
            JsValue::new(js_string!(value))
        };
        nf.format_value_to_parts(&value, context)
    }

    fn list_format_parts(
        &self,
        result: Vec<Vec<DurationPart>>,
        context: &mut Context,
    ) -> JsResult<Vec<DurationPart>> {
        if result.is_empty() {
            return Ok(Vec::new());
        }
        let strings = result
            .iter()
            .map(|parts| join_duration_parts(parts).to_std_string_escaped())
            .collect::<Vec<_>>();
        let list_style = match self.style {
            DurationStyle::Long => ListLength::Wide,
            DurationStyle::Narrow => ListLength::Narrow,
            DurationStyle::Short | DurationStyle::Digital => ListLength::Short,
        };
        let prefs = ListFormatterPreferences::from(&self.locale);
        let formatter = ListFormatter::try_new_unit_with_buffer_provider(
            context.intl_provider().erased_provider(),
            prefs,
            ListFormatterOptions::default().with_length(list_style),
        )
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;

        let list_parts = collect_list_parts(
            formatter.format(strings.iter().cloned())
        )?;
        let mut sources = result.into_iter();
        let mut flattened = Vec::new();
        for part in list_parts {
            match part {
                ListPart::Element => {
                    if let Some(parts) = sources.next() {
                        flattened.extend(parts);
                    }
                }
                ListPart::Literal(value) => flattened.push(DurationPart::literal(value)),
            }
        }
        Ok(flattened)
    }
}

fn read_unit_options(
    options: &JsObject,
    base_style: DurationStyle,
    context: &mut Context,
) -> JsResult<[DurationUnitOptions; 10]> {
    let mut result = [DurationUnitOptions {
        style: UnitStyle::Short,
        display: UnitDisplay::Auto,
    }; 10];
    let mut previous = None;

    for (index, unit) in DURATION_UNITS.iter().enumerate() {
        let explicit_style = get_option::<UnitStyle>(options, js_string!(unit.plural), context)?;
        let mut style = explicit_style;
        if let Some(style) = style
            && !unit.allowed(style)
        {
            return Err(JsNativeError::range()
                .with_message("invalid duration unit style")
                .into());
        }

        if style.is_none() {
            style = Some(default_unit_style(unit, base_style, previous));
        }
        let style = style.expect("unit style defaults are always present");
        let style = if matches!(previous, Some(UnitStyle::Numeric | UnitStyle::TwoDigit))
            && style == UnitStyle::Numeric
            && matches!(unit.plural, "minutes" | "seconds")
        {
            UnitStyle::TwoDigit
        } else {
            style
        };

        if matches!(previous, Some(UnitStyle::Numeric | UnitStyle::TwoDigit))
            && !matches!(style, UnitStyle::Numeric | UnitStyle::TwoDigit)
        {
            return Err(JsNativeError::range()
                .with_message("duration unit after numeric style must be numeric")
                .into());
        }

        let display =
            get_option::<UnitDisplay>(options, js_string!(unit.display_name), context)?
                .unwrap_or(if explicit_style.is_some() {
                    UnitDisplay::Always
                } else {
                    UnitDisplay::Auto
                });

        result[index] = DurationUnitOptions { style, display };
        previous = Some(style);
    }

    Ok(result)
}

fn default_unit_style(
    unit: &DurationUnit,
    base_style: DurationStyle,
    previous: Option<UnitStyle>,
) -> UnitStyle {
    if matches!(previous, Some(UnitStyle::Numeric | UnitStyle::TwoDigit)) {
        return if matches!(unit.plural, "minutes" | "seconds") {
            UnitStyle::TwoDigit
        } else {
            UnitStyle::Numeric
        };
    }

    match base_style {
        DurationStyle::Long => UnitStyle::Long,
        DurationStyle::Short => UnitStyle::Short,
        DurationStyle::Narrow => UnitStyle::Narrow,
        DurationStyle::Digital => {
            if unit.numeric_allowed {
                match unit.plural {
                    "hours" => UnitStyle::Numeric,
                    "minutes" | "seconds" => UnitStyle::TwoDigit,
                    _ => UnitStyle::Numeric,
                }
            } else {
                UnitStyle::Short
            }
        }
    }
}

fn duration_values(duration: &InnerDuration) -> [i128; 10] {
    [
        duration.years() as i128,
        duration.months() as i128,
        duration.weeks() as i128,
        duration.days() as i128,
        duration.hours() as i128,
        duration.minutes() as i128,
        duration.seconds() as i128,
        duration.milliseconds() as i128,
        duration.microseconds(),
        duration.nanoseconds(),
    ]
}

fn get_fractional_digits(options: &JsObject, context: &mut Context) -> JsResult<Option<u8>> {
    let value = options.get(js_string!("fractionalDigits"), context)?;
    if value.is_undefined() {
        return Ok(None);
    }
    let value = value.to_number(context)?;
    if value.is_nan() || !(0.0..=9.0).contains(&value) {
        return Err(JsNativeError::range()
            .with_message("fractionalDigits must be between 0 and 9")
            .into());
    }
    Ok(Some(value.floor() as u8))
}

fn duration_to_fractional(values: &[i128; 10], exponent: u32) -> String {
    let seconds = values[UnitIndex::Seconds as usize];
    let milliseconds = values[UnitIndex::Milliseconds as usize];
    let microseconds = values[UnitIndex::Microseconds as usize];
    let nanoseconds = values[UnitIndex::Nanoseconds as usize];

    match exponent {
        9 if milliseconds == 0 && microseconds == 0 && nanoseconds == 0 => {
            return seconds.to_string();
        }
        6 if microseconds == 0 && nanoseconds == 0 => return milliseconds.to_string(),
        3 if nanoseconds == 0 => return microseconds.to_string(),
        _ => {}
    }

    let mut ns = nanoseconds;
    if exponent >= 9 {
        ns += seconds * 1_000_000_000;
    }
    if exponent >= 6 {
        ns += milliseconds * 1_000_000;
    }
    if exponent >= 3 {
        ns += microseconds * 1_000;
    }

    let divisor = 10_i128.pow(exponent);
    let q = ns / divisor;
    let mut r = ns % divisor;
    if r < 0 {
        r = -r;
    }
    format!("{q}.{r:0width$}", width = exponent as usize)
}

#[derive(Debug, Clone)]
struct DurationPart {
    kind: &'static str,
    value: JsString,
    unit: Option<&'static str>,
}

impl DurationPart {
    fn literal(value: JsString) -> Self {
        Self {
            kind: "literal",
            value,
            unit: None,
        }
    }
}

fn join_duration_parts(parts: &[DurationPart]) -> JsString {
    let mut out = String::new();
    for part in parts {
        out.push_str(&part.value.to_std_string_escaped());
    }
    JsString::from(out)
}

fn duration_parts_to_array(parts: Vec<DurationPart>, context: &mut Context) -> JsObject {
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

#[derive(Debug)]
enum ListPart {
    Literal(JsString),
    Element,
}

fn collect_list_parts(
    formatted: impl writeable::Writeable,
) -> JsResult<Vec<ListPart>> {
    use std::fmt::Write;
    use writeable::PartsWrite;

    #[derive(Debug, Default)]
    struct StringWriter(String);

    impl Write for StringWriter {
        fn write_str(&mut self, s: &str) -> fmt::Result {
            self.0.push_str(s);
            Ok(())
        }
    }

    impl PartsWrite for StringWriter {
        type SubPartsWrite = Self;

        fn with_part(
            &mut self,
            _part: writeable::Part,
            mut f: impl FnMut(&mut Self::SubPartsWrite) -> fmt::Result,
        ) -> fmt::Result {
            f(self)
        }
    }

    #[derive(Debug, Default)]
    struct Collector(Vec<ListPart>);

    impl Write for Collector {
        fn write_str(&mut self, _: &str) -> fmt::Result {
            Ok(())
        }
    }

    impl PartsWrite for Collector {
        type SubPartsWrite = StringWriter;

        fn with_part(
            &mut self,
            part: writeable::Part,
            mut f: impl FnMut(&mut Self::SubPartsWrite) -> fmt::Result,
        ) -> fmt::Result {
            let mut writer = StringWriter::default();
            f(&mut writer)?;
            if writer.0.is_empty() {
                return Ok(());
            }
            match part.value {
                "element" => self.0.push(ListPart::Element),
                "literal" => self.0.push(ListPart::Literal(JsString::from(writer.0))),
                _ => {}
            }
            Ok(())
        }
    }

    let mut collector = Collector::default();
    formatted
        .write_to_parts(&mut collector)
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;
    Ok(collector.0)
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum DurationStyle {
    Long,
    #[default]
    Short,
    Narrow,
    Digital,
}

impl DurationStyle {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Long => js_string!("long"),
            Self::Short => js_string!("short"),
            Self::Narrow => js_string!("narrow"),
            Self::Digital => js_string!("digital"),
        }
    }
}

impl FromStr for DurationStyle {
    type Err = ParseDurationStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            "narrow" => Ok(Self::Narrow),
            "digital" => Ok(Self::Digital),
            _ => Err(ParseDurationStyleError),
        }
    }
}

impl ParsableOptionType for DurationStyle {}

#[derive(Debug)]
struct ParseDurationStyleError;

impl fmt::Display for ParseDurationStyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid duration style")
    }
}

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
enum UnitStyle {
    Long,
    Short,
    Narrow,
    Numeric,
    TwoDigit,
}

impl UnitStyle {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Long => js_string!("long"),
            Self::Short => js_string!("short"),
            Self::Narrow => js_string!("narrow"),
            Self::Numeric => js_string!("numeric"),
            Self::TwoDigit => js_string!("2-digit"),
        }
    }

    fn to_number_unit_display(self) -> NumberUnitDisplay {
        match self {
            Self::Long => NumberUnitDisplay::Long,
            Self::Short | Self::Numeric | Self::TwoDigit => NumberUnitDisplay::Short,
            Self::Narrow => NumberUnitDisplay::Narrow,
        }
    }
}

impl FromStr for UnitStyle {
    type Err = ParseUnitStyleError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "long" => Ok(Self::Long),
            "short" => Ok(Self::Short),
            "narrow" => Ok(Self::Narrow),
            "numeric" => Ok(Self::Numeric),
            "2-digit" => Ok(Self::TwoDigit),
            _ => Err(ParseUnitStyleError),
        }
    }
}

impl ParsableOptionType for UnitStyle {}

#[derive(Debug)]
struct ParseUnitStyleError;

impl fmt::Display for ParseUnitStyleError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid duration unit style")
    }
}

#[derive(Debug, Clone, Copy, Default, Eq, PartialEq)]
enum UnitDisplay {
    #[default]
    Auto,
    Always,
}

impl UnitDisplay {
    fn to_js_string(self) -> JsString {
        match self {
            Self::Auto => js_string!("auto"),
            Self::Always => js_string!("always"),
        }
    }
}

impl FromStr for UnitDisplay {
    type Err = ParseUnitDisplayError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "auto" => Ok(Self::Auto),
            "always" => Ok(Self::Always),
            _ => Err(ParseUnitDisplayError),
        }
    }
}

impl ParsableOptionType for UnitDisplay {}

#[derive(Debug)]
struct ParseUnitDisplayError;

impl fmt::Display for ParseUnitDisplayError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("provided string was not a valid duration unit display")
    }
}

#[derive(Debug, Clone, Copy)]
struct DurationUnit {
    plural: &'static str,
    singular: &'static str,
    display_name: &'static str,
    numeric_allowed: bool,
    two_digit_allowed: bool,
}

impl DurationUnit {
    fn allowed(self, style: UnitStyle) -> bool {
        match style {
            UnitStyle::Long | UnitStyle::Short | UnitStyle::Narrow => true,
            UnitStyle::Numeric => self.numeric_allowed,
            UnitStyle::TwoDigit => self.two_digit_allowed,
        }
    }
}

#[repr(usize)]
enum UnitIndex {
    Seconds = 6,
    Milliseconds = 7,
    Microseconds = 8,
    Nanoseconds = 9,
}

const DURATION_UNITS: [DurationUnit; 10] = [
    DurationUnit {
        plural: "years",
        singular: "year",
        display_name: "yearsDisplay",
        numeric_allowed: false,
        two_digit_allowed: false,
    },
    DurationUnit {
        plural: "months",
        singular: "month",
        display_name: "monthsDisplay",
        numeric_allowed: false,
        two_digit_allowed: false,
    },
    DurationUnit {
        plural: "weeks",
        singular: "week",
        display_name: "weeksDisplay",
        numeric_allowed: false,
        two_digit_allowed: false,
    },
    DurationUnit {
        plural: "days",
        singular: "day",
        display_name: "daysDisplay",
        numeric_allowed: false,
        two_digit_allowed: false,
    },
    DurationUnit {
        plural: "hours",
        singular: "hour",
        display_name: "hoursDisplay",
        numeric_allowed: true,
        two_digit_allowed: true,
    },
    DurationUnit {
        plural: "minutes",
        singular: "minute",
        display_name: "minutesDisplay",
        numeric_allowed: true,
        two_digit_allowed: true,
    },
    DurationUnit {
        plural: "seconds",
        singular: "second",
        display_name: "secondsDisplay",
        numeric_allowed: true,
        two_digit_allowed: true,
    },
    DurationUnit {
        plural: "milliseconds",
        singular: "millisecond",
        display_name: "millisecondsDisplay",
        numeric_allowed: true,
        two_digit_allowed: false,
    },
    DurationUnit {
        plural: "microseconds",
        singular: "microsecond",
        display_name: "microsecondsDisplay",
        numeric_allowed: true,
        two_digit_allowed: false,
    },
    DurationUnit {
        plural: "nanoseconds",
        singular: "nanosecond",
        display_name: "nanosecondsDisplay",
        numeric_allowed: true,
        two_digit_allowed: false,
    },
];
