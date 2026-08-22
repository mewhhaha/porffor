use super::*;

impl<'a> ScriptLowerer<'a> {
    pub(super) fn lower_call(&mut self, callee: &Expression, args: &[Expression]) -> TypedExpr {
        // A call nested in a computed property key can mutate the already
        // captured base even when its result is a primitive key. The epoch is
        // only an ordering signal; the call's normal effect analysis still
        // decides which flow facts must actually be discarded.
        self.intervening_effect_epoch = self.intervening_effect_epoch.saturating_add(1);
        let unsupported_call = |this: &mut Self, class_kind: ClassFunctionKind| {
            if class_kind != ClassFunctionKind::Constructor {
                this.unsupported_expr("indirect call")
            } else {
                TypedExpr::undefined()
            }
        };

        // Resolve a direct identifier through any preceding Object
        // Environment Records before the name-specific builtin folds below.
        // A selected with binding can shadow even `Number`/`Boolean`/`Symbol`,
        // and the Reference's base supplies CallExpression's WithBaseObject.
        if let Some(call) = self.lower_with_environment_identifier_call(callee, args) {
            return call;
        }

        if let Some(generator) = generator_expression_callee(callee) {
            if args.is_empty() && linear_generator_plan(generator.body()).is_none() {
                return self.lower_generator_iife_as_array(generator);
            }
        }

        if let Some(values) = self.static_iterator_to_array_call_values(callee, args) {
            return self.array_literal_from_static_generator_values(&values);
        }
        if let Some(values) = self.static_array_from_iterator_call_values(callee, args) {
            return self.array_literal_from_static_generator_values(&values);
        }

        if let Expression::Identifier(identifier) = callee {
            let name = self.interner.resolve_expect(identifier.sym()).to_string();
            if args.is_empty() {
                if let Some(result) = self.static_generator_call_overrides.get(&name).cloned() {
                    return result;
                }
                if let Some(elements) = self.static_generator_element_values.get(&name).cloned() {
                    return self.array_iterator_from_lowered_elements(elements);
                }
                if let Some(values) = self.static_generator_sum_values.get(&name).cloned() {
                    return self.array_iterator_from_static_generator_values(&values);
                }
            }
            if name == IS_CONSTRUCTOR_NAME && args.len() == 1 && !self.has_scope_binding(&name) {
                return TypedExpr::spec_is_constructor(self.lower_expression(&args[0]));
            }
            if name == NUMBER_NAME {
                if let Some(value) = self.static_to_number_arg(args.first()) {
                    for arg in args {
                        self.lower_expression(arg);
                    }
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Number),
                        ExprIr::Number(value.to_bits()),
                    );
                }
                if args.len() == 1 && self.expression_cannot_be_kind(&args[0], ValueKind::BigInt) {
                    return TypedExpr::spec_to_number(self.lower_expression(&args[0]));
                }
            }
            if name == STRING_NAME && args.len() == 1 && self.expression_cannot_be_symbol(&args[0])
            {
                return TypedExpr::spec_to_string(self.lower_expression(&args[0]));
            }
            if name == BOOLEAN_NAME {
                if let Some(value) = self.static_to_boolean_arg(args.first()) {
                    for arg in args {
                        self.lower_expression(arg);
                    }
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Boolean),
                        ExprIr::Boolean(value),
                    );
                }
                if args.len() == 1 {
                    return TypedExpr::spec_to_boolean(self.lower_expression(&args[0]));
                }
            }
            if name == PARSE_FLOAT_NAME {
                if let Some(value) = self.static_parse_float_arg(args.first()) {
                    for arg in args {
                        self.lower_expression(arg);
                    }
                    return TypedExpr::from_info(
                        ValueInfo::new(ValueKind::Number),
                        ExprIr::Number(value.to_bits()),
                    );
                }
            }
            if name == "Symbol" {
                // `Symbol(description)`: if description is undefined, the
                // `[[Description]]` is undefined; otherwise it is
                // `? ToString(description)` (spec 20.4.1.1). Only the first
                // argument participates; any extra arguments are still
                // evaluated for their side effects.
                let description = match args.first() {
                    None => None,
                    Some(arg) => {
                        let lowered = self.lower_expression(arg);
                        if lowered.kind == ValueKind::Undefined {
                            None
                        } else {
                            Some(Box::new(TypedExpr::spec_to_string(lowered)))
                        }
                    }
                };
                for arg in args.iter().skip(1) {
                    self.lower_expression(arg);
                }
                return TypedExpr::from_info(
                    ValueInfo::new(ValueKind::Symbol),
                    ExprIr::Symbol { description },
                );
            }
        }

        if let Expression::PropertyAccess(PropertyAccess::Simple(access)) = callee {
            if let PropertyAccessField::Const(field) = access.field() {
                let field_name = self.interner.resolve_expect(field.sym()).to_string();
                if let Expression::Identifier(target) = access.target() {
                    let target_name = self.interner.resolve_expect(target.sym()).to_string();
                    if target_name == OBJECT_NAME && field_name == "is" && args.len() == 2 {
                        let lhs = self.lower_expression(&args[0]);
                        let rhs = self.lower_expression(&args[1]);
                        return TypedExpr::spec_same_value(lhs, rhs);
                    }
                    let static_builtin = match (target_name.as_str(), field_name.as_str()) {
                        (OBJECT_NAME, "keys") => Some(StandardBuiltinId::ObjectKeys),
                        (OBJECT_NAME, "values") => Some(StandardBuiltinId::ObjectValues),
                        (OBJECT_NAME, "entries") => Some(StandardBuiltinId::ObjectEntries),
                        (STRING_NAME, "fromCharCode") => {
                            Some(StandardBuiltinId::StringFromCharCode)
                        }
                        (STRING_NAME, "fromCodePoint") => {
                            Some(StandardBuiltinId::StringFromCodePoint)
                        }
                        (STRING_NAME, "raw") => Some(StandardBuiltinId::StringRaw),
                        _ => None,
                    };
                    if let Some(builtin) = static_builtin {
                        let function_id = builtin.function_id();
                        let (effective_function_id, args, info) = self.lower_call_args_with_target(
                            &function_id,
                            args,
                            BuiltinCallContext::Call,
                        );
                        return TypedExpr::from_info(
                            info,
                            ExprIr::CallIndirect {
                                callee: Box::new(self.function_value_expr(effective_function_id)),
                                this_arg: Some(Box::new(TypedExpr::from_info(
                                    Self::standard_builtin_value_info(match builtin {
                                        StandardBuiltinId::StringFromCharCode
                                        | StandardBuiltinId::StringFromCodePoint
                                        | StandardBuiltinId::StringRaw => {
                                            StandardBuiltinId::StringConstructor
                                        }
                                        _ => StandardBuiltinId::ObjectConstructor,
                                    }),
                                    ExprIr::GlobalPropertyRead {
                                        name: target_name.clone(),
                                    },
                                ))),
                                args,
                                static_regexp_compilation: None,
                            },
                        );
                    }
                    if target_name == "ASCII_IDENTIFIER" && field_name == "test" && args.len() == 1
                    {
                        self.lower_expression(&args[0]);
                        return TypedExpr::from_info(
                            Self::boolean_value_info(),
                            ExprIr::Boolean(true),
                        );
                    }
                }
                if field_name == "toString" {
                    if args.is_empty() {
                        if let Some(value) = self.static_string_receiver_value(access.target()) {
                            return Self::static_string_typed_expr(value);
                        }
                    }
                    if let Some(value) = self.static_boolean_receiver_value(access.target()) {
                        match self.boolean_prototype_to_string_state {
                            PrototypeToStringState::ObjectPrototype => {
                                for arg in args {
                                    self.lower_expression(arg);
                                }
                                return TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::String),
                                    ExprIr::String("[object Boolean]".to_string()),
                                );
                            }
                            PrototypeToStringState::Intrinsic => {
                                for arg in args {
                                    self.lower_expression(arg);
                                }
                                return TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::String),
                                    ExprIr::String(
                                        if value { "true" } else { "false" }.to_string(),
                                    ),
                                );
                            }
                            PrototypeToStringState::Unknown => {}
                        }
                    }
                    if self.number_prototype_to_string_state
                        == PrototypeToStringState::ObjectPrototype
                        && self.is_number_prototype_property_expr(callee, "toString")
                    {
                        for arg in args {
                            self.lower_expression(arg);
                        }
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::String),
                            ExprIr::String("[object Number]".to_string()),
                        );
                    }
                    if self.number_prototype_to_string_state == PrototypeToStringState::Intrinsic
                        && self
                            .static_number_to_string_receiver_value(access.target())
                            .is_some()
                        && self.static_number_to_string_radix_is_invalid(args)
                    {
                        for arg in args {
                            self.lower_expression(arg);
                        }
                        return TypedExpr::from_info(
                            ValueInfo::undefined(),
                            ExprIr::RuntimeThrow {
                                name: NativeErrorKind::RangeError,
                                message: "Number.prototype.toString radix out of range",
                            },
                        );
                    }
                    if let Some(value) = (self.number_prototype_to_string_state
                        == PrototypeToStringState::Intrinsic)
                        .then(|| self.static_number_to_string_call(access.target(), args))
                        .flatten()
                    {
                        for arg in args {
                            self.lower_expression(arg);
                        }
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::String),
                            ExprIr::String(value),
                        );
                    }
                }
                // 21.1.3.2. The receiver-finiteness guard and the hand-rolled
                // `RuntimeThrow` that used to sit here are gone: the ordering
                // is now `NonFiniteReceiverOrder::ReceiverFirst` inside the
                // helper, and "the spec requires a RangeError" is a variant
                // rather than the same `None` that means "I could not fold
                // this". The match has **no `_` arm** on purpose — a catch-all
                // would silently absorb a fourth outcome, which is the mistake
                // class this closes.
                if field_name == "toExponential" {
                    let fold = self.static_number_to_exponential_call(access.target(), args);
                    match fold {
                        NumberFormatFold::Formatted(value) => {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::String),
                                ExprIr::String(value),
                            );
                        }
                        NumberFormatFold::RangeError(message) => {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return Self::static_number_format_range_error(message);
                        }
                        NumberFormatFold::NotStatic => {}
                    }
                }
                if field_name == "valueOf" {
                    if args.is_empty() {
                        if let Some(value) = self.static_string_receiver_value(access.target()) {
                            return Self::static_string_typed_expr(value);
                        }
                    }
                    if let Some(value) = self.static_boolean_receiver_value(access.target()) {
                        for arg in args {
                            self.lower_expression(arg);
                        }
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::Boolean),
                            ExprIr::Boolean(value),
                        );
                    }
                }
                // 21.1.3.5. This is the site that never had a range check at
                // all: its `RangeError` arm is new, and the message string with
                // it. It could not have been fixed by calling the old shared
                // `[0, 100]` predicate — step 5 here is `p < 1 or p > 100`.
                if field_name == "toPrecision" {
                    let fold = self.static_number_to_precision_call(access.target(), args);
                    match fold {
                        NumberFormatFold::Formatted(value) => {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::String),
                                ExprIr::String(value),
                            );
                        }
                        NumberFormatFold::RangeError(message) => {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return Self::static_number_format_range_error(message);
                        }
                        NumberFormatFold::NotStatic => {}
                    }
                }
                // 21.1.3.3. Steps 4-5 precede step 6, so the range check wins
                // over the non-finite receiver here and `Infinity.toFixed(101)`
                // is a RangeError. The old guard's `.is_some()` (versus
                // `toExponential`'s `.is_some_and(is_finite)` sixty lines up)
                // was exactly this ordering, spelled as a coincidence of two
                // adjacent predicates; it is now
                // `NonFiniteReceiverOrder::RangeCheckFirst`.
                if field_name == "toFixed" {
                    let fold = self.static_number_to_fixed_call(access.target(), args);
                    match fold {
                        NumberFormatFold::Formatted(value) => {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::String),
                                ExprIr::String(value),
                            );
                        }
                        NumberFormatFold::RangeError(message) => {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return Self::static_number_format_range_error(message);
                        }
                        NumberFormatFold::NotStatic => {}
                    }
                }
            }
        }

        if let Expression::PropertyAccess(access) = callee {
            match access {
                PropertyAccess::Simple(access) => {
                    if let PropertyAccessField::Const(field) = access.field() {
                        let field_name = self.interner.resolve_expect(field.sym()).to_string();
                        if field_name == "propertyIsEnumerable"
                            && args.len() == 1
                            && self.for_in_global_target(access.target())
                            && self
                                .try_static_string_key(&args[0])
                                .is_some_and(|property| {
                                    self.is_known_non_enumerable_global(&property)
                                })
                        {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::Boolean),
                                ExprIr::Boolean(false),
                            );
                        }
                    }
                    if let (Expression::Identifier(target), PropertyAccessField::Const(field)) =
                        (access.target(), access.field())
                    {
                        let target_name = self.interner.resolve_expect(target.sym()).to_string();
                        let field_name = self.interner.resolve_expect(field.sym()).to_string();
                        if target_name == MATH_NAME && field_name == "pow" && args.len() == 2 {
                            if let (Some(base), Some(exponent)) = (
                                Self::literal_number_value(&args[0]),
                                Self::literal_number_value(&args[1]),
                            ) {
                                return TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::Number),
                                    ExprIr::Number(Self::static_pow(base, exponent).to_bits()),
                                );
                            }
                        }
                        if target_name == MATH_NAME && field_name == "clz32" && args.len() == 1 {
                            if let Some(value) = self.static_number_expr(&args[0]) {
                                return TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::Number),
                                    ExprIr::Number(Self::static_clz32(value).to_bits()),
                                );
                            }
                        }
                        if target_name == MATH_NAME && field_name == "round" && args.len() == 1 {
                            if let Some(value) = self.static_number_expr(&args[0]) {
                                return TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::Number),
                                    ExprIr::Number(Self::static_round(value).to_bits()),
                                );
                            }
                        }
                        if target_name == STRING_NAME
                            && field_name == "fromCharCode"
                            && !args.is_empty()
                        {
                            let mut chars = args.iter().map(|arg| {
                                TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::String),
                                    ExprIr::StringFromCharCode {
                                        code: Box::new(self.lower_expression(arg)),
                                    },
                                )
                            });
                            let first = chars.next().expect("fromCharCode arg exists");
                            return chars.fold(first, |lhs, rhs| {
                                TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::String),
                                    ExprIr::StringConcat {
                                        lhs: Box::new(lhs),
                                        rhs: Box::new(rhs),
                                    },
                                )
                            });
                        }
                        if target_name == STRING_NAME
                            && field_name == "fromCharCode"
                            && args.is_empty()
                        {
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::String),
                                ExprIr::String(String::new()),
                            );
                        }
                        if field_name == "propertyIsEnumerable"
                            && args.len() == 1
                            && self
                                .identifier_is_builtin_native_error(target_name.as_str())
                                .is_some()
                            && self
                                .try_static_string_key(&args[0])
                                .is_some_and(|property| property == "prototype")
                        {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::Boolean),
                                ExprIr::Boolean(false),
                            );
                        }
                        if field_name == "propertyIsEnumerable"
                            && args.len() == 1
                            && self
                                .try_static_string_key(&args[0])
                                .is_some_and(|property| {
                                    self.is_known_non_enumerable_builtin_property(
                                        &target_name,
                                        &property,
                                    )
                                })
                        {
                            for arg in args {
                                self.lower_expression(arg);
                            }
                            return TypedExpr::from_info(
                                ValueInfo::new(ValueKind::Boolean),
                                ExprIr::Boolean(false),
                            );
                        }
                    }
                    let receiver = self.lower_property_target(access.target());
                    let string_from_code_point_apply_call = if let PropertyAccessField::Const(
                        field,
                    ) = access.field()
                    {
                        self.interner.resolve_expect(field.sym()).to_string() == "apply"
                            && matches!(
                                    Self::unwrap_parenthesized_expr(access.target()),
                                    Expression::PropertyAccess(target_access)
                                        if matches!(
                                            target_access,
                                            PropertyAccess::Simple(target_access)
                                                if matches!(
                                                (
                                                    Self::unwrap_parenthesized_expr(target_access.target()),
                                                    target_access.field(),
                                                ),
                                                (
                                                    Expression::Identifier(target),
                                                    PropertyAccessField::Const(field),
                                                ) if self
                                                    .interner
                                                    .resolve_expect(target.sym())
                                                    .to_string()
                                                    == STRING_NAME
                                                    && self
                                                        .interner
                                                        .resolve_expect(field.sym())
                                                        .to_string()
                                                        == "fromCodePoint"
                                            )
                                    )
                            )
                    } else {
                        false
                    };
                    if let PropertyAccessField::Const(field) = access.field() {
                        let field_name = self.interner.resolve_expect(field.sym()).to_string();
                        let receiver_is_array = receiver.possible_kinds.contains(ValueKind::Array)
                            || matches!(receiver.heap_shape.as_deref(), Some(HeapShape::Array(_)));
                        let receiver_is_iterator = self
                            .read_object_shape_property(&receiver, "forEach")
                            .is_some_and(|property| match property {
                                ObjectShapeProperty::Data(info) => info.function_targets.contains(
                                    &StandardBuiltinId::IteratorPrototypeForEach.function_id(),
                                ),
                                ObjectShapeProperty::Accessor { .. } => false,
                            });
                        let receiver_has_custom_array_prototype =
                            Self::array_shape_has_custom_prototype(&receiver);
                        if (field_name == "forEach"
                            && ((receiver_is_array && !receiver_has_custom_array_prototype)
                                || receiver_is_iterator))
                            || (matches!(
                                field_name.as_str(),
                                "every"
                                    | "some"
                                    | "find"
                                    | "reduce"
                                    | "reduceRight"
                                    | "map"
                                    | "filter"
                                    | "flatMap"
                                    | "take"
                                    | "drop"
                            ) && !receiver_is_array)
                        {
                            let Some(args) = self.lower_call_args_expanding_spread(args) else {
                                return TypedExpr::undefined();
                            };
                            if field_name != "take" && field_name != "drop" {
                                if let Some(callback) = args.first() {
                                    if let Some(callback_id) =
                                        self.resolve_single_function_target(callback)
                                    {
                                        let dynamic_value = ValueInfo {
                                            kind: ValueKind::Dynamic,
                                            possible_kinds: KindSet::all_runtime_tags(),
                                            heap_shape: None,
                                            function_targets: BTreeSet::new(),
                                        };
                                        if field_name == "reduce" || field_name == "reduceRight" {
                                            self.merge_function_param_infos(
                                                &callback_id,
                                                &[
                                                    dynamic_value.clone(),
                                                    dynamic_value.clone(),
                                                    ValueInfo::new(ValueKind::Number),
                                                    dynamic_value,
                                                ],
                                            );
                                        } else {
                                            self.merge_function_param_infos(
                                                &callback_id,
                                                &[dynamic_value, ValueInfo::new(ValueKind::Number)],
                                            );
                                        }
                                        self.merge_function_this_info(
                                            &callback_id,
                                            ValueInfo::undefined(),
                                        );
                                    }
                                }
                            }
                            let result_info = match field_name.as_str() {
                                "every" | "some" => ValueInfo::new(ValueKind::Boolean),
                                "find" | "reduce" | "reduceRight" => ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                                "map" | "filter" | "flatMap" | "take" | "drop" => ValueInfo {
                                    kind: ValueKind::Object,
                                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                                    heap_shape: Some(match field_name.as_str() {
                                        "map" => Self::iterator_map_helper_shape(),
                                        "filter" => Self::iterator_filter_helper_shape(),
                                        "flatMap" => Self::iterator_flat_map_helper_shape(),
                                        "take" => Self::iterator_take_helper_shape(),
                                        _ => Self::iterator_drop_helper_shape(),
                                    }),
                                    function_targets: BTreeSet::new(),
                                },
                                _ => ValueInfo::undefined(),
                            };
                            return TypedExpr::from_info(
                                result_info,
                                ExprIr::CallMethod {
                                    receiver: Box::new(receiver),
                                    key: PropertyKeyIr::StaticString(field_name),
                                    args,
                                },
                            );
                        }
                    }
                    if let Some(result) =
                        self.lower_static_iterator_from_wrapper_method_call(&receiver, access, args)
                    {
                        return result;
                    }
                    if let Some(result) =
                        self.lower_static_yield_star_generator_method_call(&receiver, access, args)
                    {
                        return result;
                    }
                    if let PropertyAccessField::Const(field) = access.field() {
                        let field_name = self.interner.resolve_expect(field.sym()).to_string();
                        if (field_name == "exec" || field_name == "test")
                            && args.len() == 1
                            && receiver.possible_kinds.contains(ValueKind::Object)
                        {
                            let info = if field_name == "test" {
                                ValueInfo::new(ValueKind::Boolean)
                            } else {
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                }
                            };
                            return TypedExpr::from_info(
                                info,
                                ExprIr::CallMethod {
                                    receiver: Box::new(receiver),
                                    key: PropertyKeyIr::StaticString(field_name),
                                    args: args
                                        .iter()
                                        .map(|arg| self.lower_expression(arg))
                                        .collect(),
                                },
                            );
                        }
                        if !self.array_prototype_mutated
                            && matches!(
                                receiver.heap_shape.as_deref(),
                                Some(HeapShape::Array(shape)) if shape.prototype.is_none()
                            )
                        {
                            if field_name == "join" && args.len() <= 1 {
                                let Some(args) = self.lower_call_args_expanding_spread(args) else {
                                    return TypedExpr::undefined();
                                };
                                return TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::String),
                                    ExprIr::CallMethod {
                                        receiver: Box::new(receiver),
                                        key: PropertyKeyIr::StaticString(field_name),
                                        args,
                                    },
                                );
                            }
                            if field_name == "toString" && args.is_empty() {
                                return TypedExpr::from_info(
                                    ValueInfo::new(ValueKind::String),
                                    ExprIr::CallMethod {
                                        receiver: Box::new(receiver),
                                        key: PropertyKeyIr::StaticString(field_name),
                                        args: Vec::new(),
                                    },
                                );
                            }
                            if field_name == "reverse" && args.is_empty() {
                                return TypedExpr::from_info(
                                    receiver.value_info(),
                                    ExprIr::CallMethod {
                                        receiver: Box::new(receiver),
                                        key: PropertyKeyIr::StaticString(field_name),
                                        args: Vec::new(),
                                    },
                                );
                            }
                        }
                    }
                    if let PropertyAccessField::Expr(expr) = access.field() {
                        if let Some((symbol, symbol_key)) =
                            self.lower_well_known_symbol_property_key(expr)
                        {
                            // Contract ledger entry R3, corrected. The domain
                            // here is `WellKnownSymbol x receiver`, and the R3
                            // note used to claim both axes were honoured — true
                            // only of the two `Iterator` arms. The other five
                            // fired on the symbol alone, so
                            // `({ [Symbol.match](s) { return 42; } })[Symbol.match]("x")`
                            // was typed as `RegExp.prototype[@@match]`'s
                            // `Array | Null` rather than as `Number`. The
                            // emitted IR is a generic `ExprIr::CallMethod`
                            // either way, so the damage was confined to the
                            // inferred `ValueInfo` and to anything keyed on it —
                            // but that is exactly invariant S5's subject.
                            //
                            // Most cells legitimately have no static fast path,
                            // so this match keeps a catch-all rather than nine
                            // information-free `=> None` arms. The symbols that
                            // deliberately fall through are `asyncIterator`,
                            // `dispose`, `asyncDispose`, `hasInstance`,
                            // `isConcatSpreadable`, `species`, `toPrimitive`,
                            // `toStringTag` and `unscopables`; what the enum
                            // buys is that the arms above name real variants.
                            let regexp_receiver =
                                Self::receiver_shape_allows_regexp_symbol_protocol(&receiver);
                            let builtin = match symbol {
                                WellKnownSymbol::Iterator if receiver.kind == ValueKind::String => {
                                    Some(StandardBuiltinId::StringPrototypeIterator)
                                }
                                WellKnownSymbol::Iterator if receiver.kind == ValueKind::Array => {
                                    Some(StandardBuiltinId::ArrayPrototypeValues)
                                }
                                WellKnownSymbol::Match if regexp_receiver => {
                                    Some(StandardBuiltinId::RegExpPrototypeSymbolMatch)
                                }
                                WellKnownSymbol::MatchAll if regexp_receiver => {
                                    Some(StandardBuiltinId::RegExpPrototypeSymbolMatchAll)
                                }
                                WellKnownSymbol::Replace if regexp_receiver => {
                                    Some(StandardBuiltinId::RegExpPrototypeSymbolReplace)
                                }
                                WellKnownSymbol::Search if regexp_receiver => {
                                    Some(StandardBuiltinId::RegExpPrototypeSymbolSearch)
                                }
                                WellKnownSymbol::Split if regexp_receiver => {
                                    Some(StandardBuiltinId::RegExpPrototypeSymbolSplit)
                                }
                                _ => None,
                            };
                            if let Some(builtin) = builtin {
                                let Some(args) = self.lower_call_args_expanding_spread(args) else {
                                    return TypedExpr::undefined();
                                };
                                let info = self
                                    .standard_builtin_call_info(
                                        builtin,
                                        &args,
                                        BuiltinCallContext::Call,
                                    )
                                    .unwrap_or(ValueInfo {
                                        kind: ValueKind::Dynamic,
                                        possible_kinds: KindSet::all_runtime_tags(),
                                        heap_shape: None,
                                        function_targets: BTreeSet::new(),
                                    });
                                return TypedExpr::from_info(
                                    info,
                                    ExprIr::CallMethod {
                                        receiver: Box::new(receiver),
                                        key: symbol_key,
                                        args,
                                    },
                                );
                            }
                        }
                    }
                    let receiver_has_known_own_property = match access.field() {
                        PropertyAccessField::Const(field) => {
                            let field_name = self.interner.resolve_expect(field.sym()).to_string();
                            self.read_own_object_shape_property(&receiver, &field_name)
                                .is_some()
                        }
                        PropertyAccessField::Expr(_) => false,
                    };
                    let callee = match receiver.kind {
                        ValueKind::Object | ValueKind::Function => {
                            self.lower_object_property_key(receiver.clone(), access.field())
                        }
                        ValueKind::String => {
                            if let PropertyAccessField::Const(field) = access.field() {
                                let field_name =
                                    self.interner.resolve_expect(field.sym()).to_string();
                                if field_name == "charCodeAt" && args.len() <= 1 {
                                    let index = args
                                        .first()
                                        .map(|arg| self.lower_expression(arg))
                                        .unwrap_or_else(|| {
                                            TypedExpr::from_info(
                                                ValueInfo::new(ValueKind::Number),
                                                ExprIr::Number(0.0f64.to_bits()),
                                            )
                                        });
                                    return TypedExpr::from_info(
                                        ValueInfo::new(ValueKind::Number),
                                        ExprIr::StringCharCodeAt {
                                            target: Box::new(receiver),
                                            index: Box::new(index),
                                        },
                                    );
                                }
                                if field_name == "split" && args.len() <= 2 {
                                    let Some(args) = self.lower_call_args_expanding_spread(args)
                                    else {
                                        return TypedExpr::undefined();
                                    };
                                    return TypedExpr::from_info(
                                        ValueInfo {
                                            kind: ValueKind::Array,
                                            possible_kinds: KindSet::from_kind(ValueKind::Array),
                                            heap_shape: Some(Box::new(HeapShape::Array(
                                                ArrayShape::default(),
                                            ))),
                                            function_targets: BTreeSet::new(),
                                        },
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(field_name),
                                            args,
                                        },
                                    );
                                }
                                let builtin = match field_name.as_str() {
                                    "charAt" => Some(StandardBuiltinId::StringPrototypeCharAt),
                                    "concat" => Some(StandardBuiltinId::StringPrototypeConcat),
                                    "charCodeAt" => {
                                        Some(StandardBuiltinId::StringPrototypeCharCodeAt)
                                    }
                                    "codePointAt" => {
                                        Some(StandardBuiltinId::StringPrototypeCodePointAt)
                                    }
                                    "at" => Some(StandardBuiltinId::StringPrototypeAt),
                                    "anchor" => Some(StandardBuiltinId::StringPrototypeAnchor),
                                    "big" => Some(StandardBuiltinId::StringPrototypeBig),
                                    "blink" => Some(StandardBuiltinId::StringPrototypeBlink),
                                    "bold" => Some(StandardBuiltinId::StringPrototypeBold),
                                    "fixed" => Some(StandardBuiltinId::StringPrototypeFixed),
                                    "fontcolor" => {
                                        Some(StandardBuiltinId::StringPrototypeFontcolor)
                                    }
                                    "fontsize" => Some(StandardBuiltinId::StringPrototypeFontsize),
                                    "italics" => Some(StandardBuiltinId::StringPrototypeItalics),
                                    "link" => Some(StandardBuiltinId::StringPrototypeLink),
                                    "small" => Some(StandardBuiltinId::StringPrototypeSmall),
                                    "strike" => Some(StandardBuiltinId::StringPrototypeStrike),
                                    "sub" => Some(StandardBuiltinId::StringPrototypeSub),
                                    "substr" => Some(StandardBuiltinId::StringPrototypeSubstr),
                                    "substring" => {
                                        Some(StandardBuiltinId::StringPrototypeSubstring)
                                    }
                                    "sup" => Some(StandardBuiltinId::StringPrototypeSup),
                                    "match" => Some(StandardBuiltinId::StringPrototypeMatch),
                                    "matchAll" => Some(StandardBuiltinId::StringPrototypeMatchAll),
                                    "replace" => Some(StandardBuiltinId::StringPrototypeReplace),
                                    "replaceAll" => {
                                        Some(StandardBuiltinId::StringPrototypeReplaceAll)
                                    }
                                    "search" => Some(StandardBuiltinId::StringPrototypeSearch),
                                    "indexOf" => Some(StandardBuiltinId::StringPrototypeIndexOf),
                                    "lastIndexOf" => {
                                        Some(StandardBuiltinId::StringPrototypeLastIndexOf)
                                    }
                                    "slice" => Some(StandardBuiltinId::StringPrototypeSlice),
                                    "split" => Some(StandardBuiltinId::StringPrototypeSplit),
                                    "padStart" => Some(StandardBuiltinId::StringPrototypePadStart),
                                    "padEnd" => Some(StandardBuiltinId::StringPrototypePadEnd),
                                    "repeat" => Some(StandardBuiltinId::StringPrototypeRepeat),
                                    "endsWith" => Some(StandardBuiltinId::StringPrototypeEndsWith),
                                    "includes" => Some(StandardBuiltinId::StringPrototypeIncludes),
                                    "startsWith" => {
                                        Some(StandardBuiltinId::StringPrototypeStartsWith)
                                    }
                                    "normalize" => {
                                        Some(StandardBuiltinId::StringPrototypeNormalize)
                                    }
                                    "localeCompare" => {
                                        Some(StandardBuiltinId::StringPrototypeLocaleCompare)
                                    }
                                    "toLocaleLowerCase" => {
                                        Some(StandardBuiltinId::StringPrototypeToLocaleLowerCase)
                                    }
                                    "toLocaleUpperCase" => {
                                        Some(StandardBuiltinId::StringPrototypeToLocaleUpperCase)
                                    }
                                    "toLowerCase" => {
                                        Some(StandardBuiltinId::StringPrototypeToLowerCase)
                                    }
                                    "toUpperCase" => {
                                        Some(StandardBuiltinId::StringPrototypeToUpperCase)
                                    }
                                    "toString" => Some(StandardBuiltinId::StringPrototypeToString),
                                    "valueOf" => Some(StandardBuiltinId::StringPrototypeValueOf),
                                    "trim" => Some(StandardBuiltinId::StringPrototypeTrim),
                                    "trimStart" | "trimLeft" => {
                                        Some(StandardBuiltinId::StringPrototypeTrimStart)
                                    }
                                    "trimEnd" | "trimRight" => {
                                        Some(StandardBuiltinId::StringPrototypeTrimEnd)
                                    }
                                    "isWellFormed" => {
                                        Some(StandardBuiltinId::StringPrototypeIsWellFormed)
                                    }
                                    "toWellFormed" => {
                                        Some(StandardBuiltinId::StringPrototypeToWellFormed)
                                    }
                                    _ => None,
                                };
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else {
                                    TypedExpr::from_info(
                                        ValueInfo {
                                            kind: ValueKind::Dynamic,
                                            possible_kinds: KindSet::all_runtime_tags(),
                                            heap_shape: None,
                                            function_targets: BTreeSet::new(),
                                        },
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                }
                            } else {
                                return self
                                    .unsupported_expr("indirect call: dynamic string property");
                            }
                        }
                        ValueKind::Number => {
                            if let PropertyAccessField::Const(field) = access.field() {
                                let field_name =
                                    self.interner.resolve_expect(field.sym()).to_string();
                                if field_name == "split"
                                    && self.number_prototype_split_is_string_split
                                    && args.len() <= 2
                                {
                                    if args.first().is_some_and(|separator| {
                                        self.static_to_string_returns_regexp_object_expr(separator)
                                    }) {
                                        for arg in args {
                                            self.lower_expression(arg);
                                        }
                                        return TypedExpr::from_info(
                                            ValueInfo::undefined(),
                                            ExprIr::RuntimeThrow {
                                                name: NativeErrorKind::TypeError,
                                                message: "Cannot convert object to primitive value",
                                            },
                                        );
                                    }
                                    let Some(args) = self.lower_call_args_expanding_spread(args)
                                    else {
                                        return TypedExpr::undefined();
                                    };
                                    return TypedExpr::from_info(
                                        ValueInfo {
                                            kind: ValueKind::Array,
                                            possible_kinds: KindSet::from_kind(ValueKind::Array),
                                            heap_shape: Some(Box::new(HeapShape::Array(
                                                ArrayShape::default(),
                                            ))),
                                            function_targets: BTreeSet::new(),
                                        },
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(field_name),
                                            args,
                                        },
                                    );
                                }
                                if field_name == "match"
                                    && self.number_prototype_match_is_string_match
                                    && args.len() <= 1
                                {
                                    let Some(args) = self.lower_call_args_expanding_spread(args)
                                    else {
                                        return TypedExpr::undefined();
                                    };
                                    return TypedExpr::from_info(
                                        ValueInfo {
                                            kind: ValueKind::Dynamic,
                                            possible_kinds: KindSet::all_runtime_tags(),
                                            heap_shape: None,
                                            function_targets: BTreeSet::new(),
                                        },
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(field_name),
                                            args,
                                        },
                                    );
                                }
                                let builtin = match field_name.as_str() {
                                    "toExponential" => {
                                        Some(StandardBuiltinId::NumberPrototypeToExponential)
                                    }
                                    "toFixed" => Some(StandardBuiltinId::NumberPrototypeToFixed),
                                    "toLocaleString" => {
                                        Some(StandardBuiltinId::NumberPrototypeToLocaleString)
                                    }
                                    "toPrecision" => {
                                        Some(StandardBuiltinId::NumberPrototypeToPrecision)
                                    }
                                    "toString"
                                        if self.number_prototype_to_string_state
                                            == PrototypeToStringState::Intrinsic =>
                                    {
                                        Some(StandardBuiltinId::NumberPrototypeToString)
                                    }
                                    "valueOf" => Some(StandardBuiltinId::NumberPrototypeValueOf),
                                    _ => None,
                                };
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else {
                                    TypedExpr::from_info(
                                        ValueInfo {
                                            kind: ValueKind::Dynamic,
                                            possible_kinds: KindSet::all_runtime_tags(),
                                            heap_shape: None,
                                            function_targets: BTreeSet::new(),
                                        },
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                }
                            } else {
                                return self
                                    .unsupported_expr("indirect call: dynamic number property");
                            }
                        }
                        ValueKind::Boolean => {
                            if let PropertyAccessField::Const(field) = access.field() {
                                let field_name =
                                    self.interner.resolve_expect(field.sym()).to_string();
                                let builtin = match field_name.as_str() {
                                    "toString"
                                        if self.boolean_prototype_to_string_state
                                            == PrototypeToStringState::Intrinsic =>
                                    {
                                        Some(StandardBuiltinId::BooleanPrototypeToString)
                                    }
                                    "valueOf" => Some(StandardBuiltinId::BooleanPrototypeValueOf),
                                    _ => None,
                                };
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else {
                                    TypedExpr::from_info(
                                        ValueInfo {
                                            kind: ValueKind::Dynamic,
                                            possible_kinds: KindSet::all_runtime_tags(),
                                            heap_shape: None,
                                            function_targets: BTreeSet::new(),
                                        },
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                }
                            } else {
                                return self
                                    .unsupported_expr("indirect call: dynamic boolean property");
                            }
                        }
                        ValueKind::BigInt => {
                            if let PropertyAccessField::Const(field) = access.field() {
                                let field_name =
                                    self.interner.resolve_expect(field.sym()).to_string();
                                let builtin = match field_name.as_str() {
                                    "toString" => Some(StandardBuiltinId::BigIntPrototypeToString),
                                    "toLocaleString" => {
                                        Some(StandardBuiltinId::BigIntPrototypeToLocaleString)
                                    }
                                    "valueOf" => Some(StandardBuiltinId::BigIntPrototypeValueOf),
                                    _ => None,
                                };
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else {
                                    return self.unsupported_expr(
                                        "indirect call: unsupported bigint property",
                                    );
                                }
                            } else {
                                return self
                                    .unsupported_expr("indirect call: dynamic bigint property");
                            }
                        }
                        ValueKind::Symbol => {
                            if let PropertyAccessField::Const(field) = access.field() {
                                let field_name =
                                    self.interner.resolve_expect(field.sym()).to_string();
                                let builtin = match field_name.as_str() {
                                    "toString" => Some(StandardBuiltinId::SymbolPrototypeToString),
                                    "valueOf" => Some(StandardBuiltinId::SymbolPrototypeValueOf),
                                    _ => None,
                                };
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else {
                                    // Anything else (e.g. `hasOwnProperty`,
                                    // `constructor`) is inherited from
                                    // `Object.prototype` via
                                    // `Symbol.prototype`'s own
                                    // `[[Prototype]]`; resolve it through the
                                    // generic runtime prototype-chain lookup
                                    // rather than hardcoding every name.
                                    self.lower_object_property_key(receiver.clone(), access.field())
                                }
                            } else if let PropertyAccessField::Expr(expr) = access.field() {
                                if let Some((symbol, symbol_key)) =
                                    self.lower_well_known_symbol_property_key(expr)
                                {
                                    if symbol == WellKnownSymbol::ToPrimitive {
                                        TypedExpr::from_info(
                                            Self::standard_builtin_value_info(
                                                StandardBuiltinId::SymbolPrototypeToPrimitive,
                                            ),
                                            ExprIr::PropertyRead {
                                                target: Box::new(receiver.clone()),
                                                key: symbol_key,
                                            },
                                        )
                                    } else {
                                        return self.unsupported_expr(
                                            "indirect call: dynamic symbol property",
                                        );
                                    }
                                } else {
                                    return self.unsupported_expr(
                                        "indirect call: dynamic symbol property",
                                    );
                                }
                            } else {
                                return self
                                    .unsupported_expr("indirect call: dynamic symbol property");
                            }
                        }
                        ValueKind::Array
                            if self.array_prototype_mutated
                                || Self::array_shape_has_custom_prototype(&receiver)
                                || receiver_has_known_own_property =>
                        {
                            self.lower_object_property_key(receiver.clone(), access.field())
                        }
                        ValueKind::Array => {
                            if let PropertyAccessField::Const(field) = access.field() {
                                let field_name =
                                    self.interner.resolve_expect(field.sym()).to_string();
                                if field_name == "splice"
                                    && Self::static_splice_delete_count_is_supported(args)
                                {
                                    let Some((key, args)) = self.lower_splice_zero_call_args(args)
                                    else {
                                        return self.unsupported_expr("call spread");
                                    };
                                    return TypedExpr::from_info(
                                        Self::array_value_info_from_elements(Vec::new()),
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(key),
                                            args,
                                        },
                                    );
                                }
                                if field_name == "join" && args.len() <= 1 {
                                    let Some(args) = self.lower_call_args_expanding_spread(args)
                                    else {
                                        return TypedExpr::undefined();
                                    };
                                    return TypedExpr::from_info(
                                        ValueInfo::new(ValueKind::String),
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(field_name),
                                            args,
                                        },
                                    );
                                }
                                if field_name == "reverse" && args.is_empty() {
                                    return TypedExpr::from_info(
                                        receiver.value_info(),
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(field_name),
                                            args: Vec::new(),
                                        },
                                    );
                                }
                                let builtin = match field_name.as_str() {
                                    "pop" => Some(StandardBuiltinId::ArrayPrototypePop),
                                    "push" => Some(StandardBuiltinId::ArrayPrototypePush),
                                    "shift" => Some(StandardBuiltinId::ArrayPrototypeShift),
                                    "unshift" => Some(StandardBuiltinId::ArrayPrototypeUnshift),
                                    "fill" => Some(StandardBuiltinId::ArrayPrototypeFill),
                                    "sort" => Some(StandardBuiltinId::ArrayPrototypeSort),
                                    "keys" => Some(StandardBuiltinId::ArrayPrototypeKeys),
                                    "entries" => Some(StandardBuiltinId::ArrayPrototypeEntries),
                                    "values" => Some(StandardBuiltinId::ArrayPrototypeValues),
                                    "concat" => Some(StandardBuiltinId::ArrayPrototypeConcat),
                                    "join" => Some(StandardBuiltinId::ArrayPrototypeJoin),
                                    "slice" => Some(StandardBuiltinId::ArrayPrototypeSlice),
                                    "splice" => Some(StandardBuiltinId::ArrayPrototypeSplice),
                                    "toString" => {
                                        Some(StandardBuiltinId::TypedArrayPrototypeToString)
                                    }
                                    "toLocaleString" => {
                                        Some(StandardBuiltinId::ArrayPrototypeToLocaleString)
                                    }
                                    "flat" => Some(StandardBuiltinId::ArrayPrototypeFlat),
                                    "flatMap" => Some(StandardBuiltinId::ArrayPrototypeFlatMap),
                                    "at" => Some(StandardBuiltinId::ArrayPrototypeAt),
                                    "toReversed" => {
                                        Some(StandardBuiltinId::ArrayPrototypeToReversed)
                                    }
                                    "toSpliced" => Some(StandardBuiltinId::ArrayPrototypeToSpliced),
                                    "toSorted" => Some(StandardBuiltinId::ArrayPrototypeToSorted),
                                    "with" => Some(StandardBuiltinId::ArrayPrototypeWith),
                                    "reverse" => Some(StandardBuiltinId::ArrayPrototypeReverse),
                                    "copyWithin" => {
                                        Some(StandardBuiltinId::ArrayPrototypeCopyWithin)
                                    }
                                    "includes" => Some(StandardBuiltinId::ArrayPrototypeIncludes),
                                    "indexOf" => Some(StandardBuiltinId::ArrayPrototypeIndexOf),
                                    "lastIndexOf" => {
                                        Some(StandardBuiltinId::ArrayPrototypeLastIndexOf)
                                    }
                                    "find" => Some(StandardBuiltinId::ArrayPrototypeFind),
                                    "findIndex" => Some(StandardBuiltinId::ArrayPrototypeFindIndex),
                                    "findLast" => Some(StandardBuiltinId::ArrayPrototypeFindLast),
                                    "findLastIndex" => {
                                        Some(StandardBuiltinId::ArrayPrototypeFindLastIndex)
                                    }
                                    "every" => Some(StandardBuiltinId::ArrayPrototypeEvery),
                                    "some" => Some(StandardBuiltinId::ArrayPrototypeSome),
                                    "forEach" => Some(StandardBuiltinId::ArrayPrototypeForEach),
                                    "filter" => Some(StandardBuiltinId::ArrayPrototypeFilter),
                                    "map" => Some(StandardBuiltinId::ArrayPrototypeMap),
                                    "reduce" => Some(StandardBuiltinId::ArrayPrototypeReduce),
                                    "reduceRight" => {
                                        Some(StandardBuiltinId::ArrayPrototypeReduceRight)
                                    }
                                    _ => None,
                                };
                                if field_name == "forEach" {
                                    let Some(args) = self.lower_call_args_expanding_spread(args)
                                    else {
                                        return TypedExpr::undefined();
                                    };
                                    if let Some(callback) = args.first() {
                                        if let Some(function_id) =
                                            self.resolve_single_function_target(callback)
                                        {
                                            let callback_arg_infos = [
                                                ValueInfo {
                                                    kind: ValueKind::Dynamic,
                                                    possible_kinds: KindSet::all_runtime_tags(),
                                                    heap_shape: None,
                                                    function_targets: BTreeSet::new(),
                                                },
                                                ValueInfo::new(ValueKind::Number),
                                                receiver.value_info(),
                                            ];
                                            let callback_this_info = args
                                                .get(1)
                                                .map(TypedExpr::value_info)
                                                .unwrap_or_else(ValueInfo::undefined);
                                            self.merge_function_param_infos(
                                                &function_id,
                                                &callback_arg_infos,
                                            );
                                            self.merge_function_this_info(
                                                &function_id,
                                                callback_this_info.clone(),
                                            );
                                            let original_function_id =
                                                self.original_exact_function_id(&function_id);
                                            if let Some(helper_context_id) = self
                                                .exact_context_callback_targets
                                                .get(&original_function_id)
                                                .cloned()
                                            {
                                                self.observe_exact_callback_param_infos(
                                                    &original_function_id,
                                                    &helper_context_id,
                                                    &callback_arg_infos,
                                                );
                                                self.observe_exact_callback_this_info(
                                                    &original_function_id,
                                                    &helper_context_id,
                                                    callback_this_info,
                                                );
                                            }
                                        }
                                    }
                                    return TypedExpr::from_info(
                                        ValueInfo::undefined(),
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(field_name),
                                            args,
                                        },
                                    );
                                }
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else {
                                    self.lower_array_index_key(receiver.clone(), access.field())
                                }
                            } else {
                                self.lower_array_index_key(receiver.clone(), access.field())
                            }
                        }
                        ValueKind::Arguments => {
                            self.lower_arguments_index_key(receiver.clone(), access.field())
                        }
                        ValueKind::Dynamic
                            if receiver.heap_shape.is_some()
                                && matches!(access.field(), PropertyAccessField::Const(_)) =>
                        {
                            self.lower_object_property_key(receiver.clone(), access.field())
                        }
                        ValueKind::Dynamic
                            if receiver.possible_kinds.contains(ValueKind::Function) =>
                        {
                            if let PropertyAccessField::Const(field) = access.field() {
                                let field_name =
                                    self.interner.resolve_expect(field.sym()).to_string();
                                let builtin = match field_name.as_str() {
                                    "call" => Some(StandardBuiltinId::FunctionPrototypeCall),
                                    "apply" => Some(StandardBuiltinId::FunctionPrototypeApply),
                                    "bind" => Some(StandardBuiltinId::FunctionPrototypeBind),
                                    "toString" => {
                                        Some(StandardBuiltinId::FunctionPrototypeToString)
                                    }
                                    "push"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypePush)
                                    }
                                    "shift"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeShift)
                                    }
                                    "unshift"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeUnshift)
                                    }
                                    "concat"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeConcat)
                                    }
                                    "toLocaleString"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeToLocaleString)
                                    }
                                    "flat"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeFlat)
                                    }
                                    "flatMap"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeFlatMap)
                                    }
                                    "at" if receiver.possible_kinds.contains(ValueKind::Array) => {
                                        Some(StandardBuiltinId::ArrayPrototypeAt)
                                    }
                                    "toReversed"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeToReversed)
                                    }
                                    "with"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeWith)
                                    }
                                    "toSpliced"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeToSpliced)
                                    }
                                    "toSorted"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeToSorted)
                                    }
                                    "reverse"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeReverse)
                                    }
                                    "copyWithin"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeCopyWithin)
                                    }
                                    "includes"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeIncludes)
                                    }
                                    "indexOf"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeIndexOf)
                                    }
                                    "lastIndexOf"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeLastIndexOf)
                                    }
                                    "find"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeFind)
                                    }
                                    "findIndex"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeFindIndex)
                                    }
                                    "findLast"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeFindLast)
                                    }
                                    "findLastIndex"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeFindLastIndex)
                                    }
                                    "every"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeEvery)
                                    }
                                    "some"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeSome)
                                    }
                                    "forEach"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeForEach)
                                    }
                                    "filter"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeFilter)
                                    }
                                    "map"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeMap)
                                    }
                                    "reduce"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeReduce)
                                    }
                                    "reduceRight"
                                        if receiver.possible_kinds.contains(ValueKind::Array) =>
                                    {
                                        Some(StandardBuiltinId::ArrayPrototypeReduceRight)
                                    }
                                    "getUint8"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetUint8)
                                    }
                                    "setUint8"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetUint8)
                                    }
                                    "getInt8"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetInt8)
                                    }
                                    "setInt8"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetInt8)
                                    }
                                    "getUint16"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetUint16)
                                    }
                                    "setUint16"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetUint16)
                                    }
                                    "getInt16"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetInt16)
                                    }
                                    "setInt16"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetInt16)
                                    }
                                    "getUint32"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetUint32)
                                    }
                                    "setUint32"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetUint32)
                                    }
                                    "getInt32"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetInt32)
                                    }
                                    "setInt32"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetInt32)
                                    }
                                    "getFloat16"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetFloat16)
                                    }
                                    "setFloat16"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetFloat16)
                                    }
                                    "getFloat32"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetFloat32)
                                    }
                                    "setFloat32"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetFloat32)
                                    }
                                    "getFloat64"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetFloat64)
                                    }
                                    "setFloat64"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetFloat64)
                                    }
                                    "getBigInt64"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetBigInt64)
                                    }
                                    "setBigInt64"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetBigInt64)
                                    }
                                    "getBigUint64"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeGetBigUint64)
                                    }
                                    "setBigUint64"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::DataViewPrototypeSetBigUint64)
                                    }
                                    "resize"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::ArrayBufferPrototypeResize)
                                    }
                                    "transfer"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(StandardBuiltinId::ArrayBufferPrototypeTransfer)
                                    }
                                    "transferToFixedLength"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(
                                            StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength,
                                        )
                                    }
                                    "transferToImmutable"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(
                                            StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable,
                                        )
                                    }
                                    "sliceToImmutable"
                                        if receiver.possible_kinds.contains(ValueKind::Object) =>
                                    {
                                        Some(
                                            StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable,
                                        )
                                    }
                                    _ => None,
                                };
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else if receiver.possible_kinds.contains(ValueKind::Object) {
                                    self.lower_object_property_key(receiver.clone(), access.field())
                                } else {
                                    return self.unsupported_expr(
                                        "indirect call: unsupported dynamic function property",
                                    );
                                }
                            } else {
                                self.lower_object_property_key(receiver.clone(), access.field())
                            }
                        }
                        ValueKind::Dynamic
                            if receiver.possible_kinds.contains(ValueKind::Array) =>
                        {
                            if let PropertyAccessField::Const(field) = access.field() {
                                let field_name =
                                    self.interner.resolve_expect(field.sym()).to_string();
                                if field_name == "forEach" {
                                    let Some(args) = self.lower_call_args_expanding_spread(args)
                                    else {
                                        return TypedExpr::undefined();
                                    };
                                    if let Some(callback) = args.first() {
                                        if let Some(function_id) =
                                            self.resolve_single_function_target(callback)
                                        {
                                            self.merge_function_param_infos(
                                                &function_id,
                                                &[
                                                    ValueInfo {
                                                        kind: ValueKind::Dynamic,
                                                        possible_kinds: KindSet::all_runtime_tags(),
                                                        heap_shape: None,
                                                        function_targets: BTreeSet::new(),
                                                    },
                                                    ValueInfo::new(ValueKind::Number),
                                                ],
                                            );
                                        }
                                    }
                                    return TypedExpr::from_info(
                                        ValueInfo::undefined(),
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(field_name),
                                            args,
                                        },
                                    );
                                }
                                if field_name == "splice"
                                    && Self::static_splice_delete_count_is_supported(args)
                                {
                                    let Some((key, args)) = self.lower_splice_zero_call_args(args)
                                    else {
                                        return self.unsupported_expr("call spread");
                                    };
                                    return TypedExpr::from_info(
                                        Self::array_value_info_from_elements(Vec::new()),
                                        ExprIr::CallMethod {
                                            receiver: Box::new(receiver),
                                            key: PropertyKeyIr::StaticString(key),
                                            args,
                                        },
                                    );
                                }
                                let builtin = match field_name.as_str() {
                                    "pop" => Some(StandardBuiltinId::ArrayPrototypePop),
                                    "push" => Some(StandardBuiltinId::ArrayPrototypePush),
                                    "shift" => Some(StandardBuiltinId::ArrayPrototypeShift),
                                    "unshift" => Some(StandardBuiltinId::ArrayPrototypeUnshift),
                                    "fill" => Some(StandardBuiltinId::ArrayPrototypeFill),
                                    "sort" => Some(StandardBuiltinId::ArrayPrototypeSort),
                                    "keys" => Some(StandardBuiltinId::ArrayPrototypeKeys),
                                    "entries" => Some(StandardBuiltinId::ArrayPrototypeEntries),
                                    "values" => Some(StandardBuiltinId::ArrayPrototypeValues),
                                    "concat" => Some(StandardBuiltinId::ArrayPrototypeConcat),
                                    "join" => Some(StandardBuiltinId::ArrayPrototypeJoin),
                                    "slice" => Some(StandardBuiltinId::ArrayPrototypeSlice),
                                    "splice" => Some(StandardBuiltinId::ArrayPrototypeSplice),
                                    "toString" => {
                                        Some(StandardBuiltinId::TypedArrayPrototypeToString)
                                    }
                                    "toLocaleString" => {
                                        Some(StandardBuiltinId::ArrayPrototypeToLocaleString)
                                    }
                                    "flat" => Some(StandardBuiltinId::ArrayPrototypeFlat),
                                    "flatMap" => Some(StandardBuiltinId::ArrayPrototypeFlatMap),
                                    "at" => Some(StandardBuiltinId::ArrayPrototypeAt),
                                    "toReversed" => {
                                        Some(StandardBuiltinId::ArrayPrototypeToReversed)
                                    }
                                    "toSpliced" => Some(StandardBuiltinId::ArrayPrototypeToSpliced),
                                    "toSorted" => Some(StandardBuiltinId::ArrayPrototypeToSorted),
                                    "with" => Some(StandardBuiltinId::ArrayPrototypeWith),
                                    "reverse" => Some(StandardBuiltinId::ArrayPrototypeReverse),
                                    "copyWithin" => {
                                        Some(StandardBuiltinId::ArrayPrototypeCopyWithin)
                                    }
                                    "includes" => Some(StandardBuiltinId::ArrayPrototypeIncludes),
                                    "indexOf" => Some(StandardBuiltinId::ArrayPrototypeIndexOf),
                                    "lastIndexOf" => {
                                        Some(StandardBuiltinId::ArrayPrototypeLastIndexOf)
                                    }
                                    "find" => Some(StandardBuiltinId::ArrayPrototypeFind),
                                    "findIndex" => Some(StandardBuiltinId::ArrayPrototypeFindIndex),
                                    "findLast" => Some(StandardBuiltinId::ArrayPrototypeFindLast),
                                    "findLastIndex" => {
                                        Some(StandardBuiltinId::ArrayPrototypeFindLastIndex)
                                    }
                                    "every" => Some(StandardBuiltinId::ArrayPrototypeEvery),
                                    "some" => Some(StandardBuiltinId::ArrayPrototypeSome),
                                    "forEach" => Some(StandardBuiltinId::ArrayPrototypeForEach),
                                    "filter" => Some(StandardBuiltinId::ArrayPrototypeFilter),
                                    "map" => Some(StandardBuiltinId::ArrayPrototypeMap),
                                    "reduce" => Some(StandardBuiltinId::ArrayPrototypeReduce),
                                    "reduceRight" => {
                                        Some(StandardBuiltinId::ArrayPrototypeReduceRight)
                                    }
                                    _ => None,
                                };
                                if let Some(builtin) = builtin {
                                    TypedExpr::from_info(
                                        Self::standard_builtin_value_info(builtin),
                                        ExprIr::PropertyRead {
                                            target: Box::new(receiver.clone()),
                                            key: PropertyKeyIr::StaticString(field_name),
                                        },
                                    )
                                } else {
                                    return self.unsupported_expr(
                                        "indirect call: unsupported dynamic array property",
                                    );
                                }
                            } else {
                                self.lower_object_property_key(receiver.clone(), access.field())
                            }
                        }
                        ValueKind::Dynamic => {
                            self.lower_object_property_key(receiver.clone(), access.field())
                        }
                        _ => {
                            let dynamic_receiver = TypedExpr::from_info(
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: receiver.heap_shape.clone(),
                                    function_targets: BTreeSet::new(),
                                },
                                receiver.expr.clone(),
                            );
                            self.lower_object_property_key(dynamic_receiver, access.field())
                        }
                    };
                    if callee.kind != ValueKind::Function {
                        let Some(args) = self.lower_call_args_expanding_spread(args) else {
                            return TypedExpr::undefined();
                        };
                        return self.lower_indirect_method_call(
                            ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            },
                            callee,
                            receiver,
                            args,
                            None,
                        );
                    }
                    let Some(function_id) = self.resolve_single_function_target(&callee) else {
                        let Some(args) = self.lower_call_args_expanding_spread(args) else {
                            return TypedExpr::undefined();
                        };
                        return self.lower_indirect_method_call(
                            ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            },
                            callee,
                            receiver,
                            args,
                            None,
                        );
                    };
                    let Some(signature) = self.function_signatures.get(&function_id) else {
                        return self
                            .unsupported_expr("indirect call: missing property target signature");
                    };
                    if !signature.callable
                        && signature.protocol.class_kind() != ClassFunctionKind::Constructor
                    {
                        return unsupported_call(self, signature.protocol.class_kind());
                    }
                    self.mark_host_builtin_from_function_id(&function_id);
                    self.host_builtin_calls +=
                        usize::from(HostBuiltinId::from_function_id(&function_id).is_some());
                    self.merge_function_this_info(&function_id, receiver.value_info());
                    if let Some(
                        array_builtin @ (StandardBuiltinId::ArrayPrototypePush
                        | StandardBuiltinId::ArrayPrototypePop
                        | StandardBuiltinId::ArrayPrototypeShift
                        | StandardBuiltinId::ArrayPrototypeUnshift
                        | StandardBuiltinId::ArrayPrototypeFill
                        | StandardBuiltinId::ArrayPrototypeSort
                        | StandardBuiltinId::ArrayPrototypeKeys
                        | StandardBuiltinId::ArrayPrototypeEntries
                        | StandardBuiltinId::ArrayPrototypeValues
                        | StandardBuiltinId::TypedArrayPrototypeKeys
                        | StandardBuiltinId::TypedArrayPrototypeEntries
                        | StandardBuiltinId::TypedArrayPrototypeValues
                        | StandardBuiltinId::ArrayPrototypeConcat
                        | StandardBuiltinId::ArrayPrototypeJoin
                        | StandardBuiltinId::ArrayPrototypeSlice
                        | StandardBuiltinId::ArrayPrototypeSplice
                        | StandardBuiltinId::TypedArrayPrototypeToString
                        | StandardBuiltinId::ArrayPrototypeToLocaleString
                        | StandardBuiltinId::ArrayPrototypeFlat
                        | StandardBuiltinId::ArrayPrototypeFlatMap
                        | StandardBuiltinId::ArrayPrototypeAt
                        | StandardBuiltinId::ArrayPrototypeToReversed
                        | StandardBuiltinId::ArrayPrototypeToSpliced
                        | StandardBuiltinId::ArrayPrototypeToSorted
                        | StandardBuiltinId::ArrayPrototypeWith
                        | StandardBuiltinId::ArrayPrototypeReverse
                        | StandardBuiltinId::ArrayPrototypeCopyWithin
                        | StandardBuiltinId::ArrayPrototypeIncludes
                        | StandardBuiltinId::ArrayPrototypeIndexOf
                        | StandardBuiltinId::ArrayPrototypeLastIndexOf
                        | StandardBuiltinId::TypedArrayPrototypeIncludes
                        | StandardBuiltinId::TypedArrayPrototypeIndexOf
                        | StandardBuiltinId::TypedArrayPrototypeLastIndexOf
                        | StandardBuiltinId::ArrayPrototypeFind
                        | StandardBuiltinId::ArrayPrototypeFindIndex
                        | StandardBuiltinId::TypedArrayPrototypeFind
                        | StandardBuiltinId::TypedArrayPrototypeFindIndex
                        | StandardBuiltinId::TypedArrayPrototypeFindLast
                        | StandardBuiltinId::TypedArrayPrototypeFindLastIndex
                        | StandardBuiltinId::TypedArrayPrototypeEvery
                        | StandardBuiltinId::TypedArrayPrototypeSome
                        | StandardBuiltinId::TypedArrayPrototypeMap
                        | StandardBuiltinId::TypedArrayPrototypeFilter
                        | StandardBuiltinId::TypedArrayPrototypeForEach
                        | StandardBuiltinId::TypedArrayPrototypeReduce
                        | StandardBuiltinId::TypedArrayPrototypeReduceRight
                        | StandardBuiltinId::ArrayPrototypeFindLast
                        | StandardBuiltinId::ArrayPrototypeFindLastIndex
                        | StandardBuiltinId::ArrayPrototypeEvery
                        | StandardBuiltinId::ArrayPrototypeSome
                        | StandardBuiltinId::ArrayPrototypeForEach
                        | StandardBuiltinId::ArrayPrototypeFilter
                        | StandardBuiltinId::ArrayPrototypeMap
                        | StandardBuiltinId::ArrayPrototypeReduce
                        | StandardBuiltinId::ArrayPrototypeReduceRight),
                    ) = StandardBuiltinId::from_function_id(&function_id)
                    {
                        let Some(args) = self.lower_call_args_expanding_spread(args) else {
                            return TypedExpr::undefined();
                        };
                        // An Array.prototype method can be copied onto an
                        // arbitrary object after the prototype was mutated.
                        // Its inferred builtin target is not enough to prove
                        // that the receiver satisfies the Array fast path.
                        if self.array_prototype_mutated
                            && !receiver
                                .possible_kinds
                                .is_subset_of(KindSet::from_kind(ValueKind::Array))
                        {
                            return self.lower_indirect_method_call(
                                ValueInfo::new(ValueKind::Dynamic),
                                callee,
                                receiver,
                                args,
                                None,
                            );
                        }
                        if array_builtin == StandardBuiltinId::ArrayPrototypePush {
                            if let Some(base_len) = Self::static_array_shape_len(&receiver) {
                                for (arg_offset, arg) in args.iter().enumerate() {
                                    let index = base_len + arg_offset;
                                    if index > MAX_STATIC_ARRAY_SHAPE_INDEX {
                                        self.clear_binding_shape(access.target());
                                        break;
                                    }
                                    let key =
                                        PropertyKeyIr::ArrayIndex(Box::new(TypedExpr::from_info(
                                            ValueInfo::new(ValueKind::Number),
                                            ExprIr::Number((index as f64).to_bits()),
                                        )));
                                    self.update_written_shape(
                                        access.target(),
                                        &key,
                                        &arg.value_info(),
                                    );
                                }
                            } else {
                                self.clear_binding_shape(access.target());
                            }
                        } else if matches!(
                            array_builtin,
                            StandardBuiltinId::ArrayPrototypeShift
                                | StandardBuiltinId::ArrayPrototypeUnshift
                        ) {
                            self.clear_binding_shape(access.target());
                        }
                        if matches!(
                            array_builtin,
                            StandardBuiltinId::ArrayPrototypeConcat
                                | StandardBuiltinId::ArrayPrototypeSlice
                                | StandardBuiltinId::ArrayPrototypeSplice
                                | StandardBuiltinId::ArrayPrototypeFlat
                                | StandardBuiltinId::ArrayPrototypeFlatMap
                                | StandardBuiltinId::ArrayPrototypeFilter
                                | StandardBuiltinId::ArrayPrototypeMap
                        ) {
                            self.merge_array_species_constructor_this_info(&receiver);
                        }
                        if matches!(
                            array_builtin,
                            StandardBuiltinId::ArrayPrototypeReduce
                                | StandardBuiltinId::ArrayPrototypeReduceRight
                                | StandardBuiltinId::TypedArrayPrototypeReduce
                                | StandardBuiltinId::TypedArrayPrototypeReduceRight
                        ) {
                            if let Some(callback) = args.first() {
                                if let Some(callback_id) =
                                    self.resolve_single_function_target(callback)
                                {
                                    self.merge_function_param_infos(
                                        &callback_id,
                                        &[
                                            ValueInfo {
                                                kind: ValueKind::Dynamic,
                                                possible_kinds: KindSet::all_runtime_tags(),
                                                heap_shape: None,
                                                function_targets: BTreeSet::new(),
                                            },
                                            ValueInfo {
                                                kind: ValueKind::Dynamic,
                                                possible_kinds: KindSet::all_runtime_tags(),
                                                heap_shape: None,
                                                function_targets: BTreeSet::new(),
                                            },
                                            ValueInfo::new(ValueKind::Number),
                                            receiver.value_info(),
                                        ],
                                    );
                                    self.merge_function_this_info(
                                        &callback_id,
                                        ValueInfo::undefined(),
                                    );
                                }
                            }
                        } else if matches!(
                            array_builtin,
                            StandardBuiltinId::ArrayPrototypeFlatMap
                                | StandardBuiltinId::ArrayPrototypeFind
                                | StandardBuiltinId::ArrayPrototypeFindIndex
                                | StandardBuiltinId::ArrayPrototypeFindLast
                                | StandardBuiltinId::ArrayPrototypeFindLastIndex
                                | StandardBuiltinId::ArrayPrototypeEvery
                                | StandardBuiltinId::ArrayPrototypeSome
                                | StandardBuiltinId::TypedArrayPrototypeMap
                                | StandardBuiltinId::TypedArrayPrototypeFilter
                                | StandardBuiltinId::TypedArrayPrototypeForEach
                                | StandardBuiltinId::ArrayPrototypeForEach
                                | StandardBuiltinId::ArrayPrototypeFilter
                                | StandardBuiltinId::ArrayPrototypeMap
                        ) {
                            if let Some(callback) = args.first() {
                                if let Some(callback_id) =
                                    self.resolve_single_function_target(callback)
                                {
                                    let callback_arg_infos = [
                                        ValueInfo {
                                            kind: ValueKind::Dynamic,
                                            possible_kinds: KindSet::all_runtime_tags(),
                                            heap_shape: None,
                                            function_targets: BTreeSet::new(),
                                        },
                                        ValueInfo::new(ValueKind::Number),
                                        receiver.value_info(),
                                    ];
                                    let callback_this_info = args
                                        .get(1)
                                        .map(TypedExpr::value_info)
                                        .unwrap_or_else(ValueInfo::undefined);
                                    self.merge_function_param_infos(
                                        &callback_id,
                                        &callback_arg_infos,
                                    );
                                    self.merge_function_this_info(
                                        &callback_id,
                                        callback_this_info.clone(),
                                    );
                                    let original_callback_id =
                                        self.original_exact_function_id(&callback_id);
                                    if let Some(helper_context_id) = self
                                        .exact_context_callback_targets
                                        .get(&original_callback_id)
                                        .cloned()
                                    {
                                        self.observe_exact_callback_param_infos(
                                            &original_callback_id,
                                            &helper_context_id,
                                            &callback_arg_infos,
                                        );
                                        self.observe_exact_callback_this_info(
                                            &original_callback_id,
                                            &helper_context_id,
                                            callback_this_info,
                                        );
                                    }
                                }
                            }
                        }
                        if array_builtin == StandardBuiltinId::ArrayPrototypeSort {
                            if let Some(callback_id) = args
                                .first()
                                .and_then(|callback| self.resolve_single_function_target(callback))
                            {
                                self.merge_function_param_infos(
                                    &callback_id,
                                    &[
                                        ValueInfo {
                                            kind: ValueKind::Dynamic,
                                            possible_kinds: KindSet::all_runtime_tags(),
                                            heap_shape: None,
                                            function_targets: BTreeSet::new(),
                                        },
                                        ValueInfo {
                                            kind: ValueKind::Dynamic,
                                            possible_kinds: KindSet::all_runtime_tags(),
                                            heap_shape: None,
                                            function_targets: BTreeSet::new(),
                                        },
                                    ],
                                );
                                self.merge_function_this_info(&callback_id, ValueInfo::undefined());
                            }
                        }
                        let (key, info) = match array_builtin {
                            StandardBuiltinId::ArrayPrototypePush => {
                                ("push", ValueInfo::new(ValueKind::Number))
                            }
                            StandardBuiltinId::ArrayPrototypeShift => {
                                ("shift", ValueInfo::new(ValueKind::Dynamic))
                            }
                            StandardBuiltinId::ArrayPrototypeUnshift => {
                                ("unshift", ValueInfo::new(ValueKind::Number))
                            }
                            StandardBuiltinId::ArrayPrototypeFill => {
                                ("fill", receiver.value_info())
                            }
                            StandardBuiltinId::ArrayPrototypeSort => {
                                ("sort", receiver.value_info())
                            }
                            StandardBuiltinId::ArrayPrototypeConcat => {
                                ("concat", self.array_concat_result_info(&receiver, &args))
                            }
                            StandardBuiltinId::ArrayPrototypeJoin => {
                                ("join", ValueInfo::new(ValueKind::String))
                            }
                            StandardBuiltinId::ArrayPrototypeSlice => {
                                ("slice", Self::unshaped_array_result_info())
                            }
                            StandardBuiltinId::ArrayPrototypeSplice => {
                                ("splice", Self::unshaped_array_result_info())
                            }
                            StandardBuiltinId::TypedArrayPrototypeToString => {
                                ("toString", ValueInfo::new(ValueKind::String))
                            }
                            StandardBuiltinId::ArrayPrototypeToLocaleString => {
                                ("toLocaleString", ValueInfo::new(ValueKind::String))
                            }
                            StandardBuiltinId::ArrayPrototypeFlat => {
                                ("flat", Self::unshaped_array_result_info())
                            }
                            StandardBuiltinId::ArrayPrototypeFlatMap => {
                                ("flatMap", Self::unshaped_array_result_info())
                            }
                            StandardBuiltinId::ArrayPrototypeAt => (
                                "at",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeToReversed => {
                                ("toReversed", Self::unshaped_array_result_info())
                            }
                            StandardBuiltinId::ArrayPrototypeWith => {
                                ("with", Self::unshaped_array_result_info())
                            }
                            StandardBuiltinId::ArrayPrototypeToSpliced => {
                                ("toSpliced", Self::unshaped_array_result_info())
                            }
                            StandardBuiltinId::ArrayPrototypeToSorted => {
                                ("toSorted", Self::unshaped_array_result_info())
                            }
                            StandardBuiltinId::ArrayPrototypeReverse => (
                                "reverse",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: Self::object_like_kind_set(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeCopyWithin => (
                                "copyWithin",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: Self::object_like_kind_set(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeIncludes
                            | StandardBuiltinId::TypedArrayPrototypeIncludes => {
                                ("includes", ValueInfo::new(ValueKind::Boolean))
                            }
                            StandardBuiltinId::ArrayPrototypeIndexOf
                            | StandardBuiltinId::TypedArrayPrototypeIndexOf => {
                                ("indexOf", ValueInfo::new(ValueKind::Number))
                            }
                            StandardBuiltinId::ArrayPrototypeLastIndexOf
                            | StandardBuiltinId::TypedArrayPrototypeLastIndexOf => {
                                ("lastIndexOf", ValueInfo::new(ValueKind::Number))
                            }
                            StandardBuiltinId::ArrayPrototypeFind => (
                                "find",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::TypedArrayPrototypeFind => (
                                "find",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeFindIndex
                            | StandardBuiltinId::TypedArrayPrototypeFindIndex => {
                                ("findIndex", ValueInfo::new(ValueKind::Number))
                            }
                            StandardBuiltinId::ArrayPrototypeFindLast
                            | StandardBuiltinId::TypedArrayPrototypeFindLast => (
                                "findLast",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeFindLastIndex
                            | StandardBuiltinId::TypedArrayPrototypeFindLastIndex => {
                                ("findLastIndex", ValueInfo::new(ValueKind::Number))
                            }
                            StandardBuiltinId::ArrayPrototypeEvery
                            | StandardBuiltinId::TypedArrayPrototypeEvery => {
                                ("every", ValueInfo::new(ValueKind::Boolean))
                            }
                            StandardBuiltinId::ArrayPrototypeSome
                            | StandardBuiltinId::TypedArrayPrototypeSome => {
                                ("some", ValueInfo::new(ValueKind::Boolean))
                            }
                            StandardBuiltinId::ArrayPrototypeForEach
                            | StandardBuiltinId::TypedArrayPrototypeForEach => {
                                ("forEach", ValueInfo::undefined())
                            }
                            StandardBuiltinId::ArrayPrototypeFilter => (
                                "filter",
                                ValueInfo {
                                    kind: ValueKind::Array,
                                    possible_kinds: KindSet::from_kind(ValueKind::Array),
                                    heap_shape: Some(Box::new(HeapShape::Array(
                                        ArrayShape::default(),
                                    ))),
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeMap => {
                                ("map", self.array_map_result_info(&receiver, args.first()))
                            }
                            StandardBuiltinId::TypedArrayPrototypeMap => (
                                "map",
                                ValueInfo {
                                    kind: ValueKind::Object,
                                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::TypedArrayPrototypeFilter => (
                                "filter",
                                ValueInfo {
                                    kind: ValueKind::Object,
                                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeReduce
                            | StandardBuiltinId::TypedArrayPrototypeReduce => (
                                "reduce",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeReduceRight
                            | StandardBuiltinId::TypedArrayPrototypeReduceRight => (
                                "reduceRight",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypePop => (
                                "pop",
                                ValueInfo {
                                    kind: ValueKind::Dynamic,
                                    possible_kinds: KindSet::all_runtime_tags(),
                                    heap_shape: None,
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeKeys => (
                                "keys",
                                ValueInfo {
                                    kind: ValueKind::Object,
                                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeEntries => (
                                "entries",
                                ValueInfo {
                                    kind: ValueKind::Object,
                                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            StandardBuiltinId::ArrayPrototypeValues => (
                                "values",
                                ValueInfo {
                                    kind: ValueKind::Object,
                                    possible_kinds: KindSet::from_kind(ValueKind::Object),
                                    heap_shape: Some(Box::new(Self::empty_object_shape())),
                                    function_targets: BTreeSet::new(),
                                },
                            ),
                            _ => unreachable!(),
                        };
                        return TypedExpr::from_info(
                            info,
                            ExprIr::CallMethod {
                                receiver: Box::new(receiver),
                                key: PropertyKeyIr::StaticString(key.to_string()),
                                args,
                            },
                        );
                    }
                    if let Some(method_name) =
                        match StandardBuiltinId::from_function_id(&function_id) {
                            Some(StandardBuiltinId::StringPrototypeSubstring) => Some("substring"),
                            Some(StandardBuiltinId::StringPrototypeSlice) => Some("slice"),
                            _ => None,
                        }
                    {
                        let args = args.iter().map(|arg| self.lower_expression(arg)).collect();
                        return TypedExpr::from_info(
                            ValueInfo::new(ValueKind::String),
                            ExprIr::CallMethod {
                                receiver: Box::new(receiver),
                                key: PropertyKeyIr::StaticString(method_name.to_string()),
                                args,
                            },
                        );
                    }
                    let (args, mut info) = self.lower_call_args(&function_id, args);
                    if let Some(builtin) = StandardBuiltinId::from_function_id(&function_id) {
                        if let Some(folded) =
                            Self::fold_standard_builtin_literal_call(builtin, &args)
                        {
                            return folded;
                        }
                    }
                    if let Some(ExprIr::JsonParseStaticReviver { value, reviver }) =
                        self.try_lower_static_json_parse_reviver(&function_id, &args)
                    {
                        return TypedExpr::from_info(
                            ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            },
                            ExprIr::JsonParseStaticReviver { value, reviver },
                        );
                    }
                    if matches!(
                        StandardBuiltinId::from_function_id(&function_id),
                        Some(StandardBuiltinId::TypedArrayFrom | StandardBuiltinId::TypedArrayOf)
                    ) {
                        let key_name = if StandardBuiltinId::from_function_id(&function_id)
                            == Some(StandardBuiltinId::TypedArrayOf)
                        {
                            "of"
                        } else {
                            "from"
                        };
                        let constructor_targets = receiver
                            .function_targets
                            .iter()
                            .filter_map(|function_id| {
                                StandardBuiltinId::from_function_id(function_id)
                                    .filter(|builtin| Self::is_typed_array_constructor(*builtin))
                            })
                            .collect::<Vec<_>>();
                        if let [constructor_builtin] = constructor_targets.as_slice() {
                            info = Self::value_info_from_shape(Some(
                                Self::typed_array_instance_shape_for_constructor(
                                    *constructor_builtin,
                                ),
                            ));
                        } else if !constructor_targets.is_empty() {
                            info = Self::value_info_from_shape(Some(
                                Self::typed_array_instance_shape(),
                            ));
                        }
                        return TypedExpr::from_info(
                            info,
                            ExprIr::CallMethod {
                                receiver: Box::new(receiver),
                                key: PropertyKeyIr::StaticString(key_name.to_string()),
                                args,
                            },
                        );
                    }
                    if StandardBuiltinId::from_function_id(&function_id)
                        == Some(StandardBuiltinId::StringPrototypeSubstr)
                    {
                        return TypedExpr::from_info(
                            info,
                            ExprIr::CallMethod {
                                receiver: Box::new(receiver),
                                key: PropertyKeyIr::StaticString("substr".to_string()),
                                args,
                            },
                        );
                    }
                    if let Some(method) = NonGenericBuiltinMethod::from_function_id(&function_id) {
                        match method {
                            NonGenericBuiltinMethod::BooleanToString
                            | NonGenericBuiltinMethod::BooleanValueOf
                            | NonGenericBuiltinMethod::NumberToExponential
                            | NonGenericBuiltinMethod::NumberToFixed
                            | NonGenericBuiltinMethod::NumberToLocaleString
                            | NonGenericBuiltinMethod::NumberToPrecision
                            | NonGenericBuiltinMethod::NumberToString
                            | NonGenericBuiltinMethod::NumberValueOf
                            | NonGenericBuiltinMethod::BigIntToString
                            | NonGenericBuiltinMethod::BigIntToLocaleString
                            | NonGenericBuiltinMethod::BigIntValueOf
                            | NonGenericBuiltinMethod::StringToString
                            | NonGenericBuiltinMethod::StringValueOf => {
                                // Target inference identifies the function
                                // that was acquired, not the receiver's
                                // primitive brand. Keep both Reference
                                // components so a transferred method reaches
                                // its own closed receiver check.
                                return self.lower_indirect_method_call(
                                    info, callee, receiver, args, None,
                                );
                            }
                        }
                    }
                    if let Some(string_builtin) = StandardBuiltinId::from_function_id(&function_id)
                    {
                        let method_name = match string_builtin {
                            StandardBuiltinId::StringPrototypeCharAt => "charAt",
                            StandardBuiltinId::StringPrototypeConcat => "concat",
                            StandardBuiltinId::StringPrototypeCharCodeAt => "charCodeAt",
                            StandardBuiltinId::StringPrototypeCodePointAt => "codePointAt",
                            StandardBuiltinId::StringPrototypeAt => "at",
                            StandardBuiltinId::StringPrototypePadStart => "padStart",
                            StandardBuiltinId::StringPrototypePadEnd => "padEnd",
                            StandardBuiltinId::StringPrototypeRepeat => "repeat",
                            StandardBuiltinId::StringPrototypeNormalize => "normalize",
                            StandardBuiltinId::StringPrototypeLocaleCompare => "localeCompare",
                            StandardBuiltinId::StringPrototypeToLocaleLowerCase => {
                                "toLocaleLowerCase"
                            }
                            StandardBuiltinId::StringPrototypeToLocaleUpperCase => {
                                "toLocaleUpperCase"
                            }
                            StandardBuiltinId::StringPrototypeToLowerCase => "toLowerCase",
                            StandardBuiltinId::StringPrototypeToUpperCase => "toUpperCase",
                            StandardBuiltinId::StringPrototypeIsWellFormed => "isWellFormed",
                            StandardBuiltinId::StringPrototypeToWellFormed => "toWellFormed",
                            _ => "",
                        };
                        if !method_name.is_empty() {
                            return TypedExpr::from_info(
                                info,
                                ExprIr::CallMethod {
                                    receiver: Box::new(receiver),
                                    key: PropertyKeyIr::StaticString(method_name.to_string()),
                                    args,
                                },
                            );
                        }
                    }
                    if StandardBuiltinId::from_function_id(&function_id)
                        == Some(StandardBuiltinId::ArrayIteratorNext)
                    {
                        return TypedExpr::from_info(
                            info,
                            ExprIr::CallMethod {
                                receiver: Box::new(receiver),
                                key: PropertyKeyIr::StaticString("next".to_string()),
                                args,
                            },
                        );
                    }
                    if let Some(method_name) = StandardBuiltinId::from_function_id(&function_id)
                        .and_then(StandardBuiltinId::string_html_method_name)
                    {
                        return TypedExpr::from_info(
                            info,
                            ExprIr::CallMethod {
                                receiver: Box::new(receiver),
                                key: PropertyKeyIr::StaticString(method_name.to_string()),
                                args,
                            },
                        );
                    }
                    if matches!(
                        StandardBuiltinId::from_function_id(&function_id),
                        Some(
                            StandardBuiltinId::StringPrototypeTrimStart
                                | StandardBuiltinId::StringPrototypeTrim
                                | StandardBuiltinId::StringPrototypeTrimEnd
                        )
                    ) {
                        let key = match StandardBuiltinId::from_function_id(&function_id) {
                            Some(StandardBuiltinId::StringPrototypeTrim) => "trim",
                            Some(StandardBuiltinId::StringPrototypeTrimStart) => "trimStart",
                            Some(StandardBuiltinId::StringPrototypeTrimEnd) => "trimEnd",
                            _ => unreachable!(),
                        };
                        return TypedExpr::from_info(
                            info,
                            ExprIr::CallMethod {
                                receiver: Box::new(receiver),
                                key: PropertyKeyIr::StaticString(key.to_string()),
                                args,
                            },
                        );
                    }
                    if matches!(
                        StandardBuiltinId::from_function_id(&function_id),
                        Some(
                            StandardBuiltinId::FunctionPrototypeCall
                                | StandardBuiltinId::FunctionPrototypeApply
                                | StandardBuiltinId::FunctionPrototypeBind
                        )
                    ) {
                        if let Some(target_function_id) =
                            self.resolve_single_function_target(&receiver)
                        {
                            if let Some(signature) =
                                self.function_signatures.get(&target_function_id).cloned()
                            {
                                match StandardBuiltinId::from_function_id(&function_id) {
                                    Some(StandardBuiltinId::FunctionPrototypeCall)
                                    | Some(StandardBuiltinId::FunctionPrototypeApply) => {
                                        if signature.protocol.flavor() != FunctionFlavor::Arrow {
                                            let this_info = match args.first() {
                                                Some(this_arg) => self
                                                    .explicit_this_info_for_function_target(
                                                        &target_function_id,
                                                        this_arg,
                                                        signature.this_info.clone(),
                                                    ),
                                                None => self.default_this_info_for_function_target(
                                                    &target_function_id,
                                                ),
                                            };
                                            self.merge_function_this_info(
                                                &target_function_id,
                                                this_info,
                                            );
                                            let forwarded_args = if matches!(
                                                StandardBuiltinId::from_function_id(&function_id),
                                                Some(StandardBuiltinId::FunctionPrototypeCall)
                                            ) {
                                                Some(
                                                    args.iter()
                                                        .skip(1)
                                                        .map(TypedExpr::value_info)
                                                        .collect::<Vec<_>>(),
                                                )
                                            } else {
                                                self.forwarded_apply_arg_infos(args.get(1))
                                            };
                                            if let Some(forwarded_args) = forwarded_args {
                                                self.merge_function_param_infos(
                                                    &target_function_id,
                                                    &forwarded_args,
                                                );
                                            }
                                            if matches!(
                                                StandardBuiltinId::from_function_id(
                                                    &target_function_id
                                                ),
                                                Some(StandardBuiltinId::ArrayPrototypeSort)
                                            ) && matches!(
                                                StandardBuiltinId::from_function_id(&function_id),
                                                Some(StandardBuiltinId::FunctionPrototypeCall)
                                            ) {
                                                if let Some(callback_id) =
                                                    args.get(1).and_then(|callback| {
                                                        self.resolve_single_function_target(
                                                            callback,
                                                        )
                                                    })
                                                {
                                                    self.merge_function_param_infos(
                                                        &callback_id,
                                                        &[
                                                            ValueInfo {
                                                                kind: ValueKind::Dynamic,
                                                                possible_kinds:
                                                                    KindSet::all_runtime_tags(),
                                                                heap_shape: None,
                                                                function_targets: BTreeSet::new(),
                                                            },
                                                            ValueInfo {
                                                                kind: ValueKind::Dynamic,
                                                                possible_kinds:
                                                                    KindSet::all_runtime_tags(),
                                                                heap_shape: None,
                                                                function_targets: BTreeSet::new(),
                                                            },
                                                        ],
                                                    );
                                                    self.merge_function_this_info(
                                                        &callback_id,
                                                        ValueInfo::undefined(),
                                                    );
                                                }
                                            }
                                        }
                                        if signature.protocol.class_kind()
                                            != ClassFunctionKind::Constructor
                                        {
                                            info = ValueInfo {
                                                kind: signature.return_kind,
                                                possible_kinds: signature.return_possible_kinds,
                                                heap_shape: signature.return_shape,
                                                function_targets: signature.return_targets,
                                            };
                                        }
                                    }
                                    Some(StandardBuiltinId::FunctionPrototypeBind) => {
                                        if signature.protocol.flavor() != FunctionFlavor::Arrow
                                            && !signature.protocol.is_constructable()
                                        {
                                            if let Some(this_arg) = args.first() {
                                                self.merge_function_this_info(
                                                    &target_function_id,
                                                    this_arg.value_info(),
                                                );
                                            }
                                        }
                                        let bound_arg_infos = args
                                            .iter()
                                            .skip(1)
                                            .map(TypedExpr::value_info)
                                            .collect::<Vec<_>>();
                                        if !bound_arg_infos.is_empty() {
                                            self.merge_function_param_infos(
                                                &target_function_id,
                                                &bound_arg_infos,
                                            );
                                        }
                                        info = ValueInfo {
                                            kind: ValueKind::Function,
                                            possible_kinds: KindSet::from_kind(ValueKind::Function),
                                            heap_shape: Some(Self::function_heap_shape(
                                                signature.protocol.is_constructable(),
                                            )),
                                            function_targets: BTreeSet::from([
                                                StandardBuiltinId::BoundFunctionInvoker
                                                    .function_id(),
                                            ]),
                                        };
                                        self.bound_functions += 1;
                                        self.bound_function_constructs +=
                                            usize::from(signature.protocol.is_constructable());
                                    }
                                    _ => {}
                                }
                            }
                        }
                    }
                    if string_from_code_point_apply_call
                        && matches!(
                            StandardBuiltinId::from_function_id(&function_id),
                            Some(StandardBuiltinId::FunctionPrototypeApply)
                        )
                    {
                        info = ValueInfo::new(ValueKind::String);
                    }
                    if matches!(
                        StandardBuiltinId::from_function_id(&function_id),
                        Some(StandardBuiltinId::FunctionPrototypeCall)
                    ) {
                        if let Some(target_function_id) =
                            self.resolve_single_function_target(&receiver)
                        {
                            if matches!(
                                StandardBuiltinId::from_function_id(&target_function_id),
                                Some(StandardBuiltinId::ArrayBufferSpeciesGetter)
                            ) {
                                let call_this =
                                    args.first().cloned().unwrap_or_else(TypedExpr::undefined);
                                info = if call_this.possible_kinds.is_subset_of(
                                    KindSet::from_kind(ValueKind::Undefined)
                                        .union(KindSet::from_kind(ValueKind::Null)),
                                ) {
                                    self.global_this_info()
                                } else {
                                    self.boxed_receiver_info_from_arg(&call_this)
                                        .unwrap_or_else(|| call_this.value_info())
                                };
                                let forwarded_args =
                                    args.iter().skip(1).cloned().collect::<Vec<_>>();
                                return TypedExpr::from_info(
                                    info,
                                    ExprIr::CallIndirect {
                                        callee: Box::new(receiver),
                                        this_arg: Some(Box::new(call_this)),
                                        args: forwarded_args,
                                        static_regexp_compilation: None,
                                    },
                                );
                            }
                        }
                    }
                    let static_regexp_compilation = self.static_regexp_compilation_for_direct_call(
                        &callee,
                        &function_id,
                        &args,
                    );
                    return self.lower_indirect_method_call(
                        info,
                        callee,
                        receiver,
                        args,
                        static_regexp_compilation,
                    );
                }
                PropertyAccess::Private(access) => {
                    let Some(private_name_id) = self.current_private_name_id(access.field()) else {
                        return self.unsupported_expr("private class element");
                    };
                    let receiver = self.lower_property_target(access.target());
                    let receiver_info = receiver.value_info();
                    let receiver_storage_name =
                        self.alloc_temp_binding_name("private.call.receiver.");
                    let materialized_receiver = TypedExpr::from_info(
                        receiver_info.clone(),
                        ExprIr::Identifier(receiver_storage_name.clone()),
                    );
                    let callee = TypedExpr::from_info(
                        self.read_object_shape(&receiver, &private_data_key(private_name_id))
                            .unwrap_or(ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            }),
                        ExprIr::PrivateRead {
                            target: Box::new(materialized_receiver.clone()),
                            private_name_id,
                        },
                    );
                    let function_id = (callee.kind == ValueKind::Function)
                        .then(|| self.resolve_single_function_target(&callee))
                        .flatten();
                    let (args, info) = if let Some(function_id) = function_id {
                        let Some(signature) = self.function_signatures.get(&function_id) else {
                            return self.unsupported_expr("indirect call");
                        };
                        if !signature.callable
                            && signature.protocol.class_kind() != ClassFunctionKind::Constructor
                        {
                            return unsupported_call(self, signature.protocol.class_kind());
                        }
                        self.merge_function_this_info(&function_id, receiver_info);
                        self.lower_call_args(&function_id, args)
                    } else {
                        let Some(args) = self.lower_call_args_expanding_spread(args) else {
                            return TypedExpr::undefined();
                        };
                        (
                            args,
                            ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            },
                        )
                    };
                    let call = TypedExpr::from_info(
                        info.clone(),
                        ExprIr::CallIndirect {
                            callee: Box::new(callee),
                            this_arg: Some(Box::new(materialized_receiver)),
                            args,
                            static_regexp_compilation: None,
                        },
                    );
                    return TypedExpr::from_info(
                        info,
                        ExprIr::MaterializeBinding {
                            name: receiver_storage_name,
                            value: Box::new(receiver),
                            body: Box::new(call),
                        },
                    );
                }
                PropertyAccess::Super(access) => {
                    let callee = self.lower_super_property_access(access);
                    let Some(function_id) = (callee.kind == ValueKind::Function)
                        .then(|| self.resolve_single_function_target(&callee))
                        .flatten()
                    else {
                        let Some(args) = self.lower_call_args_expanding_spread(args) else {
                            return TypedExpr::undefined();
                        };
                        return TypedExpr::from_info(
                            ValueInfo {
                                kind: ValueKind::Dynamic,
                                possible_kinds: KindSet::all_runtime_tags(),
                                heap_shape: None,
                                function_targets: BTreeSet::new(),
                            },
                            ExprIr::CallIndirect {
                                callee: Box::new(callee),
                                this_arg: Some(Box::new(TypedExpr::from_info(
                                    self.current_this_info(),
                                    ExprIr::This,
                                ))),
                                args,
                                static_regexp_compilation: None,
                            },
                        );
                    };
                    let Some(signature) = self.function_signatures.get(&function_id) else {
                        return self.unsupported_expr("indirect call");
                    };
                    if !signature.callable
                        && signature.protocol.class_kind() != ClassFunctionKind::Constructor
                    {
                        return unsupported_call(self, signature.protocol.class_kind());
                    }
                    let (args, info) = self.lower_call_args(&function_id, args);
                    return TypedExpr::from_info(
                        info,
                        ExprIr::CallIndirect {
                            callee: Box::new(callee),
                            this_arg: Some(Box::new(TypedExpr::from_info(
                                self.current_this_info(),
                                ExprIr::This,
                            ))),
                            args,
                            static_regexp_compilation: None,
                        },
                    );
                }
            }
        }
        let callee = self.lower_expression(callee);
        let callee = match callee {
            TypedExpr {
                expr: ExprIr::OptionalPropertyChain { target, mut chain },
                ..
            } => {
                let source_args = args;
                let Some(args) = self.lower_call_args_expanding_spread(args) else {
                    return TypedExpr::undefined();
                };
                let mut call_sources = already_accounted_optional_calls(&chain);
                chain.push(OptionalChainOperationIr::Call {
                    args,
                    receiver: OptionalChainCallReceiverIr::ReferenceOrUndefined,
                    shorted: false,
                    boundary_before: true,
                });
                call_sources.push(OptionalCallSource::Syntax(source_args));
                let info =
                    self.analyze_optional_property_chain(target.as_ref(), &chain, &call_sources);
                return TypedExpr::from_info(info, ExprIr::OptionalPropertyChain { target, chain });
            }
            callee => callee,
        };
        let lower_generic_indirect_call = |this: &mut Self, callee: TypedExpr| {
            let Some(args) = this.lower_call_args_expanding_spread(args) else {
                return TypedExpr::undefined();
            };
            let result = TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::CallIndirect {
                    callee: Box::new(callee),
                    this_arg: None,
                    args,
                    static_regexp_compilation: None,
                },
            );
            this.invalidate_unknown_user_code_effects();
            result
        };
        if callee.kind != ValueKind::Function {
            return lower_generic_indirect_call(self, callee);
        }
        let mut callee = callee;
        if matches!(&callee.expr, ExprIr::FunctionValue(_))
            && !self.exact_context_callback_targets.is_empty()
            && !callee.function_targets.is_empty()
        {
            let mut rewritten_targets = BTreeSet::new();
            for target_id in &callee.function_targets {
                let original_target_id = self.original_exact_function_id(target_id);
                if let Some(helper_context_id) = self
                    .exact_context_callback_targets
                    .get(&original_target_id)
                    .cloned()
                {
                    if let Some(synthetic_id) = self
                        .exact_context_callback_specializations
                        .get(&(original_target_id, helper_context_id))
                        .cloned()
                    {
                        rewritten_targets.insert(synthetic_id);
                        continue;
                    }
                }
                rewritten_targets.insert(target_id.clone());
            }
            if rewritten_targets != callee.function_targets {
                callee.function_targets = rewritten_targets;
                if let Some(single_target) = self.resolve_single_function_target(&callee) {
                    callee = self.function_value_expr(single_target);
                }
            }
        }
        let Some(mut function_id) = self.resolve_single_function_target(&callee) else {
            if !callee.function_targets.is_empty() {
                let source_args = args;
                let Some(args) = self.lower_call_args_expanding_spread(args) else {
                    return TypedExpr::undefined();
                };
                let args_have_spread = Self::call_args_have_spread(&args);
                let mut result_info: Option<ValueInfo> = None;
                let function_targets = callee.function_targets.iter().cloned().collect::<Vec<_>>();
                let source_function_may_run = function_targets
                    .iter()
                    .any(|function_id| self.analysis.function_plans.contains_key(function_id));
                let mut rejected_dynamic_source = false;
                for function_id in &function_targets {
                    let context = resolved_builtin_call_context(&callee, function_id);
                    let eval_pass_through = match self.resolve_dynamic_source_call(
                        function_id,
                        context,
                        Some(source_args),
                        &args,
                    ) {
                        None => None,
                        Some(ResolvedDynamicSourceCall::EvalPassThrough(proof)) => {
                            if let Some(builtin) = StandardBuiltinId::from_function_id(function_id)
                            {
                                self.note_standard_builtin_call(builtin);
                            }
                            Some(proof.into_result_info())
                        }
                        Some(ResolvedDynamicSourceCall::Unsupported(gap)) => {
                            self.record_unsupported_dynamic_source(function_id, gap);
                            rejected_dynamic_source = true;
                            None
                        }
                    };
                    let Some(signature) = self.function_signatures.get(function_id).cloned() else {
                        continue;
                    };
                    if !signature.callable
                        && signature.protocol.class_kind() != ClassFunctionKind::Constructor
                    {
                        continue;
                    }
                    self.mark_host_builtin_from_function_id(function_id);
                    self.host_builtin_calls +=
                        usize::from(HostBuiltinId::from_function_id(function_id).is_some());
                    let arg_infos = args.iter().map(TypedExpr::value_info).collect::<Vec<_>>();
                    let exact_prepass_call =
                        self.is_prepass && self.analysis.function_plans.contains_key(function_id);
                    if !args_have_spread && !exact_prepass_call {
                        let this_info = self.default_this_info_for_function_target(function_id);
                        self.merge_function_this_info(function_id, this_info);
                        self.merge_function_param_infos(function_id, &arg_infos);
                    } else if !args_have_spread {
                        if let Some(helper_context_id) = self
                            .exact_context_callback_targets
                            .get(&self.original_exact_function_id(function_id))
                            .cloned()
                        {
                            let original_function_id = self.original_exact_function_id(function_id);
                            let arg_infos = self.canonical_exact_context_arg_infos(&arg_infos);
                            let this_info = self.default_this_info_for_function_target(function_id);
                            self.observe_exact_callback_this_info(
                                &original_function_id,
                                &helper_context_id,
                                this_info,
                            );
                            self.observe_exact_callback_param_infos(
                                &original_function_id,
                                &helper_context_id,
                                &arg_infos,
                            );
                        }
                    }
                    if self.is_prepass && !args_have_spread {
                        self.propagate_direct_call_context(function_id, &arg_infos);
                    }
                    let next_info = eval_pass_through.unwrap_or(ValueInfo {
                        kind: signature.return_kind,
                        possible_kinds: signature.return_possible_kinds,
                        heap_shape: signature.return_shape,
                        function_targets: signature.return_targets,
                    });
                    result_info = Some(match result_info {
                        Some(existing) => self.merge_value_infos(existing, next_info),
                        None => next_info,
                    });
                }
                if rejected_dynamic_source {
                    return TypedExpr::undefined();
                }
                let result = TypedExpr::from_info(
                    result_info.unwrap_or_else(|| ValueInfo {
                        kind: ValueKind::Dynamic,
                        possible_kinds: KindSet::all_runtime_tags(),
                        heap_shape: None,
                        function_targets: BTreeSet::new(),
                    }),
                    ExprIr::CallIndirect {
                        callee: Box::new(callee),
                        this_arg: None,
                        args,
                        static_regexp_compilation: None,
                    },
                );
                if source_function_may_run {
                    self.invalidate_unknown_user_code_effects();
                }
                return result;
            }
            return lower_generic_indirect_call(self, callee);
        };
        if let Some(helper_context_id) = self
            .exact_context_callback_targets
            .get(&self.original_exact_function_id(&function_id))
            .cloned()
        {
            let original_function_id = self.original_exact_function_id(&function_id);
            if let Some(synthetic_id) = self
                .exact_context_callback_specializations
                .get(&(original_function_id, helper_context_id))
                .cloned()
            {
                function_id = synthetic_id.clone();
                if matches!(&callee.expr, ExprIr::FunctionValue(_)) {
                    callee = self.function_value_expr(synthetic_id);
                }
            }
        }
        let Some(signature) = self.function_signatures.get(&function_id) else {
            return lower_generic_indirect_call(self, callee);
        };
        if !signature.callable && signature.protocol.class_kind() != ClassFunctionKind::Constructor
        {
            return unsupported_call(self, signature.protocol.class_kind());
        }
        self.mark_host_builtin_from_function_id(&function_id);
        self.host_builtin_calls +=
            usize::from(HostBuiltinId::from_function_id(&function_id).is_some());
        let this_info = self.default_this_info_for_function_target(&function_id);
        self.merge_function_this_info(&function_id, this_info);
        let context = resolved_builtin_call_context(&callee, &function_id);
        let (effective_function_id, args, info) =
            self.lower_call_args_with_target(&function_id, args, context);
        if let Some(builtin) = StandardBuiltinId::from_function_id(&effective_function_id) {
            if let Some(folded) = Self::fold_standard_builtin_literal_call(builtin, &args) {
                return folded;
            }
        }
        // A context-specialized body is only safe to materialize directly when
        // the source expression already creates that function object here.
        // Replacing an identifier/property callee would discard the original
        // closure object's captured environment and function identity.
        let callee = if effective_function_id != function_id
            && matches!(&callee.expr, ExprIr::FunctionValue(_))
        {
            self.function_value_expr(effective_function_id.clone())
        } else {
            callee
        };
        if let Some(ExprIr::JsonParseStaticReviver { value, reviver }) =
            self.try_lower_static_json_parse_reviver(&effective_function_id, &args)
        {
            return TypedExpr::from_info(
                ValueInfo {
                    kind: ValueKind::Dynamic,
                    possible_kinds: KindSet::all_runtime_tags(),
                    heap_shape: None,
                    function_targets: BTreeSet::new(),
                },
                ExprIr::JsonParseStaticReviver { value, reviver },
            );
        }
        let static_regexp_compilation =
            self.static_regexp_compilation_for_direct_call(&callee, &effective_function_id, &args);
        TypedExpr::from_info(
            info,
            ExprIr::CallIndirect {
                callee: Box::new(callee),
                this_arg: None,
                args,
                static_regexp_compilation,
            },
        )
    }
}
