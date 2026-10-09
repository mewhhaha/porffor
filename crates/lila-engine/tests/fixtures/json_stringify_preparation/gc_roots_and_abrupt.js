var $262 = { createRealm: __lilaCreateRealm };
function check(condition, message) { if (!condition) throw new Error(message); }
const stringify = JSON.stringify;
const high = String.fromCharCode(0xd800);
const low = String.fromCharCode(0xdc00);
const pair = String.fromCharCode(0xd83d, 0xde00);
check(stringify(high + pair + low) === '"\\ud800' + pair + '\\udc00"', 'well-formed UTF16 quote');
check(stringify('\b\t\n\f\r"\\') === '"\\b\\t\\n\\f\\r\\"\\\\"', 'escape spellings');
check(stringify([undefined, Symbol('x'), function () {}, NaN, Infinity, -0]) === '[null,null,null,null,null,0]', 'array primitive roles');
const sibling = { v: 1 };
check(stringify([sibling, sibling]) === '[{"v":1},{"v":1}]', 'path identity permits siblings');
const cycle = { first: 1 };
cycle.self = cycle;
let cycleError;
try { stringify(cycle); } catch (error) { cycleError = error; }
check(cycleError instanceof TypeError, 'cycle path');
let trace = '';
const value = { a: 1, b: 2, extra: 3 };
const wrapped = new String('b');
wrapped[Symbol.toPrimitive] = function (hint) { trace += 'wrapper:' + hint + ','; return 'b'; };
const list = [wrapped, 'a', 'b'];
Object.defineProperty(list, 0, { get() { trace += 'list:0,'; list.push('extra'); return wrapped; } });
Object.defineProperty(value, 'a', { get() { trace += 'a,'; return 1; }, enumerable: true });
Object.defineProperty(value, 'b', { get() { trace += 'b,'; return 2; }, enumerable: true });
const prepared = stringify(value, list, new Number(2));
check(prepared === '{\n  "b": 2,\n  "a": 1\n}', 'List order dedup and gap');
check(trace === 'list:0,wrapper:string,b,a,', 'List length snapshot and live Get');
const gap = '123456789' + high + low;
check(stringify({ v: 1 }, null, gap) === '{\n123456789' + high + '"v": 1\n}', 'gap truncates UTF16 units');
trace = '';
const native = new Number(7);
native[Symbol.toPrimitive] = function (hint) { trace += 'unbox:' + hint + ','; return 7; };
const target = function () {};
Object.defineProperty(target, 'toJSON', { get() {
  check(this === target, 'toJSON raw receiver'); trace += 'get,';
  return new Proxy(function (key) { check(this === target && key === 'x', 'toJSON call operands'); trace += 'toJSON,'; return native; }, {});
} });
const root = { x: target };
const replacer = new Proxy(function (key, value) {
  if (key === 'x') check(this === root && value === native, 'replacer holder and whole value');
  trace += 'replace:' + key + ',';
  return value;
}, { apply(target, receiver, args) { return Reflect.apply(target, receiver, args); } });
check(stringify(root, replacer) === '{"x":7}', 'toJSON replacer unbox result');
check(trace === 'replace:,get,toJSON,replace:x,unbox:number,', 'callback order');
const raw = JSON.rawJSON('9007199254740993');
check(stringify({ raw }) === '{"raw":9007199254740993}', 'raw preserves source');
check(stringify(new Proxy(raw, {})) === '{"rawJSON":"9007199254740993"}', 'Proxy lacks raw slot');
const sentinel = function originalThrow() {};
trace = '';
const throwing = { get first() { trace += 'first,'; throw sentinel; }, get later() { trace += 'later,'; return 2; } };
try { stringify(throwing); } catch (error) { check(error === sentinel, 'Get Throw identity'); }
finally { trace += 'finally'; }
check(trace === 'first,finally', 'Get abrupt cutoff');
trace = '';
try { stringify({ a: 1, b: 2 }, function (key, value) { trace += key + ','; if (key === 'a') throw sentinel; return value; }); }
catch (error) { check(error === sentinel, 'replacer Throw identity'); }
finally { trace += 'finally'; }
check(trace === ',a,finally', 'replacer abrupt cutoff');
const foreign = $262.createRealm();
const foreignStringify = foreign.evalScript('JSON.stringify');
const ForeignTypeError = foreign.evalScript('TypeError');
let error;
try { foreignStringify({ a: 1n }); } catch (thrown) { error = thrown; }
check(error instanceof ForeignTypeError && !(error instanceof TypeError), 'foreign BigInt Realm');
error = undefined;
try { foreignStringify(cycle); } catch (thrown) { error = thrown; }
check(error instanceof ForeignTypeError && !(error instanceof TypeError), 'foreign cycle Realm');
foreign.global.savedStringify = stringify;
foreign.global.savedTypeError = TypeError;
check(foreign.evalScript('try { savedStringify(1n); false; } catch (e) { e instanceof savedTypeError && !(e instanceof TypeError); }'), 'reverse called Realm');
let restored;
try { foreignStringify({ a: 1 }, function (key, value) { if (key === 'a') throw sentinel; return value; }); }
catch (thrown) { restored = thrown; }
check(restored === sentinel, 'foreign original callback Throw');
print('json-gc-stringify:ok');
262;
