use super::*;
use crate::direct_eval_context::ScriptImportOrigin;
use crate::{
    DirectEvalContextIr, DirectEvalDerivedOwner, ExprIr, PreparedScriptOutcome, PrivateNameId,
    StatementIr, Strictness,
};
use lila_front::{decode_js_string_utf16, EvalInvocationContext, ParseDiagnosticKind};
use std::collections::BTreeMap;
mod empty;
mod regexp_literal;

fn context(strictness: Strictness, invocation: EvalInvocationContext) -> DirectEvalContextIr {
    DirectEvalContextIr::new(
        strictness,
        invocation,
        BTreeMap::new(),
        (invocation == EvalInvocationContext::DerivedConstructor)
            .then(|| "foreign.constructor".into()),
    )
}
fn caller(context: DirectEvalContextIr) -> RuntimeDirectEvalCaller {
    RuntimeDirectEvalCaller::validate(context.runtime_descriptor(), []).unwrap()
}
fn compile(
    caller: &RuntimeDirectEvalCaller,
    source: &str,
) -> Result<RuntimeDirectEvalProgram, RuntimeDirectEvalCompileError> {
    lower_runtime_direct_eval(
        caller,
        &source.encode_utf16().collect::<Vec<_>>(),
        HostSurfacePolicy::default(),
    )
}
fn root(program: &RuntimeDirectEvalProgram) -> &crate::PreparedScriptUnit {
    program
        .program()
        .script
        .as_ref()
        .unwrap()
        .prepared_script_units()
        .find(|unit| unit.id == program.script_id())
        .unwrap()
}
fn syntax(caller: &RuntimeDirectEvalCaller, source: &str) {
    let RuntimeDirectEvalCompileError::Parse(error) = compile(caller, source).unwrap_err() else {
        panic!("actual early error must remain a parser error")
    };
    assert_eq!(
        error.diagnostic().kind(),
        ParseDiagnosticKind::MalformedJavaScript
    );
}

#[test]
fn caller_strictness_and_direct_source_strictness_select_prepared_environment() {
    for strictness in [Strictness::Sloppy, Strictness::Strict] {
        let caller = caller(context(strictness, EvalInvocationContext::Function));
        for directive in ["", "'use strict';"] {
            let compiled = compile(
                &caller,
                &format!("{directive} var local = 17; let lexical = 23; 42;"),
            )
            .unwrap();
            let script = compiled.program().script.as_ref().unwrap();
            assert!(script.body.statements.is_empty());
            assert!(script.owned_env_bindings.is_empty());
            assert!(script.runtime_declarations.var_names.is_empty());
            let unit = root(&compiled);
            assert_eq!(
                unit.kind,
                PreparedScriptKind::DirectEval(caller.context().clone())
            );
            assert_eq!(
                unit.strict,
                strictness == Strictness::Strict || !directive.is_empty()
            );
            assert!(!unit.has_global_variable_environment());
            assert_eq!(unit.declarations.var_names, ["local"]);
            assert_eq!(unit.declarations.lexical_names_in_source_order, ["lexical"]);
        }
    }
    let sloppy = caller(context(Strictness::Sloppy, EvalInvocationContext::Function));
    let strict = caller(context(Strictness::Strict, EvalInvocationContext::Function));
    assert!(compile(&sloppy, "010;").is_ok());
    syntax(&strict, "010;");
    syntax(&sloppy, "'use strict'; 010;");
}

#[test]
fn actual_caller_grammar_controls_new_target_super_arguments_and_return() {
    use EvalInvocationContext::*;
    let script = caller(context(Strictness::Sloppy, Script));
    let function = caller(context(Strictness::Sloppy, Function));
    let method = caller(context(Strictness::Sloppy, Method));
    let derived = caller(context(Strictness::Strict, DerivedConstructor));
    let initializer = caller(context(Strictness::Strict, ClassFieldInitializer));
    syntax(&script, "new.target;");
    assert!(compile(&function, "new.target;").is_ok());
    syntax(&function, "super.x;");
    assert!(compile(&method, "super.x;").is_ok());
    syntax(&method, "super();");
    let compiled = compile(&derived, "super();").unwrap();
    assert!(
        matches!(root(&compiled).body.statements.last(), Some(StatementIr::Expression(value)) if matches!(value.expr, ExprIr::SuperConstruct { .. }))
    );
    syntax(&initializer, "arguments;");
    for caller in [&script, &function, &method, &derived, &initializer] {
        syntax(caller, "return 1;");
    }
}

