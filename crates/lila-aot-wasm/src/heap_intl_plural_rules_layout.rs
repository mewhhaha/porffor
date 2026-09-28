use super::heap::{
    HeapLayoutSlot, HEAP_INTL_PR_CATEGORY_COUNT_OFFSET, HEAP_INTL_PR_CATEGORY_MASK_OFFSET,
    HEAP_INTL_PR_DATA_LOCALE_OFFSET, HEAP_INTL_PR_LOCALE_OFFSET, HEAP_INTL_PR_TYPE_OFFSET,
    HEAP_INTL_PR_WORDS_OFFSET,
};
use lila_intl::NumberConfigurationWord;

pub(crate) enum IntlPluralRulesHeapSlot {
    LocalePayload,
    DataLocalePayload,
    RuleType,
    CategoryMask,
    CategoryCount,
    Configuration(NumberConfigurationWord),
}

impl IntlPluralRulesHeapSlot {
    pub(crate) const fn layout(&self) -> HeapLayoutSlot {
        let (name, offset, pointer) = match self {
            Self::LocalePayload => ("locale_payload", HEAP_INTL_PR_LOCALE_OFFSET, true),
            Self::DataLocalePayload => ("data_locale_payload", HEAP_INTL_PR_DATA_LOCALE_OFFSET, true),
            Self::RuleType => ("rule_type", HEAP_INTL_PR_TYPE_OFFSET, false),
            Self::CategoryMask => ("category_mask", HEAP_INTL_PR_CATEGORY_MASK_OFFSET, false),
            Self::CategoryCount => ("category_count", HEAP_INTL_PR_CATEGORY_COUNT_OFFSET, false),
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
                HEAP_INTL_PR_WORDS_OFFSET + word.offset(),
                false,
            ),
        };
        HeapLayoutSlot {
            record: "intl-plural-rules-record",
            name,
            offset,
            width: 8,
            pointer,
        }
    }
}

pub(crate) const HEAP_INTL_PLURAL_RULES_RECORD_LAYOUT: &[IntlPluralRulesHeapSlot] = &[
    IntlPluralRulesHeapSlot::LocalePayload,
    IntlPluralRulesHeapSlot::DataLocalePayload,
    IntlPluralRulesHeapSlot::RuleType,
    IntlPluralRulesHeapSlot::CategoryMask,
    IntlPluralRulesHeapSlot::CategoryCount,
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::Style),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::CurrencyDisplay),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::CurrencySign),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::UnitDisplay),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::Notation),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::CompactDisplay),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::MinimumInteger),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::Precision),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::MinimumFraction),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::MaximumFraction),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::MinimumSignificant),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::MaximumSignificant),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::RoundingIncrement),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::RoundingMode),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::TrailingZero),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::Grouping),
    IntlPluralRulesHeapSlot::Configuration(NumberConfigurationWord::SignDisplay),
];
