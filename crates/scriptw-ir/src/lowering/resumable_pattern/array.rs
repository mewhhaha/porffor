use super::*;
use boa_ast::pattern::{ArrayPattern, ArrayPatternElement};

impl ScriptLowerer<'_> {
    pub(in crate::lowering) fn lower_resumable_array_pattern_elements(
        &mut self,
        source: &ArrayPattern,
        execution: PatternContinuation,
        iterator: &ArrayIteratorStorageIr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<Vec<StatementIr>> {
        let mut body = Vec::new();
        for element in source.bindings() {
            match element {
                ArrayPatternElement::Elision => {
                    self.invalidate_unknown_user_code_effects();
                    body.push(ArrayDestructuringOperationIr::elision(iterator));
                }
                ArrayPatternElement::SingleName {
                    ident,
                    default_init,
                } => {
                    let target =
                        self.retain_pattern_identifier(*ident, mode, storage_names, &mut body);
                    let value = self.acquire_resumable_array_pattern_value(
                        execution,
                        iterator,
                        default_init.as_ref(),
                        &mut body,
                    )?;
                    self.put_pattern_target(execution, target, value, &mut body)?;
                }
                ArrayPatternElement::PropertyAccess {
                    access,
                    default_init,
                } => {
                    if mode.is_some() {
                        return None;
                    }
                    let target = self.retain_pattern_member(execution, access, &mut body)?;
                    let value = self.acquire_resumable_array_pattern_value(
                        execution,
                        iterator,
                        default_init.as_ref(),
                        &mut body,
                    )?;
                    self.put_pattern_target(execution, target, value, &mut body)?;
                }
                ArrayPatternElement::Pattern {
                    pattern,
                    default_init,
                } => {
                    let value = self.acquire_resumable_array_pattern_value(
                        execution,
                        iterator,
                        default_init.as_ref(),
                        &mut body,
                    )?;
                    self.put_pattern_target(
                        execution,
                        RetainedPatternTarget::Nested {
                            pattern,
                            mode,
                            storage_names: storage_names.cloned(),
                        },
                        value,
                        &mut body,
                    )?;
                }
                ArrayPatternElement::SingleNameRest { ident } => {
                    let target =
                        self.retain_pattern_identifier(*ident, mode, storage_names, &mut body);
                    let value =
                        self.acquire_resumable_array_rest(execution, iterator, &mut body)?;
                    self.put_pattern_target(execution, target, value, &mut body)?;
                }
                ArrayPatternElement::PropertyAccessRest { access } => {
                    if mode.is_some() {
                        return None;
                    }
                    let target = self.retain_pattern_member(execution, access, &mut body)?;
                    let value =
                        self.acquire_resumable_array_rest(execution, iterator, &mut body)?;
                    self.put_pattern_target(execution, target, value, &mut body)?;
                }
                ArrayPatternElement::PatternRest { pattern } => {
                    let value =
                        self.acquire_resumable_array_rest(execution, iterator, &mut body)?;
                    self.put_pattern_target(
                        execution,
                        RetainedPatternTarget::Nested {
                            pattern,
                            mode,
                            storage_names: storage_names.cloned(),
                        },
                        value,
                        &mut body,
                    )?;
                }
            }
        }
        Some(body)
    }
    fn acquire_resumable_array_pattern_value(
        &mut self,
        execution: PatternContinuation,
        iterator: &ArrayIteratorStorageIr,
        default: Option<&Expression>,
        body: &mut Vec<StatementIr>,
    ) -> Option<TypedExpr> {
        self.invalidate_unknown_user_code_effects();
        let result = owned_pattern_binding(
            self,
            execution.name(
                "generator.array.pattern.value.",
                "async.array.pattern.value.",
                "async.generator.array.pattern.value.",
            ),
            ValueInfo::new(ValueKind::Dynamic),
        );
        let (prefix, value) = ArrayDestructuringOperationIr::step_value(
            iterator,
            result,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        body.extend(prefix);
        if let Some(default) = default {
            self.apply_resumable_pattern_default(execution, &value, default, body)?;
        }
        Some(value)
    }
    fn acquire_resumable_array_rest(
        &mut self,
        execution: PatternContinuation,
        iterator: &ArrayIteratorStorageIr,
        body: &mut Vec<StatementIr>,
    ) -> Option<TypedExpr> {
        self.invalidate_unknown_user_code_effects();
        let result = owned_pattern_binding(
            self,
            execution.name(
                "generator.array.pattern.rest.",
                "async.array.pattern.rest.",
                "async.generator.array.pattern.rest.",
            ),
            ValueInfo::new(ValueKind::Array),
        );
        let (prefix, value) = ArrayDestructuringOperationIr::rest_array(
            iterator,
            result,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        body.extend(prefix);
        Some(value)
    }
}