#[test]
fn descriptor_roundtrip_retains_private_mapping_and_foreign_roles_canonically() {
    let mut context = context(
        Strictness::Strict,
        EvalInvocationContext::DerivedConstructor,
    )
    .with_script_import_origin(Some(ScriptImportOrigin::LoadedRecord(27)));
    context = DirectEvalContextIr::new(
        context.strictness(),
        context.invocation(),
        BTreeMap::from([("秘密".into(), PrivateNameId::new(7, 2))]),
        Some("P1.owner".into()),
    )
    .with_script_import_origin(context.script_import_origin());
    let descriptor = context.runtime_descriptor();
    assert_eq!(
        descriptor.derived_owner(),
        RuntimeDirectEvalDerivedOwner::ExternalCaller
    );
    assert_eq!(
        descriptor.import_origin(),
        RuntimeDirectEvalImportOrigin::CallerScriptRecord(27)
    );
    assert_eq!(descriptor.private_names()["秘密"], PrivateNameId::new(7, 2));
    let bytes = descriptor.encode();
    assert_eq!(
        RuntimeDirectEvalCallerDescriptor::decode(&bytes).unwrap(),
        descriptor
    );
    let validated =
        RuntimeDirectEvalCaller::validate(descriptor, [RuntimeDirectEvalPrivateScope::new(7, 3)])
            .unwrap();
    assert_eq!(
        validated.context().derived_owner(),
        Some(&DirectEvalDerivedOwner::ExternalCaller)
    );
    assert!(validated.context().derived_constructor_owner().is_none());
    assert!(matches!(
        validated.context().script_import_origin(),
        Some(ScriptImportOrigin::ExternalCaller(
            RuntimeDirectEvalImportOrigin::CallerScriptRecord(27)
        ))
    ));
    assert_eq!(
        validated.context().runtime_descriptor(),
        context.runtime_descriptor()
    );
}

#[test]
fn descriptor_rejects_bad_enum_words_truncation_and_noncanonical_trailers() {
    let bytes = context(Strictness::Strict, EvalInvocationContext::Function)
        .runtime_descriptor()
        .encode();
    for length in 0..bytes.len() {
        assert!(RuntimeDirectEvalCallerDescriptor::decode(&bytes[..length]).is_err());
    }
    for offset in [8, 9, 10, 11] {
        let mut malformed = bytes.clone();
        malformed[offset] = 255;
        assert!(RuntimeDirectEvalCallerDescriptor::decode(&malformed).is_err());
    }
    let mut trailing = bytes.clone();
    trailing.push(0);
    assert!(RuntimeDirectEvalCallerDescriptor::decode(&trailing).is_err());
    let mut count = bytes;
    count[12..20].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(RuntimeDirectEvalCallerDescriptor::decode(&count).is_err());
    let names = DirectEvalContextIr::new(
        Strictness::Strict,
        EvalInvocationContext::Method,
        BTreeMap::from([
            ("a".into(), PrivateNameId::new(1, 0)),
            ("b".into(), PrivateNameId::new(1, 1)),
        ]),
        None,
    )
    .runtime_descriptor()
    .encode();
    let mut duplicate = names.clone();
    duplicate.copy_within(20..37, 37);
    assert!(RuntimeDirectEvalCallerDescriptor::decode(&duplicate).is_err());
    let reordered = [&names[..20], &names[37..54], &names[20..37]].concat();
    assert!(RuntimeDirectEvalCallerDescriptor::decode(&reordered).is_err());
    let mut length = names;
    length[20..28].copy_from_slice(&u64::MAX.to_le_bytes());
    assert!(RuntimeDirectEvalCallerDescriptor::decode(&length).is_err());
}

