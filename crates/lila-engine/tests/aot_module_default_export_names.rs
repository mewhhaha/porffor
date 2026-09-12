use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, ObservedCompletion, RealmBuilder,
    RunOptions,
};

struct Modules(PathBuf);

impl Drop for Modules {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

enum EntryGoal {
    Module,
    Script,
}

fn assert_modules(files: &[(&str, &str)], goal: EntryGoal, expected: &[&str]) {
    static NEXT: AtomicU64 = AtomicU64::new(0);
    let fixture = Modules(std::env::temp_dir().join(format!(
        "lila-default-export-names-{}-{}",
        std::process::id(),
        NEXT.fetch_add(1, Ordering::Relaxed)
    )));
    std::fs::create_dir_all(&fixture.0).unwrap();
    for (name, source) in files {
        std::fs::write(fixture.0.join(name), source).unwrap();
    }
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let engine = Engine::new(RealmBuilder::new().build());
    let options = CompileOptions {
        filename: Some(fixture.0.join(files[0].0).to_str().unwrap().into()),
        module_root: Some(fixture.0.to_str().unwrap().into()),
        ..CompileOptions::default()
    };
    let run = RunOptions {
        backend: ExecutionBackend::WasmAot,
        timeout_ms: Some(30_000),
        ..RunOptions::default()
    };
    let observed = match goal {
        EntryGoal::Module => engine.observe_module(files[0].1, options, run),
        EntryGoal::Script => engine.observe_script(files[0].1, options, run),
    }
    .expect("default export graph compiles and executes through Wasm");
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

#[test]
fn default_class_name_exists_before_static_fields_and_blocks() {
    assert_modules(
        &[
            ("entry.js", "import first from './first.js'; import second from './second.js'; print(first.name + ':' + second.name);"),
            ("first.js", "export default class { static field = print('field:' + this.name); static { print('block:' + this.name); let d = Object.getOwnPropertyDescriptor(this, 'name'); print(d.value + ':' + d.writable + ':' + d.enumerable + ':' + d.configurable); } }"),
            ("second.js", "export default (class { constructor() {} static { print('explicit:' + this.name); } });"),
        ],
        EntryGoal::Module,
        &["field:default", "block:default", "default:false:false:true", "explicit:default", "default:default"],
    );
}

#[test]
fn anonymous_function_protocols_and_parenthesized_expressions_receive_default() {
    assert_modules(
        &[
            ("entry.js", "import a from './a.js'; import b from './b.js'; import c from './c.js'; import d from './d.js'; import e from './e.js'; import f from './f.js'; import g from './g.js'; print(a.name + ':' + b.name + ':' + c.name + ':' + d.name + ':' + e.name + ':' + f.name + ':' + g.name); print(a() + ':' + b().next().value + ':' + e() + ':' + g()); Promise.all([c(), d().next(), f()]).then(v => print(v[0] + ':' + v[1].value + ':' + v[2]));"),
            ("a.js", "export default function () { return 1; }"),
            ("b.js", "export default function* () { yield 2; }"),
            ("c.js", "export default async function () { return 3; }"),
            ("d.js", "export default async function* () { yield 4; }"),
            ("e.js", "export default (() => 5);"),
            ("f.js", "export default (async () => 6);"),
            ("g.js", "export default (function () { return 7; });"),
        ],
        EntryGoal::Module,
        &["default:default:default:default:default:default:default", "1:2:5:7", "3:4:6"],
    );
}

#[test]
fn explicit_names_non_definitions_and_static_name_overrides_are_preserved() {
    assert_modules(
        &[
            ("entry.js", "import a from './a.js'; import b from './b.js'; import c from './c.js'; import d from './d.js'; import e from './e.js'; import f from './f.js'; import g from './g.js'; print(a.name + ':' + b.name + ':' + c.name + ':' + d.name + ':' + e.name()); print(f.name); print('constructed:[' + g.name + ']:' + new g().value); let bare = [class { static { print('array:[' + this.name + ']'); } }][0]; print('array descriptor:[' + Object.getOwnPropertyDescriptor(bare, 'name').value + ']');"),
            ("a.js", "export default class Named { static { print(this.name); } }"),
            ("b.js", "class Exported {} export { Exported as default };"),
            ("c.js", "export default (function Explicit() {});"),
            ("d.js", "export default (0, class { static { print('anonymous:[' + this.name + ']'); } });"),
            ("e.js", "export default class { static name() { return 'override'; } static { print(this.name()); } }"),
            ("f.js", "export default (function $d6$() {});"),
            ("g.js", "export default (0, class { constructor() { this.value = 5; } static { print('constructor:[' + this.name + ']'); } });"),
        ],
        EntryGoal::Module,
        &["Named", "anonymous:[]", "override", "constructor:[]", "Named:Exported:Explicit::override", "$d6$", "constructed:[]:5", "array:[]", "array descriptor:[]"],
    );
}

#[test]
fn utf16_offsets_and_nested_same_spelled_bindings_keep_distinct_names() {
    assert_modules(
        &[
            ("entry.js", "// 🦀é\r\nimport value from './value.js'; print(value.name);"),
            ("value.js", "const prefix = '🦀é';\r\nvoid import.meta;\r\nexport default class { static { let $d1$ = class { static { print('nested:' + this.name); } }; print(prefix + ':' + this.name + ':' + $d1$.name); } };"),
        ],
        EntryGoal::Module,
        &["nested:$d1$", "🦀é:default:$d1$", "default"],
    );
}

#[test]
fn asynchronous_module_wrapper_preserves_default_definition_identity() {
    assert_modules(
        &[
            (
                "entry.js",
                "import value from './value.js'; print(value.name);",
            ),
            (
                "value.js",
                "await 0; export default class { static { print('await:' + this.name); } }",
            ),
        ],
        EntryGoal::Module,
        &["await:default", "default"],
    );
}

#[test]
fn deferred_module_thunk_names_the_class_before_first_access_finishes() {
    assert_modules(
        &[
            ("entry.js", "import defer * as ns from './value.js'; print('before'); print(ns.default.name); print(ns.default.name);"),
            ("value.js", "export default class { static { print('deferred:' + this.name); } }"),
        ],
        EntryGoal::Module,
        &["before", "deferred:default", "default", "default"],
    );
}

#[test]
fn script_import_wrapper_and_length_changing_rewrites_preserve_names() {
    assert_modules(
        &[
            ("entry.js", "import('./value.js').then(ns => print('import:' + ns.default.name));"),
            ("value.js", "void import('./other.js'); export default class { static { print('script:' + this.name); } }"),
            ("other.js", "export const other = 1;"),
        ],
        EntryGoal::Script,
        &["script:default", "import:default"],
    );
}
