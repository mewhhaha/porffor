use super::*;
use lila_ir::{RegExpProgram, RegExpProgramWord, ValidatedRegExpProgram};

fn append_uleb(bytes: &mut Vec<u8>, mut value: u32) {
    loop {
        let low = (value & 0x7f) as u8;
        value >>= 7;
        bytes.push(low | if value == 0 { 0 } else { 0x80 });
        if value == 0 {
            break;
        }
    }
}

fn read_uleb(bytes: &[u8], cursor: &mut usize) -> u32 {
    let mut value = 0;
    for shift in (0..35).step_by(7) {
        let byte = bytes[*cursor];
        *cursor += 1;
        value |= u32::from(byte & 0x7f) << shift;
        if byte & 0x80 == 0 {
            return value;
        }
    }
    panic!("invalid test artifact length");
}

/// Adds exports to a copied artifact only. The product has no compiler/debug
/// entry point, and the test discovers helper indices from encoded names.
fn with_compiler_exports(bytes: &[u8]) -> Vec<u8> {
    let mut compiler = None;
    let mut allocator = None;
    for payload in WasmParser::new(0).parse_all(bytes) {
        let WasmPayload::CustomSection(section) = payload.unwrap() else {
            continue;
        };
        let wasmparser::KnownCustom::Name(subsections) = section.as_known() else {
            continue;
        };
        for subsection in subsections {
            if let wasmparser::Name::Function(names) = subsection.unwrap() {
                for naming in names {
                    let naming = naming.unwrap();
                    match naming.name {
                        "helper::regexp_compiler" => compiler = Some(naming.index),
                        "helper::heap_alloc" => allocator = Some(naming.index),
                        _ => {}
                    }
                }
            }
        }
    }
    let exports = [
        ("test_pattern_compiler", compiler.unwrap()),
        ("test_pattern_allocate", allocator.unwrap()),
    ];
    let mut rewritten = bytes[..8].to_vec();
    let mut cursor = 8;
    let mut found = false;
    while cursor < bytes.len() {
        let id = bytes[cursor];
        cursor += 1;
        let length = read_uleb(bytes, &mut cursor) as usize;
        let end = cursor + length;
        if id == 7 {
            found = true;
            let count = read_uleb(bytes, &mut cursor);
            let mut body = Vec::new();
            append_uleb(&mut body, count + exports.len() as u32);
            body.extend_from_slice(&bytes[cursor..end]);
            for (name, index) in exports {
                append_uleb(&mut body, name.len() as u32);
                body.extend_from_slice(name.as_bytes());
                body.push(0);
                append_uleb(&mut body, index);
            }
            rewritten.push(id);
            append_uleb(&mut rewritten, body.len() as u32);
            rewritten.extend_from_slice(&body);
        } else {
            rewritten.push(id);
            append_uleb(&mut rewritten, length as u32);
            rewritten.extend_from_slice(&bytes[cursor..end]);
        }
        cursor = end;
    }
    assert!(found, "ordinary artifact must export main and memory");
    rewritten
}

struct CompilerStore {
    limits: WasmtimeStoreLimits,
}
struct RuntimeCompiler {
    store: WasmtimeStore<CompilerStore>,
    memory: WasmtimeMemory,
    allocate: wasmtime::TypedFunc<i64, i64>,
    compile: wasmtime::TypedFunc<(i64, i64, i64, i64, i64, i64, i64), (i64, i64, i64, i64)>,
}

impl RuntimeCompiler {
    fn new() -> Self {
        configure_compilation_jobs(1).unwrap();
        let engine = Engine::new(RealmBuilder::new().build());
        let unit = engine
            .compile_script("new RegExp('');", CompileOptions::default())
            .unwrap();
        let artifact = engine.emit_wasm(&unit).unwrap();
        let bytes = with_compiler_exports(&artifact.bytes);
        let wasm_engine = shared_wasm_engine().unwrap();
        let module = WasmtimeModule::new(&wasm_engine, bytes).unwrap();
        let mut store = WasmtimeStore::new(
            &wasm_engine,
            CompilerStore {
                limits: WasmtimeStoreLimitsBuilder::new()
                    .memory_size(64 * 1024 * 1024)
                    .build(),
            },
        );
        store.limiter(|state| &mut state.limits);
        store.set_epoch_deadline(u64::MAX / 2);
        let mut linker = WasmtimeLinker::new(&wasm_engine);
        for import in module.imports() {
            match import.ty() {
                WasmtimeExternType::Func(signature) => {
                    linker
                        .func_new(import.module(), import.name(), signature, |_, _, _| {
                            Err(wasmtime::Error::msg(
                                "pure Pattern compiler called a host import",
                            ))
                        })
                        .unwrap();
                }
                WasmtimeExternType::Memory(memory_type) => {
                    assert!(!memory_type.is_shared(), "fixture uses unshared memory");
                    let memory = WasmtimeMemory::new(&mut store, memory_type).unwrap();
                    linker
                        .define(&store, import.module(), import.name(), memory)
                        .unwrap();
                }
                other => panic!("unexpected fixture import: {other:?}"),
            }
        }
        let instance = linker.instantiate(&mut store, &module).unwrap();
        let memory = instance.get_memory(&mut store, "memory").unwrap();
        let allocate = instance
            .get_typed_func(&mut store, "test_pattern_allocate")
            .unwrap();
        let compile = instance
            .get_typed_func(&mut store, "test_pattern_compiler")
            .unwrap();
        Self {
            store,
            memory,
            allocate,
            compile,
        }
    }

