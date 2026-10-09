use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

struct Modules(PathBuf);

impl Drop for Modules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_fields(source: &str, dependencies: &[(&str, &str)], expected: &[&str]) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = Modules(std::env::temp_dir().join(format!(
        "lila-public-class-fields-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&fixture.0).expect("create module fixture");
    for (name, contents) in dependencies {
        std::fs::write(fixture.0.join(name), contents).expect("write module dependency");
    }
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let compile = CompileOptions {
        filename: Some(fixture.0.join("entry.js").to_str().unwrap().into()),
        module_root: Some(fixture.0.to_str().unwrap().into()),
        ..CompileOptions::default()
    };
    let run = RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(30_000),
        ..RunOptions::default()
    };
    let outcome = if dependencies.is_empty() {
        engine.observe_script(source, compile, run)
    } else {
        engine.observe_module(source, compile, run)
    }
    .expect("public class fields compile and execute through Wasm");
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
            .collect::<Vec<_>>()
    );
}

#[test]
fn returned_proxy_observes_public_field_keys_values_and_attributes() {
    assert_fields(
        r#"
const symbol = Symbol('field');
const receiver = new Proxy({}, {
  defineProperty(target, key, descriptor) {
    print((key === symbol ? 'symbol' : key) + ':' + descriptor.value + ':' +
      descriptor.writable + ':' + descriptor.enumerable + ':' + descriptor.configurable);
    return true;
  },
  set() { throw 'field used Set'; }
});
class Base { constructor() { return receiver; } }
class Derived extends Base { field = 3; [symbol] = 7; }
print(new Derived() === receiver);
"#,
        &[],
        &["field:3:true:true:true", "symbol:7:true:true:true", "true"],
    );
}

#[test]
fn failed_field_definition_propagates_before_later_initializers() {
    assert_fields(
        r#"
const marker = {};
const receiver = new Proxy({}, { defineProperty() { throw marker; } });
class Base { constructor() { return receiver; } }
class Derived extends Base {
  first = (print('first initializer'), 1);
  second = print('unexpected second initializer');
}
try { new Derived(); } catch (error) { print(error === marker); }
const frozen = Object.preventExtensions({});
class FrozenBase { constructor() { return frozen; } }
class FrozenDerived extends FrozenBase { field = 1; }
try { new FrozenDerived(); } catch (error) { print(error instanceof TypeError); }
"#,
        &[],
        &["first initializer", "true", "true"],
    );
}

#[test]
fn returned_arrays_and_typed_arrays_observe_index_definitions() {
    assert_fields(
        r#"
const array = [];
class ArrayBase { constructor() { return array; } }
class ArrayDerived extends ArrayBase { 2 = 7; }
new ArrayDerived();
print(array.length + ':' + array[2]);
const typed = new Uint8Array(1);
class TypedBase { constructor() { return typed; } }
class TypedDerived extends TypedBase { 0 = 259; }
new TypedDerived();
print(typed[0]);
"#,
        &[],
        &["3:7", "3"],
    );
}

#[test]
fn ordinary_and_static_fields_use_internal_definition_and_symbol_identity() {
    assert_fields(
        r#"
const symbol = Symbol('field');
const original = Object.defineProperty;
Object.defineProperty = () => { throw 'observable Object.defineProperty lookup'; };
class Base { set field(value) { throw 'inherited setter'; } }
class Derived extends Base { field = 3; [symbol] = 5; static [symbol] = 7; }
const receiver = new Derived();
Object.defineProperty = original;
const descriptor = Object.getOwnPropertyDescriptor(receiver, 'field');
print(receiver.field + ':' + receiver[symbol] + ':' + Derived[symbol]);
print(descriptor.writable && descriptor.enumerable && descriptor.configurable);
"#,
        &[],
        &["3:5:7", "true"],
    );
}

#[test]
fn namespace_field_definitions_evaluate_before_rejecting_exported_and_absent_keys() {
    for key in ["exported", "absent"] {
        assert_fields(
            &format!(
                r#"
import './setup.js';
import defer * as namespace from './dependency.js';
class Base {{ constructor() {{ return namespace; }} }}
class Derived extends Base {{ [{key:?}] = 10; }}
print(globalThis.evaluations);
try {{ new Derived(); }} catch (error) {{ print(error instanceof TypeError); }}
print(globalThis.evaluations);
print(namespace.exported);
"#
            ),
            &[
                ("setup.js", "globalThis.evaluations = 0;"),
                (
                    "dependency.js",
                    "globalThis.evaluations++; export const exported = 3;",
                ),
            ],
            &["0", "true", "1", "3"],
        );
    }
}
