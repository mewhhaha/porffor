use super::*;

pub(super) struct OrdinaryPropertyReferenceMetadata {
    base_value_info: ValueInfo,
    base_evaluation_may_have_intervening_effects: bool,
    key_may_call_user_code: bool,
    key_evaluation_may_have_intervening_effects: bool,
    unknown_property_hooks_possible: bool,
    getter_may_dispatch_transitive_property_hooks: bool,
    possible_getters: PropertyHookTargets,
    possible_setters: PropertyHookTargets,
}

impl<'a> ScriptLowerer<'a> {
    /// Lower the two evaluated operands and strictness which jointly identify
    /// one ordinary property Reference. The plan is non-cloneable; the cloned
    /// key is returned only for conservative shape invalidation.
    pub(super) fn lower_ordinary_property_reference_plan(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
    ) -> (
        OrdinaryPropertyReferencePlan,
        PropertyKeyIr,
        OrdinaryPropertyReferenceMetadata,
    ) {
        let base_evaluation_may_invoke_user_code =
            self.prepare_potentially_effectful_expression(access.target());
        let before_base_effect_epoch = self.intervening_effect_epoch;
        let base_and_receiver = Box::new(self.lower_property_target(access.target()));
        let base_evaluation_may_have_intervening_effects = base_evaluation_may_invoke_user_code
            || self.intervening_effect_epoch != before_base_effect_epoch;
        if base_evaluation_may_have_intervening_effects {
            self.observe_all_planned_source_as_unknown_property_hooks();
            self.invalidate_unknown_user_code_effects();
        }
        let key_evaluation_may_invoke_user_code = match access.field() {
            PropertyAccessField::Const(_) => false,
            PropertyAccessField::Expr(expression) => {
                self.prepare_potentially_effectful_expression(expression)
            }
        };
        let before_key_effect_epoch = self.intervening_effect_epoch;
        let referenced_name = match access.field() {
            PropertyAccessField::Const(name) => {
                let name = self.interner.resolve_expect(name.sym()).to_string();
                if base_and_receiver.kind == ValueKind::Array && name == "length" {
                    PropertyKeyIr::ArrayLength
                } else {
                    PropertyKeyIr::StaticString(name)
                }
            }
            PropertyAccessField::Expr(expression) => self
                .lower_static_property_key(expression)
                .unwrap_or_else(|| {
                    PropertyKeyIr::StringExpr(Box::new(self.lower_expression(expression)))
                }),
        };
        let key_evaluation_may_have_intervening_effects = key_evaluation_may_invoke_user_code
            || self.intervening_effect_epoch != before_key_effect_epoch;
        let (known_getters, known_setters) = match &referenced_name {
            PropertyKeyIr::StaticString(name) => {
                match self.read_object_shape_property(&base_and_receiver, name) {
                    Some(ObjectShapeProperty::Accessor { getter, setter }) => {
                        let getters = getter
                            .map(|getter| BTreeSet::from([getter.function_id]))
                            .unwrap_or_default();
                        let setters = setter
                            .map(|setter| BTreeSet::from([setter.function_id]))
                            .unwrap_or_default();
                        (getters, setters)
                    }
                    Some(ObjectShapeProperty::Data(_)) | None => (BTreeSet::new(), BTreeSet::new()),
                }
            }
            PropertyKeyIr::StringExpr(_) | PropertyKeyIr::ArrayIndex(_) => {
                Self::possible_shape_accessors(base_and_receiver.heap_shape.as_deref())
            }
            PropertyKeyIr::ArrayLength => (BTreeSet::new(), BTreeSet::new()),
        };
        let mut possible_getters = PropertyHookTargets::from_known(known_getters);
        let mut possible_setters = PropertyHookTargets::from_known(known_setters);
        let base_may_be_object = Self::value_info_may_be_object(&base_and_receiver.value_info());
        let key_may_call_user_code = Self::property_key_may_call_user_code(&referenced_name);
        let prior_unknown_effects = self.unknown_user_code_effects_observed;
        if base_and_receiver.heap_shape.is_none()
            || base_evaluation_may_have_intervening_effects
            || key_may_call_user_code
            || key_evaluation_may_have_intervening_effects
            || prior_unknown_effects
        {
            let (unknown_getters, unknown_setters) = self.possible_unknown_accessor_functions();
            possible_getters.extend_targets(unknown_getters);
            possible_setters.extend_targets(unknown_setters);
        }

        // EvaluatePropertyAccessWithExpressionKey applies ToPropertyKey after
        // the raw key expression and before GetValue. Object coercion can run
        // arbitrary source code, so discard facts it may mutate before the
        // logical assignment captures its skipped-branch state.
        if key_may_call_user_code || key_evaluation_may_have_intervening_effects {
            self.observe_all_planned_source_as_unknown_property_hooks();
            self.invalidate_unknown_user_code_effects();
        }
        possible_getters.extend_known(self.dynamically_installed_getters.iter().cloned());
        possible_setters.extend_known(self.dynamically_installed_setters.iter().cloned());
        let getter_may_dispatch_transitive_property_hooks = possible_getters
            .iter()
            .any(|getter| StandardBuiltinId::from_function_id(getter).is_some());
        if getter_may_dispatch_transitive_property_hooks {
            possible_getters
                .include_all_planned_source(self.analysis.planned_source_function_ids.clone());
        }

        let metadata = OrdinaryPropertyReferenceMetadata {
            base_value_info: base_and_receiver.value_info(),
            base_evaluation_may_have_intervening_effects,
            key_may_call_user_code,
            key_evaluation_may_have_intervening_effects,
            unknown_property_hooks_possible: base_may_be_object
                && (base_and_receiver.heap_shape.is_none()
                    || base_evaluation_may_have_intervening_effects
                    || key_may_call_user_code
                    || key_evaluation_may_have_intervening_effects
                    || prior_unknown_effects),
            getter_may_dispatch_transitive_property_hooks,
            possible_getters,
            possible_setters,
        };
        let plan = OrdinaryPropertyReferencePlan::new(
            base_and_receiver,
            referenced_name.clone(),
            self.reference_strictness(),
        );
        (plan, referenced_name, metadata)
    }

