const HOST_SOURCE: &str = include_str!("../src/builtins/host.rs");
const OWNER_SOURCE: &str = include_str!("../src/builtins/host/created_realm_iterator_next.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source
        .chars()
        .filter(|character| !character.is_whitespace())
        .collect()
}

fn materializer() -> &'static str {
    bounded(
        OWNER_SOURCE,
        "    pub(super) fn emit_materialize_created_realm_iterator_next(",
        "    pub(super) fn emit_publish_created_realm_iterator_next(",
    )
}

fn publisher() -> &'static str {
    OWNER_SOURCE
        .split_once("    pub(super) fn emit_publish_created_realm_iterator_next(")
        .expect("created-Realm iterator-next publisher")
        .1
        .split_once("\n    }\n}")
        .expect("created-Realm iterator-next publisher end")
        .0
}

#[test]
fn publication_context_and_token_have_one_private_move_only_owner() {
    assert_eq!(
        HOST_SOURCE
            .matches("\nmod created_realm_iterator_next;\n")
            .count(),
        1
    );
    assert!(!HOST_SOURCE.contains("pub mod created_realm_iterator_next;"));
    assert_eq!(
        HOST_SOURCE
            .matches("use created_realm_iterator_next::{")
            .count(),
        1
    );
    assert!(HOST_SOURCE
        .contains("CreatedRealmIteratorNextPublicationContext, CreatedRealmIteratorNextTarget,"));
    assert!(!HOST_SOURCE.contains("CreatedRealmIteratorNext {"));

    let context = bounded(
        OWNER_SOURCE,
        "pub(super) struct CreatedRealmIteratorNextPublicationContext<'a> {",
        "impl<'a> CreatedRealmIteratorNextPublicationContext<'a> {",
    );
    for field in [
        "realm_functions: &'a RealmFunctionMaterializationContext,",
        "type_error_prototype_local: u32,",
        "array_iterator_prototype_local: u32,",
        "string_iterator_prototype_local: u32,",
        "map_iterator_prototype_local: u32,",
        "set_iterator_prototype_local: u32,",
    ] {
        assert_eq!(context.matches(field).count(), 1, "{field}");
    }
    assert!(!context.contains("pub realm_functions:"));
    assert!(!context.contains("pub type_error_prototype_local:"));
    assert!(!context.contains("pub array_iterator_prototype_local:"));
    assert!(!context.contains("pub string_iterator_prototype_local:"));
    assert!(!context.contains("pub map_iterator_prototype_local:"));
    assert!(!context.contains("pub set_iterator_prototype_local:"));
    let constructor = bounded(
        OWNER_SOURCE,
        "    pub(super) fn new(",
        "    fn prototype_local(&self, target: &CreatedRealmIteratorNextTarget) -> u32 {",
    );
    let constructor = without_whitespace(constructor);
    assert!(constructor.contains(concat!(
        "Self{realm_functions,type_error_prototype_local,",
        "array_iterator_prototype_local,string_iterator_prototype_local,",
        "map_iterator_prototype_local,set_iterator_prototype_local,}"
    )));

    assert!(OWNER_SOURCE
        .contains("#[must_use = \"created-Realm iterator next functions must be published\"]"));
    let token = bounded(
        OWNER_SOURCE,
        "pub(super) struct CreatedRealmIteratorNext {",
        "impl<'a> FunctionBuilder<'a> {",
    );
    assert_eq!(token.matches("prototype_local: u32,").count(), 1);
    assert_eq!(token.matches("function_local: u32,").count(), 1);
    assert!(!token.contains("pub prototype_local:"));
    assert!(!token.contains("pub function_local:"));
    assert!(!OWNER_SOURCE.contains("#[derive("));
    assert!(!OWNER_SOURCE.contains("impl Clone"));
    assert!(!OWNER_SOURCE.contains("impl Copy"));
}

