use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn assert_date_time_script(source: &str) {
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
        .expect("DateTimeFormat must compile and execute through Wasm AOT");
    assert!(
        matches!(observation.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observation.completion,
    );
    assert_eq!(
        observation.output_events,
        vec![HostOutputEvent::PrintLine("true".to_owned())],
        "{source}"
    );
}

#[test]
fn chinese_year_names_and_related_years_have_distinct_parts() {
    assert_date_time_script(
        r#"
var f = new Intl.DateTimeFormat('zh-u-ca-chinese', {timeZone:'UTC', year:'numeric'});
for (var row of [[1559347200000, '2019', '己亥'], [1590969600000, '2020', '庚子']]) {
  var parts=f.formatToParts(row[0]);
  if (parts.map(p=>p.value).join('')!==f.format(row[0])) throw 'join';
  if (!parts.some(p=>p.type==='yearName' && p.value===row[2])) throw 'cyclic year';
  if (parts.some(p=>p.type==='relatedYear' && p.value!==row[1])) throw 'related year';
  if (parts.some(p=>p.type==='year')) throw 'cyclic year classified as year';
  var output=f.format(row[0]);
  if (output!==row[1]+row[2]+'年' && output!==row[2]+'年') throw output;
}
print(true);
"#,
    );
}

#[test]
fn plain_dates_keep_extreme_iso_days_and_arabic_digits() {
    assert_date_time_script(
        r#"
var f=new Intl.DateTimeFormat('ar-EG',{calendar:'iso8601',timeZone:'UTC',year:'numeric',month:'numeric',day:'numeric'});
if(f.resolvedOptions().numberingSystem!=='arab')throw 'locale default';
for(var row of [[275760,9,13,'١٣'],[-271821,4,19,'١٩']]){
 var date=new Temporal.PlainDate(row[0],row[1],row[2]);
 var parts=f.formatToParts(date);
 if(!parts.some(p=>p.type==='day' && p.value===row[3]))throw 'extreme day';
 if(parts.map(p=>p.value).join('')!==f.format(date))throw 'join';
 if(/[0-9]/.test(f.format(date)))throw 'Latin digit leakage';
}
print(true);
"#,
    );
}

#[test]
fn instant_nanoseconds_and_legacy_time_clip_remain_distinct() {
    assert_date_time_script(
        r#"
var f=new Intl.DateTimeFormat('en-US',{timeZone:'UTC',hourCycle:'h23',hour:'2-digit',minute:'2-digit',second:'2-digit',fractionalSecondDigits:3});
if(f.format(new Temporal.Instant(-1n))!=='23:59:59.999')throw 'negative nanosecond';
if(f.format(new Temporal.Instant(1n))!=='00:00:00.000')throw 'positive nanosecond';
if(f.format(-0.1)!=='00:00:00.000')throw 'TimeClip';
for(var ns of [-8640000000000000000000n,8640000000000000000000n]){
 var parts=f.formatToParts(new Temporal.Instant(ns));
 if(parts.map(p=>p.value).join('')!==f.format(new Temporal.Instant(ns)))throw 'limit';
}
print(true);
"#,
    );
}

#[test]
fn supported_locales_complete_conversion_before_options_and_preserve_request_order() {
    assert_date_time_script(
        r#"
var trace='';
var locales={get length(){trace+='L';return 4;},get 0(){trace+='a';return {toString(){trace+='A';return 'ar-EG-u-nu-latn';}};},get 2(){trace+='b';return {toString(){trace+='B';return 'AR-eg-u-nu-latn';}};},get 3(){trace+='c';return {toString(){trace+='C';return 'zh-u-ca-chinese';}};}};
var result=Intl.DateTimeFormat.supportedLocalesOf(locales,{get localeMatcher(){trace+='M';return 'lookup';}});
if(trace!=='LaAbBcCM')throw trace;
if(result.join('|')!=='ar-EG-u-nu-latn|zh-u-ca-chinese')throw result.join('|');
if(Intl.DateTimeFormat.supportedLocalesOf(['zxx'],{localeMatcher:'lookup'}).length!==0)throw 'unsupported locale';
var marker={},later=0;
try{Intl.DateTimeFormat.supportedLocalesOf({get length(){throw marker;}},{get localeMatcher(){later++;}});throw 'missing throw';}
catch(error){if(error!==marker || later!==0)throw 'abrupt order';}
print(true);
"#,
    );
}

#[test]
fn supported_locales_box_options_in_the_called_function_realm() {
    assert_date_time_script(
        r#"
for(var options of [0,true,'',1n,Symbol('options')]){
 if(Intl.DateTimeFormat.supportedLocalesOf('en',options).join()!=='en')throw 'primitive options';
}
var foreign=__lilaCreateRealm().global;
var calls=0;
Object.defineProperty(foreign.Number.prototype,'localeMatcher',{configurable:true,get(){calls++;return 'lookup';}});
if(foreign.Intl.DateTimeFormat.supportedLocalesOf('en',1).join()!=='en' || calls!==1)throw 'boxed Realm';
try{foreign.Intl.DateTimeFormat.supportedLocalesOf('en',null);throw 'missing null error';}
catch(error){if(error.constructor!==foreign.TypeError)throw 'null error Realm';}
print(true);
"#,
    );
}

#[test]
fn range_coercion_finishes_both_operands_before_time_clip() {
    assert_date_time_script(
        r#"
var f=new Intl.DateTimeFormat('en',{timeZone:'UTC'}),marker={},trace='';
var left={[Symbol.toPrimitive](hint){trace+='L'+hint;return NaN;}};
var right={[Symbol.toPrimitive](hint){trace+='R'+hint;throw marker;}};
for(var method of [f.formatRange,f.formatRangeToParts]){
 trace='';
 try{method.call(f,left,right);throw 'missing throw';}catch(error){if(error!==marker)throw 'lost right completion';}
 if(trace!=='LnumberRnumber')throw trace;
 trace='';
 try{method.call(f,left,undefined);throw 'missing undefined error';}catch(error){if(!(error instanceof TypeError))throw error;}
 if(trace!=='')throw 'coerced before undefined rejection';
}
print(true);
"#,
    );
}

#[test]
fn temporal_kind_and_calendar_errors_use_the_called_function_realm() {
    assert_date_time_script(
        r#"
var foreign=__lilaCreateRealm().global;
var f=new Intl.DateTimeFormat('en',{calendar:'gregory',timeZone:'UTC',year:'numeric',month:'numeric'});
var parts=foreign.Intl.DateTimeFormat.prototype.formatToParts;
for(var value of [new Temporal.PlainYearMonth(2024,1),new Temporal.PlainMonthDay(1,2)]){
 try{parts.call(f,value);throw 'missing calendar error';}catch(error){if(error.constructor!==foreign.RangeError)throw 'calendar error Realm';}
}
for(var method of [foreign.Intl.DateTimeFormat.prototype.formatRange,foreign.Intl.DateTimeFormat.prototype.formatRangeToParts]){
 try{method.call(f,new Temporal.Instant(0n),new Temporal.PlainDate(1970,1,1));throw 'missing type mismatch';}
 catch(error){if(error.constructor!==foreign.TypeError)throw 'kind error Realm';}
}
if(typeof parts.call(f,new Temporal.PlainDate(2024,1,2))[0].value!=='string')throw 'ISO PlainDate compatibility';
print(true);
"#,
    );
}

#[test]
fn result_arrays_and_part_objects_follow_borrowed_method_realms() {
    assert_date_time_script(
        r#"
var foreign=__lilaCreateRealm().global;
var f=new Intl.DateTimeFormat('en',{timeZone:'UTC',year:'numeric'});
for(var parts of [foreign.Intl.DateTimeFormat.prototype.formatToParts.call(f,0),foreign.Intl.DateTimeFormat.prototype.formatRangeToParts.call(f,0,31536000000)]){
 if(Object.getPrototypeOf(parts)!==foreign.Array.prototype)throw 'array Realm';
 for(var part of parts){
  if(Object.getPrototypeOf(part)!==foreign.Object.prototype)throw 'part Realm';
  for(var key of Object.keys(part)){var d=Object.getOwnPropertyDescriptor(part,key);if(!d.writable||!d.enumerable||!d.configurable)throw 'part attributes';}
 }
}
if(Object.getPrototypeOf(foreign.Intl.DateTimeFormat.prototype.resolvedOptions.call(f))!==foreign.Object.prototype)throw 'resolved Realm';
if(Object.getPrototypeOf(foreign.Intl.DateTimeFormat.supportedLocalesOf(['en']))!==foreign.Array.prototype)throw 'supported Realm';
print(true);
"#,
    );
}

#[test]
fn style_resolution_hides_component_fields_and_retains_selected_hour_cycle() {
    assert_date_time_script(
        r#"
var f=new Intl.DateTimeFormat('ar-EG',{timeZone:'UTC',dateStyle:'long',timeStyle:'short',hourCycle:'h23'});
var r=f.resolvedOptions();
if(r.dateStyle!=='long'||r.timeStyle!=='short'||r.hourCycle!=='h23'||r.hour12!==false)throw 'resolved styles';
for(var key of ['weekday','era','year','month','day','dayPeriod','hour','minute','second','fractionalSecondDigits','timeZoneName']){
 if(Object.prototype.hasOwnProperty.call(r,key))throw 'leaked style component '+key;
}
if(f.formatToParts(0).map(p=>p.value).join('')!==f.format(0))throw 'style parts';
print(true);
"#,
    );
}

#[test]
fn ranges_preserve_endpoint_ownership_and_collapse_only_displayed_equality() {
    assert_date_time_script(
        r#"
var f=new Intl.DateTimeFormat('en',{timeZone:'UTC',year:'numeric',month:'short',day:'numeric'});
var start=1704067200000,end=1704153600000;
var parts=f.formatRangeToParts(start,end);
if(parts.map(p=>p.value).join('')!==f.formatRange(start,end))throw 'range join';
if(!parts.some(p=>p.type==='day'&&p.source==='startRange'&&p.value==='1'))throw 'start source';
if(!parts.some(p=>p.type==='day'&&p.source==='endRange'&&p.value==='2'))throw 'end source';
var year=new Intl.DateTimeFormat('en',{timeZone:'UTC',year:'numeric'});
if(year.formatRange(start,end)!==year.format(start))throw 'visible equality';
if(year.formatRangeToParts(start,end).some(p=>p.source!=='shared'))throw 'equal source';
print(true);
"#,
    );
}

#[test]
fn date_and_plain_locale_methods_use_intrinsics_and_their_own_default_fields() {
    assert_date_time_script(
        r#"
var date=new Date(0);
var d=new Temporal.PlainDate(1970,1,1);
var dt=new Temporal.PlainDateTime(1970,1,1,0,0,0);
var time=new Temporal.PlainTime(0,0,0);
var ym=new Temporal.PlainYearMonth(1970,1);
var md=new Temporal.PlainMonthDay(1,1);
Intl.DateTimeFormat=function(){throw 'public constructor used';};
if(date.toLocaleTimeString('en',{timeZone:'UTC',hourCycle:'h23'})!=='00:00:00')throw 'Date time defaults';
if(time.toLocaleString('en',{hourCycle:'h23'})!=='00:00:00')throw 'PlainTime defaults';
if(d.toLocaleString('en',{timeZone:'UTC'})!=='1/1/1970')throw 'PlainDate defaults';
if(!dt.toLocaleString('en',{timeZone:'UTC',hourCycle:'h23'}).includes('00:00:00'))throw 'PlainDateTime time defaults';
if(typeof ym.toLocaleString('en',{calendar:'iso8601'})!=='string')throw 'year-month';
if(typeof md.toLocaleString('en',{calendar:'iso8601'})!=='string')throw 'month-day';
print(true);
"#,
    );
}

#[test]
fn first_endpoint_missing_format_precedes_second_endpoint_calendar_error() {
    assert_date_time_script(
        r#"
var foreign=__lilaCreateRealm().global;
var formatter=new Intl.DateTimeFormat('en',{calendar:'iso8601',hour:'numeric',timeZone:'UTC'});
var left=new Temporal.PlainDate(2024,1,1);
var right=new Temporal.PlainDate(2024,1,2,'gregory');
for(var method of [foreign.Intl.DateTimeFormat.prototype.formatRange,foreign.Intl.DateTimeFormat.prototype.formatRangeToParts]){
 try{method.call(formatter,left,right);throw 'missing availability error';}
 catch(error){if(error.constructor!==foreign.TypeError)throw 'right calendar masked left availability';}
 try{method.call(formatter,right,left);throw 'missing calendar error';}
 catch(error){if(error.constructor!==foreign.RangeError)throw 'left calendar must precede left availability';}
}
print(true);
"#,
    );
}

#[test]
fn supplied_ascii_patterns_keep_numbering_styles_and_parts_consistent() {
    assert_date_time_script(
        r#"
for (var row of [['latn','02:35:06 AM'],['arab','٠٢:٣٥:٠٦ AM'],['deva','०२:३५:०६ AM'],['hanidec','〇二:三五:〇六 AM']]) {
 var f=new Intl.DateTimeFormat('en-US-u-nu-'+row[0],{timeZone:'UTC',hour:'2-digit',minute:'2-digit',second:'2-digit'});
 if(f.format(9306000)!==row[1])throw row[0];
 var parts=f.formatToParts(9306000);
 if(parts.length!==7||parts[5].type!=='literal'||parts[5].value!==' ')throw 'literal';
 if(parts.map(p=>p.value).join('')!==row[1])throw 'join';
}
var styled=new Intl.DateTimeFormat('en-US',{timeZone:'UTC',timeStyle:'medium'});
if(styled.format(9306000)!=='2:35:06 AM')throw 'style';
var ranged=new Intl.DateTimeFormat('en-US',{timeZone:'UTC',hour:'numeric',minute:'numeric'});
var rangeParts=ranged.formatRangeToParts(0,3600000);
if(rangeParts.map(p=>p.value).join('')!==ranged.formatRange(0,3600000))throw 'range join';
if(!rangeParts.some(p=>p.type==='literal'&&p.value.indexOf('\u202f')!==-1))throw 'unprovided interval alternate';
print(true);
"#,
    );
}

#[test]
fn day_period_midnight_and_noon_use_the_context_free_policy() {
    assert_date_time_script(
        r#"
for (var width of ['short','long','narrow']) {
 var f=new Intl.DateTimeFormat('en',{timeZone:'UTC',dayPeriod:width});
 var h=new Intl.DateTimeFormat('en',{timeZone:'UTC',dayPeriod:width,hour:'numeric'});
 var noon=width==='narrow'?'n':'noon';
 for (var row of [[0,'in the morning'],[12,noon],[13,'in the afternoon'],[18,'in the evening'],[21,'at night']]) {
  var date=row[0]*3600000;
  if(f.format(date)!==row[1])throw 'day period '+width+' '+row[0];
  var parts=f.formatToParts(date);
  if(parts.length!==1||parts[0].type!=='dayPeriod'||parts[0].value!==row[1])throw 'part';
  if(h.format(date)!==((row[0]%12)||12)+' '+row[1])throw 'hour';
 }
 if(f.format(new Temporal.Instant(43200000000001n))!=='in the afternoon')throw 'exact noon boundary';
}
print(true);
"#,
    );
}

#[test]
fn era_only_temporal_defaults_keep_numeric_year_month_and_range_parts() {
    assert_date_time_script(
        r#"
var f=new Intl.DateTimeFormat('en',{era:'narrow',timeZone:'UTC'});
for(var value of [new Temporal.PlainDate(2025,11,4),new Temporal.PlainYearMonth(2025,11,'gregory'),new Temporal.PlainMonthDay(11,4,'gregory'),new Temporal.PlainDateTime(2025,11,4,14,46)]) {
 var parts=f.formatToParts(value);
 if(parts[0].type!=='month'||parts[0].value!=='11')throw 'numeric default';
 if(parts.map(p=>p.value).join('')!==f.format(value))throw 'join';
}
var left=new Temporal.PlainYearMonth(2025,11,'gregory');
var right=new Temporal.PlainYearMonth(2025,12,'gregory');
var parts=f.formatToParts(left);
if(parts.some(p=>p.type==='day')||!parts.some(p=>p.type==='era'))throw 'year-month fields';
var range=f.formatRangeToParts(left,right);
if(range.map(p=>p.value).join('')!==f.formatRange(left,right))throw 'range join';
if(!range.some(p=>p.type==='month'&&p.value==='11'&&p.source==='startRange'))throw 'left month';
if(!range.some(p=>p.type==='month'&&p.value==='12'&&p.source==='endRange'))throw 'right month';
if(range.some(p=>p.type==='day'))throw 'range extra day';
print(true);
"#,
    );
}

#[test]
fn chinese_numeric_years_survive_scalar_and_range_pattern_selection() {
    assert_date_time_script(
        r#"
var rows=[[-2208988800000,'1899','12','1'],[946684800000,'1999','11','25'],[4102444800000,'2099','11','21']];
var f=new Intl.DateTimeFormat('en-US-u-ca-chinese',{timeZone:'UTC'});
for(var row of rows){
 var parts=f.formatToParts(row[0]);
 for(var field of [['relatedYear',row[1]],['month',row[2]],['day',row[3]]]){
  if(!parts.some(p=>p.type===field[0]&&(p.value===field[1]||p.value===field[1].padStart(2,'0'))))throw 'scalar '+field[0];
 }
 if(parts.some(p=>p.type==='year'||p.type==='yearName'))throw 'numeric request';
 if(parts.map(p=>p.value).join('')!==f.format(row[0]))throw 'scalar join';
}
for(var locale of ['en-US-u-ca-chinese','zh-u-ca-chinese']){
 for(var style of [undefined,'full']){
  var formatter=new Intl.DateTimeFormat(locale,{timeZone:'UTC',dateStyle:style});
  for(var pair of [[rows[0],rows[1]],[rows[1],rows[0]],[rows[1],rows[2]]]){
   var parts=formatter.formatRangeToParts(pair[0][0],pair[1][0]);
   if(parts.map(p=>p.value).join('')!==formatter.formatRange(pair[0][0],pair[1][0]))throw 'range join';
   if(parts.some(p=>p.type==='year'))throw 'range replaced year family';
   for(var i=0;i<2;i++){
    var source=i===0?'startRange':'endRange';
    if(!parts.some(p=>p.type==='relatedYear'&&p.value===pair[i][1]&&p.source===source))throw 'related endpoint';
    if(style==='full'&&!parts.some(p=>p.type==='yearName'&&p.source===source))throw 'name endpoint';
   }
  }
 }
}
print(true);
"#,
    );
}
