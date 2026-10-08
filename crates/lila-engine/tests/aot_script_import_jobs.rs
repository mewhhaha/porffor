use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

struct Modules(PathBuf);

impl Drop for Modules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

fn assert_script(
    files: &[(&str, &str)],
    source: &str,
    expected: &[&str],
    completion: ObservedJsValue,
) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = Modules(std::env::temp_dir().join(format!(
        "lila-script-import-jobs-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&fixture.0).expect("create Script module fixture");
    for (name, text) in files {
        std::fs::write(fixture.0.join(name), text).expect("write Script module fixture");
    }
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let observed = Engine::new(RealmBuilder::new().build())
        .observe_script(
            source,
            CompileOptions {
                filename: Some(fixture.0.join(files[0].0).to_str().unwrap().into()),
                module_root: Some(fixture.0.to_str().unwrap().into()),
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                timeout_ms: Some(30_000),
                ..RunOptions::default()
            },
        )
        .expect("Script dynamic-import graph compiles and executes through Wasm AOT");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert_eq!(
        observed.completion,
        ObservedCompletion::Normal(completion),
        "{}; output events: {:?}",
        observed.note,
        observed.output_events
    );
    assert_eq!(
        observed.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>()
    );
}

fn success(files: &[(&str, &str)], expected: &[&str]) {
    assert_script(files, files[0].1, expected, ObservedJsValue::Boolean(true));
}

#[test]
fn pinned_update_to_dynamic_import_keeps_nested_evaluation_lazy() {
    const SOURCE: &str = include_str!(
        "../../../test262/vendor/test262/test/language/expressions/dynamic-import/update-to-dynamic-import.js"
    );
    const FIRST: &str = include_str!(
        "../../../test262/vendor/test262/test/language/expressions/dynamic-import/update-to-dynamic-import_FIXTURE.js"
    );
    const OTHER: &str = include_str!(
        "../../../test262/vendor/test262/test/language/expressions/dynamic-import/update-to-dynamic-import-other_FIXTURE.js"
    );
    const STA: &str = include_str!("../../../test262/vendor/test262/harness/sta.js");
    const ASSERT: &str = include_str!("../../../test262/vendor/test262/harness/assert.js");
    const DONE: &str = include_str!("../../../test262/vendor/test262/harness/doneprintHandle.js");
    const ASYNC: &str = include_str!("../../../test262/vendor/test262/harness/asyncHelpers.js");
    let files = [
        ("update-to-dynamic-import.js", SOURCE),
        ("update-to-dynamic-import_FIXTURE.js", FIRST),
        ("update-to-dynamic-import-other_FIXTURE.js", OTHER),
    ];
    for directive in ["", "\"use strict\";\n"] {
        let source = format!("{directive}{STA}\n{ASSERT}\n{DONE}\n{ASYNC}\n{SOURCE}");
        assert_script(
            &files,
            &source,
            &["Test262:AsyncTestComplete"],
            ObservedJsValue::Undefined,
        );
    }
}

#[test]
fn uncalled_import_branches_and_functions_never_evaluate_their_targets() {
    success(
        &[
            (
                "entry.js",
                include_str!("fixtures/script_import_jobs/unreached/entry.js"),
            ),
            (
                "never.js",
                include_str!("fixtures/script_import_jobs/unreached/never.js"),
            ),
            ("invalid.js", "invalid syntax!"),
        ],
        &["unreached targets stayed idle"],
    );
}

#[test]
fn sloppy_and_strict_script_globals_and_completion_survive_module_jobs() {
    const ENTRY: &str = include_str!("fixtures/script_import_jobs/root_goal/entry.js");
    let files = [
        ("entry.js", ENTRY),
        (
            "target.js",
            include_str!("fixtures/script_import_jobs/root_goal/target.js"),
        ),
    ];
    for directive in ["", "\"use strict\";\n"] {
        let source = format!("{directive}{ENTRY}");
        assert_script(
            &files,
            &source,
            &["Script globals ready", "module lexical this ready"],
            ObservedJsValue::Number(ObservedNumber::from_f64(73.0)),
        );
    }
}

#[test]
fn importing_the_script_filename_uses_a_distinct_cached_module_owner() {
    const ENTRY: &str = include_str!("fixtures/script_import_jobs/self_import/entry.js");
    for directive in ["", "\"use strict\";\n"] {
        let source = format!("{directive}{ENTRY}");
        success(
            &[("entry.js", source.as_str())],
            &[
                "Script self import queued",
                "Module self import body",
                "distinct source goals",
            ],
        );
    }
}

#[test]
fn nested_dynamic_requests_evaluate_static_dependencies_before_updating_live_cells() {
    success(
        &[
            (
                "entry.js",
                include_str!("fixtures/script_import_jobs/live_cells/entry.js"),
            ),
            (
                "first.js",
                include_str!("fixtures/script_import_jobs/live_cells/first.js"),
            ),
            (
                "other.js",
                include_str!("fixtures/script_import_jobs/live_cells/other.js"),
            ),
            (
                "dependency.js",
                include_str!("fixtures/script_import_jobs/live_cells/dependency.js"),
            ),
        ],
        &["Script body", "first:first", "other:42:live"],
    );
}

#[test]
fn repeated_imports_keep_fresh_promises_one_evaluation_and_one_namespace() {
    success(
        &[
            (
                "entry.js",
                include_str!("fixtures/script_import_jobs/repeated/entry.js"),
            ),
            (
                "target.js",
                include_str!("fixtures/script_import_jobs/repeated/target.js"),
            ),
        ],
        &["imports queued", "one module namespace"],
    );
}

#[test]
fn ordinary_script_import_settles_after_evaluation_and_its_second_reaction() {
    const ENTRY: &str = include_str!("fixtures/script_import_jobs/reactions/entry.js");
    for target in [
        "events.push('body'); export const value = 7;",
        "events.push('body'); throw 7;",
    ] {
        success(
            &[("entry.js", ENTRY), ("target.js", target)],
            &["second reaction"],
        );
    }
}

#[test]
fn cyclic_tla_dependencies_complete_before_script_import_fulfills() {
    success(
        &[
            (
                "entry.js",
                include_str!("fixtures/script_import_jobs/cyclic_tla/entry.js"),
            ),
            (
                "a.js",
                include_str!("fixtures/script_import_jobs/cyclic_tla/a.js"),
            ),
            (
                "b.js",
                include_str!("fixtures/script_import_jobs/cyclic_tla/b.js"),
            ),
        ],
        &["cyclic TLA graph completed"],
    );
}

#[test]
fn cached_object_and_undefined_rejections_keep_identity_and_evaluate_once() {
    success(
        &[
            (
                "entry.js",
                include_str!("fixtures/script_import_jobs/rejections/entry.js"),
            ),
            (
                "object.js",
                include_str!("fixtures/script_import_jobs/rejections/object.js"),
            ),
            (
                "undefined.js",
                include_str!("fixtures/script_import_jobs/rejections/undefined.js"),
            ),
        ],
        &["rejections queued", "cached arbitrary rejections"],
    );
}

#[test]
fn direct_syntax_and_transitive_link_errors_reject_at_their_owned_job_stages() {
    success(
        &[
            (
                "entry.js",
                include_str!("fixtures/script_import_jobs/error_stages/entry.js"),
            ),
            ("invalid.js", "invalid syntax!"),
            (
                "transitive.js",
                "import './invalid.js'; globalThis.invalidBody = true;",
            ),
            (
                "link.js",
                "import { absent } from './shared.js'; globalThis.invalidBody = true;",
            ),
            (
                "missing.js",
                "import defer * as absent from './absent.js'; globalThis.invalidBody = true;",
            ),
            ("valid.js", "export { value } from './shared.js';"),
            (
                "shared.js",
                "globalThis.sharedCount++; export const value = 7;",
            ),
        ],
        &["load and dependency rejection stages"],
    );
}

#[test]
fn script_import_uses_its_realm_intrinsics_after_promise_and_reflection_replacement() {
    success(
        &[
            (
                "entry.js",
                include_str!("fixtures/script_import_jobs/intrinsics/entry.js"),
            ),
            (
                "target.js",
                include_str!("fixtures/script_import_jobs/intrinsics/target.js"),
            ),
            ("invalid.js", "invalid syntax!"),
        ],
        &["Script import realm intrinsics"],
    );
}

#[test]
fn with_environment_resolves_specifiers_without_observing_private_dispatchers() {
    success(
        &[
            (
                "entry.js",
                include_str!("fixtures/script_import_jobs/with_lookup/entry.js"),
            ),
            (
                "value.js",
                include_str!("fixtures/script_import_jobs/with_lookup/value.js"),
            ),
        ],
        &["with lookup preserves private dispatcher"],
    );
}

#[test]
fn script_and_module_callables_preserve_original_import_syntax_in_to_string() {
    const ENTRY: &str = include_str!("fixtures/script_import_jobs/callable_source/entry.js");
    const TARGET: &str = include_str!("fixtures/script_import_jobs/callable_source/target.js");
    for directive in ["", "\"use strict\";\n"] {
        // Real CRLF and Unicode line terminators exercise parser UTF-16 spans
        // independently of the byte-preserving export edits.
        let source = format!("{directive}/* 🟣 */\r\n// café\u{2028}{ENTRY}");
        assert_script(
            &[
                ("entry.js", ENTRY),
                ("target.js", TARGET),
                (
                    "other.js",
                    include_str!("fixtures/script_import_jobs/callable_source/other.js"),
                ),
            ],
            &source,
            &["original callable sources"],
            ObservedJsValue::Boolean(true),
        );
    }
}

#[test]
fn script_source_jobs_reject_without_changing_global_scope_or_completion() {
    success(
        &[
            (
                "entry.js",
                r#"
var scriptMarker = 41;
const lexicalMarker = 7;
const request = import.source('./target.js');
if (!(request instanceof Promise) || this !== globalThis) throw 'Script import context';
request.then(() => { throw 'source fulfilled'; }, error => {
  if (!(error instanceof SyntaxError) || globalThis.scriptMarker !== 41 || lexicalMarker !== 7) throw 'Script source rejection';
  print('Script source target stayed idle');
});
true;
"#,
            ),
            (
                "target.js",
                "import './absent-child.js'; throw 'source target evaluated';",
            ),
        ],
        &["Script source target stayed idle"],
    );
}
