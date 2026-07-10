use super::*;

#[derive(Debug, Clone)]
pub(crate) struct WasmFunctionMeta {
    pub(crate) name: String,
    pub(crate) to_string_value: String,
    /// `Some` when this meta belongs to a standard builtin. Used to record
    /// which builtins get their function values materialized (or their bodies
    /// direct-called) during emission — see [`FunctionMetaRegistry`] — and for
    /// precise meta-to-builtin reverse lookups.
    pub(crate) standard_builtin: Option<StandardBuiltinId>,
    /// `Some` when this meta belongs to a host builtin. Same role as
    /// `standard_builtin`: host builtin bodies are stubbed unless the script
    /// references them, but they can also be reached dynamically (installed on
    /// a realm global by `__porfCreateRealm`, or direct-called from another
    /// builtin's body like `JSON.parse` -> `parseFloat`), so materializations
    /// and direct calls are recorded and force their real bodies.
    pub(crate) host_builtin: Option<HostBuiltinId>,
    pub(crate) length: u64,
    pub(crate) length_name_configurable: bool,
    pub(crate) wasm_index: u32,
    pub(crate) table_index: u32,
    pub(crate) constructable: bool,
    pub(crate) strict: bool,
    pub(crate) class_kind: ClassFunctionKind,
    pub(crate) class_heritage_kind: ClassHeritageKind,
    pub(crate) is_static_class_member: bool,
    pub(crate) is_derived_constructor: bool,
    pub(crate) is_synthetic_default_derived_constructor: bool,
    pub(crate) super_constructor_target: Option<FunctionId>,
    pub(crate) uses_super: bool,
    pub(crate) this_before_super: bool,
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeBootstrapPlan {
    pub(crate) full_standard_globals: bool,
    pub(crate) standard_roots: BTreeSet<StandardBuiltinId>,
    pub(crate) reflect_object: bool,
    pub(crate) math_object: bool,
    pub(crate) json_object: bool,
    pub(crate) atomics_object: bool,
}

impl RuntimeBootstrapPlan {
    pub(crate) fn from_script(
        script: &ScriptIr,
        compiled_standard_builtins: &[StandardBuiltinId],
    ) -> Self {
        let mut plan = Self::default();
        plan.full_standard_globals =
            script_uses_create_realm(script) || script_exposes_global_object(script);
        for builtin in compiled_standard_builtins {
            plan.require_standard_builtin(*builtin);
        }
        for name in script_referenced_global_property_names(script) {
            if let Some(binding) = script
                .global_bindings
                .iter()
                .find(|binding| binding.name == name)
            {
                plan.require_script_global_binding(binding.kind);
            }
        }
        plan.require_foundational_roots();
        plan
    }

    pub(crate) fn should_initialize_standard_builtin(&self, builtin: StandardBuiltinId) -> bool {
        self.full_standard_globals || self.standard_roots.contains(&builtin)
    }

    pub(crate) fn should_install_script_global_binding(
        &self,
        kind: ScriptGlobalBindingKind,
    ) -> bool {
        match kind {
            ScriptGlobalBindingKind::Intrinsic
            | ScriptGlobalBindingKind::Infinity
            | ScriptGlobalBindingKind::NaN
            | ScriptGlobalBindingKind::Undefined
            | ScriptGlobalBindingKind::Var
            | ScriptGlobalBindingKind::Function
            | ScriptGlobalBindingKind::HostFunction(_) => true,
            ScriptGlobalBindingKind::ReflectObject => {
                self.full_standard_globals || self.reflect_object
            }
            ScriptGlobalBindingKind::MathObject => self.full_standard_globals || self.math_object,
            ScriptGlobalBindingKind::JsonObject => self.full_standard_globals || self.json_object,
            ScriptGlobalBindingKind::AtomicsObject => {
                self.full_standard_globals || self.atomics_object
            }
            ScriptGlobalBindingKind::BuiltinFunction(builtin) => {
                self.should_initialize_standard_builtin(builtin)
            }
        }
    }

    pub(crate) fn needs_typed_array_intrinsic(&self) -> bool {
        self.full_standard_globals
            || self
                .standard_roots
                .iter()
                .any(|builtin| is_typed_array_constructor(*builtin))
    }

    fn require_script_global_binding(&mut self, kind: ScriptGlobalBindingKind) {
        match kind {
            ScriptGlobalBindingKind::ReflectObject => self.reflect_object = true,
            ScriptGlobalBindingKind::MathObject => self.math_object = true,
            ScriptGlobalBindingKind::JsonObject => self.json_object = true,
            ScriptGlobalBindingKind::AtomicsObject => self.atomics_object = true,
            ScriptGlobalBindingKind::BuiltinFunction(builtin) => {
                self.require_standard_builtin(builtin);
            }
            ScriptGlobalBindingKind::Intrinsic
            | ScriptGlobalBindingKind::Infinity
            | ScriptGlobalBindingKind::NaN
            | ScriptGlobalBindingKind::Undefined
            | ScriptGlobalBindingKind::Var
            | ScriptGlobalBindingKind::Function
            | ScriptGlobalBindingKind::HostFunction(_) => {}
        }
    }

    fn require_foundational_roots(&mut self) {
        for builtin in [
            StandardBuiltinId::FunctionConstructor,
            StandardBuiltinId::ObjectConstructor,
            StandardBuiltinId::ErrorConstructor,
            StandardBuiltinId::EvalErrorConstructor,
            StandardBuiltinId::AggregateErrorConstructor,
            StandardBuiltinId::SuppressedErrorConstructor,
            StandardBuiltinId::RangeErrorConstructor,
            StandardBuiltinId::SyntaxErrorConstructor,
            StandardBuiltinId::TypeErrorConstructor,
            StandardBuiltinId::URIErrorConstructor,
            StandardBuiltinId::ReferenceErrorConstructor,
        ] {
            self.standard_roots.insert(builtin);
        }
    }

    fn require_standard_builtin(&mut self, builtin: StandardBuiltinId) {
        self.standard_roots.insert(builtin);
        match builtin {
            StandardBuiltinId::ReflectConstruct
            | StandardBuiltinId::ReflectApply
            | StandardBuiltinId::ReflectGet
            | StandardBuiltinId::ReflectGetPrototypeOf
            | StandardBuiltinId::ReflectGetOwnPropertyDescriptor
            | StandardBuiltinId::ReflectSet
            | StandardBuiltinId::ReflectHas
            | StandardBuiltinId::ReflectDefineProperty
            | StandardBuiltinId::ReflectDeleteProperty
            | StandardBuiltinId::ReflectIsExtensible
            | StandardBuiltinId::ReflectPreventExtensions
            | StandardBuiltinId::ReflectSetPrototypeOf
            | StandardBuiltinId::ReflectOwnKeys => self.reflect_object = true,
            StandardBuiltinId::MathAbs
            | StandardBuiltinId::MathAcos
            | StandardBuiltinId::MathAcosh
            | StandardBuiltinId::MathAsin
            | StandardBuiltinId::MathAsinh
            | StandardBuiltinId::MathAtan
            | StandardBuiltinId::MathAtan2
            | StandardBuiltinId::MathAtanh
            | StandardBuiltinId::MathCbrt
            | StandardBuiltinId::MathCeil
            | StandardBuiltinId::MathClz32
            | StandardBuiltinId::MathCos
            | StandardBuiltinId::MathCosh
            | StandardBuiltinId::MathExp
            | StandardBuiltinId::MathExpm1
            | StandardBuiltinId::MathF16Round
            | StandardBuiltinId::MathFloor
            | StandardBuiltinId::MathFround
            | StandardBuiltinId::MathHypot
            | StandardBuiltinId::MathImul
            | StandardBuiltinId::MathLog
            | StandardBuiltinId::MathLog10
            | StandardBuiltinId::MathLog1p
            | StandardBuiltinId::MathLog2
            | StandardBuiltinId::MathMax
            | StandardBuiltinId::MathMin
            | StandardBuiltinId::MathPow
            | StandardBuiltinId::MathRandom
            | StandardBuiltinId::MathRound
            | StandardBuiltinId::MathSign
            | StandardBuiltinId::MathSin
            | StandardBuiltinId::MathSinh
            | StandardBuiltinId::MathSqrt
            | StandardBuiltinId::MathSumPrecise
            | StandardBuiltinId::MathTan
            | StandardBuiltinId::MathTanh
            | StandardBuiltinId::MathTrunc => self.math_object = true,
            StandardBuiltinId::JsonParse
            | StandardBuiltinId::JsonStringify
            | StandardBuiltinId::JsonRawJson
            | StandardBuiltinId::JsonIsRawJson => self.json_object = true,
            StandardBuiltinId::AtomicsAdd
            | StandardBuiltinId::AtomicsAnd
            | StandardBuiltinId::AtomicsCompareExchange
            | StandardBuiltinId::AtomicsExchange
            | StandardBuiltinId::AtomicsLoad
            | StandardBuiltinId::AtomicsNotify
            | StandardBuiltinId::AtomicsOr
            | StandardBuiltinId::AtomicsPause
            | StandardBuiltinId::AtomicsStore
            | StandardBuiltinId::AtomicsSub
            | StandardBuiltinId::AtomicsWait
            | StandardBuiltinId::AtomicsWaitAsync
            | StandardBuiltinId::AtomicsXor
            | StandardBuiltinId::AtomicsIsLockFree => self.atomics_object = true,
            StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
            | StandardBuiltinId::ArrayBufferPrototypeDetachedGetter
            | StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter
            | StandardBuiltinId::ArrayBufferPrototypeResizableGetter
            | StandardBuiltinId::ArrayBufferPrototypeResize
            | StandardBuiltinId::ArrayBufferPrototypeSlice
            | StandardBuiltinId::ArrayBufferPrototypeTransfer
            | StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength
            | StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable
            | StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable
            | StandardBuiltinId::ArrayBufferSpeciesGetter => {
                self.standard_roots
                    .insert(StandardBuiltinId::ArrayBufferConstructor);
            }
            StandardBuiltinId::ArrayPrototypeReduce
            | StandardBuiltinId::ArrayPrototypeReduceRight => {
                // Array.prototype's method properties are installed by the
                // Array constructor bootstrap block.  The reducer can be
                // reached only through a dynamic method Get, so root that
                // block as well as the body itself.
                self.standard_roots
                    .insert(StandardBuiltinId::ArrayConstructor);
            }
            StandardBuiltinId::NumberPrototypeToFixed
            | StandardBuiltinId::NumberPrototypeToExponential
            | StandardBuiltinId::NumberPrototypeToPrecision
            | StandardBuiltinId::NumberPrototypeToString
            | StandardBuiltinId::NumberPrototypeToLocaleString
            | StandardBuiltinId::NumberPrototypeValueOf => {
                // These bodies can be reached purely via dynamic property
                // dispatch on a Number-typed value (no direct call-site
                // FunctionId reference), so `should_stub_standard_builtin`
                // alone isn't enough to guarantee the property gets installed:
                // `Number.prototype`'s own-properties are only written by the
                // `NumberConstructor` bootstrap block, which is separately
                // gated on this same root set. Without forcing the
                // constructor in here too, the method body compiles but its
                // `Number.prototype` property is silently never defined, so
                // the runtime property read at the call site resolves to
                // `undefined` and traps instead of throwing/working.
                self.standard_roots
                    .insert(StandardBuiltinId::NumberConstructor);
            }
            StandardBuiltinId::BooleanPrototypeToString | StandardBuiltinId::BooleanPrototypeValueOf => {
                self.standard_roots
                    .insert(StandardBuiltinId::BooleanConstructor);
            }
            StandardBuiltinId::SymbolFor
            | StandardBuiltinId::SymbolKeyFor
            | StandardBuiltinId::SymbolPrototypeDescriptionGetter
            | StandardBuiltinId::SymbolPrototypeToString
            | StandardBuiltinId::SymbolPrototypeValueOf
            | StandardBuiltinId::SymbolPrototypeToPrimitive => {
                // `Symbol.for` / `Symbol.keyFor` live on the `Symbol`
                // constructor object and share its runtime registry, which is
                // only allocated by the `SymbolConstructor` bootstrap block.
                // The `Symbol.prototype` methods are likewise only installed
                // as properties by that same bootstrap block, and (like
                // `Number.prototype.valueOf`) their bodies can be reached
                // purely via dynamic property dispatch on a Symbol-typed
                // value with no direct call-site `FunctionId` reference.
                self.standard_roots
                    .insert(StandardBuiltinId::SymbolConstructor);
            }
            StandardBuiltinId::StringPrototypeToString | StandardBuiltinId::StringPrototypeValueOf => {
                self.standard_roots
                    .insert(StandardBuiltinId::StringConstructor);
            }
            StandardBuiltinId::BigIntPrototypeToString
            | StandardBuiltinId::BigIntPrototypeToLocaleString
            | StandardBuiltinId::BigIntPrototypeValueOf => {
                self.standard_roots
                    .insert(StandardBuiltinId::BigIntConstructor);
            }
            StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter
            | StandardBuiltinId::SharedArrayBufferPrototypeGrow
            | StandardBuiltinId::SharedArrayBufferPrototypeSlice => {
                self.standard_roots
                    .insert(StandardBuiltinId::SharedArrayBufferConstructor);
            }
            StandardBuiltinId::DataViewPrototypeBufferGetter
            | StandardBuiltinId::DataViewPrototypeByteLengthGetter
            | StandardBuiltinId::DataViewPrototypeByteOffsetGetter
            | StandardBuiltinId::DataViewPrototypeGetUint8
            | StandardBuiltinId::DataViewPrototypeSetUint8
            | StandardBuiltinId::DataViewPrototypeGetInt8
            | StandardBuiltinId::DataViewPrototypeSetInt8
            | StandardBuiltinId::DataViewPrototypeGetUint16
            | StandardBuiltinId::DataViewPrototypeSetUint16
            | StandardBuiltinId::DataViewPrototypeGetInt16
            | StandardBuiltinId::DataViewPrototypeSetInt16
            | StandardBuiltinId::DataViewPrototypeGetUint32
            | StandardBuiltinId::DataViewPrototypeSetUint32
            | StandardBuiltinId::DataViewPrototypeGetInt32
            | StandardBuiltinId::DataViewPrototypeSetInt32
            | StandardBuiltinId::DataViewPrototypeGetFloat16
            | StandardBuiltinId::DataViewPrototypeSetFloat16
            | StandardBuiltinId::DataViewPrototypeGetFloat32
            | StandardBuiltinId::DataViewPrototypeSetFloat32
            | StandardBuiltinId::DataViewPrototypeGetFloat64
            | StandardBuiltinId::DataViewPrototypeSetFloat64
            | StandardBuiltinId::DataViewPrototypeGetBigInt64
            | StandardBuiltinId::DataViewPrototypeSetBigInt64
            | StandardBuiltinId::DataViewPrototypeGetBigUint64
            | StandardBuiltinId::DataViewPrototypeSetBigUint64 => {
                self.standard_roots
                    .insert(StandardBuiltinId::DataViewConstructor);
            }
            StandardBuiltinId::DateNow
            | StandardBuiltinId::DateUtc
            | StandardBuiltinId::DatePrototypeGetTime
            | StandardBuiltinId::DatePrototypeSetTime
            | StandardBuiltinId::DatePrototypeValueOf
            | StandardBuiltinId::DatePrototypeGetFullYear
            | StandardBuiltinId::DatePrototypeGetUtcFullYear
            | StandardBuiltinId::DatePrototypeGetMonth
            | StandardBuiltinId::DatePrototypeGetUtcMonth
            | StandardBuiltinId::DatePrototypeGetDate
            | StandardBuiltinId::DatePrototypeGetUtcDate
            | StandardBuiltinId::DatePrototypeGetDay
            | StandardBuiltinId::DatePrototypeGetUtcDay
            | StandardBuiltinId::DatePrototypeGetHours
            | StandardBuiltinId::DatePrototypeGetUtcHours
            | StandardBuiltinId::DatePrototypeGetMinutes
            | StandardBuiltinId::DatePrototypeGetUtcMinutes
            | StandardBuiltinId::DatePrototypeGetSeconds
            | StandardBuiltinId::DatePrototypeGetUtcSeconds
            | StandardBuiltinId::DatePrototypeGetMilliseconds
            | StandardBuiltinId::DatePrototypeGetUtcMilliseconds
            | StandardBuiltinId::DatePrototypeGetTimezoneOffset
            | StandardBuiltinId::DatePrototypeGetYear
            | StandardBuiltinId::DatePrototypeSetYear
            | StandardBuiltinId::DatePrototypeSetFullYear
            | StandardBuiltinId::DatePrototypeSetUtcFullYear
            | StandardBuiltinId::DatePrototypeSetMonth
            | StandardBuiltinId::DatePrototypeSetUtcMonth
            | StandardBuiltinId::DatePrototypeSetDate
            | StandardBuiltinId::DatePrototypeSetUtcDate
            | StandardBuiltinId::DatePrototypeSetHours
            | StandardBuiltinId::DatePrototypeSetUtcHours
            | StandardBuiltinId::DatePrototypeSetMinutes
            | StandardBuiltinId::DatePrototypeSetUtcMinutes
            | StandardBuiltinId::DatePrototypeSetSeconds
            | StandardBuiltinId::DatePrototypeSetUtcSeconds
            | StandardBuiltinId::DatePrototypeSetMilliseconds
            | StandardBuiltinId::DatePrototypeSetUtcMilliseconds
            | StandardBuiltinId::DatePrototypeToUtcString => {
                self.standard_roots
                    .insert(StandardBuiltinId::DateConstructor);
            }
            StandardBuiltinId::RegExpEscape
            | StandardBuiltinId::RegExpSpeciesGetter
            | StandardBuiltinId::RegExpLegacyStaticGetter
            | StandardBuiltinId::RegExpLegacyStaticSetter
            | StandardBuiltinId::RegExpPrototypeSymbolMatch
            | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
            | StandardBuiltinId::RegExpPrototypeSymbolSearch => {
                self.standard_roots
                    .insert(StandardBuiltinId::RegExpConstructor);
            }
            StandardBuiltinId::TypedArrayPrototypeBufferGetter
            | StandardBuiltinId::TypedArrayPrototypeByteLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter
            | StandardBuiltinId::TypedArrayPrototypeLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeToString
            | StandardBuiltinId::TypedArrayPrototypeToLocaleString
            | StandardBuiltinId::TypedArrayFrom
            | StandardBuiltinId::TypedArrayOf => {
                self.standard_roots
                    .insert(StandardBuiltinId::Int32ArrayConstructor);
            }
            _ => {}
        }
    }
}

pub(crate) fn script_exposes_global_object(script: &ScriptIr) -> bool {
    block_exposes_global_object(&script.body)
        || script.functions.iter().any(|function| {
            function.params.iter().any(|param| {
                param
                    .default_init
                    .as_ref()
                    .is_some_and(expr_exposes_global_object)
            }) || block_exposes_global_object(&function.body)
        })
}

pub(crate) fn script_referenced_global_property_names(script: &ScriptIr) -> BTreeSet<String> {
    let mut names = BTreeSet::new();
    collect_block_global_property_names(&script.body, &mut names);
    for function in &script.functions {
        for param in &function.params {
            if let Some(init) = &param.default_init {
                collect_expr_global_property_names(init, &mut names);
            }
        }
        collect_block_global_property_names(&function.body, &mut names);
    }
    names
}

fn block_exposes_global_object(block: &BlockIr) -> bool {
    block.statements.iter().any(statement_exposes_global_object)
}

fn statement_exposes_global_object(statement: &StatementIr) -> bool {
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Throw(init)
        | StatementIr::Return(init) => expr_exposes_global_object(init),
        StatementIr::LexicalBlock(statements) => {
            statements.iter().any(statement_exposes_global_object)
        }
        StatementIr::Var(declarators) => declarators.iter().any(|declarator| {
            declarator
                .init
                .as_ref()
                .is_some_and(expr_exposes_global_object)
        }),
        StatementIr::Block(block) => block_exposes_global_object(block),
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_exposes_global_object(condition)
                || statement_exposes_global_object(then_branch)
                || else_branch
                    .as_deref()
                    .is_some_and(statement_exposes_global_object)
        }
        StatementIr::While { condition, body } | StatementIr::DoWhile { condition, body } => {
            expr_exposes_global_object(condition) || statement_exposes_global_object(body)
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => {
            init.as_ref().is_some_and(for_init_exposes_global_object)
                || test.as_ref().is_some_and(expr_exposes_global_object)
                || update.as_ref().is_some_and(expr_exposes_global_object)
                || statement_exposes_global_object(body)
        }
        StatementIr::ForOfArray { iterable, body, .. }
        | StatementIr::ForOfString { iterable, body, .. }
        | StatementIr::ForOfIterator { iterable, body, .. } => {
            expr_exposes_global_object(iterable) || statement_exposes_global_object(body)
        }
        StatementIr::ForInArray {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInString {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInObject {
            target: iterable,
            body,
            ..
        } => expr_exposes_global_object(iterable) || statement_exposes_global_object(body),
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            expr_exposes_global_object(discriminant)
                || cases.iter().any(|case| {
                    case.condition
                        .as_ref()
                        .is_some_and(expr_exposes_global_object)
                        || block_exposes_global_object(&case.body)
                })
        }
        StatementIr::Labelled { statement, .. } => statement_exposes_global_object(statement),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => block_exposes_global_object(try_block) || block_exposes_global_object(catch_block),
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => block_exposes_global_object(try_block) || block_exposes_global_object(finally_block),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            block_exposes_global_object(try_block)
                || block_exposes_global_object(catch_block)
                || block_exposes_global_object(finally_block)
        }
    }
}

