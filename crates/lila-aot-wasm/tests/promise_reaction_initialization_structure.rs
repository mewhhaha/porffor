use std::fs;
use std::path::Path;

const PROMISE_SOURCE: &str = include_str!("../src/builtins/promise.rs");
const CLI_TESTS: &str = include_str!("../../lila-cli/tests/cli/functions.rs");
const CLI_FIXTURE: &str =
    include_str!("../../lila-cli/tests/fixtures/wasm_async_execution_realm.js");

fn bounded<'a>(source: &'a str, start: &str, end: &str) -> &'a str {
    source
        .split_once(start)
        .unwrap_or_else(|| panic!("missing start marker `{start}`"))
        .1
        .split_once(end)
        .unwrap_or_else(|| panic!("missing end marker `{end}` after `{start}`"))
        .0
}

fn quoted_literal_end(source: &str, quote_start: usize, quote: u8) -> Option<usize> {
    let bytes = source.as_bytes();
    let mut offset = quote_start + 1;
    let mut escaped = false;
    while offset < bytes.len() {
        let byte = bytes[offset];
        if escaped {
            escaped = false;
        } else if byte == b'\\' {
            escaped = true;
        } else if byte == quote {
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
        b'"' => quoted_literal_end(source, start, b'"'),
        b'\'' => character_literal_end(source, start),
        b'b' if bytes.get(start + 1) == Some(&b'\'') => character_literal_end(source, start + 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'"') => {
            quoted_literal_end(source, start + 1, b'"')
        }
        b'r' => raw_literal_end(source, start, 1),
        b'b' | b'c' if bytes.get(start + 1) == Some(&b'r') => raw_literal_end(source, start, 2),
        _ => None,
    }
}

struct NormalizedRust {
    code: String,
    identifiers: String,
    routes: String,
}

fn normalize_rust(source: &str) -> NormalizedRust {
    let bytes = source.as_bytes();
    let mut code = String::new();
    let mut identifiers = String::new();
    let mut routes = String::new();
    let mut offset = 0;
    while offset < bytes.len() {
        if let Some(end) = literal_end(source, offset) {
            code.push_str(&source[offset..end]);
            identifiers.push(' ');
            routes.push('L');
            offset = end;
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"//") {
            identifiers.push(' ');
            offset += 2;
            while bytes.get(offset).is_some_and(|byte| *byte != b'\n') {
                offset += 1;
            }
            continue;
        }
        if bytes.get(offset..offset + 2) == Some(b"/*") {
            identifiers.push(' ');
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
        if bytes.get(offset..offset + 2) == Some(b"r#")
            && source[offset + 2..]
                .chars()
                .next()
                .is_some_and(|character| character == '_' || character.is_alphabetic())
        {
            offset += 2;
            continue;
        }
        let character = source[offset..].chars().next().unwrap();
        if !character.is_whitespace() {
            code.push(character);
            routes.push(character);
        }
        if character.is_whitespace() {
            identifiers.push(' ');
        } else {
            identifiers.push(character);
        }
        offset += character.len_utf8();
    }
    NormalizedRust {
        code,
        identifiers,
        routes,
    }
}

fn normalized(source: &str) -> String {
    normalize_rust(source).code
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

fn exact_route_count(source: &str, route: &str) -> usize {
    source
        .match_indices(route)
        .filter(|(offset, _)| {
            let before = source[..*offset].chars().next_back();
            let after = source[*offset + route.len()..].chars().next();
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
            exact_identifier_count(&normalize_rust(&source).identifiers, identifier)
        })
        .sum()
}

fn count_route_in_rust_sources(dir: &Path, route: &str) -> usize {
    fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .map(|path| {
            if path.is_dir() {
                return count_route_in_rust_sources(&path, route);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return 0;
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            exact_route_count(&normalize_rust(&source).routes, route)
        })
        .sum()
}

fn normalized_routes_in_rust_sources(dir: &Path) -> String {
    let mut paths = fs::read_dir(dir)
        .unwrap_or_else(|error| panic!("failed to read {}: {error}", dir.display()))
        .map(|entry| entry.expect("failed to read Rust source entry").path())
        .collect::<Vec<_>>();
    paths.sort();
    paths
        .into_iter()
        .map(|path| {
            if path.is_dir() {
                return normalized_routes_in_rust_sources(&path);
            }
            if path.extension().and_then(|extension| extension.to_str()) != Some("rs") {
                return String::new();
            }
            let source = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("failed to read {}: {error}", path.display()));
            let mut routes = normalize_rust(&source).routes;
            routes.push('\n');
            routes
        })
        .collect()
}

#[test]
fn reaction_initialization_is_the_exact_private_capability_free_domain() {
    let lexical_probe = r###"
        // AsyncAwaitContinuation::AsyncFunction
        AsyncAwaitContinuation /* nested /* ignored */ comment */ :: r#AsyncGeneratorBody;
        "PromiseReactionInitialization"; b"AsyncAwaitContinuation";
        c"PromiseReactionInitialization"; r"AsyncAwaitContinuation";
        br##"AsyncAwaitContinuation::AsyncGeneratorYield"##;
        cr#"PromiseReactionInitialization::Default"#;
        'A'; b'R'; 'lifetime;
    "###;
    let lexical_probe = normalize_rust(lexical_probe);
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "AsyncAwaitContinuation"),
        1
    );
    assert_eq!(
        exact_route_count(
            &lexical_probe.routes,
            "AsyncAwaitContinuation::AsyncGeneratorBody"
        ),
        1
    );
    assert_eq!(
        exact_identifier_count(&lexical_probe.identifiers, "lifetime"),
        1
    );

    let continuation_declaration = normalize_rust(bounded(
        PROMISE_SOURCE,
        concat!(
            "    ResolveThenable {\n",
            "        thenable_job_local: u32,\n",
            "        then_payload_local: u32,\n",
            "        then_tag_local: u32,\n",
            "    },\n",
            "}\n"
        ),
        "/// Whether an async-generator request publishes a yielded or terminal result.",
    ));
    assert_eq!(
        continuation_declaration.code,
        concat!(
            "enumAsyncAwaitContinuation{AsyncFunction,AsyncGeneratorBody,",
            "AsyncGeneratorAwaitReturn,AsyncGeneratorYield,AsyncGeneratorYieldReturn,}"
        ),
        "the continuation declaration must remain private and attribute-free"
    );

    let initialization_declaration = normalize_rust(bounded(
        PROMISE_SOURCE,
        concat!(
            "enum PromiseResolveRealmAuthority<'a> {\n",
            "    CurrentFunction,\n",
            "    AsyncExecution(&'a AsyncExecutionRealmContext),\n",
            "}\n"
        ),
        "/// The inseparable Realm-owned fields",
    ));
    assert_eq!(
        initialization_declaration.code,
        concat!(
            "enumPromiseReactionInitialization<'a>{Default,",
            "AsyncExecution{realm:&'aAsyncExecutionRealmContext,",
            "continuation:AsyncAwaitContinuation,},}"
        ),
        "the reaction initialization declaration must remain private and attribute-free"
    );

    let source_root = Path::new(env!("CARGO_MANIFEST_DIR")).join("src");
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "PromiseReactionInitialization"),
        11,
        "the declaration, two typed parameters, four producers and four exhaustive arms own every mention"
    );
    assert_eq!(
        count_route_in_rust_sources(&source_root, "PromiseReactionInitialization::Default"),
        4
    );
    assert_eq!(
        count_route_in_rust_sources(
            &source_root,
            "PromiseReactionInitialization::AsyncExecution"
        ),
        4
    );
    assert_eq!(
        count_identifier_in_rust_sources(&source_root, "AsyncAwaitContinuation"),
        17,
        "declaration, impl, nested field, three owned parameters and eleven variant routes own every mention"
    );
    for (variant, count) in [
        ("AsyncFunction", 2),
        ("AsyncGeneratorBody", 2),
        ("AsyncGeneratorAwaitReturn", 3),
        ("AsyncGeneratorYield", 2),
        ("AsyncGeneratorYieldReturn", 2),
    ] {
        assert_eq!(
            count_route_in_rust_sources(
                &source_root,
                &format!("AsyncAwaitContinuation::{variant}")
            ),
            count,
            "producer/Realm route census drifted for {variant}"
        );
    }
    let all_routes = normalized_routes_in_rust_sources(&source_root);
    for domain in ["PromiseReactionInitialization", "AsyncAwaitContinuation"] {
        for capability in ["Clone", "Copy", "Debug", "PartialEq", "Eq", "Default"] {
            assert!(!all_routes.contains(&format!("impl{capability}for{domain}")));
        }
        assert!(!all_routes.contains(&format!("{domain}as")));
    }
    assert!(!all_routes.contains("impl<'a>PromiseReactionInitialization"));
}

