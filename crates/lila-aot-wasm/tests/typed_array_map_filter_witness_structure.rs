const ARRAY_SOURCE: &str = include_str!("../src/builtins/array.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn map_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_typed_array_prototype_map_builtin(",
        "pub(crate) fn compile_typed_array_prototype_filter_builtin(",
    )
}

fn filter_body() -> &'static str {
    bounded(
        ARRAY_SOURCE,
        "pub(crate) fn compile_typed_array_prototype_filter_builtin(",
        "pub(crate) fn compile_typed_array_prototype_quantifier_builtin(",
    )
}

fn dispatcher_body() -> &'static str {
    bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::TypedArrayPrototypeSome => {",
        "StandardBuiltinId::ArrayPrototypeForEach => {",
    )
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn unique_position(body: &str, needle: &str, label: &str) -> usize {
    assert_eq!(
        body.matches(needle).count(),
        1,
        "{label} must occur exactly once"
    );
    body.find(needle)
        .unwrap_or_else(|| panic!("missing sentinel: {label}"))
}

fn positions(body: &str, needle: &str) -> Vec<usize> {
    body.match_indices(needle)
        .map(|(position, _)| position)
        .collect()
}

fn local_sequence<'a>(body: &'a str, prefix: &str, suffix: &str) -> Vec<&'a str> {
    body.lines()
        .filter_map(|line| line.trim().strip_prefix(prefix)?.strip_suffix(suffix))
        .collect()
}

fn assert_temp_lifetime(body: &str, label: &str) {
    let reservations = local_sequence(body, "let ", " = self.reserve_temp_local();");
    let releases = local_sequence(body, "self.release_temp_local(", ");");

    assert_eq!(
        body.matches("reserve_temp_local()").count(),
        reservations.len(),
        "{label} reservations must keep the reviewed binding shape"
    );
    assert_eq!(
        body.matches("release_temp_local(").count(),
        releases.len(),
        "{label} releases must keep the reviewed call shape"
    );

    let mut unique = reservations.clone();
    unique.sort_unstable();
    unique.dedup();
    assert_eq!(
        unique.len(),
        reservations.len(),
        "{label} must reserve every temporary exactly once"
    );

    let mut expected_releases = reservations;
    expected_releases.reverse();
    assert_eq!(
        releases, expected_releases,
        "{label} must release temporaries in reverse reservation order"
    );

    let result_payload = unique_position(
        body,
        "Instruction::LocalSet(self.result_local)",
        &format!("{label} result-payload publication"),
    );
    let result_tag = unique_position(
        body,
        "Instruction::LocalSet(self.result_tag_local)",
        &format!("{label} result-tag publication"),
    );
    let first_release = body
        .find("self.release_temp_local(")
        .unwrap_or_else(|| panic!("{label} must release its temporaries"));
    assert!(
        result_payload < result_tag && result_tag < first_release,
        "{label} must publish its payload and tag before releasing locals"
    );
}

#[test]
fn map_and_filter_each_consume_one_validated_entry_witness() {
    for (name, body, brand_message) in [
        (
            "map",
            map_body(),
            "TypedArray.prototype.map requires a TypedArray",
        ),
        (
            "filter",
            filter_body(),
            "TypedArray.prototype.filter requires a TypedArray",
        ),
    ] {
        for (needle, expected, label) in [
            (
                "emit_load_typed_array_private_state(",
                1,
                "private-state load",
            ),
            ("TypedArrayViewLocals::new(", 1, "immutable view"),
            ("emit_typed_array_witness(", 1, "buffer witness"),
            (
                "TypedArrayWitnessUse::ValidatedMethodEntry",
                1,
                "validated-entry projection",
            ),
            ("receiver_view", 2, "view producer and consumer"),
        ] {
            assert_eq!(
                body.matches(needle).count(),
                expected,
                "{name} must have exactly {expected} {label}"
            );
        }

        for forbidden in [
            "emit_validate_typed_array_current_byte_length(",
            "emit_typed_array_current_byte_length(",
            "emit_load_array_buffer_byte_length(",
            "emit_load_array_buffer_data(",
            "HEAP_TYPED_ARRAY_VIEWED_BUFFER_OFFSET",
            "HEAP_TYPED_ARRAY_BYTE_OFFSET",
            "HEAP_TYPED_ARRAY_BYTE_LENGTH_OFFSET",
            "HEAP_TYPED_ARRAY_BYTES_PER_ELEMENT_OFFSET",
            "HEAP_TYPED_ARRAY_LENGTH_TRACKING_OFFSET",
            "Instruction::I64DivU",
            "Instruction::LocalSet(length_local)",
        ] {
            assert!(
                !body.contains(forbidden),
                "{name} must not bypass the witness through {forbidden}"
            );
        }

        let brand = unique_position(body, brand_message, &format!("{name} brand error"));
        let private_state = unique_position(
            body,
            "emit_load_typed_array_private_state(",
            &format!("{name} private-state load"),
        );
        let view = unique_position(
            body,
            "TypedArrayViewLocals::new(",
            &format!("{name} immutable view"),
        );
        let witness = unique_position(
            body,
            "TypedArrayWitnessUse::ValidatedMethodEntry",
            &format!("{name} validated-entry witness"),
        );
        let element_kind = body
            .find("HEAP_TYPED_ARRAY_ELEMENT_KIND_OFFSET")
            .unwrap_or_else(|| panic!("{name} must retain its source element-kind load"));
        let callback = unique_position(
            body,
            "emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?",
            &format!("{name} callback validation"),
        );
        let callback_presence = body
            .find("Instruction::LocalGet(self.argc_param_local())")
            .unwrap_or_else(|| panic!("{name} must validate callback presence"));
        assert!(
            brand < private_state
                && private_state < view
                && view < witness
                && witness < element_kind
                && element_kind < callback_presence
                && callback_presence < callback,
            "{name} must validate its receiver and buffer before element-kind use and callback validation"
        );

        let normalized = without_whitespace(body)
            .replace(",)", ")")
            .replace(",]", "]");
        let witness_wiring = concat!(
            "self.emit_load_typed_array_private_state(receiver_payload_local,",
            "buffer_payload_local,byte_offset_local,byte_length_local,",
            "bytes_per_element_local,function);",
            "letreceiver_view=TypedArrayViewLocals::new(receiver_payload_local,",
            "buffer_payload_local,byte_offset_local,byte_length_local,",
            "bytes_per_element_local);",
            "self.emit_typed_array_witness(&receiver_view,",
            "TypedArrayWitnessUse::ValidatedMethodEntry{length_local},function)?;"
        );
        assert_eq!(
            normalized.matches(witness_wiring).count(),
            1,
            "{name} must wire the receiver slots and captured length to its sole witness without transposition"
        );

        assert_temp_lifetime(body, name);
    }
}

