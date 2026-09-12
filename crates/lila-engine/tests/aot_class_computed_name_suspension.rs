use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

enum Goal {
    Script,
    Module,
}

fn assert_trace(source: &str, goal: Goal, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let run = RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(30_000),
        ..RunOptions::default()
    };
    let outcome = match goal {
        Goal::Script => engine.observe_script(source, CompileOptions::default(), run),
        Goal::Module => engine.observe_module(source, CompileOptions::default(), run),
    }
    .expect("class suspension compiles and executes through Wasm");
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>(),
        "{source}"
    );
}

#[test]
fn yielded_keys_are_coerced_once_before_the_next_key_and_static_initialization() {
    assert_trace(
        r#"
function key(name) { return { [Symbol.toPrimitive](hint) { print(hint + ':' + name); return name; } }; }
function* classes() {
  let C = class {
    [yield 'method']() { return this.field; }
    [yield 'field'] = (print('instance'), 7);
    static [yield 'static'] = (print('static-init'), 9);
  };
  return C;
}
let iterator = classes();
print(iterator.next().value);
print(iterator.next(key('method')).value);
print(iterator.next(key('field')).value);
let C = iterator.next(key('own')).value;
let instance = new C();
print(instance.method() + ':' + C.own);
"#,
        Goal::Script,
        &[
            "method",
            "string:method",
            "field",
            "string:field",
            "static",
            "string:own",
            "static-init",
            "instance",
            "7:9",
        ],
    );
}

#[test]
fn accessors_and_class_declarations_keep_interleaved_activation_identity() {
    assert_trace(
        r#"
function* classes(label) {
  class Named {
    get [yield label]() { return label; }
    static set [yield 'setter'](value) { print(label + ':' + value); }
  }
  return Named;
}
let left = classes('left'), right = classes('right');
print(left.next().value); print(right.next().value);
print(right.next('read').value); print(left.next('read').value);
let R = right.next('write').value, L = left.next('write').value;
print(new L().read); print(new R().read);
L.write = 1; R.write = 2;
print(L !== R && L.prototype !== R.prototype);
"#,
        Goal::Script,
        &[
            "left", "right", "setter", "setter", "left", "right", "left:1", "right:2", "true",
        ],
    );
}

#[test]
fn heritage_get_precedes_keys_and_preserves_a_null_parent() {
    assert_trace(
        r#"
function Base() {}
let parent = new Proxy(Base, { get(target, key) { if (key === 'prototype') { print('prototype'); return null; } return target[key]; } });
function* classes() {
  let C = class extends (yield 'base') { [yield 'key']() { return 5; } };
  return C;
}
let iterator = classes();
print(iterator.next().value);
print(iterator.next(parent).value);
let C = iterator.next('method').value;
print(Object.getPrototypeOf(C.prototype) === null);
print(Object.getPrototypeOf(C) === parent);
print(new C().method());
"#,
        Goal::Script,
        &["base", "prototype", "key", "true", "true", "5"],
    );
}

#[test]
fn invalid_heritage_prototype_throws_before_any_key_evaluation() {
    assert_trace(
        r#"
function Base() {}
let parent = new Proxy(Base, { get(target, key) { if (key === 'prototype') { print('prototype'); return 1; } return target[key]; } });
function* classes() { let C = class extends parent { [yield 'forbidden']() {} }; return C; }
let iterator = classes();
try { iterator.next(); print('missed'); } catch (error) { print(error instanceof TypeError); }
print(iterator.next().done);
"#,
        Goal::Script,
        &["prototype", "true", "true"],
    );
}