    pub(super) fn pre_write_global_property_value(
        &self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
        referenced_name: &PropertyKeyIr,
    ) -> Option<PreWriteGlobalPropertyValue> {
        match referenced_name {
            PropertyKeyIr::StaticString(name) if self.is_global_this_expr(access.target()) => {
                Some(self.capture_pre_write_global_property_value(name))
            }
            PropertyKeyIr::StaticString(_)
            | PropertyKeyIr::StringExpr(_)
            | PropertyKeyIr::ArrayIndex(_)
            | PropertyKeyIr::ArrayLength => None,
        }
    }

    pub(super) fn possible_ordinary_property_setters(
        &self,
        metadata: &OrdinaryPropertyReferenceMetadata,
        intervening_user_code: bool,
    ) -> PropertyHookTargets {
        let mut setters = metadata.possible_setters.clone();
        if intervening_user_code || self.unknown_user_code_effects_observed {
            let (_, unknown_setters) = self.possible_unknown_accessor_functions();
            setters.extend_targets(unknown_setters);
        }
        setters.extend_known(self.dynamically_installed_setters.iter().cloned());
        setters
    }

    pub(super) fn possible_ordinary_property_getters(
        metadata: &OrdinaryPropertyReferenceMetadata,
    ) -> PropertyHookTargets {
        metadata.possible_getters.clone()
    }

    /// A read/modify/write carrier always performs GetValue before deciding
    /// whether or how to write, so a statically known getter observes the
    /// original Reference receiver on every normal path.
    pub(super) fn record_ordinary_property_get(
        &mut self,
        metadata: &OrdinaryPropertyReferenceMetadata,
    ) {
        let mut receiver_info = metadata.base_value_info.clone();
        if metadata.base_evaluation_may_have_intervening_effects
            || metadata.key_may_call_user_code
            || metadata.key_evaluation_may_have_intervening_effects
        {
            // The Reference still owns the same receiver value, but
            // ToPropertyKey can mutate any reachable property before [[Get]].
            receiver_info.heap_shape = None;
        }
        for getter in metadata.possible_getters.iter() {
            if metadata.unknown_property_hooks_possible
                || metadata.getter_may_dispatch_transitive_property_hooks
            {
                self.observe_unknown_property_hook(getter);
            } else {
                self.observe_ordinary_property_hook_this(getter, receiver_info.clone());
                if let Some(signature) = self.function_signatures.get_mut(getter) {
                    Self::merge_omitted_signature_params_as_undefined(signature, 0);
                }
            }
        }
        if !metadata.possible_getters.is_empty() {
            self.invalidate_unknown_user_code_effects();
        }
    }