fn for_init_exposes_global_object(init: &ForInitIr) -> bool {
    match init {
        ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => {
            expr_exposes_global_object(init)
        }
        ForInitIr::LexicalBlock(bindings) => bindings
            .iter()
            .any(|binding| expr_exposes_global_object(&binding.init)),
        ForInitIr::Var(declarators) => declarators.iter().any(|declarator| {
            declarator
                .init
                .as_ref()
                .is_some_and(expr_exposes_global_object)
        }),
    }
}

fn property_key_exposes_global_object(key: &PropertyKeyIr) -> bool {
    match key {
        PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
        PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
            expr_exposes_global_object(expr)
        }
    }
}

fn expr_is_global_object(expr: &TypedExpr) -> bool {
    matches!(&expr.expr, ExprIr::Identifier(name) if name == GLOBAL_THIS_NAME)
}

fn property_access_exposes_global_object(target: &TypedExpr, key: &PropertyKeyIr) -> bool {
    if expr_is_global_object(target) {
        !matches!(
            key,
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength
        )
    } else {
        expr_exposes_global_object(target)
    }
}

fn object_property_exposes_global_object(property: &ObjectPropertyIr) -> bool {
    match property {
        ObjectPropertyIr::Data { value, .. }
        | ObjectPropertyIr::NonEnumerableData { value, .. }
        | ObjectPropertyIr::Method {
            function: value, ..
        }
        | ObjectPropertyIr::Getter {
            function: value, ..
        }
        | ObjectPropertyIr::Setter {
            function: value, ..
        } => expr_exposes_global_object(value),
        ObjectPropertyIr::ComputedData { key, value } => {
            expr_exposes_global_object(key) || expr_exposes_global_object(value)
        }
        ObjectPropertyIr::ComputedMethod { key, function }
        | ObjectPropertyIr::ComputedGetter { key, function }
        | ObjectPropertyIr::ComputedSetter { key, function } => {
            expr_exposes_global_object(key) || expr_exposes_global_object(function)
        }
    }
}

fn expr_exposes_global_object(expr: &TypedExpr) -> bool {
    match &expr.expr {
        ExprIr::Identifier(name) => name == GLOBAL_THIS_NAME,
        ExprIr::ObjectLiteral(properties) => {
            properties.iter().any(object_property_exposes_global_object)
        }
        ExprIr::ArrayLiteral(elements) => elements.iter().any(expr_exposes_global_object),
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::SpreadArgument(value)
        | ExprIr::JsonParseStaticReviver { reviver: value, .. }
        | ExprIr::PrivateIn { rhs: value, .. } => expr_exposes_global_object(value),
        ExprIr::SpecOperation { operands, .. } => operands.iter().any(expr_exposes_global_object),
        ExprIr::PropertyRead { target, key }
        | ExprIr::DeleteProperty { target, key, .. }
        | ExprIr::PropertyUpdate { target, key, .. } => {
            property_access_exposes_global_object(target, key)
                || property_key_exposes_global_object(key)
        }
        ExprIr::PropertyWrite { target, key, value } => {
            property_access_exposes_global_object(target, key)
                || property_key_exposes_global_object(key)
                || expr_exposes_global_object(value)
        }
        ExprIr::StringCharCodeAt { target, index } => {
            expr_exposes_global_object(target) || expr_exposes_global_object(index)
        }
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::BitwiseNumber { lhs, rhs, .. }
        | ExprIr::StringConcat { lhs, rhs }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LooseEquality { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::Comma { lhs, rhs }
        | ExprIr::InstanceOf { lhs, rhs }
        | ExprIr::In { lhs, rhs } => {
            expr_exposes_global_object(lhs) || expr_exposes_global_object(rhs)
        }
        ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_exposes_global_object(condition)
                || expr_exposes_global_object(then_expr)
                || expr_exposes_global_object(else_expr)
        }
        ExprIr::CallNamed { args, .. } | ExprIr::SuperConstruct { args } => {
            args.iter().any(expr_exposes_global_object)
        }
        ExprIr::CallIndirect {
            callee,
            this_arg,
            args,
        } => {
            expr_exposes_global_object(callee)
                || this_arg.as_deref().is_some_and(expr_exposes_global_object)
                || args.iter().any(expr_exposes_global_object)
        }
        ExprIr::Construct { callee, args } => {
            expr_exposes_global_object(callee) || args.iter().any(expr_exposes_global_object)
        }
        ExprIr::CallMethod {
            receiver,
            key,
            args,
        } => {
            property_access_exposes_global_object(receiver, key)
                || property_key_exposes_global_object(key)
                || args.iter().any(expr_exposes_global_object)
        }
        ExprIr::SuperPropertyRead { key } => property_key_exposes_global_object(key),
        ExprIr::SuperPropertyWrite { key, value } => {
            property_key_exposes_global_object(key) || expr_exposes_global_object(value)
        }
        ExprIr::PrivateRead { target, .. } => expr_exposes_global_object(target),
        ExprIr::PrivateWrite { target, value, .. } => {
            expr_exposes_global_object(target) || expr_exposes_global_object(value)
        }
        ExprIr::ClassDefinition(class) => class
            .heritage
            .as_deref()
            .is_some_and(expr_exposes_global_object),
        ExprIr::AssertSameValue {
            actual, expected, ..
        } => expr_exposes_global_object(actual) || expr_exposes_global_object(expected),
        ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::BigInt(_)
        | ExprIr::Symbol { .. }
        | ExprIr::String(_)
        | ExprIr::FunctionValue(_)
        | ExprIr::This
        | ExprIr::Arguments
        | ExprIr::GlobalPropertyRead { .. }
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::GlobalPropertyUpdate { .. }
        | ExprIr::DeleteIdentifier { .. }
        | ExprIr::DeleteGlobalProperty { .. }
        | ExprIr::NewTarget
        | ExprIr::TypeOfUnresolvedIdentifier { .. }
        | ExprIr::RuntimeThrow { .. } => false,
    }
}

fn collect_block_global_property_names(block: &BlockIr, names: &mut BTreeSet<String>) {
    for statement in &block.statements {
        collect_statement_global_property_names(statement, names);
    }
}

fn collect_statement_global_property_names(statement: &StatementIr, names: &mut BTreeSet<String>) {
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Throw(init)
        | StatementIr::Return(init) => collect_expr_global_property_names(init, names),
        StatementIr::LexicalBlock(statements) => {
            for statement in statements {
                collect_statement_global_property_names(statement, names);
            }
        }
        StatementIr::Var(declarators) => {
            for declarator in declarators {
                if let Some(init) = &declarator.init {
                    collect_expr_global_property_names(init, names);
                }
            }
        }
        StatementIr::Block(block) => collect_block_global_property_names(block, names),
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            collect_expr_global_property_names(condition, names);
            collect_statement_global_property_names(then_branch, names);
            if let Some(else_branch) = else_branch {
                collect_statement_global_property_names(else_branch, names);
            }
        }
        StatementIr::While { condition, body } | StatementIr::DoWhile { condition, body } => {
            collect_expr_global_property_names(condition, names);
            collect_statement_global_property_names(body, names);
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => {
            if let Some(init) = init {
                collect_for_init_global_property_names(init, names);
            }
            if let Some(test) = test {
                collect_expr_global_property_names(test, names);
            }
            if let Some(update) = update {
                collect_expr_global_property_names(update, names);
            }
            collect_statement_global_property_names(body, names);
        }
        StatementIr::ForOfArray { iterable, body, .. }
        | StatementIr::ForOfString { iterable, body, .. }
        | StatementIr::ForOfIterator { iterable, body, .. } => {
            collect_expr_global_property_names(iterable, names);
            collect_statement_global_property_names(body, names);
        }
        StatementIr::ForInArray {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInString {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInObject {
            target: iterable,
            body,
            ..
        } => {
            collect_expr_global_property_names(iterable, names);
            collect_statement_global_property_names(body, names);
        }
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            collect_expr_global_property_names(discriminant, names);
            for case in cases {
                if let Some(condition) = &case.condition {
                    collect_expr_global_property_names(condition, names);
                }
                collect_block_global_property_names(&case.body, names);
            }
        }
        StatementIr::Labelled { statement, .. } => {
            collect_statement_global_property_names(statement, names);
        }
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => {
            collect_block_global_property_names(try_block, names);
            collect_block_global_property_names(catch_block, names);
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => {
            collect_block_global_property_names(try_block, names);
            collect_block_global_property_names(finally_block, names);
        }
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            collect_block_global_property_names(try_block, names);
            collect_block_global_property_names(catch_block, names);
            collect_block_global_property_names(finally_block, names);
        }
    }
}

fn collect_for_init_global_property_names(init: &ForInitIr, names: &mut BTreeSet<String>) {
    match init {
        ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => {
            collect_expr_global_property_names(init, names);
        }
        ForInitIr::LexicalBlock(bindings) => {
            for binding in bindings {
                collect_expr_global_property_names(&binding.init, names);
            }
        }
        ForInitIr::Var(declarators) => {
            for declarator in declarators {
                if let Some(init) = &declarator.init {
                    collect_expr_global_property_names(init, names);
                }
            }
        }
    }
}

fn collect_property_key_global_property_names(key: &PropertyKeyIr, names: &mut BTreeSet<String>) {
    match key {
        PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {}
        PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
            collect_expr_global_property_names(expr, names);
        }
    }
}

fn collect_object_property_global_property_names(
    property: &ObjectPropertyIr,
    names: &mut BTreeSet<String>,
) {
    match property {
        ObjectPropertyIr::Data { value, .. }
        | ObjectPropertyIr::NonEnumerableData { value, .. }
        | ObjectPropertyIr::Method {
            function: value, ..
        }
        | ObjectPropertyIr::Getter {
            function: value, ..
        }
        | ObjectPropertyIr::Setter {
            function: value, ..
        } => {
            collect_expr_global_property_names(value, names);
        }
        ObjectPropertyIr::ComputedData { key, value } => {
            collect_expr_global_property_names(key, names);
            collect_expr_global_property_names(value, names);
        }
        ObjectPropertyIr::ComputedMethod { key, function }
        | ObjectPropertyIr::ComputedGetter { key, function }
        | ObjectPropertyIr::ComputedSetter { key, function } => {
            collect_expr_global_property_names(key, names);
            collect_expr_global_property_names(function, names);
        }
    }
}

