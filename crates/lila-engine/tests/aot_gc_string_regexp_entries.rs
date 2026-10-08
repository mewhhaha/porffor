use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_strings(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let source = format!("var $262={{createRealm:__lilaCreateRealm}};function assert(v,m) {{ if (!v) throw new Error(m); }}\n{source}");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            &source,
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
        .expect("String/RegExp control executes through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}\n{source}",
        observed.completion
    );
}

#[test]
fn string_constructor_converts_before_new_target_and_preserves_box_brand_and_realm() {
    assert_strings(
        r#"
var order=[], value={toString:function(){order.push('value');return '\ud800';}};
function Target() {}
var target=new Proxy(Target,{get:function(t,k,r){if(k==='prototype')order.push('prototype');return Reflect.get(t,k,r);}});
var boxed=Reflect.construct(String,[value],target);
assert(order.join(',')==='value,prototype' && Object.getPrototypeOf(boxed)===Target.prototype,'conversion before selected prototype');
assert(String.prototype.valueOf.call(boxed).charCodeAt(0)===0xd800,'whole completed string box');
assert(String() === '' && new String().length===0,'omitted constructor argument');
assert(String(Symbol('x'))==='Symbol(x)','call Symbol exception');
try {new String(Symbol());throw new Error('missing Symbol conversion failure');}
catch(error){assert(error instanceof TypeError,'construct has ordinary ToString');}
try {String.prototype.valueOf.call(new Proxy(new String('x'),{}));throw new Error('Proxy brand');}
catch(error){assert(error instanceof TypeError,'Proxy does not forward StringData');}
var other=__lilaCreateRealm().global;
try {other.String.prototype.toString.call({});throw new Error('borrowed brand');}
catch(error){assert(error instanceof other.TypeError && !(error instanceof TypeError),'borrowed builtin error Realm');}
assert(Object.getPrototypeOf(new other.String('x'))===other.String.prototype,'defining String prototype');
"#,
    );
}

#[test]
fn string_numeric_constructors_use_exact_residue_and_utf16_units() {
    assert_strings(
        r#"
var value=String.fromCharCode(2**60+256,-1,Infinity,NaN,0xd800,0xdc00);
assert(value.length===6 && value.charCodeAt(0)===256 && value.charCodeAt(1)===65535,'exact ToUint16 residues');
assert(value.charCodeAt(2)===0 && value.charCodeAt(3)===0,'nonfinite residues');
assert(value.charCodeAt(4)===0xd800 && value.charCodeAt(5)===0xdc00,'separate surrogate arguments');
assert(String.fromCodePoint(0x1f600,0xd800).length===3 && String.fromCodePoint(0x1f600).codePointAt(0)===0x1f600,'astral and lone surrogate materialization');
var bad=[NaN,Infinity,-1,0x110000,0.5];
for(var i=0;i<bad.length;i++){try{String.fromCodePoint(bad[i]);throw new Error('invalid point');}catch(error){assert(error instanceof RangeError,'point validation');}}
var calls=[], token={};
try {String.fromCharCode({valueOf:function(){calls.push(1);return 65;}},{valueOf:function(){calls.push(2);throw token;}},{valueOf:function(){calls.push(3);return 67;}});throw new Error('missing conversion Throw');}
catch(error){assert(error===token && calls.join(',')==='1,2','ordered whole argument coercions');}
"#,
    );
}

#[test]
fn character_ranges_and_well_formedness_keep_exact_surrogates_and_throw_values() {
    assert_strings(
        r#"
var text='A\ud83d\ude00\ud800Z';
assert(text.length===5 && text.charAt(1).charCodeAt(0)===0xd83d && text.at(-2).charCodeAt(0)===0xd800,'single code-unit access');
assert(text.codePointAt(1)===0x1f600 && text.codePointAt(2)===0xde00,'code-point access starts at exact unit');
assert(text.charAt(Infinity)==='' && text.at(-Infinity)===undefined && text.codePointAt(-1)===undefined && Number.isNaN(text.charCodeAt(100)),'out-of-range distinctions');
assert(text.slice(1,2).charCodeAt(0)===0xd83d && text.substring(3,1)==='\ud83d\ude00','range slicing and swapping');
assert(text.substr(-2,1)==='\ud800' && text.slice(-Infinity,Infinity)===text && text.substring(NaN,1)==='A','range normalization');
assert(!text.isWellFormed() && text.toWellFormed()==='A\ud83d\ude00\ufffdZ','repair replaces only lone surrogate');
assert('\udc00\ud800'.toWellFormed()==='\ufffd\ufffd' && '\ud800\udc00'.isWellFormed(),'reversed and valid pair boundaries');
var order=[], token={}, receiver={toString:function(){order.push('receiver');return text;}};
try {String.prototype.charAt.call(receiver,{valueOf:function(){order.push('index');throw token;}});throw new Error('index Throw');}
catch(error){assert(error===token && order.join(',')==='receiver,index','receiver then index whole Throw');}
"#,
    );
}

