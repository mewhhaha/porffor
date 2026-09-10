use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_trace(source: &str, expected: &[&str]) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
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
        .expect("resumed assignments execute ordinary global PutValue");
    assert!(matches!(outcome.completion, ObservedCompletion::Normal(_)));
    assert_eq!(
        outcome.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).to_string()))
            .collect::<Vec<_>>()
    );
}

#[test]
fn generator_resume_creates_sloppy_globals_and_checks_strict_references() {
    assert_trace(
        r#"
function* sloppy() {
  created = yield 1;
  print('sloppy:continued');
  yield created + 1;
}
var generator = sloppy();
print(generator.next().value);
print(Object.prototype.hasOwnProperty.call(globalThis, 'created'));
print(generator.next(17).value);
print(globalThis.created);
print(generator.next().done);
function* strict() { 'use strict'; missing = yield 2; }
var strictGenerator = strict();
print(strictGenerator.next().value);
try { strictGenerator.next(18); } catch (error) { print(error instanceof ReferenceError); }
print(Object.prototype.hasOwnProperty.call(globalThis, 'missing'));
"#,
        &[
            "1",
            "false",
            "sloppy:continued",
            "18",
            "17",
            "true",
            "2",
            "true",
            "false",
        ],
    );
}

#[test]
fn resumed_global_writes_observe_current_properties_and_the_source_realm() {
    assert_trace(
        r#"
globalThis.locked = 1;
function* strict() { 'use strict'; locked = yield; }
var generator = strict();
generator.next();
Object.defineProperty(globalThis, 'locked', { writable: false });
try { generator.next(2); } catch (error) { print(error instanceof TypeError); }
print(globalThis.locked);
var realm = __lilaCreateRealm();
var factory = realm.evalScript('function* sequence() { remote = yield 3; } sequence;');
var foreign = factory();
print(foreign.next().value);
foreign.next(19);
print(realm.global.remote);
print(Object.prototype.hasOwnProperty.call(globalThis, 'remote'));
function* delegate() { delegated = yield* [4]; }
var delegatedGenerator = delegate();
print(delegatedGenerator.next().value);
delegatedGenerator.next();
print(Object.prototype.hasOwnProperty.call(globalThis, 'delegated'));
"#,
        &["true", "1", "3", "19", "false", "4", "true"],
    );
}

#[test]
fn await_and_async_generator_resumes_share_global_assignment_semantics() {
    assert_trace(
        r#"
async function write() {
  print('write:enter');
  awaited = await Promise.resolve(5);
  print(globalThis.awaited);
}
async function* sequence() {
  yielded = yield 6;
  awaitedInside = await Promise.resolve(8);
  print(globalThis.yielded + ':' + globalThis.awaitedInside);
  delegatedInside = yield* [9];
  print(Object.prototype.hasOwnProperty.call(globalThis, 'delegatedInside'));
}
async function check() {
  print('check:enter');
  await write();
  var generator = sequence();
  print((await generator.next()).value);
  print((await generator.next(7)).value);
  print((await generator.next()).done);
}
check().catch(function(error) { print('unexpected:' + error); });
void 0;
"#,
        &[
            "check:enter",
            "write:enter",
            "5",
            "6",
            "7:8",
            "9",
            "true",
            "true",
        ],
    );
}

#[test]
fn ordinary_and_async_closures_read_the_live_global_function_property() {
    assert_trace(
        r#"
function value() { return 17; }
function reader() { return value; }
async function asynchronousReader() { return value; }
print(reader() === value);
var replacement = function () { return 23; };
globalThis.value = replacement;
print(reader() === replacement);
asynchronousReader().then(function (result) { print(result === replacement); });
var strictReader = (0, eval)('"use strict"; function value() { return 31; } function read() { return value; } read;');
print(strictReader()());
print(value());
void 0;
"#,
        &["true", "true", "31", "23", "true"],
    );
}

#[test]
fn generator_constructor_compiles_a_body_with_an_unresolved_resume_target() {
    assert_trace(
        r#"
var GeneratorFunction = Object.getPrototypeOf(function*() {}).constructor;
var body = GeneratorFunction('x = yield');
print(typeof body);
try { GeneratorFunction('x = yield', ''); } catch (error) { print(error instanceof SyntaxError); }
var generator = body();
generator.next();
generator.next(23);
print(globalThis.x);
"#,
        &["function", "true", "23"],
    );
}
