use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_number_to_bigint(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
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
        .unwrap_or_else(|error| panic!("NumberToBigInt execution failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains("boolean(true)"),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn dynamic_integral_numbers_cross_signed_and_unsigned_word_boundaries_exactly() {
    assert_number_to_bigint(
        r#"
function convert(value){return BigInt(value);}
var signed=1n<<63n, unsigned=1n<<64n;
convert(0)===0n && convert(-0)===0n && convert(1)===1n && convert(-1)===-1n &&
convert(9007199254740991)===9007199254740991n &&
convert(2**63-1024)===signed-1024n && convert(2**63)===signed &&
convert(2**63+2048)===signed+2048n && convert(-(2**63))===-signed &&
convert(-(2**63)-2048)===-signed-2048n &&
convert(2**64-2048)===unsigned-2048n && convert(2**64)===unsigned &&
convert(2**64+4096)===unsigned+4096n && convert(-(2**64))===-unsigned;
"#,
    );
}

#[test]
fn every_integral_binary64_exponent_and_extreme_mantissas_remain_exact() {
    assert_number_to_bigint(
        r#"
function convert(value){return BigInt(value);}
for(let exponent=0;exponent<1024;exponent++){
  var power=2**exponent, expected=1n<<BigInt(exponent);
  if(convert(power)!==expected || convert(-power)!==-expected) throw 'power exponent';
  if(exponent>=52){
    var nearPower=(1+2**-52)*power;
    var nearExpected=((1n<<52n)+1n)<<BigInt(exponent-52);
    if(convert(nearPower)!==nearExpected || convert(-nearPower)!==-nearExpected) throw 'mantissa low bit';
  }
}
var maximum=((1n<<53n)-1n)<<971n;
convert(Number.MAX_VALUE)===maximum && convert(-Number.MAX_VALUE)===-maximum &&
Number(convert(Number.MAX_VALUE))===Number.MAX_VALUE &&
Number(convert(-Number.MAX_VALUE))===-Number.MAX_VALUE;
"#,
    );
}

#[test]
fn invalid_numbers_still_throw_and_to_bigint_still_rejects_number_inputs() {
    assert_number_to_bigint(
        r#"
function convert(value){return BigInt(value);}
var invalid=[NaN,Infinity,-Infinity,0.5,-0.5,1.9,-1.9,Number.MIN_VALUE,-Number.MIN_VALUE];
var rejected=0;
for(var value of invalid){
  try{convert(value);throw 'invalid Number accepted';}
  catch(error){if(!(error instanceof RangeError))throw error;rejected++;}
}
var strictRejected=false;
try{BigInt.asIntN(64,2**63);}catch(error){strictRejected=error instanceof TypeError;}
var constructorRejected=false,calls=0;
try{new BigInt({valueOf(){calls++;return 1;}});}catch(error){constructorRejected=error instanceof TypeError;}
rejected===invalid.length && strictRejected && constructorRejected && calls===0 &&
convert(true)===1n && convert(false)===0n && convert(-7n)===-7n &&
convert('0x10000000000000000')===(1n<<64n);
"#,
    );
}

#[test]
fn numeric_primitive_hooks_are_observed_once_and_abrupt_values_propagate() {
    assert_number_to_bigint(
        r#"
function convert(value){return BigInt(value);}
var trace='',marker={},caught;
var exotic={
  [Symbol.toPrimitive](hint){trace+=hint;return Number.MAX_VALUE;},
  valueOf(){throw 'unexpected valueOf';}
};
var maximum=convert(exotic);
var ordinary=convert({valueOf(){trace+=';valueOf';return 2**64;}});
try{convert({valueOf(){throw marker;}});}catch(error){caught=error;}
trace==='number;valueOf' && maximum===(((1n<<53n)-1n)<<971n) &&
ordinary===(1n<<64n) && caught===marker;
"#,
    );
}

#[test]
fn number_validation_errors_use_the_bigint_functions_defining_realm() {
    assert_number_to_bigint(
        r#"
var foreign=__lilaCreateRealm().global, convert=foreign.BigInt;
var invalid=[0.5,-0.5,NaN,Infinity,-Infinity,Number.MIN_VALUE,-Number.MIN_VALUE];
for(var value of invalid){
  var caught;
  try{convert(value);throw 'invalid Number accepted';}catch(error){caught=error;}
  if(Object.getPrototypeOf(caught)!==foreign.RangeError.prototype ||
     Object.getPrototypeOf(caught)===RangeError.prototype) throw 'Number validation Realm';
}
var hints='',marker={},received,coercionError;
try{convert.call(BigInt,{[Symbol.toPrimitive](hint){hints+=hint;return 0.5;}});}
catch(error){coercionError=error;}
try{convert({valueOf(){throw marker;}});}catch(error){received=error;}
hints==='number' && Object.getPrototypeOf(coercionError)===foreign.RangeError.prototype &&
received===marker && convert(2**64)===(1n<<64n);
"#,
    );
}
