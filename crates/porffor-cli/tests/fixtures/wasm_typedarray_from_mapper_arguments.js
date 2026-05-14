let records = [];

let result = Uint8Array.from([42, 43, 44], function (kValue, k) {
  records.push({
    kValue: kValue,
    k: k,
    argsLength: arguments.length
  });
  return kValue;
});

if (result.length !== 3) throw "result length";
if (result[0] !== 42 || result[1] !== 43 || result[2] !== 44) {
  throw "mapped values";
}

if (records.length !== 3) throw "record length";
if (records[0].kValue !== 42 || records[0].k !== 0 || records[0].argsLength !== 2) {
  throw "call 0";
}
if (records[1].kValue !== 43 || records[1].k !== 1 || records[1].argsLength !== 2) {
  throw "call 1";
}
if (records[2].kValue !== 44 || records[2].k !== 2 || records[2].argsLength !== 2) {
  throw "call 2";
}

262;
