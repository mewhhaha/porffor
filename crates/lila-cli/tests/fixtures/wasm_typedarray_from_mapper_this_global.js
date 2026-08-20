let seenThis;

let result = Uint8Array.from([1], function (value) {
  seenThis = this;
  return value;
});

if (result[0] !== 1) throw "result value";
if (seenThis !== globalThis) throw "default mapper this";

262;
