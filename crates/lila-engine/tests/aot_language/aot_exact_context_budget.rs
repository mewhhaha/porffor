use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

#[test]
fn canonical_large_callback_preserves_mutation_closure_and_thrown_identity() {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let mutations = "object.count += 1;\n".repeat(250);
    let source = format!(
        "function large(object, marker) {{ {mutations} if (marker) throw marker; \
         return function () {{ return object.count; }}; }} \
         var object = {{count: 0}}, marker = {{}}, caught; \
         var callback = large(object, null); \
         try {{ large(object, marker); }} catch (error) {{ caught = error; }} \
         callback() === 500 && caught === marker;"
    );
    let mut backends = vec![ExecutionBackend::WasmAot];
    #[cfg(feature = "spec-exec-oracle")]
    backends.push(ExecutionBackend::SpecExec);
    for backend in backends {
        let result = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions::default(),
                RunOptions {
                    backend,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("large callbacks execute through their canonical body");
        assert_eq!(result.backend_used, backend);
        assert_eq!(
            result.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{backend:?}: {source}"
        );
    }
}

fn observe_true_on_available_backends(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let mut backends = vec![ExecutionBackend::WasmAot];
    #[cfg(feature = "spec-exec-oracle")]
    backends.push(ExecutionBackend::SpecExec);
    for backend in backends {
        let result = Engine::new(RealmBuilder::new().build())
            .observe_script(
                source,
                CompileOptions::default(),
                RunOptions {
                    backend,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .expect("semantic control executes");
        assert_eq!(result.backend_used, backend);
        assert_eq!(
            result.completion,
            ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
            "{backend:?}: {result:?}"
        );
    }
}

#[test]
fn shared_property_read_preserves_operand_order_conversion_abruptness_and_receiver() {
    observe_true_on_available_backends(
        r#"
var trace = '', marker = {}, caught, conversions = 0;
var key = { [Symbol.toPrimitive](hint) {
    trace += 'convert:' + hint + ';'; conversions++; return 'value';
} };
function operand() { trace += 'key;'; return key; }
try { null[operand()]; } catch (error) { caught = error; }
if (!(caught instanceof TypeError) || trace !== 'key;' || conversions !== 0) throw 'nullish precedence';
function throwingOperand() { throw marker; }
try { null[throwingOperand()]; } catch (error) { caught = error; }
if (caught !== marker) throw 'raw key abrupt identity';
var throwingKey = { [Symbol.toPrimitive]() { throw marker; } };
try { ({})[throwingKey]; } catch (error) { caught = error; }
if (caught !== marker) throw 'converted key abrupt identity';
var proxy, seen;
var target = { get value() { return this === proxy ? 17 : 0; } };
proxy = new Proxy(target, { get(object, key, receiver) {
    seen = object === target && key === 'value' && receiver === proxy;
    return Reflect.get(object, key, receiver);
} });
var value = proxy[operand()];
var symbol = Symbol('own'), object = { [symbol]: 23 };
var symbolKey = { [Symbol.toPrimitive]() { return symbol; } };
value === 17 && seen && conversions === 1 && trace === 'key;key;convert:string;' &&
object[symbolKey] === 23 && 'abc'[1] === 'b';
"#,
    );
}

#[test]
fn intrinsic_oracle_chronology_retains_user_redefinition_order_and_proto_setter_arity() {
    observe_true_on_available_backends(
        r#"
var objectKeys = Reflect.ownKeys(Object.prototype).join(',');
var functionKeys = Reflect.ownKeys(Function.prototype).filter(function(key) {
    return typeof key === 'string';
}).join(',');
if (objectKeys !== 'constructor,hasOwnProperty,__defineGetter__,__defineSetter__,__lookupGetter__,__lookupSetter__,__proto__,propertyIsEnumerable,isPrototypeOf,toString,toLocaleString,valueOf') throw objectKeys;
if (functionKeys !== 'length,name,constructor,call,apply,bind,toString,arguments,caller') throw functionKeys;
var constructorKeys = Reflect.ownKeys(Object).join(',');
if (constructorKeys !== 'length,name,prototype,groupBy,fromEntries,assign,create,getPrototypeOf,setPrototypeOf,defineProperty,getOwnPropertyDescriptor,getOwnPropertyDescriptors,getOwnPropertyNames,getOwnPropertySymbols,keys,values,entries,hasOwn,defineProperties,is,isSealed,isFrozen,seal,freeze,isExtensible,preventExtensions') throw constructorKeys;
var errorPrototypeKeys = Reflect.ownKeys(Error.prototype).join(',');
if (errorPrototypeKeys !== 'name,message,constructor,toString,stack') throw errorPrototypeKeys;
var errorKeys = Reflect.ownKeys(Error).join(',');
if (errorKeys !== 'length,name,prototype,isError') throw errorKeys;
var setter = Object.getOwnPropertyDescriptor(Object.prototype, '__proto__').set;
if (setter.length !== 1) throw 'proto setter arity';
var descriptor = Object.getOwnPropertyDescriptor(Object.prototype, 'hasOwnProperty');
delete Object.prototype.hasOwnProperty;
Object.defineProperty(Object.prototype, 'hasOwnProperty', descriptor);
var keys = Reflect.ownKeys(Object.prototype);
keys[0] === 'constructor' && keys[keys.length - 1] === 'hasOwnProperty' &&
Object.getOwnPropertyDescriptor(Object.prototype, 'hasOwnProperty').value === descriptor.value;
"#,
    );
}

#[test]
fn bulk_string_concatenation_preserves_utf16_and_json_callback_observations() {
    observe_true_on_available_backends(
        r#"
var high = String.fromCharCode(0xd800), low = String.fromCharCode(0xdc00);
var text = high + 'middle' + low;
if (text.length !== 8 || text.charCodeAt(0) !== 0xd800 || text.charCodeAt(7) !== 0xdc00 ||
    '' + text !== text || text + '' !== text) throw 'UTF-16 concatenation';
var reads = 0, replacements = 0, items = [];
for (var i = 0; i < 64; i++) {
    items.push({ get value() { reads++; return text; } });
}
var encoded = JSON.stringify(items, function(key, value) {
    if (key === 'value') replacements++;
    return value;
}, 2);
var decoded = JSON.parse(encoded);
var wide = text.repeat(2048);
var wideEncoded = JSON.stringify([wide, '', {}, []]);
if (wideEncoded !== '[' + JSON.stringify(wide) + ',"",{},[]]' ||
    JSON.parse(wideEncoded)[0] !== wide || JSON.stringify([]) !== '[]' ||
    JSON.stringify({}) !== '{}') throw 'exact used prefix and large fragment';
reads === 64 && replacements === 64 && decoded.length === 64 &&
decoded[0].value === text && decoded[63].value === text && encoded.indexOf('\\ud800middle\\udc00') >= 0;
"#,
    );
}
