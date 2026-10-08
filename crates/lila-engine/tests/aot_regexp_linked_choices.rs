use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

#[test]
fn linked_runs_share_atomic_assertion_and_nullable_progress_boundaries() {
    lila_engine::configure_compilation_jobs(1).expect("one compiler worker");
    let source = r#"
function computed(pattern) {
  var text='';
  for(var i=0;i<pattern.length;i++) text+=String.fromCharCode(pattern.charCodeAt(i));
  return new RegExp(text,'d');
}
var cases=[
  [/^(?=(?<repeat>(?:()|a){18446744073709551616}))b$/d,'b',function(m){
    return m!==null && m[0]==='b' && m.groups.repeat==='' && m[2]==='' &&
      m.indices.groups.repeat[0]===0 && m.indices.groups.repeat[1]===0;
  }],
  [/^(?!(?:()|a){4}c)b$/d,'b',function(m){return m!==null && m[0]==='b' && m[1]===undefined;}],
  [/^(?:(?:()|a){4})*?ab$/d,'aab',function(m){return m!==null && m[0]==='aab';}],
  [/^(?:(?=(a|aa))\1a|(?=(ab))\2)c$/d,'abc',function(m){
    return m!==null && m[1]===undefined && m[2]==='ab' && m.indices[2][0]===0 && m.indices[2][1]===2;
  }]
];
for(var c=0;c<cases.length;c++) {
  var row=cases[c], expressions=[row[0],computed(row[0].source)];
  for(var e=0;e<expressions.length;e++) {
    var m=expressions[e].exec(row[1]);
    if(!row[2](m)) throw new Error('linked continuation boundary '+c+':'+e);
  }
}
true;
"#;
    for strict in [false, true] {
        let program = if strict {
            format!("'use strict';\n{source}")
        } else {
            source.to_owned()
        };
        let outcome = Engine::new(RealmBuilder::new().build())
            .run_script(
                &program,
                CompileOptions::default(),
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    ..RunOptions::default()
                },
            )
            .expect("actual linked-choice Wasm program");
        assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
        assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
    }
}
