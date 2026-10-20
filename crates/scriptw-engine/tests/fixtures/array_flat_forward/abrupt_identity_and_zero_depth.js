var marker = {};
function attempt(label, source, depth) {
  try { Array.prototype.flat.call(source, depth); print('unexpected-success'); }
  catch (error) { print(label + ':' + (error === marker)); }
}
attempt('length', { get length() { print('length-get'); throw marker; } }, { valueOf: function () { print('unexpected-depth'); return 1; } });
attempt('depth', { get length() { print('depth-length'); return 0; } }, { [Symbol.toPrimitive]: function (hint) { print('depth-' + hint); throw marker; } });
var species = [];
Object.defineProperty(species, 'constructor', { get: function () { print('constructor'); throw marker; } });
attempt('species', species, 1);
attempt('has', new Proxy([1], { has: function (object, key) { print('has:' + key); throw marker; } }), 1);
attempt('get', new Proxy([1], { get: function (object, key, receiver) {
  if (key === '0') { print('get-zero'); throw marker; }
  return Reflect.get(object, key, receiver);
} }), 1);
var nested = new Proxy([1], { get: function (object, key, receiver) {
  if (key === 'length') { print('nested-length'); throw marker; }
  return Reflect.get(object, key, receiver);
} });
var source = [nested];
Object.defineProperty(source, '1', { get: function () { print('unexpected-later'); return 2; } });
attempt('nested', source, Infinity);
var pair = Proxy.revocable([1], {});
pair.revoke();
print([pair.proxy].flat(0)[0] === pair.proxy);
try { [pair.proxy].flat(1); } catch (error) { print(error instanceof TypeError); }