fn collect_expr_global_property_names(expr: &TypedExpr, names: &mut BTreeSet<String>) {
    match &expr.expr {
        ExprIr::Identifier(name) | ExprIr::GlobalPropertyRead { name } => {
            names.insert(name.clone());
        }
        ExprIr::GlobalPropertyWrite { name, value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { name, value, .. } => {
            names.insert(name.clone());
            collect_expr_global_property_names(value, names);
        }
        ExprIr::GlobalPropertyUpdate { name, .. } | ExprIr::DeleteGlobalProperty { name } => {
            names.insert(name.clone());
        }
        ExprIr::ObjectLiteral(properties) => {
            for property in properties {
                collect_object_property_global_property_names(property, names);
            }
        }
        ExprIr::ArrayLiteral(elements) => {
            for element in elements {
                collect_expr_global_property_names(element, names);
            }
        }
        ExprIr::AssignIdentifier { name, value }
        | ExprIr::CompoundAssignIdentifier { name, value, .. } => {
            names.insert(name.clone());
            collect_expr_global_property_names(value, names);
        }
        ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::SpreadArgument(value)
        | ExprIr::JsonParseStaticReviver { reviver: value, .. }
        | ExprIr::PrivateIn { rhs: value, .. } => collect_expr_global_property_names(value, names),
        ExprIr::SpecOperation { operands, .. } => {
            for operand in operands {
                collect_expr_global_property_names(operand, names);
            }
        }
        ExprIr::PropertyRead { target, key }
        | ExprIr::DeleteProperty { target, key, .. }
        | ExprIr::PropertyUpdate { target, key, .. } => {
            collect_expr_global_property_names(target, names);
            collect_property_key_global_property_names(key, names);
        }
        ExprIr::PropertyWrite { target, key, value } => {
            collect_expr_global_property_names(target, names);
            collect_property_key_global_property_names(key, names);
            collect_expr_global_property_names(value, names);
        }
        ExprIr::StringCharCodeAt { target, index } => {
            collect_expr_global_property_names(target, names);
            collect_expr_global_property_names(index, names);
        }
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::BitwiseNumber { lhs, rhs, .. }
        | ExprIr::StringConcat { lhs, rhs }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LooseEquality { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::Comma { lhs, rhs }
        | ExprIr::InstanceOf { lhs, rhs }
        | ExprIr::In { lhs, rhs } => {
            collect_expr_global_property_names(lhs, names);
            collect_expr_global_property_names(rhs, names);
        }
        ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            collect_expr_global_property_names(condition, names);
            collect_expr_global_property_names(then_expr, names);
            collect_expr_global_property_names(else_expr, names);
        }
        ExprIr::CallNamed { args, .. } | ExprIr::SuperConstruct { args } => {
            for arg in args {
                collect_expr_global_property_names(arg, names);
            }
        }
        ExprIr::CallIndirect {
            callee,
            this_arg,
            args,
        } => {
            collect_expr_global_property_names(callee, names);
            if let Some(this_arg) = this_arg {
                collect_expr_global_property_names(this_arg, names);
            }
            for arg in args {
                collect_expr_global_property_names(arg, names);
            }
        }
        ExprIr::Construct { callee, args } => {
            collect_expr_global_property_names(callee, names);
            for arg in args {
                collect_expr_global_property_names(arg, names);
            }
        }
        ExprIr::CallMethod {
            receiver,
            key,
            args,
        } => {
            collect_expr_global_property_names(receiver, names);
            collect_property_key_global_property_names(key, names);
            for arg in args {
                collect_expr_global_property_names(arg, names);
            }
        }
        ExprIr::SuperPropertyRead { key } => collect_property_key_global_property_names(key, names),
        ExprIr::SuperPropertyWrite { key, value } => {
            collect_property_key_global_property_names(key, names);
            collect_expr_global_property_names(value, names);
        }
        ExprIr::PrivateRead { target, .. } => collect_expr_global_property_names(target, names),
        ExprIr::PrivateWrite { target, value, .. } => {
            collect_expr_global_property_names(target, names);
            collect_expr_global_property_names(value, names);
        }
        ExprIr::ClassDefinition(class) => {
            if let Some(heritage) = &class.heritage {
                collect_expr_global_property_names(heritage, names);
            }
        }
        ExprIr::AssertSameValue {
            actual, expected, ..
        } => {
            collect_expr_global_property_names(actual, names);
            collect_expr_global_property_names(expected, names);
        }
        ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::BigInt(_)
        | ExprIr::Symbol { .. }
        | ExprIr::String(_)
        | ExprIr::FunctionValue(_)
        | ExprIr::This
        | ExprIr::Arguments
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::DeleteIdentifier { .. }
        | ExprIr::NewTarget
        | ExprIr::TypeOfUnresolvedIdentifier { .. }
        | ExprIr::RuntimeThrow { .. } => {}
    }
}

pub(crate) fn script_references_standard_builtin(
    script: &ScriptIr,
    builtin: StandardBuiltinId,
) -> bool {
    let target = builtin.function_id();
    block_references_function(&script.body, &target)
        || script.functions.iter().any(|function| {
            function.params.iter().any(|param| {
                param
                    .default_init
                    .as_ref()
                    .is_some_and(|init| expr_references_function(init, &target))
            }) || block_references_function(&function.body, &target)
        })
}

pub(crate) fn script_references_memory_atomics(script: &ScriptIr) -> bool {
    [
        StandardBuiltinId::AtomicsAdd,
        StandardBuiltinId::AtomicsAnd,
        StandardBuiltinId::AtomicsCompareExchange,
        StandardBuiltinId::AtomicsExchange,
        StandardBuiltinId::AtomicsLoad,
        StandardBuiltinId::AtomicsOr,
        StandardBuiltinId::AtomicsStore,
        StandardBuiltinId::AtomicsSub,
        StandardBuiltinId::AtomicsWait,
        StandardBuiltinId::AtomicsWaitAsync,
        StandardBuiltinId::AtomicsXor,
    ]
    .into_iter()
    .any(|builtin| script_references_standard_builtin(script, builtin))
}

/// Seed stub decision from the script text alone.
///
/// This answers "does the script text force this builtin's body?" It is only
/// the *seed* of the final compiled/stubbed partition: `emit_script` then runs
/// emission to a fixpoint, promoting every stubbed builtin whose meta is
/// actually looked up during codegen (function-value installs, funcref-table
/// wiring, direct calls — see [`FunctionMetaRegistry`]) to a real body. So a
/// builtin materialized by the bootstrap plan, by `createRealm`, or from
/// inside another compiled builtin's body never needs a carve-out here.
///
/// The carve-outs that remain below are the ones the fixpoint cannot see
/// because they add *roots* rather than bodies: values of some kind flow into
/// a dynamic method dispatch (e.g. `JSON.stringify(x).split(...)`), so the
/// method must be force-compiled here to make the bootstrap plan install it as
/// a property in the first place.
pub(crate) fn should_stub_standard_builtin(script: &ScriptIr, builtin: StandardBuiltinId) -> bool {
    if script_references_standard_builtin(script, builtin) {
        return false;
    }
    if builtin == StandardBuiltinId::StringPrototypeIndexOf
        && script_references_standard_builtin(script, StandardBuiltinId::StringPrototypeMatch)
    {
        return false;
    }
    if (builtin == StandardBuiltinId::RegExpPrototypeSymbolMatch
        || builtin == StandardBuiltinId::RegExpPrototypeSymbolMatchAll
        || builtin == StandardBuiltinId::RegExpPrototypeSymbolSearch)
        && (script_references_standard_builtin(script, StandardBuiltinId::StringPrototypeMatch)
            || script_references_standard_builtin(
                script,
                StandardBuiltinId::StringPrototypeMatchAll,
            )
            || script_references_standard_builtin(script, StandardBuiltinId::StringPrototypeSearch)
            || script_references_standard_builtin(script, StandardBuiltinId::RegExpConstructor)
            || script_references_standard_builtin(
                script,
                StandardBuiltinId::RegExpPrototypeSymbolMatchAll,
            ))
    {
        return false;
    }
    if builtin == StandardBuiltinId::TypedArrayPrototypeLengthGetter
        && script_references_standard_builtin(script, StandardBuiltinId::ArrayPrototypeConcat)
    {
        return false;
    }
    if (builtin == StandardBuiltinId::ArrayIteratorNext
        || builtin == StandardBuiltinId::ArrayIteratorIdentity)
        && [
            StandardBuiltinId::ArrayFrom,
            StandardBuiltinId::TypedArrayFrom,
            StandardBuiltinId::ArrayPrototypeKeys,
            StandardBuiltinId::ArrayPrototypeEntries,
            StandardBuiltinId::ArrayPrototypeValues,
            StandardBuiltinId::IteratorFrom,
            StandardBuiltinId::IteratorPrototypeToArray,
            StandardBuiltinId::IteratorPrototypeForEach,
            StandardBuiltinId::IteratorPrototypeEvery,
            StandardBuiltinId::IteratorPrototypeSome,
            StandardBuiltinId::IteratorPrototypeFind,
            StandardBuiltinId::IteratorPrototypeReduce,
            StandardBuiltinId::IteratorPrototypeMap,
            StandardBuiltinId::IteratorPrototypeFilter,
            StandardBuiltinId::IteratorPrototypeFlatMap,
            StandardBuiltinId::IteratorPrototypeTake,
            StandardBuiltinId::IteratorPrototypeDrop,
        ]
        .into_iter()
        .any(|dependency| script_references_standard_builtin(script, dependency))
    {
        return false;
    }
    if builtin == StandardBuiltinId::ArrayPrototypeValues
        && [
            StandardBuiltinId::ArrayFrom,
            StandardBuiltinId::TypedArrayFrom,
            StandardBuiltinId::IteratorFrom,
            StandardBuiltinId::IteratorPrototypeFlatMap,
        ]
        .into_iter()
        .any(|dependency| script_references_standard_builtin(script, dependency))
    {
        return false;
    }
    if matches!(
        builtin,
        StandardBuiltinId::StringPrototypeToString
            | StandardBuiltinId::StringPrototypeValueOf
            | StandardBuiltinId::NumberPrototypeToString
            | StandardBuiltinId::NumberPrototypeValueOf
            | StandardBuiltinId::BooleanPrototypeToString
            | StandardBuiltinId::BooleanPrototypeValueOf
            | StandardBuiltinId::BigIntPrototypeToString
            | StandardBuiltinId::BigIntPrototypeValueOf
    ) && script_references_standard_builtin(script, StandardBuiltinId::JsonStringify)
    {
        // JSON.stringify coerces primitive-wrapper objects (String/Number/
        // Boolean/BigInt exotic objects, and String/Number `space` arguments)
        // to primitives by dynamically reading and invoking their `toString` /
        // `valueOf` methods, which resolve to these prototype builtins. They are
        // never statically referenced, so materialize them alongside the helper
        // instead of letting the dynamic dispatch hit the shared stub.
        return false;
    }
    if builtin == StandardBuiltinId::StringPrototypeSplit
        && script_references_standard_builtin(script, StandardBuiltinId::JsonStringify)
    {
        // `JSON.stringify` returns a value typed `String | undefined`
        // (never a single concrete `ValueKind`), so a subsequent
        // `result.split(...)` call on that value cannot be statically
        // resolved to `StringPrototypeSplit` at the call site — it goes
        // through the generic dynamic-callee dispatch path instead, which
        // records no static reference. Materialize it alongside the
        // `JSON.stringify` helper rather than letting that dispatch land on
        // the shared "not emitted" stub.
        return false;
    }
    if builtin == StandardBuiltinId::ReflectSet
        && script_references_standard_builtin(script, StandardBuiltinId::ProxyConstructor)
    {
        return false;
    }
    if builtin == StandardBuiltinId::StringPrototypeStartsWith
        && script_references_standard_builtin(script, StandardBuiltinId::ProxyConstructor)
    {
        return false;
    }
    true
}

pub(crate) fn script_uses_create_realm(script: &ScriptIr) -> bool {
    script.host_builtins.contains(&HostBuiltinId::CreateRealm)
}

pub(crate) fn is_large_deferred_standard_builtin(builtin: StandardBuiltinId) -> bool {
    is_typed_array_constructor(builtin)
        || matches!(
            builtin,
            StandardBuiltinId::JsonParse
                | StandardBuiltinId::JsonStringify
                | StandardBuiltinId::JsonRawJson
                | StandardBuiltinId::JsonIsRawJson
                | StandardBuiltinId::AtomicsAdd
                | StandardBuiltinId::AtomicsAnd
                | StandardBuiltinId::AtomicsCompareExchange
                | StandardBuiltinId::AtomicsExchange
                | StandardBuiltinId::AtomicsLoad
                | StandardBuiltinId::AtomicsNotify
                | StandardBuiltinId::AtomicsOr
                | StandardBuiltinId::AtomicsPause
                | StandardBuiltinId::AtomicsSub
                | StandardBuiltinId::AtomicsStore
                | StandardBuiltinId::AtomicsWait
                | StandardBuiltinId::AtomicsWaitAsync
                | StandardBuiltinId::AtomicsXor
                | StandardBuiltinId::AtomicsIsLockFree
                | StandardBuiltinId::ArrayFrom
                | StandardBuiltinId::ArrayOf
                | StandardBuiltinId::ArrayPrototypeConcat
                | StandardBuiltinId::ArrayPrototypeToLocaleString
                | StandardBuiltinId::ArrayPrototypeFlat
                | StandardBuiltinId::ArrayPrototypeFlatMap
                | StandardBuiltinId::ArrayPrototypeEvery
                | StandardBuiltinId::ArrayPrototypeSome
                | StandardBuiltinId::ArrayPrototypeForEach
                | StandardBuiltinId::ArrayPrototypeFilter
                | StandardBuiltinId::ArrayPrototypeMap
                | StandardBuiltinId::ArrayPrototypeReduce
                | StandardBuiltinId::ArrayPrototypeReduceRight
                | StandardBuiltinId::ArrayPrototypePop
                | StandardBuiltinId::ArrayPrototypePush
                | StandardBuiltinId::ArrayPrototypeKeys
                | StandardBuiltinId::ArrayPrototypeEntries
                | StandardBuiltinId::ArrayPrototypeValues
                | StandardBuiltinId::ArrayIteratorNext
                | StandardBuiltinId::ArrayIteratorIdentity
                | StandardBuiltinId::ArrayBufferConstructor
                | StandardBuiltinId::SharedArrayBufferConstructor
                | StandardBuiltinId::SharedArrayBufferPrototypeGrow
                | StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
                | StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
                | StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter
                | StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter
                | StandardBuiltinId::ArrayBufferPrototypeDetachedGetter
                | StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter
                | StandardBuiltinId::ArrayBufferPrototypeResizableGetter
                | StandardBuiltinId::ArrayBufferPrototypeResize
                | StandardBuiltinId::ArrayBufferPrototypeSlice
                | StandardBuiltinId::SharedArrayBufferPrototypeSlice
                | StandardBuiltinId::ArrayBufferPrototypeTransfer
                | StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength
                | StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable
                | StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable
                | StandardBuiltinId::DataViewConstructor
                | StandardBuiltinId::DataViewPrototypeBufferGetter
                | StandardBuiltinId::DataViewPrototypeByteLengthGetter
                | StandardBuiltinId::DataViewPrototypeByteOffsetGetter
                | StandardBuiltinId::TypedArrayPrototypeBufferGetter
                | StandardBuiltinId::TypedArrayPrototypeByteLengthGetter
                | StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter
                | StandardBuiltinId::TypedArrayPrototypeLengthGetter
                | StandardBuiltinId::TypedArrayPrototypeToString
                | StandardBuiltinId::TypedArrayPrototypeToLocaleString
                | StandardBuiltinId::TypedArrayFrom
                | StandardBuiltinId::TypedArrayOf
                | StandardBuiltinId::DataViewPrototypeGetUint8
                | StandardBuiltinId::DataViewPrototypeSetUint8
                | StandardBuiltinId::DataViewPrototypeGetInt8
                | StandardBuiltinId::DataViewPrototypeSetInt8
                | StandardBuiltinId::DataViewPrototypeGetUint16
                | StandardBuiltinId::DataViewPrototypeSetUint16
                | StandardBuiltinId::DataViewPrototypeGetInt16
                | StandardBuiltinId::DataViewPrototypeSetInt16
                | StandardBuiltinId::DataViewPrototypeGetUint32
                | StandardBuiltinId::DataViewPrototypeSetUint32
                | StandardBuiltinId::DataViewPrototypeGetInt32
                | StandardBuiltinId::DataViewPrototypeSetInt32
                | StandardBuiltinId::DataViewPrototypeGetFloat16
                | StandardBuiltinId::DataViewPrototypeSetFloat16
                | StandardBuiltinId::DataViewPrototypeGetFloat32
                | StandardBuiltinId::DataViewPrototypeSetFloat32
                | StandardBuiltinId::DataViewPrototypeGetFloat64
                | StandardBuiltinId::DataViewPrototypeSetFloat64
                | StandardBuiltinId::DataViewPrototypeGetBigInt64
                | StandardBuiltinId::DataViewPrototypeSetBigInt64
                | StandardBuiltinId::DataViewPrototypeGetBigUint64
                | StandardBuiltinId::DataViewPrototypeSetBigUint64
                | StandardBuiltinId::DateConstructor
                | StandardBuiltinId::DateNow
                | StandardBuiltinId::DateUtc
                | StandardBuiltinId::DatePrototypeGetTime
                | StandardBuiltinId::DatePrototypeSetTime
                | StandardBuiltinId::DatePrototypeValueOf
                | StandardBuiltinId::DatePrototypeGetFullYear
                | StandardBuiltinId::DatePrototypeGetUtcFullYear
                | StandardBuiltinId::DatePrototypeGetMonth
                | StandardBuiltinId::DatePrototypeGetUtcMonth
                | StandardBuiltinId::DatePrototypeGetDate
                | StandardBuiltinId::DatePrototypeGetUtcDate
                | StandardBuiltinId::DatePrototypeGetDay
                | StandardBuiltinId::DatePrototypeGetUtcDay
                | StandardBuiltinId::DatePrototypeGetHours
                | StandardBuiltinId::DatePrototypeGetUtcHours
                | StandardBuiltinId::DatePrototypeGetMinutes
                | StandardBuiltinId::DatePrototypeGetUtcMinutes
                | StandardBuiltinId::DatePrototypeGetSeconds
                | StandardBuiltinId::DatePrototypeGetUtcSeconds
                | StandardBuiltinId::DatePrototypeGetMilliseconds
                | StandardBuiltinId::DatePrototypeGetUtcMilliseconds
                | StandardBuiltinId::DatePrototypeGetTimezoneOffset
                | StandardBuiltinId::DatePrototypeGetYear
                | StandardBuiltinId::DatePrototypeSetYear
                | StandardBuiltinId::DatePrototypeSetFullYear
                | StandardBuiltinId::DatePrototypeSetUtcFullYear
                | StandardBuiltinId::DatePrototypeSetMonth
                | StandardBuiltinId::DatePrototypeSetUtcMonth
                | StandardBuiltinId::DatePrototypeSetDate
                | StandardBuiltinId::DatePrototypeSetUtcDate
                | StandardBuiltinId::DatePrototypeSetHours
                | StandardBuiltinId::DatePrototypeSetUtcHours
                | StandardBuiltinId::DatePrototypeSetMinutes
                | StandardBuiltinId::DatePrototypeSetUtcMinutes
                | StandardBuiltinId::DatePrototypeSetSeconds
                | StandardBuiltinId::DatePrototypeSetUtcSeconds
                | StandardBuiltinId::DatePrototypeSetMilliseconds
                | StandardBuiltinId::DatePrototypeSetUtcMilliseconds
                | StandardBuiltinId::DatePrototypeToUtcString
                | StandardBuiltinId::RegExpConstructor
                | StandardBuiltinId::RegExpLegacyStaticGetter
                | StandardBuiltinId::RegExpLegacyStaticSetter
                | StandardBuiltinId::RegExpPrototypeSymbolMatch
                | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
                | StandardBuiltinId::RegExpPrototypeSymbolSearch
                | StandardBuiltinId::ReflectSet
                | StandardBuiltinId::BigIntConstructor
                | StandardBuiltinId::BigIntAsIntN
                | StandardBuiltinId::BigIntAsUintN
                | StandardBuiltinId::BigIntPrototypeToString
                | StandardBuiltinId::BigIntPrototypeToLocaleString
                | StandardBuiltinId::BigIntPrototypeValueOf
                | StandardBuiltinId::MathAbs
                | StandardBuiltinId::AggregateErrorConstructor
                | StandardBuiltinId::SuppressedErrorConstructor
                | StandardBuiltinId::MathAcos
                | StandardBuiltinId::MathAcosh
                | StandardBuiltinId::MathAsin
                | StandardBuiltinId::MathAsinh
                | StandardBuiltinId::MathAtan
                | StandardBuiltinId::MathAtan2
                | StandardBuiltinId::MathAtanh
                | StandardBuiltinId::MathCbrt
                | StandardBuiltinId::MathCeil
                | StandardBuiltinId::MathClz32
                | StandardBuiltinId::MathCos
                | StandardBuiltinId::MathCosh
                | StandardBuiltinId::MathExp
                | StandardBuiltinId::MathExpm1
                | StandardBuiltinId::MathF16Round
                | StandardBuiltinId::MathFloor
                | StandardBuiltinId::MathFround
                | StandardBuiltinId::MathHypot
                | StandardBuiltinId::MathImul
                | StandardBuiltinId::MathLog
                | StandardBuiltinId::MathLog10
                | StandardBuiltinId::MathLog1p
                | StandardBuiltinId::MathLog2
                | StandardBuiltinId::MathPow
                | StandardBuiltinId::MathRandom
                | StandardBuiltinId::MathRound
                | StandardBuiltinId::MathSign
                | StandardBuiltinId::MathSin
                | StandardBuiltinId::MathSinh
                | StandardBuiltinId::MathSqrt
                | StandardBuiltinId::MathSumPrecise
                | StandardBuiltinId::MathTan
                | StandardBuiltinId::MathTanh
                | StandardBuiltinId::MathTrunc
                | StandardBuiltinId::MathMin
                | StandardBuiltinId::MathMax
                | StandardBuiltinId::StringPrototypeCharAt
                | StandardBuiltinId::StringPrototypeCharCodeAt
                | StandardBuiltinId::StringPrototypeCodePointAt
                | StandardBuiltinId::StringPrototypeAt
                | StandardBuiltinId::StringPrototypeAnchor
                | StandardBuiltinId::StringPrototypeBig
                | StandardBuiltinId::StringPrototypeBlink
                | StandardBuiltinId::StringPrototypeBold
                | StandardBuiltinId::StringPrototypeFixed
                | StandardBuiltinId::StringPrototypeFontcolor
                | StandardBuiltinId::StringPrototypeFontsize
                | StandardBuiltinId::StringPrototypeItalics
                | StandardBuiltinId::StringPrototypeLink
                | StandardBuiltinId::StringPrototypeSmall
                | StandardBuiltinId::StringPrototypeStrike
                | StandardBuiltinId::StringPrototypeSub
                | StandardBuiltinId::StringPrototypeSubstr
                | StandardBuiltinId::StringPrototypeSubstring
                | StandardBuiltinId::StringPrototypeSup
                | StandardBuiltinId::StringPrototypeMatch
                | StandardBuiltinId::StringPrototypeMatchAll
                | StandardBuiltinId::StringPrototypeReplace
                | StandardBuiltinId::StringPrototypeReplaceAll
                | StandardBuiltinId::StringPrototypeSearch
                | StandardBuiltinId::StringPrototypeIndexOf
                | StandardBuiltinId::StringPrototypeLastIndexOf
                | StandardBuiltinId::StringPrototypeSlice
                | StandardBuiltinId::StringPrototypeSplit
                | StandardBuiltinId::StringPrototypePadStart
                | StandardBuiltinId::StringPrototypePadEnd
                | StandardBuiltinId::StringPrototypeRepeat
                | StandardBuiltinId::StringPrototypeEndsWith
                | StandardBuiltinId::StringPrototypeIncludes
                | StandardBuiltinId::StringPrototypeStartsWith
                | StandardBuiltinId::StringPrototypeToUpperCase
                | StandardBuiltinId::StringPrototypeTrim
                | StandardBuiltinId::StringPrototypeTrimStart
                | StandardBuiltinId::StringPrototypeTrimEnd
                | StandardBuiltinId::StringPrototypeIsWellFormed
                | StandardBuiltinId::StringPrototypeToWellFormed
                | StandardBuiltinId::ErrorConstructor
                | StandardBuiltinId::EvalErrorConstructor
                | StandardBuiltinId::RangeErrorConstructor
                | StandardBuiltinId::SyntaxErrorConstructor
                | StandardBuiltinId::TypeErrorConstructor
                | StandardBuiltinId::URIErrorConstructor
                | StandardBuiltinId::ReferenceErrorConstructor
                | StandardBuiltinId::ErrorPrototypeToString
        )
}

pub(crate) fn block_references_function(block: &BlockIr, target: &FunctionId) -> bool {
    block
        .statements
        .iter()
        .any(|statement| statement_references_function(statement, target))
}

pub(crate) fn statement_references_function(statement: &StatementIr, target: &FunctionId) -> bool {
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Throw(init)
        | StatementIr::Return(init) => expr_references_function(init, target),
        StatementIr::LexicalBlock(statements) => statements
            .iter()
            .any(|statement| statement_references_function(statement, target)),
        StatementIr::Var(declarators) => declarators.iter().any(|declarator| {
            declarator
                .init
                .as_ref()
                .is_some_and(|init| expr_references_function(init, target))
        }),
        StatementIr::Block(block) => block_references_function(block, target),
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_references_function(condition, target)
                || statement_references_function(then_branch, target)
                || else_branch
                    .as_deref()
                    .is_some_and(|branch| statement_references_function(branch, target))
        }
        StatementIr::While { condition, body } | StatementIr::DoWhile { condition, body } => {
            expr_references_function(condition, target)
                || statement_references_function(body, target)
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => {
            init.as_ref()
                .is_some_and(|init| for_init_references_function(init, target))
                || test
                    .as_ref()
                    .is_some_and(|test| expr_references_function(test, target))
                || update
                    .as_ref()
                    .is_some_and(|update| expr_references_function(update, target))
                || statement_references_function(body, target)
        }
        StatementIr::ForOfArray { iterable, body, .. }
        | StatementIr::ForOfString { iterable, body, .. }
        | StatementIr::ForOfIterator { iterable, body, .. } => {
            expr_references_function(iterable, target)
                || statement_references_function(body, target)
        }
        StatementIr::ForInArray {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInString {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInObject {
            target: iterable,
            body,
            ..
        } => {
            expr_references_function(iterable, target)
                || statement_references_function(body, target)
        }
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            expr_references_function(discriminant, target)
                || cases.iter().any(|case| {
                    case.condition
                        .as_ref()
                        .is_some_and(|condition| expr_references_function(condition, target))
                        || block_references_function(&case.body, target)
                })
        }
        StatementIr::Labelled { statement, .. } => statement_references_function(statement, target),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => {
            block_references_function(try_block, target)
                || block_references_function(catch_block, target)
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => {
            block_references_function(try_block, target)
                || block_references_function(finally_block, target)
        }
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            block_references_function(try_block, target)
                || block_references_function(catch_block, target)
                || block_references_function(finally_block, target)
        }
    }
}

pub(crate) fn for_init_references_function(init: &ForInitIr, target: &FunctionId) -> bool {
    match init {
        ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => {
            expr_references_function(init, target)
        }
        ForInitIr::LexicalBlock(bindings) => bindings
            .iter()
            .any(|binding| expr_references_function(&binding.init, target)),
        ForInitIr::Var(declarators) => declarators.iter().any(|declarator| {
            declarator
                .init
                .as_ref()
                .is_some_and(|init| expr_references_function(init, target))
        }),
    }
}

pub(crate) fn property_key_references_function(key: &PropertyKeyIr, target: &FunctionId) -> bool {
    match key {
        PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
        PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
            expr_references_function(expr, target)
        }
    }
}

