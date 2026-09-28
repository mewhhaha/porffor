use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostSurfacePolicy, RealmBuilder, RunOptions,
};

fn assert_wasm_true(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions {
                host_surface_policy: HostSurfacePolicy::Test262,
                ..CompileOptions::default()
            },
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .expect("RegExp source escaping must compile and execute through Wasm AOT");
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn slash_escaping_respects_existing_escapes_and_character_classes() {
    assert_wasm_true(
        r#"
const bs = String.fromCharCode(92);
const raw = new RegExp('/');
const alreadyEscaped = new RegExp(bs + '/');
const pairedBackslashes = new RegExp(bs + bs + '/');
const inClass = new RegExp('[/]');
const escapedBackslashInClass = new RegExp('[' + bs + bs + '/]');
const escapedBrackets = new RegExp(bs + '[' + '/' + bs + ']');
const copied = new RegExp(raw);
const recompiled = /old/;
recompiled.compile('a/b', 'g');
raw.source === bs + '/' && raw.toString() === '/' + bs + '/' + '/' &&
  raw.test('/') && copied !== raw && copied.source === bs + '/' &&
  alreadyEscaped.source === bs + '/' && alreadyEscaped.toString() === '/' + bs + '/' + '/' &&
  pairedBackslashes.source === bs + bs + bs + '/' &&
  pairedBackslashes.toString() === '/' + bs + bs + bs + '/' + '/' &&
  inClass.source === '[/]' && inClass.toString() === '/[/]/' &&
  escapedBackslashInClass.source === '[' + bs + bs + '/]' &&
  escapedBrackets.source === bs + '[' + bs + '/' + bs + ']' &&
  recompiled.source === 'a' + bs + '/b' && recompiled.toString() === '/a' + bs + '/b/g' &&
  /\//.source === bs + '/' && /\//.toString() === '/' + bs + '/' + '/' &&
  /[/]/.source === '[/]';
"#,
    );
}

#[test]
fn raw_line_terminators_are_escaped_without_changing_backslash_parity() {
    assert_wasm_true(
        r#"
const bs = String.fromCharCode(92);
let valid = true;
for (const [unit, suffix] of [[10, 'n'], [13, 'r'], [0x2028, 'u2028'], [0x2029, 'u2029']]) {
  const terminator = String.fromCharCode(unit);
  const raw = new RegExp(terminator);
  const precededByOne = new RegExp(bs + terminator);
  const precededByTwo = new RegExp(bs + bs + terminator);
  valid = valid && raw.source === bs + suffix &&
    raw.toString() === '/' + bs + suffix + '/' &&
    precededByOne.source === bs + suffix &&
    precededByTwo.source === bs + bs + bs + suffix;
}
valid && new RegExp('\r\n').source === bs + 'r' + bs + 'n' &&
  new RegExp('[' + String.fromCharCode(10) + ']').source === '[' + bs + 'n]' &&
  /\n\r\u2028\u2029/.source === bs + 'n' + bs + 'r' + bs + 'u2028' + bs + 'u2029';
"#,
    );
}

#[test]
fn utf16_code_units_flags_and_empty_patterns_survive_escaping() {
    assert_wasm_true(
        r#"
const bs = String.fromCharCode(92);
const high = String.fromCharCode(0xD800);
const low = String.fromCharCode(0xDC00);
const astral = String.fromCharCode(0xD834, 0xDF06);
const lone = new RegExp(high + '/' + low, 'g');
const unicode = new RegExp(astral + '/', 'u');
const unicodeSets = new RegExp('a/b', 'v');
const empty = new RegExp('', 'im');
lone.source === high + bs + '/' + low &&
  lone.toString() === '/' + high + bs + '/' + low + '/g' &&
  unicode.source === astral + bs + '/' && unicode.toString() === '/' + astral + bs + '//u' &&
  unicodeSets.source === 'a' + bs + '/b' && unicodeSets.toString() === '/a' + bs + '/b/v' &&
  empty.source === '(?:)' && empty.toString() === '/(?:)/im' &&
  RegExp.prototype.source === '(?:)' && /a/u.source === 'a';
"#,
    );
}

#[test]
fn source_getter_escapes_cross_realm_instances_but_keeps_receiver_checks() {
    assert_wasm_true(
        r#"
const other = __lilaCreateRealm().global;
const foreign = other.RegExp('a/b');
const localGet = Object.getOwnPropertyDescriptor(RegExp.prototype, 'source').get;
const foreignGet = Object.getOwnPropertyDescriptor(other.RegExp.prototype, 'source').get;
let rejectsPrototype = false;
let rejectsProxy = false;
try { localGet.call(other.RegExp.prototype); } catch (error) { rejectsPrototype = error instanceof TypeError; }
try { localGet.call(new Proxy(/x/, {})); } catch (error) { rejectsProxy = error instanceof TypeError; }
localGet.call(foreign) === 'a\\/b' && foreignGet.call(foreign) === 'a\\/b' &&
  foreign.toString() === '/a\\/b/' &&
  localGet.call(RegExp.prototype) === '(?:)' && rejectsPrototype && rejectsProxy;
"#,
    );
}
