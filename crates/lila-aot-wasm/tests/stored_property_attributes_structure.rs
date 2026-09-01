use std::fs;
use std::path::Path;

const FUNCTIONS_SOURCE: &str = include_str!("../src/functions.rs");
const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const CONTRACT: &str =
    include_str!("../../../docs/rust-rewrite/contracts/stored-property-attributes.md");
const TASK: &str = include_str!("../../../tasks/10-object-model-descriptors-exotics.md");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn collect_rust_source(path: &Path, source: &mut String) {
    for directory_entry in fs::read_dir(path).expect("Rust source directory") {
        let directory_entry = directory_entry.expect("Rust source directory entry");
        let child_path = directory_entry.path();
        if child_path.is_dir() {
            collect_rust_source(&child_path, source);
        } else if child_path
            .extension()
            .and_then(|extension| extension.to_str())
            == Some("rs")
        {
            source.push_str(&fs::read_to_string(&child_path).expect("Rust source file"));
        }
    }
}

#[test]
fn stored_property_attributes_is_the_exact_capability_free_domain() {
    let domain = bounded(
        HEAP_SOURCE,
        "/// Static 6.2.6.6 attributes for a descriptor-kind word stored in the heap.",
        "/// A **test** against a descriptor word.",
    );
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum StoredPropertyAttributes {",
        "\n}\n\nimpl StoredPropertyAttributes",
    );
    let declaration = normalized(declaration);
    assert_eq!(
        declaration,
        concat!(
            "Data{writable:bool,enumerable:bool,configurable:bool,},",
            "Accessor{enumerable:bool,configurable:bool,},"
        )
    );
    assert!(!domain.contains("#[derive"));
    assert!(!domain.lines().any(|line| {
        line.trim_start().starts_with("impl ") && line.contains(" for StoredPropertyAttributes")
    }));
}

#[test]
fn one_exhaustive_projection_owns_both_descriptor_kinds() {
    let implementation = bounded(
        HEAP_SOURCE,
        "impl StoredPropertyAttributes {",
        "/// A **test** against a descriptor word.",
    );
    let compact = normalized(implementation);
    assert_eq!(implementation.matches("match self {").count(), 1);
    assert!(!implementation.contains("_ =>"));
    assert!(!implementation.contains("unreachable!"));
    assert!(compact.contains(concat!(
        "Self::Data{writable,enumerable,configurable,}=>",
        "DescriptorWord::of_data(writable,enumerable,configurable),"
    )));
    assert!(compact.contains(concat!(
        "Self::Accessor{enumerable,configurable,}=>",
        "DescriptorWord::of_accessor(enumerable,configurable),"
    )));
    assert!(compact.contains("pub(crate)constfndescriptor_kind_bits(self)->u64{"));
    assert!(compact.contains("self.descriptor_word().bits()"));
}

#[test]
fn every_external_producer_names_its_kind_and_attributes() {
    let mut rust_source = String::new();
    collect_rust_source(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut rust_source,
    );

    assert_eq!(
        rust_source
            .matches("StoredPropertyAttributes::Data {")
            .count(),
        14
    );
    assert_eq!(
        rust_source
            .matches("StoredPropertyAttributes::Accessor {")
            .count(),
        2
    );
    assert_eq!(
        rust_source.matches("DescriptorWord::of_data(").count(),
        HEAP_SOURCE.matches("DescriptorWord::of_data(").count()
    );
    assert_eq!(
        rust_source.matches("DescriptorWord::of_accessor(").count(),
        HEAP_SOURCE.matches("DescriptorWord::of_accessor(").count()
    );
    assert!(HEAP_SOURCE.contains("    const fn of_data("));
    assert!(HEAP_SOURCE.contains("    const fn of_accessor("));
    assert!(!HEAP_SOURCE.contains("pub(crate) const fn of_data("));
    assert!(!HEAP_SOURCE.contains("pub(crate) const fn of_accessor("));

    let function_allocation = bounded(
        FUNCTIONS_SOURCE,
        "        function.instruction(&Instruction::I64Const(meta.table_index as i64));",
        "        function.instruction(&Instruction::Call(function_object_alloc_function_index));",
    );
    let function_allocation = normalized(function_allocation);
    assert!(function_allocation.contains(concat!(
        "StoredPropertyAttributes::Data{writable:false,",
        "enumerable:false,configurable:meta.length_name_configurable,}",
        ".descriptor_kind_bits()asi64"
    )));
}

#[test]
fn task_and_contract_record_the_stored_attribute_boundary() {
    for evidence in [TASK, CONTRACT] {
        assert!(evidence.contains("StoredPropertyAttributes"));
        assert!(evidence.contains("positional boolean"));
        assert!(evidence.contains("Accessor"));
    }
}
