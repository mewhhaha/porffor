use super::*;
use lila_ir::{ArrayAccumulationElementIr, ArrayAccumulationIr};
use lila_ir::{
    ArrayDestructuringEvaluationIr, AsyncDisposableResourcesIr, ObjectDestructuringPatternIr,
    OptionalChainOperationIr, SyncDisposableScopeExecutionIr,
};

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
    /// a realm global by `__lilaCreateRealm`, or direct-called from another
    /// builtin's body like `JSON.parse` -> `parseFloat`), so materializations
    /// and direct calls are recorded and force their real bodies.
    pub(crate) host_builtin: Option<HostBuiltinId>,
    pub(crate) length: u64,
    pub(crate) length_name_configurable: bool,
    pub(crate) wasm_index: u32,
    pub(crate) table_index: u32,
    pub(crate) protocol: FunctionProtocolIr,
    pub(crate) strict: bool,
    pub(crate) is_named_expression: bool,
    pub(crate) class_element_execution_kind: ClassElementExecutionKind,
    pub(crate) class_heritage_kind: ClassHeritageKind,
    pub(crate) is_static_class_member: bool,
    pub(crate) is_derived_constructor: bool,
    pub(crate) is_synthetic_default_derived_constructor: bool,
    pub(crate) class_instance_element_plan: Option<ClassInstanceElementPlanIr>,
    pub(crate) uses_super: bool,
    pub(crate) this_before_super: bool,
    pub(crate) captures_private_environment: bool,
    pub(crate) needs_active_function_identity: bool,
}

impl WasmFunctionMeta {
    pub(crate) fn runtime_name(&self) -> &str {
        if self.protocol.class_kind() != ClassFunctionKind::Method {
            return self.name.as_str();
        }
        self.name
            .split_once('.')
            .map_or(self.name.as_str(), |(_, method_name)| method_name)
    }

    pub(crate) const fn has_class_execution_context(&self) -> bool {
        !matches!(self.protocol.class_kind(), ClassFunctionKind::None)
            || !matches!(
                self.class_element_execution_kind,
                ClassElementExecutionKind::None
            )
    }

    pub(crate) const fn has_home_object_execution_context(&self) -> bool {
        self.has_class_execution_context() || self.protocol.is_object_literal_method()
    }

