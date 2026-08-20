let rab = new ArrayBuffer(4, { maxByteLength: 4 });
let tracking = new Uint16Array(rab);
tracking[0] = 11;
tracking[1] = 22;

let calls = 0;
let first = 0;
Array.prototype.forEach.call(tracking, function (value, index) {
  calls += 1;
  if (index === 0) {
    first = value;
    rab.resize(3);
  }
});

if (calls !== 1) throw "partial Uint16 element must be absent";
if (first !== 11) throw "complete Uint16 element must remain visible";

true;
