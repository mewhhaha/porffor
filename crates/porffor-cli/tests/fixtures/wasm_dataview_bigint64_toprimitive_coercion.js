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

let offsetZero = 0x2702060280008001n;
let offsetOne = 0x020602800080017fn;

__porfAssertThrows(TypeError, function () {
  view.getBigInt64(Object(0n));
});

__porfAssertThrows(TypeError, function () {
  view.getBigUint64({
    [Symbol.toPrimitive]: function () {
      return 0n;
    },
  });
});

__porfAssertThrows(TypeError, function () {
  view.getBigInt64(Symbol("1"));
});

__porfAssertThrows(TypeError, function () {
  view.getBigUint64(Object(Symbol("1")));
});

__porfAssertThrows(TypeError, function () {
  view.getBigInt64({
    valueOf: function () {
      return Symbol("1");
    },
  });
});

(view.getBigInt64(Object(0)) === offsetZero) +
  (view.getBigUint64(Object(0)) === offsetZero) +
  (view.getBigInt64(Object(true)) === offsetOne) +
  (view.getBigUint64(Object(true)) === offsetOne) +
  (view.getBigInt64(Object("1")) === offsetOne) +
  (view.getBigUint64(Object("1")) === offsetOne) +
  (view.getBigInt64({
    [Symbol.toPrimitive]: function () {
      return 0;
    },
  }) === offsetZero) +
  (view.getBigUint64({
    [Symbol.toPrimitive]: function () {
      return "1";
    },
  }) === offsetOne);
