use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_zone_script(source: &str) {
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
        .expect("named zones must compile and execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_string())],
        "{source}"
    );
}

#[test]
fn named_zone_lookup_preserves_identifier_identity_and_single_observation() {
    assert_zone_script(
        r#"
for (var row of [['aMeRiCa/nEw_yOrK','America/New_York'],['uTc','UTC'],['etc/utc','Etc/UTC'],['Asia/Calcutta','Asia/Calcutta'],['Asia/Kolkata','Asia/Kolkata']]) {
  var gets=0, strings=0;
  var zone={toString(){strings++;return row[0];}};
  var formatter=new Intl.DateTimeFormat('en',{get timeZone(){gets++;return zone;}});
  if(formatter.resolvedOptions().timeZone!==row[1] || gets!==1 || strings!==1) throw row[0];
}
var a=new Intl.DateTimeFormat('en',{timeZone:'Asia/Calcutta',timeZoneName:'long'});
var b=new Intl.DateTimeFormat('en',{timeZone:'Asia/Kolkata',timeZoneName:'long'});
if(a.format(0)!==b.format(0)) throw 'link display equivalence';
var marker={}, later=0;
try { new Intl.DateTimeFormat('en',{get timeZone(){return {toString(){throw marker;}};},get year(){later++;}}); throw 'missing abrupt'; }
catch(error) { if(error!==marker || later!==0) throw 'coercion identity'; }
print(true);
"#,
    );
}

#[test]
fn unknown_names_and_invalid_offsets_throw_in_the_constructor_realm() {
    assert_zone_script(
        r#"
var foreign=__lilaCreateRealm().global;
for(var zone of ['Mars/Olympus_Mons','America/New_York\u0000','éurope/Paris','+24:00','+01:02:03','']) {
  var observed=0,caught=false;
  try { new foreign.Intl.DateTimeFormat('en',{timeZone:zone,get year(){observed++;}}); }
  catch(error) { caught=error.constructor===foreign.RangeError; }
  if(!caught || observed!==0) throw 'invalid zone '+zone;
}
print(true);
"#,
    );
}

#[test]
fn transition_snapshots_keep_historical_seconds_and_non_hour_changes() {
    assert_zone_script(
        r#"
function part(formatter,epoch,type){for(var p of formatter.formatToParts(epoch))if(p.type===type)return p.value;throw type;}
var rows=[
 ['Europe/Paris',-2208988800000,'GMT+00:09:21'],
 ['Europe/Paris',-1855958961001,'GMT+00:09:21'],
 ['Europe/Paris',-1855958961000,'GMT'],
 ['America/New_York',1772953199999,'GMT-05:00'],
 ['America/New_York',1772953200000,'GMT-04:00'],
 ['America/New_York',1793512799999,'GMT-04:00'],
 ['America/New_York',1793512800000,'GMT-05:00'],
 ['Australia/Lord_Howe',1775314799999,'GMT+11:00'],
 ['Australia/Lord_Howe',1775314800000,'GMT+10:30'],
 ['Pacific/Apia',1325239199999,'GMT-10:00'],
 ['Pacific/Apia',1325239200000,'GMT+14:00']
];
for(var row of rows){
 var f=new Intl.DateTimeFormat('en',{timeZone:row[0],hour:'2-digit',minute:'2-digit',second:'2-digit',hourCycle:'h23',timeZoneName:'longOffset'});
 if(part(f,row[1],'timeZoneName')!==row[2])throw row.join(':');
 if(f.formatToParts(row[1]).map(p=>p.value).join('')!==f.format(row[1]))throw 'parts '+row[0];
}
print(true);
"#,
    );
}

#[test]
fn instant_nanosecond_neighbors_select_the_correct_transition() {
    assert_zone_script(
        r#"
function name(f,ns){for(var p of f.formatToParts(new Temporal.Instant(ns)))if(p.type==='timeZoneName')return p.value;throw 'name';}
for(var row of [
 ['Europe/Paris',-1855958961000000000n,'GMT+00:09:21','GMT'],
 ['America/New_York',1772953200000000000n,'GMT-05:00','GMT-04:00']
]){
 var f=new Intl.DateTimeFormat('en',{timeZone:row[0],timeZoneName:'longOffset'});
 if(name(f,row[1]-1n)!==row[2] || name(f,row[1])!==row[3] || name(f,row[1]+1n)!==row[3])throw row[0];
}
print(true);
"#,
    );
}

#[test]
fn named_offsets_apply_to_components_at_the_full_exact_time_domain() {
    assert_zone_script(
        r#"
function fields(f,t){return f.formatToParts(t).filter(p=>p.type!=='literal').map(p=>p.type+':'+p.value).join('|');}
for(var row of [
 ['America/New_York',64076313600000,'-04:00'],
 ['Australia/Lord_Howe',1775314800000,'+10:30'],
 ['Pacific/Apia',1325239200000,'+14:00']
]){
 var options={timeZone:row[0],year:'numeric',month:'2-digit',day:'2-digit',hour:'2-digit',minute:'2-digit',second:'2-digit',hourCycle:'h23'};
 var named=new Intl.DateTimeFormat('en',options);
 options.timeZone=row[2];
 if(fields(named,row[1])!==fields(new Intl.DateTimeFormat('en',options),row[1]))throw row[0];
}
var f=new Intl.DateTimeFormat('en',{timeZone:'Europe/Paris',timeZoneName:'longOffset'});
var initialName='';for(var p of f.formatToParts(-8640000000000000))if(p.type==='timeZoneName')initialName=p.value;
if(initialName!=='GMT+00:09:21')throw 'initial record';
for(var zone of ['America/New_York','Europe/Paris','Pacific/Apia']) {
 var edge=new Intl.DateTimeFormat('en',{timeZone:zone,year:'numeric',timeZoneName:'longOffset'});
 if(typeof edge.format(8640000000000000)!=='string')throw 'positive limit';
}
print(true);
"#,
    );
}

