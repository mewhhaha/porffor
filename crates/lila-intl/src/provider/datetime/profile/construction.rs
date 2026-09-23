use super::super::pattern::{Field, GlueToken, Token};
use super::*;

impl Locale {
    pub(super) fn from_raw(
        mut raw: raw::Locale,
        digits: &BTreeMap<String, [char; 10]>,
        algorithmic: &BTreeMap<String, Vec<String>>,
    ) -> Result<Self, DateTimeFormatError> {
        let identifier = CanonicalLocaleId::from_data(raw.locale.clone())
            .map_err(|_| invalid("invalid profile locale"))?;
        if raw.parent_chain.first().map(String::as_str)
            != Some(raw.locale.replace('-', "_").as_str())
            || raw.parent_chain.last().map(String::as_str) != Some("root")
            || !digits.contains_key(&raw.default_numbering)
            || raw.territory.is_empty()
            || (raw.default_content && raw.parent_chain.len() < 2)
        {
            return Err(invalid("invalid locale inheritance/defaults"));
        }
        let default_calendar = raw
            .calendar_preferences
            .iter()
            .find_map(|calendar| DateTimeCalendar::parse(calendar))
            .ok_or_else(|| invalid("no admitted default calendar"))?;
        let hour_cycle = match raw.preferred_hour {
            'h' => DateTimeHourCycle::H12,
            'H' => DateTimeHourCycle::H23,
            'K' => DateTimeHourCycle::H11,
            'k' => DateTimeHourCycle::H24,
            _ => return Err(invalid("unknown preferred hour cycle")),
        };
        let cycles = raw
            .allowed_hours
            .iter()
            .map(|hour| match hour.as_str() {
                "h" | "hb" | "hB" => Ok(DateTimeHourCycle::H12),
                "H" | "Hb" | "HB" => Ok(DateTimeHourCycle::H23),
                "K" | "Kb" | "KB" => Ok(DateTimeHourCycle::H11),
                "k" | "kb" | "kB" => Ok(DateTimeHourCycle::H24),
                _ => Err(invalid("unknown allowed hour symbol")),
            })
            .collect::<Result<Vec<_>, _>>()?;
        let hour_cycle12 = cycles
            .iter()
            .copied()
            .find(|cycle| cycle.is_twelve_hour())
            .ok_or_else(|| invalid("missing twelve-hour preference"))?;
        let hour_cycle24 = cycles
            .iter()
            .copied()
            .find(|cycle| !cycle.is_twelve_hour())
            .ok_or_else(|| invalid("missing twenty-four-hour preference"))?;
        let decimal = symbol_table(raw.decimal_separators, digits)?;
        let minus = symbol_table(raw.minus_signs, digits)?;
        let gregorian = Calendar::from_raw(
            raw.calendars
                .remove("gregorian")
                .ok_or_else(|| invalid("missing Gregorian patterns"))?,
            digits,
            algorithmic,
        )?;
        let chinese = Calendar::from_raw(
            raw.calendars
                .remove("chinese")
                .ok_or_else(|| invalid("missing Chinese patterns"))?,
            digits,
            algorithmic,
        )?;
        if !raw.calendars.is_empty() {
            return Err(invalid("unexpected calendar profile"));
        }
        Ok(Self {
            identifier,
            territory: raw.territory,
            default_numbering: raw.default_numbering,
            decimal,
            minus,
            default_calendar,
            hour_cycle,
            hour_cycle12,
            hour_cycle24,
            periods: PeriodRules::from_raw(raw.day_period_rules)?,
            gregorian,
            chinese,
            zones: raw.zone_names,
        })
    }
}

fn symbol_table(
    rows: Vec<(String, String)>,
    digits: &BTreeMap<String, [char; 10]>,
) -> Result<BTreeMap<String, String>, DateTimeFormatError> {
    let mut table = BTreeMap::new();
    for (identifier, value) in rows {
        if value.is_empty()
            || !digits.contains_key(&identifier)
            || table.insert(identifier, value).is_some()
        {
            return Err(invalid("invalid localized numeric symbol"));
        }
    }
    if table.len() != digits.len() {
        return Err(invalid("incomplete localized numeric symbols"));
    }
    Ok(table)
}

