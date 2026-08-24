use std::fs;
use std::path::{Path, PathBuf};

const HEAP_SOURCE: &str = include_str!("../src/heap.rs");
const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const STANDARD_SOURCE: &str = include_str!("../src/builtins/standard.rs");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker: {start}"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker after: {start}"))
        .0
}

fn normalized(source: &str) -> String {
    source
        .chars()
        .filter(|ch| !ch.is_whitespace())
        .collect::<String>()
        .replace(",)", ")")
        .replace(",]", "]")
}

fn normalized_code(source: &str) -> String {
    let code = source
        .lines()
        .filter(|line| !line.trim_start().starts_with("//"))
        .collect::<Vec<_>>()
        .join("\n");
    normalized(&code)
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

fn unique_span(body: &str, needle: &str, label: &str) -> (usize, usize) {
    let start = unique_position(body, needle, label);
    (start, start + needle.len())
}

fn positions(body: &str, needle: &str) -> Vec<usize> {
    body.match_indices(needle).map(|(index, _)| index).collect()
}

fn skip_rust_trivia(source: &[u8], mut index: usize) -> usize {
    loop {
        while index < source.len() && source[index].is_ascii_whitespace() {
            index += 1;
        }
        if source
            .get(index..index + 2)
            .is_some_and(|pair| pair == b"//")
        {
            index += 2;
            while index < source.len() && source[index] != b'\n' {
                index += 1;
            }
            continue;
        }
        if source
            .get(index..index + 2)
            .is_some_and(|pair| pair == b"/*")
        {
            index += 2;
            let mut depth = 1;
            while index < source.len() && depth != 0 {
                if source
                    .get(index..index + 2)
                    .is_some_and(|pair| pair == b"/*")
                {
                    depth += 1;
                    index += 2;
                } else if source
                    .get(index..index + 2)
                    .is_some_and(|pair| pair == b"*/")
                {
                    depth -= 1;
                    index += 2;
                } else {
                    index += 1;
                }
            }
            continue;
        }
        return index;
    }
}

fn skip_quoted(source: &[u8], mut index: usize, quote: u8) -> usize {
    index += 1;
    while index < source.len() {
        if source[index] == b'\\' {
            index += 2;
        } else if source[index] == quote {
            return index + 1;
        } else {
            index += 1;
        }
    }
    index
}

fn raw_helper_non_symbolic_offsets(source: &str, helper: &str) -> usize {
    let bytes = source.as_bytes();
    let mut search_from = 0;
    let mut non_symbolic_offsets = 0;
    while let Some(relative) = source[search_from..].find(helper) {
        let helper_start = search_from + relative;
        let mut index = skip_rust_trivia(bytes, helper_start + helper.len());
        search_from = helper_start + helper.len();
        if bytes.get(index) != Some(&b'(') {
            continue;
        }
        index += 1;
        let mut paren_depth = 0;
        let mut bracket_depth = 0;
        let mut brace_depth = 0;
        let second_argument = loop {
            index = skip_rust_trivia(bytes, index);
            let Some(&byte) = bytes.get(index) else {
                break None;
            };
            match byte {
                b'"' | b'\'' => index = skip_quoted(bytes, index, byte),
                b'(' => {
                    paren_depth += 1;
                    index += 1;
                }
                b')' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                    break None;
                }
                b')' => {
                    paren_depth -= 1;
                    index += 1;
                }
                b'[' => {
                    bracket_depth += 1;
                    index += 1;
                }
                b']' => {
                    bracket_depth -= 1;
                    index += 1;
                }
                b'{' => {
                    brace_depth += 1;
                    index += 1;
                }
                b'}' => {
                    brace_depth -= 1;
                    index += 1;
                }
                b',' if paren_depth == 0 && bracket_depth == 0 && brace_depth == 0 => {
                    break Some(index + 1);
                }
                _ => index += 1,
            }
        };
        let Some(mut offset) = second_argument else {
            continue;
        };
        loop {
            offset = skip_rust_trivia(bytes, offset);
            if source[offset..].starts_with("const")
                && !bytes
                    .get(offset + "const".len())
                    .is_some_and(|byte| byte.is_ascii_alphanumeric())
            {
                offset += "const".len();
                continue;
            }
            match bytes.get(offset) {
                Some(b'(' | b'{' | b'+' | b'-') => offset += 1,
                _ => break,
            }
        }
        offset = skip_rust_trivia(bytes, offset);
        let token_start = offset;
        while bytes
            .get(offset)
            .is_some_and(|byte| byte.is_ascii_alphanumeric() || *byte == b'_')
        {
            offset += 1;
        }
        let token = &source[token_start..offset];
        let named_offset = token == "offset" || token.starts_with("HEAP_");
        offset = skip_rust_trivia(bytes, offset);
        while matches!(bytes.get(offset), Some(b')' | b'}')) {
            offset = skip_rust_trivia(bytes, offset + 1);
        }
        if !named_offset || bytes.get(offset) != Some(&b',') {
            non_symbolic_offsets += 1;
        }
    }
    non_symbolic_offsets
}

fn assert_no_raw_request_kind_offset_alias(body: &str, owner: &str) {
    for helper in [
        "load_i64_to_local_from_offset",
        "store_i64_local_at_offset",
        "store_i64_const_at_offset",
    ] {
        assert_eq!(
            raw_helper_non_symbolic_offsets(body, helper),
            0,
            "{owner} must use only a named HEAP_* constant or reviewed field-loop offset through raw helper {helper}"
        );
    }
}

fn assert_raw_helper_inventory(body: &str, owner: &str, expected: [usize; 3]) {
    let body = normalized_code(body);
    for (index, helper) in [
        "load_i64_to_local_from_offset(",
        "store_i64_local_at_offset(",
        "store_i64_const_at_offset(",
    ]
    .into_iter()
    .enumerate()
    {
        assert_eq!(
            body.matches(helper).count(),
            expected[index],
            "unexpected raw-helper call count for {helper} in {owner}"
        );
    }
}