pub(crate) fn static_property_key_name(key: &PropertyKeyIr) -> Option<&str> {
    match key {
        PropertyKeyIr::StaticString(name) => Some(name),
        PropertyKeyIr::ArrayLength => Some("length"),
        PropertyKeyIr::StringExpr(_) | PropertyKeyIr::ArrayIndex(_) => None,
    }
}

pub(crate) fn shape_accessor_references_function(
    shape: Option<&HeapShape>,
    key: &PropertyKeyIr,
    target: &FunctionId,
    include_getter: bool,
    include_setter: bool,
) -> bool {
    let Some(name) = static_property_key_name(key) else {
        return false;
    };
    let Some(ObjectShapeProperty::Accessor { getter, setter }) =
        shape.and_then(|shape| read_static_heap_shape_property(shape, name))
    else {
        return false;
    };

    (include_getter && getter.is_some_and(|getter| getter.function_id == *target))
        || (include_setter && setter.is_some_and(|setter| setter.function_id == *target))
}

pub(crate) fn shape_data_references_function(
    shape: Option<&HeapShape>,
    key: &PropertyKeyIr,
    target: &FunctionId,
) -> bool {
    let Some(name) = static_property_key_name(key) else {
        return false;
    };
    let Some(ObjectShapeProperty::Data(info)) =
        shape.and_then(|shape| read_static_heap_shape_property(shape, name))
    else {
        return false;
    };

    info.function_targets.contains(target)
}

pub(crate) fn object_property_references_function(
    property: &ObjectPropertyIr,
    target: &FunctionId,
) -> bool {
    match property {
        ObjectPropertyIr::Data { value, .. }
        | ObjectPropertyIr::NonEnumerableData { value, .. }
        | ObjectPropertyIr::Method {
            function: value, ..
        }
        | ObjectPropertyIr::Getter {
            function: value, ..
        }
        | ObjectPropertyIr::Setter {
            function: value, ..
        } => expr_references_function(value, target),
        ObjectPropertyIr::ComputedData { key, value } => {
            expr_references_function(key, target) || expr_references_function(value, target)
        }
        ObjectPropertyIr::ComputedMethod { key, function }
        | ObjectPropertyIr::ComputedGetter { key, function }
        | ObjectPropertyIr::ComputedSetter { key, function } => {
            expr_references_function(key, target) || expr_references_function(function, target)
        }
    }
}

pub(crate) fn optimized_call_method_references_function(
    key: &PropertyKeyIr,
    target: &FunctionId,
) -> bool {
    let PropertyKeyIr::StaticString(name) = key else {
        return false;
    };
    if name == "toString" {
        // Dynamically dispatched (receiver type not statically known to be a
        // literal), so no single call site pins one FunctionId. Without these
        // arms, `should_stub_standard_builtin` treats every primitive-wrapper
        // `toString` as unreferenced whenever it's only reached via runtime
        // property lookup (e.g. `computedNumber.toString(16)`), so the
        // builtin body is stubbed AND its Number.prototype/String.prototype/
        // etc. property is never installed, leaving the runtime property
        // read to resolve to `undefined` and trap on the "callee must be a
        // function" check instead of throwing.
        return StandardBuiltinId::NumberPrototypeToString.function_id() == *target
            || StandardBuiltinId::StringPrototypeToString.function_id() == *target
            || StandardBuiltinId::BooleanPrototypeToString.function_id() == *target
            || StandardBuiltinId::BigIntPrototypeToString.function_id() == *target;
    }
    if name == "valueOf" {
        return StandardBuiltinId::NumberPrototypeValueOf.function_id() == *target
            || StandardBuiltinId::StringPrototypeValueOf.function_id() == *target
            || StandardBuiltinId::BooleanPrototypeValueOf.function_id() == *target
            || StandardBuiltinId::BigIntPrototypeValueOf.function_id() == *target;
    }
    if name == "toFixed" {
        return StandardBuiltinId::NumberPrototypeToFixed.function_id() == *target;
    }
    if name == "toPrecision" {
        return StandardBuiltinId::NumberPrototypeToPrecision.function_id() == *target;
    }
    if name == "toExponential" {
        return StandardBuiltinId::NumberPrototypeToExponential.function_id() == *target;
    }
    if name == "includes" {
        return StandardBuiltinId::ArrayPrototypeIncludes.function_id() == *target
            || StandardBuiltinId::StringPrototypeIncludes.function_id() == *target;
    }
    if name == "indexOf" {
        return StandardBuiltinId::ArrayPrototypeIndexOf.function_id() == *target
            || StandardBuiltinId::StringPrototypeIndexOf.function_id() == *target;
    }
    if name == "lastIndexOf" {
        return StandardBuiltinId::ArrayPrototypeLastIndexOf.function_id() == *target
            || StandardBuiltinId::StringPrototypeLastIndexOf.function_id() == *target;
    }
    if name == "find" {
        return StandardBuiltinId::ArrayPrototypeFind.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeFind.function_id() == *target;
    }
    if name == "reduce" {
        return StandardBuiltinId::ArrayPrototypeReduce.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeReduce.function_id() == *target;
    }
    if name == "reduceRight" {
        return StandardBuiltinId::ArrayPrototypeReduceRight.function_id() == *target;
    }
    if name == "map" {
        return StandardBuiltinId::ArrayPrototypeMap.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeMap.function_id() == *target;
    }
    if name == "filter" {
        return StandardBuiltinId::ArrayPrototypeFilter.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeFilter.function_id() == *target;
    }
    if name == "flatMap" {
        return StandardBuiltinId::ArrayPrototypeFlatMap.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeFlatMap.function_id() == *target;
    }
    if name == "take" {
        return StandardBuiltinId::IteratorPrototypeTake.function_id() == *target;
    }
    if name == "drop" {
        return StandardBuiltinId::IteratorPrototypeDrop.function_id() == *target;
    }
    if name == "findIndex" {
        return StandardBuiltinId::ArrayPrototypeFindIndex.function_id() == *target;
    }
    if name == "findLast" {
        return StandardBuiltinId::ArrayPrototypeFindLast.function_id() == *target;
    }
    if name == "findLastIndex" {
        return StandardBuiltinId::ArrayPrototypeFindLastIndex.function_id() == *target;
    }
    if name == "every" {
        return StandardBuiltinId::ArrayPrototypeEvery.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeEvery.function_id() == *target;
    }
    if name == "some" {
        return StandardBuiltinId::ArrayPrototypeSome.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeSome.function_id() == *target;
    }
    if name == "forEach" {
        return StandardBuiltinId::ArrayPrototypeForEach.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeForEach.function_id() == *target;
    }
    if name == "at" {
        return StandardBuiltinId::ArrayPrototypeAt.function_id() == *target
            || StandardBuiltinId::StringPrototypeAt.function_id() == *target;
    }
    if name == "slice" {
        return StandardBuiltinId::StringPrototypeSlice.function_id() == *target;
    }
    if name == "toLocaleString" {
        return StandardBuiltinId::ArrayPrototypeToLocaleString.function_id() == *target
            || StandardBuiltinId::NumberPrototypeToLocaleString.function_id() == *target
            || StandardBuiltinId::BigIntPrototypeToLocaleString.function_id() == *target;
    }
    let builtin = match name.as_str() {
        "concat" => StandardBuiltinId::ArrayPrototypeConcat,
        "flat" => StandardBuiltinId::ArrayPrototypeFlat,
        "flatMap" => StandardBuiltinId::ArrayPrototypeFlatMap,
        "reduce" => StandardBuiltinId::ArrayPrototypeReduce,
        "reduceRight" => StandardBuiltinId::ArrayPrototypeReduceRight,
        "push" => StandardBuiltinId::ArrayPrototypePush,
        "from" => StandardBuiltinId::ArrayFrom,
        "of" => StandardBuiltinId::ArrayOf,
        "at" => StandardBuiltinId::ArrayPrototypeAt,
        "keys" => StandardBuiltinId::ArrayPrototypeKeys,
        "entries" => StandardBuiltinId::ArrayPrototypeEntries,
        "values" | "Symbol.iterator" => StandardBuiltinId::ArrayPrototypeValues,
        "charAt" => StandardBuiltinId::StringPrototypeCharAt,
        "charCodeAt" => StandardBuiltinId::StringPrototypeCharCodeAt,
        "codePointAt" => StandardBuiltinId::StringPrototypeCodePointAt,
        "endsWith" => StandardBuiltinId::StringPrototypeEndsWith,
        "match" => StandardBuiltinId::StringPrototypeMatch,
        "matchAll" => StandardBuiltinId::StringPrototypeMatchAll,
        "padStart" => StandardBuiltinId::StringPrototypePadStart,
        "padEnd" => StandardBuiltinId::StringPrototypePadEnd,
        "repeat" => StandardBuiltinId::StringPrototypeRepeat,
        "isWellFormed" => StandardBuiltinId::StringPrototypeIsWellFormed,
        "toWellFormed" => StandardBuiltinId::StringPrototypeToWellFormed,
        "search" => StandardBuiltinId::StringPrototypeSearch,
        "Symbol.match" => StandardBuiltinId::RegExpPrototypeSymbolMatch,
        "Symbol.matchAll" => StandardBuiltinId::RegExpPrototypeSymbolMatchAll,
        "Symbol.search" => StandardBuiltinId::RegExpPrototypeSymbolSearch,
        "startsWith" => StandardBuiltinId::StringPrototypeStartsWith,
        "toUpperCase" => StandardBuiltinId::StringPrototypeToUpperCase,
        _ => return false,
    };
    builtin.function_id() == *target
}

