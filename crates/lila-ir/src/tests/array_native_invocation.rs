fn retained_indexed_collection_call<'a>(
    expression: &'a TypedExpr,
    source_key: &str,
) -> (&'a TypedExpr, &'a [TypedExpr]) {
    let ExprIr::MaterializeBinding { name, body, .. } = &expression.expr else {
        panic!("expected one receiver evaluation: {expression:?}");
    };
    let ExprIr::CallIndirect {
        callee,
        this_arg: Some(receiver),
        args,
        ..
    } = &body.expr
    else {
        panic!("expected a call of the acquired source property: {body:?}");
    };
    assert!(matches!(&receiver.expr, ExprIr::Identifier(storage) if storage == name));
    assert!(
        match &callee.expr {
            ExprIr::PropertyRead {
                target,
                key: PropertyKeyIr::StaticString(key),
            } =>
                matches!(&target.expr, ExprIr::Identifier(storage) if storage == name)
                    && key == source_key,
            ExprIr::SpecOperation {
                operation: SpecOperationIr::GetV,
                operands,
            } =>
                operands.len() == 2
                    && matches!(&operands[0].expr, ExprIr::Identifier(storage) if storage == name)
                    && matches!(&operands[1].expr, ExprIr::String(key) if key == source_key),
            _ => false,
        },
        "source Reference must supply both callee and receiver: {callee:?}"
    );
    (callee, args)
}

