use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_likely_subtags(source: &str, host_surface_policy: HostSurfacePolicy) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                host_surface_policy,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("Locale likely subtags must compile and execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("ok".to_string())],
        "{source}"
    );
}

#[test]
fn likely_subtags_update_components_and_keep_idempotence() {
    assert_likely_subtags(
        r#"
var cases = [
  ['en', 'en-Latn-US', 'en'], ['en-Shaw', 'en-Shaw-GB', 'en-Shaw'],
  ['en-Arab', 'en-Arab-US', 'en-Arab'], ['en-GB', 'en-Latn-GB', 'en-GB'],
  ['it-Kana-CA', 'it-Kana-CA', 'it-Kana-CA'], ['und', 'en-Latn-US', 'en'],
  ['und-Thai', 'th-Thai-TH', 'th'], ['und-419', 'es-Latn-419', 'es-419'],
  ['und-AT', 'de-Latn-AT', 'de-AT'], ['und-Cyrl-RO', 'bg-Cyrl-RO', 'bg-RO'],
  ['und-AQ', 'en-Latn-AQ', 'en-AQ'], ['zh-Hant-TW', 'zh-Hant-TW', 'zh-TW'],
  ['ccp', 'ccp-Cakm-BD', 'ccp']
];
for (var row of cases) {
  var source = new Intl.Locale(row[0]);
  var maximal = source.maximize(), minimal = source.minimize();
  if (maximal.toString() !== row[1] || minimal.toString() !== row[2]) throw row[0];
  if (maximal === source || minimal === source || maximal === minimal) throw 'fresh';
  if (maximal.maximize().toString() !== row[1] || minimal.minimize().toString() !== row[2]) throw 'idempotence';
  if (minimal.maximize().toString() !== row[1] || maximal.minimize().toString() !== row[2]) throw 'round trip';
}
var full = new Intl.Locale('en').maximize(), small = full.minimize();
if (full.language !== 'en' || full.script !== 'Latn' || full.region !== 'US' || full.baseName !== 'en-Latn-US') throw 'max slots';
if (small.language !== 'en' || small.script !== undefined || small.region !== undefined || small.baseName !== 'en') throw 'min slots';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn likely_subtags_preserve_extensions_and_reserved_languages() {
    assert_likely_subtags(
        r#"
var original = new Intl.Locale('en-fonipa-a-foobar-t-de-latn-h0-hybrid-u-ca-islamicc-kf-upper-kn-true-nu-latn-x-keep');
var suffix = original.toString().substring(2);
var full = original.maximize(), small = full.minimize();
if (full.toString() !== 'en-Latn-US' + suffix || small.toString() !== 'en' + suffix) throw 'suffix';
for (var result of [full, small]) {
  if (result.calendar !== 'islamic-civil' || result.caseFirst !== 'upper' || !result.numeric || result.numberingSystem !== 'latn') throw 'keyword slots';
}
if (full.baseName !== 'en-Latn-US-fonipa' || small.baseName !== 'en-fonipa') throw 'variant slots';
for (var tag of ['xtg', 'xtg-Latn-FR', 'mul', 'abcdefg', 'abcde-Cyrl-RU', 'abcdefgh-Latn-US-fonipa-u-co-phonebk-x-keep']) {
  var locale = new Intl.Locale(tag), canonical = locale.toString();
  if (locale.maximize().toString() !== canonical || locale.minimize().toString() !== canonical) throw tag;
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn likely_subtags_resolve_aliases_and_preserve_extra_subtags() {
    assert_likely_subtags(
        r#"
var cases = [
  ['mo', 'ro-Latn-RO', 'ro'], ['aar', 'aa-Latn-ET', 'aa'],
  ['heb', 'he-Hebr-IL', 'he'], ['hy-arevela', 'hy-Armn-AM', 'hy'],
  ['hy-arevmda', 'hyw-Armn-AM', 'hyw'], ['art-lojban', 'jbo-Latn-001', 'jbo'],
  ['cel-gaulish', 'xtg', 'xtg'], ['zh-guoyu', 'zh-Hans-CN', 'zh'],
  ['zh-hakka', 'hak-Hans-CN', 'hak'], ['zh-xiang', 'hsn-Hans-CN', 'hsn']
];
for (var row of cases) {
  for (var suffix of ['', '-u-co-phonebk', '-a-not-assigned', '-x-private']) {
    var locale = new Intl.Locale(row[0] + suffix);
    if (locale.maximize().toString() !== row[1] + suffix || locale.minimize().toString() !== row[2] + suffix) throw row[0] + suffix;
  }
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn likely_subtag_methods_validate_brands_and_function_protocol() {
    assert_likely_subtags(
        r#"
for (var name of ['maximize', 'minimize']) {
  var descriptor = Object.getOwnPropertyDescriptor(Intl.Locale.prototype, name);
  var method = descriptor.value;
  if (typeof method !== 'function' || method.name !== name || method.length !== 0) throw 'signature';
  if (!descriptor.writable || descriptor.enumerable || !descriptor.configurable || method.hasOwnProperty('prototype')) throw 'descriptor';
  for (var receiver of [undefined, null, 1, 1n, true, 'en', Symbol(), {}, [], Intl.Locale.prototype, Object.create(Intl.Locale.prototype)]) {
    var rejected = false;
    try { method.call(receiver); } catch (error) { rejected = error instanceof TypeError; }
    if (!rejected) throw 'receiver';
  }
  var proxy = new Proxy(new Intl.Locale('en'), {get() { throw 'proxy observed'; }});
  var proxyRejected = false;
  try { method.call(proxy); } catch (error) { proxyRejected = error instanceof TypeError; }
  if (!proxyRejected) throw 'proxy brand';
  var constructRejected = false;
  try { new method(); } catch (error) { constructRejected = error instanceof TypeError; }
  if (!constructRejected) throw 'construct';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn likely_subtag_methods_use_slots_and_intrinsic_result_prototypes() {
    assert_likely_subtags(
        r#"
var Constructor = Intl.Locale, prototype = Constructor.prototype;
var maximize = prototype.maximize, minimize = prototype.minimize;
class Derived extends Constructor {}
var locale = new Derived('en');
for (var key of ['constructor', 'toString', 'language', 'script', 'region', 'baseName', Symbol.toPrimitive]) {
  Object.defineProperty(locale, key, {get() { throw 'observable lookup'; }});
}
Object.setPrototypeOf(locale, null);
Intl.Locale = function () { throw 'public constructor'; };
var ignored = {toString() { throw 'argument'; }};
var full = maximize.call(locale, ignored), small = minimize.call(full, ignored);
if (Object.getPrototypeOf(full) !== prototype || Object.getPrototypeOf(small) !== prototype) throw 'prototype';
if (full.toString() !== 'en-Latn-US' || small.toString() !== 'en') throw 'slots';
if (full === locale || small === full || full instanceof Derived || small instanceof Derived) throw 'fresh intrinsic';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}

#[test]
fn likely_subtag_methods_allocate_and_throw_in_their_defining_realm() {
    assert_likely_subtags(
        r#"
var foreign = __lilaCreateRealm().global;
var foreignPrototype = foreign.Intl.Locale.prototype;
var foreignMaximize = foreignPrototype.maximize, foreignMinimize = foreignPrototype.minimize;
var local = new Intl.Locale('en'), remote = new foreign.Intl.Locale('en-Latn-US');
foreign.Intl.Locale = function () { throw 'foreign public constructor'; };
var full = foreignMaximize.call(local), small = foreignMinimize.call(local);
if (Object.getPrototypeOf(full) !== foreignPrototype || Object.getPrototypeOf(small) !== foreignPrototype) throw 'method Realm';
if (full.toString() !== 'en-Latn-US' || small.toString() !== 'en') throw 'foreign slots';
var localResult = Intl.Locale.prototype.minimize.call(remote);
if (Object.getPrototypeOf(localResult) !== Intl.Locale.prototype || localResult.toString() !== 'en') throw 'local method Realm';
for (var method of [foreignMaximize, foreignMinimize]) {
  var correct = false;
  try { method.call({}); } catch (error) { correct = error instanceof foreign.TypeError && !(error instanceof TypeError); }
  if (!correct) throw 'error Realm';
}
print('ok');
"#,
        HostSurfacePolicy::Test262,
    );
}

#[test]
fn unknown_likely_subtag_fields_preserve_extensions_and_failed_lookups() {
    assert_likely_subtags(
        r#"
var cases = [
  ['en-Zzzz-ZZ', 'en-Latn-US', 'en'],
  ['en-Zzzz', 'en-Latn-US', 'en'], ['en-ZZ', 'en-Latn-US', 'en'],
  ['en-Zzzz-GB', 'en-Latn-GB', 'en-GB'], ['en-Latn-ZZ', 'en-Latn-US', 'en'],
  ['und-Zzzz-ZZ', 'en-Latn-US', 'en'], ['und-Zzzz-419', 'es-Latn-419', 'es-419'],
  ['zh-Hant-ZZ', 'zh-Hant-TW', 'zh-TW'], ['zh-Zzzz-SG', 'zh-Hans-SG', 'zh-SG']
];
for (var row of cases) {
  var locale = new Intl.Locale(row[0]);
  if (locale.toString() !== row[0]) throw 'constructor must retain unknown fields';
  var full = locale.maximize(), small = locale.minimize();
  if (full.toString() !== row[1] || small.toString() !== row[2]) throw row[0];
  if (full.script === 'Zzzz' || full.region === 'ZZ') throw 'unknown result slots';
  if (locale.toString() !== row[0]) throw 'receiver mutation';
}
for (var tag of ['xtg-Zzzz-ZZ', 'xtg-Zzzz-FR', 'xtg-Latn-ZZ', 'mul-Zzzz-ZZ', 'abcde-Zzzz-ZZ', 'abcdefg-Zzzz-FR', 'abcdefgh-Latn-ZZ']) {
  var locale = new Intl.Locale(tag);
  if (locale.maximize().toString() !== tag || locale.minimize().toString() !== tag) throw 'failed lookup: ' + tag;
}
var suffix = '-fonipa-t-en-zzzz-zz-u-rg-zzzzzz-x-zzzz-zz';
for (var row of [['en-Zzzz-ZZ','en-Latn-US','en'], ['xtg-Zzzz-ZZ','xtg-Zzzz-ZZ','xtg-Zzzz-ZZ'], ['abcde-Zzzz-ZZ','abcde-Zzzz-ZZ','abcde-Zzzz-ZZ']]) {
  var locale = new Intl.Locale(row[0] + suffix);
  if (locale.maximize().toString() !== row[1] + suffix || locale.minimize().toString() !== row[2] + suffix) throw 'extension or fallback: ' + row[0];
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
