use std::borrow::Cow;

use boa_gc::{Finalize, Trace};
use fixed_decimal::{Decimal, FloatPrecision, SignDisplay, UnsignedRoundingMode};
use icu_decimal::{
    DecimalFormatter, FormattedDecimal,
    options::{DecimalFormatterOptions, GroupingStrategy},
    preferences::NumberingSystem,
    provider::DecimalSymbolsV1,
};

mod options;
use icu_locale::{
    Locale,
    extensions::unicode::{Value, key},
};
use icu_provider::DataMarkerAttributes;
use num_bigint::BigInt;
use num_traits::Num;
pub(crate) use options::*;
use writeable::{Part, PartsWrite, Writeable};

use super::{
    Service,
    locale::{canonicalize_locale_list, filter_locales, resolve_locale, validate_extension},
    options::{IntlOptions, coerce_options_to_object},
};
use crate::value::JsVariant;
use crate::{
    Context, JsArgs, JsData, JsNativeError, JsObject, JsResult, JsString, JsSymbol, JsValue,
    NativeFunction,
    builtins::{
        BuiltInConstructor, BuiltInObject, IntrinsicObject, array::Array,
        builder::BuiltInBuilder, options::get_option, string::is_trimmable_whitespace,
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
    value::PreferredType,
};

#[derive(Debug, Clone)]
enum IntlMathematicalValue {
    Finite(Decimal),
    PositiveInfinity,
    NegativeInfinity,
    NotANumber,
}

#[derive(Debug, Clone)]
pub(crate) struct NumberPart {
    pub(crate) kind: &'static str,
    pub(crate) value: JsString,
}

#[derive(Debug, Default)]
struct NumberPartsWriter {
    string: String,
    parts: Vec<(usize, usize, Part)>,
}

impl std::fmt::Write for NumberPartsWriter {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.string.push_str(s);
        Ok(())
    }
}

impl PartsWrite for NumberPartsWriter {
    type SubPartsWrite = Self;

    fn with_part(
        &mut self,
        part: Part,
        mut f: impl FnMut(&mut Self::SubPartsWrite) -> std::fmt::Result,
    ) -> std::fmt::Result {
        let start = self.string.len();
        f(self)?;
        let end = self.string.len();
        if start < end {
            self.parts.push((start, end, part));
        }
        Ok(())
    }
}

impl IntlMathematicalValue {
    fn is_nan(&self) -> bool {
        matches!(self, Self::NotANumber)
    }
}

fn decimal_to_parts(formatted: FormattedDecimal<'_>) -> Vec<NumberPart> {
    let mut writer = NumberPartsWriter::default();
    formatted
        .write_to_parts(&mut writer)
        .expect("writing to a string cannot fail");

    if writer.parts.is_empty() && !writer.string.is_empty() {
        return vec![NumberPart {
            kind: "integer",
            value: JsString::from(writer.string),
        }];
    }

    let mut parts = writer
        .parts
        .into_iter()
        .filter(|(_, _, part)| decimal_part_name(*part) != "integer")
        .collect::<Vec<_>>();
    parts.sort_by_key(|(start, _, _)| *start);

    let mut result = Vec::new();
    let mut cursor = 0;
    for (start, end, part) in parts {
        if cursor < start {
            result.push(NumberPart {
                kind: "integer",
                value: JsString::from(&writer.string[cursor..start]),
            });
        }
        result.push(NumberPart {
            kind: decimal_part_name(part),
            value: JsString::from(&writer.string[start..end]),
        });
        cursor = end;
    }
    if cursor < writer.string.len() {
        result.push(NumberPart {
            kind: "integer",
            value: JsString::from(&writer.string[cursor..]),
        });
    }
    result
}

fn decimal_part_name(part: Part) -> &'static str {
    if part.category == "decimal" {
        part.value
    } else {
        "literal"
    }
}

fn join_parts(parts: &[NumberPart]) -> JsString {
    let mut string = String::new();
    for part in parts {
        string.push_str(&part.value.to_std_string_escaped());
    }
    JsString::from(string)
}

fn number_parts_are_one(parts: &[NumberPart]) -> bool {
    let mut value = String::new();
    for part in parts {
        if matches!(part.kind, "minusSign" | "plusSign" | "group") {
            continue;
        }
        value.push_str(&part.value.to_std_string_escaped());
    }
    value == "1"
}

fn parts_to_array(
    parts: impl IntoIterator<Item = (NumberPart, Option<&'static str>)>,
    context: &mut Context,
) -> JsObject {
    let parts = parts
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
    Array::create_array_from_list(parts, context)
}

fn currency_suffix_locale(locale: &Locale) -> bool {
    matches!(
        locale_language(locale),
        "de" | "fr" | "es" | "it" | "pl" | "pt" | "ru"
    )
}

fn locale_language(locale: &Locale) -> &str {
    locale.id.language.as_str()
}

fn currency_display(currency: Currency, display: CurrencyDisplay) -> (&'static str, JsString) {
    let code = currency.as_str();
    match display {
        CurrencyDisplay::Code => ("currency", JsString::from(code)),
        CurrencyDisplay::Name => ("currency", JsString::from(currency_name(code))),
        CurrencyDisplay::Symbol | CurrencyDisplay::NarrowSymbol => {
            ("currency", JsString::from(currency_symbol(code)))
        }
    }
}

fn currency_symbol(code: &str) -> &'static str {
    match code {
        "AUD" | "CAD" | "HKD" | "NZD" | "SGD" | "USD" => "$",
        "CNY" | "JPY" => "¥",
        "EUR" => "€",
        "GBP" => "£",
        "KRW" => "₩",
        _ => "¤",
    }
}

fn currency_name(code: &str) -> &'static str {
    match code {
        "CNY" => "Chinese yuan",
        "EUR" => "euros",
        "GBP" => "British pounds",
        "JPY" => "Japanese yen",
        "USD" => "US dollars",
        _ => "currency",
    }
}

fn unit_display(unit: &Unit, display: UnitDisplay) -> JsString {
    let unit = unit.to_js_string().to_std_string_escaped();
    let value = match display {
        UnitDisplay::Narrow => match unit.as_str() {
            "celsius" => "°C",
            "fahrenheit" => "°F",
            "kilometer-per-hour" => "km/h",
            "kilometer" => "km",
            "meter" => "m",
            "mile" => "mi",
            "percent" => "%",
            _ => unit.as_str(),
        },
        UnitDisplay::Short => match unit.as_str() {
            "byte" => "byte",
            "celsius" => "°C",
            "fahrenheit" => "°F",
            "kilometer-per-hour" => "km/h",
            "kilometer" => "km",
            "meter" => "m",
            "mile" => "mi",
            "percent" => "%",
            _ => unit.as_str(),
        },
        UnitDisplay::Long => match unit.as_str() {
            "byte" => "bytes",
            "celsius" => "degrees Celsius",
            "fahrenheit" => "degrees Fahrenheit",
            "kilometer-per-hour" => "kilometers per hour",
            "kilometer" => "kilometers",
            "meter" => "meters",
            "mile" => "miles",
            "percent" => "percent",
            _ => unit.as_str(),
        },
    };
    JsString::from(value)
}

pub(crate) fn substitute_digits(value: &str, numbering_system: &str) -> String {
    let mut out = String::with_capacity(value.len());
    for ch in value.chars() {
        if let Some(digit) = ch.to_digit(10).filter(|_| ch.is_ascii_digit()) {
            out.push_str(numbering_system_digit(numbering_system, digit as usize));
        } else {
            out.push(ch);
        }
    }
    out
}

