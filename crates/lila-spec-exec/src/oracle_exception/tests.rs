use super::*;
use lila_runtime::{
    EmbeddedModuleEntryInput, EmbeddedModuleGraph, EmbeddedModuleReferrer,
    EmbeddedModuleResolutionInput, EmbeddedModuleSourceInput,
};

fn script_error(source: &str) -> ExecutionError {
    execute_script_with_module_loading_policy(
        source,
        None,
        ModuleLoadingPolicy::RejectAll,
        &[],
        false,
    )
    .expect_err("fixture must finish abruptly")
}

fn module_error(source: &str, child: Option<&str>) -> ExecutionError {
    let modules = child
        .map(|source| EmbeddedModuleSourceInput {
            identity: "child.js".into(),
            source: source.into(),
            meta_url: "lila://urls/child.js".into(),
        })
        .into_iter()
        .collect();
    let resolutions = child
        .map(|_| EmbeddedModuleResolutionInput {
            referrer: EmbeddedModuleReferrer::Module("entry.js".into()),
            specifier: "child".into(),
            attributes: vec![],
            target: "child.js".into(),
        })
        .into_iter()
        .collect();
    let graph = EmbeddedModuleGraph::try_new(
        EmbeddedModuleEntryInput {
            goal: EmbeddedModuleGoal::Module,
            identity: "entry.js".into(),
            source: source.into(),
            meta_url: "lila://urls/entry.js".into(),
        },
        modules,
        resolutions,
    )
    .unwrap();
    execute_module(
        source,
        Some("entry.js"),
        ModuleHostConfig {
            module_loading_policy: ModuleLoadingPolicy::Embedded(graph),
            ..ModuleHostConfig::default()
        },
        &[],
        false,
    )
    .expect_err("fixture must finish abruptly")
}

#[test]
fn actual_error_brands_survive_misleading_text_and_created_realms() {
    for (source, expected) in [
        (
            "throw new TypeError('RangeError');",
            OracleExceptionType::Type,
        ),
        ("null.property;", OracleExceptionType::Type),
        (
            "let e = new RangeError('TypeError'); e.name = 'TypeError'; throw e;",
            OracleExceptionType::Range,
        ),
        (
            "throw new ($262.createRealm().global.TypeError)('other Realm');",
            OracleExceptionType::Type,
        ),
        ("throw 'TypeError';", OracleExceptionType::Primitive),
        (
            "throw { constructor: TypeError };",
            OracleExceptionType::UnclassifiedObject,
        ),
        (
            "let e = new RangeError(); e.constructor = TypeError; throw e;",
            OracleExceptionType::UnclassifiedObject,
        ),
        (
            "throw new SuppressedError(1, 2);",
            OracleExceptionType::Suppressed,
        ),
        (
            "let e = new Error(); e.constructor = SuppressedError; throw e;",
            OracleExceptionType::UnclassifiedObject,
        ),
        (
            "let e = new SuppressedError(1, 2); e.constructor = Error; throw e;",
            OracleExceptionType::UnclassifiedObject,
        ),
    ] {
        let error = script_error(source);
        assert_eq!(
            error.javascript_exception_phase(),
            Some(OracleExceptionPhase::Runtime),
            "{source}"
        );
        assert_eq!(
            error.javascript_exception_type(),
            Some(expected),
            "{source}"
        );
        assert!(!error.is_entry_syntax_rejection());
    }
}

#[test]
fn exception_reporting_never_runs_accessors_or_proxy_traps() {
    for (source, expected) in [
        ("let e = new TypeError(); for (let key of ['name', 'message', 'cause']) Object.defineProperty(e, key, {get() { calls++; return 'RangeError'; }}); throw e;", OracleExceptionType::Type),
        ("let e = new TypeError(); Object.defineProperty(e, 'constructor', {get() { calls++; return TypeError; }}); throw e;", OracleExceptionType::UnclassifiedObject),
        ("throw new Proxy(new TypeError(), {get() { calls++; return TypeError; }, getPrototypeOf() { calls++; return TypeError.prototype; }});", OracleExceptionType::UnclassifiedObject),
    ] {
        let mut context = Context::default();
        context.eval(Source::from_bytes("globalThis.calls = 0;")).unwrap();
        let error = context.eval(Source::from_bytes(source)).unwrap_err();
        let error = thrown(error, OracleExceptionPhase::Runtime, &mut context);
        assert_eq!(error.javascript_exception_type(), Some(expected), "{source}");
        assert_eq!(context.eval(Source::from_bytes("calls;")).unwrap(), JsValue::from(0));
    }
}