#[test]
fn string_search_observes_is_regexp_and_position_in_order() {
    assert_strings(
        r#"
var order=[], receiver={toString:function(){order.push('receiver');return 'ababa';}};
var needle={get [Symbol.match](){order.push('match');return false;},toString:function(){order.push('needle');return 'ba';}};
var position={valueOf:function(){order.push('position');return 1.9;}};
assert(String.prototype.includes.call(receiver,needle,position),'ordered search result');
assert(order.join(',')==='receiver,match,needle,position','IsRegExp before search string and position');
assert('ababa'.indexOf('ba',1.9)===1 && 'ababa'.lastIndexOf('ba',NaN)===3 && 'ababa'.lastIndexOf('',undefined)===5,'search cursor normalization');
assert('ababa'.endsWith('ba',3) && 'ababa'.startsWith('ba',1) && !'ababa'.includes('bb'),'anchored and scanning search');
assert('\ud83d\ude00'.indexOf('\ude00')===1 && '\ud800'.includes('\ud800'),'UTF16 search domain');
var touched=0;
try {'x'.startsWith(/x/,{valueOf:function(){touched++;return 0;}});throw new Error('regexp rejection');}
catch(error){assert(error instanceof TypeError && touched===0,'RegExp rejected before position conversion');}
"#,
    );
}

#[test]
fn string_raw_interleaves_literal_and_substitution_conversion_and_skips_unused_arguments() {
    assert_strings(
        r#"
var order=[];
var raw={get length(){order.push('length');return {valueOf:function(){order.push('count');return 2;}};},
    get 0(){order.push('zero');return {toString:function(){order.push('literal0');return '\ud800';}};},
    get 1(){order.push('one');return {toString:function(){order.push('literal1');return 'Z';}};}};
var template={get raw(){order.push('raw');return raw;}};
var substitution={toString:function(){order.push('substitution');return 'X';}};
var result=String.raw(template,substitution,{toString:function(){throw new Error('unused substitution');}});
assert(result==='\ud800XZ' && order.join(',')==='raw,length,count,zero,literal0,substitution,one,literal1','complete Raw ordering');
assert(String.raw({raw:{length:0,get 0(){throw new Error('zero count');}}},Symbol())==='','zero count skips all conversions');
assert(String.raw({raw:{length:3,0:'a',1:'b',2:'c'}})==='abc','missing substitutions are empty');
var token={}, late=0;
try {String.raw({raw:{length:2,0:'a',get 1(){late++;return 'b';}}},{toString:function(){throw token;}});throw new Error('raw Throw');}
catch(error){assert(error===token && late===0,'substitution Throw precedes next literal Get');}
"#,
    );
}

#[test]
fn padding_repeat_concat_and_trim_preserve_conversion_and_extent_boundaries() {
    assert_strings(
        r#"
assert(''.repeat(2**80)==='' && 'ab'.repeat(3)==='ababab' && 'x'.repeat(NaN)==='','finite count and empty receiver');
try {'x'.repeat(Infinity);throw new Error('repeat infinite');}catch(error){assert(error instanceof RangeError,'infinite repeat');}
assert('x'.padEnd(2**40,'')==='x','empty filler precedes result extent limit');
assert('X'.padStart(2,'\ud83d\ude00')==='\ud83dX' && 'X'.padEnd(2,'\ud83d\ude00')==='X\ud83d','padding truncates exact units');
var order=[], receiver={toString:function(){order.push('receiver');return 'x';}};
var length={valueOf:function(){order.push('length');return 3;}}, filler={toString:function(){order.push('filler');return 'ab';}};
assert(String.prototype.padStart.call(receiver,length,filler)==='abx' && order.join(',')==='receiver,length,filler','pad ordering');
assert('x'.padStart(1,Symbol())==='x','already-long receiver skips filler');
assert('\ufeff \ud800 \u2028'.trim()==='\ud800' && ' x '.trimStart()==='x ' && ' x '.trimEnd()===' x','ECMAScript trim boundaries');
var token={}, calls=[];
try {'x'.concat({toString:function(){calls.push(1);return '\ud800';}},{toString:function(){calls.push(2);throw token;}},{toString:function(){calls.push(3);return 'z';}});throw new Error('concat Throw');}
catch(error){assert(error===token && calls.join(',')==='1,2','whole ordered concat Throw');}
"#,
    );
}

