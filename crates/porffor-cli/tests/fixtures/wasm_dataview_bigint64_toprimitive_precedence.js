let buffer = new ArrayBuffer(12);
let view = new DataView(buffer);

view.setUint8(0, 0x27);
view.setUint8(1, 0x02);
view.setUint8(2, 0x06);
view.setUint8(3, 0x02);
view.setUint8(4, 0x80);
view.setUint8(5, 0x00);
view.setUint8(6, 0x80);
view.setUint8(7, 0x01);
view.setUint8(8, 0x7f);
view.setUint8(9, 0x00);
view.setUint8(10, 0x01);
view.setUint8(11, 0x02);

let offsetOne = 0x020602800080017fn;

__porfAssertThrows(TypeError, function () {
  view.getBigInt64({ [Symbol.toPrimitive]: 1 });
});

__porfAssertThrows(TypeError, function () {
  view.getBigUint64({
    [Symbol.toPrimitive]: function () {
      return {};
    },
  });
});

__porfAssertThrows(TypeError, function () {
  view.getBigInt64({ valueOf: null, toString: null });
});

(view.getBigInt64({
  [Symbol.toPrimitive]: function () {
    return 1;
  },
  valueOf: function () {
    throw new TypeError();
  },
  toString: function () {
    throw new TypeError();
  },
}) === offsetOne) +
  (view.getBigUint64({
    [Symbol.toPrimitive]: undefined,
    valueOf: function () {
      return 1;
    },
  }) === offsetOne) +
  (view.getBigInt64({
    valueOf: function () {
      return {};
    },
    toString: function () {
      return 1;
    },
  }) === offsetOne);
