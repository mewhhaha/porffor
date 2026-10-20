async function run() {
  var events = [], nexts = 0, closes = 0, marker = {};
  var iterable = { [Symbol.iterator]() { return {
    next() { nexts++; return nexts <= 2 ? { value: nexts, done: false } : { done: true }; },
    return() { closes++; return {}; }
  }; } };
  for (const value of iterable) {
    events.push("before" + value);
    try {
      await (value === 1 ? Promise.resolve(value) : Promise.reject(marker));
      events.push("try" + value);
    } catch (error) {
      if (error !== marker) throw "rejection identity";
      events.push("catch" + value);
      await 0;
      events.push("caught" + value);
    } finally {
      events.push("finally" + value);
      await 0;
      events.push("finalized" + value);
    }
    events.push("after" + value);
  }
  if (events.join(",") !== "before1,try1,finally1,finalized1,after1,before2,catch2,caught2,finally2,finalized2,after2") throw events.join(",");
  if (nexts !== 3 || closes !== 0) throw "handled rejection closed or repeated iterator";
}
run().then(function () { print("ok"); }, function (error) { print("failed: " + String(error)); });
