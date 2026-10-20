//! The exact inverse of the existing LNF47v2 reader, with typed dense IDs.
use super::closure::{Remap, TableIndex, TableMaps};
use super::*;
use crate::number_format::numeric::PluralOperand;

struct Writer {
    bytes: Vec<u8>,
    table: NumberProfileTable,
    index: usize,
}

impl Writer {
    fn bytes(&mut self, value: &[u8]) -> Result<(), InvalidNumberProfile> {
        self.bytes
            .try_reserve(value.len())
            .map_err(|_| error(self.table, self.index, NumberProfileError::Allocation))?;
        self.bytes.extend_from_slice(value);
        Ok(())
    }
    fn u8(&mut self, value: u8) -> Result<(), InvalidNumberProfile> {
        self.bytes(&[value])
    }
    fn boolean(&mut self, value: bool) -> Result<(), InvalidNumberProfile> {
        self.u8(u8::from(value))
    }
    fn category(&mut self, value: CardinalCategory) -> Result<(), InvalidNumberProfile> {
        self.u8(match value {
            CardinalCategory::Zero => 0,
            CardinalCategory::One => 1,
            CardinalCategory::Two => 2,
            CardinalCategory::Few => 3,
            CardinalCategory::Many => 4,
            CardinalCategory::Other => 5,
        })
    }
    fn u32(&mut self, value: u32) -> Result<(), InvalidNumberProfile> {
        self.bytes(&value.to_le_bytes())
    }
    fn u64(&mut self, value: u64) -> Result<(), InvalidNumberProfile> {
        self.bytes(&value.to_le_bytes())
    }
    fn length(&mut self, value: usize) -> Result<(), InvalidNumberProfile> {
        let value = u32::try_from(value)
            .map_err(|_| error(self.table, self.index, NumberProfileError::Index))?;
        self.u32(value)
    }
    fn text(&mut self, text: &str) -> Result<(), InvalidNumberProfile> {
        self.length(text.len())?;
        self.bytes(text.as_bytes())
    }
    fn id<I: TableIndex>(&mut self, map: &Remap<I>, id: I) -> Result<(), InvalidNumberProfile> {
        self.u32(map.get(id)?)
    }
    fn list<T>(
        &mut self,
        values: &[T],
        mut write: impl FnMut(&mut Self, &T) -> Result<(), InvalidNumberProfile>,
    ) -> Result<(), InvalidNumberProfile> {
        self.length(values.len())?;
        for value in values {
            write(self, value)?;
        }
        Ok(())
    }
    fn option<T>(
        &mut self,
        value: Option<T>,
        write: impl FnOnce(&mut Self, T) -> Result<(), InvalidNumberProfile>,
    ) -> Result<(), InvalidNumberProfile> {
        self.boolean(value.is_some())?;
        if let Some(value) = value {
            write(self, value)?;
        }
        Ok(())
    }
    fn filtered_list<T>(
        &mut self,
        values: &[T],
        selected: impl Fn(&T) -> bool,
        mut write: impl FnMut(&mut Self, &T) -> Result<(), InvalidNumberProfile>,
    ) -> Result<(), InvalidNumberProfile> {
        self.length(values.iter().filter(|value| selected(value)).count())?;
        for value in values {
            if selected(value) {
                write(self, value)?;
            }
        }
        Ok(())
    }
    fn variants<T: Copy>(
        &mut self,
        variants: &PluralVariants<T>,
        mut write: impl FnMut(&mut Self, T) -> Result<(), InvalidNumberProfile>,
    ) -> Result<(), InvalidNumberProfile> {
        for value in variants.categories {
            write(self, value)?;
        }
        self.option(variants.exact_zero, &mut write)?;
        self.option(variants.exact_one, write)
    }
    fn table<I: TableIndex, T>(
        &mut self,
        map: &Remap<I>,
        source: &[T],
        mut write: impl FnMut(&mut Self, &T) -> Result<(), InvalidNumberProfile>,
    ) -> Result<(), InvalidNumberProfile> {
        self.table = I::TABLE;
        self.index = 0;
        self.length(map.count())?;
        for old in map.old_indices() {
            write(self, &source[old])?;
            self.index += 1;
        }
        Ok(())
    }
    fn spacing(
        &mut self,
        row: &CurrencySpacing,
        maps: &TableMaps,
    ) -> Result<(), InvalidNumberProfile> {
        self.id(&maps.unicode_sets, row.currency)?;
        self.id(&maps.unicode_sets, row.surrounding)?;
        self.id(&maps.texts, row.insert)
    }
    fn compact_pattern(
        &mut self,
        value: CompactPattern,
        maps: &TableMaps,
    ) -> Result<(), InvalidNumberProfile> {
        match value {
            CompactPattern::Normal => self.boolean(true),
            CompactPattern::Pattern(id) => {
                self.boolean(false)?;
                self.id(&maps.signed_patterns, id)
            }
        }
    }
    fn token(&mut self, token: Token, maps: &TableMaps) -> Result<(), InvalidNumberProfile> {
        let (tag, text) = match token {
            Token::Literal(text) => (0, Some(text)),
            Token::Number => (1, None),
            Token::MinusSign => (2, None),
            Token::PlusSign => (3, None),
            Token::PercentSign => (4, None),
            Token::Currency => (5, None),
            Token::Compact(text) => (6, Some(text)),
            Token::Unit(text) => (7, Some(text)),
            Token::Argument1 => (8, None),
        };
        self.u8(tag)?;
        if let Some(text) = text {
            self.id(&maps.texts, text)?;
        }
        Ok(())
    }
    fn rules(&mut self, rules: &CardinalRules) -> Result<(), InvalidNumberProfile> {
        self.list(&rules.0, |writer, rule| {
            writer.category(rule.category)?;
            writer.list(&rule.alternatives, |writer, conjunction| {
                writer.list(conjunction, |writer, relation| {
                    writer.u8(match relation.operand {
                        PluralOperand::N => 0,
                        PluralOperand::I => 1,
                        PluralOperand::V => 2,
                        PluralOperand::W => 3,
                        PluralOperand::F => 4,
                        PluralOperand::T => 5,
                        PluralOperand::C => 6,
                        PluralOperand::E => 7,
                    })?;
                    writer.u64(relation.modulus.map_or(0, core::num::NonZeroU64::get))?;
                    writer.boolean(relation.integer_only)?;
                    writer.boolean(relation.negate)?;
                    writer.list(&relation.ranges, |writer, &(lower, upper)| {
                        writer.u64(lower)?;
                        writer.u64(upper)
                    })
                })
            })
        })
    }
}

