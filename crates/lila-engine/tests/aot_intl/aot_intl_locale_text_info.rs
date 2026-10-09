use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_text_info(source: &str, policy: HostSurfacePolicy) {
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
            .expect("text info must compile and execute through Wasm AOT");
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
fn locale_text_info_resolves_explicit_and_likely_scripts_and_preserves_unknowns() {
    assert_text_info(
        r#"
var cases = [
  ['en', 'ltr'], ['ar', 'rtl'], ['he', 'rtl'], ['fa', 'rtl'], ['ru', 'ltr'],
  ['pa-IN', 'ltr'], ['pa-PK', 'rtl'],
  ['en-Arab', 'rtl'], ['ar-Latn', 'ltr'], ['en-Hebr', 'rtl'],
  ['en-Brai', undefined], ['en-Zyyy', undefined], ['en-Zinh', undefined],
  ['en-Zzzz', undefined], ['en-Qaaa', undefined],
  ['ar-u-rg-uszzzz-sd-usca-fw-mon', 'rtl'], ['en-x-arab', 'ltr']
];
for (var row of cases) {
  var locale = new Intl.Locale(row[0]), tag = locale.toString();
  if (locale.getTextInfo().direction !== row[1]) throw row[0];
  if (locale.toString() !== tag) throw 'receiver mutation';
}

if (new Intl.Locale('ar', {script: 'Latn'}).getTextInfo().direction !== 'ltr') throw 'script option';
if (new Intl.Locale('en', {script: 'Arab'}).getTextInfo().direction !== 'rtl') throw 'script option';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_text_info_has_standard_metadata_and_fresh_mutable_results() {
    assert_text_info(
        r#"
var method = Intl.Locale.prototype.getTextInfo;
if (method.name !== 'getTextInfo' || method.length !== 0 || method.hasOwnProperty('prototype')) throw 'method metadata';
var descriptor = Object.getOwnPropertyDescriptor(Intl.Locale.prototype, 'getTextInfo');
if (!descriptor.writable || descriptor.enumerable || !descriptor.configurable || descriptor.value !== method) throw 'method attributes';
var caught = false;
try { new method(); } catch (e) { caught = e instanceof TypeError; }
if (!caught) throw 'constructability';
for (var tag of ['en', 'ar', 'en-Brai']) {
  var locale = new Intl.Locale(tag), a = locale.getTextInfo(), b = locale.getTextInfo();
  if (a === b || Object.getPrototypeOf(a) !== Object.prototype || Reflect.ownKeys(a).join(',') !== 'direction') throw 'fresh record';
  var d = Object.getOwnPropertyDescriptor(a, 'direction');
  if (!d.writable || !d.enumerable || !d.configurable || !('value' in d)) throw 'result attributes';
  var direction = b.direction;
  a.direction = 'changed'; delete a.direction;
  if (b.direction !== direction || locale.getTextInfo().direction !== direction) throw 'result alias';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn locale_text_info_allocates_and_throws_in_the_method_defining_realm() {
    assert_text_info(
        r#"
var foreign = __lilaCreateRealm().global;
var method = foreign.Intl.Locale.prototype.getTextInfo;
var remoteObjectPrototype = foreign.Object.prototype;
var remote = new foreign.Intl.Locale('ar'), local = new Intl.Locale('en-Brai');
foreign.Intl.Locale = function () { throw 'public constructor'; };
var a = method.call(local), b = Intl.Locale.prototype.getTextInfo.call(remote);
if (Object.getPrototypeOf(a) !== remoteObjectPrototype || a.direction !== undefined) throw 'remote result Realm';
if (!a.hasOwnProperty('direction')) throw 'unknown property';
if (Object.getPrototypeOf(b) !== Object.prototype || b.direction !== 'rtl') throw 'local result Realm';
var caught = false;
try { method.call({}); } catch (e) { caught = e instanceof foreign.TypeError && !(e instanceof TypeError); }
if (!caught) throw 'error Realm';
print('ok');
"#,
        HostSurfacePolicy::Test262,
    );
}
#[test]
fn locale_text_info_checks_slots_without_receiver_reads_or_argument_coercion() {
    assert_text_info(
        r#"
var method = Intl.Locale.prototype.getTextInfo;
for (var value of [undefined, null, true, 1, 'ar', Symbol(), {}, Intl.Locale.prototype,
                   Object.create(new Intl.Locale('ar')), new Proxy(new Intl.Locale('ar'), {})]) {
  var caught = false;
  try { method.call(value); } catch (e) { caught = e instanceof TypeError; }
  if (!caught) throw 'receiver brand';
}
var locale = new Intl.Locale('ar');
for (var key of ['toString', 'script', 'language', 'region', Symbol.toPrimitive]) {
  Object.defineProperty(locale, key, {get() { throw 'receiver read'; }});
}
Object.setPrototypeOf(locale, null);
var result = method.call(locale, {toString() { throw 'argument coercion'; }});
if (result.direction !== 'rtl') throw 'immutable locale slot';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