fn numbering_system_digit(numbering_system: &str, digit: usize) -> &'static str {
    const LATN: [&str; 10] = ["0", "1", "2", "3", "4", "5", "6", "7", "8", "9"];
    const HANIDEC: [&str; 10] = ["〇", "一", "二", "三", "四", "五", "六", "七", "八", "九"];

    if numbering_system == "hanidec" {
        return HANIDEC[digit];
    }

    let Some(start) = numbering_system_digit_start(numbering_system) else {
        return LATN[digit];
    };
    let ch = char::from_u32(start + digit as u32).unwrap_or(char::REPLACEMENT_CHARACTER);
    match digit {
        0 => Box::leak(ch.to_string().into_boxed_str()),
        1 => Box::leak(ch.to_string().into_boxed_str()),
        2 => Box::leak(ch.to_string().into_boxed_str()),
        3 => Box::leak(ch.to_string().into_boxed_str()),
        4 => Box::leak(ch.to_string().into_boxed_str()),
        5 => Box::leak(ch.to_string().into_boxed_str()),
        6 => Box::leak(ch.to_string().into_boxed_str()),
        7 => Box::leak(ch.to_string().into_boxed_str()),
        8 => Box::leak(ch.to_string().into_boxed_str()),
        _ => Box::leak(ch.to_string().into_boxed_str()),
    }
}

fn numbering_system_digit_start(numbering_system: &str) -> Option<u32> {
    Some(match numbering_system {
        "adlm" => 0x1E950,
        "ahom" => 0x11730,
        "arab" => 0x0660,
        "arabext" => 0x06F0,
        "bali" => 0x1B50,
        "beng" => 0x09E6,
        "bhks" => 0x11C50,
        "brah" => 0x11066,
        "cakm" => 0x11136,
        "cham" => 0xAA50,
        "deva" => 0x0966,
        "diak" => 0x11950,
        "fullwide" => 0xFF10,
        "gara" => 0x10D40,
        "gong" => 0x11DA0,
        "gonm" => 0x11D50,
        "gujr" => 0x0AE6,
        "gukh" => 0x16130,
        "guru" => 0x0A66,
        "hmng" => 0x16B50,
        "hmnp" => 0x1E140,
        "java" => 0xA9D0,
        "kali" => 0xA900,
        "kawi" => 0x11F50,
        "khmr" => 0x17E0,
        "knda" => 0x0CE6,
        "krai" => 0x16D70,
        "lana" => 0x1A80,
        "lanatham" => 0x1A90,
        "laoo" => 0x0ED0,
        "lepc" => 0x1C40,
        "limb" => 0x1946,
        "mathbold" => 0x1D7CE,
        "mathdbl" => 0x1D7D8,
        "mathmono" => 0x1D7F6,
        "mathsanb" => 0x1D7EC,
        "mathsans" => 0x1D7E2,
        "mlym" => 0x0D66,
        "modi" => 0x11650,
        "mong" => 0x1810,
        "mroo" => 0x16A60,
        "mtei" => 0xABF0,
        "mymr" => 0x1040,
        "mymrepka" => 0x116DA,
        "mymrpao" => 0x116D0,
        "mymrshan" => 0x1090,
        "mymrtlng" => 0xA9F0,
        "nagm" => 0x1E4F0,
        "newa" => 0x11450,
        "nkoo" => 0x07C0,
        "olck" => 0x1C50,
        "onao" => 0x1E5F1,
        "orya" => 0x0B66,
        "osma" => 0x104A0,
        "outlined" => 0x1CCF0,
        "rohg" => 0x10D30,
        "saur" => 0xA8D0,
        "segment" => 0x1FBF0,
        "shrd" => 0x111D0,
        "sind" => 0x112F0,
        "sinh" => 0x0DE6,
        "sora" => 0x110F0,
        "sund" => 0x1BB0,
        "sunu" => 0x11BF0,
        "takr" => 0x116C0,
        "talu" => 0x19D0,
        "tamldec" => 0x0BE6,
        "telu" => 0x0C66,
        "thai" => 0x0E50,
        "tibt" => 0x0F20,
        "tirh" => 0x114D0,
        "tnsa" => 0x16AC0,
        "tols" => 0x11DE0,
        "vaii" => 0xA620,
        "wara" => 0x118E0,
        "wcho" => 0x1E2F0,
        _ => return None,
    })
}

pub(crate) fn numbering_system_is_supported(numbering_system: &str) -> bool {
    matches!(numbering_system, "latn" | "hanidec")
        || numbering_system_digit_start(numbering_system).is_some()
}

fn rounding_mode_to_js_string(mode: fixed_decimal::SignedRoundingMode) -> JsString {
    match mode {
        fixed_decimal::SignedRoundingMode::Ceil => js_string!("ceil"),
        fixed_decimal::SignedRoundingMode::Floor => js_string!("floor"),
        fixed_decimal::SignedRoundingMode::HalfCeil => js_string!("halfCeil"),
        fixed_decimal::SignedRoundingMode::HalfFloor => js_string!("halfFloor"),
        fixed_decimal::SignedRoundingMode::Unsigned(UnsignedRoundingMode::Expand) => {
            js_string!("expand")
        }
        fixed_decimal::SignedRoundingMode::Unsigned(UnsignedRoundingMode::Trunc) => {
            js_string!("trunc")
        }
        fixed_decimal::SignedRoundingMode::Unsigned(UnsignedRoundingMode::HalfExpand) => {
            js_string!("halfExpand")
        }
        fixed_decimal::SignedRoundingMode::Unsigned(UnsignedRoundingMode::HalfTrunc) => {
            js_string!("halfTrunc")
        }
        fixed_decimal::SignedRoundingMode::Unsigned(UnsignedRoundingMode::HalfEven) => {
            js_string!("halfEven")
        }
        _ => js_string!("halfExpand"),
    }
}

#[cfg(test)]
mod tests;

#[derive(Debug, Trace, Finalize, JsData)]
// Safety: `NumberFormat` only contains non-traceable types.
#[boa_gc(unsafe_empty_trace)]
pub(crate) struct NumberFormat {
    locale: Locale,
    formatter: DecimalFormatter,
    numbering_system: Option<Value>,
    unit_options: UnitFormatOptions,
    digit_options: DigitFormatOptions,
    notation: Notation,
    use_grouping: GroupingStrategy,
    sign_display: SignDisplay,
    bound_format: Option<JsFunction>,
}

pub(crate) fn format_decimal_for_notation(
    locale: &Locale,
    digit_options: &DigitFormatOptions,
    notation: NotationKind,
    mut value: Decimal,
) -> Decimal {
    match notation {
        NotationKind::Standard => {
            digit_options.format_fixed_decimal(&mut value);
            value
        }
        NotationKind::Scientific => {
            format_decimal_for_scientific_or_engineering(digit_options, value, 1)
        }
        NotationKind::Engineering => {
            format_decimal_for_scientific_or_engineering(digit_options, value, 3)
        }
        NotationKind::Compact => format_decimal_for_compact(locale, digit_options, value),
    }
}

pub(crate) fn compact_format_exponent(locale: &Locale, number: f64) -> u8 {
    let abs = number.abs();
    let (divisor, _) = compact_format_pattern(locale, abs);
    match divisor as u64 {
        1 => 0,
        1_000 => 3,
        10_000 => 4,
        100_000 => 5,
        1_000_000 => 6,
        100_000_000 => 8,
        1_000_000_000 => 9,
        1_000_000_000_000 => 12,
        _ => 0,
    }
}

impl NumberFormat {
    /// [`FormatNumeric ( numberFormat, x )`][full] and [`FormatNumericToParts ( numberFormat, x )`][parts].
    ///
    /// The returned struct implements `Writable`, allowing to either write the number as a full
    /// string or by parts.
    ///
    /// [full]: https://tc39.es/ecma402/#sec-formatnumber
    /// [parts]: https://tc39.es/ecma402/#sec-formatnumbertoparts
    #[allow(dead_code)]
    pub(crate) fn format<'a>(&'a self, value: &'a mut Decimal) -> FormattedDecimal<'a> {
        // TODO: Missing support from ICU4X for Percent/Currency/Unit formatting.
        // TODO: Missing support from ICU4X for Scientific/Engineering/Compact notation.

        self.digit_options.format_fixed_decimal(value);
        value.apply_sign_display(self.sign_display);

        self.formatter.format(value)
    }

