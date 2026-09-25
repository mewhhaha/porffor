//! ECMA-402 normative-optional constructor mode (4.3 Note 1) on Wasm-AOT:
//! ChainNumberFormat/ChainDateTimeFormat, UnwrapNumberFormat/
//! UnwrapDateTimeFormat and the per-Realm `%Intl%.[[FallbackSymbol]]`.

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_legacy_constructed(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
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
        .expect("Intl legacy constructor regression must execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion,
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())],
        "{source}",
    );
}

#[test]
fn plain_calls_chain_the_formatter_onto_an_inheriting_this() {
    assert_legacy_constructed(
        r#"
for (var C of [Intl.NumberFormat, Intl.DateTimeFormat]) {
  var target = Object.create(C.prototype);
  if (C.call(target) !== target) throw 'chained this';
  var symbols = Object.getOwnPropertySymbols(target);
  if (symbols.length !== 1 || symbols[0].description !== 'IntlLegacyConstructedSymbol') throw 'symbol';
  var descriptor = Object.getOwnPropertyDescriptor(target, symbols[0]);
  if (descriptor.writable || descriptor.enumerable || descriptor.configurable ||
      !(descriptor.value instanceof C) || descriptor.value === target) throw 'descriptor';
  var other = Object.create(C.prototype);
  C.call(other);
  if (Object.getOwnPropertySymbols(other)[0] !== symbols[0]) throw 'one Symbol per Realm';
  var plain = {};
  var created = C.call(plain);
  if (created === plain || Object.getOwnPropertySymbols(plain).length !== 0 ||
      !(created instanceof C)) throw 'unrelated this';
  if (Object.getOwnPropertySymbols(new C()).length !== 0) throw 'construct does not chain';
  // A member call's `this` is the namespace object, which does not inherit
  // from the prototype: the new formatter is returned unchanged.
  var member = C === Intl.NumberFormat ? Intl.NumberFormat('en-US') : Intl.DateTimeFormat('en-US');
  if (!(member instanceof C) || member.resolvedOptions().locale !== 'en-US') throw 'member call';
  var sealed = Object.freeze(Object.create(C.prototype));
  try { C.call(sealed); throw 'frozen'; } catch (error) {
    if (!(error instanceof TypeError)) throw error;
  }
}
print(true);
"#,
    );
}

#[test]
fn format_getter_and_resolved_options_unwrap_through_an_observable_get() {
    assert_legacy_constructed(
        r#"
for (var C of [Intl.NumberFormat, Intl.DateTimeFormat]) {
  var chained = C.call(Object.create(C.prototype));
  var reads = [];
  var proxy = new Proxy(chained, { get: function (target, key) { reads.push(key); return target[key]; } });
  var locale = C.prototype.resolvedOptions.call(proxy).locale;
  if (locale !== new C().resolvedOptions().locale) throw 'resolved locale';
  if (reads.length !== 1 || typeof reads[0] !== 'symbol' ||
      reads[0].description !== 'IntlLegacyConstructedSymbol') throw 'observable Get';
  var format = Object.getOwnPropertyDescriptor(C.prototype, 'format').get.call(chained);
  if (typeof format !== 'function') throw 'bound format';
  if (format !== Object.getOwnPropertyDescriptor(C.prototype, 'format').get.call(chained)) throw 'cached format';
  try { C.prototype.formatToParts.call(chained, 0); throw 'formatToParts unwrapped'; } catch (error) {
    if (!(error instanceof TypeError)) throw error;
  }
  try { C.prototype.resolvedOptions.call(1); throw 'primitive'; } catch (error) {
    if (!(error instanceof TypeError)) throw error;
  }
}
if (Object.getOwnPropertyDescriptor(Intl.NumberFormat.prototype, 'format').get
      .call(Intl.NumberFormat.call(Object.create(Intl.NumberFormat.prototype)))(1234.5) !== '1,234.5') throw 'format value';
print(true);
"#,
    );
}

#[test]
fn every_realm_owns_a_distinct_fallback_symbol() {
    assert_legacy_constructed(
        r#"
function fallback(C) {
  var target = Object.create(C.prototype);
  C.call(target);
  return Object.getOwnPropertySymbols(target)[0];
}
var foreign = __lilaCreateRealm().global;
var local = fallback(Intl.DateTimeFormat);
var remote = fallback(foreign.Intl.DateTimeFormat);
if (typeof remote !== 'symbol' || remote.description !== 'IntlLegacyConstructedSymbol' ||
    remote === local || fallback(foreign.Intl.NumberFormat) !== remote) throw 'per-Realm Symbol';
var description = Object.getOwnPropertyDescriptor(foreign.Symbol.prototype, 'description');
if (!description || typeof description.get !== 'function' || description.set !== undefined ||
    description.enumerable || !description.configurable ||
    description.get === Object.getOwnPropertyDescriptor(Symbol.prototype, 'description').get ||
    description.get.call(remote) !== 'IntlLegacyConstructedSymbol') throw 'foreign Symbol.prototype.description';
var crossTarget = Object.create(Intl.NumberFormat.prototype);
if (foreign.Intl.NumberFormat.call(crossTarget) === crossTarget ||
    Object.getOwnPropertySymbols(crossTarget).length !== 0) throw 'foreign prototype chain';
print(true);
"#,
    );
}
