use super::*;

impl Pattern {
    pub(in crate::provider::datetime) fn with_range_pattern(
        mut self,
        companion: Pattern,
    ) -> Result<Self, DateTimeFormatError> {
        let fields = |pattern: &Pattern| {
            pattern
                .tokens
                .iter()
                .filter_map(|token| match token {
                    Token::Field(field) => Some(*field),
                    Token::Literal(_) => None,
                })
                .collect::<Vec<_>>()
        };
        if companion.range_pattern.is_some()
            || fields(&self) != fields(&companion)
            || self.numbering != companion.numbering
        {
            return Err(DateTimeFormatError::InvalidProfile(
                "range endpoint companion changes selected domain".into(),
            ));
        }
        self.range_pattern = Some(Box::new(companion));
        Ok(self)
    }
    pub(in crate::provider::datetime) fn range_endpoint(&self) -> &Self {
        self.range_pattern.as_deref().unwrap_or(self)
    }
    pub(in crate::provider::datetime) fn variants_mut(
        &mut self,
        mut apply: impl FnMut(&mut Pattern),
    ) {
        apply(self);
        if let Some(companion) = &mut self.range_pattern {
            apply(companion);
        }
    }
}
