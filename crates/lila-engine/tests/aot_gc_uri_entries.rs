use lila_engine::{
    CompileOptions, Engine, ExecutionBackend, HostOutputEvent, HostSurfacePolicy,
    ObservedCompletion, ObservedJsValue, ObservedNumber, RealmBuilder, RunOptions,
};

fn assert_uri_modes(source: &str, line: &str) {
    lila_engine::configure_compilation_jobs(1).expect("one bounded compilation worker");
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
            .expect("finite URI native control compiles and executes through Wasm AOT");
        assert_eq!(observed.backend_used, ExecutionBackend::WasmAot);
        assert_eq!(
            observed.completion,
            ObservedCompletion::Normal(ObservedJsValue::Number(ObservedNumber::from_f64(262.0))),
            "{source}"
        );
        assert_eq!(
            observed.output_events,
            vec![HostOutputEvent::PrintLine(line.into())],
            "{source}"
        );
    }
}

#[test]
fn gc_uri_codecs_and_annexb_preserve_exact_utf16_units() {
    assert_uri_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
var punctuation = "AZaz09_- .!~*'();/?:@&=+$,#";
check(encodeURI(punctuation) === "AZaz09_-%20.!~*'();/?:@&=+$,#", 'URI extra unescaped set');
check(encodeURIComponent(punctuation) === "AZaz09_-%20.!~*'()%3B%2F%3F%3A%40%26%3D%2B%24%2C%23", 'component punctuation');
var text = '\u0000\u007f\u0080\u07ff\u0800\uffff\ud83d\ude00\udbff\udfff';
var encoded = '%00%7F%C2%80%DF%BF%E0%A0%80%EF%BF%BF%F0%9F%98%80%F4%8F%BF%BF';
check(encodeURI(text) === encoded && encodeURIComponent(text) === encoded, 'all UTF8 width boundaries');
check(decodeURI(encoded) === text && decodeURIComponent(encoded.toLowerCase()) === text, 'exact scalar decoding and hex case');
var preserved = '%2f%3F%23%24%26%2b%2C%3A%3b%3D%40';
check(decodeURI(preserved) === preserved, 'reserved escapes preserve original spelling');
check(decodeURIComponent(preserved) === '/?#$&+,:;=@', 'component decodes reserved escapes');
check(decodeURI('%41%5f%2d%2e%21%7e%2a%27%28%29') === "A_-.!~*'()", 'only reserved set preserved');
check(decodeURIComponent('\ud800%41\udc00\ud83d\ude00') === '\ud800A\udc00\ud83d\ude00', 'raw UTF16 copied without URI scalar validation');
check(encodeURI('') === '' && decodeURIComponent('') === '' && escape('') === '' && unescape('') === '', 'empty String roots');
check(encodeURI() === 'undefined' && decodeURIComponent() === 'undefined', 'missing argument ToString');
check(escape("AZaz09_@*+-./ !~'()\u00ff\u0100\ud83d\ude00\ud800") === "AZaz09_@*+-./%20%21%7E%27%28%29%FF%u0100%uD83D%uDE00%uD800", 'Annex B code units and uppercase hex');
check(unescape('%uD801%uDC01') === '\ud801\udc01', 'decoded pair is one ECMAScript String');
check(unescape('%uD801\udc01') === '\ud801\udc01' && unescape('\ud801%uDC01') === '\ud801\udc01', 'decoded and raw boundaries');
check(unescape('%uD800X%uDC00%00%FF') === '\ud800X\udc00\u0000\u00ff', 'lone units and byte escapes');
check(unescape('%u12%41') === '%u12A', 'invalid long token retains percent then scans suffix');
check(unescape('%U0041%u004g%0g%u12%') === '%U0041%u004g%0g%u12%', 'malformed Annex B tokens copied literally');
check(unescape('%u00e9%41\u00e9\ud83d\ude00') === '\u00e9A\u00e9\ud83d\ude00', 'mixed raw and escaped units');
check(unescape(escape(text)) === text, 'Annex B full unit boundary roundtrip');
print('gc-uri-codecs:ok');
262;
"#,
        "gc-uri-codecs:ok",
    );
}

#[test]
fn gc_uri_rejects_malformed_octets_without_publishing_partial_results() {
    assert_uri_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
var bad = ['%', '%0', '%GG', '%u0041', '%80', '%BF', '%C0%80', '%C1%BF',
  '%C2', '%C2A0', '%C2%7F', '%C2%C0', '%E0%80%80', '%E2%82', '%E2%28%A1',
  '%ED%A0%80', '%ED%BF%BF', '%F0%80%80%80', '%F0%9F%98', '%F4%90%80%80',
  '%F5%80%80%80', '%F8%88%80%80%80', '%FF'];
var marker = {};
for (var i = 0; i < bad.length; ++i) {
  for (var j = 0; j < 2; ++j) {
    var fn = j === 0 ? decodeURI : decodeURIComponent;
    var prior = marker, caught = undefined, finallyCount = 0;
    try { prior = fn('prefix' + bad[i]); }
    catch (error) { caught = error; }
    finally { ++finallyCount; }
    check(caught instanceof URIError && prior === marker && finallyCount === 1, 'malformed decode ' + i + ':' + j);
  }
}
var lone = ['\ud800', '\udbff', '\udc00', '\udfff', 'a\ud800b', '\ud800\ud800', '\udc00\ud800'];
for (var i = 0; i < lone.length; ++i) {
  for (var j = 0; j < 2; ++j) {
    var fn = j === 0 ? encodeURI : encodeURIComponent;
    var caught = undefined;
    try { fn(lone[i]); } catch (error) { caught = error; }
    check(caught instanceof URIError, 'unpaired encoder unit ' + i + ':' + j);
  }
}
check(decodeURIComponent('%C2%80%E0%A0%80%ED%9F%BF%EE%80%80%F0%90%80%80%F4%8F%BF%BF') ===
  '\u0080\u0800\ud7ff\ue000\ud800\udc00\udbff\udfff', 'shortest form and surrogate boundary neighbors');
check(unescape('%ED%A0%80') === '\u00ed\u00a0\u0080', 'Annex B is code units not UTF8 decode');
print('gc-uri-malformed:ok');
262;
"#,
        "gc-uri-malformed:ok",
    );
}

