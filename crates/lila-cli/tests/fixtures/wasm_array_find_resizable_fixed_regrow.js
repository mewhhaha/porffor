let rab = new ArrayBuffer(4, { maxByteLength: 4 });
let fixed = new Uint8Array(rab, 0, 4);
fixed[0] = 7;
fixed[1] = 8;
fixed[2] = 9;
fixed[3] = 10;

let seen = [];
let found = Array.prototype.find.call(fixed, function (value, index) {
  seen.push(value);
  if (index === 0) rab.resize(1);
  if (index === 1) rab.resize(4);
  return index === 2 && value === 0;
});

if (seen.length !== 3) throw "find callback count";
if (seen[0] !== 7) throw "find initial value";
if (seen[1] !== undefined) throw "find fixed out-of-bounds value";
if (seen[2] !== 0) throw "find fixed regrown value";
if (found !== 0) throw "find result after regrow";

true;
