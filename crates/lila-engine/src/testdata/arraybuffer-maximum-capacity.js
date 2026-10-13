function require(condition, message) {
  if (!condition) throw message;
}
function caught(operation) {
  try { operation(); } catch (error) { return error; }
  return null;
}
function rangeError(operation) {
  require(caught(operation) instanceof RangeError, 'missing maximum-capacity RangeError');
}

const huge = 7 * 1125899906842624;
for (const maximum of [2147483641, 2147483648, huge, Number.MAX_SAFE_INTEGER]) {
  for (const length of [0, 8]) {
    rangeError(function () { new ArrayBuffer(length, { maxByteLength: maximum }); });
  }
}

const order = [];
const target = new Proxy(function Target() {}, {
  get(object, key, receiver) {
    if (key === 'prototype') { order.push('prototype'); return Object.prototype; }
    return Reflect.get(object, key, receiver);
  }
});
const length = { valueOf() { order.push('length'); return 0; } };
const options = {
  get maxByteLength() {
    order.push('maximum');
    return { valueOf() { order.push('index'); return huge; } };
  }
};
rangeError(function () { Reflect.construct(ArrayBuffer, [length, options], target); });
require(order.join(',') === 'length,maximum,index,prototype', 'maximum rejection reordered constructor effects');

order.length = 0;
rangeError(function () { Reflect.construct(ArrayBuffer, [8, { maxByteLength: 0 }], target); });
require(order.length === 0, 'length versus maximum check followed prototype selection');

const marker = {};
const abruptTarget = new Proxy(function Target() {}, {
  get(object, key, receiver) {
    if (key === 'prototype') throw marker;
    return Reflect.get(object, key, receiver);
  }
});
require(caught(function () {
  Reflect.construct(ArrayBuffer, [0, { maxByteLength: huge }], abruptTarget);
}) === marker, 'maximum rejection replaced prototype exception');
require(caught(function () {
  Reflect.construct(ArrayBuffer, [0, { maxByteLength: { valueOf() { throw marker; } } }], target);
}) === marker && order.length === 0, 'maximum conversion replaced exception or read prototype');

const maximum = 2147483640;
const buffer = new ArrayBuffer(8, { maxByteLength: maximum });
require(buffer.byteLength === 8 && buffer.maxByteLength === maximum && buffer.resizable,
        'representable maximum was rejected or narrowed');
new Uint8Array(buffer)[0] = 41;
const moved = buffer.transfer(16);
require(buffer.byteLength === 0 && moved.byteLength === 16 && moved.maxByteLength === maximum && moved.resizable,
        'transfer lost representable maximum or failed to detach');
require(new Uint8Array(moved)[0] === 41, 'transfer lost backing data');
moved.resize(24);
const fixed = moved.transferToFixedLength(8);
require(fixed.byteLength === 8 && fixed.maxByteLength === 8 && !fixed.resizable && new Uint8Array(fixed)[0] === 41,
        'fixed transfer retained resizable maximum or lost backing data');
true;
