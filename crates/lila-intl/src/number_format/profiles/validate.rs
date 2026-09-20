use super::*;
use crate::number_format::options::SingleUnit;
use crate::CanonicalLocaleId;

#[derive(Clone, Copy)]
enum PatternRole {
    Decimal,
    Percent,
    Currency,
    CompactDecimal,
    CompactCurrency,
    Message,
    Unit,
    Denominator,
    PerUnit,
    Per,
}

fn error(
    table: NumberProfileTable,
    index: usize,
    reason: NumberProfileError,
) -> InvalidNumberProfile {
    InvalidNumberProfile {
        table,
        index: index as u32,
        reason,
    }
}

fn require(
    condition: bool,
    table: NumberProfileTable,
    index: usize,
    reason: NumberProfileError,
) -> Result<(), InvalidNumberProfile> {
    if condition {
        Ok(())
    } else {
        Err(error(table, index, reason))
    }
}

fn ordered<T: Ord>(values: impl IntoIterator<Item = T>) -> bool {
    let mut previous = None;
    for value in values {
        if previous.as_ref().is_some_and(|prior| *prior >= value) {
            return false;
        }
        previous = Some(value);
    }
    true
}

impl NumberProfiles {
    fn pattern_role(&self, id: PatternId, role: PatternRole) -> Result<(), InvalidNumberProfile> {
        let table = NumberProfileTable::Patterns;
        let pattern = self
            .patterns
            .get(id.0)
            .ok_or_else(|| error(table, id.0, NumberProfileError::Index))?;
        let mut number = 0;
        let mut argument = 0;
        let mut currency = 0;
        let mut percent = 0;
        for token in &pattern.0 {
            let allowed = match token {
                Token::Number => {
                    number += 1;
                    true
                }
                Token::Argument1 => {
                    argument += 1;
                    matches!(role, PatternRole::Message | PatternRole::Per)
                }
                Token::Currency => {
                    currency += 1;
                    matches!(role, PatternRole::Currency | PatternRole::CompactCurrency)
                }
                Token::PercentSign => {
                    percent += 1;
                    matches!(role, PatternRole::Percent)
                }
                Token::MinusSign | Token::PlusSign => matches!(
                    role,
                    PatternRole::Decimal
                        | PatternRole::Percent
                        | PatternRole::Currency
                        | PatternRole::CompactDecimal
                        | PatternRole::CompactCurrency
                ),
                Token::Compact(_) => matches!(
                    role,
                    PatternRole::CompactDecimal | PatternRole::CompactCurrency
                ),
                Token::Unit(_) => matches!(
                    role,
                    PatternRole::Unit
                        | PatternRole::Denominator
                        | PatternRole::PerUnit
                        | PatternRole::Per
                ),
                Token::Literal(_) => true,
            };
            require(allowed, table, id.0, NumberProfileError::PatternRole)?;
        }
        let cardinality = match role {
            PatternRole::Decimal => number == 1 && argument == 0 && currency == 0 && percent == 0,
            PatternRole::Percent => number == 1 && argument == 0 && currency == 0 && percent == 1,
            PatternRole::Currency => number == 1 && argument == 0 && currency == 1 && percent == 0,
            PatternRole::CompactDecimal => {
                number <= 1 && argument == 0 && currency == 0 && percent == 0
            }
            PatternRole::CompactCurrency => {
                number <= 1 && argument == 0 && currency == 1 && percent == 0
            }
            PatternRole::Message | PatternRole::Per => {
                number == 1 && argument == 1 && currency == 0 && percent == 0
            }
            PatternRole::Unit => number <= 1 && argument == 0 && currency == 0 && percent == 0,
            PatternRole::PerUnit => number == 1 && argument == 0 && currency == 0 && percent == 0,
            PatternRole::Denominator => {
                number == 0 && argument == 0 && currency == 0 && percent == 0
            }
        };
        require(cardinality, table, id.0, NumberProfileError::PatternRole)
    }

    fn signed_role(
        &self,
        id: SignedPatternId,
        role: PatternRole,
    ) -> Result<(), InvalidNumberProfile> {
        let row = self.signed_patterns.get(id.0).ok_or_else(|| {
            error(
                NumberProfileTable::SignedPatterns,
                id.0,
                NumberProfileError::Index,
            )
        })?;
        self.pattern_role(row.positive, role)?;
        self.pattern_role(row.negative, role)
    }