#[test]
fn target_domain_maps_only_the_four_iterator_next_builtins() {
    let target = bounded(
        OWNER_SOURCE,
        "pub(super) enum CreatedRealmIteratorNextTarget {",
        "impl CreatedRealmIteratorNextTarget {",
    );
    for variant in ["Array", "String", "Map", "Set"] {
        assert_eq!(target.matches(&format!("    {variant},")).count(), 1);
    }
    assert!(!target.contains("u32"));
    assert!(!target.contains('{'));

    let builtin = bounded(
        OWNER_SOURCE,
        "    fn builtin(&self) -> StandardBuiltinId {",
        "pub(super) struct CreatedRealmIteratorNextPublicationContext<'a> {",
    );
    for (variant, builtin_id) in [
        ("Array", "ArrayIteratorNext"),
        ("String", "StringIteratorNext"),
        ("Map", "MapIteratorNext"),
        ("Set", "SetIteratorNext"),
    ] {
        let mapping = format!("Self::{variant} => StandardBuiltinId::{builtin_id},");
        assert_eq!(builtin.matches(&mapping).count(), 1, "{variant} mapping");
    }
    assert_eq!(builtin.matches("StandardBuiltinId::").count(), 4);
    assert!(!builtin.contains("_ =>"));

    let prototype = bounded(
        OWNER_SOURCE,
        "    fn prototype_local(&self, target: &CreatedRealmIteratorNextTarget) -> u32 {",
        "#[must_use = \"created-Realm iterator next functions must be published\"]",
    );
    for (variant, field) in [
        ("Array", "array_iterator_prototype_local"),
        ("String", "string_iterator_prototype_local"),
        ("Map", "map_iterator_prototype_local"),
        ("Set", "set_iterator_prototype_local"),
    ] {
        let mapping = format!("CreatedRealmIteratorNextTarget::{variant} => self.{field},");
        assert_eq!(
            prototype.matches(&mapping).count(),
            1,
            "{variant} prototype"
        );
    }
    assert_eq!(
        prototype
            .matches("CreatedRealmIteratorNextTarget::")
            .count(),
        4
    );
    assert!(!prototype.contains("_ =>"));
    assert!(!OWNER_SOURCE.contains("ArrayIteratorIdentity"));
}

#[test]
fn materialization_records_identity_then_the_defining_type_error_realm() {
    let materializer = materializer();
    let signature = materializer
        .split_once(") -> Result<CreatedRealmIteratorNext, EmitError> {")
        .expect("created-Realm iterator-next materializer signature")
        .0;
    assert_eq!(
        without_whitespace(signature),
        concat!(
            "&mutself,target:CreatedRealmIteratorNextTarget,",
            "context:&CreatedRealmIteratorNextPublicationContext<'_>,",
            "function:&mutFunction,"
        )
    );
    for signature_part in [
        "target: CreatedRealmIteratorNextTarget,",
        "context: &CreatedRealmIteratorNextPublicationContext<'_>,",
        ") -> Result<CreatedRealmIteratorNext, EmitError> {",
    ] {
        assert!(materializer.contains(signature_part), "{signature_part}");
    }
    assert!(!materializer.contains("realm_functions: &RealmFunctionMaterializationContext,"));
    assert!(!materializer.contains("type_error_prototype_local: u32,"));
    assert_eq!(
        materializer
            .matches("self.store_i64_local_at_offset(")
            .count(),
        2
    );
    assert_eq!(materializer.matches("self.reserve_temp_local()").count(), 1);
    assert!(!materializer.contains("emit_object_define_local_data("));
    assert!(!materializer.contains("release_temp_local("));

    let materializer = without_whitespace(materializer);
    let materialize = materializer
        .find(concat!(
            "self.emit_function_value_payload_in_realm(&meta,context.realm_functions,",
            "function_local,function,)?;"
        ))
        .expect("iterator-next function materialization");
    let environment_store = materializer
        .find(concat!(
            "self.store_i64_local_at_offset(function_local,",
            "HEAP_FUNCTION_ENV_HANDLE_OFFSET,function_local,function,);"
        ))
        .expect("self-backed function environment store");
    let type_error_store = materializer
        .find(concat!(
            "self.store_i64_local_at_offset(function_local,",
            "HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET,",
            "context.type_error_prototype_local,function,);"
        ))
        .expect("defining-Realm TypeError prototype store");
    let token = materializer
        .find("Ok(CreatedRealmIteratorNext{prototype_local,function_local,})")
        .expect("initialized iterator-next publication token");
    assert!(materializer.contains("letbuiltin=target.builtin();"));
    assert!(materializer.contains("letprototype_local=context.prototype_local(&target);"));
    assert!(materialize < environment_store);
    assert!(environment_store < type_error_store);
    assert!(type_error_store < token);
}

