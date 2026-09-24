//! `import()` written in a Script, compiled ahead of time.
//!
//! The Script stays Script code, its targets are compiled into the same
//! artifact and evaluate only from their import jobs, and a computed specifier
//! is served from the host's declared spellings.

use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, ComputedImportSpecifiers, Engine, ExecutionBackend, HostOutputEvent,
    HostSurfacePolicy, ObservedCompletion, RealmBuilder, RunOptions,
};

struct Files(PathBuf);

impl Drop for Files {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

impl Files {
    /// The first file is the Script entry, written to disk as well so that a
    /// Script can import its own file.
    fn new(files: &[(&str, &str)]) -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let fixture = Self(std::env::temp_dir().join(format!(
            "lila-script-dynamic-import-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        )));
        std::fs::create_dir_all(&fixture.0).expect("create fixture");
        for (name, source) in files {
            std::fs::write(fixture.0.join(name), source).expect("write fixture");
        }
        fixture
    }

    fn options(&self, computed: ComputedImportSpecifiers) -> CompileOptions {
        CompileOptions {
            filename: Some(self.0.join("entry.js").to_str().unwrap().into()),
            module_root: Some(self.0.to_str().unwrap().into()),
            host_surface_policy: HostSurfacePolicy::Test262,
            computed_import_specifiers: computed,
            ..CompileOptions::default()
        }
    }
}

fn run_script(
    files: &[(&str, &str)],
    computed: ComputedImportSpecifiers,
) -> Result<lila_engine::ObservedRunOutcome, lila_engine::EngineError> {
    let fixture = Files::new(files);
    lila_engine::configure_compilation_jobs(1).expect("bounded compiler");
    Engine::new(RealmBuilder::new().build()).observe_script(
        files[0].1,
        fixture.options(computed),
        RunOptions {
            backend: ExecutionBackend::WasmAot,
            timeout_ms: Some(30_000),
            ..RunOptions::default()
        },
    )
}

fn assert_script(files: &[(&str, &str)], computed: ComputedImportSpecifiers, expected: &[&str]) {
    let observed = run_script(files, computed).expect("AOT Script graph executes");
    assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
    assert!(
        matches!(observed.completion, ObservedCompletion::Normal(_)),
        "{:?}",
        observed.completion
    );
    assert_eq!(
        observed.output_events,
        expected
            .iter()
            .map(|line| HostOutputEvent::PrintLine((*line).into()))
            .collect::<Vec<_>>()
    );
}

fn closed(spellings: &[&str]) -> ComputedImportSpecifiers {
    ComputedImportSpecifiers::Closed(
        spellings
            .iter()
            .map(|spelling| (*spelling).into())
            .collect(),
    )
}

#[test]
fn a_script_import_evaluates_its_target_once_from_its_job() {
    assert_script(
        &[
            (
                "entry.js",
                r#"
globalThis.log = [];
log.push('script');
const first = import('./value.js');
const second = import('./value.js');
log.push('returned');
if (first === second) throw 'each call owns a promise';
Promise.all([first, second]).then(([a, b]) => {
  print(log.join(','));
  print(String(a === b && a.value === 7));
});
"#,
            ),
            (
                "value.js",
                "globalThis.log.push('body'); export const value = 7;",
            ),
        ],
        ComputedImportSpecifiers::default(),
        &["script,returned,body", "true"],
    );
}

#[test]
fn the_script_keeps_script_semantics_and_its_modules_keep_module_semantics() {
    assert_script(
        &[
            (
                "entry.js",
                r#"
var fromScript = 1;
undeclaredSloppy = 2;
print(String(this === globalThis));
print(String(Object.prototype.hasOwnProperty.call(globalThis, 'fromScript')));
import('./module.js').then(ns => print([ns.direct, ns.arrow, ns.strict].join(',')));
"#,
            ),
            (
                "module.js",
                "export const direct = String(this); export const arrow = String((() => this)()); export const strict = String((function () { return this; })() === undefined);",
            ),
        ],
        ComputedImportSpecifiers::default(),
        &["true", "true", "undefined,undefined,true"],
    );
}

#[test]
fn a_strict_script_stays_strict() {
    assert_script(
        &[
            (
                "entry.js",
                r#""use strict";
let threw = false;
try { undeclaredStrict = 1; } catch (error) { threw = error instanceof ReferenceError; }
print(String(threw));
import('./value.js').then(ns => print(String(ns.value)));
"#,
            ),
            ("value.js", "export const value = 3;"),
        ],
        ComputedImportSpecifiers::default(),
        &["true", "3"],
    );
}

#[test]
fn operands_are_evaluated_at_the_call_and_coercion_rejects() {
    assert_script(
        &[(
            "entry.js",
            r#"
const obj = { get err() { throw new RangeError('get'); } };
try { import(obj.err); print('no throw'); } catch (error) { print(error.constructor.name); }
const rejected = import({ toString() { throw new EvalError('toString'); } });
print(String(rejected instanceof Promise));
rejected.then(() => print('fulfilled'), error => print(error.constructor.name));
"#,
        )],
        closed(&[]),
        &["RangeError", "true", "EvalError"],
    );
}

#[test]
fn a_computed_specifier_is_served_from_the_declared_spellings() {
    assert_script(
        &[
            (
                "entry.js",
                r#"
const parts = ['./val', 'ue.js'];
Promise.allSettled([
  import(parts.join('')),
  import({ toString() { return './value.js'; } }),
  import('./missing' + '.js'),
]).then(([joined, object, missing]) => {
  print(String(joined.value === object.value && joined.value.value === 7));
  print(missing.status + ':' + missing.reason.constructor.name);
});
"#,
            ),
            ("value.js", "export const value = 7;"),
        ],
        closed(&["./value.js", "./missing.js"]),
        &["true", "rejected:TypeError"],
    );
}

#[test]
fn an_undeclared_computed_specifier_is_an_explicit_unsupported_compile() {
    let error = run_script(
        &[("entry.js", "const name = './value.js'; import(name);")],
        ComputedImportSpecifiers::Undeclared,
    )
    .expect_err("an undeclared computed specifier cannot be served ahead of time");
    assert!(
        error
            .to_string()
            .contains("the host declared no computed-import specifiers"),
        "{error}"
    );
}

#[test]
fn a_script_importing_its_own_file_gets_one_separate_module() {
    let entry = r#"
globalThis.runs = (globalThis.runs || 0) + 1;
if (globalThis.runs === 1) {
  Promise.all([import('./entry.js'), import('./entry.js')]).then(([a, b]) => {
    print(String(a === b));
    print(String(globalThis.runs));
  });
}
"#;
    assert_script(
        &[("entry.js", entry)],
        ComputedImportSpecifiers::default(),
        &["true", "2"],
    );
}

#[test]
fn failing_targets_reject_only_their_own_import() {
    assert_script(
        &[
            (
                "entry.js",
                r#"
print('script ran');
Promise.allSettled([
  import('./throws.js'),
  import('./throws.js'),
  import('./invalid.js'),
  import('./value.js'),
]).then(([first, second, invalid, value]) => {
  print(String(first.reason === second.reason && first.reason.message === 'boom'));
  print(invalid.reason.constructor.name);
  print(String(value.value.value));
});
"#,
            ),
            ("throws.js", "throw new Error('boom');"),
            ("invalid.js", "invalid syntax!"),
            ("value.js", "export const value = 5;"),
        ],
        ComputedImportSpecifiers::default(),
        &["script ran", "true", "SyntaxError", "5"],
    );
}

/// Loading a target's whole graph finishes before it is linked, so a request
/// the host cannot load anywhere in the graph rejects with the host's load
/// error even when another module in it would fail to link. A graph that
/// loads completely but fails to link rejects with the linker's SyntaxError.
/// Neither target is emitted, so a static source-phase request inside one does
/// not constrain the rest of the program.
#[test]
fn load_failures_in_a_target_graph_win_over_its_link_errors() {
    assert_script(
        &[
            (
                "entry.js",
                r#"
Promise.allSettled([
  import('./unloadable.js'),
  import('./unlinkable.js'),
]).then(([unloadable, unlinkable]) => {
  print(unloadable.reason.constructor.name);
  print(unlinkable.reason.constructor.name);
});
"#,
            ),
            (
                "unloadable.js",
                "import './unlinkable.js';\nimport source missing from './missing.js';",
            ),
            ("unlinkable.js", "import { absent } from './unlinkable.js';"),
        ],
        ComputedImportSpecifiers::default(),
        &["TypeError", "SyntaxError"],
    );
}

#[test]
fn a_source_phase_import_of_a_source_text_module_rejects_with_a_syntax_error() {
    assert_script(
        &[
            (
                "entry.js",
                r#"
const coercion = {};
Promise.allSettled([
  import.source('./value.js'),
  import.source({ toString() { return './value.js'; } }),
  import.source({ toString() { throw coercion; } }),
  import.source('./missing.js'),
]).then(([literal, computed, thrown, missing]) => {
  print(literal.reason.constructor.name + ',' + computed.reason.constructor.name);
  print(String(thrown.reason === coercion));
  print(missing.reason.constructor.name);
  print(String(globalThis.evaluated));
});
"#,
            ),
            (
                "value.js",
                "globalThis.evaluated = true; export const value = 1;",
            ),
        ],
        closed(&["./value.js"]),
        &["SyntaxError,SyntaxError", "true", "TypeError", "undefined"],
    );
}

#[test]
fn a_target_with_top_level_await_resolves_after_its_evaluation() {
    assert_script(
        &[
            (
                "entry.js",
                "import('./tla.js').then(ns => print(String(ns.value)));",
            ),
            ("tla.js", "export const value = await Promise.resolve(11);"),
        ],
        ComputedImportSpecifiers::default(),
        &["11"],
    );
}
