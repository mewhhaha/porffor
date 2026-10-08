use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_week_info(source: &str, policy: HostSurfacePolicy) {
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
            .expect("week info must compile and execute through Wasm AOT");
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
fn locale_week_info_checks_internal_slots_without_observable_receiver_reads() {
    assert_week_info(
        r#"
var method = Intl.Locale.prototype.getWeekInfo;
for (var value of [undefined, null, true, 1, 'en-US', {}, Intl.Locale.prototype,
                   Object.create(new Intl.Locale('en-US')), new Proxy(new Intl.Locale('en-US'), {})]) {
  var caught = false;
  try { method.call(value); } catch (e) { caught = e instanceof TypeError; }
  if (!caught) throw 'receiver branding';
}
var locale = new Intl.Locale('en-US');
for (var key of ['toString', 'language', 'region', 'firstDayOfWeek', Symbol.toPrimitive]) {
  Object.defineProperty(locale, key, {get() { throw 'receiver property read'; }});
}
Object.setPrototypeOf(locale, null);
var result = method.call(locale, {toString() { throw 'argument coercion'; }});
if (result.firstDay !== 7 || result.weekend.join(',') !== '6,7') throw 'internal data';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_week_info_creates_fresh_mutable_ordinary_records_and_dense_arrays() {
    assert_week_info(
        r#"
var locale = new Intl.Locale('en-US'), a = locale.getWeekInfo(), b = locale.getWeekInfo();
if (a === b || a.weekend === b.weekend) throw 'fresh results';
if (Object.getPrototypeOf(a) !== Object.prototype || Object.getPrototypeOf(a.weekend) !== Array.prototype) throw 'prototypes';
if (Reflect.ownKeys(a).join(',') !== 'firstDay,weekend' || 'minimalDays' in a) throw 'fields';
for (var key of ['firstDay', 'weekend']) {
  var d = Object.getOwnPropertyDescriptor(a, key);
  if (!d.writable || !d.enumerable || !d.configurable || !('value' in d)) throw 'record attributes';
}

for (var key of ['0', '1']) {
  var d = Object.getOwnPropertyDescriptor(a.weekend, key);
  if (!d.writable || !d.enumerable || !d.configurable || !('value' in d)) throw 'array attributes';
}
a.firstDay = 2; a.weekend[0] = 1; a.weekend.push(3); delete a.weekend[1];
if (b.firstDay !== 7 || b.weekend.join(',') !== '6,7' || locale.getWeekInfo().weekend.join(',') !== '6,7') throw 'data alias';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_week_info_method_has_standard_metadata_and_is_not_constructable() {
    assert_week_info(
        r#"
var method = Intl.Locale.prototype.getWeekInfo;
if (method.name !== 'getWeekInfo' || method.length !== 0 || method.hasOwnProperty('prototype')) throw 'metadata';
var d = Object.getOwnPropertyDescriptor(Intl.Locale.prototype, 'getWeekInfo');
if (!d.writable || d.enumerable || !d.configurable || d.value !== method) throw 'method descriptor';
for (var action of [function () { return new method(); }, function () { return Reflect.construct(method, []); }]) {
  var caught = false;
  try { action(); } catch (e) { caught = e instanceof TypeError; }
  if (!caught) throw 'constructability';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_week_info_uses_first_day_preferences_and_ignores_unknown_preferences() {
    assert_week_info(
        r#"
var names = ['mon', 'tue', 'wed', 'thu', 'fri', 'sat', 'sun'];
for (var i = 0; i < names.length; i++) {
  var a = new Intl.Locale('en-US-u-fw-' + names[i]).getWeekInfo();
  var b = new Intl.Locale('en-US-u-fw-sun', {firstDayOfWeek: names[i]}).getWeekInfo();
  var c = new Intl.Locale('en-US', {firstDayOfWeek: i + 1}).getWeekInfo();
  if (a.firstDay !== i + 1 || b.firstDay !== i + 1 || c.firstDay !== i + 1) throw 'preference';
  if (a.weekend.join(',') !== '6,7' || b.weekend.join(',') !== '6,7' || c.weekend.join(',') !== '6,7') throw 'regional weekend';
}

for (var tag of ['en-US-u-fw-unknown', 'en-US-u-fw']) {
  if (new Intl.Locale(tag).getWeekInfo().firstDay !== 7) throw 'unknown preference';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_week_info_resolves_region_subdivision_and_region_override_in_order() {
    assert_week_info(
        r#"
var cases = [
  ['en', 7, '6,7'], ['en-US', 7, '6,7'], ['en-GB', 1, '6,7'],
  ['en-u-sd-gbsct', 1, '6,7'], ['en-US-u-sd-gbsct', 7, '6,7'],
  ['en-US-u-rg-gbzzzz', 1, '6,7'], ['en-GB-u-rg-uszzzz', 7, '6,7'],
  ['en-US-u-rg-gbzzzz-fw-tue', 2, '6,7'],
  ['en-GB-u-rg-uszzzz-fw-fri', 5, '6,7'],
  ['en-IN', 7, '7'], ['en-IR', 6, '5'], ['en-AF', 6, '4,5']
];
for (var row of cases) {
  var locale = new Intl.Locale(row[0]), text = locale.toString();
  var info = locale.getWeekInfo();
  if (info.firstDay !== row[1] || info.weekend.join(',') !== row[2]) throw row[0];
  if (locale.toString() !== text) throw 'receiver mutation';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_week_info_allocates_and_throws_in_the_method_defining_realm() {
    assert_week_info(
        r#"
var foreign = __lilaCreateRealm().global;
var remoteMethod = foreign.Intl.Locale.prototype.getWeekInfo;
var remoteObjectPrototype = foreign.Object.prototype, remoteArrayPrototype = foreign.Array.prototype;
var local = new Intl.Locale('en-US'), remote = new foreign.Intl.Locale('en-GB');
foreign.Intl.Locale = function () { throw 'public constructor'; };
var a = remoteMethod.call(local), b = Intl.Locale.prototype.getWeekInfo.call(remote);
if (Object.getPrototypeOf(a) !== remoteObjectPrototype || Object.getPrototypeOf(a.weekend) !== remoteArrayPrototype) throw 'remote allocation Realm';
if (Object.getPrototypeOf(b) !== Object.prototype || Object.getPrototypeOf(b.weekend) !== Array.prototype) throw 'local allocation Realm';
if (a.firstDay !== 7 || b.firstDay !== 1) throw 'receiver slots';
var caught = false;
try { remoteMethod.call({}); } catch (e) { caught = e instanceof foreign.TypeError && !(e instanceof TypeError); }
if (!caught) throw 'error Realm';
print('ok');
"#,
        HostSurfacePolicy::Test262,
    );
}