#[test]
fn map_and_filter_preserve_their_distinct_species_and_callback_orders() {
    let map = map_body();
    let filter = filter_body();

    assert_eq!(
        map.matches("Instruction::Loop(BlockType::Empty)").count(),
        1,
        "map must retain one captured-length callback/write loop"
    );
    assert_eq!(
        filter
            .matches("Instruction::Loop(BlockType::Empty)")
            .count(),
        2,
        "filter must retain its callback-collection and selected-write loops"
    );

    for (name, body, expected_length_reads, index_tag_local, result_locals) in [
        (
            "map",
            map,
            2,
            "length_tag_local",
            ("mapped_payload_local", "mapped_tag_local"),
        ),
        (
            "filter",
            filter,
            1,
            "number_tag_local",
            ("predicate_payload_local", "predicate_tag_local"),
        ),
    ] {
        assert_eq!(
            body.matches("emit_typed_array_or_object_index_read_from_locals(")
                .count(),
            1,
            "{name} must perform one live source read in its callback loop"
        );
        assert_eq!(
            body.matches("emit_function_or_proxy_call_leave_throw_completion(")
                .count(),
            1,
            "{name} must retain one Proxy-aware callback site"
        );
        assert_eq!(
            body.matches("Instruction::LocalGet(length_local)").count(),
            expected_length_reads,
            "{name} must retain only its reviewed species/loop uses of the captured length"
        );

        let normalized = without_whitespace(body)
            .replace(",)", ")")
            .replace(",]", "]");
        let callback_arguments = format!(
            "&[(element_payload_local,element_tag_local),(index_number_payload_local,{index_tag_local}),(receiver_payload_local,receiver_tag_local)]"
        );
        let (result_payload_local, result_tag_local) = result_locals;
        let callback_call = format!(
            "self.emit_function_or_proxy_call_leave_throw_completion(callback_payload_local,callback_tag_local,this_arg_payload_local,this_arg_tag_local,{callback_arguments},{result_payload_local},{result_tag_local},function)?;"
        );
        assert_eq!(
            normalized.matches(callback_call.as_str()).count(),
            1,
            "{name} callback must receive exactly value, numeric index and the original receiver in that order"
        );
    }

    let map_callback_validation = unique_position(
        map,
        "emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?",
        "map callback validation",
    );
    let map_species = unique_position(
        map,
        "property_key_symbol_payload(\"Symbol.species\")",
        "map species lookup",
    );
    let map_construct = unique_position(
        map,
        "emit_function_or_proxy_construct_with_argv(",
        "map species target construction",
    );
    let map_target_validation = unique_position(
        map,
        "emit_validate_typed_array_from_constructed_target(",
        "map species target validation",
    );
    let map_loop = unique_position(
        map,
        "Instruction::Loop(BlockType::Empty)",
        "map callback loop",
    );
    let map_read = unique_position(
        map,
        "emit_typed_array_or_object_index_read_from_locals(",
        "map live source read",
    );
    let map_call = unique_position(
        map,
        "emit_function_or_proxy_call_leave_throw_completion(",
        "map callback call",
    );
    let map_write = unique_position(
        map,
        "emit_typed_array_element_write_from_locals(",
        "map target write",
    );
    let map_backedge = unique_position(map, "Instruction::Br(0)", "map loop back-edge");
    let map_result_publication = unique_position(
        map,
        "Instruction::LocalSet(self.result_local)",
        "map result publication",
    );
    assert_eq!(
        positions(
            &map[map_backedge..map_result_publication],
            "Instruction::End"
        )
        .len(),
        2,
        "the map back-edge must be followed by its loop and block closure before result publication"
    );
    assert!(
        map_callback_validation < map_species
            && map_species < map_construct
            && map_construct < map_target_validation
            && map_target_validation < map_loop
            && map_loop < map_read
            && map_read < map_call
            && map_call < map_write
            && map_write < map_backedge
            && map_backedge < map_result_publication,
        "map must create its species target before its live callback/write loop"
    );

    let filter_callback_validation = unique_position(
        filter,
        "emit_is_callable_i32(callback_tag_local, callback_payload_local, function)?",
        "filter callback validation",
    );
    let filter_read = unique_position(
        filter,
        "emit_typed_array_or_object_index_read_from_locals(",
        "filter live source read",
    );
    let filter_call = unique_position(
        filter,
        "emit_function_or_proxy_call_leave_throw_completion(",
        "filter callback call",
    );
    let filter_loop_positions = positions(filter, "Instruction::Loop(BlockType::Empty)");
    assert_eq!(
        filter_loop_positions.len(),
        2,
        "filter must retain exactly its collection and selected-write loops"
    );
    let filter_backedge_positions = positions(filter, "Instruction::Br(0)");
    assert_eq!(
        filter_backedge_positions.len(),
        2,
        "filter loops must retain exactly two back-edges"
    );
    let filter_species = unique_position(
        filter,
        "property_key_symbol_payload(\"Symbol.species\")",
        "filter species lookup",
    );
    let filter_constructor_selection = unique_position(
        filter,
        "for (constructor, _) in typed_array_constructor_bytes_per_element_entries()",
        "filter intrinsic constructor selection",
    );
    let filter_construct = unique_position(
        filter,
        "emit_function_or_proxy_construct_with_argv(",
        "filter species target construction",
    );
    let filter_target_validation = unique_position(
        filter,
        "emit_validate_typed_array_from_constructed_target(",
        "filter species target validation",
    );
    let filter_write = unique_position(
        filter,
        "emit_typed_array_element_write_from_locals(",
        "filter selected-value target write",
    );
    let first_loop_closers = positions(
        &filter[filter_backedge_positions[0]..filter_constructor_selection],
        "Instruction::End",
    );
    let result_publication = unique_position(
        filter,
        "Instruction::LocalSet(self.result_local)",
        "filter result publication",
    );
    let second_loop_closers = positions(
        &filter[filter_backedge_positions[1]..result_publication],
        "Instruction::End",
    );
    assert_eq!(
        (first_loop_closers.len(), second_loop_closers.len()),
        (2, 2),
        "each filter loop back-edge must be followed by its loop and block closure before the next algorithm phase"
    );
    assert!(
        filter_callback_validation < filter_loop_positions[0]
            && filter_loop_positions[0] < filter_read
            && filter_read < filter_call
            && filter_call < filter_backedge_positions[0]
            && filter_backedge_positions[0] < filter_constructor_selection
            && filter_constructor_selection < filter_species
            && filter_species < filter_construct
            && filter_construct < filter_target_validation
            && filter_target_validation < filter_loop_positions[1]
            && filter_loop_positions[1] < filter_write
            && filter_write < filter_backedge_positions[1]
            && filter_backedge_positions[1] < result_publication,
        "filter must finish its live callback collection before species creation and selected writes"
    );
}