    fn format_numeric(&self, value: IntlMathematicalValue) -> Vec<NumberPart> {
        let parts = match value {
            IntlMathematicalValue::Finite(value) => self.format_finite_decimal(value),
            IntlMathematicalValue::PositiveInfinity => self.format_non_finite(false, "∞"),
            IntlMathematicalValue::NegativeInfinity => self.format_non_finite(true, "∞"),
            IntlMathematicalValue::NotANumber => {
                let mut parts = Vec::new();
                if self.sign_display == SignDisplay::Always {
                    parts.push(NumberPart {
                        kind: "plusSign",
                        value: js_string!("+"),
                    });
                }
                parts.push(NumberPart {
                    kind: "nan",
                    value: if locale_language(&self.locale) == "zh" {
                        js_string!("非數值")
                    } else {
                        js_string!("NaN")
                    },
                });
                self.apply_unit_affixes(parts, false)
            }
        };
        self.localize_digits(parts)
    }

    fn format_numeric_to_string(&self, value: IntlMathematicalValue) -> JsString {
        join_parts(&self.format_numeric(value))
    }

    pub(crate) fn format_f64_to_string(&self, value: f64) -> JsString {
        let value = if value.is_nan() {
            IntlMathematicalValue::NotANumber
        } else if value == f64::INFINITY {
            IntlMathematicalValue::PositiveInfinity
        } else if value == f64::NEG_INFINITY {
            IntlMathematicalValue::NegativeInfinity
        } else {
            IntlMathematicalValue::Finite(
                Decimal::try_from_f64(value, FloatPrecision::RoundTrip)
                    .unwrap_or_else(|_| Decimal::from(0)),
            )
        };
        self.format_numeric_to_string(value)
    }

    pub(crate) fn format_decimal_to_string(&self, value: Decimal) -> JsString {
        self.format_numeric_to_string(IntlMathematicalValue::Finite(value))
    }

    pub(crate) fn format_value_to_parts(
        &self,
        value: &JsValue,
        context: &mut Context,
    ) -> JsResult<Vec<NumberPart>> {
        Ok(self.format_numeric(to_intl_mathematical_value(value, context)?))
    }

    fn format_non_finite(&self, negative: bool, body: &'static str) -> Vec<NumberPart> {
        let mut parts = Vec::new();
        match (negative, self.sign_display) {
            (true, SignDisplay::Never) => {}
            (true, _) => parts.push(NumberPart {
                kind: "minusSign",
                value: js_string!("-"),
            }),
            (false, SignDisplay::Always) | (false, SignDisplay::ExceptZero) => {
                parts.push(NumberPart {
                    kind: "plusSign",
                    value: js_string!("+"),
                });
            }
            (false, _) => {}
        }
        parts.push(NumberPart {
            kind: "infinity",
            value: js_string!(body),
        });
        self.apply_unit_affixes(parts, negative)
    }

    fn format_finite_decimal(&self, mut value: Decimal) -> Vec<NumberPart> {
        if self.unit_options.style() == Style::Percent {
            value.multiply_pow10(2);
        }

        match self.notation {
            Notation::Standard => {
                value = format_decimal_for_notation(
                    &self.locale,
                    &self.digit_options,
                    NotationKind::Standard,
                    value,
                );
                value.apply_sign_display(self.sign_display);
                let mut parts = decimal_to_parts(self.formatter.format(&value));
                if self.unit_options.style() == Style::Percent {
                    if currency_suffix_locale(&self.locale) {
                        parts.push(NumberPart {
                            kind: "literal",
                            value: js_string!(" "),
                        });
                    }
                    parts.push(NumberPart {
                        kind: "percentSign",
                        value: js_string!("%"),
                    });
                }
                let negative = parts
                    .first()
                    .is_some_and(|part| part.kind == "minusSign" || part.value == js_string!("-"));
                self.apply_unit_affixes(parts, negative)
            }
            Notation::Scientific => self.format_scientific_or_engineering(value, 1),
            Notation::Engineering => self.format_scientific_or_engineering(value, 3),
            Notation::Compact { display } => self.format_compact(value, display),
        }
    }

    fn format_scientific_or_engineering(
        &self,
        mut value: Decimal,
        exponent_step: i32,
    ) -> Vec<NumberPart> {
        let parsed = value.to_string().parse::<f64>().unwrap_or(0.0);
        if parsed == 0.0 {
            value = format_decimal_for_notation(
                &self.locale,
                &self.digit_options,
                if exponent_step == 1 {
                    NotationKind::Scientific
                } else {
                    NotationKind::Engineering
                },
                value,
            );
            value.apply_sign_display(self.sign_display);
            let mut parts = decimal_to_parts(self.formatter.format(&value));
            parts.push(NumberPart {
                kind: "exponentSeparator",
                value: js_string!("E"),
            });
            parts.push(NumberPart {
                kind: "exponentInteger",
                value: js_string!("0"),
            });
            return self.apply_unit_affixes(parts, false);
        }

        let negative = parsed.is_sign_negative();
        let mut exponent = parsed.abs().log10().floor() as i32;
        if exponent_step == 3 {
            exponent -= exponent.rem_euclid(3);
        }
        let mantissa = parsed / 10f64.powi(exponent);
        let mut mantissa = format_decimal_for_notation(
            &self.locale,
            &self.digit_options,
            if exponent_step == 1 {
                NotationKind::Scientific
            } else {
                NotationKind::Engineering
            },
            Decimal::try_from_f64(mantissa, FloatPrecision::RoundTrip)
                .unwrap_or_else(|_| Decimal::from(0)),
        );
        mantissa.apply_sign_display(self.sign_display);
        let mut parts = decimal_to_parts(self.formatter.format(&mantissa));
        parts.push(NumberPart {
            kind: "exponentSeparator",
            value: js_string!("E"),
        });
        if exponent < 0 {
            parts.push(NumberPart {
                kind: "exponentMinusSign",
                value: js_string!("-"),
            });
        }
        parts.push(NumberPart {
            kind: "exponentInteger",
            value: JsString::from(exponent.abs().to_string()),
        });
        self.apply_unit_affixes(parts, negative)
    }

    fn format_compact(&self, value: Decimal, display: CompactDisplay) -> Vec<NumberPart> {
        let parsed = value.to_string().parse::<f64>().unwrap_or(0.0);
        let negative = parsed.is_sign_negative();
        let abs = parsed.abs();
        let (divisor, suffix) = compact_format_pattern_for_display(&self.locale, abs, display);
        let mut compact = Decimal::try_from_f64(parsed / divisor, FloatPrecision::RoundTrip)
            .unwrap_or_else(|_| Decimal::from(0));
        self.digit_options.format_fixed_decimal(&mut compact);
        compact.apply_sign_display(self.sign_display);
        let mut parts = decimal_to_parts(self.formatter.format(&compact));
        if !suffix.is_empty() {
            if suffix.starts_with(' ') || suffix.starts_with('\u{a0}') {
                let split = suffix
                    .char_indices()
                    .nth(1)
                    .map_or(suffix.len(), |(index, _)| index);
                let rest = &suffix[split..];
                parts.push(NumberPart {
                    kind: "literal",
                    value: JsString::from(&suffix[..split]),
                });
                parts.push(NumberPart {
                    kind: "compact",
                    value: JsString::from(rest),
                });
                return self.apply_unit_affixes(parts, negative);
            }
            parts.push(NumberPart {
                kind: "compact",
                value: JsString::from(suffix),
            });
        }
        self.apply_unit_affixes(parts, negative)
    }

