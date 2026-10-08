//! Original-order dense maps retain the actual typed table domain at every edge.
use super::*;
use core::marker::PhantomData;

pub(super) trait TableIndex: Copy {
    const TABLE: NumberProfileTable;
    fn index(self) -> usize;
}

struct Selection<I> {
    rows: Vec<bool>,
    domain: PhantomData<I>,
}

impl<I: TableIndex> Selection<I> {
    fn new(length: usize, all: bool) -> Result<Self, InvalidNumberProfile> {
        let mut rows = Vec::new();
        rows.try_reserve_exact(length)
            .map_err(|_| error(I::TABLE, 0, NumberProfileError::Allocation))?;
        rows.resize(length, all);
        Ok(Self {
            rows,
            domain: PhantomData,
        })
    }

    fn mark(&mut self, id: I) -> bool {
        // Every ID comes from PinnedNumberSource's complete table admission.
        let marked = &mut self.rows[id.index()];
        let fresh = !*marked;
        *marked = true;
        fresh
    }

    fn finish(self) -> Result<Remap<I>, InvalidNumberProfile> {
        let mut rows = Vec::new();
        rows.try_reserve_exact(self.rows.len())
            .map_err(|_| error(I::TABLE, 0, NumberProfileError::Allocation))?;
        let mut count = 0usize;
        for selected in self.rows {
            rows.push(if selected {
                let mapped = u32::try_from(count)
                    .map_err(|_| error(I::TABLE, count, NumberProfileError::Index))?;
                count += 1;
                Some(mapped)
            } else {
                None
            });
        }
        Ok(Remap {
            rows,
            count,
            domain: PhantomData,
        })
    }
}

pub(super) struct Remap<I> {
    rows: Vec<Option<u32>>,
    count: usize,
    domain: PhantomData<I>,
}

impl<I: TableIndex> Remap<I> {
    pub(super) fn get(&self, id: I) -> Result<u32, InvalidNumberProfile> {
        self.rows
            .get(id.index())
            .copied()
            .flatten()
            .ok_or_else(|| error(I::TABLE, id.index(), NumberProfileError::Index))
    }
    pub(super) fn count(&self) -> usize {
        self.count
    }
    pub(super) fn old_indices(&self) -> impl Iterator<Item = usize> + '_ {
        self.rows
            .iter()
            .enumerate()
            .filter_map(|(index, value)| value.map(|_| index))
    }
}

macro_rules! tables {
    ($($field:ident : $id:ident => $table:ident),+ $(,)?) => {
        $(impl TableIndex for $id {
            const TABLE: NumberProfileTable = NumberProfileTable::$table;
            fn index(self) -> usize { self.0 }
        })+
        struct Closure { $($field: Selection<$id>,)+ }
        pub(super) struct TableMaps { $(pub(super) $field: Remap<$id>,)+ }
        impl Closure {
            fn new(profiles: &NumberProfiles, all: bool) -> Result<Self, InvalidNumberProfile> {
                Ok(Self { $($field: Selection::new(profiles.$field.len(), all)?,)+ })
            }
            fn finish(self) -> Result<TableMaps, InvalidNumberProfile> {
                Ok(TableMaps { $($field: self.$field.finish()?,)+ })
            }
        }
    };
}

tables!(
    texts: TextId => Strings,
    patterns: PatternId => Patterns,
    signed_patterns: SignedPatternId => SignedPatterns,
    pattern_choices: PatternChoicesId => PatternChoices,
    string_choices: StringChoicesId => StringChoices,
    compact_choices: CompactChoicesId => CompactChoices,
    symbols: SymbolId => Symbols,
    unicode_sets: UnicodeSetId => UnicodeSets,
    compact_sets: CompactSetId => CompactSets,
    numbering_profiles: NumberingProfileId => NumberingProfiles,
    currency_sets: CurrencySetId => CurrencySets,
    unit_sets: UnitSetId => UnitSets,
    plural_rules: PluralRulesId => PluralRules,
    ordinal_rules: OrdinalRulesId => OrdinalRules,
    plural_ranges: PluralRangeId => PluralRanges,
    profiles: ProfileId => Profiles,
);

impl TableMaps {
    pub(super) fn full(profiles: &NumberProfiles) -> Result<Self, InvalidNumberProfile> {
        Closure::new(profiles, true)?.finish()
    }
    pub(super) fn selected(
        profiles: &NumberProfiles,
        locales: &[usize],
        currency_codes: Option<&[CurrencyCode]>,
        numbering_systems: Option<&[crate::number_format::NumberingSystemOption]>,
    ) -> Result<Self, InvalidNumberProfile> {
        let mut closure = Closure::new(profiles, false)?;
        for &locale in locales {
            closure.profile(
                profiles,
                profiles.locale_profiles[locale],
                currency_codes,
                numbering_systems,
            );
        }
        for root in [
            profiles.letter_set,
            profiles.digit_set,
            profiles.whitespace_set,
        ] {
            closure.unicode_sets.mark(root);
        }
        closure.finish()
    }
}

