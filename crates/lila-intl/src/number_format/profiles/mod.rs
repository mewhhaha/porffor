use core::fmt;
use std::sync::OnceLock;

use super::numeric::CompactExponentTable;
use super::options::{CurrencyCode, FractionDigitCount};
use super::plural_rules::{CardinalCategory, CardinalRules, PluralVariants};

mod fingerprint;
mod read;
mod validate;
pub use fingerprint::NUMBER_FORMAT_DATA_SHA256;

#[cfg(test)]
mod tests;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberProfileTable {
    Header,
    Strings,
    Patterns,
    SignedPatterns,
    PatternChoices,
    StringChoices,
    CompactChoices,
    Symbols,
    UnicodeSets,
    CompactSets,
    NumberingProfiles,
    CurrencySets,
    UnitSets,
    PluralRules,
    PluralRanges,
    Profiles,
    Locales,
    NumberingSystems,
    Units,
    CurrencyFractions,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberProfileError {
    Truncated,
    Version,
    InvalidUtf8,
    UnknownCode,
    Index,
    Order,
    Cardinality,
    PatternRole,
    InvalidRange,
    InvalidLocale,
    MissingDefaultLocale,
    TrailingBytes,
    Allocation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct InvalidNumberProfile {
    pub table: NumberProfileTable,
    pub index: u32,
    pub reason: NumberProfileError,
}

impl fmt::Display for InvalidNumberProfile {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "invalid NumberFormat profile {:?}[{}]: {:?}",
            self.table, self.index, self.reason
        )
    }
}

impl std::error::Error for InvalidNumberProfile {}

macro_rules! index_domain {
    ($($name:ident),+ $(,)?) => {$(
        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
        pub(super) struct $name(pub usize);
    )+};
}

index_domain!(
    TextId,
    PatternId,
    SignedPatternId,
    PatternChoicesId,
    StringChoicesId,
    CompactChoicesId,
    SymbolId,
    UnicodeSetId,
    CompactSetId,
    NumberingProfileId,
    CurrencySetId,
    UnitSetId,
    PluralRulesId,
    PluralRangeId,
    ProfileId,
);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum Token {
    Literal(TextId),
    Number,
    MinusSign,
    PlusSign,
    PercentSign,
    Currency,
    Compact(TextId),
    Unit(TextId),
    Argument1,
}

#[derive(Debug)]
pub(super) struct Pattern(pub Box<[Token]>);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct GroupingWidths {
    pub primary: u8,
    pub secondary: u8,
}

#[derive(Debug)]
pub(super) struct SignedPattern {
    pub positive: PatternId,
    pub negative: PatternId,
    pub grouping: GroupingWidths,
    pub has_number: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) enum CompactPattern {
    Pattern(SignedPatternId),
    Normal,
}

#[derive(Debug)]
pub(super) struct CompactRow {
    pub magnitude: u32,
    pub exponent: u32,
    pub choices: CompactChoicesId,
}

#[derive(Debug)]
pub(super) struct CompactSet {
    pub rows: Box<[CompactRow]>,
    pub exponents: CompactExponentTable,
}

impl CompactSet {
    pub fn row(&self, magnitude: u32) -> Option<&CompactRow> {
        self.rows
            .binary_search_by_key(&magnitude, |row| row.magnitude)
            .ok()
            .map(|index| &self.rows[index])
    }
}

#[derive(Debug)]
pub(super) struct NumberSymbols {
    pub decimal: TextId,
    pub group: TextId,
    pub plus: TextId,
    pub minus: TextId,
    pub percent: TextId,
    pub approximately: TextId,
    pub exponent: TextId,
    pub infinity: TextId,
    pub nan: TextId,
    pub currency_decimal: TextId,
    pub currency_group: TextId,
}

#[derive(Debug)]
pub(super) struct UnicodeSet(pub Box<[(u32, u32)]>);

impl UnicodeSet {
    pub fn contains(&self, character: char) -> bool {
        let point = u32::from(character);
        let index = self.0.partition_point(|&(start, _)| start <= point);
        index > 0 && point <= self.0[index - 1].1
    }
}

#[derive(Debug)]
pub(super) struct CurrencySpacing {
    pub currency: UnicodeSetId,
    pub surrounding: UnicodeSetId,
    pub insert: TextId,
}

#[derive(Debug)]
pub(super) struct CurrencyOverride {
    pub code: [u8; 3],
    pub decimal: Option<TextId>,
    pub group: Option<TextId>,
    pub pattern: Option<SignedPatternId>,
}

