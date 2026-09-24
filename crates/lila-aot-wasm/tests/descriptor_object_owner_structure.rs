//! 6.2.6.4 FromPropertyDescriptor has one owner, and that owner alone decides
//! how a descriptor object's fields are defined.
//!
//! Before the owner existed, `Object.getOwnPropertyDescriptor({x: 1}, "x")`
//! produced an object whose four fields were *builtin* properties (writable,
//! non-enumerable, configurable): `Object.keys` saw nothing and
//! `JSON.stringify` printed `{}`. Every producer picked the attributes itself,
//! through whichever define helper was nearest. These tests pin the single
//! private field writer, its CreateDataPropertyOrThrow attributes, and the
//! census of producers routed through it.

use std::fs;
use std::path::Path;

const OWNER: &str = include_str!("../src/objects/descriptor_object.rs");
const OBJECTS: &str = include_str!("../src/objects.rs");
const REFLECT_DESCRIPTOR: &str =
    include_str!("../src/builtins/reflect/descriptor_object_prototype.rs");
const PROTOTYPE_DEFINITION: &str = include_str!("../src/builtins/object/prototype_definition.rs");
const OBJECT_BUILTINS: &str = include_str!("../src/builtins/object.rs");

fn function_body<'a>(source: &'a str, signature: &str) -> &'a str {
    assert_eq!(
        source.matches(signature).count(),
        1,
        "expected exactly one `{signature}`"
    );
    let start = source.find(signature).unwrap();
    let tail = &source[start..];
    let end = tail
        .find("\n    }\n")
        .unwrap_or_else(|| panic!("unterminated function `{signature}`"));
    &tail[..end]
}

fn positions_in_order(source: &str, markers: &[&str]) {
    let mut cursor = 0;
    for marker in markers {
        let offset = source[cursor..]
            .find(marker)
            .unwrap_or_else(|| panic!("missing marker after byte {cursor}: `{marker}`"));
        cursor += offset + marker.len();
    }
}

