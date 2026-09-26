use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};

use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, RealmBuilder, RunOptions,
};

struct Fixture(PathBuf);

#[derive(Clone, Copy)]
enum EntryGoal {
    Module,
    Script,
}

impl Fixture {
    fn new() -> Self {
        static NEXT: AtomicU64 = AtomicU64::new(0);
        let path = std::env::temp_dir().join(format!(
            "lila-aot-import-bytes-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir_all(&path).expect("create bytes module fixture");
        Self(path)
    }

    fn write(&self, name: &str, contents: impl AsRef<[u8]>) {
        std::fs::write(self.0.join(name), contents).expect("write fixture");
    }

    fn run(&self, source: &str, expected: &[&str]) {
        self.run_goal(EntryGoal::Module, source, expected);
    }

    fn run_script(&self, source: &str, expected: &[&str]) {
        self.run_goal(EntryGoal::Script, source, expected);
    }

    fn run_goal(&self, goal: EntryGoal, source: &str, expected: &[&str]) {
        self.write("entry.js", source);
        lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
        let options = CompileOptions {
            filename: Some(self.0.join("entry.js").to_str().unwrap().into()),
            module_root: Some(self.0.to_str().unwrap().into()),
            host_surface_policy: HostSurfacePolicy::Test262,
            ..CompileOptions::default()
        };
        let run = RunOptions {
            backend: ExecutionBackend::WasmAot,
            timeout_ms: Some(30_000),
            ..RunOptions::default()
        };
        let engine = Engine::new(RealmBuilder::new().build());
        let observed = match goal {
            EntryGoal::Module => engine.observe_module(source, options, run),
            EntryGoal::Script => engine.observe_script(source, options, run),
        }
        .expect("bytes module graph compiles and executes through Wasm");
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
}

impl Drop for Fixture {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}

#[test]
fn bytes_modules_preserve_non_utf8_and_empty_files_in_a_source_phase_graph() {
    let fixture = Fixture::new();
    fixture.write("raw.bin", [0, 0x89, 0xff, 10]);
    fixture.write("deferred.bin", [4, 5]);
    fixture.write("empty.bin", b"");
    fixture.write("data.json", b"{\"answer\":42}");
    fixture.write("prefix.js", "export const marker = '😀';");
    fixture.write("script.js", b"export default 7;");
    fixture.write("source.js", b"throw 'source phase must not evaluate';");
    fixture.run(
        r#"
import source unused from './source.js';
import { marker } from './prefix.js';
import raw from './raw.bin' with { type: 'bytes' };
import defer * as later from './deferred.bin' with { type: 'bytes' };
import empty from './empty.bin' with { type: 'bytes' };
import json from './data.json' with { type: 'bytes' };
import script from './script.js' with { type: 'bytes' };
import number from './script.js';
print(raw instanceof Uint8Array && raw.length === 4 && raw[0] === 0 && raw[1] === 137 && raw[2] === 255 && raw[3] === 10);
print(empty.length === 0 && empty.buffer.immutable && empty.buffer.byteLength === 0);
print(json.length === 13 && json[0] === 123 && json[12] === 125 && json.buffer.immutable);
print(script[0] === 101 && script.buffer.immutable && number === 7);
print(later.default.length === 2 && later.default[0] === 4 && later.default[1] === 5 && later.default.buffer.immutable);
print(marker === '😀');
try { raw.buffer.transfer(); } catch (error) { print(error instanceof TypeError); }
void unused;
"#,
        &["true", "true", "true", "true", "true", "true", "true"],
    );
}

#[test]
fn dynamic_bytes_import_uses_intrinsics_after_mutable_hooks_change() {
    let fixture = Fixture::new();
    fixture.write("raw.bin", [0, 0x80, 0xff]);
    fixture.run(
        r#"
const OriginalUint8Array = Uint8Array;
const typedProto = Object.getPrototypeOf(Uint8Array.prototype);
const bufferGetter = Object.getOwnPropertyDescriptor(typedProto, 'buffer').get;
let hooks = 0;
globalThis.Uint8Array = function () { hooks++; throw 'global constructor'; };
Object.defineProperty(typedProto, 'buffer', { configurable: true, get() { hooks++; throw 'buffer getter'; } });
ArrayBuffer.prototype.transferToImmutable = function () { hooks++; throw 'transfer method'; };
Reflect.apply = function () { hooks++; throw 'Reflect.apply'; };
Array.prototype[Symbol.iterator] = function () { hooks++; throw 'array iterator'; };
import('./raw.bin', { with: { type: 'bytes' } }).then(first => {
  import('./raw.bin', { with: { type: 'bytes' } }).then(second => {
    const value = first.default;
    const buffer = bufferGetter.call(value);
    print(first === second && value === second.default && value instanceof OriginalUint8Array);
    print(value.length === 3 && value[0] === 0 && value[1] === 128 && value[2] === 255);
    print(buffer.immutable && buffer.byteLength === 3 && hooks === 0);
  });
});
"#,
        &["true", "true", "true"],
    );
}

#[test]
fn ordinary_javascript_copy_of_bytes_source_cannot_use_private_intrinsics() {
    let fixture = Fixture::new();
    let synthetic = lila_ir::ModuleSourceIr::bytes(
        lila_ir::ModuleKey::from_host("bytes:raw.bin"),
        vec![0xff],
        "file:///raw.bin".into(),
    );
    fixture.write("ordinary.js", synthetic.source_text());
    fixture.run(
        "import('./ordinary.js').then(() => print(false), error => print(error instanceof TypeError));",
        &["true"],
    );
}

#[test]
fn script_entry_dynamic_bytes_import_reuses_the_same_namespace_and_view() {
    let fixture = Fixture::new();
    fixture.write("raw.bin", [9, 0xff]);
    fixture.run_script(
        r#"
if (this !== globalThis) throw 'Script entry must retain its own goal';
import('./raw.bin', { with: { type: 'bytes' } }).then(first => {
  import('./raw.bin', { with: { type: 'bytes' } }).then(second => {
    print(first === second && first.default === second.default);
    print(first.default[0] === 9 && first.default[1] === 255 && first.default.buffer.immutable);
  });
});
"#,
        &["true", "true"],
    );
}

#[test]
fn top_level_await_entry_imports_bytes_before_resuming() {
    let fixture = Fixture::new();
    fixture.write("raw.bin", [0x80, 3]);
    fixture.run(
        r#"
import bytes from './raw.bin' with { type: 'bytes' };
await Promise.resolve();
print(bytes.length === 2 && bytes[0] === 128 && bytes[1] === 3 && bytes.buffer.immutable);
"#,
        &["true"],
    );
}

#[cfg(unix)]
#[test]
fn repointed_bytes_alias_uses_current_resolution_after_a_cached_run() {
    use std::os::unix::fs::symlink;

    let fixture = Fixture::new();
    fixture.write("a.bin", [1]);
    fixture.write("b.bin", [2]);
    let alias = fixture.0.join("alias.bin");
    symlink("a.bin", &alias).expect("link first bytes target");
    let source = r#"
import a from './a.bin' with { type: 'bytes' };
import b from './b.bin' with { type: 'bytes' };
import alias from './alias.bin' with { type: 'bytes' };
print(a[0] * 100 + b[0] * 10 + alias[0]);
"#;
    fixture.run(source, &["121"]);
    std::fs::remove_file(&alias).expect("remove first bytes alias");
    symlink("b.bin", &alias).expect("link second bytes target");
    fixture.run(source, &["122"]);
}