#[test]
fn string_symbol_hooks_keep_original_receiver_and_regexp_create_policy() {
    assert_strings(
        r#"
var token={}, receiver={toString:function(){throw token;}}, pattern={};
pattern[Symbol.match]=function(value){assert(this===pattern && value===receiver,'original hook values');return 71;};
assert(String.prototype.match.call(receiver,pattern)===71,'hook precedes receiver ToString');
pattern[Symbol.match]=17;
try {String.prototype.match.call(receiver,pattern);throw new Error('noncallable hook');}
catch(error){assert(error instanceof TypeError && error!==token,'GetMethod before receiver conversion');}
var re=/a/; re[Symbol.match]=undefined; re.toString=function(){return 'b';};
assert('b'.match(re)[0]==='b' && 'a'.match(re)===null,'fallback RegExpCreate converts the original pattern');
var calls=[], all={};
Object.defineProperty(all,Symbol.match,{get:function(){calls.push('isregexp');return true;}});
Object.defineProperty(all,'flags',{get:function(){calls.push('flags');return {toString:function(){calls.push('flags-string');return 'g';}};}});
Object.defineProperty(all,Symbol.matchAll,{get:function(){calls.push('hook');return function(value){calls.push('call');return value;};}});
assert(String.prototype.matchAll.call(receiver,all)===receiver && calls.join(',')==='isregexp,flags,flags-string,hook,call','matchAll observable order');
var missingG={}; missingG[Symbol.match]=true; missingG.flags='';
Object.defineProperty(missingG,Symbol.replace,{get:function(){throw token;}});
try {'a'.replaceAll(missingG,'b');throw new Error('missing global flag');}
catch(error){assert(error instanceof TypeError,'global validation precedes replace hook');}
"#,
    );
}

#[test]
fn regexp_construction_compile_and_accessors_publish_complete_slots() {
    assert_strings(
        r#"
var order=[], pattern={};
pattern[Symbol.match]=true;
Object.defineProperty(pattern,'source',{get:function(){order.push('source');return {toString:function(){order.push('source-string');return 'b';}};}});
Object.defineProperty(pattern,'flags',{get:function(){order.push('flags');return {toString:function(){order.push('flags-string');return 'g';}};}});
var target=new Proxy(function(){},{get:function(object,key){if(key==='prototype'){order.push('prototype');return RegExp.prototype;}return object[key];}});
var created=Reflect.construct(RegExp,[pattern],target);
assert(order.join(',')==='source,flags,prototype,source-string,flags-string' && created.exec('b')[0]==='b','RegExpAlloc before string coercions');
assert(RegExp(created)===created && new RegExp(created)!==created,'call early return versus construction');
class Derived extends RegExp {}
var derived=new Derived('a'); assert(derived.compile('b')===derived && derived.test('b'),'compile accepts branded subclass');
var compiled=/a/;
try {compiled.compile('[');throw new Error('invalid pattern');}catch(error){assert(error instanceof SyntaxError,'syntax error');}
assert(compiled.source==='a' && compiled.test('a'),'failed compile retains old slots');
Object.defineProperty(compiled,'lastIndex',{writable:false});
try {compiled.compile('b','g');throw new Error('readonly index');}catch(error){assert(error instanceof TypeError,'strict Set');}
assert(compiled.source==='b' && compiled.flags==='g','successful compile slots precede readonly Set failure');
assert(new RegExp('').source==='(?:)' && new RegExp('/\n\u2028').source==='\\/\\n\\u2028','source escaping');
var flagsGetter=Object.getOwnPropertyDescriptor(RegExp.prototype,'flags').get, generic={}, keys=[];
['hasIndices','global','ignoreCase','multiline','dotAll','unicode','unicodeSets','sticky'].forEach(function(key){
Object.defineProperty(generic,key,{get:function(){keys.push(key);return true;}});
});
assert(flagsGetter.call(generic)==='dgimsuvy' && keys.join(',')==='hasIndices,global,ignoreCase,multiline,dotAll,unicode,unicodeSets,sticky','generic flags order');
"#,
    );
}