pub(crate) fn expr_references_function(expr: &TypedExpr, target: &FunctionId) -> bool {
    if expr.function_targets.contains(target) {
        return true;
    }
    match &expr.expr {
        ExprIr::FunctionValue(function_id) => function_id == target,
        ExprIr::ObjectLiteral(properties) => properties
            .iter()
            .any(|property| object_property_references_function(property, target)),
        ExprIr::ArrayLiteral(elements) => elements
            .iter()
            .any(|element| expr_references_function(element, target)),
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::SpreadArgument(value)
        | ExprIr::JsonParseStaticReviver { reviver: value, .. }
        | ExprIr::PrivateIn { rhs: value, .. } => expr_references_function(value, target),
        ExprIr::SpecOperation { operands, .. } => operands
            .iter()
            .any(|operand| expr_references_function(operand, target)),
        ExprIr::PropertyRead {
            target: object,
            key,
        } => {
            expr_references_function(object, target)
                || property_key_references_function(key, target)
                || optimized_call_method_references_function(key, target)
                || shape_data_references_function(object.heap_shape.as_deref(), key, target)
                || shape_accessor_references_function(
                    object.heap_shape.as_deref(),
                    key,
                    target,
                    true,
                    false,
                )
        }
        ExprIr::DeleteProperty {
            target: object,
            key,
            ..
        } => {
            expr_references_function(object, target)
                || property_key_references_function(key, target)
        }
        ExprIr::PropertyUpdate {
            target: object,
            key,
            ..
        } => {
            expr_references_function(object, target)
                || property_key_references_function(key, target)
                || shape_accessor_references_function(
                    object.heap_shape.as_deref(),
                    key,
                    target,
                    true,
                    true,
                )
        }
        ExprIr::PropertyWrite {
            target: object,
            key,
            value,
        } => {
            expr_references_function(object, target)
                || property_key_references_function(key, target)
                || expr_references_function(value, target)
                || shape_accessor_references_function(
                    object.heap_shape.as_deref(),
                    key,
                    target,
                    false,
                    true,
                )
        }
        ExprIr::StringCharCodeAt {
            target: object,
            index,
        } => expr_references_function(object, target) || expr_references_function(index, target),
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::BitwiseNumber { lhs, rhs, .. }
        | ExprIr::StringConcat { lhs, rhs }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LooseEquality { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::Comma { lhs, rhs }
        | ExprIr::InstanceOf { lhs, rhs }
        | ExprIr::In { lhs, rhs } => {
            expr_references_function(lhs, target) || expr_references_function(rhs, target)
        }
        ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_references_function(condition, target)
                || expr_references_function(then_expr, target)
                || expr_references_function(else_expr, target)
        }
        ExprIr::CallNamed { name, args } => {
            name == target || args.iter().any(|arg| expr_references_function(arg, target))
        }
        ExprIr::SuperConstruct { args } => {
            args.iter().any(|arg| expr_references_function(arg, target))
        }
        ExprIr::CallIndirect {
            callee,
            this_arg,
            args,
        } => {
            expr_references_function(callee, target)
                || this_arg
                    .as_deref()
                    .is_some_and(|this_arg| expr_references_function(this_arg, target))
                || args.iter().any(|arg| expr_references_function(arg, target))
        }
        ExprIr::Construct { callee, args } => {
            expr_references_function(callee, target)
                || args.iter().any(|arg| expr_references_function(arg, target))
        }
        ExprIr::CallMethod {
            receiver,
            key,
            args,
        } => {
            expr_references_function(receiver, target)
                || property_key_references_function(key, target)
                || shape_data_references_function(receiver.heap_shape.as_deref(), key, target)
                || optimized_call_method_references_function(key, target)
                || args.iter().any(|arg| expr_references_function(arg, target))
        }
        ExprIr::SuperPropertyRead { key } => property_key_references_function(key, target),
        ExprIr::SuperPropertyWrite { key, value } => {
            property_key_references_function(key, target) || expr_references_function(value, target)
        }
        ExprIr::PrivateRead { target: object, .. } => expr_references_function(object, target),
        ExprIr::PrivateWrite {
            target: object,
            value,
            ..
        } => expr_references_function(object, target) || expr_references_function(value, target),
        ExprIr::ClassDefinition(class) => class
            .heritage
            .as_deref()
            .is_some_and(|heritage| expr_references_function(heritage, target)),
        ExprIr::AssertSameValue {
            actual, expected, ..
        } => expr_references_function(actual, target) || expr_references_function(expected, target),
        ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::BigInt(_)
        | ExprIr::Symbol { .. }
        | ExprIr::String(_)
        | ExprIr::This
        | ExprIr::Arguments
        | ExprIr::Identifier(_)
        | ExprIr::GlobalPropertyRead { .. }
        | ExprIr::UpdateIdentifier { .. }
        | ExprIr::GlobalPropertyUpdate { .. }
        | ExprIr::DeleteIdentifier { .. }
        | ExprIr::DeleteGlobalProperty { .. }
        | ExprIr::NewTarget
        | ExprIr::TypeOfUnresolvedIdentifier { .. }
        | ExprIr::RuntimeThrow { .. } => false,
    }
}

/// Function-meta lookup table that records which standard builtins the
/// emission pass actually reaches.
///
/// Every path by which a module can reach a builtin's body at runtime goes
/// through one of a few codegen choke points: materializing the builtin's
/// function value (`emit_function_value_payload`, which writes its
/// funcref-table index into a function object), allocating a bound function
/// over it (`emit_alloc_bound_function_value`), or emitting a direct `call`
/// into its body. Those choke points call [`Self::record_standard_builtin`],
/// so after a full emission pass the recorded set is exactly the builtins the
/// emitted module can invoke — with no per-builtin knowledge in planning.
/// `emit_script` uses that set as a fixpoint: any *recorded* builtin whose
/// body was stubbed this pass gets a real body next pass, so a funcref
/// dispatch or property read can never land on the shared "standard builtin
/// body is not emitted unless referenced directly" stub for a builtin the
/// module actually materialized. New bootstrap arms and codegen-internal
/// dispatches are covered automatically because they cannot expose a builtin
/// without materializing its function value through those choke points.
pub(crate) struct FunctionMetaRegistry {
    metas: BTreeMap<FunctionId, WasmFunctionMeta>,
    touched_standard_builtins: std::cell::RefCell<BTreeSet<StandardBuiltinId>>,
    touched_host_builtins: std::cell::RefCell<BTreeSet<HostBuiltinId>>,
    /// When set, [`Self::record_standard_builtin`] / [`Self::record_host_builtin`]
    /// become no-ops. Codegen sets this while emitting a *provably dead* branch
    /// (guarded by a heap-shape/kind test whose constructor cannot exist in the
    /// current module — e.g. the proxy write-forwarding path when `Proxy` is not
    /// planned). Materializing a builtin function value there is still valid wasm
    /// (it points at the shared stub table slot), but must not drag the builtin's
    /// real body in through the emission fixpoint, since the branch can never run.
    suppress_recording: std::cell::Cell<bool>,
}

impl FunctionMetaRegistry {
    pub(crate) fn new(metas: BTreeMap<FunctionId, WasmFunctionMeta>) -> Self {
        Self {
            metas,
            touched_standard_builtins: std::cell::RefCell::new(BTreeSet::new()),
            touched_host_builtins: std::cell::RefCell::new(BTreeSet::new()),
            suppress_recording: std::cell::Cell::new(false),
        }
    }

    /// Set the recording-suppression flag, returning the previous value so the
    /// caller can restore it (supporting nested dead-branch scopes). See
    /// `suppress_recording`.
    pub(crate) fn set_recording_suppressed(&self, value: bool) -> bool {
        self.suppress_recording.replace(value)
    }

    pub(crate) fn get(&self, function_id: &str) -> Option<&WasmFunctionMeta> {
        self.metas.get(function_id)
    }

    /// Record that emission materialized this builtin's function value or
    /// emitted a direct call into its body, so its real body must be emitted.
    /// Called from the low-level codegen choke points
    /// (`emit_function_value_payload`, `emit_alloc_bound_function_value`,
    /// direct-call emitters), not from plain meta lookups: a lookup alone
    /// (e.g. to compare table indexes or to consult an install gate) does not
    /// make the builtin reachable.
    pub(crate) fn record_standard_builtin(&self, builtin: StandardBuiltinId) {
        if self.suppress_recording.get() {
            return;
        }
        self.touched_standard_builtins.borrow_mut().insert(builtin);
    }

    /// Host-builtin counterpart of [`Self::record_standard_builtin`].
    pub(crate) fn record_host_builtin(&self, builtin: HostBuiltinId) {
        if self.suppress_recording.get() {
            return;
        }
        self.touched_host_builtins.borrow_mut().insert(builtin);
    }

    /// Record whichever builtin (standard or host) this meta belongs to.
    /// Shared shorthand for the codegen choke points.
    pub(crate) fn record_builtin_meta(&self, meta: &WasmFunctionMeta) {
        if let Some(builtin) = meta.standard_builtin {
            self.record_standard_builtin(builtin);
        }
        if let Some(builtin) = meta.host_builtin {
            self.record_host_builtin(builtin);
        }
    }

    pub(crate) fn iter(&self) -> impl Iterator<Item = (&FunctionId, &WasmFunctionMeta)> {
        self.metas.iter()
    }

    pub(crate) fn values(&self) -> impl Iterator<Item = &WasmFunctionMeta> {
        self.metas.values()
    }

    pub(crate) fn metas(&self) -> &BTreeMap<FunctionId, WasmFunctionMeta> {
        &self.metas
    }

    pub(crate) fn touched_standard_builtins(&self) -> BTreeSet<StandardBuiltinId> {
        self.touched_standard_builtins.borrow().clone()
    }

    pub(crate) fn touched_host_builtins(&self) -> BTreeSet<HostBuiltinId> {
        self.touched_host_builtins.borrow().clone()
    }
}

pub(crate) fn build_function_metas(
    functions: &[FunctionIr],
    compiled_standard_builtins: &[StandardBuiltinId],
    stubbed_standard_builtins: &[StandardBuiltinId],
    compiled_host_builtins: &[HostBuiltinId],
    stubbed_host_builtins: &[HostBuiltinId],
    imported_function_count: u32,
) -> BTreeMap<FunctionId, WasmFunctionMeta> {
    let mut metas = BTreeMap::new();
    let mut callable_index = 0u32;
    for function in functions {
        metas.insert(
            function.id.clone(),
            WasmFunctionMeta {
                name: function.name.clone(),
                to_string_value: function.to_string_representation.materialize(),
                standard_builtin: None,
                host_builtin: None,
                length: function_length(&function.params),
                length_name_configurable: true,
                wasm_index: imported_function_count + 1 + callable_index,
                table_index: callable_index,
                constructable: function.constructable,
                strict: function.strict,
                class_kind: function.class_kind,
                class_heritage_kind: function.class_heritage_kind,
                is_static_class_member: function.is_static_class_member,
                is_derived_constructor: function.is_derived_constructor,
                is_synthetic_default_derived_constructor: function
                    .is_synthetic_default_derived_constructor,
                super_constructor_target: function.super_constructor_target.clone(),
                uses_super: function.uses_super,
                this_before_super: function.this_before_super,
            },
        );
        callable_index += 1;
    }

    let standard_builtin_meta =
        |builtin: StandardBuiltinId, callable_index: u32| WasmFunctionMeta {
            name: builtin
                .native_function_name()
                .unwrap_or_else(|| builtin.debug_name())
                .to_string(),
            to_string_value: match builtin {
                StandardBuiltinId::BoundFunctionInvoker => {
                    CallableToStringRepresentation::NativeAnonymous.materialize()
                }
                _ => builtin
                    .native_function_name()
                    .map(|name| {
                        CallableToStringRepresentation::NativeNamed(name.to_string()).materialize()
                    })
                    .unwrap_or_else(|| {
                        CallableToStringRepresentation::NativeAnonymous.materialize()
                    }),
            },
            standard_builtin: Some(builtin),
            host_builtin: None,
            length: standard_builtin_length(builtin),
            length_name_configurable: !matches!(builtin, StandardBuiltinId::ThrowTypeError),
            wasm_index: imported_function_count + 1 + callable_index,
            table_index: callable_index,
            constructable: builtin.constructable(),
            strict: true,
            class_kind: ClassFunctionKind::None,
            class_heritage_kind: ClassHeritageKind::None,
            is_static_class_member: false,
            is_derived_constructor: false,
            is_synthetic_default_derived_constructor: false,
            super_constructor_target: None,
            uses_super: false,
            this_before_super: false,
        };
    let host_builtin_meta = |builtin: HostBuiltinId, callable_index: u32| WasmFunctionMeta {
        name: builtin.as_str().to_string(),
        to_string_value: CallableToStringRepresentation::NativeNamed(builtin.as_str().to_string())
            .materialize(),
        standard_builtin: None,
        host_builtin: Some(builtin),
        length: host_builtin_length(builtin),
        length_name_configurable: true,
        wasm_index: imported_function_count + 1 + callable_index,
        table_index: callable_index,
        constructable: false,
        strict: true,
        class_kind: ClassFunctionKind::None,
        class_heritage_kind: ClassHeritageKind::None,
        is_static_class_member: false,
        is_derived_constructor: false,
        is_synthetic_default_derived_constructor: false,
        super_constructor_target: None,
        uses_super: false,
        this_before_super: false,
    };

    let mut shared_typed_array_constructor_callable_index = None;
    for builtin in compiled_standard_builtins {
        let builtin_callable_index = if is_typed_array_constructor(*builtin) {
            *shared_typed_array_constructor_callable_index.get_or_insert_with(|| {
                let index = callable_index;
                callable_index += 1;
                index
            })
        } else {
            let index = callable_index;
            callable_index += 1;
            index
        };
        metas.insert(
            builtin.function_id(),
            standard_builtin_meta(*builtin, builtin_callable_index),
        );
    }

    if !stubbed_standard_builtins.is_empty() || !stubbed_host_builtins.is_empty() {
        let shared_stub_callable_index = callable_index;
        callable_index += 1;
        for builtin in stubbed_standard_builtins {
            metas.insert(
                builtin.function_id(),
                standard_builtin_meta(*builtin, shared_stub_callable_index),
            );
        }
        for builtin in stubbed_host_builtins {
            metas.insert(
                builtin.function_id(),
                host_builtin_meta(*builtin, shared_stub_callable_index),
            );
        }
    }

    for builtin in compiled_host_builtins {
        metas.insert(
            builtin.function_id(),
            host_builtin_meta(*builtin, callable_index),
        );
        callable_index += 1;
    }
    metas
}

pub(crate) fn emitted_compiled_standard_builtins(
    compiled_standard_builtins: &[StandardBuiltinId],
) -> Vec<StandardBuiltinId> {
    let mut emitted = Vec::with_capacity(compiled_standard_builtins.len());
    let mut emitted_typed_array_constructor = false;
    for builtin in compiled_standard_builtins {
        if is_typed_array_constructor(*builtin) {
            if emitted_typed_array_constructor {
                continue;
            }
            emitted_typed_array_constructor = true;
        }
        emitted.push(*builtin);
    }
    emitted
}

pub(crate) fn function_length(params: &[FunctionParamIr]) -> u64 {
    params
        .iter()
        .take_while(|param| !param.is_rest && param.default_init.is_none())
        .count() as u64
}

