//! A checked Generator adapter consumes the one original Object element algorithm.
use super::pattern_target::PatternContinuation;
use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_generator_object_pattern(
        &mut self,
        source: GeneratorObjectPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_staged_generator_object_pattern_with_storage_names(source, value, mode, None)
    }
    pub(super) fn lower_staged_generator_object_pattern_with_storage_names(
        &mut self,
        source: GeneratorObjectPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_resumable_object_pattern_elements(
            source.pattern(),
            PatternContinuation::Generator,
            value,
            mode,
            storage_names,
        )
    }
}