#[test]
fn range_patterns_own_zone_names_and_repeated_time_collapses() {
    assert_zone_script(
        r#"
var f=new Intl.DateTimeFormat('en-US',{timeZone:'America/New_York',hour:'numeric',minute:'2-digit',hourCycle:'h23',timeZoneName:'short'});
var start=1772953199000,end=1772953200000,parts=f.formatRangeToParts(start,end);
var names=parts.filter(p=>p.type==='timeZoneName').map(p=>p.source+':'+p.value).join('|');
if(names!=='shared:ET')throw names;
function zoneName(formatter,epoch){for(var part of formatter.formatToParts(epoch))if(part.type==='timeZoneName')return part.value;throw 'missing scalar zone';}
if(zoneName(f,start)!=='EST'||zoneName(f,end)!=='EDT')throw 'scalar endpoint snapshots';
var fallback=new Intl.DateTimeFormat('en-US',{timeZone:'America/New_York',hour:'numeric',minute:'2-digit',second:'2-digit',hourCycle:'h23',timeZoneName:'short'});
var fallbackParts=fallback.formatRangeToParts(start,end);
var fallbackNames=fallbackParts.filter(p=>p.type==='timeZoneName').map(p=>p.source+':'+p.value).join('|');
if(fallbackNames!=='startRange:EST|endRange:EDT')throw fallbackNames;
if(fallbackParts.map(p=>p.value).join('')!==fallback.formatRange(start,end))throw 'fallback range parts';
if(parts.map(p=>p.value).join('')!==f.formatRange(start,end))throw 'range parts';
var early=1793511000000,late=1793514600000;
if(f.formatRange(early,late)!==f.format(early))throw 'repeated local time collapse';
for(var p of f.formatRangeToParts(early,late))if(p.source!=='shared')throw 'collapsed attribution';
print(true);
"#,
    );
}

#[test]
fn plain_temporal_values_bypass_named_zone_gaps_and_overlaps() {
    assert_zone_script(
        r#"
var values=[
 new Temporal.PlainDateTime(2026,3,8,2,30),
 new Temporal.PlainDateTime(2026,11,1,1,30),
 new Temporal.PlainDate(2026,3,8),
 new Temporal.PlainTime(2,30),
 new Temporal.PlainYearMonth(2026,3),
 new Temporal.PlainMonthDay(3,8)
];
for(var value of values){
 var utc=new Intl.DateTimeFormat('en',{calendar:'iso8601',timeZone:'UTC'});
 for(var zone of ['America/New_York','Australia/Lord_Howe','Pacific/Apia']){
  var named=new Intl.DateTimeFormat('en',{calendar:'iso8601',timeZone:zone});
  if(named.format(value)!==utc.format(value))throw zone;
  if(named.formatRange(value,value)!==utc.format(value))throw 'plain range';
 }
}
var gregory=new Intl.DateTimeFormat('en',{calendar:'gregory',timeZone:'America/New_York'});
for(var value of [values[4],values[5]]){
 for(var method of ['format','formatToParts','formatRange','formatRangeToParts']){
  var caught=false;
  try { gregory[method](value,value); } catch(error) { if(!(error instanceof RangeError))throw error; caught=true; }
  if(!caught)throw 'incompatible partial-date calendar '+method;
 }
}
print(true);
"#,
    );
}

#[test]
fn six_styles_distinguish_named_utc_and_fixed_offsets_and_localize_digits_once() {
    assert_zone_script(
        r#"
function name(zone,style,locale){var f=new Intl.DateTimeFormat(locale,{timeZone:zone,timeZoneName:style});for(var p of f.formatToParts(0))if(p.type==='timeZoneName')return p.value;throw 'name';}
var styles=['short','long','shortOffset','longOffset','shortGeneric','longGeneric'];
for(var style of styles){
 var long=style==='long'||style==='longOffset'||style==='longGeneric';
 if(name('+23:59',style,'en')!=='GMT+23:59')throw 'positive '+style;
 if(name('-23:59',style,'en')!=='GMT-23:59')throw 'negative '+style;
 if(name('+00:00',style,'en')!=='GMT')throw 'fixed zero '+style;
 if(name('+05:30',style,'en-u-nu-arab')!==(long?'GMT+٠٥:٣٠':'GMT+٥:٣٠'))throw 'digits '+style;
}
if(name('UTC','short','en')!=='UTC'||name('UTC','long','en')!=='Coordinated Universal Time')throw 'named UTC';
var fallback=new Intl.DateTimeFormat('zxx',{timeZone:'Asia/Tokyo',timeZoneName:'long'});
if(fallback.resolvedOptions().locale!=='en-US')throw 'explicit locale fallback';
print(true);
"#,
    );
}

#[test]
fn date_only_entry_points_root_named_zone_host_calls() {
    assert_zone_script(
        r#"
var epoch=new Date(0);
if(epoch.toLocaleDateString('en',{timeZone:'America/New_York',year:'numeric'})!=='1969')throw 'date';
if(epoch.toLocaleString('en',{timeZone:'America/New_York',year:'numeric'})!=='1969')throw 'combined';
if(epoch.toLocaleTimeString('en',{timeZone:'America/New_York',hour:'numeric',hourCycle:'h23'})!=='19')throw 'time';
print(true);
"#,
    );
}
