use std::fs;
use std::path::Path;

const EMIT_SOURCE: &str = include_str!("../src/emit.rs");
const COMPLETION_EXIT_SOURCE: &str = include_str!("../src/emit/completion_exit.rs");
const CONTROL_FLOW_SOURCE: &str = include_str!("../src/control_flow.rs");
const CLI_SOURCE: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const NORMAL_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections.js");
const PRIMARY_THROW_FIXTURE: &str = include_str!(
    "../../lila-cli/tests/fixtures/wasm_multiple_unhandled_rejections_with_primary_throw.js"
);

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end after {start}: {end}"))
        .0
}

fn quoted_literal_end(source: &str, quote_start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut offset = quote_start + 1;
    let mut escaped = false;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == b'"' {
            return Some(offset + 1);
        }
        offset += 1;
    }
    None
}

fn character_literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let value_start = start + 1;
    if value_start >= bytes.len() {
        return None;
    }
    let value_end = if bytes[value_start] == b'\\' {
        let mut offset = value_start + 1;
        if offset >= bytes.len() {
            return None;
        }
        if bytes[offset] == b'u' && bytes.get(offset + 1) == Some(&b'{') {
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'}') {
                offset += 1;
            }
            if bytes.get(offset) != Some(&b'}') {
                return None;
            }
            offset + 1
        } else if bytes[offset] == b'x'
            && bytes
                .get(offset + 1..offset + 3)
                .is_some_and(|digits| digits.iter().all(u8::is_ascii_hexdigit))
        {
            offset + 3
        } else {
            offset + 1
        }
    } else {
        value_start + source[value_start..].chars().next()?.len_utf8()
    };
    (bytes.get(value_end) == Some(&b'\'')).then_some(value_end + 1)
}

fn raw_literal_end(source: &str, start: usize, prefix_len: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut quote_start = start + prefix_len;
    while bytes.get(quote_start) == Some(&b'#') {
        quote_start += 1;
    }
    if bytes.get(quote_start) != Some(&b'"') {
        return None;
    }
    let hashes = quote_start - start - prefix_len;
    let mut offset = quote_start + 1;
    while offset < bytes.len() {
        if bytes[offset] == b'"'
            && bytes
                .get(offset + 1..offset + 1 + hashes)
                .is_some_and(|suffix| suffix.iter().all(|byte| *byte == b'#'))
        {
            return Some(offset + 1 + hashes);
        }
        offset += 1;
    }
    None
}

fn literal_end(source: &str, start: usize) -> Option<usize> {
    let bytes = source.as_bytes();
    match bytes.get(start).copied()? {
        b'"' => quoted_literal_end(source, start),
        b'\'' => character_literal_end(source, start),
        b'b' if bytes.get(start + 1) == Some(&b'\'') => character_literal_end(source, start + 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'"') => quoted_literal_end(source, start + 1),
        b'r' => raw_literal_end(source, start, 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'r') => raw_literal_end(source, start, 2),
        _ => None,
    }
}

fn normalized_code(source: &str) -> String {
    let bytes = source.as_bytes();
    let mut normalized = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            normalized.push('L');
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            offset += 2;
            let mut depth = 1;
            while offset < bytes.len() && depth != 0 {
                if bytes.get(offset..offset + 2) == Some(b"/*") {
                    depth += 1;
                    offset += 2;
                } else if bytes.get(offset..offset + 2) == Some(b"*/") {
                    depth -= 1;
                    offset += 2;
                } else {
                    offset += 1;
                }
            }
            assert_eq!(depth, 0, "unterminated block comment in Rust source");
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"r#") {
            let identifier_start = source[offset + 2..].chars().next();
            if identifier_start
                .is_some_and(|character| character == '_' || character.is_alphabetic())
            {
                offset += 2;
                continue;
            }
        }
        let character = source[offset..].chars().next().unwrap();
        if !character.is_whitespace() {
            normalized.push(character);
        }
        offset += character.len_utf8();
    }
    normalized
}

fn exact_identifier_count(source: &str, identifier: &str) -> usize {
    source
        .match_indices(identifier)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + identifier.len()..].chars().next();
            [before, after].into_iter().all(|edge| {
                edge.map(|character| !character.is_alphanumeric() && character != '_')
                    .unwrap_or(true)
            })
        })
        .count()
}