#[test]
fn gc_uri_coercion_original_throws_and_called_realm_errors() {
    assert_uri_modes(
        r#"
function check(ok, label) { if (!ok) throw new Error(label); }
var trace = [];
var nativeEncode = encodeURIComponent;
var original = { get: 0 };
var input = new Proxy(original, { get: function (target, key) {
  trace.push(key === Symbol.toPrimitive ? 'get:primitive' : 'unexpected');
  return new Proxy(function (hint) { return 'a b'; }, { apply: function (fn, receiver, args) {
    check(receiver === input && args.length === 1 && args[0] === 'string', 'original coercion receiver and hint');
    trace.push('apply:primitive'); return 'a b';
  } });
} });
function first() { trace.push('argument'); encodeURIComponent = function () { throw new Error('replacement must not run'); }; return input; }
function extra() { trace.push('extra'); return { toString: function () { throw new Error('ignored operand coerced'); } }; }
try {
  check(nativeEncode.call({ toString: function () { throw new Error('raw this must be ignored'); } }, first(), ...[extra()]) === 'a%20b', 'saved native and complete argv');
  check(trace.join(',') === 'argument,extra,get:primitive,apply:primitive', 'arguments precede exactly one ToString');
} finally { encodeURIComponent = nativeEncode; }
var $262 = { createRealm: __lilaCreateRealm };
var foreign = $262.createRealm().global;
var marker = new foreign.Error('original URI marker');
var locals = [escape, unescape, encodeURI, encodeURIComponent, decodeURI, decodeURIComponent];
var others = [foreign.escape, foreign.unescape, foreign.encodeURI, foreign.encodeURIComponent, foreign.decodeURI, foreign.decodeURIComponent];
var getPrototypeOf = Object.getPrototypeOf;
var localType = TypeError, localUri = URIError, foreignType = foreign.TypeError, foreignUri = foreign.URIError;
var localTypeProto = TypeError.prototype, localUriProto = URIError.prototype;
var foreignTypeProto = foreign.TypeError.prototype, foreignUriProto = foreign.URIError.prototype;
var oldForeignDecode = foreign.decodeURI;
try {
  TypeError = function () { throw new Error('poison local TypeError'); };
  URIError = function () { throw new Error('poison local URIError'); };
  foreign.TypeError = function () { throw marker; };
  foreign.URIError = function () { throw marker; };
  encodeURIComponent = function () { throw new Error('poison local codec'); };
  foreign.decodeURI = function () { throw marker; };
  for (var direction = 0; direction < 2; ++direction) {
    var methods = direction === 0 ? locals : others;
    var typeProto = direction === 0 ? localTypeProto : foreignTypeProto;
    var uriProto = direction === 0 ? localUriProto : foreignUriProto;
    for (var i = 0; i < methods.length; ++i) {
      var caught = undefined;
      try { methods[i].call(null, Symbol('cannot stringify')); } catch (error) { caught = error; }
      check(getPrototypeOf(caught) === typeProto, 'called Realm TypeError ' + direction + ':' + i);
      trace = [];
      var prior = original, finallyCount = 0;
      var throwing = { get [Symbol.toPrimitive]() { trace.push('get'); throw marker; } };
      caught = undefined;
      try { prior = methods[i](throwing, (trace.push('extra'), 1)); }
      catch (error) { caught = error; }
      finally { ++finallyCount; }
      check(caught === marker && prior === original && finallyCount === 1 && trace.join(',') === 'extra,get', 'original getter abrupt ' + direction + ':' + i);
      trace = [];
      var calling = { [Symbol.toPrimitive]: new Proxy(function () {}, { apply: function (target, receiver, args) { trace.push('apply'); throw marker; } }) };
      caught = undefined;
      try { methods[i](calling); } catch (error) { caught = error; }
      check(caught === marker && trace.join(',') === 'apply', 'original apply abrupt ' + direction + ':' + i);
    }
    for (var i = 2; i < methods.length; ++i) {
      var caught = undefined;
      try { methods[i](i < 4 ? '\ud800' : '%ED%A0%80'); } catch (error) { caught = error; }
      check(getPrototypeOf(caught) === uriProto, 'called Realm URIError ' + direction + ':' + i);
    }
  }
} finally {
  TypeError = localType; URIError = localUri; foreign.TypeError = foreignType; foreign.URIError = foreignUri;
  encodeURIComponent = nativeEncode; foreign.decodeURI = oldForeignDecode;
}
print('gc-uri-realms:ok');
262;
"#,
        "gc-uri-realms:ok",
    );
}