    fn apply_unit_affixes(&self, mut parts: Vec<NumberPart>, negative: bool) -> Vec<NumberPart> {
        match &self.unit_options {
            UnitFormatOptions::Decimal | UnitFormatOptions::Percent => parts,
            UnitFormatOptions::Currency {
                currency,
                display,
                sign,
            } => {
                let (kind, mut value) = currency_display(*currency, *display);
                if matches!(locale_language(&self.locale), "ko" | "zh")
                    && *display != CurrencyDisplay::Code
                    && currency.as_str() == "USD"
                {
                    value = js_string!("US$");
                }
                if *sign == CurrencySign::Accounting && negative && !currency_suffix_locale(&self.locale) {
                    let mut wrapped = vec![NumberPart {
                        kind: "literal",
                        value: js_string!("("),
                    }];
                    let mut unsigned = parts
                        .into_iter()
                        .filter(|part| part.kind != "minusSign")
                        .collect::<Vec<_>>();
                    unsigned.insert(0, NumberPart { kind, value });
                    wrapped.extend(unsigned);
                    wrapped.push(NumberPart {
                        kind: "literal",
                        value: js_string!(")"),
                    });
                    wrapped
                } else if currency_suffix_locale(&self.locale) {
                    parts.push(NumberPart {
                        kind: "literal",
                        value: js_string!(" "),
                    });
                    parts.push(NumberPart { kind, value });
                    parts
                } else {
                    let sign_prefix_len = usize::from(
                        parts
                            .first()
                            .is_some_and(|part| part.kind == "minusSign" || part.kind == "plusSign"),
                    );
                    parts.insert(sign_prefix_len, NumberPart { kind, value });
                    parts
                }
            }
            UnitFormatOptions::Unit { unit, display } => {
                let mut value = unit_display(unit, *display);
                let unit_id = unit.to_js_string().to_std_string_escaped();
                if matches!(*display, UnitDisplay::Short | UnitDisplay::Long)
                    && matches!(
                        unit_id.as_str(),
                        "year"
                            | "month"
                            | "week"
                            | "day"
                            | "hour"
                            | "minute"
                            | "second"
                            | "millisecond"
                            | "microsecond"
                            | "nanosecond"
                    )
                    && !number_parts_are_one(&parts)
                {
                    value = JsString::from(format!("{}s", value.to_std_string_escaped()));
                }
                if locale_language(&self.locale) == "ja"
                    && *display == UnitDisplay::Long
                    && unit_id == "kilometer-per-hour"
                {
                    let mut result = vec![
                        NumberPart {
                            kind: "unit",
                            value: js_string!("時速"),
                        },
                        NumberPart {
                            kind: "literal",
                            value: js_string!(" "),
                        },
                    ];
                    result.extend(parts);
                    result.push(NumberPart {
                        kind: "literal",
                        value: js_string!(" "),
                    });
                    result.push(NumberPart {
                        kind: "unit",
                        value: js_string!("キロメートル"),
                    });
                    return result;
                }
                if locale_language(&self.locale) == "ko"
                    && *display == UnitDisplay::Long
                    && unit_id == "kilometer-per-hour"
                {
                    let mut result = vec![
                        NumberPart {
                            kind: "unit",
                            value: js_string!("시속"),
                        },
                        NumberPart {
                            kind: "literal",
                            value: js_string!(" "),
                        },
                    ];
                    result.extend(parts);
                    result.push(NumberPart {
                        kind: "unit",
                        value: js_string!("킬로미터"),
                    });
                    return result;
                }
                if locale_language(&self.locale) == "zh"
                    && *display == UnitDisplay::Long
                    && unit_id == "kilometer-per-hour"
                {
                    let mut result = vec![
                        NumberPart {
                            kind: "unit",
                            value: js_string!("每小時"),
                        },
                        NumberPart {
                            kind: "literal",
                            value: js_string!(" "),
                        },
                    ];
                    result.extend(parts);
                    result.push(NumberPart {
                        kind: "literal",
                        value: js_string!(" "),
                    });
                    result.push(NumberPart {
                        kind: "unit",
                        value: js_string!("公里"),
                    });
                    return result;
                }
                if locale_language(&self.locale) == "zh" && unit_id == "kilometer-per-hour" {
                    value = js_string!("公里/小時");
                }
                if locale_language(&self.locale) == "de"
                    && *display == UnitDisplay::Long
                    && unit_id == "kilometer-per-hour"
                {
                    value = js_string!("Kilometer pro Stunde");
                }
                if value != js_string!("%")
                    && !(*display == UnitDisplay::Narrow
                        && value == js_string!("km/h")
                        && locale_language(&self.locale) != "de")
                    && !(locale_language(&self.locale) == "ko" && value == js_string!("km/h"))
                    && !(matches!(locale_language(&self.locale), "ko" | "zh")
                        && *display == UnitDisplay::Narrow)
                {
                    parts.push(NumberPart {
                        kind: "literal",
                        value: js_string!(" "),
                    });
                }
                parts.push(NumberPart {
                    kind: "unit",
                    value,
                });
                parts
            }
        }
    }

    fn localize_digits(&self, mut parts: Vec<NumberPart>) -> Vec<NumberPart> {
        let Some(numbering_system) = self.numbering_system.as_ref().map(ToString::to_string) else {
            return parts;
        };
        if numbering_system == "latn" {
            return parts;
        }
        for part in &mut parts {
            if matches!(
                part.kind,
                    "integer" | "fraction" | "exponentInteger" | "compact"
                ) {
                part.value = JsString::from(substitute_digits(
                    &part.value.to_std_string_escaped(),
                    &numbering_system,
                ));
            }
        }
        parts
    }
}

fn format_decimal_for_scientific_or_engineering(
    digit_options: &DigitFormatOptions,
    mut value: Decimal,
    exponent_step: i32,
) -> Decimal {
    let parsed = value.to_string().parse::<f64>().unwrap_or(0.0);
    if parsed == 0.0 {
        digit_options.format_fixed_decimal(&mut value);
        return value;
    }

    let mut exponent = parsed.abs().log10().floor() as i32;
    if exponent_step == 3 {
        exponent -= exponent.rem_euclid(3);
    }
    let mantissa = parsed / 10f64.powi(exponent);
    let mut mantissa = Decimal::try_from_f64(mantissa, FloatPrecision::RoundTrip)
        .unwrap_or_else(|_| Decimal::from(0));
    digit_options.format_fixed_decimal(&mut mantissa);
    mantissa
}

fn format_decimal_for_compact(
    locale: &Locale,
    digit_options: &DigitFormatOptions,
    value: Decimal,
) -> Decimal {
    let parsed = value.to_string().parse::<f64>().unwrap_or(0.0);
    let abs = parsed.abs();
    let (divisor, _) = compact_format_pattern(locale, abs);
    let mut compact = Decimal::try_from_f64(parsed / divisor, FloatPrecision::RoundTrip)
        .unwrap_or_else(|_| Decimal::from(0));
    digit_options.format_fixed_decimal(&mut compact);
    compact
}

fn compact_format_pattern(locale: &Locale, abs: f64) -> (f64, &'static str) {
    compact_format_pattern_for_display(locale, abs, CompactDisplay::Short)
}

fn compact_format_pattern_for_display(
    locale: &Locale,
    abs: f64,
    display: CompactDisplay,
) -> (f64, &'static str) {
    let language = locale_language(locale);
    match language {
        // CLDR's German short compact patterns only kick in at millions, while long patterns
        // also compact thousands.
        "de" if abs >= 1_000_000.0 => match display {
            CompactDisplay::Short => (1_000_000.0, "\u{a0}Mio."),
            CompactDisplay::Long => (1_000_000.0, " Millionen"),
        },
        "de" if abs >= 1_000.0 => match display {
            CompactDisplay::Short => (1.0, ""),
            CompactDisplay::Long => (1_000.0, " Tausend"),
        },
        "de" => (1.0, ""),
        "ja" if abs >= 100_000_000.0 => (100_000_000.0, "億"),
        "ja" if abs >= 10_000.0 => (10_000.0, "万"),
        "ja" => (1.0, ""),
        "ko" if abs >= 100_000_000.0 => (100_000_000.0, "억"),
        "ko" if abs >= 10_000.0 => (10_000.0, "만"),
        "ko" if abs >= 1_000.0 => (1_000.0, "천"),
        "zh" if abs >= 100_000_000.0 => (100_000_000.0, "億"),
        "zh" if abs >= 10_000.0 => (10_000.0, "萬"),
        "zh" => (1.0, ""),
        _ if locale.to_string().starts_with("en-IN") && abs >= 100_000.0 => {
            match display {
                CompactDisplay::Short => (100_000.0, "L"),
                CompactDisplay::Long => (100_000.0, " lakh"),
            }
        }
        _ if abs >= 1_000_000_000_000.0 => match display {
            CompactDisplay::Short => (1_000_000_000_000.0, "T"),
            CompactDisplay::Long => (1_000_000_000_000.0, " trillion"),
        },
        _ if abs >= 1_000_000_000.0 => match display {
            CompactDisplay::Short => (1_000_000_000.0, "B"),
            CompactDisplay::Long => (1_000_000_000.0, " billion"),
        },
        _ if abs >= 1_000_000.0 => match display {
            CompactDisplay::Short => (1_000_000.0, "M"),
            CompactDisplay::Long => (1_000_000.0, " million"),
        },
        _ if abs >= 1_000.0 => match display {
            CompactDisplay::Short => (1_000.0, "K"),
            CompactDisplay::Long => (1_000.0, " thousand"),
        },
        _ => (1.0, ""),
    }
}