#[test]
fn regexp_builtin_exec_reads_recompiled_slots_after_last_index_conversion() {
    assert_strings(
        r#"
var re=/a/g, conversions=0;
re.lastIndex={valueOf:function(){conversions++;re.compile('b','');return 99;}};
var match=re.exec('b');
assert(conversions===1 && match[0]==='b' && match.index===0 && re.flags==='','fresh slots after coercion');
var token={}, plain=/x/;
plain.lastIndex={valueOf:function(){throw token;}};
try {plain.exec('x');throw new Error('lastIndex coercion');}catch(error){assert(error===token,'nonglobal coercion retains whole Throw');}
var readonly=/x/; Object.defineProperty(readonly,'lastIndex',{value:9,writable:false});
assert(readonly.exec('x')[0]==='x' && readonly.exec('y')===null && readonly.lastIndex===9,'nonglobal never writes index');
var global=/x/g; Object.defineProperty(global,'lastIndex',{writable:false});
try {global.exec('x');throw new Error('lastIndex success Set');}catch(error){assert(error instanceof TypeError,'global strict Set failure');}
var indexed=/(?<pair>\ud83d\ude00)(?<missing>x)?/du.exec('q\ud83d\ude00');
assert(indexed.index===1 && indexed[0]==='\ud83d\ude00' && indexed[1]==='\ud83d\ude00' && indexed[2]===undefined,'UTF-16 captures');
assert(indexed.groups.pair===indexed[1] && indexed.groups.missing===undefined && Object.getPrototypeOf(indexed.groups)===null,'named groups');
assert(indexed.indices[0][0]===1 && indexed.indices[0][1]===3 && indexed.indices.groups.pair===indexed.indices[1] && indexed.indices.groups.missing===undefined,'indices pair identity');
$262.gc();
assert(indexed[1].charCodeAt(1)===0xde00 && indexed.indices.groups.pair[1]===3,'semantic roots survive GC');
"#,
    );
}

#[test]
fn regexp_exec_protocol_gets_once_and_retains_custom_call_identity() {
    assert_strings(
        r#"
var gets=0,calls=0, regexp={};
Object.defineProperty(regexp,'exec',{get:function(){gets++;return new Proxy(function(input){calls++;assert(this===regexp && input==='abc','custom Call values');return {};},{apply:function(target,receiver,args){return Reflect.apply(target,receiver,args);}});}});
assert(RegExp.prototype.test.call(regexp,{toString:function(){return 'abc';}})===true && gets===1 && calls===1,'one acquired exec');
regexp={exec:function(){return 7;}};
try {RegExp.prototype.test.call(regexp,'x');throw new Error('invalid custom result');}catch(error){assert(error instanceof TypeError,'object or null result required');}
var branded=/x/; branded.exec=null; assert(RegExp.prototype.test.call(branded,'x'),'noncallable exec uses intrinsic matcher');
var token={}, argument={toString:function(){throw token;}};
try {RegExp.prototype.exec.call({},argument);throw new Error('brand first');}catch(error){assert(error instanceof TypeError && error!==token,'intrinsic brand before argument coercion');}
try {RegExp.prototype.test.call(null,argument);throw new Error('object first');}catch(error){assert(error instanceof TypeError && error!==token,'test object admission before argument coercion');}
"#,
    );
}

#[test]
fn regexp_match_all_species_clones_before_last_index_and_iterator_exec() {
    assert_strings(
        r#"
var order=[], original={lastIndex:{valueOf:function(){order.push('index-number');return 2;}}}, clone;
Object.defineProperty(original,'constructor',{get:function(){order.push('constructor');var object={};Object.defineProperty(object,Symbol.species,{get:function(){order.push('species');return function(pattern,flags){order.push('construct');assert(pattern===original && flags==='gu','species args');clone={lastIndex:0,exec:function(input){order.push('exec');assert(input==='abc' && this===clone && clone.lastIndex===2,'iterator owns clone');return null;}};return clone;};}});return object;}});
Object.defineProperty(original,'flags',{get:function(){order.push('flags');return 'gu';}});
var iterator=RegExp.prototype[Symbol.matchAll].call(original,'abc');
assert(order.join(',')==='constructor,species,flags,construct,index-number' && clone.lastIndex===2,'clone then original lastIndex');
assert(iterator.next().done && iterator.next().done && order.join(',')==='constructor,species,flags,construct,index-number,exec','done stops repeated exec');
var matches='a\ud83d\ude00'.matchAll(/(?:)/gu), lengths=[];
for (var result of matches) lengths.push(result.index);
assert(lengths.join(',')==='0,1,3','full-Unicode empty match advancement');
"#,
    );
}

