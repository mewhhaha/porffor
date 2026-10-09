use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_keyword_aliases(source: &str) {
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
        .expect("Intl keyword alias regression must execute through Wasm AOT");
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
fn calendar_aliases_reach_constructor_options_slots_and_existing_locale_inputs() {
    assert_keyword_aliases(
        r#"
for (var pair of [['islamicc', 'islamic-civil'], ['ethiopic-amete-alem', 'ethioaa']]) {
  var alias = pair[0], canonical = pair[1];
  for (var locale of [new Intl.Locale('en-u-ca-' + alias), new Intl.Locale('en', {calendar:alias})]) {
    if (locale.calendar !== canonical || locale.toString() !== 'en-u-ca-' + canonical ||
        locale.baseName !== 'en' || new Intl.Locale(locale).calendar !== canonical) throw 'calendar alias';
  }
  if (new Intl.Locale('en-u-ca-' + alias, {calendar:'buddhist'}).calendar !== 'buddhist') throw 'override alias';
}
if (Intl.getCanonicalLocales(['en-u-ca-islamicc','en-u-ca-islamic-civil','en-u-ca-ethiopic-amete-alem','en-u-ca-ethioaa']).join('|') !==
    'en-u-ca-islamic-civil|en-u-ca-ethioaa') throw 'alias list deduplication';
print(true);
"#,
    );
}

#[test]
fn unicode_boolean_measurement_strength_and_timezone_aliases_use_their_own_keys() {
    assert_keyword_aliases(
        r#"
for (var key of ['kb','kc','kh','kk','kn']) {
  if (Intl.getCanonicalLocales('en-u-' + key + '-yes')[0] !== 'en-u-' + key) throw 'boolean alias';
}
for (var key of ['ca','ka','kf','kr','ks','kv']) {
  if (Intl.getCanonicalLocales('en-u-' + key + '-yes')[0] !== 'en-u-' + key + '-yes') throw 'unrelated yes value';
}
for (var row of [['ms','imperial','uksystem'],['ks','primary','level1'],['ks','tertiary','level3'],
                 ['tz','cnckg','cnsha'],['tz','eire','iedub'],['tz','est','papty'],
                 ['tz','gmt0','gmt'],['tz','uct','utc'],['tz','zulu','utc']]) {
  var source = 'en-u-' + row[0] + '-' + row[1], expected = 'en-u-' + row[0] + '-' + row[2];
  if (Intl.getCanonicalLocales(source)[0] !== expected || new Intl.Locale(source).toString() !== expected) throw source;
}
if (new Intl.Locale('en-u-kn-yes').numeric !== true) throw 'numeric alias getter';
print(true);
"#,
    );
}

#[test]
fn transform_aliases_keep_language_canonicalization_and_all_value_subtags() {
    assert_keyword_aliases(
        r#"
for (var row of [['d0','name','charname'],['m0','names','prprname'],['m0','beta-metsehaf','betamets'],
                 ['m0','ies-jes','iesjes'],['m0','tekie-alibekit','tekieali']]) {
  var source = 'iw-t-iw-' + row[0] + '-' + row[1];
  var expected = 'he-t-he-' + row[0] + '-' + row[2];
  if (Intl.getCanonicalLocales(source)[0] !== expected || new Intl.Locale(source).toString() !== expected) throw source;
}
var source = 'ABCDE-Armn-SU-t-abcdef-Qaai-DD-m0-names-d0-name-u-attr-ms-imperial-ca-islamicc-rg-cn11-sd-cn11-x-islamicc-true';
var expected = 'abcde-Armn-RU-t-abcdef-zinh-de-d0-charname-m0-prprname-u-attr-ca-islamic-civil-ms-uksystem-rg-cnbj-sd-cnbj-x-islamicc-true';
if (Intl.getCanonicalLocales(source)[0] !== expected || new Intl.Locale(source).toString() !== expected) throw 'combined aliases';
print(true);
"#,
    );
}

#[test]
fn compound_true_and_alias_subtags_are_preserved_exactly() {
    assert_keyword_aliases(
        r#"
for (var source of ['en-u-ca-islamicc-true','en-u-ca-true-islamicc','en-u-ca-true-foo','en-u-ca-foo-true',
                    'en-u-ca-true-true','en-u-kn-yes-true','en-u-zz-true-foo','en-t-m0-true',
                    'en-t-en-h0-true-hybrid','en-t-m0-names-true','en-t-m0-true-names','en-t-x0-true-true',
                    'en-u-ca-islamicc-foo','en-u-ms-imperial-foo','en-t-m0-names-foo']) {
  if (Intl.getCanonicalLocales(source)[0] !== source || new Intl.Locale(source).toString() !== source) throw source;
}
if (new Intl.Locale('en', {calendar:'islamicc-true'}).calendar !== 'islamicc-true' ||
    new Intl.Locale('en-u-kn-true-foo').numeric !== false || Intl.getCanonicalLocales('en-u-ca-true')[0] !== 'en-u-ca') throw 'complete true value';
print(true);
"#,
    );
}

#[test]
fn aliases_preserve_duplicate_first_wins_private_use_and_unbounded_suffixes() {
    assert_keyword_aliases(
        r#"
var source = 'en-u-ca-islamicc-ca-ethiopic-amete-alem';
if (Intl.getCanonicalLocales(source)[0] !== 'en-u-ca-islamic-civil' || new Intl.Locale(source).calendar !== 'islamic-civil') throw 'first keyword';
for (var source of ['en-x-u-ca-islamicc','en-x-t-m0-names','en-u-co-islamicc','en-t-h0-names']) {
  if (Intl.getCanonicalLocales(source)[0] !== source || new Intl.Locale(source).toString() !== source) throw source;
}
var suffix = '-abcdefgh'.repeat(100);
var source = 'abcde-t-m0-names-u-ca-islamicc-x' + suffix;
var expected = 'abcde-t-m0-prprname-u-ca-islamic-civil-x' + suffix;
if (Intl.getCanonicalLocales(source)[0] !== expected || new Intl.Locale(source).toString() !== expected) throw 'unbounded canonical aliases';
print(true);
"#,
    );
}

#[test]
fn provider_aliases_do_not_repeat_observable_gets_or_coercions() {
    assert_keyword_aliases(
        r#"
var calls = [];
var tag = {toString:function(){ calls.push('tag'); return 'en-u-ca-islamicc'; }};
var options = {get calendar(){ calls.push('calendar'); return {toString:function(){ calls.push('calendar.string'); return 'ethiopic-amete-alem'; }}; }};
var loc = new Intl.Locale(tag, options);
if (loc.calendar !== 'ethioaa' || calls.join('|') !== 'tag|calendar|calendar.string') throw 'constructor observation order';
calls = [];
var values = {length:2, get 0(){ calls.push('get0'); return tag; }, get 1(){ calls.push('get1'); return 'en-u-ca-islamic-civil'; }};
if (Intl.getCanonicalLocales(values).join('|') !== 'en-u-ca-islamic-civil' || calls.join('|') !== 'get0|tag|get1') throw 'list observation order';
var sentinel = {}; var received;
try { new Intl.Locale('en', {get calendar(){throw sentinel;}}); } catch (e) { received=e; }
if (received !== sentinel) throw 'abrupt identity';
print(true);
"#,
    );
}
