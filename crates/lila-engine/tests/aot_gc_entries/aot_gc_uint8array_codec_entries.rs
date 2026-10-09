use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_codecs(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}var $262={{createRealm:__lilaCreateRealm,detachArrayBuffer:__lilaDetachArrayBuffer}};function assert(v,m){{if(!v)throw new Error(m);}}\n{source}",
            if strict { "\"use strict\";\n" } else { "" },
        );
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
            .expect("Uint8Array codec control uses Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion,
        );
    }
}

#[test]
fn exact_byte_domains_and_saved_static_constructor() {
    assert_codecs(
        r#"
var a=new Uint8Array(256);for(var i=0;i<256;i++)a[i]=i;
var text=a.toHex(), h=Uint8Array.fromHex(text), b=Uint8Array.fromBase64(a.toBase64());
assert(text.length===512 && text.slice(0,8)==='00010203' && text.slice(-4)==='feff','lowercase width');
for(var j=0;j<256;j++)assert(h[j]===j && b[j]===j,'all byte values');
assert(new Uint8Array([102]).toBase64()==='Zg==' && new Uint8Array([102]).toBase64({omitPadding:true})==='Zg','padding');
assert(new Uint8Array([251,255]).toBase64()==='+/8=' && new Uint8Array([251,255]).toBase64({alphabet:'base64url'})==='-_8=','alphabet');
assert(Uint8Array.fromBase64('-_8=',{alphabet:'base64url'}).toHex()==='fbff','URL');
assert(Object.getPrototypeOf(Uint8Array.fromHex.call(Int16Array,'0001'))===Uint8Array.prototype,'static this ignored');
"#,
    );
}
#[test]
fn hex_utf16_parity_and_completed_error_prefix() {
    assert_codecs(
        r#"
var a=new Uint8Array([9,9,9,9]);try{a.setFromHex('0102zz');throw new Error('invalid accepted');}catch(e){assert(e instanceof SyntaxError,'invalid digit');}
assert(a.join(',')==='1,2,9,9','prefix committed before SyntaxError');
var one=new Uint8Array(1), r=one.setFromHex('01zz');assert(one[0]===1 && r.read===2 && r.written===1,'capacity does not validate suffix');
try{new Uint8Array(0).setFromHex('010');throw new Error('odd accepted');}catch(e){assert(e instanceof SyntaxError,'parity before zero capacity');}
var u=new Uint8Array([7,7]);try{u.setFromHex('01é0');throw new Error('Unicode accepted');}catch(e){assert(e instanceof SyntaxError,'UTF16 invalid digit');}
assert(u[0]===1 && u[1]===7,'UTF16 prefix');
try{Uint8Array.fromHex('aaé');throw new Error('odd accepted');}catch(e){assert(e instanceof SyntaxError,'UTF16 parity');}
"#,
    );
}
#[test]
fn base64_capacity_errors_and_quantum_read_count() {
    assert_codecs(
        r#"
var a=new Uint8Array([9,9,9,9]);try{a.setFromBase64('AQID$');throw new Error('invalid accepted');}catch(e){assert(e instanceof SyntaxError,'invalid alphabet');}
assert(a.join(',')==='1,2,3,9','prefix committed');
var one=new Uint8Array([9]), r=one.setFromBase64('AQI=');assert(r.read===0 && r.written===0 && one[0]===9,'partial quantum exceeds capacity');
r=one.setFromBase64('AQ==');assert(r.read===4 && r.written===1 && one[0]===1,'padded quantum fits');
var full=new Uint8Array(3);r=full.setFromBase64('AQID   $');assert(r.read===4 && r.written===3 && full.join(',')==='1,2,3','full capacity leaves suffix');
r=new Uint8Array(0).setFromBase64('$');assert(r.read===0 && r.written===0,'zero capacity');
"#,
    );
}
#[test]
fn base64_final_chunks_bits_and_ascii_whitespace() {
    assert_codecs(
        r#"
assert(Uint8Array.fromBase64('Zg').toHex()==='66','loose final');
assert(Uint8Array.fromBase64(' \tZ m 9 v\r\n').toHex()==='666f6f','ASCII whitespace');
assert(Uint8Array.fromBase64('Zm9vZg',{lastChunkHandling:'stop-before-partial'}).toHex()==='666f6f','partial omitted');
var r=new Uint8Array(6).setFromBase64('Zm9vZg',{lastChunkHandling:'stop-before-partial'});assert(r.read===4 && r.written===3,'whole quantum count');
assert(Uint8Array.fromBase64('AQ=',{lastChunkHandling:'stop-before-partial'}).length===0,'incomplete padding');
assert(Uint8Array.fromBase64('AQ==',{lastChunkHandling:'strict'}).toHex()==='01','strict zero extra bits');
var bad=['Zg','AR==','AQJ=','A','AQ==x','AQ\u00a0=='];for(var i=0;i<bad.length;i++){
 try{Uint8Array.fromBase64(bad[i],{lastChunkHandling:'strict'});throw new Error('accepted');}catch(e){assert(e instanceof SyntaxError,'strict invalid');}
}
try{Uint8Array.fromBase64('+A==',{alphabet:'base64url'});throw new Error('mixed alphabet');}catch(e){assert(e instanceof SyntaxError,'URL excludes plus');}
"#,
    );
}
#[test]
fn brand_input_order_options_and_current_buffer_bounds() {
    assert_codecs(
        r#"
var token={}, log=[], options={get alphabet(){log.push('a');return 'base64';},get lastChunkHandling(){log.push('l');throw token;}};
try{Uint8Array.prototype.setFromBase64.call(new Uint16Array(1),'AQ==',options);throw new Error('brand');}catch(e){assert(e instanceof TypeError && log.length===0,'brand before options');}
try{Uint8Array.fromBase64({toString:function(){throw token;}},options);throw new Error('input');}catch(e){assert(e instanceof TypeError && log.length===0,'noncoercing String admission');}
var buffer=new ArrayBuffer(3,{maxByteLength:6}), a=new Uint8Array(buffer);a.set([1,2,3]);
var text=a.toBase64({get alphabet(){log.push('a');buffer.resize(1);return 'base64';},get omitPadding(){log.push('p');return false;}});
assert(text==='AQ==' && log.join(',')==='a,p','snapshot after option effects');log=[];
var r=a.setFromBase64('AQID',{get alphabet(){log.push('a');return 'base64';},get lastChunkHandling(){log.push('l');buffer.resize(3);return 'loose';}});
assert(r.written===3 && a.join(',')==='1,2,3' && log.join(',')==='a,l','fresh bounds after options');
var detached=new Uint8Array(new ArrayBuffer(1));$262.detachArrayBuffer(detached.buffer);log=[];
try{detached.toBase64({get alphabet(){log.push('a');return 'base64';},get omitPadding(){log.push('p');return false;}});throw new Error('detached');}catch(e){assert(e instanceof TypeError && log.join(',')==='a,p','bounds after option Get');}
var immutable=new Uint8Array(new ArrayBuffer(0).transferToImmutable());
try{immutable.setFromBase64('',{get alphabet(){throw token;}});throw new Error('immutable');}catch(e){assert(e instanceof TypeError && e!==token,'empty immutable before options');}
"#,
    );
}
#[test]
fn shared_byte_owner_and_borrowed_function_realms() {
    assert_codecs(
        r#"
var shared=new Uint8Array(new SharedArrayBuffer(4)), r=shared.setFromHex('01020304');
assert(r.read===8 && r.written===4 && shared.toBase64()==='AQIDBA==' && shared.toHex()==='01020304','shared byte owner');
var realm=$262.createRealm(), other=realm.global, from=other.Uint8Array.fromHex, set=other.Uint8Array.prototype.setFromHex, prototype=other.Uint8Array.prototype;
other.Uint8Array=function Replacement(){};
var created=from.call(Uint8Array,'0708');assert(Object.getPrototypeOf(created)===prototype && created[0]===7 && created[1]===8,'saved intrinsic');
var local=new Uint8Array(2);r=set.call(local,'090a');assert(Object.getPrototypeOf(r)===other.Object.prototype && local.toHex()==='090a','count result called Realm');
try{from('z0');throw new Error('syntax');}catch(e){assert(e instanceof other.SyntaxError && !(e instanceof SyntaxError),'error called Realm');}
try{set.call(new Proxy(local,{}),'00');throw new Error('proxy');}catch(e){assert(e instanceof other.TypeError && !(e instanceof TypeError),'concrete brand');}
"#,
    );
}