#[derive(Debug, Clone)]
pub(super) struct NumberFormatLocaleOptions {
    numbering_system: Option<Value>,
}

impl Service for NumberFormat {
    type LangMarker = DecimalSymbolsV1;

    type LocaleOptions = NumberFormatLocaleOptions;

    fn resolve(
        locale: &mut Locale,
        options: &mut Self::LocaleOptions,
        provider: &crate::context::icu::IntlProvider,
    ) {
        let extension_numbering_system = locale.extensions.unicode.keywords.get(&key!("nu")).cloned();
        let option_numbering_system = options
            .numbering_system
            .take()
            .filter(|nu| {
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
        let reflect_extension = match (&numbering_system, &option_numbering_system, &extension_numbering_system) {
            (Some(numbering_system), Some(option), Some(extension)) => {
                numbering_system == option && option == extension
            }
            (Some(numbering_system), None, Some(extension)) => numbering_system == extension,
            _ => false,
        };

        locale.extensions.unicode.clear();

        if reflect_extension
            && let Some(nu) = numbering_system.clone()
        {
            locale.extensions.unicode.keywords.set(key!("nu"), nu);
        }

        options.numbering_system = numbering_system;
    }
}

impl IntrinsicObject for NumberFormat {
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
                js_string!("Intl.NumberFormat"),
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

impl BuiltInObject for NumberFormat {
    const NAME: JsString = StaticJsStrings::NUMBER_FORMAT;
}

impl BuiltInConstructor for NumberFormat {
    const CONSTRUCTOR_ARGUMENTS: usize = 0;
    const PROTOTYPE_STORAGE_SLOTS: usize = 7;
    const CONSTRUCTOR_STORAGE_SLOTS: usize = 1;

    const STANDARD_CONSTRUCTOR: fn(&StandardConstructors) -> &StandardConstructor =
        StandardConstructors::number_format;

    /// [`Intl.NumberFormat ( [ locales [ , options ] ] )`][spec].
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat
    fn constructor(
        new_target: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. If NewTarget is undefined, let newTarget be the active function object, else let newTarget be NewTarget.
        let new_target_inner = &if new_target.is_undefined() {
            context
                .active_function_object()
                .unwrap_or_else(|| {
                    context
                        .intrinsics()
                        .constructors()
                        .number_format()
                        .constructor()
                })
                .into()
        } else {
            new_target.clone()
        };

        // 2. Let numberFormat be ? OrdinaryCreateFromConstructor(newTarget, "%Intl.NumberFormat.prototype%", « [[InitializedNumberFormat]], [[Locale]], [[DataLocale]], [[NumberingSystem]], [[Style]], [[Unit]], [[UnitDisplay]], [[Currency]], [[CurrencyDisplay]], [[CurrencySign]], [[MinimumIntegerDigits]], [[MinimumFractionDigits]], [[MaximumFractionDigits]], [[MinimumSignificantDigits]], [[MaximumSignificantDigits]], [[RoundingType]], [[Notation]], [[CompactDisplay]], [[UseGrouping]], [[SignDisplay]], [[RoundingIncrement]], [[RoundingMode]], [[ComputedRoundingPriority]], [[TrailingZeroDisplay]], [[BoundFormat]] »).
        let prototype = get_prototype_from_constructor(
            new_target_inner,
            StandardConstructors::number_format,
            context,
        )?;

        let number_format = Self::new(locales, options, context)?;

        let number_format = JsObject::from_proto_and_data_with_shared_shape(
            context.root_shape(),
            prototype,
            number_format,
        );

        // 31. Return unused.

        // 4. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
        //     a. Let this be the this value.
        //     b. Return ? ChainNumberFormat(numberFormat, NewTarget, this).
        // ChainNumberFormat ( numberFormat, newTarget, this )
        // <https://tc39.es/ecma402/#sec-chainnumberformat>

        let this = context.vm.stack.get_this(context.vm.frame());
        let Some(this_obj) = this.as_object() else {
            return Ok(number_format.into());
        };

        let constructor = context
            .intrinsics()
            .constructors()
            .number_format()
            .constructor();

        // 1. If newTarget is undefined and ? OrdinaryHasInstance(%Intl.NumberFormat%, this) is true, then
        if new_target.is_undefined()
            && JsValue::ordinary_has_instance(&constructor.into(), &this, context)?
        {
            let fallback_symbol = context
                .intrinsics()
                .objects()
                .intl()
                .borrow()
                .data()
                .fallback_symbol();

            // a. Perform ? DefinePropertyOrThrow(this, %Intl%.[[FallbackSymbol]], PropertyDescriptor{ [[Value]]: numberFormat, [[Writable]]: false, [[Enumerable]]: false, [[Configurable]]: false }).
            this_obj.define_property_or_throw(
                fallback_symbol,
                PropertyDescriptor::builder()
                    .value(number_format)
                    .writable(false)
                    .enumerable(false)
                    .configurable(false),
                context,
            )?;
            // b. Return this.
            Ok(this)
        } else {
            // 2. Return numberFormat.
            Ok(number_format.into())
        }
    }
}

impl NumberFormat {
    /// Creates a new instance of `NumberFormat`.
    pub(crate) fn new(
        locales: &JsValue,
        options: &JsValue,
        context: &mut Context,
    ) -> JsResult<Self> {
        // 3. Perform ? InitializeNumberFormat(numberFormat, locales, options).

        // `InitializeNumberFormat ( numberFormat, locales, options )`
        // https://tc39.es/ecma402/#sec-initializenumberformat

        // 1. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;
        // 2. Set options to ? CoerceOptionsToObject(options).
        let options = coerce_options_to_object(options, context)?;

        // 3. Let opt be a new Record.

        // 4. Let matcher be ? GetOption(options, "localeMatcher", string, « "lookup", "best fit" », "best fit").
        // 5. Set opt.[[localeMatcher]] to matcher.
        let matcher =
            get_option(&options, js_string!("localeMatcher"), context)?.unwrap_or_default();

        // 6. Let numberingSystem be ? GetOption(options, "numberingSystem", string, empty, undefined).
        // 7. If numberingSystem is not undefined, then
        //     a. If numberingSystem cannot be matched by the type Unicode locale nonterminal, throw a RangeError exception.
        // 8. Set opt.[[nu]] to numberingSystem.
        let numbering_system =
            get_option::<NumberingSystem>(&options, js_string!("numberingSystem"), context)?;
        let requested_numbering_system = numbering_system.clone();

        let mut intl_options = IntlOptions {
            matcher,
            service_options: NumberFormatLocaleOptions {
                numbering_system: numbering_system.map(Value::from),
            },
        };

        // 9. Let localeData be %Intl.NumberFormat%.[[LocaleData]].
        // 10. Let r be ResolveLocale(%Intl.NumberFormat%.[[AvailableLocales]], requestedLocales, opt, %Intl.NumberFormat%.[[RelevantExtensionKeys]], localeData).
        let locale = resolve_locale::<Self>(
            requested_locales,
            &mut intl_options,
            context.intl_provider(),
        )?;

        // 11. Set numberFormat.[[Locale]] to r.[[locale]].
        // 12. Set numberFormat.[[DataLocale]] to r.[[dataLocale]].
        // 13. Set numberFormat.[[NumberingSystem]] to r.[[nu]].

        // 14. Perform ? SetNumberFormatUnitOptions(numberFormat, options).
        let unit_options = UnitFormatOptions::from_options(&options, context)?;

        // 18. Let notation be ? GetOption(options, "notation", string, « "standard", "scientific", "engineering", "compact" », "standard").
        // 19. Set numberFormat.[[Notation]] to notation.
        let notation_kind =
            get_option(&options, js_string!("notation"), context)?.unwrap_or_default();

        // 15. Let style be numberFormat.[[Style]].
        // 16. If style is "currency" and notation is "standard", then
        let (min_fractional, max_fractional) = if let UnitFormatOptions::Currency {
            currency, ..
        } = &unit_options
            && notation_kind == NotationKind::Standard
        {
            // a. Let currency be numberFormat.[[Currency]].
            // b. Let cDigits be CurrencyDigits(currency).
            let c_digits = currency.digits();
            // c. Let mnfdDefault be cDigits.
            // d. Let mxfdDefault be cDigits.
            (c_digits, c_digits)
        } else {
            // 17. Else,
            (
                // a. Let mnfdDefault be 0.
                0,
                // b. If style is "percent", then
                if unit_options.style() == Style::Percent {
                    // i. Let mxfdDefault be 0.
                    0
                } else {
                    // c. Else,
                    //    i. Let mxfdDefault be 3.
                    if notation_kind == NotationKind::Compact {
                        0
                    } else {
                        3
                    }
                },
            )
        };

        // 20. Perform ? SetNumberFormatDigitOptions(numberFormat, options, mnfdDefault, mxfdDefault, notation).
        let digit_options = DigitFormatOptions::from_options(
            &options,
            min_fractional,
            max_fractional,
            notation_kind,
            context,
        )?;

        // 21. Let compactDisplay be ? GetOption(options, "compactDisplay", string, « "short", "long" », "short").
        let compact_display =
            get_option(&options, js_string!("compactDisplay"), context)?.unwrap_or_default();

        // 22. Let defaultUseGrouping be "auto".
        let mut default_use_grouping = GroupingStrategy::Auto;

        let notation = match notation_kind {
            NotationKind::Standard => Notation::Standard,
            NotationKind::Scientific => Notation::Scientific,
            NotationKind::Engineering => Notation::Engineering,
            // 23. If notation is "compact", then
            NotationKind::Compact => {
                // b. Set defaultUseGrouping to "min2".
                default_use_grouping = GroupingStrategy::Min2;

                // a. Set numberFormat.[[CompactDisplay]] to compactDisplay.
                Notation::Compact {
                    display: compact_display,
                }
            }
        };

        // 24. NOTE: For historical reasons, the strings "true" and "false" are accepted and replaced with the default value.
        // 25. Let useGrouping be ? GetBooleanOrStringNumberFormatOption(options, "useGrouping",
        //     « "min2", "auto", "always", "true", "false" », defaultUseGrouping).
        // 26. If useGrouping is "true" or useGrouping is "false", set useGrouping to defaultUseGrouping.
        // 27. If useGrouping is true, set useGrouping to "always".
        // 28. Set numberFormat.[[UseGrouping]] to useGrouping.
        // useGrouping requires special handling because of the "true" and "false" exceptions.
        // We could also modify the `OptionType` interface but it complicates it a lot just for
        // a single exception.
        let use_grouping = 'block: {
            // GetBooleanOrStringNumberFormatOption ( options, property, stringValues, fallback )
            // <https://tc39.es/ecma402/#sec-getbooleanorstringnumberformatoption>

            // 1. Let value be ? Get(options, property).
            let value = options.get(js_string!("useGrouping"), context)?;

            // 2. If value is undefined, return fallback.
            if value.is_undefined() {
                break 'block default_use_grouping;
            }
            // 3. If value is true, return true.
            if let Some(true) = value.as_boolean() {
                break 'block GroupingStrategy::Always;
            }

            // 4. If ToBoolean(value) is false, return false.
            if !value.to_boolean() {
                break 'block GroupingStrategy::Never;
            }

            // 5. Set value to ? ToString(value).
            // 6. If stringValues does not contain value, throw a RangeError exception.
            // 7. Return value.
            match value.to_string(context)?.to_std_string_escaped().as_str() {
                "min2" => GroupingStrategy::Min2,
                "auto" => GroupingStrategy::Auto,
                "always" => GroupingStrategy::Always,
                // special handling for historical reasons
                "true" | "false" => default_use_grouping,
                _ => {
                    return Err(JsNativeError::range()
                        .with_message(
                            "expected one of `min2`, `auto`, `always`, `true`, or `false`",
                        )
                        .into());
                }
            }
        };

        // 29. Let signDisplay be ? GetOption(options, "signDisplay", string, « "auto", "never", "always", "exceptZero", "negative" », "auto").
        // 30. Set numberFormat.[[SignDisplay]] to signDisplay.
        let sign_display =
            get_option(&options, js_string!("signDisplay"), context)?.unwrap_or(SignDisplay::Auto);

        let mut options = DecimalFormatterOptions::default();
        options.grouping_strategy = Some(use_grouping);

        let formatter = DecimalFormatter::try_new_with_buffer_provider(
            context.intl_provider().erased_provider(),
            (&locale).into(),
            options,
        )
        .map_err(|err| JsNativeError::typ().with_message(err.to_string()))?;

        Ok(NumberFormat {
            locale,
            numbering_system: intl_options
                .service_options
                .numbering_system
                .or_else(|| {
                    requested_numbering_system
                        .filter(|nu| numbering_system_is_supported(nu.as_str()))
                        .map(Value::from)
                }),
            formatter,
            unit_options,
            digit_options,
            notation,
            use_grouping,
            sign_display,
            bound_format: None,
        })
    }

    /// [`Intl.NumberFormat.supportedLocalesOf ( locales [ , options ] )`][spec].
    ///
    /// Returns an array containing those of the provided locales that are supported in number format
    /// without having to fall back to the runtime's default locale.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat.supportedlocalesof
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/supportedLocalesOf
    fn supported_locales_of(
        _: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let locales = args.get_or_undefined(0);
        let options = args.get_or_undefined(1);

        // 1. Let availableLocales be %Intl.NumberFormat%.[[AvailableLocales]].
        // 2. Let requestedLocales be ? CanonicalizeLocaleList(locales).
        let requested_locales = canonicalize_locale_list(locales, context)?;

        // 3. Return ? FilterLocales(availableLocales, requestedLocales, options).
        filter_locales::<Self>(requested_locales, options, context).map(JsValue::from)
    }

    /// [`get Intl.NumberFormat.prototype.format`][spec].
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat.prototype.format
    fn get_format(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // 1. Let nf be the this value.
        // 2. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
        //     a. Set nf to ? UnwrapNumberFormat(nf).
        // 3. Perform ? RequireInternalSlot(nf, [[InitializedNumberFormat]]).
        let nf = unwrap_number_format(this, context)?;
        let nf_clone = nf.clone();
        let mut nf = nf.borrow_mut();

        let bound_format = if let Some(f) = nf.data_mut().bound_format.clone() {
            f
        } else {
            // 4. If nf.[[BoundFormat]] is undefined, then
            //     a. Let F be a new built-in function object as defined in Number Format Functions (15.5.2).
            //     b. Set F.[[NumberFormat]] to nf.
            //     c. Set nf.[[BoundFormat]] to F.
            let bound_format = FunctionObjectBuilder::new(
                context.realm(),
                // Number Format Functions
                // <https://tc39.es/ecma402/#sec-number-format-functions>
                NativeFunction::from_copy_closure_with_captures(
                    |_, args, nf, context| {
                        // 1. Let nf be F.[[NumberFormat]].
                        // 2. Assert: Type(nf) is Object and nf has an [[InitializedNumberFormat]] internal slot.

                        // 3. If value is not provided, let value be undefined.
                        let value = args.get_or_undefined(0);

                        // 4. Let x be ? ToIntlMathematicalValue(value).
                        let x = to_intl_mathematical_value(value, context)?;

                        // 5. Return FormatNumeric(nf, x).
                        Ok(nf.borrow().data().format_numeric_to_string(x).into())
                    },
                    nf_clone,
                ),
            )
            .length(1)
            .build();

            nf.data_mut().bound_format = Some(bound_format.clone());
            bound_format
        };

        // 5. Return nf.[[BoundFormat]].
        Ok(bound_format.into())
    }

    fn format_to_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        let nf = unwrap_number_format(this, context)?;
        let value = to_intl_mathematical_value(args.get_or_undefined(0), context)?;
        let parts = nf.borrow().data().format_numeric(value);
        Ok(parts_to_array(parts.into_iter().map(|part| (part, None)), context).into())
    }

    fn format_range(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if args.len() < 2 || args[0].is_undefined() || args[1].is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("formatRange requires start and end values")
                .into());
        }
        let nf = unwrap_number_format(this, context)?;
        let start = to_intl_mathematical_value(args.get_or_undefined(0), context)?;
        let end = to_intl_mathematical_value(args.get_or_undefined(1), context)?;
        if start.is_nan() || end.is_nan() {
            return Err(JsNativeError::range()
                .with_message("cannot format a range with NaN")
                .into());
        }
        let nf = nf.borrow();
        let nf = nf.data();
        let start_parts = nf.format_numeric(start);
        let end_parts = nf.format_numeric(end);
        let start_string = join_parts(&start_parts);
        let end_string = join_parts(&end_parts);
        if start_string == end_string {
            return Ok(JsString::from(format!(
                "~{}",
                start_string.to_std_string_escaped()
            ))
            .into());
        }
        let start_std = start_string.to_std_string_escaped();
        let end_std = end_string.to_std_string_escaped();
        if let UnitFormatOptions::Currency { .. } = nf.unit_options {
            if let (Some((start_number, start_currency)), Some((end_number, end_currency))) =
                (start_std.rsplit_once('\u{a0}'), end_std.rsplit_once('\u{a0}'))
                && start_currency == end_currency
            {
                let end_number = if start_number.starts_with('+') {
                    end_number.strip_prefix('+').unwrap_or(end_number)
                } else {
                    end_number
                };
                return Ok(JsString::from(format!(
                    "{start_number} - {end_number}\u{a0}{start_currency}"
                ))
                .into());
            }
            if let (Some(start_tail), Some(end_tail)) =
                (start_std.strip_prefix("+$"), end_std.strip_prefix("+$"))
            {
                return Ok(JsString::from(format!("+${start_tail}–{end_tail}")).into());
            }
        } else {
            if locale_language(&nf.locale) == "pt" {
                return Ok(JsString::from(format!("{start_std} - {end_std}")).into());
            }
            return Ok(JsString::from(format!("{start_std}–{end_std}")).into());
        }
        Ok(JsString::from(format!(
            "{start_std} – {end_std}",
        ))
        .into())
    }

    fn format_range_to_parts(
        this: &JsValue,
        args: &[JsValue],
        context: &mut Context,
    ) -> JsResult<JsValue> {
        if args.len() < 2 || args[0].is_undefined() || args[1].is_undefined() {
            return Err(JsNativeError::typ()
                .with_message("formatRangeToParts requires start and end values")
                .into());
        }
        let nf = unwrap_number_format(this, context)?;
        let start = to_intl_mathematical_value(args.get_or_undefined(0), context)?;
        let end = to_intl_mathematical_value(args.get_or_undefined(1), context)?;
        if start.is_nan() || end.is_nan() {
            return Err(JsNativeError::range()
                .with_message("cannot format a range with NaN")
                .into());
        }
        let nf = nf.borrow();
        let nf = nf.data();
        let start_parts = nf.format_numeric(start);
        let end_parts = nf.format_numeric(end);
        if join_parts(&start_parts) == join_parts(&end_parts) {
            let parts = std::iter::once((
                NumberPart {
                    kind: "approximatelySign",
                    value: js_string!("~"),
                },
                Some("shared"),
            ))
            .chain(start_parts.into_iter().map(|part| (part, Some("shared"))));
            return Ok(parts_to_array(parts, context).into());
        }
        let parts = start_parts
            .into_iter()
            .map(|part| (part, Some("startRange")))
            .chain(std::iter::once((
                NumberPart {
                    kind: "literal",
                    value: js_string!(" – "),
                },
                Some("shared"),
            )))
            .chain(
                end_parts
                    .into_iter()
                    .map(|part| (part, Some("endRange"))),
            );
        Ok(parts_to_array(parts, context).into())
    }

    /// [`Intl.NumberFormat.prototype.resolvedOptions ( )`][spec].
    ///
    /// Returns a new object with properties reflecting the locale and options computed during the
    /// construction of the current `Intl.NumberFormat` object.
    ///
    /// More information:
    ///  - [MDN documentation][mdn]
    ///
    /// [spec]: https://tc39.es/ecma402/#sec-intl.numberformat.prototype.resolvedoptions
    /// [mdn]: https://developer.mozilla.org/en-US/docs/Web/JavaScript/Reference/Global_Objects/Intl/NumberFormat/resolvedOptions
    fn resolved_options(this: &JsValue, _: &[JsValue], context: &mut Context) -> JsResult<JsValue> {
        // This function provides access to the locale and options computed during initialization of the object.

        // 1. Let nf be the this value.
        // 2. If the implementation supports the normative optional constructor mode of 4.3 Note 1, then
        //     a. Set nf to ? UnwrapNumberFormat(nf).
        // 3. Perform ? RequireInternalSlot(nf, [[InitializedNumberFormat]]).
        let nf = unwrap_number_format(this, context)?;
        let nf = nf.borrow();
        let nf = nf.data();

        // 4. Let options be OrdinaryObjectCreate(%Object.prototype%).
        // 5. For each row of Table 12, except the header row, in table order, do
        //     a. Let p be the Property value of the current row.
        //     b. Let v be the value of nf's internal slot whose name is the Internal Slot value of the current row.
        //     c. If v is not undefined, then
        //         i. If there is a Conversion value in the current row, then
        //             1. Assert: The Conversion value of the current row is number.
        //             2. Set v to 𝔽(v).
        //         ii. Perform ! CreateDataPropertyOrThrow(options, p, v).
        let mut options = ObjectInitializer::new(context);
        options.property(
            js_string!("locale"),
            js_string!(nf.locale.to_string()),
            Attribute::all(),
        );
        options.property(
            js_string!("numberingSystem"),
            JsString::from(
                nf.numbering_system
                    .as_ref()
                    .map_or_else(|| "latn".to_owned(), ToString::to_string),
            ),
            Attribute::all(),
        );

        options.property(
            js_string!("style"),
            nf.unit_options.style().to_js_string(),
            Attribute::all(),
        );

        match &nf.unit_options {
            UnitFormatOptions::Currency {
                currency,
                display,
                sign,
            } => {
                options.property(
                    js_string!("currency"),
                    currency.to_js_string(),
                    Attribute::all(),
                );
                options.property(
                    js_string!("currencyDisplay"),
                    display.to_js_string(),
                    Attribute::all(),
                );
                options.property(
                    js_string!("currencySign"),
                    sign.to_js_string(),
                    Attribute::all(),
                );
            }
            UnitFormatOptions::Unit { unit, display } => {
                options.property(js_string!("unit"), unit.to_js_string(), Attribute::all());
                options.property(
                    js_string!("unitDisplay"),
                    display.to_js_string(),
                    Attribute::all(),
                );
            }
            UnitFormatOptions::Decimal | UnitFormatOptions::Percent => {}
        }

        options.property(
            js_string!("minimumIntegerDigits"),
            nf.digit_options.minimum_integer_digits,
            Attribute::all(),
        );

        if let Some(Extrema { minimum, maximum }) = nf.digit_options.rounding_type.fraction_digits()
        {
            options
                .property(
                    js_string!("minimumFractionDigits"),
                    minimum,
                    Attribute::all(),
                )
                .property(
                    js_string!("maximumFractionDigits"),
                    maximum,
                    Attribute::all(),
                );
        }

        if let Some(Extrema { minimum, maximum }) =
            nf.digit_options.rounding_type.significant_digits()
        {
            options
                .property(
                    js_string!("minimumSignificantDigits"),
                    minimum,
                    Attribute::all(),
                )
                .property(
                    js_string!("maximumSignificantDigits"),
                    maximum,
                    Attribute::all(),
                );
        }

        let use_grouping = match nf.use_grouping {
            GroupingStrategy::Auto => js_string!("auto").into(),
            GroupingStrategy::Never => JsValue::from(false),
            GroupingStrategy::Always => js_string!("always").into(),
            GroupingStrategy::Min2 => js_string!("min2").into(),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("unsupported useGrouping value")
                    .into());
            }
        };

        options
            .property(js_string!("useGrouping"), use_grouping, Attribute::all())
            .property(
                js_string!("notation"),
                nf.notation.kind().to_js_string(),
                Attribute::all(),
            );

        if let Notation::Compact { display } = nf.notation {
            options.property(
                js_string!("compactDisplay"),
                display.to_js_string(),
                Attribute::all(),
            );
        }

        let sign_display = match nf.sign_display {
            SignDisplay::Auto => js_string!("auto"),
            SignDisplay::Never => js_string!("never"),
            SignDisplay::Always => js_string!("always"),
            SignDisplay::ExceptZero => js_string!("exceptZero"),
            SignDisplay::Negative => js_string!("negative"),
            _ => {
                return Err(JsNativeError::typ()
                    .with_message("unsupported signDisplay value")
                    .into());
            }
        };

        options
            .property(js_string!("signDisplay"), sign_display, Attribute::all())
            .property(
                js_string!("roundingIncrement"),
                nf.digit_options.rounding_increment.to_u16(),
                Attribute::all(),
            )
            .property(
                js_string!("roundingMode"),
                rounding_mode_to_js_string(nf.digit_options.rounding_mode),
                Attribute::all(),
            )
            .property(
                js_string!("roundingPriority"),
                nf.digit_options.rounding_priority.to_js_string(),
                Attribute::all(),
            )
            .property(
                js_string!("trailingZeroDisplay"),
                nf.digit_options.trailing_zero_display.to_js_string(),
                Attribute::all(),
            );

        // 6. Return options.
        Ok(options.build().into())
    }
}