#[derive(Debug)]
pub(super) struct NumberingProfile {
    pub symbols: SymbolId,
    pub decimal: SignedPatternId,
    pub percent: SignedPatternId,
    pub currency: SignedPatternId,
    pub accounting: SignedPatternId,
    pub currency_alpha: SignedPatternId,
    pub accounting_alpha: SignedPatternId,
    pub minimum_grouping: u8,
    pub compact_short: CompactSetId,
    pub compact_long: CompactSetId,
    pub compact_currency: CompactSetId,
    pub compact_currency_alpha: CompactSetId,
    pub range_pattern: PatternId,
    pub currency_unit_pattern: PatternChoicesId,
    pub before_currency: CurrencySpacing,
    pub after_currency: CurrencySpacing,
    pub currency_overrides: Box<[CurrencyOverride]>,
}

impl NumberingProfile {
    pub fn currency_override(&self, code: [u8; 3]) -> Option<&CurrencyOverride> {
        self.currency_overrides
            .binary_search_by_key(&code, |row| row.code)
            .ok()
            .map(|index| &self.currency_overrides[index])
    }
}

#[derive(Debug)]
pub(super) struct CurrencyLabels {
    pub code: [u8; 3],
    pub symbol: TextId,
    pub narrow: TextId,
    pub names: StringChoicesId,
}

#[derive(Debug)]
pub(super) struct CurrencySet(pub Box<[CurrencyLabels]>);

impl CurrencySet {
    pub fn find(&self, code: [u8; 3]) -> Option<&CurrencyLabels> {
        self.0
            .binary_search_by_key(&code, |row| row.code)
            .ok()
            .map(|index| &self.0[index])
    }
}

#[derive(Debug)]
pub(super) struct UnitPatterns {
    pub choices: PatternChoicesId,
    pub per_unit: Option<PatternId>,
    pub denominator: PatternId,
}

#[derive(Debug)]
pub(super) struct UnitPairPatterns {
    pub numerator: u8,
    pub denominator: u8,
    pub choices: PatternChoicesId,
}

#[derive(Debug)]
pub(super) struct UnitSet {
    pub simple: Box<[UnitPatterns]>,
    pub pairs: Box<[UnitPairPatterns]>,
    pub per_pattern: PatternId,
}

impl UnitSet {
    pub fn pair(&self, numerator: usize, denominator: usize) -> Option<PatternChoicesId> {
        self.pairs
            .binary_search_by_key(&(numerator, denominator), |row| {
                (usize::from(row.numerator), usize::from(row.denominator))
            })
            .ok()
            .map(|index| self.pairs[index].choices)
    }
}

#[derive(Debug)]
pub(super) struct LocaleProfile {
    pub default_numbering: u8,
    pub numbering: Box<[NumberingProfileId]>,
    pub plural_rules: PluralRulesId,
    pub plural_ranges: PluralRangeId,
    pub currencies: CurrencySetId,
    pub units: [UnitSetId; 3],
}

#[derive(Debug)]
pub(super) struct NumberingSystem {
    pub name: &'static str,
    pub digits: [char; 10],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CurrencyFractionRecord {
    code: [u8; 3],
    digits: FractionDigitCount,
}

impl CurrencyFractionRecord {
    pub const fn code(&self) -> [u8; 3] {
        self.code
    }

    pub const fn digits(&self) -> FractionDigitCount {
        self.digits
    }
}

#[derive(Debug)]
pub struct CurrencyFractions {
    default: FractionDigitCount,
    overrides: Box<[CurrencyFractionRecord]>,
}

impl CurrencyFractions {
    pub fn digits(&self, code: &CurrencyCode) -> FractionDigitCount {
        let code = code.clone().ascii();
        self.overrides
            .binary_search_by_key(&code, |row| row.code)
            .map_or(self.default, |index| self.overrides[index].digits)
    }

    pub const fn default_digits(&self) -> FractionDigitCount {
        self.default
    }

