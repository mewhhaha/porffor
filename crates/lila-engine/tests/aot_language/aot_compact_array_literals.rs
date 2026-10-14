use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};
fn run(source: &str) {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
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
        .unwrap();
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}
#[test]
fn compact_nested_literals_are_fresh_mutable_real_arrays_with_ordinary_own_descriptors() {
    let pairs = (0..128)
        .map(|index| format!("['Ā\\0𝄞',{index},,null,'\\uD800']"))
        .collect::<Vec<_>>()
        .join(",");
    run(&format!(
        r#"
function factory() {{ return [{pairs}]; }}
var first = factory(), second = factory(), originalPrototype = Array.prototype;
var descriptor = Object.getOwnPropertyDescriptor(first[0], '0');
var fresh = first !== second && first[0] !== second[0] && first[0] !== first[1];
var shape = Array.isArray(first) && Array.isArray(first[0]) && Object.getPrototypeOf(first) === originalPrototype && Object.getPrototypeOf(first[0]) === originalPrototype;
var holes = first[0].length === 5 && !(2 in first[0]) && (3 in first[0]) && first[0][3] === null && first[0][4].charCodeAt(0) === 0xD800;
var flags = descriptor.writable && descriptor.enumerable && descriptor.configurable;
Object.defineProperty(originalPrototype, '2', {{get() {{return 'hole inherited';}}, configurable:true}});
var inheritedHole = second[0][2] === 'hole inherited' && Object.getOwnPropertyDescriptor(second[0], '2') === undefined;
delete originalPrototype[2];
first[0][0] = 'changed'; first[0].push(9); delete first[0][1];
Object.defineProperty(first[1], '1', {{value:77, writable:false, enumerable:false, configurable:true}});
globalThis.Array = function () {{ throw 'public Array constructor'; }};
var third = factory();
fresh && shape && holes && flags && inheritedHole && second[0][0] === 'Ā\0𝄞' && second[0][1] === 0 && second[0].length === 5 &&
  !(1 in first[0]) && first[1][1] === 77 && second[1][1] === 1 && third[0][0] === 'Ā\0𝄞' && Object.getPrototypeOf(third) === originalPrototype;
"#
    ));
}
#[test]
fn compact_literal_publication_bypasses_inherited_setters_and_effectful_boundaries_stay_ordered() {
    let numbers = (0..128)
        .map(|index| index.to_string())
        .collect::<Vec<_>>()
        .join(",");
    run(&format!(
        r#"
var calls = 0;
Object.defineProperty(Array.prototype, '0', {{get() {{return 'inherited';}}, set(value) {{calls++;}}, configurable:true}});
function pure() {{ return [{numbers}]; }}
var array = pure(), descriptor = Object.getOwnPropertyDescriptor(array, '0');
var own = array[0] === 0 && descriptor.value === 0 && descriptor.writable && descriptor.enumerable && descriptor.configurable && calls === 0;
delete Array.prototype[0];
var order = [], marker = {{}}, caught;
function effect(label) {{ order.push(label); return label; }}
var mixed = [[1, 2], effect('middle'), [3, 4]];
try {{ [effect('before'), {{[Symbol.toPrimitive]() {{order.push('coerce'); throw marker;}}}} + 1, effect('after')]; }} catch(error) {{caught = error;}}
function bigintFactory() {{ return [{numbers}, 1n, -1n]; }}
var big = bigintFactory(), again = bigintFactory();
var numeric = [0, -0, NaN, Infinity, -Infinity];
own && order.join(',') === 'middle,before,coerce' && caught === marker && mixed[0] !== mixed[2] && mixed[1] === 'middle' &&
  big !== again && big[128] === 1n && big[129] === -1n && Object.is(numeric[1], -0) && numeric[2] !== numeric[2];
"#
    ));
}

#[test]
fn compact_nested_literals_use_the_defining_source_realm_array_prototype() {
    let pairs = (0..128)
        .map(|index| format!("['realm',{index}]"))
        .collect::<Vec<_>>()
        .join(",");
    let factory = format!("function factory(){{return [{pairs}];}} factory;");
    // This string is ASCII source; Rust's quoted Debug spelling is also a JS
    // string literal and does not introduce Unicode escape-policy ambiguity.
    run(&format!(
        r#"
// Raw Engine scripts expose the Test262 host primitive without its JS harness prelude.
var realm = __lilaCreateRealm();
var foreignFactory = realm.evalScript({factory:?});
var foreignPrototype = realm.evalScript('Array.prototype');
var array = foreignFactory(), again = foreignFactory();
Object.getPrototypeOf(array) === foreignPrototype && Object.getPrototypeOf(array[0]) === foreignPrototype &&
  foreignPrototype !== Array.prototype && array !== again && array[0] !== again[0] && array[127][1] === 127 && array[0][0] === 'realm';
"#
    ));
}
