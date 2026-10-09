use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_equality_effects(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("equality operands failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn mismatched_static_kinds_evaluate_both_operands_once_in_order() {
    assert_equality_effects(
        r#"
var trace='';
var equal=((trace+='L'),1)===((trace+='R'),'1');
var unequal=((trace+='A'),true)!==((trace+='B'),null);
var reverse=((trace+='C'),'1')===((trace+='D'),1);
trace==='LRABCD' && equal===false && unequal===true && reverse===false;
"#,
    );
}

#[test]
fn abrupt_operands_stop_equality_at_the_first_throw() {
    assert_equality_effects(
        r#"
var trace='',marker={},leftError,rightError;
function fail(){trace+='throw;';throw marker;}
try{(fail(),1)===((trace+='right;'),'1');}catch(error){leftError=error;}
if(trace!=='throw;' || leftError!==marker) throw 'left abrupt evaluation';
trace='';
try{((trace+='left;'),1)!==(fail(),'1');}catch(error){rightError=error;}
trace==='left;throw;' && rightError===marker;
"#,
    );
}

#[test]
fn throwing_numeric_operands_are_not_discarded_by_their_result_kinds() {
    assert_equality_effects(
        r#"
var caught=0;
try{(1n>>>0n)===0n;}catch(error){if(!(error instanceof TypeError))throw error;caught++;}
try{0n!==(1n>>>0n);}catch(error){if(!(error instanceof TypeError))throw error;caught++;}
try{(+1n)==='1';}catch(error){if(!(error instanceof TypeError))throw error;caught++;}
try{'1'!==(+1n);}catch(error){if(!(error instanceof TypeError))throw error;caught++;}
caught===4;
"#,
    );
}

#[test]
fn same_value_keeps_argument_order_abrupt_values_and_number_distinctions() {
    assert_equality_effects(
        r#"
var trace='',marker={},caught;
function fail(){trace+='throw;';throw marker;}
var mismatch=Object.is(((trace+='left;'),1),((trace+='right;'),'1'));
if(mismatch!==false || trace!=='left;right;') throw 'SameValue order';
trace='';
try{Object.is((fail(),1),((trace+='right;'),'1'));}catch(error){caught=error;}
trace==='throw;' && caught===marker && Object.is(NaN,NaN) && !Object.is(0,-0);
"#,
    );
}

#[test]
fn equality_keeps_nan_signed_zero_and_bigint_representation_semantics() {
    assert_equality_effects(
        r#"
var wide=BigInt('9223372036854775808'), narrow=wide-9223372036854775807n;
0===-0 && NaN!==NaN && narrow===1n && wide===9223372036854775808n &&
1n!==1 && Object.is(narrow,1n) && [NaN,-0,narrow].includes(NaN) &&
[NaN,-0,narrow].includes(0) && [NaN,-0,narrow].includes(1n);
"#,
    );
}