#[test]
fn four_named_producers_construct_the_exact_initialization_policy() {
    let direct_wrappers = normalized(bounded(
        PROMISE_SOURCE,
        "    fn emit_initialize_default_promise_reaction(",
        "    fn emit_append_promise_reaction(",
    ));
    let default_direct = concat!(
        "self.emit_initialize_promise_reaction(reaction_record_local,",
        "capability_record_local,handler_payload_local,handler_tag_local,reaction_type,",
        "&PromiseReactionInitialization::Default,function,)"
    );
    let async_direct = concat!(
        "self.emit_initialize_promise_reaction(reaction_record_local,",
        "capability_record_local,handler_payload_local,handler_tag_local,reaction_type,",
        "&PromiseReactionInitialization::AsyncExecution{realm,continuation,},function,)"
    );
    assert_eq!(direct_wrappers.matches(default_direct).count(), 1);
    assert_eq!(direct_wrappers.matches(async_direct).count(), 1);
    assert_eq!(
        direct_wrappers
            .matches("emit_initialize_promise_reaction(")
            .count(),
        2
    );

    let await_wrappers = normalized(bounded(
        PROMISE_SOURCE,
        "    fn emit_default_intrinsic_await_reactions(",
        "    fn emit_intrinsic_await_reactions(",
    ));
    let default_await = concat!(
        "self.emit_intrinsic_await_reactions(reaction_capability_record_local,",
        "value_payload_local,value_tag_local,on_fulfilled_payload_local,",
        "on_fulfilled_tag_local,on_rejected_payload_local,on_rejected_tag_local,",
        "PromiseReactionInitialization::Default,function,)"
    );
    let async_await = concat!(
        "self.emit_intrinsic_await_reactions(reaction_capability_record_local,",
        "value_payload_local,value_tag_local,on_fulfilled_payload_local,",
        "on_fulfilled_tag_local,on_rejected_payload_local,on_rejected_tag_local,",
        "PromiseReactionInitialization::AsyncExecution{realm,continuation,},function,)"
    );
    assert_eq!(await_wrappers.matches(default_await).count(), 1);
    assert_eq!(await_wrappers.matches(async_await).count(), 1);
    assert_eq!(
        await_wrappers
            .matches("emit_intrinsic_await_reactions(")
            .count(),
        2
    );

    let continuation_wrappers = normalized(&format!(
        "pub(crate) fn emit_async_await_reactions({}",
        bounded(
            PROMISE_SOURCE,
            "    pub(crate) fn emit_async_await_reactions(",
            "    pub(crate) fn emit_intrinsic_await_with_handlers(",
        )
    ));
    let mut previous = 0;
    for (function_name, variant) in [
        ("emit_async_await_reactions", "AsyncFunction"),
        ("emit_async_generator_await_reactions", "AsyncGeneratorBody"),
        (
            "emit_async_generator_yield_reactions",
            "AsyncGeneratorYield",
        ),
        (
            "emit_async_generator_yield_return_reactions",
            "AsyncGeneratorYieldReturn",
        ),
    ] {
        let producer = format!(
            "pub(crate)fn{function_name}(&mutself,activation_local:u32,value_payload_local:u32,\
             value_tag_local:u32,function:&mutFunction,)->Result<(),EmitError>{{\
             self.emit_await_reactions(activation_local,value_payload_local,value_tag_local,\
             AsyncAwaitContinuation::{variant},function,)}}"
        );
        assert_eq!(
            continuation_wrappers.matches(&producer).count(),
            1,
            "{variant}"
        );
        let offset = continuation_wrappers.find(&producer).unwrap();
        assert!(previous <= offset, "continuation producer order drifted");
        previous = offset;
    }
    assert_eq!(
        continuation_wrappers
            .matches("self.emit_await_reactions(")
            .count(),
        4
    );

    let await_return = normalized(bounded(
        PROMISE_SOURCE,
        "    pub(crate) fn emit_async_generator_await_return_reactions(",
        "    pub(crate) fn emit_promise_prototype_then(",
    ));
    let fulfill = concat!(
        "self.emit_initialize_async_execution_promise_reaction(",
        "fulfill_reaction_local,activation_local,undefined_payload_local,undefined_tag_local,",
        "PromiseReactionType::Fulfill,&realm,",
        "AsyncAwaitContinuation::AsyncGeneratorAwaitReturn,function,)?;"
    );
    let reject = concat!(
        "self.emit_initialize_async_execution_promise_reaction(",
        "reject_reaction_local,activation_local,undefined_payload_local,undefined_tag_local,",
        "PromiseReactionType::Reject,&realm,",
        "AsyncAwaitContinuation::AsyncGeneratorAwaitReturn,function,)?;"
    );
    assert_eq!(await_return.matches(fulfill).count(), 1);
    assert_eq!(await_return.matches(reject).count(), 1);
    assert!(await_return.find(fulfill).unwrap() < await_return.find(reject).unwrap());
    assert!(
        await_return.find(reject).unwrap()
            < await_return
                .find("self.release_async_execution_realm_context(realm);")
                .unwrap()
    );
}

