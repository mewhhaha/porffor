use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_prepared(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}var $262={{createRealm:__lilaCreateRealm}};function assert(v,m){{if(!v)throw new Error(m);}}\n{source}",
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
            .expect("prepared Script control uses independently compiled Wasm entries");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion,
        );
    }
}

#[test]
fn indirect_eval_returns_non_strings_without_observable_conversion() {
    assert_prepared(
        r#"
var token={toString:function(){throw new Error('unexpected ToString');},valueOf:function(){throw new Error('unexpected ToPrimitive');}};
var indirect=eval;
assert(indirect(token)===token && indirect(Symbol.iterator)===Symbol.iterator && indirect(17n)===17n,'complete non-String values');
assert(indirect()===undefined && indirect(null)===null,'undefined and null');
var text=new String('throw 1;');assert(indirect(text)===text,'boxed String is not parsed');
"#,
    );
}

#[test]
fn foreign_indirect_eval_preserves_completion_and_restores_caller_realm() {
    assert_prepared(
        r#"
var realm=$262.createRealm(), other=realm.global, run=other.eval, token={};
other.token=token;
assert(run('globalThis;')===other && run('31; {}')===31,'global receiver and empty completion');
try{run('throw token;');throw new Error('throw lost');}catch(e){assert(e===token,'whole arbitrary Throw');}
try{run('let duplicate; let duplicate;');throw new Error('syntax accepted');}catch(e){assert(e instanceof other.SyntaxError && !(e instanceof SyntaxError),'deferred syntax target Realm');}
try{run('missingPreparedBinding;');throw new Error('reference accepted');}catch(e){assert(e instanceof other.ReferenceError && !(e instanceof ReferenceError),'compiled runtime target Realm');}
other.eval=function Replacement(){};
assert(run('globalThis;')===other,'captured callable Realm');
assert(Object.getPrototypeOf({})===Object.prototype && Object.getPrototypeOf(new Error())===Error.prototype,'caller Realm restored after all paths');
"#,
    );
}

#[test]
fn realm_script_converts_once_and_keeps_captured_target_realm() {
    assert_prepared(
        r#"
// Keep the argument's Symbol kind across calls that may mutate global Symbol.
const sourceSymbol=Symbol('source');
var realm=$262.createRealm(), other=realm.global, run=realm.evalScript, log=[], marker=Symbol('coercion');
var text={toString:function(){log.push('string');return 'globalThis;';},valueOf:function(){throw new Error('wrong hint');}};
assert(run(text)===other && log.join(',')==='string','one string-hint conversion');
try{run({toString:function(){throw marker;}});throw new Error('coercion lost');}catch(e){assert(e===marker,'original conversion Throw');}
try{run(sourceSymbol);throw new Error('Symbol accepted');}catch(e){assert(e instanceof other.TypeError && !(e instanceof TypeError),'conversion error captured Realm');}
realm.evalScript=function Replacement(){return 0;};
assert(run('41; {}')===41,'captured Script completion');
assert(Object.getPrototypeOf({})===Object.prototype,'caller Realm after normal and Throw');
"#,
    );
}

#[test]
fn finite_global_scripts_allocate_fresh_lexical_cells_each_time() {
    assert_prepared(
        r#"
var realm=$262.createRealm(), run=realm.global.eval;
var a=run('let retained=1; function read(){return retained;} read;');
var b=run('let retained=2; function read(){return retained;} read;');
var c=run('"use strict"; let retained=3; function read(){return retained;} read;');
assert(a()===1 && b()===2 && c()===3 && a!==b && b!==c,'fresh lexical cells survive invocation');
assert(!Object.prototype.hasOwnProperty.call(realm.global,'retained'),'lexical name is not global property');
"#,
    );
}
