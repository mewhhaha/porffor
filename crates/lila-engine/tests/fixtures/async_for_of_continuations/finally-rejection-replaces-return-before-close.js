var original = {}, replacement = {}, closeMarker = {}, closes = 0, events = [];
var iterable = { [Symbol.iterator]() { return {
  next() { return { value: 1, done: false }; },
  return() { closes++; events.push("close"); throw closeMarker; }
}; } };
async function consume() {
  for (const value of iterable) {
    try { await 0; return original; }
    finally { events.push("finally"); await Promise.reject(replacement); }
  }
}
async function run() {
  var caught = false;
  try { await consume(); }
  catch (error) { if (error !== replacement) throw "wrong finalizer completion"; caught = true; }
  if (!caught || closes !== 1 || events.join(",") !== "finally,close") throw "finalizer close order";
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
