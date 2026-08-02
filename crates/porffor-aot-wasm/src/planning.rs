use super::*;
use porffor_ir::{ObjectDestructuringPatternIr, OptionalChainOperationIr};

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
    pub(crate) execution_kind: FunctionExecutionKind,
    pub(crate) constructable: bool,
    pub(crate) strict: bool,
    pub(crate) is_named_expression: bool,
    pub(crate) class_kind: ClassFunctionKind,
    pub(crate) class_element_execution_kind: ClassElementExecutionKind,
    pub(crate) class_heritage_kind: ClassHeritageKind,
    pub(crate) is_static_class_member: bool,
    pub(crate) is_derived_constructor: bool,
    pub(crate) is_synthetic_default_derived_constructor: bool,
    pub(crate) class_instance_element_plan: Option<ClassInstanceElementPlanIr>,
    pub(crate) super_constructor_target: Option<FunctionId>,
    pub(crate) uses_super: bool,
    pub(crate) this_before_super: bool,
    pub(crate) captures_private_environment: bool,
    pub(crate) needs_active_function_identity: bool,
}

impl WasmFunctionMeta {
    pub(crate) fn runtime_name(&self) -> &str {
        if self.class_kind != ClassFunctionKind::Method {
            return self.name.as_str();
        }
        self.name
            .split_once('.')
            .map_or(self.name.as_str(), |(_, method_name)| method_name)
    }

    pub(crate) const fn has_class_execution_context(&self) -> bool {
        !matches!(self.class_kind, ClassFunctionKind::None)
            || !matches!(
                self.class_element_execution_kind,
                ClassElementExecutionKind::None
            )
    }

    pub(crate) const fn has_function_context(&self) -> bool {
        self.needs_active_function_identity
            || self.has_class_execution_context()
            || self.captures_private_environment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use porffor_front::{parse, ParseOptions};
    use porffor_ir::{lower, BigIntLiteralIr};

    fn lower_script(source: &str) -> ScriptIr {
        let parsed = parse(source, ParseOptions::script()).expect("script should parse");
        lower(&parsed).script.expect("script should lower")
    }

    #[test]
    fn heap_bigint_literals_require_the_emitted_result_tag() {
        let literal = ExprIr::BigInt(BigIntLiteralIr::from_u64_payload(1_u64 << 63));

        assert!(expr_result_tag_is_runtime_dynamic(&literal));
    }

    #[test]
    fn to_bigint_operations_require_the_emitted_result_tag() {
        let conversion = ExprIr::SpecOperation {
            operation: SpecOperationIr::ToBigInt,
            operands: vec![TypedExpr::undefined()],
        };

        assert!(expr_result_tag_is_runtime_dynamic(&conversion));
    }

    #[test]
    fn materialized_bindings_preserve_dynamic_body_tags() {
        let body = TypedExpr::from_info(
            ValueInfo::new(ValueKind::BigInt),
            ExprIr::CallIndirect {
                callee: Box::new(TypedExpr::undefined()),
                this_arg: None,
                args: Vec::new(),
                static_regexp_compilation: None,
            },
        );
        let materialized = ExprIr::MaterializeBinding {
            name: "__materialized".to_string(),
            value: Box::new(TypedExpr::undefined()),
            body: Box::new(body),
        };

        assert!(expr_result_tag_is_runtime_dynamic(&materialized));
    }

    #[test]
    fn class_element_execution_metadata_requires_a_class_context() {
        let script = lower_script(
            "function ordinary() {} class C { instance = 1; static shared = 2; static {} method() {} }",
        );
        let metas = build_function_metas(&script.functions, &[], &[], &[], &[], 0);
        let meta_named = |name: &str| {
            metas
                .values()
                .find(|meta| meta.name == name)
                .unwrap_or_else(|| panic!("function meta `{name}` should exist"))
        };

        assert_eq!(
            meta_named("C.field.instance").class_element_execution_kind,
            ClassElementExecutionKind::InstanceFieldInitializer
        );
        assert_eq!(
            meta_named("C.field.shared").class_element_execution_kind,
            ClassElementExecutionKind::StaticFieldInitializer
        );
        assert_eq!(
            meta_named("C.<static>").class_element_execution_kind,
            ClassElementExecutionKind::StaticBlock
        );
        assert!(meta_named("C.field.instance").has_class_execution_context());
        assert!(meta_named("C.field.shared").has_class_execution_context());
        assert!(meta_named("C.<static>").has_class_execution_context());
        assert!(meta_named("C.method").has_class_execution_context());
        assert!(meta_named("C").has_class_execution_context());
        assert!(!meta_named("ordinary").has_class_execution_context());
    }

    #[test]
    fn nested_private_functions_use_a_non_class_function_context() {
        let script = lower_script(
            "class Base { get value() { return 1; } }
             class C extends Base {
                 #value = 2;
                 method() {
                     function ordinary(receiver) { return receiver.#value; }
                     function middle() { return () => this.#value; }
                     const arrow = () => super.value + this.#value;
                     return [ordinary, middle, arrow];
                 }
             }",
        );
        let metas = build_function_metas(&script.functions, &[], &[], &[], &[], 0);
        let ordinary = script
            .functions
            .iter()
            .find(|function| function.name == "ordinary")
            .expect("nested ordinary function should be lowered");
        let arrow = script
            .functions
            .iter()
            .find(|function| {
                function.flavor == FunctionFlavor::Arrow && function.captures_private_environment
            })
            .expect("nested arrow function should be lowered");
        let middle = script
            .functions
            .iter()
            .find(|function| function.name == "middle")
            .expect("transitive private-environment function should be lowered");

        for function in [ordinary, middle, arrow] {
            assert!(function.captures_private_environment);
            let meta = &metas[&function.id];
            assert!(!meta.has_class_execution_context());
            assert!(meta.has_function_context());
        }
    }

    #[test]
    fn string_prototype_methods_root_the_constructor_bootstrap() {
        for builtin in [
            StandardBuiltinId::StringPrototypeToLocaleLowerCase,
            StandardBuiltinId::StringPrototypeToLocaleUpperCase,
            StandardBuiltinId::StringPrototypeToLowerCase,
            StandardBuiltinId::StringPrototypeToUpperCase,
            StandardBuiltinId::StringPrototypeSlice,
            StandardBuiltinId::StringPrototypeIncludes,
            StandardBuiltinId::StringPrototypeCharAt,
        ] {
            assert!(builtin.string_prototype_method_name().is_some());
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(builtin);
            assert!(plan.standard_roots.contains(&builtin));
            assert!(plan
                .standard_roots
                .contains(&StandardBuiltinId::StringConstructor));
        }
    }

    #[test]
    fn string_call_result_chain_keeps_bootstrap_and_both_methods_live() {
        let mut plan = RuntimeBootstrapPlan::default();
        for builtin in [
            StandardBuiltinId::StringPrototypeToLocaleLowerCase,
            StandardBuiltinId::StringPrototypeToLocaleUpperCase,
            StandardBuiltinId::StringPrototypeToLowerCase,
            StandardBuiltinId::StringPrototypeToUpperCase,
            StandardBuiltinId::StringPrototypeSlice,
        ] {
            plan.require_standard_builtin(builtin);
        }

        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::StringConstructor));
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::StringPrototypeToUpperCase));
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::StringPrototypeSlice));
    }

    #[test]
    fn slice_method_references_array_and_string_builtins() {
        let key = PropertyKeyIr::StaticString("slice".to_string());
        for builtin in [
            StandardBuiltinId::ArrayPrototypeSlice,
            StandardBuiltinId::StringPrototypeSlice,
        ] {
            assert!(optimized_call_method_references_function(
                &key,
                &builtin.function_id()
            ));
        }
    }

    #[test]
    fn sort_method_references_and_roots_the_array_builtin() {
        let key = PropertyKeyIr::StaticString("sort".to_string());
        assert!(optimized_call_method_references_function(
            &key,
            &StandardBuiltinId::ArrayPrototypeSort.function_id()
        ));

        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::ArrayPrototypeSort);
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::ArrayPrototypeSort));
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::ArrayConstructor));
    }

    #[test]
    fn dynamic_symbol_string_and_value_methods_are_referenced_and_bootstrapped() {
        for (name, builtin) in [
            ("toString", StandardBuiltinId::SymbolPrototypeToString),
            ("valueOf", StandardBuiltinId::SymbolPrototypeValueOf),
        ] {
            let key = PropertyKeyIr::StaticString(name.to_string());
            assert!(optimized_call_method_references_function(
                &key,
                &builtin.function_id()
            ));

            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(builtin);
            assert!(plan
                .standard_roots
                .contains(&StandardBuiltinId::SymbolConstructor));
        }
    }

    #[test]
    fn nested_dynamic_array_method_get_roots_materialized_builtin() {
        let script = lower_script(
            "var values = []; var key = {}; key[Symbol.toPrimitive] = function (hint) { values.push(hint); return 0; };",
        );
        assert!(script_references_standard_builtin(
            &script,
            StandardBuiltinId::ArrayPrototypePush
        ));
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::ArrayPrototypePush);
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::ArrayConstructor));
    }

    #[test]
    fn iterator_prototype_members_root_the_constructor_bootstrap() {
        for builtin in [
            StandardBuiltinId::IteratorFrom,
            StandardBuiltinId::ArrayIteratorIdentity,
            StandardBuiltinId::IteratorPrototypeMap,
            StandardBuiltinId::IteratorPrototypeDrop,
            StandardBuiltinId::IteratorPrototypeSymbolDispose,
            StandardBuiltinId::IteratorPrototypeToStringTagGetter,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(builtin);
            assert!(plan.standard_roots.contains(&builtin));
            assert!(plan
                .standard_roots
                .contains(&StandardBuiltinId::IteratorConstructor));
        }
    }

    #[test]
    fn map_prototype_members_root_the_map_constructor_bootstrap() {
        for builtin in [
            StandardBuiltinId::MapGroupBy,
            StandardBuiltinId::MapPrototypeClear,
            StandardBuiltinId::MapPrototypeDelete,
            StandardBuiltinId::MapPrototypeForEach,
            StandardBuiltinId::MapPrototypeKeys,
            StandardBuiltinId::MapPrototypeValues,
            StandardBuiltinId::MapPrototypeEntries,
            StandardBuiltinId::MapIteratorNext,
            StandardBuiltinId::MapPrototypeGet,
            StandardBuiltinId::MapPrototypeGetOrInsert,
            StandardBuiltinId::MapPrototypeGetOrInsertComputed,
            StandardBuiltinId::MapPrototypeHas,
            StandardBuiltinId::MapPrototypeSet,
            StandardBuiltinId::MapPrototypeSizeGetter,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(builtin);
            assert!(plan.standard_roots.contains(&builtin));
            assert!(plan
                .standard_roots
                .contains(&StandardBuiltinId::MapConstructor));
        }
    }

    #[test]
    fn set_prototype_members_root_the_set_constructor_bootstrap() {
        for builtin in [
            StandardBuiltinId::SetPrototypeAdd,
            StandardBuiltinId::SetPrototypeClear,
            StandardBuiltinId::SetPrototypeDelete,
            StandardBuiltinId::SetPrototypeDifference,
            StandardBuiltinId::SetPrototypeForEach,
            StandardBuiltinId::SetPrototypeIntersection,
            StandardBuiltinId::SetPrototypeIsDisjointFrom,
            StandardBuiltinId::SetPrototypeIsSubsetOf,
            StandardBuiltinId::SetPrototypeIsSupersetOf,
            StandardBuiltinId::SetPrototypeSymmetricDifference,
            StandardBuiltinId::SetPrototypeUnion,
            StandardBuiltinId::SetPrototypeValues,
            StandardBuiltinId::SetPrototypeEntries,
            StandardBuiltinId::SetIteratorNext,
            StandardBuiltinId::SetPrototypeHas,
            StandardBuiltinId::SetPrototypeSizeGetter,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(builtin);
            assert!(plan.standard_roots.contains(&builtin));
            assert!(plan
                .standard_roots
                .contains(&StandardBuiltinId::SetConstructor));
        }
    }

    #[test]
    fn set_constructor_roots_sync_iterator_machinery() {
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::SetConstructor);

        for builtin in [
            StandardBuiltinId::ArrayPrototypeValues,
            StandardBuiltinId::ArrayIteratorNext,
            StandardBuiltinId::ArrayIteratorIdentity,
            StandardBuiltinId::StringPrototypeIterator,
            StandardBuiltinId::StringIteratorNext,
        ] {
            assert!(plan.standard_roots.contains(&builtin));
        }
    }

    #[test]
    fn map_constructor_roots_sync_iterator_machinery() {
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::MapConstructor);

        assert!(plan.standard_roots.contains(&StandardBuiltinId::MapGroupBy));
        for builtin in [
            StandardBuiltinId::ArrayPrototypeValues,
            StandardBuiltinId::ArrayIteratorNext,
            StandardBuiltinId::ArrayIteratorIdentity,
            StandardBuiltinId::StringPrototypeIterator,
            StandardBuiltinId::StringIteratorNext,
        ] {
            assert!(plan.standard_roots.contains(&builtin));
        }
    }

    #[test]
    fn typed_array_constructors_root_array_iterator_machinery() {
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::Uint8ArrayConstructor);

        for builtin in [
            StandardBuiltinId::ArrayPrototypeValues,
            StandardBuiltinId::TypedArrayPrototypeIncludes,
            StandardBuiltinId::TypedArrayPrototypeIndexOf,
            StandardBuiltinId::TypedArrayPrototypeLastIndexOf,
            StandardBuiltinId::TypedArrayPrototypeFind,
            StandardBuiltinId::TypedArrayPrototypeFindIndex,
            StandardBuiltinId::TypedArrayPrototypeFindLast,
            StandardBuiltinId::TypedArrayPrototypeFindLastIndex,
            StandardBuiltinId::TypedArrayPrototypeEvery,
            StandardBuiltinId::TypedArrayPrototypeSome,
            StandardBuiltinId::TypedArrayPrototypeMap,
            StandardBuiltinId::TypedArrayPrototypeFilter,
            StandardBuiltinId::TypedArrayPrototypeForEach,
            StandardBuiltinId::TypedArrayPrototypeReduce,
            StandardBuiltinId::TypedArrayPrototypeReduceRight,
            StandardBuiltinId::TypedArrayPrototypeValues,
            StandardBuiltinId::TypedArrayPrototypeKeys,
            StandardBuiltinId::TypedArrayPrototypeEntries,
            StandardBuiltinId::ArrayIteratorNext,
            StandardBuiltinId::ArrayIteratorIdentity,
        ] {
            assert!(plan.standard_roots.contains(&builtin));
        }
    }

    #[test]
    fn object_constructor_roots_group_by_and_property_key_bootstrap() {
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::ObjectConstructor);

        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::ObjectGroupBy));
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::SymbolPrototypeToPrimitive));
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::SymbolConstructor));
        for builtin in [
            StandardBuiltinId::ArrayPrototypeValues,
            StandardBuiltinId::ArrayIteratorNext,
            StandardBuiltinId::ArrayIteratorIdentity,
            StandardBuiltinId::StringPrototypeIterator,
            StandardBuiltinId::StringIteratorNext,
        ] {
            assert!(plan.standard_roots.contains(&builtin));
        }
    }

    #[test]
    fn object_constructor_roots_proto_accessor_pair() {
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::ObjectConstructor);

        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::ObjectPrototypeProtoGetter));
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::ObjectPrototypeProtoSetter));
    }

    #[test]
    fn object_locale_string_roots_primitive_to_string_methods() {
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::ObjectPrototypeToLocaleString);

        for builtin in [
            StandardBuiltinId::StringPrototypeToString,
            StandardBuiltinId::NumberPrototypeToString,
            StandardBuiltinId::BooleanPrototypeToString,
            StandardBuiltinId::SymbolPrototypeToString,
            StandardBuiltinId::BigIntPrototypeToString,
        ] {
            assert!(plan.standard_roots.contains(&builtin));
        }
    }

    #[test]
    fn descriptor_entry_points_root_generic_descriptor_lookup() {
        for builtin in [
            StandardBuiltinId::ObjectDefineProperty,
            StandardBuiltinId::ReflectDefineProperty,
            StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
            StandardBuiltinId::ObjectPrototypeLookupGetter,
            StandardBuiltinId::ObjectPrototypeLookupSetter,
            StandardBuiltinId::ObjectPrototypePropertyIsEnumerable,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(builtin);

            assert!(plan.standard_roots.contains(&builtin));
            assert!(plan
                .standard_roots
                .contains(&StandardBuiltinId::ObjectGetOwnPropertyDescriptor));
        }
    }

    #[test]
    fn reflected_call_entry_points_root_proxy_dispatch() {
        for builtin in [
            StandardBuiltinId::ReflectApply,
            StandardBuiltinId::ReflectConstruct,
            StandardBuiltinId::FunctionPrototypeApply,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(builtin);

            assert!(plan.standard_roots.contains(&builtin));
            assert!(plan
                .standard_roots
                .contains(&StandardBuiltinId::ProxyConstructor));
        }
    }

    #[test]
    fn object_has_own_roots_generic_descriptor_lookup() {
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::ObjectHasOwn);

        for builtin in [
            StandardBuiltinId::ObjectHasOwn,
            StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
            StandardBuiltinId::ObjectGetOwnPropertyDescriptor,
        ] {
            assert!(plan.standard_roots.contains(&builtin));
        }
    }

    #[test]
    fn object_descriptor_map_builtins_root_reflective_operations() {
        for entry_point in [
            StandardBuiltinId::ObjectCreate,
            StandardBuiltinId::ObjectDefineProperties,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(entry_point);

            for builtin in [
                entry_point,
                StandardBuiltinId::ObjectDefineProperties,
                StandardBuiltinId::ObjectDefineProperty,
                StandardBuiltinId::ReflectOwnKeys,
                StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
                StandardBuiltinId::ReflectDefineProperty,
            ] {
                assert!(plan.standard_roots.contains(&builtin));
            }
        }
    }

    #[test]
    fn object_integrity_builtins_root_reflective_property_operations() {
        for (integrity_builtin, dependencies) in [
            (
                StandardBuiltinId::ObjectSeal,
                &[
                    StandardBuiltinId::ReflectOwnKeys,
                    StandardBuiltinId::ReflectDefineProperty,
                    StandardBuiltinId::ObjectGetOwnPropertyDescriptor,
                ][..],
            ),
            (
                StandardBuiltinId::ObjectFreeze,
                &[
                    StandardBuiltinId::ReflectOwnKeys,
                    StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
                    StandardBuiltinId::ReflectDefineProperty,
                    StandardBuiltinId::ObjectGetOwnPropertyDescriptor,
                ][..],
            ),
            (
                StandardBuiltinId::ObjectIsSealed,
                &[
                    StandardBuiltinId::ReflectOwnKeys,
                    StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
                    StandardBuiltinId::ObjectGetOwnPropertyDescriptor,
                ][..],
            ),
            (
                StandardBuiltinId::ObjectIsFrozen,
                &[
                    StandardBuiltinId::ReflectOwnKeys,
                    StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
                    StandardBuiltinId::ObjectGetOwnPropertyDescriptor,
                ][..],
            ),
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(integrity_builtin);

            assert!(plan.standard_roots.contains(&integrity_builtin));
            for dependency in dependencies {
                assert!(plan.standard_roots.contains(dependency));
            }
        }
    }

    #[test]
    fn object_entries_and_values_root_reflective_property_operations() {
        for enumerable_own_properties_builtin in [
            StandardBuiltinId::ObjectEntries,
            StandardBuiltinId::ObjectValues,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(enumerable_own_properties_builtin);

            for builtin in [
                enumerable_own_properties_builtin,
                StandardBuiltinId::ReflectOwnKeys,
                StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
                StandardBuiltinId::ObjectGetOwnPropertyDescriptor,
            ] {
                assert!(plan.standard_roots.contains(&builtin));
            }
        }
    }

    #[test]
    fn object_property_copy_builtins_root_reflective_property_operations() {
        for builtin in [
            StandardBuiltinId::ObjectAssign,
            StandardBuiltinId::ObjectGetOwnPropertyDescriptors,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(builtin);

            for dependency in [
                builtin,
                StandardBuiltinId::ReflectOwnKeys,
                StandardBuiltinId::ReflectGetOwnPropertyDescriptor,
                StandardBuiltinId::ObjectGetOwnPropertyDescriptor,
            ] {
                assert!(plan.standard_roots.contains(&dependency));
            }

            if builtin == StandardBuiltinId::ObjectAssign {
                assert!(plan.standard_roots.contains(&StandardBuiltinId::ReflectSet));
            }
        }
    }

    #[test]
    fn regexp_literal_roots_intrinsic_regexp_bootstrap() {
        let script = lower_script("/a/;");
        let StatementIr::Expression(literal) = &script.body.statements[0] else {
            panic!("literal script should lower to an expression statement");
        };

        assert!(script_references_standard_builtin(
            &script,
            StandardBuiltinId::RegExpConstructor
        ));
        assert!(!should_stub_standard_builtin(
            &script,
            StandardBuiltinId::RegExpConstructor
        ));
        assert!(!expr_uses_calls(literal));
        assert!(!expr_uses_function_table(literal));
    }

    #[test]
    fn wasm_aot_harness_realm_fields_are_the_full_global_bootstrap_roots() {
        let create_realm = lower_script(
            "var $262 = { createRealm: function () { return __porfCreateRealm(); } };",
        );
        assert!(script_uses_create_realm(&create_realm));
        assert!(!script_exposes_global_object(&create_realm));
        assert!(RuntimeBootstrapPlan::from_script(&create_realm, &[]).full_standard_globals);

        let exposed_global = lower_script("var $262 = { global: globalThis };");
        assert!(!script_uses_create_realm(&exposed_global));
        assert!(script_exposes_global_object(&exposed_global));
        assert!(RuntimeBootstrapPlan::from_script(&exposed_global, &[]).full_standard_globals);

        let inactive_realm = lower_script(
            "var $262 = { global: undefined, createRealm: function () { throw 'inactive'; } };",
        );
        assert!(!script_uses_create_realm(&inactive_realm));
        assert!(!script_exposes_global_object(&inactive_realm));
        assert!(!RuntimeBootstrapPlan::from_script(&inactive_realm, &[]).full_standard_globals);
    }

    #[test]
    fn top_level_this_requires_the_full_global_bootstrap() {
        let script = lower_script(r#"Object.getOwnPropertyDescriptor(this, "BigInt64Array");"#);

        assert_eq!(script.top_level_this_uses, 1);
        assert!(RuntimeBootstrapPlan::from_script(&script, &[]).full_standard_globals);
    }
}

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeBootstrapPlan {
    pub(crate) full_standard_globals: bool,
    pub(crate) standard_roots: BTreeSet<StandardBuiltinId>,
    pub(crate) reflect_object: bool,
    pub(crate) math_object: bool,
    pub(crate) json_object: bool,
    pub(crate) atomics_object: bool,
    pub(crate) temporal_object: bool,
    pub(crate) intl_object: bool,
}

