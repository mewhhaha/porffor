use std::fs;
use std::path::{Path, PathBuf};

const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");
const CLOSE_HELPER: &str = "emit_iterator_flat_map_close_outer_after_throw";

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn without_whitespace(source: &str) -> String {
    source.chars().filter(|ch| !ch.is_whitespace()).collect()
}

fn assert_normalized_once(source: &str, expected: &str, message: &str) {
    let source = without_whitespace(source);
    let expected = without_whitespace(expected);
    assert_eq!(source.matches(expected.as_str()).count(), 1, "{message}");
}

fn normalized_close_call(state: &str) -> String {
    without_whitespace(&format!(
        r#"
        self.{CLOSE_HELPER}(
            this_payload_local,
            close_outer_on_throw,
            IteratorFlatMapInnerState::{state},
            function,
        )?;
        self.emit_return_current_completion(function);
        "#
    ))
}

fn assert_one_close_call(region: &str, state: &str, label: &str) {
    assert_eq!(
        region.matches(&format!("{CLOSE_HELPER}(")).count(),
        1,
        "{label} must own exactly one flatMap outer-close call"
    );
    assert_eq!(
        region.matches("IteratorFlatMapInnerState::").count(),
        1,
        "{label} must make exactly one direct inner-state choice"
    );
    assert_eq!(
        without_whitespace(region)
            .matches(normalized_close_call(state).as_str())
            .count(),
        1,
        "{label} must select {state} before returning the completion",
    );
}

fn collect_rust_sources(dir: &Path, sources: &mut Vec<(PathBuf, String)>) {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read source entry").path())
        .collect::<Vec<_>>();
    paths.sort();

    for path in paths {
        if path.is_dir() {
            collect_rust_sources(&path, sources);
        } else if path.extension().and_then(|extension| extension.to_str()) == Some("rs") {
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            sources.push((path, source));
        }
    }
}

#[test]
fn inner_state_is_closed_and_preserves_close_finalization_order() {
    let variants = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) enum IteratorFlatMapInnerState {",
        "}",
    );
    assert_eq!(
        without_whitespace(variants),
        "NotInstalled,Active,",
        "flatMap outer-close state must remain exactly pre-install or active"
    );
    assert_eq!(
        CONTROL_FLOW_SOURCE
            .matches("pub(crate) enum IteratorFlatMapInnerState {")
            .count(),
        1,
        "the inner lifecycle domain must have one crate-private definition"
    );
    assert!(!CONTROL_FLOW_SOURCE.contains("pub(super) enum IteratorFlatMapInnerState"));
    assert!(!CONTROL_FLOW_SOURCE.contains("pub enum IteratorFlatMapInnerState"));

    let signature = bounded(
        CONTROL_FLOW_SOURCE,
        &format!("pub(crate) fn {CLOSE_HELPER}"),
        ") -> Result<(), EmitError> {",
    );
    assert_eq!(
        without_whitespace(signature),
        without_whitespace(
            r#"(
                &mut self,
                helper_payload_local: u32,
                close: IteratorCloseOnThrowLocals,
                inner_state: IteratorFlatMapInnerState,
                function: &mut Function,
            "#,
        ),
        "the close helper must accept the closed lifecycle state directly"
    );
    assert!(!signature.contains(": bool"));
    assert!(!signature.contains("clear_inner_active"));

    let helper = bounded(
        CONTROL_FLOW_SOURCE,
        &format!("pub(crate) fn {CLOSE_HELPER}("),
        "pub(crate) fn compile_for_in_array(",
    );
    let helper = without_whitespace(helper);
    assert!(!helper.contains("clear_inner_active"));
    assert!(!helper.contains("ifinner_state"));
    assert!(!helper.contains("matches!(inner_state"));
    assert_eq!(helper.matches("matchinner_state{").count(), 1);
    assert_eq!(helper.matches("$IteratorFlatMapDone").count(), 1);
    assert_eq!(helper.matches("$IteratorFlatMapInnerActive").count(), 1);
    assert_eq!(helper.matches("$IteratorFlatMapExecuting").count(), 1);

    const PROJECTION: &str = r#"
        match inner_state {
            IteratorFlatMapInnerState::NotInstalled => {}
            IteratorFlatMapInnerState::Active => {
                self.emit_object_define_bool_data(
                    helper_payload_local,
                    "$IteratorFlatMapInnerActive",
                    false,
                    function,
                )?;
            }
        }
    "#;
    let projection = without_whitespace(PROJECTION);
    assert_eq!(
        helper.matches(projection.as_str()).count(),
        1,
        "only Active may clear the installed inner marker",
    );
    assert_eq!(
        helper.matches("=>").count(),
        2,
        "the lifecycle projection must remain exhaustive over exactly two variants",
    );
    assert!(!helper.contains("_=>"));

    const DONE_TRUE: &str = r#"
        self.emit_object_define_bool_data(
            helper_payload_local,
            "$IteratorFlatMapDone",
            true,
            function,
        )?;
    "#;
    const EXECUTING_FALSE: &str = r#"
        self.emit_object_define_bool_data(
            helper_payload_local,
            "$IteratorFlatMapExecuting",
            false,
            function,
        )?;
    "#;
    let done_true = without_whitespace(DONE_TRUE);
    let executing_false = without_whitespace(EXECUTING_FALSE);
    assert_eq!(
        helper.matches(done_true.as_str()).count(),
        1,
        "fatal flatMap completion must set Done to true exactly once",
    );
    assert_eq!(
        helper.matches(executing_false.as_str()).count(),
        1,
        "fatal flatMap completion must clear Executing exactly once",
    );

    let close = helper
        .find("self.emit_iterator_close_preserving_current_throw(close,function)?;")
        .expect("missing outer IteratorClose");
    let done = helper
        .find(done_true.as_str())
        .expect("missing Done=true finalization");
    let state = helper
        .find("matchinner_state{")
        .expect("missing inner-state projection");
    let inner_active = helper
        .find("\"$IteratorFlatMapInnerActive\"")
        .expect("missing Active-state clear");
    let executing = helper
        .find(executing_false.as_str())
        .expect("missing Executing=false finalization");
    assert!(
        close < done && done < state && state < inner_active && inner_active < executing,
        "outer close, Done, state projection, InnerActive clear and Executing must stay ordered"
    );
}

