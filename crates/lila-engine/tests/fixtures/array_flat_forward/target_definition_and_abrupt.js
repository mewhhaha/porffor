var marker = {};
var source = [7];
Object.defineProperty(source, '1', { get: function () { print('unexpected-later'); return 8; } });
function Throwing() {
  return new Proxy({}, { defineProperty: function (object, key, descriptor) { print('define:' + key + ':' + descriptor.value); throw marker; } });
}
source.constructor = { [Symbol.species]: Throwing };
try { source.flat(); } catch (error) { print(error === marker); }
function Refusing() { return new Proxy({}, { defineProperty: function () { print('refuse'); return false; } }); }
source.constructor = { [Symbol.species]: Refusing };
try { source.flat(); } catch (error) { print(error instanceof TypeError); }
function Frozen() { return Object.freeze([]); }
source.constructor = { [Symbol.species]: Frozen };
try { source.flat(); } catch (error) { print(error instanceof TypeError); }
function Fixed() {
  var result = [0];
  Object.defineProperty(result, '0', { configurable: false });
  return result;
}
source.constructor = { [Symbol.species]: Fixed };
try { source.flat(); } catch (error) { print(error instanceof TypeError); }
var prototype = { set 0(value) { print('unexpected-setter'); }, set length(value) { print('unexpected-length'); } };
function Ordinary() { return Object.create(prototype); }
var simple = [[7]];
simple.constructor = { [Symbol.species]: Ordinary };
var result = simple.flat();
var descriptor = Object.getOwnPropertyDescriptor(result, '0');
print(result[0] + ':' + descriptor.writable + ':' + descriptor.enumerable + ':' + descriptor.configurable + ':' + Object.prototype.hasOwnProperty.call(result, 'length'));
