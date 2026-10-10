use super::*;
use boa_ast::pattern::{ObjectPattern, ObjectPatternElement};

impl ScriptLowerer<'_> {
    pub(in crate::lowering) fn lower_resumable_object_pattern_elements(
        &mut self,
        pattern: &ObjectPattern,
        execution: PatternContinuation,
        value: TypedExpr,
        mode: Option<BindingMode>,
        storage_names: Option<&BTreeMap<String, String>>,
    ) -> Option<(Vec<StatementIr>, TypedExpr)> {
        let raw = owned_pattern_binding(
            self,
            execution.name(
                "generator.pattern.raw.",
                "async.pattern.raw.",
                "async.generator.pattern.raw.",
            ),
            value.value_info(),
        );
        let boxed = owned_pattern_binding(
            self,
            execution.name(
                "generator.pattern.boxed.",
                "async.pattern.boxed.",
                "async.generator.pattern.boxed.",
            ),
            TypedExpr::spec_to_object(TypedExpr::undefined()).value_info(),
        );
        let (mut statements, prepared) = ObjectDestructuringSourceIr::prepare(
            value,
            raw,
            boxed,
            &self.generated_owned_env_bindings,
        )
        .ok()?;
        let mut excluded = Vec::new();
        for element in pattern.bindings() {
            match element {
                ObjectPatternElement::SingleName {
                    name,
                    ident,
                    default_init,
                } => {
                    let key = self.lower_resumable_pattern_key(execution, name, &mut statements)?;
                    let target = self.retain_pattern_identifier(
                        *ident,
                        mode,
                        storage_names,
                        &mut statements,
                    );
                    let value = self.acquire_resumable_object_pattern_value(
                        execution,
                        &prepared,
                        &key,
                        default_init.as_ref(),
                        &mut statements,
                    )?;
                    self.put_pattern_target(execution, target, value, &mut statements)?;
                    excluded.push(key);
                }
                ObjectPatternElement::AssignmentPropertyAccess {
                    name,
                    access,
                    default_init,
                } => {
                    if mode.is_some() {
                        return None;
                    }
                    let key = self.lower_resumable_pattern_key(execution, name, &mut statements)?;
                    let target = self.retain_pattern_member(execution, access, &mut statements)?;
                    let value = self.acquire_resumable_object_pattern_value(
                        execution,
                        &prepared,
                        &key,
                        default_init.as_ref(),
                        &mut statements,
                    )?;
                    self.put_pattern_target(execution, target, value, &mut statements)?;
                    excluded.push(key);
                }
                ObjectPatternElement::Pattern {
                    name,
                    pattern,
                    default_init,
                } => {
                    let key = self.lower_resumable_pattern_key(execution, name, &mut statements)?;
                    let value = self.acquire_resumable_object_pattern_value(
                        execution,
                        &prepared,
                        &key,
                        default_init.as_ref(),
                        &mut statements,
                    )?;
                    self.put_pattern_target(
                        execution,
                        RetainedPatternTarget::Nested {
                            pattern,
                            mode,
                            storage_names: storage_names.cloned(),
                        },
                        value,
                        &mut statements,
                    )?;
                    excluded.push(key);
                }
                ObjectPatternElement::RestProperty { ident } => {
                    let target = self.retain_pattern_identifier(
                        *ident,
                        mode,
                        storage_names,
                        &mut statements,
                    );
                    let rest = ObjectDestructuringOperationIr::rest(&prepared, &excluded).ok()?;
                    let value = retain_pattern_value(
                        self,
                        &mut statements,
                        execution.name(
                            "generator.pattern.rest.",
                            "async.pattern.rest.",
                            "async.generator.pattern.rest.",
                        ),
                        rest.into_expr(),
                    );
                    self.put_pattern_target(execution, target, value, &mut statements)?;
                }
                ObjectPatternElement::AssignmentRestPropertyAccess { access } => {
                    if mode.is_some() {
                        return None;
                    }
                    let target = self.retain_pattern_member(execution, access, &mut statements)?;
                    let rest = ObjectDestructuringOperationIr::rest(&prepared, &excluded).ok()?;
                    let value = retain_pattern_value(
                        self,
                        &mut statements,
                        execution.name(
                            "generator.pattern.rest.",
                            "async.pattern.rest.",
                            "async.generator.pattern.rest.",
                        ),
                        rest.into_expr(),
                    );
                    self.put_pattern_target(execution, target, value, &mut statements)?;
                }
            }
        }
        let mut result = prepared.raw_receiver().clone();
        result.heap_shape = None;
        Some((statements, result))
    }
    fn lower_resumable_pattern_key(
        &mut self,
        execution: PatternContinuation,
        source: &PropertyName,
        statements: &mut Vec<StatementIr>,
    ) -> Option<ObjectDestructuringKeyIr> {
        let raw = match source {
            PropertyName::Literal(name) => TypedExpr::from_info(
                ValueInfo::new(ValueKind::String),
                ExprIr::String(self.interned_runtime_string(name.sym())),
            ),
            PropertyName::Computed(source) => {
                let (prefix, value) = self.lower_pattern_expression(execution, source)?;
                statements.extend(prefix);
                value
            }
        };
        self.record_possible_to_primitive_effects(&raw.value_info());
        let binding = owned_pattern_binding(
            self,
            execution.name(
                "generator.pattern.key.",
                "async.pattern.key.",
                "async.generator.pattern.key.",
            ),
            TypedExpr::spec_to_property_key(TypedExpr::undefined()).value_info(),
        );
        let (initialization, key) =
            ObjectDestructuringKeyIr::prepare(raw, binding, &self.generated_owned_env_bindings)
                .ok()?;
        statements.push(initialization);
        Some(key)
    }
    fn acquire_resumable_object_pattern_value(
        &mut self,
        execution: PatternContinuation,
        source: &ObjectDestructuringSourceIr,
        key: &ObjectDestructuringKeyIr,
        default: Option<&Expression>,
        statements: &mut Vec<StatementIr>,
    ) -> Option<TypedExpr> {
        self.invalidate_unknown_user_code_effects();
        let read = ObjectDestructuringOperationIr::get_v(source, key).ok()?;
        let value = retain_pattern_value(
            self,
            statements,
            execution.name(
                "generator.pattern.value.",
                "async.pattern.value.",
                "async.generator.pattern.value.",
            ),
            read.into_expr(),
        );
        if let Some(default) = default {
            self.apply_resumable_pattern_default(execution, &value, default, statements)?;
        }
        Some(value)
    }
}