#[test]
fn each_failure_region_owns_its_exact_inner_state() {
    const NEXT_ARM: &str = "StandardBuiltinId::IteratorFlatMapNext => {";
    const RETURN_ARM: &str = "StandardBuiltinId::IteratorFlatMapReturn => {";

    let standard_dispatch = STANDARD_SOURCE
        .split_once("pub(crate) fn compile_standard_builtin(")
        .expect("standard builtin dispatcher")
        .1;
    assert_eq!(
        standard_dispatch
            .matches("StandardBuiltinId::IteratorFlatMapNext")
            .count(),
        1,
        "flatMap next must have exactly one dispatcher owner"
    );
    assert_eq!(
        standard_dispatch
            .matches("StandardBuiltinId::IteratorFlatMapReturn")
            .count(),
        1,
        "flatMap return must have exactly one dispatcher owner"
    );
    assert_eq!(
        STANDARD_SOURCE.matches(NEXT_ARM).count(),
        1,
        "flatMap next must have one concrete dispatch arm"
    );
    assert_eq!(STANDARD_SOURCE.matches(RETURN_ARM).count(), 1);
    assert_eq!(
        STANDARD_SOURCE
            .matches("use crate::control_flow::IteratorFlatMapInnerState;")
            .count(),
        1,
        "the standard builtin must import the shared control-flow state directly"
    );

    let next = bounded(STANDARD_SOURCE, NEXT_ARM, RETURN_ARM);
    let next = without_whitespace(next);
    assert_eq!(next.matches(&format!("{CLOSE_HELPER}(")).count(), 8);
    assert_eq!(next.matches("IteratorFlatMapInnerState::Active").count(), 4);
    assert_eq!(
        next.matches("IteratorFlatMapInnerState::NotInstalled")
            .count(),
        4
    );

    const INSTALLATION: &str = r#"
        self.emit_object_define_local_data(
            this_payload_local,
            "$IteratorFlatMapInnerIterator",
            inner_iterator_payload_local,
            inner_iterator_tag_local,
            function,
        )?;
        self.emit_object_define_local_data(
            this_payload_local,
            "$IteratorFlatMapInnerNext",
            inner_next_payload_local,
            inner_next_tag_local,
            function,
        )?;
        self.emit_object_define_bool_data(
            this_payload_local,
            "$IteratorFlatMapInnerActive",
            true,
            function,
        )?;
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::Br(0));
    "#;
    const INNER_ITERATOR_INSTALL: &str = r#"
        self.emit_object_define_local_data(
            this_payload_local,
            "$IteratorFlatMapInnerIterator",
            inner_iterator_payload_local,
            inner_iterator_tag_local,
            function,
        )?;
    "#;
    const INNER_NEXT_INSTALL: &str = r#"
        self.emit_object_define_local_data(
            this_payload_local,
            "$IteratorFlatMapInnerNext",
            inner_next_payload_local,
            inner_next_tag_local,
            function,
        )?;
    "#;
    const INNER_ACTIVATE: &str = r#"
        self.emit_object_define_bool_data(
            this_payload_local,
            "$IteratorFlatMapInnerActive",
            true,
            function,
        )?;
    "#;
    const INNER_DEACTIVATE: &str = r#"
        self.emit_object_define_bool_data(
            this_payload_local,
            "$IteratorFlatMapInnerActive",
            false,
            function,
        )?;
    "#;

    let installation = without_whitespace(INSTALLATION);
    let inner_iterator_install = without_whitespace(INNER_ITERATOR_INSTALL);
    let inner_next_install = without_whitespace(INNER_NEXT_INSTALL);
    let inner_activate = without_whitespace(INNER_ACTIVATE);
    let inner_deactivate = without_whitespace(INNER_DEACTIVATE);
    assert_eq!(next.matches(installation.as_str()).count(), 1);
    assert_eq!(next.matches(inner_iterator_install.as_str()).count(), 1);
    assert_eq!(next.matches(inner_next_install.as_str()).count(), 1);
    assert_eq!(next.matches(inner_activate.as_str()).count(), 1);
    assert_eq!(next.matches(inner_deactivate.as_str()).count(), 1);
    assert_eq!(next.matches("$IteratorFlatMapInnerIterator").count(), 2);
    assert_eq!(next.matches("$IteratorFlatMapInnerNext").count(), 2);
    assert_eq!(next.matches("$IteratorFlatMapInnerActive").count(), 3);

    let install_offset = next
        .find(installation.as_str())
        .expect("missing sole inner installation sequence");
    let acquisition_anchor = r#"self.strings.property_key_symbol_payload("Symbol.iterator")"#;
    assert_eq!(next.matches(acquisition_anchor).count(), 1);
    let acquisition_offset = next
        .find(acquisition_anchor)
        .expect("missing mapped-value iterator acquisition");
    assert!(acquisition_offset < install_offset);

    let active_call = normalized_close_call("Active");
    let not_installed_call = normalized_close_call("NotInstalled");
    let active_offsets = next
        .match_indices(active_call.as_str())
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let not_installed_offsets = next
        .match_indices(not_installed_call.as_str())
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    let close_call_prefix = format!("self.{CLOSE_HELPER}(");
    let all_close_offsets = next
        .match_indices(close_call_prefix.as_str())
        .map(|(offset, _)| offset)
        .collect::<Vec<_>>();
    assert_eq!(active_offsets.len(), 4);
    assert_eq!(not_installed_offsets.len(), 4);
    assert_eq!(all_close_offsets.len(), 8);
    assert!(
        all_close_offsets
            .iter()
            .all(|offset| *offset < install_offset),
        "all eight abrupt calls must precede the sole textual installation/back-edge",
    );
    assert!(
        active_offsets
            .iter()
            .all(|offset| *offset < acquisition_offset),
        "the four Active calls must remain in the loop-header active-inner branch",
    );
    assert!(
        not_installed_offsets
            .iter()
            .all(|offset| acquisition_offset < *offset && *offset < install_offset),
        "the four NotInstalled calls must remain between acquisition and installation",
    );

    let active = bounded(
        &next,
        r#"self.strings.payload("$IteratorFlatMapInnerActive")"#,
        concat!(
            "function.instruction(&Instruction::Else);",
            "self.emit_function_handle_call_without_throw_propagation(next_payload_local,",
        ),
    );
    assert_normalized_once(
        active,
        r#"
        self.emit_object_own_data_field_read(
            this_payload_local,
            this_tag_local,
            key_local,
            present_local,
            inner_active_payload_local,
            inner_active_tag_local,
            function,
        );
        function.instruction(&Instruction::LocalGet(inner_active_payload_local));
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        "#,
        "the four Active calls must remain behind the runtime active-inner gate",
    );
    assert_eq!(active.matches(&format!("{CLOSE_HELPER}(")).count(), 4);
    assert_eq!(
        active.matches("IteratorFlatMapInnerState::Active").count(),
        4
    );
    assert!(!active.contains("IteratorFlatMapInnerState::NotInstalled"));

    let inner_next_throw = bounded(
        active,
        "self.emit_function_handle_call_without_throw_propagation(inner_next_payload_local,",
        "self.emit_is_heap_object_like_tag_i32(next_result_tag_local,function);",
    );
    assert_one_close_call(inner_next_throw, "Active", "inner next abrupt completion");

    let next_result_non_object = bounded(
        active,
        "self.emit_is_heap_object_like_tag_i32(next_result_tag_local,function);",
        r#"self.strings.payload("done")"#,
    );
    assert!(next_result_non_object.contains("nextresultmustbeobject"));
    assert_one_close_call(
        next_result_non_object,
        "Active",
        "inner next non-object result",
    );

    let done_throw = bounded(
        active,
        r#"self.strings.payload("done")"#,
        "self.compile_truthy_tagged_i32(done_tag_local,done_payload_local,function)?;",
    );
    assert!(done_throw.contains("done_payload_local"));
    assert_one_close_call(done_throw, "Active", "inner done getter abrupt completion");

    let value_throw = bounded(
        active,
        r#"self.strings.payload("value")"#,
        r#"self.emit_object_define_bool_data(this_payload_local,"$IteratorFlatMapExecuting",false,function,)?;"#,
    );
    assert!(value_throw.contains("value_payload_local"));
    assert_one_close_call(
        value_throw,
        "Active",
        "inner value getter abrupt completion",
    );

    let acquisition = bounded(
        &next,
        r#"self.strings.property_key_symbol_payload("Symbol.iterator")"#,
        installation.as_str(),
    );
    assert_eq!(acquisition.matches(&format!("{CLOSE_HELPER}(")).count(), 4);
    assert_eq!(
        acquisition
            .matches("IteratorFlatMapInnerState::NotInstalled")
            .count(),
        4
    );
    assert!(!acquisition.contains("IteratorFlatMapInnerState::Active"));

    let iterator_method_get_throw = acquisition
        .split_once(concat!(
            "function.instruction(&Instruction::LocalGet(iterator_method_tag_local));",
            "function.instruction(&Instruction::I64Const(",
            "ValueKind::Undefined.tag()asi64));",
        ))
        .expect("missing post-get iterator-method classification")
        .0;
    assert!(iterator_method_get_throw.contains("iterator_method_payload_local"));
    assert_one_close_call(
        iterator_method_get_throw,
        "NotInstalled",
        "Symbol.iterator getter abrupt completion",
    );

    let iterator_method_call_throw = bounded(
        acquisition,
        "self.emit_function_handle_call_without_throw_propagation(iterator_method_payload_local,",
        "self.emit_is_heap_object_like_tag_i32(inner_iterator_tag_local,function);",
    );
    assert_one_close_call(
        iterator_method_call_throw,
        "NotInstalled",
        "iterator method abrupt completion",
    );

    let iterator_result_non_object = bounded(
        acquisition,
        "self.emit_is_heap_object_like_tag_i32(inner_iterator_tag_local,function);",
        concat!(
            "function.instruction(&Instruction::Else);",
            "function.instruction(&Instruction::LocalGet(mapped_payload_local));",
        ),
    );
    assert!(iterator_result_non_object.contains("iteratormethodmustreturnobject"));
    assert_one_close_call(
        iterator_result_non_object,
        "NotInstalled",
        "iterator method non-object result",
    );

    let inner_next_get_throw = bounded(
        acquisition,
        r#"self.strings.payload("next")"#,
        concat!(
            "function.instruction(&Instruction::LocalGet(inner_next_tag_local));",
            "function.instruction(&Instruction::I64Const(",
            "ValueKind::Function.tag()asi64));",
        ),
    );
    assert!(inner_next_get_throw.contains("inner_next_payload_local"));
    assert_one_close_call(
        inner_next_get_throw,
        "NotInstalled",
        "inner next getter abrupt completion",
    );
}