#[test]
fn inferred_indexed_collection_aliases_retain_each_source_reference() {
    // Each case is lowered separately: a runtime method table would merge
    // the native targets and exercise the generic invocation fallback.
    let cases = [
        (
            StandardBuiltinId::ArrayPrototypePush,
            "Array",
            "push",
            "3",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::ArrayPrototypeUnshift,
            "Array",
            "unshift",
            "3",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::ArrayPrototypeIndexOf,
            "Array",
            "indexOf",
            "1",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::ArrayPrototypeLastIndexOf,
            "Array",
            "lastIndexOf",
            "1",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::ArrayPrototypeFindIndex,
            "Array",
            "findIndex",
            "callback",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::ArrayPrototypeFindLastIndex,
            "Array",
            "findLastIndex",
            "callback",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeIndexOf,
            "Int8Array",
            "indexOf",
            "1",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeLastIndexOf,
            "Int8Array",
            "lastIndexOf",
            "1",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeFindIndex,
            "Int8Array",
            "findIndex",
            "callback",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeFindLastIndex,
            "Int8Array",
            "findLastIndex",
            "callback",
            ValueKind::Number,
        ),
        (
            StandardBuiltinId::ArrayPrototypeIncludes,
            "Array",
            "includes",
            "1",
            ValueKind::Boolean,
        ),
        (
            StandardBuiltinId::ArrayPrototypeEvery,
            "Array",
            "every",
            "callback",
            ValueKind::Boolean,
        ),
        (
            StandardBuiltinId::ArrayPrototypeSome,
            "Array",
            "some",
            "callback",
            ValueKind::Boolean,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeIncludes,
            "Int8Array",
            "includes",
            "1",
            ValueKind::Boolean,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeEvery,
            "Int8Array",
            "every",
            "callback",
            ValueKind::Boolean,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeSome,
            "Int8Array",
            "some",
            "callback",
            ValueKind::Boolean,
        ),
        (
            StandardBuiltinId::ArrayPrototypeJoin,
            "Array",
            "join",
            "':'",
            ValueKind::String,
        ),
        (
            StandardBuiltinId::ArrayPrototypeToLocaleString,
            "Array",
            "toLocaleString",
            "",
            ValueKind::String,
        ),
        (
            StandardBuiltinId::ArrayPrototypeForEach,
            "Array",
            "forEach",
            "callback",
            ValueKind::Undefined,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeForEach,
            "Int8Array",
            "forEach",
            "callback",
            ValueKind::Undefined,
        ),
        (
            StandardBuiltinId::ArrayPrototypeConcat,
            "Array",
            "concat",
            "3",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeSlice,
            "Array",
            "slice",
            "0, 1",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeSplice,
            "Array",
            "splice",
            "0, 1, 3",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeFlat,
            "Array",
            "flat",
            "1",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeFlatMap,
            "Array",
            "flatMap",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeMap,
            "Array",
            "map",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeFilter,
            "Array",
            "filter",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeToReversed,
            "Array",
            "toReversed",
            "",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeToSpliced,
            "Array",
            "toSpliced",
            "0, 1, 3",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeToSorted,
            "Array",
            "toSorted",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeWith,
            "Array",
            "with",
            "0, 3",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeFill,
            "Array",
            "fill",
            "3",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeSort,
            "Array",
            "sort",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeReverse,
            "Array",
            "reverse",
            "",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeCopyWithin,
            "Array",
            "copyWithin",
            "0, 1",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypePop,
            "Array",
            "pop",
            "",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeShift,
            "Array",
            "shift",
            "",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeAt,
            "Array",
            "at",
            "0",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeFind,
            "Array",
            "find",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeFindLast,
            "Array",
            "findLast",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeReduce,
            "Array",
            "reduce",
            "callback, 0",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::ArrayPrototypeReduceRight,
            "Array",
            "reduceRight",
            "callback, 0",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeFind,
            "Int8Array",
            "find",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeFindLast,
            "Int8Array",
            "findLast",
            "callback",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeReduce,
            "Int8Array",
            "reduce",
            "callback, 0",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeReduceRight,
            "Int8Array",
            "reduceRight",
            "callback, 0",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeToString,
            "Int8Array",
            "toString",
            "",
            ValueKind::Dynamic,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeMap,
            "Int8Array",
            "map",
            "callback",
            ValueKind::Object,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeFilter,
            "Int8Array",
            "filter",
            "callback",
            ValueKind::Object,
        ),
        (
            StandardBuiltinId::ArrayPrototypeKeys,
            "Array",
            "keys",
            "",
            ValueKind::Object,
        ),
        (
            StandardBuiltinId::ArrayPrototypeEntries,
            "Array",
            "entries",
            "",
            ValueKind::Object,
        ),
        (
            StandardBuiltinId::ArrayPrototypeValues,
            "Array",
            "values",
            "",
            ValueKind::Object,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeKeys,
            "Int8Array",
            "keys",
            "",
            ValueKind::Object,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeEntries,
            "Int8Array",
            "entries",
            "",
            ValueKind::Object,
        ),
        (
            StandardBuiltinId::TypedArrayPrototypeValues,
            "Int8Array",
            "values",
            "",
            ValueKind::Object,
        ),
    ];
    assert_eq!(cases.len(), 55);
    let mut seen = std::collections::BTreeSet::new();
    for (builtin, constructor, method, arguments, expected_kind) in cases {
        assert!(seen.insert(builtin.function_id()));
        let receiver = if constructor == "Array" {
            "[1, 2]"
        } else {
            "new Int8Array(2)"
        };
        let source = format!(
            "function run() {{ function callback(value) {{ return value; }} \
                 const receiver = {receiver}; receiver.saved = {constructor}.prototype.{method}; \
                 return receiver.saved({arguments}); }} run();"
        );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{builtin:?}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("returned native call");
        let (callee, _) = retained_indexed_collection_call(result, "saved");
        // The ordinary `saved` Put can invoke a prototype setter. Both Array
        // and TypedArray transfers must acquire the actual subsequent Get.
        assert!(
            matches!(callee.function_targets, FunctionTargetKnowledge::Open(_)),
            "{builtin:?}: {callee:?}"
        );
        assert_eq!(
            result.kind,
            ValueKind::Dynamic,
            "unguarded {expected_kind:?} result for {builtin:?}: {result:?}"
        );
        assert_eq!(result.possible_kinds, KindSet::all_runtime_tags());
        assert!(result.heap_shape.is_none(), "{builtin:?}: {result:?}");
    }
}