    /// Record every conservative compiler-state effect shared by a possible
    /// write through an ordinary property Reference.
    pub(super) fn record_ordinary_property_possible_write(
        &mut self,
        referenced_name: &PropertyKeyIr,
        metadata: &OrdinaryPropertyReferenceMetadata,
        intervening_user_code: bool,
        written_value_info: ValueInfo,
    ) -> bool {
        let base_may_be_object = Self::value_info_may_be_object(&metadata.base_value_info);
        if base_may_be_object {
            self.array_prototype_mutated = true;
        }

        if base_may_be_object {
            match referenced_name {
                PropertyKeyIr::StaticString(name) => {
                    self.invalidate_possible_global_property_value_info(name);
                    match name.as_str() {
                        "toString" => {
                            self.number_prototype_to_string_state = PrototypeToStringState::Unknown;
                            self.boolean_prototype_to_string_state =
                                PrototypeToStringState::Unknown;
                        }
                        "match" => self.number_prototype_match_is_string_match = false,
                        "split" => self.number_prototype_split_is_string_split = false,
                        _ => {}
                    }
                }
                PropertyKeyIr::StringExpr(_) | PropertyKeyIr::ArrayIndex(_) => {
                    self.invalidate_all_possible_global_property_value_infos();
                    self.number_prototype_to_string_state = PrototypeToStringState::Unknown;
                    self.number_prototype_match_is_string_match = false;
                    self.number_prototype_split_is_string_split = false;
                    self.boolean_prototype_to_string_state = PrototypeToStringState::Unknown;
                }
                PropertyKeyIr::ArrayLength => {}
            }
        }
        let possible_setters =
            self.possible_ordinary_property_setters(metadata, intervening_user_code);
        let mut receiver_info = metadata.base_value_info.clone();
        if metadata.key_may_call_user_code
            || metadata.base_evaluation_may_have_intervening_effects
            || metadata.key_evaluation_may_have_intervening_effects
            || intervening_user_code
        {
            // RHS evaluation and ToNumeric/ToPrimitive preserve the captured
            // receiver identity but can mutate its contents before [[Set]].
            receiver_info.heap_shape = None;
        }
        let unknown_setter_provenance = metadata.unknown_property_hooks_possible
            || intervening_user_code
            || self.unknown_user_code_effects_observed;
        for setter in possible_setters.iter() {
            if unknown_setter_provenance {
                self.observe_unknown_property_hook(setter);
            } else {
                self.observe_ordinary_property_hook_this(setter, receiver_info.clone());
                self.merge_function_param_infos(setter, std::slice::from_ref(&written_value_info));
                if let Some(signature) = self.function_signatures.get_mut(setter) {
                    Self::merge_omitted_signature_params_as_undefined(signature, 1);
                }
            }
        }
        let setter_may_call_user_code = metadata.unknown_property_hooks_possible
            || self.source_functions_may_run(&possible_setters);
        if setter_may_call_user_code {
            self.invalidate_unknown_user_code_effects();
        }
        self.invalidate_ordinary_property_shape_aliases(&metadata.base_value_info);
        setter_may_call_user_code
    }

    fn observe_ordinary_property_hook_this(
        &mut self,
        function_id: &FunctionId,
        receiver_info: ValueInfo,
    ) {
        let fallback = self
            .function_signatures
            .get(function_id)
            .map(|signature| signature.this_info.clone())
            .unwrap_or_else(|| receiver_info.clone());
        let receiver = TypedExpr::from_info(receiver_info, ExprIr::Undefined);
        let this_info =
            self.explicit_this_info_for_function_target(function_id, &receiver, fallback);
        self.merge_function_this_info(function_id, this_info);
    }

    fn observe_unknown_property_hook(&mut self, function_id: &FunctionId) {
        let unknown = unknown_runtime_value_info();
        self.merge_function_this_info(function_id, unknown.clone());
        self.merge_function_param_infos(
            function_id,
            &[unknown.clone(), unknown.clone(), unknown.clone(), unknown],
        );
        if let Some(signature) = self.function_signatures.get_mut(function_id) {
            Self::merge_omitted_signature_params_as_undefined(signature, 4);
        }
    }

    pub(super) fn observe_all_planned_source_as_unknown_property_hooks(&mut self) {
        let targets = self.analysis.planned_source_function_ids.clone();
        for target in targets.iter() {
            self.observe_unknown_property_hook(target);
        }
    }

    pub(super) fn reference_operand_evaluation_may_invoke_user_code(
        &self,
        expression: &Expression,
    ) -> bool {
        match Self::unwrap_parenthesized_expr(expression) {
            Expression::Literal(_) | Expression::This(_) => false,
            Expression::Identifier(identifier) => {
                let name = self.interner.resolve_expect(identifier.sym()).to_string();
                !self.with_environment_chain.is_empty()
                    || self.lookup_binding(&name).is_none()
                    || self
                        .var_bindings
                        .get(&name)
                        .is_some_and(|binding| binding.is_script_global)
            }
            _ => true,
        }
    }

    pub(super) fn prepare_potentially_effectful_expression(
        &mut self,
        expression: &Expression,
    ) -> bool {
        let may_invoke_user_code =
            self.reference_operand_evaluation_may_invoke_user_code(expression);
        if may_invoke_user_code {
            self.observe_all_planned_source_as_unknown_property_hooks();
            self.invalidate_unknown_user_code_effects();
        }
        may_invoke_user_code
    }

    pub(super) fn invalidate_ordinary_property_shape_aliases(&mut self, base: &ValueInfo) {
        let Some(base_shape) = base.heap_shape.as_ref() else {
            return;
        };
        self.intervening_effect_epoch = self.intervening_effect_epoch.saturating_add(1);

        fn contains_alias(shape: &HeapShape, alias: &HeapShape) -> bool {
            if shape == alias {
                return true;
            }

            let property_contains_alias = |property: &ObjectShapeProperty| match property {
                ObjectShapeProperty::Data(info) => info
                    .heap_shape
                    .as_deref()
                    .is_some_and(|shape| contains_alias(shape, alias)),
                ObjectShapeProperty::Accessor { .. } => false,
            };

            match shape {
                HeapShape::Object(shape) => {
                    shape
                        .prototype
                        .as_deref()
                        .is_some_and(|shape| contains_alias(shape, alias))
                        || shape.properties.values().any(property_contains_alias)
                        || shape
                            .boxed_primitive
                            .as_deref()
                            .and_then(|info| info.heap_shape.as_deref())
                            .is_some_and(|shape| contains_alias(shape, alias))
                }
                HeapShape::Array(shape) => {
                    shape
                        .prototype
                        .as_deref()
                        .is_some_and(|shape| contains_alias(shape, alias))
                        || shape.properties.values().any(property_contains_alias)
                        || shape.elements.iter().any(|info| {
                            info.heap_shape
                                .as_deref()
                                .is_some_and(|shape| contains_alias(shape, alias))
                        })
                }
            }
        }

        let clear_if_alias_is_reachable = |shape: &mut Option<Box<HeapShape>>| {
            if shape
                .as_deref()
                .is_some_and(|shape| contains_alias(shape, base_shape))
            {
                *shape = None;
            }
        };
        self.visit_live_heap_shape_roots(clear_if_alias_is_reachable);
        self.static_to_string_regexp_object_bindings.clear();
    }

