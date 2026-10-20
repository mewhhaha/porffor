//! A checked Async adapter consumes the one original Array element algorithm.
use super::super::pattern_target::{owned_pattern_binding, retain_pattern_value};
use super::*;

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_async_array_pattern_with_storage_names(
        &mut self,
        source: AsyncArrayPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let states = source.states(self.plain_async_entry_state()?)?;
        let mut prefix = Vec::new();
        let mut result = retain_pattern_value(self, &mut prefix, "async.array.pattern.raw.", value);
        let iterator = owned_pattern_binding(
            self,
            "async.array.pattern.iterator.",
            ValueInfo::undefined(),
        );
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: iterator.name.clone(),
            init: TypedExpr::undefined(),
        });
        let iterator =
            ArrayIteratorStorageIr::new(iterator, &self.generated_owned_env_bindings).ok()?;
        let body_entry = states.body_entry();
        let body_end = states.body_end();
        self.current_async_resume_state = Some(body_entry);
        let statements = self.lower_resumable_array_pattern_elements(
            source.pattern(),
            PatternContinuation::Async,
            &iterator,
            mode,
            storage_names,
        )?;
        if self.plain_async_entry_state()? != body_end {
            return None;
        }
        let body = BlockIr {
            statements,
            result_kind: ValueKind::Undefined,
            lexical_environment: None,
        };
        let exit = states.exit();
        let plan = AsyncFunctionArrayDestructuringIr::new(
            states,
            result.clone(),
            iterator,
            body,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        prefix.push(StatementIr::AsyncFunctionArrayDestructuring(Box::new(plan)));
        self.current_async_resume_state = Some(exit);
        result.heap_shape = None;
        Some((prefix, result))
    }
}
