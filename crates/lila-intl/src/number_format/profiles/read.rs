use core::num::NonZeroU64;

use super::*;
use crate::number_format::numeric::{CompactExponentRow, PluralOperand};
use crate::number_format::plural_rules::{CategoryRule, OperandRelation};

struct Reader {
    bytes: &'static [u8],
    position: usize,
    table: NumberProfileTable,
    index: u32,
}

impl Reader {
    fn error(&self, reason: NumberProfileError) -> InvalidNumberProfile {
        InvalidNumberProfile {
            table: self.table,
            index: self.index,
            reason,
        }
    }

    fn take(&mut self, count: usize) -> Result<&'static [u8], InvalidNumberProfile> {
        let end = self
            .position
            .checked_add(count)
            .ok_or_else(|| self.error(NumberProfileError::Truncated))?;
        let result = self
            .bytes
            .get(self.position..end)
            .ok_or_else(|| self.error(NumberProfileError::Truncated))?;
        self.position = end;
        Ok(result)
    }

    fn u8(&mut self) -> Result<u8, InvalidNumberProfile> {
        Ok(self.take(1)?[0])
    }

    fn boolean(&mut self) -> Result<bool, InvalidNumberProfile> {
        match self.u8()? {
            0 => Ok(false),
            1 => Ok(true),
            _ => Err(self.error(NumberProfileError::UnknownCode)),
        }
    }

    fn u32(&mut self) -> Result<u32, InvalidNumberProfile> {
        let bytes = self.take(4)?;
        Ok(u32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]))
    }

    fn index(&mut self) -> Result<usize, InvalidNumberProfile> {
        usize::try_from(self.u32()?).map_err(|_| self.error(NumberProfileError::Index))
    }

    fn u64(&mut self) -> Result<u64, InvalidNumberProfile> {
        let bytes = self.take(8)?;
        Ok(u64::from_le_bytes([
            bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
        ]))
    }

    fn text(&mut self) -> Result<&'static str, InvalidNumberProfile> {
        let length = self.index()?;
        core::str::from_utf8(self.take(length)?)
            .map_err(|_| self.error(NumberProfileError::InvalidUtf8))
    }

    fn currency(&mut self) -> Result<[u8; 3], InvalidNumberProfile> {
        let bytes = self.take(3)?;
        if !bytes.iter().all(u8::is_ascii_uppercase) {
            return Err(self.error(NumberProfileError::UnknownCode));
        }
        Ok([bytes[0], bytes[1], bytes[2]])
    }

    fn list<T>(
        &mut self,
        mut read: impl FnMut(&mut Self) -> Result<T, InvalidNumberProfile>,
    ) -> Result<Box<[T]>, InvalidNumberProfile> {
        let length = self.index()?;
        // Every record in this schema consumes at least one byte. Reject an
        // impossible count before reserving any allocation for its payload.
        if length > self.bytes.len() - self.position {
            return Err(self.error(NumberProfileError::Truncated));
        }
        let mut result = Vec::new();
        result
            .try_reserve_exact(length)
            .map_err(|_| self.error(NumberProfileError::Allocation))?;
        for _ in 0..length {
            result.push(read(self)?);
        }
        Ok(result.into_boxed_slice())
    }

    fn table<T>(
        &mut self,
        table: NumberProfileTable,
        mut read: impl FnMut(&mut Self) -> Result<T, InvalidNumberProfile>,
    ) -> Result<Box<[T]>, InvalidNumberProfile> {
        self.table = table;
        self.index = 0;
        self.list(|reader| {
            let result = read(reader)?;
            reader.index += 1;
            Ok(result)
        })
    }

    fn option<T>(
        &mut self,
        read: impl FnOnce(&mut Self) -> Result<T, InvalidNumberProfile>,
    ) -> Result<Option<T>, InvalidNumberProfile> {
        if self.boolean()? {
            read(self).map(Some)
        } else {
            Ok(None)
        }
    }

    fn variants<T: Copy>(
        &mut self,
        mut read: impl FnMut(&mut Self) -> Result<T, InvalidNumberProfile>,
    ) -> Result<PluralVariants<T>, InvalidNumberProfile> {
        let first = read(self)?;
        let mut categories = [first; 6];
        for value in &mut categories[1..] {
            *value = read(self)?;
        }
        let exact_zero = self.option(&mut read)?;
        let exact_one = self.option(&mut read)?;
        Ok(PluralVariants {
            categories,
            exact_zero,
            exact_one,
        })
    }

    fn category(&mut self) -> Result<CardinalCategory, InvalidNumberProfile> {
        CardinalCategory::decode(self.u8()?)
            .ok_or_else(|| self.error(NumberProfileError::UnknownCode))
    }

    fn fraction_digits(&mut self) -> Result<FractionDigitCount, InvalidNumberProfile> {
        FractionDigitCount::new(self.u8()?)
            .map_err(|_| self.error(NumberProfileError::InvalidRange))
    }

    fn token(&mut self) -> Result<Token, InvalidNumberProfile> {
        match self.u8()? {
            0 => Ok(Token::Literal(TextId(self.index()?))),
            1 => Ok(Token::Number),
            2 => Ok(Token::MinusSign),
            3 => Ok(Token::PlusSign),
            4 => Ok(Token::PercentSign),
            5 => Ok(Token::Currency),
            6 => Ok(Token::Compact(TextId(self.index()?))),
            7 => Ok(Token::Unit(TextId(self.index()?))),
            8 => Ok(Token::Argument1),
            _ => Err(self.error(NumberProfileError::UnknownCode)),
        }
    }

    fn compact_pattern(&mut self) -> Result<CompactPattern, InvalidNumberProfile> {
        if self.boolean()? {
            Ok(CompactPattern::Normal)
        } else {
            Ok(CompactPattern::Pattern(SignedPatternId(self.index()?)))
        }
    }

    fn spacing(&mut self) -> Result<CurrencySpacing, InvalidNumberProfile> {
        Ok(CurrencySpacing {
            currency: UnicodeSetId(self.index()?),
            surrounding: UnicodeSetId(self.index()?),
            insert: TextId(self.index()?),
        })
    }
}