fn assert_writer_raw_helper_allowlist(body: &str) {
    let body = normalized_code(body);
    let expected = [
        (
            r#"self.load_i64_to_local_from_offset(
                promise_payload_local,
                HEAP_OBJECT_BOXED_PAYLOAD_OFFSET,
                promise_record_local,
                function,
            );"#,
            1,
        ),
        (
            r#"self.load_i64_to_local_from_offset(
                this_payload_local,
                HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
                receiver_brand_local,
                function,
            );"#,
            1,
        ),
        (
            r#"self.load_i64_to_local_from_offset(
                this_payload_local,
                HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET,
                activation_local,
                function,
            );"#,
            1,
        ),
        (
            r#"self.load_i64_to_local_from_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_QUEUE_TAIL_OFFSET,
                queue_tail_local,
                function,
            );"#,
            1,
        ),
        (
            "self.store_i64_local_at_offset(request_local, offset, source_local, function);",
            1,
        ),
        (
            r#"self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
                request_local,
                function,
            );"#,
            1,
        ),
        (
            r#"self.store_i64_local_at_offset(
                queue_tail_local,
                HEAP_ASYNC_GENERATOR_REQUEST_NEXT_OFFSET,
                request_local,
                function,
            );"#,
            1,
        ),
        (
            r#"self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_QUEUE_TAIL_OFFSET,
                request_local,
                function,
            );"#,
            1,
        ),
        (
            r#"self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
                request_local,
                function,
            );"#,
            2,
        ),
        (
            r#"self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
                argument_payload_local,
                function,
            );"#,
            1,
        ),
        (
            r#"self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
                argument_tag_local,
                function,
            );"#,
            1,
        ),
        (
            r#"self.store_i64_const_at_offset(
                request_local,
                HEAP_ASYNC_GENERATOR_REQUEST_NEXT_OFFSET,
                0,
                function,
            );"#,
            1,
        ),
    ];
    let mut reviewed_calls = 0;
    for (call, expected_count) in expected {
        let call = normalized(call);
        assert_eq!(
            body.matches(call.as_str()).count(),
            expected_count,
            "writer raw helper must retain exact receiver, offset and value-local shape: {call}"
        );
        reviewed_calls += expected_count;
    }
    let actual_calls = [
        "load_i64_to_local_from_offset(",
        "store_i64_local_at_offset(",
        "store_i64_const_at_offset(",
    ]
    .into_iter()
    .map(|helper| body.matches(helper).count())
    .sum::<usize>();
    assert_eq!(
        actual_calls, reviewed_calls,
        "every writer raw helper call must belong to the exact allowlist"
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

fn request_writer_owner() -> &'static str {
    bounded(
        STANDARD_SOURCE,
        "StandardBuiltinId::AsyncGeneratorPrototypeNext\n            | StandardBuiltinId::AsyncGeneratorPrototypeReturn\n            | StandardBuiltinId::AsyncGeneratorPrototypeThrow => {",
        "StandardBuiltinId::ArrayIteratorNext => {",
    )
}

fn drain_owner() -> &'static str {
    bounded(
        PROMISE_SOURCE,
        "pub(crate) fn emit_drain_async_generator_queue(",
        "fn emit_run_async_generator_await_job(",
    )
}

fn yield_owner() -> &'static str {
    bounded(
        PROMISE_SOURCE,
        "pub(crate) fn emit_complete_async_generator_yield(",
        "fn emit_run_promise_reaction_callback(",
    )
}

#[test]
fn request_completion_kind_is_one_closed_domain_projected_through_completion_kind() {
    let declaration = bounded(
        HEAP_SOURCE,
        "pub(crate) enum AsyncGeneratorRequestCompletionKind {",
        "}\n\nimpl AsyncGeneratorRequestCompletionKind {",
    );
    let variants = declaration
        .lines()
        .map(str::trim)
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>();
    assert_eq!(
        variants,
        ["Normal,", "Return,", "Throw,"],
        "the request domain must remain exactly Normal, Return and Throw"
    );

    let domain = bounded(
        HEAP_SOURCE,
        "/// The closed Completion Record subset persisted in an async-generator",
        "pub(crate) const GENERATOR_RESUME_STATE_INITIALIZING",
    );
    assert!(domain.contains("#[derive(Debug, Clone, Copy, PartialEq, Eq)]"));
    assert!(!domain.contains("repr("));
    assert!(!HEAP_SOURCE.contains("impl Default for AsyncGeneratorRequestCompletionKind"));
    assert!(!HEAP_SOURCE.contains("impl From<u64> for AsyncGeneratorRequestCompletionKind"));
    assert!(!HEAP_SOURCE.contains("impl From<i64> for AsyncGeneratorRequestCompletionKind"));
    assert!(!HEAP_SOURCE.contains("impl From<bool> for AsyncGeneratorRequestCompletionKind"));

    let policy = bounded(
        HEAP_SOURCE,
        "impl AsyncGeneratorRequestCompletionKind {",
        "/// One strictly validated snapshot of an async-generator request's completion",
    );
    let exact_policy = normalized_code(
        r#"
        const ALL: [Self; 3] = [Self::Normal, Self::Throw, Self::Return];

        const fn completion_kind(self) -> CompletionKind {
            match self {
                Self::Normal => CompletionKind::Normal,
                Self::Return => CompletionKind::Return,
                Self::Throw => CompletionKind::Throw,
            }
        }

        const fn word(self) -> u64 {
            self.completion_kind().code() as u64
        }
    }
        "#,
    );
    let policy = normalized_code(policy);
    assert_eq!(
        policy, exact_policy,
        "the request domain must have one stable list and one exhaustive CompletionKind projection"
    );
    assert_eq!(
        policy
            .matches("[Self::Normal,Self::Throw,Self::Return]")
            .count(),
        1,
        "ALL must follow the stable ABI word order"
    );
    assert_eq!(policy.matches("=>").count(), 3);
    assert_eq!(policy.matches(".code()").count(), 1);
    for projection in [
        "Self::Normal=>CompletionKind::Normal",
        "Self::Return=>CompletionKind::Return",
        "Self::Throw=>CompletionKind::Throw",
    ] {
        assert_eq!(
            policy.matches(projection).count(),
            1,
            "missing {projection}"
        );
    }
    for forbidden in ["_=>", "=>0", "=>1", "=>2", "selfas", "transmute"] {
        assert!(
            !policy.contains(forbidden),
            "the ABI projection must not contain {forbidden}"
        );
    }

    let token = bounded(
        HEAP_SOURCE,
        "/// One strictly validated snapshot of an async-generator request's completion",
        "/// The closed `[[AsyncGeneratorState]]` lifecycle stored in an activation.",
    );
    assert_eq!(
        normalized_code(token),
        normalized_code(
            r#"
            #[must_use = "a loaded request completion kind must be routed and released"]
            pub(crate) struct LoadedAsyncGeneratorRequestCompletionKind(u32);
            "#,
        ),
        "the loaded token declaration must remain opaque and derive neither Clone nor Copy"
    );
    assert!(!HEAP_SOURCE.contains("impl LoadedAsyncGeneratorRequestCompletionKind"));
    assert!(!HEAP_SOURCE.contains("Deref for LoadedAsyncGeneratorRequestCompletionKind"));

    let heap = normalized_code(HEAP_SOURCE);
    assert_eq!(
        heap.matches("AsyncGeneratorRequestCompletionKind").count(),
        11,
        "the closed domain and opaque token must have no extra constructors, impls or raw accessors"
    );
    assert_eq!(
        heap.matches("LoadedAsyncGeneratorRequestCompletionKind(")
            .count(),
        2,
        "only the token declaration and strict loader may spell construction"
    );
    assert_eq!(
        heap.matches("LoadedAsyncGeneratorRequestCompletionKind")
            .count(),
        6,
        "only declaration, strict mint, comparison, ABI copy and consuming release may name the token"
    );
    assert!(!PROMISE_SOURCE.contains("LoadedAsyncGeneratorRequestCompletionKind"));
    assert!(!STANDARD_SOURCE.contains("LoadedAsyncGeneratorRequestCompletionKind"));
}