#[test]
fn original_host_failures_keep_their_provenance_after_catch_and_rethrow() {
    for source in [
        "$262.agent.receiveBroadcast(function () {});",
        "try { $262.agent.receiveBroadcast(function () {}); } catch (error) { error.name = 'TypeError'; throw error; }",
        "print('Test262:AsyncTestFailure:TypeError');",
    ] {
        let error = script_error(source);
        assert_eq!(error.javascript_exception_phase(), None, "{source}");
        assert_eq!(error.javascript_exception_type(), None, "{source}");
        assert!(!error.is_entry_syntax_rejection());
    }
    // A fresh user exception after handling a host failure is still a throw.
    let error = script_error(
        "try { $262.agent.receiveBroadcast(function () {}); } catch (_) {} throw new TypeError();",
    );
    assert_eq!(
        error.javascript_exception_type(),
        Some(OracleExceptionType::Type)
    );
}

#[test]
fn host_diagnostics_survive_rethrow_without_observing_mutated_error_properties() {
    for source in [
        "print('Test262:AsyncTestFailure:original marker');",
        "try { print('Test262:AsyncTestFailure:original marker'); } catch (error) { for (let key of ['name', 'message', 'constructor']) Object.defineProperty(error, key, {get() { throw 'must not observe'; }}); throw error; }",
    ] {
        let error = script_error(source);
        assert!(error.message().contains("Test262:AsyncTestFailure:original marker"));
        assert_eq!(error.javascript_exception_type(), None);
    }
    let error = script_error("__lilaUnsupportedHostCapability('createRealm');");
    assert!(error
        .message()
        .contains("local harness host createRealm unsupported"));
    assert_eq!(error.javascript_exception_type(), None);
    let user_error =
        script_error("throw { message: 'local harness host createRealm unsupported' };");
    assert_eq!(
        user_error.javascript_exception_type(),
        Some(OracleExceptionType::UnclassifiedObject)
    );
    assert!(!user_error.message().contains("createRealm"));
}

#[test]
fn module_loading_linking_and_evaluation_have_distinct_provenance() {
    let linked = module_error(
        "import { missing } from 'child';",
        Some("export const present = 1;"),
    );
    assert_eq!(
        linked.javascript_exception_phase(),
        Some(OracleExceptionPhase::Resolution)
    );
    assert_eq!(
        linked.javascript_exception_type(),
        Some(OracleExceptionType::Syntax)
    );

    let evaluated = module_error("throw new SyntaxError('evaluation');", None);
    assert_eq!(
        evaluated.javascript_exception_phase(),
        Some(OracleExceptionPhase::Runtime)
    );
    assert_eq!(
        evaluated.javascript_exception_type(),
        Some(OracleExceptionType::Syntax)
    );

    for source in [
        "import 'missing';",
        "await import('missing');",
        "try { await import('missing'); } catch (error) { throw error; }",
    ] {
        let loading = module_error(source, None);
        assert_eq!(loading.javascript_exception_phase(), None, "{source}");
        assert!(!loading.is_entry_syntax_rejection());
    }
    let malformed_child = module_error("import 'child';", Some("const value = ;"));
    assert_eq!(malformed_child.javascript_exception_phase(), None);
    assert!(!malformed_child.is_entry_syntax_rejection());
}

#[test]
fn runtime_limits_cannot_mint_javascript_exception_proof() {
    let mut context = Context::default();
    let opaque = JsNativeError::runtime_limit().to_opaque(&mut context);
    let opaque_error = thrown(
        JsError::from_opaque(opaque.into()),
        OracleExceptionPhase::Runtime,
        &mut context,
    );
    assert_eq!(opaque_error.javascript_exception_phase(), None);
    let error = thrown(
        JsNativeError::runtime_limit()
            .with_message("TypeError")
            .into(),
        OracleExceptionPhase::Runtime,
        &mut context,
    );
    assert_eq!(error.javascript_exception_phase(), None);
}
