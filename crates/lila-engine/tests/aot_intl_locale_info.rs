use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_prints_ok(source: &str) {
    assert_prints_ok_with(source, HostSurfacePolicy::default());
}

fn assert_prints_ok_with(source: &str, host_surface_policy: HostSurfacePolicy) {
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
        .expect("Locale information must compile and execute through Wasm AOT");
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
fn locale_information_methods_follow_region_preference_and_keywords() {
    assert_prints_ok(
        r#"
function same(actual, expected, label) {
  if (JSON.stringify(actual) !== JSON.stringify(expected)) throw label + ': ' + JSON.stringify(actual);
}
same(new Intl.Locale('fr-CA').getHourCycles(), ['h23', 'h12'], 'language-region time data');
same(new Intl.Locale('und-CA').getHourCycles(), ['h12', 'h23'], 'region time data');
same(new Intl.Locale('en-US-u-sd-gbeng-rg-gbzzzz').getHourCycles(), new Intl.Locale('en-GB').getHourCycles(), 'rg');
same(new Intl.Locale('en-u-sd-gbeng').getHourCycles(), new Intl.Locale('en-GB').getHourCycles(), 'sd');
same(new Intl.Locale('eo').getHourCycles(), new Intl.Locale('eo-001').getHourCycles(), '001');
same(new Intl.Locale('en', { hourCycle: 'h11' }).getHourCycles(), ['h11'], 'hc');
same(new Intl.Locale('zh-TW').getCalendars(), ['gregory', 'roc', 'chinese'], 'calendars');
same(new Intl.Locale('fa-JP-u-sd-inka-rg-thzzzz').getCalendars(), ['buddhist', 'gregory'], 'rg calendars');
same(new Intl.Locale('th', { calendar: 'buddhist' }).getCalendars(), ['buddhist'], 'ca');
same(new Intl.Locale('de-AT').getCollations(), ['emoji', 'eor', 'phonebk'], 'collations');
same(new Intl.Locale('qfz').getCollations(), ['emoji', 'eor'], 'unmatched collations');
same(new Intl.Locale('und-Latn-US').getCollations(), ['emoji', 'eor'], 'und collations');
same(new Intl.Locale('ar-EG').getNumberingSystems(), ['arab'], 'numbering');
same(new Intl.Locale('en').getTimeZones(), undefined, 'no region');
same(new Intl.Locale('en-419').getTimeZones(), [], 'no zones');
same(new Intl.Locale('fr-FR').getTimeZones(), ['Europe/Paris'], 'zones');
var text = new Intl.Locale('he').getTextInfo();
same(Reflect.ownKeys(text), ['direction'], 'text keys');
same(text.direction, 'rtl', 'rtl');
var unknown = new Intl.Locale('en-Zyyy').getTextInfo();
same(Reflect.ownKeys(unknown), ['direction'], 'undefined direction is defined');
if (unknown.direction !== undefined) throw 'undefined direction';
var week = new Intl.Locale('fa-JP-u-sd-inka-rg-afzzzz').getWeekInfo();
same(week, { firstDay: 6, weekend: [4, 5] }, 'week rg');
same(Reflect.ownKeys(week), ['firstDay', 'weekend'], 'week keys');
if (Object.getPrototypeOf(week) !== Object.prototype || !Array.isArray(week.weekend)) throw 'week shape';
same(new Intl.Locale('en', { firstDayOfWeek: 0 }).getWeekInfo().firstDay, 7, 'fw');
var getCalendars = Intl.Locale.prototype.getCalendars;
try { getCalendars.call(Intl.Locale.prototype); throw 'branding'; } catch (error) {
  if (!(error instanceof TypeError)) throw error;
}
print('ok');
"#,
    );
}

#[test]
fn supported_values_of_reads_one_closed_key_after_to_string() {
    assert_prints_ok(
        r#"
var calendars = Intl.supportedValuesOf({ toString() { return 'calendar'; } });
if (JSON.stringify(calendars) !== '["buddhist","chinese","coptic","dangi","ethioaa","ethiopic","gregory","hebrew","indian","islamic-civil","islamic-tbla","islamic-umalqura","iso8601","japanese","persian","roc"]') throw 'calendars';
for (var key of ['collation', 'currency', 'numberingSystem', 'timeZone', 'unit']) {
  var values = Intl.supportedValuesOf(key);
  if (!Array.isArray(values) || values.length === 0) throw key;
  for (var i = 1; i < values.length; i++) if (!(values[i - 1] < values[i])) throw key + ' order';
}
if (Intl.supportedValuesOf('timeZone').indexOf('UTC') < 0) throw 'UTC';
if (Intl.supportedValuesOf('collation').indexOf('standard') >= 0) throw 'standard';
try { Intl.supportedValuesOf('calendars'); throw 'key'; } catch (error) {
  if (!(error instanceof RangeError)) throw error;
}
if (Intl.supportedValuesOf.length !== 1 || Intl.supportedValuesOf.name !== 'supportedValuesOf') throw 'metadata';
print('ok');
"#,
    );
}

#[test]
fn created_realms_install_their_own_locale_information_functions() {
    assert_prints_ok_with(
        r#"
var other = __lilaCreateRealm().global;
var method = other.Intl.Locale.prototype.getWeekInfo;
if (method === Intl.Locale.prototype.getWeekInfo) throw 'identity';
var info = method.call(new Intl.Locale('en'));
if (Object.getPrototypeOf(info) !== other.Object.prototype) throw 'realm prototype';
if (Object.getPrototypeOf(info.weekend) !== other.Array.prototype) throw 'realm array';
var values = other.Intl.supportedValuesOf('unit');
if (Object.getPrototypeOf(values) !== other.Array.prototype) throw 'values realm';
try { other.Intl.supportedValuesOf('bad'); throw 'key'; } catch (error) {
  if (!(error instanceof other.RangeError)) throw 'error realm';
}
print('ok');
"#,
        HostSurfacePolicy::Test262,
    );
}