#[test]
fn every_hidden_runtime_scope_contributes_to_new_private_identity_floor() {
    let context = DirectEvalContextIr::new(
        Strictness::Strict,
        EvalInvocationContext::Method,
        BTreeMap::from([("secret".into(), PrivateNameId::new(7, 2))]),
        None,
    );
    let caller = RuntimeDirectEvalCaller::validate(
        context.runtime_descriptor(),
        [
            RuntimeDirectEvalPrivateScope::new(7, 3),
            RuntimeDirectEvalPrivateScope::new(90, 8),
        ],
    )
    .unwrap();
    assert_eq!(caller.private_environment_floor(), 91);
    let compiled = compile(
        &caller,
        "class Local { #local; method() { return this.#local; } } this.#secret;",
    )
    .unwrap();
    let ids: Vec<_> = compiled
        .program()
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .flat_map(|function| function.private_name_ids.values())
        .collect();
    assert!(!ids.is_empty());
    assert!(ids.iter().all(|id| id.class_scope() >= 91));
    assert_eq!(
        caller.context().private_names()["secret"],
        PrivateNameId::new(7, 2)
    );
    syntax(&caller, "this.#missing;");
}

#[test]
fn private_chain_validation_rejects_missing_ordinal_duplicate_and_scope_overflow() {
    let descriptor = DirectEvalContextIr::new(
        Strictness::Strict,
        EvalInvocationContext::Method,
        BTreeMap::from([("secret".into(), PrivateNameId::new(7, u32::MAX))]),
        None,
    )
    .runtime_descriptor();
    assert!(RuntimeDirectEvalCaller::validate(descriptor.clone(), []).is_err());
    assert!(RuntimeDirectEvalCaller::validate(
        descriptor,
        [RuntimeDirectEvalPrivateScope::new(7, u32::MAX)]
    )
    .is_err());
    let descriptor =
        context(Strictness::Sloppy, EvalInvocationContext::Function).runtime_descriptor();
    assert!(RuntimeDirectEvalCaller::validate(
        descriptor.clone(),
        [
            RuntimeDirectEvalPrivateScope::new(9, 0),
            RuntimeDirectEvalPrivateScope::new(9, 1)
        ]
    )
    .is_err());
    assert!(RuntimeDirectEvalCaller::validate(
        descriptor,
        [RuntimeDirectEvalPrivateScope::new(u32::MAX, 0)]
    )
    .is_err());
}

#[test]
fn nearly_exhausted_private_space_counts_real_classes_before_analysis() {
    let descriptor =
        context(Strictness::Sloppy, EvalInvocationContext::Function).runtime_descriptor();
    let caller = RuntimeDirectEvalCaller::validate(
        descriptor,
        [RuntimeDirectEvalPrivateScope::new(u32::MAX - 2, 0)],
    )
    .unwrap();
    assert!(compile(&caller, "class Empty {} class One { #x; }").is_ok());
    let error = compile(&caller, "class One { #x; } class Two { #y; }").unwrap_err();
    assert!(
        matches!(error, RuntimeDirectEvalCompileError::Lowering(diagnostics) if diagnostics.iter().any(|diagnostic| diagnostic.message.contains("private-scope identity space is exhausted")))
    );
}

#[test]
fn inherited_constructor_role_reaches_nested_eval_without_a_foreign_function_id() {
    let caller = caller(context(
        Strictness::Strict,
        EvalInvocationContext::DerivedConstructor,
    ));
    let compiled = compile(&caller, "eval('super()'); (() => new.target);").unwrap();
    let nested = compiled
        .program()
        .script
        .as_ref()
        .unwrap()
        .prepared_script_units()
        .find(|unit| {
            unit.id != compiled.script_id()
                && matches!(unit.kind, PreparedScriptKind::DirectEval(_))
        })
        .unwrap();
    let PreparedScriptKind::DirectEval(context) = &nested.kind else {
        unreachable!()
    };
    assert_eq!(
        context.derived_owner(),
        Some(&DirectEvalDerivedOwner::ExternalCaller)
    );
    assert!(context.derived_constructor_owner().is_none());
}

