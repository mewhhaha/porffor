//! A checked Async adapter consumes the one original Object element algorithm.
use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_async_object_pattern_with_storage_names(
        &mut self,
        source: AsyncObjectPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let mut source_end = self.plain_async_entry_state()?;
        source.append(&mut source_end, &mut Vec::new())?;
        let result = self.lower_resumable_object_pattern_elements(
            source.pattern(),
            PatternContinuation::Async,
            value,
            mode,
            storage_names,
        )?;
        if self.plain_async_entry_state()? != source_end {
            return None;
        }
        Some(result)
    }
}