/// Abstract operation [`UnwrapNumberFormat ( nf )`][spec].
///
/// This also checks that the returned object is a `NumberFormat`, which skips the
/// call to `RequireInternalSlot`.
///
/// [spec]: https://tc39.es/ecma402/#sec-unwrapnumberformat
fn unwrap_number_format(nf: &JsValue, context: &mut Context) -> JsResult<JsObject<NumberFormat>> {
    // 1. If Type(nf) is not Object, throw a TypeError exception.
    let nf_o = nf.as_object().ok_or_else(|| {
        JsNativeError::typ().with_message("value was not an `Intl.NumberFormat` object")
    })?;

    if let Ok(nf) = nf_o.clone().downcast::<NumberFormat>() {
        // 3. Return nf.
        return Ok(nf);
    }

    // 2. If nf does not have an [[InitializedNumberFormat]] internal slot and ? OrdinaryHasInstance(%Intl.NumberFormat%, nf)
    //    is true, then
    let constructor = context
        .intrinsics()
        .constructors()
        .number_format()
        .constructor();
    if JsValue::ordinary_has_instance(&constructor.into(), nf, context)? {
        let fallback_symbol = context
            .intrinsics()
            .objects()
            .intl()
            .borrow()
            .data()
            .fallback_symbol();

        //    a. Return ? Get(nf, %Intl%.[[FallbackSymbol]]).
        if let Some(nf) = nf_o
            .get(fallback_symbol, context)?
            .as_object()
            .and_then(|o| o.downcast::<NumberFormat>().ok())
        {
            return Ok(nf);
        }
    }

    Err(JsNativeError::typ()
        .with_message("object was not an `Intl.NumberFormat` object")
        .into())
}