#[test]
fn request_completion_kind_heap_boundary_is_private_strict_and_opaque() {
    let raw_offset_declarations = HEAP_SOURCE
        .lines()
        .map(str::trim)
        .filter(|line| {
            line.contains("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET: u64 = 0;")
        })
        .collect::<Vec<_>>();
    assert_eq!(
        raw_offset_declarations,
        ["const HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET: u64 = 0;"],
        "the raw request-kind offset declaration must remain module-private"
    );
    assert_eq!(
        HEAP_SOURCE
            .matches("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET")
            .count(),
        4,
        "only declaration, layout, typed store and strict load may own the raw offset"
    );
    assert!(!PROMISE_SOURCE.contains("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET"));
    assert!(!STANDARD_SOURCE.contains("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET"));

    let store = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_store_async_generator_request_completion_kind(",
        "/// Load and strictly validate one async-generator request completion kind.",
    );
    assert_eq!(
        normalized_code(store),
        normalized_code(
            r#"
            &self,
            request_local: u32,
            kind: AsyncGeneratorRequestCompletionKind,
            function: &mut Function,
        ) {
            self.store_i64_const_at_offset(
                request_local,
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET,
                kind.word(),
                function,
            );
        }
            "#,
        ),
        "the typed store must persist exactly the selected request-domain ABI word"
    );
    assert!(store.contains("kind: AsyncGeneratorRequestCompletionKind,"));
    assert_eq!(
        store
            .matches("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET")
            .count(),
        1
    );
    assert_eq!(store.matches("kind.word()").count(), 1);

    let loader = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_load_async_generator_request_completion_kind_strict(",
        "/// Emit one comparison against a strictly loaded request completion kind.",
    );
    assert_eq!(
        normalized_code(loader),
        normalized_code(
            r#"
            &mut self,
            request_local: u32,
            function: &mut Function,
        ) -> LoadedAsyncGeneratorRequestCompletionKind {
            let kind_word_local = self.reserve_temp_local();
            self.load_i64_to_local_from_offset(
                request_local,
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET,
                kind_word_local,
                function,
            );

            let mut open_dispatch_arms = 0;
            for kind in AsyncGeneratorRequestCompletionKind::ALL {
                function.instruction(&Instruction::LocalGet(kind_word_local));
                function.instruction(&Instruction::I64Const(kind.word() as i64));
                function.instruction(&Instruction::I64Eq);
                function.instruction(&Instruction::If(BlockType::Empty));
                function.instruction(&Instruction::Else);
                open_dispatch_arms += 1;
            }
            function.instruction(&Instruction::Unreachable);
            for _ in 0..open_dispatch_arms {
                function.instruction(&Instruction::End);
            }

            LoadedAsyncGeneratorRequestCompletionKind(kind_word_local)
        }
            "#,
        ),
        "the strict loader must preserve its one load, closed dispatch, trap and sole mint"
    );
    assert!(loader.contains(") -> LoadedAsyncGeneratorRequestCompletionKind {"));
    assert_eq!(loader.matches("reserve_temp_local()").count(), 1);
    assert_eq!(loader.matches("load_i64_to_local_from_offset(").count(), 1);
    assert_eq!(
        loader
            .matches("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET")
            .count(),
        1
    );
    assert_eq!(
        loader
            .matches("for kind in AsyncGeneratorRequestCompletionKind::ALL")
            .count(),
        1
    );
    assert_eq!(loader.matches("kind.word()").count(), 1);
    assert_eq!(loader.matches("Instruction::Unreachable").count(), 1);
    assert_eq!(
        loader
            .matches("LoadedAsyncGeneratorRequestCompletionKind(kind_word_local)")
            .count(),
        1
    );

    let strict_dispatch = normalized_code(
        r#"
        let mut open_dispatch_arms = 0;
        for kind in AsyncGeneratorRequestCompletionKind::ALL {
            function.instruction(&Instruction::LocalGet(kind_word_local));
            function.instruction(&Instruction::I64Const(kind.word() as i64));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            open_dispatch_arms += 1;
        }
        function.instruction(&Instruction::Unreachable);
        for _ in 0..open_dispatch_arms {
            function.instruction(&Instruction::End);
        }

        LoadedAsyncGeneratorRequestCompletionKind(kind_word_local)
        "#,
    );
    assert_eq!(
        normalized_code(loader)
            .matches(strict_dispatch.as_str())
            .count(),
        1,
        "all three valid ABI words must precede the sole unknown-word trap"
    );
    for instruction in [
        "Instruction::If",
        "Instruction::Else",
        "Instruction::Unreachable",
        "Instruction::End",
    ] {
        assert_eq!(
            loader.matches(instruction).count(),
            1,
            "the strict decoder must have one emission site for {instruction}"
        );
    }
    for bypass in [
        "Instruction::Block",
        "Instruction::Loop",
        "Instruction::Br(",
        "Instruction::BrIf",
    ] {
        assert!(
            !loader.contains(bypass),
            "the strict decoder must not bypass its closed dispatch with {bypass}"
        );
    }

    let comparer = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_async_generator_request_completion_kind_equals(",
        "/// Copy a validated request word into the generic completion transport",
    );
    assert_eq!(
        normalized_code(comparer),
        normalized_code(
            r#"
            &self,
            loaded: &LoadedAsyncGeneratorRequestCompletionKind,
            expected: AsyncGeneratorRequestCompletionKind,
            function: &mut Function,
        ) {
            function.instruction(&Instruction::LocalGet(loaded.0));
            function.instruction(&Instruction::I64Const(expected.word() as i64));
            function.instruction(&Instruction::I64Eq);
        }
            "#,
        ),
        "the typed comparer must emit exactly equality against the selected ABI word"
    );
    assert!(comparer.contains("loaded: &LoadedAsyncGeneratorRequestCompletionKind,"));
    assert!(comparer.contains("expected: AsyncGeneratorRequestCompletionKind,"));
    assert_eq!(comparer.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(comparer.matches("expected.word()").count(), 1);

    let adapter = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_copy_async_generator_request_completion_kind_to_step_completion(",
        "/// Release the private local owned by a loaded request-kind snapshot.",
    );
    assert_eq!(
        normalized_code(adapter),
        normalized_code(
            r#"
            &self,
            loaded: &LoadedAsyncGeneratorRequestCompletionKind,
            step_completion_kind_local: u32,
            function: &mut Function,
        ) {
            function.instruction(&Instruction::LocalGet(loaded.0));
            function.instruction(&Instruction::LocalSet(step_completion_kind_local));
        }
            "#,
        ),
        "the bounded ABI adapter must copy the validated word without synthesizing a kind"
    );
    assert!(adapter.contains("loaded: &LoadedAsyncGeneratorRequestCompletionKind,"));
    assert!(adapter.contains("step_completion_kind_local: u32,"));
    assert_eq!(adapter.matches("LocalGet(loaded.0)").count(), 1);
    assert_eq!(
        adapter
            .matches("LocalSet(step_completion_kind_local)")
            .count(),
        1
    );
    assert!(!adapter.contains("-> u32"));

    let release = bounded(
        HEAP_SOURCE,
        "pub(crate) fn release_loaded_async_generator_request_completion_kind(",
        "/// Store one state from the closed async-generator execution lifecycle.",
    );
    assert_eq!(
        normalized_code(release),
        normalized_code(
            r#"
            &mut self,
            loaded: LoadedAsyncGeneratorRequestCompletionKind,
        ) {
            self.release_temp_local(loaded.0);
        }
            "#,
        ),
        "the consuming release must release exactly the token's private local"
    );
    assert!(release.contains("loaded: LoadedAsyncGeneratorRequestCompletionKind,"));
    assert!(!release.contains("&LoadedAsyncGeneratorRequestCompletionKind"));
    assert_eq!(release.matches("release_temp_local(loaded.0)").count(), 1);

    let boundary = bounded(
        HEAP_SOURCE,
        "pub(crate) fn emit_store_async_generator_request_completion_kind(",
        "/// Store one state from the closed async-generator execution lifecycle.",
    );
    assert_eq!(
        boundary.matches("loaded.0").count(),
        3,
        "only comparison, ABI copy and consuming release may inspect the private local"
    );
}

