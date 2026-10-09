var $262 = { createRealm: __lilaCreateRealm };
function check(condition, message) { if (!condition) throw new Error(message); }
const parse = JSON.parse;
const raw = JSON.rawJSON;
const isRaw = JSON.isRawJSON;
const high = String.fromCharCode(0xd800);
const low = String.fromCharCode(0xdc00);
const pair = String.fromCharCode(0xd83d, 0xde00);
const unicode = parse('"' + high + pair + low + '"');
check(unicode.length === 4 && unicode.charCodeAt(0) === 0xd800 &&
      unicode.charCodeAt(3) === 0xdc00, 'UTF16 parse units');
check(parse('"\\ud800\\ud83d\\ude00\\udc00"') === unicode, 'escaped UTF16 units');
check(1 / parse('-0') === -Infinity && parse('1e400') === Infinity, 'number grammar');
check(parse('9007199254740993') === 9007199254740992, 'binary64 rounding');
const invalid = ['[1,]', '{"x":1,}', '01', '1.', '1e+', '\ufeff1', '"\n"', '"\x00"', 'true false'];
for (let i = 0; i < invalid.length; i++) {
  let rejected = false;
  try { parse(invalid[i]); } catch (error) { rejected = error instanceof SyntaxError; }
  check(rejected, 'invalid grammar ' + i);
}
let sourceForX;
const duplicates = parse('{"__proto__":7,"x":1,"x":2,"neg":-0,"str":"\\ud800"}',
  new Proxy(function (key, value, context) {
    if (key === 'x') sourceForX = context.source;
    if (key === 'neg') check(context.source === '-0', 'negative zero source');
    if (key === 'str') check(context.source === '"\\ud800"', 'String source spelling');
    return value;
  }, { apply(target, receiver, args) { return Reflect.apply(target, receiver, args); } }));
check(Object.getPrototypeOf(duplicates) === Object.prototype &&
      Object.hasOwn(duplicates, '__proto__') && duplicates.__proto__ === 7 &&
      duplicates.x === 2 && sourceForX === '2', 'last duplicate and proto property');
let trace = '';
const mutated = parse('{"first":0,"later":{"old":1},"tail":9}', function (key, value, context) {
  if (key === 'first') {
    this.later = { fresh: 3 };
    delete this.tail;
    Object.defineProperty(this, 'inserted', { value: 5, enumerable: true });
  }
  if (key === 'fresh') check(!Object.hasOwn(context, 'source'), 'replacement source omitted');
  if (key === 'tail') check(value === undefined && !Object.hasOwn(context, 'source'), 'live deleted value');
  trace += key + ',';
  return value;
});
check(trace === 'first,fresh,later,tail,,', 'live values over own-key snapshot');
check(mutated.later.fresh === 3 && mutated.inserted === 5 && !Object.hasOwn(mutated, 'tail'), 'reviver mutation');
let sparseTrace = '';
const sparse = parse('[0,1]', function (key, value, context) {
  if (key === '0') {
    delete this[1];
    Object.setPrototypeOf(this, { 1: 8 });
  }
  if (key === '1') {
    sparseTrace += value;
    check(!Object.hasOwn(context, 'source'), 'inherited source omitted');
  }
  return value;
});
check(sparseTrace === '8' && Object.hasOwn(sparse, '1') && sparse[1] === 8, 'array live inherited Get');
const text = raw('"\\ud800"');
check(isRaw(text) && !isRaw(new Proxy(text, {})), 'raw exact brand');
check(Object.getPrototypeOf(text) === null && Object.isFrozen(text) &&
      JSON.stringify([text, raw('-0'), raw('1e400')]) === '["\\ud800",-0,1e400]', 'raw text publication');
for (const bad of [' 1', '1 ', '{}', '[]', '', 'false\n']) {
  let failed = false;
  try { raw(bad); } catch (error) { failed = error instanceof SyntaxError; }
  check(failed, 'raw primitive grammar');
}
const foreign = $262.createRealm();
const foreignParse = foreign.evalScript('JSON.parse');
const foreignRaw = foreign.evalScript('JSON.rawJSON');
const ForeignSyntaxError = foreign.evalScript('SyntaxError');
const ForeignObject = foreign.evalScript('Object');
let seenContext;
foreignParse('{"v":1}', function (key, value, context) {
  if (key === 'v') seenContext = Object.getPrototypeOf(context);
  return value;
});
check(seenContext === ForeignObject.prototype, 'called Realm context');
for (const method of [foreignParse, foreignRaw]) {
  let error;
  try { method('['); } catch (thrown) { error = thrown; }
  check(error instanceof ForeignSyntaxError && !(error instanceof SyntaxError), 'foreign syntax error');
}
foreign.global.savedParse = parse;
foreign.global.savedSyntax = SyntaxError;
check(foreign.evalScript('try { savedParse("["); false; } catch (e) { e instanceof savedSyntax && !(e instanceof SyntaxError); }'), 'reverse called Realm');
const sentinel = Symbol('original');
let abruptTrace = '';
try {
  parse({ toString() { abruptTrace += 'text,'; throw sentinel; } }, function () { abruptTrace += 'reviver,'; });
} catch (error) { check(error === sentinel, 'text Throw identity'); }
finally { abruptTrace += 'finally'; }
check(abruptTrace === 'text,finally', 'text abrupt cutoff');
let proxyTrace = '';
const failing = new Proxy(function () {}, { apply() { proxyTrace += 'call,'; throw sentinel; } });
try { parse('[1,2]', failing); } catch (error) { check(error === sentinel, 'reviver Throw identity'); }
finally { proxyTrace += 'finally'; }
check(proxyTrace === 'call,finally', 'reviver abrupt cutoff');
print('json-gc-parse-reviver:ok');
262;
