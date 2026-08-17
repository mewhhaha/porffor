#[test]
fn date_time_format_reservation_is_tagged_ordered_and_one_way() {
    let source = include_str!("../src/builtins/intl_datetimeformat.rs");
    let functions = include_str!("../src/functions.rs");

    for state in [
        "ReservedIntlDateTimeFormatObjectLocal",
        "InitializedIntlDateTimeFormatObjectLocal",
    ] {
        let declaration = format!("struct {state}(u32);");
        assert_eq!(source.matches(&declaration).count(), 1);
        let before = source
            .split_once(&declaration)
            .expect("DateTimeFormat lifecycle state should exist")
            .0;
        let attributes = before
            .rsplit_once("\n\n")
            .expect("lifecycle state should be separated from its predecessor")
            .1;
        assert!(attributes.contains("#[must_use]"));
        assert!(
            !attributes.contains("derive"),
            "{state} must remain non-Copy"
        );
    }

    let reserve = source
        .split_once("fn emit_reserve_intl_date_time_format_object(")
        .expect("DateTimeFormat reserve transition should exist")
        .1
        .split_once("/// Consume the unreachable reserved result")
        .expect("DateTimeFormat reserve transition should be bounded")
        .0;
    let initializer = source
        .split_once("fn emit_initialize_intl_date_time_format_object(")
        .expect("DateTimeFormat initialize transition should exist")
        .1
        .split_once("/// Publish the only DateTimeFormat lifecycle state")
        .expect("DateTimeFormat initialize transition should be bounded")
        .0;
    let publisher = source
        .split_once("fn emit_publish_intl_date_time_format_object(")
        .expect("DateTimeFormat publish transition should exist")
        .1
        .split_once("pub(crate) fn emit_intl_date_time_format_constructor(")
        .expect("DateTimeFormat publish transition should be bounded")
        .0;
    let constructor = source
        .split_once("pub(crate) fn emit_intl_date_time_format_constructor(")
        .expect("DateTimeFormat constructor should exist")
        .1
        .split_once("fn emit_intl_dtf_note_component_present(")
        .expect("DateTimeFormat constructor should be bounded")
        .0;

    let retained = reserve
        .find("let object_payload_local = self.reserve_temp_local();")
        .expect("retained result local should be reserved");
    let prototype_payload = reserve
        .find("let prototype_payload_local = self.reserve_temp_local();")
        .expect("prototype payload local should be reserved");
    let prototype_tag = reserve
        .find("let prototype_tag_local = self.reserve_temp_local();")
        .expect("prototype tag local should be reserved");
    let tagged_pair = reserve
        .find("let prototype = TaggedLocals::new(prototype_payload_local, prototype_tag_local);")
        .expect("prototype payload and tag should remain paired");
    assert!(retained < prototype_payload);
    assert!(prototype_payload < prototype_tag);
    assert!(prototype_tag < tagged_pair);
    assert_eq!(
        reserve
            .matches("emit_new_target_prototype_to_locals(")
            .count(),
        1
    );
    assert_eq!(
        reserve
            .matches("NewTargetPrototypeFallback::CurrentGlobal")
            .count(),
        1
    );
    assert_eq!(
        reserve
            .matches("emit_alloc_plain_object_with_prototype_and_tag(")
            .count(),
        1
    );
    assert!(reserve.contains("Some(prototype.payload),"));
    assert!(reserve.contains("Some(prototype.tag),"));
    let tag_release = reserve
        .find("self.release_temp_local(prototype.tag);")
        .expect("prototype tag should be released");
    let payload_release = reserve
        .find("self.release_temp_local(prototype.payload);")
        .expect("prototype payload should be released");
    assert!(tag_release < payload_release);

    assert!(initializer.contains("reserved: ReservedIntlDateTimeFormatObjectLocal"));
    assert!(initializer.contains("-> InitializedIntlDateTimeFormatObjectLocal"));
    assert!(initializer.contains("OBJECT_INTERNAL_BRAND_INTL_DATE_TIME_FORMAT"));
    assert!(initializer.contains("HEAP_OBJECT_BOXED_PAYLOAD_OFFSET"));
    assert!(publisher.contains("initialized: InitializedIntlDateTimeFormatObjectLocal"));
    assert!(publisher.contains("Instruction::LocalSet(self.result_local)"));
    assert!(publisher.contains("self.release_temp_local(initialized.0);"));

    let reserve_call = constructor
        .find("let reserved_object = self.emit_reserve_intl_date_time_format_object(function)?;")
        .expect("constructor should reserve its result");
    let locale_observation = constructor
        .find("self.emit_builtin_arg_to_locals(0,")
        .expect("constructor should load locales");
    let options_observation = constructor
        .find("self.emit_builtin_arg_to_locals(1,")
        .expect("constructor should load options");
    assert!(reserve_call < locale_observation);
    assert!(locale_observation < options_observation);

    let completed_record = constructor
        .find("HEAP_INTL_DTF_BOUND_FORMAT_OFFSET")
        .expect("constructor should complete its represented record");
    let initialize_call = constructor
        .find("self.emit_initialize_intl_date_time_format_object(")
        .expect("constructor should initialize its reserved result");
    let publish_call = constructor
        .find("self.emit_publish_intl_date_time_format_object(initialized_object, function);")
        .expect("constructor should publish its initialized result");
    assert!(completed_record < initialize_call);
    assert!(initialize_call < publish_call);
    assert_eq!(
        constructor
            .matches("emit_reserve_intl_date_time_format_object(")
            .count(),
        1
    );
    assert_eq!(
        constructor
            .matches("emit_initialize_intl_date_time_format_object(")
            .count(),
        1
    );
    assert_eq!(
        constructor
            .matches("emit_publish_intl_date_time_format_object(")
            .count(),
        1
    );
    for forbidden in [
        "emit_error_new_target_prototype_to_local(",
        "Instruction::LocalSet(self.result_local)",
    ] {
        assert!(
            !constructor.contains(forbidden),
            "constructor bypassed its typed lifecycle through {forbidden}"
        );
    }
    assert_eq!(
        constructor
            .matches("emit_alloc_plain_object_with_prototype(None, None, function)?;")
            .count(),
        1,
        "the sole untagged allocation is the internal default options object"
    );
    assert_eq!(
        constructor
            .matches("emit_alloc_plain_object_with_prototype(")
            .count(),
        1,
        "no second untagged allocation may bypass the reserved result"
    );
    let default_options_allocation = constructor
        .find("emit_alloc_plain_object_with_prototype(None, None, function)?;")
        .expect("undefined options should allocate their internal default object");
    assert!(options_observation < default_options_allocation);

    let construct = functions
        .split_once("pub(crate) fn emit_function_handle_construct_with_argv(")
        .expect("shared construct dispatcher should exist")
        .1
        .split_once("pub(crate) fn emit_function_handle_call_with_argv(")
        .expect("shared construct dispatcher should be bounded")
        .0;
    let direct_domain = construct
        .split_once("let direct_returning_constructor_table_indices: Vec<i64> = [")
        .expect("direct-returning constructor domain should exist")
        .1
        .split_once("]\n        .into_iter()")
        .expect("direct-returning constructor domain should be bounded")
        .0;
    assert_eq!(
        direct_domain
            .matches("StandardBuiltinId::IntlDateTimeFormatConstructor,")
            .count(),
        1,
        "DateTimeFormat must enter its body before generic receiver allocation"
    );
    let direct_dispatch = construct
        .find("for table_index in direct_returning_constructor_table_indices {")
        .expect("direct-returning constructor dispatch should exist");
    let generic_prototype_get = construct
        .find("self.strings.payload(\"prototype\")")
        .expect("generic constructor path should read NewTarget.prototype");
    let generic_preallocation = construct
        .find("self.emit_alloc_plain_object_with_prototype_and_tag(")
        .expect("generic constructor path should allocate its receiver");
    assert!(direct_dispatch < generic_prototype_get);
    assert!(generic_prototype_get < generic_preallocation);
    let direct_dispatch_body = &construct[direct_dispatch..generic_prototype_get];
    assert!(direct_dispatch_body.contains("Instruction::CallIndirect {"));
    assert!(
        direct_dispatch_body.contains("function.instruction(&Instruction::Br(1));"),
        "a direct-returning constructor must leave the generic construct block"
    );
}