#[test]
fn external_import_origin_survives_as_a_real_operation_without_eager_rejection() {
    for (origin, expected) in [
        (None, RuntimeDirectEvalImportOrigin::Absent),
        (
            Some(ScriptImportOrigin::Discovery),
            RuntimeDirectEvalImportOrigin::CallerScriptDiscovery,
        ),
        (
            Some(ScriptImportOrigin::LoadedRecord(91)),
            RuntimeDirectEvalImportOrigin::CallerScriptRecord(91),
        ),
    ] {
        let caller = caller(
            context(Strictness::Sloppy, EvalInvocationContext::Function)
                .with_script_import_origin(origin),
        );
        let compiled = compile(&caller, "import('./dependency.js');").unwrap();
        assert_eq!(caller.descriptor().import_origin(), expected);
        assert!(
            matches!(root(&compiled).body.statements.last(), Some(StatementIr::Expression(value))
            if matches!(&value.expr, ExprIr::DynamicImport { referrer: crate::ImportCallSourceOriginIr::ExternalCaller(actual), specifier, .. }
                if *actual == expected && matches!(&specifier.expr, ExprIr::String(text) if text == "./dependency.js")))
        );
        assert!(compiled.program().diagnostics.is_empty());
    }
}

#[test]
fn unexecuted_nested_import_keeps_its_external_owner_and_actual_operands() {
    let caller = caller(
        context(Strictness::Sloppy, EvalInvocationContext::Function)
            .with_script_import_origin(Some(ScriptImportOrigin::LoadedRecord(73))),
    );
    let compiled = compile(
        &caller,
        "function unused(specifier, options) { return import(specifier, options); } 42;",
    )
    .unwrap();
    let function = compiled
        .program()
        .script
        .as_ref()
        .unwrap()
        .functions
        .iter()
        .find(|function| function.name == "unused")
        .unwrap();
    assert!(function.body.statements.iter().any(|statement| matches!(statement, StatementIr::Return(value)
        if matches!(&value.expr, ExprIr::DynamicImport {
            referrer: crate::ImportCallSourceOriginIr::ExternalCaller(RuntimeDirectEvalImportOrigin::CallerScriptRecord(73)),
            specifier, options: Some(options), ..
        } if matches!(specifier.expr, ExprIr::Identifier(_) | ExprIr::EnvironmentIdentifier(_))
            && matches!(options.expr, ExprIr::Identifier(_) | ExprIr::EnvironmentIdentifier(_))))));
    assert!(
        matches!(root(&compiled).body.statements.last(), Some(StatementIr::Expression(value))
        if matches!(value.expr, ExprIr::Number(bits) if bits == 42.0_f64.to_bits()))
    );
}

#[test]
fn exact_utf16_source_and_literals_survive_native_direct_eval_lowering() {
    let caller = caller(context(Strictness::Sloppy, EvalInvocationContext::Function));
    for literal in [vec![0xd800], vec![0xdfff], vec![0xdb80, 0xdc00]] {
        let source = [vec![0x27], literal.clone(), vec![0x27, 0x3b]].concat();
        let compiled =
            lower_runtime_direct_eval(&caller, &source, HostSurfacePolicy::default()).unwrap();
        let prepared = compiled.program().script.as_ref().unwrap().prepared_scripts.iter()
            .find(|row| matches!(&row.outcome, PreparedScriptOutcome::Executable(unit) if unit.id == compiled.script_id())).unwrap();
        assert_eq!(decode_js_string_utf16(&prepared.source), source);
        assert!(
            matches!(root(&compiled).body.statements.last(), Some(StatementIr::Expression(value)) if matches!(&value.expr, ExprIr::String(text) if decode_js_string_utf16(text) == literal))
        );
    }
    assert!(matches!(
        lower_runtime_direct_eval(
            &caller,
            &[0x6c, 0x65, 0x74, 0x20, 0xd800, 0x3b],
            HostSurfacePolicy::default()
        ),
        Err(RuntimeDirectEvalCompileError::Parse(_))
    ));
}

#[test]
fn empty_completion_proof_requires_empty_actual_syntax_and_instantiation() {
    empty::empty_completion_proof_requires_empty_actual_syntax_and_instantiation();
}

#[test]
fn empty_completion_proof_rejects_observable_or_corrupt_preparation_metadata() {
    empty::empty_completion_proof_rejects_observable_or_corrupt_preparation_metadata();
}