pub(crate) fn standard_builtin_length(builtin: StandardBuiltinId) -> u64 {
    match builtin {
        StandardBuiltinId::FunctionConstructor => 1,
        StandardBuiltinId::EvalFunction => 1,
        StandardBuiltinId::FunctionPrototypeCall => 1,
        StandardBuiltinId::FunctionPrototypeApply => 2,
        StandardBuiltinId::FunctionPrototypeBind => 1,
        StandardBuiltinId::ObjectConstructor => 1,
        StandardBuiltinId::ObjectCreate => 2,
        StandardBuiltinId::ObjectGetPrototypeOf => 1,
        StandardBuiltinId::ObjectSetPrototypeOf => 2,
        StandardBuiltinId::ObjectDefineProperty => 3,
        StandardBuiltinId::ObjectDefineProperties => 2,
        StandardBuiltinId::ObjectGetOwnPropertyDescriptor => 2,
        StandardBuiltinId::ObjectGetOwnPropertyNames => 1,
        StandardBuiltinId::ObjectGetOwnPropertySymbols => 1,
        StandardBuiltinId::ObjectKeys => 1,
        StandardBuiltinId::ObjectValues => 1,
        StandardBuiltinId::ObjectHasOwn => 2,
        StandardBuiltinId::ObjectIs => 2,
        StandardBuiltinId::ObjectIsSealed => 1,
        StandardBuiltinId::ObjectIsFrozen => 1,
        StandardBuiltinId::ObjectFreeze => 1,
        StandardBuiltinId::ObjectIsExtensible => 1,
        StandardBuiltinId::ObjectPreventExtensions => 1,
        StandardBuiltinId::ObjectPrototypeHasOwnProperty => 1,
        StandardBuiltinId::ObjectPrototypePropertyIsEnumerable => 1,
        StandardBuiltinId::ObjectPrototypeIsPrototypeOf => 1,
        StandardBuiltinId::SymbolConstructor => 0,
        StandardBuiltinId::SymbolFor => 1,
        StandardBuiltinId::SymbolKeyFor => 1,
        StandardBuiltinId::SymbolPrototypeDescriptionGetter => 0,
        StandardBuiltinId::SymbolPrototypeToString => 0,
        StandardBuiltinId::SymbolPrototypeValueOf => 0,
        StandardBuiltinId::SymbolPrototypeToPrimitive => 1,
        StandardBuiltinId::ObjectPrototypeToString => 0,
        StandardBuiltinId::ObjectPrototypeToLocaleString => 0,
        StandardBuiltinId::ObjectPrototypeValueOf => 0,
        StandardBuiltinId::ProxyConstructor => 2,
        StandardBuiltinId::ProxyRevocable => 2,
        StandardBuiltinId::ProxyRevoke => 0,
        StandardBuiltinId::ReflectConstruct => 2,
        StandardBuiltinId::ReflectApply => 3,
        StandardBuiltinId::ReflectGet => 2,
        StandardBuiltinId::ReflectGetPrototypeOf => 1,
        StandardBuiltinId::ReflectGetOwnPropertyDescriptor => 2,
        StandardBuiltinId::ReflectSet => 3,
        StandardBuiltinId::ReflectHas => 2,
        StandardBuiltinId::ReflectDefineProperty => 3,
        StandardBuiltinId::ReflectDeleteProperty => 2,
        StandardBuiltinId::ReflectIsExtensible => 1,
        StandardBuiltinId::ReflectPreventExtensions => 1,
        StandardBuiltinId::ReflectSetPrototypeOf => 2,
        StandardBuiltinId::ReflectOwnKeys => 1,
        StandardBuiltinId::ArrayConstructor => 1,
        StandardBuiltinId::ArrayFrom => 1,
        StandardBuiltinId::ArrayOf => 0,
        StandardBuiltinId::TypedArrayFrom => 1,
        StandardBuiltinId::TypedArrayOf => 0,
        StandardBuiltinId::ArrayIsArray => 1,
        StandardBuiltinId::ArrayPrototypeToLocaleString => 0,
        StandardBuiltinId::ArrayPrototypeFlat => 0,
        StandardBuiltinId::ArrayPrototypeFlatMap => 1,
        StandardBuiltinId::ArrayPrototypeAt => 1,
        StandardBuiltinId::ArrayPrototypeIncludes => 1,
        StandardBuiltinId::ArrayPrototypeIndexOf => 1,
        StandardBuiltinId::ArrayPrototypeLastIndexOf => 1,
        StandardBuiltinId::ArrayPrototypeFind => 1,
        StandardBuiltinId::ArrayPrototypeFindIndex => 1,
        StandardBuiltinId::ArrayPrototypeFindLast => 1,
        StandardBuiltinId::ArrayPrototypeFindLastIndex => 1,
        StandardBuiltinId::ArrayPrototypeEvery => 1,
        StandardBuiltinId::ArrayPrototypeSome => 1,
        StandardBuiltinId::ArrayPrototypeForEach => 1,
        StandardBuiltinId::ArrayPrototypeFilter => 1,
        StandardBuiltinId::ArrayPrototypeMap => 1,
        StandardBuiltinId::ArrayPrototypeReduce => 1,
        StandardBuiltinId::ArrayPrototypeReduceRight => 1,
        StandardBuiltinId::ArrayPrototypeConcat => 1,
        StandardBuiltinId::ArrayPrototypePop => 0,
        StandardBuiltinId::ArrayPrototypePush => 1,
        StandardBuiltinId::ArrayPrototypeKeys => 0,
        StandardBuiltinId::ArrayPrototypeEntries => 0,
        StandardBuiltinId::ArrayPrototypeValues => 0,
        StandardBuiltinId::ArrayIteratorNext => 0,
        StandardBuiltinId::ArrayIteratorIdentity => 0,
        StandardBuiltinId::IteratorConstructor => 0,
        StandardBuiltinId::IteratorFrom => 1,
        StandardBuiltinId::IteratorPrototypeToArray => 0,
        StandardBuiltinId::IteratorPrototypeForEach => 1,
        StandardBuiltinId::IteratorPrototypeEvery => 1,
        StandardBuiltinId::IteratorPrototypeSome => 1,
        StandardBuiltinId::IteratorPrototypeFind => 1,
        StandardBuiltinId::IteratorPrototypeReduce => 1,
        StandardBuiltinId::IteratorPrototypeMap => 1,
        StandardBuiltinId::IteratorMapNext => 0,
        StandardBuiltinId::IteratorMapReturn => 0,
        StandardBuiltinId::IteratorPrototypeFilter => 1,
        StandardBuiltinId::IteratorFilterNext => 0,
        StandardBuiltinId::IteratorFilterReturn => 0,
        StandardBuiltinId::IteratorPrototypeFlatMap => 1,
        StandardBuiltinId::IteratorFlatMapNext => 0,
        StandardBuiltinId::IteratorFlatMapReturn => 0,
        StandardBuiltinId::IteratorPrototypeTake => 1,
        StandardBuiltinId::IteratorTakeNext => 0,
        StandardBuiltinId::IteratorTakeReturn => 0,
        StandardBuiltinId::IteratorPrototypeDrop => 1,
        StandardBuiltinId::IteratorDropNext => 0,
        StandardBuiltinId::IteratorDropReturn => 0,
        StandardBuiltinId::IteratorPrototypeConstructorGetter => 0,
        StandardBuiltinId::IteratorPrototypeConstructorSetter => 1,
        StandardBuiltinId::IteratorPrototypeSymbolDispose => 0,
        StandardBuiltinId::IteratorPrototypeToStringTagGetter => 0,
        StandardBuiltinId::IteratorPrototypeToStringTagSetter => 1,
        StandardBuiltinId::IteratorFromWrapperNext => 0,
        StandardBuiltinId::IteratorFromWrapperReturn => 0,
        StandardBuiltinId::ArrayBufferConstructor
        | StandardBuiltinId::SharedArrayBufferConstructor => 1,
        StandardBuiltinId::ArrayBufferIsView => 1,
        StandardBuiltinId::ArrayBufferPrototypeByteLengthGetter
        | StandardBuiltinId::SharedArrayBufferPrototypeByteLengthGetter
        | StandardBuiltinId::SharedArrayBufferPrototypeMaxByteLengthGetter
        | StandardBuiltinId::SharedArrayBufferPrototypeGrowableGetter => 0,
        StandardBuiltinId::SharedArrayBufferPrototypeGrow => 1,
        StandardBuiltinId::ArrayBufferPrototypeMaxByteLengthGetter => 0,
        StandardBuiltinId::ArrayBufferPrototypeDetachedGetter => 0,
        StandardBuiltinId::ArrayBufferPrototypeResizableGetter => 0,
        StandardBuiltinId::ArrayBufferPrototypeResize => 1,
        StandardBuiltinId::ArrayBufferPrototypeSlice
        | StandardBuiltinId::SharedArrayBufferPrototypeSlice => 2,
        StandardBuiltinId::ArrayBufferPrototypeTransfer
        | StandardBuiltinId::ArrayBufferPrototypeTransferToFixedLength
        | StandardBuiltinId::ArrayBufferPrototypeTransferToImmutable => 0,
        StandardBuiltinId::ArrayBufferPrototypeSliceToImmutable => 2,
        StandardBuiltinId::AtomicsAdd => 3,
        StandardBuiltinId::AtomicsAnd => 3,
        StandardBuiltinId::AtomicsCompareExchange => 4,
        StandardBuiltinId::AtomicsExchange => 3,
        StandardBuiltinId::AtomicsLoad => 2,
        StandardBuiltinId::AtomicsNotify => 3,
        StandardBuiltinId::AtomicsOr => 3,
        StandardBuiltinId::AtomicsPause => 0,
        StandardBuiltinId::AtomicsSub => 3,
        StandardBuiltinId::AtomicsStore => 3,
        StandardBuiltinId::AtomicsWait => 4,
        StandardBuiltinId::AtomicsWaitAsync => 4,
        StandardBuiltinId::AtomicsXor => 3,
        StandardBuiltinId::AtomicsIsLockFree => 1,
        StandardBuiltinId::DataViewConstructor => 1,
        StandardBuiltinId::DateConstructor => 7,
        StandardBuiltinId::RegExpConstructor => 2,
        StandardBuiltinId::RegExpLegacyStaticGetter => 0,
        StandardBuiltinId::RegExpLegacyStaticSetter => 1,
        StandardBuiltinId::RegExpPrototypeSymbolMatch
        | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
        | StandardBuiltinId::RegExpPrototypeSymbolSearch => 1,
        StandardBuiltinId::RegExpEscape => 1,
        StandardBuiltinId::JsonParse => 2,
        StandardBuiltinId::JsonStringify => 3,
        StandardBuiltinId::JsonRawJson => 1,
        StandardBuiltinId::JsonIsRawJson => 1,
        StandardBuiltinId::DateUtc => 7,
        StandardBuiltinId::DateNow
        | StandardBuiltinId::DatePrototypeGetTime
        | StandardBuiltinId::DatePrototypeValueOf
        | StandardBuiltinId::DatePrototypeGetFullYear
        | StandardBuiltinId::DatePrototypeGetUtcFullYear
        | StandardBuiltinId::DatePrototypeGetMonth
        | StandardBuiltinId::DatePrototypeGetUtcMonth
        | StandardBuiltinId::DatePrototypeGetDate
        | StandardBuiltinId::DatePrototypeGetUtcDate
        | StandardBuiltinId::DatePrototypeGetDay
        | StandardBuiltinId::DatePrototypeGetUtcDay
        | StandardBuiltinId::DatePrototypeGetHours
        | StandardBuiltinId::DatePrototypeGetUtcHours
        | StandardBuiltinId::DatePrototypeGetMinutes
        | StandardBuiltinId::DatePrototypeGetUtcMinutes
        | StandardBuiltinId::DatePrototypeGetSeconds
        | StandardBuiltinId::DatePrototypeGetUtcSeconds
        | StandardBuiltinId::DatePrototypeGetMilliseconds
        | StandardBuiltinId::DatePrototypeGetUtcMilliseconds
        | StandardBuiltinId::DatePrototypeGetTimezoneOffset
        | StandardBuiltinId::DatePrototypeGetYear
        | StandardBuiltinId::DatePrototypeToUtcString => 0,
        StandardBuiltinId::DatePrototypeSetTime | StandardBuiltinId::DatePrototypeSetYear => 1,
        StandardBuiltinId::DatePrototypeSetFullYear
        | StandardBuiltinId::DatePrototypeSetUtcFullYear
        | StandardBuiltinId::DatePrototypeSetMinutes
        | StandardBuiltinId::DatePrototypeSetUtcMinutes => 3,
        StandardBuiltinId::DatePrototypeSetMonth
        | StandardBuiltinId::DatePrototypeSetUtcMonth
        | StandardBuiltinId::DatePrototypeSetSeconds
        | StandardBuiltinId::DatePrototypeSetUtcSeconds => 2,
        StandardBuiltinId::DatePrototypeSetDate
        | StandardBuiltinId::DatePrototypeSetUtcDate
        | StandardBuiltinId::DatePrototypeSetMilliseconds
        | StandardBuiltinId::DatePrototypeSetUtcMilliseconds => 1,
        StandardBuiltinId::DatePrototypeSetHours | StandardBuiltinId::DatePrototypeSetUtcHours => 4,
        StandardBuiltinId::DataViewPrototypeBufferGetter
        | StandardBuiltinId::DataViewPrototypeByteLengthGetter
        | StandardBuiltinId::DataViewPrototypeByteOffsetGetter
        | StandardBuiltinId::TypedArrayPrototypeBufferGetter
        | StandardBuiltinId::TypedArrayPrototypeByteLengthGetter
        | StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter
        | StandardBuiltinId::TypedArrayPrototypeLengthGetter
        | StandardBuiltinId::TypedArrayPrototypeToString
        | StandardBuiltinId::TypedArrayPrototypeToLocaleString => 0,
        StandardBuiltinId::DataViewPrototypeGetUint8
        | StandardBuiltinId::DataViewPrototypeGetInt8
        | StandardBuiltinId::DataViewPrototypeGetUint16
        | StandardBuiltinId::DataViewPrototypeGetInt16
        | StandardBuiltinId::DataViewPrototypeGetUint32
        | StandardBuiltinId::DataViewPrototypeGetInt32
        | StandardBuiltinId::DataViewPrototypeGetFloat16
        | StandardBuiltinId::DataViewPrototypeGetFloat32
        | StandardBuiltinId::DataViewPrototypeGetFloat64
        | StandardBuiltinId::DataViewPrototypeGetBigInt64
        | StandardBuiltinId::DataViewPrototypeGetBigUint64 => 1,
        StandardBuiltinId::DataViewPrototypeSetUint8
        | StandardBuiltinId::DataViewPrototypeSetInt8
        | StandardBuiltinId::DataViewPrototypeSetUint16
        | StandardBuiltinId::DataViewPrototypeSetInt16
        | StandardBuiltinId::DataViewPrototypeSetUint32
        | StandardBuiltinId::DataViewPrototypeSetInt32
        | StandardBuiltinId::DataViewPrototypeSetFloat16
        | StandardBuiltinId::DataViewPrototypeSetFloat32
        | StandardBuiltinId::DataViewPrototypeSetFloat64
        | StandardBuiltinId::DataViewPrototypeSetBigInt64
        | StandardBuiltinId::DataViewPrototypeSetBigUint64 => 2,
        StandardBuiltinId::Float64ArrayConstructor
        | StandardBuiltinId::Float32ArrayConstructor
        | StandardBuiltinId::Int32ArrayConstructor
        | StandardBuiltinId::Int16ArrayConstructor
        | StandardBuiltinId::Int8ArrayConstructor
        | StandardBuiltinId::Uint32ArrayConstructor
        | StandardBuiltinId::Uint16ArrayConstructor
        | StandardBuiltinId::Uint8ArrayConstructor
        | StandardBuiltinId::Uint8ClampedArrayConstructor
        | StandardBuiltinId::BigInt64ArrayConstructor
        | StandardBuiltinId::BigUint64ArrayConstructor => 3,
        StandardBuiltinId::NumberConstructor
        | StandardBuiltinId::BigIntConstructor
        | StandardBuiltinId::NumberIsInteger
        | StandardBuiltinId::NumberIsSafeInteger
        | StandardBuiltinId::NumberIsFinite
        | StandardBuiltinId::NumberIsNaN
        | StandardBuiltinId::NumberPrototypeToExponential
        | StandardBuiltinId::NumberPrototypeToFixed
        | StandardBuiltinId::NumberPrototypeToPrecision
        | StandardBuiltinId::NumberPrototypeToString
        | StandardBuiltinId::GlobalIsFinite
        | StandardBuiltinId::GlobalIsNaN
        | StandardBuiltinId::MathAbs
        | StandardBuiltinId::MathAcos
        | StandardBuiltinId::MathAcosh
        | StandardBuiltinId::MathAsin
        | StandardBuiltinId::MathAsinh
        | StandardBuiltinId::MathAtan
        | StandardBuiltinId::MathCbrt
        | StandardBuiltinId::MathAtanh
        | StandardBuiltinId::MathCeil
        | StandardBuiltinId::MathClz32
        | StandardBuiltinId::MathCos
        | StandardBuiltinId::MathCosh
        | StandardBuiltinId::MathExp
        | StandardBuiltinId::MathExpm1
        | StandardBuiltinId::MathF16Round
        | StandardBuiltinId::MathFloor
        | StandardBuiltinId::MathFround
        | StandardBuiltinId::MathLog
        | StandardBuiltinId::MathLog10
        | StandardBuiltinId::MathLog1p
        | StandardBuiltinId::MathLog2
        | StandardBuiltinId::MathRound
        | StandardBuiltinId::MathSign
        | StandardBuiltinId::MathSin
        | StandardBuiltinId::MathSinh
        | StandardBuiltinId::MathSqrt
        | StandardBuiltinId::MathSumPrecise
        | StandardBuiltinId::MathTan
        | StandardBuiltinId::MathTanh
        | StandardBuiltinId::MathTrunc => 1,
        StandardBuiltinId::NumberPrototypeToLocaleString
        | StandardBuiltinId::NumberPrototypeValueOf
        | StandardBuiltinId::BigIntPrototypeToString
        | StandardBuiltinId::BigIntPrototypeToLocaleString
        | StandardBuiltinId::BigIntPrototypeValueOf
        | StandardBuiltinId::StringPrototypeToString
        | StandardBuiltinId::StringPrototypeValueOf
        | StandardBuiltinId::StringPrototypeToUpperCase
        | StandardBuiltinId::BooleanPrototypeToString
        | StandardBuiltinId::BooleanPrototypeValueOf => 0,
        StandardBuiltinId::BigIntAsIntN | StandardBuiltinId::BigIntAsUintN => 2,
        StandardBuiltinId::MathAtan2
        | StandardBuiltinId::MathHypot
        | StandardBuiltinId::MathImul
        | StandardBuiltinId::MathPow
        | StandardBuiltinId::MathMin
        | StandardBuiltinId::MathMax => 2,
        StandardBuiltinId::MathRandom => 0,
        StandardBuiltinId::StringConstructor
        | StandardBuiltinId::StringPrototypeCharAt
        | StandardBuiltinId::StringPrototypeCharCodeAt
        | StandardBuiltinId::StringPrototypeCodePointAt
        | StandardBuiltinId::StringPrototypeAt
        | StandardBuiltinId::StringPrototypeAnchor
        | StandardBuiltinId::StringPrototypeFontcolor
        | StandardBuiltinId::StringPrototypeFontsize
        | StandardBuiltinId::StringPrototypeLink => 1,
        StandardBuiltinId::StringPrototypeSubstr
        | StandardBuiltinId::StringPrototypeSubstring
        | StandardBuiltinId::StringPrototypeSlice => 2,
        StandardBuiltinId::StringPrototypeMatch
        | StandardBuiltinId::StringPrototypeMatchAll
        | StandardBuiltinId::StringPrototypeSearch
        | StandardBuiltinId::StringPrototypeIndexOf
        | StandardBuiltinId::StringPrototypeLastIndexOf
        | StandardBuiltinId::StringPrototypePadStart
        | StandardBuiltinId::StringPrototypePadEnd
        | StandardBuiltinId::StringPrototypeRepeat
        | StandardBuiltinId::StringPrototypeEndsWith
        | StandardBuiltinId::StringPrototypeIncludes
        | StandardBuiltinId::StringPrototypeStartsWith => 1,
        StandardBuiltinId::StringPrototypeReplace
        | StandardBuiltinId::StringPrototypeReplaceAll
        | StandardBuiltinId::StringPrototypeSplit => 2,
        StandardBuiltinId::StringPrototypeBig
        | StandardBuiltinId::StringPrototypeBlink
        | StandardBuiltinId::StringPrototypeBold
        | StandardBuiltinId::StringPrototypeFixed
        | StandardBuiltinId::StringPrototypeItalics
        | StandardBuiltinId::StringPrototypeSmall
        | StandardBuiltinId::StringPrototypeStrike
        | StandardBuiltinId::StringPrototypeSub
        | StandardBuiltinId::StringPrototypeSup
        | StandardBuiltinId::StringPrototypeTrim
        | StandardBuiltinId::StringPrototypeTrimStart
        | StandardBuiltinId::StringPrototypeTrimEnd
        | StandardBuiltinId::StringPrototypeIsWellFormed
        | StandardBuiltinId::StringPrototypeToWellFormed => 0,
        StandardBuiltinId::BooleanConstructor => 1,
        StandardBuiltinId::ErrorIsError => 1,
        StandardBuiltinId::SuppressedErrorConstructor => 3,
        StandardBuiltinId::AggregateErrorConstructor => 2,
        StandardBuiltinId::ErrorConstructor
        | StandardBuiltinId::EvalErrorConstructor
        | StandardBuiltinId::RangeErrorConstructor
        | StandardBuiltinId::SyntaxErrorConstructor
        | StandardBuiltinId::TypeErrorConstructor
        | StandardBuiltinId::URIErrorConstructor
        | StandardBuiltinId::ReferenceErrorConstructor => 1,
        StandardBuiltinId::ArraySpeciesGetter
        | StandardBuiltinId::ArrayBufferSpeciesGetter
        | StandardBuiltinId::RegExpSpeciesGetter
        | StandardBuiltinId::FunctionPrototypeToString
        | StandardBuiltinId::ErrorPrototypeToString
        | StandardBuiltinId::ThrowTypeError
        | StandardBuiltinId::BoundFunctionInvoker => 0,
        StandardBuiltinId::Escape | StandardBuiltinId::Unescape => 1,
    }
}

