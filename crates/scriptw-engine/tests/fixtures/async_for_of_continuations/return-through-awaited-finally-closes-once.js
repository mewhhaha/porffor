var events = [], nexts = 0, closes = 0, marker = {};
var iterable = { [Symbol.iterator]() { return {
  next() { nexts++; return { value: nexts, done: false }; },
  return() { closes++; events.push("close"); return {}; }
}; } };
async function consume() {
  for (const value of iterable) {
    try { await 0; return marker; }
    finally { events.push("finally"); await 0; events.push("finalized"); }
  }
  throw "fell through return";
}
async function run() {
  var result = await consume();
  if (result !== marker || nexts !== 1 || closes !== 1) throw "return completion";
  if (events.join(",") !== "finally,finalized,close") throw events.join(",");
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