#[test]
fn consumers_borrow_one_policy_for_resolve_and_both_reactions() {
    let continuation_projection = normalized(bounded(
        PROMISE_SOURCE,
        "impl AsyncAwaitContinuation {",
        "impl<'a> FunctionBuilder<'a> {",
    ));
    assert_eq!(
        continuation_projection,
        concat!(
            "fnreaction_callback_kind(&self)->PromiseReactionCallbackKind{matchself{",
            "Self::AsyncFunction=>PromiseReactionCallbackKind::AsyncFunction,",
            "Self::AsyncGeneratorBody=>PromiseReactionCallbackKind::AsyncGeneratorAwait,",
            "Self::AsyncGeneratorAwaitReturn=>{",
            "PromiseReactionCallbackKind::AsyncGeneratorAwaitReturn}",
            "Self::AsyncGeneratorYield=>PromiseReactionCallbackKind::AsyncGeneratorYield,",
            "Self::AsyncGeneratorYieldReturn=>{",
            "PromiseReactionCallbackKind::AsyncGeneratorYieldReturn}}}}"
        ),
        "all five continuation rows must project through one borrowed exhaustive match"
    );

    let initializer = bounded(
        PROMISE_SOURCE,
        "    fn emit_initialize_promise_reaction(",
        "    fn emit_initialize_default_promise_reaction(",
    );
    assert!(initializer.contains("initialization: &PromiseReactionInitialization<'_>,"));
    let callback_projection = normalized(bounded(
        initializer,
        "        let callback_kind = match initialization {",
        "        };",
    ));
    assert_eq!(
        callback_projection,
        concat!(
            "PromiseReactionInitialization::Default=>{",
            "self.store_i64_const_at_offset(reaction_record_local,",
            "HEAP_PROMISE_REACTION_REALM_OFFSET,0,function,);",
            "PromiseReactionCallbackKind::Default}",
            "PromiseReactionInitialization::AsyncExecution{realm,continuation,}=>{",
            "self.store_i64_local_at_offset(reaction_record_local,",
            "HEAP_PROMISE_REACTION_REALM_OFFSET,realm.realm_local,function,);",
            "continuation.reaction_callback_kind()}"
        )
    );

    let intrinsic = bounded(
        PROMISE_SOURCE,
        "    fn emit_intrinsic_await_reactions(",
        "    pub(crate) fn emit_async_generator_await_return_reactions(",
    );
    assert!(intrinsic.contains("initialization: PromiseReactionInitialization<'_>,"));
    let resolve_projection = normalized(bounded(
        intrinsic,
        "        let resolve_realm_authority = match &initialization {",
        "        };",
    ));
    assert_eq!(
        resolve_projection,
        concat!(
            "PromiseReactionInitialization::Default=>",
            "PromiseResolveRealmAuthority::CurrentFunction,",
            "PromiseReactionInitialization::AsyncExecution{realm,..}=>{",
            "PromiseResolveRealmAuthority::AsyncExecution(*realm)}"
        )
    );

    let normalized_intrinsic = normalized(intrinsic);
    let fulfill = concat!(
        "self.emit_initialize_promise_reaction(fulfill_reaction_local,",
        "reaction_capability_record_local,on_fulfilled_payload_local,",
        "on_fulfilled_tag_local,PromiseReactionType::Fulfill,&initialization,function,)?;"
    );
    let reject = concat!(
        "self.emit_initialize_promise_reaction(reject_reaction_local,",
        "reaction_capability_record_local,on_rejected_payload_local,on_rejected_tag_local,",
        "PromiseReactionType::Reject,&initialization,function,)?;"
    );
    assert_eq!(normalized_intrinsic.matches(fulfill).count(), 1);
    assert_eq!(normalized_intrinsic.matches(reject).count(), 1);
    let projection_offset = normalized_intrinsic
        .find("letresolve_realm_authority=match&initialization{")
        .unwrap();
    let resolve_offset = normalized_intrinsic
        .find("self.emit_intrinsic_promise_resolve_realm_context(resolve_realm_authority,function)")
        .unwrap();
    let fulfill_offset = normalized_intrinsic.find(fulfill).unwrap();
    let reject_offset = normalized_intrinsic.find(reject).unwrap();
    assert!(projection_offset < resolve_offset);
    assert!(resolve_offset < fulfill_offset);
    assert!(fulfill_offset < reject_offset);

    assert_eq!(normalized_intrinsic.matches("&initialization").count(), 3);
    assert!(normalized_intrinsic.ends_with(concat!(
        "self.release_temp_local(reject_reaction_local);",
        "self.release_temp_local(fulfill_reaction_local);",
        "self.release_temp_local(source_result_tag_local);",
        "self.release_temp_local(source_result_payload_local);",
        "self.release_temp_local(resolve_error_tag_local);",
        "self.release_temp_local(resolve_error_payload_local);",
        "self.release_temp_local(rejected_promise_constructor_tag_local);",
        "self.release_temp_local(rejected_promise_capability_local);",
        "self.release_temp_local(awaited_promise_record_local);",
        "self.release_temp_local(awaited_promise_payload_local);Ok(())}"
    )));

    let await_outer = normalized(bounded(
        PROMISE_SOURCE,
        "    fn emit_await_reactions(",
        "    fn emit_default_intrinsic_await_reactions(",
    ));
    let expected_await_outer = r#"
        &mut self,
        activation_local: u32,
        value_payload_local: u32,
        value_tag_local: u32,
        continuation: AsyncAwaitContinuation,
        function: &mut Function,
    ) -> Result<(), EmitError> {
        let realm = match &continuation {
            AsyncAwaitContinuation::AsyncFunction => self
                .emit_async_function_execution_realm_context_from_activation(
                    activation_local,
                    function,
                ),
            AsyncAwaitContinuation::AsyncGeneratorBody
            | AsyncAwaitContinuation::AsyncGeneratorAwaitReturn
            | AsyncAwaitContinuation::AsyncGeneratorYield
            | AsyncAwaitContinuation::AsyncGeneratorYieldReturn => self
                .emit_async_generator_execution_realm_context_from_activation(
                    activation_local,
                    function,
                ),
        };
        let undefined_payload_local = self.reserve_temp_local();
        let undefined_tag_local = self.reserve_temp_local();
        function.instruction(&Instruction::I64Const(0));
        function.instruction(&Instruction::LocalSet(undefined_payload_local));
        function.instruction(&Instruction::I64Const(ValueKind::Undefined.tag() as i64));
        function.instruction(&Instruction::LocalSet(undefined_tag_local));
        let result = self.emit_async_execution_intrinsic_await_reactions(
            activation_local,
            value_payload_local,
            value_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            undefined_payload_local,
            undefined_tag_local,
            &realm,
            continuation,
            function,
        );
        self.release_temp_local(undefined_tag_local);
        self.release_temp_local(undefined_payload_local);
        self.release_async_execution_realm_context(realm);
        result
    }

