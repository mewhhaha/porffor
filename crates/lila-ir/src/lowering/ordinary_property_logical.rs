use super::*;

impl<'a> ScriptLowerer<'a> {
    /// Lower one ordinary property logical assignment into a single consuming
    /// Reference carrier. The RHS is retained unevaluated by the backend until
    /// the selected logical branch; base and raw key have already been lowered
    /// in source order by the shared producer plan.
    pub(super) fn lower_ordinary_property_logical_assignment(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
        op: LogicalBinaryOp,
        rhs: &Expression,
    ) -> TypedExpr {
        let (plan, referenced_name, metadata) = self.lower_ordinary_property_reference_plan(access);
        self.record_ordinary_property_get(&metadata);
        let skipped_rhs = self.capture_conditional_flow_facts();
        let rhs_effect_accounting = self.prepare_potentially_effectful_expression(rhs);
        let before_rhs_effect_epoch = self.intervening_effect_epoch;
        let rhs = self.lower_expression(rhs);
        let rhs_may_have_intervening_effects = rhs_effect_accounting
            .intervening_effects_observed(before_rhs_effect_epoch, self.intervening_effect_epoch);
        if rhs_may_have_intervening_effects {
            self.observe_all_planned_source_as_unknown_property_hooks();
            self.invalidate_unknown_user_code_effects();
        }
        let taken_rhs = self.capture_conditional_flow_facts();
        self.merge_conditional_flow_facts(skipped_rhs, taken_rhs);
        let rhs_info = rhs.value_info();
        // The conditional transaction has already joined the skipped state
        // with every effect of the taken RHS. Preserve that whole pre-write
        // domain: a reflective RHS can mutate the property and then make the
        // outer Set fail, leaving its own value observable.
        let pre_write_global_value = self.pre_write_global_property_value(access, &referenced_name);
        let possible_getters = Self::possible_ordinary_property_getters(&metadata);
        let possible_setters =
            self.possible_ordinary_property_setters(&metadata, rhs_may_have_intervening_effects);
        let result = plan.logical_assignment(op, rhs, possible_getters, possible_setters);

        // Either the old property value or the RHS can be published. Until the
        // runtime branch and GetValue result are known, retained shape metadata
        // must admit the full result domain.
        let setter_may_call_user_code = self.record_ordinary_property_possible_write(
            &referenced_name,
            &metadata,
            rhs_may_have_intervening_effects,
            rhs_info.clone(),
        );
        if !setter_may_call_user_code {
            if let (PropertyKeyIr::StaticString(name), Some(pre_write_global_value)) =
                (&referenced_name, pre_write_global_value)
            {
                self.merge_possible_global_property_value_info(
                    name,
                    pre_write_global_value,
                    rhs_info,
                );
            }
        }
        result
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};

    fn lower(source: &str) -> ProgramIr {
        let source = parse(source, ParseOptions::script()).expect("script should parse");
        crate::lower(&source)
    }

