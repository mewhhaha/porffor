use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_intl_locale(source: &str) {
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
        .expect("Intl.Locale constructor regression must execute through Wasm AOT");
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
fn locale_options_populate_canonical_tag_and_getters() {
    assert_intl_locale(
        r#"
var loc = new Intl.Locale('en-US-fonipa', {
  language:'DE', script:'latn', region:'at', variants:'spanglis-oxendict',
  calendar:'BUDDHIST', collation:'PHONEBK', firstDayOfWeek:1,
  hourCycle:'h23', caseFirst:'upper', numeric:true, numberingSystem:'ARAB'
});
if (loc.toString() !== 'de-Latn-AT-oxendict-spanglis-u-ca-buddhist-co-phonebk-fw-mon-hc-h23-kf-upper-kn-nu-arab' ||
    loc.language !== 'de' || loc.script !== 'Latn' || loc.region !== 'AT' ||
    loc.baseName !== 'de-Latn-AT-oxendict-spanglis' || loc.variants !== 'oxendict-spanglis' ||
    loc.calendar !== 'buddhist' || loc.collation !== 'phonebk' || loc.firstDayOfWeek !== 'mon' ||
    loc.hourCycle !== 'h23' || loc.caseFirst !== 'upper' || loc.numeric !== true ||
    loc.numberingSystem !== 'arab') throw 'canonical option fields';
print(true);
"#,
    );
}

#[test]
fn locale_key_replacement_preserves_attributes_other_extensions_and_private_use() {
    assert_intl_locale(
        r#"
var loc = new Intl.Locale('en-a-other-u-attr-ca-gregory-co-phonebk-kn-false-nu-latn-z-last-x-u-ca-private', {
  calendar:'hebrew', numeric:true, numberingSystem:'arab', hourCycle:'h12'
});
if (loc.toString() !== 'en-a-other-u-attr-ca-hebrew-co-phonebk-hc-h12-kn-nu-arab-z-last-x-u-ca-private') {
  throw loc.toString();
}
var added = new Intl.Locale('en-a-other-x-u-ca-private', {calendar:'gregory'});
if (added.toString() !== 'en-a-other-u-ca-gregory-x-u-ca-private' || added.calendar !== 'gregory') throw 'new extension';
var attributes = new Intl.Locale('en-u-attr', {collation:'phonebk'});
if (attributes.toString() !== 'en-u-attr-co-phonebk') throw 'attributes before keys';
if (new Intl.Locale('en-x-u-ca-private').calendar !== undefined) throw 'private-use key';
print(true);
"#,
    );
}

#[test]
fn locale_absent_empty_and_duplicate_unicode_keywords_are_distinct() {
    assert_intl_locale(
        r#"
var plain = new Intl.Locale('en');
for (var name of ['calendar','collation','firstDayOfWeek','hourCycle','caseFirst','numberingSystem','variants']) {
  if (plain[name] !== undefined) throw 'absent field';
}
if (plain.numeric !== false) throw 'absent numeric';
var empty = new Intl.Locale('en-u-ca-co-fw-hc-kf-kn-nu');
for (var name of ['calendar','collation','firstDayOfWeek','hourCycle','caseFirst','numberingSystem']) {
  if (empty[name] !== '') throw 'present empty field';
}
if (empty.numeric !== true) throw 'present empty numeric';
var duplicate = new Intl.Locale('da-u-ca-gregory-ca-buddhist-kn-false-kn-true');
if (duplicate.calendar !== 'gregory' || duplicate.numeric !== false ||
    duplicate.toString() !== 'da-u-ca-gregory-kn-false') throw 'first keyword wins';
var canonicalTrue = new Intl.Locale('en', {calendar:'true', numeric:true});
if (canonicalTrue.calendar !== '' || canonicalTrue.numeric !== true || canonicalTrue.toString() !== 'en-u-ca-kn') {
  throw 'true canonicalizes to empty';
}
print(true);
"#,
    );
}

#[test]
fn locale_observes_each_option_and_conversion_in_spec_order() {
    assert_intl_locale(
        r#"
var log = '', receiverCorrect = true;
var values = {language:'de', script:'Latn', region:'AT', variants:'1901', calendar:'gregory',
  collation:'phonebk', firstDayOfWeek:'mon', hourCycle:'h23', caseFirst:'upper', numberingSystem:'latn'};
var names = ['language','script','region','variants','calendar','collation','firstDayOfWeek','hourCycle','caseFirst','numeric','numberingSystem'];
var options = new Proxy({}, {get(target, key, receiver) {
  receiverCorrect = receiverCorrect && receiver === options;
  log += key + ';';
  if (key === 'numeric') return {valueOf() { throw 'numeric must not coerce object'; }, toString() { throw 'numeric string'; }};
  return {toString() { log += key + ':string;'; return values[key]; }};
}});
var locale = new Intl.Locale('en', options);
var expected = '';
for (var name of names) { expected += name + ';'; if (name !== 'numeric') expected += name + ':string;'; }
if (!receiverCorrect || log !== expected || locale.numeric !== true) throw 'ordered live options';
print(true);
"#,
    );
}

#[test]
fn locale_invalid_or_abrupt_options_prevent_later_observation() {
    assert_intl_locale(
        r#"
var names = ['language','script','region','variants','calendar','collation','firstDayOfWeek','hourCycle','caseFirst','numeric','numberingSystem'];
var invalid = {language:'abcd', script:'latin', region:'U', variants:'1901-1901', calendar:'ab',
  collation:'ab', firstDayOfWeek:9, hourCycle:'H23', caseFirst:'UPPER', numberingSystem:'ab'};
for (var name of names) {
  if (name === 'numeric') continue;
  var index = names.indexOf(name), reads = [], received;
  var options = new Proxy({}, {get(target,key) { reads.push(key); return key === name ? invalid[name] : undefined; }});
  try { new Intl.Locale('en', options); } catch (error) { received = error; }
  if (!(received instanceof RangeError) || reads.join(',') !== names.slice(0,index+1).join(',')) throw 'validation order';
}
var sentinel = {}, later = false, received;
try { new Intl.Locale('en', {get calendar() { throw sentinel; }, get collation() { later = true; }}); }
catch (error) { received = error; }
if (received !== sentinel || later) throw 'getter abrupt identity';
received = undefined;
try { new Intl.Locale('en', {calendar:{toString() { throw sentinel; }}, get collation() { later = true; }}); }
catch (error) { received = error; }
if (received !== sentinel || later) throw 'coercion abrupt identity';
received = undefined;
try { new Intl.Locale('en', {calendar:Symbol('calendar')}); } catch (error) { received = error; }
if (!(received instanceof TypeError)) throw 'Symbol ToString';
print(true);
"#,
    );
}

#[test]
fn locale_variants_override_and_validate_the_variant_grammar() {
    assert_intl_locale(
        r#"
var retained = new Intl.Locale('en-fonipa-u-ca-gregory', {variants:undefined});
var replaced = new Intl.Locale('en-fonipa-u-ca-gregory', {variants:'spanglis-oxendict'});
if (retained.variants !== 'fonipa' || replaced.variants !== 'oxendict-spanglis' ||
    replaced.toString() !== 'en-oxendict-spanglis-u-ca-gregory') throw 'variants replacement';
for (var invalid of ['', 'Latn', 'US', 'abcd', 'a-long', '1901-1901', 'FONIPA-fonipa', 'fonipa-u-ca-gregory', 'abcdefghi']) {
  var received;
  try { new Intl.Locale('en', {variants:invalid}); } catch (error) { received = error; }
  if (!(received instanceof RangeError)) throw 'variant syntax';
}
var loc = new Intl.Locale('xx', {variants:'1xyz-1234-abcde-12345678'});
if (loc.variants !== '1234-12345678-1xyz-abcde') throw 'variant sorting';
print(true);
"#,
    );
}

#[test]
fn locale_resolves_language_aliases_before_overrides_and_reuses_locale_slots() {
    assert_intl_locale(
        r#"
var loc = new Intl.Locale('und-Armn-SU', {language:'ru'});
if (loc.toString() !== 'ru-Armn-AM' || loc.language !== 'ru' || loc.script !== 'Armn' ||
    loc.region !== 'AM' || loc.baseName !== 'ru-Armn-AM') throw 'canonicalize before override';
var hebrew = new Intl.Locale('iw-IL', {calendar:'hebrew'});
Object.defineProperty(hebrew, 'toString', {get() { throw 'Locale input must use slot'; }});
var copy = new Intl.Locale(hebrew, {numeric:true});
if (copy.toString() !== 'he-IL-u-ca-hebrew-kn' || copy.calendar !== 'hebrew' || copy.numeric !== true) throw 'Locale slot input';
var proxy = new Proxy(hebrew, {}), received;
try { new Intl.Locale(proxy); } catch (error) { received = error; }
if (received !== 'Locale input must use slot') throw 'proxy is not a Locale slot holder';
print(true);
"#,
    );
}

#[test]
fn locale_numeric_uses_to_boolean_without_observing_objects() {
    assert_intl_locale(
        r#"
for (var value of [false, 0, -0, NaN, 0n, '', null]) {
  var loc = new Intl.Locale('en-u-kn', {numeric:value});
  if (loc.numeric !== false || loc.toString() !== 'en-u-kn-false') throw 'falsy numeric';
}
for (var value of [true, 1, -1, 1n, 'false', Symbol('numeric'), {valueOf() { throw 'valueOf'; }}]) {
  var loc = new Intl.Locale('en-u-kn-false', {numeric:value});
  if (loc.numeric !== true || loc.toString() !== 'en-u-kn') throw 'truthy numeric';
}
if (new Intl.Locale('en-u-kn-false', {numeric:undefined}).numeric !== false) throw 'undefined retains keyword';
print(true);
"#,
    );
}

#[test]
fn locale_weekdays_and_string_option_type_grammar_are_observable() {
    assert_intl_locale(
        r#"
var weekdays = ['sun','mon','tue','wed','thu','fri','sat','sun'];
for (var index=0; index<8; index++) {
  var numeric = new Intl.Locale('en', {firstDayOfWeek:index});
  var string = new Intl.Locale('en', {firstDayOfWeek:String(index)});
  if (numeric.firstDayOfWeek !== weekdays[index] || string.firstDayOfWeek !== weekdays[index]) throw 'weekday mapping';
}
var unknown = new Intl.Locale('en', {firstDayOfWeek:'ABCDEFGH-abc123', calendar:null, collation:'1234abcd-abc123', numberingSystem:'abcdefgh'});
if (unknown.firstDayOfWeek !== 'abcdefgh-abc123' || unknown.calendar !== 'null' || unknown.collation !== '1234abcd-abc123') throw 'well-formed unknown options';
print(true);
"#,
    );
}

#[test]
fn locale_getter_identity_brand_and_error_realm_follow_intrinsics() {
    assert_intl_locale(
        r#"
var foreign = __lilaCreateRealm().global;
var names = ['calendar','collation','firstDayOfWeek','hourCycle','caseFirst','numeric','numberingSystem','variants'];
for (var name of names) {
  var descriptor = Object.getOwnPropertyDescriptor(Intl.Locale.prototype,name);
  var other = Object.getOwnPropertyDescriptor(foreign.Intl.Locale.prototype,name).get;
  if (typeof descriptor.get !== 'function' || descriptor.set !== undefined || descriptor.enumerable ||
      !descriptor.configurable || descriptor.get.name !== 'get '+name || descriptor.get.length !== 0 ||
      descriptor.get === other || Object.getPrototypeOf(other) !== foreign.Function.prototype) throw 'getter metadata';
  for (var receiver of [undefined, null, {}, Intl.Locale.prototype, new Proxy(new Intl.Locale('en'), {})]) {
    var received;
    try { other.call(receiver); } catch (error) { received = error; }
    if (!(received instanceof foreign.TypeError) || received instanceof TypeError) throw 'getter realm branding';
  }
}
var loc = new foreign.Intl.Locale('de-1901-u-ca-gregory');
if (Object.getOwnPropertyDescriptor(Intl.Locale.prototype,'calendar').get.call(loc) !== 'gregory' ||
    Object.getOwnPropertyDescriptor(Intl.Locale.prototype,'variants').get.call(loc) !== '1901') throw 'cross-realm slots';
print(true);
"#,
    );
}

#[test]
fn locale_and_canonical_lists_preserve_long_private_use_without_recoercion() {
    assert_intl_locale(
        r#"
var tail = '-abcdefgh'.repeat(100), source = 'iw-IL-x'+tail, reads=0;
var tag = {toString() {reads++; return source;}};
var loc = new Intl.Locale(tag, {calendar:'gregory'});
if (loc.toString() !== 'he-IL-u-ca-gregory-x'+tail || reads !== 1 || loc.baseName !== 'he-IL') throw 'long Locale tag';
var list = Intl.getCanonicalLocales([tag]);
if (list.length !== 1 || list[0] !== 'he-IL-x'+tail || reads !== 2) throw 'long canonical locale list';
print(true);
"#,
    );
}

#[test]
fn locale_reserved_languages_preserve_unknown_language_territory_semantics() {
    assert_intl_locale(
        r#"
for (var language of ['abcde','abcdef','abcdefg','abcdefgh']) {
  var loc = new Intl.Locale(language, {calendar:'gregory', variants:'1901'});
  if (loc.language !== language || loc.toString() !== language+'-1901-u-ca-gregory') throw 'reserved language';
}
var unknown = new Intl.Locale('abcde-Armn-SU');
var absent = new Intl.Locale('und-Armn-SU');
if (unknown.region !== 'RU' || absent.region !== 'AM') throw 'unknown language differs from und';
var transformed = new Intl.Locale('abcde-Qaai-DD-t-abcdef-Qaai-DD-h0-hybrid-u-ca-gregory');
if (transformed.toString() !== 'abcde-Zinh-DE-t-abcdef-zinh-de-h0-hybrid-u-ca-gregory') throw 'independent transform aliases';
if (new Intl.Locale('abcde-t-iw-il').toString() !== 'abcde-t-he-il') throw 'native transform language alias';
if (Intl.getCanonicalLocales('ABCDE')[0] !== 'abcde') throw 'shared provider domain';
print(true);
"#,
    );
}