    fn string(&mut self, bytes: &[u8]) -> i64 {
        let pointer = self
            .allocate
            .call(&mut self.store, bytes.len() as i64)
            .unwrap();
        self.memory
            .write(&mut self.store, pointer as usize, bytes)
            .unwrap();
        (pointer << 32) | bytes.len() as i64
    }

    fn heap_pointer(&mut self) -> i64 {
        self.allocate.call(&mut self.store, 0).unwrap()
    }

    fn call(&mut self, source: &str, flags: &str) -> (i64, i64, i64, i64, i64) {
        let source = self.string(source.as_bytes());
        let flags = self.string(flags.as_bytes());
        let before = self.heap_pointer();
        let (handle, status, offset, detail) = self
            .compile
            .call(&mut self.store, (source, flags, 0, 0, 0, 0, 0))
            .unwrap();
        (handle, status, offset, detail, before)
    }

    fn descriptor(&self, handle: i64) -> Vec<u8> {
        let pointer = (handle as u64 >> 32) as usize;
        let length = handle as u32 as usize;
        self.memory.data(&self.store)[pointer..pointer + length].to_vec()
    }
}

#[test]
fn emitted_pattern_compiler_roundtrips_owned_descriptors_without_candidate_lookup_or_host_calls() {
    run_on_sized_stack(|| {
        let mut runtime = RuntimeCompiler::new();
        let nested = format!("{}a{}", "(".repeat(200), ")".repeat(200));
        let nested_modifiers = format!("{}^a${}", "(?m-s:".repeat(200), ")".repeat(200));
        for (source, flags) in [
            ("", ""),
            ("abc", ""),
            (r"\c", ""),
            (r"\c0", ""),
            (r"\c_", ""),
            (r"\c+", ""),
            ("a|bc|", ""),
            ("(ab)+?", ""),
            ("(a?)*b", ""),
            ("(?:a?){1,3}?b", ""),
            ("(a(b)){0}", "d"),
            ("(?=(a+))a+", ""),
            ("a(?!bc)d", ""),
            ("(a)\\1", ""),
            ("\\1(a)", ""),
            ("(a|)\\1*", ""),
            ("(?s:^.$)", ""),
            ("(?-s:^.$)", "s"),
            ("(?m:^a$)", ""),
            ("(?-m:^a$)", "m"),
            ("(?ms:^.(?-ms:^.$).$)^.$", ""),
            ("(?s:(.)(?:.)(?=.)).", ""),
            ("(?i:(?-i:a))", ""),
            (r"(?-i:(a))(?i:\1)", "i"),
            ("(?im-s:.)|.", "s"),
            ("(?i-:.)", ""),
            (nested.as_str(), ""),
            (nested_modifiers.as_str(), "s"),
            ("😀+", ""),
            ("(?:😀)+", ""),
        ] {
            let (handle, status, _, _, before) = runtime.call(source, flags);
            assert_eq!(status, 0, "{source:?}/{flags}");
            assert_ne!(handle, 0);
            assert_eq!(
                (handle as u64 >> 32) as i64,
                before,
                "workspace must compact to entry checkpoint"
            );
            let bytes = runtime.descriptor(handle);
            let program = ValidatedRegExpProgram::from_bytes(bytes.clone()).unwrap();
            let expected = ValidatedRegExpProgram::from_program(
                &RegExpProgram::compile(source, flags).unwrap(),
            )
            .unwrap();
            assert_eq!(program.bytes(), expected.bytes(), "{source:?}/{flags}");
            assert_eq!(
                runtime.heap_pointer(),
                (before + bytes.len() as i64 + 7) & !7
            );
        }
        let (first, status, _, _, _) = runtime.call("(ab)+", "d");
        assert_eq!(status, 0);
        let retained = runtime.descriptor(first);
        for _ in 0..6 {
            assert_eq!(runtime.call("(x?)*y", "i").1, 0);
        }
        assert_eq!(
            runtime.descriptor(first),
            retained,
            "later compiler workspace must not borrow retained descriptor storage"
        );
    });
}

