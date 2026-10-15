use super::*;

impl ProgramInstanceImage<'_> {
    pub(crate) fn require_pool(&self, strings: &StringPool) -> Result<(), EmitError> {
        if std::ptr::eq(self.strings, strings) {
            Ok(())
        } else {
            Err(EmitError::unsupported(
                "ProgramInstance resources require their actual checked source pool",
            ))
        }
    }
}

impl ProgramLiteralIndex {
    pub(crate) fn for_pool(
        strings: &StringPool,
        index: PooledStringIndex,
    ) -> Result<Self, EmitError> {
        let base = strings.compiler_owned_boundary().strings();
        let count = strings
            .pooled_string_count()
            .checked_sub(base)
            .ok_or_else(|| EmitError::unsupported("P literals do not retain the runtime prefix"))?;
        index
            .ordinal()
            .checked_sub(base)
            .filter(|ordinal| *ordinal < count)
            .map(Self)
            .ok_or_else(|| EmitError::unsupported("literal does not belong to this P image"))
    }

    pub(crate) const fn ordinal(self) -> u32 {
        self.0
    }
}