fn count_identifier_in_rust_sources(dir: &Path, identifier: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_identifier_in_rust_sources(&path, identifier);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_identifier_count(&source, identifier)
        })
        .sum()
}

fn count_in_normalized_rust_sources(dir: &Path, needle: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_in_normalized_rust_sources(&path, needle);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            normalized_code(&source).matches(needle).count()
        })
        .sum()
}

#[test]
fn completion_exit_has_one_private_file_owner_and_exact_visibility() {
    assert_eq!(EMIT_SOURCE.matches("\nmod completion_exit;\n").count(), 1);
    assert!(!EMIT_SOURCE.contains("\npub mod completion_exit;\n"));
    assert!(!EMIT_SOURCE.contains("\nmod completion_exit {\n"));
    assert_eq!(
        EMIT_SOURCE
            .matches("pub(crate) use completion_exit::CompletionExit;")
            .count(),
        1
    );
    assert!(COMPLETION_EXIT_SOURCE.starts_with("use super::*;\n\n"));

    let declaration = bounded(
        COMPLETION_EXIT_SOURCE,
        "use super::*;\n",
        "impl CompletionExit {",
    );
    assert_eq!(
        normalized_code(declaration),
        concat!(
            "pub(crate)structCompletionExit(CompletionExitState);",
            "enumCompletionExitState{MainExport,MainJobCheckpoint(ControlTarget),MultiValue,}"
        )
    );
    assert!(!declaration.contains("#["));
    let normalized_source = normalized_code(COMPLETION_EXIT_SOURCE);
    for authority in ["CompletionExit", "CompletionExitState"] {
        for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
            assert!(!normalized_source.contains(&format!("impl{capability}for{authority}")));
        }
    }

    assert_eq!(
        COMPLETION_EXIT_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(|line| {
                line.starts_with("struct ")
                    || line.starts_with("enum ")
                    || line.starts_with("pub struct ")
                    || line.starts_with("pub enum ")
                    || line.starts_with("pub(") && line.contains(" struct ")
                    || line.starts_with("pub(") && line.contains(" enum ")
            })
            .collect::<Vec<_>>(),
        [
            "pub(crate) struct CompletionExit(CompletionExitState);",
            "enum CompletionExitState {",
        ]
    );
    for former_parent_owner in [
        "struct CompletionExit(",
        "enum CompletionExitState {",
        "impl CompletionExit {",
    ] {
        assert!(
            !EMIT_SOURCE.contains(former_parent_owner),
            "parent retained `{former_parent_owner}`"
        );
    }

    assert_eq!(
        COMPLETION_EXIT_SOURCE
            .lines()
            .map(str::trim_start)
            .filter(
                |line| line.starts_with("fn ") || line.starts_with("pub") && line.contains(" fn ")
            )
            .collect::<Vec<_>>(),
        [
            "pub(super) fn for_return_abi(return_abi: ReturnAbi) -> Self {",
            "pub(crate) const fn return_abi(&self) -> ReturnAbi {",
            "pub(crate) const fn main_job_checkpoint_target(&self) -> Option<ControlTarget> {",
            "pub(super) fn enter_main_job_checkpoint(&mut self, target: ControlTarget) {",
            "pub(super) fn leave_main_job_checkpoint(&mut self, target: ControlTarget) {",
        ]
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "CompletionExit"),
        5
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "CompletionExitState"),
        18
    );

    let route_probe = r###"
        completion_exit . r#return_abi ::<> ();
        CompletionExit :: /* stored route */ r#for_return_abi (return_abi);
        completion_exit . r#main_job_checkpoint_target ();
        completion_exit . r#enter_main_job_checkpoint (target);
        completion_exit . r#leave_main_job_checkpoint (target);
        let text = "CompletionExit::for_return_abi completion_exit.return_abi()";
        let raw = r#"completion_exit.main_job_checkpoint_target()"#;
        let borrowed: &'a str = value;
    "###;
    assert_eq!(
        normalized_code(route_probe),
        concat!(
            "completion_exit.return_abi::<>();",
            "CompletionExit::for_return_abi(return_abi);",
            "completion_exit.main_job_checkpoint_target();",
            "completion_exit.enter_main_job_checkpoint(target);",
            "completion_exit.leave_main_job_checkpoint(target);",
            "lettext=L;letraw=L;letborrowed:&'astr=value;"
        )
    );

    for method in [
        "for_return_abi",
        "return_abi",
        "main_job_checkpoint_target",
        "enter_main_job_checkpoint",
        "leave_main_job_checkpoint",
    ] {
        assert_eq!(
            normalized_source.matches(&format!("fn{method}(")).count(),
            1,
            "{method} must have one owner definition"
        );
    }
    for (route, count) in [
        ("::for_return_abi", 1),
        (".for_return_abi", 0),
        (".return_abi", 9),
        ("::return_abi", 0),
        (".main_job_checkpoint_target", 1),
        ("::main_job_checkpoint_target", 0),
        (".enter_main_job_checkpoint", 1),
        ("::enter_main_job_checkpoint", 0),
        (".leave_main_job_checkpoint", 1),
        ("::leave_main_job_checkpoint", 0),
    ] {
        assert_eq!(
            count_in_normalized_rust_sources(&source_root, route),
            count,
            "unexpected normalized method route `{route}`"
        );
    }
}

