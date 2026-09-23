#![allow(
    dead_code,
    reason = "T05 layout metadata precedes the atomic Wasm-GC value cutover"
)]

use super::heap::{
    HeapLayoutSlot, HEAP_INTL_NF_BOUND_FORMAT_OFFSET, HEAP_INTL_NF_DATA_LOCALE_OFFSET,
    HEAP_INTL_NF_LOCALE_OFFSET, HEAP_INTL_NF_NUMBERING_SYSTEM_OFFSET,
    HEAP_INTL_NF_STYLE_TEXT_OFFSET, HEAP_INTL_NF_WORDS_OFFSET,
};
use lila_intl::NumberConfigurationWord;

pub(crate) enum IntlNumberFormatHeapSlot {
    LocalePayload,
    DataLocalePayload,
    NumberingSystemPayload,
    StyleTextPayload,
    BoundFormatPayload,
    Configuration(NumberConfigurationWord),
}
impl IntlNumberFormatHeapSlot {
    pub(crate) const fn layout(&self) -> HeapLayoutSlot {
        let (name, offset, pointer) = match self {
            Self::LocalePayload => ("locale_payload", HEAP_INTL_NF_LOCALE_OFFSET, true),
            Self::DataLocalePayload => {
                ("data_locale_payload", HEAP_INTL_NF_DATA_LOCALE_OFFSET, true)
            }
            Self::NumberingSystemPayload => (
                "numbering_system_payload",
                HEAP_INTL_NF_NUMBERING_SYSTEM_OFFSET,
                true,
            ),
            Self::StyleTextPayload => ("style_text_payload", HEAP_INTL_NF_STYLE_TEXT_OFFSET, true),
            Self::BoundFormatPayload => (
                "bound_format_payload",
                HEAP_INTL_NF_BOUND_FORMAT_OFFSET,
                true,
            ),
            Self::Configuration(word) => (
                match word {
                    NumberConfigurationWord::Style => "style",
                    NumberConfigurationWord::CurrencyDisplay => "currency_display",
                    NumberConfigurationWord::CurrencySign => "currency_sign",
                    NumberConfigurationWord::UnitDisplay => "unit_display",
                    NumberConfigurationWord::Notation => "notation",
                    NumberConfigurationWord::CompactDisplay => "compact_display",
                    NumberConfigurationWord::MinimumInteger => "minimum_integer",
                    NumberConfigurationWord::Precision => "precision",
                    NumberConfigurationWord::MinimumFraction => "minimum_fraction",
                    NumberConfigurationWord::MaximumFraction => "maximum_fraction",
                    NumberConfigurationWord::MinimumSignificant => "minimum_significant",
                    NumberConfigurationWord::MaximumSignificant => "maximum_significant",
                    NumberConfigurationWord::RoundingIncrement => "rounding_increment",
                    NumberConfigurationWord::RoundingMode => "rounding_mode",
                    NumberConfigurationWord::TrailingZero => "trailing_zero",
                    NumberConfigurationWord::Grouping => "grouping",
                    NumberConfigurationWord::SignDisplay => "sign_display",
                },
                HEAP_INTL_NF_WORDS_OFFSET + word.offset(),
                false,
            ),
        };
        HeapLayoutSlot {
            record: "intl-number-format-record",
            name,
            offset,
            width: 8,
            pointer,
        }
    }
}

pub(crate) const HEAP_INTL_NUMBER_FORMAT_RECORD_LAYOUT: &[IntlNumberFormatHeapSlot] = &[
    IntlNumberFormatHeapSlot::LocalePayload,
    IntlNumberFormatHeapSlot::DataLocalePayload,
    IntlNumberFormatHeapSlot::NumberingSystemPayload,
    IntlNumberFormatHeapSlot::StyleTextPayload,
    IntlNumberFormatHeapSlot::BoundFormatPayload,
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::Style),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::CurrencyDisplay),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::CurrencySign),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::UnitDisplay),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::Notation),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::CompactDisplay),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::MinimumInteger),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::Precision),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::MinimumFraction),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::MaximumFraction),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::MinimumSignificant),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::MaximumSignificant),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::RoundingIncrement),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::RoundingMode),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::TrailingZero),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::Grouping),
    IntlNumberFormatHeapSlot::Configuration(NumberConfigurationWord::SignDisplay),
];
