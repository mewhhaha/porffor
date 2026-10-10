use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, RealmBuilder, RunOptions,
};

fn assert_keys(source: &str, expected_output: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    for directive in ["", "'use strict';\n"] {
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &format!("{directive}{source}"),
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("UTF-16 property keys execute through Wasm AOT");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            outcome.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true))
        );
        assert_eq!(
            outcome.output_events,
            vec![HostOutputEvent::PrintLine(expected_output.into())]
        );
    }
}

#[test]
fn literal_computed_and_class_keys_preserve_code_units_and_inferred_names() {
    assert_keys(
        r#"
function check(value, label) { if (!value) throw label; }
var lone = '\uD800', escaped = '\\uD800', marker = '\u{F0000}D800';
var object = {'\uD800': 1, '\\uD800': 2, '\u{F0000}D800': 3};
check(Reflect.ownKeys(object).length === 3, 'distinct-literal-keys');
check(Reflect.ownKeys(object)[0] === lone && Reflect.ownKeys(object)[1] === escaped && Reflect.ownKeys(object)[2] === marker, 'key-code-units');
check(object[lone] === 1 && object[escaped] === 2 && object[marker] === 3, 'computed-key-reads');
object['\uD800']++; delete object['\\uD800'];
check(object[lone] === 2 && !Object.hasOwn(object, escaped), 'computed-update-delete');
var computed = {['\uD800']: 4, ['\\uD800']: 5, ['\u{F0000}D800']: 6};
check(computed[lone] === 4 && computed[escaped] === 5 && computed[marker] === 6, 'computed-literal-keys');
var methods = {'\uD801'() {return 7;}, get '\uD802'() {return 8;}};
check(methods['\uD801']() === 7 && methods['\uD801'].name === '\uD801', 'method-key-and-name');
var getter = Object.getOwnPropertyDescriptor(methods, '\uD802').get;
check(getter.name === 'get \uD802' && methods['\uD802'] === 8, 'getter-key-and-name');
class Keys {
  '\uD800' = 9;
  '\\uD800' = 10;
  '\u{F0000}D800' = 11;
  '\uD801'() {return 12;}
  get '\uD802'() {return 13;}
}
var instance = new Keys();
check(instance[lone] === 9 && instance[escaped] === 10 && instance[marker] === 11, 'class-field-keys');
check(instance['\uD801']() === 12 && instance['\uD801'].name === '\uD801', 'class-method-key-and-name');
check(instance['\uD802'] === 13 && Object.getOwnPropertyDescriptor(Keys.prototype, '\uD802').get.name === 'get \uD802', 'class-getter-key-and-name');
var literalTemplate = `\u{F0000}D800`;
function tag(strings) { return strings[0]; }
check(literalTemplate === marker && tag`\u{F0000}D800` === marker, 'template-pool-marker');
print('utf16-keys:ok');
true;
"#,
        "utf16-keys:ok",
    );
}

#[test]
fn ordinary_and_suspended_object_patterns_keep_literal_keys_and_rest_exclusions() {
    assert_keys(
        r#"
function check(value, label) { if (!value) throw label; }
var lone = '\uD800', escaped = '\\uD800';
var object = {'\uD800': undefined, '\\uD800': 2};
var {'\uD800': direct = 3, ...rest} = object;
check(direct === 3 && rest[escaped] === 2 && !Object.hasOwn(rest, lone), 'ordinary-binding-keys');
var assigned;
({'\uD800': assigned = 4} = object);
check(assigned === 4, 'ordinary-assignment-key');
function parameter({'\uD800': value = 5}) {return value;}
check(parameter(object) === 5, 'parameter-key');
function* values() {
  var {'\uD800': value = yield 'default', ...tail} = object;
  return value === 6 && tail[escaped] === 2 && !Object.hasOwn(tail, lone);
}
var iterator = values();
check(iterator.next().value === 'default', 'suspended-default');
gc();check(iterator.next(6).value === true, 'suspended-binding-keys');
async function asyncValue() {
  var {'\uD800': value = await 7, ...tail} = object;
  check(value === 7 && tail[escaped] === 2 && !Object.hasOwn(tail, lone), 'awaited-binding-keys');
}
asyncValue().then(function() {print('utf16-patterns:ok');}, function(error) {throw error;});
true;
"#,
        "utf16-patterns:ok",
    );
}