#[test]
fn completion_exit_transitions_and_callers_are_closed() {
    assert_eq!(
        COMPLETION_EXIT_SOURCE
            .matches("CompletionExitState")
            .count(),
        18
    );
    let states = bounded(
        COMPLETION_EXIT_SOURCE,
        "enum CompletionExitState {",
        "impl CompletionExit {",
    );
    assert_eq!(
        states
            .lines()
            .map(str::trim)
            .filter(|line| !line.is_empty() && *line != "}")
            .collect::<Vec<_>>(),
        [
            "MainExport,",
            "MainJobCheckpoint(ControlTarget),",
            "MultiValue,",
        ]
    );

    let constructor = bounded(
        COMPLETION_EXIT_SOURCE,
        "pub(super) fn for_return_abi(",
        "pub(crate) const fn return_abi(",
    );
    assert_eq!(
        normalized_code(constructor),
        concat!(
            "return_abi:ReturnAbi)->Self{Self(matchreturn_abi{",
            "ReturnAbi::MainExport=>CompletionExitState::MainExport,",
            "ReturnAbi::MultiValue=>CompletionExitState::MultiValue,})}"
        )
    );

    let return_abi = bounded(
        COMPLETION_EXIT_SOURCE,
        "pub(crate) const fn return_abi(",
        "pub(crate) const fn main_job_checkpoint_target(",
    );
    assert_eq!(
        normalized_code(return_abi),
        concat!(
            "&self)->ReturnAbi{match&self.0{",
            "CompletionExitState::MainExport|CompletionExitState::MainJobCheckpoint(_)=>",
            "{ReturnAbi::MainExport}",
            "CompletionExitState::MultiValue=>ReturnAbi::MultiValue,}}"
        )
    );

    let checkpoint_target = bounded(
        COMPLETION_EXIT_SOURCE,
        "pub(crate) const fn main_job_checkpoint_target(",
        "pub(super) fn enter_main_job_checkpoint(",
    );
    assert_eq!(
        normalized_code(checkpoint_target),
        concat!(
            "&self)->Option<ControlTarget>{match&self.0{",
            "CompletionExitState::MainJobCheckpoint(target)=>Some(*target),",
            "CompletionExitState::MainExport|CompletionExitState::MultiValue=>None,}}"
        )
    );

    let enter = bounded(
        COMPLETION_EXIT_SOURCE,
        "pub(super) fn enter_main_job_checkpoint(",
        "pub(super) fn leave_main_job_checkpoint(",
    );
    assert_eq!(
        normalized_code(enter),
        concat!(
            "&mutself,target:ControlTarget){assert!(match&self.0{",
            "CompletionExitState::MainExport=>true,",
            "CompletionExitState::MainJobCheckpoint(_)|CompletionExitState::MultiValue=>false,",
            "});self.0=CompletionExitState::MainJobCheckpoint(target);}"
        )
    );
    let leave = bounded(
        COMPLETION_EXIT_SOURCE,
        "pub(super) fn leave_main_job_checkpoint(",
        "\n    }\n}\n",
    );
    assert_eq!(
        normalized_code(leave),
        concat!(
            "&mutself,target:ControlTarget){assert!(match&self.0{",
            "CompletionExitState::MainJobCheckpoint(active)=>*active==target,",
            "CompletionExitState::MainExport|CompletionExitState::MultiValue=>false,",
            "});self.0=CompletionExitState::MainExport;"
        )
    );

    for (definition, caller_source, caller) in [
        (
            "for_return_abi(",
            EMIT_SOURCE,
            "CompletionExit::for_return_abi(return_abi)",
        ),
        (
            "pub(crate) const fn return_abi(",
            EMIT_SOURCE,
            "self.completion_exit.return_abi()",
        ),
        (
            "main_job_checkpoint_target(",
            CONTROL_FLOW_SOURCE,
            "self.completion_exit.main_job_checkpoint_target()",
        ),
        (
            "enter_main_job_checkpoint(",
            EMIT_SOURCE,
            "self.completion_exit.enter_main_job_checkpoint(target)",
        ),
        (
            "leave_main_job_checkpoint(",
            EMIT_SOURCE,
            "self.completion_exit.leave_main_job_checkpoint(target)",
        ),
    ] {
        assert_eq!(COMPLETION_EXIT_SOURCE.matches(definition).count(), 1);
        assert_eq!(
            caller_source.matches(caller).count(),
            1,
            "caller `{caller}`"
        );
    }
}

