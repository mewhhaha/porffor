function checkReduce(TA) {
  let sample = new TA([42n, 43n, 44n]);
  let calls = 0;
  let result = sample.reduce(function (accumulator, value, index, receiver) {
    if (receiver !== sample) throw "receiver";
    if (index !== calls + 1) throw "index";
    if (calls === 0 && (accumulator !== 42n || value !== 43n)) throw "first values";
    if (calls === 1 && (accumulator !== 41n || value !== 44n)) throw "second values";
    calls = calls + 1;
    return accumulator - 1n;
  });
  return calls === 2 && result === 40n;
}

if (!checkReduce(BigInt64Array)) throw "BigInt64Array";
if (!checkReduce(BigUint64Array)) throw "BigUint64Array";
131;