#[test]
fn request_writer_initializes_the_closed_kind_before_queue_publication() {
    let writer_owner = request_writer_owner();
    assert_no_raw_request_kind_offset_alias(writer_owner, "request writer");
    assert_raw_helper_inventory(writer_owner, "request writer", [4, 8, 1]);
    assert_writer_raw_helper_allowlist(writer_owner);
    let writer = normalized_code(writer_owner);
    assert_eq!(
        writer
            .matches("emit_store_async_generator_request_completion_kind(")
            .count(),
        1
    );
    assert_eq!(
        writer
            .matches("AsyncGeneratorRequestCompletionKind::Normal")
            .count(),
        1
    );
    assert_eq!(
        writer
            .matches("AsyncGeneratorRequestCompletionKind::Return")
            .count(),
        1
    );
    assert_eq!(
        writer
            .matches("AsyncGeneratorRequestCompletionKind::Throw")
            .count(),
        1
    );

    let mapping = bounded(
        request_writer_owner(),
        "let request_completion_kind = match builtin {",
        "self.emit_store_async_generator_request_completion_kind(",
    );
    let mapping = normalized_code(mapping);
    for arm in [
        "StandardBuiltinId::AsyncGeneratorPrototypeNext=>{AsyncGeneratorRequestCompletionKind::Normal}",
        "StandardBuiltinId::AsyncGeneratorPrototypeReturn=>{AsyncGeneratorRequestCompletionKind::Return}",
        "StandardBuiltinId::AsyncGeneratorPrototypeThrow=>{AsyncGeneratorRequestCompletionKind::Throw}",
    ] {
        assert_eq!(mapping.matches(arm).count(), 1, "missing writer arm {arm}");
    }
    assert_eq!(mapping.matches("=>").count(), 4);
    assert_eq!(mapping.matches("_=>unreachable!()").count(), 1);

    let receiver_rejection_sentinel = normalized(
        r#"
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(receiver_brand_local));
        function.instruction(&Instruction::LocalGet(this_tag_local));
        function.instruction(&Instruction::I64Const(ValueKind::Object.tag() as i64));
        function.instruction(&Instruction::I64Eq);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_OBJECT_INTERNAL_BRAND_OFFSET,
            receiver_brand_local,
            function,
        );
        function.instruction(&Instruction::End);
        function.instruction(&Instruction::LocalGet(receiver_brand_local));
        function.instruction(&Instruction::I64Const(
            OBJECT_INTERNAL_BRAND_ASYNC_GENERATOR as i64,
        ));
        function.instruction(&Instruction::I64Ne);
        function.instruction(&Instruction::If(BlockType::Empty));
        self.emit_throw_current_function_realm_type_error(
            "AsyncGenerator method called on incompatible receiver",
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.emit_settle_promise_record(
            promise_record_local,
            PromiseSettlement::Reject,
            self.result_local,
            self.result_tag_local,
            function,
        )?;
        self.set_completion_kind(CompletionKind::Normal, function);
        function.instruction(&Instruction::LocalGet(promise_payload_local));
        function.instruction(&Instruction::LocalSet(self.result_local));
        function.instruction(&Instruction::LocalGet(promise_tag_local));
        function.instruction(&Instruction::LocalSet(self.result_tag_local));
        self.emit_return_current_completion(function);
        function.instruction(&Instruction::End);
        "#,
    );
    let (receiver_rejection, receiver_rejection_end) = unique_span(
        &writer,
        &receiver_rejection_sentinel,
        "complete incompatible-receiver rejection",
    );
    let request_construction_sentinel = normalized(
        r#"
        self.load_i64_to_local_from_offset(
            this_payload_local,
            HEAP_ASYNC_GENERATOR_ACTIVATION_OFFSET,
            activation_local,
            function,
        );
        self.emit_builtin_arg_to_locals(
            0,
            argument_payload_local,
            argument_tag_local,
            function,
        );

        self.emit_heap_alloc_const(HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE, function)?;
        function.instruction(&Instruction::LocalSet(request_local));
        let request_completion_kind = match builtin {
            StandardBuiltinId::AsyncGeneratorPrototypeNext => {
                AsyncGeneratorRequestCompletionKind::Normal
            }
            StandardBuiltinId::AsyncGeneratorPrototypeReturn => {
                AsyncGeneratorRequestCompletionKind::Return
            }
            StandardBuiltinId::AsyncGeneratorPrototypeThrow => {
                AsyncGeneratorRequestCompletionKind::Throw
            }
            _ => unreachable!(),
        };
        self.emit_store_async_generator_request_completion_kind(
            request_local,
            request_completion_kind,
            function,
        );
        for (offset, source_local) in [
            (
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET,
                argument_tag_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET,
                argument_payload_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_CAPABILITY_OFFSET,
                capability_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_PAYLOAD_OFFSET,
                promise_payload_local,
            ),
            (
                HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_RECORD_OFFSET,
                promise_record_local,
            ),
        ] {
            self.store_i64_local_at_offset(request_local, offset, source_local, function);
        }
        self.store_i64_const_at_offset(
            request_local,
            HEAP_ASYNC_GENERATOR_REQUEST_NEXT_OFFSET,
            0,
            function,
        );
        "#,
    );
    let (request_construction, request_construction_end) = unique_span(
        &writer,
        &request_construction_sentinel,
        "argument capture and complete request construction",
    );

    let brand = unique_position(
        &writer,
        "OBJECT_INTERNAL_BRAND_ASYNC_GENERATOR",
        "receiver brand validation",
    );
    let capability = unique_position(
        &writer,
        "self.emit_new_promise_capability(constructor_payload_local,constructor_tag_local,capability_local,promise_payload_local,promise_tag_local,function)?;",
        "Promise capability creation",
    );
    assert_eq!(
        writer.matches("emit_new_promise_capability(").count(),
        1,
        "the reviewed capability creation must be the sole call regardless of receiver spelling"
    );
    let allocation = unique_position(
        &writer,
        "self.emit_heap_alloc_const(HEAP_ASYNC_GENERATOR_REQUEST_RECORD_SIZE,function)?;",
        "request allocation",
    );
    let kind_store = unique_position(
        &writer,
        "self.emit_store_async_generator_request_completion_kind(request_local,request_completion_kind,function);",
        "typed request-kind store",
    );
    let argument_capture = unique_position(
        &writer,
        "self.emit_builtin_arg_to_locals(0,argument_payload_local,argument_tag_local,function);",
        "request argument capture",
    );
    assert!(capability < kind_store);
    assert!(receiver_rejection <= brand && brand < receiver_rejection_end);
    assert_eq!(receiver_rejection_end, request_construction);
    assert!(request_construction < argument_capture && argument_capture < allocation);
    assert!(allocation < kind_store && kind_store < request_construction_end);

    let request_value_stores = unique_position(
        &writer,
        &normalized(
            r#"
            for (offset, source_local) in [
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET,
                    argument_tag_local,
                ),
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET,
                    argument_payload_local,
                ),
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_CAPABILITY_OFFSET,
                    capability_local,
                ),
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_PAYLOAD_OFFSET,
                    promise_payload_local,
                ),
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_PROMISE_RECORD_OFFSET,
                    promise_record_local,
                ),
            ] {
                self.store_i64_local_at_offset(request_local, offset, source_local, function);
            }
            "#,
        ),
        "exact request payload, tag and Promise field stores",
    );
    assert!(argument_capture < request_value_stores);
    assert!(request_value_stores < request_construction_end);
    assert_eq!(
        writer
            .matches("self.store_i64_local_at_offset(request_local,")
            .count(),
        1,
        "the exact reviewed field loop must be the only raw request-local store"
    );
    assert_eq!(
        writer
            .matches("self.store_i64_const_at_offset(request_local,")
            .count(),
        1,
        "only the named next-pointer store may write a request-local constant"
    );
    assert_eq!(
        writer
            .matches("self.load_i64_to_local_from_offset(request_local,")
            .count(),
        0,
        "the request writer must not reconstruct any request field"
    );

    let initialized_fields = [
        kind_store,
        request_value_stores,
        unique_position(
            &writer,
            "self.store_i64_const_at_offset(request_local,HEAP_ASYNC_GENERATOR_REQUEST_NEXT_OFFSET,0,function);",
            "request null next pointer",
        ),
    ];
    let head_publication = unique_position(
        &writer,
        "self.store_i64_local_at_offset(activation_local,HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,request_local,function);",
        "queue-head publication",
    );
    let tail_publication = unique_position(
        &writer,
        "self.store_i64_local_at_offset(queue_tail_local,HEAP_ASYNC_GENERATOR_REQUEST_NEXT_OFFSET,request_local,function);",
        "queue-tail-next publication",
    );
    for initialized in initialized_fields {
        assert!(
            initialized < head_publication && initialized < tail_publication,
            "every request field must be initialized before either publication path"
        );
    }
}