fn rust_sources(directory: &Path, sources: &mut Vec<(String, String)>) {
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", directory.display()))
    {
        let path = entry.expect("readable source entry").path();
        if path.is_dir() {
            rust_sources(&path, sources);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let relative = path
                .strip_prefix(Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
                .expect("source under src")
                .to_string_lossy()
                .replace('\\', "/");
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            sources.push((relative, source));
        }
    }
}

fn all_sources() -> Vec<(String, String)> {
    let mut sources = Vec::new();
    rust_sources(
        &Path::new(env!("CARGO_MANIFEST_DIR")).join("src"),
        &mut sources,
    );
    sources.sort();
    sources
}

#[test]
fn descriptor_fields_have_one_private_writer_with_create_data_property_attributes() {
    assert!(OBJECTS.contains("\nmod descriptor_object;\n"));
    assert!(!OBJECTS.contains("pub mod descriptor_object;"));
    assert!(!OBJECTS.contains("pub(crate) mod descriptor_object;"));

    // The writer is private to the owner module, has no attribute
    // parameter, and is called only by the owner's two presence arms.
    assert_eq!(
        OWNER
            .matches("    fn emit_descriptor_object_field(")
            .count(),
        1
    );
    assert!(!OWNER.contains("pub(crate) fn emit_descriptor_object_field("));
    assert!(!OWNER.contains("pub(super) fn emit_descriptor_object_field("));
    let writer = function_body(OWNER, "    fn emit_descriptor_object_field(");
    let parameters = writer.split_once(") -> Result<(), EmitError>").unwrap().0;
    assert!(
        !parameters.contains("bool"),
        "the writer must not take attributes"
    );
    assert!(parameters.contains("field: DescriptorField,"));
    assert_eq!(
        writer
            .matches("self.emit_object_define_enumerable_data(")
            .count(),
        1
    );
    assert!(writer.contains("self.strings.payload(field.key())"));
    for sources in all_sources() {
        let (path, source) = sources;
        let expected = usize::from(path == "objects/descriptor_object.rs") * 3;
        assert_eq!(
            source.matches("emit_descriptor_object_field(").count(),
            expected,
            "descriptor fields are written only by the owner: {path}"
        );
    }

    // No other define helper, and so no other attribute set, is reachable
    // from the owner.
    for forbidden in [
        "emit_object_define_data(",
        "emit_object_define_data_with",
        "emit_object_define_local_data",
        "emit_object_define_bool_data",
        "emit_object_append_data_property",
        "emit_object_define_accessor",
    ] {
        assert!(!OWNER.contains(forbidden), "owner reaches `{forbidden}`");
    }
    assert_eq!(
        OWNER.matches("emit_object_define_enumerable_data(").count(),
        1
    );
}

#[test]
fn from_property_descriptor_allocates_once_and_visits_fields_in_6_2_6_4_order() {
    let owner = function_body(OWNER, "    pub(crate) fn emit_from_property_descriptor(");
    assert!(owner.contains("prototype: DescriptorObjectPrototype,"));
    assert!(owner.contains("fields: &DescriptorObjectFields,"));
    positions_in_order(
        owner,
        &[
            "DescriptorObjectPrototype::MainRealmObjectPrototype => self",
            ".emit_alloc_plain_object_with_prototype(",
            "Some(OBJECT_PROTOTYPE_GLOBAL_INDEX),",
            "DescriptorObjectPrototype::ObjectPrototypeLocal(prototype_local) => {",
            "self.emit_alloc_plain_object_with_prototype(Some(prototype_local), None, function)?",
            "DescriptorObjectPrototype::PrivateCarrier => {",
            "self.emit_alloc_plain_object_with_prototype(None, None, function)?",
            "for field in DescriptorField::ALL {",
            "DescriptorField::Value =>",
            "DescriptorField::Writable =>",
            "DescriptorField::Get =>",
            "DescriptorField::Set =>",
            "DescriptorField::Enumerable =>",
            "DescriptorField::Configurable =>",
            "Presence::Absent => {}",
            "Presence::Present(value) => {",
            "self.emit_descriptor_object_field(object_local, field, value, function)?;",
            "Presence::Runtime { present, value } => {",
            "self.emit_descriptor_object_field(object_local, field, value, function)?;",
            "function.instruction(&Instruction::LocalSet(result_payload_local));",
        ],
    );
    assert!(!owner.contains("_ =>"));
    assert_eq!(
        OWNER
            .matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        3
    );
    assert_eq!(
        owner
            .matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        3
    );

    // The value/flag pairing is made once, in the exhaustive field match.
    for field in [
        "Value",
        "Writable",
        "Get",
        "Set",
        "Enumerable",
        "Configurable",
    ] {
        assert_eq!(
            owner
                .matches(&format!("DescriptorField::{field} =>"))
                .count(),
            1,
            "field arm `{field}`"
        );
    }
    for (field, carrier) in [
        ("value", "Value"),
        ("writable", "Flag"),
        ("get", "Value"),
        ("set", "Value"),
        ("enumerable", "Flag"),
        ("configurable", "Flag"),
    ] {
        assert!(
            owner.contains(&format!(
                "field_presence(&fields.{field}, DescriptorObjectFieldValue::{carrier})"
            )),
            "field `{field}` must be carried as {carrier}"
        );
    }
}

#[test]
fn complete_descriptors_and_flags_cannot_misstate_their_shape() {
    let complete = function_body(
        OWNER,
        "    pub(crate) fn emit_from_complete_property_descriptor(",
    );
    assert!(complete.contains("descriptor: CompleteDescriptor<DescriptorObjectLocals>,"));
    positions_in_order(
        complete,
        &[
            "CompleteDescriptor::Data {",
            "get: Presence::Absent,",
            "set: Presence::Absent,",
            "CompleteDescriptor::Accessor {",
            "value: Presence::Absent,",
            "writable: Presence::Absent,",
            "self.emit_from_property_descriptor(prototype, &fields, result_payload_local, function)",
        ],
    );
    assert!(!complete.contains("_ =>"));

    let flag = OWNER
        .split_once("pub(crate) enum DescriptorFlag {")
        .expect("descriptor flag domain")
        .1
        .split_once("\n}\n")
        .unwrap()
        .0;
    assert!(flag.contains("Known(bool),"));
    assert!(flag.contains("BooleanPayload(u32),"));
    assert!(!flag.contains("TaggedLocals"), "a flag is always a Boolean");

    let prototype = OWNER
        .split_once("pub(crate) enum DescriptorObjectPrototype {")
        .expect("descriptor prototype domain")
        .1
        .split_once("\n}\n")
        .unwrap()
        .0;
    for variant in [
        "MainRealmObjectPrototype,",
        "ObjectPrototypeLocal(u32),",
        "PrivateCarrier,",
    ] {
        assert!(prototype.contains(variant), "missing prototype `{variant}`");
    }

    let carrier = function_body(
        OWNER,
        "    pub(crate) fn emit_create_data_property_descriptor_carrier(",
    );
    positions_in_order(
        carrier,
        &[
            "DescriptorObjectPrototype::PrivateCarrier,",
            "CompleteDescriptor::Data {",
            "writable: DescriptorFlag::Known(true),",
            "enumerable: DescriptorFlag::Known(true),",
            "configurable: DescriptorFlag::Known(true),",
        ],
    );
}

#[test]
fn every_descriptor_object_producer_routes_through_the_owner() {
    // The pre-owner allocators that duplicated each other, and the field
    // helpers they chose builtin attributes through, are gone.
    for source in all_sources() {
        let (path, source) = source;
        for removed in [
            "emit_alloc_value_descriptor_from_locals",
            "emit_alloc_data_property_descriptor_object_from_locals",
        ] {
            assert!(!source.contains(removed), "{path} names `{removed}`");
        }
    }
    for wrapper in [
        "fn emit_alloc_data_descriptor_from_locals(",
        "fn emit_alloc_data_descriptor_from_locals_with_flag_locals(",
        "fn emit_alloc_accessor_descriptor_from_locals_with_flag_local(",
        "fn emit_create_data_property_descriptor_carrier(",
        "fn emit_from_complete_property_descriptor(",
        "fn emit_from_property_descriptor(",
    ] {
        let definitions = all_sources()
            .into_iter()
            .filter(|(_, source)| source.contains(wrapper))
            .map(|(path, _)| path)
            .collect::<Vec<_>>();
        assert_eq!(
            definitions,
            vec!["objects/descriptor_object.rs".to_string()],
            "`{wrapper}` must be defined only by the owner"
        );
    }

    // The census of producers. A new descriptor-object producer must be added
    // here, which is the point: it has to be written against the owner.
    let expected: &[(&str, &str, usize)] = &[
        (
            "builtins/object/get_own_property_descriptor.rs",
            "self.emit_alloc_data_descriptor_from_locals(",
            10,
        ),
        (
            "builtins/object/get_own_property_descriptor.rs",
            "self.emit_alloc_data_descriptor_from_locals_with_flag_locals(",
            5,
        ),
        (
            "builtins/object/get_own_property_descriptor.rs",
            "self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(",
            4,
        ),
        (
            "builtins/array.rs",
            "self.emit_alloc_data_descriptor_from_locals_with_flag_locals(",
            1,
        ),
        (
            "builtins/array.rs",
            "self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(",
            1,
        ),
        (
            "objects/module_namespace.rs",
            "self.emit_alloc_data_descriptor_from_locals_with_flag_locals(",
            1,
        ),
        (
            "builtins/array.rs",
            "self.emit_create_data_property_descriptor_carrier(",
            1,
        ),
        (
            "builtins/array_from_async.rs",
            "self.emit_create_data_property_descriptor_carrier(",
            1,
        ),
        (
            "builtins/json.rs",
            "self.emit_create_data_property_descriptor_carrier(",
            1,
        ),
        (
            "functions/class_definition.rs",
            "self.emit_create_data_property_descriptor_carrier(",
            1,
        ),
        (
            "objects.rs",
            "self.emit_create_data_property_descriptor_carrier(",
            2,
        ),
        ("objects.rs", "self.emit_from_property_descriptor(", 2),
        (
            "builtins/reflect/descriptor_object_prototype.rs",
            "self.emit_from_property_descriptor(",
            1,
        ),
        (
            "builtins/object/prototype_definition.rs",
            "self.emit_from_property_descriptor(",
            1,
        ),
        (
            "builtins/object.rs",
            "self.emit_from_property_descriptor(",
            3,
        ),
        (
            "builtins/object/get_own_property_descriptor/proxy.rs",
            "self.emit_from_property_descriptor(",
            1,
        ),
    ];
    let sources = all_sources();
    for entry in [
        "self.emit_alloc_data_descriptor_from_locals(",
        "self.emit_alloc_data_descriptor_from_locals_with_flag_locals(",
        "self.emit_alloc_accessor_descriptor_from_locals_with_flag_local(",
        "self.emit_create_data_property_descriptor_carrier(",
        "self.emit_from_property_descriptor(",
    ] {
        for (path, source) in &sources {
            if path == "objects/descriptor_object.rs" {
                continue;
            }
            let expected_count = expected
                .iter()
                .filter(|(file, marker, _)| file == path && *marker == entry)
                .map(|(_, _, count)| *count)
                .sum::<usize>();
            assert_eq!(
                source.matches(entry).count(),
                expected_count,
                "{path}: `{entry}`"
            );
        }
    }

    // Public 6.2.6.4 results take a Realm %Object.prototype%; engine-private
    // ToPropertyDescriptor arguments take the null-prototype carrier.
    assert!(REFLECT_DESCRIPTOR
        .contains("DescriptorObjectPrototype::ObjectPrototypeLocal(prototype_local),"));
    assert!(OBJECTS.contains("DescriptorObjectPrototype::ObjectPrototypeLocal(prototype_local),"));
    assert_eq!(
        PROTOTYPE_DEFINITION
            .matches("DescriptorObjectPrototype::PrivateCarrier,")
            .count(),
        1
    );
    assert_eq!(
        OBJECT_BUILTINS
            .matches("DescriptorObjectPrototype::PrivateCarrier,")
            .count(),
        3
    );
    assert_eq!(
        OBJECTS
            .matches("DescriptorObjectPrototype::PrivateCarrier,")
            .count(),
        1
    );
}