#[test]
fn indexed_collection_reference_keeps_spread_and_trailing_argument_order() {
    let program = lower_script(
        "function run() { const receiver = [1, 2]; receiver.saved = Array.prototype.join; \
             return (0, receiver).saved(':', ...Object.keys({ x: 1 }), 'tail'); } run();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let run = program
        .script
        .as_ref()
        .expect("script")
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run");
    let result = function_return(run).expect("join result");
    let (_, args) = retained_indexed_collection_call(result, "saved");
    assert_eq!(result.kind, ValueKind::Dynamic);
    assert_eq!(result.possible_kinds, KindSet::all_runtime_tags());
    let [separator, spread, trailing] = args else {
        panic!("{args:?}")
    };
    assert!(matches!(&separator.expr, ExprIr::String(value) if value == ":"));
    assert!(matches!(&spread.expr, ExprIr::SpreadArgument(value)
            if value.protocol == SpreadArgumentProtocol::ARGUMENT_LIST));
    assert!(matches!(&trailing.expr, ExprIr::String(value) if value == "tail"));
}

#[test]
fn transferred_comparator_and_typed_predicate_calls_widen_prior_omitted_parameters() {
    for method in ["sort", "toSorted"] {
        let source = format!(
            "function compare(left, right) {{ return 0; }} compare(); \
                 const receiver = [2, 1]; receiver.saved = Array.prototype.{method}; \
                 receiver.saved(compare);"
        );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{method}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script");
        let compare = script
            .functions
            .iter()
            .find(|function| function.name == "compare")
            .expect("comparator");
        assert!(
            compare
                .params
                .iter()
                .all(|param| param.kind == ValueKind::Dynamic),
            "{method}: {:?}",
            compare.params
        );
    }
    for (method, expected) in [
        ("every", ValueKind::Boolean),
        ("some", ValueKind::Boolean),
        ("find", ValueKind::Dynamic),
        ("findIndex", ValueKind::Number),
        ("findLast", ValueKind::Dynamic),
        ("findLastIndex", ValueKind::Number),
    ] {
        // The first call has no user effects; its omitted arguments narrow
        // parameters without routing the later native call to a fallback.
        let source = format!(
            "function predicate(value, index, receiver) {{ return value; }} predicate(); \
                 const receiver = new Int8Array(2); receiver.saved = Int8Array.prototype.{method}; \
                 receiver.saved(predicate);"
        );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{method}: {:?}",
            program.diagnostics
        );
        let script = program.script.as_ref().expect("script");
        let predicate = script
            .functions
            .iter()
            .find(|function| function.name == "predicate")
            .expect("predicate");
        assert!(
            predicate
                .params
                .iter()
                .all(|param| param.kind == ValueKind::Dynamic),
            "{method}: {:?}",
            predicate.params
        );
        let StatementIr::Expression(result) = script.body.statements.last().expect("call") else {
            panic!("final predicate call")
        };
        retained_indexed_collection_call(result, "saved");
        // The observable `saved` Put does not prove the following Get returns
        // that native method, although its possible callback must be widened.
        assert_eq!(
            result.kind,
            ValueKind::Dynamic,
            "unguarded {expected:?} result for {method}: {result:?}"
        );
        assert_eq!(result.possible_kinds, KindSet::all_runtime_tags());
    }
}

