use scriptw_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_reads(source: &str) {
    scriptw_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}function assert(value, message) {{ if (!value) throw new Error(message); }}\n{source}",
            if strict { "'use strict';\n" } else { "" }
        );
        let outcome = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("property plans execute compiled Wasm");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(outcome.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            outcome.completion
        );
    }
}

#[test]
fn property_slot_candidates_check_actual_order_and_later_descriptors() {
    assert_reads(
        r#"
var token = {}, pair = {x: token, y: 7}, reversed = {z: 11, a: token};
assert(pair.x === token && pair.y === 7, 'ordinary own data');
assert(reversed.a === token && reversed.z === 11, 'shape name order is not insertion order');
var reads = 0;
Object.defineProperty(pair, 'x', {get: function() { reads++; return token; }, configurable: true});
assert(pair.x === token && reads === 1, 'data to accessor remains observable');
delete pair.y; pair.y = 19;
assert(pair.y === 19 && Object.keys(pair).join(',') === 'x,y', 'deleted slot and later insertion');
var inherited = Object.create({x: token});
assert(inherited.x === token, 'inherited value is not an own-slot fact');
var proxy = new Proxy(pair, {get: function(target, key, receiver) { reads++; return Reflect.get(target, key, receiver); }});
assert(proxy.x === token && reads === 3, 'proxy and getter retain both effects');
"#,
    );
}

#[test]
fn numeric_array_reads_preserve_nonindices_holes_and_receivers() {
    assert_reads(
        r#"
var token = {}, a = [token, 7], indexes = [-0, 0, 1];
for (var i = 0; i < indexes.length; i++) assert(a[indexes[i]] === (i === 2 ? 7 : token), 'numeric index');
a[-1] = 11; a[1.5] = 13; a[NaN] = 17; a[Infinity] = 19; a[4294967295] = 23;
assert(a[-1] === 11 && a[1.5] === 13 && a[NaN] === 17 && a[Infinity] === 19 && a[4294967295] === 23, 'nonindices remain names');
a['-0'] = 29;
assert(a[-0] === token && a['-0'] === 29 && a.length === 2, 'negative zero and length');
var proto = Object.create(Array.prototype), reads = 0;
Object.defineProperty(proto, '3', {get: function() { reads++; assert(this === a, 'inherited receiver'); return token; }, configurable: true});
Object.setPrototypeOf(a, proto); a.length = 4;
assert(a[3] === token && reads === 1 && !Object.hasOwn(a, '3'), 'hole preserves prototype getter');
Object.defineProperty(a, '1', {get: function() { reads++; throw token; }, configurable: true});
try { a[1]; throw 1; } catch (error) { assert(error === token && reads === 2, 'own accessor abrupt identity'); }
"#,
    );
}

#[test]
fn property_reads_evaluate_keys_once_before_receiver_errors() {
    assert_reads(
        r#"
var token = {}, a = [token], evaluations = 0, coercions = 0;
function numeric() { evaluations++; return 0; }
assert(a[numeric()] === token && evaluations === 1, 'numeric key evaluation once');
var key = {}; key[Symbol.toPrimitive] = function(hint) { coercions++; assert(hint === 'string', 'key hint'); a[0] = 7; return '0'; };
assert(a[key] === 7 && coercions === 1, 'effectful key conversion before lookup');
try { null[numeric()]; throw 1; } catch (error) { assert(error instanceof TypeError && evaluations === 2, 'key evaluated before nullish check'); }
try { undefined[key]; throw 1; } catch (error) { assert(error instanceof TypeError && coercions === 1, 'nullish check precedes key conversion'); }
"#,
    );
}

#[test]
fn numeric_keys_observe_receiver_mutation_before_own_data_dispatch() {
    assert_reads(
        r#"
var before = {}, after = {}, array = [before], evaluations = 0, reads = 0;
function replace() { evaluations++; array[0] = after; return 0; }
assert(array[replace()] === after && evaluations === 1, 'numeric key mutation precedes data read');
var proto = Object.create(Array.prototype);
Object.defineProperty(proto, '0', {get: function() { reads++; assert(this === array, 'deleted index keeps receiver'); return before; }, configurable: true});
Object.setPrototypeOf(array, proto);
function remove() { evaluations++; delete array[0]; return 0; }
assert(array[remove()] === before && evaluations === 2 && reads === 1, 'key deletion reaches inherited getter');
function redefine() {
  evaluations++;
  Object.defineProperty(array, '0', {get: function() { reads++; assert(this === array, 'redefined getter receiver'); throw after; }, configurable: true});
  return 0;
}
try { array[redefine()]; throw 1; } catch (error) {
  assert(error === after && evaluations === 3 && reads === 2, 'key redefinition preserves thrown identity');
}
var proxyReads = 0, target = [after], proxy;
proxy = new Proxy(target, {get: function(object, key, receiver) {
  proxyReads++;
  assert(object === target && key === '0' && receiver === proxy, 'numeric proxy trap arguments');
  return Reflect.get(object, key, receiver);
}});
assert(proxy[0] === after && proxyReads === 1, 'numeric proxy read is never an array own-data shortcut');
"#,
    );
}

#[test]
fn property_candidates_preserve_last_index_and_descriptor_attribute_transitions() {
    assert_reads(
        r#"
var token = {}, edge = [];
edge[4294967294] = token;
edge[4294967295] = 17;
assert(edge.length === 4294967295 && edge[4294967294] === token && edge[4294967295] === 17, 'last array index differs from ordinary numeric name');
assert(Object.keys(edge).join(',') === '4294967294,4294967295', 'boundary keys retain order');
var sealed = [token];
Object.seal(sealed);
assert(Reflect.set(sealed, '0', 7) && sealed[0] === 7, 'sealed writable element can change');
assert(!Reflect.deleteProperty(sealed, '0') && sealed[0] === 7, 'sealed element remains present');
Object.freeze(sealed);
assert(!Reflect.set(sealed, '0', token) && sealed[0] === 7 && sealed.length === 1, 'frozen element read observes committed value');
var record = {x: token};
Object.seal(record);
assert(Reflect.set(record, 'x', 23) && record.x === 23, 'sealed ordinary slot can change');
Object.freeze(record);
assert(!Reflect.set(record, 'x', token) && record.x === 23, 'frozen ordinary slot retains its value');
"#,
    );
}
