use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn run_boolean(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("direct eval preserves ordinary call evaluation and intrinsic identity");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn the_original_callee_is_captured_before_every_argument() {
    run_boolean(
        r#"
var original = eval;
var trace = [];
Object.defineProperty(globalThis, 'eval', {
  configurable: true,
  get() { trace.push('callee'); return original; }
});
function caller() {
  let local = 31;
  return eval('local;', trace.push('first'),
    Object.defineProperty(globalThis, 'eval', {
      configurable: true, writable: true,
      value: function(source, answer) { trace.push('replacement'); return answer; }
    }), trace.push('last'));
}
var direct = caller();
var replacement = eval('let duplicate; let duplicate;', 9, trace.push('extra'));
globalThis.eval = original;
direct === 31 && replacement === 9
  && trace.join(',') === 'callee,first,last,extra,replacement';
"#,
    );
}

#[test]
fn only_the_current_realms_original_eval_has_direct_call_authority() {
    run_boolean(
        r#"
var marker = 7;
var other = __lilaCreateRealm().global;
other.marker = 99;
function invoke(eval) {
  let marker = 31;
  return eval('marker;');
}
var direct = invoke(eval);
var bound = invoke(eval.bind(undefined));
var foreign = invoke(other.eval);
var proxied = invoke(new Proxy(eval, {}));
direct === 31 && bound === 7 && foreign === 99 && proxied === 7;
"#,
    );
}

#[test]
fn non_string_arguments_pass_through_after_all_arguments_run() {
    run_boolean(
        r#"
var calls = 0;
var value = { toString() { throw new Error('must not coerce'); } };
function invoke(argument) { return eval(argument, calls += 1); }
var empty = eval();
empty === undefined && invoke(value) === value && calls === 1
  && invoke(17) === 17 && calls === 2;
"#,
    );
}

#[test]
fn deferred_syntax_errors_follow_argument_effects_and_do_not_return_from_the_caller() {
    run_boolean(
        r#"
var trace = [];
function caller() {
  try {
    eval('let duplicate; let duplicate;', trace.push('argument'));
  } catch (error) {
    trace.push(error instanceof SyntaxError ? 'syntax' : 'wrong');
  }
  var answer = eval('40 + 2;');
  trace.push('continued');
  return answer;
}
caller() === 42 && trace.join(',') === 'argument,syntax,continued';
"#,
    );
}

#[test]
fn with_environment_references_preserve_the_replacement_receiver() {
    run_boolean(
        r#"
var scope = { answer: 19, eval: function() { return this.answer; } };
var replacement;
with (scope) { replacement = eval('let duplicate; let duplicate;'); }
scope.eval = eval;
var direct;
with (scope) { direct = eval('answer;'); }
replacement === 19 && direct === 19;
"#,
    );
}

#[test]
fn comma_eval_candidates_preserve_callee_identity_and_argument_effects() {
    run_boolean(
        r#"
eval('');
var trace = '';
var first = (trace += 'l', eval)('23', (trace += 'r', eval = function(source) {
  trace += 'c';
  return 'custom:' + source;
}));
var second = (trace += 's', eval)('23');
first === 23 && second === 'custom:23' && trace === 'lrsc';
"#,
    );
}

#[test]
fn comma_eval_in_a_named_function_environment_stays_indirect() {
    run_boolean(
        r#"
var value = 7;
function run() {
  var value = 11;
  eval('');
  return (0, eval)('value');
}
run() === 7;
"#,
    );
}