    fn possible_shape_accessors(
        shape: Option<&HeapShape>,
    ) -> (BTreeSet<FunctionId>, BTreeSet<FunctionId>) {
        fn collect(
            shape: &HeapShape,
            getters: &mut BTreeSet<FunctionId>,
            setters: &mut BTreeSet<FunctionId>,
        ) {
            let (properties, prototype) = match shape {
                HeapShape::Object(shape) => (&shape.properties, shape.prototype.as_deref()),
                HeapShape::Array(shape) => (&shape.properties, shape.prototype.as_deref()),
            };
            for property in properties.values() {
                if let ObjectShapeProperty::Accessor { getter, setter } = property {
                    if let Some(getter) = getter {
                        getters.insert(getter.function_id.clone());
                    }
                    if let Some(setter) = setter {
                        setters.insert(setter.function_id.clone());
                    }
                }
            }
            if let Some(prototype) = prototype {
                collect(prototype, getters, setters);
            }
        }

        let mut getters = BTreeSet::new();
        let mut setters = BTreeSet::new();
        if let Some(shape) = shape {
            collect(shape, &mut getters, &mut setters);
        }
        (getters, setters)
    }

    fn possible_unknown_accessor_functions(&self) -> (PropertyHookTargets, PropertyHookTargets) {
        let mut known_getters =
            BTreeSet::from([StandardBuiltinId::ObjectPrototypeProtoGetter.function_id()]);
        let mut known_setters =
            BTreeSet::from([StandardBuiltinId::ObjectPrototypeProtoSetter.function_id()]);

        let mut collect = |info: &ValueInfo| {
            let (shape_getters, shape_setters) =
                Self::possible_shape_accessors(info.heap_shape.as_deref());
            known_getters.extend(shape_getters);
            known_setters.extend(shape_setters);
        };
        for property in self.global_properties.values() {
            collect(&property.value_info);
        }
        for scope in &self.scopes {
            for binding in scope.values() {
                collect(&ValueInfo {
                    kind: binding.kind,
                    possible_kinds: binding.possible_kinds,
                    heap_shape: binding.heap_shape.clone(),
                    function_targets: binding.function_targets.clone(),
                });
            }
        }
        for binding in self.var_bindings.values() {
            collect(&ValueInfo {
                kind: binding.kind,
                possible_kinds: binding.possible_kinds,
                heap_shape: binding.heap_shape.clone(),
                function_targets: binding.function_targets.clone(),
            });
        }
        if let CurrentThisBinding::Activation(info) = &self.current_this_binding {
            collect(info);
        }
        if let Some(info) = &self.current_construct_this_info {
            collect(info);
        }

        let mut getters = PropertyHookTargets::from_known(known_getters);
        let mut setters = PropertyHookTargets::from_known(known_setters);
        getters.include_all_planned_source(self.analysis.planned_source_function_ids.clone());
        setters.include_all_planned_source(self.analysis.planned_source_function_ids.clone());
        (getters, setters)
    }

    fn source_functions_may_run(&self, functions: &PropertyHookTargets) -> bool {
        functions.includes_all_planned_source()
            || functions
                .iter()
                .any(|function| self.analysis.function_plans.contains_key(function))
    }

