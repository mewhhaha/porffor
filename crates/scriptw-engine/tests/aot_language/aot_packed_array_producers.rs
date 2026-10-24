use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_producers(source: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .unwrap_or_else(|error| panic!("Array producer execution failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn fresh_number_producers_keep_ieee_values_data_attributes_and_mixed_references() {
    let compact = (0..128)
        .map(|value| value.to_string())
        .collect::<Vec<_>>()
        .join(",");
    let source = r#"
function fresh(value) { return [value, -0, NaN, Infinity]; }
function defaultData(array, key) {
  var descriptor = Object.getOwnPropertyDescriptor(array, key);
  return descriptor.writable && descriptor.enumerable && descriptor.configurable && 'value' in descriptor;
}
var fixed = fresh(7), compact = [COMPACT_VALUES];
var parsed = JSON.parse('[-0,7,1e400]');
var own = Object.values({first: -0, last: 17});
var entries = Object.entries({first: 7});
var match = /a(b)?/d.exec('a');
var proxied = new Proxy(function(){}, {apply(target, receiver, args) { return args; }});
var fromList = proxied(-0, NaN, Infinity);
var marker = {}, symbol = Symbol('element');
fixed[2] = marker; compact[127] = symbol;
gc();
fixed[0] === 7 && Object.is(fixed[1], -0) && fixed[2] === marker && fixed[3] === Infinity &&
compact.length === 128 && compact[0] === 0 && compact[126] === 126 && compact[127] === symbol &&
Object.is(parsed[0], -0) && parsed[1] === 7 && parsed[2] === Infinity &&
Object.is(own[0], -0) && own[1] === 17 && entries[0][0] === 'first' && entries[0][1] === 7 &&
match[0] === 'a' && match[1] === undefined && match.indices[0][0] === 0 && match.indices[0][1] === 1 &&
match.indices[1] === undefined && Object.is(fromList[0], -0) && Number.isNaN(fromList[1]) && fromList[2] === Infinity &&
defaultData(fixed, '0') && defaultData(compact, '126') && defaultData(parsed, '0') &&
defaultData(own, '0') && defaultData(entries, '0') && defaultData(match.indices[0], '0') && defaultData(fromList, '0');
"#.replace("COMPACT_VALUES", &compact);
    assert_producers(&source);
}

#[test]
fn private_fresh_publication_preserves_generic_push_species_and_length_checks() {
    assert_producers(
        r#"
var setterCalls = 0, pushed;
Object.defineProperty(Array.prototype, '0', {configurable: true, set(value) { setterCalls++; pushed = value; }});
function fresh(value) { return [value]; }
var fixed = fresh(3), parsed = JSON.parse('[5]'), own = Object.values({x: 7});
var match = /a/d.exec('a');
var ordinary = [];
ordinary.push(11);
delete Array.prototype[0];
var frozen = [13], rejected = false;
Object.preventExtensions(frozen);
try { frozen.push(17); } catch (error) { rejected = error instanceof TypeError; }
var readonly = [19]; Object.defineProperty(readonly, 'length', {writable: false});
var readonlyRejected = false;
try { readonly.push(23); } catch (error) { readonlyRejected = error instanceof TypeError; }
var created = {}, speciesCalls = 0;
var source = [29];
source.constructor = { [Symbol.species]: function(length) { speciesCalls++; return created; } };
var mapped = source.map(value => value + 1);
setterCalls === 1 && pushed === 11 && !Object.hasOwn(ordinary, '0') && ordinary.length === 1 &&
fixed[0] === 3 && parsed[0] === 5 && own[0] === 7 && match[0] === 'a' && match.indices[0][1] === 1 &&
rejected && frozen.length === 1 && frozen[0] === 13 && readonlyRejected && readonly.length === 1 &&
speciesCalls === 1 && mapped === created && created[0] === 30;
"#,
    );
}

#[test]
fn array_like_argument_snapshots_keep_holes_getter_mutations_proxy_gets_and_gc_roots() {
    assert_producers(
        r#"
function collect(a, b, c, d) { gc(); return [a, b, c, d]; }
var marker = {}, values = [-0, 1, 2, NaN], gets = 0;
Object.defineProperty(values, '1', {configurable: true, get() { gets++; values[2] = marker; gc(); return 7; }});
var captured = Reflect.apply(collect, null, values);
var inheritedGets = 0, prototype = Object.create(Array.prototype), holes = [3,,5];
Object.defineProperty(prototype, '1', {get() { inheritedGets++; return marker; }});
Object.setPrototypeOf(holes, prototype);
var inherited = collect.apply(null, holes);
var log = '';
var proxy = new Proxy([11,13], {get(target, key, receiver) { log += key + '|'; return Reflect.get(target, key, receiver); }});
var trapped = Reflect.apply(collect, null, proxy);
var spread = collect(...[-0, NaN, Infinity, marker]);
Object.is(captured[0], -0) && captured[1] === 7 && captured[2] === marker && Number.isNaN(captured[3]) && gets === 1 &&
inherited[0] === 3 && inherited[1] === marker && inherited[2] === 5 && inherited[3] === undefined && inheritedGets === 1 &&
trapped[0] === 11 && trapped[1] === 13 && log === 'length|0|1|' &&
Object.is(spread[0], -0) && Number.isNaN(spread[1]) && spread[2] === Infinity && spread[3] === marker;
"#,
    );
}