pub(crate) fn host_builtin_length(builtin: HostBuiltinId) -> u64 {
    match builtin {
        HostBuiltinId::Print => 1,
        HostBuiltinId::Gc => 0,
        HostBuiltinId::AssertThrows => 2,
        HostBuiltinId::IsConstructor => 1,
        HostBuiltinId::CreateRealm => 0,
        HostBuiltinId::ParseInt => 2,
        HostBuiltinId::ParseFloat => 1,
        HostBuiltinId::DetachArrayBuffer => 1,
    }
}

pub(crate) fn function_param_types() -> Vec<ValType> {
    std::iter::repeat_n(ValType::I64, JS_FUNCTION_PARAM_COUNT).collect()
}

pub(crate) fn expr_result_tag_is_runtime_dynamic(expr: &ExprIr) -> bool {
    matches!(
        expr,
        ExprIr::Identifier(_)
            | ExprIr::PropertyRead { .. }
            | ExprIr::GlobalPropertyRead { .. }
            | ExprIr::CallNamed { .. }
            | ExprIr::SpreadArgument(_)
            | ExprIr::RuntimeThrow { .. }
            | ExprIr::CallIndirect { .. }
            | ExprIr::JsonParseStaticReviver { .. }
            | ExprIr::CallMethod { .. }
            | ExprIr::Construct { .. }
            | ExprIr::SuperConstruct { .. }
            | ExprIr::SpecOperation {
                operation: SpecOperationIr::Get
                    | SpecOperationIr::GetV
                    | SpecOperationIr::GetMethod
                    | SpecOperationIr::HasProperty
                    | SpecOperationIr::Call
                    | SpecOperationIr::Construct,
                ..
            }
    )
}

pub(crate) fn count_param_locals(return_abi: ReturnAbi) -> usize {
    match return_abi {
        ReturnAbi::MainExport => 0,
        ReturnAbi::MultiValue => JS_FUNCTION_PARAM_COUNT,
    }
}

pub(crate) fn count_param_binding_locals(
    params: &[FunctionParamIr],
    owned_env_bindings: &[OwnedEnvBindingIr],
) -> usize {
    let owned = owned_env_bindings
        .iter()
        .map(|binding| binding.name.as_str())
        .collect::<BTreeSet<_>>();
    let mut locals = 0;
    for param in params {
        if !owned.contains(param.name.as_str()) {
            locals += 2;
        }
    }
    locals
}

pub(crate) fn script_uses_env(script: &ScriptIr) -> bool {
    !script.owned_env_bindings.is_empty()
        || script
            .functions
            .iter()
            .any(|function| !function.owned_env_bindings.is_empty())
}

pub(crate) fn script_uses_calls(script: &ScriptIr) -> bool {
    script
        .functions
        .iter()
        .any(|function| block_uses_calls(&function.body))
        || block_uses_calls(&script.body)
}

pub(crate) fn script_uses_function_heap(script: &ScriptIr) -> bool {
    script
        .functions
        .iter()
        .any(|function| function.flavor == FunctionFlavor::Ordinary)
}

pub(crate) fn script_uses_function_table(script: &ScriptIr) -> bool {
    script
        .functions
        .iter()
        .any(|function| block_uses_function_table(&function.body))
        || block_uses_function_table(&script.body)
}

pub(crate) fn block_uses_function_table(block: &BlockIr) -> bool {
    block.statements.iter().any(statement_uses_function_table)
}

pub(crate) fn block_uses_calls(block: &BlockIr) -> bool {
    block.statements.iter().any(statement_uses_calls)
}

pub(crate) fn statement_uses_calls(statement: &StatementIr) -> bool {
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. } | StatementIr::Expression(init) => expr_uses_calls(init),
        StatementIr::LexicalBlock(statements) => statements.iter().any(statement_uses_calls),
        StatementIr::Return(value) | StatementIr::Throw(value) => expr_uses_calls(value),
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_calls),
        StatementIr::Block(block) => block_uses_calls(block),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => block_uses_calls(try_block) || block_uses_calls(catch_block),
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => block_uses_calls(try_block) || block_uses_calls(finally_block),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            block_uses_calls(try_block)
                || block_uses_calls(catch_block)
                || block_uses_calls(finally_block)
        }
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_uses_calls(condition)
                || statement_uses_calls(then_branch)
                || else_branch
                    .as_deref()
                    .map(statement_uses_calls)
                    .unwrap_or(false)
        }
        StatementIr::While { condition, body } => {
            expr_uses_calls(condition) || statement_uses_calls(body)
        }
        StatementIr::DoWhile { body, condition } => {
            statement_uses_calls(body) || expr_uses_calls(condition)
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => {
            init.as_ref().map(for_init_uses_calls).unwrap_or(false)
                || test.as_ref().map(expr_uses_calls).unwrap_or(false)
                || update.as_ref().map(expr_uses_calls).unwrap_or(false)
                || statement_uses_calls(body)
        }
        StatementIr::ForOfArray { iterable, body, .. }
        | StatementIr::ForOfString { iterable, body, .. }
        | StatementIr::ForOfIterator { iterable, body, .. }
        | StatementIr::ForInArray {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInString {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInObject {
            target: iterable,
            body,
            ..
        } => expr_uses_calls(iterable) || statement_uses_calls(body),
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            expr_uses_calls(discriminant)
                || cases.iter().any(|case| {
                    case.condition
                        .as_ref()
                        .map(expr_uses_calls)
                        .unwrap_or(false)
                        || block_uses_calls(&case.body)
                })
        }
        StatementIr::Labelled { statement, .. } => statement_uses_calls(statement),
    }
}

pub(crate) fn for_init_uses_calls(init: &ForInitIr) -> bool {
    match init {
        ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => expr_uses_calls(init),
        ForInitIr::LexicalBlock(bindings) => bindings
            .iter()
            .any(|binding| expr_uses_calls(&binding.init)),
        ForInitIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_calls),
    }
}

pub(crate) fn statement_uses_function_table(statement: &StatementIr) -> bool {
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Return(init)
        | StatementIr::Throw(init) => expr_uses_function_table(init),
        StatementIr::LexicalBlock(statements) => {
            statements.iter().any(statement_uses_function_table)
        }
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_function_table),
        StatementIr::Block(block) => block_uses_function_table(block),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => block_uses_function_table(try_block) || block_uses_function_table(catch_block),
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => block_uses_function_table(try_block) || block_uses_function_table(finally_block),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            block_uses_function_table(try_block)
                || block_uses_function_table(catch_block)
                || block_uses_function_table(finally_block)
        }
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => {
            expr_uses_function_table(condition)
                || statement_uses_function_table(then_branch)
                || else_branch
                    .as_deref()
                    .map(statement_uses_function_table)
                    .unwrap_or(false)
        }
        StatementIr::While { condition, body } => {
            expr_uses_function_table(condition) || statement_uses_function_table(body)
        }
        StatementIr::DoWhile { body, condition } => {
            statement_uses_function_table(body) || expr_uses_function_table(condition)
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => {
            init.as_ref()
                .map(for_init_uses_function_table)
                .unwrap_or(false)
                || test.as_ref().map(expr_uses_function_table).unwrap_or(false)
                || update
                    .as_ref()
                    .map(expr_uses_function_table)
                    .unwrap_or(false)
                || statement_uses_function_table(body)
        }
        StatementIr::ForOfArray { iterable, body, .. }
        | StatementIr::ForOfString { iterable, body, .. }
        | StatementIr::ForOfIterator { iterable, body, .. }
        | StatementIr::ForInArray {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInString {
            target: iterable,
            body,
            ..
        }
        | StatementIr::ForInObject {
            target: iterable,
            body,
            ..
        } => expr_uses_function_table(iterable) || statement_uses_function_table(body),
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            expr_uses_function_table(discriminant)
                || cases.iter().any(|case| {
                    case.condition
                        .as_ref()
                        .map(expr_uses_function_table)
                        .unwrap_or(false)
                        || block_uses_function_table(&case.body)
                })
        }
        StatementIr::Labelled { statement, .. } => statement_uses_function_table(statement),
    }
}

pub(crate) fn for_init_uses_function_table(init: &ForInitIr) -> bool {
    match init {
        ForInitIr::Lexical { init, .. } | ForInitIr::Expression(init) => {
            expr_uses_function_table(init)
        }
        ForInitIr::LexicalBlock(bindings) => bindings
            .iter()
            .any(|binding| expr_uses_function_table(&binding.init)),
        ForInitIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .any(expr_uses_function_table),
    }
}

pub(crate) fn expr_uses_function_table(expr: &TypedExpr) -> bool {
    match &expr.expr {
        ExprIr::FunctionValue(_)
        | ExprIr::CallIndirect { .. }
        | ExprIr::JsonParseStaticReviver { .. }
        | ExprIr::CallMethod { .. }
        | ExprIr::Construct { .. }
        | ExprIr::ClassDefinition(_)
        | ExprIr::SuperConstruct { .. }
        | ExprIr::SuperPropertyRead { .. }
        | ExprIr::SuperPropertyWrite { .. }
        | ExprIr::PrivateRead { .. }
        | ExprIr::PrivateWrite { .. }
        | ExprIr::PrivateIn { .. } => true,
        ExprIr::GlobalPropertyRead { .. } | ExprIr::GlobalPropertyUpdate { .. } => false,
        ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { value, .. } => expr_uses_function_table(value),
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value } => expr_uses_function_table(value),
        ExprIr::SpecOperation { operands, .. } => operands.iter().any(expr_uses_function_table),
        ExprIr::StringCharCodeAt { target, index } => {
            expr_uses_function_table(target) || expr_uses_function_table(index)
        }
        ExprIr::DeleteIdentifier { .. } | ExprIr::DeleteGlobalProperty { .. } => false,
        ExprIr::TypeOfUnresolvedIdentifier { .. } => false,
        ExprIr::NewTarget => false,
        ExprIr::ObjectLiteral(properties) => properties.iter().any(|property| match property {
            ObjectPropertyIr::Data { value, .. }
            | ObjectPropertyIr::NonEnumerableData { value, .. } => expr_uses_function_table(value),
            ObjectPropertyIr::ComputedData { key, value } => {
                expr_uses_function_table(key) || expr_uses_function_table(value)
            }
            ObjectPropertyIr::ComputedMethod { key, function }
            | ObjectPropertyIr::ComputedGetter { key, function }
            | ObjectPropertyIr::ComputedSetter { key, function } => {
                expr_uses_function_table(key) || expr_uses_function_table(function)
            }
            ObjectPropertyIr::Method { function, .. }
            | ObjectPropertyIr::Getter { function, .. }
            | ObjectPropertyIr::Setter { function, .. } => expr_uses_function_table(function),
        }),
        ExprIr::ArrayLiteral(elements) => elements.iter().any(expr_uses_function_table),
        ExprIr::PropertyRead { target, key } => {
            matches!(target.kind, ValueKind::Object)
                || expr_uses_function_table(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_function_table(expr)
                    }
                }
        }
        ExprIr::PropertyWrite { target, key, value } => {
            matches!(target.kind, ValueKind::Object)
                || expr_uses_function_table(target)
                || expr_uses_function_table(value)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_function_table(expr)
                    }
                }
        }
        ExprIr::PropertyUpdate { target, key, .. } => {
            matches!(target.kind, ValueKind::Object)
                || expr_uses_function_table(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_function_table(expr)
                    }
                }
        }
        ExprIr::DeleteProperty { target, key, .. } => {
            matches!(target.kind, ValueKind::Object)
                || expr_uses_function_table(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_function_table(expr)
                    }
                }
        }
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::BitwiseNumber { lhs, rhs, .. }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LooseEquality { lhs, rhs, .. }
        | ExprIr::AssertSameValue {
            actual: lhs,
            expected: rhs,
            ..
        }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::In { lhs, rhs }
        | ExprIr::StringConcat { lhs, rhs }
        | ExprIr::Comma { lhs, rhs } => {
            expr_uses_function_table(lhs)
                || expr_uses_function_table(rhs)
                || lhs.possible_kinds.contains(ValueKind::Object)
                || rhs.possible_kinds.contains(ValueKind::Object)
        }
        ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => {
            expr_uses_function_table(condition)
                || expr_uses_function_table(then_expr)
                || expr_uses_function_table(else_expr)
        }
        ExprIr::CallNamed { args, .. } => args.iter().any(expr_uses_function_table),
        ExprIr::SpreadArgument(value) => expr_uses_function_table(value),
        ExprIr::InstanceOf { lhs, rhs } => {
            expr_uses_function_table(lhs) || expr_uses_function_table(rhs)
        }
        ExprIr::Arguments => false,
        ExprIr::RuntimeThrow { .. } => false,
        ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::BigInt(_)
        | ExprIr::Symbol { .. }
        | ExprIr::String(_)
        | ExprIr::This
        | ExprIr::Identifier(_)
        | ExprIr::UpdateIdentifier { .. } => false,
    }
}

pub(crate) fn expr_uses_calls(expr: &TypedExpr) -> bool {
    match &expr.expr {
        ExprIr::CallNamed { .. }
        | ExprIr::SpreadArgument(_)
        | ExprIr::CallIndirect { .. }
        | ExprIr::JsonParseStaticReviver { .. }
        | ExprIr::CallMethod { .. }
        | ExprIr::Construct { .. }
        | ExprIr::ClassDefinition(_)
        | ExprIr::SuperConstruct { .. }
        | ExprIr::SuperPropertyRead { .. }
        | ExprIr::SuperPropertyWrite { .. }
        | ExprIr::PrivateRead { .. }
        | ExprIr::PrivateWrite { .. }
        | ExprIr::PrivateIn { .. } => true,
        ExprIr::GlobalPropertyRead { .. } | ExprIr::GlobalPropertyUpdate { .. } => false,
        ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { value, .. } => expr_uses_calls(value),
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value } => expr_uses_calls(value),
        ExprIr::SpecOperation { operands, .. } => operands.iter().any(expr_uses_calls),
        ExprIr::StringCharCodeAt { target, index } => {
            expr_uses_calls(target) || expr_uses_calls(index)
        }
        ExprIr::DeleteIdentifier { .. } | ExprIr::DeleteGlobalProperty { .. } => false,
        ExprIr::TypeOfUnresolvedIdentifier { .. } => false,
        ExprIr::NewTarget => false,
        ExprIr::ObjectLiteral(properties) => properties.iter().any(|property| match property {
            ObjectPropertyIr::Data { value, .. }
            | ObjectPropertyIr::NonEnumerableData { value, .. } => expr_uses_calls(value),
            ObjectPropertyIr::ComputedData { key, value } => {
                expr_uses_calls(key) || expr_uses_calls(value)
            }
            ObjectPropertyIr::ComputedMethod { key, function }
            | ObjectPropertyIr::ComputedGetter { key, function }
            | ObjectPropertyIr::ComputedSetter { key, function } => {
                expr_uses_calls(key) || expr_uses_calls(function)
            }
            ObjectPropertyIr::Method { function, .. }
            | ObjectPropertyIr::Getter { function, .. }
            | ObjectPropertyIr::Setter { function, .. } => expr_uses_calls(function),
        }),
        ExprIr::ArrayLiteral(elements) => elements.iter().any(expr_uses_calls),
        ExprIr::PropertyRead { target, key } => {
            expr_uses_calls(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_calls(expr)
                    }
                }
        }
        ExprIr::PropertyWrite { target, key, value } => {
            expr_uses_calls(target)
                || expr_uses_calls(value)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_calls(expr)
                    }
                }
        }
        ExprIr::PropertyUpdate { target, key, .. } => {
            expr_uses_calls(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_calls(expr)
                    }
                }
        }
        ExprIr::DeleteProperty { target, key, .. } => {
            expr_uses_calls(target)
                || match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        expr_uses_calls(expr)
                    }
                }
        }
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::BitwiseNumber { lhs, rhs, .. }
        | ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::StrictEquality { lhs, rhs, .. }
        | ExprIr::LooseEquality { lhs, rhs, .. }
        | ExprIr::AssertSameValue {
            actual: lhs,
            expected: rhs,
            ..
        }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::In { lhs, rhs }
        | ExprIr::StringConcat { lhs, rhs }
        | ExprIr::Comma { lhs, rhs } => {
            expr_uses_calls(lhs)
                || expr_uses_calls(rhs)
                || lhs.possible_kinds.contains(ValueKind::Object)
                || rhs.possible_kinds.contains(ValueKind::Object)
        }
        ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => expr_uses_calls(condition) || expr_uses_calls(then_expr) || expr_uses_calls(else_expr),
        ExprIr::InstanceOf { lhs, rhs } => expr_uses_calls(lhs) || expr_uses_calls(rhs),
        ExprIr::Arguments
        | ExprIr::RuntimeThrow { .. }
        | ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::BigInt(_)
        | ExprIr::Symbol { .. }
        | ExprIr::String(_)
        | ExprIr::FunctionValue(_)
        | ExprIr::This
        | ExprIr::Identifier(_)
        | ExprIr::UpdateIdentifier { .. } => false,
    }
}

pub(crate) fn count_block_lexicals(block: &BlockIr) -> usize {
    block.statements.iter().map(count_statement_lexicals).sum()
}

pub(crate) fn count_block_temp_locals(block: &BlockIr) -> usize {
    block
        .statements
        .iter()
        .map(count_statement_temp_locals)
        .max()
        .unwrap_or(0)
}