#[test]
fn publication_consumes_the_token_then_releases_its_owned_locals() {
    let publisher = publisher();
    let signature = publisher
        .split_once(") -> Result<(), EmitError> {")
        .expect("created-Realm iterator-next publisher signature")
        .0;
    assert_eq!(
        without_whitespace(signature),
        "&mutself,iterator_next:CreatedRealmIteratorNext,function:&mutFunction,"
    );
    assert!(publisher.contains("iterator_next: CreatedRealmIteratorNext,"));
    assert!(!publisher.contains("iterator_next: &CreatedRealmIteratorNext"));
    assert!(!publisher.contains("prototype_local: u32"));
    assert!(!publisher.contains("HEAP_FUNCTION_ENV_HANDLE_OFFSET"));
    assert!(!publisher.contains("HEAP_FUNCTION_REALM_TYPE_ERROR_PROTOTYPE_OFFSET"));
    assert_eq!(publisher.matches("self.reserve_temp_local()").count(), 1);
    assert_eq!(
        publisher
            .matches("self.emit_object_define_local_data(")
            .count(),
        1
    );
    assert_eq!(publisher.matches("self.release_temp_local(").count(), 2);

    let publisher = without_whitespace(publisher);
    let destructure = publisher
        .find("letCreatedRealmIteratorNext{prototype_local,function_local,}=iterator_next;")
        .expect("consumed iterator-next publication token");
    let function_tag = publisher
        .find("Instruction::I64Const(ValueKind::Function.tag()asi64)")
        .expect("function publication tag");
    let tag_store = publisher
        .find("function.instruction(&Instruction::LocalSet(tag_local));")
        .expect("function publication tag local store");
    let publication = publisher
        .find(concat!(
            "self.emit_object_define_local_data(prototype_local,\"next\",",
            "function_local,tag_local,function,)?;"
        ))
        .expect("literal next publication");
    let release_tag = publisher
        .find("self.release_temp_local(tag_local);")
        .expect("publication tag release");
    let release_function = publisher
        .find("self.release_temp_local(function_local);")
        .expect("function payload release");
    let complete = publisher.find("Ok(())").expect("publication completion");
    assert_eq!(
        publisher
            .matches(concat!(
                "function.instruction(&Instruction::I64Const(",
                "ValueKind::Function.tag()asi64));",
                "function.instruction(&Instruction::LocalSet(tag_local));",
                "self.emit_object_define_local_data(prototype_local,\"next\",",
                "function_local,tag_local,function,)?;"
            ))
            .count(),
        1
    );
    assert_eq!(
        publisher
            .matches("function.instruction(&Instruction::LocalSet(tag_local));")
            .count(),
        1
    );
    assert!(destructure < function_tag);
    assert!(function_tag < tag_store);
    assert!(tag_store < publication);
    assert!(publication < release_tag);
    assert!(release_tag < release_function);
    assert!(release_function < complete);
}

#[test]
fn created_realm_bootstrap_uses_the_exact_four_typed_targets() {
    let create_realm = bounded(
        HOST_SOURCE,
        "    pub(crate) fn compile_host_create_realm_builtin(",
        "    pub(crate) fn compile_host_realm_eval_script_builtin(",
    );
    assert_eq!(
        create_realm
            .matches("self.emit_materialize_created_realm_iterator_next(")
            .count(),
        4
    );
    assert_eq!(
        create_realm
            .matches("self.emit_publish_created_realm_iterator_next(")
            .count(),
        4
    );
    assert_eq!(
        create_realm
            .matches("CreatedRealmIteratorNextTarget::")
            .count(),
        4
    );
    assert_eq!(
        create_realm
            .matches("CreatedRealmIteratorNextPublicationContext::new(")
            .count(),
        1
    );

    let identity = bounded(
        create_realm,
        "        let array_iterator_identity_payload_local = self.reserve_temp_local();",
        "        self.emit_object_define_string_data(\n            array_iterator_prototype_local,",
    );
    assert!(identity.contains("&array_iterator_identity_meta,"));
    assert!(identity.contains("\"Symbol.iterator\","));
    assert!(!identity.contains("CreatedRealmIteratorNextTarget"));
    assert!(!identity.contains("emit_materialize_created_realm_iterator_next"));
    assert!(!identity.contains("emit_publish_created_realm_iterator_next"));

    let create_realm = without_whitespace(create_realm);
    let context = concat!(
        "letiterator_next_publication_context=",
        "CreatedRealmIteratorNextPublicationContext::new(&realm_functions,",
        "type_error_prototype_local,array_iterator_prototype_local,",
        "string_iterator_prototype_local,map_iterator_prototype_local,",
        "set_iterator_prototype_local,);"
    );
    assert_eq!(create_realm.matches(context).count(), 1);
    let context_position = create_realm
        .find(context)
        .expect("single created-Realm iterator-next context construction");

    for (name, variant) in [
        ("array", "Array"),
        ("string", "String"),
        ("map", "Map"),
        ("set", "Set"),
    ] {
        let lifecycle = format!(
            concat!(
                "let{name}_iterator_next=self.emit_materialize_created_realm_iterator_next(",
                "CreatedRealmIteratorNextTarget::{variant},",
                "&iterator_next_publication_context,function,)?;",
                "self.emit_publish_created_realm_iterator_next({name}_iterator_next,function)?;"
            ),
            name = name,
            variant = variant
        );
        assert_eq!(
            create_realm.matches(&lifecycle).count(),
            1,
            "{variant} lifecycle"
        );
        assert!(context_position < create_realm.find(&lifecycle).expect("typed lifecycle"));
    }
}