impl RuntimeBootstrapPlan {
    pub(crate) fn from_script(
        script: &ScriptIr,
        compiled_standard_builtins: &[StandardBuiltinId],
    ) -> Self {
        let mut plan = Self::default();
        // Script-level `this` is the global object and can flow through calls
        // whose reflected property names are not visible to static planning.
        plan.full_standard_globals = script.top_level_this_uses > 0
            || script_uses_create_realm(script)
            || script_exposes_global_object(script);
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
        if script
            .functions
            .iter()
            .any(|function| function.execution_kind == FunctionExecutionKind::Async)
        {
            plan.require_standard_builtin(StandardBuiltinId::PromiseConstructor);
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
            ScriptGlobalBindingKind::TemporalObject => {
                self.full_standard_globals || self.temporal_object
            }
            ScriptGlobalBindingKind::IntlObject => self.full_standard_globals || self.intl_object,
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
            ScriptGlobalBindingKind::TemporalObject => {
                self.temporal_object = true;
                self.require_standard_builtin(StandardBuiltinId::TemporalInstantConstructor);
            }
            ScriptGlobalBindingKind::IntlObject => {
                self.intl_object = true;
                self.require_standard_builtin(StandardBuiltinId::IntlLocaleConstructor);
                self.require_standard_builtin(StandardBuiltinId::IntlGetCanonicalLocales);
            }
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
            self.require_standard_builtin(builtin);
        }
    }

