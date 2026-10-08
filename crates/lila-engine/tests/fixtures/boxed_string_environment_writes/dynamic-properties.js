function assign(target, key, value) { return target[key] = value; }
function assignStrict(target, key, value) {
  'use strict';
  return target[key] = value;
}
var box = new String('a\uD83D\uDE00bxyz'), keyCalls = 0;
var key = { toString: function () { keyCalls++; return 'length'; } };
if (assign(box, key, 99) !== 99 || keyCalls !== 1 || box.length !== 7) {
  throw 'dynamic immutable length';
}
if (assign(box, '1', 'changed') !== 'changed' || box[1] !== '\uD83D' ||
    assign(box, '6', 'changed') !== 'changed' || box[6] !== 'z') {
  throw 'immutable code-unit index';
}
var caught = 0;
try { assignStrict(box, 'length', 99); throw 'strict length returned'; }
catch (error) { if (!(error instanceof TypeError)) throw error; caught++; }
try { assignStrict(box, '6', 'changed'); throw 'strict index returned'; }
catch (error) { if (!(error instanceof TypeError)) throw error; caught++; }
var indexDescriptor = Object.getOwnPropertyDescriptor(box, '6');
if (caught !== 2 || indexDescriptor.value !== 'z' || indexDescriptor.writable ||
    !indexDescriptor.enumerable || indexDescriptor.configurable) {
  throw 'code-unit descriptor';
}
var symbol = Symbol('length');
assign(box, symbol, 11);
assign(box, '7', 12);
assign(box, '01', 13);
assign(box, '-0', 14);
assign(box, 'extra', 15);
if (box[symbol] !== 11 || box[7] !== 12 || box['01'] !== 13 ||
    box['-0'] !== 14 || box.extra !== 15) throw 'ordinary boxed properties';
Object.defineProperty(box, 'fixed', { value: 5, writable: false });
if (assign(box, 'fixed', 6) !== 6 || box.fixed !== 5) throw 'ordinary non-writable';
try { assignStrict(box, 'fixed', 6); throw 'strict fixed returned'; }
catch (error) { if (!(error instanceof TypeError)) throw error; }
var prototype = Object.create(String.prototype), setterCalls = 0, marker = {};
Object.defineProperty(prototype, 'inherited', {
  set: function (value) {
    if (this !== box || value !== 16) throw 'setter receiver';
    setterCalls++;
  }
});
Object.defineProperty(prototype, 'abrupt', { set: function () { throw marker; } });
Object.setPrototypeOf(box, prototype);
if (assign(box, 'inherited', 16) !== 16 || setterCalls !== 1 ||
    Object.hasOwn(box, 'inherited')) throw 'inherited setter';
try { assign(box, 'abrupt', 17); throw 'setter throw lost'; }
catch (error) { if (error !== marker) throw error; }
Object.preventExtensions(box);
assign(box, 'extra', 18);
if (box.extra !== 18) throw 'existing property on non-extensible wrapper';
if (assign(box, 'newProperty', 19) !== 19 || Object.hasOwn(box, 'newProperty')) {
  throw 'sloppy non-extensible addition';
}
try { assignStrict(box, 'newProperty', 19); throw 'strict addition returned'; }
catch (error) { if (!(error instanceof TypeError)) throw error; }
print('ok');
