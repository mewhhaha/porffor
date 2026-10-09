use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_hour_cycles(source: &str, policy: HostSurfacePolicy) {
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
            .expect("hour cycles must compile and execute through Wasm AOT");
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
fn locale_hour_cycles_preserves_pinned_regional_and_language_preferences() {
    assert_hour_cycles(
        r#"
var cases = [
  ['en-US', 'h12,h23'], ['en-GB', 'h23,h12'], ['en-JP', 'h23,h11,h12'],
  ['en-CD', 'h12,h23'], ['en-CA', 'h12,h23'], ['fr-CA', 'h23,h12'],
  ['en', 'h12,h23'], ['en-XY', 'h23'], ['abcde', 'h23,h12']
];
for (var row of cases) {
  var locale = new Intl.Locale(row[0]), tag = locale.toString();
  if (locale.getHourCycles().join(',') !== row[1]) throw row[0];
  if (locale.toString() !== tag) throw 'receiver mutation';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_hour_cycles_respects_explicit_slots_and_whole_region_keywords() {
    assert_hour_cycles(
        r#"
for (var cycle of ['h11', 'h12', 'h23', 'h24']) {
  if (new Intl.Locale('en-US-u-hc-' + cycle).getHourCycles().join(',') !== cycle) throw 'extension';
  if (new Intl.Locale('en-US-u-hc-h12', {hourCycle: cycle}).getHourCycles().join(',') !== cycle) throw 'option';
}
for (var row of [['foobar', 'foobar'], ['h11-abc', 'h11-abc'], ['', ''], ['true', '']]) {
  var tag = 'en-US-u-hc' + (row[0] ? '-' + row[0] : '');
  var locale = new Intl.Locale(tag), result = locale.getHourCycles();
  if (result.length !== 1 || !result.hasOwnProperty('0') || result[0] !== row[1]) throw 'open explicit singleton';
  if (locale.hourCycle !== result[0]) throw 'getter consistency';
  if (new Intl.Locale(tag, {hourCycle: 'h24'}).getHourCycles()[0] !== 'h24') throw 'open option precedence';
}
for (var invalid of ['foobar', 'h11-abc', '']) {
  var caught = false;
  try { new Intl.Locale('en', {hourCycle: invalid}); } catch (e) { caught = e instanceof RangeError; }
  if (!caught) throw 'closed option validation';
}
var cases = [
  ['en-u-sd-gbsct', 'h23,h12'], ['en-US-u-sd-gbsct', 'h12,h23'],
  ['en-US-u-rg-gbzzzz', 'h23,h12'], ['en-US-u-rg-aqzzzz', 'h23,h12'],
  ['en-US-u-rg-xyzzzz', 'h12,h23'], ['en-US-u-rg-gbzzzz-abc', 'h12,h23'],
  ['en-US-u-rg-gbzzzzz', 'h12,h23'], ['en-u-sd-xyfoo', 'h23'],
  ['en-US-u-fw-mon', 'h12,h23'], ['en-US-u-hc-foobar', 'foobar'],
  ['en-US-u-hc-h11-abc', 'h11-abc'], ['en-US-x-u-hc-h11', 'h12,h23']
];
for (var row of cases) {
  if (new Intl.Locale(row[0]).getHourCycles().join(',') !== row[1]) throw row[0];
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_hour_cycles_has_standard_metadata_and_fresh_dense_mutable_arrays() {
    assert_hour_cycles(
        r#"
var method = Intl.Locale.prototype.getHourCycles;
if (method.name !== 'getHourCycles' || method.length !== 0 || method.hasOwnProperty('prototype')) throw 'metadata';
var d = Object.getOwnPropertyDescriptor(Intl.Locale.prototype, 'getHourCycles');
if (!d.writable || d.enumerable || !d.configurable || d.value !== method) throw 'method descriptor';
var caught = false;
try { new method(); } catch (e) { caught = e instanceof TypeError; }
if (!caught) throw 'constructability';
var locale = new Intl.Locale('en-JP'), a = locale.getHourCycles(), b = locale.getHourCycles();
if (a === b || !Array.isArray(a) || Object.getPrototypeOf(a) !== Array.prototype) throw 'fresh array';
if (Reflect.ownKeys(a).join(',') !== '0,1,2,length') throw 'dense own keys';
for (var key of ['0', '1', '2']) {
  var entry = Object.getOwnPropertyDescriptor(a, key);
  if (!entry.writable || !entry.enumerable || !entry.configurable || !('value' in entry)) throw 'entry attributes';
}
var length = Object.getOwnPropertyDescriptor(a, 'length');
if (!length.writable || length.enumerable || length.configurable || length.value !== 3) throw 'length attributes';
a[0] = 'changed'; a.length = 0;
if (b.join(',') !== 'h23,h11,h12' || locale.getHourCycles().join(',') !== 'h23,h11,h12') throw 'result alias';
for (var tag of ['en-US-u-hc-foobar', 'en-US-u-hc']) {
  var explicit = new Intl.Locale(tag), one = explicit.getHourCycles(), two = explicit.getHourCycles();
  if (one === two || one.length !== 1 || two.length !== 1 || Object.getPrototypeOf(one) !== Array.prototype) throw 'fresh explicit';
  var entry = Object.getOwnPropertyDescriptor(one, '0'), expected = two[0];
  if (!entry.writable || !entry.enumerable || !entry.configurable || !('value' in entry)) throw 'explicit descriptor';
  one[0] = 'changed'; delete one[0];
  if (two[0] !== expected || explicit.getHourCycles()[0] !== expected) throw 'explicit result alias';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_hour_cycles_checks_brand_without_observing_receiver_or_arguments() {
    assert_hour_cycles(
        r#"
var method = Intl.Locale.prototype.getHourCycles;
for (var value of [undefined, null, true, 1, 'en', Symbol(), {}, Intl.Locale.prototype,
                   Object.create(new Intl.Locale('en')), new Proxy(new Intl.Locale('en'), {})]) {
  var caught = false;
  try { method.call(value); } catch (e) { caught = e instanceof TypeError; }
  if (!caught) throw 'receiver brand';
}
var locale = new Intl.Locale('en-US-u-hc-h24');
for (var key of ['toString', 'hourCycle', 'language', 'region', Symbol.toPrimitive]) {
  Object.defineProperty(locale, key, {get() { throw 'receiver read'; }});
}
Object.setPrototypeOf(locale, null);
if (method.call(locale, {toString() { throw 'argument coercion'; }}).join(',') !== 'h24') throw 'slot';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_hour_cycles_allocates_arrays_and_errors_in_the_method_defining_realm() {
    assert_hour_cycles(
        r#"
var foreign = __lilaCreateRealm().global;
var method = foreign.Intl.Locale.prototype.getHourCycles;
var remoteArrayPrototype = foreign.Array.prototype;
var remote = new foreign.Intl.Locale('en-US-u-hc-h24'), local = new Intl.Locale('en-JP');
var localExplicit = new Intl.Locale('en-US-u-hc-foobar');
foreign.Intl.Locale = function () { throw 'public Locale'; };
foreign.Array = function () { throw 'public Array'; };
var a = method.call(local), b = Intl.Locale.prototype.getHourCycles.call(remote);
if (Object.getPrototypeOf(a) !== remoteArrayPrototype || a.join(',') !== 'h23,h11,h12') throw 'remote Array Realm';
if (Object.getPrototypeOf(b) !== Array.prototype || b.join(',') !== 'h24') throw 'local Array Realm';
var remoteExplicit = method.call(localExplicit);
if (Object.getPrototypeOf(remoteExplicit) !== remoteArrayPrototype || remoteExplicit.length !== 1 || remoteExplicit[0] !== 'foobar') throw 'remote explicit Array Realm';
var caught = false;
try { method.call({}); } catch (e) { caught = e instanceof foreign.TypeError && !(e instanceof TypeError); }
if (!caught) throw 'error Realm';
print('ok');
"#,
        HostSurfacePolicy::Test262,
    );
}