#[test]
fn name_tdz_and_private_environment_survive_key_suspension() {
    assert_trace(
        r#"
function* classes() {
  let C = class Named {
    #value = 11;
    [yield (() => Named)]() {}
    [yield (value => value.#value)]() {}
  };
  return C;
}
let iterator = classes();
let readName = iterator.next().value;
try { readName(); print('missed'); } catch (error) { print(error instanceof ReferenceError); }
let readPrivate = iterator.next('first').value;
let C = iterator.next('second').value;
print(readName() === C);
print(readPrivate(new C()));
"#,
        Goal::Script,
        &["true", "true", "11"],
    );
}

#[test]
fn external_throw_and_return_skip_pending_definitions_and_leave_the_class_environment() {
    assert_trace(
        r#"
let Named = 'outer', effects = 0, marker = {};
function* classes() {
  try {
    let C = class Named { [yield 'key']() {} static value = ++effects; };
    return C;
  } finally { print(Named); }
}
let throwing = classes(); print(throwing.next().value);
try { throwing.throw(marker); print('missed'); } catch (error) { print(error === marker); }
let returning = classes(); print(returning.next().value);
let result = returning.return(19); print(result.value + ':' + result.done);
print(effects);
"#,
        Goal::Script,
        &["key", "outer", "true", "key", "outer", "19:true", "0"],
    );
}

#[test]
fn module_await_keys_cover_methods_fields_accessors_and_later_computed_calls() {
    assert_trace(
        r#"
let C = class {
  [await 'method']() { return this.field; }
  [await 'field'] = 7;
  get [await 'read']() { return this.field; }
  set [await 'write'](value) { this.field = value; }
  static [await 'own'] = 9;
};
let instance = new C();
print(instance[await 'method']());
instance[await 'write'] = 12;
print(instance[String(await 'read')] + ':' + C[await 'own']);
"#,
        Goal::Module,
        &["7", "12:9"],
    );
}

#[test]
fn rejected_await_and_abrupt_key_coercion_skip_later_keys_and_static_initializers() {
    assert_trace(
        r#"
let marker = {}, effects = 0;
try { let C = class { [await Promise.reject(marker)]() {} static value = ++effects; }; }
catch (error) { print(error === marker); }
let badKey = { [Symbol.toPrimitive]() { throw marker; } };
try { let C = class { [await badKey]() {} [await (++effects)]() {} static value = ++effects; }; }
catch (error) { print(error === marker); }
print(effects);
"#,
        Goal::Module,
        &["true", "true", "0"],
    );
}

#[test]
fn heritage_yield_invalidates_captured_facts_before_later_key_folding() {
    assert_trace(
        r#"
function* classes() {
  let key = 'before';
  let change = () => { key = 'after'; };
  let C = class extends (yield change) { [key.toString()]() { return 13; } };
  return C;
}
let iterator = classes();
iterator.next().value();
let C = iterator.next(function Base() {}).value;
print(Object.prototype.hasOwnProperty.call(C.prototype, 'after'));
print(Object.prototype.hasOwnProperty.call(C.prototype, 'before'));
"#,
        Goal::Script,
        &["true", "false"],
    );
}

#[test]
fn nested_named_classes_restore_each_pending_class_environment() {
    assert_trace(
        r#"
function* classes() {
  let C = class Outer {
    [yield (class Inner { [yield 'inner']() { return 3; } })]() { return 7; }
  };
  return C;
}
let iterator = classes();
print(iterator.next().value);
let Inner = iterator.next('inside').value;
print(new Inner().inside());
let Outer = iterator.next('outside').value;
print(new Outer().outside());
print(Inner.name + ':' + Outer.name);
"#,
        Goal::Script,
        &["inner", "3", "7", "Inner:Outer"],
    );
}

#[test]
fn heritage_prototype_get_invalidates_captured_key_facts_before_folding() {
    assert_trace(
        r#"
function* classes() {
  let key = 'before';
  function Base() {}
  let parent = new Proxy(Base, { get(target, name) {
    if (name === 'prototype') { key = 'after'; return target.prototype; }
    return target[name];
  } });
  let C = class extends parent { [key.toString()]() {} [yield 'pause']() {} };
  return C;
}
let iterator = classes(); print(iterator.next().value);
let C = iterator.next('other').value;
print(Object.prototype.hasOwnProperty.call(C.prototype, 'after'));
print(Object.prototype.hasOwnProperty.call(C.prototype, 'before'));
"#,
        Goal::Script,
        &["pause", "true", "false"],
    );
}