#[test]
fn emitted_pattern_compiler_distinguishes_syntax_capability_resources_and_rolls_back_each_failure()
{
    run_on_sized_stack(|| {
        let mut runtime = RuntimeCompiler::new();
        for (source, flags, expected) in [
            ("(", "", 1),
            ("[", "", 1),
            ("[z-a]", "", 1),
            ("a{3,2}", "", 1),
            (
                "a{999999999999999999999999,888888888888888888888888}",
                "",
                1,
            ),
            ("a", "ii", 1),
            ("a", "uv", 1),
            ("a", "u", 2),
            ("a", "v", 2),
            ("(?<name>a)", "", 2),
            ("(?<=a)b", "", 2),
            ("(?-:a)", "", 1),
            ("(?ii:a)", "", 1),
            ("(?m-m:a)", "", 1),
            ("(?i-ss:a)", "", 1),
            ("(?i--s:a)", "", 1),
            ("(?ig:a)", "", 1),
            ("(?d:a)", "", 1),
            ("(?i", "", 1),
            ("(?i:a", "", 1),
            ("a{5000}", "", 3),
        ] {
            let (handle, status, _, detail, before) = runtime.call(source, flags);
            assert_eq!(status, expected, "{source:?}/{flags}; detail={detail}");
            assert_eq!(handle, 0, "failure cannot publish a program");
            assert_eq!(runtime.heap_pointer(), before, "{source:?}/{flags}");
        }
        let (handle, status, _, _, _) = runtime.call("(?:){999999999999999999999999}", "");
        assert_eq!(
            status, 0,
            "state-free empty repetition must not expand or loop over the decimal count"
        );
        let descriptor = ValidatedRegExpProgram::from_bytes(runtime.descriptor(handle)).unwrap();
        assert_eq!(descriptor.word(RegExpProgramWord::InstructionCount), 1);
        let (erased, status, _, _, _) = runtime.call("(a{999999999999999999999999}){0}", "");
        assert_eq!(
            status, 0,
            "erased expansion still retains capture cardinality"
        );
        let erased = ValidatedRegExpProgram::from_bytes(runtime.descriptor(erased)).unwrap();
        assert_eq!(erased.word(RegExpProgramWord::InstructionCount), 1);
        assert_eq!(erased.word(RegExpProgramWord::CaptureCount), 1);
    });
}

#[test]
fn emitted_pattern_compiler_clears_released_memory_before_allocator_reuse() {
    run_on_sized_stack(|| {
        let mut runtime = RuntimeCompiler::new();
        let sentinel = runtime.string(&[0xa5; 128]);
        let sentinel_pointer = (sentinel as u64 >> 32) as usize;
        let mut retained = Vec::new();
        for (source, flags, expected_status) in [
            (r"(a)?\1*", "", 0),
            (r"(?:(a)|b)\1*", "", 0),
            (r"(?i:(a|b)+)\1", "d", 0),
            ("(abc", "", 1),
            ("[z-a]", "", 1),
            ("abc(?<name>x)", "", 2),
            ("(ab){5000}", "", 3),
            ("a", "ii", 1),
            ("a", "u", 2),
        ] {
            let (handle, status, _, detail, before) = runtime.call(source, flags);
            assert_eq!(
                status, expected_status,
                "{source:?}/{flags}; detail={detail}"
            );
            let after = runtime.heap_pointer();
            if status == 0 {
                let bytes = runtime.descriptor(handle);
                ValidatedRegExpProgram::from_bytes(bytes.clone()).unwrap();
                assert_eq!(after, (before + bytes.len() as i64 + 7) & !7);
                retained.push((handle, bytes));
            } else {
                assert_eq!(handle, 0);
                assert_eq!(after, before);
            }
            let abandoned = &runtime.memory.data(&runtime.store)[after as usize..];
            let dirty = abandoned.iter().position(|byte| *byte != 0);
            assert_eq!(
                dirty, None,
                "{source:?}/{flags}: released workspace contains bytes at {dirty:?}"
            );
            assert_eq!(
                &runtime.memory.data(&runtime.store)[sentinel_pointer..sentinel_pointer + 128],
                &[0xa5; 128],
                "compilation may not clear allocations before its checkpoint"
            );
            for (handle, bytes) in &retained {
                assert_eq!(runtime.descriptor(*handle), *bytes);
            }
            let reused = runtime.allocate.call(&mut runtime.store, 16_384).unwrap();
            assert_eq!(reused, after, "next allocation reuses the released region");
            assert!(
                runtime.memory.data(&runtime.store)[reused as usize..reused as usize + 16_384]
                    .iter()
                    .all(|byte| *byte == 0)
            );
        }
    });
}