    pub(crate) const fn has_function_context(&self) -> bool {
        self.needs_active_function_identity
            || self.has_home_object_execution_context()
            || self.captures_private_environment
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use lila_front::{parse, ParseOptions};
    use lila_ir::{
        lower_with_host_surface_policy, BigIntLiteralIr, ForOfAssignmentIr, HostSurfacePolicy,
        IteratorProtocolWitness, ObjectAccessorShape, ObjectShape,
    };

    fn lower_script(source: &str) -> ScriptIr {
        let parsed = parse(source, ParseOptions::script()).expect("script should parse");
        lower_with_host_surface_policy(&parsed, HostSurfacePolicy::Test262)
            .script
            .expect("script should lower")
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
    fn dynamic_property_keys_root_every_possible_shape_accessor() {
        let getter = "dynamic.getter".to_string();
        let setter = "dynamic.setter".to_string();
        let shape = HeapShape::Object(ObjectShape {
            properties: BTreeMap::from([(
                "p".to_string(),
                ObjectShapeProperty::Accessor {
                    getter: Some(ObjectAccessorShape {
                        function_id: getter.clone(),
                    }),
                    setter: Some(ObjectAccessorShape {
                        function_id: setter.clone(),
                    }),
                },
            )]),
            ..ObjectShape::default()
        });
        let key = PropertyKeyIr::StringExpr(Box::new(TypedExpr::from_info(
            ValueInfo::new(ValueKind::String),
            ExprIr::Identifier("key".to_string()),
        )));

        assert!(shape_accessor_references_function(
            Some(&shape),
            &key,
            &getter,
            ShapeAccessorReferenceSelection::Getter,
        ));
        assert!(shape_accessor_references_function(
            Some(&shape),
            &key,
            &setter,
            ShapeAccessorReferenceSelection::Setter,
        ));
        assert!(!shape_accessor_references_function(
            Some(&shape),
            &key,
            &getter,
            ShapeAccessorReferenceSelection::Setter,
        ));
        assert!(shape_accessor_references_function(
            Some(&shape),
            &key,
            &getter,
            ShapeAccessorReferenceSelection::GetterOrSetter,
        ));
    }

    #[test]
    fn joined_logical_property_base_roots_every_carried_builtin_accessor() {
        let script = lower_script(
            "function readSize(flag, map, set) { return (flag ? map : set).size ||= 1; } readSize(true, new Map(), new Set());",
        );

        for builtin in [
            StandardBuiltinId::MapPrototypeSizeGetter,
            StandardBuiltinId::SetPrototypeSizeGetter,
        ] {
            assert!(
                !should_stub_standard_builtin(&script, builtin),
                "joined logical base lost {builtin:?}"
            );
        }
    }

    #[test]
    fn joined_eager_property_base_roots_every_carried_builtin_getter() {
        let script = lower_script(
            "function addSize(flag, map, set) { return (flag ? map : set).size += 1; } addSize(true, new Map(), new Set());",
        );

        for builtin in [
            StandardBuiltinId::MapPrototypeSizeGetter,
            StandardBuiltinId::SetPrototypeSizeGetter,
        ] {
            assert!(!should_stub_standard_builtin(&script, builtin));
        }
    }

    #[test]
    fn joined_numeric_property_base_roots_every_carried_builtin_getter() {
        let script = lower_script(
            "function incrementSize(flag, map, set) { return (flag ? map : set).size++; } incrementSize(true, new Map(), new Set());",
        );

        for builtin in [
            StandardBuiltinId::MapPrototypeSizeGetter,
            StandardBuiltinId::SetPrototypeSizeGetter,
        ] {
            assert!(!should_stub_standard_builtin(&script, builtin));
        }
    }

    #[test]
    fn joined_plain_property_base_roots_its_carried_builtin_setter() {
        let script = lower_script(
            "function setProto(flag, map, set, value) { return (flag ? map : set).__proto__ = value; } setProto(true, new Map(), new Set(), null);",
        );

        assert!(!should_stub_standard_builtin(
            &script,
            StandardBuiltinId::ObjectPrototypeProtoSetter,
        ));
    }

    #[test]
    fn proven_non_proxy_proto_getter_does_not_root_unrelated_builtin_accessors() {
        let script = lower_script(
            "const prototype = {}; const target = {}; target.__proto__ = prototype; target.__proto__;",
        );
        let plan = RuntimeBootstrapPlan::from_script(&script, &[]);

        assert!(!plan
            .standard_roots
            .contains(&StandardBuiltinId::RegExpPrototypeFlagsGetter));
    }

    #[test]
    fn deeply_nested_strict_logical_properties_budget_failed_set_errors() {
        const DEPTH: usize = 186;

        let script = lower_script("\"use strict\"; let target = {}; target.p ||= 0;");
        let single = count_block_temp_locals(&script.body);
        assert_eq!(
            single,
            ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS
                + ORDINARY_PROPERTY_FAILED_SET_ERROR_TEMP_LOCALS
        );

        let actual = (1..DEPTH).fold(single, |child, _| {
            ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS
                + child
                    .max(ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_FAILED_SET_ERROR_TEMP_LOCALS)
        });

        let expected = DEPTH * ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS
            + ORDINARY_PROPERTY_FAILED_SET_ERROR_TEMP_LOCALS;
        assert!(
            expected > 2048,
            "regression must cross the local-count floor"
        );
        assert_eq!(actual, expected);
    }

    #[test]
    fn deeply_nested_sync_for_of_budgets_every_live_iterator_record() {
        const DEPTH: usize = 90;

        let statement = (0..DEPTH).fold(StatementIr::Empty, |body, index| {
            StatementIr::ForOfIterator {
                head: ForOfIteratorHeadIr::Assignment {
                    binding: ForOfAssignmentIr {
                        mode: BindingMode::Const,
                        name: format!("value{index}"),
                    },
                    async_plan: None,
                    protocol: IteratorProtocolWitness::SYNC_ITERATOR_PROTOCOL,
                },
                iterable: TypedExpr::undefined(),
                body: Box::new(body),
                lexical_environment: None,
            }
        });
        let actual = count_statement_temp_locals(&statement);
        let expected = DEPTH * SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS
            + FOR_OF_ITERATOR_HELPER_TEMP_LOCALS;

        assert!(
            expected > 2048,
            "regression must cross the local-count floor"
        );
        assert_eq!(actual, expected);
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
                function.protocol.flavor() == FunctionFlavor::Arrow
                    && function.captures_private_environment
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

    /// The rooting walk must terminate on a cyclic dependency, and must still
    /// root both ends of the cycle.
    ///
    /// `Temporal.PlainDateTime`'s arm requires `TemporalZonedDateTimeConstructor`
    /// (its `toZonedDateTime` returns one) and `Temporal.ZonedDateTime`'s arm
    /// requires `TemporalPlainDateTimeConstructor` (its `toPlainDateTime`
    /// returns one). Before the `walked` guard this pair recursed until the
    /// engine's 64 MiB worker stack was gone and the process died with SIGABRT
    /// — from `print(typeof Temporal.ZonedDateTime)`, from
    /// `print(typeof globalThis)`, and from any test262 case containing
    /// `var global = this;`, which is how it took down a full-suite sweep and
    /// aborted the whole `--test cli` process ten tests in.
    ///
    /// A stack overflow aborts the test binary rather than failing one test, so
    /// a regression here is loud: the whole `lila-aot-wasm` lib target dies.
    /// That is the intended behaviour of this test, not a flaw in it — there is
    /// no way to catch a blown guard page in-process.
    ///
    /// Batch 6 added five more entry points, and it is worth being exact about
    /// what they buy, because the comment here first claimed more. All 28
    /// ZonedDateTime members share ONE or-pattern arm (`:2225`) whose body does
    /// not branch per member, so entering at
    /// `...PrototypeAdd` executes byte-identical code to entering at
    /// `...PrototypeEraGetter` — the only difference is which id
    /// `standard_roots.insert` receives first. **These are shape assertions
    /// that the arithmetic members really are in that arm, not a longer walk.**
    /// A member left out of the arm is already a compile error, since the
    /// `match` over `StandardBuiltinId` is exhaustive; what these entries pin is
    /// that nobody moves them into an arm that does not root both ends.
    ///
    /// The entry point that would give genuinely new cycle coverage is a
    /// `Temporal.Duration.prototype.round`/`total` member, on the day that arm
    /// grows the `relativeTo` edge — see the note at the Duration root in the
    /// ZonedDateTime arm. Non-termination shows up as a SIGABRT in a
    /// whole-suite sweep, never as a red unit test.
    ///
    /// `TemporalDurationConstructor` is deliberately **not** an entry point
    /// here: its own arm is a leaf (it inserts its family directly into
    /// `standard_roots` and requires nothing), so entering there roots neither
    /// end of the cycle and the two assertions below would be false. That is
    /// the current shape of the Duration arm, not an invariant — if it ever
    /// grows a `require_standard_builtin(TemporalZonedDateTimeConstructor)` for
    /// `relativeTo`, add it to this list at the same time.
    #[test]
    fn a_cyclic_rooting_dependency_terminates_and_roots_both_ends() {
        for entry in [
            StandardBuiltinId::TemporalZonedDateTimeConstructor,
            StandardBuiltinId::TemporalPlainDateTimeConstructor,
            StandardBuiltinId::TemporalZonedDateTimePrototypeEraGetter,
            StandardBuiltinId::TemporalZonedDateTimePrototypeEraYearGetter,
            StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime,
            StandardBuiltinId::TemporalZonedDateTimePrototypeAdd,
            StandardBuiltinId::TemporalZonedDateTimePrototypeSubtract,
            StandardBuiltinId::TemporalZonedDateTimePrototypeUntil,
            StandardBuiltinId::TemporalZonedDateTimePrototypeSince,
            StandardBuiltinId::TemporalZonedDateTimePrototypeWithCalendar,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(entry);

            assert!(
                plan.standard_roots.contains(&entry),
                "the entry point itself must be rooted: {entry:?}"
            );
            // Both ends of the cycle, whichever end we entered from. This is
            // the half that a naive `if !standard_roots.insert(..) { return }`
            // guard would have been free to drop.
            assert!(
                plan.standard_roots
                    .contains(&StandardBuiltinId::TemporalZonedDateTimeConstructor),
                "entering at {entry:?} must still root the ZonedDateTime constructor"
            );
            assert!(
                plan.standard_roots
                    .contains(&StandardBuiltinId::TemporalPlainDateTimeConstructor),
                "entering at {entry:?} must still root the PlainDateTime constructor"
            );
            assert!(
                plan.temporal_namespace_members().is_some(),
                "entering at {entry:?} must root the complete Temporal namespace"
            );
        }
    }

    /// What `zdt.add`/`subtract`/`until`/`since` must find already emitted.
    ///
    /// **The Duration assertion is pre-satisfied and is deliberately not the
    /// point of this test.** An earlier version of this doc claimed
    /// `a_cyclic_rooting_dependency_terminates_and_roots_both_ends` "would still
    /// pass with the Duration line removed — hence a second test". That is
    /// false, and so is the implied "the Duration root has no other witness":
    /// the ZonedDateTime arm's first statement requires
    /// `TemporalPlainDateTimeConstructor`, whose arm's first statement requires
    /// `TemporalDurationConstructor`, so Duration is rooted transitively for
    /// every ZonedDateTime member with or without that line — and was before
    /// batch 6. It is kept below as a statement of what these four bodies need,
    /// not as coverage of a line that could regress.
    ///
    /// The real coverage here is the delegation list: these bodies are
    /// composition, they look their delegates up through `emit_direct_js_call`
    /// on `self.functions`, and a delegate that is not rooted is an
    /// `EmitError::unsupported` at emit time rather than a compile error. That
    /// half is a genuine contract statement for a shape the type system does
    /// not cover.
    #[test]
    fn zoned_date_time_arithmetic_roots_the_duration_family_it_allocates() {
        for entry in [
            StandardBuiltinId::TemporalZonedDateTimePrototypeAdd,
            StandardBuiltinId::TemporalZonedDateTimePrototypeSubtract,
            StandardBuiltinId::TemporalZonedDateTimePrototypeUntil,
            StandardBuiltinId::TemporalZonedDateTimePrototypeSince,
        ] {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(entry);

            assert!(
                plan.standard_roots
                    .contains(&StandardBuiltinId::TemporalDurationConstructor),
                "{entry:?} allocates or reads a Temporal.Duration and must root its constructor \
                 (pre-satisfied transitively via TemporalPlainDateTimeConstructor; see this \
                 test's doc comment before treating a failure here as a Duration-line regression)"
            );
            // The bodies delegate to the PlainDateTime namesakes rather than
            // re-deriving the arithmetic, so those four have to be emitted, not
            // merely their constructor.
            for delegate in [
                StandardBuiltinId::TemporalPlainDateTimePrototypeAdd,
                StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract,
                StandardBuiltinId::TemporalPlainDateTimePrototypeUntil,
                StandardBuiltinId::TemporalPlainDateTimePrototypeSince,
                StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime,
                StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime,
                StandardBuiltinId::TemporalZonedDateTimeFrom,
            ] {
                assert!(
                    plan.standard_roots.contains(&delegate),
                    "{entry:?} delegates to {delegate:?}, which must be rooted"
                );
            }
        }

        // `withCalendar` does *not* delegate — it rewrites the record in place
        // — so it is deliberately not in the loop above. It still needs the
        // ZonedDateTime prototype global, which its own arm roots.
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(
            StandardBuiltinId::TemporalZonedDateTimePrototypeWithCalendar,
        );
        assert!(plan
            .standard_roots
            .contains(&StandardBuiltinId::TemporalZonedDateTimeConstructor));
    }

    /// Requiring the same builtin twice must be a no-op the second time, and
    /// must not shrink the answer.
    ///
    /// `walked` makes the second call return early, so this pins the claim that
    /// the arms are idempotent: walking once yields the same `standard_roots`
    /// as walking repeatedly.
    #[test]
    fn requiring_a_builtin_twice_is_idempotent() {
        let mut once = RuntimeBootstrapPlan::default();
        once.require_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor);

        let mut twice = RuntimeBootstrapPlan::default();
        twice.require_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor);
        twice.require_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor);

        assert_eq!(once.standard_roots, twice.standard_roots);
        assert_eq!(once.temporal, twice.temporal);
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
    fn math_sum_precise_roots_sync_iterator_machinery() {
        let mut plan = RuntimeBootstrapPlan::default();
        plan.require_standard_builtin(StandardBuiltinId::MathSumPrecise);

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
            StandardBuiltinId::ObjectPrototypeHasOwnProperty,
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
        assert!(matches!(
            &script.body.statements[0],
            StatementIr::Expression(_)
        ));

        assert!(script_references_standard_builtin(
            &script,
            StandardBuiltinId::RegExpConstructor
        ));
        assert!(!should_stub_standard_builtin(
            &script,
            StandardBuiltinId::RegExpConstructor
        ));
    }

    #[test]
    fn wasm_aot_harness_realm_fields_are_the_full_global_bootstrap_roots() {
        let create_realm = lower_script(
            "var $262 = { createRealm: function () { return __lilaCreateRealm(); } };",
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

    #[test]
    fn every_installed_intl_namespace_member_is_rooted_by_the_namespace_plan() {
        // Kept only for its message. The containment itself is now a `const _`
        // block beside `INTL_NAMESPACE_ROOTS`, so the drift this used to catch
        // at `cargo test` no longer compiles; do not read this test's presence
        // as meaning the check lives at test time.
        for (name, builtin) in INTL_NAMESPACE_CONSTRUCTORS {
            assert!(
                INTL_NAMESPACE_ROOTS.contains(builtin),
                "`Intl.{name}` is installed from `INTL_NAMESPACE_CONSTRUCTORS` but \
                 `INTL_NAMESPACE_ROOTS` never roots `{}`",
                builtin.debug_name()
            );
        }
    }

    #[test]
    fn rooting_any_intl_builtin_installs_the_whole_namespace() {
        // Reaching one member of the family — say `Intl.DateTimeFormat` through
        // a folded member expression, with no bare `Intl` anywhere — must still
        // produce an `Intl` object carrying everything the shape declares.
        for entry_point in INTL_NAMESPACE_ROOTS {
            let mut plan = RuntimeBootstrapPlan::default();
            plan.require_standard_builtin(entry_point);

            assert!(
                plan.intl_namespace_members().is_some(),
                "rooting `{}` must install the `Intl` namespace object",
                entry_point.debug_name()
            );
            for builtin in INTL_NAMESPACE_ROOTS {
                assert!(
                    plan.should_initialize_standard_builtin(builtin),
                    "rooting `{}` must also root `{}`",
                    entry_point.debug_name(),
                    builtin.debug_name()
                );
            }
        }
    }

    #[test]
    fn a_bare_intl_reference_roots_every_intl_namespace_member() {
        // `intl402/DateTimeFormat/prop-desc.js` reaches `DateTimeFormat` only
        // through `verifyProperty(Intl, "DateTimeFormat", ...)`, so it never
        // appears as a member expression and never reaches
        // `compiled_standard_builtins`. The namespace binding is the only thing
        // that can root it.
        let script = lower_script(r#"verifyProperty(Intl, "DateTimeFormat", {});"#);
        let plan = RuntimeBootstrapPlan::from_script(&script, &[]);

        assert!(
            !plan.full_standard_globals,
            "this script must exercise the namespace-binding path, not the full bootstrap"
        );
        assert!(plan.should_install_script_global_binding(&GlobalPropertyInitializerIr::IntlObject));
        assert!(plan.intl_namespace_members().is_some());
        for builtin in INTL_NAMESPACE_ROOTS {
            assert!(
                plan.should_initialize_standard_builtin(builtin),
                "a bare `Intl` reference must root `{}`",
                builtin.debug_name()
            );
        }
    }

    /// A bare namespace reference must root both levels of the shape the IR
    /// assigns to it: every constructor on `Temporal` and every function on
    /// `Temporal.Now`.
    #[test]
    fn a_bare_temporal_reference_roots_its_declared_namespace_shape() {
        let script = lower_script("var namespace = Temporal;");
        let plan = RuntimeBootstrapPlan::from_script(&script, &[]);

        assert!(!plan.full_standard_globals);
        assert!(
            plan.should_install_script_global_binding(&GlobalPropertyInitializerIr::TemporalObject)
        );
        assert!(plan.temporal_namespace_members().is_some());
        for (name, declared) in TEMPORAL_NAMESPACE_CONSTRUCTORS
            .iter()
            .chain(TEMPORAL_NOW_NAMESPACE_MEMBERS)
        {
            assert!(
                plan.should_initialize_standard_builtin(*declared),
                "a bare `Temporal` reference must root `{name}` (`{}`), which its declared \
                 namespace shape carries unconditionally",
                declared.debug_name()
            );
        }
    }

    /// No `Intl` builtin may exist outside the two namespace lists.
    ///
    /// The three checks around this one partition as CONSTRUCTORS ⊆ ROOTS,
    /// ROOTS ⊆ the `require_standard_builtin` match arm, and (at debug runtime
    /// only) match arm ⊆ ROOTS. None of them can see the drift that actually
    /// happens next: a *new* `Intl` builtin — `Intl.NumberFormat` is the
    /// obvious follow-on — added to neither list. Its symptom is precisely what
    /// this lane exists to prevent: a builtin reachable as a member expression
    /// but never rooted from a bare `Intl`, so `intl402/**/prop-desc.js`-shaped
    /// reflective reads find it missing.
    ///
    /// `debug_name()` spells every `Intl` id with an `Intl.` prefix, which is
    /// what makes the total direction checkable at all without a fourth list.
    #[test]
    fn every_intl_standard_builtin_is_in_the_namespace_root_list() {
        for builtin in StandardBuiltinId::all_functions() {
            if !builtin.debug_name().contains("Intl.") {
                continue;
            }
            assert!(
                INTL_NAMESPACE_ROOTS.contains(builtin),
                "`{}` is an `Intl` builtin but appears in no namespace list, so a bare \
                 `Intl` reference will not root it",
                builtin.debug_name()
            );
        }
    }
}

/// Every builtin the `Intl` namespace object depends on, declared exactly once.
///
/// Two independent obligations force a single list:
///
/// - `ScriptLowerer::intl_object_value_info` puts every
///   `INTL_NAMESPACE_CONSTRUCTORS` member on the `Intl` shape
///   *unconditionally*. An `Intl` object missing one of them contradicts the
///   shape the same program was compiled against, which is invisible to any
///   test that only reaches the member through a folded member expression and
///   visible to every test that reads it reflectively — that is
///   `intl402/DateTimeFormat/prop-desc.js`.
/// - `Intl.DateTimeFormat` is not a usable constructor without
///   `supportedLocalesOf`, its prototype accessors and its bound-format body,
///   and the five `Temporal.Plain*.prototype.toLocaleString` emitters look the
///   constructor and the format getter up by function id and return
///   `EmitError::unsupported` when either is missing.
///
/// `require_standard_builtin` does not recurse through its own match, so a
/// caller that needs the formatter must seed every id itself rather than
/// relying on one id dragging in the rest.
const INTL_NAMESPACE_ROOTS: [StandardBuiltinId; 15] = [
    StandardBuiltinId::IntlGetCanonicalLocales,
    StandardBuiltinId::IntlLocaleConstructor,
    StandardBuiltinId::IntlLocalePrototypeLanguageGetter,
    StandardBuiltinId::IntlLocalePrototypeScriptGetter,
    StandardBuiltinId::IntlLocalePrototypeRegionGetter,
    StandardBuiltinId::IntlLocalePrototypeBaseNameGetter,
    StandardBuiltinId::IntlLocalePrototypeToString,
    StandardBuiltinId::IntlDateTimeFormatConstructor,
    StandardBuiltinId::IntlDateTimeFormatSupportedLocalesOf,
    StandardBuiltinId::IntlDateTimeFormatPrototypeResolvedOptions,
    StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter,
    StandardBuiltinId::IntlDateTimeFormatPrototypeFormatToParts,
    StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRange,
    StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRangeToParts,
    StandardBuiltinId::IntlDateTimeFormatBoundFormat,
];

/// `INTL_NAMESPACE_CONSTRUCTORS` ⊆ [`INTL_NAMESPACE_ROOTS`], checked by the
/// compiler rather than by a test.
///
/// This is the one link the two types cannot carry. [`IntlNamespacePlan::rooted`]
/// seeds `INTL_NAMESPACE_ROOTS` and [`IntlNamespacePlan::members`] hands out
/// `INTL_NAMESPACE_CONSTRUCTORS`, which lives in `lila-ir`; nothing relates
/// them, so [`IntlNamespaceMembers`]'s "every member is rooted" claim rests on
/// this containment alone. Adding `Intl.NumberFormat` to the shape list and
/// forgetting the root list is the plausible mistake, and it is not a missing
/// property: `init_intl_object` walks the member list with no per-member guard,
/// so an unseeded member becomes a `GlobalGet` on a global that is never
/// `GlobalSet`, paired with an unconditional `Function` tag — a callable with a
/// zero payload.
///
/// `StandardBuiltinId` is fieldless, so `as u32` is a total, `const`-evaluable
/// identity for it and the whole check runs at compile time.
const _: () = {
    let mut member = 0;
    while member < INTL_NAMESPACE_CONSTRUCTORS.len() {
        let needle = INTL_NAMESPACE_CONSTRUCTORS[member].1 as u32;
        let mut root = 0;
        let mut found = false;
        while root < INTL_NAMESPACE_ROOTS.len() {
            if INTL_NAMESPACE_ROOTS[root] as u32 == needle {
                found = true;
            }
            root += 1;
        }
        assert!(
            found,
            "an `Intl` namespace member declared on the IR shape is missing from \
             `INTL_NAMESPACE_ROOTS`, so a bare `Intl` reference would install it as a \
             `Function`-tagged value with a never-written payload"
        );
        member += 1;
    }
};

pub(crate) use intl_namespace::{IntlNamespaceMembers, IntlNamespacePlan};
pub(crate) use temporal_namespace::{TemporalNamespaceMembers, TemporalNamespacePlan};

/// The `Intl` namespace plan and its member-list witness, in a module of their
/// own.
///
/// The module boundary is the enforcement. Both types were previously declared
/// beside [`RuntimeBootstrapPlan`], where their doc comments claimed the
/// installed variant could only come from the seeding constructor — true for
/// every module *except* the 6,000-line one where the next namespace-rooting
/// arm actually gets written, and in which
/// `self.intl = IntlNamespacePlan::RootedWithDateTimeFormatFamily;` compiled
/// fine. Here the installed variant carries a payload whose field is private to
/// this module and [`IntlNamespaceMembers`] has no constructor at all, so
/// neither can be built anywhere else in the crate, `planning` included.
///
/// The failure that shape prevents is not a missing property. `init_intl_object`
/// walks the member list with no per-member guard (deliberately — the guard it
/// used to have silently `continue`d past unrooted members), emitting
/// `GlobalGet` for each constructor's global paired with an unconditional
/// `Function` tag. A global that is never `GlobalSet` still resolves, so an
/// unseeded member becomes a `Function`-tagged value with a never-written
/// payload: a bogus callable, which is worse than the absent property the old
/// guard produced.
mod intl_namespace;
mod temporal_namespace;

#[derive(Debug, Clone, Default)]
pub(crate) struct RuntimeBootstrapPlan {
    pub(crate) full_standard_globals: bool,
    pub(crate) standard_roots: BTreeSet<StandardBuiltinId>,
    pub(crate) reflect_object: bool,
    pub(crate) math_object: bool,
    pub(crate) json_object: bool,
    pub(crate) atomics_object: bool,
    /// Private because installing `Temporal` carries the obligation represented
    /// by [`RuntimeBootstrapPlan::temporal_namespace_members`].
    temporal: TemporalNamespacePlan,
    /// Deliberately private for the same reason as `temporal`: namespace
    /// installation carries a rooting obligation, and `planning` is the only
    /// module allowed to discharge it. Read it through
    /// [`RuntimeBootstrapPlan::intl_namespace_members`].
    intl: IntlNamespacePlan,
    /// Which builtins [`RuntimeBootstrapPlan::require_standard_builtin`] has
    /// already *walked*, as opposed to which ones are rooted.
    ///
    /// The dependency graph these arms describe has cycles in it, and one of
    /// them is reachable from a one-line program. `Temporal.PlainDateTime`'s
    /// arm requires `TemporalZonedDateTimeConstructor` (for `toZonedDateTime`)
    /// and `Temporal.ZonedDateTime`'s arm requires
    /// `TemporalPlainDateTimeConstructor` (for `toPlainDateTime`), so
    /// `print(typeof Temporal.ZonedDateTime)` recursed until it exhausted the
    /// engine's 64 MiB worker stack and aborted the process with SIGABRT. Top
    /// level `this` and `globalThis` reach it too, because they root every
    /// global; that is how a `var global = this;` case in
    /// `built-ins/Array/prototype/map` came to kill a full-suite sweep.
    ///
    /// This must NOT be replaced by guarding on `standard_roots` itself, however
    /// tempting the one-liner is. Several arms below add their dependencies with
    /// a bare `standard_roots.insert(dep)` rather than a recursive `require`, so
    /// a builtin can already be *rooted* without ever having been *walked*.
    /// Guarding on `standard_roots` would then skip that builtin's arm the first
    /// time anything genuinely required it, silently dropping every root the arm
    /// would have added — trading an abort for a wrong answer.
    ///
    /// Walking each builtin exactly once reaches the same fixpoint as walking it
    /// repeatedly: every effect in these arms is a set insertion or a `= true`
    /// flag, so they are idempotent and order-independent.
    walked: BTreeSet<StandardBuiltinId>,
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
                plan.require_script_global_binding(&binding.initializer);
            }
        }
        if script
            .functions
            .iter()
            .any(|function| function.protocol.execution_kind() == FunctionExecutionKind::Async)
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
        initializer: &GlobalPropertyInitializerIr,
    ) -> bool {
        match initializer {
            GlobalPropertyInitializerIr::Intrinsic
            | GlobalPropertyInitializerIr::Infinity
            | GlobalPropertyInitializerIr::NaN
            | GlobalPropertyInitializerIr::Undefined
            | GlobalPropertyInitializerIr::FreshUndefined
            | GlobalPropertyInitializerIr::SourceFunction(_)
            | GlobalPropertyInitializerIr::HostFunction(_) => true,
            GlobalPropertyInitializerIr::ReflectObject => {
                self.full_standard_globals || self.reflect_object
            }
            GlobalPropertyInitializerIr::MathObject => {
                self.full_standard_globals || self.math_object
            }
            GlobalPropertyInitializerIr::JsonObject => {
                self.full_standard_globals || self.json_object
            }
            GlobalPropertyInitializerIr::AtomicsObject => {
                self.full_standard_globals || self.atomics_object
            }
            GlobalPropertyInitializerIr::TemporalObject => {
                self.temporal_namespace_members().is_some()
            }
            GlobalPropertyInitializerIr::IntlObject => self.intl_namespace_members().is_some(),
            GlobalPropertyInitializerIr::BuiltinFunction(builtin) => {
                self.should_initialize_standard_builtin(*builtin)
            }
        }
    }

    /// The `Intl` namespace members to install, or `None` when this program
    /// never gets an `Intl` object.
    ///
    /// This is the single gate `builtins::bootstrap` consults: it calls
    /// `init_intl_object` only while holding the returned witness, and the
    /// witness is the only source of the member list. There is deliberately no
    /// bool accessor beside it — a caller that can ask "is `Intl` installed?"
    /// without receiving the proof is a caller that can install a partial one.
    pub(crate) fn intl_namespace_members(&self) -> Option<IntlNamespaceMembers> {
        self.intl.members(self.full_standard_globals)
    }

    /// The complete `Temporal` namespace member witness, or `None` when this
    /// plan emits no `Temporal` object.
    pub(crate) fn temporal_namespace_members(&self) -> Option<TemporalNamespaceMembers> {
        self.temporal.members(self.full_standard_globals)
    }

    pub(crate) fn needs_typed_array_intrinsic(&self) -> bool {
        self.full_standard_globals
            || self.standard_roots.iter().any(|builtin| {
                *builtin == StandardBuiltinId::TypedArrayConstructor
                    || is_typed_array_constructor(*builtin)
            })
    }

    fn require_script_global_binding(&mut self, initializer: &GlobalPropertyInitializerIr) {
        match initializer {
            GlobalPropertyInitializerIr::ReflectObject => self.reflect_object = true,
            GlobalPropertyInitializerIr::MathObject => self.math_object = true,
            GlobalPropertyInitializerIr::JsonObject => self.json_object = true,
            GlobalPropertyInitializerIr::AtomicsObject => self.atomics_object = true,
            GlobalPropertyInitializerIr::TemporalObject => {
                self.require_temporal_namespace();
            }
            GlobalPropertyInitializerIr::IntlObject => {
                // A bare `Intl` reference gets the namespace object
                // `ScriptLowerer::intl_object_value_info` describes, and that
                // shape declares every `INTL_NAMESPACE_CONSTRUCTORS` member
                // unconditionally — including `DateTimeFormat`, which
                // `intl402/DateTimeFormat/prop-desc.js` only ever reaches
                // through `verifyProperty(Intl, "DateTimeFormat", ...)`, never
                // as a member expression, so it never lands in
                // `compiled_standard_builtins`. Rooting the family here is what
                // makes the emitted object and the compiled-against shape agree.
                //
                // This arm used to say
                // `require_standard_builtin(IntlLocaleConstructor)` plus
                // `require_standard_builtin(IntlGetCanonicalLocales)`, which
                // reaches the identical root set — but only by way of the Intl
                // arm four hundred lines down inside `require_standard_builtin`.
                // Same set, stated locally.
                self.require_intl_date_time_format_family();
            }
            GlobalPropertyInitializerIr::BuiltinFunction(builtin) => {
                self.require_standard_builtin(*builtin);
            }
            GlobalPropertyInitializerIr::Intrinsic
            | GlobalPropertyInitializerIr::Infinity
            | GlobalPropertyInitializerIr::NaN
            | GlobalPropertyInitializerIr::Undefined
            | GlobalPropertyInitializerIr::FreshUndefined
            | GlobalPropertyInitializerIr::SourceFunction(_)
            | GlobalPropertyInitializerIr::HostFunction(_) => {}
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

    /// Root the whole `Intl.DateTimeFormat` family and mark the namespace object
    /// as installed.
    ///
    /// Both effects come from one assignment because [`IntlNamespacePlan`] has
    /// no other way to reach its installed state: the seeding happens inside the
    /// only constructor of that variant. See [`INTL_NAMESPACE_ROOTS`] for why
    /// the family is all-or-nothing.
    fn require_intl_date_time_format_family(&mut self) {
        self.intl = IntlNamespacePlan::rooted(&mut self.standard_roots);
    }

    /// Root both IR-advertised Temporal namespace levels before making the
    /// namespace available to bootstrap.
    fn require_temporal_namespace(&mut self) {
        TemporalNamespacePlan::root(self);
    }

    fn require_standard_builtin(&mut self, builtin: StandardBuiltinId) {
        self.standard_roots.insert(builtin);
        // Rooting is unconditional above; *walking* happens at most once. See
        // the `walked` field's documentation for why the guard cannot live on
        // `standard_roots`, and for the cycle that makes it necessary at all.
        if !self.walked.insert(builtin) {
            return;
        }
        if builtin == StandardBuiltinId::FunctionConstructor {
            // `%Function.prototype%` is a callable intrinsic, not an Object
            // shell, and its non-configurable `@@hasInstance` property publishes
            // another exact function value. Root both bodies whenever the
            // foundational Function family is present so bootstrap cannot
            // publish either a stubbed call target or a half-installed
            // prototype.
            for dependency in [
                StandardBuiltinId::FunctionPrototype,
                StandardBuiltinId::FunctionPrototypeSymbolHasInstance,
            ] {
                self.require_standard_builtin(dependency);
            }
        }
        if builtin == StandardBuiltinId::DisposableStackConstructor {
            // The constructor installer publishes the complete prototype as one
            // intrinsic unit. Root every function value it reads so an emitted
            // constructor can never observe a half-installed prototype.
            for dependency in [
                StandardBuiltinId::DisposableStackPrototypeUse,
                StandardBuiltinId::DisposableStackPrototypeAdopt,
                StandardBuiltinId::DisposableStackPrototypeDefer,
                StandardBuiltinId::DisposableStackPrototypeMove,
                StandardBuiltinId::DisposableStackPrototypeDispose,
                StandardBuiltinId::DisposableStackPrototypeDisposedGetter,
            ] {
                self.require_standard_builtin(dependency);
            }
        }
        if builtin == StandardBuiltinId::DisposableStackPrototypeDispose {
            // DisposeResources constructs this intrinsic when a later disposer
            // suppresses an earlier abrupt completion.
            self.require_standard_builtin(StandardBuiltinId::SuppressedErrorConstructor);
        }
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
            self.require_standard_builtin(StandardBuiltinId::TypedArrayConstructor);
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
                | StandardBuiltinId::MathSumPrecise
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
                | StandardBuiltinId::ObjectPrototypeHasOwnProperty
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
            StandardBuiltinId::FunctionPrototypeSymbolHasInstance => {
                // This body is installed only by the Function constructor
                // intrinsic family. Walking back to its owner closes direct IR
                // references without leaving property installation optional.
                self.require_standard_builtin(StandardBuiltinId::FunctionConstructor);
            }
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
            | StandardBuiltinId::IntlLocalePrototypeToString
            | StandardBuiltinId::IntlDateTimeFormatConstructor
            | StandardBuiltinId::IntlDateTimeFormatSupportedLocalesOf
            | StandardBuiltinId::IntlDateTimeFormatPrototypeResolvedOptions
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatToParts
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRange
            | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRangeToParts
            | StandardBuiltinId::IntlDateTimeFormatBoundFormat => {
                // This or-pattern and `INTL_NAMESPACE_ROOTS` are two spellings
                // of the same set, and only one of them can be a `match`
                // pattern. The assertion pins the direction the types cannot:
                // an id matched here but absent from the const would be rooted
                // without its namespace being installable.
                //
                // It is a backstop, not the primary check, and it is compiled
                // out of release builds: `every_intl_standard_builtin_is_in_the_
                // namespace_root_list` covers the same drift at rung 1 by
                // walking `all_functions()`, which no arm can hide from.
                debug_assert!(
                    INTL_NAMESPACE_ROOTS.contains(&builtin),
                    "`{}` is matched as an `Intl` namespace builtin but is not in \
                     `INTL_NAMESPACE_ROOTS`",
                    builtin.debug_name()
                );
                self.require_intl_date_time_format_family();
            }
            // Any Temporal entry first closes the namespace contract. The
            // private Rooting state makes these recursive calls no-ops until
            // all advertised constructors and `Now` members are rooted.
            StandardBuiltinId::TemporalNowTimeZoneId => {
                self.require_temporal_namespace();
            }
            StandardBuiltinId::TemporalNowInstant => {
                self.require_temporal_namespace();
                self.require_standard_builtin(StandardBuiltinId::TemporalInstantConstructor);
            }
            StandardBuiltinId::TemporalNowZonedDateTimeIso => {
                self.require_temporal_namespace();
                self.require_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor);
            }
            // The whole `Temporal.PlainDate` family installs together: the
            // prototype is built once, so rooting one member without the rest
            // would leave the object half-populated.
            StandardBuiltinId::TemporalPlainDateConstructor
            | StandardBuiltinId::TemporalPlainDateFrom
            | StandardBuiltinId::TemporalPlainDateCompare
            | StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeEraGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainDatePrototypeWith
            | StandardBuiltinId::TemporalPlainDatePrototypeWithCalendar
            | StandardBuiltinId::TemporalPlainDatePrototypeEquals
            | StandardBuiltinId::TemporalPlainDatePrototypeToString
            | StandardBuiltinId::TemporalPlainDatePrototypeToJson
            | StandardBuiltinId::TemporalPlainDatePrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainDatePrototypeAdd
            | StandardBuiltinId::TemporalPlainDatePrototypeSubtract
            | StandardBuiltinId::TemporalPlainDatePrototypeUntil
            | StandardBuiltinId::TemporalPlainDatePrototypeSince
            | StandardBuiltinId::TemporalPlainDatePrototypeToPlainDateTime
            | StandardBuiltinId::TemporalPlainDatePrototypeToPlainYearMonth
            | StandardBuiltinId::TemporalPlainDatePrototypeToPlainMonthDay
            | StandardBuiltinId::TemporalPlainDatePrototypeValueOf => {
                self.require_temporal_namespace();
                for dependency in [
                    StandardBuiltinId::TemporalPlainDateConstructor,
                    StandardBuiltinId::TemporalPlainDateFrom,
                    StandardBuiltinId::TemporalPlainDateCompare,
                    StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeEraGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeYearGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeDayGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter,
                    StandardBuiltinId::TemporalPlainDatePrototypeWith,
                    StandardBuiltinId::TemporalPlainDatePrototypeWithCalendar,
                    StandardBuiltinId::TemporalPlainDatePrototypeEquals,
                    StandardBuiltinId::TemporalPlainDatePrototypeToString,
                    StandardBuiltinId::TemporalPlainDatePrototypeToJson,
                    StandardBuiltinId::TemporalPlainDatePrototypeToLocaleString,
                    StandardBuiltinId::TemporalPlainDatePrototypeValueOf,
                    StandardBuiltinId::TemporalPlainDatePrototypeAdd,
                    StandardBuiltinId::TemporalPlainDatePrototypeSubtract,
                    StandardBuiltinId::TemporalPlainDatePrototypeUntil,
                    StandardBuiltinId::TemporalPlainDatePrototypeSince,
                    StandardBuiltinId::TemporalPlainDatePrototypeToPlainDateTime,
                    StandardBuiltinId::TemporalPlainDatePrototypeToPlainYearMonth,
                    StandardBuiltinId::TemporalPlainDatePrototypeToPlainMonthDay,
                ] {
                    self.standard_roots.insert(dependency);
                }
                // `Temporal.PlainDate.prototype.toLocaleString` builds an
                // `Intl.DateTimeFormat` and calls its bound format function.
                self.require_intl_date_time_format_family();
            }
            // The whole `Temporal.PlainYearMonth` family installs together: one shared prototype, and `until`/`since`/`toPlainDate` hand back sibling types.
            StandardBuiltinId::TemporalPlainYearMonthConstructor
            | StandardBuiltinId::TemporalPlainYearMonthFrom
            | StandardBuiltinId::TemporalPlainYearMonthCompare
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthsInYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeWith
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeAdd
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeSubtract
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeUntil
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeSince
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeEquals
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToString
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToJson
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeValueOf
            | StandardBuiltinId::TemporalPlainYearMonthPrototypeToPlainDate => {
                self.require_temporal_namespace();
                self.require_standard_builtin(StandardBuiltinId::TemporalDurationConstructor);
                self.require_standard_builtin(StandardBuiltinId::TemporalPlainDateConstructor);
                for dependency in [
                    StandardBuiltinId::TemporalPlainYearMonthConstructor,
                    StandardBuiltinId::TemporalPlainYearMonthFrom,
                    StandardBuiltinId::TemporalPlainYearMonthCompare,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeCalendarIdGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeEraGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeEraYearGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeYearGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthCodeGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInYearGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInMonthGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthsInYearGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeInLeapYearGetter,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeWith,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeAdd,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeSubtract,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeUntil,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeSince,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeEquals,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeToString,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeToJson,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeToLocaleString,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeValueOf,
                    StandardBuiltinId::TemporalPlainYearMonthPrototypeToPlainDate,
                ] {
                    self.standard_roots.insert(dependency);
                }
                // `Temporal.PlainYearMonth.prototype.toLocaleString` builds an
                // `Intl.DateTimeFormat` and calls its bound format function.
                self.require_intl_date_time_format_family();
            }
            // The whole `Temporal.PlainMonthDay` family installs together; `toPlainDate` hands back a `Temporal.PlainDate`.
            StandardBuiltinId::TemporalPlainMonthDayConstructor
            | StandardBuiltinId::TemporalPlainMonthDayFrom
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeDayGetter
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeWith
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeEquals
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToString
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToJson
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeValueOf
            | StandardBuiltinId::TemporalPlainMonthDayPrototypeToPlainDate => {
                self.require_temporal_namespace();
                self.require_standard_builtin(StandardBuiltinId::TemporalPlainDateConstructor);
                for dependency in [
                    StandardBuiltinId::TemporalPlainMonthDayConstructor,
                    StandardBuiltinId::TemporalPlainMonthDayFrom,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeDayGetter,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeWith,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeEquals,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeToString,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeToJson,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeToLocaleString,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeValueOf,
                    StandardBuiltinId::TemporalPlainMonthDayPrototypeToPlainDate,
                ] {
                    self.standard_roots.insert(dependency);
                }
                // `Temporal.PlainMonthDay.prototype.toLocaleString` builds an
                // `Intl.DateTimeFormat` and calls its bound format function.
                self.require_intl_date_time_format_family();
            }
            // The whole `Temporal.PlainTime` family installs together, for the
            // same reason `Temporal.PlainDate` does: one shared prototype.
            StandardBuiltinId::TemporalPlainTimeConstructor
            | StandardBuiltinId::TemporalPlainTimeFrom
            | StandardBuiltinId::TemporalPlainTimeCompare
            | StandardBuiltinId::TemporalPlainTimePrototypeHourGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeNanosecondGetter
            | StandardBuiltinId::TemporalPlainTimePrototypeWith
            | StandardBuiltinId::TemporalPlainTimePrototypeAdd
            | StandardBuiltinId::TemporalPlainTimePrototypeSubtract
            | StandardBuiltinId::TemporalPlainTimePrototypeUntil
            | StandardBuiltinId::TemporalPlainTimePrototypeSince
            | StandardBuiltinId::TemporalPlainTimePrototypeRound
            | StandardBuiltinId::TemporalPlainTimePrototypeEquals
            | StandardBuiltinId::TemporalPlainTimePrototypeToString
            | StandardBuiltinId::TemporalPlainTimePrototypeToJson
            | StandardBuiltinId::TemporalPlainTimePrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainTimePrototypeValueOf => {
                self.require_temporal_namespace();
                // `until`/`since` hand back a `Temporal.Duration`, and `add`
                // and `subtract` read one, so the Duration family has to come
                // along whenever any PlainTime member does.
                self.require_standard_builtin(StandardBuiltinId::TemporalDurationConstructor);
                for dependency in [
                    StandardBuiltinId::TemporalPlainTimeConstructor,
                    StandardBuiltinId::TemporalPlainTimeFrom,
                    StandardBuiltinId::TemporalPlainTimeCompare,
                    StandardBuiltinId::TemporalPlainTimePrototypeHourGetter,
                    StandardBuiltinId::TemporalPlainTimePrototypeMinuteGetter,
                    StandardBuiltinId::TemporalPlainTimePrototypeSecondGetter,
                    StandardBuiltinId::TemporalPlainTimePrototypeMillisecondGetter,
                    StandardBuiltinId::TemporalPlainTimePrototypeMicrosecondGetter,
                    StandardBuiltinId::TemporalPlainTimePrototypeNanosecondGetter,
                    StandardBuiltinId::TemporalPlainTimePrototypeWith,
                    StandardBuiltinId::TemporalPlainTimePrototypeAdd,
                    StandardBuiltinId::TemporalPlainTimePrototypeSubtract,
                    StandardBuiltinId::TemporalPlainTimePrototypeUntil,
                    StandardBuiltinId::TemporalPlainTimePrototypeSince,
                    StandardBuiltinId::TemporalPlainTimePrototypeRound,
                    StandardBuiltinId::TemporalPlainTimePrototypeEquals,
                    StandardBuiltinId::TemporalPlainTimePrototypeToString,
                    StandardBuiltinId::TemporalPlainTimePrototypeToJson,
                    StandardBuiltinId::TemporalPlainTimePrototypeToLocaleString,
                    StandardBuiltinId::TemporalPlainTimePrototypeValueOf,
                ] {
                    self.standard_roots.insert(dependency);
                }
                // `Temporal.PlainTime.prototype.toLocaleString` builds an
                // `Intl.DateTimeFormat` and calls its bound format function.
                self.require_intl_date_time_format_family();
            }
            // The whole `Temporal.PlainDateTime` family installs together, for the
            // same reason `Temporal.PlainDate` does: one shared prototype.
            StandardBuiltinId::TemporalPlainDateTimeConstructor
            | StandardBuiltinId::TemporalPlainDateTimeFrom
            | StandardBuiltinId::TemporalPlainDateTimeCompare
            | StandardBuiltinId::TemporalPlainDateTimePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeEraGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeEraYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthCodeGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeHourGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMinuteGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMillisecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMicrosecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeNanosecondGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWeekOfYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeYearOfWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInWeekGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInMonthGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthsInYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeInLeapYearGetter
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWith
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWithPlainTime
            | StandardBuiltinId::TemporalPlainDateTimePrototypeWithCalendar
            | StandardBuiltinId::TemporalPlainDateTimePrototypeAdd
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract
            | StandardBuiltinId::TemporalPlainDateTimePrototypeUntil
            | StandardBuiltinId::TemporalPlainDateTimePrototypeSince
            | StandardBuiltinId::TemporalPlainDateTimePrototypeRound
            | StandardBuiltinId::TemporalPlainDateTimePrototypeEquals
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToString
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToJson
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToLocaleString
            | StandardBuiltinId::TemporalPlainDateTimePrototypeValueOf
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainDate
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainTime
            | StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime => {
                self.require_temporal_namespace();
                // `until`/`since` hand back a `Temporal.Duration` and `add`/`subtract`
                // read one; `toPlainDate`, `toPlainTime` and `toZonedDateTime` hand back
                // the three sibling types. All four families come along.
                self.require_standard_builtin(StandardBuiltinId::TemporalDurationConstructor);
                self.require_standard_builtin(StandardBuiltinId::TemporalPlainDateConstructor);
                self.require_standard_builtin(StandardBuiltinId::TemporalPlainTimeConstructor);
                self.require_standard_builtin(StandardBuiltinId::TemporalZonedDateTimeConstructor);
                for dependency in [
                    StandardBuiltinId::TemporalPlainDateTimeConstructor,
                    StandardBuiltinId::TemporalPlainDateTimeFrom,
                    StandardBuiltinId::TemporalPlainDateTimeCompare,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeCalendarIdGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeEraGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeEraYearGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeYearGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMonthGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMonthCodeGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeDayGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeHourGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMinuteGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeSecondGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMillisecondGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMicrosecondGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeNanosecondGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfWeekGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfYearGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeWeekOfYearGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeYearOfWeekGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInWeekGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInMonthGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInYearGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeMonthsInYearGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeInLeapYearGetter,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeWith,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeWithPlainTime,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeWithCalendar,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeAdd,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeUntil,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeSince,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeRound,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeEquals,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeToString,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeToJson,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeToLocaleString,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeValueOf,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainDate,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainTime,
                    StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime,
                ] {
                    self.standard_roots.insert(dependency);
                }
                // `Temporal.PlainDateTime.prototype.toLocaleString` builds an
                // `Intl.DateTimeFormat` and calls its bound format function.
                self.require_intl_date_time_format_family();
            }
            // The whole `Temporal.Duration` family installs together, for the
            // same reason `Temporal.PlainDate` does: one shared prototype.
            StandardBuiltinId::TemporalDurationConstructor
            | StandardBuiltinId::TemporalDurationFrom
            | StandardBuiltinId::TemporalDurationCompare
            | StandardBuiltinId::TemporalDurationPrototypeYearsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMonthsGetter
            | StandardBuiltinId::TemporalDurationPrototypeWeeksGetter
            | StandardBuiltinId::TemporalDurationPrototypeDaysGetter
            | StandardBuiltinId::TemporalDurationPrototypeHoursGetter
            | StandardBuiltinId::TemporalDurationPrototypeMinutesGetter
            | StandardBuiltinId::TemporalDurationPrototypeSecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter
            | StandardBuiltinId::TemporalDurationPrototypeSignGetter
            | StandardBuiltinId::TemporalDurationPrototypeBlankGetter
            | StandardBuiltinId::TemporalDurationPrototypeWith
            | StandardBuiltinId::TemporalDurationPrototypeNegated
            | StandardBuiltinId::TemporalDurationPrototypeAbs
            | StandardBuiltinId::TemporalDurationPrototypeAdd
            | StandardBuiltinId::TemporalDurationPrototypeSubtract
            | StandardBuiltinId::TemporalDurationPrototypeRound
            | StandardBuiltinId::TemporalDurationPrototypeTotal
            | StandardBuiltinId::TemporalDurationPrototypeToString
            | StandardBuiltinId::TemporalDurationPrototypeToJson
            | StandardBuiltinId::TemporalDurationPrototypeToLocaleString
            | StandardBuiltinId::TemporalDurationPrototypeValueOf => {
                self.require_temporal_namespace();
                for dependency in [
                    StandardBuiltinId::TemporalDurationConstructor,
                    StandardBuiltinId::TemporalDurationFrom,
                    StandardBuiltinId::TemporalDurationCompare,
                    StandardBuiltinId::TemporalDurationPrototypeYearsGetter,
                    StandardBuiltinId::TemporalDurationPrototypeMonthsGetter,
                    StandardBuiltinId::TemporalDurationPrototypeWeeksGetter,
                    StandardBuiltinId::TemporalDurationPrototypeDaysGetter,
                    StandardBuiltinId::TemporalDurationPrototypeHoursGetter,
                    StandardBuiltinId::TemporalDurationPrototypeMinutesGetter,
                    StandardBuiltinId::TemporalDurationPrototypeSecondsGetter,
                    StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter,
                    StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter,
                    StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter,
                    StandardBuiltinId::TemporalDurationPrototypeSignGetter,
                    StandardBuiltinId::TemporalDurationPrototypeBlankGetter,
                    StandardBuiltinId::TemporalDurationPrototypeWith,
                    StandardBuiltinId::TemporalDurationPrototypeNegated,
                    StandardBuiltinId::TemporalDurationPrototypeAbs,
                    StandardBuiltinId::TemporalDurationPrototypeAdd,
                    StandardBuiltinId::TemporalDurationPrototypeSubtract,
                    StandardBuiltinId::TemporalDurationPrototypeRound,
                    StandardBuiltinId::TemporalDurationPrototypeTotal,
                    StandardBuiltinId::TemporalDurationPrototypeToString,
                    StandardBuiltinId::TemporalDurationPrototypeToJson,
                    StandardBuiltinId::TemporalDurationPrototypeToLocaleString,
                    StandardBuiltinId::TemporalDurationPrototypeValueOf,
                ] {
                    self.standard_roots.insert(dependency);
                }
            }
            // The whole `Temporal.Instant` family installs together: the
            // prototype is built once, so rooting one member without the rest
            // would leave the object half-populated.
            StandardBuiltinId::TemporalInstantConstructor
            | StandardBuiltinId::TemporalInstantFrom
            | StandardBuiltinId::TemporalInstantCompare
            | StandardBuiltinId::TemporalInstantFromEpochMilliseconds
            | StandardBuiltinId::TemporalInstantFromEpochNanoseconds
            | StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter
            | StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter
            | StandardBuiltinId::TemporalInstantPrototypeEquals
            | StandardBuiltinId::TemporalInstantPrototypeToString
            | StandardBuiltinId::TemporalInstantPrototypeToJson
            | StandardBuiltinId::TemporalInstantPrototypeValueOf => {
                self.require_temporal_namespace();
                for dependency in [
                    StandardBuiltinId::TemporalInstantConstructor,
                    StandardBuiltinId::TemporalInstantFrom,
                    StandardBuiltinId::TemporalInstantCompare,
                    StandardBuiltinId::TemporalInstantFromEpochMilliseconds,
                    StandardBuiltinId::TemporalInstantFromEpochNanoseconds,
                    StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter,
                    StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter,
                    StandardBuiltinId::TemporalInstantPrototypeEquals,
                    StandardBuiltinId::TemporalInstantPrototypeToString,
                    StandardBuiltinId::TemporalInstantPrototypeToJson,
                    StandardBuiltinId::TemporalInstantPrototypeValueOf,
                ] {
                    self.standard_roots.insert(dependency);
                }
            }
            StandardBuiltinId::TemporalZonedDateTimeConstructor
            | StandardBuiltinId::TemporalZonedDateTimeFrom
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEraGetter
            | StandardBuiltinId::TemporalZonedDateTimePrototypeEraYearGetter
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
            | StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime
            | StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone
            | StandardBuiltinId::TemporalZonedDateTimePrototypeWithCalendar
            | StandardBuiltinId::TemporalZonedDateTimePrototypeAdd
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSubtract
            | StandardBuiltinId::TemporalZonedDateTimePrototypeUntil
            | StandardBuiltinId::TemporalZonedDateTimePrototypeSince => {
                self.require_temporal_namespace();
                // `toPlainDateTime` hands back a `Temporal.PlainDateTime`, the
                // mirror of the `TemporalZonedDateTimeConstructor` requirement
                // the PlainDateTime arm above carries for `toZonedDateTime`.
                // Without it `emit_alloc_temporal_plain_date_time` reads a
                // prototype global nothing has bootstrapped.
                //
                // SIZE CONSEQUENCE, recorded deliberately. This arm is shared
                // by all 24 ZonedDateTime members, so a program that touches
                // *any* of them — `zdt.hour`, say — now roots the whole
                // PlainDateTime constructor family. It cannot be narrowed in
                // place: the same arm unconditionally inserts
                // `...PrototypeToPlainDateTime` into `standard_roots` below,
                // and that emitter reads
                // `TEMPORAL_PLAIN_DATE_TIME_PROTOTYPE_GLOBAL_INDEX`
                // (`intrinsics/temporal_plain_date_time.rs`). So the growth is
                // inherent to the existing root-them-all shape rather than a
                // placement mistake, and it is unmeasured against batch 3's
                // emit-size budget. The real fix, if it ever matters, is to
                // split this arm so the accessors do not drag in
                // `toPlainDateTime`.
                self.require_standard_builtin(StandardBuiltinId::TemporalPlainDateTimeConstructor);
                // `until`/`since` hand back a `Temporal.Duration` and
                // `add`/`subtract` read one, so this arm states that edge
                // directly — the same sentence the PlainTime (`:1923`) and
                // PlainDateTime (`:1999`) arms already carry.
                //
                // IT IS REDUNDANT TODAY, and batch 6 first shipped it with a
                // comment claiming the opposite ("THE LINE THIS ARM WAS MISSING
                // WHEN IT ONLY HELD 22 MEMBERS"). Traced instead of assumed:
                // `require_standard_builtin` roots unconditionally and walks
                // once (`:1408`), the line directly above enters the
                // PlainDateTime arm, and that arm's FIRST statement (`:2079`)
                // is `require_standard_builtin(TemporalDurationConstructor)`.
                // So `standard_roots` contains the whole Duration family with
                // this line deleted — for `zdt.add`, and equally for `zdt.hour`,
                // before batch 6 as well as after. The emitter-reads-an-
                // unbootstrapped-global failure mode was never reachable
                // through this path, and no test can witness this line.
                //
                // Keep it anyway, but for the honest reason: it says what this
                // arm depends on rather than what the arm above happens to drag
                // in, and it becomes load-bearing the moment the arm is split
                // as the comment above proposes.
                //
                // TERMINATION, checked against the Duration arm rather than
                // assumed. This is a new edge out of an arm that already sits
                // on the PlainDateTime <-> ZonedDateTime cycle contained by
                // `RuntimeBootstrapPlan::walked` (`:1161`, `:1334`). It does
                // *not* widen that cycle today: the `Temporal.Duration` arm
                // below inserts its whole family straight into `standard_roots`
                // and calls `require_standard_builtin` for nothing, so it is a
                // leaf in the walk. The edge that would close a second cycle is
                // `Duration.prototype.round`/`total` taking a `relativeTo` that
                // may be a ZonedDateTime — the day that arm starts requiring
                // `TemporalZonedDateTimeConstructor`, this line is what turns it
                // into a real cycle, and `walked` is what keeps it terminating.
                //
                // SIZE, corrected. The unmeasured growth batch 6 added to this
                // arm is NOT the Duration root (which changed nothing, above):
                // it is the five new members in the unconditional
                // `standard_roots` list below, so every program touching any
                // ZonedDateTime member — `zdt.hour` included — now emits
                // `withCalendar`, `add`, `subtract`, `until` and `since` as
                // well. `add`/`subtract` and `until`/`since` each inline the
                // whole `emit_temporal_zoned_date_time_to_plain_date_time` body
                // on top of two or three `emit_direct_js_call` sequences, so
                // this is five function bodies, two of them large. No budget
                // test reddens on it (`LILA_EMIT_SIZE_REPORT[_PATH]` is
                // opt-in); it shows up as slower cold compiles in a family b5
                // already measured at ~300 s/case. Recorded as owned debt in
                // `target/lane-notes/zdt-arithmetic-surface-b6-integration.md`;
                // the fix is the arm split the comment above names.
                self.require_standard_builtin(StandardBuiltinId::TemporalDurationConstructor);
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
                    StandardBuiltinId::TemporalZonedDateTimePrototypeEraGetter,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeEraYearGetter,
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
                    StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeWithCalendar,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeAdd,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeSubtract,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeUntil,
                    StandardBuiltinId::TemporalZonedDateTimePrototypeSince,
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
            | StandardBuiltinId::AsyncIteratorPrototypeAsyncDispose
            | StandardBuiltinId::AsyncDisposableStackPrototypeDisposeAsync
            | StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled
            | StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected => {
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
                        | StandardBuiltinId::AsyncDisposableStackPrototypeDisposeAsync
                        | StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled
                        | StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected
                ) {
                    self.standard_roots
                        .insert(StandardBuiltinId::PromiseCapabilityExecutor);
                }
                // `disposeAsync` reaches its two settlement callbacks only as
                // function *values* parked in a promise reaction, never by a
                // direct call, and all three need the constructor's intrinsic
                // installer to have run for the prototype to exist at all.
                if matches!(
                    builtin,
                    StandardBuiltinId::AsyncDisposableStackPrototypeDisposeAsync
                        | StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled
                        | StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected
                ) {
                    self.standard_roots
                        .insert(StandardBuiltinId::AsyncDisposableStackConstructor);
                    self.standard_roots
                        .insert(StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled);
                    self.standard_roots
                        .insert(StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected);
                    self.standard_roots
                        .insert(StandardBuiltinId::SuppressedErrorConstructor);
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
            // Every prototype member roots the constructor, because the
            // constructor's intrinsic installer is what puts the member on
            // `AsyncDisposableStack.prototype` in the first place. `disposeAsync`
            // and its two settlement callbacks are matched by the Promise arm
            // above -- one arm per id, so they root the constructor there.
            StandardBuiltinId::AsyncDisposableStackPrototypeUse
            | StandardBuiltinId::AsyncDisposableStackPrototypeAdopt
            | StandardBuiltinId::AsyncDisposableStackPrototypeDefer
            | StandardBuiltinId::AsyncDisposableStackPrototypeMove
            | StandardBuiltinId::AsyncDisposableStackPrototypeDisposedGetter => {
                self.standard_roots
                    .insert(StandardBuiltinId::AsyncDisposableStackConstructor);
            }
            StandardBuiltinId::DisposableStackPrototypeUse
            | StandardBuiltinId::DisposableStackPrototypeAdopt
            | StandardBuiltinId::DisposableStackPrototypeDefer
            | StandardBuiltinId::DisposableStackPrototypeMove
            | StandardBuiltinId::DisposableStackPrototypeDispose
            | StandardBuiltinId::DisposableStackPrototypeDisposedGetter => {
                self.require_standard_builtin(StandardBuiltinId::DisposableStackConstructor);
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

/// Visits the evaluated operands stored by a suspended property Reference.
/// `[[Strict]]` has no planning dependency, while every future base/receiver
/// shape must extend the exhaustive use-view match here.
fn for_each_suspended_property_reference_operand(
    reference: &SuspendedPropertyReferenceIr,
    mut visit: impl FnMut(&TypedExpr),
) {
    match reference.use_view() {
        SuspendedPropertyReferenceUse::Ordinary {
            base_and_receiver,
            key,
            strictness: _,
        } => {
            visit(base_and_receiver);
            if let PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) = key {
                visit(expr);
            }
        }
    }
}

fn suspended_property_reference_operand_matches(
    reference: &SuspendedPropertyReferenceIr,
    mut predicate: impl FnMut(&TypedExpr) -> bool,
) -> bool {
    let mut matched = false;
    for_each_suspended_property_reference_operand(reference, |expr| {
        matched |= predicate(expr);
    });
    matched
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
                || match resume_mode {
                    GeneratorResumeModeIr::AssignProperty(reference) => {
                        suspended_property_reference_operand_matches(
                            reference,
                            expr_exposes_global_object,
                        )
                    }
                    GeneratorResumeModeIr::Ignore
                    | GeneratorResumeModeIr::Return
                    | GeneratorResumeModeIr::AssignIdentifier(_) => false,
                }
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
        StatementIr::AsyncFunctionForOfIterator { iterable, plan } => {
            expr_exposes_global_object(iterable)
                || plan
                    .before_await()
                    .iter()
                    .chain(std::iter::once(plan.await_statement()))
                    .chain(plan.after_await())
                    .any(statement_exposes_global_object)
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
        StatementIr::ForOfIterator { iterable, body, .. } => {
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
        StatementIr::SyncDisposableScope {
            resources, body, ..
        } => {
            resources
                .iter()
                .any(|resource| expr_exposes_global_object(&resource.initializer))
                || block_exposes_global_object(body)
        }
        StatementIr::AsyncDisposableScope {
            resources, body, ..
        } => {
            resources
                .iter()
                .any(|resource| expr_exposes_global_object(resource.initializer()))
                || block_exposes_global_object(body)
        }
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
        ForInitIr::Statements(statements) => statements.iter().any(statement_exposes_global_object),
        ForInitIr::SyncDisposable(resources) => resources
            .iter()
            .any(|resource| expr_exposes_global_object(&resource.initializer)),
        ForInitIr::AsyncDisposable(init) => init
            .resources()
            .iter()
            .any(|resource| expr_exposes_global_object(resource.initializer())),
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
        | ObjectPropertyIr::NonEnumerableData { value, .. } => expr_exposes_global_object(value),
        ObjectPropertyIr::Method { .. }
        | ObjectPropertyIr::Getter { .. }
        | ObjectPropertyIr::Setter { .. } => false,
        ObjectPropertyIr::ComputedData { key, value } => {
            expr_exposes_global_object(key) || expr_exposes_global_object(value)
        }
        ObjectPropertyIr::ComputedMethod { key, .. }
        | ObjectPropertyIr::ComputedGetter { key, .. }
        | ObjectPropertyIr::ComputedSetter { key, .. } => expr_exposes_global_object(key),
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

fn array_accumulation_expressions(
    accumulation: &ArrayAccumulationIr,
) -> impl Iterator<Item = &TypedExpr> {
    accumulation
        .elements()
        .iter()
        .filter_map(|element| match element {
            ArrayAccumulationElementIr::Elision => None,
            ArrayAccumulationElementIr::Value(value) => Some(value),
            ArrayAccumulationElementIr::Spread(spread) => Some(spread.value.as_ref()),
        })
}

fn array_accumulation_has_spread(accumulation: &ArrayAccumulationIr) -> bool {
    accumulation
        .elements()
        .iter()
        .any(|element| matches!(element, ArrayAccumulationElementIr::Spread(_)))
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
        ExprIr::ArrayAccumulation(accumulation) => {
            array_accumulation_expressions(accumulation).any(expr_exposes_global_object)
        }
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { value, .. }
        | ExprIr::UnaryPlus { expr: value }
        | ExprIr::UnaryMinusNumeric { expr: value }
        | ExprIr::UnaryBitwiseNumeric { expr: value, .. }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::SpreadArgument(SpreadArgumentIr { value, .. })
        | ExprIr::PrivateIn { rhs: value, .. } => expr_exposes_global_object(value),
        ExprIr::JsonParseStaticReviver {
            callee,
            input,
            reviver,
            ..
        } => {
            expr_exposes_global_object(callee)
                || expr_exposes_global_object(input)
                || expr_exposes_global_object(reviver)
        }
        ExprIr::SpecOperation { operands, .. } => operands.iter().any(expr_exposes_global_object),
        ExprIr::PropertyRead { target, key } | ExprIr::DeleteProperty { target, key, .. } => {
            property_access_exposes_global_object(target, key)
                || property_key_exposes_global_object(key)
        }
        ExprIr::OrdinaryPropertyAssignment(assignment) => {
            property_access_exposes_global_object(
                assignment.base_and_receiver(),
                assignment.referenced_name(),
            ) || property_key_exposes_global_object(assignment.referenced_name())
                || expr_exposes_global_object(assignment.rhs())
        }
        ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {
            property_access_exposes_global_object(
                assignment.base_and_receiver(),
                assignment.referenced_name(),
            ) || property_key_exposes_global_object(assignment.referenced_name())
                || expr_exposes_global_object(assignment.rhs())
        }
        ExprIr::OrdinaryPropertyNumericUpdate(update) => {
            property_access_exposes_global_object(
                update.base_and_receiver(),
                update.referenced_name(),
            ) || property_key_exposes_global_object(update.referenced_name())
        }
        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {
            property_access_exposes_global_object(
                mutation.base_and_receiver(),
                mutation.referenced_name(),
            ) || property_key_exposes_global_object(mutation.referenced_name())
                || expr_exposes_global_object(mutation.result())
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
        ExprIr::PropertyWrite {
            target, key, value, ..
        } => {
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
        | ExprIr::BitwiseNumeric { lhs, rhs, .. }
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
        ExprIr::SuperPropertyRead { key, receiver } => {
            property_key_exposes_global_object(key) || expr_exposes_global_object(receiver)
        }
        ExprIr::SuperPropertyWrite {
            key,
            receiver,
            value,
            ..
        } => {
            property_key_exposes_global_object(key)
                || expr_exposes_global_object(receiver)
                || expr_exposes_global_object(value)
        }
        ExprIr::SuperPropertyMutation(mutation) => {
            property_key_exposes_global_object(mutation.referenced_name())
                || expr_exposes_global_object(mutation.receiver())
                || match mutation.operation() {
                    SuperPropertyMutationOperationIr::NumericUpdate { .. } => false,
                    SuperPropertyMutationOperationIr::EagerCompound { result, .. } => {
                        expr_exposes_global_object(result)
                    }
                }
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
            if let GeneratorResumeModeIr::AssignProperty(reference) = resume_mode {
                for_each_suspended_property_reference_operand(reference, |expr| {
                    collect_expr_global_property_names(expr, names);
                });
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
        StatementIr::AsyncFunctionForOfIterator { iterable, plan } => {
            collect_expr_global_property_names(iterable, names);
            for statement in plan
                .before_await()
                .iter()
                .chain(std::iter::once(plan.await_statement()))
                .chain(plan.after_await())
            {
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
        StatementIr::ForOfIterator { iterable, body, .. } => {
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
        StatementIr::SyncDisposableScope {
            resources, body, ..
        } => {
            for resource in resources.iter() {
                collect_expr_global_property_names(&resource.initializer, names);
            }
            collect_block_global_property_names(body, names);
        }
        StatementIr::AsyncDisposableScope {
            resources, body, ..
        } => {
            for resource in resources.iter() {
                collect_expr_global_property_names(resource.initializer(), names);
            }
            collect_block_global_property_names(body, names);
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
        ForInitIr::Statements(statements) => {
            for statement in statements {
                collect_statement_global_property_names(statement, names);
            }
        }
        ForInitIr::SyncDisposable(resources) => {
            for resource in resources.iter() {
                collect_expr_global_property_names(&resource.initializer, names);
            }
        }
        ForInitIr::AsyncDisposable(init) => {
            for resource in init.resources().iter() {
                collect_expr_global_property_names(resource.initializer(), names);
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
        | ObjectPropertyIr::NonEnumerableData { value, .. } => {
            collect_expr_global_property_names(value, names);
        }
        ObjectPropertyIr::Method { .. }
        | ObjectPropertyIr::Getter { .. }
        | ObjectPropertyIr::Setter { .. } => {}
        ObjectPropertyIr::ComputedData { key, value } => {
            collect_expr_global_property_names(key, names);
            collect_expr_global_property_names(value, names);
        }
        ObjectPropertyIr::ComputedMethod { key, .. }
        | ObjectPropertyIr::ComputedGetter { key, .. }
        | ObjectPropertyIr::ComputedSetter { key, .. } => {
            collect_expr_global_property_names(key, names);
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
        ExprIr::GlobalPropertyUpdate { name, .. } | ExprIr::DeleteGlobalProperty { name, .. } => {
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
        ExprIr::ArrayAccumulation(accumulation) => {
            for element in array_accumulation_expressions(accumulation) {
                collect_expr_global_property_names(element, names);
            }
        }
        ExprIr::AssignIdentifier { name, value }
        | ExprIr::CompoundAssignIdentifier { name, value, .. } => {
            names.insert(name.clone());
            collect_expr_global_property_names(value, names);
        }
        ExprIr::UnaryPlus { expr: value }
        | ExprIr::UnaryMinusNumeric { expr: value }
        | ExprIr::UnaryBitwiseNumeric { expr: value, .. }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::SpreadArgument(SpreadArgumentIr { value, .. })
        | ExprIr::PrivateIn { rhs: value, .. } => collect_expr_global_property_names(value, names),
        ExprIr::JsonParseStaticReviver {
            callee,
            input,
            reviver,
            ..
        } => {
            collect_expr_global_property_names(callee, names);
            collect_expr_global_property_names(input, names);
            collect_expr_global_property_names(reviver, names);
        }
        ExprIr::SpecOperation { operands, .. } => {
            for operand in operands {
                collect_expr_global_property_names(operand, names);
            }
        }
        ExprIr::PropertyRead { target, key } | ExprIr::DeleteProperty { target, key, .. } => {
            collect_expr_global_property_names(target, names);
            collect_property_key_global_property_names(key, names);
        }
        ExprIr::OrdinaryPropertyAssignment(assignment) => {
            collect_expr_global_property_names(assignment.base_and_receiver(), names);
            collect_property_key_global_property_names(assignment.referenced_name(), names);
            collect_expr_global_property_names(assignment.rhs(), names);
        }
        ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {
            collect_expr_global_property_names(assignment.base_and_receiver(), names);
            collect_property_key_global_property_names(assignment.referenced_name(), names);
            collect_expr_global_property_names(assignment.rhs(), names);
        }
        ExprIr::OrdinaryPropertyNumericUpdate(update) => {
            collect_expr_global_property_names(update.base_and_receiver(), names);
            collect_property_key_global_property_names(update.referenced_name(), names);
        }
        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {
            collect_expr_global_property_names(mutation.base_and_receiver(), names);
            collect_property_key_global_property_names(mutation.referenced_name(), names);
            collect_expr_global_property_names(mutation.result(), names);
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
        ExprIr::PropertyWrite {
            target, key, value, ..
        } => {
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
        | ExprIr::BitwiseNumeric { lhs, rhs, .. }
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
                        DestructuringTargetIr::AssignmentIdentifier(reference) => {
                            if let IdentifierWriteDisposition::Global {
                                referenced_name, ..
                            } = reference.write_disposition()
                            {
                                names.insert(referenced_name.to_string());
                            }
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
                    DestructuringTargetIr::AssignmentIdentifier(reference) => {
                        if let IdentifierWriteDisposition::Global {
                            referenced_name, ..
                        } = reference.write_disposition()
                        {
                            names.insert(referenced_name.to_string());
                        }
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
                    DestructuringTargetIr::AssignmentIdentifier(reference) => {
                        if let IdentifierWriteDisposition::Global {
                            referenced_name, ..
                        } = reference.write_disposition()
                        {
                            names.insert(referenced_name.to_string());
                        }
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
        ExprIr::SuperPropertyRead { key, receiver } => {
            collect_property_key_global_property_names(key, names);
            collect_expr_global_property_names(receiver, names);
        }
        ExprIr::SuperPropertyWrite {
            key,
            receiver,
            value,
            ..
        } => {
            collect_property_key_global_property_names(key, names);
            collect_expr_global_property_names(receiver, names);
            collect_expr_global_property_names(value, names);
        }
        ExprIr::SuperPropertyMutation(mutation) => {
            collect_property_key_global_property_names(mutation.referenced_name(), names);
            collect_expr_global_property_names(mutation.receiver(), names);
            match mutation.operation() {
                SuperPropertyMutationOperationIr::NumericUpdate { .. } => {}
                SuperPropertyMutationOperationIr::EagerCompound { result, .. } => {
                    collect_expr_global_property_names(result, names);
                }
            }
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
            form,
            resume_mode,
            ..
        } => {
            expr_references_function(value, target)
                || match resume_mode {
                    GeneratorResumeModeIr::AssignProperty(reference) => {
                        suspended_property_reference_operand_matches(reference, |expr| {
                            expr_references_function(expr, target)
                        })
                    }
                    GeneratorResumeModeIr::Ignore
                    | GeneratorResumeModeIr::Return
                    | GeneratorResumeModeIr::AssignIdentifier(_) => false,
                }
                || match form {
                    YieldForm::Plain => false,
                    YieldForm::Delegate(_) => [
                        StandardBuiltinId::ArrayPrototypeValues,
                        StandardBuiltinId::ArrayIteratorNext,
                        StandardBuiltinId::ArrayIteratorIdentity,
                        StandardBuiltinId::StringPrototypeIterator,
                        StandardBuiltinId::StringIteratorNext,
                    ]
                    .into_iter()
                    .any(|builtin| builtin.function_id() == target.as_str()),
                }
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
        StatementIr::AsyncFunctionForOfIterator { iterable, plan } => {
            expr_references_function(iterable, target)
                || plan
                    .before_await()
                    .iter()
                    .chain(std::iter::once(plan.await_statement()))
                    .chain(plan.after_await())
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
        StatementIr::ForOfIterator { iterable, body, .. } => {
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
        StatementIr::SyncDisposableScope {
            resources, body, ..
        } => {
            resources
                .iter()
                .any(|resource| expr_references_function(&resource.initializer, target))
                || block_references_function(body, target)
        }
        StatementIr::AsyncDisposableScope {
            resources, body, ..
        } => {
            resources
                .iter()
                .any(|resource| expr_references_function(resource.initializer(), target))
                || block_references_function(body, target)
        }
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
        ForInitIr::Statements(statements) => statements
            .iter()
            .any(|statement| statement_references_function(statement, target)),
        ForInitIr::SyncDisposable(resources) => resources
            .iter()
            .any(|resource| expr_references_function(&resource.initializer, target)),
        ForInitIr::AsyncDisposable(init) => init
            .resources()
            .iter()
            .any(|resource| expr_references_function(resource.initializer(), target)),
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

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum ShapeAccessorReferenceSelection {
    Getter,
    Setter,
    GetterOrSetter,
}

fn shape_accessor_references_function(
    shape: Option<&HeapShape>,
    key: &PropertyKeyIr,
    target: &FunctionId,
    selection: ShapeAccessorReferenceSelection,
) -> bool {
    let Some(shape) = shape else {
        return false;
    };
    if let Some(name) = static_property_key_name(key) {
        let Some(ObjectShapeProperty::Accessor { getter, setter }) =
            read_static_heap_shape_property(shape, name)
        else {
            return false;
        };
        return match selection {
            ShapeAccessorReferenceSelection::Getter => {
                getter.is_some_and(|getter| getter.function_id == *target)
            }
            ShapeAccessorReferenceSelection::Setter => {
                setter.is_some_and(|setter| setter.function_id == *target)
            }
            ShapeAccessorReferenceSelection::GetterOrSetter => {
                getter.is_some_and(|getter| getter.function_id == *target)
                    || setter.is_some_and(|setter| setter.function_id == *target)
            }
        };
    }

    fn any_accessor(
        shape: &HeapShape,
        target: &FunctionId,
        selection: ShapeAccessorReferenceSelection,
    ) -> bool {
        let (properties, prototype) = match shape {
            HeapShape::Object(shape) => (&shape.properties, shape.prototype.as_deref()),
            HeapShape::Array(shape) => (&shape.properties, shape.prototype.as_deref()),
        };
        properties.values().any(|property| {
            let ObjectShapeProperty::Accessor { getter, setter } = property else {
                return false;
            };
            match selection {
                ShapeAccessorReferenceSelection::Getter => getter
                    .as_ref()
                    .is_some_and(|getter| getter.function_id == *target),
                ShapeAccessorReferenceSelection::Setter => setter
                    .as_ref()
                    .is_some_and(|setter| setter.function_id == *target),
                ShapeAccessorReferenceSelection::GetterOrSetter => {
                    getter
                        .as_ref()
                        .is_some_and(|getter| getter.function_id == *target)
                        || setter
                            .as_ref()
                            .is_some_and(|setter| setter.function_id == *target)
                }
            }
        }) || prototype.is_some_and(|prototype| any_accessor(prototype, target, selection))
    }

    any_accessor(shape, target, selection)
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

    info.function_targets.known_targets().contains(target)
}

pub(crate) fn object_property_references_function(
    property: &ObjectPropertyIr,
    target: &FunctionId,
) -> bool {
    match property {
        ObjectPropertyIr::PrototypeSetter { value }
        | ObjectPropertyIr::Data { value, .. }
        | ObjectPropertyIr::NonEnumerableData { value, .. } => {
            expr_references_function(value, target)
        }
        ObjectPropertyIr::Method { function, .. }
        | ObjectPropertyIr::Getter { function, .. }
        | ObjectPropertyIr::Setter { function, .. } => function.function_id() == target,
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
            expr_references_function(key, target) || function.function_id() == target
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
    if expr.function_targets.known_targets().contains(target) {
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
        ExprIr::ArrayAccumulation(accumulation) => {
            (array_accumulation_has_spread(accumulation)
                && (*target == StandardBuiltinId::ArrayPrototypeValues.function_id()
                    || *target == StandardBuiltinId::ArrayIteratorNext.function_id()
                    || *target == StandardBuiltinId::StringConstructor.function_id()))
                || array_accumulation_expressions(accumulation)
                    .any(|element| expr_references_function(element, target))
        }
        ExprIr::AssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyWrite { value, .. }
        | ExprIr::CompoundAssignIdentifier { value, .. }
        | ExprIr::GlobalPropertyCompoundAssign { value, .. }
        | ExprIr::UnaryPlus { expr: value }
        | ExprIr::UnaryMinusNumeric { expr: value }
        | ExprIr::UnaryBitwiseNumeric { expr: value, .. }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value }
        | ExprIr::TypeOf { expr: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::PrivateIn { rhs: value, .. } => expr_references_function(value, target),
        ExprIr::JsonParseStaticReviver {
            callee,
            input,
            reviver,
            ..
        } => {
            expr_references_function(callee, target)
                || expr_references_function(input, target)
                || expr_references_function(reviver, target)
        }
        ExprIr::SpreadArgument(spread) => {
            *target == StandardBuiltinId::ArrayPrototypeValues.function_id()
                || *target == StandardBuiltinId::ArrayIteratorNext.function_id()
                || *target == StandardBuiltinId::StringConstructor.function_id()
                || expr_references_function(&spread.value, target)
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
                                ShapeAccessorReferenceSelection::Getter,
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
                    ShapeAccessorReferenceSelection::Getter,
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
        ExprIr::OrdinaryPropertyAssignment(assignment) => {
            expr_references_function(assignment.base_and_receiver(), target)
                || property_key_references_function(assignment.referenced_name(), target)
                || expr_references_function(assignment.rhs(), target)
                || assignment.possible_setters().contains(target)
                || shape_accessor_references_function(
                    assignment.base_and_receiver().heap_shape.as_deref(),
                    assignment.referenced_name(),
                    target,
                    ShapeAccessorReferenceSelection::Setter,
                )
        }
        ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {
            expr_references_function(assignment.base_and_receiver(), target)
                || property_key_references_function(assignment.referenced_name(), target)
                || expr_references_function(assignment.rhs(), target)
                || assignment.possible_getters().contains(target)
                || assignment.possible_setters().contains(target)
                || shape_accessor_references_function(
                    assignment.base_and_receiver().heap_shape.as_deref(),
                    assignment.referenced_name(),
                    target,
                    ShapeAccessorReferenceSelection::GetterOrSetter,
                )
        }
        ExprIr::OrdinaryPropertyNumericUpdate(update) => {
            expr_references_function(update.base_and_receiver(), target)
                || property_key_references_function(update.referenced_name(), target)
                || update.possible_getters().contains(target)
                || update.possible_setters().contains(target)
                || shape_accessor_references_function(
                    update.base_and_receiver().heap_shape.as_deref(),
                    update.referenced_name(),
                    target,
                    ShapeAccessorReferenceSelection::GetterOrSetter,
                )
        }
        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {
            expr_references_function(mutation.base_and_receiver(), target)
                || property_key_references_function(mutation.referenced_name(), target)
                || expr_references_function(mutation.result(), target)
                || mutation.possible_getters().contains(target)
                || mutation.possible_setters().contains(target)
                || shape_accessor_references_function(
                    mutation.base_and_receiver().heap_shape.as_deref(),
                    mutation.referenced_name(),
                    target,
                    ShapeAccessorReferenceSelection::GetterOrSetter,
                )
        }
        ExprIr::PropertyWrite {
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
                    ShapeAccessorReferenceSelection::Setter,
                )
        }
        ExprIr::StringCharCodeAt {
            target: object,
            index,
        } => expr_references_function(object, target) || expr_references_function(index, target),
        ExprIr::BinaryNumber { lhs, rhs, .. }
        | ExprIr::CoerciveAdd { lhs, rhs }
        | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. }
        | ExprIr::BitwiseNumeric { lhs, rhs, .. }
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
        ExprIr::SuperPropertyRead { key, receiver } => {
            property_key_references_function(key, target)
                || expr_references_function(receiver, target)
        }
        ExprIr::SuperPropertyWrite {
            key,
            receiver,
            value,
            ..
        } => {
            property_key_references_function(key, target)
                || expr_references_function(receiver, target)
                || expr_references_function(value, target)
        }
        ExprIr::SuperPropertyMutation(mutation) => {
            property_key_references_function(mutation.referenced_name(), target)
                || expr_references_function(mutation.receiver(), target)
                || match mutation.operation() {
                    SuperPropertyMutationOperationIr::NumericUpdate { .. } => false,
                    SuperPropertyMutationOperationIr::EagerCompound { result, .. } => {
                        expr_references_function(result, target)
                    }
                }
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
pub(crate) struct NumberPowImportFunctionIndex(u32);
pub(crate) struct WallClockMillisImportFunctionIndex(u32);
pub(crate) struct SharedMemoryAllocImportFunctionIndex(u32);
pub(crate) struct MonotonicClockNanosImportFunctionIndex(u32);
pub(crate) struct SleepNanosImportFunctionIndex(u32);
pub(crate) struct AgentCallImportFunctionIndex(u32);
pub(crate) struct IntlCallImportFunctionIndex(u32);
pub(crate) struct RandomF64ImportFunctionIndex(u32);

macro_rules! host_import_function_index_role {
    ($role:ident) => {
        impl $role {
            pub(crate) const fn new(index: u32) -> Self {
                Self(index)
            }
        }
    };
}

host_import_function_index_role!(NumberPowImportFunctionIndex);
host_import_function_index_role!(WallClockMillisImportFunctionIndex);
host_import_function_index_role!(SharedMemoryAllocImportFunctionIndex);
host_import_function_index_role!(MonotonicClockNanosImportFunctionIndex);
host_import_function_index_role!(SleepNanosImportFunctionIndex);
host_import_function_index_role!(AgentCallImportFunctionIndex);
host_import_function_index_role!(IntlCallImportFunctionIndex);
host_import_function_index_role!(RandomF64ImportFunctionIndex);

#[must_use]
pub(crate) struct HostImportFunctionIndices {
    number_pow: Option<NumberPowImportFunctionIndex>,
    wall_clock_millis: Option<WallClockMillisImportFunctionIndex>,
    shared_memory_alloc: Option<SharedMemoryAllocImportFunctionIndex>,
    monotonic_clock_nanos: Option<MonotonicClockNanosImportFunctionIndex>,
    sleep_nanos: Option<SleepNanosImportFunctionIndex>,
    agent_call: Option<AgentCallImportFunctionIndex>,
    intl_call: Option<IntlCallImportFunctionIndex>,
    random_f64: Option<RandomF64ImportFunctionIndex>,
}

impl HostImportFunctionIndices {
    #[allow(clippy::too_many_arguments)]
    pub(crate) const fn new(
        number_pow: Option<NumberPowImportFunctionIndex>,
        wall_clock_millis: Option<WallClockMillisImportFunctionIndex>,
        shared_memory_alloc: Option<SharedMemoryAllocImportFunctionIndex>,
        monotonic_clock_nanos: Option<MonotonicClockNanosImportFunctionIndex>,
        sleep_nanos: Option<SleepNanosImportFunctionIndex>,
        agent_call: Option<AgentCallImportFunctionIndex>,
        intl_call: Option<IntlCallImportFunctionIndex>,
        random_f64: Option<RandomF64ImportFunctionIndex>,
    ) -> Self {
        Self {
            number_pow,
            wall_clock_millis,
            shared_memory_alloc,
            monotonic_clock_nanos,
            sleep_nanos,
            agent_call,
            intl_call,
            random_f64,
        }
    }
}

pub(crate) struct FunctionMetaRegistry {
    metas: BTreeMap<FunctionId, WasmFunctionMeta>,
    compiled_host_builtins: BTreeSet<HostBuiltinId>,
    touched_standard_builtins: std::cell::RefCell<BTreeSet<StandardBuiltinId>>,
    touched_host_builtins: std::cell::RefCell<BTreeSet<HostBuiltinId>>,
    host_import_function_indices: HostImportFunctionIndices,
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
        compiled_host_builtins: BTreeSet<HostBuiltinId>,
        host_import_function_indices: HostImportFunctionIndices,
    ) -> Self {
        Self {
            metas,
            compiled_host_builtins,
            touched_standard_builtins: std::cell::RefCell::new(BTreeSet::new()),
            touched_host_builtins: std::cell::RefCell::new(BTreeSet::new()),
            host_import_function_indices,
            touched_number_pow_import: std::cell::Cell::new(false),
            suppress_recording: std::cell::Cell::new(false),
        }
    }

    pub(crate) fn number_pow_import_function_index(&self) -> Option<u32> {
        if !self.suppress_recording.get() {
            self.touched_number_pow_import.set(true);
        }
        self.host_import_function_indices
            .number_pow
            .as_ref()
            .map(|index| index.0)
    }

    pub(crate) fn touched_number_pow_import(&self) -> bool {
        self.touched_number_pow_import.get()
    }

    pub(crate) fn wall_clock_millis_import_function_index(&self) -> Option<u32> {
        self.host_import_function_indices
            .wall_clock_millis
            .as_ref()
            .map(|index| index.0)
    }

    pub(crate) fn shared_memory_alloc_function_index(&self) -> Option<u32> {
        self.host_import_function_indices
            .shared_memory_alloc
            .as_ref()
            .map(|index| index.0)
    }

    pub(crate) fn monotonic_clock_nanos_import_function_index(&self) -> Option<u32> {
        self.host_import_function_indices
            .monotonic_clock_nanos
            .as_ref()
            .map(|index| index.0)
    }

    pub(crate) fn sleep_nanos_import_function_index(&self) -> Option<u32> {
        self.host_import_function_indices
            .sleep_nanos
            .as_ref()
            .map(|index| index.0)
    }

    pub(crate) fn agent_call_import_function_index(&self) -> Option<u32> {
        self.host_import_function_indices
            .agent_call
            .as_ref()
            .map(|index| index.0)
    }

    pub(crate) fn intl_call_import_function_index(&self) -> Option<u32> {
        self.host_import_function_indices
            .intl_call
            .as_ref()
            .map(|index| index.0)
    }

    pub(crate) fn random_f64_import_function_index(&self) -> Option<u32> {
        self.host_import_function_indices
            .random_f64
            .as_ref()
            .map(|index| index.0)
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

    pub(crate) fn contains_compiled_host_builtin(&self, builtin: HostBuiltinId) -> bool {
        self.compiled_host_builtins.contains(&builtin)
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
                protocol: function.protocol,
                strict: function.strict,
                is_named_expression: function.is_named_expression,
                class_element_execution_kind: function.class_element_execution_kind,
                class_heritage_kind: function.class_heritage_kind,
                is_static_class_member: function.is_static_class_member,
                is_derived_constructor: function.is_derived_constructor,
                is_synthetic_default_derived_constructor: function
                    .is_synthetic_default_derived_constructor,
                class_instance_element_plan: function.class_instance_element_plan.clone(),
                uses_super: function.uses_super,
                this_before_super: function.this_before_super,
                captures_private_environment: function.captures_private_environment,
                needs_active_function_identity: function.protocol.flavor()
                    == FunctionFlavor::Ordinary,
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
            protocol: if builtin.constructable() {
                FunctionProtocolIr::OrdinaryCallAndConstruct
            } else {
                FunctionProtocolIr::OrdinaryCallOnly
            },
            strict: true,
            is_named_expression: false,
            class_element_execution_kind: ClassElementExecutionKind::None,
            class_heritage_kind: ClassHeritageKind::None,
            is_static_class_member: false,
            is_derived_constructor: false,
            is_synthetic_default_derived_constructor: false,
            class_instance_element_plan: None,
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
        protocol: FunctionProtocolIr::OrdinaryCallOnly,
        strict: true,
        is_named_expression: false,
        class_element_execution_kind: ClassElementExecutionKind::None,
        class_heritage_kind: ClassHeritageKind::None,
        is_static_class_member: false,
        is_derived_constructor: false,
        is_synthetic_default_derived_constructor: false,
        class_instance_element_plan: None,
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
        | StandardBuiltinId::FunctionPrototypeSymbolHasInstance
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
        | StandardBuiltinId::TemporalInstantFromEpochMilliseconds
        | StandardBuiltinId::TemporalInstantFromEpochNanoseconds
        | StandardBuiltinId::TemporalInstantPrototypeEquals
        | StandardBuiltinId::TemporalZonedDateTimeFrom
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEquals
        | StandardBuiltinId::TemporalPlainDateFrom
        | StandardBuiltinId::TemporalPlainDatePrototypeWith
        | StandardBuiltinId::TemporalPlainDatePrototypeWithCalendar
        | StandardBuiltinId::TemporalPlainDatePrototypeAdd
        | StandardBuiltinId::TemporalPlainDatePrototypeSubtract
        | StandardBuiltinId::TemporalPlainDatePrototypeUntil
        | StandardBuiltinId::TemporalPlainDatePrototypeSince
        | StandardBuiltinId::TemporalPlainDatePrototypeEquals
        | StandardBuiltinId::TemporalPlainTimeFrom
        | StandardBuiltinId::TemporalPlainTimePrototypeWith
        | StandardBuiltinId::TemporalPlainTimePrototypeAdd
        | StandardBuiltinId::TemporalPlainTimePrototypeSubtract
        | StandardBuiltinId::TemporalPlainTimePrototypeUntil
        | StandardBuiltinId::TemporalPlainTimePrototypeSince
        | StandardBuiltinId::TemporalPlainTimePrototypeRound
        | StandardBuiltinId::TemporalPlainTimePrototypeEquals
        | StandardBuiltinId::TemporalZonedDateTimePrototypeWithTimeZone
        // Each takes one required argument and an optional options bag, so
        // `length` is 1 — the same value the PlainDateTime namesakes two rows
        // below already carry, and what
        // `built-ins/Temporal/ZonedDateTime/prototype/*/length.js` asserts.
        | StandardBuiltinId::TemporalZonedDateTimePrototypeWithCalendar
        | StandardBuiltinId::TemporalZonedDateTimePrototypeAdd
        | StandardBuiltinId::TemporalZonedDateTimePrototypeSubtract
        | StandardBuiltinId::TemporalZonedDateTimePrototypeUntil
        | StandardBuiltinId::TemporalZonedDateTimePrototypeSince
        | StandardBuiltinId::TemporalPlainDateTimeFrom
        | StandardBuiltinId::TemporalPlainDateTimePrototypeWith
        | StandardBuiltinId::TemporalPlainDateTimePrototypeWithCalendar
        | StandardBuiltinId::TemporalPlainDateTimePrototypeAdd
        | StandardBuiltinId::TemporalPlainDateTimePrototypeSubtract
        | StandardBuiltinId::TemporalPlainDateTimePrototypeUntil
        | StandardBuiltinId::TemporalPlainDateTimePrototypeSince
        | StandardBuiltinId::TemporalPlainDateTimePrototypeRound
        | StandardBuiltinId::TemporalPlainDateTimePrototypeEquals
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToZonedDateTime => 1,
        StandardBuiltinId::TemporalPlainYearMonthConstructor
        | StandardBuiltinId::TemporalPlainYearMonthCompare
        | StandardBuiltinId::TemporalPlainMonthDayConstructor => 2,
        StandardBuiltinId::TemporalPlainYearMonthFrom
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeWith
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeAdd
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeSubtract
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeUntil
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeSince
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeEquals
        | StandardBuiltinId::TemporalPlainMonthDayFrom
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeWith
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeEquals
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeToPlainDate
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeToPlainDate => 1,
        StandardBuiltinId::TemporalPlainYearMonthPrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeEraYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeDaysInMonthGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeMonthsInYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeInLeapYearGetter
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeToString
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeToJson
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainYearMonthPrototypeValueOf
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeDayGetter
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeToString
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeToJson
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainMonthDayPrototypeValueOf => 0,
        StandardBuiltinId::TemporalZonedDateTimeConstructor => 2,
        StandardBuiltinId::TemporalInstantCompare => 2,
        StandardBuiltinId::TemporalPlainDateCompare => 2,
        StandardBuiltinId::TemporalPlainTimeCompare => 2,
        StandardBuiltinId::TemporalPlainDateTimeCompare => 2,
        StandardBuiltinId::TemporalPlainDateTimeConstructor => 3,
        StandardBuiltinId::TemporalPlainDateConstructor => 3,
        StandardBuiltinId::TemporalDurationConstructor => 0,
        StandardBuiltinId::TemporalDurationCompare => 2,
        StandardBuiltinId::TemporalDurationFrom
        | StandardBuiltinId::TemporalDurationPrototypeWith
        | StandardBuiltinId::TemporalDurationPrototypeAdd
        | StandardBuiltinId::TemporalDurationPrototypeSubtract
        | StandardBuiltinId::TemporalDurationPrototypeRound
        | StandardBuiltinId::TemporalDurationPrototypeTotal => 1,
        StandardBuiltinId::TemporalDurationPrototypeYearsGetter
        | StandardBuiltinId::TemporalDurationPrototypeMonthsGetter
        | StandardBuiltinId::TemporalDurationPrototypeWeeksGetter
        | StandardBuiltinId::TemporalDurationPrototypeDaysGetter
        | StandardBuiltinId::TemporalDurationPrototypeHoursGetter
        | StandardBuiltinId::TemporalDurationPrototypeMinutesGetter
        | StandardBuiltinId::TemporalDurationPrototypeSecondsGetter
        | StandardBuiltinId::TemporalDurationPrototypeMillisecondsGetter
        | StandardBuiltinId::TemporalDurationPrototypeMicrosecondsGetter
        | StandardBuiltinId::TemporalDurationPrototypeNanosecondsGetter
        | StandardBuiltinId::TemporalDurationPrototypeSignGetter
        | StandardBuiltinId::TemporalDurationPrototypeBlankGetter
        | StandardBuiltinId::TemporalDurationPrototypeNegated
        | StandardBuiltinId::TemporalDurationPrototypeAbs
        | StandardBuiltinId::TemporalDurationPrototypeToString
        | StandardBuiltinId::TemporalDurationPrototypeToJson
        | StandardBuiltinId::TemporalDurationPrototypeToLocaleString
        | StandardBuiltinId::TemporalDurationPrototypeValueOf => 0,
        StandardBuiltinId::IntlGetCanonicalLocales | StandardBuiltinId::IntlLocaleConstructor => 1,
        // ECMA-402 11.1.1/11.2.2/11.3.4/11.1.5: `Intl.DateTimeFormat` has
        // length 0, `supportedLocalesOf` and the format functions length 1.
        StandardBuiltinId::IntlDateTimeFormatConstructor
        | StandardBuiltinId::IntlDateTimeFormatPrototypeResolvedOptions
        | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatGetter => 0,
        StandardBuiltinId::IntlDateTimeFormatSupportedLocalesOf
        | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatToParts
        | StandardBuiltinId::IntlDateTimeFormatBoundFormat => 1,
        // ECMA-402 11.4.6/11.4.7: `formatRange` and `formatRangeToParts` each
        // take (startDate, endDate), so their `length` is 2, not 1.
        StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRange
        | StandardBuiltinId::IntlDateTimeFormatPrototypeFormatRangeToParts => 2,
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
        | StandardBuiltinId::TypedArrayConstructor
        | StandardBuiltinId::ArrayBufferSpeciesGetter
        | StandardBuiltinId::RegExpSpeciesGetter
        | StandardBuiltinId::PromiseSpeciesGetter
        | StandardBuiltinId::FunctionPrototype
        | StandardBuiltinId::FunctionPrototypeToString
        | StandardBuiltinId::ErrorPrototypeToString
        | StandardBuiltinId::ThrowTypeError
        | StandardBuiltinId::BoundFunctionInvoker
        | StandardBuiltinId::TemporalNowInstant
        | StandardBuiltinId::TemporalNowTimeZoneId
        | StandardBuiltinId::TemporalNowZonedDateTimeIso
        | StandardBuiltinId::TemporalInstantPrototypeEpochMillisecondsGetter
        | StandardBuiltinId::TemporalInstantPrototypeEpochNanosecondsGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeEraGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeEraYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeMonthGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDayGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDayOfWeekGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDayOfYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeWeekOfYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeYearOfWeekGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDaysInWeekGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDaysInMonthGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeDaysInYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeMonthsInYearGetter
        | StandardBuiltinId::TemporalPlainDatePrototypeInLeapYearGetter
        | StandardBuiltinId::TemporalPlainTimeConstructor
        | StandardBuiltinId::TemporalPlainTimePrototypeHourGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeMinuteGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeSecondGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeMillisecondGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeMicrosecondGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeNanosecondGetter
        | StandardBuiltinId::TemporalPlainTimePrototypeToString
        | StandardBuiltinId::TemporalPlainTimePrototypeToJson
        | StandardBuiltinId::TemporalPlainTimePrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainTimePrototypeValueOf
        // `toString ( [ options ] )` takes only an optional parameter, so its
        // `length` is 0 even though the emitter reads argument 0.
        | StandardBuiltinId::TemporalPlainDatePrototypeToString
        | StandardBuiltinId::TemporalPlainDatePrototypeToJson
        | StandardBuiltinId::TemporalPlainDatePrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainDatePrototypeValueOf
        | StandardBuiltinId::TemporalPlainDatePrototypeToPlainDateTime
        | StandardBuiltinId::TemporalPlainDatePrototypeToPlainYearMonth
        | StandardBuiltinId::TemporalPlainDatePrototypeToPlainMonthDay
        | StandardBuiltinId::TemporalPlainDateTimePrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeEraGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeEraYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthCodeGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDayGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeHourGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMinuteGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeSecondGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMillisecondGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMicrosecondGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeNanosecondGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfWeekGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDayOfYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeWeekOfYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeYearOfWeekGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInWeekGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInMonthGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeDaysInYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeMonthsInYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeInLeapYearGetter
        | StandardBuiltinId::TemporalPlainDateTimePrototypeWithPlainTime
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToString
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToJson
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToLocaleString
        | StandardBuiltinId::TemporalPlainDateTimePrototypeValueOf
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainDate
        | StandardBuiltinId::TemporalPlainDateTimePrototypeToPlainTime
        | StandardBuiltinId::TemporalInstantPrototypeToString
        | StandardBuiltinId::TemporalInstantPrototypeToJson
        | StandardBuiltinId::TemporalInstantPrototypeValueOf => 0,
        StandardBuiltinId::TemporalZonedDateTimePrototypeEpochMillisecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEpochNanosecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeOffsetNanosecondsGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeTimeZoneIdGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeCalendarIdGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEraGetter
        | StandardBuiltinId::TemporalZonedDateTimePrototypeEraYearGetter
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
        | StandardBuiltinId::TemporalZonedDateTimePrototypeToInstant
        | StandardBuiltinId::TemporalZonedDateTimePrototypeToPlainDateTime => 0,
        StandardBuiltinId::Escape
        | StandardBuiltinId::Unescape
        | StandardBuiltinId::EncodeUri
        | StandardBuiltinId::EncodeUriComponent
        | StandardBuiltinId::DecodeUri
        | StandardBuiltinId::DecodeUriComponent => 1,
        // Pinned by the two DisposableStack families' `length.js` files, one
        // file per row. The async settlement callbacks are anonymous reaction
        // handlers and take the settled value, like the `AsyncIterator`
        // `@@asyncDispose` pair.
        StandardBuiltinId::AsyncDisposableStackConstructor
        | StandardBuiltinId::DisposableStackConstructor
        | StandardBuiltinId::AsyncDisposableStackPrototypeMove
        | StandardBuiltinId::DisposableStackPrototypeMove
        | StandardBuiltinId::DisposableStackPrototypeDispose
        | StandardBuiltinId::AsyncDisposableStackPrototypeDisposeAsync
        | StandardBuiltinId::AsyncDisposableStackPrototypeDisposedGetter
        | StandardBuiltinId::DisposableStackPrototypeDisposedGetter => 0,
        StandardBuiltinId::AsyncDisposableStackPrototypeUse
        | StandardBuiltinId::AsyncDisposableStackPrototypeDefer
        | StandardBuiltinId::DisposableStackPrototypeUse
        | StandardBuiltinId::DisposableStackPrototypeDefer
        | StandardBuiltinId::AsyncDisposableStackDisposeAsyncFulfilled
        | StandardBuiltinId::AsyncDisposableStackDisposeAsyncRejected => 1,
        StandardBuiltinId::AsyncDisposableStackPrototypeAdopt
        | StandardBuiltinId::DisposableStackPrototypeAdopt => 2,
    }
}

pub(crate) fn host_builtin_length(builtin: HostBuiltinId) -> u64 {
    match builtin {
        HostBuiltinId::Print => 1,
        HostBuiltinId::Gc => 0,
        HostBuiltinId::AssertThrows => 2,
        HostBuiltinId::IsConstructor => 1,
        HostBuiltinId::CreateRealm => 0,
        HostBuiltinId::RealmEvalScript => 1,
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
        // BigInt arithmetic reports whether its result ended up inline or
        // heap-backed at runtime; the static kind cannot say which.
        ExprIr::BinaryNumber { lhs, rhs, .. } | ExprIr::CoerciveBinaryNumber { lhs, rhs, .. } => {
            lhs.possible_kinds.contains(ValueKind::BigInt)
                || rhs.possible_kinds.contains(ValueKind::BigInt)
        }
        // Every BigInt-capable bitwise operator may obtain a heap-backed
        // result through observable object coercion even when neither raw
        // operand advertises BigInt in its pre-ToPrimitive kind set.
        ExprIr::BitwiseNumeric { op, .. } => op.bigint_op().is_some(),
        ExprIr::UnaryMinusNumeric { .. } | ExprIr::UnaryBitwiseNumeric { .. } => true,
        ExprIr::UpdateIdentifier {
            value_kind: NumericUpdateValueKind::Dynamic,
            ..
        }
        | ExprIr::GlobalPropertyUpdate {
            value_kind: NumericUpdateValueKind::Dynamic,
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
        ExprIr::SuperPropertyMutation(mutation) => match mutation.operation() {
            SuperPropertyMutationOperationIr::NumericUpdate { value_kind, .. } => {
                *value_kind == NumericUpdateValueKind::Dynamic
            }
            SuperPropertyMutationOperationIr::EagerCompound { result, .. } => {
                expr_result_tag_is_runtime_dynamic(&result.expr)
            }
        },
        ExprIr::OrdinaryPropertyNumericUpdate(update) => {
            update.value_kind() == NumericUpdateValueKind::Dynamic
        }
        ExprIr::OrdinaryPropertyAssignment(assignment) => {
            expr_result_tag_is_runtime_dynamic(&assignment.rhs().expr)
        }
        ExprIr::OrdinaryPropertyLogicalAssignment(_) => true,
        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {
            expr_result_tag_is_runtime_dynamic(&mutation.result().expr)
        }
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
            value, evaluation, ..
        } => match *evaluation {
            ArrayDestructuringEvaluationIr::BindingInitialization => false,
            ArrayDestructuringEvaluationIr::AssignmentEvaluation => {
                expr_result_tag_is_runtime_dynamic(&value.expr)
            }
        },
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

fn count_for_in_of_binding_lexicals(
    mode: BindingMode,
    name: &str,
    lexical_environment: Option<&ForInOfEnvironmentIr>,
) -> usize {
    if mode == BindingMode::Var {
        return 0;
    }
    let Some(environment) = lexical_environment else {
        return 2;
    };
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
                .any(|binding| binding.name == name)
        }) {
        0
    } else {
        2
    };
    tdz_locals + iteration_locals
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
                    evaluation,
                    ..
                },
            ..
        }) => match *evaluation {
            ArrayDestructuringEvaluationIr::BindingInitialization => {
                let mut count = 0;
                pattern.visit_bindings(&mut |mode, _| {
                    count += usize::from(mode != BindingMode::Var) * 2
                });
                count
            }
            ArrayDestructuringEvaluationIr::AssignmentEvaluation => 0,
        },
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
        StatementIr::SyncDisposableScope {
            resources, body, ..
        } => resources.len() * 2 + count_block_lexicals(body),
        StatementIr::AsyncDisposableScope {
            resources, body, ..
        } => resources.len() * 2 + count_block_lexicals(body),
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
                    ForInitIr::Statements(statements) => {
                        statements.iter().map(count_statement_lexicals).sum()
                    }
                    ForInitIr::SyncDisposable(resources) => 2 * resources.len(),
                    ForInitIr::AsyncDisposable(init) => 2 * init.resources().len(),
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
                    ForInitIr::Statements(statements) => {
                        statements.iter().map(count_statement_lexicals).sum()
                    }
                    ForInitIr::SyncDisposable(resources) => 2 * resources.len(),
                    ForInitIr::AsyncDisposable(init) => 2 * init.resources().len(),
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
        StatementIr::AsyncFunctionForOfIterator { plan, .. } => {
            count_for_in_of_binding_lexicals(
                plan.value_mode(),
                plan.value_name(),
                plan.head_environment(),
            ) + plan
                .before_await()
                .iter()
                .chain(std::iter::once(plan.await_statement()))
                .chain(plan.after_await())
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
        StatementIr::ForOfIterator {
            head,
            body,
            lexical_environment,
            ..
        } => {
            let (mode, name) = match head {
                ForOfIteratorHeadIr::Assignment { binding, .. } => {
                    (binding.mode, binding.name.as_str())
                }
                ForOfIteratorHeadIr::SyncDisposable(head) => {
                    (BindingMode::Const, head.binding_name())
                }
                ForOfIteratorHeadIr::AsyncDisposable(head) => {
                    (BindingMode::Const, head.binding_name())
                }
            };
            count_for_in_of_binding_lexicals(mode, name, lexical_environment.as_ref())
                + count_statement_lexicals(body)
        }
        StatementIr::ForInArray {
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
            count_for_in_of_binding_lexicals(*mode, name, lexical_environment.as_ref())
                + count_statement_lexicals(body)
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

/// Shared PutValue phase: ordinary and resumed assignments retain the same
/// six evaluated operands before boxing the target, coercing the key and Set.
fn count_ordinary_property_assignment_completion_temp_locals() -> usize {
    let canonical_phase = ORDINARY_PROPERTY_ASSIGNMENT_CANONICAL_TEMP_LOCALS
        + ORDINARY_PROPERTY_ASSIGNMENT_TO_OBJECT_TEMP_LOCALS
            .max(ORDINARY_PROPERTY_ASSIGNMENT_TO_PROPERTY_KEY_TEMP_LOCALS);
    let write_phase = ORDINARY_PROPERTY_ASSIGNMENT_READY_TEMP_LOCALS
        + ORDINARY_PROPERTY_ASSIGNMENT_SET_HELPER_TEMP_LOCALS
            .max(ORDINARY_PROPERTY_FAILED_SET_ERROR_TEMP_LOCALS);
    canonical_phase.max(write_phase)
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
                GeneratorResumeModeIr::AssignProperty(reference) => {
                    let mut operand_locals = 0;
                    for_each_suspended_property_reference_operand(reference, |expr| {
                        operand_locals = operand_locals.max(count_expr_temp_locals(expr));
                    });
                    let preparation_phase =
                        ORDINARY_PROPERTY_ASSIGNMENT_RAW_TEMP_LOCALS + operand_locals;
                    preparation_phase
                        .max(count_ordinary_property_assignment_completion_temp_locals())
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
        StatementIr::SyncDisposableScope {
            execution,
            resources,
            body,
        } => count_sync_disposable_scope_temp_locals(
            execution,
            resources,
            count_block_temp_locals(body),
        ),
        StatementIr::AsyncDisposableScope {
            resources, body, ..
        } => count_async_disposable_scope_temp_locals(resources, count_block_temp_locals(body)),
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
        } => {
            let test_temps = test.as_ref().map(count_expr_temp_locals).unwrap_or(0);
            let update_temps = update.as_ref().map(count_expr_temp_locals).unwrap_or(0);
            let body_temps = count_statement_temp_locals(body);
            match init.as_ref() {
                Some(ForInitIr::SyncDisposable(resources)) => {
                    count_sync_disposable_resources_temp_locals(
                        resources,
                        test_temps.max(update_temps).max(body_temps),
                    )
                }
                Some(ForInitIr::AsyncDisposable(init)) => count_async_disposable_scope_temp_locals(
                    init.resources(),
                    test_temps.max(update_temps).max(body_temps),
                ),
                _ => init
                    .as_ref()
                    .map(count_for_init_temp_locals)
                    .unwrap_or(0)
                    .max(test_temps)
                    .max(update_temps)
                    .max(body_temps),
            }
        }
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
        StatementIr::AsyncFunctionForOfIterator { iterable, plan } => {
            RESUMABLE_SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS
                + count_expr_temp_locals(iterable)
                    .max(
                        plan.before_await()
                            .iter()
                            .chain(std::iter::once(plan.await_statement()))
                            .chain(plan.after_await())
                            .map(count_statement_temp_locals)
                            .max()
                            .unwrap_or(0),
                    )
                    .max(FOR_OF_ITERATOR_HELPER_TEMP_LOCALS)
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
        StatementIr::ForOfIterator {
            head,
            iterable,
            body,
            ..
        } => match head {
            ForOfIteratorHeadIr::Assignment { async_plan, .. } => {
                let persistent = if async_plan.is_some() {
                    ASYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS
                } else {
                    SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS
                };
                persistent
                    + count_expr_temp_locals(iterable)
                        .max(count_statement_temp_locals(body))
                        .max(FOR_OF_ITERATOR_HELPER_TEMP_LOCALS)
            }
            ForOfIteratorHeadIr::SyncDisposable(_) => {
                SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS
                    + 5
                    + count_expr_temp_locals(iterable)
                        .max(count_statement_temp_locals(body))
                        .max(SYNC_DISPOSABLE_SCOPE_COMPLETION_TEMP_LOCALS)
            }
            ForOfIteratorHeadIr::AsyncDisposable(_) => {
                ASYNC_DISPOSABLE_FOR_OF_PERSISTENT_TEMP_LOCALS
                    + count_expr_temp_locals(iterable)
                        .max(count_statement_temp_locals(body))
                        .max(
                            ACTIVATION_ASYNC_DISPOSE_WALKER_TEMP_LOCALS
                                + ACTIVATION_ASYNC_DISPOSE_HELPER_TEMP_LOCALS,
                        )
                        .max(ASYNC_DISPOSABLE_FOR_OF_BINDING_RESTORE_TEMP_LOCALS)
            }
        },
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
        ForInitIr::Statements(statements) => statements
            .iter()
            .map(count_statement_temp_locals)
            .max()
            .unwrap_or(0),
        ForInitIr::SyncDisposable(resources) => {
            count_sync_disposable_resources_temp_locals(resources, 0)
        }
        ForInitIr::AsyncDisposable(init) => {
            count_async_disposable_scope_temp_locals(init.resources(), 0)
        }
    }
}

fn call_args_have_spread(args: &[TypedExpr]) -> bool {
    args.iter()
        .any(|arg| matches!(arg.expr, ExprIr::SpreadArgument(_)))
}

/// Temp locals a Reference write holds for its carried `[[Strict]]`.
///
/// One, and the same one, at every site that calls
/// `FunctionBuilder::with_reference_strictness`. Named rather than spelled `1`
/// so the emitter side and the budget side move together: `reserve_temp_local`
/// asserts against the budget this function computes, so an emitter that grows
/// a second flag local and a planner that still says one is a panic in the
/// middle of code generation, not a compile error.
pub(crate) const REFERENCE_STRICTNESS_FLAG_LOCALS: usize = 1;

// Four captured-completion locals, seven disposal-walker locals, and the same
// 64-local indirect-call allowance used by `ExprIr::CallIndirect` below. The
// phases do not overlap initializer/body child temporaries, so their maximum
// is the accurate budget rather than an additive guess.
const SYNC_DISPOSABLE_SCOPE_COMPLETION_TEMP_LOCALS: usize = 4 + 7 + 64;

// The synchronous emitter holds seven tagged pairs, one property-key local and
// two four-local saved-completion bundles across iterable evaluation, iterator
// calls and the body. The async emitter instead holds its state, nine tagged
// pairs, key, close-method auxiliary, resume-kind flag and one saved bundle.
// Both call through the same conservative indirect-call allowance while those
// persistent locals remain live.
const SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS: usize = 7 * 2 + 1 + 2 * 4;
const ASYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS: usize = 1 + 9 * 2 + 1 + 1 + 1 + 4;
// The resumable synchronous emitter retains state, the eleven shared iterator
// locals, the iterator/return method pair, the raw done flag and two completion
// bundles. Iterable evaluation reuses the shared value pair before stepping.
const RESUMABLE_SYNC_FOR_OF_ITERATOR_PERSISTENT_TEMP_LOCALS: usize = 1 + 11 + 2 + 1 + 2 * 4;
const FOR_OF_ITERATOR_HELPER_TEMP_LOCALS: usize = 64;

// The activation-backed path holds five detached-capability locals while it
// materializes five locals per statically possible entry. Acquisition instead
// holds the active three-local capability and one five-local resource; its
// initializer/call phase never overlaps the body or detached walk.
const ACTIVATION_SYNC_DISPOSE_DETACHED_TEMP_LOCALS: usize = 5;
const ACTIVATION_SYNC_DISPOSE_ACTIVE_TEMP_LOCALS: usize = 3 + 5;

// Await-using acquisition holds the three-local published capability and one
// five-local resource while evaluating/validating that entry. The resumable
// finalizer instead holds its activation/capability cursor and one complete
// entry/call result bundle (17 locals); it never overlaps acquisition or the
// body and schedules at most one Await before returning to the driver.
const ACTIVATION_ASYNC_DISPOSE_ACTIVE_TEMP_LOCALS: usize = 3 + 5;
const ACTIVATION_ASYNC_DISPOSE_WALKER_TEMP_LOCALS: usize = 17;
const ACTIVATION_ASYNC_DISPOSE_HELPER_TEMP_LOCALS: usize = 64;

// State; iterable, method, Iterator, NextMethod, result, done and value pairs;
// key; two four-local saved-completion bundles; and the five-local acquired
// resource remain live across the selected phase. The walker/helper allowance
// is added by the exhaustive head arm above rather than hidden in this count.
const ASYNC_DISPOSABLE_FOR_OF_PERSISTENT_TEMP_LOCALS: usize = 1 + 7 * 2 + 1 + 2 * 4 + 5;
// Object/tag, boxed record, first entry and the entry's value pair are one
// bounded phase used only to restore a nested-body resume's immutable binding.
const ASYNC_DISPOSABLE_FOR_OF_BINDING_RESTORE_TEMP_LOCALS: usize = 6;

// Five mutation-result locals (old payload/tag, new payload/tag, Set result)
// stay live below the six-local raw/coerced Super Reference carrier. Each
// following constant is one non-overlapping phase above those persistent
// locals: GetValue's property-key/read work, dynamic ToNumeric, the selected
// ordinary-Set helper's four own locals plus its argument vector's two nested
// locals, and the carried-Strictness guard emitted only after that helper has
// returned and released its locals.
const SUPER_PROPERTY_MUTATION_PERSISTENT_TEMP_LOCALS: usize = 5 + 6;
const SUPER_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS: usize = 2;
const SUPER_PROPERTY_MUTATION_TO_NUMERIC_TEMP_LOCALS: usize = 4;
const SUPER_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS: usize = 4 + 2;

// A read/modify/write ordinary-property Reference is emitted in two
// non-overlapping phases. GetValue retains the two old-value locals below the
// four-local raw base/key carrier and the distinct two-local boxed target.
// Applying the result adds its payload/tag and the Set result local. The Set
// helper then reserves four own locals plus its two-local argument-vector
// phase. Strictness itself is a compile-time property of this fused Reference,
// so there is no carried flag local; a strict false Set still builds an error
// object while every write-persistent local remains live, however.
const ORDINARY_PROPERTY_MUTATION_READ_PERSISTENT_TEMP_LOCALS: usize = 2 + 4 + 2;
const ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS: usize = 2 + 4 + 2 + 3;
const ORDINARY_PROPERTY_MUTATION_TO_OBJECT_TEMP_LOCALS: usize = 2 + 3 + 3;
const ORDINARY_PROPERTY_MUTATION_TO_PROPERTY_KEY_TEMP_LOCALS: usize = 2;
const ORDINARY_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS: usize = 2;
const ORDINARY_PROPERTY_MUTATION_TO_NUMERIC_TEMP_LOCALS: usize = 4;
const ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS: usize = 4 + 2;
// `emit_runtime_error_object` retains object/key/value payload/value tag while
// `emit_object_define_data` materializes its three complete-descriptor flags.
const ORDINARY_PROPERTY_FAILED_SET_ERROR_TEMP_LOCALS: usize = 4 + 3;

// Plain assignment has three explicit, non-overlapping phases. Reference
// evaluation retains the four-local raw base/key carrier while its children
// run. RHS evaluation adds its payload/tag. PutValue then canonicalizes the
// retained raw key after reserving the distinct boxed target, then adds its
// single Set-result local. The final phase is the larger of the Set helper's
// four own locals plus two-local argument vector and a strict failed-Set error.
pub(crate) const ORDINARY_PROPERTY_ASSIGNMENT_RAW_TEMP_LOCALS: usize = 4;
pub(crate) const ORDINARY_PROPERTY_ASSIGNMENT_EVALUATED_TEMP_LOCALS: usize = 4 + 2;
const ORDINARY_PROPERTY_ASSIGNMENT_CANONICAL_TEMP_LOCALS: usize = 4 + 2 + 2;
const ORDINARY_PROPERTY_ASSIGNMENT_READY_TEMP_LOCALS: usize = 4 + 2 + 2 + 1;
// ToObject's widest branch boxes a string: prototype + wrapper object, three
// retained `length` definition operands, and three complete-descriptor flags.
// Its two UTF-16-length locals are a smaller nested phase under those first
// five, so the boxed-string peak is eight.
const ORDINARY_PROPERTY_ASSIGNMENT_TO_OBJECT_TEMP_LOCALS: usize = 2 + 3 + 3;
const ORDINARY_PROPERTY_ASSIGNMENT_TO_PROPERTY_KEY_TEMP_LOCALS: usize = 2;
const ORDINARY_PROPERTY_ASSIGNMENT_SET_HELPER_TEMP_LOCALS: usize = 4 + 2;

fn count_sync_disposable_resources_temp_locals(
    resources: &SyncDisposableResourcesIr,
    active_scope_temps: usize,
) -> usize {
    let initializer_temps = resources
        .iter()
        .map(|resource| count_expr_temp_locals(&resource.initializer))
        .max()
        .unwrap_or(0);
    resources.len() * 5
        + initializer_temps
            .max(active_scope_temps)
            .max(SYNC_DISPOSABLE_SCOPE_COMPLETION_TEMP_LOCALS)
}

fn count_sync_disposable_scope_temp_locals(
    execution: &SyncDisposableScopeExecutionIr,
    resources: &SyncDisposableResourcesIr,
    body_temps: usize,
) -> usize {
    match execution {
        SyncDisposableScopeExecutionIr::Immediate => {
            count_sync_disposable_resources_temp_locals(resources, body_temps)
        }
        SyncDisposableScopeExecutionIr::PlainGenerator(_)
        | SyncDisposableScopeExecutionIr::AsyncFunction(_)
        | SyncDisposableScopeExecutionIr::AsyncGenerator(_) => {
            let initializer_temps = resources
                .iter()
                .map(|resource| count_expr_temp_locals(&resource.initializer))
                .max()
                .unwrap_or(0);
            let acquisition_peak = ACTIVATION_SYNC_DISPOSE_ACTIVE_TEMP_LOCALS
                + initializer_temps.max(SYNC_DISPOSABLE_SCOPE_COMPLETION_TEMP_LOCALS);
            let disposal_peak = ACTIVATION_SYNC_DISPOSE_DETACHED_TEMP_LOCALS
                + resources.len() * 5
                + SYNC_DISPOSABLE_SCOPE_COMPLETION_TEMP_LOCALS;
            acquisition_peak.max(disposal_peak).max(body_temps)
        }
    }
}

fn count_async_disposable_scope_temp_locals(
    resources: &AsyncDisposableResourcesIr,
    body_temps: usize,
) -> usize {
    let initializer_temps = resources
        .iter()
        .map(|resource| count_expr_temp_locals(resource.initializer()))
        .max()
        .unwrap_or(0);
    let acquisition_peak = ACTIVATION_ASYNC_DISPOSE_ACTIVE_TEMP_LOCALS
        + initializer_temps.max(ACTIVATION_ASYNC_DISPOSE_HELPER_TEMP_LOCALS);
    let disposal_peak =
        ACTIVATION_ASYNC_DISPOSE_WALKER_TEMP_LOCALS + ACTIVATION_ASYNC_DISPOSE_HELPER_TEMP_LOCALS;
    acquisition_peak.max(disposal_peak).max(body_temps)
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
        // The three global-write arms each hold the Reference's carried
        // `[[Strict]]` in one extra temp local for the duration of the write
        // (`FunctionBuilder::emit_reference_global_property_write` ->
        // `with_reference_strictness`), so PutValue 3.d's guard can read it at
        // run time as well as 2.a's presence test reading it at compile time.
        ExprIr::GlobalPropertyWrite { value, .. } => {
            count_expr_temp_locals(value).max(12) + REFERENCE_STRICTNESS_FLAG_LOCALS
        }
        // These two additionally moved their write-back from the *unchecked*
        // `emit_global_property_write` (3 temps) to
        // `emit_global_property_write_checked` (4: it also holds
        // `has_property_local` for PutValue 2.a's presence test), so their base
        // is one higher than it was as well.
        ExprIr::GlobalPropertyUpdate { return_mode, .. } => {
            let base = match return_mode {
                UpdateReturnMode::Prefix => 13,
                UpdateReturnMode::Postfix => 14,
            };
            base + REFERENCE_STRICTNESS_FLAG_LOCALS
        }
        ExprIr::GlobalPropertyCompoundAssign { value, .. } => {
            count_expr_temp_locals(value).max(14) + REFERENCE_STRICTNESS_FLAG_LOCALS
        }
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
                    ObjectPropertyIr::ComputedMethod { key, .. }
                    | ObjectPropertyIr::ComputedGetter { key, .. }
                    | ObjectPropertyIr::ComputedSetter { key, .. } => count_expr_temp_locals(key),
                    ObjectPropertyIr::Method { .. }
                    | ObjectPropertyIr::Getter { .. }
                    | ObjectPropertyIr::Setter { .. } => 0,
                })
                .max()
                .unwrap_or(0);
            // A named getter followed by its paired setter retains both
            // accessor values while the second HomeObject-bearing function
            // context is materialized.
            child.max(13)
        }
        ExprIr::ArrayLiteral(elements) => {
            let child = elements
                .iter()
                .map(count_expr_temp_locals)
                .max()
                .unwrap_or(0);
            child.max(6)
        }
        ExprIr::ArrayAccumulation(accumulation) => {
            let child = array_accumulation_expressions(accumulation)
                .map(count_expr_temp_locals)
                .max()
                .unwrap_or(0);
            if array_accumulation_has_spread(accumulation) {
                child.saturating_add(32).max(256)
            } else {
                child.saturating_add(8).max(16)
            }
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
        ExprIr::PropertyWrite {
            target, key, value, ..
        } => {
            let child = count_expr_temp_locals(target)
                .max(count_expr_temp_locals(value))
                .max(match key {
                    PropertyKeyIr::StaticString(_) => 0,
                    PropertyKeyIr::ArrayLength => 0,
                    PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                        count_expr_temp_locals(expr)
                    }
                });
            // +1 for the Reference's carried `[[Strict]]` flag local, held
            // across the whole write by
            // `FunctionBuilder::with_reference_strictness`. `reserve_temp_local`
            // asserts against this budget, so the extra live local has to be
            // counted here and not discovered as a panic.
            child.max(96) + REFERENCE_STRICTNESS_FLAG_LOCALS
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
        ExprIr::OrdinaryPropertyAssignment(assignment) => {
            let key_child = match assignment.referenced_name() {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            let raw_phase = ORDINARY_PROPERTY_ASSIGNMENT_RAW_TEMP_LOCALS
                + count_expr_temp_locals(assignment.base_and_receiver()).max(key_child);
            let evaluated_phase = ORDINARY_PROPERTY_ASSIGNMENT_EVALUATED_TEMP_LOCALS
                + count_expr_temp_locals(assignment.rhs());
            raw_phase
                .max(evaluated_phase)
                .max(count_ordinary_property_assignment_completion_temp_locals())
        }
        ExprIr::OrdinaryPropertyLogicalAssignment(assignment) => {
            let key_child = match assignment.referenced_name() {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            let read_phase = ORDINARY_PROPERTY_MUTATION_READ_PERSISTENT_TEMP_LOCALS
                + count_expr_temp_locals(assignment.base_and_receiver())
                    .max(key_child)
                    .max(ORDINARY_PROPERTY_MUTATION_TO_OBJECT_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_MUTATION_TO_PROPERTY_KEY_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS);
            let taken_phase = ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS
                + count_expr_temp_locals(assignment.rhs())
                    .max(ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_FAILED_SET_ERROR_TEMP_LOCALS);
            read_phase.max(taken_phase)
        }
        ExprIr::OrdinaryPropertyNumericUpdate(update) => {
            let key_child = match update.referenced_name() {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            let to_numeric_temps = match update.value_kind() {
                NumericUpdateValueKind::Dynamic => {
                    ORDINARY_PROPERTY_MUTATION_TO_NUMERIC_TEMP_LOCALS
                }
                NumericUpdateValueKind::Number | NumericUpdateValueKind::BigInt => 0,
            };
            let read_phase = ORDINARY_PROPERTY_MUTATION_READ_PERSISTENT_TEMP_LOCALS
                + count_expr_temp_locals(update.base_and_receiver())
                    .max(key_child)
                    .max(ORDINARY_PROPERTY_MUTATION_TO_OBJECT_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_MUTATION_TO_PROPERTY_KEY_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS)
                    .max(to_numeric_temps);
            let write_phase = ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS
                + ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS
                    .max(ORDINARY_PROPERTY_FAILED_SET_ERROR_TEMP_LOCALS);
            read_phase.max(write_phase)
        }
        ExprIr::OrdinaryPropertyEagerCompoundAssignment(mutation) => {
            let key_child = match mutation.referenced_name() {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            let read_phase = ORDINARY_PROPERTY_MUTATION_READ_PERSISTENT_TEMP_LOCALS
                + count_expr_temp_locals(mutation.base_and_receiver())
                    .max(key_child)
                    .max(ORDINARY_PROPERTY_MUTATION_TO_OBJECT_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_MUTATION_TO_PROPERTY_KEY_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS);
            let write_phase = ORDINARY_PROPERTY_MUTATION_WRITE_PERSISTENT_TEMP_LOCALS
                + count_expr_temp_locals(mutation.result())
                    .max(ORDINARY_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS)
                    .max(ORDINARY_PROPERTY_FAILED_SET_ERROR_TEMP_LOCALS);
            read_phase.max(write_phase)
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
        | ExprIr::UnaryPlus { expr: value }
        | ExprIr::StringFromCharCode { code: value }
        | ExprIr::LogicalNot { expr: value }
        | ExprIr::Void { expr: value }
        | ExprIr::DeleteValue { expr: value } => count_expr_temp_locals(value),
        ExprIr::UnaryMinusNumeric { expr: value } => 2 + count_expr_temp_locals(value).max(2),
        ExprIr::UnaryBitwiseNumeric { expr: value, .. } => 2 + count_expr_temp_locals(value).max(2),
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
        ExprIr::BitwiseNumeric { lhs, rhs, .. } => {
            4 + count_expr_temp_locals(lhs)
                .max(count_expr_temp_locals(rhs))
                .max(2)
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
                            // The write-back runs under
                            // `with_reference_strictness`, which reserves the
                            // carried-`[[Strict]]` flag local, so this budget
                            // carries the same `REFERENCE_STRICTNESS_FLAG_LOCALS`
                            // the reference-write expression arms do.
                            DestructuringTargetIr::AssignmentProperty {
                                target,
                                key,
                                strictness: _,
                            } => {
                                4 + REFERENCE_STRICTNESS_FLAG_LOCALS
                                    + count_expr_temp_locals(target).max(match key {
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
                            DestructuringTargetIr::AssignmentIdentifier(reference) => {
                                // The fixed 32-local destructuring allowance
                                // covers the checked global writer itself; a
                                // global Reference additionally holds its
                                // carried strictness flag across that write.
                                match reference.write_disposition() {
                                    IdentifierWriteDisposition::Global { .. } => {
                                        REFERENCE_STRICTNESS_FLAG_LOCALS
                                    }
                                    IdentifierWriteDisposition::MutableBinding { .. }
                                    | IdentifierWriteDisposition::IgnoreImmutableBinding
                                    | IdentifierWriteDisposition::Throw { .. } => 0,
                                }
                            }
                            DestructuringTargetIr::Binding { .. } => 0,
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
        ExprIr::SpreadArgument(spread) => count_expr_temp_locals(&spread.value).max(2),
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
        ExprIr::JsonParseStaticReviver {
            callee,
            input,
            reviver,
            ..
        } => count_expr_temp_locals(callee)
            .max(count_expr_temp_locals(input))
            .max(count_expr_temp_locals(reviver))
            .max(64),
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
        ExprIr::SuperPropertyRead { key, receiver } => match key {
            PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => {
                count_expr_temp_locals(receiver).max(8)
            }
            PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                count_expr_temp_locals(expr)
                    .max(count_expr_temp_locals(receiver))
                    .max(8)
            }
        },
        ExprIr::SuperPropertyWrite {
            key,
            receiver,
            value,
            ..
        } => {
            let key_child = match key {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            count_expr_temp_locals(receiver)
                .max(count_expr_temp_locals(value))
                .max(key_child)
                .max(12)
                + REFERENCE_STRICTNESS_FLAG_LOCALS
        }
        ExprIr::SuperPropertyMutation(mutation) => {
            let key_child = match mutation.referenced_name() {
                PropertyKeyIr::StaticString(_) | PropertyKeyIr::ArrayLength => 0,
                PropertyKeyIr::StringExpr(expr) | PropertyKeyIr::ArrayIndex(expr) => {
                    count_expr_temp_locals(expr)
                }
            };
            let operation_child = match mutation.operation() {
                SuperPropertyMutationOperationIr::NumericUpdate { .. } => 0,
                SuperPropertyMutationOperationIr::EagerCompound { result, .. } => {
                    count_expr_temp_locals(result)
                }
            };
            SUPER_PROPERTY_MUTATION_PERSISTENT_TEMP_LOCALS
                + count_expr_temp_locals(mutation.receiver())
                    .max(key_child)
                    .max(operation_child)
                    .max(SUPER_PROPERTY_MUTATION_GET_VALUE_TEMP_LOCALS)
                    .max(SUPER_PROPERTY_MUTATION_TO_NUMERIC_TEMP_LOCALS)
                    .max(SUPER_PROPERTY_MUTATION_SET_HELPER_TEMP_LOCALS)
                    .max(REFERENCE_STRICTNESS_FLAG_LOCALS)
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

/// A loop head can declare `var`s in every form except `Expression`, so this
/// walks the init exhaustively. The `Statements` form matters most: a pattern
/// head (`for (var { x } = o; …)`) lowers to ordinary `StatementIr::Var`
/// declarators, and missing them here leaves `x` unallocated at emit time.
pub(crate) fn collect_hoisted_vars_for_init(init: &ForInitIr, names: &mut BTreeSet<String>) {
    match init {
        ForInitIr::Var(declarators) => {
            for declarator in declarators {
                names.insert(declarator.name.clone());
            }
        }
        ForInitIr::Statements(statements) => {
            for statement in statements {
                collect_hoisted_vars_statement(statement, names);
            }
        }
        ForInitIr::Lexical { .. }
        | ForInitIr::LexicalBlock(_)
        | ForInitIr::Expression(_)
        | ForInitIr::SyncDisposable(_)
        | ForInitIr::AsyncDisposable(_) => {}
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
        StatementIr::AnnexBFunctionCopy { target, .. } => {
            let name = match target {
                AnnexBFunctionCopyTargetIr::OwnerBinding { storage_name } => storage_name,
                AnnexBFunctionCopyTargetIr::ScriptGlobal { name } => name,
            };
            names.insert(name.clone());
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
            if let Some(init) = init {
                collect_hoisted_vars_for_init(init, names);
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
            if let Some(init) = init {
                collect_hoisted_vars_for_init(init, names);
            }
            for statement in before_suspension {
                collect_hoisted_vars_statement(statement, names);
            }
            collect_hoisted_vars_statement(suspension_statement, names);
            for statement in after_suspension {
                collect_hoisted_vars_statement(statement, names);
            }
        }
        StatementIr::AsyncFunctionForOfIterator { plan, .. } => {
            if plan.value_mode() == BindingMode::Var {
                names.insert(plan.value_name().to_string());
            }
            for statement in plan
                .before_await()
                .iter()
                .chain(std::iter::once(plan.await_statement()))
                .chain(plan.after_await())
            {
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
        StatementIr::ForOfIterator { head, body, .. } => {
            if let ForOfIteratorHeadIr::Assignment { binding, .. } = head {
                if binding.mode == BindingMode::Var {
                    names.insert(binding.name.clone());
                }
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
        StatementIr::SyncDisposableScope { body, .. } => {
            collect_hoisted_vars_block(body, names);
        }
        StatementIr::AsyncDisposableScope { body, .. } => {
            collect_hoisted_vars_block(body, names);
        }
        StatementIr::Expression(TypedExpr {
            expr:
                ExprIr::ArrayDestructure {
                    pattern,
                    evaluation,
                    ..
                },
            ..
        }) => match *evaluation {
            ArrayDestructuringEvaluationIr::BindingInitialization => {
                pattern.visit_bindings(&mut |mode, name| {
                    if mode == BindingMode::Var {
                        names.insert(name.to_string());
                    }
                });
            }
            ArrayDestructuringEvaluationIr::AssignmentEvaluation => {}
        },
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
