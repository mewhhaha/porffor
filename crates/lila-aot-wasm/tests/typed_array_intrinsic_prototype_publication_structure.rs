const BOOTSTRAP: &str = include_str!("../src/builtins/bootstrap.rs");

fn typed_array_intrinsic() -> &'static str {
    BOOTSTRAP
        .split_once("    pub(crate) fn init_typed_array_intrinsic(")
        .expect("TypedArray intrinsic bootstrap")
        .1
        .split_once("    pub(crate) fn repair_typed_array_constructor_graph(")
        .expect("TypedArray intrinsic bootstrap end")
        .0
}

fn normalized(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

#[test]
fn typed_array_constructor_receives_its_prototype_at_birth() {
    let bootstrap = typed_array_intrinsic();
    let prototype_publication = bootstrap
        .split_once("self.emit_function_value_payload_with_prototype_materialization(")
        .expect("bootstrap-supplied TypedArray constructor")
        .1
        .split_once(
            "function.instruction(&Instruction::I64Const(self.strings.payload(\"constructor\")))",
        )
        .expect("TypedArray prototype publication end")
        .0;

    assert!(prototype_publication.contains("FunctionPrototypeMaterialization::BootstrapSupplied"));
    assert!(!prototype_publication.contains("self.emit_function_value_payload(function_meta"));
    assert_eq!(
        prototype_publication
            .matches("self.emit_object_append_data_property_with_flags(")
            .count(),
        1
    );
    assert!(prototype_publication.contains(
        "typed_array_constructor_local,\n            key_local,\n            typed_array_prototype_local,\n            tag_local,\n            false,\n            false,\n            false,"
    ));
    assert!(!prototype_publication.contains("self.emit_object_define_data("));

    let publication = normalized(prototype_publication);
    assert!(publication.contains(
        "function.instruction(&Instruction::I64Const(ValueKind::Object.tag()asi64));function.instruction(&Instruction::LocalSet(tag_local));function.instruction(&Instruction::GlobalGet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX));function.instruction(&Instruction::LocalSet(typed_array_prototype_local));self.store_i64_local_at_offset(typed_array_constructor_local,HEAP_FUNCTION_PROTOTYPE_TAG_OFFSET,tag_local,function,);self.store_i64_local_at_offset(typed_array_constructor_local,HEAP_FUNCTION_PROTOTYPE_PAYLOAD_OFFSET,typed_array_prototype_local,function,);"
    ));
}

#[test]
fn typed_array_methods_are_published_on_the_same_prototype() {
    let bootstrap = typed_array_intrinsic();
    let prototype_members = bootstrap
        .split_once("        let species_meta = self")
        .expect("TypedArray species publication")
        .1
        .split_once("        let from_meta = self")
        .expect("TypedArray static method publication")
        .0;
    let normalized_members = normalized(prototype_members);

    assert_eq!(
        bootstrap
            .matches("GlobalGet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX)")
            .count(),
        2
    );
    assert_eq!(
        normalized_members
            .matches(
                "self.emit_object_append_accessor_property_with_flags(typed_array_prototype_local,"
            )
            .count(),
        2,
        "the four named accessors and @@toStringTag must target the intrinsic prototype"
    );
    assert_eq!(
        normalized_members
            .matches("self.emit_object_define_function_data(typed_array_prototype_local,")
            .count(),
        prototype_members
            .matches("self.emit_object_define_function_data(")
            .count(),
        "every prototype method publication must target the intrinsic prototype"
    );
    assert_eq!(
        normalized_members
            .matches(
                "self.emit_object_define_function_data_with_aliases(typed_array_prototype_local,"
            )
            .count(),
        1
    );
    assert_eq!(
        normalized_members
            .matches("self.emit_object_define_function_global_data(typed_array_prototype_local,")
            .count(),
        1
    );
    for method in [
        "StandardBuiltinId::TypedArrayPrototypeValues",
        "StandardBuiltinId::TypedArrayPrototypeKeys",
        "StandardBuiltinId::TypedArrayPrototypeEntries",
    ] {
        assert!(
            bootstrap.contains(method),
            "missing publication for {method}"
        );
    }

    let reverse_link = normalized(
        bootstrap
            .split_once(
                "function.instruction(&Instruction::I64Const(self.strings.payload(\"constructor\")))",
            )
            .expect("TypedArray reverse constructor link")
            .1
            .split_once("        let species_meta = self")
            .expect("TypedArray reverse constructor link end")
            .0,
    );
    assert!(reverse_link.contains(
        "function.instruction(&Instruction::LocalGet(typed_array_constructor_local));function.instruction(&Instruction::LocalSet(payload_local));function.instruction(&Instruction::I64Const(ValueKind::Function.tag()asi64));function.instruction(&Instruction::LocalSet(tag_local));self.emit_object_append_data_property_with_flags(typed_array_prototype_local,key_local,payload_local,tag_local,true,false,true,function,)"
    ));
}

#[test]
fn typed_array_prototype_exists_before_hidden_and_concrete_constructors() {
    let runtime_roots = BOOTSTRAP
        .split_once("    pub(crate) fn init_runtime_roots(")
        .expect("runtime root bootstrap")
        .1
        .split_once("    pub(crate) fn init_script_global_object(")
        .expect("runtime root bootstrap end")
        .0;
    let allocation = runtime_roots
        .find("GlobalSet(TYPED_ARRAY_PROTOTYPE_GLOBAL_INDEX)")
        .expect("TypedArray prototype allocation");
    let hidden_constructor = runtime_roots
        .find("self.init_typed_array_intrinsic(function)?;")
        .expect("hidden TypedArray constructor initialization");
    let concrete_constructors = runtime_roots
        .find("for builtin in [\n            StandardBuiltinId::Float64ArrayConstructor")
        .expect("concrete TypedArray constructor initialization");

    assert!(allocation < hidden_constructor);
    assert!(hidden_constructor < concrete_constructors);
}