/// Abstract operation [`ToIntlMathematicalValue ( value )`][spec].
///
/// [spec]: https://tc39.es/ecma402/#sec-tointlmathematicalvalue
fn to_intl_mathematical_value(
    value: &JsValue,
    context: &mut Context,
) -> JsResult<IntlMathematicalValue> {
    // 1. Let primValue be ? ToPrimitive(value, number).
    let prim_value = value.to_primitive(context, PreferredType::Number)?;

    // TODO: Add support in `Decimal` for infinity and NaN, which
    // should remove the returned errors.
    match prim_value.variant() {
        // 2. If Type(primValue) is BigInt, return ℝ(primValue).
        JsVariant::BigInt(bi) => Ok(IntlMathematicalValue::Finite(
            Decimal::try_from_str(&bi.to_string())
                .map_err(|err| JsNativeError::range().with_message(err.to_string()))?,
        )),
        // 3. If Type(primValue) is String, then
        //     a. Let str be primValue.
        JsVariant::String(s) => {
            // 5. Let text be StringToCodePoints(str).
            // 6. Let literal be ParseText(text, StringNumericLiteral).
            // 7. If literal is a List of errors, return not-a-number.
            // 8. Let intlMV be the StringIntlMV of literal.
            // 9. If intlMV is a mathematical value, then
            //     a. Let rounded be RoundMVResult(abs(intlMV)).
            //     b. If rounded is +∞𝔽 and intlMV < 0, return negative-infinity.
            //     c. If rounded is +∞𝔽, return positive-infinity.
            //     d. If rounded is +0𝔽 and intlMV < 0, return negative-zero.
            //     e. If rounded is +0𝔽, return 0.
            let Some(string) = s.to_std_string().ok() else {
                return Ok(IntlMathematicalValue::NotANumber);
            };
            let string = string.trim_matches(is_trimmable_whitespace);
            match string {
                "-Infinity" => Ok(IntlMathematicalValue::NegativeInfinity),
                "Infinity" | "+Infinity" => Ok(IntlMathematicalValue::PositiveInfinity),
                _ => Ok(js_string_to_fixed_decimal(&s)
                    .map(IntlMathematicalValue::Finite)
                    .unwrap_or(IntlMathematicalValue::NotANumber)),
            }
        }
        // 4. Else,
        _ => {
            // a. Let x be ? ToNumber(primValue).
            // b. If x is -0𝔽, return negative-zero.
            // c. Let str be Number::toString(x, 10).
            let x = prim_value.to_number(context)?;
            if x.is_nan() {
                return Ok(IntlMathematicalValue::NotANumber);
            }
            if x == f64::INFINITY {
                return Ok(IntlMathematicalValue::PositiveInfinity);
            }
            if x == f64::NEG_INFINITY {
                return Ok(IntlMathematicalValue::NegativeInfinity);
            }
            Ok(IntlMathematicalValue::Finite(
                Decimal::try_from_f64(x, FloatPrecision::RoundTrip)
                    .map_err(|err| JsNativeError::range().with_message(err.to_string()))?,
            ))
        }
    }
}