pub(crate) fn count_statement_lexicals(statement: &StatementIr) -> usize {
    match statement {
        StatementIr::Empty
        | StatementIr::Var(_)
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Return(_)
        | StatementIr::Throw(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => 0,
        StatementIr::Lexical { .. } => 2,
        StatementIr::LexicalBlock(statements) => {
            statements.iter().map(count_statement_lexicals).sum()
        }
        StatementIr::Block(block) => count_block_lexicals(block),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => count_block_lexicals(try_block) + 2 + count_block_lexicals(catch_block),
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => count_block_lexicals(try_block) + count_block_lexicals(finally_block),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            count_block_lexicals(try_block)
                + 2
                + count_block_lexicals(catch_block)
                + count_block_lexicals(finally_block)
        }
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            count_statement_lexicals(then_branch)
                + else_branch
                    .as_deref()
                    .map(count_statement_lexicals)
                    .unwrap_or(0)
        }
        StatementIr::While { body, .. } | StatementIr::DoWhile { body, .. } => {
            count_statement_lexicals(body)
        }
        StatementIr::For { init, body, .. } => {
            init.as_ref()
                .map(|init| match init {
                    ForInitIr::Lexical { .. } => 1,
                    ForInitIr::LexicalBlock(bindings) => bindings.len(),
                    ForInitIr::Var(_) => 0,
                    ForInitIr::Expression(_) => 0,
                })
                .unwrap_or(0)
                + count_statement_lexicals(body)
        }
        StatementIr::ForOfArray { body, .. }
        | StatementIr::ForOfString { body, .. }
        | StatementIr::ForOfIterator { body, .. }
        | StatementIr::ForInArray { body, .. }
        | StatementIr::ForInString { body, .. }
        | StatementIr::ForInObject { body, .. } => 2 + count_statement_lexicals(body),
        StatementIr::Switch { cases, .. } => cases
            .iter()
            .map(|case| count_block_lexicals(&case.body))
            .sum(),
        StatementIr::Labelled { statement, .. } => count_statement_lexicals(statement),
    }
}

pub(crate) fn count_statement_temp_locals(statement: &StatementIr) -> usize {
    match statement {
        StatementIr::Empty
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => 0,
        StatementIr::Return(value) | StatementIr::Throw(value) => count_expr_temp_locals(value),
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0),
        StatementIr::Lexical { init, .. } | StatementIr::Expression(init) => {
            count_expr_temp_locals(init)
        }
        StatementIr::LexicalBlock(statements) => statements
            .iter()
            .map(count_statement_temp_locals)
            .max()
            .unwrap_or(0),
        StatementIr::Block(block) => count_block_temp_locals(block),
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => count_block_temp_locals(try_block)
            .max(count_block_temp_locals(catch_block))
            .max(2),
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => count_block_temp_locals(try_block).max(count_block_temp_locals(finally_block)),
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => count_block_temp_locals(try_block)
            .max(count_block_temp_locals(catch_block))
            .max(count_block_temp_locals(finally_block))
            .max(2),
        StatementIr::If {
            condition,
            then_branch,
            else_branch,
        } => count_expr_temp_locals(condition)
            .max(count_statement_temp_locals(then_branch))
            .max(
                else_branch
                    .as_deref()
                    .map(count_statement_temp_locals)
                    .unwrap_or(0),
            ),
        StatementIr::While { condition, body } => {
            count_expr_temp_locals(condition).max(count_statement_temp_locals(body))
        }
        StatementIr::DoWhile { body, condition } => {
            count_statement_temp_locals(body).max(count_expr_temp_locals(condition))
        }
        StatementIr::For {
            init,
            test,
            update,
            body,
        } => init
            .as_ref()
            .map(count_for_init_temp_locals)
            .unwrap_or(0)
            .max(test.as_ref().map(count_expr_temp_locals).unwrap_or(0))
            .max(update.as_ref().map(count_expr_temp_locals).unwrap_or(0))
            .max(count_statement_temp_locals(body)),
        StatementIr::ForOfArray { iterable, body, .. } => 7
            .max(count_expr_temp_locals(iterable))
            .max(count_statement_temp_locals(body)),
        StatementIr::ForOfString { iterable, body, .. } => 12
            .max(count_expr_temp_locals(iterable))
            .max(count_statement_temp_locals(body)),
        StatementIr::ForOfIterator { iterable, body, .. } => 18
            .max(count_expr_temp_locals(iterable))
            .max(count_statement_temp_locals(body)),
        StatementIr::ForInArray { target, body, .. } => 10
            .max(count_expr_temp_locals(target))
            .max(count_statement_temp_locals(body)),
        StatementIr::ForInString { target, body, .. } => 10
            .max(count_expr_temp_locals(target))
            .max(count_statement_temp_locals(body)),
        StatementIr::ForInObject { target, body, .. } => 9
            .max(count_expr_temp_locals(target))
            .max(count_statement_temp_locals(body)),
        StatementIr::Switch {
            discriminant,
            cases,
        } => {
            let case_max = cases
                .iter()
                .map(|case| {
                    case.condition
                        .as_ref()
                        .map(count_expr_temp_locals)
                        .unwrap_or(0)
                        .max(count_block_temp_locals(&case.body))
                })
                .max()
                .unwrap_or(0);
            4 + count_expr_temp_locals(discriminant).max(case_max)
        }
        StatementIr::Labelled { statement, .. } => count_statement_temp_locals(statement),
    }
}

pub(crate) fn count_for_init_temp_locals(init: &ForInitIr) -> usize {
    match init {
        ForInitIr::Lexical { init, .. } => count_expr_temp_locals(init),
        ForInitIr::LexicalBlock(bindings) => bindings
            .iter()
            .map(|binding| count_expr_temp_locals(&binding.init))
            .max()
            .unwrap_or(0),
        ForInitIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0),
        ForInitIr::Expression(expr) => count_expr_temp_locals(expr),
    }
}

pub(crate) fn count_expr_temp_locals(expr: &TypedExpr) -> usize {
    match &expr.expr {
        ExprIr::GlobalPropertyRead { .. } => 12,
        ExprIr::GlobalPropertyWrite { value, .. } => count_expr_temp_locals(value).max(12),
        ExprIr::GlobalPropertyUpdate { return_mode, .. } => match return_mode {
            UpdateReturnMode::Prefix => 12,
            UpdateReturnMode::Postfix => 13,
        },
        ExprIr::GlobalPropertyCompoundAssign { value, .. } => count_expr_temp_locals(value).max(13),
        ExprIr::ObjectLiteral(properties) => {
            let child = properties
                .iter()
                .map(|property| match property {
                    ObjectPropertyIr::Data { value, .. }
                    | ObjectPropertyIr::NonEnumerableData { value, .. } => {
                        count_expr_temp_locals(value)
                    }
                    ObjectPropertyIr::ComputedData { key, value } => {
                        count_expr_temp_locals(key).max(count_expr_temp_locals(value))
                    }
                    ObjectPropertyIr::ComputedMethod { key, function }
                    | ObjectPropertyIr::ComputedGetter { key, function }
                    | ObjectPropertyIr::ComputedSetter { key, function } => {
                        count_expr_temp_locals(key).max(count_expr_temp_locals(function))
                    }
                    ObjectPropertyIr::Method { function, .. }
                    | ObjectPropertyIr::Getter { function, .. }
                    | ObjectPropertyIr::Setter { function, .. } => count_expr_temp_locals(function),
                })
                .max()
                .unwrap_or(0);
            child.max(12)
        }
        ExprIr::ArrayLiteral(elements) => {
            let child = elements
                .iter()
                .map(count_expr_temp_locals)
                .max()
                .unwrap_or(0);
            child.max(6)
        }
        ExprIr::PropertyRead { target, key } => {
            let child = count_expr_temp_locals(target).max(match key {
                PropertyKeyIr::StaticString(_) => 0,
                PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            });
            child.max(12)
        }
        ExprIr::PropertyWrite { target, key, value } => {
            let child = count_expr_temp_locals(target)
                .max(count_expr_temp_locals(value))
                .max(match key {
                    PropertyKeyIr::StaticString(_) => 0,
                    PropertyKeyIr::ArrayLength => 0,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        count_expr_temp_locals(expr)
                    }
                });
            child.max(96)
        }
        ExprIr::DeleteProperty { target, key, .. } => {
            let child = count_expr_temp_locals(target).max(match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            });
            child.max(12)
        }
        ExprIr::PropertyUpdate { target, key, .. } => {
            let child = count_expr_temp_locals(target).max(match key {
                PropertyKeyIr::StaticString(_) => 0,
                PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            });
            child.max(14)
        }
        ExprIr::DeleteIdentifier { .. } => 0,
        ExprIr::DeleteGlobalProperty { .. } => 12,
        ExprIr::UpdateIdentifier { return_mode, .. } => match return_mode {
            UpdateReturnMode::Prefix => 0,
            UpdateReturnMode::Postfix => 1,
        },
        ExprIr::CompoundAssignIdentifier { op, value, .. } => {
            let child = count_expr_temp_locals(value);
            if matches!(op, ArithmeticBinaryOp::Add) {
                5 + child
            } else if matches!(op, ArithmeticBinaryOp::Exp) {
                6 + child
            } else {
                3 + child
            }
        }
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::UnaryNumber { expr: value, .. }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value } => count_expr_temp_locals(value),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::IsCallable,
            operands,
        } => {
            2 + operands
                .iter()
                .map(count_expr_temp_locals)
                .max()
                .unwrap_or(0)
                .max(4)
        }
        ExprIr::SpecOperation {
            operation: SpecOperationIr::IsConstructor,
            operands,
        } => {
            2 + operands
                .iter()
                .map(count_expr_temp_locals)
                .max()
                .unwrap_or(0)
                .max(6)
        }
        ExprIr::SpecOperation {
            operation: SpecOperationIr::IsPropertyKey,
            operands,
        } => {
            2 + operands
                .iter()
                .map(count_expr_temp_locals)
                .max()
                .unwrap_or(0)
        }
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToBoolean,
            operands,
        } => {
            1 + operands
                .iter()
                .map(count_expr_temp_locals)
                .max()
                .unwrap_or(0)
        }
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToPrimitive(_),
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(16),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToNumeric,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(4),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToNumber,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(12),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToBigInt,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(16),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToString,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(16),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToObject,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(8),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToPropertyKey,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(16),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToIntegerOrInfinity,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(3),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToLength,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(3),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::ToIndex,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(3),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::SameValue,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(4),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::SameValueZero,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(4),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::StrictEqualityComparison,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(4),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::IsLooselyEqual,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(9),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::Get | SpecOperationIr::GetV,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(14),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::GetMethod,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(16),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::Call,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(4 + operands.len() * 2),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::Construct,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(6 + operands.len() * 2),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::Set,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(32),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::HasProperty,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(14),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::HasOwnProperty,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(16),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::DeletePropertyOrThrow,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(18),
        ExprIr::SpecOperation {
            operation: SpecOperationIr::CreateDataPropertyOrThrow,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(24),
        ExprIr::TypeOf { expr: value } => count_expr_temp_locals(value).max(5),
        ExprIr::StringCharCodeAt { target, index } => {
            16 + count_expr_temp_locals(target).max(count_expr_temp_locals(index))
        }
        ExprIr::TypeOfUnresolvedIdentifier { .. } => 0,
        ExprIr::NewTarget => 0,
        ExprIr::BinaryNumber { op, lhs, rhs } | ExprIr::CoerciveBinaryNumber { op, lhs, rhs } => {
            let child = count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs));
            if matches!(op, ArithmeticBinaryOp::Exp) {
                child.max(12)
            } else {
                child
            }
        }
        ExprIr::CompareNumber { lhs, rhs, .. }
        | ExprIr::CompareValue { lhs, rhs, .. }
        | ExprIr::LogicalShortCircuit { lhs, rhs, .. }
        | ExprIr::In { lhs, rhs } => count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs)),
        ExprIr::BitwiseNumber { lhs, rhs, .. } => {
            2 + count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs))
        }
        ExprIr::StringConcat { lhs, rhs } => {
            18 + count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs))
        }
        ExprIr::CoerciveAdd { lhs, rhs } => count_expr_temp_locals(lhs)
            .max(count_expr_temp_locals(rhs))
            .max(96),
        ExprIr::Comma { lhs, rhs } => count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs)),
        ExprIr::Conditional {
            condition,
            then_expr,
            else_expr,
        } => count_expr_temp_locals(condition)
            .max(count_expr_temp_locals(then_expr))
            .max(count_expr_temp_locals(else_expr)),
        ExprIr::StrictEquality { lhs, rhs, .. } => {
            let child = count_expr_temp_locals(lhs).max(count_expr_temp_locals(rhs));
            if expr_result_tag_is_runtime_dynamic(&lhs.expr)
                || expr_result_tag_is_runtime_dynamic(&rhs.expr)
            {
                child + 4
            } else {
                child
            }
        }
        ExprIr::LooseEquality { lhs, rhs, .. } => count_expr_temp_locals(lhs)
            .max(count_expr_temp_locals(rhs))
            .max(5),
        ExprIr::AssertSameValue {
            actual, expected, ..
        } => count_expr_temp_locals(actual)
            .max(count_expr_temp_locals(expected))
            .max(4),
        ExprIr::CallNamed { args, .. } => args
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(4 + args.len() * 2),
        ExprIr::SpreadArgument(value) => count_expr_temp_locals(value).max(2),
        ExprIr::RuntimeThrow { .. } => 4,
        ExprIr::CallIndirect {
            callee,
            args,
            this_arg,
        } => count_expr_temp_locals(callee)
            .max(this_arg.as_deref().map(count_expr_temp_locals).unwrap_or(0))
            .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
            .max(64),
        ExprIr::JsonParseStaticReviver { reviver, .. } => count_expr_temp_locals(reviver).max(64),
        ExprIr::Construct { callee, args } => count_expr_temp_locals(callee)
            .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
            .max(10 + args.len() * 2),
        ExprIr::CallMethod {
            receiver,
            key,
            args,
        } => {
            let key_child = match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            count_expr_temp_locals(receiver)
                .max(key_child)
                .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
                .max(16 + args.len() * 2)
        }
        ExprIr::InstanceOf { lhs, rhs } => count_expr_temp_locals(lhs)
            .max(count_expr_temp_locals(rhs))
            .max(8),
        ExprIr::ClassDefinition(_) => 24,
        ExprIr::SuperConstruct { args } => args
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(12),
        ExprIr::SuperPropertyRead { key } => match key {
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 8,
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                count_expr_temp_locals(expr).max(8)
            }
        },
        ExprIr::SuperPropertyWrite { key, value } => {
            let key_child = match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            count_expr_temp_locals(value).max(key_child).max(10)
        }
        ExprIr::PrivateRead { target, .. } => count_expr_temp_locals(target).max(8),
        ExprIr::PrivateWrite { target, value, .. } => count_expr_temp_locals(target)
            .max(count_expr_temp_locals(value))
            .max(10),
        ExprIr::PrivateIn { rhs, .. } => count_expr_temp_locals(rhs).max(8),
        ExprIr::Arguments => 0,
        ExprIr::Undefined
        | ExprIr::ArrayHole
        | ExprIr::Null
        | ExprIr::Boolean(_)
        | ExprIr::Number(_)
        | ExprIr::BigInt(_)
        | ExprIr::Symbol { .. }
        | ExprIr::String(_)
        | ExprIr::FunctionValue(_)
        | ExprIr::This
        | ExprIr::Identifier(_) => 0,
    }
}

pub(crate) fn collect_hoisted_vars_block_root(block: &BlockIr) -> Vec<String> {
    let mut names = BTreeSet::new();
    collect_hoisted_vars_block(block, &mut names);
    names.into_iter().collect()
}

pub(crate) fn collect_hoisted_vars_block(block: &BlockIr, names: &mut BTreeSet<String>) {
    for statement in &block.statements {
        collect_hoisted_vars_statement(statement, names);
    }
}

pub(crate) fn collect_hoisted_vars_statement(
    statement: &StatementIr,
    names: &mut BTreeSet<String>,
) {
    match statement {
        StatementIr::Var(declarators) => {
            for declarator in declarators {
                names.insert(declarator.name.clone());
            }
        }
        StatementIr::LexicalBlock(statements) => {
            for statement in statements {
                collect_hoisted_vars_statement(statement, names);
            }
        }
        StatementIr::Block(block) => collect_hoisted_vars_block(block, names),
        StatementIr::If {
            then_branch,
            else_branch,
            ..
        } => {
            collect_hoisted_vars_statement(then_branch, names);
            if let Some(else_branch) = else_branch {
                collect_hoisted_vars_statement(else_branch, names);
            }
        }
        StatementIr::While { body, .. }
        | StatementIr::DoWhile { body, .. }
        | StatementIr::Labelled {
            statement: body, ..
        } => collect_hoisted_vars_statement(body, names),
        StatementIr::For { init, body, .. } => {
            if let Some(ForInitIr::Var(declarators)) = init {
                for declarator in declarators {
                    names.insert(declarator.name.clone());
                }
            }
            collect_hoisted_vars_statement(body, names);
        }
        StatementIr::ForOfArray {
            mode, name, body, ..
        }
        | StatementIr::ForOfString {
            mode, name, body, ..
        }
        | StatementIr::ForOfIterator {
            mode, name, body, ..
        } => {
            if *mode == BindingMode::Var {
                names.insert(name.clone());
            }
            collect_hoisted_vars_statement(body, names);
        }
        StatementIr::ForInArray {
            mode, name, body, ..
        }
        | StatementIr::ForInString {
            mode, name, body, ..
        }
        | StatementIr::ForInObject {
            mode, name, body, ..
        } => {
            if *mode == BindingMode::Var {
                names.insert(name.clone());
            }
            collect_hoisted_vars_statement(body, names);
        }
        StatementIr::Switch { cases, .. } => {
            for case in cases {
                collect_hoisted_vars_block(&case.body, names);
            }
        }
        StatementIr::TryCatch {
            try_block,
            catch_block,
            ..
        } => {
            collect_hoisted_vars_block(try_block, names);
            collect_hoisted_vars_block(catch_block, names);
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
        } => {
            collect_hoisted_vars_block(try_block, names);
            collect_hoisted_vars_block(finally_block, names);
        }
        StatementIr::TryCatchFinally {
            try_block,
            catch_block,
            finally_block,
            ..
        } => {
            collect_hoisted_vars_block(try_block, names);
            collect_hoisted_vars_block(catch_block, names);
            collect_hoisted_vars_block(finally_block, names);
        }
        StatementIr::Empty
        | StatementIr::Lexical { .. }
        | StatementIr::Expression(_)
        | StatementIr::Debugger
        | StatementIr::Return(_)
        | StatementIr::Throw(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
    }
}
