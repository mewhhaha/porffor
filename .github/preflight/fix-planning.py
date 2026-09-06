from pathlib import Path

p = Path('crates/lila-aot-wasm/src/planning.rs')
s = p.read_text()
old = '''            StandardBuiltinId::ArrayPrototypeSlice | StandardBuiltinId::ArrayPrototypeSplice
        ) {
            self.require_standard_builtin(StandardBuiltinId::ObjectDefineProperty);
'''
new = '''            StandardBuiltinId::ArrayPrototypeFlatMap
                | StandardBuiltinId::ArrayPrototypeSlice
                | StandardBuiltinId::ArrayPrototypeSplice
        ) {
            // These algorithms create result properties through the shared
            // descriptor path, even when user code never references Object.
            self.require_standard_builtin(StandardBuiltinId::ObjectDefineProperty);
'''
assert s.count(old) == 1
p.write_text(s.replace(old, new))

p = Path('crates/lila-engine/tests/aot_flat_map.rs')
with p.open('a') as f:
    f.write('''
#[test]
fn minimal_program_roots_target_property_definition_without_object_references() {
    assert_wasm_true(r#"
var result = [2, 3].flatMap(function(value) { return [value, value * 2]; });
var empty = [].flatMap(function() { throw 1; });
result.length === 4 && result[0] === 2 && result[1] === 4 &&
result[2] === 3 && result[3] === 6 && empty.length === 0;
"#);
}
''')

p = Path('crates/lila-aot-wasm/tests/array_flat_map_algorithm_owner_structure.rs')
with p.open('a') as f:
    f.write('''
#[test]
fn flat_map_roots_the_shared_target_definition_builtin() {
    let planning = include_str!("../src/planning.rs");
    let start = planning.find("            StandardBuiltinId::ArrayPrototypeFlatMap\\n").expect("flatMap dependency arm");
    let arm = &planning[start..];
    let end = arm.find("\\n        }").expect("dependency arm end");
    assert!(arm[..end].contains("self.require_standard_builtin(StandardBuiltinId::ObjectDefineProperty);"));
}
''')

for path in ['docs/rust-rewrite/aot-flat-map.md', 'tasks/16-arrays-and-array-builtins.md']:
    p = Path(path)
    s = p.read_text().replace('sixteen named', 'seventeen named').replace('Sixteen new', 'Seventeen new')
    p.write_text(s)
