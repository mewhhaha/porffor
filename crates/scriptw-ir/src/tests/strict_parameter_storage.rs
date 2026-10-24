#[test]
fn strict_simple_parameters_only_own_cells_for_real_environment_consumers() {
    for (source, retains_parameter) in [
        (
            "'use strict'; function owner(value) { return value; }",
            false,
        ),
        ("function owner(value) { return value; }", true),
        (
            "'use strict'; function owner(value) { return () => value; }",
            true,
        ),
        (
            "'use strict'; function owner(value) { return eval('value'); }",
            true,
        ),
        (
            "'use strict'; function* owner(value) { yield value; return value; }",
            true,
        ),
        (
            "'use strict'; async function owner(value) { await 0; return value; }",
            true,
        ),
    ] {
        let program = lower_script(source);
        assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
        let owner = program
            .script
            .as_ref()
            .unwrap()
            .functions
            .iter()
            .find(|function| function.name == "owner")
            .expect("owner function");
        assert_eq!(
            owner
                .owned_env_bindings
                .iter()
                .any(|binding| binding.name == "value"),
            retains_parameter,
            "{source}: {:?}",
            owner.owned_env_bindings
        );
        for child in &program.script.as_ref().unwrap().functions {
            for capture in child
                .captured_bindings
                .iter()
                .filter(|binding| binding.name == "value")
            {
                assert!(owner
                    .owned_env_bindings
                    .iter()
                    .any(|binding| binding.name == capture.name && binding.slot == capture.slot));
            }
        }
    }
}

#[test]
fn strict_capture_hops_skip_only_activations_without_physical_storage() {
    let program = lower_script(
        "'use strict'; function outer(value) { function middle(unused) { function read() { return value; } return read; } return middle; }",
    );
    assert!(program.is_wasm_supported(), "{:?}", program.diagnostics);
    let functions = &program.script.as_ref().unwrap().functions;
    let outer = functions
        .iter()
        .find(|function| function.name == "outer")
        .unwrap();
    let middle = functions
        .iter()
        .find(|function| function.name == "middle")
        .unwrap();
    let read = functions
        .iter()
        .find(|function| function.name == "read")
        .unwrap();
    assert!(!middle
        .owned_env_bindings
        .iter()
        .any(|binding| binding.name == "unused"));
    let retained = outer
        .owned_env_bindings
        .iter()
        .find(|binding| binding.name == "value")
        .unwrap();
    let capture = read
        .captured_bindings
        .iter()
        .find(|binding| binding.name == "value")
        .unwrap();
    assert_eq!(capture.slot, retained.slot);
    assert_eq!(
        capture.hops, 0,
        "an empty middle activation is absent from the environment chain"
    );
}