pub(super) fn decode(bytes: &'static [u8]) -> Result<NumberProfiles, InvalidNumberProfile> {
    let mut reader = Reader {
        bytes,
        position: 0,
        table: NumberProfileTable::Header,
        index: 0,
    };
    if reader.take(8)? != b"LNF47\0\x01\0" {
        return Err(reader.error(NumberProfileError::Version));
    }
    let texts = reader.table(NumberProfileTable::Strings, Reader::text)?;
    let patterns = reader.table(NumberProfileTable::Patterns, |r| {
        r.list(Reader::token).map(Pattern)
    })?;
    let signed_patterns = reader.table(NumberProfileTable::SignedPatterns, |r| {
        Ok(SignedPattern {
            positive: PatternId(r.index()?),
            negative: PatternId(r.index()?),
            has_number: r.boolean()?,
            grouping: GroupingWidths {
                primary: r.u8()?,
                secondary: r.u8()?,
            },
        })
    })?;
    let pattern_choices = reader.table(NumberProfileTable::PatternChoices, |r| {
        r.variants(|r| r.index().map(PatternId))
    })?;
    let string_choices = reader.table(NumberProfileTable::StringChoices, |r| {
        r.variants(|r| r.index().map(TextId))
    })?;
    let compact_choices = reader.table(NumberProfileTable::CompactChoices, |r| {
        r.variants(Reader::compact_pattern)
    })?;
    let symbols = reader.table(NumberProfileTable::Symbols, |r| {
        Ok(NumberSymbols {
            decimal: TextId(r.index()?),
            group: TextId(r.index()?),
            plus: TextId(r.index()?),
            minus: TextId(r.index()?),
            percent: TextId(r.index()?),
            approximately: TextId(r.index()?),
            exponent: TextId(r.index()?),
            infinity: TextId(r.index()?),
            nan: TextId(r.index()?),
            currency_decimal: TextId(r.index()?),
            currency_group: TextId(r.index()?),
        })
    })?;
    let unicode_sets = reader.table(NumberProfileTable::UnicodeSets, |r| {
        r.list(|r| Ok((r.u32()?, r.u32()?))).map(UnicodeSet)
    })?;
    let compact_sets = reader.table(NumberProfileTable::CompactSets, |r| {
        let rows = r.list(|r| {
            Ok(CompactRow {
                magnitude: r.u32()?,
                exponent: r.u32()?,
                choices: CompactChoicesId(r.index()?),
            })
        })?;
        let mut exponents = Vec::new();
        exponents
            .try_reserve_exact(rows.len())
            .map_err(|_| r.error(NumberProfileError::Allocation))?;
        for row in &rows {
            exponents.push(
                CompactExponentRow::new(row.magnitude, row.exponent)
                    .map_err(|_| r.error(NumberProfileError::InvalidRange))?,
            );
        }
        let exponents = CompactExponentTable::new(exponents.into_boxed_slice())
            .map_err(|_| r.error(NumberProfileError::Order))?;
        Ok(CompactSet { rows, exponents })
    })?;
    let numbering_profiles = reader.table(NumberProfileTable::NumberingProfiles, |r| {
        Ok(NumberingProfile {
            symbols: SymbolId(r.index()?),
            decimal: SignedPatternId(r.index()?),
            percent: SignedPatternId(r.index()?),
            currency: SignedPatternId(r.index()?),
            accounting: SignedPatternId(r.index()?),
            currency_alpha: SignedPatternId(r.index()?),
            accounting_alpha: SignedPatternId(r.index()?),
            minimum_grouping: r.u8()?,
            compact_short: CompactSetId(r.index()?),
            compact_long: CompactSetId(r.index()?),
            compact_currency: CompactSetId(r.index()?),
            compact_currency_alpha: CompactSetId(r.index()?),
            range_pattern: PatternId(r.index()?),
            currency_unit_pattern: PatternChoicesId(r.index()?),
            before_currency: r.spacing()?,
            after_currency: r.spacing()?,
            currency_overrides: r.list(|r| {
                Ok(CurrencyOverride {
                    code: r.currency()?,
                    decimal: r.option(|r| r.index().map(TextId))?,
                    group: r.option(|r| r.index().map(TextId))?,
                    pattern: r.option(|r| r.index().map(SignedPatternId))?,
                })
            })?,
        })
    })?;
    let currency_sets = reader.table(NumberProfileTable::CurrencySets, |r| {
        r.list(|r| {
            Ok(CurrencyLabels {
                code: r.currency()?,
                symbol: TextId(r.index()?),
                narrow: TextId(r.index()?),
                names: StringChoicesId(r.index()?),
            })
        })
        .map(CurrencySet)
    })?;
    let unit_sets = reader.table(NumberProfileTable::UnitSets, |r| {
        Ok(UnitSet {
            simple: r.list(|r| {
                Ok(UnitPatterns {
                    choices: PatternChoicesId(r.index()?),
                    per_unit: r.option(|r| r.index().map(PatternId))?,
                    denominator: PatternId(r.index()?),
                })
            })?,
            pairs: r.list(|r| {
                Ok(UnitPairPatterns {
                    numerator: r.u8()?,
                    denominator: r.u8()?,
                    choices: PatternChoicesId(r.index()?),
                })
            })?,
            per_pattern: PatternId(r.index()?),
        })
    })?;
    let plural_rules = reader.table(NumberProfileTable::PluralRules, |r| {
        r.list(|r| {
            Ok(CategoryRule {
                category: r.category()?,
                alternatives: r.list(|r| {
                    r.list(|r| {
                        let operand = match r.u8()? {
                            0 => PluralOperand::N,
                            1 => PluralOperand::I,
                            2 => PluralOperand::V,
                            3 => PluralOperand::W,
                            4 => PluralOperand::F,
                            5 => PluralOperand::T,
                            6 => PluralOperand::C,
                            7 => PluralOperand::E,
                            _ => return Err(r.error(NumberProfileError::UnknownCode)),
                        };
                        Ok(OperandRelation {
                            operand,
                            modulus: NonZeroU64::new(r.u64()?),
                            integer_only: r.boolean()?,
                            negate: r.boolean()?,
                            ranges: r.list(|r| Ok((r.u64()?, r.u64()?)))?,
                        })
                    })
                })?,
            })
        })
        .map(CardinalRules)
    })?;
    let plural_ranges = reader.table(NumberProfileTable::PluralRanges, |r| {
        let mut categories = [CardinalCategory::Other; 36];
        for category in &mut categories {
            *category = r.category()?;
        }
        Ok(categories)
    })?;
    let profiles = reader.table(NumberProfileTable::Profiles, |r| {
        Ok(LocaleProfile {
            default_numbering: r.u8()?,
            numbering: r.list(|r| r.index().map(NumberingProfileId))?,
            plural_rules: PluralRulesId(r.index()?),
            plural_ranges: PluralRangeId(r.index()?),
            currencies: CurrencySetId(r.index()?),
            units: [
                UnitSetId(r.index()?),
                UnitSetId(r.index()?),
                UnitSetId(r.index()?),
            ],
        })
    })?;
    let locale_rows = reader.table(NumberProfileTable::Locales, |r| {
        Ok((r.text()?, ProfileId(r.index()?)))
    })?;
    let mut locales = Vec::new();
    let mut locale_profiles = Vec::new();
    locales
        .try_reserve_exact(locale_rows.len())
        .map_err(|_| reader.error(NumberProfileError::Allocation))?;
    locale_profiles
        .try_reserve_exact(locale_rows.len())
        .map_err(|_| reader.error(NumberProfileError::Allocation))?;
    for &(locale, profile) in &locale_rows {
        locales.push(locale);
        locale_profiles.push(profile);
    }
    let systems = reader.table(NumberProfileTable::NumberingSystems, |r| {
        let name = r.text()?;
        let mut digits = ['0'; 10];
        for digit in &mut digits {
            *digit = char::from_u32(r.u32()?)
                .ok_or_else(|| r.error(NumberProfileError::InvalidRange))?;
        }
        Ok(NumberingSystem { name, digits })
    })?;
    let mut system_names = Vec::new();
    system_names
        .try_reserve_exact(systems.len())
        .map_err(|_| reader.error(NumberProfileError::Allocation))?;
    for system in &systems {
        system_names.push(system.name);
    }
    let unit_names = reader.table(NumberProfileTable::Units, Reader::text)?;
    reader.table = NumberProfileTable::CurrencyFractions;
    reader.index = 0;
    let default = reader.fraction_digits()?;
    let overrides = reader.list(|r| {
        Ok(CurrencyFractionRecord {
            code: r.currency()?,
            digits: r.fraction_digits()?,
        })
    })?;
    let letter_set = UnicodeSetId(reader.index()?);
    let digit_set = UnicodeSetId(reader.index()?);
    let whitespace_set = UnicodeSetId(reader.index()?);
    if reader.position != bytes.len() {
        return Err(reader.error(NumberProfileError::TrailingBytes));
    }
    let result = NumberProfiles {
        texts,
        patterns,
        signed_patterns,
        pattern_choices,
        string_choices,
        compact_choices,
        symbols,
        unicode_sets,
        compact_sets,
        numbering_profiles,
        currency_sets,
        unit_sets,
        plural_rules,
        plural_ranges,
        profiles,
        locales: locales.into_boxed_slice(),
        locale_profiles: locale_profiles.into_boxed_slice(),
        systems,
        system_names: system_names.into_boxed_slice(),
        unit_names,
        fractions: CurrencyFractions { default, overrides },
        letter_set,
        digit_set,
        whitespace_set,
    };
    result.validate()?;
    Ok(result)
}
