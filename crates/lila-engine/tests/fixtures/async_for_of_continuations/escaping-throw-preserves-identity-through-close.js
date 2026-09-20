async function run() {
  var events = [], nexts = 0, closes = 0, marker = {}, closeMarker = {};
  var iterable = { [Symbol.iterator]() { return {
    next() { nexts++; return { value: nexts, done: false }; },
    return() { closes++; events.push("close"); throw closeMarker; }
  }; } };
  var caught = false;
  try {
    for (const value of iterable) {
      try { await Promise.reject(marker); }
      catch (error) { events.push("catch"); await 0; throw error; }
      finally { events.push("finally"); await 0; events.push("finalized"); }
    }
  } catch (error) { if (error !== marker) throw "close replaced original throw"; caught = true; }
  if (!caught || nexts !== 1 || closes !== 1) throw "incorrect abrupt iteration";
  if (events.join(",") !== "catch,finally,finalized,close") throw events.join(",");
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