#[test]
fn map_and_filter_dispatchers_have_one_swap_resistant_owner_each() {
    let dispatcher = dispatcher_body();
    let normalized = without_whitespace(dispatcher).replace(",)", ")");

    for (builtin, compiler) in [
        (
            "TypedArrayPrototypeMap",
            "compile_typed_array_prototype_map_builtin",
        ),
        (
            "TypedArrayPrototypeFilter",
            "compile_typed_array_prototype_filter_builtin",
        ),
    ] {
        assert_eq!(
            dispatcher
                .matches(&format!("StandardBuiltinId::{builtin}"))
                .count(),
            1,
            "StandardBuiltinId::{builtin} must have one dispatcher owner"
        );
        assert_eq!(
            dispatcher.matches(&format!("self.{compiler}(")).count(),
            1,
            "{compiler} must have one dispatcher call"
        );

        let mapping = format!("StandardBuiltinId::{builtin}=>{{self.{compiler}(function)?;}}");
        assert_eq!(
            normalized.matches(mapping.as_str()).count(),
            1,
            "StandardBuiltinId::{builtin} must map to {compiler}"
        );
    }

    assert_eq!(
        ARRAY_SOURCE
            .matches("fn compile_typed_array_prototype_map_builtin(")
            .count(),
        1
    );
    assert_eq!(
        ARRAY_SOURCE
            .matches("fn compile_typed_array_prototype_filter_builtin(")
            .count(),
        1
    );
}
