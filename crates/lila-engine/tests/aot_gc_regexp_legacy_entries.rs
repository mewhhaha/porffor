use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_legacy_modes(source: &str, line: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compiler worker");
    for directive in ["", "'use strict';\n"] {
        let source = format!("{directive}{source}");
        let observed = Engine::new(RealmBuilder::new().build())
            .observe_script(
                &source,
                CompileOptions {
                    host_surface_policy: HostSurfacePolicy::Test262,
                    ..CompileOptions::default()
                },
                RunOptions {
                    backend: ExecutionBackend::WasmAot,
                    timeout_ms: Some(30_000),
                    ..RunOptions::default()
                },
            )
            .unwrap_or_else(|error| panic!("legacy RegExp AOT control: {error}\n{source}"));
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0)))
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(line.into())]
        );
    }
}

#[test]
fn legacy_match_state_and_input_conversion_preserve_utf16_captures_and_reentrancy() {
    assert_legacy_modes(
        r#"
function check(value, label) { if (!value) throw new Error(label); }
var names = ['input', '$_', 'lastMatch', '$&', 'lastParen', '$+', 'leftContext', '$`', 'rightContext', "$'", '$1', '$9'];
for (var i = 0; i < names.length; i++) check(RegExp[names[i]] === '', 'initial ' + names[i]);
var captures = /(a)(b)?()/;
var lastIndexDescriptor = Object.getOwnPropertyDescriptor(captures, 'lastIndex');
check(lastIndexDescriptor.value === 0 && lastIndexDescriptor.writable === true && lastIndexDescriptor.enumerable === false && lastIndexDescriptor.configurable === false, 'literal lastIndex descriptor');
var result = captures.exec('\uD83D\uDE00a!');
check(result[0] === 'a' && result[2] === undefined && result[3] === '', 'actual capture array');
check(RegExp.input === '\uD83D\uDE00a!' && RegExp.$_ === RegExp.input, 'input aliases');
check(RegExp.lastMatch === 'a' && RegExp['$&'] === 'a', 'whole match aliases');
check(RegExp.leftContext === '\uD83D\uDE00' && RegExp['$`'] === RegExp.leftContext, 'UTF16 left context');
check(RegExp.rightContext === '!' && RegExp["$'"] === '!', 'right context');
check(RegExp.$1 === 'a' && RegExp.$2 === '' && RegExp.$3 === '' && RegExp.$9 === '' && RegExp.lastParen === '', 'empty and unmatched captures');
/(a)(b)(c)(d)(e)(f)(g)(h)(i)(j)/.exec('abcdefghij');
check(RegExp.$1 === 'a' && RegExp.$9 === 'i' && RegExp.lastParen === 'j' && RegExp['$+'] === 'j', 'last capture beyond nine');
/x/.exec('nothing');
check(RegExp.input === 'abcdefghij' && RegExp.lastMatch === 'abcdefghij', 'failed match retains state');
var order = [];
var captured = 1;
function readCaptured() { return captured; }
RegExp.input = { toString: function () { order.push('convert'); captured = 7; /q/.exec('xqz'); return 'assigned'; } };
check(captured === 7 && readCaptured() === 7, 'setter invalidates captured facts');
check(order.join(',') === 'convert' && RegExp.input === 'assigned' && RegExp.lastMatch === 'q' && RegExp.leftContext === 'x', 'setter reentrant state');
var token = {};
var caught;
try { RegExp.$_ = { toString: function () { throw token; } }; } catch (error) { caught = error; }
check(caught === token && RegExp.input === 'assigned' && RegExp.lastMatch === 'q', 'abrupt setter retains original');
caught = undefined;
try { RegExp.input = Symbol('input'); } catch (error) { caught = error; }
check(caught instanceof TypeError && RegExp.input === 'assigned', 'Symbol implicit ToString');
var descriptor = Object.getOwnPropertyDescriptor(RegExp, 'input');
check(descriptor.get.name === 'get input' && descriptor.get.length === 0 && descriptor.set.name === 'set input' && descriptor.set.length === 1, 'accessor metadata');
check(descriptor.enumerable === false && descriptor.configurable === true, 'accessor descriptor');
check(Object.getOwnPropertyDescriptor(RegExp, 'lastMatch').set === undefined, 'read-only match accessor');
var converted = 0;
caught = undefined;
try { descriptor.set.call({}, { toString: function () { converted++; return 'wrong'; } }); } catch (error) { caught = error; }
check(caught instanceof TypeError && converted === 0 && RegExp.input === 'assigned', 'receiver precedes conversion');
print('regexp-legacy-state:ok');
262;
"#,
        "regexp-legacy-state:ok",
    );
}

#[test]
fn legacy_subclass_invalidation_and_borrowed_realms_preserve_actual_owners() {
    assert_legacy_modes(
        r#"
function check(value, label) { if (!value) throw new Error(label); }
/m/.exec('main');
var $262 = { createRealm: __lilaCreateRealm };
var foreign = $262.createRealm();
var ForeignRegExp = foreign.global.RegExp;
var foreignRx = new ForeignRegExp('(z)');
foreignRx.exec('azb');
check(ForeignRegExp.lastMatch === 'z' && ForeignRegExp.input === 'azb', 'foreign native match owns foreign state');
check(RegExp.lastMatch === 'm' && RegExp.input === 'main', 'foreign call preserves main state');
RegExp.prototype.exec.call(foreignRx, 'czd');
check(ForeignRegExp.input === 'azb' && RegExp.input === 'main', 'borrowed local exec does not update foreign state');
var local = /m/;
ForeignRegExp.prototype.exec.call(local, 'more');
check(ForeignRegExp.input === 'azb' && RegExp.input === 'main', 'borrowed foreign exec does not update local state');
var foreignGetter = Object.getOwnPropertyDescriptor(ForeignRegExp, 'lastMatch').get;
var caught;
try { foreignGetter.call(RegExp); } catch (error) { caught = error; }
check(Object.getPrototypeOf(caught) === foreign.global.TypeError.prototype, 'getter error owns called Realm');
caught = undefined;
try { RegExp.prototype.compile.call(foreignRx, 'changed'); } catch (error) { caught = error; }
check(caught instanceof TypeError && foreignRx.source === '(z)', 'cross Realm compile rejects before mutation');
class Derived extends RegExp {}
var derived = new Derived('(d)');
check(derived.exec('d')[0] === 'd', 'subclass matcher remains real');
var inputSetter = Object.getOwnPropertyDescriptor(RegExp, 'input').set;
for (var name of ['input', 'lastMatch', 'lastParen', 'leftContext', 'rightContext', '$1', '$9']) {
  caught = undefined;
  try { RegExp[name]; } catch (error) { caught = error; }
  check(caught instanceof TypeError, 'invalidated ' + name);
}
inputSetter.call(RegExp, 'restored-input');
check(RegExp.input === 'restored-input', 'setter restores only input');
caught = undefined;
try { RegExp.lastMatch; } catch (error) { caught = error; }
check(caught instanceof TypeError, 'other invalidated slots remain unavailable');
caught = undefined;
try { RegExp.prototype.compile.call(derived, 'x'); } catch (error) { caught = error; }
check(caught instanceof TypeError && derived.source === '(d)', 'subclass compile rejects');
/(r)/.exec('recovered');
check(RegExp.input === 'recovered' && RegExp.$1 === 'r' && RegExp.lastMatch === 'r', 'ordinary success restores all slots');
check(ForeignRegExp.input === 'azb', 'Realm states remain independent');
print('regexp-legacy-realms:ok');
262;
"#,
        "regexp-legacy-realms:ok",
    );
}
