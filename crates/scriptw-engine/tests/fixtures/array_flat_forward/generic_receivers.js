var result = Array.prototype.flat.call('ab');
print(result.length + ':' + result[0] + ':' + result[1]);
print(Array.prototype.flat.call(true).length);
globalThis.length = 1;
globalThis[0] = [11];
result = Array.prototype.flat.call(globalThis);
print(result.length + ':' + result[0]);
delete globalThis.length;
delete globalThis[0];
var typed = new Uint8Array([10, 12]);
Object.defineProperty(typed, 'length', { get: function () { print('typed-length'); return 1; } });
result = Array.prototype.flat.call(typed);
print(result.length + ':' + result[0]);
function argumentsReceiver() {
  var args = arguments;
  Object.defineProperty(args, 'length', { get: function () { print('arguments-length'); return 1; } });
  return Array.prototype.flat.call(args);
}
result = argumentsReceiver([13], [14]);
print(result.length + ':' + result[0]);
