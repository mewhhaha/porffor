use super::*;

#[test]
fn array_present_index_helper_preserves_growth_holes_values_and_updates() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            r#"
var object = { retained: 17 };
var array = [, undefined, object, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16];
if (array.length !== 17 || 0 in array || !(1 in array) || array[1] !== undefined || array[2] !== object)
  throw 'literal presence';
for (var i = 17; i < 80; i++) array[i] = i;
array[2] = object;
array[4097] = object;
array[4097] = 23;
if (array[2] !== object || array[4097] !== 23 || array.length !== 4098)
  throw 'present update';
var keys = Object.keys(array);
if (keys.length !== 80 || keys[0] !== '1' || keys[79] !== '4097') throw 'present keys';
delete array[3];
array[3] = object;
if (array[3].retained !== 17 || Object.keys(array).length !== 80) throw 'delete and reinsert';
var sentinel = {};
var seen = 0;
function value() { seen++; if (seen === 2) throw sentinel; return object; }
var caught = false;
try { [value(), value(), value()]; } catch (error) { caught = error === sentinel; }
if (!caught || seen !== 2) throw 'element order';
true;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}
