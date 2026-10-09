use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_numbering_systems(source: &str, policy: HostSurfacePolicy) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    for strict in [false, true] {
        let source = format!("{}{source}", if strict { "'use strict';\n" } else { "" });
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: policy,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(60_000),
                    ..RunOptions::default()
                },
            )
            .expect("numbering systems must compile and execute through Wasm AOT");
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
        assert_eq!(
            observed.output_events,
            [HostOutputEvent::PrintLine("ok".into())],
            "{source}"
        );
    }
}

#[test]
fn locale_numbering_systems_uses_number_locale_prefix_defaults_and_latn_fallback() {
    assert_numbering_systems(
        r#"
var cases = [
  ['ar', 'latn'], ['ar-EG', 'arab'], ['bn-BD-fonipa', 'beng'],
  ['ar-u-ca-islamic', 'latn'], ['ar-u-rg-egzzzz', 'latn'],
  ['ar-u-sd-egc', 'latn'], ['ar-EG-u-rg-uszzzz', 'arab'],
  ['ar-x-nu-arab', 'latn'], ['qaa', 'latn']
];
for (var row of cases) {
  var locale = new Intl.Locale(row[0]), tag = locale.toString();
  var result = locale.getNumberingSystems();
  if (result.length !== 1 || !result.hasOwnProperty('0') || result[0] !== row[1]) throw row[0];
  if (locale.toString() !== tag) throw 'receiver mutation';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_numbering_systems_preserves_open_explicit_values_and_option_precedence() {
    assert_numbering_systems(
        r#"
for (var row of [['arab', 'arab'], ['foobar', 'foobar'], ['latn-abc', 'latn-abc'], ['', ''], ['true', ''], ['true-abc', 'true-abc']]) {
  var tag = 'ar-u-nu' + (row[0] ? '-' + row[0] : '');
  var locale = new Intl.Locale(tag), result = locale.getNumberingSystems();
  if (result.length !== 1 || !result.hasOwnProperty('0') || result[0] !== row[1]) throw 'open explicit singleton';
  if (locale.numberingSystem !== result[0]) throw 'getter consistency';
  if (new Intl.Locale(tag, {numberingSystem: 'beng'}).getNumberingSystems()[0] !== 'beng') throw 'option precedence';
}
for (var value of ['foobar', 'latn-abc', 'true-abc']) {
  var locale = new Intl.Locale('ar-EG-u-nu-arab', {numberingSystem: value});
  if (locale.getNumberingSystems()[0] !== value || locale.numberingSystem !== value) throw 'open option';
}
var caught = false;
try { new Intl.Locale('ar', {numberingSystem: ''}); } catch (e) { caught = e instanceof RangeError; }
if (!caught) throw 'empty option validation';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_numbering_systems_has_standard_metadata_and_fresh_dense_mutable_arrays() {
    assert_numbering_systems(
        r#"
var method = Intl.Locale.prototype.getNumberingSystems;
if (method.name !== 'getNumberingSystems' || method.length !== 0 || method.hasOwnProperty('prototype')) throw 'metadata';
var d = Object.getOwnPropertyDescriptor(Intl.Locale.prototype, 'getNumberingSystems');
if (!d.writable || d.enumerable || !d.configurable || d.value !== method) throw 'method descriptor';
var caught = false;
try { new method(); } catch (e) { caught = e instanceof TypeError; }
if (!caught) throw 'constructability';
for (var row of [['ar-EG', 'arab'], ['ar-u-nu-foobar', 'foobar'], ['ar-u-nu', '']]) {
  var locale = new Intl.Locale(row[0]), a = locale.getNumberingSystems(), b = locale.getNumberingSystems();
  if (a === b || !Array.isArray(a) || Object.getPrototypeOf(a) !== Array.prototype) throw 'fresh array';
  if (Reflect.ownKeys(a).join(',') !== '0,length' || a[0] !== row[1]) throw 'dense own keys';
  var entry = Object.getOwnPropertyDescriptor(a, '0');
  if (!entry.writable || !entry.enumerable || !entry.configurable || !('value' in entry)) throw 'entry attributes';
  var length = Object.getOwnPropertyDescriptor(a, 'length');
  if (!length.writable || length.enumerable || length.configurable || length.value !== 1) throw 'length attributes';
  a[0] = 'changed'; a.length = 0;
  if (b[0] !== row[1] || locale.getNumberingSystems()[0] !== row[1]) throw 'result alias';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_numbering_systems_checks_brand_without_observing_receiver_or_arguments() {
    assert_numbering_systems(
        r#"
var method = Intl.Locale.prototype.getNumberingSystems;
for (var value of [undefined, null, true, 1, 'ar', Symbol(), {}, Intl.Locale.prototype,
                   Object.create(new Intl.Locale('ar')), new Proxy(new Intl.Locale('ar'), {})]) {
  var caught = false;
  try { method.call(value); } catch (e) { caught = e instanceof TypeError; }
  if (!caught) throw 'receiver brand';
}
for (var row of [['ar-EG', 'arab'], ['ar-u-nu-foobar', 'foobar'], ['ar-u-nu', '']]) {
  var locale = new Intl.Locale(row[0]);
  for (var key of ['toString', 'numberingSystem', 'language', 'region', Symbol.toPrimitive]) {
    Object.defineProperty(locale, key, {get() { throw 'receiver read'; }});
  }
  Object.setPrototypeOf(locale, null);
  var result = method.call(locale, {toString() { throw 'argument coercion'; }});
  if (result.length !== 1 || result[0] !== row[1]) throw 'slot';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_numbering_systems_allocates_arrays_and_errors_in_the_method_defining_realm() {
    assert_numbering_systems(
        r#"
var foreign = __lilaCreateRealm().global;
var method = foreign.Intl.Locale.prototype.getNumberingSystems;
var remoteArrayPrototype = foreign.Array.prototype;
var remote = new foreign.Intl.Locale('ar-u-nu-foobar'), local = new Intl.Locale('ar-EG');
var localEmpty = new Intl.Locale('ar-u-nu');
foreign.Intl.Locale = function () { throw 'public Locale'; };
foreign.Array = function () { throw 'public Array'; };
var a = method.call(local), b = Intl.Locale.prototype.getNumberingSystems.call(remote);
if (Object.getPrototypeOf(a) !== remoteArrayPrototype || a.length !== 1 || a[0] !== 'arab') throw 'remote Array Realm';
if (Object.getPrototypeOf(b) !== Array.prototype || b.length !== 1 || b[0] !== 'foobar') throw 'local Array Realm';
var remoteEmpty = method.call(localEmpty);
if (Object.getPrototypeOf(remoteEmpty) !== remoteArrayPrototype || remoteEmpty.length !== 1 || remoteEmpty[0] !== '') throw 'remote empty Array Realm';
var caught = false;
try { method.call({}); } catch (e) { caught = e instanceof foreign.TypeError && !(e instanceof TypeError); }
if (!caught) throw 'error Realm';
print('ok');
"#,
        HostSurfacePolicy::Test262,
    );
}