    pub fn overrides(&self) -> &[CurrencyFractionRecord] {
        &self.overrides
    }
}

#[derive(Debug)]
pub struct NumberProfiles {
    texts: Box<[&'static str]>,
    patterns: Box<[Pattern]>,
    signed_patterns: Box<[SignedPattern]>,
    pattern_choices: Box<[PluralVariants<PatternId>]>,
    string_choices: Box<[PluralVariants<TextId>]>,
    compact_choices: Box<[PluralVariants<CompactPattern>]>,
    symbols: Box<[NumberSymbols]>,
    unicode_sets: Box<[UnicodeSet]>,
    compact_sets: Box<[CompactSet]>,
    numbering_profiles: Box<[NumberingProfile]>,
    currency_sets: Box<[CurrencySet]>,
    unit_sets: Box<[UnitSet]>,
    plural_rules: Box<[CardinalRules]>,
    plural_ranges: Box<[[CardinalCategory; 36]]>,
    profiles: Box<[LocaleProfile]>,
    locales: Box<[&'static str]>,
    locale_profiles: Box<[ProfileId]>,
    systems: Box<[NumberingSystem]>,
    system_names: Box<[&'static str]>,
    unit_names: Box<[&'static str]>,
    fractions: CurrencyFractions,
    letter_set: UnicodeSetId,
    digit_set: UnicodeSetId,
    whitespace_set: UnicodeSetId,
}

impl NumberProfiles {
    pub fn available_locales(&self) -> &[&'static str] {
        &self.locales
    }

    pub fn numbering_systems(&self) -> &[&'static str] {
        &self.system_names
    }

    pub fn currency_fractions(&self) -> &CurrencyFractions {
        &self.fractions
    }

    pub(super) fn profile(&self, locale: &str) -> Option<&LocaleProfile> {
        self.locales
            .binary_search(&locale)
            .ok()
            .map(|index| &self.profiles[self.locale_profiles[index].0])
    }

    pub(super) fn system(&self, name: &str) -> Option<(usize, &NumberingSystem)> {
        self.system_names
            .binary_search(&name)
            .ok()
            .map(|index| (index, &self.systems[index]))
    }

    pub(super) fn text(&self, id: TextId) -> &'static str {
        self.texts[id.0]
    }
    pub(super) fn pattern(&self, id: PatternId) -> &Pattern {
        &self.patterns[id.0]
    }
    pub(super) fn signed(&self, id: SignedPatternId) -> &SignedPattern {
        &self.signed_patterns[id.0]
    }
    pub(super) fn pattern_choices(&self, id: PatternChoicesId) -> &PluralVariants<PatternId> {
        &self.pattern_choices[id.0]
    }
    pub(super) fn string_choices(&self, id: StringChoicesId) -> &PluralVariants<TextId> {
        &self.string_choices[id.0]
    }
    pub(super) fn compact_choices(&self, id: CompactChoicesId) -> &PluralVariants<CompactPattern> {
        &self.compact_choices[id.0]
    }
    pub(super) fn symbols(&self, id: SymbolId) -> &NumberSymbols {
        &self.symbols[id.0]
    }
    pub(super) fn compact(&self, id: CompactSetId) -> &CompactSet {
        &self.compact_sets[id.0]
    }
    pub(super) fn numbering(&self, id: NumberingProfileId) -> &NumberingProfile {
        &self.numbering_profiles[id.0]
    }
    pub(super) fn currencies(&self, id: CurrencySetId) -> &CurrencySet {
        &self.currency_sets[id.0]
    }
    pub(super) fn units(&self, id: UnitSetId) -> &UnitSet {
        &self.unit_sets[id.0]
    }
    pub(super) fn rules(&self, id: PluralRulesId) -> &CardinalRules {
        &self.plural_rules[id.0]
    }
    pub(super) fn range_category(
        &self,
        id: PluralRangeId,
        start: CardinalCategory,
        end: CardinalCategory,
    ) -> CardinalCategory {
        self.plural_ranges[id.0][start.index() * 6 + end.index()]
    }
    pub(super) fn in_set(&self, id: UnicodeSetId, character: char) -> bool {
        self.unicode_sets[id.0].contains(character)
    }
    pub(super) fn is_letter(&self, character: char) -> bool {
        self.in_set(self.letter_set, character)
    }
    pub(super) fn is_digit(&self, character: char) -> bool {
        self.in_set(self.digit_set, character)
    }
    pub(super) fn is_whitespace(&self, character: char) -> bool {
        self.in_set(self.whitespace_set, character)
    }
}

pub fn embedded_number_profiles() -> Result<&'static NumberProfiles, InvalidNumberProfile> {
    static PROFILES: OnceLock<Result<NumberProfiles, InvalidNumberProfile>> = OnceLock::new();
    PROFILES
        .get_or_init(|| {
            let bytes = include_bytes!("../../../data/number-cldr-47/profiles.bin");
            read::decode(bytes)
        })
        .as_ref()
        .map_err(|error| *error)
}