#[test]
fn main_checkpoint_wraps_source_and_routes_abrupt_completion_before_return() {
    let checkpoint_anchor = r#"            self.emit_propagate_throw_from_locals_if_needed(
                self.result_local,
                self.result_tag_local,
                &mut function,
            )?;
        }
"#;
    assert_eq!(EMIT_SOURCE.matches(checkpoint_anchor).count(), 1);
    let main_body = bounded(
        EMIT_SOURCE,
        checkpoint_anchor,
        "if matches!(self.return_abi(), ReturnAbi::MultiValue)",
    );
    assert_eq!(
        normalized_code(main_body),
        concat!(
            "letmain_job_checkpoint=ifself.is_main()&&self.uses_heap{",
            "lettarget=self.open_frame(ControlFrameKind::Block,&mutfunction);",
            "self.completion_exit.enter_main_job_checkpoint(target);Some(target)",
            "}else{None};self.compile_block_contents(self.body,&mutfunction)?;",
            "ifletSome(target)=main_job_checkpoint{",
            "self.completion_exit.leave_main_job_checkpoint(target);",
            "self.pop_control(ControlFrameKind::Block);",
            "function.instruction(&Instruction::End);}"
        )
    );

    let completion_return = bounded(
        CONTROL_FLOW_SOURCE,
        "pub(crate) fn emit_return_current_completion(",
        "        self.verify_and_clear_runtime_gc_anchor_root(function);",
    );
    assert_eq!(
        normalized_code(completion_return),
        concat!(
            "&self,function:&mutFunction){ifletSome(target)=",
            "self.completion_exit.main_job_checkpoint_target(){",
            "self.emit_branch_to_target(target,function);return;}",
            "for_in0..self.environment_depth{self.load_i64_to_local_from_offset(",
            "self.current_env_local,ENV_PARENT_OFFSET,self.current_env_local,function,);}"
        )
    );

    for (test_name, fixture_name, fixture) in [
        (
            "run_wasm_backend_reports_every_unhandled_rejection_in_fifo_order",
            "wasm_multiple_unhandled_rejections.js",
            NORMAL_FIXTURE,
        ),
        (
            "run_wasm_backend_reports_all_rejections_without_replacing_a_primary_throw",
            "wasm_multiple_unhandled_rejections_with_primary_throw.js",
            PRIMARY_THROW_FIXTURE,
        ),
    ] {
        assert_eq!(
            CLI_SOURCE.matches(&format!("fn {test_name}() {{")).count(),
            1
        );
        assert_eq!(CLI_SOURCE.matches(fixture_name).count(), 1);
        assert!(fixture.contains("Promise.reject"));
    }
}