    fn choice_role(
        &self,
        id: PatternChoicesId,
        role: PatternRole,
    ) -> Result<(), InvalidNumberProfile> {
        let rows = self.pattern_choices.get(id.0).ok_or_else(|| {
            error(
                NumberProfileTable::PatternChoices,
                id.0,
                NumberProfileError::Index,
            )
        })?;
        for pattern in rows.all() {
            self.pattern_role(pattern, role)?;
        }
        Ok(())
    }

    fn compact_role(
        &self,
        id: CompactSetId,
        role: PatternRole,
    ) -> Result<(), InvalidNumberProfile> {
        let rows = self.compact_sets.get(id.0).ok_or_else(|| {
            error(
                NumberProfileTable::CompactSets,
                id.0,
                NumberProfileError::Index,
            )
        })?;
        for row in &rows.rows {
            for selected in self.compact_choices[row.choices.0].all() {
                match selected {
                    CompactPattern::Normal => {}
                    CompactPattern::Pattern(pattern) => self.signed_role(pattern, role)?,
                }
            }
        }
        Ok(())
    }

    pub(super) fn validate(&self) -> Result<(), InvalidNumberProfile> {
        for (index, pattern) in self.patterns.iter().enumerate() {
            for token in &pattern.0 {
                match token {
                    Token::Literal(text) | Token::Compact(text) | Token::Unit(text) => require(
                        text.0 < self.texts.len(),
                        NumberProfileTable::Patterns,
                        index,
                        NumberProfileError::Index,
                    )?,
                    Token::Number
                    | Token::MinusSign
                    | Token::PlusSign
                    | Token::PercentSign
                    | Token::Currency
                    | Token::Argument1 => {}
                }
            }
        }
        for (index, row) in self.signed_patterns.iter().enumerate() {
            let table = NumberProfileTable::SignedPatterns;
            require(
                row.positive.0 < self.patterns.len() && row.negative.0 < self.patterns.len(),
                table,
                index,
                NumberProfileError::Index,
            )?;
            require(
                (row.grouping.primary == 0) == (row.grouping.secondary == 0),
                table,
                index,
                NumberProfileError::InvalidRange,
            )?;
            for pattern in [row.positive, row.negative] {
                let count = self
                    .pattern(pattern)
                    .0
                    .iter()
                    .filter(|token| matches!(token, Token::Number))
                    .count();
                require(
                    count == usize::from(row.has_number),
                    table,
                    index,
                    NumberProfileError::PatternRole,
                )?;
            }
        }
        for (index, rows) in self.pattern_choices.iter().enumerate() {
            require(
                rows.all().all(|id| id.0 < self.patterns.len()),
                NumberProfileTable::PatternChoices,
                index,
                NumberProfileError::Index,
            )?;
        }
        for (index, rows) in self.string_choices.iter().enumerate() {
            require(
                rows.all().all(|id| id.0 < self.texts.len()),
                NumberProfileTable::StringChoices,
                index,
                NumberProfileError::Index,
            )?;
        }
        for (index, rows) in self.compact_choices.iter().enumerate() {
            require(
                rows.all().all(|pattern| match pattern {
                    CompactPattern::Normal => true,
                    CompactPattern::Pattern(id) => id.0 < self.signed_patterns.len(),
                }),
                NumberProfileTable::CompactChoices,
                index,
                NumberProfileError::Index,
            )?;
        }
        for (index, row) in self.symbols.iter().enumerate() {
            require(
                [
                    row.decimal,
                    row.group,
                    row.plus,
                    row.minus,
                    row.percent,
                    row.approximately,
                    row.exponent,
                    row.infinity,
                    row.nan,
                    row.currency_decimal,
                    row.currency_group,
                ]
                .iter()
                .all(|id| id.0 < self.texts.len()),
                NumberProfileTable::Symbols,
                index,
                NumberProfileError::Index,
            )?;
        }
        for (index, ranges) in self.unicode_sets.iter().enumerate() {
            let valid = ranges
                .0
                .iter()
                .all(|&(start, end)| start <= end && end <= 0x10ffff)
                && ranges.0.windows(2).all(|pair| pair[0].1 < pair[1].0);
            require(
                valid,
                NumberProfileTable::UnicodeSets,
                index,
                NumberProfileError::InvalidRange,
            )?;
        }
        for (index, set) in self.compact_sets.iter().enumerate() {
            require(
                !set.rows.is_empty(),
                NumberProfileTable::CompactSets,
                index,
                NumberProfileError::Cardinality,
            )?;
            for (magnitude, row) in set.rows.iter().enumerate() {
                require(
                    usize::try_from(row.magnitude).ok() == Some(magnitude)
                        && row.exponent <= row.magnitude,
                    NumberProfileTable::CompactSets,
                    index,
                    NumberProfileError::Order,
                )?;
                require(
                    row.choices.0 < self.compact_choices.len(),
                    NumberProfileTable::CompactSets,
                    index,
                    NumberProfileError::Index,
                )?;
            }
        }
        for (index, row) in self.numbering_profiles.iter().enumerate() {
            let table = NumberProfileTable::NumberingProfiles;
            require(
                row.symbols.0 < self.symbols.len(),
                table,
                index,
                NumberProfileError::Index,
            )?;
            require(
                (1..=3).contains(&row.minimum_grouping),
                table,
                index,
                NumberProfileError::InvalidRange,
            )?;
            self.signed_role(row.decimal, PatternRole::Decimal)?;
            self.signed_role(row.percent, PatternRole::Percent)?;
            for pattern in [
                row.currency,
                row.accounting,
                row.currency_alpha,
                row.accounting_alpha,
            ] {
                self.signed_role(pattern, PatternRole::Currency)?;
            }
            for set in [row.compact_short, row.compact_long] {
                self.compact_role(set, PatternRole::CompactDecimal)?;
            }
            for set in [row.compact_currency, row.compact_currency_alpha] {
                self.compact_role(set, PatternRole::CompactCurrency)?;
            }
            let compact = self.compact(row.compact_currency);
            let alpha = self.compact(row.compact_currency_alpha);
            require(
                compact.exponents == alpha.exponents,
                table,
                index,
                NumberProfileError::InvalidRange,
            )?;
            self.pattern_role(row.range_pattern, PatternRole::Message)?;
            self.choice_role(row.currency_unit_pattern, PatternRole::Message)?;
            for spacing in [&row.before_currency, &row.after_currency] {
                require(
                    spacing.currency.0 < self.unicode_sets.len()
                        && spacing.surrounding.0 < self.unicode_sets.len()
                        && spacing.insert.0 < self.texts.len(),
                    table,
                    index,
                    NumberProfileError::Index,
                )?;
            }
            require(
                ordered(row.currency_overrides.iter().map(|value| value.code)),
                table,
                index,
                NumberProfileError::Order,
            )?;
            for value in &row.currency_overrides {
                require(
                    [value.decimal, value.group]
                        .into_iter()
                        .flatten()
                        .all(|id| id.0 < self.texts.len()),
                    table,
                    index,
                    NumberProfileError::Index,
                )?;
                if let Some(pattern) = value.pattern {
                    self.signed_role(pattern, PatternRole::Currency)?;
                }
            }
        }
        for (index, set) in self.currency_sets.iter().enumerate() {
            let table = NumberProfileTable::CurrencySets;
            require(
                !set.0.is_empty() && ordered(set.0.iter().map(|row| row.code)),
                table,
                index,
                NumberProfileError::Order,
            )?;
            for row in &set.0 {
                require(
                    row.symbol.0 < self.texts.len()
                        && row.narrow.0 < self.texts.len()
                        && row.names.0 < self.string_choices.len(),
                    table,
                    index,
                    NumberProfileError::Index,
                )?;
            }
        }
        for (index, set) in self.unit_sets.iter().enumerate() {
            let table = NumberProfileTable::UnitSets;
            require(
                set.simple.len() == SingleUnit::ALL.len(),
                table,
                index,
                NumberProfileError::Cardinality,
            )?;
            for row in &set.simple {
                self.choice_role(row.choices, PatternRole::Unit)?;
                if let Some(pattern) = row.per_unit {
                    self.pattern_role(pattern, PatternRole::PerUnit)?;
                }
                self.pattern_role(row.denominator, PatternRole::Denominator)?;
            }
            require(
                ordered(set.pairs.iter().map(|row| (row.numerator, row.denominator))),
                table,
                index,
                NumberProfileError::Order,
            )?;
            for row in &set.pairs {
                require(
                    usize::from(row.numerator) < set.simple.len()
                        && usize::from(row.denominator) < set.simple.len(),
                    table,
                    index,
                    NumberProfileError::Index,
                )?;
                self.choice_role(row.choices, PatternRole::Unit)?;
            }
            self.pattern_role(set.per_pattern, PatternRole::Per)?;
        }
        for (index, rules) in self.plural_rules.iter().enumerate() {
            let mut seen = [false; 6];
            for rule in &rules.0 {
                let table = NumberProfileTable::PluralRules;
                require(
                    !seen[rule.category.index()],
                    table,
                    index,
                    NumberProfileError::Order,
                )?;
                seen[rule.category.index()] = true;
                require(
                    rule.alternatives.is_empty() == (rule.category == CardinalCategory::Other),
                    table,
                    index,
                    NumberProfileError::Cardinality,
                )?;
                for conjunction in &rule.alternatives {
                    require(
                        !conjunction.is_empty(),
                        table,
                        index,
                        NumberProfileError::Cardinality,
                    )?;
                    for relation in conjunction {
                        require(
                            !relation.ranges.is_empty()
                                && relation.ranges.iter().all(|&(lower, upper)| lower <= upper),
                            table,
                            index,
                            NumberProfileError::InvalidRange,
                        )?;
                    }
                }
            }
        }
        require(
            self.systems.len() == 77 && ordered(self.system_names.iter().copied()),
            NumberProfileTable::NumberingSystems,
            0,
            NumberProfileError::Cardinality,
        )?;
        for (index, system) in self.systems.iter().enumerate() {
            require(
                (3..=8).contains(&system.name.len())
                    && system
                        .name
                        .bytes()
                        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()),
                NumberProfileTable::NumberingSystems,
                index,
                NumberProfileError::UnknownCode,
            )?;
            for position in 0..10 {
                require(
                    !system.digits[..position].contains(&system.digits[position]),
                    NumberProfileTable::NumberingSystems,
                    index,
                    NumberProfileError::Order,
                )?;
            }
        }
        require(
            self.unit_names.len() == SingleUnit::ALL.len()
                && self
                    .unit_names
                    .iter()
                    .zip(SingleUnit::ALL)
                    .all(|(name, unit)| *name == unit.name()),
            NumberProfileTable::Units,
            0,
            NumberProfileError::Cardinality,
        )?;
        for (index, profile) in self.profiles.iter().enumerate() {
            let table = NumberProfileTable::Profiles;
            require(
                profile.numbering.len() == self.systems.len()
                    && usize::from(profile.default_numbering) < self.systems.len(),
                table,
                index,
                NumberProfileError::Cardinality,
            )?;
            require(
                profile
                    .numbering
                    .iter()
                    .all(|id| id.0 < self.numbering_profiles.len())
                    && profile.plural_rules.0 < self.plural_rules.len()
                    && profile.plural_ranges.0 < self.plural_ranges.len()
                    && profile.currencies.0 < self.currency_sets.len()
                    && profile.units.iter().all(|id| id.0 < self.unit_sets.len()),
                table,
                index,
                NumberProfileError::Index,
            )?;
        }
        require(
            self.locales.len() == self.locale_profiles.len()
                && ordered(self.locales.iter().copied()),
            NumberProfileTable::Locales,
            0,
            NumberProfileError::Order,
        )?;
        require(
            self.locales.binary_search(&"en-US").is_ok(),
            NumberProfileTable::Locales,
            0,
            NumberProfileError::MissingDefaultLocale,
        )?;
        for (index, (&locale, profile)) in self
            .locales
            .iter()
            .zip(self.locale_profiles.iter())
            .enumerate()
        {
            let table = NumberProfileTable::Locales;
            require(
                profile.0 < self.profiles.len(),
                table,
                index,
                NumberProfileError::Index,
            )?;
            let mut owned = String::new();
            owned
                .try_reserve_exact(locale.len())
                .map_err(|_| error(table, index, NumberProfileError::Allocation))?;
            owned.push_str(locale);
            require(
                !locale.split('-').any(|part| part.len() == 1)
                    && CanonicalLocaleId::from_data(owned.into_boxed_str()).is_ok(),
                table,
                index,
                NumberProfileError::InvalidLocale,
            )?;
        }
        require(
            ordered(self.fractions.overrides.iter().map(|row| row.code)),
            NumberProfileTable::CurrencyFractions,
            0,
            NumberProfileError::Order,
        )?;
        require(
            [self.letter_set, self.digit_set, self.whitespace_set]
                .iter()
                .all(|id| id.0 < self.unicode_sets.len()),
            NumberProfileTable::UnicodeSets,
            0,
            NumberProfileError::Index,
        )?;
        Ok(())
    }
}