impl Closure {
    fn profile(
        &mut self,
        source: &NumberProfiles,
        id: ProfileId,
        currency_codes: Option<&[CurrencyCode]>,
        numbering_systems: Option<&[crate::number_format::NumberingSystemOption]>,
    ) {
        if !self.profiles.mark(id) {
            return;
        }
        let row = &source.profiles[id.0];
        for (index, &numbering) in row.numbering.iter().enumerate() {
            if numbering_selected(source, row, index, numbering_systems) {
                self.numbering(
                    source,
                    numbering.expect("complete pinned source numbering association"),
                    currency_codes,
                );
            }
        }
        self.plural_rules.mark(row.plural_rules);
        self.ordinal_rules.mark(row.ordinal_rules);
        self.plural_ranges.mark(row.plural_ranges);
        self.currencies(source, row.currencies, currency_codes);
        for unit in row.units {
            self.units(source, unit);
        }
    }

    fn numbering(
        &mut self,
        source: &NumberProfiles,
        id: NumberingProfileId,
        currency_codes: Option<&[CurrencyCode]>,
    ) {
        if !self.numbering_profiles.mark(id) {
            return;
        }
        let row = &source.numbering_profiles[id.0];
        self.symbols(source, row.symbols);
        for signed in [
            row.decimal,
            row.percent,
            row.currency,
            row.accounting,
            row.currency_alpha,
            row.accounting_alpha,
        ] {
            self.signed(source, signed);
        }
        for compact in [
            row.compact_short,
            row.compact_long,
            row.compact_currency,
            row.compact_currency_alpha,
        ] {
            self.compact(source, compact);
        }
        self.pattern(source, row.range_pattern);
        self.pattern_choices(source, row.currency_unit_pattern);
        for spacing in [&row.before_currency, &row.after_currency] {
            self.unicode_sets.mark(spacing.currency);
            self.unicode_sets.mark(spacing.surrounding);
            self.texts.mark(spacing.insert);
        }
        for override_ in &row.currency_overrides {
            if !currency_selected(currency_codes, override_.code) {
                continue;
            }
            for text in [override_.decimal, override_.group].into_iter().flatten() {
                self.texts.mark(text);
            }
            if let Some(pattern) = override_.pattern {
                self.signed(source, pattern);
            }
        }
    }

    fn symbols(&mut self, source: &NumberProfiles, id: SymbolId) {
        if !self.symbols.mark(id) {
            return;
        }
        let row = &source.symbols[id.0];
        for text in [
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
        ] {
            self.texts.mark(text);
        }
    }

    fn signed(&mut self, source: &NumberProfiles, id: SignedPatternId) {
        if !self.signed_patterns.mark(id) {
            return;
        }
        let row = &source.signed_patterns[id.0];
        self.pattern(source, row.positive);
        self.pattern(source, row.negative);
    }

    fn pattern(&mut self, source: &NumberProfiles, id: PatternId) {
        if !self.patterns.mark(id) {
            return;
        }
        for token in &source.patterns[id.0].0 {
            match token {
                Token::Literal(text) | Token::Compact(text) | Token::Unit(text) => {
                    self.texts.mark(*text);
                }
                Token::Number
                | Token::MinusSign
                | Token::PlusSign
                | Token::PercentSign
                | Token::Currency
                | Token::Argument1 => {}
            }
        }
    }

    fn pattern_choices(&mut self, source: &NumberProfiles, id: PatternChoicesId) {
        if !self.pattern_choices.mark(id) {
            return;
        }
        for pattern in source.pattern_choices[id.0].all() {
            self.pattern(source, pattern);
        }
    }

    fn compact(&mut self, source: &NumberProfiles, id: CompactSetId) {
        if !self.compact_sets.mark(id) {
            return;
        }
        for row in &source.compact_sets[id.0].rows {
            if self.compact_choices.mark(row.choices) {
                for choice in source.compact_choices[row.choices.0].all() {
                    match choice {
                        CompactPattern::Normal => {}
                        CompactPattern::Pattern(pattern) => self.signed(source, pattern),
                    }
                }
            }
        }
    }

    fn currencies(
        &mut self,
        source: &NumberProfiles,
        id: CurrencySetId,
        currency_codes: Option<&[CurrencyCode]>,
    ) {
        if !self.currency_sets.mark(id) {
            return;
        }
        for row in &source.currency_sets[id.0].0 {
            if !currency_selected(currency_codes, row.code) {
                continue;
            }
            self.texts.mark(row.symbol);
            self.texts.mark(row.narrow);
            if self.string_choices.mark(row.names) {
                for text in source.string_choices[row.names.0].all() {
                    self.texts.mark(text);
                }
            }
        }
    }

    fn units(&mut self, source: &NumberProfiles, id: UnitSetId) {
        if !self.unit_sets.mark(id) {
            return;
        }
        let row = &source.unit_sets[id.0];
        for unit in &row.simple {
            self.pattern_choices(source, unit.choices);
            if let Some(pattern) = unit.per_unit {
                self.pattern(source, pattern);
            }
            self.pattern(source, unit.denominator);
        }
        for pair in &row.pairs {
            self.pattern_choices(source, pair.choices);
        }
        self.pattern(source, row.per_pattern);
    }
}
