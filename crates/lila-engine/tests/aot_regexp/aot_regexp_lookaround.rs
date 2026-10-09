use lila_engine::{CompileOptions, Engine, ExecutionBackend, RealmBuilder, RunOptions};

fn assert_lookaround(source: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one compilation worker");
    let outcome = Engine::new(RealmBuilder::new().build())
        .run_script(
            source,
            CompileOptions::default(),
            RunOptions {
                backend: ExecutionBackend::WasmAot,
                ..RunOptions::default()
            },
        )
        .unwrap_or_else(|error| panic!("lookaround execution failed: {error}\n{source}"));
    assert_eq!(outcome.backend_used, ExecutionBackend::WasmAot);
    assert!(outcome.note.contains("boolean(true)"), "{}", outcome.note);
}

#[test]
fn lookahead_atoms_use_their_grammar_and_scoped_flags() {
    assert_lookaround(
        r#"
var java = /[Jj]ava([Ss]cript)?(?=\:)/.exec('taste of java: the cookbook ');
/(?=.)/.test('x') && !/(?=.)/.test('\n') && /(?=.)/s.test('\n') &&
/(?=$)/.test('') && /(?=^abc)/m.exec('x\nabc').index === 2 &&
/(?=a)/i.test('A') && /(?i:(?=a))A/.test('A') && !/(?i:(?-i:(?=a)))A/.test('A') &&
/(?=)/.test('') && !/(?!)/.test('') && /(?!ab)ac/.test('ac') &&
java.length === 2 && java[0] === 'java' && java[1] === undefined && java.index === 9;
"#,
    );
}

#[test]
fn positive_captures_are_atomic_and_do_not_consume_the_input() {
    assert_lookaround(
        r#"
var a = /(?=(a+))/.exec('baaabac');
var b = /(?=(a+))a*b\1/.exec('baaabac');
var c = /(?=(a|aa))\1b/.exec('aab');
var d = /(?=(?<pair>ab))\k<pair>/.exec('ab');
a[0] === '' && a[1] === 'aaa' && a.index === 1 &&
b[0] === 'aba' && b[1] === 'a' && b.index === 3 &&
c[0] === 'ab' && c[1] === 'a' && c.index === 1 &&
d[0] === 'ab' && d.groups.pair === 'ab';
"#,
    );
}

#[test]
fn negative_lookahead_restores_captures_and_ordered_outer_alternatives() {
    assert_lookaround(
        r#"
var a = /(.*?)a(?!(a+)b\2c)\2(.*)/.exec('baaabaac');
var b = /(?!(a)b)c|(?=(d))d/.exec('d');
a.length === 4 && a[0] === 'baaabaac' && a[1] === 'ba' &&
a[2] === undefined && a[3] === 'abaac' &&
b[0] === 'd' && b[1] === undefined && b[2] === 'd' &&
/(?!(a)b)\1c/.exec('c')[0] === 'c';
"#,
    );
}

#[test]
fn nested_assertions_restore_the_enclosing_matching_direction() {
    assert_lookaround(
        r#"
var a = /(?<=a(?=b))b/.exec('ab');
var b = /(?<=a(?!c))b/.exec('ab');
var c = /(?=a(?<=x)a)aa/.exec('xaa');
var d = /(?<=a(?=(?<!x)b))b/.exec('ab');
var e = /(?<=(a)(?=(b)))b/.exec('ab');
var f = /(?<=a(?=(?!c)b))b/.exec('ab');
a[0] === 'b' && b[0] === 'b' && c === null && d[0] === 'b' &&
e[0] === 'b' && e[1] === 'a' && e[2] === 'b' && f[0] === 'b' &&
/(?<=a(?!(?=b)b))b/.exec('ab') === null &&
/(?<=(?:x(?=c)|b))c/.exec('bc')[0] === 'c' &&
/(?=(?<=x)aa)aa/.exec('xaa')[0] === 'aa';
"#,
    );
}

#[test]
fn lookahead_preserves_utf16_indices_global_progress_and_sticky_last_index() {
    assert_lookaround(
        r#"
var a = /(?=(?<face>😀))/du.exec('x😀');
var b = /(?=😀)/u.exec('x😀');
var c = /(?=\p{Script=Han})\p{Script=Han}/v.exec('x𠮷');
var sticky = /(?=(ab))ab/dy;
sticky.lastIndex = 1;
var d = sticky.exec('xab');
var matches = 'aba'.match(/(?=a)/g);
a.index === 1 && a[0] === '' && a.groups.face === '😀' &&
a.indices[0][0] === 1 && a.indices[0][1] === 1 &&
a.indices.groups.face[0] === 1 && a.indices.groups.face[1] === 3 &&
b.index === 1 && c.index === 1 && c[0] === '𠮷' &&
d[0] === 'ab' && d[1] === 'ab' && d.indices[1][0] === 1 &&
d.indices[1][1] === 3 && sticky.lastIndex === 3 && matches.length === 2;
"#,
    );
}

#[test]
fn legacy_quantified_lookaheads_keep_capture_and_zero_progress_semantics() {
    assert_lookaround(
        r#"
var optional = /(?=(a))*\1b/.exec('b');
var required = /(?=(a)){2,3}a/.exec('a');
var outer = /(?:(?=a))*a/.exec('a');
var nullable = /(?:(?!(a))\1)*b/.exec('b');
optional[0] === 'b' && optional[1] === undefined &&
required[0] === 'a' && required[1] === 'a' &&
outer[0] === 'a' && nullable[0] === 'b' && nullable[1] === undefined;
"#,
    );
}

#[test]
fn repeated_assertions_clear_captures_from_an_earlier_iteration() {
    assert_lookaround(
        r#"
var a = /(?:(?=(a)|(b)).){2}/.exec('ab');
var b = /(?:(?!(a)c).){2}/.exec('ab');
var c = /(?=(?:(a)|b)+)ab/.exec('ab');
a[0] === 'ab' && a[1] === undefined && a[2] === 'b' &&
b[0] === 'ab' && b[1] === undefined && c[0] === 'ab' && c[1] === undefined;
"#,
    );
}

#[test]
fn lookbehind_uses_existing_reverse_scalar_ranges_and_legacy_pair_matching() {
    assert_lookaround(
        r#"
var a = /(?<=😀)x/.exec('😀x');
var b = /(?<=(😀))x/du.exec('😀x');
var c = /(?<=\p{Script=Han})x/v.exec('𠮷x');
var d = /(?<=😀+)x/.exec('😀\uDE00x');
a[0] === 'x' && a.index === 2 && b[0] === 'x' && b[1] === '😀' &&
b.indices[1][0] === 0 && b.indices[1][1] === 2 &&
c[0] === 'x' && c.index === 2 && d[0] === 'x' && d.index === 3;
"#,
    );
}

#[test]
fn reverse_ascii_classes_do_not_alias_non_ascii_codepoints() {
    assert_lookaround(
        r#"
!/(?<=[a-z])b/.test('áb') && /(?<=[^a-z])b/.test('áb') &&
!/(?<=[a-z])b/u.test('áb') && /(?<=[^a-z])b/u.test('áb') &&
!/(?<=[A-Z])b/.test('Áb') && /(?<=[^A-Z])b/.test('Áb') &&
!/(?=[a-z])/.test('á') && /(?=[^a-z])/.test('á');
"#,
    );
}
