use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};
fn assert_lists(source: &str, policy: HostSurfacePolicy) {
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
            .expect("Locale list source must compile and execute through Wasm AOT");
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
fn locale_calendars_preserves_pinned_preference_order_and_actual_availability() {
    assert_lists(
        r#"
var rows=[['th-TH','buddhist,gregory'],['fa-IR','persian,gregory,islamic-civil,islamic-tbla'],['en-SA','gregory,islamic-umalqura'],['en-CN','gregory,chinese'],['en-XY','gregory'],['und-001','gregory']];
for(var row of rows) { var locale=new Intl.Locale(row[0]), before=locale.toString(); if(locale.getCalendars().join(',')!==row[1]) throw row[0]; if(locale.toString()!==before) throw 'mutation'; }
for(var name of new Intl.Locale('fa-IR').getCalendars()) if(!Intl.supportedValuesOf('calendar').includes(name)) throw 'unavailable calendar';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
#[test]
fn locale_calendars_region_preference_keeps_override_subdivision_and_base_rules() {
    assert_lists(
        r#"
var rows=[['en-US-u-rg-thzzzz','buddhist,gregory'],['en-u-sd-th10','buddhist,gregory'],['en-US-u-sd-th10','gregory'],['th-TH-u-rg-aqzzzz','gregory'],['th-TH-u-rg-xyzzzz','buddhist,gregory'],['en-US-u-rg-thzzzz-abc','gregory'],['th','buddhist,gregory'],['en-US-x-rg-thzzzz','gregory']];
for(var row of rows) if(new Intl.Locale(row[0]).getCalendars().join(',')!==row[1]) throw row[0];
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
#[test]
fn locale_collations_uses_matched_sort_profile_without_a_global_union() {
    assert_lists(
        r#"
var de=new Intl.Locale('de').getCollations(), en=new Intl.Locale('en').getCollations();
if(!de.includes('phonebk') || en.includes('phonebk')) throw 'locale profile';
if(new Intl.Locale('de-DE-fonipa').getCollations().join(',')!==de.join(',')) throw 'prefix';
if(new Intl.Locale('de-u-rg-uszzzz-sd-usca').getCollations().join(',')!==de.join(',')) throw 'irrelevant region';
if(new Intl.Locale('qaa').getCollations().join(',')!=='emoji,eor') throw 'unmatched';
for(var names of [de,en,new Intl.Locale('ko').getCollations()]) { if(names.join(',')!==names.slice().sort().join(',')) throw 'order'; for(var name of names) if(['standard','search','searchjl'].includes(name)) throw 'search purpose'; }
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
#[test]
fn locale_calendars_and_collations_preserve_complete_open_explicit_slots() {
    assert_lists(
        r#"
for(var row of [['ca','calendar','getCalendars','gregory'],['co','collation','getCollations','phonebk']]) {
 for(var value of ['foobar','gregory-abc','','true']) {
  var tag='en-US-u-'+row[0]+(value?'-'+value:''), locale=new Intl.Locale(tag), expected=value==='true'?'':value;
  var result=locale[row[2]](); if(result.length!==1 || !result.hasOwnProperty('0') || result[0]!==expected || locale[row[1]]!==expected) throw 'open slot';
  var options={};options[row[1]]=row[3]; if(new Intl.Locale(tag,options)[row[2]]()[0]!==row[3]) throw 'option precedence';
 }
 for(var invalid of ['', 'xy', 'a/b']) { var options={};options[row[1]]=invalid;var caught=false;try{new Intl.Locale('en',options);}catch(e){caught=e instanceof RangeError;}if(!caught)throw 'option syntax'; }
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
#[test]
fn locale_time_zones_uses_only_explicit_base_region_and_retains_optional_result() {
    assert_lists(
        r#"
for(var tag of ['en','abcde','abcdefgh-u-rg-jpzzzz','en-u-rg-jpzzzz','en-u-sd-jp13','en-t-ja-jp','en-x-jp']) { var value=new Intl.Locale(tag).getTimeZones(); if(value!==undefined || String(value)!=='undefined') throw 'missing base region'; }
for(var tag of ['en-XY','en-001']) { var value=new Intl.Locale(tag).getTimeZones();if(!Array.isArray(value)||value.length!==0)throw 'empty present region'; }
if(new Intl.Locale('abcde-JP').getTimeZones().join(',')!=='Asia/Tokyo')throw 'reserved language';
if(new Intl.Locale('en-JP-u-rg-uszzzz').getTimeZones().join(',')!=='Asia/Tokyo') throw 'explicit JP';
var us=new Intl.Locale('en-US-u-rg-jpzzzz').getTimeZones();if(!us.includes('America/New_York')||us.includes('Asia/Tokyo'))throw 'base US';
if(new Intl.Locale('de-DE').getTimeZones().join(',')!=='Europe/Berlin,Europe/Busingen')throw 'primary country exception';
if(!new Intl.Locale('en-IN').getTimeZones().includes('Asia/Kolkata')||new Intl.Locale('en-IN').getTimeZones().includes('Asia/Calcutta'))throw 'alias canonicalization';
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
#[test]
fn locale_information_methods_have_standard_descriptors_and_fresh_dense_arrays() {
    assert_lists(
        r#"
for(var name of ['getCalendars','getCollations','getTimeZones']) {
 var method=Intl.Locale.prototype[name],d=Object.getOwnPropertyDescriptor(Intl.Locale.prototype,name);
 if(method.name!==name||method.length!==0||method.hasOwnProperty('prototype')||!d.writable||d.enumerable||!d.configurable||d.value!==method)throw 'metadata';
 var caught=false;try{new method();}catch(e){caught=e instanceof TypeError;}if(!caught)throw 'constructability';
 var locale=new Intl.Locale('th-TH'),a=method.call(locale),b=method.call(locale);if(a===b||!Array.isArray(a)||Object.getPrototypeOf(a)!==Array.prototype)throw 'fresh array';
 for(var i=0;i<a.length;i++){var entry=Object.getOwnPropertyDescriptor(a,String(i));if(!entry.writable||!entry.enumerable||!entry.configurable||!('value'in entry))throw 'dense descriptor';}
 var length=Object.getOwnPropertyDescriptor(a,'length');if(!length.writable||length.enumerable||length.configurable)throw 'length';
 var original=b.join(',');a.length=0;if(method.call(locale).join(',')!==original)throw 'alias';
}
for(var name of ['getCalendars','getCollations']) { var locale=new Intl.Locale(name==='getCalendars'?'en-u-ca':'en-u-co-foobar'),a=locale[name](),b=locale[name]();a[0]='changed';if(a===b||b[0]!==locale[name]()[0])throw 'explicit freshness'; }
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
#[test]
fn locale_information_brand_checks_precede_receiver_and_argument_observation() {
    assert_lists(
        r#"
for(var name of ['getCalendars','getCollations','getTimeZones']) {
 var method=Intl.Locale.prototype[name];for(var value of [undefined,null,true,1,'en',Symbol(),{},Intl.Locale.prototype,Object.create(new Intl.Locale('en')),new Proxy(new Intl.Locale('en'),{})]) {var caught=false;try{method.call(value);}catch(e){caught=e instanceof TypeError;}if(!caught)throw 'brand';}
 var locale=new Intl.Locale('en-JP');for(var key of ['toString','calendar','collation','language','region',Symbol.toPrimitive])Object.defineProperty(locale,key,{get(){throw 'observable receiver';}});Object.setPrototypeOf(locale,null);
 if(!Array.isArray(method.call(locale,{toString(){throw 'argument coercion';}})))throw 'slots';
}
print('ok');
"#,
        HostSurfacePolicy::default(),
    );
}
#[test]
fn locale_information_allocates_arrays_and_errors_in_defining_realm() {
    assert_lists(
        r#"
var foreign=__lilaCreateRealm().global,remotePrototype=foreign.Array.prototype,captured=[];
for(var name of ['getCalendars','getCollations','getTimeZones']) {
 var remote=foreign.Intl.Locale.prototype[name],local=Intl.Locale.prototype[name],native=new Intl.Locale('en-JP'),other=new foreign.Intl.Locale('en-JP');captured.push(remote);
 if(Object.getPrototypeOf(remote.call(native))!==remotePrototype||Object.getPrototypeOf(local.call(other))!==Array.prototype)throw 'result Realm';
 var caught=false;try{remote.call({});}catch(e){caught=e instanceof foreign.TypeError&&!(e instanceof TypeError);}if(!caught)throw 'error Realm';
 if(name!=='getTimeZones'){var explicit=new Intl.Locale(name==='getCalendars'?'en-u-ca':'en-u-co-foobar');if(Object.getPrototypeOf(remote.call(explicit))!==remotePrototype)throw 'explicit Realm';}
 else if(remote.call(new Intl.Locale('en'))!==undefined)throw 'undefined Realm';
}
foreign.Array=function(){throw 'public Array'};foreign.Intl.Locale=function(){throw 'public Locale'};
for(var method of captured)if(Object.getPrototypeOf(method.call(new Intl.Locale('en-JP')))!==remotePrototype)throw 'public constructor lookup';
print('ok');
"#,
        HostSurfacePolicy::Test262,
    );
}
