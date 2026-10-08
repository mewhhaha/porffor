function check(value, label) { if (!value) throw label; }
var nativeArray = Array, entryOnly = {marker: 17};
globalThis.nativeHostEntryDeclaration = entryOnly;
var realm = __lilaCreateRealm(), foreign = realm.global;
check(foreign !== globalThis && foreign.Object !== Object && foreign.Array !== nativeArray, 'fresh Realm identities');
check(foreign.nativeHostEntryDeclaration === undefined && foreign.__lilaAgentReport === undefined,
  'created global excludes source and entry-only host rows');
check(foreign.parseInt('10', 16) === 16 && foreign.parseFloat('1.5tail') === 1.5, 'canonical every-Realm parsers');
realm.evalScript("globalThis.nativeHostPrepared = 29;");
check(foreign.nativeHostPrepared === 29 && globalThis.nativeHostPrepared === undefined, 'prepared Script uses retained global Environment');
var LocalTypeError = TypeError, ForeignTypeError = foreign.TypeError;
var localTypePrototype = LocalTypeError.prototype, foreignTypePrototype = ForeignTypeError.prototype;
var getPrototypeOf = Object.getPrototypeOf;
globalThis.TypeError = function() { throw 'mutable local TypeError'; };
foreign.TypeError = function() { throw 'mutable foreign TypeError'; };
try {
  var parsers = [parseInt, foreign.parseInt, parseFloat, foreign.parseFloat];
  for (var i = 0; i < parsers.length; i++) {
    var error = undefined;
    try { parsers[i](Symbol('invalid string')); } catch (caught) { error = caught; }
    check(error !== undefined && getPrototypeOf(error) === (i % 2 === 0 ? localTypePrototype : foreignTypePrototype), 'parser error belongs to called Realm');
  }
} finally { globalThis.TypeError = LocalTypeError; foreign.TypeError = ForeignTypeError; }
var live = {child: {value: 31}}, readLive = function() { return live.child.value; };
gc();
check(readLive() === 31, 'collector retains object and closure graph');
function Constructor() {}
var constructable = new Proxy(Constructor, {});
check(__lilaIsConstructor(constructable) === true && __lilaIsConstructor(() => 0) === false, 'constructor capability of Proxy and Arrow');
var revoked = Proxy.revocable(Constructor, {}); revoked.revoke();
check(__lilaIsConstructor(revoked.proxy) === true, 'revoked Proxy retains constructor capability');
var html = __lilaCreateHTMLDDA();
check(typeof html === 'undefined' && html == null && !html && html() === null, 'HTMLDDA complete callable value');
Object.defineProperty(html, 'owned', {value: 37, configurable: true});
gc();
check(html.owned === 37, 'HTMLDDA remains extensible and rooted');
var expected = function Expected() {}, gets = 0, calls = 0;
var thrown = new Proxy({}, {get(target, key, receiver) {
  if (key === 'constructor') { gets++; return expected; }
  return Reflect.get(target, key, receiver);
}});
var callback = new Proxy(function() { calls++; throw thrown; }, {});
check(__lilaAssertThrows(expected, callback) === undefined && calls === 1 && gets === 1,
  'callable Proxy and observable constructor once');
var wrong = Object.create(expected.prototype), wrongConstructor = function Wrong() {};
wrong.constructor = wrongConstructor;
var rejected = false;
try { __lilaAssertThrows(expected, function() { throw wrong; }); } catch (error) { rejected = true; }
check(rejected, 'same prototype never replaces constructor identity');
var marker = Symbol('constructor getter abrupt'), caught;
var getterThrow = {get constructor() { throw marker; }};
try { __lilaAssertThrows(expected, function() { throw getterThrow; }); } catch (error) { caught = error; }
check(caught === marker, 'whole thrown constructor getter abrupt');
check(__lilaAssertThrows(ForeignTypeError, function() { throw new ForeignTypeError('foreign'); }) === undefined,
  'foreign exact constructor identity');
print('native-host-realms:ok');
262;
