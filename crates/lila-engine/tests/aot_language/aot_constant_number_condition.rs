use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_condition_completion(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
        .unwrap_or_else(|error| panic!("numeric guard execution failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(
        outcome.note.contains(expected),
        "{}\n{source}",
        outcome.note
    );
}

#[test]
fn constant_numeric_guards_select_the_correct_branch() {
    assert_condition_completion(
        r#"
var selected=0;
if (4294967297 << 33 === 2) selected++; else throw 'left shift';
if (-2147483649 >> 33 === 1073741823) selected++; else throw 'signed shift';
if (-1 >>> 0 === 4294967295) selected++; else throw 'unsigned shift';
if (7.9 << 1.9 === 14) selected++; else throw 'truncate';
if (1 << -1 === -2147483648) selected++; else throw 'shift mask';
if ((9007199254740991 | 0) === -1) selected++; else throw 'wide residue';
if ((18446744073709551616 | 0) === 0) selected++; else throw 'large residue';
if ((~4294967295) === 0) selected++; else throw 'complement';
if (((7 & 3) ^ 1) === 2) selected++; else throw 'bitwise';
if ((0/0) !== (0/0)) selected++; else throw 'NaN equality';
if (0/0 >= 1) throw 'NaN relation'; else selected++;
if (0/0 < 1) throw 'NaN relation'; else selected++;
if (1/-0 < 0) selected++; else throw 'negative zero division';
if (-0) throw 'negative zero truthiness'; else selected++;
if (0/0) throw 'NaN truthiness'; else selected++;
if ((1/0) >>> 0) throw 'infinite residue'; else selected++;
if (1/(-4%2) < 0) selected++; else throw 'negative zero remainder';
if ((2*3-1) == 5) selected++; else throw 'arithmetic';
if (!(1 << 0 !== 1)) selected++; else throw 'logical not';
selected===19;
"#,
        "boolean(true)",
    );
}

#[test]
fn selected_ordinary_constructor_calls_and_throws_remain_observable() {
    assert_condition_completion(
        r#"
var calls=0, caught;
function Failure(message){calls++;this.message=message;}
if (1 << 0 !== 1) throw new Failure('unreachable');
try { if (1 << 0 === 1) throw new Failure('selected'); else throw 'wrong branch'; }
catch(error){caught=error;}
calls===1 && caught instanceof Failure && caught.message==='selected';
"#,
        "boolean(true)",
    );
}

#[test]
fn reads_calls_and_comma_effects_remain_runtime_conditions() {
    assert_condition_completion(
        r#"
var trace='',reads=0,calls=0,assigned=0;
var object={get value(){reads++;trace+='get;';return 1;}};
function read(){calls++;trace+='call;';return 1;}
if (object.value << 0 !== 1) throw 'getter value';
if (read() << 0 !== 1) throw 'call value';
if ((assigned=1,1) << 0 !== 1) throw 'assignment comma';
if ((read(),1) !== 1) throw 'call comma';
function shadowed(Number,Math,NaN,Infinity){
  if (Number.EPSILON!==7) throw 'shadowed Number';
  if (Math.PI!==8) throw 'shadowed Math';
  if (NaN!==9) throw 'shadowed NaN';
  if (Infinity!==10) throw 'shadowed Infinity';
}
shadowed({get EPSILON(){reads++;return 7;}},{get PI(){reads++;return 8;}},9,10);
reads===3 && calls===2 && assigned===1 && trace==='get;call;call;';
"#,
        "boolean(true)",
    );
}

#[test]
fn bigint_and_mixed_numeric_errors_are_not_folded_away() {
    assert_condition_completion(
        r#"
var throws=0;
try { if ((1n >>> 0n) === 0n) throw 'unsigned BigInt accepted'; }
catch(error){if(!(error instanceof TypeError))throw error;throws++;}
try { if ((1n << 1) === 2n) throw 'mixed shift accepted'; }
catch(error){if(!(error instanceof TypeError))throw error;throws++;}
try { if (+1n === 1) throw 'unary plus BigInt accepted'; }
catch(error){if(!(error instanceof TypeError))throw error;throws++;}
if ((1n << 1n) !== 2n) throw 'BigInt shift';
throws===3;
"#,
        "boolean(true)",
    );
}

#[test]
fn selected_branches_preserve_hoists_labels_and_finally_completions() {
    assert_condition_completion(
        r#"
function hoisted(){
  if (1 << 0 !== 1){var retained=7;function hidden(){return 1;}}
  return typeof retained==='undefined' && typeof hidden==='undefined';
}
var trace='';
outer: for(let index=0;index<3;index++){
  if (1 << 0 === 1){if(index===1)continue outer;trace+=index;}
  else throw 'unreachable loop branch';
}
exit: { if (1 << 1 === 2){break exit;} throw 'selected break'; }
function returning(){
  try{if (1 << 1 === 2){return 9;}else throw 'unreachable return branch';}
  finally{trace+='finally';}
}
hoisted() && returning()===9 && trace==='02finally';
"#,
        "boolean(true)",
    );
}

#[test]
fn if_completion_still_updates_empty_with_undefined() {
    for (source, expected) in [
        ("23; if (1 << 0 !== 1) { 42; }", "undefined(undefined)"),
        ("23; if (1 << 0 === 1) {}", "undefined(undefined)"),
        (
            "23; if (1 << 0 !== 1) { 42; } else {}",
            "undefined(undefined)",
        ),
        ("23; if (1 << 0 === 1) { 42; }", "number(42)"),
        ("23; if (1 << 0 === 1) { 42; var retained; }", "number(42)"),
    ] {
        assert_condition_completion(source, expected);
    }
}