#[test]
fn request_readers_route_one_strict_snapshot_and_release_it_last() {
    let drain_owner = drain_owner();
    assert_no_raw_request_kind_offset_alias(drain_owner, "queue-drain reader");
    assert_raw_helper_inventory(drain_owner, "queue-drain reader", [2, 1, 1]);
    let drain = normalized_code(drain_owner);
    assert_eq!(
        drain
            .matches("emit_load_async_generator_request_completion_kind_strict(")
            .count(),
        1
    );
    assert_eq!(
        drain
            .matches("emit_async_generator_request_completion_kind_equals(")
            .count(),
        3
    );
    assert_eq!(
        drain
            .matches("emit_copy_async_generator_request_completion_kind_to_step_completion(")
            .count(),
        2
    );
    assert_eq!(
        drain
            .matches("release_loaded_async_generator_request_completion_kind(")
            .count(),
        1
    );
    assert!(!drain.contains("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET"));
    assert!(!drain.contains("Instruction::Unreachable"));
    assert_eq!(
        drain
            .matches("self.load_i64_to_local_from_offset(request_local,")
            .count(),
        1,
        "drain may read request payload/tag only through its exact reviewed loop"
    );
    assert_eq!(
        drain
            .matches("self.store_i64_local_at_offset(request_local,")
            .count()
            + drain
                .matches("self.store_i64_const_at_offset(request_local,")
                .count(),
        0,
        "drain must never write through a request-local raw offset"
    );
    assert_eq!(
        drain
            .matches("HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET")
            .count(),
        2,
        "drain must have one active publication and one empty-queue clear"
    );
    assert_eq!(
        drain
            .matches("emit_store_async_generator_execution_state(")
            .count(),
        1,
        "only empty-queue cleanup may publish Completed"
    );
    assert_eq!(drain.matches("stop_draining_local").count(), 6);
    assert_eq!(drain.matches("step_completion_kind_local").count(), 8);

    let loop_entry_sentinel = normalized(
        r#"
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(undefined_payload_local));
            function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
            function.instruction(&Instruction::LocalSet(undefined_tag_local));
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(stop_draining_local));
            function.instruction(&Instruction::Block(BlockType::Empty));
            function.instruction(&Instruction::Loop(BlockType::Empty));
            self.load_i64_to_local_from_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
                request_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(request_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::BrIf(1));
            "#,
    );
    let (loop_entry, loop_entry_end) = unique_span(
        &drain,
        &loop_entry_sentinel,
        "undefined terminal value, zeroed drain stop flag and loop entry",
    );
    let request_dataflow_sentinel = normalized(
        r#"
            let request_completion_kind = self
                .emit_load_async_generator_request_completion_kind_strict(request_local, function);
            for (offset, destination_local) in [
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET,
                    completion_payload_local,
                ),
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET,
                    completion_tag_local,
                ),
            ] {
                self.load_i64_to_local_from_offset(
                    request_local,
                    offset,
                    destination_local,
                    function,
                );
            }
            self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
                request_local,
                function,
            );
            "#,
    );
    let (request_dataflow, request_dataflow_end) = unique_span(
        &drain,
        &request_dataflow_sentinel,
        "drain strict kind, payload/tag snapshot and active publication",
    );
    let load = unique_position(
        &drain,
        "self.emit_load_async_generator_request_completion_kind_strict(request_local,function)",
        "drain strict request-kind load",
    );
    let request_value_loads = unique_position(
        &drain,
        &normalized(
            r#"
            for (offset, destination_local) in [
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET,
                    completion_payload_local,
                ),
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET,
                    completion_tag_local,
                ),
            ] {
                self.load_i64_to_local_from_offset(
                    request_local,
                    offset,
                    destination_local,
                    function,
                );
            }
            "#,
        ),
        "drain request payload/tag loads",
    );
    let active = unique_position(
        &drain,
        "self.store_i64_local_at_offset(activation_local,HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,request_local,function);",
        "active-request publication",
    );
    let normal_route_sentinel = normalized(
        r#"
            self.emit_async_generator_request_completion_kind_equals(
                &request_completion_kind,
                AsyncGeneratorRequestCompletionKind::Normal,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_copy_async_generator_request_completion_kind_to_step_completion(
                &request_completion_kind,
                step_completion_kind_local,
                function,
            );
            self.emit_complete_async_generator_step(
                activation_local,
                undefined_payload_local,
                undefined_tag_local,
                step_completion_kind_local,
                AsyncGeneratorCompleteStepKind::Completed,
                function,
            )?;
            function.instruction(&Instruction::Else);
            "#,
    );
    let (normal_route, normal_route_end) = unique_span(
        &drain,
        &normal_route_sentinel,
        "drain Normal comparison and completion arm",
    );
    let throw_route_sentinel = normalized(
        r#"
            self.emit_async_generator_request_completion_kind_equals(
                &request_completion_kind,
                AsyncGeneratorRequestCompletionKind::Throw,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_copy_async_generator_request_completion_kind_to_step_completion(
                &request_completion_kind,
                step_completion_kind_local,
                function,
            );
            self.emit_complete_async_generator_step(
                activation_local,
                completion_payload_local,
                completion_tag_local,
                step_completion_kind_local,
                AsyncGeneratorCompleteStepKind::Completed,
                function,
            )?;
            function.instruction(&Instruction::Else);
            "#,
    );
    let (throw_route, throw_route_end) = unique_span(
        &drain,
        &throw_route_sentinel,
        "drain Throw comparison and rejection arm",
    );
    let return_route_sentinel = normalized(
        r#"
            self.emit_async_generator_request_completion_kind_equals(
                &request_completion_kind,
                AsyncGeneratorRequestCompletionKind::Return,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_async_generator_await_return_reactions(
                activation_local,
                completion_payload_local,
                completion_tag_local,
                resolved_promise_payload_local,
                resolved_promise_tag_local,
                function,
            )?;
            function.instruction(&Instruction::LocalGet(self.completion_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_THROW));
            function.instruction(&Instruction::I64Eq);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.set_completion_kind(CompletionKind::Normal, function);
            function.instruction(&Instruction::I64Const(CompletionKind::Throw.code()));
            function.instruction(&Instruction::LocalSet(step_completion_kind_local));
            self.emit_complete_async_generator_step(
                activation_local,
                resolved_promise_payload_local,
                resolved_promise_tag_local,
                step_completion_kind_local,
                AsyncGeneratorCompleteStepKind::Completed,
                function,
            )?;
            function.instruction(&Instruction::Else);
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(stop_draining_local));
            "#,
    );
    let (return_route, return_route_end) = unique_span(
        &drain,
        &return_route_sentinel,
        "drain Return comparison, AwaitReturn failure and stop assignment",
    );
    let loop_tail_sentinel = normalized(
        r#"
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            self.set_completion_kind(CompletionKind::Normal, function);
            function.instruction(&Instruction::LocalGet(stop_draining_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::I32Eqz);
            function.instruction(&Instruction::BrIf(1));
            function.instruction(&Instruction::Br(0));
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);

            function.instruction(&Instruction::LocalGet(stop_draining_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            self.store_i64_const_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
                0,
                function,
            );
            self.emit_store_async_generator_execution_state(
                activation_local,
                AsyncGeneratorExecutionState::Completed,
                function,
            );
            function.instruction(&Instruction::End);
            "#,
    );
    let (loop_tail, loop_tail_end) = unique_span(
        &drain,
        &loop_tail_sentinel,
        "drain loop stop control and empty-queue cleanup",
    );
    let normal = unique_position(
        &drain,
        "self.emit_async_generator_request_completion_kind_equals(&request_completion_kind,AsyncGeneratorRequestCompletionKind::Normal,function);",
        "drain Normal comparison",
    );
    let throw = unique_position(
        &drain,
        "self.emit_async_generator_request_completion_kind_equals(&request_completion_kind,AsyncGeneratorRequestCompletionKind::Throw,function);",
        "drain Throw comparison",
    );
    let returned = unique_position(
        &drain,
        "self.emit_async_generator_request_completion_kind_equals(&request_completion_kind,AsyncGeneratorRequestCompletionKind::Return,function);",
        "drain Return comparison",
    );
    let copies = positions(
        &drain,
        "self.emit_copy_async_generator_request_completion_kind_to_step_completion(&request_completion_kind,step_completion_kind_local,function);",
    );
    let complete_steps = positions(&drain, "self.emit_complete_async_generator_step(");
    let normal_step = unique_position(
        &drain,
        "self.emit_complete_async_generator_step(activation_local,undefined_payload_local,undefined_tag_local,step_completion_kind_local,AsyncGeneratorCompleteStepKind::Completed,function)?;",
        "drain Normal completion",
    );
    let throw_step = unique_position(
        &drain,
        "self.emit_complete_async_generator_step(activation_local,completion_payload_local,completion_tag_local,step_completion_kind_local,AsyncGeneratorCompleteStepKind::Completed,function)?;",
        "drain Throw completion",
    );
    let failed_return_step = unique_position(
        &drain,
        "self.emit_complete_async_generator_step(activation_local,resolved_promise_payload_local,resolved_promise_tag_local,step_completion_kind_local,AsyncGeneratorCompleteStepKind::Completed,function)?;",
        "failed AwaitReturn rejection",
    );
    assert_eq!(copies.len(), 2);
    assert_eq!(complete_steps.len(), 3);
    assert_eq!(
        complete_steps,
        vec![normal_step, throw_step, failed_return_step]
    );
    assert_eq!(loop_entry_end, request_dataflow);
    assert!(loop_entry < load && load < request_value_loads && request_value_loads < active);
    assert_eq!(request_dataflow_end, normal_route);
    assert_eq!(normal_route, normal);
    assert_eq!(normal_route_end, throw_route);
    assert_eq!(throw_route, throw);
    assert_eq!(throw_route_end, return_route);
    assert_eq!(return_route, returned);
    assert_eq!(return_route_end, loop_tail);
    assert!(normal < copies[0] && copies[0] < normal_step);
    assert!(normal_step < throw && throw < copies[1]);
    assert!(copies[1] < throw_step && throw_step < returned);

    let await_return = unique_position(
        &drain,
        "self.emit_async_generator_await_return_reactions(",
        "AwaitReturn setup",
    );
    let normalizations = positions(
        &drain,
        "self.set_completion_kind(CompletionKind::Normal,function);",
    );
    assert_eq!(
        normalizations.len(),
        2,
        "failed AwaitReturn and the loop tail must each normalize the emitter"
    );
    let normalize_failure = normalizations[0];
    let failed_return_throw = unique_position(
        &drain,
        "Instruction::I64Const(CompletionKind::Throw.code())",
        "fresh generic Throw step kind",
    );
    let stop_stores = positions(&drain, "Instruction::LocalSet(stop_draining_local)");
    assert_eq!(
        stop_stores.len(),
        2,
        "the stop flag must have one initialization and one pending-Return store"
    );
    let stop = stop_stores[1];
    let release = unique_position(
        &drain,
        "self.release_loaded_async_generator_request_completion_kind(request_completion_kind);",
        "drain loaded-kind release",
    );
    assert_eq!(
        drain.matches("release_temp_local(").count(),
        9,
        "drain must retain exactly its reviewed ordinary temporary releases"
    );
    assert!(returned < await_return && await_return < normalize_failure);
    assert!(normalize_failure < failed_return_throw);
    assert!(failed_return_throw < failed_return_step && failed_return_step < stop);
    assert!(stop < loop_tail && loop_tail < release);
    assert_eq!(loop_tail_end, release);
    let stop_release = unique_position(
        &drain,
        "self.release_temp_local(stop_draining_local);",
        "first earlier drain temporary release",
    );
    let first_temp_release = positions(&drain, "self.release_temp_local(")
        .into_iter()
        .next()
        .expect("drain must release its earlier temporaries");
    assert_eq!(first_temp_release, stop_release);
    assert!(release < stop_release);

    let yield_owner = yield_owner();
    assert_no_raw_request_kind_offset_alias(yield_owner, "live-yield reader");
    assert_raw_helper_inventory(yield_owner, "live-yield reader", [2, 3, 0]);
    let yield_route = normalized_code(yield_owner);
    assert_eq!(
        yield_route
            .matches("emit_load_async_generator_request_completion_kind_strict(")
            .count(),
        1
    );
    assert_eq!(
        yield_route
            .matches("emit_async_generator_request_completion_kind_equals(")
            .count(),
        2
    );
    assert_eq!(
        yield_route
            .matches("release_loaded_async_generator_request_completion_kind(")
            .count(),
        1
    );
    assert_eq!(
        yield_route
            .matches("emit_complete_async_generator_step(")
            .count(),
        1,
        "yield must complete exactly the current active request before reading the next queue head"
    );
    assert!(!yield_route.contains("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET"));
    assert!(!yield_route
        .contains("emit_copy_async_generator_request_completion_kind_to_step_completion("));
    assert_eq!(
        yield_route
            .matches("self.load_i64_to_local_from_offset(request_local,")
            .count(),
        1,
        "yield may read request payload/tag only through its exact reviewed loop"
    );
    assert_eq!(
        yield_route
            .matches("self.store_i64_local_at_offset(request_local,")
            .count()
            + yield_route
                .matches("self.store_i64_const_at_offset(request_local,")
                .count(),
        0,
        "yield must never write through a request-local raw offset"
    );
    for (field, expected, label) in [
        (
            "HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET",
            1,
            "active-request publication",
        ),
        (
            "HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET",
            1,
            "resume payload",
        ),
        ("HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET", 1, "resume tag"),
    ] {
        assert_eq!(
            yield_route.matches(field).count(),
            expected,
            "yield must have exactly one reviewed {label} write"
        );
    }
    assert_eq!(
        yield_route
            .matches("emit_store_async_generator_execution_state(")
            .count(),
        1,
        "yield must remain Executing through the typed execution-state boundary"
    );
    assert_eq!(yield_route.matches("resume_body_local").count(), 3);
    assert!(!yield_route.contains("resume_kind_local"));
    assert_eq!(
        yield_route
            .matches("emit_store_async_generator_resume_kind(")
            .count(),
        2,
        "yield must publish Normal or Throw through the typed resume-kind boundary"
    );
    assert_eq!(
        yield_route
            .matches("emit_store_async_generator_body_status(")
            .count(),
        1,
        "yield must publish Await through the typed body-status boundary"
    );

    let yield_request_entry_sentinel = normalized(
        r#"
            function.instruction(&Instruction::I64Const(0));
            function.instruction(&Instruction::LocalSet(resume_body_local));
            function.instruction(&Instruction::I64Const(COMPLETION_KIND_NORMAL));
            function.instruction(&Instruction::LocalSet(completion_kind_local));
            self.emit_complete_async_generator_step(
                activation_local,
                yield_payload_local,
                yield_tag_local,
                completion_kind_local,
                AsyncGeneratorCompleteStepKind::Yielded,
                function,
            )?;
            self.load_i64_to_local_from_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_QUEUE_HEAD_OFFSET,
                request_local,
                function,
            );
            function.instruction(&Instruction::LocalGet(request_local));
            function.instruction(&Instruction::I64Eqz);
            function.instruction(&Instruction::If(BlockType::Empty));
            function.instruction(&Instruction::Else);
            self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_ACTIVE_REQUEST_OFFSET,
                request_local,
                function,
            );
            for (offset, destination_local) in [
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_PAYLOAD_OFFSET,
                    request_payload_local,
                ),
                (
                    HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_TAG_OFFSET,
                    request_tag_local,
                ),
            ] {
                self.load_i64_to_local_from_offset(
                    request_local,
                    offset,
                    destination_local,
                    function,
                );
            }
            let request_completion_kind = self
                .emit_load_async_generator_request_completion_kind_strict(request_local, function);
            "#,
    );
    let (yield_request_entry, yield_request_entry_end) = unique_span(
        &yield_route,
        &yield_request_entry_sentinel,
        "yield prelude, queue selection, active publication and exact request loads",
    );
    let yield_load = unique_position(
        &yield_route,
        "self.emit_load_async_generator_request_completion_kind_strict(request_local,function)",
        "yield strict request-kind load",
    );
    let yield_return_route_sentinel = normalized(
        r#"
            self.emit_async_generator_request_completion_kind_equals(
                &request_completion_kind,
                AsyncGeneratorRequestCompletionKind::Return,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_async_generator_yield_return_reactions(
                activation_local,
                request_payload_local,
                request_tag_local,
                function,
            )?;
            self.emit_store_async_generator_body_status(
                activation_local,
                AsyncGeneratorBodyStatus::Await,
                function,
            );
            self.emit_store_async_generator_execution_state(
                activation_local,
                AsyncGeneratorExecutionState::Executing,
                function,
            );
            function.instruction(&Instruction::Else);
            "#,
    );
    let (yield_return_route, yield_return_route_end) = unique_span(
        &yield_route,
        &yield_return_route_sentinel,
        "yield Return comparison and Await suspension arm",
    );
    let yield_normal_throw_route_sentinel = normalized(
        r#"
            function.instruction(&Instruction::I64Const(1));
            function.instruction(&Instruction::LocalSet(resume_body_local));
            self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,
                request_payload_local,
                function,
            );
            self.store_i64_local_at_offset(
                activation_local,
                HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,
                request_tag_local,
                function,
            );
            self.emit_async_generator_request_completion_kind_equals(
                &request_completion_kind,
                AsyncGeneratorRequestCompletionKind::Throw,
                function,
            );
            function.instruction(&Instruction::If(BlockType::Empty));
            self.emit_store_async_generator_resume_kind(
                activation_local,
                AsyncGeneratorResumeKind::Throw,
                function,
            );
            function.instruction(&Instruction::Else);
            self.emit_store_async_generator_resume_kind(
                activation_local,
                AsyncGeneratorResumeKind::Normal,
                function,
            );
            function.instruction(&Instruction::End);
            function.instruction(&Instruction::End);
            "#,
    );
    let (yield_normal_throw_route, yield_normal_throw_route_end) = unique_span(
        &yield_route,
        &yield_normal_throw_route_sentinel,
        "yield Normal/Throw resumption assignments and resume-kind store",
    );
    let yield_exit_sentinel = normalized(
        r#"
            function.instruction(&Instruction::End);
            self.release_loaded_async_generator_request_completion_kind(request_completion_kind);
            "#,
    );
    let (yield_exit, _) = unique_span(
        &yield_route,
        &yield_exit_sentinel,
        "yield queue-guard close and loaded-kind release",
    );
    let yield_return = unique_position(
        &yield_route,
        "self.emit_async_generator_request_completion_kind_equals(&request_completion_kind,AsyncGeneratorRequestCompletionKind::Return,function);",
        "yield Return comparison",
    );
    let yield_await = unique_position(
        &yield_route,
        "self.emit_async_generator_yield_return_reactions(",
        "yield Return Await setup",
    );
    let body_await = unique_position(
        &yield_route,
        "AsyncGeneratorBodyStatus::Await",
        "yield Return body Await state",
    );
    let executing_await = unique_position(
        &yield_route,
        "AsyncGeneratorExecutionState::Executing",
        "yield Return execution state",
    );
    let normal_resume = unique_position(
        &yield_route,
        "AsyncGeneratorResumeKind::Normal",
        "yield Normal resume kind",
    );
    let resume_payload = unique_position(
        &yield_route,
        "self.store_i64_local_at_offset(activation_local,HEAP_ASYNC_GENERATOR_RESUME_PAYLOAD_OFFSET,request_payload_local,function);",
        "yield request payload resumption",
    );
    let resume_tag = unique_position(
        &yield_route,
        "self.store_i64_local_at_offset(activation_local,HEAP_ASYNC_GENERATOR_RESUME_TAG_OFFSET,request_tag_local,function);",
        "yield request tag resumption",
    );
    let yield_throw = unique_position(
        &yield_route,
        "self.emit_async_generator_request_completion_kind_equals(&request_completion_kind,AsyncGeneratorRequestCompletionKind::Throw,function);",
        "yield Throw comparison",
    );
    let throw_resume = unique_position(
        &yield_route,
        "AsyncGeneratorResumeKind::Throw",
        "yield Throw resume kind",
    );
    let yield_release = unique_position(
        &yield_route,
        "self.release_loaded_async_generator_request_completion_kind(request_completion_kind);",
        "yield loaded-kind release",
    );
    assert_eq!(
        yield_route.matches("release_temp_local(").count(),
        4,
        "yield must retain exactly its reviewed ordinary temporary releases"
    );
    assert!(yield_request_entry < yield_load);
    assert_eq!(yield_request_entry_end, yield_return_route);
    assert_eq!(yield_return_route, yield_return);
    assert!(yield_return < yield_await);
    assert!(yield_await < body_await && body_await < executing_await);
    assert!(executing_await < yield_return_route_end);
    assert_eq!(yield_return_route_end, yield_normal_throw_route);
    assert!(yield_normal_throw_route < resume_payload && resume_payload < resume_tag);
    assert!(resume_tag < yield_throw && yield_throw < throw_resume);
    assert!(throw_resume < normal_resume && normal_resume < yield_normal_throw_route_end);
    assert_eq!(yield_normal_throw_route_end, yield_exit);
    assert!(yield_exit < yield_release);
    let first_expected_temp_release = unique_position(
        &yield_route,
        "self.release_temp_local(request_tag_local);",
        "first earlier yield temporary release",
    );
    let first_yield_temp_release = positions(&yield_route, "self.release_temp_local(")
        .into_iter()
        .next()
        .expect("yield must release its earlier temporaries");
    assert_eq!(first_yield_temp_release, first_expected_temp_release);
    assert!(yield_release < first_expected_temp_release);

    let return_arm = yield_route
        .split_once(
            "self.emit_async_generator_request_completion_kind_equals(&request_completion_kind,AsyncGeneratorRequestCompletionKind::Return,function);",
        )
        .expect("yield Return comparison should exist")
        .1
        .split_once("function.instruction(&Instruction::Else);")
        .expect("yield Return route should retain an explicit Else")
        .0;
    assert!(
        !return_arm.contains("resume_body_local"),
        "Return must not directly resume the async-generator body"
    );
}

