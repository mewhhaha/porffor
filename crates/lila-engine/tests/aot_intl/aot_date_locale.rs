use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_date_locale(source: &str) {
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
        .expect("Date locale regression must compile and execute through Wasm AOT");
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
fn date_only_program_roots_locale_initialization_and_bound_formatting() {
    assert_date_locale(
        r#"
var date = new Date(0);
var options = {timeZone:'UTC',year:'numeric'};
if (date.toLocaleString('en', options) !== '1970') throw 'combined locale fields';
if (date.toLocaleDateString('en', options) !== '1970') throw 'date locale fields';
var time = date.toLocaleTimeString('en', {timeZone:'UTC',hour12:false});
if (typeof time !== 'string' || time.length === 0) throw 'time locale formatting';
print(true);
"#,
    );
}

#[test]
fn all_three_date_methods_share_numbering_and_correct_default_fields() {
    assert_date_locale(
        r#"
var date = new Date(1704076506789);
var rows = [
  ['toLocaleString', {year:'numeric',month:'numeric',day:'numeric',hour:'numeric',minute:'numeric',second:'numeric'}],
  ['toLocaleDateString', {year:'numeric',month:'numeric',day:'numeric'}],
  ['toLocaleTimeString', {hour:'numeric',minute:'numeric',second:'numeric'}]
];
for (var row of rows) {
  var options = Object.freeze({timeZone:'UTC',numberingSystem:'arab',hour12:false});
  var expectedOptions = Object.assign({}, row[1], options);
  var expected = new Intl.DateTimeFormat('en', expectedOptions).format(date);
  if (date[row[0]]('en', options) !== expected) throw row[0] + ' defaults';
}
var components = Object.freeze({timeZone:'UTC',year:'2-digit',month:'long',day:'numeric',hour:'2-digit',minute:'2-digit'});
var explicit = new Intl.DateTimeFormat('en-u-nu-deva', components).format(date);
for (var row of rows) {
  if (date[row[0]]('en-u-nu-deva', components) !== explicit) throw row[0] + ' explicit fields';
}
print(true);
"#,
    );
}

#[test]
fn required_fields_control_defaults_without_erasing_other_explicit_components() {
    assert_date_locale(
        r#"
var date = new Date(0);
var rows = [
  ['toLocaleString', {era:'narrow'}, {year:'numeric',month:'numeric',day:'numeric',hour:'numeric',minute:'numeric',second:'numeric'}],
  ['toLocaleDateString', {hour:'2-digit'}, {year:'numeric',month:'numeric',day:'numeric'}],
  ['toLocaleTimeString', {year:'2-digit'}, {hour:'numeric',minute:'numeric',second:'numeric'}],
  ['toLocaleTimeString', {fractionalSecondDigits:3}, {}]
];
for (var row of rows) {
  var options = Object.assign({timeZone:'UTC'}, row[1]);
  var expectedOptions = Object.assign({}, options, row[2]);
  var expected = new Intl.DateTimeFormat('en', expectedOptions).format(date);
  if (date[row[0]]('en', options) !== expected) throw row[0] + ' required/default fields';
}
print(true);
"#,
    );
}

#[test]
fn locale_and_option_observations_happen_once_in_specification_order() {
    assert_date_locale(
        r#"
var names = ['localeMatcher','calendar','numberingSystem','hour12','hourCycle','timeZone','weekday','era','year','month','day','dayPeriod','hour','minute','second','fractionalSecondDigits','timeZoneName','formatMatcher','dateStyle','timeStyle'];
for (var method of ['toLocaleString','toLocaleDateString','toLocaleTimeString']) {
  var trace = [];
  var locales = {get length() { trace.push('locales'); return 1; }, get 0() { trace.push('locale'); return 'en'; }};
  var options = new Proxy({}, { get(target, name) { trace.push(name); return name === 'timeZone' ? 'UTC' : undefined; }});
  Date.prototype[method].call(new Date(0), locales, options);
  if (trace.join(',') !== ['locales','locale'].concat(names).join(',')) throw method + ':' + trace.join(',');
}
print(true);
"#,
    );
}

#[test]
fn null_locales_reject_before_options_in_the_executing_method_realm() {
    assert_date_locale(
        r#"
var foreign = __lilaCreateRealm().global;
var observations = 0;
var options = {get timeZone() { observations++; throw 'options after null locales'; }};
for (var method of ['toLocaleString','toLocaleDateString','toLocaleTimeString']) {
  var caught = false;
  try { foreign.Date.prototype[method].call(new Date(0), null, options); }
  catch (error) { caught = error.constructor === foreign.TypeError; }
  if (!caught) throw method + ' null locales Realm';
  if (foreign.Date.prototype[method].call(new Date(NaN), null, options) !== 'Invalid Date') throw method + ' invalid Date';
}
if (observations !== 0) throw 'null locales observed options';
print(true);
"#,
    );
}

#[test]
fn locale_instances_use_their_internal_identifier_before_array_like_observation() {
    assert_date_locale(
        r#"
var locale = new Intl.Locale('en-u-nu-arab');
locale.toString = function () { throw 'locale toString'; };
Object.defineProperty(locale, 'length', {get() { throw 'locale length'; }});
var date = new Date(0);
var options = {timeZone:'UTC',year:'numeric',hour:'numeric'};
Intl.getCanonicalLocales = function () { throw 'replaced canonicalization'; };
for (var method of ['toLocaleString','toLocaleDateString','toLocaleTimeString']) {
  var expected = date[method]('en-u-nu-arab', options);
  if (date[method](locale, options) !== expected) throw method + ' locale identifier';
}
print(true);
"#,
    );
}

#[test]
fn locale_lists_skip_absent_indices_and_reject_present_undefined_entries() {
    assert_date_locale(
        r#"
var sparse = new Array(3);
sparse[2] = 'en';
var trace = [];
var locales = new Proxy(sparse, {
  has(target, key) { trace.push('has:' + key); return key in target; },
  get(target, key) { trace.push('get:' + key); return target[key]; }
});
var date = new Date(0);
for (var method of ['toLocaleString','toLocaleDateString','toLocaleTimeString']) {
  trace = [];
  date[method](locales, {timeZone:'UTC'});
  if (trace.join(',') !== 'get:length,has:0,has:1,has:2,get:2') throw method + ':' + trace.join(',');
  var caught = false;
  try { date[method](['en', undefined], {timeZone:'UTC'}); }
  catch (error) { caught = error instanceof TypeError; }
  if (!caught) throw method + ' present undefined locale';
}
print(true);
"#,
    );
}

#[test]
fn locale_element_conversion_precedes_later_getters_and_preserves_abrupt_values() {
    assert_date_locale(
        r#"
var date = new Date(0), trace = [];
var locales = {
  get length() { trace.push('length'); return 2; },
  get 0() { trace.push('get0'); return {toString() { trace.push('string0'); return 'en'; }}; },
  get 1() { trace.push('get1'); return 'en'; }
};
var marker = {}, later = 0;
var abrupt = {
  length:2,
  0:{toString() { throw marker; }},
  get 1() { later++; return 'en'; }
};
for (var method of ['toLocaleString','toLocaleDateString','toLocaleTimeString']) {
  trace = [];
  date[method](locales, {timeZone:'UTC'});
  if (trace.join(',') !== 'length,get0,string0,get1') throw method + ':' + trace.join(',');
  var caught = false;
  try { date[method](abrupt, {get timeZone() { later++; return 'UTC'; }}); }
  catch (error) { caught = error === marker; }
  if (!caught || later !== 0) throw method + ' abrupt locale conversion';
}
print(true);
"#,
    );
}

#[test]
fn invalid_dates_return_before_locale_options_and_receiver_coercion() {
    assert_date_locale(
        r#"
var marker = {}, observations = 0;
var locales = {get length() { observations++; throw marker; }};
var options = new Proxy({}, {get() { observations++; throw marker; }});
for (var method of ['toLocaleString','toLocaleDateString','toLocaleTimeString']) {
  var invalid = new Date(NaN);
  invalid.valueOf = function () { observations++; throw marker; };
  if (Date.prototype[method].call(invalid, locales, options) !== 'Invalid Date') throw method;
  var caught = false;
  try { Date.prototype[method].call({valueOf() { observations++; return 0; }}, locales, options); }
  catch (error) { caught = error instanceof TypeError; }
  if (!caught) throw method + ' brand';
}
if (observations !== 0) throw 'unexpected coercion';
print(true);
"#,
    );
}

#[test]
fn date_value_is_snapshotted_before_options_and_intrinsic_formatting_ignores_replacements() {
    assert_date_locale(
        r#"
var date = new Date(0);
var constructor = Intl.DateTimeFormat;
var expected = new constructor('en', {timeZone:'UTC',year:'numeric'}).format(0);
var calls = 0;
var options = {timeZone:'UTC', get year() { calls++; date.setTime(31536000000); return 'numeric'; }};
Intl.DateTimeFormat = function () { throw 'replaced constructor'; };
Object.defineProperty(constructor.prototype, 'format', {get() { throw 'replaced format'; }});
date.valueOf = function () { throw 'receiver valueOf'; };
var result = date.toLocaleDateString('en', options);
if (result !== expected || calls !== 1 || date.getTime() !== 31536000000) throw 'date snapshot';
print(true);
"#,
    );
}

#[test]
fn primitive_options_are_boxed_in_the_executing_method_realm() {
    assert_date_locale(
        r#"
var foreign = __lilaCreateRealm().global;
var calls = 0;
Object.defineProperty(Number.prototype, 'year', {get() { throw 'caller Number prototype'; }});
Object.defineProperty(foreign.Number.prototype, 'timeZone', {value:'UTC'});
Object.defineProperty(foreign.Number.prototype, 'year', {get() {
  calls++;
  if (Object.getPrototypeOf(this) !== foreign.Number.prototype) throw 'options wrapper Realm';
  return 'numeric';
}});
var result = foreign.Date.prototype.toLocaleDateString.call(new Date(0), 'en', 42);
if (result !== '1970' || calls !== 1) throw 'Date primitive options';
var formatter = foreign.Intl.DateTimeFormat('en', 42);
if (formatter.format(0) !== '1970' || calls !== 2) throw 'constructor primitive options';
print(true);
"#,
    );
}

#[test]
fn invalid_styles_and_locales_use_the_executing_method_realm_and_preserve_abrupt_values() {
    assert_date_locale(
        r#"
var foreign = __lilaCreateRealm().global;
var cases = [
  ['toLocaleDateString', 'en', {timeStyle:'short'}, foreign.TypeError],
  ['toLocaleTimeString', 'en', {dateStyle:'short'}, foreign.TypeError],
  ['toLocaleString', 'en', {dateStyle:'short',year:'numeric'}, foreign.TypeError],
  ['toLocaleString', 'bad_tag', {}, foreign.RangeError],
  ['toLocaleString', 'en', null, foreign.TypeError]
];
for (var row of cases) {
  var caught = false;
  try { foreign.Date.prototype[row[0]].call(new Date(0), row[1], row[2]); }
  catch (error) { caught = error.constructor === row[3]; }
  if (!caught) throw row[0] + ' error Realm';
}
var marker = {}, caught = false;
try { foreign.Date.prototype.toLocaleString.call(new Date(0), 'en', {get timeZone() { throw marker; }}); }
catch (error) { caught = error === marker; }
if (!caught) throw 'abrupt option value';
print(true);
"#,
    );
}