#[test]
fn computed_pattern_workspace_reuse_preserves_capture_arrays_and_fresh_objects() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            include_str!("testdata/runtime-regexp-workspace-reuse.js"),
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn runtime_workspace_memory_growth_failure_is_a_typed_rollback_not_a_trap() {
    run_on_sized_stack(|| {
        let mut runtime = RuntimeCompiler::new();
        let source = runtime.string(b"(abc)+");
        let flags = runtime.string(b"i");
        let before = runtime.heap_pointer();
        let physical = runtime.memory.data_size(&runtime.store);
        runtime.store.data_mut().limits = WasmtimeStoreLimitsBuilder::new()
            .memory_size(physical)
            .build();
        let (handle, status, _, detail) = runtime
            .compile
            .call(&mut runtime.store, (source, flags, 0, 0, 0, 0, 0))
            .expect("memory.grow refusal must be caught inside compiler");
        assert_eq!((handle, status, detail), (0, 3, 2));
        assert_eq!(runtime.heap_pointer(), before);
    });
}

#[test]
fn computed_legacy_patterns_and_constructor_protocols_execute_in_wasm() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let source = include_str!("testdata/runtime-regexp-grammar.js");
    let outcome = engine
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn regexp_clones_recompile_changed_flags_and_retain_static_capabilities_for_bookkeeping_flags() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            r#"
var re = /(?<x>Ā+)/du;
var clone = new RegExp(re, 'dgu');
var iterator = 'ĀĀ Ā'.matchAll(clone);
var first = iterator.next();
var second = iterator.next();
var plain = /a/g;
Object.defineProperty(plain, 'flags', { value: 'ig' });
var folded = plain[Symbol.matchAll]('A').next();
var separator = /a/;
Object.defineProperty(separator, 'flags', { value: 'i' });
var split = 'xAy'.split(separator);
var cloned = new RegExp(/a/, 'i').exec('A');
var mutated = /a/;
var flags = { toString: function() { mutated.compile('b'); return 'i'; } };
var snapshot = new RegExp(mutated, flags);
first.value.groups.x === 'ĀĀ' && second.value.groups.x === 'Ā' &&
first.value.indices.groups.x[1] === 2 && iterator.next().done &&
!folded.done && folded.value[0] === 'A' && split.length === 2 &&
split[0] === 'x' && split[1] === 'y' && cloned[0] === 'A' &&
snapshot.source === 'a' && snapshot.test('A') && !snapshot.test('B') && mutated.source === 'b';
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn static_control_escapes_and_unmatched_backreferences_preserve_legacy_matching() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            r#"
if (/\c/.test('c')) throw 'invalid control escape lost its backslash';
if (/\c/.exec('\\c')[0] !== '\\c') throw 'invalid control escape match';
if (/\c0/.exec('\\c0')[0] !== '\\c0') throw 'digit control escape outside class';
if (/\c_/.exec('\\c_')[0] !== '\\c_') throw 'underscore control escape outside class';
if (/[\c0]/.exec('\x10')[0] !== '\x10') throw 'class digit control escape';
if (/[\c_]/.exec('\x1f')[0] !== '\x1f') throw 'class underscore control escape';

var optional = /(a)?\1*/.exec('');
var alternative = /(?:(a)|b)\1*/.exec('b');
optional[0] === '' && optional[1] === undefined &&
alternative[0] === 'b' && alternative[1] === undefined;
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn computed_scoped_modifiers_restore_lexical_flags_and_reject_invalid_prefixes() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            include_str!("testdata/runtime-regexp-modifiers.js"),
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn regexp_modifier_clones_recompile_changed_global_flags_without_changing_scopes() {
    configure_compilation_jobs(1).unwrap();
    let engine = Engine::new(RealmBuilder::new().build());
    let outcome = engine
        .run_script(
            r#"
var localCase = new RegExp(/(?i:aB)/i, '');
var sensitive = new RegExp(/(?-i:aB)/, 'i');
var localDot = new RegExp(/(?s:^.$)/s, '');
var ordinaryDot = new RegExp(/(?-s:^.$)/, 's');
var localLine = new RegExp(/(?m:es$)/m, '');
var wholeLine = new RegExp(/^(?-m:es$)/, 'm');
var original = /(?i:a)b/g;
original.lastIndex = 9;
var clone = new RegExp(original, 'ig');
var independent = clone.exec('AB');
localCase.test('Ab') && !localCase.ignoreCase &&
sensitive.test('aB') && !sensitive.test('Ab') && sensitive.ignoreCase &&
localDot.test('\n') && !localDot.dotAll &&
!ordinaryDot.test('\n') && ordinaryDot.test('x') && ordinaryDot.dotAll &&
localLine.exec('yes\nno').index === 1 && !localLine.multiline &&
wholeLine.test('es') && !wholeLine.test('es\nno') && wholeLine.multiline &&
independent[0] === 'AB' && clone.lastIndex === 2 && original.lastIndex === 9 &&
clone.source === original.source && clone.flags === 'gi';
"#,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap();
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}