#[test]
fn request_completion_kind_has_exactly_one_writer_and_two_readers() {
    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    let mut sources = Vec::new();
    collect_rust_sources(&source_root, &mut sources);

    let mut totals = [0; 6];
    for (path, source) in sources {
        let relative = path
            .strip_prefix(&source_root)
            .expect("collected source must remain below src")
            .to_string_lossy();
        let expected = match relative.as_ref() {
            "heap.rs" => (4, 1, 1, 1, 1, 1),
            "builtins/standard.rs" => (0, 1, 0, 0, 0, 0),
            "builtins/promise.rs" => (0, 0, 2, 5, 2, 2),
            _ => (0, 0, 0, 0, 0, 0),
        };
        let actual = (
            source
                .matches("HEAP_ASYNC_GENERATOR_REQUEST_COMPLETION_KIND_OFFSET")
                .count(),
            source
                .matches("emit_store_async_generator_request_completion_kind(")
                .count(),
            source
                .matches("emit_load_async_generator_request_completion_kind_strict(")
                .count(),
            source
                .matches("emit_async_generator_request_completion_kind_equals(")
                .count(),
            source
                .matches("emit_copy_async_generator_request_completion_kind_to_step_completion(")
                .count(),
            source
                .matches("release_loaded_async_generator_request_completion_kind(")
                .count(),
        );
        assert_eq!(
            actual, expected,
            "unexpected async-generator request completion-kind owner in {relative}"
        );
        totals[0] += actual.0;
        totals[1] += actual.1;
        totals[2] += actual.2;
        totals[3] += actual.3;
        totals[4] += actual.4;
        totals[5] += actual.5;
    }

    assert_eq!(totals, [4, 2, 3, 6, 3, 3]);
    assert_eq!(
        STANDARD_SOURCE
            .matches("AsyncGeneratorRequestCompletionKind::")
            .count(),
        3,
        "the sole writer must map exactly three builtin entry points"
    );
    assert_eq!(
        PROMISE_SOURCE
            .matches("AsyncGeneratorRequestCompletionKind::")
            .count(),
        5,
        "the two readers must emit exactly the inventoried route comparisons"
    );
}