    pub(super) fn property_key_may_call_user_code(key: &PropertyKeyIr) -> bool {
        match key {
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                Self::value_info_may_be_object(&expr.value_info())
            }
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
        }
    }

    fn value_info_may_be_object(info: &ValueInfo) -> bool {
        [
            ValueKind::Object,
            ValueKind::Array,
            ValueKind::Function,
            ValueKind::Arguments,
        ]
        .into_iter()
        .any(|kind| info.possible_kinds.contains(kind))
    }

    /// Lower a source-level plain assignment into one retained ordinary
    /// property Reference. Base and raw key are lowered before the RHS; the
    /// carrier leaves ToObject, ToPropertyKey, and Set to its backend consumer.
    pub(super) fn lower_ordinary_property_plain_assignment(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
        rhs: &Expression,
    ) -> TypedExpr {
        let (plan, referenced_name, metadata) = self.lower_ordinary_property_reference_plan(access);
        let rhs_may_invoke_user_code = self.prepare_potentially_effectful_expression(rhs);
        let before_rhs_effect_epoch = self.intervening_effect_epoch;
        let rhs_value = self.lower_expression(rhs);
        let rhs_may_have_intervening_effects =
            rhs_may_invoke_user_code || self.intervening_effect_epoch != before_rhs_effect_epoch;
        if rhs_may_have_intervening_effects {
            self.observe_all_planned_source_as_unknown_property_hooks();
            self.invalidate_unknown_user_code_effects();
        }
        let written_value_info = rhs_value.value_info();
        let possible_setters =
            self.possible_ordinary_property_setters(&metadata, rhs_may_have_intervening_effects);

        self.record_ordinary_property_possible_write(
            &referenced_name,
            &metadata,
            rhs_may_have_intervening_effects,
            written_value_info,
        );
        // Set can return false for a non-writable data property even in sloppy
        // code. Until descriptor attributes are part of the shape proof, do
        // not restore an exact global value or prototype-method identity from
        // the syntactic RHS alone.
        plan.plain_assignment(rhs_value, possible_setters)
    }

    /// Lower one ordinary property Reference directly into its fused eager
    /// mutation carrier. The base and raw computed-key expression are lowered
    /// before the RHS; their runtime GetValue/PutValue staging remains owned by
    /// the carrier's single backend consumer.
    pub(super) fn lower_ordinary_property_eager_compound_assignment(
        &mut self,
        access: &boa_ast::expression::access::SimplePropertyAccess,
        op: EagerCompoundAssignmentOp,
        rhs: &Expression,
    ) -> TypedExpr {
        let (plan, referenced_name, metadata) = self.lower_ordinary_property_reference_plan(access);
        self.record_ordinary_property_get(&metadata);
        let possible_getters = Self::possible_ordinary_property_getters(&metadata);
        let rhs_may_invoke_user_code = self.prepare_potentially_effectful_expression(rhs);
        let before_rhs_effect_epoch = self.intervening_effect_epoch;
        let rhs = self.lower_expression(rhs);
        let rhs_may_have_intervening_effects =
            rhs_may_invoke_user_code || self.intervening_effect_epoch != before_rhs_effect_epoch;
        if rhs_may_have_intervening_effects {
            self.observe_all_planned_source_as_unknown_property_hooks();
            self.invalidate_unknown_user_code_effects();
        }
        // The old property value is coerced only after the RHS has been
        // evaluated. Even a literal RHS therefore cannot prove this phase
        // effect-free: ToPrimitive/ToNumeric may run source `valueOf` or
        // @@toPrimitive code before [[Set]].
        let possible_setters = self.possible_ordinary_property_setters(&metadata, true);
        let old_value_binding = self.alloc_temp_binding_name("ordinary.property.compound.old.");
        let result = plan.eager_compound_assignment(
            old_value_binding,
            op,
            rhs,
            possible_getters,
            possible_setters,
        );
        self.record_ordinary_property_possible_write(
            &referenced_name,
            &metadata,
            true,
            result.value_info(),
        );
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
    ) -> &'a OrdinaryPropertyEagerCompoundAssignmentIr {
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
        let ExprIr::OrdinaryPropertyEagerCompoundAssignment(assignment) = &value.expr else {
            panic!(
                "expected fused ordinary property assignment, got {:?}",
                value.expr
            );
        };
        assignment
    }

    fn returned_plain_assignment<'a>(
        script: &'a ScriptIr,
        function_name: &str,
    ) -> &'a OrdinaryPropertyAssignmentIr {
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
        let ExprIr::OrdinaryPropertyAssignment(assignment) = &value.expr else {
            panic!(
                "expected fused ordinary property plain assignment, got {:?}",
                value.expr
            );
        };
        assignment
    }

    fn applied_lhs(result: &TypedExpr) -> &TypedExpr {
        match &result.expr {
            ExprIr::CoerciveAdd { lhs, .. }
            | ExprIr::CoerciveBinaryNumber { lhs, .. }
            | ExprIr::BitwiseNumeric { lhs, .. } => lhs,
            other => panic!("unexpected eager operation {other:?}"),
        }
    }

    #[test]
    fn ordinary_property_eager_compound_assignment_owns_one_reference() {
        let program = lower(
            r#"
            function add(base, key, rhs) { "use strict"; return base[key] += rhs; }
            function multiply(base, key, rhs) { "use strict"; return base[key] *= rhs; }
            function xor(base, key, rhs) { "use strict"; return base[key] ^= rhs; }
            function exponentiate(base, key, rhs) { "use strict"; return base[key] **= rhs; }
            "#,
        );
        let script = program.script.as_ref().expect("script IR should exist");

        for name in ["add", "multiply", "xor", "exponentiate"] {
            let assignment = returned_assignment(&script, name);
            assert!(matches!(
                &assignment.base_and_receiver().expr,
                ExprIr::Identifier(_)
            ));
            assert!(matches!(
                assignment.referenced_name(),
                PropertyKeyIr::StringExpr(key)
                    if matches!(&key.expr, ExprIr::Identifier(_))
            ));
            assert_eq!(assignment.strictness(), Strictness::Strict);
            assert!(assignment
                .old_value_binding()
                .starts_with("$ordinary.property.compound.old."));
            assert!(matches!(
                &applied_lhs(assignment.result()).expr,
                ExprIr::Identifier(binding) if binding == assignment.old_value_binding()
            ));
        }

        assert!(matches!(
            &returned_assignment(&script, "add").result().expr,
            ExprIr::CoerciveAdd { .. }
        ));
        assert!(matches!(
            &returned_assignment(&script, "multiply").result().expr,
            ExprIr::CoerciveBinaryNumber {
                op: ArithmeticBinaryOp::Mul,
                ..
            }
        ));
        assert!(matches!(
            &returned_assignment(&script, "xor").result().expr,
            ExprIr::BitwiseNumeric {
                op: BitwiseBinaryOp::Xor,
                ..
            }
        ));
        assert!(matches!(
            &returned_assignment(&script, "exponentiate").result().expr,
            ExprIr::CoerciveBinaryNumber {
                op: ArithmeticBinaryOp::Exp,
                ..
            }
        ));
    }

    #[test]
    fn ordinary_property_plain_assignment_retains_base_key_rhs_and_strictness() {
        let program = lower(
            r#"
            function computed(base, key, rhs) {
                "use strict";
                return base()[key()] = rhs();
            }
            function named(base, rhs) {
                return base.prop = rhs();
            }
            function nullBase(rhs) {
                let base = null;
                return base.prop = rhs();
            }
            function undefinedBase(rhs) {
                let base = undefined;
                return base.prop = rhs();
            }
            "#,
        );
        let script = program.script.as_ref().expect("script IR should exist");

        let computed = returned_plain_assignment(script, "computed");
        assert!(matches!(
            &computed.base_and_receiver().expr,
            ExprIr::CallIndirect { .. }
        ));
        assert!(matches!(
            computed.referenced_name(),
            PropertyKeyIr::StringExpr(key)
                if matches!(&key.expr, ExprIr::CallIndirect { .. })
        ));
        assert!(matches!(&computed.rhs().expr, ExprIr::CallIndirect { .. }));
        assert_eq!(computed.strictness(), Strictness::Strict);

        let named = returned_plain_assignment(script, "named");
        assert!(matches!(
            named.referenced_name(),
            PropertyKeyIr::StaticString(name) if name == "prop"
        ));
        assert!(matches!(&named.rhs().expr, ExprIr::CallIndirect { .. }));
        assert_eq!(named.strictness(), Strictness::Sloppy);

        let null_base = returned_plain_assignment(script, "nullBase");
        assert_eq!(null_base.base_and_receiver().kind, ValueKind::Null);
        assert!(matches!(&null_base.rhs().expr, ExprIr::CallIndirect { .. }));

        let undefined_base = returned_plain_assignment(script, "undefinedBase");
        assert_eq!(
            undefined_base.base_and_receiver().kind,
            ValueKind::Undefined
        );
        assert!(matches!(
            &undefined_base.rhs().expr,
            ExprIr::CallIndirect { .. }
        ));
    }

    #[test]
    fn plain_assignment_does_not_assume_a_sloppy_set_succeeds() {
        let program = lower(
            "globalThis.plainWritableFact = 1; Object.defineProperty(globalThis, 'plainWritableFact', { value: 1, writable: false }); globalThis.plainWritableFact = 's'; globalThis.plainWritableFact + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "a possibly failed sloppy Set cannot publish the RHS as a fact: {:?}",
            result.expr
        );
    }

    #[test]
    fn ordinary_property_write_invalidates_nested_object_and_array_alias_shapes() {
        for source in [
            "let target = { p: 0 }; let holder = { alias: target }; target.p ||= 's'; holder.alias.p + 1;",
            "let target = { p: 0 }; let holder = [target]; target.p ||= 's'; holder[0].p + 1;",
        ] {
            let program = lower(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
                panic!("expected follow-up addition");
            };
            assert!(
                matches!(result.expr, ExprIr::CoerciveAdd { .. }),
                "a nested alias must not retain the pre-write Number shape: {:?}",
                result.expr
            );
        }
    }

    #[test]
    fn ordinary_property_mutation_hooks_contribute_arbitrary_catch_values() {
        let program = lower(
            r#"
            var target = {
                get p() { throw 'getter'; },
                set p(value) { throw 'setter'; }
            };
            function catchPlain() {
                "use strict";
                try { target.p = 1; } catch (error) { return typeof error; }
            }
            function catchEager() {
                "use strict";
                try { target.p += 1; } catch (error) { return typeof error; }
            }
            function catchNumeric() {
                "use strict";
                try { target.p++; } catch (error) { return typeof error; }
            }
            "#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");

        for function_name in ["catchPlain", "catchEager", "catchNumeric"] {
            let function = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("missing function {function_name}"));
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
    }

    #[test]
    fn ordinary_setter_observes_the_published_value_after_a_direct_call() {
        let program = lower(
            r#"
            let prototype = {
                set value(value) { return value + 1; }
            };
            let setter = Object.getOwnPropertyDescriptor(prototype, 'value').set;
            setter(1);
            let target = { __proto__: prototype };
            target.value = 'published';
            "#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let setters = script
            .functions
            .iter()
            .filter(|function| function.name.contains("value") && function.params.len() == 1)
            .collect::<Vec<_>>();
        assert!(
            setters
                .iter()
                .any(|setter| setter.params[0].kind == ValueKind::Dynamic),
            "setter signatures: {:?}",
            setters
                .iter()
                .map(|setter| (&setter.id, setter.params[0].kind))
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn lost_proxy_hook_observation_stays_unknown_after_a_direct_call() {
        let program = lower(
            r#"
            function trap(target, key, value, receiver) { return target; }
            let handler = globalThis.choose ? { set: trap } : {};
            let proxy = new Proxy({}, handler);
            proxy.value = 'published';
            trap(1, 2, 3, 4);
            "#,
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let traps = script
            .functions
            .iter()
            .filter(|function| function.name == "trap")
            .collect::<Vec<_>>();
        assert!(
            traps.iter().any(|trap| {
                trap.params.len() == 4
                    && trap
                        .params
                        .iter()
                        .all(|param| param.kind == ValueKind::Dynamic)
            }),
            "trap signatures: {:?}",
            traps
                .iter()
                .map(|trap| {
                    (
                        &trap.id,
                        trap.params
                            .iter()
                            .map(|param| param.kind)
                            .collect::<Vec<_>>(),
                    )
                })
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn sloppy_primitive_property_hooks_observe_boxed_receivers() {
        for (source, function_name) in [
            (
                "Object.defineProperty(Number.prototype, 'compoundPrimitiveGetter', { configurable: true, get: function sloppyPrimitiveGetter() { return this + 1; } }); (1).compoundPrimitiveGetter += 2;",
                "sloppyPrimitiveGetter",
            ),
            (
                "Object.defineProperty(Number.prototype, 'compoundPrimitiveSetter', { configurable: true, set: function sloppyPrimitiveSetter(value) { return this + value; } }); (1).compoundPrimitiveSetter = 2;",
                "sloppyPrimitiveSetter",
            ),
        ] {
            let program = lower(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let hook = script
                .functions
                .iter()
                .find(|function| function.name == function_name)
                .unwrap_or_else(|| panic!("missing {function_name}"));
            let StatementIr::Return(result) = hook
                .body
                .statements
                .iter()
                .find(|statement| matches!(statement, StatementIr::Return(_)))
                .expect("hook should return a value")
            else {
                unreachable!("selected statement is a return")
            };
            assert!(
                matches!(result.expr, ExprIr::CoerciveAdd { .. }),
                "sloppy primitive hook receiver must be boxed: {:?}",
                result.expr
            );
        }
    }

    #[test]
    fn property_delete_invalidates_a_cached_function_return_shape() {
        let program = lower(
            "let target = { value: 1 }; function readTarget() { return target; } readTarget(); delete target.value; readTarget().value + 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let StatementIr::Expression(result) = script.body.statements.last().unwrap() else {
            panic!("expected follow-up addition");
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "delete must prevent a cached return shape from restoring the property: {:?}",
            result.expr
        );
    }

    #[test]
    fn computed_key_effects_do_not_restore_the_captured_base_shape() {
        for (setup, key) in [
            ("", "(base.marker = 's', 'value')"),
            (
                "",
                "(Object.defineProperty(base, 'marker', { value: 's' }), 'value')",
            ),
            (
                "function key() { base.marker = 's'; return 'value'; }",
                "key()",
            ),
        ] {
            let program = lower(&format!(
                "let base = {{ marker: 1, get value() {{ return this.marker + 1; }} }}; {setup} base[{key}] ||= 2;"
            ));
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let getter = script
                .functions
                .iter()
                .find(|function| function.name.contains("value") && function.params.is_empty())
                .expect("object-literal getter should be lowered");
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
                "key evaluation must invalidate the captured Number shape: {:?}",
                result.expr
            );
        }
    }

    #[test]
    fn reference_operand_effects_precede_a_plain_assignment_rhs() {
        for source in [
            "let x = 1; function make() { x = 's'; return { p: 0 }; } function run() { return make().p = x + 1; }",
            "let x = 1; const key = { get value() { x = 's'; return 'p'; } }; const base = {}; function run() { return base[key.value] = x + 1; }",
            "let x = 1; const key = { valueOf() { x = 's'; return 1; } }; const base = {}; function run() { return base[key + 0] = x + 1; }",
        ] {
            let program = lower(source);
            assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
            let script = program.script.as_ref().expect("script IR should exist");
            let assignment = returned_plain_assignment(script, "run");
            assert!(
                matches!(assignment.rhs().expr, ExprIr::CoerciveAdd { .. }),
                "base/raw-key effects must invalidate x before the RHS: {:?}",
                assignment.rhs().expr
            );
        }
    }

    #[test]
    fn implicit_hooks_widen_later_subexpressions_within_reference_stages() {
        let base_program = lower(
            "function outer() { let x = 'u'; const key = { get p() { x = 's'; return 0; } }; return (x = 1, key.p, { value: 0 }).value = x + 1; }",
        );
        let base_script = base_program
            .script
            .as_ref()
            .expect("script IR should exist");
        assert!(
            matches!(
                returned_plain_assignment(base_script, "outer").rhs().expr,
                ExprIr::CoerciveAdd { .. }
            ),
            "an implicit hook inside the base must precede the RHS"
        );

        let key_program = lower(
            "function outer() { let x = 'u'; const key = { get p() { x = 's'; return 0; } }; const base = {}; return base[(x = 1, key.p, x + 1)] = 2; }",
        );
        let key_script = key_program.script.as_ref().expect("script IR should exist");
        let PropertyKeyIr::StringExpr(key) =
            returned_plain_assignment(key_script, "outer").referenced_name()
        else {
            panic!("computed key should remain carried");
        };
        assert!(
            matches!(
                &key.expr,
                ExprIr::Comma { rhs, .. } if matches!(rhs.expr, ExprIr::CoerciveAdd { .. })
            ),
            "an implicit hook must widen later raw-key subexpressions: {:?}",
            key.expr
        );

        let rhs_program = lower(
            "function outer() { let x = 'u'; const key = { get p() { x = 's'; return 0; } }; const base = {}; return base.value = (x = 1, key.p, x + 1); }",
        );
        let rhs_script = rhs_program.script.as_ref().expect("script IR should exist");
        let rhs = returned_plain_assignment(rhs_script, "outer").rhs();
        assert!(
            matches!(
                &rhs.expr,
                ExprIr::Comma { rhs, .. } if matches!(rhs.expr, ExprIr::CoerciveAdd { .. })
            ),
            "an implicit hook must widen later RHS subexpressions: {:?}",
            rhs.expr
        );

        let call_program = lower(
            "let x = 'u'; function mutate() { x = 's'; } function outer() { const base = {}; return base.value = (x = 1, mutate(), x + 1); }",
        );
        let call_script = call_program
            .script
            .as_ref()
            .expect("script IR should exist");
        let rhs = returned_plain_assignment(call_script, "outer").rhs();
        assert!(
            matches!(
                &rhs.expr,
                ExprIr::Comma { rhs, .. } if matches!(rhs.expr, ExprIr::CoerciveAdd { .. })
            ),
            "a source call must widen later RHS subexpressions: {:?}",
            rhs.expr
        );
    }

    #[test]
    fn unknown_proxy_hooks_join_omitted_formals_with_undefined() {
        let program = lower(
            "function hook(a, b, c, d, e) { return e + 1; } hook(0, 0, 0, 0, 1); const handler = { set: hook }; const proxy = new Proxy({}, handler); proxy.x = 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let hook = script
            .functions
            .iter()
            .find(|function| function.name == "hook")
            .expect("hook should be lowered");
        let StatementIr::Return(result) = hook
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("hook should return")
        else {
            unreachable!("selected statement is a return")
        };
        let ExprIr::CoerciveBinaryNumber { lhs, .. } = &result.expr else {
            panic!(
                "Number-or-undefined addition should remain numeric: {:?}",
                result.expr
            );
        };
        assert_eq!(
            lhs.possible_kinds,
            KindSet::from_kind(ValueKind::Number).union(KindSet::from_kind(ValueKind::Undefined)),
            "the fifth Proxy trap formal must include its omitted undefined argument"
        );
    }

    #[test]
    fn eager_old_value_coercion_invalidates_followup_flow_facts() {
        let program = lower(
            "function outer() { let x = 1; const value = { valueOf() { x = 's'; return 1; } }; const base = [value]; base[0] += 1; return x + 1; }",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let outer = script
            .functions
            .iter()
            .find(|function| function.name == "outer")
            .expect("outer should be lowered");
        let StatementIr::Return(result) = outer
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("outer should return")
        else {
            unreachable!("selected statement is a return")
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "deferred old-value coercion must widen later facts: {:?}",
            result.expr
        );
    }

    #[test]
    fn primitive_base_key_coercion_does_not_box_the_key_hook_receiver() {
        let program = lower(
            "let out = 0; const key = { get length() { out = 's'; return 'v'; }, toString() { return this.length + 1; } }; ('x')[key] = 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let coercion = script
            .functions
            .iter()
            .find(|function| function.name == "toString")
            .expect("key coercion method should be lowered");
        let StatementIr::Return(result) = coercion
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("key coercion method should return")
        else {
            unreachable!("selected statement is a return")
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "the key object, not the primitive property base, is the coercion receiver: {:?}",
            result.expr
        );
    }

    #[test]
    fn eager_old_value_coercion_observes_unknown_hook_receivers() {
        let program = lower(
            "const value = { get length() { return 's'; }, valueOf() { return this.length + 1; } }; const base = [value]; base[0] += 1;",
        );
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let script = program.script.as_ref().expect("script IR should exist");
        let coercion = script
            .functions
            .iter()
            .find(|function| function.name == "valueOf")
            .expect("old-value coercion method should be lowered");
        let StatementIr::Return(result) = coercion
            .body
            .statements
            .iter()
            .find(|statement| matches!(statement, StatementIr::Return(_)))
            .expect("old-value coercion method should return")
        else {
            unreachable!("selected statement is a return")
        };
        assert!(
            matches!(result.expr, ExprIr::CoerciveAdd { .. }),
            "deferred ToPrimitive must not reuse the property base as this: {:?}",
            result.expr
        );
    }
}
