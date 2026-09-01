const MODULE_SOURCE: &str = include_str!("../src/module.rs");
const OWNER_SOURCE: &str = include_str!("../src/module/compiled_module_package.rs");
const EMIT_SOURCE: &str = include_str!("../src/emit.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

#[test]
fn compiled_module_package_has_one_private_owner_and_narrow_reexport() {
    let module_production = MODULE_SOURCE
        .split_once("#[cfg(test)]")
        .expect("module test boundary")
        .0;
    assert_eq!(
        module_production
            .matches("\nmod compiled_module_package;\n")
            .count(),
        1
    );
    assert!(!module_production.contains("\npub mod compiled_module_package;\n"));
    assert!(!module_production.contains("\nmod compiled_module_package {\n"));
    assert!(OWNER_SOURCE.contains("\nuse super::*;\n\n"));

    let reexport = bounded(
        module_production,
        "pub(crate) use compiled_module_package::{",
        "};",
    );
    for surface in [
        "ModuleAssemblySections",
        "ModuleGlobalSectionBuilder",
        "ModuleTypeRegistry",
    ] {
        assert_eq!(reexport.matches(surface).count(), 1, "{surface}");
    }
    for private_state in [
        "FinalizedModuleSections",
        "CompiledModulePackage",
        "CallableFunctionTableSections",
    ] {
        assert!(!reexport.contains(private_state), "{private_state}");
        assert!(!module_production.contains(&format!("struct {private_state}")));
        assert_eq!(
            OWNER_SOURCE
                .matches(&format!("struct {private_state}"))
                .count(),
            1,
            "{private_state}"
        );
    }

    for sole_owner in [
        "struct ModuleTypeRegistry",
        "struct FinalizedModuleSections",
        "struct CompiledModulePackage",
        "struct CallableFunctionTableSections",
        "struct ModuleAssemblySections",
        "struct ModuleTypeSectionBuilder",
        "struct ModuleGlobalSectionBuilder",
    ] {
        assert_eq!(OWNER_SOURCE.matches(sole_owner).count(), 1, "{sole_owner}");
        assert!(!module_production.contains(sole_owner), "{sole_owner}");
    }
}

#[test]
fn package_lifecycle_is_consume_once_and_compile_time_checked() {
    assert!(OWNER_SOURCE.contains("globals: globals.finish(runtime),"));
    assert!(OWNER_SOURCE.contains("compilation.compile_into(&self.globals, &mut code)?"));
    assert!(OWNER_SOURCE.contains("let (code, function_table) = code.finish();"));

    for ownership_gate in [
        "FinalizedModuleSections::compile_main;",
        "CompiledModulePackage::append_remaining_functions;",
        "CompiledModulePackage::append_to_module;",
    ] {
        assert_eq!(
            OWNER_SOURCE.matches(ownership_gate).count(),
            1,
            "{ownership_gate}"
        );
    }
    assert!(OWNER_SOURCE.contains("const _: fn(&mut CompiledModulePackage, Vec<EmittedFunction>)"));
    assert!(OWNER_SOURCE
        .contains("const _: fn(CompiledModulePackage, &mut Module, ModuleAssemblySections)"));

    let assembly = bounded(
        OWNER_SOURCE,
        "let ModuleAssemblySections {",
        "function_table\n    }",
    );
    let expected_order = [
        "module.section(&types);",
        "module.section(&imports);",
        "module.section(&functions);",
        "module.section(&tables);",
        "module.section(&memories);",
        "module.section(&globals);",
        "module.section(&exports);",
        "module.section(&elements);",
        "module.section(&code);",
        "module.section(&data);",
    ];
    let mut prior_offset = 0;
    for section_append in expected_order {
        let offset = assembly
            .find(section_append)
            .unwrap_or_else(|| panic!("missing section append: {section_append}"));
        assert!(
            offset >= prior_offset,
            "section append out of order: {section_append}"
        );
        prior_offset = offset;
    }
}

#[test]
fn callable_function_table_is_a_mandatory_package_input() {
    for source in [OWNER_SOURCE, EMIT_SOURCE] {
        assert!(!source.contains("uses_function_table"));
    }

    for optional_section in ["Option<TableSection>", "Option<ElementSection>"] {
        assert!(
            !OWNER_SOURCE.contains(optional_section),
            "{optional_section}"
        );
    }

    assert_eq!(
        MODULE_SOURCE
            .matches("pub(crate) const JS_FUNCTION_TYPE_INDEX: u32 = 1;")
            .count(),
        1
    );
    let type_registry = bounded(
        OWNER_SOURCE,
        "impl ModuleTypeRegistry {",
        "    /// Consumes every scalar/dynamic global",
    );
    let main_signature = "types.function([], [ValType::I64]);";
    let javascript_signature = "types.function(\n            function_param_types(),\n            [ValType::I64, ValType::I64, ValType::I64, ValType::I64],\n        );";
    let heap_allocation_signature = "types.function([ValType::I64], [ValType::I64]);";
    let registry_creation = "let mut types = ModuleTypeSectionBuilder::new();";
    let registry_creation_offset = type_registry
        .find(registry_creation)
        .expect("type registry construction");
    let main_type = type_registry
        .find(main_signature)
        .expect("main function type registration");
    let javascript_type = type_registry
        .find(javascript_signature)
        .expect("JavaScript function type registration");
    let heap_allocation_type = type_registry
        .find(heap_allocation_signature)
        .expect("heap-allocation function type registration");
    assert!(
        registry_creation_offset < main_type
            && main_type < javascript_type
            && javascript_type < heap_allocation_type,
        "the registry must construct main, JavaScript and heap-allocation types in index order"
    );
    assert!(
        type_registry[registry_creation_offset + registry_creation.len()..main_type]
            .trim()
            .is_empty(),
        "the main signature must remain type index 0"
    );
    assert!(
        type_registry[main_type + main_signature.len()..javascript_type]
            .trim()
            .is_empty(),
        "the JavaScript signature must immediately follow the main signature"
    );
    assert!(
        type_registry[javascript_type + javascript_signature.len()..heap_allocation_type]
            .trim()
            .is_empty(),
        "the JavaScript signature must remain type index 1"
    );
    assert_eq!(type_registry.matches("function_param_types()").count(), 1);

    assert!(!OWNER_SOURCE.contains("pub(crate) struct CallableFunctionTableSections"));
    let callable_sections = bounded(
        OWNER_SOURCE,
        "struct CallableFunctionTableSections {",
        "/// The non-runtime core sections",
    );
    for paired_section in [
        "tables: TableSection,",
        "elements: ElementSection,",
        "let mut tables = TableSection::new();",
        "element_type: RefType::FUNCREF,",
        "minimum: callable_function_count as u64,",
        "maximum: Some(callable_function_count as u64),",
        "let mut elements = ElementSection::new();",
        "first_callable_wasm_index + callable_function_count as u32",
        "elements.active(",
        "Some(0),",
        "&ConstExpr::i32_const(0),",
        "Elements::Functions(Cow::Owned(function_indexes)),",
    ] {
        assert!(
            callable_sections.contains(paired_section),
            "{paired_section}"
        );
    }

    let assembly_state = bounded(
        OWNER_SOURCE,
        "pub(crate) struct ModuleAssemblySections {",
        "impl ModuleAssemblySections {",
    );
    assert!(assembly_state.contains("callable_function_table: CallableFunctionTableSections,"));
    for raw_section_field in ["tables: TableSection,", "elements: ElementSection,"] {
        assert!(
            !assembly_state.contains(raw_section_field),
            "{raw_section_field}"
        );
    }

    for compile_time_gate in [
        "const _: fn() -> ModuleTypeRegistry = ModuleTypeRegistry::new;",
        "FunctionSection,\n    u32,\n    usize,\n    Option<MemorySection>,\n    ExportSection,",
    ] {
        assert!(
            OWNER_SOURCE.contains(compile_time_gate),
            "{compile_time_gate}"
        );
    }

    for emitter_construction in [
        "let module_types = ModuleTypeRegistry::new();",
        "let first_callable_wasm_index = imported_function_count + 1;",
    ] {
        assert_eq!(
            EMIT_SOURCE.matches(emitter_construction).count(),
            1,
            "{emitter_construction}"
        );
    }
    for raw_section_constructor in ["TableSection::new()", "ElementSection::new()"] {
        assert!(
            !EMIT_SOURCE.contains(raw_section_constructor),
            "{raw_section_constructor}"
        );
        assert_eq!(
            OWNER_SOURCE.matches(raw_section_constructor).count(),
            1,
            "{raw_section_constructor}"
        );
    }
    let emitter_assembly = bounded(EMIT_SOURCE, "ModuleAssemblySections::new(", "),\n    );");
    let first_index = emitter_assembly
        .find("first_callable_wasm_index,")
        .expect("callable function first index");
    let function_count = emitter_assembly
        .find("callable_function_count,")
        .expect("callable function count");
    assert!(first_index < function_count);

    let assembly = bounded(
        OWNER_SOURCE,
        "let ModuleAssemblySections {",
        "function_table\n    }",
    );
    for mandatory_section in ["module.section(&tables);", "module.section(&elements);"] {
        assert_eq!(
            assembly.matches(mandatory_section).count(),
            1,
            "{mandatory_section}"
        );
    }
}

#[test]
fn emitter_consumes_the_package_through_the_reviewed_lifecycle() {
    for (call, count) in [
        ("ModuleTypeRegistry::new(", 1),
        ("ModuleGlobalSectionBuilder::new(", 1),
        ("module_types.finalize_globals(globals)", 1),
        ("module_sections.compile_main(", 1),
        ("module_package.append_remaining_functions(", 1),
        ("module_package.main_emitted_local_count()", 1),
        ("module_package.append_to_module(", 1),
        ("ModuleAssemblySections::new(", 1),
    ] {
        assert_eq!(EMIT_SOURCE.matches(call).count(), count, "{call}");
    }

    for escaped_append in [
        "module.section(&types)",
        "module.section(&globals)",
        "module.section(&code)",
    ] {
        assert!(!EMIT_SOURCE.contains(escaped_append), "{escaped_append}");
    }
    assert!(!EMIT_SOURCE.contains("CompiledModulePackage"));
    assert!(!EMIT_SOURCE.contains("FinalizedModuleSections"));
}