"#;
    assert_eq!(
        await_outer,
        normalized(expected_await_outer),
        "owned continuation must be borrowed for Realm selection, moved once, then released in order"
    );

    let async_bridge = normalized(bounded(
        PROMISE_SOURCE,
        "    fn emit_async_execution_intrinsic_await_reactions(",
        "    fn emit_intrinsic_await_reactions(",
    ));
    assert_eq!(
        async_bridge,
        concat!(
            "&mutself,reaction_capability_record_local:u32,value_payload_local:u32,",
            "value_tag_local:u32,on_fulfilled_payload_local:u32,on_fulfilled_tag_local:u32,",
            "on_rejected_payload_local:u32,on_rejected_tag_local:u32,",
            "realm:&AsyncExecutionRealmContext,continuation:AsyncAwaitContinuation,",
            "function:&mutFunction,)->Result<(),EmitError>{",
            "self.emit_intrinsic_await_reactions(reaction_capability_record_local,",
            "value_payload_local,value_tag_local,on_fulfilled_payload_local,",
            "on_fulfilled_tag_local,on_rejected_payload_local,on_rejected_tag_local,",
            "PromiseReactionInitialization::AsyncExecution{realm,continuation,},function,)}"
        ),
        "the bridge must move its owned continuation into initialization exactly once"
    );

    for forbidden in ["_=>", "initialization==", "initialization!=", "matches!("] {
        assert!(
            !normalized_intrinsic.contains(forbidden),
            "initialization consumer contains `{forbidden}`"
        );
    }
}

#[test]
fn existing_fixture_covers_default_and_async_execution_reactions() {
    assert!(CLI_TESTS
        .contains("fn run_wasm_backend_uses_async_function_realms_for_promises_and_reactions()"));
    for marker in [
        "other.Promise.resolve(0).then(",
        "await value;",
        "async captured reaction Realm",
        "async-generator captured reaction Realm",
        "async-execution-realm:ok",
    ] {
        assert!(
            CLI_FIXTURE.contains(marker),
            "missing fixture marker `{marker}`"
        );
    }
}
