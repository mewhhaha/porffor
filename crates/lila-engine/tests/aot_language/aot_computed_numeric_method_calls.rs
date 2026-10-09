use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, ObservedCompletion, ObservedJsValue, RealmBuilder,
    RunOptions,
};

fn assert_computed_numeric_call(source: &str) {
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
        .unwrap_or_else(|error| panic!("computed numeric call failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.output_events.is_empty());
    assert_eq!(
        outcome.completion,
        ObservedCompletion::Normal(ObservedJsValue::Boolean(true)),
        "{source}"
    );
}

#[test]
fn computed_numeric_formatters_call_the_observable_intrinsics() {
    assert_computed_numeric_call(
        r#"
var key = 'toLocaleString';
var number = (123)['toLocaleString']('en-US', {minimumFractionDigits: 2});
var bigint = (123n)[key]('en-US', {minimumFractionDigits: 2});
number === '123.00' && bigint === '123.00';
"#,
    );
}

#[test]
fn base_key_getter_arguments_and_call_preserve_order_and_primitive_receivers() {
    for (prototype, receiver) in [("Number.prototype", "7"), ("BigInt.prototype", "7n")] {
        assert_computed_numeric_call(&format!(
            r#"
var trace = '';
Object.defineProperty({prototype}, 'observed', {{
  configurable: true,
  get: function() {{
    'use strict';
    if (this !== {receiver}) throw 'getter receiver';
    trace += 'get;';
    return function(argument) {{
      'use strict';
      if (this !== {receiver} || argument !== 9) throw 'call receiver or argument';
      trace += 'call;';
      return 17;
    }};
  }}
}});
var key = {{ [Symbol.toPrimitive]: function(hint) {{
  if (hint !== 'string') throw 'key hint';
  trace += 'coerce;';
  return 'observed';
}} }};
var result = (trace += 'base;', {receiver})[(trace += 'key;', key)]((
  trace += 'arg;',
  Object.defineProperty({prototype}, 'observed', {{
    configurable: true,
    value: function() {{ throw 'callee read again after argument'; }}
  }}),
  9
));
result === 17 && trace === 'base;key;coerce;get;arg;call;';
"#
        ));
    }
}

#[test]
fn symbol_and_numeric_property_keys_preserve_their_identity() {
    for (prototype, receiver) in [("Number.prototype", "7"), ("BigInt.prototype", "7n")] {
        assert_computed_numeric_call(&format!(
            r#"
var symbol = Symbol('method');
var method = function() {{ 'use strict'; return this; }};
{prototype}[symbol] = method;
{prototype}['Symbol(method)'] = function() {{ throw 'symbol became a string'; }};
{prototype}['0'] = method;
{prototype}['9'] = method;
var conversions = 0;
var key = {{ [Symbol.toPrimitive]: function(hint) {{
  if (hint !== 'string') throw 'key hint';
  conversions++;
  return symbol;
}} }};
({receiver})[symbol]() === {receiver} &&
  ({receiver})[key]() === {receiver} && conversions === 1 &&
  ({receiver})[-0]() === {receiver} && ({receiver})[9n]() === {receiver};
"#
        ));
    }
}

#[test]
fn computed_reference_abrupts_stop_later_evaluation_and_keep_identity() {
    for (prototype, receiver) in [("Number.prototype", "7"), ("BigInt.prototype", "7n")] {
        assert_computed_numeric_call(&format!(
            r#"
var marker = {{}};
var trace = '';
function argument() {{ trace += 'arg;'; return 0; }}
function keyExpression() {{ trace += 'key;'; throw marker; }}
try {{ ({receiver})[keyExpression()](argument()); throw 'missing key throw'; }}
catch (error) {{ if (error !== marker) throw error; }}
if (trace !== 'key;') throw 'key expression did not stop arguments';
trace = '';
var key = {{ [Symbol.toPrimitive]: function() {{ trace += 'coerce;'; throw marker; }} }};
try {{ ({receiver})[key](argument()); throw 'missing coercion throw'; }}
catch (error) {{ if (error !== marker) throw error; }}
if (trace !== 'coerce;') throw 'key coercion did not stop arguments';
trace = '';
Object.defineProperty({prototype}, 'fail', {{
  configurable: true,
  get: function() {{ trace += 'get;'; throw marker; }}
}});
try {{ ({receiver})['fail'](argument()); throw 'missing getter throw'; }}
catch (error) {{ if (error !== marker) throw error; }}
if (trace !== 'get;') throw 'getter did not stop arguments';
trace = '';
Object.defineProperty({prototype}, 'fail', {{
  configurable: true,
  get: function() {{
    trace += 'get;';
    return function() {{ trace += 'call;'; }};
  }}
}});
function throwingArgument() {{ trace += 'arg;'; throw marker; }}
try {{ ({receiver})['fail'](throwingArgument()); throw 'missing argument throw'; }}
catch (error) {{ if (error !== marker) throw error; }}
if (trace !== 'get;arg;') throw 'argument did not stop call';
trace = '';
Object.defineProperty({prototype}, 'fail', {{configurable: true, value: 0}});
try {{ ({receiver})['fail'](argument()); throw 'missing callable check'; }}
catch (error) {{ if (!(error instanceof TypeError)) throw error; }}
trace === 'arg;';
"#
        ));
    }
}
