use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

fn assert_saved_value(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observation = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("saved compound-assignment value must execute through Wasm AOT");
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
fn later_addition_observes_the_saved_primitive_result_kind() {
    assert_saved_value(
        r#"
function number(){let value=1;value += (value='changed',2);return value+1;}
function string(){let value='x';value += (value=10,2);return value+1;}
function bigint(){let value=1n;value += (value={},2n);return value+1n;}
function indirect(){let value=1;const replace=()=>{value='changed';return 2;};value += replace();return value+1;}
if(number()!==4 || string()!=='x21' || bigint()!==4n || indirect()!==4) throw 'saved primitive';
globalThis.savedCompoundNumber=1;
savedCompoundNumber += (savedCompoundNumber='changed',2);
if(savedCompoundNumber+1!==4) throw 'saved global primitive';
print(true);
"#,
    );
}

#[test]
fn arithmetic_and_bitwise_assignments_publish_the_saved_numeric_kind() {
    assert_saved_value(
        r#"
function subtract(){let value=8;value -= (value='changed',2);return value+1;}
function multiply(){let value=8;value *= (value='changed',2);return value+1;}
function divide(){let value=8;value /= (value='changed',2);return value+1;}
function remainder(){let value=8;value %= (value='changed',3);return value+1;}
function power(){let value=8;value **= (value='changed',2);return value+1;}
function bigSubtract(){let value=8n;value -= (value=0,2n);return value+1n;}
function shift(){let value=8n;value >>= (value='changed',1n);return value+1n;}
function bitwise(){let value=8;value |= (value=1n,2);return value+1;}
if(subtract()!==7 || multiply()!==17 || divide()!==5 || remainder()!==3 || power()!==65 ||
   bigSubtract()!==7n || shift()!==5n || bitwise()!==11) throw 'saved numeric kind';
print(true);
"#,
    );
}

#[test]
fn rhs_mutations_change_live_conversion_hooks_of_the_saved_object() {
    assert_saved_value(
        r#"
function replaced(){
  let trace='',value={valueOf(){throw 'old valueOf';}},original=value;
  value += (trace+='rhs;',original.valueOf=function(){trace+='left;';return 3;},value={valueOf(){throw 'replacement used';}},2);
  if(trace!=='rhs;left;') throw trace;
  return value+1;
}
function direct(){
  let value={valueOf(){return 3;}};
  value -= (value.valueOf=function(){return 7n;},1n);
  return value+1n;
}
function primitiveHook(){
  let value={valueOf(){throw 'valueOf before new hook';}},original=value;
  value += (original[Symbol.toPrimitive]=function(hint){if(hint!=='default')throw hint;return 'new';},value=1,2);
  return value+1;
}
function bits(){
  let value={valueOf(){return 3;}},original=value;
  value &= (original.valueOf=function(){return 7n;},value='replacement',3n);
  return value+1n;
}
function callable(){
  let value=function original(){},original=value;
  value += (original[Symbol.toPrimitive]=function(){return 7;},value=null,2);
  return value+1;
}
function array(){
  let value=[1],original=value;
  value -= (original.valueOf=function(){return 7n;},value=null,1n);
  return value+1n;
}
if(replaced()!==6 || direct()!==7n || primitiveHook()!=='new21' || bits()!==4n || callable()!==10 || array()!==7n) throw 'live saved-object coercion';
print(true);
"#,
    );
}

#[test]
fn abrupt_rhs_and_coercion_complete_before_the_final_binding_write() {
    assert_saved_value(
        r#"
var sentinel={};
function rhsThrow(){throw sentinel;}
function checkRhs(){let value=1;try{value += (value='replacement',rhsThrow());}catch(error){if(error!==sentinel)throw error;}return value;}
if(checkRhs()!=='replacement')throw 'rhs replacement survives throw';
function checkCoercion(){
  let value={valueOf(){return 1;}},original=value;
  try{value += (original.valueOf=function(){throw sentinel;},value='replacement',2);}
  catch(error){if(error!==sentinel)throw error;return value;}
  throw 'missing coercion throw';
}
if(checkCoercion()!=='replacement')throw 'coercion replacement survives throw';
function checkConst(){
  const value={valueOf(){return 1;}};
  let received;
  try{value += (value.valueOf=function(){throw sentinel;},2);}catch(error){received=error;}
  return received===sentinel;
}
if(!checkConst())throw 'const write must follow coercion';
print(true);
"#,
    );
}