pub(super) fn encode(
    source: &NumberProfiles,
    maps: &TableMaps,
    locales: &[usize],
    currency_codes: Option<&[CurrencyCode]>,
    numbering_systems: Option<&[crate::number_format::NumberingSystemOption]>,
) -> Result<Vec<u8>, InvalidNumberProfile> {
    let mut writer = Writer {
        bytes: Vec::new(),
        table: NumberProfileTable::Header,
        index: 0,
    };
    writer.bytes(if numbering_systems.is_some() {
        b"LNF47\0\x03\0"
    } else {
        b"LNF47\0\x02\0"
    })?;
    writer.table(&maps.texts, &source.texts, |writer, value| {
        writer.text(value)
    })?;
    writer.table(&maps.patterns, &source.patterns, |writer, value| {
        writer.list(&value.0, |writer, &token| writer.token(token, maps))
    })?;
    writer.table(
        &maps.signed_patterns,
        &source.signed_patterns,
        |writer, value| {
            writer.id(&maps.patterns, value.positive)?;
            writer.id(&maps.patterns, value.negative)?;
            writer.boolean(value.has_number)?;
            writer.u8(value.grouping.primary)?;
            writer.u8(value.grouping.secondary)
        },
    )?;
    writer.table(
        &maps.pattern_choices,
        &source.pattern_choices,
        |writer, value| writer.variants(value, |writer, id| writer.id(&maps.patterns, id)),
    )?;
    writer.table(
        &maps.string_choices,
        &source.string_choices,
        |writer, value| writer.variants(value, |writer, id| writer.id(&maps.texts, id)),
    )?;
    writer.table(
        &maps.compact_choices,
        &source.compact_choices,
        |writer, value| writer.variants(value, |writer, value| writer.compact_pattern(value, maps)),
    )?;
    writer.table(&maps.symbols, &source.symbols, |writer, value| {
        for text in [
            value.decimal,
            value.group,
            value.plus,
            value.minus,
            value.percent,
            value.approximately,
            value.exponent,
            value.infinity,
            value.nan,
            value.currency_decimal,
            value.currency_group,
        ] {
            writer.id(&maps.texts, text)?;
        }
        Ok(())
    })?;
    writer.table(&maps.unicode_sets, &source.unicode_sets, |writer, value| {
        writer.list(&value.0, |writer, &(start, end)| {
            writer.u32(start)?;
            writer.u32(end)
        })
    })?;
    writer.table(&maps.compact_sets, &source.compact_sets, |writer, value| {
        writer.list(&value.rows, |writer, row| {
            writer.u32(row.magnitude)?;
            writer.u32(row.exponent)?;
            writer.id(&maps.compact_choices, row.choices)
        })
    })?;
    writer.table(
        &maps.numbering_profiles,
        &source.numbering_profiles,
        |writer, value| {
            writer.id(&maps.symbols, value.symbols)?;
            for pattern in [
                value.decimal,
                value.percent,
                value.currency,
                value.accounting,
                value.currency_alpha,
                value.accounting_alpha,
            ] {
                writer.id(&maps.signed_patterns, pattern)?;
            }
            writer.u8(value.minimum_grouping)?;
            for compact in [
                value.compact_short,
                value.compact_long,
                value.compact_currency,
                value.compact_currency_alpha,
            ] {
                writer.id(&maps.compact_sets, compact)?;
            }
            writer.id(&maps.patterns, value.range_pattern)?;
            writer.id(&maps.pattern_choices, value.currency_unit_pattern)?;
            writer.spacing(&value.before_currency, maps)?;
            writer.spacing(&value.after_currency, maps)?;
            writer.filtered_list(
                &value.currency_overrides,
                |row| currency_selected(currency_codes, row.code),
                |writer, row| {
                    writer.bytes(&row.code)?;
                    writer.option(row.decimal, |writer, id| writer.id(&maps.texts, id))?;
                    writer.option(row.group, |writer, id| writer.id(&maps.texts, id))?;
                    writer.option(row.pattern, |writer, id| {
                        writer.id(&maps.signed_patterns, id)
                    })
                },
            )
        },
    )?;
    writer.table(
        &maps.currency_sets,
        &source.currency_sets,
        |writer, value| {
            writer.filtered_list(
                &value.0,
                |row| currency_selected(currency_codes, row.code),
                |writer, row| {
                    writer.bytes(&row.code)?;
                    writer.id(&maps.texts, row.symbol)?;
                    writer.id(&maps.texts, row.narrow)?;
                    writer.id(&maps.string_choices, row.names)
                },
            )
        },
    )?;
    writer.table(&maps.unit_sets, &source.unit_sets, |writer, value| {
        writer.list(&value.simple, |writer, row| {
            writer.id(&maps.pattern_choices, row.choices)?;
            writer.option(row.per_unit, |writer, id| writer.id(&maps.patterns, id))?;
            writer.id(&maps.patterns, row.denominator)
        })?;
        writer.list(&value.pairs, |writer, row| {
            writer.u8(row.numerator)?;
            writer.u8(row.denominator)?;
            writer.id(&maps.pattern_choices, row.choices)
        })?;
        writer.id(&maps.patterns, value.per_pattern)
    })?;
    writer.table(&maps.plural_rules, &source.plural_rules, Writer::rules)?;
    writer.table(&maps.ordinal_rules, &source.ordinal_rules, Writer::rules)?;
    writer.table(
        &maps.plural_ranges,
        &source.plural_ranges,
        |writer, value| {
            for category in value {
                writer.category(*category)?;
            }
            Ok(())
        },
    )?;
    writer.table(&maps.profiles, &source.profiles, |writer, value| {
        writer.u8(value.default_numbering)?;
        writer.length(value.numbering.len())?;
        for (index, &id) in value.numbering.iter().enumerate() {
            if numbering_selected(source, value, index, numbering_systems) {
                writer.id(
                    &maps.numbering_profiles,
                    id.ok_or_else(|| {
                        error(
                            NumberProfileTable::Profiles,
                            index,
                            NumberProfileError::Index,
                        )
                    })?,
                )?;
            } else {
                writer.u32(u32::MAX)?;
            }
        }
        writer.id(&maps.plural_rules, value.plural_rules)?;
        writer.id(&maps.ordinal_rules, value.ordinal_rules)?;
        writer.id(&maps.plural_ranges, value.plural_ranges)?;
        writer.id(&maps.currency_sets, value.currencies)?;
        for unit in value.units {
            writer.id(&maps.unit_sets, unit)?;
        }
        Ok(())
    })?;
    writer.table = NumberProfileTable::Locales;
    writer.index = 0;
    writer.list(locales, |writer, &index| {
        writer.text(&source.locales[index])?;
        writer.id(&maps.profiles, source.locale_profiles[index])
    })?;
    // These are global inventories, not locale labels. Every projected profile
    // keeps the complete system-index domain, unit-index domain and fractions.
    writer.table = NumberProfileTable::NumberingSystems;
    writer.list(&source.systems, |writer, system| {
        writer.text(&system.name)?;
        for digit in system.digits {
            writer.u32(u32::from(digit))?;
        }
        Ok(())
    })?;
    writer.table = NumberProfileTable::Units;
    writer.list(&source.unit_names, |writer, name| writer.text(name))?;
    writer.table = NumberProfileTable::CurrencyFractions;
    writer.u8(source.fractions.default.get())?;
    writer.list(&source.fractions.overrides, |writer, row| {
        writer.bytes(&row.code)?;
        writer.u8(row.digits.get())
    })?;
    for root in [source.letter_set, source.digit_set, source.whitespace_set] {
        writer.id(&maps.unicode_sets, root)?;
    }
    Ok(writer.bytes)
}
