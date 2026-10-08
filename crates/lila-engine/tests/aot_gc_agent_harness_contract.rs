use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_harness(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    for strict in [false, true] {
        let source = format!(
            "{}{}\nfunction assert(v,m){{if(!v)throw new Error(m);}}\n{source}",
            if strict { "\"use strict\";\n" } else { "" },
            include_str!("../../lila-test262/assets/local-harness/wasm-aot-host.js")
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
            .expect("actual harness compiles through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert!(
            matches!(observed.completion, ObservedCompletion::Normal(_)),
            "{:?}\n{source}",
            observed.completion
        );
    }
}

#[test]
fn broadcast_forwards_both_arguments_without_harness_coercion() {
    assert_harness(
        r#"
var buffer={},id={valueOf:function(){throw new Error('wrapper coerced id');}},result={},calls=0;
__lilaAgentBroadcast=function(receivedBuffer,receivedId){
  calls++;
  assert(arguments.length===2 && receivedBuffer===buffer && receivedId===id,'both broadcast arguments');
  return result;
};
assert($262.agent.broadcast(buffer,id)===result && calls===1,'host result forwarded');
var marker={};__lilaAgentBroadcast=function(){throw marker;};
try{$262.agent.broadcast(buffer,id);throw new Error('missing Throw');}catch(e){assert(e===marker,'host Throw retained');}
"#,
    );
}

#[test]
fn receive_unpacks_the_pair_before_calling_the_callback() {
    assert_harness(
        r#"
var buffer={},id=-17,result={},calls=0,receives=0;
__lilaAgentReceiveBroadcast=function(){receives++;return [buffer,id];};
assert($262.agent.receiveBroadcast(function(receivedBuffer,receivedId){
  'use strict';calls++;
  assert(this===undefined && arguments.length===2,'ordinary callback Call');
  assert(receivedBuffer===buffer && receivedId===id,'both received entries');
  return result;
})===result && calls===1 && receives===1,'one receive before callback');
var marker={};
try{$262.agent.receiveBroadcast(function(){throw marker;});throw new Error('missing Throw');}catch(e){assert(e===marker,'callback Throw retained');}
assert(receives===2,'fresh receive for each invocation');
"#,
    );
}