#[test]
fn indexed_collection_call_results_do_not_invent_species_or_join_shapes() {
    for (method, arguments) in [
        ("concat", "[2]"),
        ("slice", "0"),
        ("splice", "0, 1"),
        ("flat", "1"),
        ("flatMap", "function(value) { return value; }"),
        ("map", "function(value) { return value; }"),
        ("filter", "function(value) { return value; }"),
        ("fill", "2"),
        ("sort", ""),
    ] {
        let comma = if arguments.is_empty() { "" } else { ", " };
        let source = format!(
            "function run() {{ const saved = Array.prototype.{method}; \
                 return saved.call([1]{comma}{arguments}); }} run();"
        );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("native result");
        assert_eq!(result.kind, ValueKind::Dynamic, "{source}: {result:?}");
        assert!(result.heap_shape.is_none(), "{source}: {result:?}");
        for kind in [
            ValueKind::Array,
            ValueKind::Object,
            ValueKind::Function,
            ValueKind::Arguments,
        ] {
            assert!(result.possible_kinds.contains(kind), "{source}: {result:?}");
        }
    }
    let program = lower_script(
        "function run() { const saved = Array.prototype.toString; \
             return saved.call({ join() { return 17; } }); } run();",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let run = program
        .script
        .as_ref()
        .expect("script")
        .functions
        .iter()
        .find(|function| function.name == "run")
        .expect("run");
    let result = function_return(run).expect("join result");
    assert_eq!(result.kind, ValueKind::Dynamic);
    assert_eq!(result.possible_kinds, KindSet::all_runtime_tags());
    assert!(result.heap_shape.is_none());
    assert!(StandardBuiltinId::TypedArrayPrototypeToString.may_run_user_code_synchronously());
}

#[test]
fn indexed_collection_hooks_invalidate_captured_caller_facts() {
    for source in [
            "let captured = { value: 1 }; const receiver = { length: 1, \
             get 0() { captured.value = 'changed'; return 1; }, saved: Array.prototype.join }; \
             receiver.saved(); captured.value + 1;",
            "let captured = { value: 1 }; const receiver = { \
             join() { captured.value = 'changed'; return 1; }, saved: Int8Array.prototype.toString }; \
             receiver.saved(); captured.value + 1;",
        ] {
            assert_caller_flow_invalidation_reaches_final_addition(source);
        }
}

#[test]
fn fresh_iterator_results_do_not_prove_mutable_inherited_method_targets() {
    for (constructor, method, receiver) in [
        ("Array", "keys", "[1]"),
        ("Array", "entries", "[1]"),
        ("Array", "values", "[1]"),
        ("Int8Array", "keys", "new Int8Array(1)"),
        ("Int8Array", "entries", "new Int8Array(1)"),
        ("Int8Array", "values", "new Int8Array(1)"),
        ("String", "[Symbol.iterator]", "'x'"),
    ] {
        let access = if constructor == "String" {
            "[Symbol.iterator]".to_owned()
        } else {
            format!(".{method}")
        };
        for mutate in [false, true] {
            let mutation = if mutate {
                "Object.getPrototypeOf(saved.call(receiver)).next = function () { return 17; };"
            } else {
                ""
            };
            let source = format!(
                "function run() {{ const saved = {constructor}.prototype{access}; \
                     const receiver = {receiver}; {mutation} \
                     const fresh = saved.call(receiver); return fresh.next; }} run();"
            );
            let program = lower_script(&source);
            assert!(
                program.is_wasm_supported(),
                "{source}: {:?}",
                program.diagnostics
            );
            let run = program
                .script
                .as_ref()
                .expect("script")
                .functions
                .iter()
                .find(|function| function.name == "run")
                .expect("run");
            let read = function_return(run).expect("inherited next read");
            assert!(
                read.function_targets.known_targets().is_empty(),
                "{source}: {read:?}"
            );
            assert!(read.heap_shape.is_none(), "{source}: {read:?}");
        }
    }
}

#[test]
fn direct_array_calls_retain_the_acquired_reference_without_unguarded_result_facts() {
    for (method, arguments) in [
        ("join", "':'"),
        ("join", ""),
        ("toString", ""),
        ("reverse", ""),
    ] {
        let source = format!(
                "function run() {{ const receiver = [1, 'two']; return receiver.{method}({arguments}); }} run();"
            );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("method result");
        let (callee, args) = retained_indexed_collection_call(result, method);
        assert!(matches!(
            callee.function_targets,
            FunctionTargetKnowledge::Open(_)
        ));
        assert_eq!(args.len(), usize::from(!arguments.is_empty()));
        assert_eq!(result.kind, ValueKind::Dynamic, "{source}: {result:?}");
        assert!(result.heap_shape.is_none(), "{source}: {result:?}");
        assert_eq!(result.possible_kinds, KindSet::all_runtime_tags());
    }
}

#[test]
fn direct_array_calls_keep_the_runtime_reference_after_an_own_method_write() {
    for method in ["join", "toString", "reverse"] {
        let source = format!(
                "function run() {{ const receiver = []; receiver.{method} = function () {{ return 17; }}; \
                 return receiver.{method}(); }} run();"
            );
        let program = lower_script(&source);
        assert!(
            program.is_wasm_supported(),
            "{source}: {:?}",
            program.diagnostics
        );
        let run = program
            .script
            .as_ref()
            .expect("script")
            .functions
            .iter()
            .find(|function| function.name == "run")
            .expect("run");
        let result = function_return(run).expect("overridden method result");
        let (callee, args) = retained_indexed_collection_call(result, method);
        // Set may encounter an inherited setter. The call must acquire the
        // actual property even when that prevents an exact return-type proof.
        assert!(callee.function_targets.exact_single_target().is_none());
        assert!(args.is_empty());
        assert!(result.possible_kinds.contains(ValueKind::Number));
        assert_eq!(result.kind, ValueKind::Dynamic, "{source}: {result:?}");
    }
}

#[test]
fn direct_reverse_discards_prior_index_facts_after_mutation() {
    assert_caller_flow_invalidation_reaches_final_addition(
        "let values = [1, 'two']; values.reverse(); values[0] + 1;",
    );
}
