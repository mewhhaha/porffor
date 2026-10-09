use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_receiver_observation(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("receiver observation failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(outcome.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        outcome.completion
    );
    assert_eq!(
        outcome.output_events,
        vec![HostOutputEvent::PrintLine("ok".to_string())],
        "{source}"
    );
}

#[test]
fn intrinsic_calls_and_property_hooks_observe_each_receiver() {
    assert_receiver_observation(
        r#"
var join = Array.prototype.join;
var left = { 0: 'left', length: 1 };
var right = ['right', 'last'];
if (join.call(left, ':') !== 'left') throw 'first receiver';
if (Reflect.apply(join, right, ['|']) !== 'right|last') throw 'second receiver';
left[0] = 'changed';
if (join.apply(left, ['/']) !== 'changed') throw 'mutated receiver';
if (String.prototype.charAt.call(1234, 2) !== '3') throw 'primitive receiver';
var size = Object.getOwnPropertyDescriptor(Map.prototype, 'size').get;
var first = new Map([[1, 'a'], [2, 'b']]);
var second = new Map([[3, 'c']]);
Object.defineProperty(first, 'count', { get: size });
Object.defineProperty(second, 'count', { get: size });
if (first.count !== 2 || second.count !== 1) throw 'intrinsic getter receiver';
var rejected = false;
try { size.call({}); } catch (error) { rejected = error instanceof TypeError; }
if (!rejected) throw 'intrinsic getter brand';
print('ok');
"#,
    );
}

#[test]
fn source_function_and_arrow_receivers_keep_activation_identity() {
    assert_receiver_observation(
        r#"
function capture() { 'use strict'; return () => this; }
var left = { name: 'left' };
var right = { name: 'right' };
var readLeft = capture.call(left);
var readRight = Reflect.apply(capture, right, []);
if (readLeft() !== left || readRight() !== right) throw 'source activation receiver';
if (readLeft.call(right) !== left || readRight.call(left) !== right) throw 'lexical receiver';
if (capture()() !== undefined) throw 'strict default receiver';
function sloppy() { return () => this; }
if (sloppy()() !== globalThis || sloppy.call(null)() !== globalThis) throw 'sloppy default receiver';
var boxed = sloppy.call(7)();
if (Number.prototype.valueOf.call(boxed) !== 7) throw 'sloppy primitive receiver';
print('ok');
"#,
    );
}

#[test]
fn constructor_receivers_remain_distinct_from_intrinsic_returns() {
    assert_receiver_observation(
        r#"
function Box(value) {
  this.value = value;
  this.read = () => this.value;
}
var first = new Box('first');
var second = new Box('second');
if (first.read() !== 'first' || second.read() !== 'second') throw 'constructed receiver';
first.value = 'changed';
if (first.read.call(second) !== 'changed') throw 'constructed lexical receiver';
var array = new Array(1, 2);
var map = new Map([[first, second]]);
if (array.length !== 2 || array[1] !== 2 || map.get(first) !== second) throw 'intrinsic constructor return';
print('ok');
"#,
    );
}

#[test]
fn empty_dynamic_intrinsics_keep_return_protocols_with_explicit_receivers() {
    assert_receiver_observation(
        r#"
var marker = {};
var ordinary = Function.call(marker);
if (ordinary() !== undefined || ordinary.length !== 0) throw 'empty ordinary function';
var Generator = (function* () {}).constructor;
var first = Generator.call(marker);
var second = Reflect.apply(Generator, null, []);
if (first === second || first.prototype === second.prototype) throw 'fresh generators';
var iterator = first();
var result = iterator.next();
if (Object.getPrototypeOf(iterator) !== first.prototype) throw 'generator prototype';
if (result.done !== true || result.value !== undefined) throw 'generator completion';
var rejected = false;
try { new first(); } catch (error) { rejected = error instanceof TypeError; }
if (!rejected) throw 'generator constructability';
print('ok');
"#,
    );
}
