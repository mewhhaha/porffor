use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

fn observe(source: &str, expected: &str) {
    lila_engine::configure_compilation_jobs(1).unwrap();
    let observed = Engine::new(RealmBuilder::new().build())
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
        .unwrap();
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        [HostOutputEvent::PrintLine(expected.into())]
    );
}

#[test]
fn ordinary_generator_for_in_retains_lazy_enumeration_and_original_cells_across_abrupt_resumes() {
    let source = include_str!("fixtures/generator_for_in/regions.js");
    for prefix in ["", "'use strict';\n"] {
        observe(&format!("{prefix}{source}"), "generator-forin-regions:ok");
    }
}

#[test]
fn ordinary_generator_for_in_initializes_eager_original_references_and_patterns_once_per_key() {
    let source = include_str!("fixtures/generator_for_in/assignments.js");
    for prefix in ["", "'use strict';\n"] {
        observe(
            &format!("{prefix}{source}"),
            "generator-forin-assignments:ok",
        );
    }
}

#[test]
fn ordinary_generator_for_in_with_assignment_uses_the_original_object_record_on_each_key() {
    observe(
        r#"
function same(a,b,message){if(a!==b)throw new Error(message);}
var key='outer';
var view={key:'initial'};
function* values(scope){with(scope){for(key in {a:1,b:1}){yield key;yield ()=>key;}}}
var iterator=values(view);
same(iterator.next().value,'a','first original Object record write');
delete view.key;
gc();
var read=iterator.next().value;
same(read(),'outer','original Object record observes deletion');
same(iterator.next().value,'b','next original Reference falls through to global');
same(key,'b','same global Reference written');
same(iterator.next().value(),'b','reader shares original environment chain');
same(iterator.next().done,true,'original record completes');
same(read(),'b','escaped reader keeps original record');
function* nested(scope){with(scope){for(let name in {x:1}){if(true){for(let item of [1]){if(item)name;}}yield name;}}}
same(nested({}).next().value,'x','eager foreign ForOf keeps its actual continuation domain');
print('generator-forin-with:ok');
"#,
        "generator-forin-with:ok",
    );
}

#[test]
fn ordinary_generator_for_in_preserves_real_ignored_and_abrupt_named_function_puts() {
    let source = r#"
function same(a,b,message){if(a!==b)throw new Error(message);}
var strict=(function(){return this===undefined;})();
var original=function* retained(view){for(retained in view){yield retained;}};
var iterator=original({a:1,b:1});
if(strict){
  var caught;
  try{iterator.next();}catch(error){caught=error;}
  same(caught instanceof TypeError,true,'strict named function Reference Put is abrupt');
}else{
  same(iterator.next().value,original,'sloppy named function first Put is ignored');
  gc();
  same(iterator.next().value,original,'sloppy named function resumed next Put is ignored');
  same(iterator.next().done,true,'ignored Reference cursor completes');
}
var target={};
var selected=function* retained(scope){with(scope){for(retained in {x:1,y:1}){yield retained;}}};
var intercepted=selected(target);
same(intercepted.next().value,selected,'absent With binding uses original ignored fallback');
target.retained='visible';
gc();
same(intercepted.next().value,'y','next Reference selects actual mutable Object record');
same(target.retained,'y','selected Object record consumes real Put');
same(intercepted.next().done,true,'original With cursor completes');
print('generator-forin-named-reference:ok');
"#;
    observe(source, "generator-forin-named-reference:ok");
    // With is a sloppy-only statement. The strict case retains the same
    // named-function source and actual TypeError without parsing that branch.
    observe(
        r#"'use strict';
var original=function* retained(view){for(retained in view){yield retained;}};
var caught;try{original({a:1}).next();}catch(error){caught=error;}
if(!(caught instanceof TypeError))throw new Error('strict original Reference');
print('generator-forin-named-reference:ok');
"#,
        "generator-forin-named-reference:ok",
    );
}
