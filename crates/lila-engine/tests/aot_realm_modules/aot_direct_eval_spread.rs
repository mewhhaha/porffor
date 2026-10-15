use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn run_boolean(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("prepared eval spread keeps ordinary argument evaluation");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn empty_leading_iterables_select_the_first_collected_argument() {
    for strict in ["", "'use strict';"] {
        run_boolean(&format!(
            r#"
{strict}
var nextCount = 0;
var iterable = {{}};
iterable[Symbol.iterator] = function() {{
    return {{ next: function() {{ nextCount++; return {{ done: true }}; }} }};
}};
var value = 'global';
function caller() {{
    var value = 'local';
    eval(...iterable, ...[], 'value = 0;');
    return value;
}}
caller() === 0 && value === 'global' && nextCount === 1;
"#
        ));
    }
}

#[test]
fn custom_iterators_evaluate_every_argument_and_only_the_first_source() {
    for strict in ["", "'use strict';"] {
        run_boolean(&format!(
            r#"
{strict}
var texts = ['value = 1;', 'value = 2;'];
var nextCount = 0;
var iterable = {{}};
iterable[Symbol.iterator] = function() {{
    return {{ next: function() {{
        var index = nextCount++;
        if (index < texts.length) return {{ done: false, value: texts[index] }};
        return {{ done: true }};
    }} }};
}};
var value = 'global';
function caller() {{ var value = 'local'; eval(...iterable); return value; }}
caller() === 1 && value === 'global' && nextCount === 3;
"#
        ));
    }
}

#[test]
fn iterator_gets_and_source_dispatch_follow_the_captured_callee() {
    run_boolean(
        r#"
var original = eval;
var trace = '';
var texts = ["trace += 'E'; value = 1;", 'value = 2;'];
var index = 0;
var iterable = {
    get [Symbol.iterator]() {
        trace += 'I';
        return function() {
            trace += 'M';
            return { get next() {
                trace += 'N';
                return function() {
                    trace += 'n';
                    var i = index++;
                    return {
                        get done() { trace += 'd'; return i >= texts.length; },
                        get value() {
                            trace += 'v';
                            Object.defineProperty(globalThis, 'eval', {
                                configurable: true, writable: true,
                                value: function() { throw 'replacement must not run'; }
                            });
                            return texts[i];
                        }
                    };
                };
            } };
        };
    }
};
Object.defineProperty(globalThis, 'eval', {
    configurable: true, get() { trace += 'C'; return original; }
});
var value = 'global';
function caller() {
    var value = 'local';
    eval(...iterable, (trace += 'l'));
    return value;
}
var answer = caller();
globalThis.eval = original;
answer === 1 && value === 'global' && index === 3 && trace === 'CIMNndvndvndlE';
"#,
    );
}

#[test]
fn iterator_abrupt_completions_skip_eval_and_later_arguments() {
    run_boolean(
        r#"
var marker = {};
var stage = '';
var ran = 0;
var later = 0;
var closed = 0;
var index = 0;
var iterable = {
    get [Symbol.iterator]() {
        if (stage === 'iterator') throw marker;
        return function() {
            return {
                get next() {
                    if (stage === 'next-get') throw marker;
                    return function() {
                        if (stage === 'next-call') throw marker;
                        var i = index++;
                        return {
                            get done() { if (stage === 'done') throw marker; return i > 0; },
                            get value() { if (stage === 'value') throw marker; return 'ran += 1;'; }
                        };
                    };
                },
                return() { closed++; return {}; }
            };
        };
    }
};
var caught = 0;
for (stage of ['iterator', 'next-get', 'next-call', 'done', 'value']) {
    index = 0;
    try { eval(...iterable, later++); } catch (error) {
        if (error !== marker) throw 'wrong abrupt identity';
        caught++;
    }
}
caught === 5 && ran === 0 && later === 0 && closed === 0;
"#,
    );
}

#[test]
fn replacement_eval_and_non_string_arguments_keep_ordinary_semantics() {
    run_boolean(
        r#"
var original = eval;
var calls = 0;
var empty = { [Symbol.iterator]() { return { next() { calls++; return { done: true }; } }; } };
function invoke(eval) { return eval(...empty, 'let duplicate; let duplicate;', 17); }
var replacement = invoke(function(source, value) { return value; });
var object = { toString() { throw 'eval must not coerce'; } };
var passthrough = eval(...[object], calls++);
var absent = eval(...empty);
replacement === 17 && passthrough === object && absent === undefined && calls === 3 && eval === original;
"#,
    );
}

#[test]
fn spread_source_syntax_errors_follow_all_argument_effects() {
    run_boolean(
        r#"
var trace = '';
var texts = ['let duplicate; let duplicate;'];
var index = 0;
var iterable = { [Symbol.iterator]() { return { next() {
    trace += 'n';
    return { done: index > 0, value: texts[index++] };
} }; } };
var caught = false;
try { eval(...iterable, (trace += 'l')); } catch (error) { caught = error instanceof SyntaxError; }
caught && trace === 'nnl';
"#,
    );
}

#[test]
fn for_of_array_bindings_prepare_live_helper_eval_without_replacing_binding_values() {
    for strict in ["", "'use strict';"] {
        run_boolean(&format!(
            r#"
{strict}
function evaluate(source) {{ var local = 40; return eval(source); }}
var rows = [['local + 1', true], ['local + 2', false]];
var sum = 0, flags = '';
for (var [source, named] of rows) {{ sum += evaluate(source); flags += named ? 'T' : 'F'; }}
for (let [source, named] of rows) {{ const read = () => source; sum += evaluate(read()); flags += named ? 'T' : 'F'; }}
for (const [[source], named] of [[['local + 1'], true], [['local + 2'], false]]) {{
    sum += evaluate(source); flags += named ? 'T' : 'F';
}}
var source = 'local - 1';
var empty = [['local + 7']];
empty[Symbol.iterator] = function() {{ return {{next() {{ return {{done: true}}; }}}}; }};
for (var [source] of empty) {{ throw 'empty iterator entered its body'; }}
sum === 249 && flags === 'TFTFTF' && evaluate(source) === 39;
"#
        ));
    }
}

#[test]
fn for_of_array_source_hints_preserve_iterator_defaults_and_reassignment_effects() {
    for strict in ["", "'use strict';"] {
        run_boolean(&format!(
            r#"
{strict}
function evaluate(source) {{ var local = 40; return eval(source); }}
var rows = [['local + 1', true], ['local + 2', false]];
var trace = '', sum = 0, flags = '';
rows[0][Symbol.iterator] = function() {{
    trace += 'i';
    var index = 0;
    return {{
        next() {{ trace += 'n'; return {{done: false, value: index++ === 0 ? 'local + 2' : false}}; }},
        return() {{ trace += 'r'; return {{}}; }}
    }};
}};
for (let [source, named] of rows) {{
    trace += 'b'; sum += evaluate(source); flags += named ? 'T' : 'F';
}}
var defaults = 0;
for (const [source = (defaults++, 'local + 3')] of [[undefined], ['local + 1']]) {{
    sum += evaluate(source);
}}
for (let [source] of [['local + 1']]) {{ source = 'local + 4'; sum += evaluate(source); }}
trace === 'innrbb' && sum === 212 && flags === 'FF' && defaults === 1;
"#
        ));
    }
}

#[test]
fn for_of_array_source_hints_reject_unprepared_reassigned_strings_at_the_existing_guard() {
    use lila_engine::WasmExecutionFailureKind;
    use lila_ir::DynamicSourceRuntimeOperation;

    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    for strict in ["", "'use strict';"] {
        let error = Engine::new(RealmBuilder::new().build())
            .run_script(
                &format!(
                    r#"
{strict}
for (var [source] of [['40 + 2']]) {{
    source = 'throw 997; /*' + Math.random() + '*/';
    try {{ eval(source); }} catch (error) {{ throw 'capability rejection became catchable'; }}
}}
throw 'unprepared source continued';
"#
                ),
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect_err("projection must not authorize an unmatched live source");
        assert_eq!(
            error.wasm_execution_failure_kind(),
            Some(WasmExecutionFailureKind::DynamicSource),
            "{error}"
        );
        assert_eq!(
            error.runtime_dynamic_source_operations(),
            vec![DynamicSourceRuntimeOperation::Eval],
            "{error}"
        );
        assert!(error.parse_diagnostic().is_none(), "{error}");
        assert!(error.ir_diagnostic().is_none(), "{error}");
        assert_eq!(
            error.wasm_javascript_exception_constructor_name(),
            None,
            "{error}"
        );
    }
}