    fn require_standard_builtin(&mut self, builtin: StandardBuiltinId) {
        self.standard_roots.insert(builtin);
        if builtin == StandardBuiltinId::ArrayFromAsync {
            self.standard_roots
                .insert(StandardBuiltinId::ArrayConstructor);
            for dependency in [
                StandardBuiltinId::ArrayFromAsyncFulfilled,
                StandardBuiltinId::ArrayFromAsyncRejected,
                StandardBuiltinId::PromiseResolve,
            ] {
                self.require_standard_builtin(dependency);
            }
        }
        if builtin == StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose {
            for dependency in [
                StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeFulfilled,
                StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeRejected,
            ] {
                self.require_standard_builtin(dependency);
            }
        }
        if is_typed_array_constructor(builtin) {
            for iterator_builtin in [
                StandardBuiltinId::ArrayPrototypeValues,
                StandardBuiltinId::TypedArrayPrototypeIncludes,
                StandardBuiltinId::TypedArrayPrototypeIndexOf,
                StandardBuiltinId::TypedArrayPrototypeLastIndexOf,
                StandardBuiltinId::TypedArrayPrototypeFind,
                StandardBuiltinId::TypedArrayPrototypeFindIndex,
                StandardBuiltinId::TypedArrayPrototypeFindLast,
                StandardBuiltinId::TypedArrayPrototypeFindLastIndex,
                StandardBuiltinId::TypedArrayPrototypeEvery,
                StandardBuiltinId::TypedArrayPrototypeSome,
                StandardBuiltinId::TypedArrayPrototypeMap,
                StandardBuiltinId::TypedArrayPrototypeFilter,
                StandardBuiltinId::TypedArrayPrototypeForEach,
                StandardBuiltinId::TypedArrayPrototypeReduce,
                StandardBuiltinId::TypedArrayPrototypeReduceRight,
                StandardBuiltinId::TypedArrayPrototypeValues,
                StandardBuiltinId::TypedArrayPrototypeKeys,
                StandardBuiltinId::TypedArrayPrototypeEntries,
                StandardBuiltinId::ArrayIteratorNext,
                StandardBuiltinId::ArrayIteratorIdentity,
            ] {
                self.require_standard_builtin(iterator_builtin);
            }
        }
        if builtin == StandardBuiltinId::ObjectConstructor {
            self.require_standard_builtin(StandardBuiltinId::ObjectGroupBy);
            self.require_standard_builtin(StandardBuiltinId::ObjectFromEntries);
            self.require_standard_builtin(StandardBuiltinId::ObjectAssign);
            self.require_standard_builtin(StandardBuiltinId::ObjectGetOwnPropertyDescriptors);
        }
        if matches!(
            builtin,
            StandardBuiltinId::MapConstructor
                | StandardBuiltinId::WeakMapConstructor
                | StandardBuiltinId::WeakSetConstructor
                | StandardBuiltinId::SetConstructor
                | StandardBuiltinId::ObjectGroupBy
                | StandardBuiltinId::ObjectFromEntries
        ) {
            if builtin == StandardBuiltinId::MapConstructor {
                self.require_standard_builtin(StandardBuiltinId::MapGroupBy);
                self.require_standard_builtin(StandardBuiltinId::MapSpeciesGetter);
            }
            if builtin == StandardBuiltinId::SetConstructor {
                self.require_standard_builtin(StandardBuiltinId::SetSpeciesGetter);
            }
            for iterator_builtin in [
                StandardBuiltinId::ArrayPrototypeValues,
                StandardBuiltinId::ArrayIteratorNext,
                StandardBuiltinId::ArrayIteratorIdentity,
                StandardBuiltinId::StringPrototypeIterator,
                StandardBuiltinId::StringIteratorNext,
            ] {
                self.require_standard_builtin(iterator_builtin);
            }
        }
        if builtin.string_prototype_method_name().is_some()
            || matches!(
                builtin,
                StandardBuiltinId::StringFromCharCode
                    | StandardBuiltinId::StringFromCodePoint
                    | StandardBuiltinId::StringRaw
            )
        {
            // String.prototype methods are installed by the String
            // constructor bootstrap block. Dynamic method calls can root the
            // method body without otherwise rooting that installer.
            self.standard_roots
                .insert(StandardBuiltinId::StringConstructor);
        }
        if matches!(
            builtin,
            StandardBuiltinId::ArrayPrototypeConcat
                | StandardBuiltinId::ArrayPrototypeJoin
                | StandardBuiltinId::ArrayPrototypeSlice
                | StandardBuiltinId::ArrayPrototypeSplice
                | StandardBuiltinId::ArrayPrototypeFill
                | StandardBuiltinId::ArrayPrototypeSort
                | StandardBuiltinId::TypedArrayPrototypeToString
                | StandardBuiltinId::ArrayPrototypeToLocaleString
                | StandardBuiltinId::ArrayPrototypeFlat
                | StandardBuiltinId::ArrayPrototypeFlatMap
                | StandardBuiltinId::ArrayPrototypeAt
                | StandardBuiltinId::TypedArrayPrototypeAt
                | StandardBuiltinId::TypedArrayPrototypeCopyWithin
                | StandardBuiltinId::TypedArrayPrototypeIncludes
                | StandardBuiltinId::TypedArrayPrototypeIndexOf
                | StandardBuiltinId::TypedArrayPrototypeLastIndexOf
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
                | StandardBuiltinId::TypedArrayPrototypeValues
                | StandardBuiltinId::TypedArrayPrototypeKeys
                | StandardBuiltinId::TypedArrayPrototypeEntries
                | StandardBuiltinId::ArrayPrototypeToReversed
                | StandardBuiltinId::ArrayPrototypeToSpliced
                | StandardBuiltinId::ArrayPrototypeToSorted
                | StandardBuiltinId::ArrayPrototypeWith
                | StandardBuiltinId::ArrayPrototypeReverse
                | StandardBuiltinId::ArrayPrototypeCopyWithin
                | StandardBuiltinId::ArrayPrototypeIncludes
                | StandardBuiltinId::ArrayPrototypeIndexOf
                | StandardBuiltinId::ArrayPrototypeLastIndexOf
                | StandardBuiltinId::ArrayPrototypeFind
                | StandardBuiltinId::ArrayPrototypeFindIndex
                | StandardBuiltinId::ArrayPrototypeFindLast
                | StandardBuiltinId::ArrayPrototypeFindLastIndex
                | StandardBuiltinId::ArrayPrototypeEvery
                | StandardBuiltinId::ArrayPrototypeSome
                | StandardBuiltinId::ArrayPrototypeForEach
                | StandardBuiltinId::ArrayPrototypeFilter
                | StandardBuiltinId::ArrayPrototypeMap
                | StandardBuiltinId::ArrayPrototypeReduce
                | StandardBuiltinId::ArrayPrototypeReduceRight
                | StandardBuiltinId::ArrayPrototypePop
                | StandardBuiltinId::ArrayPrototypePush
                | StandardBuiltinId::ArrayPrototypeShift
                | StandardBuiltinId::ArrayPrototypeUnshift
                | StandardBuiltinId::ArrayPrototypeKeys
                | StandardBuiltinId::ArrayPrototypeEntries
                | StandardBuiltinId::ArrayPrototypeValues
        ) {
            // These properties are all installed by the Array constructor's
            // bootstrap block. A dynamic GetV can make a method body reachable
            // without otherwise referencing `Array`, so root the installer as
            // well as the body.
            self.standard_roots
                .insert(StandardBuiltinId::ArrayConstructor);
        }
        if matches!(
            builtin,
            StandardBuiltinId::ArrayPrototypeSlice | StandardBuiltinId::ArrayPrototypeSplice
        ) {
            self.require_standard_builtin(StandardBuiltinId::ObjectDefineProperty);
        }
        if builtin == StandardBuiltinId::IteratorZipKeyed {
            self.require_standard_builtin(StandardBuiltinId::ReflectOwnKeys);
            self.require_standard_builtin(StandardBuiltinId::ReflectGetOwnPropertyDescriptor);
        }
        if matches!(
            builtin,
            StandardBuiltinId::ObjectAssign
                | StandardBuiltinId::ObjectEntries
                | StandardBuiltinId::ObjectGetOwnPropertyDescriptors
                | StandardBuiltinId::ObjectValues
        ) {
            self.require_standard_builtin(StandardBuiltinId::ReflectOwnKeys);
            self.require_standard_builtin(StandardBuiltinId::ReflectGetOwnPropertyDescriptor);
        }
        if builtin == StandardBuiltinId::ObjectAssign {
            self.require_standard_builtin(StandardBuiltinId::ReflectSet);
        }
        if builtin == StandardBuiltinId::ObjectHasOwn {
            self.require_standard_builtin(StandardBuiltinId::ReflectGetOwnPropertyDescriptor);
        }
        if matches!(
            builtin,
            StandardBuiltinId::ReflectApply
                | StandardBuiltinId::ReflectConstruct
                | StandardBuiltinId::FunctionPrototypeApply
        ) {
            self.require_standard_builtin(StandardBuiltinId::ProxyConstructor);
        }
        if builtin == StandardBuiltinId::ObjectCreate {
            self.require_standard_builtin(StandardBuiltinId::ObjectDefineProperties);
        }
        if builtin == StandardBuiltinId::ObjectDefineProperties {
            self.require_standard_builtin(StandardBuiltinId::ObjectDefineProperty);
            self.require_standard_builtin(StandardBuiltinId::ReflectOwnKeys);
            self.require_standard_builtin(StandardBuiltinId::ReflectGetOwnPropertyDescriptor);
            self.require_standard_builtin(StandardBuiltinId::ReflectDefineProperty);
        }
        if matches!(
            builtin,
            StandardBuiltinId::ObjectSeal
                | StandardBuiltinId::ObjectFreeze
                | StandardBuiltinId::ObjectIsSealed
                | StandardBuiltinId::ObjectIsFrozen
        ) {
            self.require_standard_builtin(StandardBuiltinId::ReflectOwnKeys);
        }
        if matches!(
            builtin,
            StandardBuiltinId::ObjectSeal | StandardBuiltinId::ObjectFreeze
        ) {
            self.require_standard_builtin(StandardBuiltinId::ReflectDefineProperty);
        }
        if matches!(
            builtin,
            StandardBuiltinId::ObjectFreeze
                | StandardBuiltinId::ObjectIsSealed
                | StandardBuiltinId::ObjectIsFrozen
        ) {
            self.require_standard_builtin(StandardBuiltinId::ReflectGetOwnPropertyDescriptor);
        }
        if builtin == StandardBuiltinId::DatePrototypeToTemporalInstant {
            self.require_standard_builtin(StandardBuiltinId::TemporalInstantConstructor);
        }
        if matches!(
            builtin,
            StandardBuiltinId::ObjectDefineProperty
                | StandardBuiltinId::ReflectDefineProperty
                | StandardBuiltinId::ReflectGetOwnPropertyDescriptor
                | StandardBuiltinId::ObjectPrototypeLookupGetter
                | StandardBuiltinId::ObjectPrototypeLookupSetter
                | StandardBuiltinId::ObjectPrototypePropertyIsEnumerable
        ) {
            // These entry points dispatch through the generic descriptor
            // builtin so exotic and nested-Proxy [[GetOwnProperty]] semantics
            // are observed exactly once. Keep its real body rooted rather than
            // calling a lazy stub.
            self.require_standard_builtin(StandardBuiltinId::ObjectGetOwnPropertyDescriptor);
        }
        match builtin {
            StandardBuiltinId::ObjectGroupBy | StandardBuiltinId::ObjectFromEntries => {
                self.standard_roots
                    .insert(StandardBuiltinId::ObjectConstructor);
            }
            StandardBuiltinId::ObjectConstructor => {
                // Object boxing can expose a Symbol wrapper to ToPrimitive.
                // Keep the real Symbol.prototype @@toPrimitive installed so
                // the default hook (and any own-property override) remains
                // observable through dynamic lookup.
                self.require_standard_builtin(StandardBuiltinId::SymbolPrototypeToPrimitive);
                self.require_standard_builtin(StandardBuiltinId::ObjectPrototypeProtoGetter);
                self.require_standard_builtin(StandardBuiltinId::ObjectPrototypeProtoSetter);
            }
            StandardBuiltinId::ObjectPrototypeToLocaleString => {
                // Invoke(this, "toString") can resolve through any primitive
                // prototype without a statically visible method call.
                for builtin in [
                    StandardBuiltinId::StringPrototypeToString,
                    StandardBuiltinId::NumberPrototypeToString,
                    StandardBuiltinId::BooleanPrototypeToString,
                    StandardBuiltinId::SymbolPrototypeToString,
                    StandardBuiltinId::BigIntPrototypeToString,
                ] {
                    self.require_standard_builtin(builtin);
                }
            }
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
            | StandardBuiltinId::AtomicsXor
            | StandardBuiltinId::AtomicsIsLockFree => self.atomics_object = true,
            StandardBuiltinId::AtomicsWaitAsync => {
                self.atomics_object = true;
                self.standard_roots
                    .insert(StandardBuiltinId::PromiseConstructor);
                self.standard_roots
                    .insert(StandardBuiltinId::PromiseSpeciesGetter);
            }
            StandardBuiltinId::IntlGetCanonicalLocales
            | StandardBuiltinId::IntlLocaleConstructor
            | StandardBuiltinId::IntlLocalePrototypeLanguageGetter
            | StandardBuiltinId::IntlLocalePrototypeScriptGetter
            | StandardBuiltinId::IntlLocalePrototypeRegionGetter
            | StandardBuiltinId::IntlLocalePrototypeBaseNameGetter
            | StandardBuiltinId::IntlLocalePrototypeToString => {
                self.intl_object = true;
                for dependency in [
                    StandardBuiltinId::IntlGetCanonicalLocales,
                    StandardBuiltinId::IntlLocaleConstructor,
                    StandardBuiltinId::IntlLocalePrototypeLanguageGetter,
                    StandardBuiltinId::IntlLocalePrototypeScriptGetter,
                    StandardBuiltinId::IntlLocalePrototypeRegionGetter,
                    StandardBuiltinId::IntlLocalePrototypeBaseNameGetter,
                    StandardBuiltinId::IntlLocalePrototypeToString,
                ] {
                    self.standard_roots.insert(dependency);
                }
            }
            StandardBuiltinId::TemporalInstantConstructor
            | StandardBuiltinId::TemporalInstantFrom
            | StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter
            | StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter
            | StandardBuiltinId::TemporalInstantPrototypeEquals
            | StandardBuiltinId::TemporalInstantPrototypeToString => {
                self.temporal_object = true;
                self.standard_roots
                    .insert(StandardBuiltinId::TemporalInstantConstructor);
                self.standard_roots
                    .insert(StandardBuiltinId::TemporalInstantFrom);
                self.standard_roots
                    .insert(StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter);
                self.standard_roots
                    .insert(StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter);
                self.standard_roots
                    .insert(StandardBuiltinId::TemporalInstantPrototypeEquals);
                self.standard_roots
                    .insert(StandardBuiltinId::TemporalInstantPrototypeToString);
            }
            StandardBuiltinId::TemporalZonedDateTimeConstructor
            | StandardBuiltinId::TemporalZonedDateTimeFrom
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeYearGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeDayGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeHourGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeNanosecondGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEquals
            | StandardBuiltinId::TemporalZonedDateTimePrototypeToInstant
            | StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone => {
                self.temporal_object = true;
                for dependency in [
                    StandardBuiltinId::TemporalInstantConstructor,
                    StandardBuiltinId::TemporalZonedDateTimeConstructor,
                    StandardBuiltinId::TemporalZonedDateTimeFrom,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeYearGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeMonthGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeDayGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeHourGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeMinuteGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeSecondGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeMillisecondGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeMicrosecondGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeNanosecondGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeEquals,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeToInstant,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone,
                ] {
                    self.standard_roots.insert(dependency);
                }
            }
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
            StandardBuiltinId::BooleanPrototypeToString
            | StandardBuiltinId::BooleanPrototypeValueOf => {
                self.standard_roots
                    .insert(StandardBuiltinId::BooleanConstructor);
            }
            StandardBuiltinId::PromiseConstructor
            | StandardBuiltinId::PromisePrototypeThen
            | StandardBuiltinId::PromisePrototypeCatch
            | StandardBuiltinId::PromisePrototypeFinally
            | StandardBuiltinId::PromiseThenFinally
            | StandardBuiltinId::PromiseCatchFinally
            | StandardBuiltinId::PromiseValueThunk
            | StandardBuiltinId::PromiseThrower
            | StandardBuiltinId::PromiseSpeciesGetter
            | StandardBuiltinId::PromiseResolve
            | StandardBuiltinId::PromiseWithResolvers
            | StandardBuiltinId::PromiseTry
            | StandardBuiltinId::PromiseReject
            | StandardBuiltinId::PromiseAll
            | StandardBuiltinId::PromiseAllSettled
            | StandardBuiltinId::PromiseAllKeyed
            | StandardBuiltinId::PromiseAllSettledKeyed
            | StandardBuiltinId::PromiseAny
            | StandardBuiltinId::PromiseRace
            | StandardBuiltinId::ArrayFromAsync
            | StandardBuiltinId::AsyncGeneratorPrototypeNext
            | StandardBuiltinId::AsyncGeneratorPrototypeReturn
            | StandardBuiltinId::AsyncGeneratorPrototypeThrow
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose => {
                self.standard_roots
                    .insert(StandardBuiltinId::PromiseConstructor);
                self.standard_roots
                    .insert(StandardBuiltinId::PromiseSpeciesGetter);
                if matches!(
                    builtin,
                    StandardBuiltinId::PromisePrototypeThen
                        | StandardBuiltinId::PromisePrototypeFinally
                        | StandardBuiltinId::PromiseThenFinally
                        | StandardBuiltinId::PromiseCatchFinally
                        | StandardBuiltinId::PromiseResolve
                        | StandardBuiltinId::PromiseWithResolvers
                        | StandardBuiltinId::PromiseTry
                        | StandardBuiltinId::PromiseReject
                        | StandardBuiltinId::PromiseAll
                        | StandardBuiltinId::PromiseAllSettled
                        | StandardBuiltinId::PromiseAllKeyed
                        | StandardBuiltinId::PromiseAllSettledKeyed
                        | StandardBuiltinId::PromiseAny
                        | StandardBuiltinId::PromiseRace
                        | StandardBuiltinId::ArrayFromAsync
                        | StandardBuiltinId::AsyncGeneratorPrototypeNext
                        | StandardBuiltinId::AsyncGeneratorPrototypeReturn
                        | StandardBuiltinId::AsyncGeneratorPrototypeThrow
                        | StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose
                ) {
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseCapabilityExecutor);
                }
                if builtin == StandardBuiltinId::PromiseAll {
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseAllResolveElement);
                }
                if builtin == StandardBuiltinId::PromiseAllSettled {
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseAllSettledResolveElement);
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseAllSettledRejectElement);
                }
                if builtin == StandardBuiltinId::PromiseAny {
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseAnyRejectElement);
                    self.standard_roots
                        .insert(StandardBuiltinId::AggregateErrorConstructor);
                }
                if builtin == StandardBuiltinId::PromiseAllKeyed {
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseAllKeyedResolveElement);
                    self.standard_roots
                        .insert(StandardBuiltinId::ReflectOwnKeys);
                    self.standard_roots
                        .insert(StandardBuiltinId::ReflectGetOwnPropertyDescriptor);
                }
                if builtin == StandardBuiltinId::PromiseAllSettledKeyed {
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseAllSettledKeyedResolveElement);
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseAllSettledKeyedRejectElement);
                    self.standard_roots
                        .insert(StandardBuiltinId::ReflectOwnKeys);
                    self.standard_roots
                        .insert(StandardBuiltinId::ReflectGetOwnPropertyDescriptor);
                }
                if matches!(
                    builtin,
                    StandardBuiltinId::PromisePrototypeFinally
                        | StandardBuiltinId::PromiseThenFinally
                        | StandardBuiltinId::PromiseCatchFinally
                ) {
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseThenFinally);
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseCatchFinally);
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseValueThunk);
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseThrower);
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseResolve);
                }
            }
            StandardBuiltinId::MapSpeciesGetter
            | StandardBuiltinId::MapGroupBy
            | StandardBuiltinId::MapPrototypeClear
            | StandardBuiltinId::MapPrototypeDelete
            | StandardBuiltinId::MapPrototypeForEach
            | StandardBuiltinId::MapPrototypeKeys
            | StandardBuiltinId::MapPrototypeValues
            | StandardBuiltinId::MapPrototypeEntries
            | StandardBuiltinId::MapIteratorNext
            | StandardBuiltinId::MapPrototypeGet
            | StandardBuiltinId::MapPrototypeGetOrInsert
            | StandardBuiltinId::MapPrototypeGetOrInsertComputed
            | StandardBuiltinId::MapPrototypeHas
            | StandardBuiltinId::MapPrototypeSet
            | StandardBuiltinId::MapPrototypeSizeGetter => {
                self.standard_roots
                    .insert(StandardBuiltinId::MapConstructor);
            }
            StandardBuiltinId::WeakMapPrototypeDelete
            | StandardBuiltinId::WeakMapPrototypeGet
            | StandardBuiltinId::WeakMapPrototypeGetOrInsert
            | StandardBuiltinId::WeakMapPrototypeGetOrInsertComputed
            | StandardBuiltinId::WeakMapPrototypeHas
            | StandardBuiltinId::WeakMapPrototypeSet => {
                self.standard_roots
                    .insert(StandardBuiltinId::WeakMapConstructor);
            }
            StandardBuiltinId::WeakSetPrototypeAdd
            | StandardBuiltinId::WeakSetPrototypeDelete
            | StandardBuiltinId::WeakSetPrototypeHas => {
                self.standard_roots
                    .insert(StandardBuiltinId::WeakSetConstructor);
            }
            StandardBuiltinId::WeakRefPrototypeDeref => {
                self.standard_roots
                    .insert(StandardBuiltinId::WeakRefConstructor);
            }
            StandardBuiltinId::FinalizationRegistryPrototypeRegister
            | StandardBuiltinId::FinalizationRegistryPrototypeUnregister => {
                self.standard_roots
                    .insert(StandardBuiltinId::FinalizationRegistryConstructor);
            }
            StandardBuiltinId::SetSpeciesGetter
            | StandardBuiltinId::SetPrototypeAdd
            | StandardBuiltinId::SetPrototypeClear
            | StandardBuiltinId::SetPrototypeDelete
            | StandardBuiltinId::SetPrototypeDifference
            | StandardBuiltinId::SetPrototypeForEach
            | StandardBuiltinId::SetPrototypeIntersection
            | StandardBuiltinId::SetPrototypeIsDisjointFrom
            | StandardBuiltinId::SetPrototypeIsSubsetOf
            | StandardBuiltinId::SetPrototypeIsSupersetOf
            | StandardBuiltinId::SetPrototypeSymmetricDifference
            | StandardBuiltinId::SetPrototypeUnion
            | StandardBuiltinId::SetPrototypeValues
            | StandardBuiltinId::SetPrototypeEntries
            | StandardBuiltinId::SetIteratorNext
            | StandardBuiltinId::SetPrototypeHas
            | StandardBuiltinId::SetPrototypeSizeGetter => {
                self.standard_roots
                    .insert(StandardBuiltinId::SetConstructor);
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
            StandardBuiltinId::DatePrototypeToJson => {
                self.standard_roots
                    .insert(StandardBuiltinId::DateConstructor);
                self.standard_roots
                    .insert(StandardBuiltinId::DatePrototypeToIsoString);
            }
            StandardBuiltinId::DateNow
            | StandardBuiltinId::DateParse
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
            | StandardBuiltinId::DatePrototypeToIsoString
            | StandardBuiltinId::DatePrototypeToPrimitive
            | StandardBuiltinId::DatePrototypeToDateString
            | StandardBuiltinId::DatePrototypeToLocaleDateString
            | StandardBuiltinId::DatePrototypeToLocaleString
            | StandardBuiltinId::DatePrototypeToLocaleTimeString
            | StandardBuiltinId::DatePrototypeToTemporalInstant
            | StandardBuiltinId::DatePrototypeToTimeString
            | StandardBuiltinId::DatePrototypeToString
            | StandardBuiltinId::DatePrototypeToUtcString => {
                self.standard_roots
                    .insert(StandardBuiltinId::DateConstructor);
            }
            StandardBuiltinId::RegExpEscape
            | StandardBuiltinId::RegExpSpeciesGetter
            | StandardBuiltinId::RegExpPrototypeFlagsGetter
            | StandardBuiltinId::RegExpPrototypeSourceGetter
            | StandardBuiltinId::RegExpPrototypeHasIndicesGetter
            | StandardBuiltinId::RegExpPrototypeGlobalGetter
            | StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter
            | StandardBuiltinId::RegExpPrototypeMultilineGetter
            | StandardBuiltinId::RegExpPrototypeDotAllGetter
            | StandardBuiltinId::RegExpPrototypeUnicodeGetter
            | StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter
            | StandardBuiltinId::RegExpPrototypeStickyGetter
            | StandardBuiltinId::RegExpLegacyStaticGetter
            | StandardBuiltinId::RegExpLegacyStaticSetter
            | StandardBuiltinId::RegExpPrototypeCompile
            | StandardBuiltinId::RegExpPrototypeExec
            | StandardBuiltinId::RegExpPrototypeTest
            | StandardBuiltinId::RegExpPrototypeToString
            | StandardBuiltinId::RegExpPrototypeSymbolMatch
            | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
            | StandardBuiltinId::RegExpPrototypeSymbolReplace
            | StandardBuiltinId::RegExpPrototypeSymbolSearch
            | StandardBuiltinId::RegExpPrototypeSymbolSplit => {
                self.standard_roots
                    .insert(StandardBuiltinId::RegExpConstructor);
            }
            StandardBuiltinId::ArrayIteratorIdentity
            | StandardBuiltinId::IteratorFrom
            | StandardBuiltinId::IteratorConcat
            | StandardBuiltinId::IteratorZip
            | StandardBuiltinId::IteratorZipKeyed
            | StandardBuiltinId::IteratorHelperNext
            | StandardBuiltinId::IteratorHelperReturn
            | StandardBuiltinId::IteratorPrototypeToArray
            | StandardBuiltinId::IteratorPrototypeForEach
            | StandardBuiltinId::IteratorPrototypeEvery
            | StandardBuiltinId::IteratorPrototypeSome
            | StandardBuiltinId::IteratorPrototypeFind
            | StandardBuiltinId::IteratorPrototypeReduce
            | StandardBuiltinId::IteratorPrototypeMap
            | StandardBuiltinId::IteratorPrototypeFilter
            | StandardBuiltinId::IteratorPrototypeFlatMap
            | StandardBuiltinId::IteratorPrototypeTake
            | StandardBuiltinId::IteratorPrototypeDrop
            | StandardBuiltinId::IteratorPrototypeConstructorGetter
            | StandardBuiltinId::IteratorPrototypeConstructorSetter
            | StandardBuiltinId::IteratorPrototypeSymbolDispose
            | StandardBuiltinId::IteratorPrototypeToStringTagGetter
            | StandardBuiltinId::IteratorPrototypeToStringTagSetter => {
                // All public Iterator helpers are installed by the Iterator
                // constructor bootstrap block. A method body can be retained
                // through an iterator instance without the global `Iterator`
                // binding otherwise being referenced.
                self.standard_roots
                    .insert(StandardBuiltinId::IteratorConstructor);
            }
            StandardBuiltinId::TypedArrayPrototypeBufferGetter
            | StandardBuiltinId::TypedArrayPrototypeByteLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeByteOffsetGetter
            | StandardBuiltinId::TypedArrayPrototypeLengthGetter
            | StandardBuiltinId::TypedArrayPrototypeToStringTagGetter
            | StandardBuiltinId::TypedArrayPrototypeToString
            | StandardBuiltinId::TypedArrayPrototypeToLocaleString
            | StandardBuiltinId::TypedArrayPrototypeSubarray
            | StandardBuiltinId::TypedArrayPrototypeSlice
            | StandardBuiltinId::TypedArrayPrototypeSet
            | StandardBuiltinId::TypedArrayPrototypeReverse
            | StandardBuiltinId::TypedArrayPrototypeCopyWithin
            | StandardBuiltinId::TypedArrayPrototypeSort
            | StandardBuiltinId::TypedArrayPrototypeToReversed
            | StandardBuiltinId::TypedArrayPrototypeToSorted
            | StandardBuiltinId::TypedArrayPrototypeWith
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
        StatementIr::ModuleUnitOnce { block, .. } => block_exposes_global_object(block),
        StatementIr::Empty
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Throw(init)
        | StatementIr::Return(init) => expr_exposes_global_object(init),
        StatementIr::GeneratorYield {
            value, resume_mode, ..
        } => {
            expr_exposes_global_object(value)
                || matches!(
                    resume_mode,
                    GeneratorResumeModeIr::AssignProperty { target, .. }
                        if expr_exposes_global_object(target)
                )
                || matches!(
                    resume_mode,
                    GeneratorResumeModeIr::AssignProperty {
                        key: PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr),
                        ..
                    } if expr_exposes_global_object(expr)
                )
        }
        StatementIr::AsyncAwait { value, .. } => expr_exposes_global_object(value),
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => {
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
            ..
        } => {
            init.as_ref().is_some_and(for_init_exposes_global_object)
                || test.as_ref().is_some_and(expr_exposes_global_object)
                || update.as_ref().is_some_and(expr_exposes_global_object)
                || statement_exposes_global_object(body)
        }
        StatementIr::GeneratorLoop {
            init,
            test,
            update,
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
        } => {
            init.as_ref().is_some_and(for_init_exposes_global_object)
                || test.as_ref().is_some_and(expr_exposes_global_object)
                || update.as_ref().is_some_and(expr_exposes_global_object)
                || before_suspension
                    .iter()
                    .any(statement_exposes_global_object)
                || statement_exposes_global_object(suspension_statement)
                || after_suspension.iter().any(statement_exposes_global_object)
        }
        StatementIr::GeneratorIf {
            condition,
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => {
            expr_exposes_global_object(condition)
                || then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                    .any(statement_exposes_global_object)
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
            lexical_declarations,
            cases,
            ..
        } => {
            expr_exposes_global_object(discriminant)
                || lexical_declarations
                    .iter()
                    .any(statement_exposes_global_object)
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
            ..
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
        ObjectPropertyIr::PrototypeSetter { value }
        | ObjectPropertyIr::Spread { source: value }
        | ObjectPropertyIr::Data { value, .. }
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

fn array_destructuring_pattern_any_expression(
    pattern: &ArrayDestructuringPatternIr,
    mut predicate: impl FnMut(&TypedExpr) -> bool,
) -> bool {
    let mut found = false;
    pattern.visit_expressions(&mut |expr| found |= predicate(expr));
    found
}

fn object_destructuring_pattern_any_expression(
    pattern: &ObjectDestructuringPatternIr,
    mut predicate: impl FnMut(&TypedExpr) -> bool,
) -> bool {
    let mut found = false;
    pattern.visit_expressions(&mut |expr| found |= predicate(expr));
    found
}

fn expr_exposes_global_object(expr: &TypedExpr) -> bool {
    match &expr.expr {
        // Module top-level `this` is `undefined`, and neither a namespace
        // object nor `import.meta` can reach the global object.
        ExprIr::ImportMeta { .. } | ExprIr::ModuleNamespace { .. } => false,
        ExprIr::DynamicImport {
            specifier, options, ..
        } => {
            expr_exposes_global_object(specifier)
                || options.as_deref().is_some_and(expr_exposes_global_object)
        }
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
        ExprIr::PropertyCompoundAssign {
            target, key, value, ..
        } => {
            property_access_exposes_global_object(target, key)
                || property_key_exposes_global_object(key)
                || expr_exposes_global_object(value)
        }
        ExprIr::OptionalPropertyChain { target, chain } => {
            expr_exposes_global_object(target)
                || chain.iter().any(|operation| match operation {
                    OptionalChainOperationIr::Property { key, .. } => {
                        property_key_exposes_global_object(key)
                            || property_access_exposes_global_object(target, key)
                    }
                    OptionalChainOperationIr::PrivateProperty { .. } => false,
                    OptionalChainOperationIr::Call { args, .. } => {
                        args.iter().any(expr_exposes_global_object)
                    }
                })
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
        ExprIr::MaterializeBinding { value, body, .. } => {
            expr_exposes_global_object(value) || expr_exposes_global_object(body)
        }
        ExprIr::ArrayDestructure { value, pattern, .. } => {
            expr_exposes_global_object(value)
                || array_destructuring_pattern_any_expression(pattern, expr_exposes_global_object)
        }
        ExprIr::ObjectDestructure { value, pattern } => {
            expr_exposes_global_object(value)
                || object_destructuring_pattern_any_expression(pattern, expr_exposes_global_object)
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
            ..
        } => {
            expr_exposes_global_object(callee)
                || this_arg.as_deref().is_some_and(expr_exposes_global_object)
                || args.iter().any(expr_exposes_global_object)
        }
        ExprIr::Construct { callee, args, .. } => {
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
        | ExprIr::TemplateObject(_)
        | ExprIr::RegExpLiteral { .. }
        | ExprIr::FunctionValue(_)
        | ExprIr::This
        | ExprIr::Arguments
        | ExprIr::GlobalPropertyRead { .. }
        | ExprIr::GlobalIdentifierRead { .. }
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
        StatementIr::ModuleUnitOnce { block, .. } => {
            collect_block_global_property_names(block, names);
        }
        StatementIr::Empty
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Throw(init)
        | StatementIr::Return(init) => collect_expr_global_property_names(init, names),
        StatementIr::GeneratorYield {
            value, resume_mode, ..
        } => {
            if let GeneratorResumeModeIr::AssignProperty { target, key } = resume_mode {
                collect_expr_global_property_names(target, names);
                collect_property_key_global_property_names(key, names);
            }
            collect_expr_global_property_names(value, names)
        }
        StatementIr::AsyncAwait { value, .. } => collect_expr_global_property_names(value, names),
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => {
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
            ..
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
        StatementIr::GeneratorLoop {
            init,
            test,
            update,
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
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
            for statement in before_suspension {
                collect_statement_global_property_names(statement, names);
            }
            collect_statement_global_property_names(suspension_statement, names);
            for statement in after_suspension {
                collect_statement_global_property_names(statement, names);
            }
        }
        StatementIr::GeneratorIf {
            condition,
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => {
            collect_expr_global_property_names(condition, names);
            for statement in then_before_yield
                .iter()
                .chain(then_yield_statement.as_deref())
                .chain(then_after_yield)
                .chain(else_before_yield)
                .chain(else_yield_statement.as_deref())
                .chain(else_after_yield)
            {
                collect_statement_global_property_names(statement, names);
            }
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
            lexical_declarations,
            cases,
            ..
        } => {
            collect_expr_global_property_names(discriminant, names);
            for declaration in lexical_declarations {
                collect_statement_global_property_names(declaration, names);
            }
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
            ..
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
        ObjectPropertyIr::PrototypeSetter { value }
        | ObjectPropertyIr::Spread { source: value }
        | ObjectPropertyIr::Data { value, .. }
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
        ExprIr::ImportMeta { .. } | ExprIr::ModuleNamespace { .. } => {}
        ExprIr::DynamicImport {
            specifier, options, ..
        } => {
            collect_expr_global_property_names(specifier, names);
            if let Some(options) = options {
                collect_expr_global_property_names(options, names);
            }
        }
        ExprIr::Identifier(name)
        | ExprIr::GlobalPropertyRead { name }
        | ExprIr::GlobalIdentifierRead { name } => {
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
        ExprIr::PropertyCompoundAssign {
            target, key, value, ..
        } => {
            collect_expr_global_property_names(target, names);
            collect_property_key_global_property_names(key, names);
            collect_expr_global_property_names(value, names);
        }
        ExprIr::OptionalPropertyChain { target, chain } => {
            collect_expr_global_property_names(target, names);
            for operation in chain {
                match operation {
                    OptionalChainOperationIr::Property { key, .. } => {
                        collect_property_key_global_property_names(key, names);
                    }
                    OptionalChainOperationIr::PrivateProperty { .. } => {}
                    OptionalChainOperationIr::Call { args, .. } => {
                        for arg in args {
                            collect_expr_global_property_names(arg, names);
                        }
                    }
                }
            }
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
        ExprIr::MaterializeBinding { value, body, .. } => {
            collect_expr_global_property_names(value, names);
            collect_expr_global_property_names(body, names);
        }
        ExprIr::ArrayDestructure { value, pattern, .. } => {
            collect_expr_global_property_names(value, names);
            pattern.visit_expressions(&mut |expr| collect_expr_global_property_names(expr, names));
            fn collect_assignment_globals(
                pattern: &ArrayDestructuringPatternIr,
                names: &mut BTreeSet<String>,
            ) {
                for element in &pattern.elements {
                    let target = match element {
                        ArrayDestructuringElementIr::Elision => continue,
                        ArrayDestructuringElementIr::Target { target, .. }
                        | ArrayDestructuringElementIr::Rest { target } => target,
                    };
                    match target {
                        DestructuringTargetIr::AssignmentIdentifier {
                            name, global: true, ..
                        } => {
                            names.insert(name.clone());
                        }
                        DestructuringTargetIr::NestedArray(pattern) => {
                            collect_assignment_globals(pattern, names)
                        }
                        DestructuringTargetIr::NestedObject(pattern) => {
                            for property in &pattern.properties {
                                collect_target_globals(&property.target, names);
                            }
                            if let Some(rest) = &pattern.rest {
                                collect_target_globals(rest, names);
                            }
                        }
                        _ => {}
                    }
                }
            }
            fn collect_target_globals(
                target: &DestructuringTargetIr,
                names: &mut BTreeSet<String>,
            ) {
                match target {
                    DestructuringTargetIr::AssignmentIdentifier {
                        name, global: true, ..
                    } => {
                        names.insert(name.clone());
                    }
                    DestructuringTargetIr::NestedArray(pattern) => {
                        collect_assignment_globals(pattern, names)
                    }
                    DestructuringTargetIr::NestedObject(pattern) => {
                        for property in &pattern.properties {
                            collect_target_globals(&property.target, names);
                        }
                        if let Some(rest) = &pattern.rest {
                            collect_target_globals(rest, names);
                        }
                    }
                    _ => {}
                }
            }
            collect_assignment_globals(pattern, names);
        }
        ExprIr::ObjectDestructure { value, pattern } => {
            collect_expr_global_property_names(value, names);
            pattern.visit_expressions(&mut |expr| collect_expr_global_property_names(expr, names));
            fn collect_target_globals(
                target: &DestructuringTargetIr,
                names: &mut BTreeSet<String>,
            ) {
                match target {
                    DestructuringTargetIr::AssignmentIdentifier {
                        name, global: true, ..
                    } => {
                        names.insert(name.clone());
                    }
                    DestructuringTargetIr::NestedArray(pattern) => {
                        for element in &pattern.elements {
                            match element {
                                ArrayDestructuringElementIr::Elision => {}
                                ArrayDestructuringElementIr::Target { target, .. }
                                | ArrayDestructuringElementIr::Rest { target } => {
                                    collect_target_globals(target, names);
                                }
                            }
                        }
                    }
                    DestructuringTargetIr::NestedObject(pattern) => {
                        for property in &pattern.properties {
                            collect_target_globals(&property.target, names);
                        }
                        if let Some(rest) = &pattern.rest {
                            collect_target_globals(rest, names);
                        }
                    }
                    _ => {}
                }
            }
            for property in &pattern.properties {
                collect_target_globals(&property.target, names);
            }
            if let Some(rest) = &pattern.rest {
                collect_target_globals(rest, names);
            }
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
            ..
        } => {
            collect_expr_global_property_names(callee, names);
            if let Some(this_arg) = this_arg {
                collect_expr_global_property_names(this_arg, names);
            }
            for arg in args {
                collect_expr_global_property_names(arg, names);
            }
        }
        ExprIr::Construct { callee, args, .. } => {
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
        | ExprIr::TemplateObject(_)
        | ExprIr::RegExpLiteral { .. }
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

pub(crate) fn standard_builtin_uses_memory_atomics(builtin: StandardBuiltinId) -> bool {
    matches!(
        builtin,
        StandardBuiltinId::AtomicsAdd
            | StandardBuiltinId::AtomicsAnd
            | StandardBuiltinId::AtomicsCompareExchange
            | StandardBuiltinId::AtomicsExchange
            | StandardBuiltinId::AtomicsLoad
            | StandardBuiltinId::AtomicsNotify
            | StandardBuiltinId::AtomicsOr
            | StandardBuiltinId::AtomicsStore
            | StandardBuiltinId::AtomicsSub
            | StandardBuiltinId::AtomicsWait
            | StandardBuiltinId::AtomicsWaitAsync
            | StandardBuiltinId::AtomicsXor
    )
}

pub(crate) fn script_references_memory_atomics(script: &ScriptIr) -> bool {
    StandardBuiltinId::all_functions()
        .iter()
        .copied()
        .filter(|builtin| standard_builtin_uses_memory_atomics(*builtin))
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
    if matches!(
        builtin,
        StandardBuiltinId::GeneratorPrototypeNext
            | StandardBuiltinId::GeneratorPrototypeReturn
            | StandardBuiltinId::GeneratorPrototypeThrow
            | StandardBuiltinId::AsyncGeneratorPrototypeNext
            | StandardBuiltinId::AsyncGeneratorPrototypeReturn
            | StandardBuiltinId::AsyncGeneratorPrototypeThrow
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeFulfilled
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeRejected
    ) {
        return false;
    }
    if builtin == StandardBuiltinId::StringPrototypeIndexOf
        && script_references_standard_builtin(script, StandardBuiltinId::StringPrototypeMatch)
    {
        return false;
    }
    if (builtin == StandardBuiltinId::RegExpPrototypeExec
        || builtin == StandardBuiltinId::RegExpPrototypeSymbolMatch
        || builtin == StandardBuiltinId::RegExpPrototypeSymbolMatchAll
        || builtin == StandardBuiltinId::RegExpPrototypeSymbolReplace
        || builtin == StandardBuiltinId::RegExpPrototypeSymbolSearch)
        && (script_references_standard_builtin(script, StandardBuiltinId::StringPrototypeMatch)
            || script_references_standard_builtin(
                script,
                StandardBuiltinId::StringPrototypeMatchAll,
            )
            || script_references_standard_builtin(script, StandardBuiltinId::StringPrototypeSearch)
            || script_references_standard_builtin(
                script,
                StandardBuiltinId::StringPrototypeReplace,
            )
            || script_references_standard_builtin(
                script,
                StandardBuiltinId::StringPrototypeReplaceAll,
            )
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
            StandardBuiltinId::ArrayFromAsync,
            StandardBuiltinId::TypedArrayFrom,
            StandardBuiltinId::ArrayPrototypeKeys,
            StandardBuiltinId::ArrayPrototypeEntries,
            StandardBuiltinId::ArrayPrototypeValues,
            StandardBuiltinId::TypedArrayPrototypeFind,
            StandardBuiltinId::TypedArrayPrototypeFindIndex,
            StandardBuiltinId::TypedArrayPrototypeFindLast,
            StandardBuiltinId::TypedArrayPrototypeFindLastIndex,
            StandardBuiltinId::TypedArrayPrototypeEvery,
            StandardBuiltinId::TypedArrayPrototypeSome,
            StandardBuiltinId::TypedArrayPrototypeMap,
            StandardBuiltinId::TypedArrayPrototypeFilter,
            StandardBuiltinId::TypedArrayPrototypeForEach,
            StandardBuiltinId::TypedArrayPrototypeReduce,
            StandardBuiltinId::TypedArrayPrototypeReduceRight,
            StandardBuiltinId::TypedArrayPrototypeKeys,
            StandardBuiltinId::TypedArrayPrototypeEntries,
            StandardBuiltinId::TypedArrayPrototypeValues,
            StandardBuiltinId::IteratorFrom,
            StandardBuiltinId::IteratorConcat,
            StandardBuiltinId::IteratorZip,
            StandardBuiltinId::IteratorZipKeyed,
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
    if builtin == StandardBuiltinId::StringIteratorNext
        && script_references_standard_builtin(script, StandardBuiltinId::StringPrototypeIterator)
    {
        return false;
    }
    if builtin == StandardBuiltinId::ArrayPrototypeValues
        && [
            StandardBuiltinId::ArrayFrom,
            StandardBuiltinId::ArrayFromAsync,
            StandardBuiltinId::TypedArrayFrom,
            StandardBuiltinId::IteratorFrom,
            StandardBuiltinId::IteratorConcat,
            StandardBuiltinId::IteratorZip,
            StandardBuiltinId::IteratorZipKeyed,
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
                | StandardBuiltinId::ArrayFromAsync
                | StandardBuiltinId::ArrayOf
                | StandardBuiltinId::ArrayPrototypeConcat
                | StandardBuiltinId::ArrayPrototypeJoin
                | StandardBuiltinId::ArrayPrototypeSlice
                | StandardBuiltinId::ArrayPrototypeSplice
                | StandardBuiltinId::ArrayPrototypeFill
                | StandardBuiltinId::ArrayPrototypeSort
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
                | StandardBuiltinId::ArrayPrototypeShift
                | StandardBuiltinId::ArrayPrototypeUnshift
                | StandardBuiltinId::ArrayPrototypeKeys
                | StandardBuiltinId::ArrayPrototypeEntries
                | StandardBuiltinId::ArrayPrototypeValues
                | StandardBuiltinId::TypedArrayPrototypeIncludes
                | StandardBuiltinId::TypedArrayPrototypeIndexOf
                | StandardBuiltinId::TypedArrayPrototypeLastIndexOf
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
                | StandardBuiltinId::TypedArrayPrototypeKeys
                | StandardBuiltinId::TypedArrayPrototypeEntries
                | StandardBuiltinId::TypedArrayPrototypeValues
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
                | StandardBuiltinId::TypedArrayPrototypeToStringTagGetter
                | StandardBuiltinId::TypedArrayPrototypeToString
                | StandardBuiltinId::TypedArrayPrototypeToLocaleString
                | StandardBuiltinId::TypedArrayPrototypeSubarray
                | StandardBuiltinId::TypedArrayPrototypeSlice
                | StandardBuiltinId::TypedArrayPrototypeSet
                | StandardBuiltinId::TypedArrayPrototypeReverse
                | StandardBuiltinId::TypedArrayPrototypeCopyWithin
                | StandardBuiltinId::TypedArrayPrototypeSort
                | StandardBuiltinId::TypedArrayPrototypeToReversed
                | StandardBuiltinId::TypedArrayPrototypeToSorted
                | StandardBuiltinId::TypedArrayPrototypeWith
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
                | StandardBuiltinId::DateParse
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
                | StandardBuiltinId::DatePrototypeToIsoString
                | StandardBuiltinId::DatePrototypeToJson
                | StandardBuiltinId::DatePrototypeToPrimitive
                | StandardBuiltinId::DatePrototypeToDateString
                | StandardBuiltinId::DatePrototypeToLocaleDateString
                | StandardBuiltinId::DatePrototypeToLocaleString
                | StandardBuiltinId::DatePrototypeToLocaleTimeString
                | StandardBuiltinId::DatePrototypeToTemporalInstant
                | StandardBuiltinId::DatePrototypeToTimeString
                | StandardBuiltinId::DatePrototypeToString
                | StandardBuiltinId::DatePrototypeToUtcString
                | StandardBuiltinId::RegExpConstructor
                | StandardBuiltinId::RegExpLegacyStaticGetter
                | StandardBuiltinId::RegExpLegacyStaticSetter
                | StandardBuiltinId::RegExpPrototypeSymbolMatch
                | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
                | StandardBuiltinId::RegExpPrototypeSymbolReplace
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
                | StandardBuiltinId::StringPrototypeConcat
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
                | StandardBuiltinId::StringPrototypeNormalize
                | StandardBuiltinId::StringPrototypeLocaleCompare
                | StandardBuiltinId::StringPrototypeToLocaleLowerCase
                | StandardBuiltinId::StringPrototypeToLocaleUpperCase
                | StandardBuiltinId::StringPrototypeToLowerCase
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
        StatementIr::ModuleUnitOnce { block, .. } => block_references_function(block, target),
        StatementIr::Empty
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Throw(init)
        | StatementIr::Return(init) => expr_references_function(init, target),
        StatementIr::GeneratorYield {
            value,
            delegate,
            resume_mode,
            ..
        } => {
            expr_references_function(value, target)
                || matches!(
                    resume_mode,
                    GeneratorResumeModeIr::AssignProperty {
                        target: assignment_target,
                        ..
                    } if expr_references_function(assignment_target, target)
                )
                || matches!(
                    resume_mode,
                    GeneratorResumeModeIr::AssignProperty { key, .. }
                        if property_key_references_function(key, target)
                )
                || (*delegate
                    && [
                        StandardBuiltinId::ArrayPrototypeValues,
                        StandardBuiltinId::ArrayIteratorNext,
                        StandardBuiltinId::ArrayIteratorIdentity,
                        StandardBuiltinId::StringPrototypeIterator,
                        StandardBuiltinId::StringIteratorNext,
                    ]
                    .into_iter()
                    .any(|builtin| builtin.function_id() == target.as_str()))
        }
        StatementIr::AsyncAwait { value, .. } => expr_references_function(value, target),
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => statements
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
            ..
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
        StatementIr::GeneratorLoop {
            init,
            test,
            update,
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
        } => {
            init.as_ref()
                .is_some_and(|init| for_init_references_function(init, target))
                || test
                    .as_ref()
                    .is_some_and(|test| expr_references_function(test, target))
                || update
                    .as_ref()
                    .is_some_and(|update| expr_references_function(update, target))
                || before_suspension
                    .iter()
                    .any(|statement| statement_references_function(statement, target))
                || statement_references_function(suspension_statement, target)
                || after_suspension
                    .iter()
                    .any(|statement| statement_references_function(statement, target))
        }
        StatementIr::GeneratorIf {
            condition,
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => {
            expr_references_function(condition, target)
                || then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                    .any(|statement| statement_references_function(statement, target))
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
            lexical_declarations,
            cases,
            ..
        } => {
            expr_references_function(discriminant, target)
                || lexical_declarations
                    .iter()
                    .any(|declaration| statement_references_function(declaration, target))
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
            ..
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
        ObjectPropertyIr::PrototypeSetter { value }
        | ObjectPropertyIr::Data { value, .. }
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
        ObjectPropertyIr::Spread { source } => {
            expr_references_function(source, target)
                || *target == StandardBuiltinId::ReflectOwnKeys.function_id()
                || *target == StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id()
        }
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
            || StandardBuiltinId::BigIntPrototypeToString.function_id() == *target
            || StandardBuiltinId::SymbolPrototypeToString.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeToString.function_id() == *target;
    }
    if name == "join" {
        return StandardBuiltinId::ArrayPrototypeJoin.function_id() == *target;
    }
    if name == "splice" {
        return StandardBuiltinId::ArrayPrototypeSplice.function_id() == *target;
    }
    if name == "fill" {
        return StandardBuiltinId::ArrayPrototypeFill.function_id() == *target;
    }
    if name == "sort" {
        return StandardBuiltinId::ArrayPrototypeSort.function_id() == *target;
    }
    if name == "valueOf" {
        return StandardBuiltinId::NumberPrototypeValueOf.function_id() == *target
            || StandardBuiltinId::StringPrototypeValueOf.function_id() == *target
            || StandardBuiltinId::BooleanPrototypeValueOf.function_id() == *target
            || StandardBuiltinId::BigIntPrototypeValueOf.function_id() == *target
            || StandardBuiltinId::SymbolPrototypeValueOf.function_id() == *target;
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
            || StandardBuiltinId::TypedArrayPrototypeIncludes.function_id() == *target
            || StandardBuiltinId::StringPrototypeIncludes.function_id() == *target;
    }
    if name == "indexOf" {
        return StandardBuiltinId::ArrayPrototypeIndexOf.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeIndexOf.function_id() == *target
            || StandardBuiltinId::StringPrototypeIndexOf.function_id() == *target;
    }
    if name == "lastIndexOf" {
        return StandardBuiltinId::ArrayPrototypeLastIndexOf.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeLastIndexOf.function_id() == *target
            || StandardBuiltinId::StringPrototypeLastIndexOf.function_id() == *target;
    }
    if name == "find" {
        return StandardBuiltinId::ArrayPrototypeFind.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeFind.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeFind.function_id() == *target;
    }
    if name == "reduce" {
        return StandardBuiltinId::ArrayPrototypeReduce.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeReduce.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeReduce.function_id() == *target;
    }
    if name == "reduceRight" {
        return StandardBuiltinId::ArrayPrototypeReduceRight.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeReduceRight.function_id() == *target;
    }
    if name == "map" {
        return StandardBuiltinId::ArrayPrototypeMap.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeMap.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeMap.function_id() == *target;
    }
    if name == "filter" {
        return StandardBuiltinId::ArrayPrototypeFilter.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeFilter.function_id() == *target
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
    if name == "zip" {
        return StandardBuiltinId::IteratorZip.function_id() == *target;
    }
    if name == "zipKeyed" {
        return StandardBuiltinId::IteratorZipKeyed.function_id() == *target;
    }
    if name == "concat" {
        return StandardBuiltinId::ArrayPrototypeConcat.function_id() == *target
            || StandardBuiltinId::IteratorConcat.function_id() == *target;
    }
    if name == "findIndex" {
        return StandardBuiltinId::ArrayPrototypeFindIndex.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeFindIndex.function_id() == *target;
    }
    if name == "findLast" {
        return StandardBuiltinId::ArrayPrototypeFindLast.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeFindLast.function_id() == *target;
    }
    if name == "findLastIndex" {
        return StandardBuiltinId::ArrayPrototypeFindLastIndex.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeFindLastIndex.function_id() == *target;
    }
    if name == "every" {
        return StandardBuiltinId::ArrayPrototypeEvery.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeEvery.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeEvery.function_id() == *target;
    }
    if name == "some" {
        return StandardBuiltinId::ArrayPrototypeSome.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeSome.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeSome.function_id() == *target;
    }
    if name == "forEach" {
        return StandardBuiltinId::ArrayPrototypeForEach.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeForEach.function_id() == *target
            || StandardBuiltinId::IteratorPrototypeForEach.function_id() == *target;
    }
    if name == "at" {
        return StandardBuiltinId::ArrayPrototypeAt.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeAt.function_id() == *target
            || StandardBuiltinId::StringPrototypeAt.function_id() == *target;
    }
    if name == "toReversed" {
        return StandardBuiltinId::ArrayPrototypeToReversed.function_id() == *target;
    }
    if name == "with" {
        return StandardBuiltinId::ArrayPrototypeWith.function_id() == *target;
    }
    if name == "toSpliced" {
        return StandardBuiltinId::ArrayPrototypeToSpliced.function_id() == *target;
    }
    if name == "toSorted" {
        return StandardBuiltinId::ArrayPrototypeToSorted.function_id() == *target;
    }
    if name == "reverse" {
        return StandardBuiltinId::ArrayPrototypeReverse.function_id() == *target;
    }
    if name == "copyWithin" {
        return StandardBuiltinId::ArrayPrototypeCopyWithin.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeCopyWithin.function_id() == *target;
    }
    if name == "slice" {
        return StandardBuiltinId::ArrayPrototypeSlice.function_id() == *target
            || StandardBuiltinId::StringPrototypeSlice.function_id() == *target;
    }
    if name == "toLocaleString" {
        return StandardBuiltinId::ArrayPrototypeToLocaleString.function_id() == *target
            || StandardBuiltinId::NumberPrototypeToLocaleString.function_id() == *target
            || StandardBuiltinId::BigIntPrototypeToLocaleString.function_id() == *target;
    }
    if name == "concat" {
        return StandardBuiltinId::ArrayPrototypeConcat.function_id() == *target
            || StandardBuiltinId::StringPrototypeConcat.function_id() == *target;
    }
    if name == "Symbol.iterator" {
        return StandardBuiltinId::ArrayPrototypeValues.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeValues.function_id() == *target
            || StandardBuiltinId::StringPrototypeIterator.function_id() == *target;
    }
    if name == "values" {
        return StandardBuiltinId::ArrayPrototypeValues.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeValues.function_id() == *target;
    }
    if name == "keys" {
        return StandardBuiltinId::ArrayPrototypeKeys.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeKeys.function_id() == *target;
    }
    if name == "entries" {
        return StandardBuiltinId::ArrayPrototypeEntries.function_id() == *target
            || StandardBuiltinId::TypedArrayPrototypeEntries.function_id() == *target;
    }
    if name == "next" {
        return StandardBuiltinId::ArrayIteratorNext.function_id() == *target
            || StandardBuiltinId::StringIteratorNext.function_id() == *target;
    }
    let builtin = match name.as_str() {
        "join" => StandardBuiltinId::ArrayPrototypeJoin,
        "slice" => StandardBuiltinId::ArrayPrototypeSlice,
        "splice" => StandardBuiltinId::ArrayPrototypeSplice,
        "fill" => StandardBuiltinId::ArrayPrototypeFill,
        "sort" => StandardBuiltinId::ArrayPrototypeSort,
        "flat" => StandardBuiltinId::ArrayPrototypeFlat,
        "flatMap" => StandardBuiltinId::ArrayPrototypeFlatMap,
        "reduce" => StandardBuiltinId::ArrayPrototypeReduce,
        "reduceRight" => StandardBuiltinId::ArrayPrototypeReduceRight,
        "pop" => StandardBuiltinId::ArrayPrototypePop,
        "push" => StandardBuiltinId::ArrayPrototypePush,
        "shift" => StandardBuiltinId::ArrayPrototypeShift,
        "unshift" => StandardBuiltinId::ArrayPrototypeUnshift,
        "from" => StandardBuiltinId::ArrayFrom,
        "fromAsync" => StandardBuiltinId::ArrayFromAsync,
        "of" => StandardBuiltinId::ArrayOf,
        "at" => StandardBuiltinId::ArrayPrototypeAt,
        "toReversed" => StandardBuiltinId::ArrayPrototypeToReversed,
        "toSpliced" => StandardBuiltinId::ArrayPrototypeToSpliced,
        "toSorted" => StandardBuiltinId::ArrayPrototypeToSorted,
        "with" => StandardBuiltinId::ArrayPrototypeWith,
        "reverse" => StandardBuiltinId::ArrayPrototypeReverse,
        "copyWithin" => StandardBuiltinId::ArrayPrototypeCopyWithin,
        "keys" => StandardBuiltinId::ArrayPrototypeKeys,
        "entries" => StandardBuiltinId::ArrayPrototypeEntries,
        "values" => StandardBuiltinId::ArrayPrototypeValues,
        "charAt" => StandardBuiltinId::StringPrototypeCharAt,
        "charCodeAt" => StandardBuiltinId::StringPrototypeCharCodeAt,
        "codePointAt" => StandardBuiltinId::StringPrototypeCodePointAt,
        "endsWith" => StandardBuiltinId::StringPrototypeEndsWith,
        "match" => StandardBuiltinId::StringPrototypeMatch,
        "matchAll" => StandardBuiltinId::StringPrototypeMatchAll,
        "padStart" => StandardBuiltinId::StringPrototypePadStart,
        "padEnd" => StandardBuiltinId::StringPrototypePadEnd,
        "repeat" => StandardBuiltinId::StringPrototypeRepeat,
        "normalize" => StandardBuiltinId::StringPrototypeNormalize,
        "localeCompare" => StandardBuiltinId::StringPrototypeLocaleCompare,
        "isWellFormed" => StandardBuiltinId::StringPrototypeIsWellFormed,
        "toWellFormed" => StandardBuiltinId::StringPrototypeToWellFormed,
        "search" => StandardBuiltinId::StringPrototypeSearch,
        "compile" => StandardBuiltinId::RegExpPrototypeCompile,
        "exec" => StandardBuiltinId::RegExpPrototypeExec,
        "test" => StandardBuiltinId::RegExpPrototypeTest,
        "Symbol.match" => StandardBuiltinId::RegExpPrototypeSymbolMatch,
        "Symbol.matchAll" => StandardBuiltinId::RegExpPrototypeSymbolMatchAll,
        "Symbol.replace" => StandardBuiltinId::RegExpPrototypeSymbolReplace,
        "Symbol.search" => StandardBuiltinId::RegExpPrototypeSymbolSearch,
        "Symbol.split" => StandardBuiltinId::RegExpPrototypeSymbolSplit,
        "startsWith" => StandardBuiltinId::StringPrototypeStartsWith,
        "toLocaleLowerCase" => StandardBuiltinId::StringPrototypeToLocaleLowerCase,
        "toLocaleUpperCase" => StandardBuiltinId::StringPrototypeToLocaleUpperCase,
        "toLowerCase" => StandardBuiltinId::StringPrototypeToLowerCase,
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
        ExprIr::ImportMeta { .. } | ExprIr::ModuleNamespace { .. } => false,
        ExprIr::DynamicImport {
            specifier, options, ..
        } => {
            expr_references_function(specifier, target)
                || options
                    .as_deref()
                    .is_some_and(|options| expr_references_function(options, target))
        }
        ExprIr::RegExpLiteral { .. } => {
            // A literal allocates directly and never calls the mutable global
            // RegExp binding. It does, however, require the intrinsic
            // constructor's bootstrap to initialize RegExp.prototype.
            *target == StandardBuiltinId::RegExpConstructor.function_id()
        }
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
        | ExprIr::JsonParseStaticReviver { reviver: value, .. }
        | ExprIr::PrivateIn { rhs: value, .. } => expr_references_function(value, target),
        ExprIr::SpreadArgument(value) => {
            *target == StandardBuiltinId::ArrayPrototypeValues.function_id()
                || *target == StandardBuiltinId::ArrayIteratorNext.function_id()
                || *target == StandardBuiltinId::StringConstructor.function_id()
                || expr_references_function(value, target)
        }
        ExprIr::SpecOperation {
            operation,
            operands,
        } => {
            operands
                .iter()
                .any(|operand| expr_references_function(operand, target))
                || matches!(operation, SpecOperationIr::CopyDataProperties)
                    && (*target == StandardBuiltinId::ReflectOwnKeys.function_id()
                        || *target
                            == StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id())
                || matches!(
                    operation,
                    SpecOperationIr::Get | SpecOperationIr::GetV | SpecOperationIr::GetMethod
                ) && operands.get(1).is_some_and(|key| {
                    let ExprIr::String(name) = &key.expr else {
                        return false;
                    };
                    optimized_call_method_references_function(
                        &PropertyKeyIr::StaticString(name.clone()),
                        target,
                    )
                })
        }
        ExprIr::OptionalPropertyChain {
            target: object,
            chain,
        } => {
            expr_references_function(object, target)
                || chain.iter().any(|operation| match operation {
                    OptionalChainOperationIr::Property { key, .. } => {
                        property_key_references_function(key, target)
                            || optimized_call_method_references_function(key, target)
                            || shape_data_references_function(
                                object.heap_shape.as_deref(),
                                key,
                                target,
                            )
                            || shape_accessor_references_function(
                                object.heap_shape.as_deref(),
                                key,
                                target,
                                true,
                                false,
                            )
                    }
                    OptionalChainOperationIr::PrivateProperty { .. } => false,
                    OptionalChainOperationIr::Call { args, .. } => {
                        args.iter().any(|arg| expr_references_function(arg, target))
                    }
                })
        }
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
        ExprIr::PropertyCompoundAssign {
            target: object,
            key,
            value,
            ..
        } => {
            expr_references_function(object, target)
                || property_key_references_function(key, target)
                || expr_references_function(value, target)
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
        ExprIr::MaterializeBinding { value, body, .. } => {
            expr_references_function(value, target) || expr_references_function(body, target)
        }
        ExprIr::ArrayDestructure { value, pattern, .. } => {
            *target == StandardBuiltinId::ArrayPrototypeValues.function_id()
                || *target == StandardBuiltinId::ArrayIteratorNext.function_id()
                || *target == StandardBuiltinId::StringConstructor.function_id()
                || expr_references_function(value, target)
                || array_destructuring_pattern_any_expression(pattern, |expr| {
                    expr_references_function(expr, target)
                })
        }
        ExprIr::ObjectDestructure { value, pattern } => {
            (pattern.rest.is_some()
                && (*target == StandardBuiltinId::ReflectOwnKeys.function_id()
                    || *target == StandardBuiltinId::ReflectGetOwnPropertyDescriptor.function_id()))
                || expr_references_function(value, target)
                || object_destructuring_pattern_any_expression(pattern, |expr| {
                    expr_references_function(expr, target)
                })
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
            ..
        } => {
            expr_references_function(callee, target)
                || this_arg
                    .as_deref()
                    .is_some_and(|this_arg| expr_references_function(this_arg, target))
                || args.iter().any(|arg| expr_references_function(arg, target))
        }
        ExprIr::Construct { callee, args, .. } => {
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
        | ExprIr::TemplateObject(_)
        | ExprIr::This
        | ExprIr::Arguments
        | ExprIr::Identifier(_)
        | ExprIr::GlobalPropertyRead { .. }
        | ExprIr::GlobalIdentifierRead { .. }
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
    number_pow_import_function_index: Option<u32>,
    wall_clock_millis_import_function_index: Option<u32>,
    shared_memory_alloc_function_index: Option<u32>,
    monotonic_clock_nanos_import_function_index: Option<u32>,
    sleep_nanos_import_function_index: Option<u32>,
    agent_call_import_function_index: Option<u32>,
    touched_number_pow_import: std::cell::Cell<bool>,
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
    pub(crate) fn new(
        metas: BTreeMap<FunctionId, WasmFunctionMeta>,
        number_pow_import_function_index: Option<u32>,
        wall_clock_millis_import_function_index: Option<u32>,
        shared_memory_alloc_function_index: Option<u32>,
        monotonic_clock_nanos_import_function_index: Option<u32>,
        sleep_nanos_import_function_index: Option<u32>,
        agent_call_import_function_index: Option<u32>,
    ) -> Self {
        Self {
            metas,
            touched_standard_builtins: std::cell::RefCell::new(BTreeSet::new()),
            touched_host_builtins: std::cell::RefCell::new(BTreeSet::new()),
            number_pow_import_function_index,
            wall_clock_millis_import_function_index,
            shared_memory_alloc_function_index,
            monotonic_clock_nanos_import_function_index,
            sleep_nanos_import_function_index,
            agent_call_import_function_index,
            touched_number_pow_import: std::cell::Cell::new(false),
            suppress_recording: std::cell::Cell::new(false),
        }
    }

    pub(crate) fn number_pow_import_function_index(&self) -> Option<u32> {
        if !self.suppress_recording.get() {
            self.touched_number_pow_import.set(true);
        }
        self.number_pow_import_function_index
    }

    pub(crate) fn touched_number_pow_import(&self) -> bool {
        self.touched_number_pow_import.get()
    }

    pub(crate) fn wall_clock_millis_import_function_index(&self) -> Option<u32> {
        self.wall_clock_millis_import_function_index
    }

    pub(crate) fn shared_memory_alloc_function_index(&self) -> Option<u32> {
        self.shared_memory_alloc_function_index
    }

    pub(crate) fn monotonic_clock_nanos_import_function_index(&self) -> Option<u32> {
        self.monotonic_clock_nanos_import_function_index
    }

    pub(crate) fn sleep_nanos_import_function_index(&self) -> Option<u32> {
        self.sleep_nanos_import_function_index
    }

    pub(crate) fn agent_call_import_function_index(&self) -> Option<u32> {
        self.agent_call_import_function_index
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
                execution_kind: function.execution_kind,
                constructable: function.constructable,
                strict: function.strict,
                is_named_expression: function.is_named_expression,
                class_kind: function.class_kind,
                class_element_execution_kind: function.class_element_execution_kind,
                class_heritage_kind: function.class_heritage_kind,
                is_static_class_member: function.is_static_class_member,
                is_derived_constructor: function.is_derived_constructor,
                is_synthetic_default_derived_constructor: function
                    .is_synthetic_default_derived_constructor,
                class_instance_element_plan: function.class_instance_element_plan.clone(),
                super_constructor_target: function.super_constructor_target.clone(),
                uses_super: function.uses_super,
                this_before_super: function.this_before_super,
                captures_private_environment: function.captures_private_environment,
                needs_active_function_identity: function.flavor == FunctionFlavor::Ordinary,
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
            execution_kind: FunctionExecutionKind::Ordinary,
            constructable: builtin.constructable(),
            strict: true,
            is_named_expression: false,
            class_kind: ClassFunctionKind::None,
            class_element_execution_kind: ClassElementExecutionKind::None,
            class_heritage_kind: ClassHeritageKind::None,
            is_static_class_member: false,
            is_derived_constructor: false,
            is_synthetic_default_derived_constructor: false,
            class_instance_element_plan: None,
            super_constructor_target: None,
            uses_super: false,
            this_before_super: false,
            captures_private_environment: false,
            needs_active_function_identity: false,
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
        execution_kind: FunctionExecutionKind::Ordinary,
        constructable: false,
        strict: true,
        is_named_expression: false,
        class_kind: ClassFunctionKind::None,
        class_element_execution_kind: ClassElementExecutionKind::None,
        class_heritage_kind: ClassHeritageKind::None,
        is_static_class_member: false,
        is_derived_constructor: false,
        is_synthetic_default_derived_constructor: false,
        class_instance_element_plan: None,
        super_constructor_target: None,
        uses_super: false,
        this_before_super: false,
        captures_private_environment: false,
        needs_active_function_identity: false,
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
        StandardBuiltinId::FunctionConstructor
        | StandardBuiltinId::WeakRefConstructor
        | StandardBuiltinId::FinalizationRegistryConstructor
        | StandardBuiltinId::FinalizationRegistryPrototypeUnregister => 1,
        StandardBuiltinId::PromiseConstructor
        | StandardBuiltinId::PromiseResolve
        | StandardBuiltinId::PromiseTry
        | StandardBuiltinId::PromiseReject
        | StandardBuiltinId::PromiseAll
        | StandardBuiltinId::PromiseAllSettled
        | StandardBuiltinId::PromiseAllKeyed
        | StandardBuiltinId::PromiseAllSettledKeyed
        | StandardBuiltinId::PromiseAny
        | StandardBuiltinId::PromiseRace
        | StandardBuiltinId::PromiseAllResolveElement
        | StandardBuiltinId::PromiseAllSettledResolveElement
        | StandardBuiltinId::PromiseAllSettledRejectElement
        | StandardBuiltinId::PromiseAnyRejectElement
        | StandardBuiltinId::PromiseAllKeyedResolveElement
        | StandardBuiltinId::PromiseAllSettledKeyedResolveElement
        | StandardBuiltinId::PromiseAllSettledKeyedRejectElement
        | StandardBuiltinId::PromiseResolveFunction
        | StandardBuiltinId::PromiseRejectFunction
        | StandardBuiltinId::PromisePrototypeCatch
        | StandardBuiltinId::PromisePrototypeFinally
        | StandardBuiltinId::PromiseThenFinally
        | StandardBuiltinId::PromiseCatchFinally => 1,
        StandardBuiltinId::PromiseWithResolvers
        | StandardBuiltinId::PromiseValueThunk
        | StandardBuiltinId::PromiseThrower => 0,
        StandardBuiltinId::PromisePrototypeThen
        | StandardBuiltinId::FinalizationRegistryPrototypeRegister => 2,
        StandardBuiltinId::PromiseCapabilityExecutor => 2,
        StandardBuiltinId::MapConstructor
        | StandardBuiltinId::MapSpeciesGetter
        | StandardBuiltinId::MapPrototypeClear
        | StandardBuiltinId::MapPrototypeKeys
        | StandardBuiltinId::MapPrototypeValues
        | StandardBuiltinId::MapPrototypeEntries
        | StandardBuiltinId::MapIteratorNext
        | StandardBuiltinId::MapPrototypeSizeGetter
        | StandardBuiltinId::WeakMapConstructor
        | StandardBuiltinId::WeakSetConstructor
        | StandardBuiltinId::WeakRefPrototypeDeref => 0,
        StandardBuiltinId::MapPrototypeDelete
        | StandardBuiltinId::MapPrototypeForEach
        | StandardBuiltinId::MapPrototypeGet
        | StandardBuiltinId::MapPrototypeHas
        | StandardBuiltinId::WeakMapPrototypeDelete
        | StandardBuiltinId::WeakMapPrototypeGet
        | StandardBuiltinId::WeakMapPrototypeHas
        | StandardBuiltinId::WeakSetPrototypeAdd
        | StandardBuiltinId::WeakSetPrototypeDelete
        | StandardBuiltinId::WeakSetPrototypeHas => 1,
        StandardBuiltinId::ObjectFromEntries => 1,
        StandardBuiltinId::MapGroupBy
        | StandardBuiltinId::ObjectGroupBy
        | StandardBuiltinId::MapPrototypeGetOrInsert
        | StandardBuiltinId::MapPrototypeGetOrInsertComputed
        | StandardBuiltinId::MapPrototypeSet
        | StandardBuiltinId::WeakMapPrototypeGetOrInsert
        | StandardBuiltinId::WeakMapPrototypeGetOrInsertComputed
        | StandardBuiltinId::WeakMapPrototypeSet => 2,
        StandardBuiltinId::SetConstructor
        | StandardBuiltinId::SetSpeciesGetter
        | StandardBuiltinId::SetPrototypeClear
        | StandardBuiltinId::SetPrototypeValues
        | StandardBuiltinId::SetPrototypeEntries
        | StandardBuiltinId::SetIteratorNext
        | StandardBuiltinId::SetPrototypeSizeGetter => 0,
        StandardBuiltinId::SetPrototypeAdd
        | StandardBuiltinId::SetPrototypeDelete
        | StandardBuiltinId::SetPrototypeDifference
        | StandardBuiltinId::SetPrototypeForEach
        | StandardBuiltinId::SetPrototypeHas
        | StandardBuiltinId::SetPrototypeIntersection
        | StandardBuiltinId::SetPrototypeIsDisjointFrom
        | StandardBuiltinId::SetPrototypeIsSubsetOf
        | StandardBuiltinId::SetPrototypeIsSupersetOf
        | StandardBuiltinId::SetPrototypeSymmetricDifference
        | StandardBuiltinId::SetPrototypeUnion => 1,
        StandardBuiltinId::EvalFunction => 1,
        StandardBuiltinId::FunctionPrototypeCall => 1,
        StandardBuiltinId::FunctionPrototypeApply => 2,
        StandardBuiltinId::FunctionPrototypeBind => 1,
        StandardBuiltinId::ObjectConstructor => 1,
        StandardBuiltinId::ObjectAssign => 2,
        StandardBuiltinId::ObjectCreate => 2,
        StandardBuiltinId::ObjectGetPrototypeOf => 1,
        StandardBuiltinId::ObjectSetPrototypeOf => 2,
        StandardBuiltinId::ObjectDefineProperty => 3,
        StandardBuiltinId::ObjectDefineProperties => 2,
        StandardBuiltinId::ObjectGetOwnPropertyDescriptor => 2,
        StandardBuiltinId::ObjectGetOwnPropertyDescriptors => 1,
        StandardBuiltinId::ObjectGetOwnPropertyNames => 1,
        StandardBuiltinId::ObjectGetOwnPropertySymbols => 1,
        StandardBuiltinId::ObjectKeys => 1,
        StandardBuiltinId::ObjectValues => 1,
        StandardBuiltinId::ObjectEntries => 1,
        StandardBuiltinId::ObjectHasOwn => 2,
        StandardBuiltinId::ObjectIs => 2,
        StandardBuiltinId::ObjectIsSealed => 1,
        StandardBuiltinId::ObjectIsFrozen => 1,
        StandardBuiltinId::ObjectSeal => 1,
        StandardBuiltinId::ObjectFreeze => 1,
        StandardBuiltinId::ObjectIsExtensible => 1,
        StandardBuiltinId::ObjectPreventExtensions => 1,
        StandardBuiltinId::ObjectPrototypeHasOwnProperty => 1,
        StandardBuiltinId::ObjectPrototypeLookupGetter => 1,
        StandardBuiltinId::ObjectPrototypeLookupSetter => 1,
        StandardBuiltinId::ObjectPrototypeProtoGetter => 0,
        StandardBuiltinId::ObjectPrototypeProtoSetter => 1,
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
        StandardBuiltinId::ArrayFromAsync
        | StandardBuiltinId::ArrayFromAsyncFulfilled
        | StandardBuiltinId::ArrayFromAsyncRejected => 1,
        StandardBuiltinId::ArrayOf => 0,
        StandardBuiltinId::TypedArrayFrom => 1,
        StandardBuiltinId::TypedArrayOf => 0,
        StandardBuiltinId::ArrayIsArray => 1,
        StandardBuiltinId::ArrayPrototypeToLocaleString => 0,
        StandardBuiltinId::ArrayPrototypeFlat => 0,
        StandardBuiltinId::ArrayPrototypeFlatMap => 1,
        StandardBuiltinId::ArrayPrototypeAt | StandardBuiltinId::TypedArrayPrototypeAt => 1,
        StandardBuiltinId::TypedArrayPrototypeFind
        | StandardBuiltinId::TypedArrayPrototypeFindIndex
        | StandardBuiltinId::TypedArrayPrototypeFindLast
        | StandardBuiltinId::TypedArrayPrototypeFindLastIndex => 1,
        StandardBuiltinId::TypedArrayPrototypeValues
        | StandardBuiltinId::TypedArrayPrototypeKeys
        | StandardBuiltinId::TypedArrayPrototypeEntries => 0,
        StandardBuiltinId::ArrayPrototypeToReversed => 0,
        StandardBuiltinId::ArrayPrototypeToSpliced => 2,
        StandardBuiltinId::ArrayPrototypeToSorted => 1,
        StandardBuiltinId::ArrayPrototypeWith => 2,
        StandardBuiltinId::ArrayPrototypeReverse => 0,
        StandardBuiltinId::ArrayPrototypeCopyWithin
        | StandardBuiltinId::TypedArrayPrototypeCopyWithin => 2,
        StandardBuiltinId::ArrayPrototypeIncludes
        | StandardBuiltinId::TypedArrayPrototypeIncludes => 1,
        StandardBuiltinId::ArrayPrototypeIndexOf
        | StandardBuiltinId::TypedArrayPrototypeIndexOf => 1,
        StandardBuiltinId::ArrayPrototypeLastIndexOf
        | StandardBuiltinId::TypedArrayPrototypeLastIndexOf => 1,
        StandardBuiltinId::ArrayPrototypeFind => 1,
        StandardBuiltinId::ArrayPrototypeFindIndex => 1,
        StandardBuiltinId::ArrayPrototypeFindLast => 1,
        StandardBuiltinId::ArrayPrototypeFindLastIndex => 1,
        StandardBuiltinId::ArrayPrototypeEvery | StandardBuiltinId::TypedArrayPrototypeEvery => 1,
        StandardBuiltinId::ArrayPrototypeSome | StandardBuiltinId::TypedArrayPrototypeSome => 1,
        StandardBuiltinId::ArrayPrototypeForEach
        | StandardBuiltinId::TypedArrayPrototypeForEach => 1,
        StandardBuiltinId::ArrayPrototypeFilter | StandardBuiltinId::TypedArrayPrototypeFilter => 1,
        StandardBuiltinId::ArrayPrototypeMap | StandardBuiltinId::TypedArrayPrototypeMap => 1,
        StandardBuiltinId::ArrayPrototypeReduce | StandardBuiltinId::TypedArrayPrototypeReduce => 1,
        StandardBuiltinId::ArrayPrototypeReduceRight
        | StandardBuiltinId::TypedArrayPrototypeReduceRight => 1,
        StandardBuiltinId::ArrayPrototypeConcat => 1,
        StandardBuiltinId::StringPrototypeConcat => 1,
        StandardBuiltinId::StringPrototypeLocaleCompare => 1,
        StandardBuiltinId::ArrayPrototypeJoin | StandardBuiltinId::TypedArrayPrototypeJoin => 1,
        StandardBuiltinId::ArrayPrototypeSlice => 2,
        StandardBuiltinId::TypedArrayPrototypeSubarray => 2,
        StandardBuiltinId::TypedArrayPrototypeSlice => 2,
        StandardBuiltinId::TypedArrayPrototypeSet => 1,
        StandardBuiltinId::TypedArrayPrototypeReverse => 0,
        StandardBuiltinId::TypedArrayPrototypeSort => 1,
        StandardBuiltinId::TypedArrayPrototypeToReversed => 0,
        StandardBuiltinId::TypedArrayPrototypeToSorted => 1,
        StandardBuiltinId::TypedArrayPrototypeWith => 2,
        StandardBuiltinId::ArrayPrototypeSplice => 2,
        StandardBuiltinId::ArrayPrototypeFill => 1,
        StandardBuiltinId::ArrayPrototypeSort => 1,
        StandardBuiltinId::ArrayPrototypePop => 0,
        StandardBuiltinId::ArrayPrototypePush => 1,
        StandardBuiltinId::ArrayPrototypeShift => 0,
        StandardBuiltinId::ArrayPrototypeUnshift => 1,
        StandardBuiltinId::ArrayPrototypeKeys => 0,
        StandardBuiltinId::ArrayPrototypeEntries => 0,
        StandardBuiltinId::ArrayPrototypeValues => 0,
        StandardBuiltinId::ArrayIteratorNext => 0,
        StandardBuiltinId::ArrayIteratorIdentity => 0,
        StandardBuiltinId::StringIteratorNext => 0,
        StandardBuiltinId::GeneratorPrototypeNext
        | StandardBuiltinId::GeneratorPrototypeReturn
        | StandardBuiltinId::GeneratorPrototypeThrow
        | StandardBuiltinId::AsyncGeneratorPrototypeNext
        | StandardBuiltinId::AsyncGeneratorPrototypeReturn
        | StandardBuiltinId::AsyncGeneratorPrototypeThrow => 1,
        StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose => 0,
        StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeFulfilled
        | StandardBuiltinId::AsyncIteratorPrototypeAsyncDisposeRejected => 1,
        StandardBuiltinId::StringPrototypeIterator => 0,
        StandardBuiltinId::IteratorConstructor => 0,
        StandardBuiltinId::IteratorFrom => 1,
        StandardBuiltinId::IteratorConcat => 0,
        StandardBuiltinId::IteratorConcatNext => 0,
        StandardBuiltinId::IteratorConcatReturn => 0,
        StandardBuiltinId::IteratorZip => 1,
        StandardBuiltinId::IteratorZipKeyed => 1,
        StandardBuiltinId::IteratorZipNext => 0,
        StandardBuiltinId::IteratorZipReturn => 0,
        StandardBuiltinId::IteratorHelperNext => 0,
        StandardBuiltinId::IteratorHelperReturn => 0,
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
        StandardBuiltinId::RegExpPrototypeFlagsGetter => 0,
        StandardBuiltinId::RegExpPrototypeSourceGetter
        | StandardBuiltinId::RegExpPrototypeHasIndicesGetter
        | StandardBuiltinId::RegExpPrototypeGlobalGetter
        | StandardBuiltinId::RegExpPrototypeIgnoreCaseGetter
        | StandardBuiltinId::RegExpPrototypeMultilineGetter
        | StandardBuiltinId::RegExpPrototypeDotAllGetter
        | StandardBuiltinId::RegExpPrototypeUnicodeGetter
        | StandardBuiltinId::RegExpPrototypeUnicodeSetsGetter
        | StandardBuiltinId::RegExpPrototypeStickyGetter => 0,
        StandardBuiltinId::RegExpLegacyStaticGetter => 0,
        StandardBuiltinId::RegExpLegacyStaticSetter => 1,
        StandardBuiltinId::RegExpPrototypeCompile => 2,
        StandardBuiltinId::RegExpPrototypeToString => 0,
        StandardBuiltinId::RegExpPrototypeExec
        | StandardBuiltinId::RegExpPrototypeTest
        | StandardBuiltinId::RegExpPrototypeSymbolMatch
        | StandardBuiltinId::RegExpPrototypeSymbolMatchAll
        | StandardBuiltinId::RegExpPrototypeSymbolSearch => 1,
        StandardBuiltinId::RegExpPrototypeSymbolReplace
        | StandardBuiltinId::RegExpPrototypeSymbolSplit => 2,
        StandardBuiltinId::RegExpEscape => 1,
        StandardBuiltinId::JsonParse => 2,
        StandardBuiltinId::JsonStringify => 3,
        StandardBuiltinId::JsonRawJson => 1,
        StandardBuiltinId::JsonIsRawJson => 1,
        StandardBuiltinId::DateUtc => 7,
        StandardBuiltinId::DateParse => 1,
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
        | StandardBuiltinId::DatePrototypeToIsoString
        | StandardBuiltinId::DatePrototypeToDateString
        | StandardBuiltinId::DatePrototypeToLocaleDateString
        | StandardBuiltinId::DatePrototypeToLocaleString
        | StandardBuiltinId::DatePrototypeToLocaleTimeString
        | StandardBuiltinId::DatePrototypeToTemporalInstant
        | StandardBuiltinId::DatePrototypeToTimeString
        | StandardBuiltinId::DatePrototypeToString
        | StandardBuiltinId::DatePrototypeToUtcString => 0,
        StandardBuiltinId::DatePrototypeSetTime
        | StandardBuiltinId::DatePrototypeSetYear
        | StandardBuiltinId::DatePrototypeToJson
        | StandardBuiltinId::DatePrototypeToPrimitive => 1,
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
        | StandardBuiltinId::TypedArrayPrototypeToStringTagGetter
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
        | StandardBuiltinId::StringPrototypeToLocaleLowerCase
        | StandardBuiltinId::StringPrototypeToLocaleUpperCase
        | StandardBuiltinId::StringPrototypeToLowerCase
        | StandardBuiltinId::StringPrototypeToUpperCase
        | StandardBuiltinId::StringPrototypeNormalize
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
        | StandardBuiltinId::StringFromCharCode
        | StandardBuiltinId::StringFromCodePoint
        | StandardBuiltinId::StringRaw
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
        StandardBuiltinId::BooleanConstructor
        | StandardBuiltinId::TemporalInstantConstructor
        | StandardBuiltinId::TemporalInstantFrom
        | StandardBuiltinId::TemporalInstantPrototypeEquals
        | StandardBuiltinId::TemporalZonedDateTimeFrom
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEquals
        | StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone => 1,
        StandardBuiltinId::TemporalZonedDateTimeConstructor => 2,
        StandardBuiltinId::IntlGetCanonicalLocales | StandardBuiltinId::IntlLocaleConstructor => 1,
        StandardBuiltinId::IntlLocalePrototypeLanguageGetter
        | StandardBuiltinId::IntlLocalePrototypeScriptGetter
        | StandardBuiltinId::IntlLocalePrototypeRegionGetter
        | StandardBuiltinId::IntlLocalePrototypeBaseNameGetter
        | StandardBuiltinId::IntlLocalePrototypeToString => 0,
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
        | StandardBuiltinId::TypedArraySpeciesGetter
        | StandardBuiltinId::ArrayBufferSpeciesGetter
        | StandardBuiltinId::RegExpSpeciesGetter
        | StandardBuiltinId::PromiseSpeciesGetter
        | StandardBuiltinId::FunctionPrototypeToString
        | StandardBuiltinId::ErrorPrototypeToString
        | StandardBuiltinId::ThrowTypeError
        | StandardBuiltinId::BoundFunctionInvoker
        | StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter
        | StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter
        | StandardBuiltinId::TemporalInstantPrototypeToString => 0,
        StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeYearGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeDayGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeHourGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMinuteGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeSecondGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMillisecondGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeMicrosecondGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeNanosecondGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeToInstant => 0,
        StandardBuiltinId::Escape
        | StandardBuiltinId::Unescape
        | StandardBuiltinId::EncodeUri
        | StandardBuiltinId::EncodeUriComponent
        | StandardBuiltinId::DecodeUri
        | StandardBuiltinId::DecodeUriComponent => 1,
    }
}

pub(crate) fn host_builtin_length(builtin: HostBuiltinId) -> u64 {
    match builtin {
        HostBuiltinId::Print => 1,
        HostBuiltinId::Gc => 0,
        HostBuiltinId::AssertThrows => 2,
        HostBuiltinId::IsConstructor => 1,
        HostBuiltinId::CreateRealm => 0,
        HostBuiltinId::CreateHTMLDDA | HostBuiltinId::HTMLDDA => 0,
        HostBuiltinId::ParseInt => 2,
        HostBuiltinId::ParseFloat => 1,
        HostBuiltinId::DetachArrayBuffer => 1,
        HostBuiltinId::AgentStart => 1,
        HostBuiltinId::AgentBroadcast => 1,
        HostBuiltinId::AgentReceiveBroadcast => 0,
        HostBuiltinId::AgentReport => 1,
        HostBuiltinId::AgentGetReport => 0,
        HostBuiltinId::AgentSleep => 1,
        HostBuiltinId::AgentMonotonicNow => 0,
        HostBuiltinId::AgentLeaving => 0,
    }
}

pub(crate) fn function_param_types() -> Vec<ValType> {
    std::iter::repeat_n(ValType::I64, JS_FUNCTION_PARAM_COUNT).collect()
}

/// Returns true when payload-only emission cannot reconstruct the tag from the inferred value kind.
pub(crate) fn expr_result_tag_is_runtime_dynamic(expr: &ExprIr) -> bool {
    match expr {
        ExprIr::BigInt(value) => value.requires_arbitrary_precision_storage,
        ExprIr::UpdateIdentifier {
            value_kind: ValueKind::Dynamic,
            ..
        }
        | ExprIr::GlobalPropertyUpdate {
            value_kind: ValueKind::Dynamic,
            ..
        }
        | ExprIr::This
        | ExprIr::Identifier(_)
        | ExprIr::PropertyRead { .. }
        | ExprIr::OptionalPropertyChain { .. }
        | ExprIr::GlobalPropertyRead { .. }
        | ExprIr::CallNamed { .. }
        | ExprIr::SpreadArgument(_)
        | ExprIr::RuntimeThrow { .. }
        | ExprIr::CallIndirect { .. }
        | ExprIr::JsonParseStaticReviver { .. }
        | ExprIr::CallMethod { .. }
        | ExprIr::Construct { .. }
        | ExprIr::SuperConstruct { .. }
        | ExprIr::SuperPropertyRead { .. }
        | ExprIr::PrivateRead { .. }
        | ExprIr::SpecOperation {
            operation:
                SpecOperationIr::Get
                | SpecOperationIr::GetV
                | SpecOperationIr::GetMethod
                | SpecOperationIr::HasProperty
                | SpecOperationIr::Call
                | SpecOperationIr::Construct
                | SpecOperationIr::ToBigInt,
            ..
        } => true,
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::PropertyWrite { value, .. }
        | ExprIr::SuperPropertyWrite { value, .. }
        | ExprIr::PrivateWrite { value, .. }
        | ExprIr::Comma { rhs: value, .. }
        | ExprIr::MaterializeBinding { body: value, .. }
        | ExprIr::ObjectDestructure { value, .. } => {
            expr_result_tag_is_runtime_dynamic(&value.expr)
        }
        ExprIr::ArrayDestructure {
            value,
            assignment: true,
            ..
        } => expr_result_tag_is_runtime_dynamic(&value.expr),
        ExprIr::LogicalShortCircuit { lhs, rhs, .. } => {
            expr_result_tag_is_runtime_dynamic(&lhs.expr)
                || expr_result_tag_is_runtime_dynamic(&rhs.expr)
        }
        ExprIr::Conditional {
            then_expr,
            else_expr,
            ..
        } => {
            expr_result_tag_is_runtime_dynamic(&then_expr.expr)
                || expr_result_tag_is_runtime_dynamic(&else_expr.expr)
        }
        _ => false,
    }
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
        // A module unit body runs arbitrary code.
        StatementIr::ModuleUnitOnce { .. } => true,
        StatementIr::Empty
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. } | StatementIr::Expression(init) => expr_uses_calls(init),
        StatementIr::GeneratorYield {
            value, resume_mode, ..
        } => {
            expr_uses_calls(value)
                || matches!(
                    resume_mode,
                    GeneratorResumeModeIr::AssignProperty { target, .. }
                        if expr_uses_calls(target)
                )
                || matches!(
                    resume_mode,
                    GeneratorResumeModeIr::AssignProperty {
                        key: PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr),
                        ..
                    } if expr_uses_calls(expr)
                )
        }
        StatementIr::AsyncAwait { .. } => true,
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => {
            statements.iter().any(statement_uses_calls)
        }
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
            ..
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
            ..
        } => {
            init.as_ref().map(for_init_uses_calls).unwrap_or(false)
                || test.as_ref().map(expr_uses_calls).unwrap_or(false)
                || update.as_ref().map(expr_uses_calls).unwrap_or(false)
                || statement_uses_calls(body)
        }
        StatementIr::GeneratorLoop {
            init,
            test,
            update,
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
        } => {
            init.as_ref().is_some_and(for_init_uses_calls)
                || test.as_ref().is_some_and(expr_uses_calls)
                || update.as_ref().is_some_and(expr_uses_calls)
                || before_suspension.iter().any(statement_uses_calls)
                || statement_uses_calls(suspension_statement)
                || after_suspension.iter().any(statement_uses_calls)
        }
        StatementIr::GeneratorIf {
            condition,
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => {
            expr_uses_calls(condition)
                || then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                    .any(statement_uses_calls)
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
            lexical_declarations,
            cases,
            ..
        } => {
            expr_uses_calls(discriminant)
                || lexical_declarations.iter().any(statement_uses_calls)
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
        StatementIr::ModuleUnitOnce { .. } => true,
        StatementIr::Empty
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => false,
        StatementIr::Lexical { init, .. }
        | StatementIr::Expression(init)
        | StatementIr::Return(init)
        | StatementIr::Throw(init) => expr_uses_function_table(init),
        StatementIr::GeneratorYield {
            value, resume_mode, ..
        } => {
            expr_uses_function_table(value)
                || matches!(
                    resume_mode,
                    GeneratorResumeModeIr::AssignProperty { target, .. }
                        if expr_uses_function_table(target)
                )
                || matches!(
                    resume_mode,
                    GeneratorResumeModeIr::AssignProperty {
                        key: PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr),
                        ..
                    } if expr_uses_function_table(expr)
                )
        }
        StatementIr::AsyncAwait { .. } => true,
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => {
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
            ..
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
            ..
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
        StatementIr::GeneratorLoop {
            init,
            test,
            update,
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
        } => {
            init.as_ref().is_some_and(for_init_uses_function_table)
                || test.as_ref().is_some_and(expr_uses_function_table)
                || update.as_ref().is_some_and(expr_uses_function_table)
                || before_suspension.iter().any(statement_uses_function_table)
                || statement_uses_function_table(suspension_statement)
                || after_suspension.iter().any(statement_uses_function_table)
        }
        StatementIr::GeneratorIf {
            condition,
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => {
            expr_uses_function_table(condition)
                || then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                    .any(statement_uses_function_table)
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
            lexical_declarations,
            cases,
            ..
        } => {
            expr_uses_function_table(discriminant)
                || lexical_declarations
                    .iter()
                    .any(statement_uses_function_table)
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
        ExprIr::ImportMeta { .. } | ExprIr::ModuleNamespace { .. } => false,
        // `import()` materializes a promise with function reactions.
        ExprIr::DynamicImport { .. } => true,
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
        ExprIr::GlobalPropertyRead { .. }
        | ExprIr::GlobalIdentifierRead { .. }
        | ExprIr::GlobalPropertyUpdate { .. } => false,
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
            ObjectPropertyIr::PrototypeSetter { value }
            | ObjectPropertyIr::Spread { source: value }
            | ObjectPropertyIr::Data { value, .. }
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
        ExprIr::OptionalPropertyChain { target, chain } => {
            let mut chain_uses_function_table = false;
            let mut has_call = false;
            for operation in chain {
                match operation {
                    OptionalChainOperationIr::Property { key, .. } => {
                        chain_uses_function_table |= match key {
                            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                                expr_uses_function_table(expr.as_ref())
                            }
                        };
                    }
                    OptionalChainOperationIr::PrivateProperty { .. } => {
                        chain_uses_function_table = true;
                    }
                    OptionalChainOperationIr::Call { args, .. } => {
                        chain_uses_function_table |= args.iter().any(expr_uses_function_table);
                        has_call = true;
                    }
                }
            }
            matches!(target.kind, ValueKind::Object)
                || expr_uses_function_table(target)
                || chain_uses_function_table
                // The callee itself must be dynamically callable.
                || has_call
        }
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
        ExprIr::PropertyCompoundAssign {
            target, key, value, ..
        } => {
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
        ExprIr::MaterializeBinding { value, body, .. } => {
            expr_uses_function_table(value)
                || expr_uses_function_table(body)
                || value.possible_kinds.contains(ValueKind::Object)
                || body.possible_kinds.contains(ValueKind::Object)
        }
        ExprIr::ArrayDestructure { value, pattern, .. } => {
            let _ = (value, pattern);
            true
        }
        ExprIr::ObjectDestructure { value, pattern } => {
            let _ = (value, pattern);
            true
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
        ExprIr::SpreadArgument(_) => true,
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
        | ExprIr::TemplateObject(_)
        | ExprIr::RegExpLiteral { .. }
        | ExprIr::This
        | ExprIr::Identifier(_)
        | ExprIr::UpdateIdentifier { .. } => false,
    }
}

pub(crate) fn expr_uses_calls(expr: &TypedExpr) -> bool {
    match &expr.expr {
        ExprIr::ImportMeta { .. } | ExprIr::ModuleNamespace { .. } => false,
        ExprIr::DynamicImport { .. } => true,
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
        ExprIr::GlobalPropertyRead { .. }
        | ExprIr::GlobalIdentifierRead { .. }
        | ExprIr::GlobalPropertyUpdate { .. } => false,
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
            ObjectPropertyIr::PrototypeSetter { value }
            | ObjectPropertyIr::Spread { source: value }
            | ObjectPropertyIr::Data { value, .. }
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
        ExprIr::OptionalPropertyChain { target, chain } => {
            let mut chain_uses_calls = false;
            let mut has_call = false;
            for operation in chain {
                match operation {
                    OptionalChainOperationIr::Property { key, .. } => {
                        chain_uses_calls |= match key {
                            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => false,
                            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                                expr_uses_calls(expr.as_ref())
                            }
                        };
                    }
                    OptionalChainOperationIr::PrivateProperty { .. } => {
                        chain_uses_calls = true;
                    }
                    OptionalChainOperationIr::Call { args, .. } => {
                        chain_uses_calls |= args.iter().any(expr_uses_calls);
                        has_call = true;
                    }
                }
            }
            expr_uses_calls(target) || chain_uses_calls || has_call
        }
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
        ExprIr::PropertyCompoundAssign {
            target, key, value, ..
        } => {
            expr_uses_calls(target)
                || expr_uses_calls(value)
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
        ExprIr::MaterializeBinding { value, body, .. } => {
            expr_uses_calls(value)
                || expr_uses_calls(body)
                || value.possible_kinds.contains(ValueKind::Object)
                || body.possible_kinds.contains(ValueKind::Object)
        }
        ExprIr::ArrayDestructure { .. } | ExprIr::ObjectDestructure { .. } => true,
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
        | ExprIr::TemplateObject(_)
        | ExprIr::RegExpLiteral { .. }
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
        StatementIr::ModuleUnitOnce { block, .. } => {
            block.statements.iter().map(count_statement_lexicals).sum()
        }
        StatementIr::Expression(TypedExpr {
            expr:
                ExprIr::ArrayDestructure {
                    pattern,
                    assignment: false,
                    ..
                },
            ..
        }) => {
            let mut count = 0;
            pattern
                .visit_bindings(&mut |mode, _| count += usize::from(mode != BindingMode::Var) * 2);
            count
        }
        StatementIr::Expression(TypedExpr {
            expr: ExprIr::ObjectDestructure { pattern, .. },
            ..
        }) => {
            let mut count = 0;
            pattern
                .visit_bindings(&mut |mode, _| count += usize::from(mode != BindingMode::Var) * 2);
            count
        }
        StatementIr::Empty
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Var(_)
        | StatementIr::Expression(_)
        | StatementIr::GeneratorYield { .. }
        | StatementIr::AsyncAwait { .. }
        | StatementIr::Debugger
        | StatementIr::Return(_)
        | StatementIr::Throw(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => 0,
        StatementIr::Lexical { .. } => 2,
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => {
            statements.iter().map(count_statement_lexicals).sum()
        }
        StatementIr::Block(block) => count_block_lexicals(block),
        StatementIr::TryCatch {
            try_block,
            catch_parameter_environment,
            catch_block,
            ..
        } => {
            count_block_lexicals(try_block)
                + usize::from(catch_parameter_environment.is_none()) * 2
                + count_block_lexicals(catch_block)
        }
        StatementIr::TryFinally {
            try_block,
            finally_block,
            ..
        } => count_block_lexicals(try_block) + count_block_lexicals(finally_block),
        StatementIr::TryCatchFinally {
            try_block,
            catch_parameter_environment,
            catch_block,
            finally_block,
            ..
        } => {
            count_block_lexicals(try_block)
                + usize::from(catch_parameter_environment.is_none()) * 2
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
                    ForInitIr::Lexical { .. } => 2,
                    ForInitIr::LexicalBlock(bindings) => 2 * bindings.len(),
                    ForInitIr::Var(_) => 0,
                    ForInitIr::Expression(_) => 0,
                })
                .unwrap_or(0)
                + count_statement_lexicals(body)
        }
        StatementIr::GeneratorLoop {
            init,
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
        } => {
            init.as_ref()
                .map(|init| match init {
                    ForInitIr::Lexical { .. } => 2,
                    ForInitIr::LexicalBlock(bindings) => 2 * bindings.len(),
                    ForInitIr::Var(_) | ForInitIr::Expression(_) => 0,
                })
                .unwrap_or(0)
                + before_suspension
                    .iter()
                    .map(count_statement_lexicals)
                    .sum::<usize>()
                + count_statement_lexicals(suspension_statement)
                + after_suspension
                    .iter()
                    .map(count_statement_lexicals)
                    .sum::<usize>()
        }
        StatementIr::GeneratorIf {
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => then_before_yield
            .iter()
            .chain(then_yield_statement.as_deref())
            .chain(then_after_yield)
            .chain(else_before_yield)
            .chain(else_yield_statement.as_deref())
            .chain(else_after_yield)
            .map(count_statement_lexicals)
            .sum(),
        StatementIr::ForOfArray {
            mode,
            name,
            body,
            lexical_environment,
            ..
        }
        | StatementIr::ForOfString {
            mode,
            name,
            body,
            lexical_environment,
            ..
        }
        | StatementIr::ForOfIterator {
            mode,
            name,
            body,
            lexical_environment,
            ..
        }
        | StatementIr::ForInArray {
            mode,
            name,
            body,
            lexical_environment,
            ..
        }
        | StatementIr::ForInString {
            mode,
            name,
            body,
            lexical_environment,
            ..
        }
        | StatementIr::ForInObject {
            mode,
            name,
            body,
            lexical_environment,
            ..
        } => {
            let binding_locals =
                if *mode == BindingMode::Var {
                    0
                } else if let Some(environment) = lexical_environment {
                    let tdz_locals = 2 * environment
                        .tdz_binding_names
                        .iter()
                        .filter(|name| {
                            environment
                                .tdz_environment
                                .as_ref()
                                .map(|tdz_environment| {
                                    !tdz_environment
                                        .bindings
                                        .iter()
                                        .any(|binding| binding.name == ***name)
                                })
                                .unwrap_or(true)
                        })
                        .count();
                    let iteration_locals = if environment
                        .iteration_environment
                        .as_ref()
                        .is_some_and(|iteration| {
                            iteration
                                .bindings
                                .iter()
                                .any(|binding| binding.name == *name)
                        }) {
                        0
                    } else {
                        2
                    };
                    tdz_locals + iteration_locals
                } else {
                    2
                };
            binding_locals + count_statement_lexicals(body)
        }
        StatementIr::Switch {
            lexical_declarations,
            cases,
            ..
        } => {
            lexical_declarations
                .iter()
                .map(count_statement_lexicals)
                .sum::<usize>()
                + cases
                    .iter()
                    .map(|case| count_block_lexicals(&case.body))
                    .sum::<usize>()
        }
        StatementIr::Labelled { statement, .. } => count_statement_lexicals(statement),
    }
}

pub(crate) fn count_statement_temp_locals(statement: &StatementIr) -> usize {
    match statement {
        StatementIr::ModuleUnitOnce { block, .. } => block
            .statements
            .iter()
            .map(count_statement_temp_locals)
            .max()
            .unwrap_or(0),
        StatementIr::Empty
        | StatementIr::AnnexBFunctionCopy { .. }
        | StatementIr::Debugger
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => 0,
        StatementIr::Return(value) | StatementIr::Throw(value) => count_expr_temp_locals(value),
        StatementIr::GeneratorYield {
            value, resume_mode, ..
        } => {
            let assignment_target_locals = match resume_mode {
                GeneratorResumeModeIr::AssignProperty { target, key } => {
                    let key_locals = match key {
                        PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                        PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                            count_expr_temp_locals(expr)
                        }
                    };
                    count_expr_temp_locals(target).max(key_locals) + 4
                }
                GeneratorResumeModeIr::Ignore
                | GeneratorResumeModeIr::Return
                | GeneratorResumeModeIr::AssignIdentifier(_) => 0,
            };
            count_expr_temp_locals(value).max(assignment_target_locals)
        }
        StatementIr::AsyncAwait { value, .. } => count_expr_temp_locals(value) + 16,
        StatementIr::Var(declarators) => declarators
            .iter()
            .filter_map(|declarator| declarator.init.as_ref())
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0),
        StatementIr::Lexical { init, .. } | StatementIr::Expression(init) => {
            count_expr_temp_locals(init)
        }
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => statements
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
            ..
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
            ..
        } => init
            .as_ref()
            .map(count_for_init_temp_locals)
            .unwrap_or(0)
            .max(test.as_ref().map(count_expr_temp_locals).unwrap_or(0))
            .max(update.as_ref().map(count_expr_temp_locals).unwrap_or(0))
            .max(count_statement_temp_locals(body)),
        StatementIr::GeneratorLoop {
            init,
            test,
            update,
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
        } => init
            .as_ref()
            .map(count_for_init_temp_locals)
            .unwrap_or(0)
            .max(test.as_ref().map(count_expr_temp_locals).unwrap_or(0))
            .max(update.as_ref().map(count_expr_temp_locals).unwrap_or(0))
            .max(
                before_suspension
                    .iter()
                    .chain(std::iter::once(suspension_statement.as_ref()))
                    .chain(after_suspension)
                    .map(count_statement_temp_locals)
                    .max()
                    .unwrap_or(0),
            )
            .max(1),
        StatementIr::GeneratorIf {
            condition,
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => count_expr_temp_locals(condition)
            .max(
                then_before_yield
                    .iter()
                    .chain(then_yield_statement.as_deref())
                    .chain(then_after_yield)
                    .chain(else_before_yield)
                    .chain(else_yield_statement.as_deref())
                    .chain(else_after_yield)
                    .map(count_statement_temp_locals)
                    .max()
                    .unwrap_or(0),
            )
            .max(1),
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
            lexical_declarations,
            cases,
            ..
        } => {
            let declaration_max = lexical_declarations
                .iter()
                .map(count_statement_temp_locals)
                .max()
                .unwrap_or(0);
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
            declaration_max.max(4 + count_expr_temp_locals(discriminant).max(case_max))
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

fn call_args_have_spread(args: &[TypedExpr]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.expr, ExprIr::SpreadArgument(_)))
}

pub(crate) fn count_expr_temp_locals(expr: &TypedExpr) -> usize {
    match &expr.expr {
        ExprIr::ImportMeta { .. } | ExprIr::ModuleNamespace { .. } => 2,
        ExprIr::DynamicImport {
            specifier, options, ..
        } => {
            let operands = count_expr_temp_locals(specifier)
                .max(options.as_deref().map_or(0, count_expr_temp_locals));
            operands + 64
        }
        ExprIr::GlobalPropertyRead { .. } => 12,
        ExprIr::GlobalIdentifierRead { .. } => 24,
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
                    ObjectPropertyIr::PrototypeSetter { value }
                    | ObjectPropertyIr::Data { value, .. }
                    | ObjectPropertyIr::NonEnumerableData { value, .. } => {
                        count_expr_temp_locals(value)
                    }
                    ObjectPropertyIr::Spread { source } => count_expr_temp_locals(source).max(40),
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
        ExprIr::RegExpLiteral { .. } => 7,
        ExprIr::OptionalPropertyChain { target, chain } => {
            let child = chain
                .iter()
                .fold(count_expr_temp_locals(target), |acc, operation| {
                    acc.max(match operation {
                        OptionalChainOperationIr::Property { key, .. } => match key {
                            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                                count_expr_temp_locals(expr.as_ref())
                            }
                        },
                        OptionalChainOperationIr::PrivateProperty { .. } => 8,
                        OptionalChainOperationIr::Call { args, .. } => {
                            let arg_child =
                                args.iter().map(count_expr_temp_locals).max().unwrap_or(0);
                            let call_locals = if call_args_have_spread(args) {
                                256
                            } else {
                                96 + args.len() * 2
                            };
                            arg_child.max(call_locals)
                        }
                    })
                });
            // The chain emitter keeps receiver/reference/call locals live
            // while evaluating keys and arguments.
            child.max(24)
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
        ExprIr::PropertyCompoundAssign {
            target, key, value, ..
        } => {
            let child = count_expr_temp_locals(target)
                .max(count_expr_temp_locals(value))
                .max(match key {
                    PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        count_expr_temp_locals(expr)
                    }
                });
            child.max(96)
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
        ExprIr::SpecOperation {
            operation: SpecOperationIr::CopyDataProperties,
            operands,
        } => operands
            .iter()
            .map(count_expr_temp_locals)
            .max()
            .unwrap_or(0)
            .max(40),
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
        ExprIr::MaterializeBinding { value, body, .. } => {
            2 + count_expr_temp_locals(value).max(count_expr_temp_locals(body))
        }
        ExprIr::ArrayDestructure { value, pattern, .. } => {
            fn pattern_temp_locals(pattern: &ArrayDestructuringPatternIr) -> usize {
                pattern
                    .elements
                    .iter()
                    .map(|element| {
                        let (target, default, rest) = match element {
                            ArrayDestructuringElementIr::Elision => return 0,
                            ArrayDestructuringElementIr::Target { target, default } => {
                                (target, default.as_ref(), false)
                            }
                            ArrayDestructuringElementIr::Rest { target } => (target, None, true),
                        };
                        let target_locals = match target {
                            DestructuringTargetIr::AssignmentProperty { target, key } => {
                                4 + count_expr_temp_locals(target).max(match key {
                                    DestructuringPropertyKeyIr::Static(_) => 0,
                                    DestructuringPropertyKeyIr::Computed(key) => {
                                        count_expr_temp_locals(key)
                                    }
                                })
                            }
                            DestructuringTargetIr::AssignmentPrivate { target, .. } => {
                                11 + count_expr_temp_locals(target)
                            }
                            DestructuringTargetIr::NestedArray(pattern) => {
                                32 + pattern_temp_locals(pattern)
                            }
                            DestructuringTargetIr::NestedObject(pattern) => {
                                let mut child_locals = 0;
                                pattern.visit_expressions(&mut |expr| {
                                    child_locals = child_locals.max(count_expr_temp_locals(expr));
                                });
                                128 + pattern.properties.len() * 2 + child_locals
                            }
                            DestructuringTargetIr::Binding { .. }
                            | DestructuringTargetIr::AssignmentIdentifier { .. } => 0,
                        };
                        target_locals.max(default.map(count_expr_temp_locals).unwrap_or(0))
                            + usize::from(rest) * 2
                    })
                    .max()
                    .unwrap_or(0)
            }
            32 + count_expr_temp_locals(value).max(pattern_temp_locals(pattern))
        }
        ExprIr::ObjectDestructure { value, pattern } => {
            let mut child_locals = count_expr_temp_locals(value);
            pattern.visit_expressions(&mut |expr| {
                child_locals = child_locals.max(count_expr_temp_locals(expr));
            });
            128 + pattern.properties.len() * 2 + child_locals
        }
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
            .max(if call_args_have_spread(args) {
                192
            } else {
                4 + args.len() * 2
            }),
        ExprIr::SpreadArgument(value) => count_expr_temp_locals(value).max(2),
        ExprIr::RuntimeThrow { .. } => 4,
        ExprIr::CallIndirect {
            callee,
            args,
            this_arg,
            ..
        } => count_expr_temp_locals(callee)
            .max(this_arg.as_deref().map(count_expr_temp_locals).unwrap_or(0))
            .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
            .max(if call_args_have_spread(args) { 192 } else { 64 }),
        ExprIr::JsonParseStaticReviver { reviver, .. } => count_expr_temp_locals(reviver).max(64),
        ExprIr::Construct { callee, args, .. } => count_expr_temp_locals(callee)
            .max(args.iter().map(count_expr_temp_locals).max().unwrap_or(0))
            .max(if call_args_have_spread(args) {
                192
            } else {
                10 + args.len() * 2
            }),
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
                .max(if call_args_have_spread(args) {
                    192
                } else {
                    16 + args.len() * 2
                })
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
            .max(if call_args_have_spread(args) { 192 } else { 12 }),
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
        | ExprIr::TemplateObject(_)
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
        // Module top-level `var`s are environment bindings of the module they
        // are written in, not hoisted vars of the merged script body.
        StatementIr::ModuleUnitOnce { .. } => {}
        StatementIr::AnnexBFunctionCopy {
            variable_storage_name,
            ..
        } => {
            names.insert(variable_storage_name.clone());
        }
        StatementIr::Var(declarators) => {
            for declarator in declarators {
                names.insert(declarator.name.clone());
            }
        }
        StatementIr::LexicalBlock(statements)
        | StatementIr::ParameterInitialization { statements, .. } => {
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
        StatementIr::GeneratorLoop {
            init,
            before_suspension,
            suspension_statement,
            after_suspension,
            ..
        } => {
            if let Some(ForInitIr::Var(declarators)) = init {
                for declarator in declarators {
                    names.insert(declarator.name.clone());
                }
            }
            for statement in before_suspension {
                collect_hoisted_vars_statement(statement, names);
            }
            collect_hoisted_vars_statement(suspension_statement, names);
            for statement in after_suspension {
                collect_hoisted_vars_statement(statement, names);
            }
        }
        StatementIr::GeneratorIf {
            then_before_yield,
            then_yield_statement,
            then_after_yield,
            else_before_yield,
            else_yield_statement,
            else_after_yield,
            ..
        } => {
            for statement in then_before_yield
                .iter()
                .chain(then_yield_statement.as_deref())
                .chain(then_after_yield)
                .chain(else_before_yield)
                .chain(else_yield_statement.as_deref())
                .chain(else_after_yield)
            {
                collect_hoisted_vars_statement(statement, names);
            }
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
            ..
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
        StatementIr::Expression(TypedExpr {
            expr:
                ExprIr::ArrayDestructure {
                    pattern,
                    assignment: false,
                    ..
                },
            ..
        }) => {
            pattern.visit_bindings(&mut |mode, name| {
                if mode == BindingMode::Var {
                    names.insert(name.to_string());
                }
            });
        }
        StatementIr::Expression(TypedExpr {
            expr: ExprIr::ObjectDestructure { pattern, .. },
            ..
        }) => {
            pattern.visit_bindings(&mut |mode, name| {
                if mode == BindingMode::Var {
                    names.insert(name.to_string());
                }
            });
        }
        StatementIr::Empty
        | StatementIr::Lexical { .. }
        | StatementIr::Expression(_)
        | StatementIr::GeneratorYield { .. }
        | StatementIr::AsyncAwait { .. }
        | StatementIr::Debugger
        | StatementIr::Return(_)
        | StatementIr::Throw(_)
        | StatementIr::Break { .. }
        | StatementIr::Continue { .. } => {}
    }
}