    fn returned_assignment<'a>(
        script: &'a ScriptIr,
        function_name: &str,
    ) -> &'a OrdinaryPropertyLogicalAssignmentIr {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("missing function {function_name}"));
        let StatementIr::Return(value) = function
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("function should return its assignment")
        else {
            unreachable!("selected statement is a return")
        };
        let ExprIr::OrdinaryPropertyLogicalAssignment(assignment) = &value.expr else {
            panic!(
                "expected fused ordinary property logical assignment, got {:?}",
                value.expr
            );
        };
        assignment
    }

    fn returned_expression<'a>(script: &'a ScriptIr, function_name: &str) -> &'a TypedExpr {
        let function = script
            .functions
            .iter()
            .find(|function| function.name == function_name)
            .unwrap_or_else(|| panic!("missing function {function_name}"));
        let StatementIr::Return(value) = function
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("function should return a value")
        else {
            unreachable!("selected statement is a return")
        };
        value
    }

    #[test]
    fn ordinary_property_logical_assignment_owns_one_reference_and_branch_rhs() {
        let program = lower(
            r#"
            function andAssign(base, key, rhs) { "use strict"; return base[key] &&= rhs; }
            function orAssign(base, key, rhs) { "use strict"; return base[key] ||= rhs; }
            function coalesceAssign(base, key, rhs) { "use strict"; return base[key] ??= rhs; }
            "#,
        );
        let script = program.script.as_ref().expect("script IR should exist");

        for (name, expected_op) in [
            ("andAssign", LogicalBinaryOp::And),
            ("orAssign", LogicalBinaryOp::Or),
            ("coalesceAssign", LogicalBinaryOp::Coalesce),
        ] {
            let assignment = returned_assignment(script, name);
            assert!(matches!(
                &assignment.base_and_receiver().expr,
                ExprIr::Identifier(_)
            ));
            assert!(matches!(
                assignment.referenced_name(),
                PropertyKeyIr::StringExpr(key)
                    if matches!(&key.expr, ExprIr::Identifier(_))
            ));
            assert!(matches!(&assignment.rhs().expr, ExprIr::Identifier(_)));
            assert_eq!(assignment.op(), expected_op);
            assert_eq!(assignment.strictness(), Strictness::Strict);
        }
    }

    #[test]
    fn ordinary_property_logical_assignment_merges_followup_global_property_facts() {
        let program = lower(
            "globalThis.logicalFact = 0; globalThis.logicalFact ||= 's'; globalThis.logicalFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = &script.body.statements[2] else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "possible logical write must prevent Number-only lowering: {:?}",
            result.expr
        );
    }

    #[test]
    fn ordinary_property_logical_assignment_preserves_pre_rhs_global_property_facts() {
        let program = lower(
            "globalThis.logicalFact = 1; globalThis.logicalFact ||= (globalThis.logicalFact = 's'); globalThis.logicalFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = &script.body.statements[2] else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "the skipped RHS must not erase the pre-branch Number fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_global_merge_retains_taken_rhs_mutation_when_outer_set_can_fail() {
        let program = lower(
            "globalThis.logicalFailedOuterSetFact = 0; globalThis.logicalFailedOuterSetFact ||= (Object.defineProperty(globalThis, 'logicalFailedOuterSetFact', { value: 's', writable: false }), 2); globalThis.logicalFailedOuterSetFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "a failed outer Set can expose the taken RHS mutation: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_global_property_write_updates_the_script_var_mirror() {
        let program =
            lower("var logicalVarFact = 0; globalThis.logicalVarFact ||= 's'; logicalVarFact + 1;");
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "the script-global binding mirror must follow a property write: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_rhs_side_effects_join_unrelated_global_property_facts() {
        let program = lower(
            "globalThis.unrelatedFact = 1; const guard = { p: 1 }; guard.p ||= (globalThis.unrelatedFact = 's'); globalThis.unrelatedFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "a skipped RHS must not overwrite an unrelated global fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_assignment_invalidates_a_global_this_alias() {
        let program = lower(
            "globalThis.aliasFact = 1; const globalAlias = globalThis; globalAlias.aliasFact ||= 's'; globalThis.aliasFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "an exact globalThis alias must invalidate the canonical global fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_assignment_invalidates_a_joined_global_this_alias() {
        let program = lower(
            "function joinedAlias(flag) { globalThis.joinedAliasFact = 0; const target = flag ? globalThis : {}; target.joinedAliasFact ||= 's'; return globalThis.joinedAliasFact + 1; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let result = returned_expression(script, "joinedAlias");
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "a joined alias must invalidate the canonical global fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn dynamic_global_key_invalidates_every_possible_global_fact() {
        let program = lower(
            "function dynamicGlobalKey(key) { globalThis.dynamicKeyFact = 0; globalThis[key] ||= 's'; return globalThis.dynamicKeyFact + 1; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let result = returned_expression(script, "dynamicGlobalKey");
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "a dynamic global key must invalidate every writable fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn joined_shape_records_every_declared_accessor_receiver() {
        let program = lower(
            "globalThis.joinedAccessorValue = 'g'; var joinedAccessorTarget = { joinedAccessorValue: 1, get p() { return this.joinedAccessorValue + 1; } }; function readJoinedAccessor(flag) { return (flag ? joinedAccessorTarget : {}).p ||= 99; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let getter = script
            .functions
            .iter()
            .find(|function| function.protocol == FunctionProtocolIr::ObjectGetter)
            .expect("object getter should be lowered");
        let StatementIr::Return(result) = getter
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("getter should return a value")
        else {
            unreachable!("selected statement is a return")
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "a joined base must make the accessor receiver conservative: {:?}",
            result.expr
        );
    }

    #[test]
    fn joined_conditional_receiver_carries_each_builtin_getter() {
        for source in [
            "function readSize(flag, map, set) { return (flag ? map : set).size ||= 1; } readSize(true, new Map(), new Set());",
            "function readSize(flag, map, set) { return (flag ? map : set).size ||= 1; } readSize(true, new Map(null), new Set(undefined));",
        ] {
            let program = lower(source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let script = program.script.as_ref().expect("script IR should exist");
            let assignment = returned_assignment(script, "readSize");

            for getter in [
                StandardBuiltinId::MapPrototypeSizeGetter,
                StandardBuiltinId::SetPrototypeSizeGetter,
            ] {
                assert!(
                    assignment
                        .possible_getters()
                        .contains(&getter.function_id()),
                    "joined conditional receiver lost {getter:?} for {source}: {assignment:?}"
                );
            }
        }
    }

    #[test]
    fn dynamically_installed_primitive_getter_observes_the_primitive_receiver() {
        let program = lower(
            "Object.defineProperty(String.prototype, 'logicalPrimitiveAccessor', { configurable: true, get: function() { 'use strict'; return this + 1; } }); ('a').logicalPrimitiveAccessor ||= 99;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let getter = script
            .functions
            .iter()
            .find(|function| function.strict)
            .expect("descriptor getter should be lowered");
        let StatementIr::Return(result) = getter
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("getter should return a value")
        else {
            unreachable!("selected statement is a return")
        };
        assert!(
            matches!(
                result.expr,
                ExprIr::StringConcat { .. } | ExprIr::CoerciveAdd { .. }
            ),
            "the primitive receiver must prevent numeric-only getter lowering: {:?}",
            result.expr
        );
    }

    #[test]
    fn shadowed_global_this_does_not_restore_a_canonical_global_fact() {
        let program = lower(
            "globalThis.shadowedGlobalFact = 1; { let globalThis = {}; globalThis.shadowedGlobalFact = 's'; } globalThis.shadowedGlobalFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            !matches!(result.expr, ExprIr::StringConcat { .. }),
            "a shadowed globalThis write cannot install a canonical String fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn shadowed_number_constructor_does_not_refine_the_intrinsic_prototype() {
        let program = lower(
            "function shadowedNumber() { let Number = { prototype: {} }; Number.prototype.toString = Object.prototype.toString; return (1).toString(); }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let result = returned_expression(script, "shadowedNumber");
        assert!(
            !matches!(&result.expr, ExprIr::String(value) if value == "[object Number]"),
            "a shadowed Number write cannot refine the intrinsic prototype: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_assignment_invalidates_a_refined_number_prototype_alias() {
        let program = lower(
            "const numberPrototypeAlias = Number.prototype; numberPrototypeAlias.extra = 1; numberPrototypeAlias.toString &&= Object.prototype.toString; (1).toString();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up call");
        };
        assert!(
            matches!(
                &result.expr,
                ExprIr::MaterializeBinding { body, .. }
                    if matches!(body.expr, ExprIr::CallIndirect { .. })
            ),
            "a refined prototype alias must disable exact builtin selection: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_key_coercion_invalidates_global_facts_before_get() {
        let program = lower(
            "globalThis.keyEffectFact = 1; var key = { toString() { globalThis.keyEffectFact = 's'; return 'length'; } }; 'a'[key] ||= 2; globalThis.keyEffectFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "ToPropertyKey user code must invalidate the prior global fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_key_coercion_discards_the_getter_receivers_old_shape() {
        let program = lower(
            "globalThis.logicalKeyReceiverOut = 0; const target = { marker: 1, get p() { globalThis.logicalKeyReceiverOut = this.marker + 1; return 1; } }; const key = { toString() { target.marker = 's'; return 'p'; } }; target[key] ||= 2;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let getter = script
            .functions
            .iter()
            .find(|function| function.protocol == FunctionProtocolIr::ObjectGetter)
            .expect("object getter should be lowered");
        let StatementIr::Expression(write) = getter
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Expression(_)))
            .expect("getter should write the observed value")
        else {
            unreachable!("selected statement is an expression")
        };
        let value = match &write.expr {
            ExprIr::GlobalPropertyWrite { value, .. } => value.as_ref(),
            ExprIr::OrdinaryPropertyAssignment(assignment) => assignment.rhs(),
            _ => panic!("getter should write a global property: {:?}", write.expr),
        };
        assert!(
            matches!(value.expr, ExprIr::CoerciveAdd { .. }),
            "ToPropertyKey can change the receiver shape before [[Get]]: {:?}",
            value.expr
        );
    }

    #[test]
    fn logical_getter_effects_invalidate_global_and_prototype_facts() {
        let program = lower(
            "globalThis.getterEffectFact = 1; var target = { get p() { globalThis.getterEffectFact = 's'; Number.prototype.toString = Object.prototype.toString; return 1; } }; target.p ||= 2; globalThis.getterEffectFact + 1; (1).toString();",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(global_result) = &script.body.statements[3] else {
            panic!("expected follow-up addition");
        };
        assert!(matches!(global_result.expr, ExprIr::CoerciveAdd { .. }));
        let StatementIr::Expression(prototype_result) = &script.body.statements[4] else {
            panic!("expected follow-up prototype call");
        };
        assert!(
            !matches!(prototype_result.expr, ExprIr::String(_)),
            "getter effects must disable intrinsic Number.prototype folding: {:?}",
            prototype_result.expr
        );
    }

    #[test]
    fn logical_proxy_traps_invalidate_global_facts() {
        let program = lower(
            "globalThis.proxyEffectFact = 1; var proxy = new Proxy({ p: 1 }, { get(target, key) { globalThis.proxyEffectFact = 's'; return target[key]; } }); proxy.p ||= 2; globalThis.proxyEffectFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "Proxy get traps must invalidate the prior global fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn nested_descriptor_installation_on_a_known_shape_records_the_getter_receiver() {
        let program = lower(
            "var descriptorTarget = { value: 1 }; function descriptorGetter() { 'use strict'; return this.value + 1; } function installDescriptor() { Object.defineProperty(descriptorTarget, 'p', { get: descriptorGetter }); } installDescriptor(); descriptorTarget.p ||= 99;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let getter = script
            .functions
            .iter()
            .find(|function| function.name == "descriptorGetter")
            .expect("descriptor getter should be lowered");
        let StatementIr::Return(result) = getter
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("getter should return a value")
        else {
            unreachable!("selected statement is a return")
        };
        assert!(
            !matches!(result.expr, ExprIr::StringConcat { .. }),
            "descriptor getter must observe the installed target, not global this: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_carrier_retains_a_setter_installed_by_its_rhs() {
        let program = lower(
            "var seen; var target = { marker: 1, p: 0 }; function installPrototypeSetter() { return target.p ||= (Object.setPrototypeOf(target, { set p(value) { seen = this.marker + value; } }), delete target.p, 2); }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let setter = script
            .functions
            .iter()
            .find(|function| function.protocol == FunctionProtocolIr::ObjectSetter)
            .expect("RHS setter should be lowered");
        let assignment = returned_assignment(script, "installPrototypeSetter");
        assert!(
            assignment.possible_setters().contains(&setter.id),
            "post-RHS setter provenance must be carried into planning"
        );
    }

    #[test]
    fn logical_rhs_discards_the_setter_receivers_old_shape() {
        let program = lower(
            "globalThis.logicalSetterReceiverOut = 0; const prototype = { set p(value) { globalThis.logicalSetterReceiverOut = this.marker + 1; } }; const target = { __proto__: prototype, marker: 1 }; target.p ||= (target.marker = 's', 2);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let setter = script
            .functions
            .iter()
            .find(|function| function.protocol == FunctionProtocolIr::ObjectSetter)
            .expect("object setter should be lowered");
        let StatementIr::Expression(write) = setter.body.statements.last().unwrap() else {
            panic!("setter should write the observed value");
        };
        let value = match &write.expr {
            ExprIr::GlobalPropertyWrite { value, .. } => value.as_ref(),
            ExprIr::OrdinaryPropertyAssignment(assignment) => assignment.rhs(),
            _ => panic!("setter should write a global property: {:?}", write.expr),
        };
        assert!(
            matches!(value.expr, ExprIr::CoerciveAdd { .. }),
            "the RHS can change the receiver shape before [[Set]]: {:?}",
            value.expr
        );
    }

    #[test]
    fn reflective_structure_mutation_keeps_later_accessor_provenance() {
        let program = lower(
            r#"
            function prototypeGetter() { return 0; }
            function descriptorsGetter() { return 0; }
            function mutatePrototype() {
                let target = {};
                Object.setPrototypeOf(target, { get p() { return prototypeGetter(); } });
                return target.p ||= 1;
            }
            function defineProperties() {
                let target = {};
                Object.defineProperties(target, { p: { get: descriptorsGetter } });
                return target.p ||= 1;
            }
            "#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");

        let prototype_getter = script
            .functions
            .iter()
            .find(|function| function.protocol == FunctionProtocolIr::ObjectGetter)
            .expect("prototype getter should be lowered");
        assert!(
            returned_assignment(script, "mutatePrototype")
                .possible_getters()
                .contains(&prototype_getter.id),
            "Object.setPrototypeOf lost the installed getter"
        );

        let descriptors_getter = script
            .functions
            .iter()
            .find(|function| function.name == "descriptorsGetter")
            .expect("descriptor getter should be lowered");
        assert!(
            returned_assignment(script, "defineProperties")
                .possible_getters()
                .contains(&descriptors_getter.id),
            "Object.defineProperties lost the installed getter"
        );
    }

    #[test]
    fn delete_exposing_an_inherited_getter_invalidates_the_old_own_shape() {
        let program = lower(
            "globalThis.logicalDeleteShapeFact = 1; const prototype = { get q() { globalThis.logicalDeleteShapeFact = 's'; return 0; } }; const target = { __proto__: prototype, q: 1 }; delete target.q; target.q ||= 2; globalThis.logicalDeleteShapeFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "delete can expose an inherited getter with arbitrary effects: {:?}",
            result.expr
        );
    }

    #[test]
    fn direct_global_delete_invalidates_a_global_this_alias_shape() {
        let program = lower(
            "Object.defineProperty(globalThis, '__proto__', { value: 1, writable: true, configurable: true }); const alias = globalThis; delete globalThis.__proto__; alias.__proto__ ||= null;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let assignment = script
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::Expression(TypedExpr {
                    expr: ExprIr::OrdinaryPropertyLogicalAssignment(assignment),
                    ..
                }) => Some(assignment),
                _ => None,
            })
            .expect("logical assignment should be lowered");
        assert!(
            assignment
                .possible_getters()
                .contains(&StandardBuiltinId::ObjectPrototypeProtoGetter.function_id()),
            "delete must expose and retain the inherited __proto__ getter"
        );
    }

    #[test]
    fn destructuring_property_write_invalidates_a_later_reference_shape() {
        let program = lower(
            "globalThis.logicalDestructureShapeFact = 1; const prototype = { get q() { globalThis.logicalDestructureShapeFact = 's'; return 0; } }; const target = {}; ({ value: target.__proto__ } = { value: prototype }); target.q ||= 2; globalThis.logicalDestructureShapeFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "a destructuring property setter can change the later prototype: {:?}",
            result.expr
        );
    }

    #[test]
    fn logical_property_hooks_contribute_arbitrary_catch_values() {
        let program = lower(
            "var target = { get p() { throw 's'; } }; function catchGetterThrow() { 'use strict'; try { target.p ||= 1; } catch (error) { return typeof error; } }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let function = script
            .functions
            .iter()
            .find(|function| function.name == "catchGetterThrow")
            .expect("catch function should be lowered");
        let catch_block = function
            .body
            .statements
            .iter()
            .find_map(|statement| match statement {
                StatementIr::TryCatch { catch_block, .. } => Some(catch_block),
                _ => None,
            })
            .expect("expected try/catch");
        let StatementIr::Return(result) = &catch_block.statements[0] else {
            panic!("catch should return typeof error");
        };
        let ExprIr::TypeOf { expr: caught } = &result.expr else {
            panic!(
                "arbitrary thrown values must prevent typeof folding: {:?}",
                result.expr
            );
        };
        assert_eq!(caught.kind, ValueKind::Dynamic);
        assert_eq!(caught.possible_kinds, KindSet::all_runtime_tags());
    }

    #[test]
    fn reflective_object_results_do_not_invent_empty_shapes() {
        for source in [
            "let x = 1; const prototype = { get p() { x = 's'; return 0; } }; const object = Object.create(prototype); const result = Reflect.getPrototypeOf(object); result.p ||= 2; x + 1;",
            "let x = 1; const prototype = { get p() { x = 's'; return 0; } }; const object = Object.create(prototype); const result = object.valueOf(); result.p ||= 2; x + 1;",
            "let x = 1; function C() { return { get p() { x = 's'; return 0; } }; } const result = Reflect.construct(C, []); result.p ||= 2; x + 1;",
        ] {
            let program = lower(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
                panic!("expected follow-up addition");
            };
            assert!(
                matches!(result.expr, ExprIr::CoerciveAdd { .. }),
                "reflective object results must preserve possible accessors: {:?}",
                result.expr
            );
        }
    }

    #[test]
    fn logical_rhs_implicit_hooks_widen_later_branch_subexpressions() {
        let program = lower(
            "function outer() { let x = 'u'; const key = { get p() { x = 's'; return 0; } }; const base = { value: 0 }; return base.value ||= (x = 1, key.p, x + 1); }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let assignment = returned_assignment(script, "outer");
        assert!(
            matches!(
                &assignment.rhs().expr,
                ExprIr::Comma { rhs, .. } if matches!(rhs.expr, ExprIr::CoerciveAdd { .. })
            ),
            "the taken RHS must invalidate before lowering its later operands: {:?}",
            assignment.rhs().expr
        );
    }

    #[test]
    fn unobserved_sloppy_this_shape_does_not_hide_runtime_accessors() {
        let program = lower(
            "let x = 1; function f() { const self = this; return self.p ||= x; } const object = { get p() { x = 's'; return 0; } }; Reflect.apply(f, object, []);",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        assert_eq!(
            returned_assignment(script, "f").rhs().kind,
            ValueKind::Dynamic
        );
    }

    #[test]
    fn reflective_get_effects_reach_a_later_reference() {
        let program = lower(
            "function outer() { let x = 1; const object = { get p() { x = 's'; return 0; } }; Reflect.get(object, 'p'); const base = { q: 0 }; return base.q ||= x + 1; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        assert!(
            matches!(
                &returned_assignment(script, "outer").rhs().expr,
                ExprIr::CoerciveAdd { .. }
            ),
            "Reflect.get must invalidate captured flow facts before the later Reference"
        );
    }

    #[test]
    fn builtin_accessor_gets_retain_transitive_source_getters() {
        let program = lower(
            "function outer() { let x = 1; const object = { __proto__: RegExp.prototype, get global() { x = 's'; return false; } }; return object.flags ||= x; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let getter = script
            .functions
            .iter()
            .find(|function| function.protocol == FunctionProtocolIr::ObjectGetter)
            .expect("transitively read getter should be lowered");
        let assignment = returned_assignment(script, "outer");
        assert!(assignment.possible_getters().contains(&getter.id));
        assert_eq!(assignment.rhs().kind, ValueKind::Dynamic);
    }
}