impl Calendar {
    fn from_raw(
        mut raw: raw::Calendar,
        digits: &BTreeMap<String, [char; 10]>,
        algorithmic: &BTreeMap<String, Vec<String>>,
    ) -> Result<Self, DateTimeFormatError> {
        if !matches!(raw.calendar.as_str(), "gregorian" | "chinese")
            || raw.available.is_empty()
            || raw.intervals.is_empty()
            || raw.excluded_non_ecma_formats.iter().any(|row| {
                row.path.is_empty()
                    || row.reason.is_empty()
                    || !row.skeleton.chars().any(|c| "YuUqQwWDFgA".contains(c))
            })
        {
            return Err(invalid("invalid calendar inventory"));
        }
        let mut styles = Vec::new();
        for style in ["full", "long", "medium", "short"] {
            let raw = raw
                .styles
                .remove(style)
                .ok_or_else(|| invalid("missing date/time style"))?;
            styles.push(Style {
                date: pattern(raw.date, digits, algorithmic)?,
                time: pattern(raw.time, digits, algorithmic)?,
                standard: glue(raw.standard)?,
                at_time: glue(raw.at_time)?,
            });
        }
        if !raw.styles.is_empty() {
            return Err(invalid("unknown date/time style"));
        }
        let mut available = Vec::new();
        let mut skeletons = BTreeSet::new();
        for row in raw.available {
            if !skeletons.insert(row.skeleton.clone())
                || row.skeleton.is_empty()
                || !row.skeleton.bytes().all(|byte| byte.is_ascii_alphabetic())
            {
                return Err(invalid("invalid or duplicate available skeleton"));
            }
            available.push(
                pattern(
                    raw::Pattern {
                        source: row.source,
                        tokens: row.tokens,
                        numbering_overrides: row.numbering_overrides,
                    },
                    digits,
                    algorithmic,
                )?
                .with_skeleton(&row.skeleton)?,
            );
        }
        let mut intervals = Vec::new();
        let mut keys = BTreeSet::new();
        for row in raw.intervals {
            if row.skeleton.is_empty()
                || !"GyMdahHmsB".contains(row.greatest_difference)
                || !keys.insert((row.skeleton.clone(), row.greatest_difference))
            {
                return Err(invalid("invalid or duplicate interval selector"));
            }
            let pattern = pattern(
                raw::Pattern {
                    source: row.source,
                    tokens: row.tokens,
                    numbering_overrides: row.numbering_overrides,
                },
                digits,
                algorithmic,
            )?
            .with_skeleton(&row.skeleton)?;
            if row.second_start == 0
                || row.second_start >= pattern.tokens.len()
                || !matches!(pattern.tokens[row.second_start], Token::Field(_))
            {
                return Err(invalid("invalid interval endpoint split"));
            }
            let field_names = |tokens: &[Token]| -> BTreeSet<&'static str> {
                tokens
                    .iter()
                    .filter_map(|token| match token {
                        Token::Field(field) => Some(field.part().as_str()),
                        Token::Literal(_) => None,
                    })
                    .collect()
            };
            let left = field_names(&pattern.tokens[..row.second_start]);
            let right = field_names(&pattern.tokens[row.second_start..]);
            if left
                .symmetric_difference(&right)
                .copied()
                .collect::<Vec<_>>()
                != row.shared_fields
            {
                return Err(invalid("interval shared fields disagree with its split"));
            }
            let latest_first = match row.endpoint_order.as_str() {
                "earliest_first" => false,
                "latest_first" => true,
                _ => return Err(invalid("invalid interval endpoint order")),
            };
            intervals.push(Interval {
                difference: row.greatest_difference,
                pattern,
                second_start: row.second_start,
                latest_first,
            });
        }
        Ok(Self {
            names: FieldNames::from_raw(raw.names)?,
            styles: styles
                .try_into()
                .map_err(|_| invalid("invalid style count"))?,
            available,
            intervals,
            interval_fallback: glue(raw.interval_fallback)?,
            append_zone: glue(raw.append_zone)?,
            append_era: raw.append_era.map(glue).transpose()?,
        })
    }
}

fn numbering(
    rows: Vec<raw::NumberingOverride>,
    digits: &BTreeMap<String, [char; 10]>,
    algorithmic: &BTreeMap<String, Vec<String>>,
) -> Result<Vec<(Option<char>, String)>, DateTimeFormatError> {
    let mut keys = BTreeSet::new();
    let mut result = Vec::new();
    for row in rows {
        if !keys.insert(row.field)
            || (!digits.contains_key(&row.numbering)
                && !(row.field == Some('d') && algorithmic.contains_key(&row.numbering)))
        {
            return Err(invalid("unsupported pattern numbering override"));
        }
        result.push((row.field, row.numbering));
    }
    Ok(result)
}

fn pattern(
    raw: raw::Pattern,
    digits: &BTreeMap<String, [char; 10]>,
    algorithmic: &BTreeMap<String, Vec<String>>,
) -> Result<Pattern, DateTimeFormatError> {
    if raw.source.is_empty() {
        return Err(invalid("empty source pattern"));
    }
    let mut tokens = Vec::new();
    for token in raw.tokens {
        tokens.push(match token {
            raw::Token::Literal(token) if !token.literal.is_empty() => {
                Token::Literal(token.literal)
            }
            raw::Token::Field(token) => Token::Field(Field::from_ldml(token.field, token.width)?),
            _ => return Err(invalid("unexpected token in field pattern")),
        });
    }
    if !tokens.iter().any(|token| matches!(token, Token::Field(_))) {
        return Err(invalid("pattern has no date/time field"));
    }
    Ok(Pattern::new(
        tokens,
        numbering(raw.numbering_overrides, digits, algorithmic)?,
    ))
}
fn glue(raw: raw::Pattern) -> Result<Glue, DateTimeFormatError> {
    if raw.source.is_empty() || !raw.numbering_overrides.is_empty() {
        return Err(invalid("invalid connector source"));
    }
    let mut tokens = Vec::new();
    let mut seen = [false; 2];
    for token in raw.tokens {
        tokens.push(match token {
            raw::Token::Literal(token) if !token.literal.is_empty() => {
                GlueToken::Literal(token.literal)
            }
            raw::Token::Placeholder(token)
                if token.placeholder < 2 && !seen[usize::from(token.placeholder)] =>
            {
                seen[usize::from(token.placeholder)] = true;
                if token.placeholder == 0 {
                    GlueToken::First
                } else {
                    GlueToken::Second
                }
            }
            _ => return Err(invalid("invalid connector token or duplicate placeholder")),
        });
    }
    if seen != [true, true] {
        return Err(invalid("connector is missing a placeholder"));
    }
    Ok(Glue { tokens })
}
