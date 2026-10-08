//! A checked Generator adapter consumes the one original Array element algorithm.
use super::pattern_target::{owned_pattern_binding, retain_pattern_value, PatternContinuation};
use super::*;
use crate::generator_loop_control::GeneratorLoopRegionIr;

impl ScriptLowerer<'_> {
    pub(super) fn lower_staged_generator_array_pattern(
        &mut self,
        source: GeneratorArrayPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        self.lower_staged_generator_array_pattern_with_storage_names(source, value, mode, None)
    }
    pub(super) fn lower_staged_generator_array_pattern_with_storage_names(
        &mut self,
        source: GeneratorArrayPatternSource<'_>,
        value: TypedExpr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let states = source.states(self.plain_generator_entry_state()?)?;
        let mut prefix = Vec::new();
        let mut result =
            retain_pattern_value(self, &mut prefix, "generator.array.pattern.raw.", value);
        let iterator = owned_pattern_binding(
            self,
            "generator.array.pattern.iterator.",
            ValueInfo::undefined(),
        );
        prefix.push(StatementIr::Lexical {
            mode: BindingMode::Let,
            name: iterator.name.clone(),
            init: TypedExpr::undefined(),
        });
        let iterator =
            ArrayIteratorStorageIr::new(iterator, &self.generated_owned_env_bindings).ok()?;
        let body_range = states.body();
        self.current_generator_resume_state = Some(body_range.entry);
        let statements = self.lower_resumable_array_pattern_elements(
            source.pattern(),
            PatternContinuation::Generator,
            &iterator,
            mode,
            storage_names,
        )?;
        if self.plain_generator_entry_state()? != body_range.end {
            return None;
        }
        let body = GeneratorLoopRegionIr::new(
            BlockIr {
                statements,
                result_kind: ValueKind::Undefined,
                lexical_environment: None,
            },
            body_range,
        )
        .ok()?;
        let exit = states.exit();
        let plan = OrdinaryGeneratorArrayDestructuringIr::new(
            states,
            result.clone(),
            iterator,
            body,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        prefix.push(StatementIr::OrdinaryGeneratorArrayDestructuring(Box::new(
            plan,
        )));
        self.current_generator_resume_state = Some(exit);
        result.heap_shape = None;
        Some((prefix, result))
    }
}
