const COLLECTIONS_PARENT: &str = include_str!("../src/builtins/collections.rs");
const MAP_GET_OR_INSERT: &str = include_str!("../src/builtins/collections/map_get_or_insert.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}`"))
        .0
}

fn assert_before(source: &str, earlier: &str, later: &str) {
    let earlier_offset = source
        .find(earlier)
        .unwrap_or_else(|| panic!("missing earlier operation `{earlier}`"));
    let later_offset = source
        .find(later)
        .unwrap_or_else(|| panic!("missing later operation `{later}`"));
    assert!(
        earlier_offset < later_offset,
        "`{earlier}` must precede `{later}`"
    );
}

fn get_or_insert_emitter() -> &'static str {
    bounded(
        MAP_GET_OR_INSERT,
        "    pub(crate) fn emit_map_prototype_get_or_insert(",
        "\n    }\n}",
    )
}

#[test]
fn value_source_is_one_private_closed_domain() {
    let declaration = bounded(
        MAP_GET_OR_INSERT,
        "enum MapGetOrInsertValueSource {",
        "\n}\n\nimpl<'a> FunctionBuilder<'a> {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .take_while(|line| *line != "}")
        .collect::<Vec<_>>();
    assert_eq!(variants, ["ValueArgument,", "ComputedCallback,"]);
    assert!(!declaration.contains("pub"));
    assert!(!declaration.contains("bool"));
    assert!(!MAP_GET_OR_INSERT.contains("impl Default for MapGetOrInsertValueSource"));
    assert_eq!(
        COLLECTIONS_PARENT.matches("mod map_get_or_insert;").count(),
        1
    );
    assert!(!COLLECTIONS_PARENT.contains("MapGetOrInsertValueSource"));
    assert!(!COLLECTIONS_PARENT.contains("emit_map_prototype_get_or_insert_inner("));
    assert!(!COLLECTIONS_PARENT.contains("map_get_or_insert::"));
}

#[test]
fn named_wrappers_own_the_collection_and_value_source_pairings() {
    for (start, end, collection, value_source) in [
        (
            "    pub(crate) fn emit_map_prototype_get_or_insert(",
            "    pub(crate) fn emit_map_prototype_get_or_insert_computed(",
            "MapCollectionKind::Map",
            "MapGetOrInsertValueSource::ValueArgument",
        ),
        (
            "    pub(crate) fn emit_map_prototype_get_or_insert_computed(",
            "    pub(crate) fn emit_weak_map_prototype_get_or_insert(",
            "MapCollectionKind::Map",
            "MapGetOrInsertValueSource::ComputedCallback",
        ),
        (
            "    pub(crate) fn emit_weak_map_prototype_get_or_insert(",
            "    pub(crate) fn emit_weak_map_prototype_get_or_insert_computed(",
            "MapCollectionKind::WeakMap",
            "MapGetOrInsertValueSource::ValueArgument",
        ),
        (
            "    pub(crate) fn emit_weak_map_prototype_get_or_insert_computed(",
            "    fn emit_map_prototype_get_or_insert_inner(",
            "MapCollectionKind::WeakMap",
            "MapGetOrInsertValueSource::ComputedCallback",
        ),
    ] {
        let wrapper = bounded(MAP_GET_OR_INSERT, start, end);
        assert_eq!(wrapper.matches(collection).count(), 1, "wrapper `{start}`");
        assert_eq!(
            wrapper.matches(value_source).count(),
            1,
            "wrapper `{start}`"
        );
        assert!(!wrapper.contains("true"), "wrapper `{start}`");
        assert!(!wrapper.contains("false"), "wrapper `{start}`");
    }

    assert_eq!(
        MAP_GET_OR_INSERT
            .matches("emit_map_prototype_get_or_insert_inner(")
            .count(),
        5,
        "the shared emitter must have four named wrappers and one definition"
    );
}

#[test]
fn shared_emitter_exhaustively_preserves_value_source_ordering() {
    let emitter = get_or_insert_emitter();
    let signature = bounded(
        emitter,
        "    fn emit_map_prototype_get_or_insert_inner(",
        ") -> Result<(), EmitError> {",
    );
    assert!(signature.contains("value_source: MapGetOrInsertValueSource,"));
    assert!(!signature.contains("computed"));
    assert!(!signature.contains("bool"));

    assert_eq!(emitter.matches("match value_source {").count(), 2);
    assert_eq!(
        emitter
            .matches("MapGetOrInsertValueSource::ValueArgument =>")
            .count(),
        2
    );
    assert_eq!(
        emitter
            .matches("MapGetOrInsertValueSource::ComputedCallback =>")
            .count(),
        2
    );
    assert!(!emitter.contains("_ =>"));
    assert!(!emitter.contains("unreachable!"));
    assert!(!emitter.contains("if computed"));
    assert!(!emitter.contains("!computed"));
    assert_eq!(
        MAP_GET_OR_INSERT
            .matches("MapGetOrInsertValueSource")
            .count(),
        10
    );
    assert_eq!(
        MAP_GET_OR_INSERT
            .matches("MapGetOrInsertValueSource::")
            .count(),
        8
    );

    let preparation = bounded(
        emitter,
        "        match value_source {",
        "        function.instruction(&Instruction::LocalGet(key_tag_local));",
    );
    let value_argument = bounded(
        preparation,
        "            MapGetOrInsertValueSource::ValueArgument => {",
        "            MapGetOrInsertValueSource::ComputedCallback => {",
    );
    assert_eq!(value_argument.matches("emit_require_weak_key(").count(), 1);
    assert_eq!(
        value_argument
            .matches("self.emit_builtin_arg_to_locals(")
            .count(),
        1
    );
    assert_before(
        value_argument,
        "self.emit_require_weak_key(",
        "self.emit_builtin_arg_to_locals(",
    );

    let computed_callback = preparation
        .split_once("            MapGetOrInsertValueSource::ComputedCallback => {")
        .expect("computed callback preparation")
        .1;
    assert_eq!(
        computed_callback.matches("emit_require_weak_key(").count(),
        1
    );
    assert_eq!(
        computed_callback.matches("emit_is_callable_i32(").count(),
        1
    );
    assert_before(
        computed_callback,
        "emit_is_callable_i32(",
        "emit_require_weak_key(",
    );

    let missing_entry = emitter
        .rsplit_once("        match value_source {")
        .expect("missing-entry value-source match")
        .1
        .split_once("        self.emit_find_map_entry(")
        .expect("missing-entry value-source match end")
        .0;
    assert!(missing_entry.contains("MapGetOrInsertValueSource::ValueArgument => {}"));
    assert_eq!(
        missing_entry
            .matches("MapGetOrInsertValueSource::ComputedCallback => {")
            .count(),
        1
    );
    assert_eq!(
        missing_entry
            .matches("emit_function_or_proxy_call_leave_throw_completion(")
            .count(),
        1
    );
    assert_eq!(
        missing_entry
            .matches("emit_return_current_completion_if_throw(function)")
            .count(),
        1
    );
}