#[test]
fn helper_and_state_have_no_uninventoried_source_bypass() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut helper_mentions = 0;
    let mut state_projections = 0;
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected_helper_mentions = match relative.as_ref() {
            "control_flow.rs" => 1,
            "builtins/standard.rs" => 8,
            _ => 0,
        };
        let actual_helper_mentions = source.matches(&format!("{CLOSE_HELPER}(")).count();
        assert_eq!(
            actual_helper_mentions, expected_helper_mentions,
            "unexpected flatMap close-helper definition or caller in {relative}"
        );
        assert_eq!(
            source.matches(&format!("::{CLOSE_HELPER}")).count(),
            0,
            "the helper must not escape as an associated method item in {relative}"
        );
        assert!(
            !source.contains("clear_inner_active"),
            "the raw Boolean policy must not reappear in {relative}"
        );
        helper_mentions += actual_helper_mentions;

        let expected_state_projections = match relative.as_ref() {
            "control_flow.rs" => 2,
            "builtins/standard.rs" => 8,
            _ => 0,
        };
        let actual_state_projections = source.matches("IteratorFlatMapInnerState::").count();
        assert_eq!(
            actual_state_projections, expected_state_projections,
            "unexpected flatMap inner-state projection in {relative}"
        );
        state_projections += actual_state_projections;
    }

    assert_eq!(
        helper_mentions, 9,
        "the helper must have one definition and exactly eight callers"
    );
    assert_eq!(
        state_projections, 10,
        "only the two helper arms and eight inventoried callers may project the state"
    );
}