#[test]
fn regexp_and_literal_replacement_preserve_capture_and_callback_order() {
    assert_strings(
        r#"
var template="$$-$&-$'-$"+String.fromCharCode(96);
assert('abc'.replace('b',template)==='a$-b-c-ac','literal substitution tokens');
assert('aaa'.replaceAll('aa','x')==='xa' && 'ab'.replaceAll('','-')==='-a-b-','nonoverlapping and empty matches');
var order=[], next=0, regexp={flags:'g',lastIndex:0,exec:function(){order.push('exec'+next);return next++<2?{0:'a',length:2,1:undefined,index:next-1,groups:undefined}:null;}};
var replaced=RegExp.prototype[Symbol.replace].call(regexp,'aa',function(match,capture,position,input){
'use strict';order.push('replace'+position);assert(this===undefined && match==='a' && capture===undefined && input==='aa','functional arguments');return 'x';
});
assert(replaced==='xx' && order.join(',')==='exec0,exec1,exec2,replace0,replace1','collect results before replacers');
assert('a'.replace(/(?<name>a)(b)?/,'$<name>:$1:$2:$01:$0:$<missing>')==='a:a::a:$0:','capture substitutions');
var token={}, named={};Object.defineProperty(named,'name',{get:function(){throw token;}});
regexp={flags:'',exec:function(){return {0:'a',length:1,index:0,groups:named};}};
try {RegExp.prototype[Symbol.replace].call(regexp,'a','$<name>');throw new Error('named getter Throw');}catch(error){assert(error===token,'named getter retains whole Throw');}
"#,
    );
}

#[test]
fn regexp_and_literal_split_preserve_units_captures_and_limit_order() {
    assert_strings(
        r#"
var units='\ud83d\ude00'.split(''); assert(units.length===2 && units[0]==='\ud83d' && units[1]==='\ude00','literal split is code units');
assert('\ud83d\ude00'.split(/(?:)/u).length===1 && '\ud83d\ude00'.split(/(?:)/).length===2,'RegExp Unicode cursor');
var parts='a-b'.split(/(-)(x)?/);assert(parts.length===4 && parts[0]==='a' && parts[1]==='-' && parts[2]===undefined && parts[3]==='b','raw capture insertion');
var order=[], separator={toString:function(){order.push('separator');return '-';}}, limit={valueOf:function(){order.push('limit');return 0;}};
assert('a-b'.split(separator,limit).length===0 && order.join(',')==='limit,separator','separator converts even for zero limit');
var regexp={flags:'',constructor:{}};
regexp.constructor[Symbol.species]=function(pattern,flags){order.push('construct');assert(pattern===regexp && flags==='y','sticky species');return {exec:function(){throw new Error('zero limit executes');}};};
order=[];assert(RegExp.prototype[Symbol.split].call(regexp,'ab',limit).length===0 && order.join(',')==='construct,limit','construct precedes split limit');
assert('a--'.split('-',2).join(',')==='a,' && ''.split('').length===0 && ''.split('-').length===1,'tail and empty String distinctions');
"#,
    );
}

#[test]
fn regexp_search_restores_the_whole_previous_index_before_result_index_get() {
    assert_strings(
        r#"
var previous={}, regexp={lastIndex:previous,exec:function(input){assert(this.lastIndex===0 && input==='x','search starts at +0');this.lastIndex=4;return {get index(){assert(regexp.lastIndex===previous,'restore precedes index Get');return Symbol.for('position');}};}};
assert(RegExp.prototype[Symbol.search].call(regexp,'x')===Symbol.for('position') && regexp.lastIndex===previous,'raw previous value restored');
var sets=0,current=-0;
regexp={exec:function(){return null;}};
Object.defineProperty(regexp,'lastIndex',{get:function(){return current;},set:function(value){sets++;current=value;}});
assert(RegExp.prototype[Symbol.search].call(regexp,'x')===-1 && sets===2 && Object.is(current,-0),'-0 differs from +0');
var token={};regexp={lastIndex:previous,exec:function(){this.lastIndex=6;throw token;}};
try {RegExp.prototype[Symbol.search].call(regexp,'x');throw new Error('custom exec Throw');}catch(error){assert(error===token && regexp.lastIndex===6,'abrupt exec exits before restoration');}
"#,
    );
}