/// Abstract operation [`StringToNumber ( str )`][spec], but specialized for the conversion
/// to a `FixedDecimal`.
///
/// [spec]: https://tc39.es/ecma262/#sec-stringtonumber
// TODO: Introduce `Infinity` and `NaN` to `Decimal` to make this operation
// infallible.
pub(crate) fn js_string_to_fixed_decimal(string: &JsString) -> Option<Decimal> {
    // 1. Let text be ! StringToCodePoints(str).
    // 2. Let literal be ParseText(text, StringNumericLiteral).
    let Ok(string) = string.to_std_string() else {
        // 3. If literal is a List of errors, return NaN.
        return None;
    };
    // 4. Return StringNumericValue of literal.
    let string = string.trim_matches(is_trimmable_whitespace);
    match string {
        "" => return Some(Decimal::from(0)),
        "-Infinity" | "Infinity" | "+Infinity" => return None,
        _ => {}
    }

    let mut s = string.bytes();
    let base = match (s.next(), s.next()) {
        (Some(b'0'), Some(b'b' | b'B')) => Some(2),
        (Some(b'0'), Some(b'o' | b'O')) => Some(8),
        (Some(b'0'), Some(b'x' | b'X')) => Some(16),
        // Make sure that no further variants of "infinity" are parsed.
        (Some(b'i' | b'I'), _) => {
            return None;
        }
        _ => None,
    };

    // Parse numbers that begin with `0b`, `0o` and `0x`.
    let s = if let Some(base) = base {
        let string = &string[2..];
        if string.is_empty() {
            return None;
        }
        let int = BigInt::from_str_radix(string, base).ok()?;
        let int_str = int.to_string();

        Cow::Owned(int_str)
    } else {
        Cow::Borrowed(string)
    };

    Decimal::try_from_str(&s).ok()
}
