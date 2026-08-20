let q = new Array(0);
Object.defineProperty(q, 0, {
  value: 17,
  writable: false,
  enumerable: false,
  configurable: true
});

let before = Object.getOwnPropertyDescriptor(q, 0);
q[0] = 23;
let afterWrite = Object.getOwnPropertyDescriptor(q, "0");

before.value === 17
  && before.writable === false
  && before.enumerable === false
  && before.configurable === true
  && afterWrite.value === 17
  && afterWrite.writable === false
  && afterWrite.enumerable === false
  && afterWrite.configurable === true;
